//! Independent per-hand physical phase curves over the original routed tip.
//! Endpoint measurements and source times bind each curve; final dense queries
//! certify sampled constraints, not skin clearance or aesthetic naturalness.
use crate::{
    clearance, equipment_motion,
    hand_coordination::{self, HandCoordinationInput, OrientationSample, SwivelSample},
    model::Vec3,
    motion::{add, distance, sub},
    posture,
    rig::{rotate, HandSolution, Quat, RigProfile},
};
use serde::{Deserialize, Serialize};
#[path = "virtual_hand_math.rs"]
mod math;
#[path = "phase_optimizer.rs"]
mod optimizer;

pub(crate) const MAX_SEGMENTS: usize = hand_coordination::MAX_CONTROL_KNOTS * 6;
const CHECK_HZ: f64 = 480.;
const DERIVATIVE_STEP: f64 = 1e-5;
const NUMERIC_TOLERANCE: f64 = 1e-8;
const REFINEMENT_ROUNDS: usize = 4;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Key {
    forearm: Vec3,
    forearm_rate: Vec3,
    plane: f64,
    plane_rate: f64,
    rotation: Quat,
    rotation_rate: Quat,
    source: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Segment {
    limb: usize,
    start: f64,
    end: f64,
    from: Key,
    to: Key,
    parameters: optimizer::Parameters,
}
fn mul(v: Vec3, s: f64) -> Vec3 {
    v.map(|x| x * s)
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn unit(v: Vec3) -> Vec3 {
    mul(v, 1. / distance(v, [0.; 3]).max(1e-12))
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn inverse(q: Quat) -> Quat {
    [-q[0], -q[1], -q[2], q[3]]
}
fn rotation(v: Vec3) -> Quat {
    let angle = distance(v, [0.; 3]);
    if angle < 1e-12 {
        return [0., 0., 0., 1.];
    }
    let v = mul(v, (angle * 0.5).sin() / angle);
    [v[0], v[1], v[2], (angle * 0.5).cos()]
}
fn normalize(q: Quat) -> Option<Quat> {
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    (n.is_finite() && n > 1e-10).then(|| q.map(|x| x / n))
}
fn wrap(v: f64) -> f64 {
    (v + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
fn hermite<const N: usize>(
    a: [f64; N],
    b: [f64; N],
    da: [f64; N],
    db: [f64; N],
    u: f64,
    dt: f64,
) -> [f64; N] {
    std::array::from_fn(|i| {
        (2. * u * u * u - 3. * u * u + 1.) * a[i]
            + (u * u * u - 2. * u * u + u) * dt * da[i]
            + (-2. * u * u * u + 3. * u * u) * b[i]
            + (u * u * u - u * u) * dt * db[i]
    })
}
fn correction_weight(u: f64, from_source: bool, to_source: bool) -> f64 {
    let a = if from_source { 1. } else { 2. };
    let b = if to_source { 1. } else { 2. };
    let peak: f64 = a / (a + b);
    u.powf(a) * (1. - u).powf(b) / (peak.powf(a) * (1. - peak).powf(b))
}
fn original(
    input: &HandCoordinationInput,
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
) -> ([HandSolution; 2], [f64; 2]) {
    let c = hand_coordination::at_with_orientations(swivels, orientations, input.time);
    let hands = std::array::from_fn(|i| {
        let hand = input
            .profile
            .solve_hand_with_swivel_and_orientation(
                i,
                input.tips[i],
                input.pitches[i],
                c.angles[i],
                c.orientations[i],
            )
            .expect("calibrated hand");
        input.cue_orientation.map_or(hand, |cue| {
            cue.apply(&input.profile, i, input.tips[i], c.angles[i], hand)
        })
    });
    (hands, c.angles)
}
fn key_at(
    time: f64,
    limb: usize,
    source: bool,
    inside_direction: f64,
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> Option<Key> {
    let sample = |t| {
        let f = input(t);
        let h = original(&f, swivels, orientations).0[limb];
        Some((
            rotate(inverse(h.rotation), unit(sub(h.end, h.middle))),
            math::source_plane(&f.profile, limb, &h)?,
            h.rotation,
        ))
    };
    let (forearm, plane, rotation) = sample(time)?;
    // Match this side of the original path. The routed tip is immutable, so
    // at an existing tip-velocity corner an outside shape tangent would mix
    // incompatible sides and amplify the original whole-arm derivative jump.
    let step = DERIVATIVE_STEP * inside_direction;
    let (la, pa, mut qa) = sample(time + step)?;
    let (lb, pb, mut qb) = sample(time + 2. * step)?;
    for q in [&mut qa, &mut qb] {
        if q.iter().zip(rotation).map(|(a, b)| a * b).sum::<f64>() < 0. {
            *q = q.map(|x| -x);
        }
    }
    Some(Key {
        forearm,
        plane,
        rotation,
        source,
        forearm_rate: if source {
            [0.; 3]
        } else {
            std::array::from_fn(|i| (-3. * forearm[i] + 4. * la[i] - lb[i]) / (2. * step))
        },
        plane_rate: if source {
            0.
        } else {
            (4. * wrap(pa - plane) - wrap(pb - plane)) / (2. * step)
        },
        rotation_rate: std::array::from_fn(|i| {
            (-3. * rotation[i] + 4. * qa[i] - qb[i]) / (2. * step)
        }),
    })
}
impl Segment {
    fn at(
        &self,
        profile: &RigProfile,
        time: f64,
        _ordinary_swivel: f64,
        tip: Vec3,
    ) -> Option<HandSolution> {
        let u = ((time - self.start) / (self.end - self.start)).clamp(0., 1.);
        let dt = self.end - self.start;
        let weight = correction_weight(u, self.from.source, self.to.source);
        let skew = 2. * u - 1.;
        let p = self.parameters;
        let frame = profile.hand_frames.as_ref()?.get(self.limb)?;
        let neutral = rotate(inverse(profile.contact_rotations[self.limb]), frame.forearm);
        let palm = rotate(inverse(profile.contact_rotations[self.limb]), frame.palm);
        let basis = unit(cross(neutral, palm));
        let other = unit(cross(neutral, basis));
        let lower = unit(hermite(
            self.from.forearm,
            self.to.forearm,
            self.from.forearm_rate,
            self.to.forearm_rate,
            u,
            dt,
        ));
        let lower = rotate(
            rotation(mul(
                add(
                    mul(basis, p[4] + p[6] * skew),
                    mul(other, p[5] + p[7] * skew),
                ),
                weight,
            )),
            lower,
        );
        let target_plane = self.from.plane + wrap(self.to.plane - self.from.plane);
        let plane = hermite(
            [self.from.plane],
            [target_plane],
            [self.from.plane_rate],
            [self.to.plane_rate],
            u,
            dt,
        )[0];
        let plane = plane + (p[0] + p[1] * skew) * weight;
        let sign = if self
            .from
            .rotation
            .iter()
            .zip(self.to.rotation)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            < 0.
        {
            -1.
        } else {
            1.
        };
        let prior = normalize(hermite(
            self.from.rotation,
            self.to.rotation.map(|x| x * sign),
            self.from.rotation_rate,
            self.to.rotation_rate.map(|x| x * sign),
            u,
            dt,
        ))?;
        math::solve(
            profile,
            self.limb,
            tip,
            lower,
            plane,
            prior,
            (p[2] + p[3] * skew) * weight,
        )
    }
}
fn active_segment(segments: &[Segment], limb: usize, time: f64) -> Option<&Segment> {
    let index = segments
        .partition_point(|s| s.limb < limb || s.limb == limb && s.start <= time)
        .checked_sub(1)?;
    let s = &segments[index];
    (s.limb == limb && time <= s.end).then_some(s)
}
pub(crate) fn at(
    segments: &[Segment],
    profile: &RigProfile,
    limb: usize,
    time: f64,
    ordinary_swivel: f64,
    target: Vec3,
) -> Option<HandSolution> {
    active_segment(segments, limb, time)?.at(profile, time, ordinary_swivel, target)
}
/// Ordinary motion preserves the requested path; an unreachable legacy solve
/// must not silently replace it with its clamped physical tip. Cue connectors
/// deliberately couple wrist and rotation and own their resulting physical tip.
pub(crate) fn effective_target(routed: Vec3, has_cue: bool, hand: &HandSolution) -> Vec3 {
    if has_cue {
        hand.tip
    } else {
        routed
    }
}
struct Sample {
    input: HandCoordinationInput,
    hands: [HandSolution; 2],
    targets: [Vec3; 2],
    angles: [f64; 2],
}
fn sample(
    time: f64,
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> Sample {
    let input = input(time);
    let (hands, angles) = original(&input, swivels, orientations);
    let targets = std::array::from_fn(|i| {
        effective_target(input.tips[i], input.cue_orientation.is_some(), &hands[i])
    });
    Sample {
        input,
        hands,
        targets,
        angles,
    }
}
fn evaluated(s: &Sample, selected: &[Segment]) -> [HandSolution; 2] {
    std::array::from_fn(|i| {
        at(
            selected,
            &s.input.profile,
            i,
            s.input.time,
            s.angles[i],
            s.targets[i],
        )
        .unwrap_or(s.hands[i])
    })
}
fn constraints(s: &Sample, hands: &[HandSolution; 2], limb: usize) -> [f64; 5] {
    let p = &s.input.profile;
    let h = &hands[limb];
    let margin = if s.input.close_tool_approach {
        0.
    } else {
        posture::MARGIN
    };
    [
        s.input.equipment.as_ref().map_or(-1., |ctx| {
            10. * (posture::MARGIN
                - equipment_motion::tool_clearance(
                    p,
                    limb,
                    h,
                    ctx,
                    s.input.intended_pieces[limb].as_deref(),
                ))
        }),
        10. * (margin - clearance::tool_pair_clearance(p, &hands[0], &hands[1])),
        10. * clearance::core_penetration(p, limb, h),
        posture::anatomical_violation(p, limb, h.tip, h).sqrt(),
        p.hand_orientation_policy_deficit(limb, h.rotation),
    ]
}
fn candidate_hands(
    s: &Sample,
    segment: &Segment,
    selected: &[Segment],
) -> Option<[HandSolution; 2]> {
    let mut h = s.hands;
    for (limb, hand) in h.iter_mut().enumerate() {
        if let Some(active) = active_segment(selected, limb, s.input.time) {
            *hand = active.at(
                &s.input.profile,
                s.input.time,
                s.angles[limb],
                s.targets[limb],
            )?;
        }
    }
    h[segment.limb] = segment.at(
        &s.input.profile,
        s.input.time,
        s.angles[segment.limb],
        s.targets[segment.limb],
    )?;
    Some(h)
}
fn physical_valid(s: &Sample, hands: &[HandSolution; 2], limb: usize) -> bool {
    let c = constraints(s, hands, limb);
    c[0] <= NUMERIC_TOLERANCE
        && c[1] <= NUMERIC_TOLERANCE
        && c[2] <= NUMERIC_TOLERANCE
        && posture::anatomical(&s.input.profile, limb, hands[limb].tip, &hands[limb])
        && s.input
            .profile
            .hand_orientation_policy_valid(limb, hands[limb].rotation)
}
fn motion_cost(
    samples: &[Sample],
    active: &[usize],
    segment: &Segment,
    selected: &[Segment],
) -> f64 {
    let mut total = 0.;
    let mut previous = None;
    for &index in active {
        let s = &samples[index];
        let Some(h) = candidate_hands(s, segment, selected) else {
            return f64::INFINITY;
        };
        if let Some((prior, profile, time)) = previous {
            total += hand_coordination::physical_motion_cost(
                profile,
                &s.input.profile,
                segment.limb,
                &prior,
                &h[segment.limb],
                s.input.time - time,
            );
        }
        previous = Some((h[segment.limb], &s.input.profile, s.input.time));
    }
    total
}
fn original_motion_cost(samples: &[Sample], selected: &[Segment], limb: usize) -> f64 {
    let mut total = 0.;
    let mut previous = None;
    for s in samples {
        let hand = evaluated(s, selected)[limb];
        if let Some((prior, profile, time)) = previous {
            total += hand_coordination::physical_motion_cost(
                profile,
                &s.input.profile,
                limb,
                &prior,
                &hand,
                s.input.time - time,
            );
        }
        previous = Some((hand, &s.input.profile, s.input.time));
    }
    total
}
fn shared_sample_times(
    spans: &[Vec<[f64; 2]>; 2],
    swivels: &[SwivelSample],
    check_times: &[f64],
) -> Vec<f64> {
    let mut times: Vec<_> = spans
        .iter()
        .flatten()
        .flatten()
        .copied()
        .chain(swivels.iter().map(|s| s.time))
        .chain(check_times.iter().copied())
        .collect();
    times.sort_by(f64::total_cmp);
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let Some((&start, &end)) = times.first().zip(times.last()) else {
        return times;
    };
    let mut grid = Vec::new();
    for tick in (start * CHECK_HZ).ceil() as i64..=(end * CHECK_HZ).floor() as i64 {
        let time = tick as f64 / CHECK_HZ;
        let index = times.partition_point(|t| *t < time);
        if ![index.checked_sub(1), Some(index)]
            .into_iter()
            .flatten()
            .any(|i| times.get(i).is_some_and(|t| (*t - time).abs() < 1e-9))
        {
            grid.push(time);
        }
    }
    times.extend(grid);
    times.sort_by(f64::total_cmp);
    times
}
fn samples(
    start: f64,
    end: f64,
    shared_times: &[f64],
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> Vec<Sample> {
    // One time lattice is shared by both hands and worker restoration. Adding
    // the other hand cannot hide a pair collision between two offset grids.
    let begin = shared_times.partition_point(|t| *t <= start + 1e-9);
    let finish = shared_times.partition_point(|t| *t < end - 1e-9);
    std::iter::once(start)
        .chain(shared_times[begin..finish.max(begin)].iter().copied())
        .chain(std::iter::once(end))
        .map(|t| sample(t, swivels, orientations, input))
        .collect()
}
fn has_held_cue(samples: &[Sample]) -> bool {
    samples.iter().enumerate().any(|(i, s)| {
        i > 0
            && i + 1 < samples.len()
            && (s.input.intended_stick_contact
                || s.input.cue_orientation.is_some_and(|c| c.weight >= 1.))
    })
}
fn optimize_segment(segment: Segment, samples: &[Sample], selected: &[Segment]) -> Option<Segment> {
    if has_held_cue(samples) {
        return None;
    }
    // Coefficients cannot change either endpoint pose. Do not spend a numerical
    // solve on a phase whose immutable endpoint still needs a static witness.
    if [samples.first()?, samples.last()?].into_iter().any(|s| {
        !candidate_hands(s, &segment, selected).is_some_and(|h| physical_valid(s, &h, segment.limb))
    }) {
        return None;
    }
    let already_valid = samples
        .iter()
        .all(|s| physical_valid(s, &evaluated(s, selected), segment.limb));
    let all: Vec<_> = (0..samples.len()).collect();
    let original_cost =
        already_valid.then(|| original_motion_cost(samples, selected, segment.limb));
    let improves = |candidate: &Segment| {
        original_cost.is_none_or(|old| {
            motion_cost(samples, &all, candidate, selected) < old - 1e-12 * old.abs().max(1.)
        })
    };
    // A physically safe path may still swing between different static witnesses.
    // Only consider replacing it when the measured endpoint interpolation reduces
    // the existing actual-motion objective. Accepted replacements must improve
    // that same dense objective and preserve all existing physical gates.
    if already_valid && !improves(&segment) {
        return None;
    }
    // Prefer the measured endpoint interpolation when it already repairs the
    // path. No finite-difference motion refinement is needed for this simple
    // zero-correction baseline; the same dense physical gates still apply.
    if samples.iter().all(|s| {
        candidate_hands(s, &segment, selected).is_some_and(|h| physical_valid(s, &h, segment.limb))
    }) {
        return Some(segment);
    }
    // A certified restart is shared by both nested search spaces. The simpler
    // search keeps measured wrist flex and adjusts only the arm plane and rigid
    // tool lever. Full wrist-flex freedom is used only when that cannot pass.
    let worst = samples
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let cost = |s: &Sample| {
                candidate_hands(s, &segment, selected).map_or(f64::INFINITY, |h| {
                    constraints(s, &h, segment.limb)
                        .into_iter()
                        .map(|x| x.max(0.).powi(2))
                        .sum::<f64>()
                })
            };
            cost(a).total_cmp(&cost(b))
        })?
        .0;
    let mut restart = None;
    for preserve_forearm in [true, false] {
        if let Some(fitted) =
            fit_segment(segment.clone(), samples, selected, None, preserve_forearm)
        {
            return improves(&fitted).then_some(fitted);
        }
        let parameters =
            restart.get_or_insert_with(|| certified_seed(&segment, &samples[worst], selected));
        if let Some(parameters) = parameters {
            let mut seeded = segment.clone();
            seeded.parameters = *parameters;
            if let Some(fitted) =
                fit_segment(seeded, samples, selected, Some(worst), preserve_forearm)
            {
                return improves(&fitted).then_some(fitted);
            }
        }
    }
    None
}
fn fit_segment(
    mut segment: Segment,
    samples: &[Sample],
    selected: &[Segment],
    required: Option<usize>,
    preserve_forearm: bool,
) -> Option<Segment> {
    if preserve_forearm {
        segment.parameters[4..].fill(0.);
    }
    // Use sparse collocation for numerical work. Dense validation adds the
    // worst missing constraints; it is never replaced by the optimizer score.
    let mut active: Vec<usize> = (0..=8).map(|i| i * (samples.len() - 1) / 8).collect();
    active.extend(required);
    // Retain the feasible seed's actual witness and the known initial extrema.
    // A uniform sparse grid alone can erase its obstacle-side choice before a
    // later dense check has a chance to add the omitted constraint.
    let initial: Vec<_> = samples
        .iter()
        .map(|s| {
            candidate_hands(s, &segment, selected)
                .map_or([1.; 5], |h| constraints(s, &h, segment.limb))
        })
        .collect();
    for kind in 0..5 {
        if let Some((i, _)) = initial
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[kind].total_cmp(&b.1[kind]))
        {
            active.push(i);
        }
    }
    active.sort_unstable();
    active.dedup();
    for _ in 0..REFINEMENT_ROUNDS {
        let current = segment.clone();
        let residuals = |parameters: &optimizer::Parameters| {
            let mut c = current.clone();
            c.parameters = *parameters;
            active
                .iter()
                .flat_map(|&i| {
                    candidate_hands(&samples[i], &c, selected)
                        .map_or([1.; 5], |h| constraints(&samples[i], &h, c.limb))
                })
                .collect()
        };
        let cost = |parameters: &optimizer::Parameters| {
            let mut c = current.clone();
            c.parameters = *parameters;
            motion_cost(samples, &active, &c, selected)
        };
        segment.parameters = if preserve_forearm {
            optimizer::optimize_plane_twist(segment.parameters, residuals, cost)
        } else {
            optimizer::optimize(segment.parameters, residuals, cost)
        };
        let failures: Vec<_> = samples
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match candidate_hands(s, &segment, selected) {
                Some(h) if physical_valid(s, &h, segment.limb) => None,
                Some(h) => Some((i, constraints(s, &h, segment.limb))),
                None => Some((i, [1.; 5])),
            })
            .collect();
        if samples.iter().all(|s| {
            candidate_hands(s, &segment, selected)
                .is_some_and(|h| physical_valid(s, &h, segment.limb))
        }) {
            return Some(segment);
        }
        let count = active.len();
        for kind in 0..5 {
            if let Some((index, _)) = failures
                .iter()
                .max_by(|a, b| a.1[kind].total_cmp(&b.1[kind]))
            {
                active.push(*index);
            }
        }
        active.sort_unstable();
        active.dedup();
        if active.len() == count {
            break;
        }
    }
    None
}
fn multiply(a: Quat, b: Quat) -> Quat {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}
fn seed_from_hand(
    segment: &Segment,
    s: &Sample,
    hand: &HandSolution,
) -> Option<optimizer::Parameters> {
    let limb = segment.limb;
    let p = &s.input.profile;
    let u = (s.input.time - segment.start) / (segment.end - segment.start);
    let weight = correction_weight(u, segment.from.source, segment.to.source);
    if weight < 1e-6 {
        return None;
    }
    let mut seed = segment.clone();
    seed.parameters = [0.; 8];
    let current = seed.at(p, s.input.time, s.angles[limb], s.targets[limb])?;
    let skew = 2. * u - 1.;
    let divisor = weight * (1. + skew * skew);
    let plane = wrap(math::source_plane(p, limb, hand)? - math::source_plane(p, limb, &current)?);
    seed.parameters[0] = plane / divisor;
    seed.parameters[1] = plane * skew / divisor;
    let local = rotate(
        inverse(current.rotation),
        unit(sub(current.end, current.middle)),
    );
    let desired = rotate(inverse(hand.rotation), unit(sub(hand.end, hand.middle)));
    let frame = p.hand_frames.as_ref()?.get(limb)?;
    let neutral = rotate(inverse(p.contact_rotations[limb]), frame.forearm);
    let palm = rotate(inverse(p.contact_rotations[limb]), frame.palm);
    let basis = unit(cross(neutral, palm));
    let other = unit(cross(neutral, basis));
    // Exact axis constrained to the two represented wrist-flex directions.
    // A projected shortest-arc rotation would generally miss the target.
    let raw_axis = cross(sub(local, desired), neutral);
    if distance(raw_axis, [0.; 3]) > 1e-10 {
        let axis = unit(raw_axis);
        let along = dot(axis, local);
        let angle = dot(axis, cross(local, desired)).atan2(dot(local, desired) - along * along);
        let delta = mul(axis, angle / divisor);
        seed.parameters[4] = dot(delta, basis);
        seed.parameters[6] = seed.parameters[4] * skew;
        seed.parameters[5] = dot(delta, other);
        seed.parameters[7] = seed.parameters[5] * skew;
    } else if distance(local, desired) > 1e-8 {
        return None;
    }
    let current = seed.at(p, s.input.time, s.angles[limb], s.targets[limb])?;
    let relative = multiply(hand.rotation, inverse(current.rotation));
    let axis = unit(sub(hand.tip, hand.middle));
    let twist = wrap(2. * dot([relative[0], relative[1], relative[2]], axis).atan2(relative[3]));
    seed.parameters[2] = twist / divisor;
    seed.parameters[3] = twist * skew / divisor;
    seed.parameters
        .iter()
        .all(|v| v.is_finite() && v.abs() <= optimizer::MAX_COEFFICIENT)
        .then_some(seed.parameters)
}
fn certified_seed(
    segment: &Segment,
    s: &Sample,
    selected: &[Segment],
) -> Option<optimizer::Parameters> {
    let limb = segment.limb;
    let p = &s.input.profile;
    let old = s.hands[limb];
    let pair = evaluated(s, selected);
    let environment = equipment_motion::HandEnvironment {
        equipment: s.input.equipment.as_deref(),
        intended_piece: s.input.intended_pieces[limb].as_deref(),
    };
    let hand = equipment_motion::PITCH_OFFSETS
        .into_iter()
        .flat_map(|pitch| {
            posture::hand_candidates_in(
                p,
                limb,
                s.targets[limb],
                pitch,
                posture::HandSearch::Full,
                environment,
            )
        })
        .filter(|h| {
            let mut hands = pair;
            hands[limb] = *h;
            physical_valid(s, &hands, limb)
        })
        .min_by(|a, b| {
            let cost = |h: &HandSolution| {
                distance(h.middle, old.middle).powi(2) + distance(h.end, old.end).powi(2)
            };
            cost(a).total_cmp(&cost(b))
        })?;
    seed_from_hand(segment, s, &hand)
}
fn source_at(times: &[f64], time: f64) -> bool {
    times.iter().any(|t| (*t - time).abs() < 1e-9)
}
fn outward_tangent(at: f64, near: f64, far: f64) -> bool {
    // A compact ordinary-join correction has zero first derivative. Reject
    // only an active bound whose inherited first-order departure is outward;
    // second-order deviations remain repairable by the polynomial curve.
    let first_order = 4. * near - far - 3. * at;
    let curvature = (far - 2. * near + at).abs();
    at >= -NUMERIC_TOLERANCE && first_order > NUMERIC_TOLERANCE + 4. * curvature
}
fn feasible_endpoint(
    time: f64,
    limb: usize,
    inside_direction: f64,
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> bool {
    let s = sample(time, swivels, orientations, input);
    if !physical_valid(&s, &s.hands, limb) {
        return false;
    }
    let near = sample(
        time + inside_direction * DERIVATIVE_STEP,
        swivels,
        orientations,
        input,
    );
    let far = sample(
        time + inside_direction * 2. * DERIVATIVE_STEP,
        swivels,
        orientations,
        input,
    );
    let a = constraints(&s, &s.hands, limb);
    let b = constraints(&near, &near.hands, limb);
    let c = constraints(&far, &far.hands, limb);
    (0..5).all(|i| !outward_tangent(a[i], b[i], c[i]))
}
fn available_spans(
    spans: &[[f64; 2]],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> (Vec<[f64; 2]>, Vec<f64>) {
    // A source preparation can overlap a cue. Keep the analytic held pulse
    // intact and let its entry/exit connectors remain independent phases.
    let mut held_boundaries: Vec<f64> = spans
        .iter()
        .flatten()
        .copied()
        .filter(|&t| input(t).cue_orientation.is_some_and(|c| c.weight >= 1.))
        .collect();
    held_boundaries.sort_by(f64::total_cmp);
    held_boundaries.dedup();
    let mut available = Vec::new();
    for &[start, end] in spans {
        let mut cuts = vec![start];
        cuts.extend(
            held_boundaries
                .iter()
                .copied()
                .filter(|&t| t > start && t < end),
        );
        cuts.push(end);
        for pair in cuts.windows(2) {
            let mid = input((pair[0] + pair[1]) * 0.5);
            if !mid.cue_orientation.is_some_and(|c| c.weight >= 1.) {
                available.push([pair[0], pair[1]]);
            }
        }
    }
    available.sort_by(|a, b| a[0].total_cmp(&b[0]).then(b[1].total_cmp(&a[1])));
    available.dedup();
    (available, held_boundaries)
}
fn canonical(
    spans: &[Vec<[f64; 2]>; 2],
    source_times: &[Vec<f64>; 2],
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> [Vec<[f64; 2]>; 2] {
    std::array::from_fn(|limb| {
        let mut out: Vec<[f64; 2]> = Vec::new();
        let (available, held_boundaries) = available_spans(&spans[limb], input);
        for [mut start, mut end] in available {
            // Source and analytic cue boundaries own their exact times. All
            // other phase joins belong to existing ordinary controls: retaining
            // a fractional recovery end would make the overlapping idle control
            // window disappear, leaving an unrepairable uncovered tail.
            if !source_at(&source_times[limb], start) && !source_at(&held_boundaries, start) {
                let previous = source_times[limb]
                    .iter()
                    .copied()
                    .rev()
                    .find(|t| *t < start)
                    .unwrap_or(f64::NEG_INFINITY);
                let lower = held_boundaries
                    .iter()
                    .copied()
                    .filter(|t| *t < start)
                    .fold(previous, f64::max);
                let Some(k) = swivels.iter().rev().find(|k| {
                    k.time <= start
                        && k.time >= lower
                        && (source_at(&source_times[limb], k.time)
                            || source_at(&held_boundaries, k.time)
                            || feasible_endpoint(k.time, limb, 1., swivels, orientations, input))
                }) else {
                    continue;
                };
                start = k.time;
            }
            if !source_at(&source_times[limb], end) && !source_at(&held_boundaries, end) {
                let next = source_times[limb]
                    .iter()
                    .copied()
                    .find(|t| *t > end)
                    .unwrap_or(f64::INFINITY);
                let upper = held_boundaries
                    .iter()
                    .copied()
                    .filter(|t| *t > end)
                    .fold(next, f64::min);
                let Some(k) = swivels.iter().find(|k| {
                    k.time >= end
                        && k.time <= upper
                        && (source_at(&source_times[limb], k.time)
                            || source_at(&held_boundaries, k.time)
                            || feasible_endpoint(k.time, limb, -1., swivels, orientations, input))
                }) else {
                    continue;
                };
                end = k.time;
            }
            if end > start {
                out.push([start, end]);
            }
        }
        // Alignment can make recovery and preparation overlap. Preserve their
        // complete union, then restore every semantic boundary; never discard a
        // tail merely because a preceding interval already owns part of it.
        let mut protected = source_times[limb].clone();
        protected.extend(held_boundaries);
        out = merged_phase_coverage(out, &protected);
        for pair in swivels.windows(2) {
            let a = pair[0].time;
            let b = pair[1].time;
            if !out.iter().any(|s| a < s[1] && b > s[0])
                && !source_times[limb].iter().any(|t| *t > a && *t < b)
            {
                out.push([a, b]);
            }
        }
        out.sort_by(|a, b| a[0].total_cmp(&b[0]));
        out
    })
}
fn merged_phase_coverage(mut spans: Vec<[f64; 2]>, protected: &[f64]) -> Vec<[f64; 2]> {
    spans.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut merged: Vec<[f64; 2]> = Vec::new();
    for span in spans {
        if let Some(prior) = merged.last_mut().filter(|prior| span[0] < prior[1]) {
            prior[1] = prior[1].max(span[1]);
        } else {
            merged.push(span);
        }
    }
    let mut result = Vec::new();
    for [start, end] in merged {
        let mut cuts = vec![start, end];
        cuts.extend(protected.iter().copied().filter(|t| *t > start && *t < end));
        cuts.sort_by(f64::total_cmp);
        cuts.dedup();
        result.extend(cuts.windows(2).map(|w| [w[0], w[1]]));
    }
    result
}
fn coalesced_span(spans: &[[f64; 2]], index: usize, sources: &[f64]) -> Option<[f64; 2]> {
    let [mut start, mut end] = spans[index];
    if !source_at(sources, start) {
        if let Some(prior) = index.checked_sub(1).and_then(|i| spans.get(i)) {
            if (prior[1] - start).abs() < 1e-9 {
                start = prior[0];
            }
        }
    }
    if !source_at(sources, end) {
        if let Some(next) = spans.get(index + 1) {
            if (next[0] - end).abs() < 1e-9 {
                end = next[1];
            }
        }
    }
    ([start, end] != spans[index] && !sources.iter().any(|t| *t > start && *t < end))
        .then_some([start, end])
}
fn make_segment(
    limb: usize,
    start: f64,
    end: f64,
    sources: &[f64],
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    input: &impl Fn(f64) -> HandCoordinationInput,
) -> Option<Segment> {
    Some(Segment {
        limb,
        start,
        end,
        from: key_at(
            start,
            limb,
            source_at(sources, start),
            1.,
            swivels,
            orientations,
            input,
        )?,
        to: key_at(
            end,
            limb,
            source_at(sources, end),
            -1.,
            swivels,
            orientations,
            input,
        )?,
        parameters: [0.; 8],
    })
}
pub(crate) fn plan(
    inputs: &[HandCoordinationInput],
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    spans: &[Vec<[f64; 2]>; 2],
    source_times: &[Vec<f64>; 2],
    input: impl Fn(f64) -> HandCoordinationInput,
) -> Vec<Segment> {
    let mut checks: Vec<_> = inputs.iter().map(|sample| sample.time).collect();
    checks.extend(spans.iter().flatten().flatten().copied());
    let spans = canonical(spans, source_times, swivels, orientations, &input);
    let shared_times = shared_sample_times(&spans, swivels, &checks);
    let mut selected = Vec::new();
    let mut accepted: Vec<(usize, f64, f64)> = Vec::new();
    type Attempt = Option<(usize, [f64; 2])>;
    let mut attempted: [Vec<Attempt>; 2] = std::array::from_fn(|i| vec![None; spans[i].len()]);
    // A first hand may only become feasible after the counterpart is repaired.
    // Revisit a failure once, only when an overlapping counterpart changed.
    for _ in 0..2 {
        let before = accepted.len();
        for limb in 0..2 {
            for (index, &[start, end]) in spans[limb].iter().enumerate() {
                if attempted[limb][index].is_some_and(|(generation, range)| {
                    !accepted[generation..]
                        .iter()
                        .any(|&(other, a, b)| other != limb && a <= range[1] && b >= range[0])
                }) {
                    continue;
                }
                if selected
                    .iter()
                    .any(|s: &Segment| s.limb == limb && s.start <= start && s.end >= end)
                {
                    continue;
                }
                attempted[limb][index] = Some((accepted.len(), [start, end]));
                let phase_samples =
                    samples(start, end, &shared_times, swivels, orientations, &input);
                let original_valid = phase_samples
                    .iter()
                    .all(|s| physical_valid(s, &evaluated(s, &selected), limb));
                let Some(base) = make_segment(
                    limb,
                    start,
                    end,
                    &source_times[limb],
                    swivels,
                    orientations,
                    &input,
                ) else {
                    continue;
                };
                let mut fitted = optimize_segment(base, &phase_samples, &selected);
                // One adjacent ordinary-span coalescing gives a correction room
                // before/after a too-short inherited C1 boundary. Own source
                // contacts and held cues remain immutable; there is no recursion.
                if fitted.is_none() && !original_valid {
                    if let Some([a, b]) = coalesced_span(&spans[limb], index, &source_times[limb]) {
                        attempted[limb][index] = Some((accepted.len(), [a, b]));
                        let overlaps = |s: &Segment| s.limb == limb && s.start < b && s.end > a;
                        if !selected
                            .iter()
                            .any(|s| overlaps(s) && (s.start < a || s.end > b))
                        {
                            let outside: Vec<_> =
                                selected.iter().filter(|s| !overlaps(s)).cloned().collect();
                            if let Some(merged) = make_segment(
                                limb,
                                a,
                                b,
                                &source_times[limb],
                                swivels,
                                orientations,
                                &input,
                            ) {
                                let merged_samples =
                                    samples(a, b, &shared_times, swivels, orientations, &input);
                                fitted = optimize_segment(merged, &merged_samples, &outside);
                            }
                        }
                    }
                }
                if let Some(segment) = fitted {
                    accepted.push((limb, segment.start, segment.end));
                    selected.retain(|s| {
                        !(s.limb == limb && s.start < segment.end && s.end > segment.start)
                    });
                    selected.push(segment);
                    selected.sort_by(|a, b| a.limb.cmp(&b.limb).then(a.start.total_cmp(&b.start)));
                }
            }
        }
        if accepted.len() == before {
            break;
        }
    }
    selected
}
pub(crate) fn validate(
    segments: &[Segment],
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    spans: &[Vec<[f64; 2]>; 2],
    source_times: &[Vec<f64>; 2],
    check_times: &[f64],
    input: impl Fn(f64) -> HandCoordinationInput,
) -> Result<(), String> {
    let error = || "준비된 연주의 물리 팔 경로가 유효하지 않습니다.".to_owned();
    if segments.len() > swivels.len().saturating_mul(6) {
        return Err(error());
    }
    // Cheap shape and ordering checks precede canonical endpoint queries and
    // any dense sampling. No payload supplies a free frequency or sample count.
    for (i, s) in segments.iter().enumerate() {
        if s.limb >= 2
            || !s.start.is_finite()
            || !s.end.is_finite()
            || s.start >= s.end
            || s.parameters
                .iter()
                .any(|v| !v.is_finite() || v.abs() > optimizer::MAX_COEFFICIENT)
            || i > 0
                && (segments[i - 1].limb > s.limb
                    || segments[i - 1].limb == s.limb && segments[i - 1].end > s.start)
        {
            return Err(error());
        }
    }
    let canonical = canonical(spans, source_times, swivels, orientations, &input);
    if segments.len() > canonical.iter().map(Vec::len).sum::<usize>() {
        return Err(error());
    }
    for s in segments {
        if !canonical[s.limb].contains(&[s.start, s.end])
            && !(0..canonical[s.limb].len()).any(|i| {
                coalesced_span(&canonical[s.limb], i, &source_times[s.limb])
                    == Some([s.start, s.end])
            })
        {
            return Err(error());
        }
        let Some(from) = key_at(
            s.start,
            s.limb,
            source_at(&source_times[s.limb], s.start),
            1.,
            swivels,
            orientations,
            &input,
        ) else {
            return Err(error());
        };
        let Some(to) = key_at(
            s.end,
            s.limb,
            source_at(&source_times[s.limb], s.end),
            -1.,
            swivels,
            orientations,
            &input,
        ) else {
            return Err(error());
        };
        if s.from != from || s.to != to {
            return Err(error());
        }
    }
    let mut checks = check_times.to_vec();
    checks.extend(spans.iter().flatten().flatten().copied());
    let shared_times = shared_sample_times(&canonical, swivels, &checks);
    for s in segments {
        let samples = samples(s.start, s.end, &shared_times, swivels, orientations, &input);
        if has_held_cue(&samples)
            || !samples.iter().all(|sample| {
                let Some(hands) = candidate_hands(sample, s, segments) else {
                    return false;
                };
                physical_valid(sample, &hands, s.limb)
                    && distance(hands[s.limb].tip, sample.targets[s.limb]) < 1e-8
            })
        {
            return Err(error());
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "physical_hand_path_tests.rs"]
mod tests;
