//! Closed-form neutral-wrist contact. Treat the forearm and held tool as one
//! rigid link while solving the elbow, then recover the actual wrist. This
//! preserves both bone lengths and the calibrated hand/tool relationship.
use super::{pitch_rotation, rotate, HandControlFrame, Quat, RigProfile};
use crate::model::Vec3;
use crate::motion::{add, distance, sub};
use std::f64::consts::PI;

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
/// Select forearm roll about the compound link, retaining the requested palm
/// direction where geometrically possible. No animation history is required.
fn palm_roll(delta: Quat, axis: Vec3, reference: &HandControlFrame) -> Quat {
    let lateral = rotate(delta, cross(reference.forearm, reference.palm));
    let desired = reference.desired_palm;
    let c = dot(desired, axis) * dot(axis, lateral);
    let a = dot(desired, lateral) - c;
    let b = dot(desired, cross(axis, lateral));
    let radius = a.hypot(b);
    if radius < 1e-9 {
        return delta;
    }
    let phase = b.atan2(a);
    let angle = (-c / radius).clamp(-1.0, 1.0).acos();
    let candidates = if c.abs() <= radius {
        [phase + angle, phase - angle]
    } else {
        let closest = phase + if c > 0.0 { PI } else { 0.0 };
        [closest, closest]
    };
    let mut selected = delta;
    let mut best = f64::NEG_INFINITY;
    for angle in candidates {
        let candidate = normalize(multiply(axis_rotation(axis, angle), delta));
        let alignment = dot(rotate(candidate, reference.palm), desired);
        if alignment > best {
            best = alignment;
            selected = candidate;
        }
    }
    selected
}

pub fn solve(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    reference: &HandControlFrame,
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
    // Recover the calibrated elbow pole so a neutral rest target reproduces
    // its original pose instead of introducing a posture jump at preparation.
    let reference_wrist = sub(
        profile.rest_tips[limb],
        profile.offset(limb, profile.contact_rotations[limb]),
    );
    let reference_elbow = sub(reference_wrist, mul(reference.forearm, lower));
    let bend = projected(sub(reference_elbow, root), axis);
    let along =
        (upper * upper - compound_length * compound_length + length * length) / (2.0 * length);
    let middle = add(
        add(root, mul(axis, along)),
        mul(bend, (upper * upper - along * along).max(0.0).sqrt()),
    );
    let effective = sub(tip, middle);
    let effective_axis = unit(effective);
    let delta = multiply(
        frame(effective, reference.desired_palm),
        inverse(frame(compound, reference.desired_palm)),
    );
    let delta = palm_roll(delta, effective_axis, reference);
    let end = add(middle, mul(rotate(delta, reference.forearm), lower));
    let rotation = normalize(multiply(delta, template));
    // Return the actual tool contact, including unreachable-target clamping.
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
