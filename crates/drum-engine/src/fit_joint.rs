//! Bounded recovery after the fast layout pass. Seat/foot/hat witnesses remain
//! coupled; retries never change collision tolerances, limb lengths or source data.
use super::*;

pub(super) fn translated_profile(
    profile: &RigProfile,
    offset: Vec3,
    kit: &[KitPiece],
) -> RigProfile {
    let mut p = profile.clone();
    let translate = |v: &mut Vec3| {
        for i in 0..3 {
            v[i] += offset[i];
        }
    };
    for root in &mut p.roots {
        translate(root);
    }
    for tip in &mut p.rest_tips[..2] {
        translate(tip);
    }
    for point in [&mut p.pelvis, &mut p.chest, &mut p.head] {
        translate(point);
    }
    if let Some(pivot) = &mut p.torso_pivot {
        translate(pivot);
    }
    if let Some(body) = &mut p.body_geometry {
        translate(&mut body.torso.center);
    }
    for (limb, instrument) in [(2, Instrument::PedalHat), (3, Instrument::Kick)] {
        if let Some(pedal) = kit.iter().find(|p| p.instrument == instrument) {
            p.rest_tips[limb] = pedal.position;
        }
    }
    p
}

pub(super) fn foot_fits(profile: &RigProfile, piece: &KitPiece, limb: usize) -> bool {
    [0., -0.09, -0.18].into_iter().all(|pitch| {
        let rotation = crate::rig::pitch_rotation(profile.contact_rotations[limb], pitch);
        let ankle = crate::motion::sub(
            crate::equipment::pedal_contact(piece, pitch),
            profile.offset(limb, rotation),
        );
        let distance = crate::motion::distance(profile.roots[limb], ankle);
        let [a, b] = profile.lengths[limb];
        let knee = ((a * a + b * b - distance * distance) / (2. * a * b))
            .clamp(-1., 1.)
            .acos();
        distance >= (a - b).abs() + 1e-5
            && distance <= a + b - 1e-4
            && (55_f64.to_radians()..=155_f64.to_radians()).contains(&knee)
    })
}
pub(super) fn feet_fit(profile: &RigProfile, kit: &[KitPiece]) -> bool {
    [(2, Instrument::PedalHat), (3, Instrument::Kick)]
        .into_iter()
        .all(|(limb, instrument)| {
            kit.iter()
                .find(|p| p.instrument == instrument)
                .is_none_or(|p| foot_fits(profile, p, limb))
        })
}

/// Role regions supplement, never replace, full geometry. Only already selected
/// pieces constrain a partial branch; later pieces may still move within bounds.
pub(super) fn roles_valid(
    profile: &RigProfile,
    kit: &[KitPiece],
    locked: &[usize],
    index: usize,
) -> bool {
    let selected = |kind| {
        kit.iter()
            .enumerate()
            .find(|(i, p)| p.instrument == kind && (*i == index || locked.contains(i)))
            .map(|(_, p)| p)
    };
    let Some(snare) = selected(Instrument::Snare) else {
        return true;
    };
    let middle = (profile.roots[2][0] + profile.roots[3][0]) * 0.5;
    let half_width = ((profile.roots[2][0] - profile.roots[3][0]).abs() * 0.5).max(0.12);
    if (snare.position[0] - middle).abs() > half_width {
        return false;
    }
    if let Some(hat) = selected(Instrument::Hihat) {
        if hat.position[0] > snare.position[0] - 0.08 || hat.position[1] < snare.position[1] + 0.02
        {
            return false;
        }
    }
    for kind in [Instrument::TomHigh, Instrument::TomMid] {
        if let Some(tom) = selected(kind) {
            if tom.position[2] < snare.position[2] + 0.10 {
                return false;
            }
        }
    }
    if let (Some(high), Some(mid)) = (selected(Instrument::TomHigh), selected(Instrument::TomMid)) {
        if high.position[0] >= mid.position[0] || (high.position[2] - mid.position[2]).abs() > 0.15
        {
            return false;
        }
    }
    if let Some(floor) = selected(Instrument::TomLow) {
        if floor.position[0] < snare.position[0] + 0.10
            || (floor.position[2] - snare.position[2]).abs() > 0.25
        {
            return false;
        }
    }
    selected(Instrument::Ride).is_none_or(|ride| ride.position[0] >= snare.position[0] + 0.10)
}

pub(super) fn roles_complete(profile: &RigProfile, kit: &[KitPiece]) -> bool {
    roles_valid(profile, kit, &(0..kit.len()).collect::<Vec<_>>(), 0)
}

struct Seed {
    offset: Vec3,
    result: KitFitProposal,
    cost: f64,
}

pub(super) fn fit(profile: &RigProfile, failed: KitFitProposal) -> KitFitProposal {
    let measured = failed.kit.clone();
    let baseline = measured.as_slice();
    let cache = FitCache::default();
    cache.refine.set(true);
    cache.candidates.set(failed.search_candidates);
    *cache.failures.borrow_mut() = failed.search_failures.clone();
    let Some(pedal_index) = baseline
        .iter()
        .position(|p| p.instrument == Instrument::PedalHat)
    else {
        return failed;
    };
    let Some(hat_index) = baseline
        .iter()
        .position(|p| p.instrument == Instrument::Hihat)
    else {
        return failed;
    };
    let original_pedal = baseline[pedal_index].position;
    let mut seats = vec![[0.; 3]];
    for dy in [0., 0.06, -0.06, 0.12, -0.12] {
        for dz in [0., 0.08, -0.08, 0.16, 0.24] {
            let offset = [0., dy, dz];
            if offset != [0.; 3] {
                seats.push(offset);
            }
        }
    }
    let mut seeds = Vec::new();
    let seed_budget = (failed.search_candidates + 20_000).min(60_000);
    cache.limit.set(seed_budget);
    for offset in seats {
        if cache.exhausted.get() {
            break;
        }
        if !(0.3..=1.2).contains(&(profile.pelvis[1] + offset[1]))
            || (profile.pelvis[2] + offset[2]).abs() > 0.5
        {
            continue;
        }
        // Memoized hand queries are keyed by target under one immutable body.
        cache.hands.borrow_mut().clear();
        cache.pairs.borrow_mut().clear();
        let p = translated_profile(profile, offset, baseline);
        if baseline
            .iter()
            .find(|k| k.instrument == Instrument::Kick)
            .is_some_and(|k| !foot_fits(&p, k, 3))
        {
            cache.fail("feet", "kick");
            continue;
        }
        let reference = snare_reference(&p);
        let mut seat_seeds = Vec::new();
        for scale in [1., 0.84, 0.68, 0.60] {
            let mut kit = transform(baseline, scale, [0.; 3]);
            let Some(snare) = kit.iter().find(|k| k.instrument == Instrument::Snare) else {
                return failed;
            };
            let translation = [
                0.,
                (reference[1] - snare.position[1]).clamp(-0.5, 0.5),
                (reference[2] - snare.position[2]).clamp(-0.25, 0.1),
            ];
            kit = transform(baseline, scale, translation);
            let desired = [
                reference[0]
                    - kit
                        .iter()
                        .find(|k| k.instrument == Instrument::Snare)
                        .unwrap()
                        .radius
                    - kit[hat_index].radius
                    - 0.035,
                original_pedal[1],
                reference[2] + 0.05 - crate::equipment::HIHAT_PEDAL_TO_AXIS,
            ];
            for dx in [0., 0.06, -0.06, 0.12, 0.18] {
                for dz in [0., -0.06, 0.06, -0.12] {
                    let position = [desired[0] + dx, original_pedal[1], desired[2] + dz];
                    if (position[0] - original_pedal[0]).abs() > 0.25
                        || (position[2] - original_pedal[2]).abs() > 0.25
                    {
                        continue;
                    }
                    let mut k = kit.clone();
                    k[pedal_index].position = position;
                    if !foot_fits(&p, &k[pedal_index], 2) {
                        continue;
                    }
                    let anchor = crate::equipment::anchored_hihat_position(&k[hat_index], &k);
                    for i in 0..=20 {
                        let y = if i % 2 == 0 {
                            i as f64 / 2.
                        } else {
                            -(i as f64 + 1.) / 2.
                        } * 0.025;
                        if !cache.candidate() {
                            break;
                        }
                        let mut hat = k[hat_index].clone();
                        hat.position = [anchor[0], anchor[1] + y, anchor[2]];
                        if !piece_posture_fits(&p, &hat, &cache) {
                            continue;
                        }
                        let mut feet: Vec<_> =
                            k.iter().filter(|p| !hand_piece(p)).cloned().collect();
                        feet.push(hat.clone());
                        let ctx = crate::equipment_motion::EquipmentContext {
                            geometry: crate::equipment::kit_geometry(&feet),
                            kit: feet,
                        };
                        if !ctx.geometry.unresolved_piece_ids.is_empty()
                            || !equipment_posture_fits(&p, &hat, &ctx)
                        {
                            continue;
                        }
                        let mut candidate = k.clone();
                        for piece in &mut candidate {
                            if matches!(piece.instrument, Instrument::Hihat | Instrument::OpenHat) {
                                piece.position = hat.position;
                            }
                        }
                        if !valid(&candidate) {
                            continue;
                        }
                        let mut result = proposal(&p, candidate, scale, translation);
                        result.seat_offset = offset;
                        let cost = shift_cost(offset) * 4.
                            + (1. - scale).powi(2) * 0.04
                            + dx * dx
                            + dz * dz
                            + y * y;
                        seat_seeds.push(Seed {
                            offset,
                            result,
                            cost,
                        });
                        // Height is refined again with the coupled snare; retain one
                        // witness for each horizontal anchor, rather than all heights.
                        break;
                    }
                }
            }
        }
        seat_seeds.sort_by(|a, b| a.cost.total_cmp(&b.cost));
        let mut scales = Vec::new();
        for seed in seat_seeds {
            if !scales.contains(&seed.result.scale) {
                scales.push(seed.result.scale);
                seeds.push(seed);
            }
        }
    }
    cache.exhausted.set(false);
    cache.limit.set(MAX_FIT_LAYOUT_CANDIDATES);
    seeds.sort_by(|a, b| a.cost.total_cmp(&b.cost));
    // Revisit earlier anchor witnesses when later placements fail. All scales,
    // seating candidates, placement orders and backtracking share the same budget.
    for seed in &seeds {
        for rank in 0..3 {
            for rack_first in [false, true] {
                if cache.candidates.get() >= MAX_FIT_LAYOUT_CANDIDATES {
                    break;
                }
                cache.exhausted.set(false);
                cache
                    .limit
                    .set((cache.candidates.get() + 8_000).min(MAX_FIT_LAYOUT_CANDIDATES));
                cache.hands.borrow_mut().clear();
                cache.pairs.borrow_mut().clear();
                let p = translated_profile(profile, seed.offset, &seed.result.kit);
                let mut result =
                    repair_posture_order(&p, seed.result.clone(), &cache, rack_first, Some(rank));
                if result.complete && valid(&result.kit) && feet_fit(&p, &result.kit) {
                    result.piece_adjustments = baseline
                        .iter()
                        .zip(&result.kit)
                        .filter_map(|(old, new)| {
                            let uniform = transform(
                                std::slice::from_ref(old),
                                result.scale,
                                result.translation,
                            );
                            let translation =
                                std::array::from_fn(|i| new.position[i] - uniform[0].position[i]);
                            (shift_cost(translation) > 1e-18).then(|| PieceAdjustment {
                                piece_id: new.id.clone(),
                                translation,
                            })
                        })
                        .collect();
                    return cache.finish(result);
                }
            }
        }
    }
    if seeds.is_empty() {
        cache.fail("coupled-anchor", "hihat");
    }
    cache.finish(failed)
}
