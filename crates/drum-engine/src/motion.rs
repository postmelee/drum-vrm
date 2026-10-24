use crate::model::*;
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

#[derive(Clone, Copy)]
struct LimbState {
    time: f64,
    target: Vec3,
}

#[derive(Clone)]
struct Candidate {
    states: [LimbState; 4],
    choices: Vec<Option<usize>>,
    cost: f64,
}

fn assignment_cost(limb: usize, hit: &Hit, state: LimbState) -> Option<f64> {
    if !reachable(limb, hit.target) {
        return None;
    }
    let dt = hit.time - state.time;
    let dist = distance(state.target, hit.target);
    if dt < MIN_INTERVAL - 1e-9 || dist > MAX_TIP_SPEED * dt {
        return None;
    }
    let preference = if preferred_limb(hit.instrument) == limb {
        0.0
    } else {
        0.3
    };
    // Fast repetitions prefer alternating hands; long rests return to handedness.
    Some(preference + dist * 0.6 + 0.02 / dt.min(2.0))
}

/// Beam width 8 within a simultaneous group. Two future hand hits contribute a
/// bounded reach/travel heuristic; fixed MIDI timestamps are never modified.
pub fn assign(plan: &mut PerformancePlan) {
    let mut states = std::array::from_fn(|limb| LimbState {
        time: -PRE_ROLL,
        target: rest_tip(limb),
    });
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
        let beam_end = end.min(start + 32);
        for hit in &plan.hits[start..beam_end] {
            let allowed: &[usize] = match hit.instrument {
                Instrument::Kick => &[3],
                Instrument::PedalHat => &[2],
                _ => &[0, 1],
            };
            let mut expanded = Vec::new();
            for candidate in beam {
                for &limb in allowed {
                    if let Some(cost) = assignment_cost(limb, hit, candidate.states[limb]) {
                        let mut next = candidate.clone();
                        next.cost += cost;
                        next.states[limb] = LimbState {
                            time: hit.time,
                            target: hit.target,
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
        for candidate in &mut beam {
            for future in plan.hits[end..(end + 16).min(plan.hits.len())]
                .iter()
                .filter(|h| !matches!(h.instrument, Instrument::Kick | Instrument::PedalHat))
                .take(2)
            {
                let forecast = (0..2)
                    .filter_map(|limb| assignment_cost(limb, future, candidate.states[limb]))
                    .fold(2.0, f64::min);
                candidate.cost += forecast * 0.2;
            }
        }
        beam.sort_by(|a, b| {
            a.cost
                .total_cmp(&b.cost)
                .then_with(|| a.choices.cmp(&b.choices))
        });
        let best = &beam[0];
        states = best.states;
        for (offset, hit) in plan.hits[start..end].iter_mut().enumerate() {
            hit.limb = best.choices.get(offset).copied().flatten();
            if hit.limb.is_none() {
                plan.diagnostics.push(Diagnostic::for_hit("UNRESOLVED_HIT", format!("{:.3}초 MIDI 음표 {}: 동시 타격·이동 속도·도달 한계로 손발을 배정하지 못했습니다. 원래 시각의 소리와 타점 강조는 유지합니다.", hit.time, hit.note), hit));
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

fn arc(a: Vec3, b: Vec3, phase: f64, height: f64) -> Vec3 {
    let phase = phase.clamp(0.0, 1.0);
    let mut point = lerp(a, b, smooth(phase));
    point[1] += (PI * phase).sin() * height;
    point
}

pub struct Motion {
    pub plan: PerformancePlan,
    limbs: [Vec<usize>; 4],
}

impl Motion {
    pub fn new(plan: PerformancePlan) -> Result<Self, String> {
        if plan.schema_version != 1 || plan.calibration_id != "primitive-v1" {
            return Err("지원하지 않는 연주 계획 버전입니다.".into());
        }
        if !plan.duration.is_finite()
            || !(0.0..=MAX_DURATION).contains(&plan.duration)
            || plan.hits.len() > MAX_HITS
        {
            return Err("연주 계획이 입력 한계를 벗어납니다.".into());
        }
        if plan.pre_roll != PRE_ROLL || plan.tail != TAIL {
            return Err("지원하지 않는 준비/여운 시간입니다.".into());
        }
        let mut limbs: [Vec<usize>; 4] = std::array::from_fn(|_| Vec::new());
        let mut previous = -1.0;
        for (index, hit) in plan.hits.iter().enumerate() {
            if hit.id != index
                || !hit.time.is_finite()
                || hit.time < previous
                || hit.time < 0.0
                || hit.time > plan.duration
                || hit.target.iter().any(|x| !x.is_finite() || x.abs() > 10.0)
            {
                return Err("연주 계획의 타격 시각·ID·좌표가 유효하지 않습니다.".into());
            }
            if let Some(limb) = hit.limb {
                if limb >= 4 || !reachable(limb, hit.target) {
                    return Err("연주 계획의 손발 배정이 유효하지 않습니다.".into());
                }
                if limbs[limb]
                    .last()
                    .is_some_and(|last| plan.hits[*last].time >= hit.time)
                {
                    return Err("같은 손발에 동시 타격이 배정되어 있습니다.".into());
                }
                limbs[limb].push(index);
            }
            previous = hit.time;
        }
        Ok(Self { plan, limbs })
    }

    fn tip_at(&self, limb: usize, time: f64) -> Vec3 {
        let hits = &self.limbs[limb];
        let at = hits.partition_point(|id| self.plan.hits[*id].time <= time);
        let previous = at.checked_sub(1).map(|i| &self.plan.hits[hits[i]]);
        let next = hits.get(at).map(|id| &self.plan.hits[*id]);
        let rest = rest_tip(limb);
        let foot = limb >= 2;
        let max_height = if foot { 0.045 } else { 0.24 };
        match (previous, next) {
            (Some(prev), _) if (time - prev.time).abs() < 1e-9 => prev.target,
            (Some(prev), Some(next)) if next.time - prev.time <= 0.42 => {
                let phase = (time - prev.time) / (next.time - prev.time);
                let height = max_height
                    * (0.35 + 0.65 * next.velocity as f64 / 127.0)
                    * ((next.time - prev.time) / 0.15).min(1.0);
                arc(prev.target, next.target, phase, height)
            }
            (_, Some(next)) if next.time - time <= 0.22 => {
                let phase = 1.0 - (next.time - time) / 0.22;
                arc(
                    rest,
                    next.target,
                    phase,
                    max_height * (0.35 + 0.65 * next.velocity as f64 / 127.0),
                )
            }
            (Some(prev), _) if time - prev.time < 0.18 => {
                let phase = (time - prev.time) / 0.18;
                arc(prev.target, rest, phase, max_height * 0.22)
            }
            _ => rest,
        }
    }

    pub fn evaluate(&self, song_time: f64) -> Vec<f32> {
        let time = if song_time.is_finite() {
            song_time.clamp(-PRE_ROLL, MAX_DURATION + 10.0)
        } else {
            -PRE_ROLL
        };
        let mut pose = Vec::with_capacity(POSE_FLOATS);
        for limb in 0..4 {
            let tip = self.tip_at(limb, time);
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
        for point in [
            [0.0, 0.71, -0.15],
            [0.0, 1.24, -0.10],
            [0.0, 1.62 - nod, -0.10 + nod],
        ] {
            pose.extend(point.map(|x| x as f32));
        }
        pose
    }
}
