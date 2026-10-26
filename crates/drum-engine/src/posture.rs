//! Shared bounded static-posture search. A failed search is not a proof that no
//! physical posture exists; routes and rendered skin require separate evidence.
use crate::{
    clearance,
    model::{KitPiece, Vec3},
    rig::{HandSolution, RigProfile},
};
use serde::Serialize;
pub const MARGIN: f64 = 0.002;
pub const SWIVELS: [f64; 5] = [0.0, -0.4, 0.4, -0.8, 0.8];

pub fn anatomical(profile: &RigProfile, limb: usize, target: Vec3, hand: &HandSolution) -> bool {
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    hand.reachable
        && (target[1] > root[1] || hand.middle[1] <= root[1] + upper * 0.1)
        && hand.middle[2] >= root[2] - upper * 0.35
        && hand.end[2] >= root[2] + lower * 0.15
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
        .filter(|hand| {
            anatomical(profile, limb, target, hand)
                && clearance::hand_clearance(profile, limb, hand) >= MARGIN
        })
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
pub fn pair_solution(
    profile: &RigProfile,
    tips: [Vec3; 2],
    pitches: [f64; 2],
) -> Option<[HandSolution; 2]> {
    pair_solution_with_equipment(profile, tips, pitches, None, [None; 2])
}
pub fn pair_solution_with_equipment(
    profile: &RigProfile,
    tips: [Vec3; 2],
    pitches: [f64; 2],
    equipment: Option<&crate::equipment_motion::EquipmentContext>,
    intended_ids: [Option<&str>; 2],
) -> Option<[HandSolution; 2]> {
    let mut right_cache: [Option<Option<HandSolution>>; 5] = [None; 5];
    for left in hand_candidates(profile, 0, tips[0], pitches[0]) {
        if equipment.is_some_and(|ctx| {
            crate::equipment_motion::hand_clearance(profile, 0, &left, ctx, intended_ids[0])
                < MARGIN
        }) {
            continue;
        }
        for (index, &angle) in SWIVELS.iter().enumerate() {
            let right = *right_cache[index]
                .get_or_insert_with(|| hand_candidate(profile, 1, tips[1], pitches[1], angle));
            if let Some(right) = right {
                if clearance::pair_clearance(profile, &left, &right).minimum() >= MARGIN
                    && equipment.is_none_or(|ctx| {
                        crate::equipment_motion::hand_clearance(
                            profile,
                            1,
                            &right,
                            ctx,
                            intended_ids[1],
                        ) >= MARGIN
                    })
                {
                    return Some([left, right]);
                }
            }
        }
    }
    None
}
pub fn pair_available(profile: &RigProfile, tips: [Vec3; 2], pitches: [f64; 2]) -> bool {
    pair_solution(profile, tips, pitches).is_some()
}
/// Ready tips occupy distinct small regions of the same head. This is a fixed
/// calibration probe, not random strike variation or a mutation of MIDI hits.
pub fn ready_pair(profile: &RigProfile, left: &KitPiece, right: &KitPiece) -> [Vec3; 2] {
    let observer =
        std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
    let mut tips = [
        crate::contact::target(left, "center", observer),
        crate::contact::target(right, "center", observer),
    ];
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
    let ctx = crate::equipment_motion::EquipmentContext::for_kit(kit);
    if pair_solution_with_equipment(
        profile,
        ready_pair(profile, snare, snare),
        [0.0; 2],
        ctx.as_deref(),
        [Some(snare.id.as_str()); 2],
    )
    .is_none()
    {
        missing.push("snare-ready".into());
    }
    if let Some(hat) = kit
        .iter()
        .find(|p| p.instrument == crate::model::Instrument::Hihat)
    {
        if pair_solution_with_equipment(
            profile,
            ready_pair(profile, snare, hat),
            [0.0; 2],
            ctx.as_deref(),
            [Some(snare.id.as_str()), Some(hat.id.as_str())],
        )
        .is_none()
        {
            missing.push("crossed-groove".into());
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
