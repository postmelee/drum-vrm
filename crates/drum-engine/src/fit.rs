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
    pub clearance_piece_ids: Vec<String>,
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
                && crate::clearance::hand_clearance(profile, limb, &solution) >= -0.002
        })
}
fn preferred_hands(piece: &KitPiece) -> &'static [usize] {
    match piece.instrument {
        Instrument::Snare
        | Instrument::TomHigh
        | Instrument::TomMid
        | Instrument::TomLow
        | Instrument::Crash => &[0, 1],
        Instrument::Hihat | Instrument::OpenHat | Instrument::Ride => &[1],
        _ => &[],
    }
}
/// Nominal procedural envelopes match `pieceBodyEnvelope` in kit-hardware.ts.
/// Cylinders include rim/lug allowance. The kick uses a conservative circumscribed
/// footprint; cosmetic overrides and skinned avatars are outside this certificate.
#[derive(Clone, Copy)]
struct BodyEnvelope {
    center: Vec3,
    radius: f64,
    min_y: f64,
    max_y: f64,
}
fn body_envelope(piece: &KitPiece) -> Option<BodyEnvelope> {
    use Instrument::*;
    let [x, y, z] = piece.position;
    if matches!(piece.instrument, OpenHat | PedalHat) {
        return None;
    }
    if piece.instrument == Kick {
        return Some(BodyEnvelope {
            center: [x, piece.radius + 0.018, z + 0.34],
            radius: (piece.radius + 0.017).hypot(0.22),
            min_y: 0.0,
            max_y: 2.0 * piece.radius + 0.035,
        });
    }
    let cymbal = matches!(piece.instrument, Hihat | Crash | Ride);
    Some(BodyEnvelope {
        center: [x, y, z + if cymbal { piece.radius * 0.5 } else { 0.0 }],
        radius: piece.radius + if cymbal { 0.001 } else { 0.014 },
        min_y: y - if cymbal {
            if piece.instrument == Hihat {
                0.045
            } else {
                0.02
            }
        } else {
            match piece.instrument {
                Snare => 0.135,
                TomLow => 0.295,
                _ => 0.225,
            }
        },
        max_y: y + if cymbal { 0.05 } else { 0.008 },
    })
}
fn bodies_clear(a: BodyEnvelope, b: BodyEnvelope) -> bool {
    let vertical = a.min_y.max(b.min_y) - a.max_y.min(b.max_y);
    let horizontal =
        (a.center[0] - b.center[0]).hypot(a.center[2] - b.center[2]) - a.radius - b.radius;
    vertical.max(horizontal) >= 0.005 - 1e-9
}
fn piece_clear(index: usize, kit: &[KitPiece]) -> bool {
    let Some(body) = body_envelope(&kit[index]) else {
        return true;
    };
    kit.iter().enumerate().all(|(other, p)| {
        if other == index {
            return true;
        }
        let Some(obstacle) = body_envelope(p) else {
            return true;
        };
        if !bodies_clear(body, obstacle) {
            return false;
        }
        // A hi-hat pull rod cannot use the movable boom of crash/ride stands.
        let straight_clear = |hat: BodyEnvelope, obstacle: BodyEnvelope| {
            obstacle.min_y > hat.center[1] + 0.09
                || (hat.center[0] - obstacle.center[0]).hypot(hat.center[2] - obstacle.center[2])
                    >= obstacle.radius + 0.025
        };
        (kit[index].instrument != Instrument::Hihat || straight_clear(body, obstacle))
            && (p.instrument != Instrument::Hihat || straight_clear(obstacle, body))
    })
}
pub fn kit_clearance_ids(kit: &[KitPiece]) -> Vec<String> {
    (0..kit.len())
        .filter(|&i| !piece_clear(i, kit))
        .map(|i| kit[i].id.clone())
        .collect()
}
/// A comfortable reference is derived from the seated body before placing the
/// snare. It is a soft search center, not a universal anatomical angle.
fn snare_reference(profile: &RigProfile) -> Vec3 {
    let upper = (profile.lengths[0][0] + profile.lengths[1][0]) * 0.5;
    let lower = (profile.lengths[0][1] + profile.lengths[1][1]) * 0.5;
    let tool = (crate::motion::distance(profile.tip_offsets[0], [0.0; 3])
        + crate::motion::distance(profile.tip_offsets[1], [0.0; 3]))
        * 0.5;
    [
        (profile.roots[0][0] + profile.roots[1][0]) * 0.5 - 0.08 * (upper + lower),
        (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - upper * 0.8 - 0.08,
        (profile.roots[0][2] + profile.roots[1][2]) * 0.5 + lower * 0.8 + tool * 0.75,
    ]
}
fn snare_ready_fits(profile: &RigProfile, piece: &KitPiece) -> bool {
    if piece.instrument != Instrument::Snare || profile.hand_frames.is_none() {
        return true;
    }
    (0..2).all(|limb| {
        let root = profile.roots[limb];
        let upper = profile.lengths[limb][0];
        piece.position[1] <= root[1] - upper * 0.4 && piece.position[1] >= root[1] - upper * 1.6
    })
}
fn piece_posture_fits(profile: &RigProfile, piece: &KitPiece) -> bool {
    snare_ready_fits(profile, piece)
        && preferred_hands(piece)
            .iter()
            .all(|&limb| hand_posture_fits(profile, limb, piece.position))
}
fn missed(profile: &RigProfile, kit: &[KitPiece], posture: bool) -> Vec<String> {
    kit.iter()
        .filter(|piece| {
            (posture && !snare_ready_fits(profile, piece))
                || preferred_hands(piece).iter().any(|&limb| {
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
    let clearance_piece_ids = kit_clearance_ids(&kit);
    KitFitProposal {
        complete: unreachable_piece_ids.is_empty(),
        kit,
        scale,
        translation,
        unreachable_piece_ids,
        clearance_piece_ids,
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
    translation.iter().map(|v| v * v).sum()
}
/// Prefer the largest kit, then the smallest common translation, with a bounded
/// search. A completed proposal is idempotent; incomplete results stay explicit.
/// `complete` means designated hands pass sampled contact/preparation guards,
/// nominal bodies remain separated and the hi-hat pull rod clears those bodies.
/// It does not certify whole motion paths, cosmetic variants or skinned meshes.
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
    let reference = snare_reference(profile);
    let snare = kit
        .iter()
        .find(|piece| piece.instrument == Instrument::Snare);
    let snare_height = snare.map_or(0.0, |piece| reference[1] - piece.position[1]);
    let mut heights = vec![0.0, snare_height.clamp(-0.5, 0.5)];
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
    unchanged.clearance_piece_ids = kit_clearance_ids(kit);
    unchanged.complete =
        unchanged.unreachable_piece_ids.is_empty() && unchanged.clearance_piece_ids.is_empty();
    Ok(unchanged)
}
/// First retain the original bounded uniform reach fit, then repair only
/// surfaces that fail the common posture guard. No partial unsafe proposal is
/// applied: failure returns the original layout with explicit offending IDs.
fn repair_posture(profile: &RigProfile, mut result: KitFitProposal) -> KitFitProposal {
    // Solve the supported ready surface first. A later target repair must keep
    // already placed bodies clear instead of fixing one hand by intersecting a tom.
    let mut order: Vec<usize> = (0..result.kit.len())
        .filter(|&i| hand_piece(&result.kit[i]))
        .collect();
    order.sort_by_key(|&i| match result.kit[i].instrument {
        Instrument::Snare => 0,
        Instrument::Hihat | Instrument::OpenHat => 1,
        Instrument::TomLow => 2,
        Instrument::TomHigh => 3,
        Instrument::TomMid => 4,
        _ => 5,
    });
    let mut locked: Vec<usize> = (0..result.kit.len())
        .filter(|&i| !hand_piece(&result.kit[i]))
        .collect();
    for index in order {
        let against_locked = |candidate: &[KitPiece]| {
            let mut fixed: Vec<KitPiece> = locked
                .iter()
                .filter(|&&i| i != index && candidate[i].instrument != Instrument::OpenHat)
                .map(|&i| candidate[i].clone())
                .collect();
            fixed.push(candidate[index].clone());
            piece_clear(fixed.len() - 1, &fixed)
        };
        if piece_posture_fits(profile, &result.kit[index]) && against_locked(&result.kit) {
            locked.push(index);
            continue;
        }
        let coupled_hat = matches!(
            result.kit[index].instrument,
            Instrument::Hihat | Instrument::OpenHat
        );
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
        let original = result.kit[index].position;
        let desired = if result.kit[index].instrument == Instrument::Snare {
            snare_reference(profile)
        } else {
            original
        };
        let mut shifts = Vec::new();
        // Coarse bounded XYZ proposals favor spacing before another uniform size
        // reduction. The body reference is included exactly, not quantized away.
        for lateral in -6..=6 {
            for vertical in -5..=5 {
                for forward in -6..=6 {
                    shifts.push([
                        lateral as f64 * 0.05,
                        vertical as f64 * 0.05,
                        forward as f64 * 0.05,
                    ]);
                }
            }
        }
        let reference_shift: Vec3 = std::array::from_fn(|axis| desired[axis] - original[axis]);
        if reference_shift[0].abs() <= 0.30
            && reference_shift[1].abs() <= 0.25
            && reference_shift[2].abs() <= 0.30
        {
            shifts.push(reference_shift);
        }
        shifts.sort_by(|a, b| {
            let score = |shift: Vec3| {
                let distance =
                    std::array::from_fn(|axis| original[axis] + shift[axis] - desired[axis]);
                shift_cost(distance) + 0.15 * shift_cost(shift)
            };
            score(*a).total_cmp(&score(*b))
        });
        for shift in shifts {
            let mut candidate = result.kit.clone();
            for &i in &indices {
                candidate[i].position =
                    std::array::from_fn(|axis| candidate[i].position[axis] + shift[axis]);
            }
            if !valid(&candidate)
                || !against_locked(&candidate)
                || indices
                    .iter()
                    .any(|&i| !piece_posture_fits(profile, &candidate[i]))
            {
                continue;
            }
            result.kit = candidate;
            for i in &indices {
                if let Some(previous) = result
                    .piece_adjustments
                    .iter_mut()
                    .find(|a| a.piece_id == result.kit[*i].id)
                {
                    previous.translation =
                        std::array::from_fn(|axis| previous.translation[axis] + shift[axis]);
                } else {
                    result.piece_adjustments.push(PieceAdjustment {
                        piece_id: result.kit[*i].id.clone(),
                        translation: shift,
                    });
                }
            }
            break;
        }
        locked.push(index);
    }
    result.unreachable_piece_ids = missed(profile, &result.kit, true);
    result.clearance_piece_ids = kit_clearance_ids(&result.kit);
    result.complete =
        result.unreachable_piece_ids.is_empty() && result.clearance_piece_ids.is_empty();
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
#[path = "fit_tests.rs"]
mod tests;
