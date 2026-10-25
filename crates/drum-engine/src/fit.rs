//! Explicit whole-kit proposals. No avatar names, source event edits, bone
//! scaling, or automatic changes when importing a character.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::rig::RigProfile;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceAdjustment {
    pub piece_id: String,
    pub translation: Vec3,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitFitProposal {
    pub kit: Vec<KitPiece>,
    pub scale: f64,
    pub translation: Vec3,
    pub unreachable_piece_ids: Vec<String>,
    pub complete: bool,
    pub piece_adjustments: Vec<PieceAdjustment>,
}
fn hand_piece(piece: &KitPiece) -> bool {
    !matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat)
}
fn hand_posture_fits(profile: &RigProfile, limb: usize, target: Vec3) -> bool {
    if !profile.reachable(limb, target) {
        return false;
    }
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    // Check the contact and a bounded preparation envelope. Annulus reach alone
    // can put a near target behind a folded forearm/tool link. These common
    // skeletal bounds are a posture guard, not a skinned-mesh collision test.
    [(0.0, 0.0), (0.055, -0.2), (0.11, -0.42)]
        .iter()
        .all(|&(lift, pitch)| {
            let raised = [target[0], target[1] + lift, target[2]];
            let Some(solution) = profile.solve_hand(limb, raised, pitch) else {
                return true;
            };
            solution.reachable
                && (raised[1] > root[1] || solution.middle[1] <= root[1] + upper * 0.1)
                && solution.middle[2] >= root[2] - upper * 0.35
                && solution.end[2] >= root[2] + lower * 0.15
        })
}
fn preferred_hands(piece: &KitPiece) -> &'static [usize] {
    match piece.instrument {
        Instrument::Snare => &[0, 1],
        Instrument::Hihat
        | Instrument::OpenHat
        | Instrument::Ride
        | Instrument::TomMid
        | Instrument::TomLow => &[1],
        Instrument::Crash if piece.id == "crashRight" || piece.position[0] > 0.0 => &[1],
        Instrument::Crash | Instrument::TomHigh => &[0],
        _ => &[],
    }
}
fn missed(profile: &RigProfile, kit: &[KitPiece], posture: bool) -> Vec<String> {
    kit.iter()
        .filter(|piece| {
            preferred_hands(piece).iter().any(|&limb| {
                if posture {
                    !hand_posture_fits(profile, limb, piece.position)
                } else {
                    !profile.reachable(limb, piece.position)
                }
            })
        })
        .map(|piece| piece.id.clone())
        .collect()
}
fn valid(kit: &[KitPiece]) -> bool {
    let mut ids = HashSet::new();
    let closed = kit.iter().find(|piece| piece.id == "hihat");
    let open = kit.iter().find(|piece| piece.id == "openHat");
    if let (Some(closed), Some(open)) = (closed, open) {
        if closed.position != open.position {
            return false;
        }
    }
    !kit.is_empty()
        && kit.len() <= 128
        && kit.iter().all(|piece| {
            let [x, y, z] = piece.position;
            !piece.id.is_empty()
                && piece.id.len() <= 128
                && ids.insert(piece.id.as_str())
                && piece.position.iter().all(|v| v.is_finite())
                && x.abs() <= 2.0
                && (0.02..=2.5).contains(&y)
                && (-0.5..=2.5).contains(&z)
                && piece.radius.is_finite()
                && piece.radius > 0.0
                && piece.radius <= 1.0
        })
}
fn proposal(
    profile: &RigProfile,
    kit: Vec<KitPiece>,
    scale: f64,
    translation: Vec3,
) -> KitFitProposal {
    let unreachable_piece_ids = missed(profile, &kit, false);
    KitFitProposal {
        complete: unreachable_piece_ids.is_empty(),
        kit,
        scale,
        translation,
        unreachable_piece_ids,
        piece_adjustments: Vec::new(),
    }
}
fn transform(kit: &[KitPiece], scale: f64, translation: Vec3) -> Vec<KitPiece> {
    // Stable, documented kit origin; uniform scaling preserves relative surface
    // order and distances. The kick body and both pedal contacts stay fixed.
    const ORIGIN: Vec3 = [0.0, 1.235, 0.06];
    kit.iter()
        .map(|piece| {
            let mut next = piece.clone();
            if hand_piece(piece) {
                next.position = std::array::from_fn(|i| {
                    ORIGIN[i] + (piece.position[i] - ORIGIN[i]) * scale + translation[i]
                });
                next.radius *= scale;
            }
            next
        })
        .collect()
}
fn shift_cost(translation: Vec3) -> f64 {
    translation[1].powi(2) + translation[2].powi(2)
}
/// Prefer the largest kit, then the smallest common translation, with a bounded
/// search. A completed proposal is idempotent; incomplete results stay explicit.
/// `complete` means preferred hands pass contact and preparation posture guards.
/// It does not certify transfer speed or clearance of skinned meshes.
fn uniform_fit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    profile.validate()?;
    if !valid(kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    let original = proposal(profile, kit.to_vec(), 1.0, [0.0; 3]);
    if original.complete {
        let repaired = repair_posture(profile, original.clone());
        if repaired.complete {
            return Ok(repaired);
        }
    }
    let height_delta = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 1.235;
    let mut heights = vec![0.0];
    for step in -4..=4 {
        heights.push(height_delta + step as f64 * 0.03);
    }
    heights.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
    heights.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    for step in 0..=20 {
        let scale = 1.0 - step as f64 * 0.02;
        let mut completed: Option<KitFitProposal> = None;
        for &height in &heights {
            // Keep even an extreme custom body from requesting an unbounded shift.
            if height.abs() > 0.5 {
                continue;
            }
            for forward in -10..=4 {
                let translation = [0.0, height, forward as f64 * 0.025];
                let adjusted = transform(kit, scale, translation);
                if !valid(&adjusted) {
                    continue;
                }
                let candidate = proposal(profile, adjusted, scale, translation);
                if candidate.complete {
                    let candidate = repair_posture(profile, candidate);
                    if !candidate.complete {
                        continue;
                    }
                    let cost = |proposal: &KitFitProposal| {
                        shift_cost(proposal.translation) * kit.len() as f64
                            + proposal
                                .piece_adjustments
                                .iter()
                                .map(|p| shift_cost(p.translation))
                                .sum::<f64>()
                    };
                    if completed
                        .as_ref()
                        .is_none_or(|old| cost(&candidate) < cost(old))
                    {
                        completed = Some(candidate);
                    }
                }
            }
        }
        if let Some(completed) = completed {
            return Ok(completed);
        }
    }
    let mut unchanged = original;
    unchanged.unreachable_piece_ids = missed(profile, kit, true);
    unchanged.complete = unchanged.unreachable_piece_ids.is_empty();
    Ok(unchanged)
}
/// First retain the original bounded uniform reach fit, then repair only
/// surfaces that fail the common posture guard. No partial unsafe proposal is
/// applied: failure returns the original layout with explicit offending IDs.
fn repair_posture(profile: &RigProfile, mut result: KitFitProposal) -> KitFitProposal {
    for index in 0..result.kit.len() {
        let piece = &result.kit[index];
        if preferred_hands(piece)
            .iter()
            .all(|&limb| hand_posture_fits(profile, limb, piece.position))
        {
            continue;
        }
        let coupled_hat = matches!(piece.instrument, Instrument::Hihat | Instrument::OpenHat);
        let indices: Vec<usize> = result
            .kit
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                (i == index
                    || coupled_hat
                        && matches!(p.instrument, Instrument::Hihat | Instrument::OpenHat))
                .then_some(i)
            })
            .collect();
        let mut best: Option<Vec3> = None;
        for forward in 0..=12 {
            for lower in 0..=10 {
                let shift = [0.0, -(lower as f64) * 0.025, forward as f64 * 0.025];
                let acceptable = indices.iter().all(|&i| {
                    let p = &result.kit[i];
                    let target = std::array::from_fn(|j| p.position[j] + shift[j]);
                    target[1] >= 0.02
                        && target[2] <= 2.5
                        && preferred_hands(p)
                            .iter()
                            .all(|&limb| hand_posture_fits(profile, limb, target))
                });
                if acceptable
                    && best.is_none_or(|previous| shift_cost(shift) < shift_cost(previous))
                {
                    best = Some(shift);
                }
            }
        }
        if let Some(shift) = best {
            for i in indices {
                let p = &mut result.kit[i];
                p.position = std::array::from_fn(|j| p.position[j] + shift[j]);
                result.piece_adjustments.push(PieceAdjustment {
                    piece_id: p.id.clone(),
                    translation: shift,
                });
            }
        }
    }
    result.unreachable_piece_ids = missed(profile, &result.kit, true);
    result.complete = result.unreachable_piece_ids.is_empty();
    result
}

pub fn fit_kit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    uniform_fit(profile, kit)
}

pub fn fit_kit_json(profile_json: &str, kit_json: &str) -> Result<String, String> {
    if kit_json.len() > 128 * 1024 {
        return Err("키트 배치 크기가 한계를 초과합니다.".into());
    }
    let profile = RigProfile::from_json(profile_json)?;
    // KitPiece's legacy deserializer fills missing voice fields by physical id.
    let kit: Vec<KitPiece> =
        serde_json::from_str(kit_json).map_err(|e| format!("키트 배치 형식 오류: {e}"))?;
    serde_json::to_string(&fit_kit(&profile, &kit)?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::default_kit;
    use crate::rig::HandControlFrame;
    fn profile(scale: f64) -> RigProfile {
        let mut p = RigProfile::primitive();
        let y = 1.235 - (1.0 - scale) * 0.25;
        for side in 0..2 {
            let sign = if side == 0 { -1.0 } else { 1.0 };
            p.roots[side] = [sign * 0.19, y, 0.06];
            p.lengths[side] = [0.3 * scale, 0.27 * scale];
            let elbow = [
                p.roots[side][0] + sign * 0.24 * scale,
                y - 0.18 * scale,
                0.06,
            ];
            let wrist = [elbow[0], elbow[1], elbow[2] + 0.27 * scale];
            p.rest_tips[side] =
                crate::motion::add(wrist, p.offset(side, p.contact_rotations[side]));
        }
        let frame = HandControlFrame {
            forearm: [0.0, 0.0, 1.0],
            palm: [0.0, 1.0, 0.0],
            desired_palm: [0.0, 1.0, 0.0],
        };
        p.hand_frames = Some([frame.clone(), frame]);
        p
    }
    #[test]
    fn bounded_common_fit_keeps_pedals_unchanged_and_is_idempotent() {
        for scale in [0.65, 0.8, 1.0] {
            let profile = profile(scale);
            let original = default_kit();
            let before = serde_json::to_string(&original).unwrap();
            let fitted = fit_kit(&profile, &original).unwrap();
            assert!(
                fitted.complete,
                "{scale} {:?}",
                fitted.unreachable_piece_ids
            );
            assert!((0.6..=1.0).contains(&fitted.scale));
            assert!((-0.25..=0.10).contains(&fitted.translation[2]));
            assert_eq!(before, serde_json::to_string(&original).unwrap());
            for (old, new) in original.iter().zip(&fitted.kit) {
                if hand_piece(old) {
                    assert!((new.radius - old.radius * fitted.scale).abs() < 1e-12);
                    let uniform =
                        transform(std::slice::from_ref(old), fitted.scale, fitted.translation);
                    let shift = fitted
                        .piece_adjustments
                        .iter()
                        .find(|change| change.piece_id == old.id)
                        .map_or([0.0; 3], |change| change.translation);
                    assert_eq!(shift[0], 0.0);
                    assert!((-0.25..=0.0).contains(&shift[1]));
                    assert!((0.0..=0.3 + 1e-12).contains(&shift[2]));
                    for (axis, offset) in shift.iter().enumerate() {
                        assert!(
                            (new.position[axis] - uniform[0].position[axis] - offset).abs() < 1e-10
                        );
                    }
                } else {
                    assert_eq!(
                        serde_json::to_string(old).unwrap(),
                        serde_json::to_string(new).unwrap()
                    );
                }
            }
            let snare = fitted.kit.iter().find(|piece| piece.id == "snare").unwrap();
            for limb in 0..2 {
                for (lift, pitch) in [(0.0, 0.0), (0.055, -0.2), (0.11, -0.42)] {
                    let target = [
                        snare.position[0],
                        snare.position[1] + lift,
                        snare.position[2],
                    ];
                    let solved = profile.solve_hand(limb, target, pitch).unwrap();
                    assert!(solved.reachable);
                    if target[1] <= profile.roots[limb][1] {
                        assert!(
                            solved.middle[1]
                                <= profile.roots[limb][1] + profile.lengths[limb][0] * 0.1 + 1e-9
                        );
                    }
                    assert!(
                        solved.end[2]
                            >= profile.roots[limb][2] + profile.lengths[limb][1] * 0.15 - 1e-9
                    );
                    assert!(crate::motion::distance(solved.tip, target) < 1e-9);
                }
            }
            let repeated = fit_kit(&profile, &fitted.kit).unwrap();
            assert_eq!(repeated.scale, 1.0);
            assert_eq!(repeated.translation, [0.0; 3]);
            assert!(repeated.piece_adjustments.is_empty());
            assert_eq!(
                serde_json::to_string(&repeated.kit).unwrap(),
                serde_json::to_string(&fitted.kit).unwrap()
            );
        }
    }
    #[test]
    fn annulus_reach_is_not_accepted_as_a_safe_near_snare_posture() {
        let profile = profile(1.0);
        let target = [-0.18, 1.05, 0.35];
        assert!(profile.reachable(0, target));
        assert!(!hand_posture_fits(&profile, 0, target));
        let mut kit = default_kit();
        kit.iter_mut()
            .find(|piece| piece.id == "snare")
            .unwrap()
            .position = target;
        let result = fit_kit(&profile, &kit).unwrap();
        assert!(result.complete);
        assert!(result
            .piece_adjustments
            .iter()
            .any(|change| change.piece_id == "snare"));
        let again = fit_kit(&profile, &result.kit).unwrap();
        assert_eq!(
            serde_json::to_string(&again.kit).unwrap(),
            serde_json::to_string(&result.kit).unwrap()
        );
        assert!(again.piece_adjustments.is_empty());
        assert_eq!(again.scale, 1.0);
        assert_eq!(again.translation, [0.0; 3]);
    }
    #[test]
    fn incomplete_fit_remains_explicit_and_invalid_layouts_fail() {
        let mut profile = profile(0.65);
        profile.roots[1][0] = 3.0;
        profile.rest_tips[1][0] += 3.0 - 0.19;
        let fitted = fit_kit(&profile, &default_kit()).unwrap();
        assert!(!fitted.complete);
        assert!(!fitted.unreachable_piece_ids.is_empty());
        assert_eq!(
            serde_json::to_string(&fitted.kit).unwrap(),
            serde_json::to_string(&default_kit()).unwrap()
        );
        assert!(fitted.piece_adjustments.is_empty());
        assert!(fit_kit(&profile, &[]).is_err());
        let mut invalid = default_kit();
        invalid[0].position[0] = f64::NAN;
        assert!(fit_kit(&profile, &invalid).is_err());
        let mut uncoupled = default_kit();
        uncoupled
            .iter_mut()
            .find(|piece| piece.id == "openHat")
            .unwrap()
            .position[2] += 0.1;
        assert!(fit_kit(&profile, &uncoupled).is_err());
    }
}
