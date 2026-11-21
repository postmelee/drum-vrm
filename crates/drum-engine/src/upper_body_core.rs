//! Ordinary body proportions, separate from measured costume envelopes.
//! These conservative numeric cores are not fitted skin surfaces.
use super::{dot, mul, torso_clearance_with_geometry};
use crate::{
    model::Vec3,
    motion::{add, distance, sub},
    rig::{HandSolution, RigProfile, TorsoGeometry},
};
use serde::Serialize;

/// Horizontal shoulder-right and body-forward directions. World translation
/// and torso yaw cannot change the meaning of a forward arm or body clearance.
pub fn body_axes(profile: &RigProfile) -> [Vec3; 2] {
    let axis = sub(profile.roots[1], profile.roots[0]);
    let horizontal = axis[0].hypot(axis[2]);
    let right = if horizontal > 1e-9 {
        [axis[0] / horizontal, 0., axis[2] / horizontal]
    } else {
        [1., 0., 0.]
    };
    [right, [-right[2], 0., right[0]]]
}

/// The same ordinary torso proportions as the existing skeletal fallback,
/// always derived from semantic joints. A broad garment cannot enlarge it.
pub fn core_torso(profile: &RigProfile) -> TorsoGeometry {
    let width = distance(profile.roots[0], profile.roots[1]);
    let low = profile.pelvis[1] + 0.04;
    let high = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 0.025;
    TorsoGeometry {
        center: [
            (profile.roots[0][0] + profile.roots[1][0]) * 0.5,
            (low + high) * 0.5,
            (profile.roots[0][2] + profile.roots[1][2]) * 0.5,
        ],
        half_extents: [
            (width * 0.44).clamp(0.055, 0.30),
            ((high - low) * 0.5).max(0.01),
            (width * 0.29).clamp(0.045, 0.20),
        ],
    }
}

/// Existing ordinary arm proportions, unaffected by sleeve measurements.
pub fn core_arm_radii(profile: &RigProfile, limb: usize) -> [f64; 2] {
    let upper = (profile.lengths[limb][0] * 0.085).clamp(0.015, 0.035);
    [upper, upper * 0.8]
}

pub fn core_torso_clearance(profile: &RigProfile, a: Vec3, b: Vec3, radius: f64) -> f64 {
    torso_clearance_with_geometry(profile, &core_torso(profile), a, b, radius)
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpperBodyCoreClearance {
    pub upper_arm: f64,
    pub forearm: f64,
    pub hand: f64,
    pub tool: f64,
}
impl UpperBodyCoreClearance {
    pub fn minimum(self) -> f64 {
        self.upper_arm
            .min(self.forearm)
            .min(self.hand)
            .min(self.tool)
    }
}

/// Test actual finite arm, grip and complete tool segments against the ordinary
/// torso. A connected upper arm moving radially outward has one proximal
/// attachment interval in the convex torso; its elbow capsule certifies exit.
/// Other directions exclude only a local socket of two upper-arm radii. No
/// inward route gains the outward-arm exemption; the distal arm, forearm, hand
/// and tool remain obstacles.
pub fn upper_body_core(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
) -> UpperBodyCoreClearance {
    let query = CoreQuery::new(profile, limb);
    let gaps = query
        .segments(hand)
        .map(|(a, b, radius)| torso_clearance_with_geometry(profile, &query.torso, a, b, radius));
    UpperBodyCoreClearance {
        upper_arm: gaps[0],
        forearm: gaps[1],
        hand: gaps[2],
        tool: gaps[3],
    }
}

/// The exact negative part of `upper_body_core().minimum()`. Hot selection and
/// path-cost callers need only penetration, so segments with a proof that the
/// existing full query is nonnegative can skip its golden-section refinement.
/// Positive inspection distances still use the original full query above.
pub fn core_penetration(profile: &RigProfile, limb: usize, hand: &HandSolution) -> f64 {
    CoreQuery::new(profile, limb).penetration(hand)
}

/// Profile-owned terms of the ordinary body query, shared by all immutable
/// candidates in one solve. No distance approximation or acceptance is cached.
pub(crate) struct CoreQuery<'a> {
    profile: &'a RigProfile,
    limb: usize,
    torso: TorsoGeometry,
    axes: [Vec3; 2],
    root: Vec3,
    radii: [f64; 2],
    from_center: Vec3,
    socket_radius: f64,
    connected: bool,
}
impl<'a> CoreQuery<'a> {
    pub(crate) fn new(profile: &'a RigProfile, limb: usize) -> Self {
        let torso = core_torso(profile);
        let root = profile.roots[limb];
        let radii = core_arm_radii(profile, limb);
        let axes = body_axes(profile);
        let side = if limb == 0 { -1. } else { 1. };
        let from_center = sub(root, torso.center);
        let own_side = side * dot(from_center, axes[0]) > 0.;
        let socket_radius = 2. * radii[0];
        // A connected shoulder must already meet the torso side/cap. The
        // local socket allowance cannot excuse a detached or inward arm.
        let connected = own_side
            && torso_clearance_with_geometry(profile, &torso, root, root, socket_radius) <= 0.;
        Self {
            profile,
            limb,
            torso,
            axes,
            root,
            radii,
            from_center,
            socket_radius,
            connected,
        }
    }
    pub(crate) fn penetration(&self, hand: &HandSolution) -> f64 {
        self.segments(hand)
            .into_iter()
            .map(|(a, b, radius)| {
                if proven_clear(&self.torso, self.axes, a, b, radius) {
                    0.
                } else {
                    (-torso_clearance_with_geometry(self.profile, &self.torso, a, b, radius))
                        .max(0.)
                }
            })
            .fold(0_f64, f64::max)
    }
    fn segments(&self, hand: &HandSolution) -> [(Vec3, Vec3, f64); 4] {
        let Self {
            profile,
            limb,
            root,
            radii: [upper_radius, lower_radius],
            axes: [right, forward],
            from_center,
            socket_radius,
            connected,
            ..
        } = *self;
        let torso = &self.torso;
        let side = if limb == 0 { -1. } else { 1. };
        let upper = sub(hand.middle, root);
        let radial_derivative = dot(from_center, right) * dot(upper, right)
            / torso.half_extents[0].powi(2)
            + dot(from_center, forward) * dot(upper, forward) / torso.half_extents[2].powi(2);
        let outward = side * dot(upper, right) >= 0.
            && side * dot(sub(hand.middle, torso.center), right) > 0.
            && radial_derivative >= 0.;
        let upper_start = if connected {
            let length = distance(root, hand.middle);
            add(root, mul(upper, (socket_radius / length.max(1e-9)).min(1.)))
        } else {
            root
        };
        // The ordinary hand core ignores measured costume centres/capsules.
        let grip = add(
            hand.end,
            crate::rig::rotate(hand.rotation, profile.grip_offsets[limb]),
        );
        let hand_radius = (profile.lengths[limb][1] * 0.10).clamp(0.015, 0.030);
        let (butt, tip) = crate::equipment_motion::stick_segment(profile, limb, hand);
        let tool_radius = crate::stick_geometry::profile()
            .terminal
            .iter()
            .map(|point| point[1])
            .fold(0_f64, f64::max);
        [
            (
                if connected && outward {
                    hand.middle
                } else {
                    upper_start
                },
                hand.middle,
                upper_radius,
            ),
            (hand.middle, hand.end, lower_radius),
            (hand.end, grip, hand_radius),
            (butt, tip, tool_radius),
        ]
    }
}

fn proven_clear(
    torso: &TorsoGeometry,
    [right, forward]: [Vec3; 2],
    a: Vec3,
    b: Vec3,
    radius: f64,
) -> bool {
    let local = |p| {
        let p = sub(p, torso.center);
        [
            dot(p, right) / torso.half_extents[0],
            p[1],
            dot(p, forward) / torso.half_extents[2],
        ]
    };
    let start = local(a);
    let end = local(b);
    let delta = [end[0] - start[0], end[2] - start[2]];
    let squared = delta[0] * delta[0] + delta[1] * delta[1];
    let t = if squared > 0. {
        (-(start[0] * delta[0] + start[2] * delta[1]) / squared).clamp(0., 1.)
    } else {
        0.
    };
    let radial = (start[0] + t * delta[0]).hypot(start[2] + t * delta[1]);
    let radial_bound = (radial - 1.) * torso.half_extents[0].min(torso.half_extents[2]);
    let vertical_bound = if start[1] * end[1] >= 0. {
        start[1].abs().min(end[1].abs())
    } else {
        0.
    } - torso.half_extents[1];
    // max(min radial, min vertical) <= min(max(radial, vertical)). Also retain
    // the full query's final interval/Lipschitz reserve, not just true geometric
    // separation: a packing-near-boundary candidate must take the exact path.
    let reserve = if a == b {
        0.
    } else {
        super::TORSO_QUERY_RATIO.powi(super::TORSO_QUERY_STEPS)
            * distance(a, b)
            * 0.5
            * (1. + 1e-10)
    };
    let roundoff = 1e-12
        * (1.
            + a.iter()
                .chain(&b)
                .chain(&torso.center)
                .map(|v| v.abs())
                .sum::<f64>());
    radial_bound.max(vertical_bound) > radius + reserve + roundoff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> RigProfile {
        let mut p = RigProfile::primitive();
        p.roots[0] = [-0.18, 1.25, 0.];
        p.roots[1] = [0.18, 1.25, 0.];
        p.lengths[0] = [0.30, 0.27];
        p.lengths[1] = [0.30, 0.27];
        p.pelvis = [0., 0.75, 0.];
        p.torso_pivot = Some([0., 1., 0.]);
        p
    }
    fn hand(middle: Vec3, end: Vec3, tip: Vec3) -> HandSolution {
        HandSolution {
            middle,
            end,
            tip,
            rotation: [0., 0., 0., 1.],
            reachable: true,
        }
    }

    #[test]
    fn broad_phase_preserves_exact_penetration_for_crossing_clear_and_rotated_arms() {
        let mut seed = 0x64f4_23a1_u64;
        let mut random = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (seed >> 11) as f64 / ((1_u64 << 53) as f64)
        };
        let (mut clear, mut crossing) = (0, 0);
        for index in 0..4000 {
            let yaw = (random() - 0.5) * 5.;
            let p = profile().turned(yaw);
            let middle = [random() - 0.5, 0.7 + random() * 0.7, random() - 0.3];
            let end = [random() - 0.5, 0.7 + random() * 0.7, random() - 0.3];
            let h = hand(middle, end, add(end, [random() * 0.1, 0., 0.3]));
            let exact = (-upper_body_core(&p, index % 2, &h).minimum()).max(0.);
            let fast = core_penetration(&p, index % 2, &h);
            assert_eq!(fast, exact, "index {index}");
            if exact > 0. {
                crossing += 1;
            } else {
                clear += 1;
            }
        }
        assert!(clear > 100 && crossing > 100);
    }

    #[test]
    fn broad_phase_keeps_the_full_queries_numerical_boundary_reserve() {
        let p = profile();
        let torso = core_torso(&p);
        let radius = 0.02;
        let x = torso.half_extents[0] * (1. + radius / torso.half_extents[2]);
        let a = add(torso.center, [x, -0.1, 0.]);
        let b = add(torso.center, [x, 0.1, 0.]);
        // The analytic unrefined radial margin is exactly zero. The existing
        // final interval reserve makes the reported gap slightly negative.
        assert!(core_torso_clearance(&p, a, b, radius) < 0.);
        assert!(!proven_clear(&torso, body_axes(&p), a, b, radius));
        let far_a = add(a, [0.1, 0., 0.]);
        let far_b = add(b, [0.1, 0., 0.]);
        assert!(proven_clear(&torso, body_axes(&p), far_a, far_b, radius));
        assert!(core_torso_clearance(&p, far_a, far_b, radius) > 0.);
    }

    #[test]
    fn garment_envelopes_cannot_change_ordinary_core() {
        let mut p = profile();
        let h = hand([-0.30, 1., 0.06], [-0.17, 1., 0.3], [-0.1, 1., 0.6]);
        let before = upper_body_core(&p, 0, &h);
        p.body_geometry = Some(
            serde_json::from_value(serde_json::json!({
                "version":1,"source":"mesh-estimate","sampleCount":500,
                "torso":{"center":[0.,1.,0.],"halfExtents":[0.6,0.6,0.6]},
                "armRadii":[[0.2,0.2],[0.2,0.2]],"handRadii":[0.2,0.2],
                "handCenters":[[0.1,0.2,0.3],[0.1,0.2,0.3]]
            }))
            .unwrap(),
        );
        let after = upper_body_core(&p, 0, &h);
        assert!(before.minimum() > 0.002, "{before:?}");
        assert_eq!(
            serde_json::to_string(&before).unwrap(),
            serde_json::to_string(&after).unwrap()
        );
        assert!(super::super::hand_clearance(&p, 0, &h) < -0.1);
    }

    #[test]
    fn shoulder_socket_does_not_excuse_distal_body_crossings() {
        let p = profile();
        let clear = hand([-0.30, 1., 0.06], [-0.17, 1., 0.3], [-0.1, 1., 0.6]);
        let clear_result = upper_body_core(&p, 0, &clear);
        assert!(clear_result.minimum() > 0.002, "{clear_result:?}");
        let inward = hand([0.20, 1., 0.], [0.22, 1., 0.3], [0.22, 1., 0.6]);
        assert!(upper_body_core(&p, 0, &inward).upper_arm < -0.05);
        let forearm = hand([-0.25, 1., 0.], [0.25, 1., 0.], [0.25, 1., 0.3]);
        assert!(upper_body_core(&p, 0, &forearm).forearm < -0.10);
        let embedded_elbow = hand([-0.1, 1., 0.], [-0.1, 1., 0.3], [-0.1, 1., 0.6]);
        assert!(upper_body_core(&p, 0, &embedded_elbow).upper_arm < -0.02);
    }

    #[test]
    fn attached_outward_arm_uses_convex_exit_instead_of_an_arbitrary_cutoff() {
        let p = profile();
        let h = hand([-0.25, 1., 0.06], [-0.17, 1., 0.3], [-0.1, 1., 0.6]);
        let radius = core_arm_radii(&p, 0)[0];
        let root = p.roots[0];
        let at_two_radii = add(
            root,
            mul(sub(h.middle, root), 2. * radius / distance(root, h.middle)),
        );
        assert!(core_torso_clearance(&p, at_two_radii, h.middle, radius) < 0.);
        assert!(upper_body_core(&p, 0, &h).minimum() > 0.002);
        // Even for an outward attachment the elbow must actually exit.
        let embedded = hand([-0.181, 1., 0.], [-0.17, 1., 0.3], [-0.1, 1., 0.6]);
        assert!(upper_body_core(&p, 0, &embedded).upper_arm < 0.);
    }

    #[test]
    fn ordinary_core_is_invariant_under_body_yaw_and_translation() {
        let p = profile();
        let h = hand([-0.30, 1., 0.06], [-0.17, 1., 0.3], [-0.1, 1., 0.6]);
        let baseline = upper_body_core(&p, 0, &h);
        for yaw in [-1.2_f64, -0.4, 0.4, 1.2] {
            let q = [0., (yaw * 0.5).sin(), 0., (yaw * 0.5).cos()];
            let pivot = p.torso_pivot.unwrap();
            let shift = [2.3, 0.17, -1.7];
            let turn = |v| add(add(pivot, crate::rig::rotate(q, sub(v, pivot))), shift);
            let mut posed = p.turned(yaw);
            for root in &mut posed.roots {
                *root = add(*root, shift);
            }
            posed.pelvis = add(posed.pelvis, shift);
            let hand = HandSolution {
                middle: turn(h.middle),
                end: turn(h.end),
                tip: turn(h.tip),
                rotation: q,
                ..h
            };
            let got = upper_body_core(&posed, 0, &hand);
            for (a, b) in [
                (baseline.upper_arm, got.upper_arm),
                (baseline.forearm, got.forearm),
                (baseline.hand, got.hand),
                (baseline.tool, got.tool),
            ] {
                assert!((a - b).abs() < 1e-10, "{a} vs {b}, yaw {yaw}");
            }
        }
    }
}
