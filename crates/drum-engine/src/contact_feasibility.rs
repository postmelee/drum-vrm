//! Common static hand-contact gate for fitting and score assignment. A witness
//! is a bounded neutral-torso pose, not proof of a collision-free temporal path.
use crate::{
    equipment_motion::{EquipmentContext, PITCH_OFFSETS},
    model::Vec3,
    posture,
    rig::{HandSolution, RigProfile},
};
use std::collections::HashMap;

// Preserve safe witnesses first, then exact contact, physical tool clearance and
// posture deficit. An unsupported pose must not win merely because pitch zero
// has the smallest stylistic cost while its wrist is outside the arm annulus.
type ImpactRank = (usize, usize, f64, f64, f64);

fn impact_rank(
    profile: &RigProfile,
    equipment: &EquipmentContext,
    hit: &crate::model::Hit,
    limb: usize,
    pitch: f64,
) -> ImpactRank {
    let preference = pitch * pitch * 0.005;
    if let Some(hand) = source_at_pitch(
        profile,
        limb,
        hit.target,
        pitch,
        Some(equipment),
        Some(&hit.piece_id),
    ) {
        return (
            0,
            0,
            0.,
            0.,
            posture::comfort_cost(profile, limb, hit.target, &hand) + preference,
        );
    }
    let best = (-16..=16)
        .filter_map(|step| {
            crate::equipment_motion::HandEnvironment {
                equipment: Some(equipment),
                intended_piece: Some(&hit.piece_id),
            }
            .solve(profile, limb, hit.target, pitch, step as f64 * 0.05, false)
        })
        .map(|hand| {
            let tool_clearance = crate::equipment_motion::tool_clearance(
                profile,
                limb,
                &hand,
                equipment,
                Some(&hit.piece_id),
            );
            (
                usize::from(!hand.reachable),
                (posture::MARGIN - tool_clearance).max(0.).powi(2)
                    + crate::clearance::core_penetration(profile, limb, &hand).powi(2),
                posture::anatomical_violation(profile, limb, hit.target, &hand),
                posture::comfort_cost(profile, limb, hit.target, &hand),
            )
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or((1, f64::INFINITY, f64::INFINITY, 0.39));
    (1, best.0, best.1, best.2, best.3 + preference)
}

/// Impact, preparation and recovery use the same physical target and tool.
pub const PROBES: [(f64, f64); 3] = [(0., 0.), (0.055, -0.2), (0.11, -0.42)];

/// A witness for the requested pitch prior, including its actual selected tool
/// orientation. Runtime preservation uses this query rather than
/// borrowing a feasible but unevaluated pitch from the bounded static search.
pub fn at_pitch(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> Option<HandSolution> {
    at_pitch_mode(profile, limb, target, pitch, equipment, piece, true)
}
/// Source anchors are retained by all later phase curves. Their certificate
/// must include the same ordinary-core constraint as the hand graph, while
/// intermediate approach searches retain their existing diagnostic fallback.
pub(crate) fn source_at_pitch(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> Option<HandSolution> {
    at_pitch_search_with_core(
        profile,
        limb,
        target,
        pitch,
        equipment,
        piece,
        true,
        posture::HandSearch::Full,
        true,
    )
}

pub(crate) fn at_pitch_mode(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
) -> Option<HandSolution> {
    at_pitch_search(
        profile,
        limb,
        target,
        pitch,
        equipment,
        piece,
        refine,
        posture::HandSearch::Full,
    )
}
// Keep the query inputs identical across the two internal candidate families.
#[allow(clippy::too_many_arguments)]
fn at_pitch_search(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
    search: posture::HandSearch,
) -> Option<HandSolution> {
    at_pitch_search_with_core(
        profile, limb, target, pitch, equipment, piece, refine, search, false,
    )
}

#[allow(clippy::too_many_arguments)]
fn at_pitch_search_with_core(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
    search: posture::HandSearch,
    require_core: bool,
) -> Option<HandSolution> {
    let valid = |hand: &HandSolution| {
        (!require_core || crate::clearance::core_penetration(profile, limb, hand) == 0.)
            && equipment.is_none_or(|ctx| {
                crate::equipment_motion::tool_clearance(profile, limb, hand, ctx, piece)
                    >= posture::MARGIN
            })
    };
    let environment = crate::equipment_motion::HandEnvironment {
        equipment,
        intended_piece: piece,
    };
    posture::hand_candidates_in(profile, limb, target, pitch, search, environment)
        .find(valid)
        .or_else(|| {
            posture::refined_hand_candidates_in(profile, limb, target, pitch, search, environment)
                .take(if refine { 28 } else { 0 })
                .find(valid)
        })
}

pub fn candidate(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> Option<HandSolution> {
    candidate_mode(profile, limb, target, pitch, equipment, piece, true)
}
fn candidate_mode(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
) -> Option<HandSolution> {
    posture::HAND_SEARCHES.into_iter().find_map(|search| {
        PITCH_OFFSETS.into_iter().find_map(|delta| {
            at_pitch_search(
                profile,
                limb,
                target,
                pitch + delta,
                equipment,
                piece,
                refine,
                search,
            )
        })
    })
}

/// Fit guarantees neutral facing. Optional runtime torso motion must preserve
/// each contact using its actual selected pitch; it cannot supply hidden reach.
pub fn hand_probe_fits(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> bool {
    hand_probe_mode(profile, limb, target, equipment, piece, true)
}

/// Prefer a hand with complete ordinary-body approach witnesses before musical
/// sticking costs. A bounded neutral-body miss is not a physical impossibility:
/// retain it as a diagnosed fallback when the other hand is unavailable (torso
/// coordination may still clear it). In particular, never drop source hits or
/// reject a whole kit merely because this additional preference has no witness.
pub fn has_clear_body_approach(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> bool {
    if limb >= 2 || profile.hand_frames.is_none() || equipment.is_none() {
        return true;
    }
    PROBES.into_iter().all(|(lift, pitch)| {
        posture::HAND_SEARCHES.into_iter().any(|search| {
            PITCH_OFFSETS.into_iter().any(|delta| {
                at_pitch_search_with_core(
                    profile,
                    limb,
                    [target[0], target[1] + lift, target[2]],
                    pitch + delta,
                    equipment,
                    piece,
                    true,
                    search,
                    true,
                )
                .is_some()
            })
        })
    })
}
pub(crate) fn hand_probe_mode(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
) -> bool {
    if limb >= 2 {
        return false;
    }
    if profile.hand_frames.is_none() {
        return profile.reachable(limb, target);
    }
    PROBES.into_iter().all(|(lift, pitch)| {
        let point = [target[0], target[1] + lift, target[2]];
        // Rotation cannot place the contact outside the arm-plus-held-tool
        // triangle envelope. Reject that impossible probe before enumerating
        // wrist approaches; a positive envelope result still needs the exact
        // existing posture/equipment witness below.
        if !profile.may_reach_contact(limb, point) {
            return false;
        }
        candidate_mode(profile, limb, point, pitch, equipment, piece, refine).is_some()
    })
}

pub fn impact_comfort_cost(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
) -> f64 {
    candidate(profile, limb, target, 0., equipment, piece).map_or(0., |hand| {
        posture::comfort_cost(profile, limb, target, &hand)
    })
}

// Refine only the transitions between a certified and unsupported coarse
// pitch. Three bisections add at most 24 certified candidates to the nine-value
// baseline; unsupported midpoints never become new diagnostic fallbacks.
const IMPACT_PITCH_BOUNDARY_STEPS: usize = 3;
const MAX_IMPACT_PITCH_CANDIDATES: usize =
    PITCH_OFFSETS.len() + (PITCH_OFFSETS.len() - 1) * IMPACT_PITCH_BOUNDARY_STEPS;
type ContactKey<'a> = ([u64; 3], &'a str);
struct ImpactPitches<'a> {
    hit: &'a crate::model::Hit,
    values: Vec<(f64, ImpactRank)>,
}
fn contact_key(hit: &crate::model::Hit) -> ContactKey<'_> {
    (hit.target.map(f64::to_bits), hit.piece_id.as_str())
}
fn refine_impact_pitch_boundaries(
    candidates: &mut ImpactPitches<'_>,
    profile: &RigProfile,
    equipment: &EquipmentContext,
    limb: usize,
) {
    let mut coarse = candidates.values.clone();
    coarse.sort_by(|a, b| a.0.total_cmp(&b.0));
    for pair in coarse.windows(2) {
        let [(mut lo, left), (mut hi, right)] = [pair[0], pair[1]];
        let supported_left = left.0 == 0;
        if supported_left == (right.0 == 0) {
            continue;
        }
        for _ in 0..IMPACT_PITCH_BOUNDARY_STEPS {
            let pitch = (lo + hi) * 0.5;
            let rank = impact_rank(profile, equipment, candidates.hit, limb, pitch);
            if rank.0 == 0 && candidates.values.len() < MAX_IMPACT_PITCH_CANDIDATES {
                candidates.values.push((pitch, rank));
            }
            // This locates a sampled boundary, not an assumed monotone safe
            // interval: every retained pitch has its own complete witness.
            if (rank.0 == 0) == supported_left {
                lo = pitch;
            } else {
                hi = pitch;
            }
        }
    }
}
fn impact_pitch_schedule(
    hits: &[&crate::model::Hit],
    ranks: &HashMap<ContactKey<'_>, ImpactPitches<'_>>,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let choices: Vec<_> = hits
        .iter()
        .map(|hit| &ranks[&contact_key(hit)].values)
        .collect();
    let unreachable = (
        usize::MAX,
        usize::MAX,
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
    );
    let mut costs = Vec::new();
    let mut parents: Vec<Vec<usize>> = choices.iter().map(|values| vec![0; values.len()]).collect();
    for (i, values) in choices.iter().enumerate() {
        let mut next = vec![unreachable; values.len()];
        for (j, &(pitch, rank)) in values.iter().enumerate() {
            if i == 0 {
                next[j] = rank;
                continue;
            }
            let limit =
                crate::equipment_motion::PITCH_RATE * (hits[i].time - hits[i - 1].time) + 1e-9;
            for (k, &(previous, _)) in choices[i - 1].iter().enumerate() {
                let cost: ImpactRank = costs[k];
                if !cost.4.is_finite() || (pitch - previous).abs() > limit {
                    continue;
                }
                let candidate = (
                    cost.0 + rank.0,
                    cost.1 + rank.1,
                    cost.2 + rank.2,
                    cost.3 + rank.3,
                    cost.4 + rank.4 + (pitch - previous).powi(2) * 0.01,
                );
                if candidate < next[j] {
                    next[j] = candidate;
                    parents[i][j] = k;
                }
            }
        }
        if next.iter().all(|v| !v.4.is_finite()) {
            return None;
        }
        costs = next;
    }
    let mut state = (0..costs.len()).min_by(|&a, &b| costs[a].partial_cmp(&costs[b]).unwrap())?;
    let mut anchors = vec![0.; hits.len()];
    let mut unsupported = Vec::new();
    for i in (0..hits.len()).rev() {
        let (pitch, rank) = choices[i][state];
        anchors[i] = pitch;
        if rank.0 != 0 {
            unsupported.push(hits[i].time);
        }
        state = parents[i][state];
    }
    unsupported.reverse();
    Some((anchors, unsupported))
}

/// Preserve the maximum number of feasible neutral impact witnesses within the
/// existing pitch-rate budget, then prefer comfortable solutions. An incompatible
/// fill does not discard anchors in the rest of the song. Returned source times
/// have no retained safe witness and must remain explicit diagnostics.
pub fn preserve_impact_pitches(
    track: &mut [crate::equipment_motion::PathSample],
    profile: &RigProfile,
    equipment: &EquipmentContext,
    hits: &[&crate::model::Hit],
    limb: usize,
) -> Vec<f64> {
    if hits.is_empty() || profile.hand_frames.is_none() {
        return Vec::new();
    }
    let failed = || hits.iter().map(|h| h.time).collect();
    // Repeated notes share one neutral contact query, including any bounded
    // refinement. The ordinary schedule retains its original nine candidates
    // and ordering; successful songs do not pay for another search.
    let mut ranks = HashMap::new();
    for &hit in hits {
        if !profile.may_reach_contact(limb, hit.target) {
            return failed();
        }
        ranks
            .entry(contact_key(hit))
            .or_insert_with(|| ImpactPitches {
                hit,
                values: PITCH_OFFSETS
                    .map(|pitch| (pitch, impact_rank(profile, equipment, hit, limb, pitch)))
                    .to_vec(),
            });
    }
    let Some((mut anchors, mut unsupported)) = impact_pitch_schedule(hits, &ranks) else {
        return failed();
    };
    if !unsupported.is_empty() {
        for candidates in ranks.values_mut() {
            refine_impact_pitch_boundaries(candidates, profile, equipment, limb);
        }
        if let Some(refined) = impact_pitch_schedule(hits, &ranks) {
            (anchors, unsupported) = refined;
        }
    }
    let mut lower = vec![-0.60f64; track.len()];
    let mut upper = vec![0.60f64; track.len()];
    for (hit, &pitch) in hits.iter().zip(&anchors) {
        let Ok(i) = track.binary_search_by(|s| s.time.total_cmp(&hit.time)) else {
            return failed();
        };
        lower[i] = pitch;
        upper[i] = pitch;
    }
    for i in 1..track.len() {
        let step = crate::equipment_motion::PITCH_RATE * (track[i].time - track[i - 1].time);
        lower[i] = lower[i].max(lower[i - 1] - step);
        upper[i] = upper[i].min(upper[i - 1] + step);
    }
    for i in (0..track.len().saturating_sub(1)).rev() {
        let step = crate::equipment_motion::PITCH_RATE * (track[i + 1].time - track[i].time);
        lower[i] = lower[i].max(lower[i + 1] - step);
        upper[i] = upper[i].min(upper[i + 1] + step);
    }
    if lower.iter().zip(&upper).any(|(a, b)| a > &(b + 1e-9)) {
        return failed();
    }
    for i in 0..track.len() {
        let mut lo = lower[i];
        let mut hi = upper[i].max(lo);
        if i > 0 {
            let step = crate::equipment_motion::PITCH_RATE * (track[i].time - track[i - 1].time);
            lo = lo.max(track[i - 1].pitch[limb] - step);
            hi = hi.min(track[i - 1].pitch[limb] + step).max(lo);
        }
        track[i].pitch[limb] = track[i].pitch[limb].clamp(lo, hi);
    }
    unsupported
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig::{pitch_rotation, HandControlFrame};
    fn native_reference() -> (RigProfile, Vec<crate::model::KitPiece>) {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        (
            serde_json::from_value(fixture["profile"].clone()).unwrap(),
            serde_json::from_value(fixture["kit"].clone()).unwrap(),
        )
    }

    #[test]
    fn source_pitch_certificate_rejects_body_crossing_while_retaining_safe_existing_pitch() {
        let (_, mut kit) = native_reference();
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-natural-grip30.json")).unwrap();
        let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        // Authored open-hat contact on the current forward left-kit layout.
        // The old pitch certificate admitted a reachable/tool-clear wrist
        // whose upper arm traversed the ordinary torso.
        for piece in &mut kit {
            if matches!(
                piece.instrument,
                crate::model::Instrument::Hihat
                    | crate::model::Instrument::OpenHat
                    | crate::model::Instrument::PedalHat
            ) {
                piece.position[2] += 0.08;
            }
        }
        let ctx = EquipmentContext::for_kit(&kit).unwrap();
        let hat = kit.iter().find(|p| p.id == "hihat").unwrap();
        let observer = std::array::from_fn(|i| (profile.roots[0][i] + profile.roots[1][i]) * 0.5);
        let mut target = crate::contact::target(hat, "bow", observer);
        target[1] += 0.025;
        let old = at_pitch(&profile, 1, target, -0.6, Some(&ctx), Some("openHat")).unwrap();
        assert!(crate::clearance::core_penetration(&profile, 1, &old) > 0.01);
        assert!(source_at_pitch(&profile, 1, target, -0.6, Some(&ctx), Some("openHat")).is_none());
        let safe = source_at_pitch(&profile, 1, target, 0., Some(&ctx), Some("openHat")).unwrap();
        assert_eq!(crate::clearance::core_penetration(&profile, 1, &safe), 0.);
        assert!(posture::anatomical(&profile, 1, target, &safe));
        assert!(profile.hand_orientation_policy_valid(1, safe.rotation));
        assert!(crate::motion::distance(safe.tip, target) < 1e-9);
    }

    #[test]
    fn body_crossing_legacy_arm_does_not_hide_the_tool_approach_search() {
        let (profile, kit) = native_reference();
        let ctx = EquipmentContext::for_kit(&kit).unwrap();
        let hihat = kit.iter().find(|piece| piece.id == "hihat").unwrap();
        let observer = std::array::from_fn(|i| (profile.roots[0][i] + profile.roots[1][i]) * 0.5);
        let target = crate::contact::target(hihat, "edge", observer);
        // This older lateral hat fixture had an exact arm-preserving witness,
        // but its forearm passed through the ordinary torso. It must no longer
        // prevent the existing bounded orientation fallback from being tried.
        assert!(PITCH_OFFSETS.iter().all(|&pitch| at_pitch_search(
            &profile,
            1,
            target,
            pitch,
            Some(&ctx),
            Some("hihat"),
            true,
            posture::HandSearch::PreserveArm,
        )
        .is_none()));
        let selected = candidate(&profile, 1, target, 0., Some(&ctx), Some("hihat")).unwrap();
        assert!(profile.hand_orientation_policy_valid(1, selected.rotation));
        assert!(posture::anatomical(&profile, 1, target, &selected));
        assert!(
            crate::motion::distance(
                crate::motion::add(selected.end, profile.offset(1, selected.rotation)),
                target
            ) < 1e-10
        );
        assert!(
            crate::equipment_motion::tool_clearance(&profile, 1, &selected, &ctx, Some("hihat"))
                >= posture::MARGIN
        );
    }

    #[test]
    fn arm_first_frontier_retains_existing_physical_contact_certificates() {
        let (profile, kit) = native_reference();
        let ctx = EquipmentContext::for_kit(&kit).unwrap();
        let observer = std::array::from_fn(|i| (profile.roots[0][i] + profile.roots[1][i]) * 0.5);
        let mut retained = 0;
        for piece in &kit {
            for articulation in crate::contact::articulations_for_fit(piece) {
                if matches!(
                    piece.instrument,
                    crate::model::Instrument::Kick
                        | crate::model::Instrument::PedalHat
                        | crate::model::Instrument::OpenHat
                ) {
                    continue;
                }
                let target = crate::contact::target(piece, articulation, observer);
                for limb in 0..2 {
                    for (lift, pitch) in PROBES {
                        let tip = crate::motion::add(target, [0., lift, 0.]);
                        let full = PITCH_OFFSETS.into_iter().find_map(|delta| {
                            at_pitch(
                                &profile,
                                limb,
                                tip,
                                pitch + delta,
                                Some(&ctx),
                                Some(&piece.id),
                            )
                        });
                        let selected =
                            candidate(&profile, limb, tip, pitch, Some(&ctx), Some(&piece.id));
                        if full.is_some() {
                            retained += 1;
                            assert!(
                                selected.is_some(),
                                "{}/{articulation}/{limb}/{lift}",
                                piece.id
                            );
                        }
                        if let Some(hand) = selected {
                            assert!(posture::anatomical(&profile, limb, tip, &hand));
                            assert!(profile.hand_orientation_policy_valid(limb, hand.rotation));
                            assert!(
                                crate::equipment_motion::tool_clearance(
                                    &profile,
                                    limb,
                                    &hand,
                                    &ctx,
                                    Some(&piece.id)
                                ) >= posture::MARGIN
                            );
                        }
                    }
                }
            }
        }
        assert!(retained > 40);
    }
    fn profile() -> RigProfile {
        let mut profile = RigProfile::primitive();
        profile.id = "generic-pitched-contact".into();
        profile.roots[0] = [-0.08, 1., 0.];
        profile.roots[1] = [0.08, 1., 0.];
        profile.lengths[0] = [0.25, 0.22];
        profile.lengths[1] = [0.25, 0.22];
        profile.grip_offsets[0] = [0., -0.045, 0.03];
        profile.grip_offsets[1] = [0., -0.045, 0.03];
        profile.tip_offsets[0] = [0., 0., 0.31];
        profile.tip_offsets[1] = [0., 0., 0.31];
        profile.contact_rotations[0] = pitch_rotation([0., 0.5, 0., 3_f64.sqrt() / 2.], -0.12);
        profile.contact_rotations[1] = pitch_rotation([0., -0.5, 0., 3_f64.sqrt() / 2.], -0.12);
        let d = 0.5_f64.sqrt();
        profile.hand_frames = Some([
            HandControlFrame {
                forearm: [0.5, -0.5, d],
                palm: [d, d, 0.],
                desired_palm: [d, -d, 0.],
                palm_roll_range: None,
                palm_reference: None,
            },
            HandControlFrame {
                forearm: [-0.5, -0.5, d],
                palm: [-d, d, 0.],
                desired_palm: [-d, -d, 0.],
                palm_roll_range: None,
                palm_reference: None,
            },
        ]);
        profile
    }
    #[test]
    fn pitched_annulus_matches_hand_solver_in_both_directions() {
        let profile = profile();
        for limb in 0..2 {
            for pitch in PITCH_OFFSETS {
                for z in [0.2, 0.5, 0.7, 0.8, 1.1] {
                    let tip = [0., 0.85, z];
                    assert_eq!(
                        profile.reachable_at_pitch(limb, tip, pitch),
                        profile.solve_hand(limb, tip, pitch).unwrap().reachable
                    );
                }
            }
        }
        assert!(!profile.reachable_at_pitch(4, [0.; 3], 0.));
        assert!(!profile.reachable_at_pitch(0, [f64::NAN, 0., 0.], 0.));
    }
    #[test]
    fn exact_pitch_reach_does_not_alias_the_neutral_annulus() {
        let profile = profile();
        let mut difference = false;
        for z in (20..100).map(|v| v as f64 / 100.) {
            let tip = [0., 0.85, z];
            for pitch in PITCH_OFFSETS {
                difference |=
                    profile.reachable(0, tip) != profile.reachable_at_pitch(0, tip, pitch);
            }
        }
        assert!(
            difference,
            "pitch must participate in the compound-link reach gate"
        );
    }

    #[test]
    fn preparation_gate_requires_a_complete_safe_probe_set() {
        let mut profile = RigProfile::primitive();
        for side in 0..2 {
            profile.roots[side] = [if side == 0 { -0.19 } else { 0.19 }, 1.235, 0.06];
            profile.lengths[side] = [0.3, 0.27];
        }
        let frame = HandControlFrame {
            forearm: [0., 0., 1.],
            palm: [0., 1., 0.],
            desired_palm: [0., 1., 0.],
            palm_roll_range: None,
            palm_reference: None,
        };
        profile.hand_frames = Some([frame.clone(), frame]);
        assert!(hand_probe_fits(&profile, 0, [-0.15, 0.9, 0.55], None, None));
        assert!(!hand_probe_fits(&profile, 0, [0., 1.8, 1.2], None, None));
        assert!(!hand_probe_fits(
            &profile,
            2,
            [-0.15, 0.9, 0.55],
            None,
            None
        ));
    }

    #[test]
    fn probe_envelope_preserves_rotated_tool_witnesses_and_only_rejects_impossible_reach() {
        let (base, _) = native_reference();
        let mut witnessed = 0;
        let mut outside = 0;
        let mut inside_but_unplayable = 0;
        for grip in [[0.; 3], [0.08, -0.03, 0.12]] {
            let mut profile = base.clone();
            profile.grip_offsets[0] = grip;
            for relative in [
                [0., -0.35, 0.45],
                [0.15, -0.25, 0.60],
                [0., -0.25, 0.82],
                [-0.35, -0.10, 0.55],
                [0.35, -0.10, 0.55],
                [0., 0., 0.],
                [0., 0.80, 0.80],
                [0., -0.05, 1.15],
            ] {
                let target = crate::motion::add(profile.roots[0], relative);
                // Compare against actual bounded wrist witnesses, not the
                // default orientation or a bare-bone length approximation.
                let mut exact = true;
                for (lift, pitch) in PROBES {
                    let point = [target[0], target[1] + lift, target[2]];
                    let hand = candidate_mode(&profile, 0, point, pitch, None, None, true);
                    let envelope = profile.may_reach_contact(0, point);
                    if let Some(hand) = hand {
                        witnessed += 1;
                        assert!(envelope);
                        assert!(crate::motion::distance(hand.tip, point) < 1e-9);
                    } else {
                        exact = false;
                        if envelope {
                            inside_but_unplayable += 1;
                        } else {
                            outside += 1;
                        }
                    }
                }
                assert_eq!(
                    hand_probe_mode(&profile, 0, target, None, None, true),
                    exact
                );
            }
        }
        assert!(witnessed > 0 && outside > 0 && inside_but_unplayable > 0);
    }
    #[test]
    fn impact_anchors_survive_rate_projection_with_exact_times_and_safe_poses() {
        let mut p = RigProfile::primitive();
        for side in 0..2 {
            p.roots[side] = [if side == 0 { -0.19 } else { 0.19 }, 1.235, 0.06];
            p.lengths[side] = [0.3, 0.27];
        }
        let frame = HandControlFrame {
            forearm: [0., 0., 1.],
            palm: [0., 1., 0.],
            desired_palm: [0., 1., 0.],
            palm_roll_range: None,
            palm_reference: None,
        };
        p.hand_frames = Some([frame.clone(), frame]);
        let ctx = EquipmentContext::new(
            Vec::new(),
            crate::equipment::EquipmentGeometry {
                version: 1,
                bodies: Vec::new(),
                supports: Vec::new(),
                pedals: Vec::new(),
                mechanisms: Vec::new(),
                unresolved_piece_ids: Vec::new(),
            },
        );
        let mut hits: Vec<_> = [0.2, 0.5, 0.8]
            .into_iter()
            .enumerate()
            .map(|(id, time)| crate::model::Hit {
                id,
                time,
                time_us: (time * 1e6) as u64,
                tick: id as u64 * 480,
                track: 0,
                event_index: id,
                channel: 9,
                note: 38,
                velocity: 90,
                instrument: crate::model::Instrument::Snare,
                piece_id: "snare".into(),
                articulation: "center".into(),
                source_event_id: format!("0:{id}"),
                limb: Some(0),
                target: [-0.15, 0.9, 0.55],
            })
            .collect();
        let mut track: Vec<_> = (0..=10)
            .map(|i| crate::equipment_motion::PathSample {
                time: i as f64 / 10.,
                lift: [0.; 2],
                pitch: [if i % 2 == 0 { -0.6 } else { 0.6 }; 2],
            })
            .collect();
        let times: Vec<_> = track.iter().map(|s| s.time).collect();
        let errors =
            preserve_impact_pitches(&mut track, &p, &ctx, &hits.iter().collect::<Vec<_>>(), 0);
        assert!(errors.is_empty());
        assert_eq!(times, track.iter().map(|s| s.time).collect::<Vec<_>>());
        assert!(track
            .windows(2)
            .all(|w| (w[1].pitch[0] - w[0].pitch[0]).abs() <= 2. * (w[1].time - w[0].time) + 1e-9));
        for hit in &hits {
            let pitch = crate::equipment_motion::sample(&track, hit.time, 0).1;
            assert!(at_pitch(&p, 0, hit.target, pitch, Some(&ctx), Some("snare")).is_some());
        }
        let before = track.clone();
        hits[1].target = [8.; 3];
        assert!(!preserve_impact_pitches(
            &mut track,
            &p,
            &ctx,
            &hits.iter().collect::<Vec<_>>(),
            0
        )
        .is_empty());
        assert_eq!(
            track, before,
            "no fabricated trajectory when no reachable pitch exists"
        );
    }

    #[test]
    fn off_grid_safe_pitch_boundaries_preserve_fast_contacts_without_relaxing_rates() {
        let (profile, mut kit) = native_reference();
        kit.retain(|piece| matches!(piece.id.as_str(), "tomLow" | "crashRight"));
        kit.iter_mut()
            .find(|piece| piece.id == "crashRight")
            .unwrap()
            .position[1] = 1.30;
        let ctx = EquipmentContext::for_kit(&kit).unwrap();
        let observer = std::array::from_fn(|i| (profile.roots[0][i] + profile.roots[1][i]) * 0.5);
        // An original numeric fixture: two repeated 60 ms floor-to-crash moves.
        // The coarse safe sets cannot meet the existing 2 rad/s limit, although
        // a certified pitch near the floor's boundary can. Repetition also
        // exercises reuse of the same contact's refined candidates.
        let hits: Vec<_> = [
            ("tomLow", 0.20),
            ("crashRight", 0.26),
            ("tomLow", 0.80),
            ("crashRight", 0.86),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, (piece_id, time))| {
            let piece = kit.iter().find(|piece| piece.id == piece_id).unwrap();
            let articulation = if piece_id == "tomLow" {
                "center"
            } else {
                "crash"
            };
            crate::model::Hit {
                id,
                time,
                time_us: (time * 1e6) as u64,
                tick: id as u64 * 120,
                track: 0,
                event_index: id,
                channel: 9,
                note: if piece_id == "tomLow" { 43 } else { 57 },
                velocity: 100,
                instrument: piece.instrument,
                piece_id: piece.id.clone(),
                articulation: articulation.into(),
                source_event_id: format!("0:{id}"),
                limb: Some(1),
                target: crate::contact::target(piece, articulation, observer),
            }
        })
        .collect();
        let coarse: Vec<Vec<_>> = hits[..2]
            .iter()
            .map(|hit| {
                PITCH_OFFSETS
                    .into_iter()
                    .filter(|&pitch| {
                        at_pitch(
                            &profile,
                            1,
                            hit.target,
                            pitch,
                            Some(&ctx),
                            Some(&hit.piece_id),
                        )
                        .is_some()
                    })
                    .collect()
            })
            .collect();
        assert!(coarse.iter().all(|pitches| !pitches.is_empty()));
        let limit = crate::equipment_motion::PITCH_RATE * (hits[1].time - hits[0].time);
        assert!(!coarse[0]
            .iter()
            .any(|a| coarse[1].iter().any(|b| (a - b).abs() <= limit)));

        let mut track: Vec<_> = [0., 0.20, 0.23, 0.26, 0.50, 0.80, 0.83, 0.86, 1.]
            .into_iter()
            .map(|time| crate::equipment_motion::PathSample {
                time,
                lift: [0.; 2],
                pitch: [0.; 2],
            })
            .collect();
        let source = serde_json::to_string(&hits).unwrap();
        let times: Vec<_> = track.iter().map(|sample| sample.time).collect();
        assert!(preserve_impact_pitches(
            &mut track,
            &profile,
            &ctx,
            &hits.iter().collect::<Vec<_>>(),
            1
        )
        .is_empty());
        assert_eq!(source, serde_json::to_string(&hits).unwrap());
        assert_eq!(
            times,
            track.iter().map(|sample| sample.time).collect::<Vec<_>>()
        );
        assert!(track.windows(2).all(|pair| {
            (pair[1].pitch[1] - pair[0].pitch[1]).abs()
                <= crate::equipment_motion::PITCH_RATE * (pair[1].time - pair[0].time) + 1e-9
        }));
        for hit in &hits {
            let pitch = crate::equipment_motion::sample(&track, hit.time, 1).1;
            assert!(at_pitch(
                &profile,
                1,
                hit.target,
                pitch,
                Some(&ctx),
                Some(&hit.piece_id)
            )
            .is_some());
        }
        let first = crate::equipment_motion::sample(&track, hits[0].time, 1).1;
        assert!(PITCH_OFFSETS
            .iter()
            .all(|coarse| (first - coarse).abs() > 1e-8));
    }

    #[test]
    fn unsupported_anchors_preserve_possible_contact_before_stylistic_pitch_cost() {
        let p = profile();
        let target = [0.2, 1.0, 0.8];
        assert!(!p.solve_hand(1, target, 0.).unwrap().reachable);
        // An independently authored obstruction makes every contact unsafe.
        // The failure must remain reported, but zero pitch must not introduce
        // an avoidable reach miss on top of that physical obstruction.
        let ctx = EquipmentContext::new(
            Vec::new(),
            crate::equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![crate::equipment::EquipmentBox {
                    piece_id: "obstruction".into(),
                    center: [0., 1., 0.4],
                    half_extents: [2.; 3],
                    rotation: None,
                    kind: "body",
                }],
                supports: Vec::new(),
                pedals: Vec::new(),
                mechanisms: Vec::new(),
                unresolved_piece_ids: Vec::new(),
            },
        );
        let hits: Vec<_> = [0.2, 0.5, 0.8]
            .into_iter()
            .enumerate()
            .map(|(id, time)| crate::model::Hit {
                id,
                time,
                time_us: (time * 1e6) as u64,
                tick: id as u64 * 480,
                track: 0,
                event_index: id,
                channel: 9,
                note: 51,
                velocity: 90,
                instrument: crate::model::Instrument::Ride,
                piece_id: "ride".into(),
                articulation: "bell".into(),
                source_event_id: format!("0:{id}"),
                limb: Some(1),
                target,
            })
            .collect();
        let mut track: Vec<_> = (0..=10)
            .map(|i| crate::equipment_motion::PathSample {
                time: i as f64 / 10.,
                lift: [0.; 2],
                pitch: [0.; 2],
            })
            .collect();
        let original_hits = serde_json::to_string(&hits).unwrap();
        let times: Vec<_> = track.iter().map(|s| s.time).collect();
        let errors =
            preserve_impact_pitches(&mut track, &p, &ctx, &hits.iter().collect::<Vec<_>>(), 1);
        assert_eq!(errors, vec![0.2, 0.5, 0.8]);
        assert_eq!(serde_json::to_string(&hits).unwrap(), original_hits);
        assert_eq!(track.iter().map(|s| s.time).collect::<Vec<_>>(), times);
        assert!(track
            .windows(2)
            .all(|w| (w[1].pitch[1] - w[0].pitch[1]).abs() <= 2. * (w[1].time - w[0].time) + 1e-9));
        for hit in &hits {
            let pitch = crate::equipment_motion::sample(&track, hit.time, 1).1;
            assert!(at_pitch(&p, 1, target, pitch, Some(&ctx), Some("ride")).is_none());
            assert!((-16..=16).any(|step| {
                let hand = p
                    .solve_hand_with_swivel(1, target, pitch, step as f64 * 0.05)
                    .unwrap();
                hand.reachable && crate::motion::distance(hand.tip, target) < 1e-9
            }));
        }
    }
}
