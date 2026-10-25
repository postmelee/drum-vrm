//! Explicit whole-kit proposals. No avatar names, source event edits, bone
//! scaling, or automatic changes when importing a character.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::rig::RigProfile;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitFitProposal {
    pub kit: Vec<KitPiece>,
    pub scale: f64,
    pub translation: Vec3,
    pub unreachable_piece_ids: Vec<String>,
    pub complete: bool,
}
fn hand_piece(piece: &KitPiece) -> bool {
    !matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat)
}
fn missed(profile: &RigProfile, kit: &[KitPiece]) -> Vec<String> {
    kit.iter()
        .filter(|piece| {
            let hands: &[usize] = match piece.instrument {
                Instrument::Snare => &[0, 1],
                Instrument::Hihat
                | Instrument::OpenHat
                | Instrument::Ride
                | Instrument::TomMid
                | Instrument::TomLow => &[1],
                Instrument::Crash if piece.id == "crashRight" || piece.position[0] > 0.0 => &[1],
                Instrument::Crash | Instrument::TomHigh => &[0],
                _ => &[],
            };
            hands
                .iter()
                .any(|&limb| !profile.reachable(limb, piece.position))
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
    let unreachable_piece_ids = missed(profile, &kit);
    KitFitProposal {
        complete: unreachable_piece_ids.is_empty(),
        kit,
        scale,
        translation,
        unreachable_piece_ids,
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
/// `complete` means the preferred hands reach the static contact points. It does
/// not certify transfer speed, joint comfort, or clearance of skinned meshes.
pub fn fit_kit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    profile.validate()?;
    if !valid(kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    let mut best = proposal(profile, kit.to_vec(), 1.0, [0.0; 3]);
    if best.complete {
        return Ok(best);
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
                    if completed
                        .as_ref()
                        .is_none_or(|old| shift_cost(translation) < shift_cost(old.translation))
                    {
                        completed = Some(candidate);
                    }
                } else if candidate.unreachable_piece_ids.len() < best.unreachable_piece_ids.len()
                    || candidate.unreachable_piece_ids.len() == best.unreachable_piece_ids.len()
                        && (candidate.scale > best.scale
                            || candidate.scale == best.scale
                                && shift_cost(translation) < shift_cost(best.translation))
                {
                    best = candidate;
                }
            }
        }
        if let Some(completed) = completed {
            return Ok(completed);
        }
    }
    Ok(best)
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
                } else {
                    assert_eq!(
                        serde_json::to_string(old).unwrap(),
                        serde_json::to_string(new).unwrap()
                    );
                }
            }
            let repeated = fit_kit(&profile, &fitted.kit).unwrap();
            assert_eq!(repeated.scale, 1.0);
            assert_eq!(repeated.translation, [0.0; 3]);
            assert_eq!(
                serde_json::to_string(&repeated.kit).unwrap(),
                serde_json::to_string(&fitted.kit).unwrap()
            );
        }
    }
    #[test]
    fn incomplete_fit_remains_explicit_and_invalid_layouts_fail() {
        let mut profile = profile(0.65);
        profile.roots[1][0] = 3.0;
        profile.rest_tips[1][0] += 3.0 - 0.19;
        let fitted = fit_kit(&profile, &default_kit()).unwrap();
        assert!(!fitted.complete);
        assert!(!fitted.unreachable_piece_ids.is_empty());
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
