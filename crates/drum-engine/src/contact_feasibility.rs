//! Common static hand-contact gate for fitting and score assignment. A witness
//! is a bounded neutral-torso pose, not proof of a collision-free temporal path.
use crate::{
    equipment_motion::{EquipmentContext, PITCH_OFFSETS},
    model::Vec3,
    posture,
    rig::{HandSolution, RigProfile},
};

/// Impact, preparation and recovery use the same physical target and tool.
pub const PROBES: [(f64, f64); 3] = [(0., 0.), (0.055, -0.2), (0.11, -0.42)];

/// A fixed-pitch witness. Runtime preservation uses this query rather than
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
pub(crate) fn at_pitch_mode(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    equipment: Option<&EquipmentContext>,
    piece: Option<&str>,
    refine: bool,
) -> Option<HandSolution> {
    let valid = |hand: &HandSolution| {
        equipment.is_none_or(|ctx| {
            crate::equipment_motion::hand_clearance(profile, limb, hand, ctx, piece)
                >= posture::MARGIN
        })
    };
    posture::hand_candidates(profile, limb, target, pitch)
        .find(valid)
        .or_else(|| {
            posture::refined_hand_candidates(profile, limb, target, pitch)
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
    PITCH_OFFSETS
        .into_iter()
        .find_map(|delta| at_pitch(profile, limb, target, pitch + delta, equipment, piece))
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
        PITCH_OFFSETS.into_iter().any(|delta| {
            at_pitch_mode(
                profile,
                limb,
                [target[0], target[1] + lift, target[2]],
                pitch + delta,
                equipment,
                piece,
                refine,
            )
            .is_some()
        })
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
    let mut costs = [(usize::MAX, f64::INFINITY); 9];
    let mut parents = vec![[0usize; 9]; hits.len()];
    let mut supported = vec![[false; 9]; hits.len()];
    for (i, hit) in hits.iter().enumerate() {
        let mut next = [(usize::MAX, f64::INFINITY); 9];
        for (j, &pitch) in PITCH_OFFSETS.iter().enumerate() {
            if !profile.reachable_at_pitch(limb, hit.target, pitch) {
                continue;
            }
            let witness = at_pitch(
                profile,
                limb,
                hit.target,
                pitch,
                Some(equipment),
                Some(&hit.piece_id),
            );
            supported[i][j] = witness.is_some();
            let missing = usize::from(witness.is_none());
            let cost = witness.map_or(0.39, |hand| {
                posture::comfort_cost(profile, limb, hit.target, &hand)
            }) + pitch * pitch * 0.005;
            if i == 0 {
                next[j] = (missing, cost);
                continue;
            }
            let limit = 2.0 * (hit.time - hits[i - 1].time) + 1e-9;
            for (k, &previous) in PITCH_OFFSETS.iter().enumerate() {
                if !costs[k].1.is_finite() || (pitch - previous).abs() > limit {
                    continue;
                }
                let candidate = (
                    costs[k].0 + missing,
                    costs[k].1 + cost + (pitch - previous).powi(2) * 0.01,
                );
                if candidate < next[j] {
                    next[j] = candidate;
                    parents[i][j] = k;
                }
            }
        }
        if next.iter().all(|v| !v.1.is_finite()) {
            return failed();
        }
        costs = next;
    }
    let mut state = (0..9)
        .min_by(|&a, &b| costs[a].partial_cmp(&costs[b]).unwrap())
        .unwrap();
    let mut anchors = vec![0.; hits.len()];
    let mut unsupported = Vec::new();
    for i in (0..hits.len()).rev() {
        anchors[i] = PITCH_OFFSETS[state];
        if !supported[i][state] {
            unsupported.push(hits[i].time);
        }
        state = parents[i][state];
    }
    unsupported.reverse();
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
        let step = 2.0 * (track[i].time - track[i - 1].time);
        lower[i] = lower[i].max(lower[i - 1] - step);
        upper[i] = upper[i].min(upper[i - 1] + step);
    }
    for i in (0..track.len().saturating_sub(1)).rev() {
        let step = 2.0 * (track[i + 1].time - track[i].time);
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
            let step = 2.0 * (track[i].time - track[i - 1].time);
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
            },
            HandControlFrame {
                forearm: [-0.5, -0.5, d],
                palm: [-d, d, 0.],
                desired_palm: [-d, -d, 0.],
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
        };
        p.hand_frames = Some([frame.clone(), frame]);
        let ctx = EquipmentContext {
            kit: Vec::new(),
            geometry: crate::equipment::EquipmentGeometry {
                version: 1,
                bodies: Vec::new(),
                supports: Vec::new(),
                pedals: Vec::new(),
                mechanisms: Vec::new(),
                unresolved_piece_ids: Vec::new(),
            },
        };
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
}
