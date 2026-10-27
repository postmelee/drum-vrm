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
/// Closest points of finite segments, including degenerate/parallel cases.
pub fn segment_distance(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> f64 {
    let u = sub(b, a);
    let v = sub(d, c);
    let w = sub(a, c);
    let (aa, bb, cc, dd, ee) = (dot(u, u), dot(u, v), dot(v, v), dot(u, w), dot(v, w));
    let (mut s, mut t);
    if aa <= 1e-14 && cc <= 1e-14 {
        return distance(a, c);
    }
    if aa <= 1e-14 {
        s = 0.0;
        t = (ee / cc).clamp(0.0, 1.0);
    } else if cc <= 1e-14 {
        t = 0.0;
        s = (-dd / aa).clamp(0.0, 1.0);
    } else {
        let den = aa * cc - bb * bb;
        s = if den > 1e-14 {
            ((bb * ee - cc * dd) / den).clamp(0.0, 1.0)
        } else {
            0.0
        };
        t = (bb * s + ee) / cc;
        if t < 0.0 {
            t = 0.0;
            s = (-dd / aa).clamp(0.0, 1.0);
        } else if t > 1.0 {
            t = 1.0;
            s = ((bb - dd) / aa).clamp(0.0, 1.0);
        }
    }
    distance(add(a, mul(u, s)), add(c, mul(v, t)))
}
pub fn geometry_source(profile: &RigProfile) -> &str {
    profile
        .body_geometry
        .as_ref()
        .map_or("skeleton-fallback", |b| b.source.as_str())
}
pub fn arm_radii(profile: &RigProfile, limb: usize) -> [f64; 2] {
    profile.body_geometry.as_ref().map_or_else(
        || {
            let upper = (profile.lengths[limb][0] * 0.085).clamp(0.015, 0.035);
            [upper, upper * 0.8]
        },
        |body| body.arm_radii[limb],
    )
}
pub fn hand_radius(profile: &RigProfile, limb: usize) -> f64 {
    profile
        .body_geometry
        .as_ref()
        .map_or(0.025, |body| body.hand_radii[limb])
}
pub fn hand_center(profile: &RigProfile, limb: usize, hand: &HandSolution) -> Vec3 {
    profile
        .body_geometry
        .as_ref()
        .and_then(|b| b.hand_centers.as_ref())
        .map_or(hand.end, |centers| {
            add(hand.end, crate::rig::rotate(hand.rotation, centers[limb]))
        })
}
pub fn hand_volume(profile: &RigProfile, limb: usize, hand: &HandSolution) -> (Vec3, Vec3, f64) {
    profile
        .body_geometry
        .as_ref()
        .and_then(|b| b.hand_capsules.as_ref())
        .map_or_else(
            || {
                let center = hand_center(profile, limb, hand);
                (center, center, hand_radius(profile, limb))
            },
            |capsules| {
                let c = &capsules[limb];
                (
                    add(hand.end, crate::rig::rotate(hand.rotation, c.start)),
                    add(hand.end, crate::rig::rotate(hand.rotation, c.end)),
                    c.radius,
                )
            },
        )
}
pub fn torso_clearance(profile: &RigProfile, a: Vec3, b: Vec3, radius: f64) -> f64 {
    let width = distance(profile.roots[0], profile.roots[1]);
    let low = profile.pelvis[1] + 0.04;
    let high = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 0.025;
    let (center, extents) = profile.body_geometry.as_ref().map_or(
        (
            [
                (profile.roots[0][0] + profile.roots[1][0]) * 0.5,
                (low + high) * 0.5,
                (profile.roots[0][2] + profile.roots[1][2]) * 0.5,
            ],
            [
                (width * 0.44).clamp(0.055, 0.30),
                ((high - low) * 0.5).max(0.01),
                (width * 0.29).clamp(0.045, 0.20),
            ],
        ),
        |body| (body.torso.center, body.torso.half_extents),
    );
    let right = sub(profile.roots[1], profile.roots[0]);
    let horizontal = right[0].hypot(right[2]).max(1e-9);
    let right = [right[0] / horizontal, 0.0, right[2] / horizontal];
    let forward = [-right[2], 0.0, right[0]];
    let delta = sub(b, a);
    // A convex 1-Lipschitz lower bound on distance to a capped elliptical
    // cylinder. Minimize over the complete segment, not 13 isolated points.
    // The remaining interval's Lipschitz bound prevents a narrow missed crossing.
    let value = |t: f64| {
        let point = sub(add(a, mul(delta, t)), center);
        let radial = ((dot(point, right) / extents[0]).powi(2)
            + (dot(point, forward) / extents[2]).powi(2))
        .sqrt();
        ((radial - 1.0) * extents[0].min(extents[2])).max(point[1].abs() - extents[1])
    };
    if a == b {
        return value(0.0) - radius;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    const R: f64 = 0.618_033_988_749_894_9;
    let (mut x, mut y) = (hi - R * (hi - lo), lo + R * (hi - lo));
    let (mut vx, mut vy) = (value(x), value(y));
    for _ in 0..16 {
        if vx < vy {
            hi = y;
            y = x;
            vy = vx;
            x = hi - R * (hi - lo);
            vx = value(x);
        } else {
            lo = x;
            x = y;
            vx = vy;
            y = lo + R * (hi - lo);
            vy = value(y);
        }
    }
    value((lo + hi) * 0.5) - (hi - lo) * distance(a, b) * 0.5 - radius
}
pub fn hand_clearance(profile: &RigProfile, limb: usize, hand: &HandSolution) -> f64 {
    let root = profile.roots[limb];
    let radii = arm_radii(profile, limb);
    // A connected shoulder is allowed to overlap its own torso while the upper
    // arm moves toward its own outer side. A capped convex torso has only one
    // proximal attachment interval: requiring the elbow capsule to clear is
    // sufficient beyond that interval. Inward/cross-body routes receive no such
    // exemption. This derives the attachment from geometry instead of assuming
    // every avatar's upper-arm origin lies on the outer skin surface.
    let center = profile.body_geometry.as_ref().map_or(
        [
            (profile.roots[0][0] + profile.roots[1][0]) * 0.5,
            profile.chest[1],
            (profile.roots[0][2] + profile.roots[1][2]) * 0.5,
        ],
        |body| body.torso.center,
    );
    let shoulder_axis = sub(profile.roots[1], profile.roots[0]);
    let horizontal = shoulder_axis[0].hypot(shoulder_axis[2]).max(1e-9);
    let right = [
        shoulder_axis[0] / horizontal,
        0.0,
        shoulder_axis[2] / horizontal,
    ];
    let sign = if limb == 0 { -1.0 } else { 1.0 };
    let outward = sign * dot(sub(hand.middle, root), right) >= 0.0
        && sign * dot(sub(hand.middle, center), right) > 0.0;
    // Torso measurements are capped proxies; the connected shoulder joint may
    // lie just above the cap instead of inside it. Both describe the same
    // anatomical attachment when the joint projects inside the elliptical cap,
    // is within one upper-arm diameter above it, and the upper-arm path moves outward
    // on its own side. A convex torso has one intersection interval: the clear
    // elbow capsule bounds its exit, and the full forearm/hand/tool are checked.
    // No fixed fraction of the arm is ignored and inward routes are never exempt.
    let near_cap = profile.body_geometry.as_ref().is_some_and(|body| {
        let top = body.torso.center[1] + body.torso.half_extents[1];
        let offset = sub(root, body.torso.center);
        let forward = [-right[2], 0.0, right[0]];
        let footprint = (dot(offset, right) / body.torso.half_extents[0]).powi(2)
            + (dot(offset, forward) / body.torso.half_extents[2]).powi(2);
        root[1] >= top && root[1] - top <= 2.0 * radii[0] && footprint <= 1.0
    });
    let connected = outward && (torso_clearance(profile, root, root, radii[0]) <= 0.0 || near_cap);
    let upper_clearance = if connected {
        torso_clearance(profile, hand.middle, hand.middle, radii[0])
    } else {
        torso_clearance(profile, root, hand.middle, radii[0])
    };
    let (hand_start, hand_end, hand_r) = hand_volume(profile, limb, hand);
    upper_clearance
        .min(torso_clearance(profile, hand.middle, hand.end, radii[1]))
        .min(torso_clearance(profile, hand_start, hand_end, hand_r))
        .min(torso_clearance(profile, hand.end, hand.tip, 0.006))
}
/// Shared between calibration, torso/elbow planning and inspection. Own-hand
/// grip/tool attachment is allowed; the opposite hand/arm is an obstacle.
#[derive(Clone, Copy, Debug)]
pub struct PairClearance {
    pub arms: f64,
    pub sticks: f64,
    pub cross: f64,
}
impl PairClearance {
    pub fn minimum(self) -> f64 {
        self.arms.min(self.sticks).min(self.cross)
    }
}
pub fn pair_clearance(
    profile: &RigProfile,
    left: &HandSolution,
    right: &HandSolution,
) -> PairClearance {
    let segments = |limb: usize, hand: &HandSolution| {
        let r = arm_radii(profile, limb);
        [
            (profile.roots[limb], hand.middle, r[0]),
            (hand.middle, hand.end, r[1]),
            hand_volume(profile, limb, hand),
        ]
    };
    let a = segments(0, left);
    let b = segments(1, right);
    let mut arms = f64::INFINITY;
    for (start, end, r) in a {
        for (other_start, other_end, other_r) in b {
            arms = arms.min(segment_distance(start, end, other_start, other_end) - r - other_r);
        }
    }
    let mut cross = f64::INFINITY;
    for (start, end, r) in b {
        cross = cross.min(segment_distance(left.end, left.tip, start, end) - 0.006 - r);
    }
    for (start, end, r) in a {
        cross = cross.min(segment_distance(right.end, right.tip, start, end) - 0.006 - r);
    }
    PairClearance {
        arms,
        sticks: segment_distance(left.end, left.tip, right.end, right.tip) - 0.012,
        cross,
    }
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
    pub cross_clearance_meters: f64,
    pub geometry_source: String,
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
                rotation: std::array::from_fn(|i| pose[57 + limb * 4 + i] as f64),
                reachable: true,
            };
            torso = torso.min(hand_clearance(profile, limb, &s));
        }
    }
    let hands: [HandSolution; 2] = std::array::from_fn(|i| HandSolution {
        middle: point(i * 12 + 3),
        end: point(i * 12 + 6),
        tip: point(i * 12 + 9),
        rotation: std::array::from_fn(|j| pose[57 + i * 4 + j] as f64),
        reachable: true,
    });
    let pair = pair_clearance(profile, &hands[0], &hands[1]);
    PoseInspection {
        scope: "skeletal-proxies",
        torso_clearance_meters: torso,
        arm_clearance_meters: pair.arms,
        stick_clearance_meters: pair.sticks,
        cross_clearance_meters: pair.cross,
        geometry_source: geometry_source(profile).into(),
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
    #[test]
    fn closest_segments_cover_endpoints_degenerate_and_order_symmetry() {
        for (a, b, c, d, expected) in [
            ([0., 0., 0.], [0., 0., 0.], [1., 0., 0.], [1., 0., 0.], 1.),
            ([0., 0., 0.], [0., 0., 0.], [-1., 2., 0.], [1., 2., 0.], 2.),
            (
                [0., 0., 0.],
                [1., 0., 0.],
                [2., 1., 0.],
                [2., 2., 0.],
                2f64.sqrt(),
            ),
            (
                [0., 0., 0.],
                [1., 0., 0.],
                [0.5, -1., 1.],
                [0.5, 1., 1.],
                1.,
            ),
            (
                [0., 0., 0.],
                [1., 0., 0.],
                [0.25, 0., 0.],
                [0.75, 0., 0.],
                0.,
            ),
        ] {
            for result in [
                segment_distance(a, b, c, d),
                segment_distance(c, d, a, b),
                segment_distance(b, a, d, c),
            ] {
                assert!((result - expected).abs() < 1e-10, "{result} vs {expected}");
            }
        }
    }
    #[test]
    fn segment_torso_check_catches_narrow_crossing_and_measured_depth() {
        let mut profile = RigProfile::primitive();
        profile.body_geometry = Some(crate::rig::BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            sample_count: 500,
            torso: crate::rig::TorsoGeometry {
                center: [0.0, 1.0, 0.0],
                half_extents: [0.01, 0.2, 0.2],
            },
            arm_radii: [[0.03, 0.02]; 2],
            hand_radii: [0.025; 2],
            leg_radii: None,
            foot_radii: None,
            leg_boxes: None,
            leg_rest_frames: None,
            foot_boxes: None,
            hand_centers: None,
            hand_capsules: None,
        });
        let a = [-1.07, 1., 0.15];
        let b = [0.93, 1., 0.15];
        assert!(torso_clearance(&profile, a, b, 0.006) < 0.0);
        assert!(torso_clearance(&profile, [-1., 1., 0.5], [1., 1., 0.5], 0.006) > 0.0);
        let shallow = {
            let mut p = profile.clone();
            p.body_geometry.as_mut().unwrap().torso.half_extents[2] = 0.05;
            p
        };
        assert!(torso_clearance(&shallow, a, b, 0.006) > 0.0);
    }
    #[test]
    fn connected_shoulders_allow_outward_attachment_but_not_cross_body_routes() {
        let mut profile = RigProfile::primitive();
        profile.roots[0] = [-0.08, 1.2, 0.0];
        profile.roots[1] = [0.08, 1.2, 0.0];
        profile.body_geometry = Some(crate::rig::BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            sample_count: 100,
            torso: crate::rig::TorsoGeometry {
                center: [0.0, 1.0, 0.0],
                half_extents: [0.15, 0.25, 0.12],
            },
            arm_radii: [[0.075, 0.04]; 2],
            hand_radii: [0.03; 2],
            leg_radii: None,
            foot_radii: None,
            leg_boxes: None,
            leg_rest_frames: None,
            foot_boxes: None,
            hand_centers: None,
            hand_capsules: None,
        });
        let hand = |x| HandSolution {
            middle: [x, 1.1, 0.1],
            end: [x, 1.1, 0.4],
            tip: [x, 1.1, 0.7],
            rotation: [0.0, 0.0, 0.0, 1.0],
            reachable: true,
        };
        assert!(torso_clearance(&profile, profile.roots[0], profile.roots[0], 0.075) < 0.0);
        assert!(hand_clearance(&profile, 0, &hand(-0.3)) > 0.002);
        assert!(hand_clearance(&profile, 0, &hand(0.3)) < 0.0);
        let mut embedded_elbow = hand(-0.1);
        embedded_elbow.middle[2] = 0.0;
        assert!(hand_clearance(&profile, 0, &embedded_elbow) < 0.0);
    }
    #[test]
    fn shoulder_attachment_is_bounded_and_continuous_across_the_torso_cap() {
        let mut profile = RigProfile::primitive();
        profile.body_geometry = Some(crate::rig::BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            sample_count: 100,
            torso: crate::rig::TorsoGeometry {
                center: [0., 1., 0.],
                half_extents: [0.15, 0.2, 0.12],
            },
            arm_radii: [[0.015, 0.02]; 2],
            hand_radii: [0.025; 2],
            leg_radii: None,
            foot_radii: None,
            leg_boxes: None,
            leg_rest_frames: None,
            foot_boxes: None,
            hand_centers: None,
            hand_capsules: None,
        });
        let hand = HandSolution {
            middle: [-0.28, 1.05, 0.05],
            end: [-0.25, 1.05, 0.35],
            tip: [-0.22, 1.05, 0.65],
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        for y in [1.199, 1.214, 1.216, 1.225] {
            profile.roots[0] = [-0.10, y, 0.];
            profile.roots[1] = [0.10, y, 0.];
            assert!(
                hand_clearance(&profile, 0, &hand) > 0.002,
                "attachment at {y}"
            );
            let crossed = HandSolution {
                middle: [0.28, 1.05, 0.05],
                ..hand
            };
            assert!(hand_clearance(&profile, 0, &crossed) < 0., "crossed at {y}");
            let embedded = HandSolution {
                middle: [-0.12, 1.05, 0.05],
                ..hand
            };
            assert!(
                hand_clearance(&profile, 0, &embedded) < 0.,
                "distal upper arm at {y}"
            );
        }
        profile.roots[0] = [-0.10, 1.24, 0.];
        profile.roots[1] = [0.10, 1.24, 0.];
        assert!(
            hand_clearance(&profile, 0, &hand) < 0.,
            "a detached joint does not inherit the proximal exemption"
        );
    }
    #[test]
    fn shoulder_above_but_outside_cap_cannot_traverse_the_torso() {
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0., 1., 0.]);
        profile.body_geometry = Some(crate::rig::BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            sample_count: 100,
            torso: crate::rig::TorsoGeometry {
                center: [0., 1., 0.],
                half_extents: [0.15, 0.2, 0.12],
            },
            arm_radii: [[0.015, 0.02]; 2],
            hand_radii: [0.025; 2],
            leg_radii: None,
            foot_radii: None,
            leg_boxes: None,
            leg_rest_frames: None,
            foot_boxes: None,
            hand_centers: None,
            hand_capsules: None,
        });
        // Both rear-to-front and front-to-rear upper arms remain on their own
        // lateral side, but cross the torso from outside the cap footprint.
        for direction in [-1., 1.] {
            profile.roots[0] = [-0.08, 1.225, -0.13 * direction];
            profile.roots[1] = [0.08, 1.225, -0.13 * direction];
            let hand = HandSolution {
                middle: [-0.10, 1.05, 0.18 * direction],
                end: [-0.12, 1.05, 0.45 * direction],
                tip: [-0.12, 1.05, 0.72 * direction],
                rotation: [0., 0., 0., 1.],
                reachable: true,
            };
            for yaw in [0., 0.6, -0.6] {
                let turned = profile.turned(yaw);
                let q = [0., (yaw * 0.5).sin(), 0., (yaw * 0.5).cos()];
                let pivot = profile.torso_pivot.unwrap();
                let turn = |v| add(pivot, crate::rig::rotate(q, sub(v, pivot)));
                let posed = HandSolution {
                    middle: turn(hand.middle),
                    end: turn(hand.end),
                    tip: turn(hand.tip),
                    rotation: q,
                    ..hand
                };
                assert!(torso_clearance(&turned, turned.roots[0], turned.roots[0], 0.015) > 0.);
                assert!(torso_clearance(&turned, turned.roots[0], posed.middle, 0.015) < -0.06);
                assert!(
                    hand_clearance(&turned, 0, &posed) < -0.06,
                    "outside cap, direction {direction}, yaw {yaw}"
                );
            }
        }
    }
    #[test]
    fn opposite_stick_arm_collision_is_not_hidden_by_stick_separation() {
        let profile = RigProfile::primitive();
        let hand = |middle, end, tip| HandSolution {
            middle,
            end,
            tip,
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        let left = hand([-0.4, 1., 0.], [-0.2, 1., 0.], [0.4, 1., 0.]);
        let right = hand([0.2, 1., -0.3], [0.2, 1., 0.3], [0.2, 1., 0.6]);
        let pair = pair_clearance(&profile, &left, &right);
        assert!(pair.sticks > 0.0);
        assert!(pair.cross < 0.0);
    }
}
