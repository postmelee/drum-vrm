//! Target-oriented held-tool contact. Prefer the shaft approach, recover the wrist
//! from measured tool markers, and solve an ordinary two-bone arm. A bounded
//! refinement reduces wrist swing without changing the grip. Static bounded
//! palm witnesses are replayed through separately prepared tool orientations.
use super::{rotate, HandControlFrame, OrientationControl, Quat, RigProfile};
use crate::model::Vec3;
use crate::motion::{add, distance, sub};

#[derive(Clone, Copy, Debug)]
pub struct HandSolution {
    pub middle: Vec3,
    pub end: Vec3,
    pub tip: Vec3,
    pub rotation: Quat,
    pub reachable: bool,
}
fn mul(v: Vec3, scale: f64) -> Vec3 {
    v.map(|x| x * scale)
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(v: Vec3) -> Vec3 {
    let n = distance(v, [0.0; 3]);
    if n < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        mul(v, 1.0 / n)
    }
}
fn projected(normal: Vec3, axis: Vec3) -> Vec3 {
    let mut value = sub(normal, mul(axis, dot(normal, axis)));
    if distance(value, [0.0; 3]) < 1e-8 {
        let index = (0..3)
            .min_by(|a, b| axis[*a].abs().total_cmp(&axis[*b].abs()))
            .unwrap();
        let mut fallback = [0.0; 3];
        fallback[index] = 1.0;
        value = sub(fallback, mul(axis, axis[index]));
    }
    unit(value)
}
fn multiply(a: Quat, b: Quat) -> Quat {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}
fn inverse(q: Quat) -> Quat {
    [-q[0], -q[1], -q[2], q[3]]
}
fn normalize(q: Quat) -> Quat {
    let n = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    q.map(|v| v / n)
}
fn axis_rotation(axis: Vec3, angle: f64) -> Quat {
    let (s, c) = (angle * 0.5).sin_cos();
    [axis[0] * s, axis[1] * s, axis[2] * s, c]
}
#[cfg(test)]
fn frame(direction: Vec3, normal: Vec3) -> Quat {
    let x = unit(direction);
    let y = projected(normal, x);
    let z = cross(x, y);
    let (m11, m12, m13, m21, m22, m23, m31, m32, m33) =
        (x[0], y[0], z[0], x[1], y[1], z[1], x[2], y[2], z[2]);
    let trace = m11 + m22 + m33;
    let q = if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        [(m32 - m23) * s, (m13 - m31) * s, (m21 - m12) * s, 0.25 / s]
    } else if m11 > m22 && m11 > m33 {
        let s = 2.0 * (1.0 + m11 - m22 - m33).sqrt();
        [0.25 * s, (m12 + m21) / s, (m13 + m31) / s, (m32 - m23) / s]
    } else if m22 > m33 {
        let s = 2.0 * (1.0 + m22 - m11 - m33).sqrt();
        [(m12 + m21) / s, 0.25 * s, (m23 + m32) / s, (m13 - m31) / s]
    } else {
        let s = 2.0 * (1.0 + m33 - m11 - m22).sqrt();
        [(m13 + m31) / s, (m23 + m32) / s, 0.25 * s, (m21 - m12) / s]
    };
    normalize(q)
}
/// Deterministic shortest swing, with a calibrated antipodal fallback.
fn shortest_arc(from: Vec3, to: Vec3, reference: Vec3) -> Quat {
    let from = unit(from);
    let to = unit(to);
    let cosine = dot(from, to);
    if cosine < -1.0 + 1e-9 {
        return axis_rotation(projected(reference, from), std::f64::consts::PI);
    }
    let axis = cross(from, to);
    normalize([axis[0], axis[1], axis[2], 1.0 + cosine])
}
/// Initial shaft inclination is a common approach prior, not a surface-clearance
/// certificate. The bounded pitch family and full tool query own acceptance.
const APPROACH_INCLINE: f64 = 0.25;
const ROLL_REFINEMENTS: usize = 4;
// One bounded continuous yaw search; no yaw/roll graph dimension is added.
const YAW_REFINEMENTS: usize = 12;
const MAX_APPROACH_YAW: f64 = std::f64::consts::FRAC_PI_4;

/// Maximize forearm agreement over an arc, including both ends: clamping the
/// unrestricted atan2 result alone is wrong across the periodic branch cut.
fn bounded_roll(sine: f64, cosine: f64, minimum: f64, maximum: f64) -> f64 {
    let interior = sine.atan2(cosine).clamp(minimum, maximum);
    let score = |angle: f64| cosine * angle.cos() + sine * angle.sin();
    [interior, minimum, maximum]
        .into_iter()
        .max_by(|a, b| {
            score(*a)
                .total_cmp(&score(*b))
                .then_with(|| b.abs().total_cmp(&a.abs()))
        })
        .unwrap()
}

fn approach_rotation(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
) -> Quat {
    let delta = sub(target, profile.roots[limb]);
    let horizontal = if delta[0].hypot(delta[2]) < 1e-8 {
        [0., 0., 1.]
    } else {
        unit([delta[0], 0.0, delta[2]])
    };
    let incline = APPROACH_INCLINE + pitch;
    let shaft = add(mul(horizontal, incline.cos()), [0., -incline.sin(), 0.]);
    let base = profile.contact_rotations[limb];
    let swing = shortest_arc(
        rotate(base, unit(profile.tip_offsets[limb])),
        shaft,
        reference.palm,
    );
    let palm = projected(rotate(swing, reference.palm), shaft);
    let desired_palm = if reference.palm_reference.is_some() {
        rotate(
            axis_rotation([0., 1., 0.], horizontal[0].atan2(horizontal[2])),
            reference.desired_palm,
        )
    } else {
        reference.desired_palm
    };
    let desired = projected(desired_palm, shaft);
    let roll = if reference.palm_roll_range.is_some() {
        dot(shaft, cross(palm, desired)).atan2(dot(palm, desired))
    } else {
        // Preserve the legacy profile path when no directional policy was supplied.
        (dot(shaft, cross(palm, desired)) / (dot(palm, desired).abs() + 0.2)).atan()
    };
    normalize(multiply(axis_rotation(shaft, roll), multiply(swing, base)))
}
pub fn wrist_deviation(profile: &RigProfile, limb: usize, hand: &HandSolution) -> f64 {
    let Some(frames) = &profile.hand_frames else {
        return 0.;
    };
    let Some(frame) = frames.get(limb) else {
        return 0.;
    };
    let delta = multiply(hand.rotation, inverse(profile.contact_rotations[limb]));
    dot(
        unit(sub(hand.end, hand.middle)),
        rotate(delta, frame.forearm),
    )
    .clamp(-1., 1.)
    .acos()
}

pub fn solve(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
) -> HandSolution {
    solve_with_swivel(profile, limb, target, pitch, reference, 0.0)
}

/// Keep the established anatomical elbow style before changing tool approach.
/// A rigid twist about wrist-to-tip can change the palm without moving any arm
/// joint or contact. Only when that exact family has no admissible witness do
/// we search a bounded shaft yaw, still using the same anatomical elbow pole.
/// Runtime replays the chosen full orientation; this static search never runs
/// between prepared controls.
pub fn solve_with_swivel(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
) -> HandSolution {
    solve_with_swivel_where(profile, limb, target, pitch, reference, swivel, &|_| true)
}

/// Select the same bounded candidates using the caller's physical constraint.
/// A kinematically valid arm-preserving hand must not hide another approach
/// when its actual shaft is obstructed by the equipment.
pub(crate) fn solve_with_swivel_where(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
    accept: &dyn Fn(&HandSolution) -> bool,
) -> HandSolution {
    if reference.palm_roll_range.is_none() {
        return solve_legacy(profile, limb, target, pitch, reference, swivel);
    }
    let old_reference = legacy_reference(reference);
    let old = solve_legacy(profile, limb, target, pitch, &old_reference, swivel);
    if let Some(hand) =
        twist_preserving_arm_where(profile, limb, target, reference, old, false, accept)
    {
        return hand;
    }
    if !old.reachable {
        if let Some(hand) =
            project_reach_cone(profile, limb, target, reference, swivel, old, accept)
        {
            return hand;
        }
    }
    let hand = solve_approach_yaw(profile, limb, target, pitch, reference, swivel, old, accept);
    project_reach_cone(profile, limb, target, reference, swivel, hand, accept).unwrap_or(hand)
}

fn legacy_reference(reference: &HandControlFrame) -> HandControlFrame {
    let mut old_reference = reference.clone();
    old_reference.palm_roll_range = None;
    old_reference.palm_reference = None;
    if reference.palm_reference.is_some() {
        old_reference.desired_palm = unit([reference.desired_palm[0], -1., 0.]);
    }
    old_reference
}

/// First search the existing pitch/swivel family without changing its arm
/// joints. An infeasible rigid twist returns no witness; it never substitutes
/// a yaw/reach correction or an out-of-policy legacy hand.
pub fn solve_preserving_arm_with_swivel(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
) -> Option<HandSolution> {
    solve_preserving_arm_with_swivel_where(profile, limb, target, pitch, reference, swivel, &|_| {
        true
    })
}

pub(crate) fn solve_preserving_arm_with_swivel_where(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
    accept: &dyn Fn(&HandSolution) -> bool,
) -> Option<HandSolution> {
    if reference.palm_roll_range.is_none() {
        return Some(solve_legacy(
            profile, limb, target, pitch, reference, swivel,
        ))
        .filter(accept);
    }
    let old = solve_legacy(
        profile,
        limb,
        target,
        pitch,
        &legacy_reference(reference),
        swivel,
    );
    twist_preserving_arm_where(profile, limb, target, reference, old, true, accept)
}

/// Move only an overextended approach onto the reachable cone of the actual
/// wrist-to-tip offset. This is the shortest rigid orientation change that
/// satisfies the existing elbow extension bound, not another angular search.
/// The actual orientation, rather than its initial pitch prior, is replayed.
#[allow(clippy::too_many_arguments)]
fn project_reach_cone(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    reference: &HandControlFrame,
    swivel: f64,
    hand: HandSolution,
    accept: &dyn Fn(&HandSolution) -> bool,
) -> Option<HandSolution> {
    let offset = profile.offset(limb, hand.rotation);
    let toward = sub(target, profile.roots[limb]);
    let length = distance(offset, [0.; 3]);
    let reach = distance(toward, [0.; 3]);
    if length < 1e-10 || reach < 1e-10 {
        return None;
    }
    let [upper, lower] = profile.lengths[limb];
    let arm = (upper * upper + lower * lower
        - 2. * upper * lower * crate::posture::MAX_ELBOW_ANGLE.cos())
    .sqrt()
        - 1e-8;
    let cosine = (reach * reach + length * length - arm * arm) / (2. * reach * length);
    if !(-1. ..=1.).contains(&cosine) {
        return None;
    }
    let (from, to) = (unit(offset), unit(toward));
    let angle = dot(from, to).clamp(-1., 1.).acos() - cosine.acos();
    if angle <= 0. {
        return None;
    }
    let crossed = cross(from, to);
    let axis = if distance(crossed, [0.; 3]) < 1e-10 {
        projected(reference.palm, from)
    } else {
        unit(crossed)
    };
    let rotation = normalize(multiply(axis_rotation(axis, angle), hand.rotation));
    let corrected = solve_wrist(profile, limb, target, rotation, swivel);
    let corrected = if orientation_policy_valid(profile, limb, rotation)
        && crate::posture::anatomical(profile, limb, target, &corrected)
        && accept(&corrected)
    {
        corrected
    } else {
        twist_preserving_arm_where(profile, limb, target, reference, corrected, false, accept)?
    };
    (corrected.reachable
        && distance(corrected.tip, target) < 1e-9
        && crate::posture::anatomical(profile, limb, target, &corrected)
        && orientation_policy_valid(profile, limb, corrected.rotation)
        && accept(&corrected))
    .then_some(corrected)
}

fn solve_legacy(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
) -> HandSolution {
    let mut rotation = approach_rotation(profile, limb, target, pitch, reference);
    let shaft = rotate(rotation, unit(profile.tip_offsets[limb]));
    for _ in 0..ROLL_REFINEMENTS {
        let hand = solve_wrist(profile, limb, target, rotation, swivel);
        let actual = unit(sub(hand.end, hand.middle));
        let neutral = rotate(
            multiply(rotation, inverse(profile.contact_rotations[limb])),
            reference.forearm,
        );
        let a = projected(neutral, shaft);
        let b = projected(actual, shaft);
        let roll = dot(shaft, cross(a, b)).atan2(dot(a, b));
        rotation = normalize(multiply(axis_rotation(shaft, roll), rotation));
    }
    solve_wrist(profile, limb, target, rotation, swivel)
}

/// Feasible rigid-twist intervals are intersections of three sinusoidal
/// half-planes: both palm bounds and the existing wrist swing limit. Finding
/// their roots is bounded and exact; it does not introduce an angular grid.
#[cfg(test)]
fn preserve_arm_with_twist(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    reference: &HandControlFrame,
    old: HandSolution,
) -> Option<HandSolution> {
    twist_preserving_arm(profile, limb, target, reference, old, false)
}

#[cfg(test)]
fn twist_preserving_arm(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    reference: &HandControlFrame,
    old: HandSolution,
    nearest: bool,
) -> Option<HandSolution> {
    twist_preserving_arm_where(profile, limb, target, reference, old, nearest, &|_| true)
}

fn twist_preserving_arm_where(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    reference: &HandControlFrame,
    old: HandSolution,
    nearest: bool,
    accept: &dyn Fn(&HandSolution) -> bool,
) -> Option<HandSolution> {
    if !old.reachable {
        return None;
    }
    let axis = unit(sub(old.tip, old.end));
    let delta = multiply(old.rotation, inverse(profile.contact_rotations[limb]));
    let palm = rotate(delta, reference.palm);
    let forward = rotate(delta, reference.forearm);
    let shaft = rotate(old.rotation, unit(profile.tip_offsets[limb]));
    let actual = unit(sub(old.end, old.middle));
    let [minimum, maximum] = reference.palm_roll_range?;
    if reference.palm_reference.is_some() && !natural_palm_interval(reference) {
        return None;
    }
    let palm_bounds = if reference.palm_reference.is_some() {
        // The default matched-grip interval is inward/thumb-up through down.
        // (shaft x palm).Y transports the inward half-plane with shaft heading.
        [
            (palm, [0., -1., 0.]),
            (
                cross(shaft, palm),
                [0., reference.desired_palm[0].signum(), 0.],
            ),
        ]
    } else {
        let tangent = cross(palm, shaft);
        [
            (
                sub(mul(tangent, minimum.cos()), mul(palm, minimum.sin())),
                reference.desired_palm,
            ),
            (
                sub(mul(palm, maximum.sin()), mul(tangent, maximum.cos())),
                reference.desired_palm,
            ),
        ]
    };
    let constraints = [
        (palm_bounds[0].0, palm_bounds[0].1, 0.),
        (palm_bounds[1].0, palm_bounds[1].1, 0.),
        (forward, actual, crate::posture::MAX_WRIST_SWING.cos()),
    ];
    let coefficients = constraints.map(|(v, normal, threshold)| {
        let parallel = dot(v, axis);
        (
            dot(normal, sub(v, mul(axis, parallel))),
            dot(normal, cross(axis, v)),
            dot(normal, axis) * parallel - threshold,
        )
    });
    let mut boundaries = vec![-std::f64::consts::PI, 0., std::f64::consts::PI];
    for (a, b, c) in coefficients {
        let radius = a.hypot(b);
        if radius > 1e-12 && c.abs() <= radius {
            let phase = b.atan2(a);
            let half = (-c / radius).acos();
            for angle in [phase - half, phase + half] {
                boundaries.push(
                    (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI,
                );
            }
        }
    }
    boundaries.sort_by(f64::total_cmp);
    let mut candidates = boundaries.clone();
    // The exact wrist-agreement extremum can lie inside a feasible interval.
    // Retaining it avoids choosing a boundary wrist merely because it needs
    // the smallest hand rotation; all candidates keep the same arm joints.
    candidates.push(coefficients[2].1.atan2(coefficients[2].0));
    for pair in boundaries.windows(2) {
        let inset = ((pair[1] - pair[0]) * 0.25).min(1e-9);
        candidates.extend([pair[0] + inset, (pair[0] + pair[1]) * 0.5, pair[1] - inset]);
    }
    candidates.sort_by(|a, b| a.abs().total_cmp(&b.abs()).then_with(|| a.total_cmp(b)));
    let mut best: Option<(f64, f64, HandSolution)> = None;
    for angle in candidates {
        if coefficients
            .iter()
            .any(|&(a, b, c)| a * angle.cos() + b * angle.sin() + c < 0.)
        {
            continue;
        }
        let hand = HandSolution {
            rotation: normalize(multiply(axis_rotation(axis, angle), old.rotation)),
            ..old
        };
        if orientation_policy_valid(profile, limb, hand.rotation)
            && crate::posture::anatomical(profile, limb, target, &hand)
        {
            let cost = if nearest {
                angle.abs()
            } else {
                wrist_comfort(profile, limb, &hand)
                    + inward_grip_cost(profile, limb, hand.rotation)
                    + 0.01 * (1. - (angle * 0.5).cos().abs())
            };
            if best
                .as_ref()
                .is_none_or(|current| (cost, angle.abs()) < (current.0, current.1))
                && accept(&hand)
            {
                best = Some((cost, angle.abs(), hand));
            }
        }
    }
    best.map(|value| value.2)
}

fn wrist_comfort(profile: &RigProfile, limb: usize, hand: &HandSolution) -> f64 {
    0.06 * (wrist_deviation(profile, limb, hand) / crate::posture::MAX_WRIST_SWING).powi(4)
}

/// Natural matched grip prefers thumb-up/inward, with only the pronation needed
/// by reach and wrist comfort. Explicit palm policies keep their own intent.
pub fn inward_grip_cost(profile: &RigProfile, limb: usize, rotation: Quat) -> f64 {
    let Some(reference) = profile.hand_frames.as_ref().map(|frames| &frames[limb]) else {
        return 0.;
    };
    if !natural_palm_interval(reference) {
        return 0.;
    }
    let (roll, _) = palm_orientation(profile, limb, reference, rotation);
    0.08 * (roll / std::f64::consts::FRAC_PI_2).powi(2)
}

fn style_score(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    old: &HandSolution,
    hand: &HandSolution,
) -> (bool, f64, f64) {
    let reach = profile.lengths[limb][0] + profile.lengths[limb][1];
    // Feasibility remains first. Among feasible approaches, retain the old
    // elbow/wrist locations instead of trading a raised arm for minimum wrist.
    let displacement = (distance(old.middle, hand.middle).powi(2)
        + distance(old.end, hand.end).powi(2))
        / reach.powi(2);
    let agreement = old
        .rotation
        .iter()
        .zip(hand.rotation)
        .map(|(a, b)| a * b)
        .sum::<f64>()
        .abs()
        .min(1.);
    (
        !hand.reachable,
        crate::posture::anatomical_violation(profile, limb, target, hand),
        displacement
            + 0.01 * (1. - agreement)
            // A pose selected exactly at the wrist limit loses feasibility
            // under tiny neighboring swivel changes. This is a smooth style
            // cost, not another gate or a relaxation of the physical limit.
            + wrist_comfort(profile, limb, hand)
            + inward_grip_cost(profile, limb, hand.rotation)
            + 0.01 * crate::posture::arm_comfort_cost(profile, limb, target, hand),
    )
}

// Carry the same static contact query and its physical acceptance predicate.
#[allow(clippy::too_many_arguments)]
fn solve_approach_yaw(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
    old: HandSolution,
    accept: &dyn Fn(&HandSolution) -> bool,
) -> HandSolution {
    let [minimum, maximum] = reference.palm_roll_range.unwrap();
    let evaluate = |yaw: f64| {
        let heading = add(
            profile.roots[limb],
            rotate(
                axis_rotation([0., 1., 0.], yaw),
                sub(target, profile.roots[limb]),
            ),
        );
        let base = approach_rotation(profile, limb, heading, pitch, reference);
        let shaft = rotate(base, unit(profile.tip_offsets[limb]));
        let neutral = rotate(
            multiply(base, inverse(profile.contact_rotations[limb])),
            reference.forearm,
        );
        let a = sub(neutral, mul(shaft, dot(neutral, shaft)));
        let mut rotation = base;
        for _ in 0..ROLL_REFINEMENTS {
            let hand = solve_wrist(profile, limb, target, rotation, swivel);
            let actual = unit(sub(hand.end, hand.middle));
            let b = sub(actual, mul(shaft, dot(actual, shaft)));
            let roll = bounded_roll(dot(shaft, cross(a, b)), dot(a, b), minimum, maximum);
            rotation = normalize(multiply(axis_rotation(shaft, roll), base));
        }
        let mut hand = solve_wrist(profile, limb, target, rotation, swivel);
        let rank = |hand: &HandSolution| {
            let score = style_score(profile, limb, target, &old, hand);
            (
                (score.0 || score.1 > 0. || !accept(hand)),
                score.0,
                score.1,
                score.2,
            )
        };
        let mut score = rank(&hand);
        for roll in [minimum, maximum] {
            let alternative = solve_wrist(
                profile,
                limb,
                target,
                normalize(multiply(axis_rotation(shaft, roll), base)),
                swivel,
            );
            let alternative_score = rank(&alternative);
            if alternative_score < score {
                hand = alternative;
                score = alternative_score;
            }
        }
        // Each immutable candidate is compared repeatedly by golden refinement.
        // Retain its exact score instead of recalculating posture trigonometry.
        score.3 += 0.001 * yaw.powi(2);
        (yaw, hand, score)
    };
    let better = |a: &(f64, HandSolution, (bool, bool, f64, f64)),
                  b: &(f64, HandSolution, (bool, bool, f64, f64))| { a.2 < b.2 };
    let mut best = evaluate(0.);
    let (mut lo, mut hi) = (-MAX_APPROACH_YAW, MAX_APPROACH_YAW);
    for yaw in [lo, hi] {
        let candidate = evaluate(yaw);
        if better(&candidate, &best) {
            best = candidate;
        }
    }
    let ratio = (5_f64.sqrt() - 1.) * 0.5;
    let (mut left, mut right) = (
        evaluate(hi - ratio * (hi - lo)),
        evaluate(lo + ratio * (hi - lo)),
    );
    for _ in 0..YAW_REFINEMENTS {
        for candidate in [left, right] {
            if better(&candidate, &best) {
                best = candidate;
            }
        }
        if better(&left, &right) {
            hi = right.0;
            right = left;
            left = evaluate(hi - ratio * (hi - lo));
        } else {
            lo = left.0;
            left = right;
            right = evaluate(lo + ratio * (hi - lo));
        }
    }
    for candidate in [left, right] {
        if better(&candidate, &best) {
            best = candidate;
        }
    }
    best.1
}

fn natural_palm_interval(reference: &HandControlFrame) -> bool {
    let Some([minimum, maximum]) = reference.palm_roll_range else {
        return false;
    };
    reference.palm_reference.is_some()
        && reference.desired_palm[1].abs() < 1e-10
        && reference.desired_palm[2].abs() < 1e-10
        && ((reference.desired_palm[0] - 1.).abs() < 1e-10
            && (minimum + std::f64::consts::FRAC_PI_2).abs() < 1e-10
            && maximum.abs() < 1e-10
            || (reference.desired_palm[0] + 1.).abs() < 1e-10
                && minimum.abs() < 1e-10
                && (maximum - std::f64::consts::FRAC_PI_2).abs() < 1e-10)
}

/// Check the actual physical orientation against the declared palm interval.
/// This also checks interpolated controls; finite/unit quaternion validation is
/// separate. A hand without a policy retains its legacy orientation freedom.
pub fn orientation_policy_valid(profile: &RigProfile, limb: usize, rotation: Quat) -> bool {
    let Some(reference) = profile
        .hand_frames
        .as_ref()
        .and_then(|frames| frames.get(limb))
    else {
        return true;
    };
    let Some([minimum, maximum]) = reference.palm_roll_range else {
        return true;
    };
    let (roll, up) = palm_orientation(profile, limb, reference, rotation);
    roll >= minimum - 1e-9
        && roll <= maximum + 1e-9
        && (!natural_palm_interval(reference) || up <= 1e-9)
}

fn palm_orientation(
    profile: &RigProfile,
    limb: usize,
    reference: &HandControlFrame,
    rotation: Quat,
) -> (f64, f64) {
    let shaft = rotate(rotation, unit(profile.tip_offsets[limb]));
    let palm = rotate(
        multiply(rotation, inverse(profile.contact_rotations[limb])),
        reference.palm,
    );
    let desired = if reference.palm_reference.is_some() {
        rotate(
            axis_rotation([0., 1., 0.], shaft[0].atan2(shaft[2])),
            reference.desired_palm,
        )
    } else {
        reference.desired_palm
    };
    let base = projected(desired, shaft);
    let actual = projected(palm, shaft);
    let roll = dot(shaft, cross(base, actual)).atan2(dot(base, actual));
    (roll, palm[1])
}

/// Retain the selected actual tool orientation relative to its contact
/// approach. Static arm-preserving witnesses keep their joints at controls;
/// intermediate poses follow one shortest quaternion arc, without a new solve.
pub fn encode_orientation(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    rotation: Quat,
) -> OrientationControl {
    let approach = approach_rotation(profile, limb, target, pitch, reference);
    normalize(multiply(inverse(approach), rotation))
}
pub fn decode_orientation(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    control: OrientationControl,
) -> Quat {
    normalize(multiply(
        approach_rotation(profile, limb, target, pitch, reference),
        control,
    ))
}

/// Evaluate a prepared physical orientation without a pointwise search. The
/// measured mount, real tool contact and anatomical elbow pole are unchanged.
pub fn solve_with_rotation(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    rotation: Quat,
    swivel: f64,
) -> HandSolution {
    // Prepared orientations are replayed verbatim. A pointwise projection can
    // jump when a disconnected feasible twist interval appears, even when the
    // input curve is smooth. Planning and inspection retain any policy/posture
    // failure instead of concealing it with an unplanned runtime correction.
    solve_wrist(profile, limb, target, rotation, swivel)
}

/// Recover the selected bounded roll in the same shaft/approach frame used by
/// fixed-roll replay. Quaternion signs do not change the returned angle.
pub fn selected_roll(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    hand: &HandSolution,
) -> Option<f64> {
    let [minimum, maximum] = reference.palm_roll_range?;
    let base = approach_rotation(profile, limb, target, pitch, reference);
    let shaft = rotate(base, unit(profile.tip_offsets[limb]));
    if distance(
        shaft,
        rotate(hand.rotation, unit(profile.tip_offsets[limb])),
    ) > 1e-8
    {
        return None;
    }
    let delta = multiply(hand.rotation, inverse(base));
    let angle = 2. * dot([delta[0], delta[1], delta[2]], shaft).atan2(delta[3]);
    let mut roll =
        (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
    if roll < minimum - 1e-9 {
        roll += std::f64::consts::TAU;
    }
    if roll > maximum + 1e-9 {
        roll -= std::f64::consts::TAU;
    }
    Some(roll.clamp(minimum, maximum))
}

/// Evaluate the prepared roll without an orientation search. Contact and bone
/// lengths remain exact, and the same elbow swivel convention is retained.
pub fn solve_with_roll(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
    roll: f64,
) -> HandSolution {
    let base = approach_rotation(profile, limb, target, pitch, reference);
    let shaft = rotate(base, unit(profile.tip_offsets[limb]));
    solve_wrist(
        profile,
        limb,
        target,
        normalize(multiply(axis_rotation(shaft, roll), base)),
        swivel,
    )
}

fn solve_wrist(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    rotation: Quat,
    swivel: f64,
) -> HandSolution {
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    let wrist = sub(target, profile.offset(limb, rotation));
    let desired_distance = distance(root, wrist);
    let axis = unit(sub(wrist, root));
    let inner = (upper - lower).abs();
    let outer = upper + lower;
    let reachable = desired_distance >= inner + 1e-5 && desired_distance <= outer - 1e-4;
    let length = desired_distance.clamp(inner + 1e-6, outer - 1e-6);
    let end = add(root, mul(axis, length));
    let outward = if limb == 0 { -1. } else { 1. };
    let shoulders = sub(profile.roots[1], profile.roots[0]);
    let horizontal = [shoulders[0], 0., shoulders[2]];
    let right = if distance(horizontal, [0.; 3]) < 1e-8 {
        [1., 0., 0.]
    } else {
        unit(horizontal)
    };
    let forward = [-right[2], 0., right[0]];
    let transverse = unit(add(mul(right, outward * 0.45), [0., -1., 0.]));
    // Hand pronation never changes the elbow reference plane. The coordinator
    // now bounds the same body-relative swivel that fixed-orientation replay uses.
    let pole = rotate(shortest_arc(forward, axis, transverse), transverse);
    let bend = rotate(axis_rotation(axis, swivel), pole);
    let along = (upper * upper - lower * lower + length * length) / (2. * length);
    let radius = (upper * upper - along * along).max(0.).sqrt();
    let middle = add(add(root, mul(axis, along)), mul(bend, radius));
    HandSolution {
        middle,
        end,
        tip: add(end, profile.offset(limb, rotation)),
        rotation,
        reachable,
    }
}

#[cfg(test)]
#[path = "hand_solver_tests.rs"]
mod tests;
