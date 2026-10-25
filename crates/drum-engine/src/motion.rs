use crate::interpretation::{normalize_legacy, validate_interpretation};
use crate::model::*;
use crate::rig::{pitch_rotation, RigProfile, RIG_POSE_FLOATS};
use std::f64::consts::PI;

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
    instrument: Option<Instrument>,
}

#[derive(Clone)]
struct Candidate {
    states: [LimbState; 4],
    choices: Vec<Option<usize>>,
    cost: f64,
}

fn assignment_cost(
    limb: usize,
    hit: &Hit,
    state: LimbState,
    profile: Option<&RigProfile>,
) -> Option<f64> {
    if !profile.map_or_else(
        || reachable(limb, hit.target),
        |p| p.reachable(limb, hit.target),
    ) {
        return None;
    }
    let dt = hit.time - state.time;
    let dist = distance(state.target, hit.target);
    if dt < MIN_INTERVAL - 1e-9 || dist > MAX_TIP_SPEED * dt {
        return None;
    }
    let preference = if (if hit.piece_id == "crashRight" {
        1
    } else {
        preferred_limb(hit.instrument)
    }) == limb
    {
        0.0
    } else {
        0.3
    };
    // Fast repetitions prefer alternating hands; long rests return to handedness.
    let repeated_snare = hit.instrument == Instrument::Snare
        && state.instrument == Some(Instrument::Snare)
        && dt < 0.18;
    Some(
        preference + dist * 0.6 + 0.02 / dt.min(2.0) + if repeated_snare { 0.08 / dt } else { 0.0 },
    )
}

/// Receding-horizon beam search spans up to 24 events / 0.35 seconds.
/// Future hits consume limbs in each candidate, unlike independent forecasts.
/// Commit only the current simultaneous group; original timestamps never move.
pub fn assign(plan: &mut PerformancePlan) {
    assign_for_profile(plan, None);
}

fn assign_for_profile(plan: &mut PerformancePlan, profile: Option<&RigProfile>) {
    let mut states = std::array::from_fn(|limb| LimbState {
        time: -PRE_ROLL,
        instrument: None,
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
    plan.planner_version = "phrase-beam-8-horizon-24-v1".into();
    let mut start = 0;
    while start < plan.hits.len() {
        let mut end = start + 1;
        while end < plan.hits.len() && plan.hits[end].time_us == plan.hits[start].time_us {
            end += 1;
        }
        let mut beam = vec![Candidate {
            states,
            choices: Vec::new(),
            cost: 0.0,
        }];
        // Pathological chords remain audible/diagnosed, but cannot expand a beam
        // proportional to an arbitrarily large simultaneous group.
        let mut horizon = end;
        while horizon < plan.hits.len()
            && horizon < start + 24
            && plan.hits[horizon].time - plan.hits[start].time <= 0.35
        {
            horizon += 1;
        }
        let beam_end = horizon.min(start + 32);
        for hit in &plan.hits[start..beam_end] {
            let allowed: &[usize] = match hit.instrument {
                Instrument::Kick => &[3],
                Instrument::PedalHat => &[2],
                _ => &[0, 1],
            };
            let mut expanded = Vec::new();
            for candidate in beam {
                for &limb in allowed {
                    if let Some(cost) = assignment_cost(limb, hit, candidate.states[limb], profile)
                    {
                        let mut next = candidate.clone();
                        next.cost += cost;
                        next.states[limb] = LimbState {
                            time: hit.time,
                            target: hit.target,
                            instrument: Some(hit.instrument),
                        };
                        next.choices.push(Some(limb));
                        expanded.push(next);
                    }
                }
                let mut unresolved = candidate;
                unresolved.cost += 100.0 + hit.velocity as f64 / 127.0;
                unresolved.choices.push(None);
                expanded.push(unresolved);
            }
            expanded.sort_by(|a, b| {
                a.cost
                    .total_cmp(&b.cost)
                    .then_with(|| a.choices.cmp(&b.choices))
            });
            expanded.truncate(8);
            beam = expanded;
        }
        beam.sort_by(|a, b| {
            a.cost
                .total_cmp(&b.cost)
                .then_with(|| a.choices.cmp(&b.choices))
        });
        let best = &beam[0];
        for (offset, hit) in plan.hits[start..end].iter_mut().enumerate() {
            hit.limb = best.choices.get(offset).copied().flatten();
            if let Some(limb) = hit.limb {
                states[limb] = LimbState {
                    time: hit.time,
                    target: hit.target,
                    instrument: Some(hit.instrument),
                };
            }
            if hit.limb.is_none() {
                let allowed: &[usize] = match hit.instrument {
                    Instrument::Kick => &[3],
                    Instrument::PedalHat => &[2],
                    _ => &[0, 1],
                };
                let unreachable = profile
                    .is_some_and(|p| allowed.iter().all(|limb| !p.reachable(*limb, hit.target)));
                let (code, reason) = if unreachable {
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
fn two_bone_lengths(root: Vec3, target: Vec3, pole: Vec3, lengths: [f64; 2]) -> (Vec3, Vec3) {
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
            || hit.limb.is_some_and(|limb| limb >= 4)
            || hit.channel > 15
            || hit.velocity == 0
            || hit.velocity > 127
            || (plan.schema_version == 1 && map_note(hit.note) != Some(hit.instrument))
            || !plan.kit.iter().any(|piece| {
                piece.id == hit.piece_id
                    && piece.instrument == hit.instrument
                    && distance(piece.position, hit.target) < 1e-9
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

fn arc(a: Vec3, b: Vec3, phase: f64, height: f64) -> Vec3 {
    let phase = phase.clamp(0.0, 1.0);
    let mut point = lerp(a, b, smooth(phase));
    point[1] += (PI * phase).sin() * height;
    point
}

pub struct Motion {
    pub plan: PerformancePlan,
    limbs: [Vec<usize>; 4],
    profile: Option<RigProfile>,
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
                "UNRESOLVED_HIT" | "RIG_UNREACHABLE_HIT" | "RIG_UNREACHABLE_PEDAL"
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
        assign_for_profile(&mut plan, Some(&profile));
        plan.calibration_id.clone_from(&profile.id);
        Self::assemble(plan, Some(profile))
    }

    fn assemble(plan: PerformancePlan, profile: Option<RigProfile>) -> Result<Self, String> {
        validate_plan(&plan)?;
        let mut limbs: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
        let mut states: [LimbState; 4] = std::array::from_fn(|limb| LimbState {
            time: -PRE_ROLL,
            instrument: None,
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
                if !allowed || assignment_cost(limb, hit, states[limb], profile.as_ref()).is_none()
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
                    instrument: Some(hit.instrument),
                };
                limbs[limb].push(index);
            }
        }
        Ok(Self {
            plan,
            limbs,
            profile,
        })
    }

    fn stroke_at(&self, limb: usize, time: f64) -> (Vec3, f64) {
        let hits = &self.limbs[limb];
        let at = hits.partition_point(|id| self.plan.hits[*id].time <= time);
        let previous = at.checked_sub(1).map(|i| &self.plan.hits[hits[i]]);
        let next = hits.get(at).map(|id| &self.plan.hits[*id]);
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
                        .map(|piece| piece.position)
                });
                if limb < 2 {
                    if let Some(target) = target {
                        let ready = add(target, [0.0, 0.035, 0.0]);
                        if p.reachable(limb, ready) {
                            return ready;
                        }
                    }
                }
                p.rest_tips[limb]
            },
        );
        let foot = limb >= 2;
        let max_height = if foot { 0.045 } else { 0.11 };
        let stroke_pitch = |velocity: u8, interval: f64| {
            (0.12 + 0.30 * velocity as f64 / 127.0) * (interval / 0.20).min(1.0)
        };
        match (previous, next) {
            (Some(prev), _) if (time - prev.time).abs() < 1e-9 => (prev.target, 0.0),
            (Some(prev), Some(next)) if next.time - prev.time <= 0.42 => {
                let phase = (time - prev.time) / (next.time - prev.time);
                let height = max_height
                    * (0.35 + 0.65 * next.velocity as f64 / 127.0)
                    * ((next.time - prev.time) / 0.15).min(1.0);
                (
                    arc(prev.target, next.target, phase, height),
                    (PI * phase).sin() * stroke_pitch(next.velocity, next.time - prev.time),
                )
            }
            (_, Some(next)) if next.time - time <= 0.22 => {
                let phase = 1.0 - (next.time - time) / 0.22;
                (
                    arc(
                        rest,
                        next.target,
                        phase,
                        max_height * (0.35 + 0.65 * next.velocity as f64 / 127.0),
                    ),
                    (PI * phase).sin() * stroke_pitch(next.velocity, 0.22),
                )
            }
            (Some(prev), _) if time - prev.time < 0.18 => {
                let phase = (time - prev.time) / 0.18;
                (
                    arc(prev.target, rest, phase, max_height * 0.22),
                    (PI * phase).sin() * stroke_pitch(prev.velocity, 0.18) * 0.35,
                )
            }
            _ => (rest, 0.0),
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

    pub fn evaluate(&self, song_time: f64) -> Vec<f32> {
        let mut pose = self.evaluate_rig(song_time);
        pose.truncate(POSE_FLOATS);
        pose
    }

    /// v2: the v1 position block followed by LH/RH/LF/RF world xyzw quaternions.
    pub fn evaluate_rig(&self, song_time: f64) -> Vec<f32> {
        let time = if song_time.is_finite() {
            song_time.clamp(-PRE_ROLL, MAX_DURATION + 10.0)
        } else {
            -PRE_ROLL
        };
        let mut pose = Vec::with_capacity(RIG_POSE_FLOATS);
        let mut rotations = [[0.0, 0.0, 0.0, 1.0]; 4];
        for (limb, rotation) in rotations.iter_mut().enumerate() {
            let (tip, pitch) = self.stroke_at(limb, time);
            if let Some(profile) = &self.profile {
                if limb < 2 {
                    if let Some(solution) = profile.solve_hand(limb, tip, -pitch) {
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
        let (pelvis, chest, mut head) = self.profile.as_ref().map_or(
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
        pose
    }
}
