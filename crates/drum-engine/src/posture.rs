//! Shared bounded static-posture search. A failed search is not a proof that no
//! physical posture exists; routes and rendered skin require separate evidence.
use crate::{
    clearance,
    model::{KitPiece, Vec3},
    rig::{HandSolution, RigProfile},
};
use serde::Serialize;
pub const MARGIN: f64 = 0.002;
/// Common measured hand-to-forearm swing bound (34.4 degrees).
pub const MAX_WRIST_SWING: f64 = 0.6;
pub const SWIVELS: [f64; 5] = [0.0, -0.4, 0.4, -0.8, 0.8];

/// Common geometric posture measures, independent of the avatar's bone axes.
/// These are animation priors, not clinical limits or proof of skin clearance.
#[derive(Clone, Copy, Debug)]
pub struct ArmPosture {
    /// Zero hangs down; pi/2 is a horizontal upper arm.
    pub elevation: f64,
    /// Interior elbow angle; pi is straight.
    pub elbow: f64,
    pub forearm_slope: f64,
}
pub fn arm_posture(profile: &RigProfile, limb: usize, hand: &HandSolution) -> ArmPosture {
    let upper = crate::motion::sub(hand.middle, profile.roots[limb]);
    let lower = crate::motion::sub(hand.end, hand.middle);
    let a = profile.lengths[limb][0];
    let b = profile.lengths[limb][1];
    ArmPosture {
        elevation: (-upper[1] / a).clamp(-1.0, 1.0).acos(),
        elbow: (-upper.iter().zip(lower).map(|(x, y)| x * y).sum::<f64>() / (a * b))
            .clamp(-1.0, 1.0)
            .acos(),
        forearm_slope: (lower[1] / b).clamp(-1.0, 1.0).asin(),
    }
}
fn elevation_allowance(profile: &RigProfile, limb: usize, target: Vec3) -> f64 {
    // A low snare/hat should not require shrugging. Higher cymbal preparation
    // legitimately raises the arm, with a continuous target-height allowance.
    ((target[1] - profile.roots[limb][1]) / profile.lengths[limb][0] + 0.6).clamp(0.0, 1.0)
}
/// Shared by layout, ready pose, equipment routing, torso and elbow planning.
/// Bounded below 0.4 per arm so a relaxed pose cannot buy a collision clearance.
pub fn comfort_cost(profile: &RigProfile, limb: usize, target: Vec3, hand: &HandSolution) -> f64 {
    let p = arm_posture(profile, limb, hand);
    let allowance = elevation_allowance(profile, limb, target);
    let excess = (p.elevation - (35.0 + 55.0 * allowance).to_radians()).max(0.0);
    let folded = (80_f64.to_radians() - p.elbow).max(0.0);
    let extended = (p.elbow - 140_f64.to_radians()).max(0.0);
    let downward = (-p.forearm_slope - 20_f64.to_radians()).max(0.0);
    (0.22 * excess.powi(2)
        + 0.12 * folded.powi(2)
        + 0.06 * extended.powi(2)
        + 0.06 * downward.powi(2))
    .min(0.39)
}

/// Squared dimensionless deficit of the exact shared hard posture bounds.
/// Zero is feasible; positive values guide bounded repairs without changing any
/// bound. Position deficits use the corresponding measured bone length.
pub fn anatomical_violation(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    hand: &HandSolution,
) -> f64 {
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    let posture = arm_posture(profile, limb, hand);
    let values = [
        profile.wrist_deviation(limb, hand) - MAX_WRIST_SWING,
        posture.elevation - (70.0 + 45.0 * elevation_allowance(profile, limb, target)).to_radians(),
        55_f64.to_radians() - posture.elbow,
        posture.elbow - 165_f64.to_radians(),
        (root[2] - upper * 0.35 - hand.middle[2]) / upper,
        (root[2] + lower * 0.15 - hand.end[2]) / lower,
    ];
    if values.iter().any(|v| !v.is_finite()) {
        return f64::INFINITY;
    }
    values.into_iter().map(|v| v.max(0.).powi(2)).sum::<f64>()
        + if hand.reachable { 0. } else { 1. }
}
pub fn anatomical(profile: &RigProfile, limb: usize, target: Vec3, hand: &HandSolution) -> bool {
    anatomical_violation(profile, limb, target, hand) == 0.
}

fn hand_candidate(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    angle: f64,
) -> Option<HandSolution> {
    profile
        .solve_hand_with_swivel(limb, target, pitch, angle)
        .filter(|hand| anatomical(profile, limb, target, hand))
}
/// Lazy enumeration lets feasibility checks stop at their first witness.
pub fn hand_candidates(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
) -> impl Iterator<Item = HandSolution> + '_ {
    SWIVELS
        .iter()
        .filter_map(move |&angle| hand_candidate(profile, limb, target, pitch, angle))
}
/// Refine the existing swivel interval on the temporal solver's 0.05-radian
/// grid only after coarse feasibility fails. No joint limit or margin changes.
pub fn refined_hand_candidates(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
) -> impl Iterator<Item = HandSolution> + '_ {
    (-15_i32..=15)
        .filter(|i| i % 8 != 0)
        .filter_map(move |i| hand_candidate(profile, limb, target, pitch, i as f64 * 0.05))
}
pub fn pair_solution_with_equipment(
    profile: &RigProfile,
    tips: [Vec3; 2],
    pitches: [f64; 2],
    equipment: Option<&crate::equipment_motion::EquipmentContext>,
    intended_ids: [Option<&str>; 2],
) -> Option<[HandSolution; 2]> {
    pair_solution_mode(profile, tips, pitches, equipment, intended_ids, true)
}
pub(crate) fn pair_solution_mode(
    profile: &RigProfile,
    tips: [Vec3; 2],
    pitches: [f64; 2],
    equipment: Option<&crate::equipment_motion::EquipmentContext>,
    intended_ids: [Option<&str>; 2],
    refine: bool,
) -> Option<[HandSolution; 2]> {
    pair_search(profile, tips, pitches, equipment, intended_ids, false).or_else(|| {
        if refine {
            pair_search(profile, tips, pitches, equipment, intended_ids, true)
        } else {
            None
        }
    })
}
fn pair_search(
    profile: &RigProfile,
    tips: [Vec3; 2],
    pitches: [f64; 2],
    equipment: Option<&crate::equipment_motion::EquipmentContext>,
    intended_ids: [Option<&str>; 2],
    refine: bool,
) -> Option<[HandSolution; 2]> {
    let candidates = |limb: usize| {
        let mut choices = Vec::new();
        for delta in crate::equipment_motion::PITCH_OFFSETS {
            for hand in hand_candidates(profile, limb, tips[limb], pitches[limb] + delta).chain(
                refined_hand_candidates(profile, limb, tips[limb], pitches[limb] + delta)
                    .take(if refine { 28 } else { 0 }),
            ) {
                let assessment = equipment.map(|ctx| {
                    crate::equipment_motion::assess_hand(
                        profile,
                        limb,
                        &hand,
                        ctx,
                        intended_ids[limb],
                    )
                });
                if assessment.is_some_and(|value| value.tool < MARGIN) {
                    continue;
                }
                let body = clearance::hand_clearance(profile, limb, &hand)
                    .min(assessment.map_or(f64::INFINITY, |value| value.body));
                let cost = comfort_cost(profile, limb, tips[limb], &hand)
                    + clearance::body_penalty(body)
                    + 0.015 * delta * delta;
                choices.push((cost, hand));
            }
        }
        choices.sort_by(|a, b| a.0.total_cmp(&b.0));
        choices
    };
    let left = candidates(0);
    let right = candidates(1);
    let mut best: Option<(f64, [HandSolution; 2])> = None;
    for &(a, left) in &left {
        if best.as_ref().is_some_and(|(cost, _)| a >= *cost) {
            break;
        }
        for &(b, right) in &right {
            if best.as_ref().is_some_and(|(cost, _)| a + b >= *cost) {
                break;
            }
            if clearance::tool_pair_clearance(profile, &left, &right) >= MARGIN {
                let pair = clearance::pair_clearance(profile, &left, &right);
                let cost = a + b + clearance::body_penalty(pair.arms.min(pair.cross));
                if best.as_ref().is_none_or(|(current, _)| cost < *current) {
                    best = Some((cost, [left, right]));
                }
            }
        }
    }
    best.map(|(_, hands)| hands)
}
/// Ready tips occupy distinct small regions of the same head. This is a fixed
/// calibration probe, not random strike variation or a mutation of MIDI hits.
pub fn ready_pair(profile: &RigProfile, left: &KitPiece, right: &KitPiece) -> [Vec3; 2] {
    let observer =
        std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
    let target = |piece: &KitPiece| {
        let articulation = match piece.instrument {
            crate::model::Instrument::Crash => "crash",
            crate::model::Instrument::Ride => "outer",
            _ => "center",
        };
        crate::contact::target(piece, articulation, observer)
    };
    let mut tips = [target(left), target(right)];
    for tip in &mut tips {
        tip[1] += 0.035;
    }
    if left.id == right.id {
        let separation = (left.radius * 0.18).clamp(0.015, 0.035);
        let axis = crate::motion::sub(profile.roots[1], profile.roots[0]);
        let length = axis[0].hypot(axis[2]).max(1e-9);
        for coordinate in [0, 2] {
            tips[0][coordinate] -= separation * axis[coordinate] / length;
            tips[1][coordinate] += separation * axis[coordinate] / length;
        }
    }
    tips
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairCapability {
    pub left_piece_id: String,
    pub right_piece_id: String,
    pub status: &'static str,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub scope: &'static str,
    pub geometry_source: String,
    pub hand_target_checks: usize,
    pub pair_checks: usize,
    pub available_pairs: usize,
    pub required_pairs_available: bool,
    pub unresolved_required_pairs: Vec<String>,
    pub pairs: Vec<PairCapability>,
}
pub fn required_pairs(profile: &RigProfile, kit: &[KitPiece]) -> Vec<String> {
    if profile.hand_frames.is_none()
        || !kit
            .iter()
            .any(|p| p.instrument == crate::model::Instrument::Snare)
    {
        return Vec::new();
    }
    let ctx = crate::equipment_motion::EquipmentContext::for_kit(kit);
    required_pairs_with_equipment(profile, kit, ctx.as_deref())
}

/// Reuse the immutable geometry already routed for this exact fit candidate.
pub(crate) fn required_pairs_with_equipment(
    profile: &RigProfile,
    kit: &[KitPiece],
    ctx: Option<&crate::equipment_motion::EquipmentContext>,
) -> Vec<String> {
    required_pairs_mode(profile, kit, ctx, true)
}
pub(crate) fn required_pairs_mode(
    profile: &RigProfile,
    kit: &[KitPiece],
    ctx: Option<&crate::equipment_motion::EquipmentContext>,
    refine: bool,
) -> Vec<String> {
    if profile.hand_frames.is_none() {
        return Vec::new();
    }
    let Some(snare) = kit
        .iter()
        .find(|p| p.instrument == crate::model::Instrument::Snare)
    else {
        return Vec::new();
    };
    let mut missing = Vec::new();
    if pair_solution_mode(
        profile,
        ready_pair(profile, snare, snare),
        [0.0; 2],
        ctx,
        [Some(snare.id.as_str()); 2],
        refine,
    )
    .is_none()
    {
        missing.push("snare-ready".into());
    }
    if let Some(hat) = kit
        .iter()
        .find(|p| p.instrument == crate::model::Instrument::Hihat)
    {
        if pair_solution_mode(
            profile,
            ready_pair(profile, snare, hat),
            [0.0; 2],
            ctx,
            [Some(snare.id.as_str()), Some(hat.id.as_str())],
            refine,
        )
        .is_none()
        {
            missing.push("crossed-groove".into());
        }
    }
    let crashes: Vec<_> = kit
        .iter()
        .filter(|p| p.instrument == crate::model::Instrument::Crash)
        .take(2)
        .collect();
    if crashes.len() == 2 {
        let valid = [false, true].iter().any(|&swap| {
            let a = crashes[usize::from(swap)];
            let b = crashes[usize::from(!swap)];
            let ready = ready_pair(profile, a, b);
            // A raised pair alone does not prove that both sticks can contact
            // their surfaces simultaneously. Require the same hand assignment
            // at impact and ready; intermediate trajectories remain a separate
            // runtime check.
            [0.035, 0.0].into_iter().all(|lower| {
                let tips = ready.map(|tip| [tip[0], tip[1] - lower, tip[2]]);
                pair_solution_mode(
                    profile,
                    tips,
                    [0.0; 2],
                    ctx,
                    [Some(a.id.as_str()), Some(b.id.as_str())],
                    refine,
                )
                .is_some()
            })
        });
        if !valid {
            missing.push("dual-crash".into());
        }
    }
    missing
}
pub fn audit(profile: &RigProfile, kit: &[KitPiece]) -> Capabilities {
    let ctx = crate::equipment_motion::EquipmentContext::for_kit(kit);
    let targets: Vec<_> = kit
        .iter()
        .filter(|p| {
            !matches!(
                p.instrument,
                crate::model::Instrument::Kick
                    | crate::model::Instrument::PedalHat
                    | crate::model::Instrument::OpenHat
            )
        })
        .collect();
    let missing = required_pairs(profile, kit);
    let mut pairs = Vec::new();
    // The API accepts up to 128 kit entries. Bound this optional diagnostic work,
    // while canonical kits are fully enumerated. Counts disclose inspected scope.
    for left in targets.iter().take(16) {
        for right in targets.iter().take(16) {
            let tips = ready_pair(profile, left, right);
            let status = if profile.hand_frames.is_none() {
                "unmeasured"
            } else if pair_solution_with_equipment(
                profile,
                tips,
                [0.0; 2],
                ctx.as_deref(),
                [Some(left.id.as_str()), Some(right.id.as_str())],
            )
            .is_some()
            {
                "available"
            } else {
                "not-found"
            };
            pairs.push(PairCapability {
                left_piece_id: left.id.clone(),
                right_piece_id: right.id.clone(),
                status,
            });
        }
    }
    Capabilities {
        scope: "sampled-body-equipment-proxies",
        geometry_source: clearance::geometry_source(profile).into(),
        hand_target_checks: targets.len().min(16) * 2,
        pair_checks: pairs.len(),
        available_pairs: pairs.iter().filter(|p| p.status == "available").count(),
        required_pairs_available: missing.is_empty(),
        unresolved_required_pairs: missing,
        pairs,
    }
}

/// A no-MIDI reference pose uses the exact same dual-snare probe accepted by fit.
/// This is calibration, not a replacement for the source-driven performance.
pub fn ready_pose(profile: &RigProfile, kit: &[KitPiece]) -> Result<Vec<f32>, String> {
    profile.validate()?;
    let snare = kit
        .iter()
        .find(|p| p.instrument == crate::model::Instrument::Snare)
        .ok_or("스네어 기준 타점이 없습니다.")?;
    let ctx = crate::equipment_motion::EquipmentContext::for_kit(kit);
    let hands = pair_solution_with_equipment(
        profile,
        ready_pair(profile, snare, snare),
        [0.0; 2],
        ctx.as_deref(),
        [Some(snare.id.as_str()); 2],
    )
    .ok_or("현재 배치에서 충돌 여유가 있는 양손 준비 자세를 찾지 못했습니다.")?;
    let mut pose = Vec::with_capacity(crate::rig::RIG_POSE_FLOATS);
    let mut rotations = profile.contact_rotations;
    for limb in 0..4 {
        if limb < 2 {
            let hand = hands[limb];
            rotations[limb] = hand.rotation;
            for point in [profile.roots[limb], hand.middle, hand.end, hand.tip] {
                pose.extend(point.map(|v| v as f32));
            }
        } else {
            let id = if limb == 2 { "pedalHat" } else { "kick" };
            if let Some(piece) = kit.iter().find(|p| p.id == id) {
                rotations[limb] = crate::equipment::pedal_rotation(piece, rotations[limb], 0.);
            }
            let tip = kit
                .iter()
                .find(|p| p.id == id)
                .map_or(profile.rest_tips[limb], |p| p.position);
            let offset = profile.offset(limb, rotations[limb]);
            let (middle, end) = crate::motion::two_bone_lengths(
                profile.roots[limb],
                crate::motion::sub(tip, offset),
                profile.bend_poles[limb],
                profile.lengths[limb],
            );
            for point in [
                profile.roots[limb],
                middle,
                end,
                crate::motion::add(end, offset),
            ] {
                pose.extend(point.map(|v| v as f32));
            }
        }
    }
    for point in [profile.pelvis, profile.chest, profile.head] {
        pose.extend(point.map(|v| v as f32));
    }
    for q in rotations {
        pose.extend(q.map(|v| v as f32));
    }
    pose.extend([0.0, 0.0, 0.0, 1.0]);
    Ok(pose)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn measured_clothing_does_not_remove_anatomical_contact_candidates() {
        let mut p = RigProfile::primitive();
        for i in 0..2 {
            p.roots[i] = [if i == 0 { -0.19 } else { 0.19 }, 1.235, 0.06];
            p.lengths[i] = [0.3, 0.27];
            p.grip_offsets[i] = [0.; 3];
            p.tip_offsets[i] = [0., 0., 0.31];
            p.contact_rotations[i] = [0., 0., 0., 1.];
        }
        let frame = crate::rig::HandControlFrame {
            forearm: [0., 0., 1.],
            palm: [0., 1., 0.],
            desired_palm: [0., 1., 0.],
            palm_roll_range: None,
            palm_reference: None,
        };
        p.hand_frames = Some([frame.clone(), frame]);
        let target = [-0.2, 0.95, 0.55];
        let before: Vec<_> = hand_candidates(&p, 0, target, 0.).collect();
        assert!(!before.is_empty());
        p.body_geometry = Some(
            serde_json::from_value(serde_json::json!({
                "version":1,"source":"mesh-estimate","sampleCount":100,
                "torso":{"center":[0.,1.,0.2],"halfExtents":[0.4,0.4,0.4]},
                "armRadii":[[0.2,0.2],[0.2,0.2]],"handRadii":[0.2,0.2]
            }))
            .unwrap(),
        );
        let after: Vec<_> = hand_candidates(&p, 0, target, 0.).collect();
        assert_eq!(after.len(), before.len());
        for (a, b) in after.iter().zip(before) {
            assert_eq!(a.middle, b.middle);
            assert_eq!(a.end, b.end);
            assert!(clearance::hand_clearance(&p, 0, a) < 0.);
            assert!(anatomical(&p, 0, target, a));
            assert!((crate::motion::distance(p.roots[0], a.middle) - p.lengths[0][0]).abs() < 1e-9);
            assert!((crate::motion::distance(a.middle, a.end) - p.lengths[0][1]).abs() < 1e-9);
        }
        assert!(hand_candidates(&p, 0, [-0.2, 0.95, 3.], 0.)
            .next()
            .is_none());
    }

    #[test]
    fn relaxed_low_target_is_preferred_without_banning_high_accents() {
        let mut p = RigProfile::primitive();
        p.roots[0] = [0.0, 1.2, 0.0];
        p.lengths[0] = [0.3, 0.27];
        let low = HandSolution {
            middle: [0.0, 0.9, 0.0],
            end: [0.0, 0.9, 0.27],
            tip: [0.0, 0.9, 0.58],
            rotation: [0.0, 0.0, 0.0, 1.0],
            reachable: true,
        };
        let high = HandSolution {
            middle: [-0.3, 1.2, 0.0],
            end: [-0.3, 0.93, 0.0],
            ..low
        };
        assert!(comfort_cost(&p, 0, low.tip, &low) < comfort_cost(&p, 0, low.tip, &high));
        assert!(anatomical(&p, 0, low.tip, &low));
        assert!(!anatomical(&p, 0, low.tip, &high));
        // A deliberate cymbal accent may elevate the upper arm; the same
        // raised arm must not become the normal low-snare ready posture.
        let forward_high = HandSolution {
            end: [-0.3, 1.2, 0.27],
            ..high
        };
        assert!(anatomical(&p, 0, [0.0, 1.4, 0.58], &forward_high));
    }
    #[test]
    fn comfort_is_dimensionless_and_reflection_invariant() {
        let p = RigProfile::primitive();
        let hand = HandSolution {
            middle: [-0.3, 0.95, 0.1],
            end: [-0.3, 0.95, 0.37],
            tip: [-0.3, 0.95, 0.68],
            rotation: [0.0, 0.0, 0.0, 1.0],
            reachable: true,
        };
        let cost = comfort_cost(&p, 0, hand.tip, &hand);
        let mut scaled = p.clone();
        scaled.roots = p.roots.map(|v| v.map(|x| x * 0.7));
        scaled.lengths = p.lengths.map(|v| v.map(|x| x * 0.7));
        let small = HandSolution {
            middle: hand.middle.map(|x| x * 0.7),
            end: hand.end.map(|x| x * 0.7),
            tip: hand.tip.map(|x| x * 0.7),
            ..hand
        };
        assert!((cost - comfort_cost(&scaled, 0, small.tip, &small)).abs() < 1e-12);
        let mirror = |v: Vec3| [-v[0], v[1], v[2]];
        let reflected = HandSolution {
            middle: mirror(hand.middle),
            end: mirror(hand.end),
            tip: mirror(hand.tip),
            ..hand
        };
        assert!((cost - comfort_cost(&p, 1, reflected.tip, &reflected)).abs() < 1e-12);
    }
}
