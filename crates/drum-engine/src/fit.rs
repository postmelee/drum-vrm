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
pub struct FitFailure {
    pub stage: String,
    pub piece_id: String,
    pub branches: usize,
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
    pub seat_offset: Vec3,
    pub search_failures: Vec<FitFailure>,
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
    failures: RefCell<Vec<FitFailure>>,
    limit: Cell<usize>,
    refine: Cell<bool>,
}
impl FitCache {
    const LIMIT: usize = 100_000;
    fn candidate(&self) -> bool {
        if self.candidates.get()
            >= if self.limit.get() == 0 {
                MAX_FIT_LAYOUT_CANDIDATES
            } else {
                self.limit.get()
            }
        {
            self.exhausted.set(true);
            return false;
        }
        self.candidates.set(self.candidates.get() + 1);
        true
    }
    fn finish(&self, mut result: KitFitProposal) -> KitFitProposal {
        result.search_candidates = self.candidates.get();
        result.search_limit_reached = self.candidates.get() >= MAX_FIT_LAYOUT_CANDIDATES;
        result.search_failures = self.failures.borrow().clone();
        result
    }
    fn fail(&self, stage: &str, piece: &str) {
        let mut failures = self.failures.borrow_mut();
        if let Some(f) = failures
            .iter_mut()
            .find(|f| f.stage == stage && f.piece_id == piece)
        {
            f.branches += 1;
        } else if failures.len() < 32 {
            failures.push(FitFailure {
                stage: stage.into(),
                piece_id: piece.into(),
                branches: 1,
            });
        }
    }
    fn hand(&self, profile: &RigProfile, limb: usize, target: Vec3) -> bool {
        let key = (limb, target.map(f64::to_bits));
        if let Some(&result) = self.hands.borrow().get(&key) {
            return result;
        }
        let result = crate::contact_feasibility::hand_probe_mode(
            profile,
            limb,
            target,
            None,
            None,
            self.refine.get(),
        );
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
        let result = crate::posture::pair_solution_mode(
            profile,
            tips,
            [0.; 2],
            None,
            [None; 2],
            self.refine.get(),
        )
        .is_some();
        if self.pairs.borrow().len() < Self::LIMIT {
            self.pairs.borrow_mut().insert(key, result);
        }
        result
    }
}
fn hand_piece(piece: &KitPiece) -> bool {
    !matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat)
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
fn hands_satisfied(piece: &KitPiece, mut check: impl FnMut(usize) -> bool) -> bool {
    let hands = preferred_hands(piece);
    if piece.instrument == Instrument::Snare {
        hands.iter().all(|&limb| check(limb))
    } else {
        hands.is_empty() || hands.iter().any(|&limb| check(limb))
    }
}
fn piece_targets(profile: &RigProfile, piece: &KitPiece) -> Vec<Vec3> {
    let observer =
        std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
    crate::contact::articulations_for_fit(piece)
        .iter()
        .map(|&articulation| crate::contact::target(piece, articulation, observer))
        .collect()
}
/// Each canonical articulation must have a feasible source-neutral contact and
/// preparation route. The runtime consumes the same numeric probe contract.
fn equipment_posture_fits(
    profile: &RigProfile,
    piece: &KitPiece,
    ctx: &crate::equipment_motion::EquipmentContext,
) -> bool {
    equipment_posture_mode(profile, piece, ctx, true)
}
fn equipment_posture_mode(
    profile: &RigProfile,
    piece: &KitPiece,
    ctx: &crate::equipment_motion::EquipmentContext,
    refine: bool,
) -> bool {
    piece_targets(profile, piece).iter().all(|&target| {
        hands_satisfied(piece, |limb| {
            crate::contact_feasibility::hand_probe_mode(
                profile,
                limb,
                target,
                Some(ctx),
                Some(piece.id.as_str()),
                refine,
            )
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
        (shoulder_forward + lower * 0.95 + tool * 0.95)
            .max(body_forward + lower * 0.5 + tool * 0.6),
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
        && piece_targets(profile, piece)
            .iter()
            .all(|&target| hands_satisfied(piece, |limb| cache.hand(profile, limb, target)))
}
fn missed(profile: &RigProfile, kit: &[KitPiece], posture: bool) -> Vec<String> {
    missed_mode(profile, kit, posture, false)
}
fn missed_mode(profile: &RigProfile, kit: &[KitPiece], posture: bool, refine: bool) -> Vec<String> {
    let cache = FitCache::default();
    cache.refine.set(refine);
    kit.iter()
        .filter(|piece| {
            (posture && !snare_ready_fits_cached(profile, piece, &cache))
                || piece_targets(profile, piece).iter().any(|&target| {
                    !hands_satisfied(piece, |limb| {
                        if posture {
                            cache.hand(profile, limb, target)
                        } else {
                            crate::equipment_motion::PITCH_OFFSETS
                                .iter()
                                .any(|&pitch| profile.reachable_at_pitch(limb, target, pitch))
                        }
                    })
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
        seat_offset: [0.; 3],
        search_failures: Vec::new(),
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
fn uniform_fit(
    profile: &RigProfile,
    kit: &[KitPiece],
    cache: &FitCache,
) -> Result<KitFitProposal, String> {
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
    let original = proposal(profile, kit.to_vec(), 1.0, [0.0; 3]);
    let repaired = repair_posture(profile, original.clone(), cache);
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
                cache,
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
    unchanged.unreachable_piece_ids = missed_mode(profile, kit, true, cache.refine.get());
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
    let first = repair_posture_order(profile, result.clone(), cache, false, None);
    if first.complete || cache.exhausted.get() {
        return first;
    }
    // A rack-first alternative avoids locking a floor tom into the rack's only
    // usable corridor. Both orders share exactly the same finite work budget.
    repair_posture_order(profile, result, cache, true, None)
}
/// Keep the direct hi-hat assembly near the seated left foot while leaving room
/// for the snare. Candidates preserve real leg/foot dimensions and test the full
/// footboard pitch range, rather than detaching the cymbal from a fixed pedal.
fn place_hihat_pedal(profile: &RigProfile, result: &mut KitFitProposal) {
    let Some(hat) = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Hihat)
    else {
        return;
    };
    let Some(snare) = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Snare)
    else {
        return;
    };
    let Some(index) = result
        .kit
        .iter()
        .position(|p| p.instrument == Instrument::PedalHat)
    else {
        return;
    };
    let original = result.kit[index].position;
    let reference = snare_reference(profile);
    let desired = [
        reference[0] - snare.radius - hat.radius - 0.035,
        original[1],
        reference[2] + 0.05 - crate::equipment::HIHAT_PEDAL_TO_AXIS,
    ];
    let candidates = [
        desired,
        [desired[0], original[1], original[2]],
        [original[0], original[1], desired[2]],
        original,
    ];
    for position in candidates {
        if (position[0] - original[0]).abs() > 0.25 || (position[2] - original[2]).abs() > 0.25 {
            continue;
        }
        let mut piece = result.kit[index].clone();
        piece.position = position;
        let feasible = joint::foot_fits(profile, &piece, 2);
        if feasible {
            let shift = std::array::from_fn(|axis| position[axis] - original[axis]);
            if crate::motion::distance(shift, [0.0; 3]) > 1e-9 {
                result.piece_adjustments.push(PieceAdjustment {
                    piece_id: piece.id.clone(),
                    translation: shift,
                });
                result.kit[index] = piece;
            }
            return;
        }
    }
}
fn repair_posture_order(
    profile: &RigProfile,
    mut result: KitFitProposal,
    cache: &FitCache,
    rack_first: bool,
    anchor_rank: Option<usize>,
) -> KitFitProposal {
    if anchor_rank.is_none() {
        place_hihat_pedal(profile, &mut result);
    }
    // A direct hi-hat is one mechanical assembly with its foot contact.
    // Its surface may change height, but fitting cannot detach the mast from
    // its pedal merely to make the right hand reachable.
    if let Some(hat) = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Hihat)
    {
        let anchored = crate::equipment::anchored_hihat_position(hat, &result.kit);
        for piece in result
            .kit
            .iter_mut()
            .filter(|p| matches!(p.instrument, Instrument::Hihat | Instrument::OpenHat))
        {
            let shift = std::array::from_fn(|axis| anchored[axis] - piece.position[axis]);
            if crate::motion::distance(shift, [0.0; 3]) > 1e-9 {
                result.piece_adjustments.push(PieceAdjustment {
                    piece_id: piece.id.clone(),
                    translation: shift,
                });
                piece.position = anchored;
            }
        }
    }
    // The direct hi-hat's X/Z and foot mechanics are already fixed for this
    // branch. Filter its height witnesses once, not for every snare candidate.
    let hat_candidates: Vec<KitPiece> = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Hihat)
        .map_or(Vec::new(), |hat| {
            (0..=10)
                .filter_map(|i| {
                    if !cache.candidate() {
                        return None;
                    }
                    let step = if i % 2 == 0 {
                        i as f64 / 2.0
                    } else {
                        -(i as f64 + 1.0) / 2.0
                    };
                    let mut hat = hat.clone();
                    hat.position[1] += step * 0.05;
                    if !piece_posture_fits(profile, &hat, cache) {
                        return None;
                    }
                    let mut kit: Vec<_> = result
                        .kit
                        .iter()
                        .filter(|p| !hand_piece(p))
                        .cloned()
                        .collect();
                    kit.push(hat.clone());
                    let ctx = crate::equipment_motion::EquipmentContext {
                        geometry: crate::equipment::kit_geometry(&kit),
                        kit,
                    };
                    (ctx.geometry.unresolved_piece_ids.is_empty()
                        && equipment_posture_mode(profile, &hat, &ctx, cache.refine.get()))
                    .then_some(hat)
                })
                .collect()
        });
    if result.kit.iter().any(|p| p.instrument == Instrument::Hihat) && hat_candidates.is_empty() {
        cache.fail("hihat-witness", "hihat");
        result.complete = false;
        return result;
    }
    // Solve the supported ready surface first. A later target repair must keep
    // already placed bodies clear instead of fixing one hand by intersecting a tom.
    let mut order: Vec<usize> = (0..result.kit.len())
        .filter(|&i| hand_piece(&result.kit[i]))
        .collect();
    // Place the right-hand-only ride corridor before instruments that can use
    // either hand. A later crash must not consume the ride's only approach.
    order.sort_by_key(|&i| match result.kit[i].instrument {
        Instrument::Snare => 0,
        Instrument::Hihat | Instrument::OpenHat => 1,
        Instrument::Ride => 2,
        Instrument::TomLow => {
            if rack_first {
                5
            } else {
                3
            }
        }
        Instrument::TomHigh => {
            if rack_first {
                3
            } else {
                4
            }
        }
        Instrument::TomMid => {
            if rack_first {
                4
            } else {
                5
            }
        }
        _ => 6,
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
                && equipment_posture_mode(profile, &candidate[index], &ctx, cache.refine.get())
                && fixed
                    .iter()
                    .filter(|p| p.id != candidate[index].id)
                    .all(|p| equipment_posture_mode(profile, p, &ctx, cache.refine.get()))
                && crate::posture::required_pairs_mode(
                    profile,
                    &fixed,
                    Some(&ctx),
                    cache.refine.get(),
                )
                .is_empty()
                && if candidate[index].instrument == Instrument::Snare {
                    // The snare reference and direct hi-hat form one coupled
                    // ready posture. Do not greedily lock a snare that occupies
                    // the fixed pedal mast's only playable corridor.
                    hat_candidates.is_empty()
                        || hat_candidates.iter().any(|hat| {
                            if !cache.candidate() {
                                return false;
                            }
                            let tips = crate::posture::ready_pair(profile, &candidate[index], hat);
                            if !cache.pair(profile, tips) {
                                return false;
                            }
                            let mut pair_kit = fixed.clone();
                            pair_kit.push(hat.clone());
                            let pair_ctx = crate::equipment_motion::EquipmentContext {
                                geometry: crate::equipment::kit_geometry(&pair_kit),
                                kit: pair_kit,
                            };
                            pair_ctx.geometry.unresolved_piece_ids.is_empty()
                                && equipment_posture_mode(
                                    profile,
                                    &candidate[index],
                                    &pair_ctx,
                                    cache.refine.get(),
                                )
                                && equipment_posture_mode(
                                    profile,
                                    hat,
                                    &pair_ctx,
                                    cache.refine.get(),
                                )
                                && crate::posture::pair_solution_mode(
                                    profile,
                                    tips,
                                    [0.0; 2],
                                    Some(&pair_ctx),
                                    [Some(candidate[index].id.as_str()), Some(hat.id.as_str())],
                                    cache.refine.get(),
                                )
                                .is_some()
                        })
                } else {
                    true
                }
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
                            ) && crate::posture::pair_solution_mode(
                                profile,
                                crate::posture::ready_pair(profile, snare, &candidate[index]),
                                [0.0; 2],
                                Some(&ctx),
                                [Some(snare.id.as_str()), Some(candidate[index].id.as_str())],
                                cache.refine.get(),
                            )
                            .is_some()
                        })
                } else {
                    true
                }
        };
        if anchor_rank.unwrap_or(0) == 0
            && piece_posture_fits(profile, &result.kit[index], cache)
            && against_locked(&result.kit)
            && (anchor_rank.is_none() || joint::roles_valid(profile, &result.kit, &locked, index))
        {
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
                    if coupled_hat && (lateral != 0 || forward != 0) {
                        continue;
                    }
                    shifts.push([
                        lateral as f64 * 0.05,
                        vertical as f64 * 0.05,
                        forward as f64 * 0.05,
                    ]);
                }
            }
        }
        if result.kit[index].instrument == Instrument::Snare {
            // Resolve the narrow feasible corridor of a measured hand/sleeve
            // around the body reference before falling back to the coarse kit
            // grid. Resolution scales with the arm, not an avatar identity.
            let step = ((profile.lengths[0][0] + profile.lengths[1][0]) * 0.05).clamp(0.015, 0.03);
            for x in -5..=5 {
                for y in -5..=5 {
                    for z in -6..=6 {
                        let shift = [
                            desired[0] + x as f64 * step - original[0],
                            desired[1] + y as f64 * step - original[1],
                            desired[2] + z as f64 * step - original[2],
                        ];
                        if shift[0].abs() <= 0.30
                            && shift[1].abs() <= 0.25
                            && shift[2].abs() <= 0.30
                        {
                            shifts.push(shift);
                        }
                    }
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
        let mut accepted = 0;
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
                || (anchor_rank.is_some()
                    && !joint::roles_valid(profile, &candidate, &locked, index))
                || !against_locked(&candidate)
            {
                continue;
            }
            if result.kit[index].instrument == Instrument::Snare
                && accepted < anchor_rank.unwrap_or(0)
            {
                accepted += 1;
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
            cache.fail("placement", &result.kit[index].id);
            break;
        }
        locked.push(index);
    }
    result.unreachable_piece_ids = missed_mode(profile, &result.kit, true, cache.refine.get());
    add_equipment_misses(profile, &mut result);
    result.clearance_piece_ids = kit_clearance_ids(&result.kit);
    result.complete = result.unreachable_piece_ids.is_empty()
        && result.clearance_piece_ids.is_empty()
        && crate::posture::required_pairs(profile, &result.kit).is_empty()
        && (anchor_rank.is_none() || joint::roles_complete(profile, &result.kit));
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
    let cache = FitCache::default();
    profile.validate()?;
    if !valid(kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    // A previously accepted refined witness must remain stable on a repeat fit.
    // Check the unchanged full layout before pedal seeding can overwrite it.
    let mut existing = proposal(profile, kit.to_vec(), 1., [0.; 3]);
    let keep_existing = profile.hand_frames.is_some()
        && existing.clearance_piece_ids.is_empty()
        && existing.unreachable_piece_ids.is_empty()
        && joint::feet_fit(profile, kit)
        && joint::roles_complete(profile, kit)
        && missed_mode(profile, kit, true, true).is_empty()
        && {
            add_equipment_misses(profile, &mut existing);
            existing.unreachable_piece_ids.is_empty()
                && crate::posture::required_pairs(profile, kit).is_empty()
        };
    let mut result = if keep_existing {
        existing
    } else {
        uniform_fit(profile, kit, &cache)?
    };
    if !result.complete && profile.hand_frames.is_some() {
        result = joint::fit(profile, result);
    }
    let fitted_profile = joint::translated_profile(profile, result.seat_offset, &result.kit);
    result.capabilities = Some(crate::posture::audit(&fitted_profile, &result.kit));
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

#[path = "fit_joint.rs"]
mod joint;

#[cfg(test)]
#[path = "fit_tests.rs"]
mod tests;
