//! Body-first whole-kit proposals shared by manual and automatic browser fitting.
//! No avatar names, source event edits or bone scaling.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::rig::RigProfile;
use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

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
    pub search_candidates: usize,
    pub search_limit_reached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<crate::posture::Capabilities>,
}
/// Exact candidate memoization is local to one immutable profile/fit request.
/// It changes neither arithmetic nor search ordering. Bounded storage prevents
/// pathological custom kits from turning this optimization into unbounded work.
pub const MAX_FIT_LAYOUT_CANDIDATES: usize = 80_000;
#[derive(Default)]
struct FitCache {
    candidates: Cell<usize>,
    exhausted: Cell<bool>,
    hands: RefCell<HashMap<(usize, [u64; 3]), bool>>,
    pairs: RefCell<HashMap<[u64; 6], bool>>,
}
impl FitCache {
    const LIMIT: usize = 100_000;
    fn candidate(&self) -> bool {
        if self.candidates.get() >= MAX_FIT_LAYOUT_CANDIDATES {
            self.exhausted.set(true);
            return false;
        }
        self.candidates.set(self.candidates.get() + 1);
        true
    }
    fn finish(&self, mut result: KitFitProposal) -> KitFitProposal {
        result.search_candidates = self.candidates.get();
        result.search_limit_reached = self.exhausted.get();
        result
    }
    fn hand(&self, profile: &RigProfile, limb: usize, target: Vec3) -> bool {
        let key = (limb, target.map(f64::to_bits));
        if let Some(&result) = self.hands.borrow().get(&key) {
            return result;
        }
        let result = hand_posture_fits(profile, limb, target);
        if self.hands.borrow().len() < Self::LIMIT {
            self.hands.borrow_mut().insert(key, result);
        }
        result
    }
    fn pair(&self, profile: &RigProfile, tips: [Vec3; 2]) -> bool {
        let key = std::array::from_fn(|i| tips[i / 3][i % 3].to_bits());
        if let Some(&result) = self.pairs.borrow().get(&key) {
            return result;
        }
        let result = crate::posture::pair_available(profile, tips, [0.0; 2]);
        if self.pairs.borrow().len() < Self::LIMIT {
            self.pairs.borrow_mut().insert(key, result);
        }
        result
    }
}
fn hand_piece(piece: &KitPiece) -> bool {
    !matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat)
}
fn fit_yaws(profile: &RigProfile) -> &'static [f64] {
    // Same range as phrase-facing. One yaw must support all three probes;
    // target-to-target transitions remain a separate temporal constraint.
    if profile.torso_pivot.is_some() {
        &[0.0, -0.12, 0.12, -0.24, 0.24]
    } else {
        &[0.0]
    }
}
fn hand_posture_fits(profile: &RigProfile, limb: usize, target: Vec3) -> bool {
    if profile.hand_frames.is_none() {
        return profile.reachable(limb, target);
    }
    fit_yaws(profile).iter().any(|&yaw| {
        let turned = profile.turned(yaw);
        [(0.0, 0.0), (0.055, -0.2), (0.11, -0.42)]
            .iter()
            .all(|&(lift, pitch)| {
                let raised = [target[0], target[1] + lift, target[2]];
                crate::equipment_motion::PITCH_OFFSETS.iter().any(|delta| {
                    crate::posture::hand_candidates(&turned, limb, raised, pitch + delta)
                        .next()
                        .is_some()
                })
            })
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
fn default_target(profile: &RigProfile, piece: &KitPiece) -> Vec3 {
    let observer =
        std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
    let articulation = match piece.instrument {
        Instrument::Crash => "crash",
        Instrument::Ride => "outer",
        _ => "center",
    };
    crate::contact::target(piece, articulation, observer)
}
/// Fit all designated hands against the complete shaft and rigid equipment in
/// impact, preparation and recovery probes. Pitch candidates are common numeric
/// alternatives; no source hit time or bone dimension changes.
fn equipment_posture_fits(
    profile: &RigProfile,
    piece: &KitPiece,
    ctx: &crate::equipment_motion::EquipmentContext,
) -> bool {
    let target = default_target(profile, piece);
    preferred_hands(piece).iter().all(|&limb| {
        fit_yaws(profile).iter().any(|&yaw| {
            let turned = profile.turned(yaw);
            [(0.0, 0.0), (0.055, -0.2), (0.11, -0.42)]
                .iter()
                .all(|&(lift, pitch)| {
                    let raised = [target[0], target[1] + lift, target[2]];
                    crate::equipment_motion::PITCH_OFFSETS.iter().any(|&delta| {
                        crate::posture::hand_candidates(&turned, limb, raised, pitch + delta).any(
                            |hand| {
                                crate::equipment_motion::hand_clearance(
                                    &turned,
                                    limb,
                                    &hand,
                                    ctx,
                                    Some(piece.id.as_str()),
                                ) >= crate::posture::MARGIN
                            },
                        )
                    })
                })
        })
    })
}
/// Static equipment, playable head approaches and all supports use the same
/// numeric geometry that the renderer and temporal inspector consume.
fn piece_clear(index: usize, kit: &[KitPiece]) -> bool {
    let piece = &kit[index];
    let Some(body) = crate::equipment::body(piece) else {
        return true;
    };
    if kit.iter().enumerate().any(|(other, p)| {
        other != index
            && crate::equipment::body(p).is_some_and(|obstacle| {
                !crate::equipment::boxes_clear(&body, &obstacle)
                    || !crate::equipment::playable_pair(piece, p)
            })
    }) {
        return false;
    }
    true
}
pub fn kit_clearance_ids(kit: &[KitPiece]) -> Vec<String> {
    crate::equipment::kit_geometry(kit).unresolved_piece_ids
}
/// A comfortable reference is derived from the seated body before placing the
/// snare. It is a soft search center, not a universal anatomical angle.
fn snare_reference(profile: &RigProfile) -> Vec3 {
    let upper = (profile.lengths[0][0] + profile.lengths[1][0]) * 0.5;
    let lower = (profile.lengths[0][1] + profile.lengths[1][1]) * 0.5;
    let tool = (crate::motion::distance(profile.tip_offsets[0], [0.0; 3])
        + crate::motion::distance(profile.tip_offsets[1], [0.0; 3]))
        * 0.5;
    let shoulder_forward = (profile.roots[0][2] + profile.roots[1][2]) * 0.5;
    let body_forward = profile
        .body_geometry
        .as_ref()
        .map_or(shoulder_forward, |b| {
            b.torso.center[2] + b.torso.half_extents[2]
        });
    [
        (profile.roots[0][0] + profile.roots[1][0]) * 0.5 - 0.08 * (upper + lower),
        (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - upper * 0.8 - 0.08,
        (shoulder_forward + lower * 0.8 + tool * 0.75).max(body_forward + lower * 0.5 + tool * 0.6),
    ]
}
fn snare_ready_fits_cached(profile: &RigProfile, piece: &KitPiece, cache: &FitCache) -> bool {
    if piece.instrument != Instrument::Snare || profile.hand_frames.is_none() {
        return true;
    }
    (0..2).all(|limb| {
        let root = profile.roots[limb];
        let upper = profile.lengths[limb][0];
        piece.position[1] <= root[1] - upper * 0.4 && piece.position[1] >= root[1] - upper * 1.6
    }) && cache.pair(profile, crate::posture::ready_pair(profile, piece, piece))
}
#[cfg(test)]
fn snare_ready_fits(profile: &RigProfile, piece: &KitPiece) -> bool {
    snare_ready_fits_cached(profile, piece, &FitCache::default())
}
fn piece_posture_fits(profile: &RigProfile, piece: &KitPiece, cache: &FitCache) -> bool {
    snare_ready_fits_cached(profile, piece, cache)
        && preferred_hands(piece)
            .iter()
            .all(|&limb| cache.hand(profile, limb, default_target(profile, piece)))
}
fn missed(profile: &RigProfile, kit: &[KitPiece], posture: bool) -> Vec<String> {
    let cache = FitCache::default();
    kit.iter()
        .filter(|piece| {
            (posture && !snare_ready_fits_cached(profile, piece, &cache))
                || preferred_hands(piece).iter().any(|&limb| {
                    if posture {
                        !hand_posture_fits(profile, limb, default_target(profile, piece))
                    } else {
                        !profile.reachable(limb, default_target(profile, piece))
                    }
                })
        })
        .map(|piece| piece.id.clone())
        .collect()
}
pub(crate) fn valid(kit: &[KitPiece]) -> bool {
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
                && crate::equipment::valid_piece_geometry(piece)
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
        search_candidates: 0,
        search_limit_reached: false,
        capabilities: None,
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
/// measured/default rigid body proxies and every generated support remain clear,
/// and drumhead approach regions are accessible. It does not certify whole
/// motion paths or skinned meshes; those require separate temporal inspection.
fn uniform_fit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    profile.validate()?;
    if !valid(kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    // Freeze physical dimensions before any radius change. Without a reference
    // descriptor, shrinking radius left legacy depth unchanged in both fitting
    // and rendering. Every candidate now scales the complete rigid body.
    let measured: Vec<_> = kit
        .iter()
        .map(|piece| {
            let mut next = piece.clone();
            next.geometry = Some(crate::equipment::local_geometry(piece));
            next
        })
        .collect();
    let kit = measured.as_slice();
    let cache = FitCache::default();
    let original = proposal(profile, kit.to_vec(), 1.0, [0.0; 3]);
    let repaired = repair_posture(profile, original.clone(), &cache);
    if repaired.complete {
        return Ok(cache.finish(repaired));
    }
    let reference = snare_reference(profile);
    let height_delta = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 1.235;
    let attempt = |scale: f64| -> Option<KitFitProposal> {
        let scaled = transform(kit, scale, [0.0; 3]);
        let snare = scaled.iter().find(|p| p.instrument == Instrument::Snare);
        let dy = snare
            .map_or(height_delta, |p| reference[1] - p.position[1])
            .clamp(-0.5, 0.5);
        let dz = snare
            .map_or(0.0, |p| reference[2] - p.position[2])
            .clamp(-0.25, 0.10);
        // A posture anchor and one untranslated-forward alternative replace the
        // old exhaustive translation product. Failed search remains explicit;
        // this is not a proof that every larger physical kit is impossible.
        let mut translations = vec![[0.0, dy, dz], [0.0, dy, 0.0]];
        translations.dedup();
        for translation in translations {
            if cache.exhausted.get() {
                return None;
            }
            let adjusted = transform(kit, scale, translation);
            if !valid(&adjusted) {
                continue;
            }
            let candidate = repair_posture(
                profile,
                proposal(profile, adjusted, scale, translation),
                &cache,
            );
            if candidate.complete {
                return Some(candidate);
            }
        }
        None
    };
    let mut previous = 1.0;
    for scale in [1.0, 0.92, 0.84, 0.76, 0.68, 0.60] {
        if cache.exhausted.get() {
            break;
        }
        if let Some(mut complete) = attempt(scale) {
            // At most three fine probes recover larger safe dimensions between
            // the preceding coarse failure and the first coarse success.
            let mut finer = previous - 0.02;
            while finer > scale + 1e-9 && !cache.exhausted.get() {
                if let Some(candidate) = attempt(finer) {
                    complete = candidate;
                    break;
                }
                finer -= 0.02;
            }
            return Ok(cache.finish(complete));
        }
        previous = scale;
    }
    let mut unchanged = original;
    unchanged.unreachable_piece_ids = missed(profile, kit, true);
    add_equipment_misses(profile, &mut unchanged);
    unchanged.clearance_piece_ids = kit_clearance_ids(kit);
    unchanged.complete = unchanged.unreachable_piece_ids.is_empty()
        && unchanged.clearance_piece_ids.is_empty()
        && crate::posture::required_pairs(profile, &unchanged.kit).is_empty();
    Ok(cache.finish(unchanged))
}
/// First retain the original bounded uniform reach fit, then repair only
/// surfaces that fail the common posture guard. No partial unsafe proposal is
/// applied: failure returns the original layout with explicit offending IDs.
fn repair_posture(
    profile: &RigProfile,
    result: KitFitProposal,
    cache: &FitCache,
) -> KitFitProposal {
    let first = repair_posture_order(profile, result.clone(), cache, false);
    if first.complete || cache.exhausted.get() {
        return first;
    }
    // A rack-first alternative avoids locking a floor tom into the rack's only
    // usable corridor. Both orders share exactly the same finite work budget.
    repair_posture_order(profile, result, cache, true)
}
fn repair_posture_order(
    profile: &RigProfile,
    mut result: KitFitProposal,
    cache: &FitCache,
    rack_first: bool,
) -> KitFitProposal {
    // Solve the supported ready surface first. A later target repair must keep
    // already placed bodies clear instead of fixing one hand by intersecting a tom.
    let mut order: Vec<usize> = (0..result.kit.len())
        .filter(|&i| hand_piece(&result.kit[i]))
        .collect();
    order.sort_by_key(|&i| match result.kit[i].instrument {
        Instrument::Snare => 0,
        Instrument::Hihat | Instrument::OpenHat => 1,
        Instrument::TomLow => {
            if rack_first {
                4
            } else {
                2
            }
        }
        Instrument::TomHigh => {
            if rack_first {
                2
            } else {
                3
            }
        }
        Instrument::TomMid => {
            if rack_first {
                3
            } else {
                4
            }
        }
        _ => 5,
    });
    let mut locked: Vec<usize> = (0..result.kit.len())
        .filter(|&i| !hand_piece(&result.kit[i]))
        .collect();
    for index in order {
        if !cache.candidate() {
            break;
        }
        let clear_locked = |candidate: &[KitPiece]| {
            let piece = &candidate[index];
            let Some(body) = crate::equipment::body(piece) else {
                return true;
            };
            locked.iter().all(|&i| {
                i == index
                    || crate::equipment::body(&candidate[i]).is_none_or(|obstacle| {
                        crate::equipment::boxes_clear(&body, &obstacle)
                            && crate::equipment::playable_pair(piece, &candidate[i])
                    })
            })
        };
        let against_locked = |candidate: &[KitPiece]| {
            let mut fixed: Vec<KitPiece> = locked
                .iter()
                .filter(|&&i| i != index && candidate[i].instrument != Instrument::OpenHat)
                .map(|&i| candidate[i].clone())
                .collect();
            fixed.push(candidate[index].clone());
            if !piece_clear(fixed.len() - 1, &fixed) {
                return false;
            }
            let geometry = crate::equipment::kit_geometry(&fixed);
            let ctx = crate::equipment_motion::EquipmentContext {
                kit: fixed.clone(),
                geometry,
            };
            ctx.geometry.unresolved_piece_ids.is_empty()
                && equipment_posture_fits(profile, &candidate[index], &ctx)
                && if matches!(
                    candidate[index].instrument,
                    Instrument::Hihat | Instrument::OpenHat
                ) {
                    let snare = fixed.iter().find(|p| p.instrument == Instrument::Snare);
                    profile.hand_frames.is_none()
                        || snare.is_none_or(|snare| {
                            cache.pair(
                                profile,
                                crate::posture::ready_pair(profile, snare, &candidate[index]),
                            ) && crate::posture::pair_solution_with_equipment(
                                profile,
                                crate::posture::ready_pair(profile, snare, &candidate[index]),
                                [0.0; 2],
                                Some(&ctx),
                                [Some(snare.id.as_str()), Some(candidate[index].id.as_str())],
                            )
                            .is_some()
                        })
                } else {
                    true
                }
        };
        if piece_posture_fits(profile, &result.kit[index], cache) && against_locked(&result.kit) {
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
                shift_cost(distance) + 0.15 * shift_cost(shift) + 2.0 * shift[1] * shift[1]
            };
            score(*a).total_cmp(&score(*b))
        });
        let mut placed = false;
        for shift in shifts {
            if !cache.candidate() {
                break;
            }
            let mut candidate = result.kit.clone();
            for &i in &indices {
                candidate[i].position =
                    std::array::from_fn(|axis| candidate[i].position[axis] + shift[axis]);
            }
            if !valid(&candidate)
                || !clear_locked(&candidate)
                || indices
                    .iter()
                    .any(|&i| !piece_posture_fits(profile, &candidate[i], cache))
                || !against_locked(&candidate)
            {
                continue;
            }
            result.kit = candidate;
            placed = true;
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
        // Later greedy placements never move an already locked piece. Exhausting
        // this piece's bounded candidates therefore ends this branch; spending
        // more work on later instruments cannot turn it into a complete fit.
        if !placed {
            break;
        }
        locked.push(index);
    }
    result.unreachable_piece_ids = missed(profile, &result.kit, true);
    add_equipment_misses(profile, &mut result);
    result.clearance_piece_ids = kit_clearance_ids(&result.kit);
    result.complete = result.unreachable_piece_ids.is_empty()
        && result.clearance_piece_ids.is_empty()
        && crate::posture::required_pairs(profile, &result.kit).is_empty();
    result
}

fn add_equipment_misses(profile: &RigProfile, result: &mut KitFitProposal) {
    let ctx = crate::equipment_motion::EquipmentContext {
        kit: result.kit.clone(),
        geometry: crate::equipment::kit_geometry(&result.kit),
    };
    for piece in &result.kit {
        if !equipment_posture_fits(profile, piece, &ctx)
            && !result.unreachable_piece_ids.contains(&piece.id)
        {
            result.unreachable_piece_ids.push(piece.id.clone());
        }
    }
}

pub fn fit_kit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    let mut result = uniform_fit(profile, kit)?;
    result.capabilities = Some(crate::posture::audit(profile, &result.kit));
    Ok(result)
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
