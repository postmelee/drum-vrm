//! Body-first whole-kit proposals shared by manual and automatic browser fitting.
//! No avatar names, source event edits or bone scaling.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::rig::RigProfile;
use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
#[path = "fit_layout.rs"]
mod layout;
#[path = "fit_lower_limb.rs"]
mod lower_limb;

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
    pub layout_violations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub advisory_layout_violations: Vec<String>,
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
/// Automatic fitting may choose a bounded compact kit; it may not keep
/// shrinking until a different, miniature arrangement happens to pass.
pub const MIN_FIT_SCALE: f64 = 0.76;
#[derive(Default)]
struct FitCache {
    candidates: Cell<usize>,
    exhausted: Cell<bool>,
    hands: RefCell<HashMap<(usize, [u64; 3]), bool>>,
    pairs: RefCell<HashMap<[u64; 6], bool>>,
    failures: RefCell<Vec<FitFailure>>,
    limit: Cell<usize>,
    refine: Cell<bool>,
    #[cfg(test)]
    family_visits: RefCell<Vec<f64>>,
}
impl FitCache {
    const LIMIT: usize = 100_000;
    fn ceiling(&self) -> usize {
        match self.limit.get() {
            0 => MAX_FIT_LAYOUT_CANDIDATES,
            limit => limit.min(MAX_FIT_LAYOUT_CANDIDATES),
        }
    }
    fn candidate(&self) -> bool {
        if self.candidates.get() >= self.ceiling() {
            self.exhausted.set(true);
            return false;
        }
        self.candidates.set(self.candidates.get() + 1);
        true
    }
    /// A child branch may use its allowance without closing later siblings.
    /// Every probe still increments the same request-wide candidate count.
    fn scoped<T>(&self, allowance: usize, run: impl FnOnce() -> T) -> T {
        let previous_limit = self.limit.get();
        let parent_limit = self.ceiling();
        let child_limit = parent_limit.min(self.candidates.get().saturating_add(allowance));
        self.limit.set(child_limit);
        self.exhausted.set(self.candidates.get() >= child_limit);
        let result = run();
        self.limit.set(previous_limit);
        self.exhausted.set(self.candidates.get() >= parent_limit);
        result
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
fn hand_posture_fits(profile: &RigProfile, limb: usize, target: Vec3) -> bool {
    crate::contact_feasibility::hand_probe_fits(profile, limb, target, None, None)
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
    piece_targets(profile, piece).iter().all(|&target| {
        hands_satisfied(piece, |limb| {
            crate::contact_feasibility::hand_probe_fits(
                profile,
                limb,
                target,
                Some(ctx),
                Some(piece.id.as_str()),
            )
        })
    })
}

/// Keep the canonical two-crash pair usable while greedily placing its members.
/// A lone crash may be played with either hand. For layout search, however, a
/// same-side impact/ready witness prevents locking a close crash that only the
/// opposite hand can reach. Score assignment remains free to use either hand.
/// This is a fallback search preference, not a new unchanged-kit certificate.
fn crash_role_search_fits(
    profile: &RigProfile,
    kit: &[KitPiece],
    index: usize,
    ctx: &crate::equipment_motion::EquipmentContext,
) -> bool {
    if profile.hand_frames.is_none() || kit[index].instrument != Instrument::Crash {
        return true;
    }
    // Match the two semantic crashes audited by posture::required_pairs.
    let crashes: Vec<_> = kit
        .iter()
        .enumerate()
        .filter(|(_, piece)| piece.instrument == Instrument::Crash)
        .map(|(i, _)| i)
        .take(2)
        .collect();
    if crashes.len() != 2 || !crashes.contains(&index) {
        return true;
    }
    let right = crate::motion::sub(profile.roots[1], profile.roots[0]);
    let separation = crate::motion::sub(kit[crashes[1]].position, kit[crashes[0]].position);
    let side = right[0] * separation[0] + right[2] * separation[2];
    if side.abs() < 1e-9 {
        return true;
    }
    let limb = usize::from((index == crashes[0]) != (side > 0.));
    piece_targets(profile, &kit[index]).iter().all(|&target| {
        [0., 0.035].into_iter().all(|lift| {
            crate::contact_feasibility::candidate(
                profile,
                limb,
                [target[0], target[1] + lift, target[2]],
                0.,
                Some(ctx),
                Some(&kit[index].id),
            )
            .is_some()
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
    // Semantic adapters already construct a relaxed, bent-arm reference from
    // measured segment lengths and the actual grip/tool transform. Reusing its
    // depth avoids deriving a nearly extended reach from forearm + stick length,
    // or pushing a kit away merely because a costume extends the torso envelope.
    // This is only a repair seed; fixed equipment/seat candidates are tried first
    // and every eventual placement still passes the complete physical guards.
    let depth = if profile.hand_frames.is_some() {
        (profile.rest_tips[0][2] + profile.rest_tips[1][2]) * 0.5
    } else {
        shoulder_forward + lower * 0.95 + tool * 0.95
    };
    [
        (profile.roots[0][0] + profile.roots[1][0]) * 0.5 - 0.08 * (upper + lower),
        (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - upper * 0.8 - 0.08,
        depth,
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
    let cache = FitCache::default();
    cache.refine.set(true);
    snare_ready_fits_cached(profile, piece, &cache)
}
fn piece_posture_fits(profile: &RigProfile, piece: &KitPiece, cache: &FitCache) -> bool {
    snare_ready_fits_cached(profile, piece, cache)
        && piece_targets(profile, piece)
            .iter()
            .all(|&target| hands_satisfied(piece, |limb| cache.hand(profile, limb, target)))
}
fn missed(profile: &RigProfile, kit: &[KitPiece], posture: bool) -> Vec<String> {
    let cache = FitCache::default();
    // Final certification must admit the same refined snare-ready witnesses as
    // the shared hand/pair gates; coarse search is only a candidate filter.
    cache.refine.set(posture);
    kit.iter()
        .filter(|piece| {
            (posture && !snare_ready_fits_cached(profile, piece, &cache))
                || piece_targets(profile, piece).iter().any(|&target| {
                    !hands_satisfied(piece, |limb| {
                        if posture {
                            hand_posture_fits(profile, limb, target)
                        } else if profile.hand_frames.is_some() && limb < 2 {
                            // This is only a broad-phase rejection. A default
                            // hand orientation is not the reach envelope of the
                            // target-oriented wrist solver. Final certification
                            // still requires the shared anatomical/tool witness.
                            profile.may_reach_contact(limb, target)
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
    let layout_violations = layout::violations(profile, &kit);
    KitFitProposal {
        complete: unreachable_piece_ids.is_empty(),
        kit,
        scale,
        translation,
        unreachable_piece_ids,
        clearance_piece_ids,
        layout_violations,
        advisory_layout_violations: Vec::new(),
        piece_adjustments: Vec::new(),
        search_candidates: 0,
        search_limit_reached: false,
        seat_offset: [0.; 3],
        search_failures: Vec::new(),
        capabilities: None,
    }
}
fn transform(kit: &[KitPiece], scale: f64, translation: Vec3) -> Vec<KitPiece> {
    // Stable kit origin for the shared layout factor. Only shell dimensions
    // shrink; cymbals keep nominal diameter. The bass body/contact stay fixed
    // and the direct hi-hat pedal is solved as its own connected assembly.
    const ORIGIN: Vec3 = [0.0, 1.235, 0.06];
    kit.iter()
        .map(|piece| {
            let mut next = piece.clone();
            if hand_piece(piece) {
                next.position = std::array::from_fn(|i| {
                    ORIGIN[i] + (piece.position[i] - ORIGIN[i]) * scale + translation[i]
                });
                // Compact shell families do not turn standard cymbals into
                // miniatures. Position search adapts their centres/stands while
                // preserving the authored hi-hat, crash and ride diameters.
                if matches!(
                    piece.instrument,
                    Instrument::Snare
                        | Instrument::TomHigh
                        | Instrument::TomMid
                        | Instrument::TomLow
                ) {
                    next.radius *= scale;
                }
            }
            next
        })
        .collect()
}
fn shift_cost(translation: Vec3) -> f64 {
    translation.iter().map(|v| v * v).sum()
}

fn seat_offsets(profile: &RigProfile) -> Vec<Vec3> {
    let mut offsets = vec![[0.; 3]];
    if profile.hand_frames.is_some() {
        offsets.clear();
        for dy in [0., -0.06, 0.06, -0.12, 0.12] {
            for dz in [0., 0.08, -0.08, 0.16, 0.24] {
                offsets.push([0., dy, dz]);
            }
        }
        offsets.sort_by(|a, b| shift_cost(*a).total_cmp(&shift_cost(*b)));
    }
    offsets.retain(|offset| {
        (0.3..=1.2).contains(&(profile.pelvis[1] + offset[1]))
            && (profile.pelvis[2] + offset[2]).abs() <= 0.5
    });
    offsets
}

/// Exhaust the existing bounded seating choices against the unchanged kit
/// before moving any instrument. A valid fixed setup must not lose to an
/// earlier greedy per-piece repair. Every accepted seat uses the same final
/// certificate as the layout fallback; no reach or clearance gate is skipped.
fn fixed_layout_fit(
    profile: &RigProfile,
    kit: &[KitPiece],
    offsets: &[Vec3],
    cache: &FitCache,
) -> Option<KitFitProposal> {
    let measured: Vec<_> = kit
        .iter()
        .map(|piece| {
            let mut next = piece.clone();
            next.geometry = Some(crate::equipment::local_geometry(piece));
            next
        })
        .collect();
    for &offset in offsets {
        // Preserve the zero-work unchanged certificate and account for every
        // additional seat in the existing request-wide search budget.
        if offset != [0.; 3] && !cache.candidate() {
            break;
        }
        let moved = fit_profile::candidate(profile, offset, &measured);
        if !fit_profile::feet_fit(&moved, &measured) {
            continue;
        }
        let mut result = proposal(&moved, measured.clone(), 1., [0.; 3]);
        if !result.unreachable_piece_ids.is_empty() {
            continue;
        }
        certify(&moved, &mut result);
        if result.complete {
            result.seat_offset = offset;
            return Some(result);
        }
    }
    None
}
/// Prefer the largest kit, then the smallest common translation, with a bounded
/// search. A completed proposal is idempotent; incomplete results stay explicit.
/// `complete` means designated hands pass sampled contact/preparation guards,
/// complete sticks, structural leg cores and every generated support remain clear,
/// and drumhead approach regions are accessible. It does not certify whole
/// motion paths or skinned meshes; those require separate temporal inspection.
fn uniform_fit(
    profile: &RigProfile,
    kit: &[KitPiece],
    cache: &FitCache,
    original_only: bool,
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
    let mut original = proposal(profile, kit.to_vec(), 1.0, [0.0; 3]);
    certify(profile, &mut original);
    if original.complete {
        return Ok(cache.finish(original));
    }
    // Nominal seating and compact families share this exact repair/certify
    // path. The outer scheduler reserves separate work lanes so meaningful
    // compact probes cannot delay every small, unscaled seat adjustment.
    if original_only {
        let repaired = repair_posture(profile, original.clone(), cache);
        return Ok(cache.finish(if repaired.complete {
            repaired
        } else {
            original
        }));
    }
    const FAMILY_BUDGET: usize = 6_000;
    let reference = snare_reference(profile);
    let height_delta = (profile.roots[0][1] + profile.roots[1][1]) * 0.5 - 1.235;
    let translation = |scale: f64, reference_depth: bool| -> Vec3 {
        let scaled = transform(kit, scale, [0.0; 3]);
        let snare = scaled.iter().find(|p| p.instrument == Instrument::Snare);
        let dy = snare
            .map_or(height_delta, |p| reference[1] - p.position[1])
            .clamp(-0.5, 0.5);
        let dz = if reference_depth {
            snare
                .map_or(0.0, |p| reference[2] - p.position[2])
                .clamp(-0.25, 0.10)
        } else {
            0.0
        };
        [0.0, dy, dz]
    };
    let attempt = |scale: f64, shift: Vec3| -> Option<KitFitProposal> {
        if cache.exhausted.get() {
            return None;
        }
        let adjusted = transform(kit, scale, shift);
        if !valid(&adjusted) {
            return None;
        }
        #[cfg(test)]
        cache.family_visits.borrow_mut().push(scale);
        let candidate = cache.scoped(FAMILY_BUDGET, || {
            repair_posture(profile, proposal(profile, adjusted, scale, shift), cache)
        });
        candidate.complete.then_some(candidate)
    };
    let scales = [1.0, 0.92, 0.84, MIN_FIT_SCALE];
    // First visit the body-anchored translation of every coarse shell family.
    // Zero-depth alternatives are a second wave, not a second full nominal
    // search that can starve every smaller family before it is even visited.
    for reference_depth in [true, false] {
        for (i, scale) in scales.into_iter().enumerate() {
            if cache.exhausted.get() {
                break;
            }
            let shift = translation(scale, reference_depth);
            if !reference_depth && shift == translation(scale, true) {
                continue;
            }
            if let Some(mut complete) = attempt(scale, shift) {
                let previous = if i == 0 { 1.0 } else { scales[i - 1] };
                let mut finer = previous - 0.02;
                while finer > scale + 1e-9 && !cache.exhausted.get() {
                    if let Some(candidate) = attempt(finer, translation(finer, reference_depth)) {
                        complete = candidate;
                        break;
                    }
                    finer -= 0.02;
                }
                return Ok(cache.finish(complete));
            }
        }
    }
    Ok(cache.finish(original))
}
/// First retain the original bounded uniform reach fit, then repair only
/// surfaces that fail the common posture guard. No partial unsafe proposal is
/// applied: failure returns the original layout with explicit offending IDs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RepairProgress {
    HatRejected,
    ReferenceRejected,
    LaterBlocked,
}
fn repair_posture(
    profile: &RigProfile,
    result: KitFitProposal,
    cache: &FitCache,
) -> KitFitProposal {
    let pedals = hihat_pedal_candidates(profile, &result);
    let mut last = result.clone();
    let mut promising = 0;
    for pedal in pedals {
        if cache.exhausted.get() {
            break;
        }
        let mut branch = result.clone();
        if let Some(pedal) = pedal {
            let index = branch
                .kit
                .iter()
                .position(|p| p.instrument == Instrument::PedalHat)
                .unwrap();
            let shift =
                std::array::from_fn(|axis| pedal.position[axis] - branch.kit[index].position[axis]);
            branch.kit[index] = pedal;
            if crate::motion::distance(shift, [0.; 3]) > 1e-9 {
                branch.piece_adjustments.push(PieceAdjustment {
                    piece_id: branch.kit[index].id.clone(),
                    translation: shift,
                });
            }
        }
        let profile = fit_profile::candidate(profile, [0.; 3], &branch.kit);
        cache.hands.borrow_mut().clear();
        cache.pairs.borrow_mut().clear();
        let mut excluded_references = Vec::new();
        let mut branch_progress = RepairProgress::HatRejected;
        // A valid snare/hat reference is not necessarily compatible with the
        // rack or outer cymbals. Revisit at most three distinct validated snare
        // anchors with the same existing repair path and physical predicates.
        for _ in 0..3 {
            if cache.exhausted.get() {
                break;
            }
            let (first, progress) =
                repair_posture_order(&profile, branch.clone(), cache, true, &excluded_references);
            if first.complete {
                return first;
            }
            branch_progress = progress;
            let rejected_reference = first
                .kit
                .iter()
                .find(|piece| piece.instrument == Instrument::Snare)
                .map(|piece| piece.position);
            last = first;
            if progress != RepairProgress::LaterBlocked || cache.exhausted.get() {
                break;
            }
            let (second, _) =
                repair_posture_order(&profile, branch.clone(), cache, false, &excluded_references);
            if second.complete {
                return second;
            }
            last = second;
            let Some(reference) = rejected_reference else {
                break;
            };
            excluded_references.push(reference);
        }
        // Hat-only rejection remains cheap; reserve deep reference retries for
        // at most three feasible pedal assemblies. All repeated probes count.
        if branch_progress != RepairProgress::HatRejected {
            promising += 1;
            if promising >= 3 {
                break;
            }
        }
    }
    last
}
/// Keep the direct hi-hat assembly near the seated left foot while leaving room
/// for the snare. Candidates preserve real leg/foot dimensions and test the full
/// footboard pitch range, rather than detaching the cymbal from a fixed pedal.
fn hihat_pedal_candidates(profile: &RigProfile, result: &KitFitProposal) -> Vec<Option<KitPiece>> {
    let Some(hat) = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Hihat)
    else {
        return vec![None];
    };
    let Some(snare) = result
        .kit
        .iter()
        .find(|p| p.instrument == Instrument::Snare)
    else {
        return vec![None];
    };
    let Some(index) = result
        .kit
        .iter()
        .position(|p| p.instrument == Instrument::PedalHat)
    else {
        return vec![None];
    };
    let original = result.kit[index].position;
    let reference = snare_reference(profile);
    let desired = [
        reference[0] - snare.radius - hat.radius - 0.035,
        original[1],
        reference[2] + 0.05 - crate::equipment::HIHAT_PEDAL_TO_AXIS,
    ];
    let original_mast = crate::equipment::pedal_frame_point(
        &result.kit[index],
        [0., 0., crate::equipment::HIHAT_PEDAL_TO_AXIS],
    );
    let desired_mast = [
        desired[0],
        original[1],
        desired[2] + crate::equipment::HIHAT_PEDAL_TO_AXIS,
    ];
    let mut candidates = vec![
        original_mast,
        desired_mast,
        [desired_mast[0], original[1], original_mast[2]],
        [original_mast[0], original[1], desired_mast[2]],
    ];
    // Bounded mechanical assembly offsets preserve its pedal-to-mast linkage.
    // Hand/posture/equipment feasibility is tested before choosing this branch.
    for step in [0.06, 0.12] {
        for [x, z] in [
            [-1., 0.],
            [1., 0.],
            [0., -1.],
            [0., 1.],
            [-1., -1.],
            [1., -1.],
            [-1., 1.],
            [1., 1.],
        ] {
            candidates.push([
                original_mast[0] + x * step,
                original[1],
                original_mast[2] + z * step,
            ]);
        }
    }
    let mut out = Vec::new();
    for mast in candidates {
        let direction = crate::equipment::horizontal_direction(profile.roots[2], mast);
        let position = [
            mast[0] - direction[0] * crate::equipment::HIHAT_PEDAL_TO_AXIS,
            original[1],
            mast[2] - direction[2] * crate::equipment::HIHAT_PEDAL_TO_AXIS,
        ];
        if (position[0] - original[0]).abs() > 0.25 || (position[2] - original[2]).abs() > 0.25 {
            continue;
        }
        let mut piece = result.kit[index].clone();
        piece.position = position;
        piece.pedal_forward = Some(direction);
        let feasible = fit_profile::foot_fits(profile, &piece, 2);
        if feasible
            && !out.iter().any(|p: &Option<KitPiece>| {
                p.as_ref()
                    .is_some_and(|p| crate::motion::distance(p.position, position) < 1e-8)
            })
        {
            out.push(Some(piece));
        }
    }
    if out.is_empty() {
        out.push(None);
    }
    out
}
/// Place both bass-mounted toms as one shallow row. The bounded group search
/// changes centre, spacing and shared elevation; neither member can escape into
/// a separate front/back row to make a local reach/obstacle test pass.
fn place_rack_group(
    profile: &RigProfile,
    result: &mut KitFitProposal,
    rack: &[usize],
    locked: &[usize],
    cache: &FitCache,
) -> bool {
    let high = *rack
        .iter()
        .find(|&&i| result.kit[i].instrument == Instrument::TomHigh)
        .unwrap();
    let mid = *rack
        .iter()
        .find(|&&i| result.kit[i].instrument == Instrument::TomMid)
        .unwrap();
    let h = result.kit[high].position;
    let m = result.kit[mid].position;
    let center: Vec3 = std::array::from_fn(|axis| (h[axis] + m[axis]) * 0.5);
    let sum = crate::equipment::surface(&result.kit[high]).radius
        + crate::equipment::surface(&result.kit[mid]).radius;
    let spacing = (m[0] - h[0]).max(sum + 0.015);
    let height_delta = (h[1] - m[1]).clamp(-0.03, 0.03);
    let depth_delta = (h[2] - m[2]).clamp(-0.03, 0.03);
    let mut candidates = Vec::new();
    for x in -4..=4 {
        for y in -3..=3 {
            for z in -5..=4 {
                for width in [spacing, sum + 0.025, sum + 0.06] {
                    let c = [
                        center[0] + x as f64 * 0.05,
                        center[1] + y as f64 * 0.05,
                        center[2] + z as f64 * 0.05,
                    ];
                    let positions = [
                        [
                            c[0] - width * 0.5,
                            c[1] + height_delta * 0.5,
                            c[2] + depth_delta * 0.5,
                        ],
                        [
                            c[0] + width * 0.5,
                            c[1] - height_delta * 0.5,
                            c[2] - depth_delta * 0.5,
                        ],
                    ];
                    if positions.iter().zip([h, m]).all(|(p, o)| {
                        (p[0] - o[0]).abs() <= 0.30 + 1e-9
                            && (p[1] - o[1]).abs() <= 0.25 + 1e-9
                            && (p[2] - o[2]).abs() <= 0.30 + 1e-9
                    }) {
                        let cost = positions
                            .iter()
                            .zip([h, m])
                            .map(|(p, o)| shift_cost(std::array::from_fn(|a| p[a] - o[a])))
                            .sum::<f64>();
                        candidates.push((cost, positions));
                    }
                }
            }
        }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, positions) in candidates {
        if !cache.candidate() {
            return false;
        }
        let mut candidate = result.kit.clone();
        candidate[high].position = positions[0];
        candidate[mid].position = positions[1];
        let mut fixed: Vec<KitPiece> = locked
            .iter()
            .filter(|&&i| candidate[i].instrument != Instrument::OpenHat)
            .map(|&i| candidate[i].clone())
            .collect();
        fixed.push(candidate[high].clone());
        fixed.push(candidate[mid].clone());
        if !layout::violations(profile, &fixed).is_empty()
            || !piece_clear(fixed.len() - 1, &fixed)
            || !piece_clear(fixed.len() - 2, &fixed)
            || !piece_posture_fits(profile, &candidate[high], cache)
            || !piece_posture_fits(profile, &candidate[mid], cache)
        {
            continue;
        }
        let geometry = crate::equipment::kit_geometry(&fixed);
        let ctx = crate::equipment_motion::EquipmentContext::new(fixed, geometry);
        if !ctx.geometry.unresolved_piece_ids.is_empty()
            || !lower_limb::structural_violations(profile, &ctx.kit, &ctx.geometry).is_empty()
            || !ctx
                .kit
                .iter()
                .all(|p| equipment_posture_fits(profile, p, &ctx))
            || !crate::posture::required_pairs_with_equipment(profile, &ctx.kit, Some(&ctx))
                .is_empty()
        {
            continue;
        }
        for (i, old) in [(high, h), (mid, m)] {
            let translation = std::array::from_fn(|axis| candidate[i].position[axis] - old[axis]);
            if crate::motion::distance(translation, [0.; 3]) > 1e-9 {
                result.piece_adjustments.push(PieceAdjustment {
                    piece_id: candidate[i].id.clone(),
                    translation,
                });
            }
        }
        result.kit = candidate;
        return true;
    }
    false
}

/// Tripod phase is a real hardware degree of freedom, stored in the accepted
/// kit so runtime reconstruction cannot silently choose another leg direction.
/// Try it only when the current piece's support intersects the lower body;
/// moving an unrelated obstruction or weakening an envelope is not a repair.
fn choose_leg_clear_stand_phase(
    profile: &RigProfile,
    candidate: &mut [KitPiece],
    index: usize,
    locked: &[usize],
    cache: &FitCache,
) -> bool {
    let fixed = |candidate: &[KitPiece]| {
        let mut kit: Vec<_> = locked
            .iter()
            .filter(|&&i| i != index && candidate[i].instrument != Instrument::OpenHat)
            .map(|&i| candidate[i].clone())
            .collect();
        kit.push(candidate[index].clone());
        kit
    };
    let kit = fixed(candidate);
    let geometry = crate::equipment::kit_geometry(&kit);
    let failures = lower_limb::structural_violations(profile, &kit, &geometry);
    if failures.is_empty() {
        return true;
    }
    let rotatable = matches!(
        candidate[index].instrument,
        Instrument::Snare | Instrument::Hihat | Instrument::Crash | Instrument::Ride
    ) || (candidate[index].instrument == Instrument::TomLow
        && candidate[index].mount.is_none());
    if !rotatable
        || failures
            .iter()
            .any(|code| !code.ends_with(&format!(":{}", candidate[index].id)))
    {
        return false;
    }
    // This piece's shell, footboards and leg IK do not change with its stand
    // phase. The guard above already excludes failures owned by other pieces.
    // Reject these exact invariant obstructions before spending phase probes.
    let mut invariant_geometry = geometry.clone();
    invariant_geometry
        .supports
        .retain(|support| support.piece_id != candidate[index].id);
    if !lower_limb::structural_violations(profile, &kit, &invariant_geometry).is_empty() {
        return false;
    }
    let original = candidate[index].stand_rotation;
    for step in 0..24 {
        if !cache.candidate() {
            break;
        }
        let angle = -std::f64::consts::PI + step as f64 * std::f64::consts::PI / 12.;
        candidate[index].stand_rotation = Some(angle);
        let kit = fixed(candidate);
        let geometry = crate::equipment::kit_geometry(&kit);
        if geometry.unresolved_piece_ids.is_empty()
            && lower_limb::structural_violations(profile, &kit, &geometry).is_empty()
        {
            return true;
        }
    }
    candidate[index].stand_rotation = original;
    false
}

fn repair_posture_order(
    profile: &RigProfile,
    mut result: KitFitProposal,
    cache: &FitCache,
    rack_first: bool,
    excluded_references: &[Vec3],
) -> (KitFitProposal, RepairProgress) {
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
                    let index = kit.len() - 1;
                    let fixed: Vec<_> = (0..index).collect();
                    if !choose_leg_clear_stand_phase(profile, &mut kit, index, &fixed, cache) {
                        return None;
                    }
                    let hat = kit[index].clone();
                    let geometry = crate::equipment::kit_geometry(&kit);
                    let ctx = crate::equipment_motion::EquipmentContext::new(kit, geometry);
                    (ctx.geometry.unresolved_piece_ids.is_empty()
                        && lower_limb::structural_violations(profile, &ctx.kit, &ctx.geometry)
                            .is_empty()
                        && equipment_posture_fits(profile, &hat, &ctx))
                    .then_some(hat)
                })
                .collect()
        });
    if result.kit.iter().any(|p| p.instrument == Instrument::Hihat) && hat_candidates.is_empty() {
        cache.fail("hihat-witness", "hihat");
        result.complete = false;
        return (result, RepairProgress::HatRejected);
    }
    // Solve the supported ready surface first. A later target repair must keep
    // already placed bodies clear instead of fixing one hand by intersecting a tom.
    let mut order: Vec<usize> = (0..result.kit.len())
        .filter(|&i| hand_piece(&result.kit[i]))
        .collect();
    // Establish the snare/hat reference, coupled rack row and floor flank
    // before fitting the outer cymbals around those fixed playing regions.
    order.sort_by_key(|&i| match result.kit[i].instrument {
        Instrument::Snare => 0,
        Instrument::Hihat | Instrument::OpenHat => 1,
        Instrument::TomHigh | Instrument::TomMid => {
            if rack_first {
                2
            } else {
                3
            }
        }
        Instrument::TomLow => {
            if rack_first {
                3
            } else {
                2
            }
        }
        Instrument::Ride => 4,
        _ => 5,
    });
    // Equipment attachments are a dependency graph, not a presentation order.
    // A suspended tom must retain its host stand in every partial certificate.
    // Missing/cyclic hosts remain explicit unresolved geometry in certification.
    for _ in 0..order.len() {
        let mut moved = false;
        for position in 0..order.len() {
            let Some(crate::model::PieceMount::StandClamp { support_piece_id }) =
                &result.kit[order[position]].mount
            else {
                continue;
            };
            if let Some(host_position) = order
                .iter()
                .position(|&i| result.kit[i].id == *support_piece_id)
            {
                if host_position > position {
                    let host = order.remove(host_position);
                    order.insert(position, host);
                    moved = true;
                    break;
                }
            }
        }
        if !moved {
            break;
        }
    }
    let mut locked: Vec<usize> = (0..result.kit.len())
        .filter(|&i| !hand_piece(&result.kit[i]))
        .collect();
    for index in order {
        if locked.contains(&index) {
            continue;
        }
        if matches!(
            result.kit[index].instrument,
            Instrument::TomHigh | Instrument::TomMid
        ) {
            let rack: Vec<usize> = result
                .kit
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    matches!(p.instrument, Instrument::TomHigh | Instrument::TomMid).then_some(i)
                })
                .collect();
            if rack.len() == 2
                && rack
                    .iter()
                    .any(|&i| result.kit[i].instrument == Instrument::TomHigh)
                && rack
                    .iter()
                    .any(|&i| result.kit[i].instrument == Instrument::TomMid)
            {
                if !place_rack_group(profile, &mut result, &rack, &locked, cache) {
                    cache.fail("placement", "rack-row");
                    break;
                }
                locked.extend(rack);
                continue;
            }
        }
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
            if !layout::violations(profile, &fixed).is_empty() {
                return false;
            }
            let geometry = crate::equipment::kit_geometry(&fixed);
            let ctx = crate::equipment_motion::EquipmentContext::new(fixed.clone(), geometry);
            ctx.geometry.unresolved_piece_ids.is_empty()
                && crash_role_search_fits(profile, candidate, index, &ctx)
                && equipment_posture_fits(profile, &candidate[index], &ctx)
                && fixed
                    .iter()
                    .filter(|p| p.id != candidate[index].id)
                    .all(|p| equipment_posture_fits(profile, p, &ctx))
                && crate::posture::required_pairs_with_equipment(profile, &fixed, Some(&ctx))
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
                            let geometry = crate::equipment::kit_geometry(&pair_kit);
                            let pair_ctx =
                                crate::equipment_motion::EquipmentContext::new(pair_kit, geometry);
                            pair_ctx.geometry.unresolved_piece_ids.is_empty()
                                && equipment_posture_fits(profile, &candidate[index], &pair_ctx)
                                && equipment_posture_fits(profile, hat, &pair_ctx)
                                && crate::posture::pair_solution_with_equipment(
                                    profile,
                                    tips,
                                    [0.0; 2],
                                    Some(&pair_ctx),
                                    [Some(candidate[index].id.as_str()), Some(hat.id.as_str())],
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
        let placement_fits = |candidate: &mut [KitPiece]| {
            if candidate[index].instrument == Instrument::Snare
                && excluded_references.iter().any(|previous| {
                    crate::motion::distance(*previous, candidate[index].position) < 1e-8
                })
            {
                return false;
            }
            if !against_locked(candidate) {
                return false;
            }
            let phase = candidate[index].stand_rotation;
            choose_leg_clear_stand_phase(profile, candidate, index, &locked, cache)
                && (phase == candidate[index].stand_rotation || against_locked(candidate))
        };
        let mut initial = result.kit.clone();
        if piece_posture_fits(profile, &initial[index], cache) && placement_fits(&mut initial) {
            result.kit = initial;
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
            // Resolve the narrow feasible tool corridor around the measured
            // relaxed reference before falling back to the coarse kit
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
                || !placement_fits(&mut candidate)
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
            cache.fail("placement", &result.kit[index].id);
            break;
        }
        locked.push(index);
    }
    certify(profile, &mut result);
    let reference_locked = [Instrument::Snare, Instrument::Hihat].iter().all(|role| {
        !result.kit.iter().any(|p| p.instrument == *role)
            || locked.iter().any(|&i| result.kit[i].instrument == *role)
    });
    (
        result,
        if reference_locked {
            RepairProgress::LaterBlocked
        } else {
            RepairProgress::ReferenceRejected
        },
    )
}

/// The same acceptance predicate gates a fresh proposal and an existing fitted
/// kit. Keeping this separate from search makes repeated preparation idempotent.
fn certify(profile: &RigProfile, result: &mut KitFitProposal) {
    result.unreachable_piece_ids = missed(profile, &result.kit, true);
    // Envelope clearance alone does not enforce the interior knee limits.
    // Apply the same real pitched sole reach/posture gate to unchanged inputs,
    // generated pedal branches and every final returned candidate.
    if !fit_profile::feet_fit(profile, &result.kit) {
        for (limb, instrument) in [(2, Instrument::PedalHat), (3, Instrument::Kick)] {
            if let Some(piece) = result
                .kit
                .iter()
                .find(|piece| piece.instrument == instrument)
            {
                if !fit_profile::foot_fits(profile, piece, limb)
                    && !result.unreachable_piece_ids.contains(&piece.id)
                {
                    result.unreachable_piece_ids.push(piece.id.clone());
                }
            }
        }
    }
    add_equipment_misses(profile, result);
    result.clearance_piece_ids = kit_clearance_ids(&result.kit);
    result.layout_violations = layout::violations(profile, &result.kit);
    result
        .layout_violations
        .extend(lower_limb::structural_violations(
            profile,
            &result.kit,
            &crate::equipment::kit_geometry(&result.kit),
        ));
    result.complete = result.layout_violations.is_empty()
        && result.unreachable_piece_ids.is_empty()
        && result.clearance_piece_ids.is_empty()
        && crate::posture::required_pairs(profile, &result.kit).is_empty();
}

fn add_equipment_misses(profile: &RigProfile, result: &mut KitFitProposal) {
    let ctx = crate::equipment_motion::EquipmentContext::new(
        result.kit.clone(),
        crate::equipment::kit_geometry(&result.kit),
    );
    for piece in &result.kit {
        if !equipment_posture_fits(profile, piece, &ctx)
            && !result.unreachable_piece_ids.contains(&piece.id)
        {
            result.unreachable_piece_ids.push(piece.id.clone());
        }
    }
}

pub fn fit_kit(profile: &RigProfile, kit: &[KitPiece]) -> Result<KitFitProposal, String> {
    profile.validate()?;
    if !valid(kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    let cache = FitCache::default();
    let original_profile = fit_profile::candidate(profile, [0.; 3], kit);
    let offsets = seat_offsets(profile);
    let fixed = fixed_layout_fit(profile, kit, &offsets, &cache);
    cache.limit.set(12_000);
    let mut result = match fixed {
        Some(result) => result,
        None => uniform_fit(&original_profile, kit, &cache, true)?,
    };
    if !result.complete {
        cache.refine.set(true);
        // Prefer small seat changes with the nominal kit, while reserving at
        // least 50k of the 80k request budget for real compact-family coverage.
        // The second lane uses four 6k coarse-family allowances at each seat;
        // cheap rejection leaves its unused budget available to later families.
        'passes: for original_only in [true, false] {
            if original_only && profile.hand_frames.is_none() {
                continue;
            }
            let pass_limit = if original_only {
                30_000
            } else {
                MAX_FIT_LAYOUT_CANDIDATES
            };
            let seat_allowance = if original_only { 6_000 } else { 24_000 };
            for &offset in &offsets {
                if cache.candidates.get() >= pass_limit {
                    break;
                }
                if !(0.3..=1.2).contains(&(profile.pelvis[1] + offset[1]))
                    || (profile.pelvis[2] + offset[2]).abs() > 0.5
                {
                    continue;
                }
                let moved = fit_profile::candidate(profile, offset, kit);
                // Only the fixed kick can reject a seat before assembly search.
                // The movable left pedal and both final feet keep the same gates.
                if kit
                    .iter()
                    .find(|piece| piece.instrument == Instrument::Kick)
                    .is_some_and(|piece| !fit_profile::foot_fits(&moved, piece, 3))
                {
                    cache.fail("feet", "kick");
                    continue;
                }
                cache
                    .limit
                    .set((cache.candidates.get() + seat_allowance).min(pass_limit));
                cache.exhausted.set(false);
                cache.hands.borrow_mut().clear();
                cache.pairs.borrow_mut().clear();
                let mut candidate = uniform_fit(&moved, kit, &cache, original_only)?;
                if candidate.complete {
                    candidate.seat_offset = offset;
                    result = candidate;
                    break 'passes;
                }
            }
        }
    }
    let fitted_profile = fit_profile::candidate(profile, result.seat_offset, &result.kit);
    certify(&fitted_profile, &mut result);
    // Inspect costume envelopes once for the returned setup. They remain useful
    // diagnostics without forcing a relocation or repeating costly mesh-proxy
    // checks for every candidate in the bounded mechanical search.
    result.advisory_layout_violations = lower_limb::violations(
        &fitted_profile,
        &result.kit,
        &crate::equipment::kit_geometry(&result.kit),
    )
    .into_iter()
    .filter(|violation| !result.layout_violations.contains(violation))
    .collect();
    result.capabilities = Some(crate::posture::audit(&fitted_profile, &result.kit));
    Ok(cache.finish(result))
}

#[path = "fit_profile.rs"]
mod fit_profile;

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
