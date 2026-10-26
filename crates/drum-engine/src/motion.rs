use crate::interpretation::{normalize_legacy, validate_interpretation};
use crate::model::*;
use crate::rig::{pitch_rotation, RigProfile, RIG_POSE_FLOATS};
use std::f64::consts::PI;

#[path = "prepared.rs"]
mod prepared;
#[path = "sticking.rs"]
mod sticking;
pub(crate) const PLANNER_VERSION: &str = "phrase-beam-8-beat-horizon-24-equipment-v8";

// Rig calibration is independent of any Three.js object/bone names.
pub const ARM_ROOTS: [Vec3; 2] = [[-0.22, 1.35, -0.10], [0.22, 1.35, -0.10]];
pub const LEG_ROOTS: [Vec3; 2] = [[-0.14, 0.71, -0.15], [0.14, 0.71, -0.15]];
pub const STICK_VECTOR: Vec3 = [0.0, -0.09, 0.30];
const TOE_VECTOR: Vec3 = [0.0, 0.0, 0.20];
const BONE_LENGTH: f64 = 0.43;
const MIN_INTERVAL: f64 = 0.045;
const MAX_TIP_SPEED: f64 = 12.0;

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: Vec3, n: f64) -> Vec3 {
    [a[0] * n, a[1] * n, a[2] * n]
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn distance(a: Vec3, b: Vec3) -> f64 {
    dot(sub(a, b), sub(a, b)).sqrt()
}
fn normalized(a: Vec3) -> Vec3 {
    let length = dot(a, a).sqrt();
    if length < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        mul(a, 1.0 / length)
    }
}
fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    add(mul(a, 1.0 - t), mul(b, t))
}
fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn rest_tip(limb: usize) -> Vec3 {
    match limb {
        0 => [-0.28, 1.06, 0.40],
        1 => [0.28, 1.06, 0.40],
        2 => [-0.22, 0.105, 0.55],
        _ => [0.20, 0.105, 0.62],
    }
}

fn reachable(limb: usize, tip: Vec3) -> bool {
    let (root, offset) = if limb < 2 {
        (ARM_ROOTS[limb], STICK_VECTOR)
    } else {
        (LEG_ROOTS[limb - 2], TOE_VECTOR)
    };
    distance(root, sub(tip, offset)) <= BONE_LENGTH * 2.0 - 0.0001
}

fn preferred_limb(instrument: Instrument) -> usize {
    use Instrument::*;
    match instrument {
        Kick => 3,
        PedalHat => 2,
        Snare | TomHigh | Crash => 0,
        _ => 1,
    }
}

/// Feet live on the kit's pedal contacts, including songs with no pedal notes.
/// A profile-only fallback supports authored plans without a complete kit.
fn pedal_contact(plan: &PerformancePlan, profile: &RigProfile, limb: usize) -> Vec3 {
    let instrument = if limb == 2 {
        Instrument::PedalHat
    } else {
        Instrument::Kick
    };
    plan.kit
        .iter()
        .find(|piece| piece.instrument == instrument)
        .map_or(profile.rest_tips[limb], |piece| piece.position)
}

#[derive(Clone, Copy)]
struct LimbState {
    time: f64,
    target: Vec3,
}

#[derive(Clone)]
struct Candidate {
    states: [LimbState; 4],
    choices: Vec<Option<usize>>,
    history: sticking::History,
    unresolved: usize,
    role_changes: usize,
    cost: f64,
}

impl Candidate {
    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        // A reachable/available default groove hand must not lose its role just
        // because its cross-body route is longer. Keep all feasible events first;
        // exceptions are still available for chords, fills and explicit edits.
        self.unresolved
            .cmp(&other.unresolved)
            .then_with(|| self.role_changes.cmp(&other.role_changes))
            .then_with(|| self.cost.total_cmp(&other.cost))
            .then_with(|| self.choices.cmp(&other.choices))
    }
}

fn assignment_cost(
    limb: usize,
    hit: &Hit,
    state: LimbState,
    contact_reachable: bool,
    preferred: usize,
) -> Option<f64> {
    if !contact_reachable {
        return None;
    }
    let dt = hit.time - state.time;
    let dist = distance(state.target, hit.target);
    if dt < MIN_INTERVAL - 1e-9 || dist > MAX_TIP_SPEED * dt {
        return None;
    }
    let preference = if preferred == limb {
        0.0
    } else if matches!(
        hit.instrument,
        Instrument::Hihat | Instrument::OpenHat | Instrument::Ride | Instrument::Crash
    ) {
        0.65
    } else {
        0.3
    };
    // The hard speed ceiling only rules out impossible assignments. Below it,
    // a short, fast cross-kit transfer should still lose to an available hand
    // with time to prepare. This scale is an engineering preference, not a
    // measured physiological limit or a change to the source event time.
    let transfer_effort = if limb < 2 {
        (dist / dt / 3.0).powi(2)
    } else {
        0.0
    };
    Some(preference + dist * 0.6 + 0.02 / dt.min(2.0) + transfer_effort)
}

/// A two-cymbal chord has an outward ownership preference, independent of MIDI
/// event order or kit IDs. A groove's right hand preference for a single left
/// crash must not turn a simultaneous pair into crossed forearms. Feasibility
/// and explicit hand overrides still take precedence over this role preference.
fn dual_crash_hands(plan: &PerformancePlan, profile: Option<&RigProfile>) -> Vec<Option<usize>> {
    let shoulders = profile.map_or(ARM_ROOTS, |p| [p.roots[0], p.roots[1]]);
    let right_axis = normalized(sub(shoulders[1], shoulders[0]));
    let mut result = vec![None; plan.hits.len()];
    let mut start = 0;
    while start < plan.hits.len() {
        let mut end = start + 1;
        while end < plan.hits.len() && plan.hits[end].time_us == plan.hits[start].time_us {
            end += 1;
        }
        let crashes: Vec<_> = (start..end)
            .filter(|&index| plan.hits[index].instrument == Instrument::Crash)
            .collect();
        if let [a, b] = crashes.as_slice() {
            let difference = dot(sub(plan.hits[*b].target, plan.hits[*a].target), right_axis);
            if difference.abs() > 1e-4 {
                result[*a] = Some(usize::from(difference < 0.0));
                result[*b] = Some(usize::from(difference > 0.0));
            }
        }
        start = end;
    }
    result
}

/// Receding-horizon beam search spans up to 24 events / two local quarter notes,
/// bounded to 0.35–1.2 seconds. Sticking candidates include both lead hands.
/// Future hits consume limbs in each candidate, unlike independent forecasts.
/// Commit only the current simultaneous group; original timestamps never move.
pub fn assign(plan: &mut PerformancePlan) {
    assign_for_profile(plan, None);
}

fn assign_for_profile(plan: &mut PerformancePlan, profile: Option<&RigProfile>) {
    let mut states = std::array::from_fn(|limb| LimbState {
        time: -PRE_ROLL,
        target: profile.map_or_else(
            || rest_tip(limb),
            |p| {
                if limb >= 2 {
                    pedal_contact(plan, p, limb)
                } else {
                    p.rest_tips[limb]
                }
            },
        ),
    });
    let contexts = sticking::contexts(plan);
    let dual_crash_hands = dual_crash_hands(plan, profile);
    // Reach/clearance depends on the physical contact and body, not the current
    // beam history. Cache it before expanding candidates so wrist solving does
    // not multiply with every lookahead branch.
    let mut contacts = std::collections::BTreeMap::new();
    let contact_checks: Vec<([bool; 4], [f64; 2])> = plan
        .hits
        .iter()
        .map(|hit| {
            let mut clearance = [0.0; 2];
            let valid = std::array::from_fn(|limb| {
                let allowed = match hit.instrument {
                    Instrument::Kick => limb == 3,
                    Instrument::PedalHat => limb == 2,
                    _ => limb < 2,
                };
                if !allowed {
                    return false;
                }
                let (valid, cost) = *contacts
                    .entry((hit.target.map(f64::to_bits), limb))
                    .or_insert_with(|| {
                        let valid = profile.map_or_else(
                            || reachable(limb, hit.target),
                            |p| p.reachable(limb, hit.target),
                        );
                        let cost = if valid && limb < 2 {
                            profile
                                .map_or(0.0, |p| crate::clearance::hand_cost(p, limb, hit.target))
                        } else {
                            0.0
                        };
                        (valid, cost)
                    });
                if limb < 2 {
                    clearance[limb] = cost;
                }
                valid
            });
            (valid, clearance)
        })
        .collect();
    let mut history = sticking::History::default();
    plan.planner_version = PLANNER_VERSION.into();
    let mut start = 0;
    while start < plan.hits.len() {
        let mut end = start + 1;
        while end < plan.hits.len() && plan.hits[end].time_us == plan.hits[start].time_us {
            end += 1;
        }
        let mut beam = vec![Candidate {
            states,
            choices: Vec::new(),
            history: history.clone(),
            unresolved: 0,
            role_changes: 0,
            cost: 0.0,
        }];
        // Pathological chords remain audible/diagnosed, but cannot expand a beam
        // proportional to an arbitrarily large simultaneous group.
        let mut horizon = end;
        while horizon < plan.hits.len()
            && horizon < start + 24
            && plan.hits[horizon].time - plan.hits[start].time <= contexts[start].horizon
        {
            horizon += 1;
        }
        let beam_end = horizon.min(start + 32);
        for (offset, hit) in plan.hits[start..beam_end].iter().enumerate() {
            let context = contexts[start + offset];
            let (contact_reachable, clearance) = contact_checks[start + offset];
            let allowed: &[usize] = match hit.instrument {
                Instrument::Kick => &[3],
                Instrument::PedalHat => &[2],
                _ => &[0, 1],
            };
            let mut expanded = Vec::new();
            for candidate in beam {
                for &limb in allowed {
                    if plan
                        .hand_overrides
                        .get(&hit.source_event_id)
                        .is_some_and(|forced| *forced != limb)
                    {
                        continue;
                    }
                    if let Some(cost) = assignment_cost(
                        limb,
                        hit,
                        candidate.states[limb],
                        contact_reachable[limb],
                        context.preferred,
                    ) {
                        let mut next = candidate.clone();
                        let role = dual_crash_hands[start + offset].or(context.role);
                        if !plan.hand_overrides.contains_key(&hit.source_event_id)
                            && role.is_some_and(|owner| owner != limb)
                        {
                            next.role_changes += 1;
                        }
                        next.cost += cost
                            + candidate.history.cost(limb, hit, context)
                            + if limb < 2 { clearance[limb] } else { 0.0 };
                        next.states[limb] = LimbState {
                            time: hit.time,
                            target: hit.target,
                        };
                        next.history.record(limb, hit);
                        next.choices.push(Some(limb));
                        expanded.push(next);
                    }
                }
                let mut unresolved = candidate;
                unresolved.unresolved += 1;
                unresolved.cost += 100.0 + hit.velocity as f64 / 127.0;
                unresolved.choices.push(None);
                expanded.push(unresolved);
            }
            expanded.sort_by(Candidate::compare);
            expanded.truncate(8);
            beam = expanded;
        }
        beam.sort_by(Candidate::compare);
        let best = &beam[0];
        for (offset, hit) in plan.hits[start..end].iter_mut().enumerate() {
            hit.limb = best.choices.get(offset).copied().flatten();
            if let Some(limb) = hit.limb {
                states[limb] = LimbState {
                    time: hit.time,
                    target: hit.target,
                };
                history.record(limb, hit);
            }
            if hit.limb.is_none() {
                let allowed: &[usize] = match hit.instrument {
                    Instrument::Kick => &[3],
                    Instrument::PedalHat => &[2],
                    _ => &[0, 1],
                };
                let unreachable = profile
                    .is_some_and(|p| allowed.iter().all(|limb| !p.reachable(*limb, hit.target)));
                let (code, reason) = if plan.hand_overrides.contains_key(&hit.source_event_id) {
                    ("HAND_OVERRIDE_INFEASIBLE", "지정한 손으로 원래 시각에 타격할 수 없습니다. 자동으로 다른 손을 사용하지 않습니다")
                } else if unreachable {
                    (
                        "RIG_UNREACHABLE_HIT",
                        "현재 체형·좌석·그립에서 타점에 도달하지 못했습니다",
                    )
                } else {
                    (
                        "UNRESOLVED_HIT",
                        "동시 타격·이동 속도·도달 한계로 손발을 배정하지 못했습니다",
                    )
                };
                plan.diagnostics.push(Diagnostic::for_hit(code, format!("{:.3}초 MIDI 음표 {}: {reason}. 원래 시각의 소리와 타점 강조는 유지합니다.", hit.time, hit.note), hit));
            }
        }
        start = end;
    }
}

/// Analytic two-bone IK in a stable plane, initialized from calibration each time.
/// Clamping protects intermediate arcs; every assigned contact is prevalidated.
fn two_bone(root: Vec3, target: Vec3, pole: Vec3) -> (Vec3, Vec3) {
    let delta = sub(target, root);
    let raw_dist = dot(delta, delta).sqrt();
    let direction = normalized(delta);
    let length = raw_dist.clamp(0.00001, BONE_LENGTH * 2.0 - 0.000001);
    let endpoint = add(root, mul(direction, length));
    let mut bend = sub(pole, mul(direction, dot(pole, direction)));
    if dot(bend, bend) < 1e-10 {
        bend = sub([0.0, 0.0, 1.0], mul(direction, direction[2]));
    }
    let along = length * 0.5;
    let height = (BONE_LENGTH * BONE_LENGTH - along * along).max(0.0).sqrt();
    (
        add(
            add(root, mul(direction, along)),
            mul(normalized(bend), height),
        ),
        endpoint,
    )
}

/// Unequal segment lengths use the law of cosines, including the inner reach
/// limit. No endpoint is overwritten with an unreachable desired contact.
pub(crate) fn two_bone_lengths(
    root: Vec3,
    target: Vec3,
    pole: Vec3,
    lengths: [f64; 2],
) -> (Vec3, Vec3) {
    let delta = sub(target, root);
    let direction = normalized(delta);
    let [a, b] = lengths;
    let length = distance(root, target).clamp((a - b).abs() + 1e-6, a + b - 1e-6);
    let endpoint = add(root, mul(direction, length));
    let mut bend = sub(pole, mul(direction, dot(pole, direction)));
    if dot(bend, bend) < 1e-10 {
        // Choose the least parallel world axis, including vertical/zero targets.
        let axis = (0..3)
            .min_by(|a, b| direction[*a].abs().total_cmp(&direction[*b].abs()))
            .unwrap();
        let mut fallback = [0.0; 3];
        fallback[axis] = 1.0;
        bend = sub(fallback, mul(direction, direction[axis]));
    }
    let along = (a * a - b * b + length * length) / (2.0 * length);
    let height = (a * a - along * along).max(0.0).sqrt();
    (
        add(
            add(root, mul(direction, along)),
            mul(normalized(bend), height),
        ),
        endpoint,
    )
}

fn validate_plan(plan: &PerformancePlan) -> Result<(), String> {
    validate_interpretation(plan)?;
    let point = |v: &Vec3| v.iter().all(|x| x.is_finite() && x.abs() <= 10.0);
    if !matches!(plan.schema_version, 1 | 2)
        || !plan.duration.is_finite()
        || !(0.0..=MAX_DURATION).contains(&plan.duration)
        || plan.hits.len() > MAX_HITS
        || plan.kit.len() > 128
        || plan.diagnostics.len() > MAX_EVENTS
        || plan.tempo_map.len() > MAX_EVENTS
        || plan.time_signatures.len() > MAX_EVENTS
    {
        return Err("연주 계획이 입력 한계를 벗어납니다.".into());
    }
    if plan.pre_roll != PRE_ROLL || plan.tail != TAIL {
        return Err("지원하지 않는 준비/여운 시간입니다.".into());
    }
    if plan.kit.iter().any(|k| {
        !point(&k.position)
            || !k.radius.is_finite()
            || !(0.0..=10.0).contains(&k.radius)
            || k.radius == 0.0
            || !crate::equipment::valid_piece_geometry(k)
    }) || plan
        .diagnostics
        .iter()
        .any(|d| d.time.is_some_and(|t| !t.is_finite()))
        || plan
            .tempo_map
            .iter()
            .any(|t| !t.time.is_finite() || t.time < 0.0 || t.microseconds_per_quarter == 0)
    {
        return Err("연주 계획의 키트·진단·템포 수치가 유효하지 않습니다.".into());
    }
    let ids: std::collections::BTreeSet<_> = plan.kit.iter().map(|p| &p.id).collect();
    if ids.len() != plan.kit.len() {
        return Err("중복된 드럼 부품 ID입니다.".into());
    }
    let mut previous = -1.0;
    for (index, hit) in plan.hits.iter().enumerate() {
        if hit.id != index
            || !hit.time.is_finite()
            || hit.time < previous
            || hit.time < 0.0
            || hit.time > plan.duration
            || !point(&hit.target)
            || hit.time_us > 300_000_000
            || (hit.time - hit.time_us as f64 / 1_000_000.0).abs() > 1e-9
            || hit.limb.is_some_and(|limb| {
                limb >= 4
                    || plan
                        .hand_overrides
                        .get(&hit.source_event_id)
                        .is_some_and(|forced| *forced != limb)
            })
            || hit.channel > 15
            || hit.velocity == 0
            || hit.velocity > 127
            || (plan.schema_version == 1 && map_note(hit.note) != Some(hit.instrument))
            || !plan.kit.iter().any(|piece| {
                piece.id == hit.piece_id
                    && piece.instrument == hit.instrument
                    && crate::contact::valid_target(piece, hit.target)
            })
            || hit.track >= 512
            || hit.event_index >= MAX_EVENTS
            || hit.tick > 9_007_199_254_740_991
        {
            return Err("연주 계획의 타격 시각·ID·좌표·MIDI 값이 유효하지 않습니다.".into());
        }
        previous = hit.time;
    }
    Ok(())
}

pub struct Motion {
    pub plan: PerformancePlan,
    limbs: [Vec<usize>; 4],
    profile: Option<RigProfile>,
    body_turns: Vec<f64>,
    hand_swivels: Vec<crate::hand_coordination::SwivelSample>,
    equipment: Option<std::sync::Arc<crate::equipment_motion::EquipmentContext>>,
    equipment_path: Vec<crate::equipment_motion::PathSample>,
    count_in: crate::count_in::CountInTrack,
}

impl Motion {
    pub fn new(mut plan: PerformancePlan) -> Result<Self, String> {
        normalize_legacy(&mut plan);
        if plan.calibration_id != "primitive-v1" {
            return Err("이 연주 계획에는 해당 리그 프로필이 필요합니다.".into());
        }
        Self::assemble(plan, None)
    }

    pub fn with_profile(mut plan: PerformancePlan, profile: RigProfile) -> Result<Self, String> {
        normalize_legacy(&mut plan);
        profile.validate()?;
        validate_plan(&plan)?;
        // Replace only planner-owned diagnostics. Parsing/mapping evidence survives
        // repeated calibration, while stale reach failures cannot accumulate.
        plan.diagnostics.retain(|d| {
            !matches!(
                d.code.as_str(),
                "UNRESOLVED_HIT"
                    | "RIG_UNREACHABLE_HIT"
                    | "RIG_UNREACHABLE_PEDAL"
                    | "HAND_OVERRIDE_INFEASIBLE"
                    | "BODY_PATH_CLEARANCE"
                    | "EQUIPMENT_PATH_CLEARANCE"
                    | "EQUIPMENT_LAYOUT_CLEARANCE"
                    | "COUNT_IN_GESTURE_UNAVAILABLE"
            )
        });
        for limb in 2..4 {
            let anchor = pedal_contact(&plan, &profile, limb);
            let instrument = if limb == 2 {
                Instrument::PedalHat
            } else {
                Instrument::Kick
            };
            if plan
                .hits
                .iter()
                .any(|hit| hit.instrument == instrument && distance(hit.target, anchor) > 1e-6)
            {
                return Err("페달 타격과 키트의 고정 접촉 위치가 일치하지 않습니다.".into());
            }
            if !profile.reachable(limb, anchor) {
                plan.diagnostics.push(Diagnostic::new(
                    "RIG_UNREACHABLE_PEDAL",
                    format!(
                        "{} 페달에 발이 닿지 않습니다. 좌석 또는 페달 위치를 조절하세요. 다리 길이는 늘리지 않습니다.",
                        if limb == 2 { "왼쪽" } else { "오른쪽" }
                    ),
                ));
            }
        }
        crate::contact::apply(&mut plan, &profile);
        assign_for_profile(&mut plan, Some(&profile));
        plan.calibration_id.clone_from(&profile.id);
        Self::assemble(plan, Some(profile))
    }

    fn assemble(plan: PerformancePlan, profile: Option<RigProfile>) -> Result<Self, String> {
        Self::assemble_with_tracks(plan, profile, None)
    }
    fn assemble_with_tracks(
        plan: PerformancePlan,
        profile: Option<RigProfile>,
        tracks: Option<(
            Vec<f64>,
            Vec<crate::hand_coordination::SwivelSample>,
            Vec<crate::equipment_motion::PathSample>,
        )>,
    ) -> Result<Self, String> {
        validate_plan(&plan)?;
        let mut limbs: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
        let mut states: [LimbState; 4] = std::array::from_fn(|limb| LimbState {
            time: -PRE_ROLL,
            target: profile.as_ref().map_or_else(
                || rest_tip(limb),
                |p| {
                    if limb >= 2 {
                        pedal_contact(&plan, p, limb)
                    } else {
                        p.rest_tips[limb]
                    }
                },
            ),
        });
        for (index, hit) in plan.hits.iter().enumerate() {
            if let Some(limb) = hit.limb {
                let allowed = match hit.instrument {
                    Instrument::Kick => limb == 3,
                    Instrument::PedalHat => limb == 2,
                    _ => limb < 2,
                };
                if !allowed
                    || assignment_cost(
                        limb,
                        hit,
                        states[limb],
                        profile.as_ref().map_or_else(
                            || reachable(limb, hit.target),
                            |p| p.reachable(limb, hit.target),
                        ),
                        preferred_limb(hit.instrument),
                    )
                    .is_none()
                {
                    return Err("연주 계획의 손발 배정이 유효하지 않습니다.".into());
                }
                if limbs[limb]
                    .last()
                    .is_some_and(|last| plan.hits[*last].time >= hit.time)
                {
                    return Err("같은 손발에 동시 타격이 배정되어 있습니다.".into());
                }
                states[limb] = LimbState {
                    time: hit.time,
                    target: hit.target,
                };
                limbs[limb].push(index);
            }
        }
        let cue_profile = profile.clone().unwrap_or_else(RigProfile::primitive);
        let count_in = crate::count_in::CountInTrack::prepare(&plan, &cue_profile);
        let equipment = crate::equipment_motion::EquipmentContext::for_kit(&plan.kit);
        let mut motion = Self {
            plan,
            limbs,
            profile,
            body_turns: Vec::new(),
            hand_swivels: Vec::new(),
            equipment,
            equipment_path: Vec::new(),
            count_in,
        };
        if let Some((body_turns, hand_swivels, equipment_path)) = tracks {
            motion.body_turns = body_turns;
            motion.hand_swivels = hand_swivels;
            motion.equipment_path = equipment_path;
            return Ok(motion);
        }
        motion.prepare_equipment_path()?;
        if let Some(profile) = &motion.profile {
            if profile.torso_pivot.is_some() {
                let count = ((motion.plan.duration + PRE_ROLL + TAIL) / crate::body_motion::STEP)
                    .ceil() as usize
                    + 1;
                let mut yaw = 0.0;
                for index in 0..count {
                    let time = index as f64 * crate::body_motion::STEP - PRE_ROLL;
                    let samples = [motion.stroke_at(0, time), motion.stroke_at(1, time)];
                    yaw = crate::body_motion::candidate(
                        profile,
                        samples.map(|s| s.0),
                        samples.map(|s| -s.1),
                        yaw,
                    );
                    motion.body_turns.push(yaw);
                }
                let contacts: Vec<_> = motion
                    .plan
                    .hits
                    .iter()
                    .filter_map(|hit| {
                        hit.limb
                            .filter(|&limb| limb < 2)
                            .map(|limb| (hit.time, limb, hit.target))
                    })
                    .collect();
                crate::body_motion::preserve_contacts(&mut motion.body_turns, profile, &contacts);
            }
        }
        if let Some(profile) = &motion.profile {
            if profile.hand_frames.is_some() {
                let make_input = |time| {
                    let samples = [motion.stroke_at(0, time), motion.stroke_at(1, time)];
                    crate::hand_coordination::HandCoordinationInput {
                        time,
                        profile: profile.turned(crate::body_motion::at(&motion.body_turns, time)),
                        tips: samples.map(|s| s.0),
                        pitches: samples.map(|s| -s.1),
                        equipment: motion.equipment.clone(),
                        intended_pieces: std::array::from_fn(|limb| {
                            motion.contact_piece_at(limb, time).map(str::to_owned)
                        }),
                        intended_stick_contact: motion
                            .count_in
                            .sample(time, samples.map(|s| s.0), samples.map(|s| -s.1))
                            .intended_stick_contact,
                    }
                };
                let control_times = prepared::control_times(&motion.plan)?;
                let inputs: Vec<_> = control_times.into_iter().map(make_input).collect();
                let check_count =
                    ((motion.plan.duration + PRE_ROLL + TAIL) * 30.0).ceil() as usize + 1;
                let mut times: Vec<_> = (0..check_count)
                    .map(|i| i as f64 / 30.0 - PRE_ROLL)
                    .collect();
                times.extend(motion.plan.hits.iter().map(|hit| hit.time));
                times.sort_by(f64::total_cmp);
                times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
                let checks: Vec<_> = times.into_iter().map(make_input).collect();
                motion.hand_swivels = crate::hand_coordination::plan_with_checks(&inputs, &checks)?;
            }
        }
        motion
            .plan
            .diagnostics
            .extend(motion.count_in.diagnostics.clone());
        motion.inspect_equipment_path();
        // Assignment/reach success does not certify collision-free motion.
        // Aggregate remaining actual impact-pose proxy failures without changing
        // source notes or flooding the plan with one warning per mesh segment.
        if let Some(profile) = &motion.profile {
            if profile.hand_frames.is_some() {
                let mut times: Vec<_> = motion
                    .plan
                    .hits
                    .iter()
                    .filter(|h| h.limb.is_some_and(|limb| limb < 2))
                    .map(|h| h.time)
                    .collect();
                times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
                let (mut failures, mut worst) = (0usize, 0.0f64);
                for time in times {
                    let pose = motion.evaluate_rig(time);
                    let p = profile.turned(crate::body_motion::at(&motion.body_turns, time));
                    let check = crate::clearance::inspect(&p, &pose);
                    let minimum = check
                        .torso_clearance_meters
                        .min(check.arm_clearance_meters)
                        .min(check.stick_clearance_meters)
                        .min(check.cross_clearance_meters);
                    if minimum < -1e-6 {
                        failures += 1;
                        worst = worst.max(-minimum);
                    }
                }
                if failures > 0 {
                    motion.plan.diagnostics.push(Diagnostic::new("BODY_PATH_CLEARANCE",format!("타격 시각 {failures}곳에서 신체·스틱 대리 형상 겹침이 남았습니다(최대 {:.1} mm). 배치 완료와 별도로 연주 경로를 확인하세요.",worst*1000.0)));
                }
            }
        }
        Ok(motion)
    }

    fn pose_equipment_clearance(&self, time: f64, pose: &[f32]) -> Option<f64> {
        let ctx = self.equipment.as_ref()?;
        let profile = self
            .profile
            .as_ref()?
            .turned(crate::body_motion::at(&self.body_turns, time));
        let mut minimum = f64::INFINITY;
        for limb in 0..2 {
            let offset = limb * 12;
            let rotation = 57 + limb * 4;
            let point = |at: usize| [pose[at] as f64, pose[at + 1] as f64, pose[at + 2] as f64];
            let hand = crate::rig::HandSolution {
                middle: point(offset + 3),
                end: point(offset + 6),
                tip: point(offset + 9),
                rotation: std::array::from_fn(|i| pose[rotation + i] as f64),
                reachable: true,
            };
            minimum = minimum.min(crate::equipment_motion::hand_clearance(
                &profile,
                limb,
                &hand,
                ctx,
                self.contact_piece_at(limb, time),
            ));
        }
        Some(minimum)
    }
    fn inspect_equipment_path(&mut self) {
        let Some(ctx) = &self.equipment else {
            return;
        };
        if !ctx.geometry.unresolved_piece_ids.is_empty() {
            self.plan.diagnostics.push(Diagnostic::new("EQUIPMENT_LAYOUT_CLEARANCE",format!("악기·받침대 배치 확인 필요: {}. 표시된 배치는 충돌 없음이 확인되지 않았습니다.",ctx.geometry.unresolved_piece_ids.join(", "))));
        }
        let mut times: Vec<_> = self.equipment_path.iter().map(|s| s.time).collect();
        times.extend(crate::count_in::control_times_for_plan(&self.plan));
        times.sort_by(f64::total_cmp);
        times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        let (mut failures, mut worst, mut first) = (0, 0.0f64, None);
        for time in times {
            let pose = self.evaluate_rig(time);
            if let Some(clear) = self.pose_equipment_clearance(time, &pose) {
                if clear < -0.00001 {
                    failures += 1;
                    worst = worst.max(-clear);
                    first.get_or_insert(time.max(0.));
                }
            }
        }
        if failures > 0 {
            let mut d=Diagnostic::new("EQUIPMENT_PATH_CLEARANCE",format!("준비·타격·회수·종료 경로 표본 {failures}곳에서 팔·스틱과 악기/받침대의 대리 형상 겹침이 남았습니다(최대 {:.1} mm). 원본 시각과 타점은 유지합니다.",worst*1000.));
            d.time = first;
            self.plan.diagnostics.push(d);
        }
    }
    fn prepare_equipment_path(&mut self) -> Result<(), String> {
        let (Some(ctx), Some(profile)) = (&self.equipment, &self.profile) else {
            return Ok(());
        };
        let times = prepared::equipment_times(&self.plan)?;
        let mut track = Vec::with_capacity(times.len());
        for time in times {
            let mut lift = [0.; 2];
            let mut pitch = [0.; 2];
            for limb in 0..2 {
                let (tip, p) = self.raw_stroke_at(limb, time);
                let impact = self.limbs[limb]
                    .binary_search_by(|id| self.plan.hits[*id].time.total_cmp(&time))
                    .is_ok();
                (lift[limb], pitch[limb]) = crate::equipment_motion::correction(
                    profile,
                    ctx,
                    limb,
                    tip,
                    -p,
                    impact,
                    self.contact_piece_at(limb, time),
                );
            }
            track.push(crate::equipment_motion::PathSample { time, lift, pitch });
        }
        crate::equipment_motion::limit_rates(
            &mut track,
            &std::array::from_fn(|limb| {
                self.limbs[limb]
                    .iter()
                    .map(|id| self.plan.hits[*id].time)
                    .collect()
            }),
        );
        self.equipment_path = track;
        Ok(())
    }
    fn stroke_at(&self, limb: usize, time: f64) -> (Vec3, f64) {
        if limb >= 2 {
            return self.raw_stroke_at(limb, time);
        }
        let mut samples = [self.raw_stroke_at(0, time), self.raw_stroke_at(1, time)];
        for (i, s) in samples.iter_mut().enumerate() {
            let (lift, pitch) = crate::equipment_motion::sample(&self.equipment_path, time, i);
            s.0[1] += lift;
            s.1 -= pitch;
        }
        let cue = self
            .count_in
            .sample(time, samples.map(|s| s.0), samples.map(|s| -s.1));
        (cue.tips[limb], -cue.pitches[limb])
    }
    fn contact_piece_at(&self, limb: usize, time: f64) -> Option<&str> {
        if self.count_in.is_active(time) {
            None
        } else {
            self.raw_stroke_with_piece(limb, time).2
        }
    }
    fn raw_stroke_at(&self, limb: usize, time: f64) -> (Vec3, f64) {
        let sample = self.raw_stroke_with_piece(limb, time);
        (sample.0, sample.1)
    }
    fn raw_stroke_with_piece(&self, limb: usize, time: f64) -> (Vec3, f64, Option<&str>) {
        let hits = &self.limbs[limb];
        let at = hits.partition_point(|id| self.plan.hits[*id].time <= time);
        let previous = at.checked_sub(1).map(|i| &self.plan.hits[hits[i]]);
        let next = hits.get(at).map(|id| &self.plan.hits[*id]);
        let mut rest_piece = None;
        let rest = self.profile.as_ref().map_or_else(
            || rest_tip(limb),
            |p| {
                let home = if limb == 0 { "snare" } else { "hihat" };
                let same_piece = previous.zip(next).filter(|(a, b)| a.piece_id == b.piece_id);
                let target = same_piece.map(|(hit, _)| hit.target).or_else(|| {
                    self.plan
                        .kit
                        .iter()
                        .find(|piece| piece.id == home)
                        .map(|piece| {
                            crate::contact::target(
                                piece,
                                if limb == 0 { "center" } else { "closed" },
                                p.chest,
                            )
                        })
                });
                if limb < 2 {
                    if let Some(target) = target {
                        let ready = add(target, [0.0, 0.035, 0.0]);
                        if p.reachable(limb, ready) {
                            rest_piece =
                                Some(same_piece.map_or(home, |(hit, _)| hit.piece_id.as_str()));
                            return ready;
                        }
                    }
                }
                p.rest_tips[limb]
            },
        );
        let transfer = |a: Vec3, b: Vec3, phase: f64, lift: f64| {
            let weight = smooth(phase.clamp(0.0, 1.0));
            let mut tip = lerp(a, b, weight);
            tip[1] += lift;
            if limb < 2 {
                if let Some(profile) = &self.profile {
                    // Straight chords across the kit can fold a long forearm/tool
                    // toward the chest. Preserve interpolated shoulder distance
                    // while moving around the front of the body instead. Apply
                    // this after the lift: an upward preparation must not fold
                    // the effective forearm/tool link back toward the shoulder.
                    let root = profile.roots[limb];
                    let radius = distance(root, a) * (1.0 - weight) + distance(root, b) * weight;
                    let direction = sub(tip, root);
                    let actual = distance(tip, root);
                    if actual > 1e-6 && radius > actual {
                        tip = add(root, mul(direction, radius / actual));
                    }
                }
            }
            tip
        };
        let foot = limb >= 2;
        let max_height = if foot { 0.045 } else { 0.11 };
        let stroke_pitch = |velocity: u8, interval: f64| {
            (0.12 + 0.30 * velocity as f64 / 127.0) * (interval / 0.20).min(1.0)
        };
        // Long moves to/from the ready position need time. If both windows
        // would overlap, connect the two source contacts directly instead of
        // abruptly abandoning an unfinished recovery for the next preparation.
        let recovery = previous.map_or(0.18, |hit| {
            if foot {
                0.18
            } else {
                (distance(hit.target, rest) / 2.5).clamp(0.18, 0.65)
            }
        });
        let preparation = next.map_or(0.22, |hit| {
            if foot {
                0.22
            } else {
                (distance(rest, hit.target) / 2.5).clamp(0.22, 0.65)
            }
        });
        match (previous, next) {
            (Some(prev), _) if (time - prev.time).abs() < 1e-9 => {
                (prev.target, 0.0, Some(prev.piece_id.as_str()))
            }
            (Some(prev), Some(next)) if next.time - prev.time <= recovery + preparation + 0.02 => {
                let phase = (time - prev.time) / (next.time - prev.time);
                let height = max_height
                    * (0.35 + 0.65 * next.velocity as f64 / 127.0)
                    * ((next.time - prev.time) / 0.15).min(1.0);
                let envelope = sticking::stroke_envelope(phase, prev.velocity, next.velocity);
                let tip = transfer(prev.target, next.target, phase, envelope * height);
                (
                    tip,
                    envelope * stroke_pitch(next.velocity, next.time - prev.time),
                    Some(if phase < 0.5 {
                        prev.piece_id.as_str()
                    } else {
                        next.piece_id.as_str()
                    }),
                )
            }
            (_, Some(next)) if next.time - time <= preparation => {
                let phase = 1.0 - (next.time - time) / preparation;
                (
                    transfer(
                        rest,
                        next.target,
                        phase,
                        (PI * phase).sin()
                            * max_height
                            * (0.35 + 0.65 * next.velocity as f64 / 127.0),
                    ),
                    (PI * phase).sin() * stroke_pitch(next.velocity, preparation),
                    if phase < 0.5 {
                        rest_piece
                    } else {
                        Some(next.piece_id.as_str())
                    },
                )
            }
            (Some(prev), _) if time - prev.time < recovery => {
                let phase = (time - prev.time) / recovery;
                (
                    transfer(
                        prev.target,
                        rest,
                        phase,
                        (PI * phase).sin() * max_height * 0.22,
                    ),
                    (PI * phase).sin() * stroke_pitch(prev.velocity, recovery) * 0.35,
                    if phase < 0.5 {
                        Some(prev.piece_id.as_str())
                    } else {
                        rest_piece
                    },
                )
            }
            _ => (rest, 0.0, rest_piece),
        }
    }

    /// Small toe-up preparation around a fixed ankle, with zero pitch at every
    /// immutable impact time and at rest. Squared sine keeps angular velocity
    /// continuous at the boundaries; rapid repetitions share one release arc.
    fn pedal_pitch_at(&self, limb: usize, time: f64) -> f64 {
        if limb == 2 {
            let control_at = self
                .plan
                .controls
                .partition_point(|control| control.time_us as f64 / 1_000_000.0 < time);
            if self
                .plan
                .controls
                .get(control_at)
                .is_some_and(|control| time >= control.release_start_time_us as f64 / 1_000_000.0)
            {
                // Zero at either endpoint is an authoritative closed state,
                // not permission for another foot preparation to take over.
                return -0.18 * crate::hihat::evaluate(&self.plan.controls, time);
            }
        }
        let hits = &self.limbs[limb];
        let at = hits.partition_point(|id| self.plan.hits[*id].time <= time);
        let previous = at.checked_sub(1).map(|i| &self.plan.hits[hits[i]]);
        let next = hits.get(at).map(|id| &self.plan.hits[*id]);
        if previous.is_some_and(|hit| (time - hit.time).abs() < 1e-9) {
            return 0.0;
        }
        let Some(next) = next else {
            return 0.0;
        };
        let interval = previous.map_or(f64::INFINITY, |hit| next.time - hit.time);
        let window = if interval <= 0.32 { interval } else { 0.16 };
        let mut start = next.time - window;
        if limb == 2 {
            // Look relative to the next literal strike, including controls that
            // have not started yet. Its preparation cannot cross their closure.
            let before_next = self
                .plan
                .controls
                .partition_point(|control| control.time_us <= next.time_us);
            if let Some(control) = before_next
                .checked_sub(1)
                .map(|index| &self.plan.controls[index])
            {
                start = start.max(control.time_us as f64 / 1_000_000.0);
            }
        }
        if time <= start {
            return 0.0;
        }
        let phase = (time - start) / (next.time - start);
        let amplitude = 0.18 * (0.35 + 0.65 * next.velocity as f64 / 127.0);
        -amplitude * (PI * phase).sin().powi(2)
    }

    pub fn inspect_pose(&self, song_time: f64) -> String {
        let song_time = if song_time.is_finite() {
            song_time.clamp(-PRE_ROLL, MAX_DURATION + 10.)
        } else {
            -PRE_ROLL
        };
        let pose = self.evaluate_rig(song_time);
        let yaw = crate::body_motion::at(
            &self.body_turns,
            if song_time.is_finite() {
                song_time
            } else {
                -PRE_ROLL
            },
        );
        let profile = self
            .profile
            .clone()
            .unwrap_or_else(RigProfile::primitive)
            .turned(yaw);
        let mut result = serde_json::to_value(crate::clearance::inspect(&profile, &pose))
            .expect("finite inspection");
        if let Some(clear) = self
            .pose_equipment_clearance(song_time, &pose)
            .filter(|v| v.is_finite())
        {
            result["equipmentClearanceMeters"] = serde_json::json!(clear);
        }
        let samples = [self.stroke_at(0, song_time), self.stroke_at(1, song_time)];
        let intended = self
            .count_in
            .sample(song_time, samples.map(|s| s.0), samples.map(|s| -s.1))
            .intended_stick_contact;
        result["intentionalStickContact"] = serde_json::json!(intended);
        if intended {
            let hands: [crate::rig::HandSolution; 2] = std::array::from_fn(|limb| {
                let at = limb * 12;
                let q = 57 + limb * 4;
                let p = |i: usize| [pose[i] as f64, pose[i + 1] as f64, pose[i + 2] as f64];
                crate::rig::HandSolution {
                    middle: p(at + 3),
                    end: p(at + 6),
                    tip: p(at + 9),
                    rotation: std::array::from_fn(|i| pose[q + i] as f64),
                    reachable: true,
                }
            });
            let pair =
                crate::count_in::contact_pair_clearance(&profile, &hands[0], &hands[1], true);
            result["stickClearanceMeters"] = serde_json::json!(pair.sticks);
        }
        if let Some(cue) = self
            .count_in
            .cues
            .iter()
            .find(|c| (c.time - song_time).abs() <= 0.002)
        {
            result["countIn"] = serde_json::json!({"sourceEventId":cue.source_event_id,"time":cue.time,"contactPoint":cue.contact_point});
        }
        serde_json::to_string(&result).expect("finite inspection")
    }

    pub fn evaluate(&self, song_time: f64) -> Vec<f32> {
        let mut pose = self.evaluate_rig(song_time);
        pose.truncate(POSE_FLOATS);
        pose
    }

    /// v3: positions, four tool quaternions, then the coordinated torso quaternion.
    pub fn evaluate_rig(&self, song_time: f64) -> Vec<f32> {
        let time = if song_time.is_finite() {
            song_time.clamp(-PRE_ROLL, MAX_DURATION + 10.0)
        } else {
            -PRE_ROLL
        };
        let yaw = crate::body_motion::at(&self.body_turns, time);
        let posed_profile = self.profile.as_ref().map(|p| p.turned(yaw));
        let swivels = crate::hand_coordination::at(&self.hand_swivels, time);
        let mut pose = Vec::with_capacity(RIG_POSE_FLOATS);
        let mut rotations = [[0.0, 0.0, 0.0, 1.0]; 4];
        for (limb, rotation) in rotations.iter_mut().enumerate() {
            let (tip, pitch) = self.stroke_at(limb, time);
            if let Some(profile) = &posed_profile {
                if limb < 2 {
                    if let Some(solution) =
                        profile.solve_hand_with_swivel(limb, tip, -pitch, swivels[limb])
                    {
                        *rotation = solution.rotation;
                        for point in [
                            profile.roots[limb],
                            solution.middle,
                            solution.end,
                            solution.tip,
                        ] {
                            pose.extend(point.map(|x| x as f32));
                        }
                        continue;
                    }
                }
                *rotation = pitch_rotation(
                    profile.contact_rotations[limb],
                    if limb < 2 {
                        -pitch
                    } else {
                        self.pedal_pitch_at(limb, time)
                    },
                );
                let offset = profile.offset(limb, *rotation);
                let root = profile.roots[limb];
                let desired_end = if limb < 2 {
                    sub(tip, offset)
                } else {
                    // Keep hip/knee/ankle fixed while the foot rotates. A rig's
                    // actual ankle-to-sole offset controls the toe contact arc.
                    sub(
                        pedal_contact(&self.plan, profile, limb),
                        profile.offset(limb, profile.contact_rotations[limb]),
                    )
                };
                let (middle, end) = two_bone_lengths(
                    root,
                    desired_end,
                    profile.bend_poles[limb],
                    profile.lengths[limb],
                );
                for point in [root, middle, end, add(end, offset)] {
                    pose.extend(point.map(|x| x as f32));
                }
                continue;
            }
            let (root, offset, pole) = if limb < 2 {
                (
                    ARM_ROOTS[limb],
                    STICK_VECTOR,
                    [if limb == 0 { -1.0 } else { 1.0 }, -0.5, -0.25],
                )
            } else {
                (LEG_ROOTS[limb - 2], TOE_VECTOR, [0.0, 0.0, 1.0])
            };
            let (middle, end) = two_bone(root, sub(tip, offset), pole);
            for point in [root, middle, end, add(end, offset)] {
                pose.extend(point.map(|x| x as f32));
            }
        }
        // Secondary head motion follows the same immutable timeline; body roots stay
        // fixed so visual expression cannot invalidate calibrated stick contacts.
        let last = self.plan.hits.partition_point(|h| h.time <= time);
        let nod = if last > 0 {
            (-(time - self.plan.hits[last - 1].time) * 12.0).exp() * 0.012
        } else {
            0.0
        };
        let (pelvis, chest, mut head) = posed_profile.as_ref().map_or(
            ([0.0, 0.71, -0.15], [0.0, 1.24, -0.10], [0.0, 1.62, -0.10]),
            |p| (p.pelvis, p.chest, p.head),
        );
        head[1] -= nod;
        head[2] += nod;
        for point in [pelvis, chest, head] {
            pose.extend(point.map(|x| x as f32));
        }
        for rotation in rotations {
            pose.extend(rotation.map(|x| x as f32));
        }
        pose.extend([0.0, (yaw * 0.5).sin() as f32, 0.0, (yaw * 0.5).cos() as f32]);
        pose
    }
}
