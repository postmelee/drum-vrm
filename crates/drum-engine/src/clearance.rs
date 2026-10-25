//! Conservative skeletal proxies, not a certificate for clothing or skin meshes.
use crate::model::Vec3;
use crate::motion::{add, distance, sub};
use crate::rig::{HandSolution, RigProfile};
use serde::Serialize;
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn mul(a: Vec3, s: f64) -> Vec3 {
    a.map(|v| v * s)
}
pub fn segment_distance(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> f64 {
    let u = sub(b, a);
    let v = sub(d, c);
    let w = sub(a, c);
    let (aa, bb, cc, dd, ee) = (dot(u, u), dot(u, v), dot(v, v), dot(u, w), dot(v, w));
    let den = aa * cc - bb * bb;
    let mut s = if den > 1e-12 {
        ((bb * ee - cc * dd) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if cc > 1e-12 {
        ((bb * s + ee) / cc).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if aa > 1e-12 {
        s = ((bb * t - dd) / aa).clamp(0.0, 1.0);
    }
    if cc > 1e-12 {
        t = ((bb * s + ee) / cc).clamp(0.0, 1.0);
    }
    distance(add(a, mul(u, s)), add(c, mul(v, t)))
}
pub fn torso_clearance(profile: &RigProfile, a: Vec3, b: Vec3, radius: f64) -> f64 {
    // Elliptical upright torso, estimated only from the semantic shoulder span.
    let width = distance(profile.roots[0], profile.roots[1]);
    let rx = (width * 0.44).clamp(0.055, 0.30);
    let rz = (width * 0.29).clamp(0.045, 0.20);
    let right = sub(profile.roots[1], profile.roots[0]);
    let horizontal = (right[0] * right[0] + right[2] * right[2]).sqrt().max(1e-9);
    let right = [right[0] / horizontal, 0.0, right[2] / horizontal];
    let forward = [-right[2], 0.0, right[0]];
    let center = [
        (profile.roots[0][0] + profile.roots[1][0]) * 0.5,
        0.0,
        (profile.roots[0][2] + profile.roots[1][2]) * 0.5,
    ];
    let low = profile.pelvis[1] + 0.04;
    let high = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 0.025;
    let mut minimum = f64::INFINITY;
    for i in 0..=12 {
        let p = add(a, mul(sub(b, a), i as f64 / 12.0));
        let delta = sub(p, center);
        let radial = ((dot(delta, right) / rx).powi(2) + (dot(delta, forward) / rz).powi(2)).sqrt();
        let side = (radial - 1.0) * rx.min(rz);
        let vertical = (low - p[1]).max(p[1] - high);
        minimum = minimum.min(side.max(vertical) - radius);
    }
    minimum
}
pub fn hand_clearance(profile: &RigProfile, limb: usize, hand: &HandSolution) -> f64 {
    let root = profile.roots[limb];
    let arm_radius = (profile.lengths[limb][0] * 0.085).clamp(0.015, 0.035);
    // The shoulder joint intentionally touches the torso; inspect its outward path.
    let start = add(root, mul(sub(hand.middle, root), 0.22));
    torso_clearance(profile, start, hand.middle, arm_radius)
        .min(torso_clearance(
            profile,
            hand.middle,
            hand.end,
            arm_radius * 0.8,
        ))
        .min(torso_clearance(profile, hand.end, hand.tip, 0.006))
}
pub fn hand_cost(profile: &RigProfile, limb: usize, target: Vec3) -> f64 {
    profile.solve_hand(limb, target, 0.0).map_or(0.0, |s| {
        (-hand_clearance(profile, limb, &s)).max(0.0) * 80.0
    })
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoseInspection {
    pub scope: &'static str,
    pub torso_clearance_meters: f64,
    pub arm_clearance_meters: f64,
    pub stick_clearance_meters: f64,
    pub bone_length_error_meters: f64,
    pub pose_floats: usize,
}
pub fn inspect(profile: &RigProfile, pose: &[f32]) -> PoseInspection {
    let point = |o: usize| [pose[o] as f64, pose[o + 1] as f64, pose[o + 2] as f64];
    let mut torso = f64::INFINITY;
    let mut bones: f64 = 0.0;
    for limb in 0..4 {
        let o = limb * 12;
        bones = bones.max((distance(point(o), point(o + 3)) - profile.lengths[limb][0]).abs());
        bones = bones.max((distance(point(o + 3), point(o + 6)) - profile.lengths[limb][1]).abs());
        if limb < 2 {
            let s = HandSolution {
                middle: point(o + 3),
                end: point(o + 6),
                tip: point(o + 9),
                rotation: [0.0, 0.0, 0.0, 1.0],
                reachable: true,
            };
            torso = torso.min(hand_clearance(profile, limb, &s));
        }
    }
    let mut arm = f64::INFINITY;
    for a in [0, 3] {
        for b in [12, 15] {
            arm = arm.min(segment_distance(point(a), point(a + 3), point(b), point(b + 3)) - 0.04);
        }
    }
    let stick = segment_distance(point(6), point(9), point(18), point(21)) - 0.012;
    PoseInspection {
        scope: "skeletal-proxies",
        torso_clearance_meters: torso,
        arm_clearance_meters: arm,
        stick_clearance_meters: stick,
        bone_length_error_meters: bones,
        pose_floats: pose.len(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segment_crossing_and_parallel_distances() {
        assert!(segment_distance([-1., 0., 0.], [1., 0., 0.], [0., -1., 0.], [0., 1., 0.]) < 1e-10);
        assert!(
            (segment_distance([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]) - 1.).abs()
                < 1e-10
        );
    }
}
