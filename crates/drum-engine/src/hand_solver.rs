//! Closed-form neutral-wrist contact. Treat the forearm and held tool as one
//! rigid link while solving the elbow, then recover the actual wrist. This
//! preserves both bone lengths and the calibrated hand/tool relationship.
use super::{pitch_rotation, rotate, HandControlFrame, Quat, RigProfile};
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
#[cfg(test)]
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
/// The shortest swing preserves the calibrated neutral wrist without building
/// an up-vector frame that flips when the effective link crosses vertical.
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
/// A bounded continuous correction retains palm preference without selecting
/// between two exact roots. The neutral calibration remains the roll reference;
/// a near-degenerate projected palm cannot cause an arbitrary half turn.
fn palm_roll(delta: Quat, axis: Vec3, reference: &HandControlFrame) -> Quat {
    let palm = rotate(delta, reference.palm);
    let desired = reference.desired_palm;
    let a = dot(desired, palm) - dot(desired, axis) * dot(axis, palm);
    let b = dot(desired, cross(axis, palm));
    let angle = (b / (a.abs() + 0.2)).atan();
    normalize(multiply(axis_rotation(axis, angle), delta))
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

/// Swivel changes only the elbow plane around the shoulder/contact axis. The
/// numeric coordinator owns its bounded timeline; contact and lengths stay exact.
pub fn solve_with_swivel(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
    swivel: f64,
) -> HandSolution {
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    let template = pitch_rotation(profile.contact_rotations[limb], pitch);
    let compound = add(
        mul(reference.forearm, lower),
        profile.offset(limb, template),
    );
    let compound_length = distance(compound, [0.0; 3]);
    let desired_distance = distance(root, target);
    let axis = unit(sub(target, root));
    if compound_length < 1e-6 {
        // Degenerate custom profiles can fold the tool back onto the elbow.
        // Keep finite real bone lengths and expose this as unreachable.
        let middle = add(root, mul(axis, upper));
        let end = add(middle, mul(reference.forearm, lower));
        return HandSolution {
            middle,
            end,
            tip: add(end, profile.offset(limb, template)),
            rotation: template,
            reachable: false,
        };
    }
    let inner = (upper - compound_length).abs();
    let outer = upper + compound_length;
    let reachable = desired_distance >= inner + 1e-5 && desired_distance <= outer - 1e-4;
    let length = desired_distance.clamp(inner + 1e-6, outer - 1e-6);
    let tip = add(root, mul(axis, length));
    // Prefer a low elbow with modest outward/front clearance. A pole recovered
    // from the wrist rest target can project upward and behind the shoulder
    // when a long forearm/tool link folds toward a near drum target.
    let outward = if limb == 0 { -1.0 } else { 1.0 };
    // Transport a transverse anatomical pole from the forward hemisphere.
    // Projecting one fixed 3D pole onto the target axis instead has a singularity
    // at a reachable drum target parallel to that pole and can flip the elbow.
    let shoulder_delta = sub(profile.roots[1], profile.roots[0]);
    let horizontal = [shoulder_delta[0], 0.0, shoulder_delta[2]];
    let shoulder_axis = if distance(horizontal, [0.0; 3]) < 1e-8 {
        [1.0, 0.0, 0.0]
    } else {
        unit(horizontal)
    };
    let forward = [-shoulder_axis[2], 0.0, shoulder_axis[0]];
    let transverse = unit(add(mul(shoulder_axis, outward), [0.0, -0.8, 0.0]));
    let bend = rotate(shortest_arc(forward, axis, transverse), transverse);
    let bend = rotate(axis_rotation(axis, swivel), bend);
    let along =
        (upper * upper - compound_length * compound_length + length * length) / (2.0 * length);
    let radius = (upper * upper - along * along).max(0.0).sqrt();
    let middle = add(add(root, mul(axis, along)), mul(bend, radius));
    let effective = sub(tip, middle);
    let delta = shortest_arc(compound, effective, reference.palm);
    let delta = palm_roll(delta, unit(effective), reference);
    let end = add(middle, mul(rotate(delta, reference.forearm), lower));
    let rotation = normalize(multiply(delta, template));
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
