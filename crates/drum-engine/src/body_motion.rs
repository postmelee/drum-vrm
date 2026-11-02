//! Bounded phrase-scale facing, shared with numerical shoulder roots and renderer.
//! This is a conservative procedural candidate, not a learned whole-body solver.
use crate::{model::Vec3, rig::RigProfile};
pub const STEP: f64 = 1.0 / 12.0;

/// Anticipate upcoming targets within the same bounded facing candidates used
/// for recovery. A one-way greedy pass reacts only after a target changes. The
/// reverse pass supplies the other side of that transition; averaging the two
/// bounded tracks preserves their yaw and neighboring-step limits.
pub fn plan(profile: &RigProfile, samples: &[([Vec3; 2], [f64; 2])]) -> Vec<f64> {
    let mut previous = 0.;
    let mut result: Vec<_> = samples
        .iter()
        .map(|&(tips, pitches)| {
            previous = candidate(profile, tips, pitches, previous);
            previous
        })
        .collect();
    let Some(mut next) = result.last().copied() else {
        return result;
    };
    for (&(tips, pitches), yaw) in samples.iter().zip(&mut result).rev() {
        next = candidate(profile, tips, pitches, next);
        *yaw = (*yaw + next) * 0.5;
    }
    // A backward plan has no neutral predecessor. Restore the existing initial
    // anchor and propagate its step bound without moving any timeline sample.
    result[0] = result[0].clamp(-0.06, 0.06);
    for i in 1..result.len() {
        result[i] = result[i].clamp(result[i - 1] - 0.06, result[i - 1] + 0.06);
    }
    result
}

pub fn candidate(profile: &RigProfile, tips: [Vec3; 2], pitches: [f64; 2], previous: f64) -> f64 {
    if profile.torso_pivot.is_none() {
        return 0.0;
    }
    let center = (profile.roots[0][0] + profile.roots[1][0]) * 0.5;
    let forward = (profile.roots[0][2] + profile.roots[1][2]) * 0.5;
    let desired = (((tips[0][0] + tips[1][0]) * 0.5 - center)
        .atan2(((tips[0][2] + tips[1][2]) * 0.5 - forward).max(0.15))
        * 0.3)
        .clamp(-0.24, 0.24);
    let mut best = (f64::INFINITY, previous);
    for step in -6..=6 {
        let yaw = (previous + step as f64 * 0.01).clamp(-0.24, 0.24);
        let p = profile.turned(yaw);
        let mut cost = (yaw - desired).powi(2) * 0.8 + (yaw - previous).powi(2) * 3.0;
        if p.hand_frames.is_some() {
            cost += crate::hand_coordination::unconstrained_cost(&p, tips, pitches);
        } else {
            for (limb, tip) in tips.iter().enumerate() {
                if !p.reachable(limb, *tip) {
                    cost += 1e9;
                }
            }
        }
        if cost < best.0 {
            best = (cost, yaw);
        }
    }
    best.1
}
pub fn at(samples: &[f64], time: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let frame = ((time + crate::model::PRE_ROLL) / STEP).max(0.0);
    let i = (frame.floor() as usize).min(samples.len() - 1);
    let j = (i + 1).min(samples.len() - 1);
    let t = (frame - i as f64).clamp(0.0, 1.0);
    // C1 interpolation with bounded neighboring samples, independent of render order.
    samples[i] + (samples[j] - samples[i]) * t * t * (3.0 - 2.0 * t)
}

/// Keep both bracketing knots neutral so an off-grid held interval has exactly
/// the roots used by its fixed-wrist witness. Propagate the existing yaw step
/// bound outwards by reducing neighboring turns, never retiming a contact.
pub fn reserve_neutral(samples: &mut [f64], intervals: &[[f64; 2]]) {
    if samples.is_empty() || intervals.is_empty() {
        return;
    }
    for &[start, end] in intervals {
        let first = (((start + crate::model::PRE_ROLL) / STEP).floor().max(0.) as usize)
            .min(samples.len() - 1);
        let last = (((end + crate::model::PRE_ROLL) / STEP).ceil().max(0.) as usize)
            .min(samples.len() - 1);
        samples[first..=last].fill(0.);
    }
    for i in 1..samples.len() {
        let limit = samples[i - 1].abs() + 0.06;
        samples[i] = samples[i].clamp(-limit, limit);
    }
    for i in (0..samples.len() - 1).rev() {
        let limit = samples[i + 1].abs() + 0.06;
        samples[i] = samples[i].clamp(-limit, limit);
    }
}

/// Reach failures reduce only the neighboring time knots, then propagate the
/// existing yaw-rate bound outward. Other phrases retain useful torso facing.
/// Every adjustment moves angles toward neutral; final fallback remains explicit
/// neutral when this bounded refinement cannot preserve all source contacts.
pub fn preserve_contacts(
    samples: &mut [f64],
    profile: &RigProfile,
    contacts: &[(f64, usize, Vec3, f64, Option<&str>)],
    equipment: Option<&crate::equipment_motion::EquipmentContext>,
    source_times: &[Vec<f64>; 2],
) {
    if samples.is_empty() {
        return;
    }
    let valid =
        |yaw, &(time, limb, target, pitch, piece): &(f64, usize, Vec3, f64, Option<&str>)| {
            let p = profile.turned(yaw);
            if equipment.is_some() && p.hand_frames.is_some() {
                let times = &source_times[limb];
                let index = times.partition_point(|t| *t < time - 1e-9);
                if times.get(index).is_some_and(|t| (*t - time).abs() < 1e-9) {
                    crate::contact_feasibility::source_at_pitch(
                        &p, limb, target, pitch, equipment, piece,
                    )
                    .is_some()
                } else {
                    crate::contact_feasibility::at_pitch(&p, limb, target, pitch, equipment, piece)
                        .is_some()
                }
            } else {
                p.reachable_at_pitch(limb, target, pitch)
            }
        };
    // Preserve a known neutral witness; baseline path failures remain explicit
    // and do not erase useful body motion in unrelated phrases.
    let protected: Vec<_> = contacts.iter().map(|c| valid(0., c)).collect();
    let enforce_rate = |values: &mut [f64]| {
        for reverse in [false, true] {
            for k in 1..values.len() {
                let i = if reverse { values.len() - k } else { k };
                let (a, b) = (values[i - 1], values[i]);
                if (a - b).abs() <= 0.06 {
                    continue;
                }
                if a * b >= 0.0 {
                    if a.abs() > b.abs() {
                        values[i - 1] = a.signum() * (b.abs() + 0.06);
                    } else {
                        values[i] = b.signum() * (a.abs() + 0.06);
                    }
                } else {
                    let scale = 0.06 / (a.abs() + b.abs());
                    values[i - 1] *= scale;
                    values[i] *= scale;
                }
            }
        }
    };
    for pass in 0..16 {
        let mut affected = vec![false; samples.len()];
        for (contact, &protect) in contacts.iter().zip(&protected) {
            let time = contact.0;
            if !protect || valid(at(samples, time), contact) {
                continue;
            }
            let frame = ((time + crate::model::PRE_ROLL) / STEP).max(0.0);
            let i = (frame.floor() as usize).min(samples.len() - 1);
            affected[i] = true;
            affected[(i + 1).min(samples.len() - 1)] = true;
        }
        if !affected.iter().any(|v| *v) {
            return;
        }
        for (value, bad) in samples.iter_mut().zip(affected) {
            if bad {
                *value = if pass < 10 { *value * 0.5 } else { 0.0 };
            }
        }
        enforce_rate(samples);
    }
    if contacts
        .iter()
        .zip(&protected)
        .any(|(c, &protect)| protect && !valid(at(samples, c.0), c))
    {
        samples.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_plan,
        model::Selection,
        motion::{distance, Motion},
        rig::{rotate, Quat},
    };
    fn native_reference() -> RigProfile {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        serde_json::from_value(fixture["profile"].clone()).unwrap()
    }

    #[test]
    fn facing_anticipates_the_next_phrase_with_the_existing_turn_bounds() {
        let profile = native_reference();
        let centered = ([[-0.12, 0.95, 0.68], [0.12, 0.95, 0.68]], [0.; 2]);
        let leftward = ([[-0.12, 0.85, 0.62], [-0.40, 1.10, 0.60]], [0.; 2]);
        let mut phrase = vec![centered; 3];
        phrase.extend([leftward; 3]);
        let unchanged = phrase.clone();
        let planned = plan(&profile, &phrase);
        let quiet = plan(&profile, &[centered; 6]);
        assert!(
            planned[2] < quiet[2] - 0.005,
            "facing should prepare for the future leftward phrase: {planned:?}"
        );
        assert_eq!(phrase, unchanged, "facing cannot retime or move targets");
        phrase.reverse();
        for track in [planned, plan(&profile, &phrase)] {
            assert_eq!(track.len(), phrase.len());
            assert!(track[0].abs() <= 0.06 + 1e-12);
            assert!(track
                .iter()
                .all(|yaw| yaw.is_finite() && yaw.abs() <= 0.24 + 1e-12));
            assert!(track
                .windows(2)
                .all(|pair| (pair[1] - pair[0]).abs() <= 0.06 + 1e-12));
        }
        assert!(plan(&profile, &[]).is_empty());
        let single = plan(&profile, &[leftward]);
        assert_eq!(single.len(), 1);
        assert!(single[0].abs() <= 0.06 + 1e-12);
    }

    #[test]
    fn facing_preserves_an_ordinary_body_witness_with_or_without_mesh_geometry() {
        let reference = native_reference();
        // Independently authored cross-body contact and resting opposite hand.
        // A merely reachable facing previously left every otherwise valid RH
        // witness inside the ordinary torso, although the next bounded turn
        // admitted a clear pose. No imported avatar or recorded song is needed.
        let tips = [[-0.12, 0.85, 0.62], [-0.40, 1.10, 0.60]];
        for geometry in 0..3 {
            let mut profile = reference.clone();
            match geometry {
                0 => profile.body_geometry = None,
                1 => {
                    // A small visual estimate must not shrink the semantic
                    // torso used by common motion planning.
                    let visual = profile.body_geometry.as_mut().unwrap();
                    visual.torso.half_extents = [0.02, 0.05, 0.02];
                    visual.arm_radii = [[0.01; 2]; 2];
                }
                _ => {}
            }
            let lengths = profile.lengths;
            let yaw = candidate(&profile, tips, [0.; 2], 0.);
            assert!(yaw.abs() <= 0.06 + 1e-12);
            let turned = profile.turned(yaw);
            assert_eq!(turned.lengths, lengths);
            for (limb, &target) in tips.iter().enumerate() {
                assert!(
                    crate::posture::SWIVELS.iter().any(|&swivel| {
                        turned
                            .solve_hand_with_swivel(limb, target, 0., swivel)
                            .is_some_and(|hand| {
                                hand.reachable
                                    && distance(hand.tip, target) < 1e-9
                                    && crate::posture::anatomical(&turned, limb, target, &hand)
                                    && turned.hand_orientation_policy_valid(limb, hand.rotation)
                                    && crate::clearance::upper_body_core(&turned, limb, &hand)
                                        .minimum()
                                        >= 0.
                            })
                    }),
                    "geometry {geometry}, limb {limb}, yaw {yaw}"
                );
            }
        }
    }

    #[test]
    fn coordinated_turn_keeps_contacts_support_and_shuffled_evaluation() {
        let track = [
            0, 0x99, 42, 80, 0x81, 0x70, 0x99, 38, 110, 0x81, 0x70, 0x99, 49, 120, 0x81, 0x70,
            0x99, 42, 70, 0, 0xff, 0x2f, 0,
        ];
        let mut midi = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        midi.extend((track.len() as u32).to_be_bytes());
        midi.extend(track);
        let plan = build_plan(
            &midi,
            &Selection {
                tracks: vec![0],
                channel: None,
                ..Selection::default()
            },
        )
        .unwrap();
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0.0, 0.9, -0.1]);
        let motion = Motion::with_profile(plan, profile.clone()).unwrap();
        let times = [-0.5, 0.0, 0.125, 0.25, 0.5, 0.625, 0.75, 1.0];
        let poses: Vec<_> = times.iter().map(|&t| motion.evaluate_rig(t)).collect();
        for i in [7, 0, 5, 2, 6, 1, 4, 3] {
            assert_eq!(poses[i], motion.evaluate_rig(times[i]));
        }
        for pose in &poses {
            assert_eq!(pose.len(), 77);
            let yaw = 2.0 * (pose[74] as f64).atan2(pose[76] as f64);
            assert!(yaw.abs() <= 0.24 + 1e-10);
            let turned = profile.turned(yaw);
            for limb in 0..4 {
                let point = |o| [pose[o] as f64, pose[o + 1] as f64, pose[o + 2] as f64];
                assert!(distance(point(limb * 12), turned.roots[limb]) < 1e-6);
                let q: Quat = std::array::from_fn(|i| pose[57 + limb * 4 + i] as f64);
                let end = point(limb * 12 + 6);
                let offset = rotate(
                    q,
                    crate::motion::add(profile.grip_offsets[limb], profile.tip_offsets[limb]),
                );
                assert!(distance(crate::motion::add(end, offset), point(limb * 12 + 9)) < 1e-6);
            }
            assert_eq!(&pose[48..51], &poses[0][48..51]);
        }
        for hit in &motion.plan.hits {
            if let Some(limb) = hit.limb {
                let pose = motion.evaluate_rig(hit.time);
                let o = limb * 12 + 9;
                assert!(
                    distance(
                        [pose[o] as f64, pose[o + 1] as f64, pose[o + 2] as f64],
                        hit.target
                    ) < 1e-6
                );
            }
        }
    }

    #[test]
    fn facing_curve_keeps_outer_reach_contacts_at_and_between_sample_knots() {
        for (with_frames, tick, target) in [
            (false, 0u8, [-1.079, 1.26, 0.2]),
            // Near the outer wrist annulus with the target-oriented shaft
            // inclination; the former compound-link boundary is not a pose.
            (true, 30u8, [-1.374, 1.35, -0.1]),
        ] {
            let track = [tick, 0x90, 38, 100, 120, 0xff, 0x2f, 0];
            let mut midi = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
            midi.extend((track.len() as u32).to_be_bytes());
            midi.extend(track);
            let mut selection = Selection {
                tracks: vec![0],
                channel: None,
                ..Selection::default()
            };
            selection.hand_overrides.insert("0:0".into(), 0);
            let mut plan = build_plan(&midi, &selection).unwrap();
            plan.kit
                .iter_mut()
                .find(|piece| piece.id == "snare")
                .unwrap()
                .position = target;
            plan.hits[0].target = target;
            let mut profile = RigProfile::primitive();
            profile.torso_pivot = Some([0.0, 0.9, -0.1]);
            if with_frames {
                let frame = crate::rig::HandControlFrame {
                    forearm: [0.0, 0.0, 1.0],
                    palm: [0.0, 1.0, 0.0],
                    desired_palm: [0.0, 1.0, 0.0],
                    palm_roll_range: None,
                    palm_reference: None,
                };
                profile.hand_frames = Some([frame.clone(), frame]);
            }
            assert!(profile.reachable(0, target));
            let motion = Motion::with_profile(plan, profile).unwrap();
            assert_eq!(motion.plan.hits[0].limb, Some(0));
            let time = motion.plan.hits[0].time;
            let pose = motion.evaluate_rig(time);
            let actual = [pose[9] as f64, pose[10] as f64, pose[11] as f64];
            assert!(
                distance(actual, target) < 1e-6,
                "frames={with_frames}, error={}",
                distance(actual, target)
            );
            motion.evaluate_rig(time + 0.1);
            motion.evaluate_rig(-0.5);
            assert_eq!(pose, motion.evaluate_rig(time));
        }
    }
    #[test]
    fn reach_guard_preserves_unrelated_facing_and_rate_bounds() {
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0.0, 0.9, -0.1]);
        let target = [-1.079, 1.26, 0.2];
        assert!(profile.reachable(0, target));
        let mut samples = vec![0.24; 80];
        preserve_contacts(
            &mut samples,
            &profile,
            &[(0.0, 0, target, 0.0, None)],
            None,
            &[vec![], vec![]],
        );
        assert!(profile.turned(at(&samples, 0.0)).reachable(0, target));
        assert!((samples[79] - 0.24).abs() < 1e-12);
        assert!(samples
            .windows(2)
            .all(|p| (p[1] - p[0]).abs() <= 0.06 + 1e-12));
        let first = samples.clone();
        preserve_contacts(
            &mut samples,
            &profile,
            &[(0.0, 0, target, 0.0, None)],
            None,
            &[vec![], vec![]],
        );
        assert_eq!(samples, first);
    }
    #[test]
    fn neutral_reservation_covers_off_grid_interval_and_survives_contact_preservation() {
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0., 0.9, -0.1]);
        let mut samples = vec![0.24; 80];
        reserve_neutral(&mut samples, &[[0.123, 0.917]]);
        let target = [-1.079, 1.26, 0.2];
        preserve_contacts(
            &mut samples,
            &profile,
            &[(1.23, 0, target, 0., None)],
            None,
            &[vec![], vec![]],
        );
        for time in [0.123 - 1e-9, 0.123, 0.5, 0.917, 0.917 + 1e-9] {
            assert_eq!(at(&samples, time), 0.);
        }
        assert!(samples
            .windows(2)
            .all(|p| (p[1] - p[0]).abs() <= 0.06 + 1e-12));
        assert!(profile.turned(at(&samples, 1.23)).reachable(0, target));
        assert_eq!(samples[79], 0.24);
    }
}
