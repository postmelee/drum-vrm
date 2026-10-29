//! Target-oriented held-tool contact. Fix the shaft approach, recover the wrist
//! from measured tool markers, and solve an ordinary two-bone arm. A bounded
//! refinement reduces wrist swing without changing the grip. Static bounded
//! palm witnesses are replayed through a separately prepared continuous roll.
use super::{rotate, HandControlFrame, Quat, RigProfile};
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
const ACTUAL_ROLL_REFINEMENTS: usize = 12;

#[derive(Clone, Copy)]
struct RollCandidate {
    hand: HandSolution,
    score: (bool, f64, f64, f64),
}
fn roll_candidate(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    hand: HandSolution,
    roll: f64,
) -> RollCandidate {
    let wrist = wrist_deviation(profile, limb, &hand);
    RollCandidate {
        hand,
        score: (
            !hand.reachable,
            crate::posture::anatomical_violation(profile, limb, target, &hand),
            wrist * wrist + crate::posture::comfort_cost(profile, limb, target, &hand),
            roll.abs(),
        ),
    }
}

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

/// The numeric coordinator owns pitch and elbow swivel. Roll around the fixed
/// shaft reduces measured wrist swing and anatomical deficits within the selected
/// grip-roll interval.
/// Legacy profiles retain their unconstrained refinement. Each iteration
/// recomputes the exact wrist and arm, so no bone
/// stretching or marker correction is hidden in the returned pose.
pub fn solve_with_swivel(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
) -> HandSolution {
    let mut rotation = approach_rotation(profile, limb, target, pitch, reference);
    let shaft = rotate(rotation, unit(profile.tip_offsets[limb]));
    if let Some([minimum, maximum]) = reference.palm_roll_range {
        let base = rotation;
        let neutral = rotate(
            multiply(base, inverse(profile.contact_rotations[limb])),
            reference.forearm,
        );
        let a = sub(neutral, mul(shaft, dot(neutral, shaft)));
        let mut analytic_roll = 0.;
        for _ in 0..ROLL_REFINEMENTS {
            let hand = solve_wrist(profile, limb, target, rotation, swivel);
            let actual = unit(sub(hand.end, hand.middle));
            let b = sub(actual, mul(shaft, dot(actual, shaft)));
            analytic_roll = bounded_roll(dot(shaft, cross(a, b)), dot(a, b), minimum, maximum);
            rotation = normalize(multiply(axis_rotation(shaft, analytic_roll), base));
        }
        let final_hand = solve_wrist(profile, limb, target, rotation, swivel);
        // A valid static witness needs no further style optimization. Roll
        // preserves the reachable tool centerline, so the caller's physical
        // tool checks are unchanged. Prepared curves own runtime continuity.
        if final_hand.reachable
            && crate::posture::anatomical_violation(profile, limb, target, &final_hand) == 0.
        {
            return final_hand;
        }
        // Static feasibility retains hard-priority witnesses. The final elbow
        // depends on roll, so compare exact coupled poses and refine the same
        // scalar interval. Runtime replay uses the prepared roll curve below;
        // it must not rerun this pointwise argmin between control knots.
        let evaluate = |roll: f64| {
            roll_candidate(
                profile,
                limb,
                target,
                solve_wrist(
                    profile,
                    limb,
                    target,
                    normalize(multiply(axis_rotation(shaft, roll), base)),
                    swivel,
                ),
                roll,
            )
        };
        let mut best = roll_candidate(profile, limb, target, final_hand, analytic_roll);
        let mut consider = |candidate: RollCandidate| {
            if candidate.score < best.score {
                best = candidate;
            }
        };
        consider(evaluate(minimum));
        consider(evaluate(maximum));
        let ratio = (5_f64.sqrt() - 1.) * 0.5;
        let (mut lo, mut hi) = (minimum, maximum);
        let mut left_roll = hi - ratio * (hi - lo);
        let mut right_roll = lo + ratio * (hi - lo);
        let (mut left, mut right) = (evaluate(left_roll), evaluate(right_roll));
        consider(left);
        consider(right);
        for _ in 0..ACTUAL_ROLL_REFINEMENTS {
            if left.score < right.score {
                hi = right_roll;
                right_roll = left_roll;
                right = left;
                left_roll = hi - ratio * (hi - lo);
                left = evaluate(left_roll);
                consider(left);
            } else {
                lo = left_roll;
                left_roll = right_roll;
                left = right;
                right_roll = lo + ratio * (hi - lo);
                right = evaluate(right_roll);
                consider(right);
            }
        }
        return best.hand;
    }
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
    let mut pole = rotate(shortest_arc(forward, axis, transverse), transverse);
    if let Some(reference) = profile
        .hand_frames
        .as_ref()
        .and_then(|f| f.get(limb))
        .filter(|f| f.palm_roll_range.is_some())
    {
        let neutral = rotate(
            multiply(rotation, inverse(profile.contact_rotations[limb])),
            reference.forearm,
        );
        let preferred = sub(mul(axis, dot(neutral, axis)), neutral);
        // The closest exact elbow on the arm's bend circle aligns the
        // measured forearm with the hand. Axial alignment leaves that circle
        // underdetermined, so retain the transported anatomical pole there.
        if distance(preferred, [0.; 3]) > 1e-8 {
            pole = unit(preferred);
        }
    }
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
