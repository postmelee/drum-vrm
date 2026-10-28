use super::*;

/// Independently authored numeric arm with a neutral forward-pointing tool.
fn profile(scale: f64, palm: Vec3) -> RigProfile {
    let mut profile = RigProfile::primitive();
    profile.roots[0] = [-0.19, 1.235, 0.06];
    profile.lengths[0] = [0.30 * scale, 0.27 * scale];
    profile.grip_offsets[0] = [0.0, 0.0, 0.03];
    profile.tip_offsets[0] = [0.0, 0.0, 0.31];
    profile.contact_rotations[0] = frame([0.0, 0.0, 1.0], palm);
    // With this source tool, frame maps X to forward. Express the offset along
    // that frame's forward direction rather than relying on its raw axis.
    profile.grip_offsets[0] = [0.03, 0.0, 0.0];
    profile.tip_offsets[0] = [0.31, 0.0, 0.0];
    let elbow = add(profile.roots[0], [0.0, -profile.lengths[0][0], 0.0]);
    let wrist = add(elbow, [0.0, 0.0, profile.lengths[0][1]]);
    profile.rest_tips[0] = add(wrist, profile.offset(0, profile.contact_rotations[0]));
    let reference = HandControlFrame {
        forearm: [0.0, 0.0, 1.0],
        palm,
        desired_palm: palm,
    };
    profile.hand_frames = Some([reference.clone(), reference]);
    profile
}

#[test]
fn target_oriented_wrists_keep_actual_contact_and_bone_lengths() {
    for scale in [0.75, 1.0, 1.25] {
        for palm in [[0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0]] {
            let profile = profile(scale, palm);
            let source = profile.hand_frames.as_ref().unwrap()[0].clone();
            for target in [
                [-0.18, 0.79, 0.4],
                [-0.48, 0.96, 0.44],
                [-0.66, 1.26, 0.68],
                [0.66, 1.26, 0.68],
                profile.rest_tips[0],
            ] {
                let solved = profile.solve_hand(0, target, 0.0).unwrap();
                assert_eq!(profile.reachable(0, target), solved.reachable);
                assert!(
                    (distance(profile.roots[0], solved.middle) - profile.lengths[0][0]).abs()
                        < 1e-9
                );
                assert!((distance(solved.middle, solved.end) - profile.lengths[0][1]).abs() < 1e-9);
                assert!(
                    distance(
                        add(solved.end, profile.offset(0, solved.rotation)),
                        solved.tip
                    ) < 1e-9
                );
                assert!(solved.rotation.iter().all(|v| v.is_finite()));
                if solved.reachable {
                    assert!(distance(solved.tip, target) < 1e-9);
                }
                let delta = multiply(solved.rotation, inverse(profile.contact_rotations[0]));
                let hand_forward = rotate(delta, source.forearm);
                let swing = dot(hand_forward, unit(sub(solved.end, solved.middle)))
                    .clamp(-1., 1.)
                    .acos();
                assert!((profile.wrist_deviation(0, &solved) - swing).abs() < 1e-9);
                if crate::posture::anatomical(&profile, 0, target, &solved) {
                    assert!(swing <= crate::posture::MAX_WRIST_SWING);
                }
                if solved.reachable {
                    assert!(profile.may_reach_contact(0, target));
                }
                let repeated = profile.solve_hand(0, target, 0.0).unwrap();
                assert_eq!(solved.middle, repeated.middle);
                assert_eq!(solved.rotation, repeated.rotation);
            }
        }
    }
}

#[test]
fn unreachable_and_degenerate_contacts_remain_finite_without_stretching() {
    let mut profile = profile(1.0, [0.0, 1.0, 0.0]);
    for target in [profile.roots[0], [0.0, 4.0, 4.0], [-0.19, 3.0, 0.06]] {
        let solved = profile.solve_hand(0, target, 0.0).unwrap();
        if distance(target, profile.roots[0]) > 1. {
            assert!(!solved.reachable);
        }
        assert!((distance(profile.roots[0], solved.middle) - 0.30).abs() < 1e-8);
        assert!((distance(solved.middle, solved.end) - 0.27).abs() < 1e-8);
        assert!(solved
            .rotation
            .iter()
            .chain(solved.tip.iter())
            .all(|v| v.is_finite()));
    }
    profile.grip_offsets[0] = [0.0, 0.0, 0.0];
    profile.tip_offsets[0] = [-0.27, 0.0, 0.0];
    let solved = profile.solve_hand(0, [0.0, 1.0, 0.5], 0.0).unwrap();
    assert!(solved.tip.iter().all(|v| v.is_finite()));
}

#[test]
fn pitch_controls_shaft_incline_without_changing_measured_grip() {
    let profile = profile(1.0, [0.0, 1.0, 0.0]);
    for pitch in [-0.7, -0.3, 0.0, 0.3] {
        for target in [[-0.18, 0.79, 0.4], [-0.48, 0.96, 0.44], [-0.66, 1.26, 0.68]] {
            let solved = profile.solve_hand(0, target, pitch).unwrap();
            assert!(
                distance(
                    add(solved.end, profile.offset(0, solved.rotation)),
                    solved.tip
                ) < 1e-9
            );
            assert!((distance(solved.middle, solved.end) - 0.27).abs() < 1e-9);
            let shaft = rotate(solved.rotation, unit(profile.tip_offsets[0]));
            assert!((shaft[1] + (APPROACH_INCLINE + pitch).sin()).abs() < 1e-9);
            let toward = sub(target, profile.roots[0]);
            assert!((shaft[0] * toward[2] - shaft[2] * toward[0]).abs() < 1e-9);
        }
    }
}

/// Reauthor the local tool axes while preserving identical world calibration.
/// This catches accidental use of raw +Z as the physical shaft direction.
#[test]
fn equivalent_rotated_tool_frames_produce_identical_world_poses() {
    let original = profile(0.88, [0., 1., 0.]);
    let local = axis_rotation(unit([0.2, 0.7, 0.4]), 0.73);
    let mut rotated = original.clone();
    rotated.contact_rotations[0] = multiply(original.contact_rotations[0], inverse(local));
    rotated.grip_offsets[0] = rotate(local, original.grip_offsets[0]);
    rotated.tip_offsets[0] = rotate(local, original.tip_offsets[0]);
    for target in [[-0.18, 0.79, 0.4], [-0.48, 0.96, 0.44], [0.1, 1.1, 0.65]] {
        for pitch in [-0.42, 0., 0.3] {
            for swivel in [-0.8, 0., 0.8] {
                let a = original
                    .solve_hand_with_swivel(0, target, pitch, swivel)
                    .unwrap();
                let b = rotated
                    .solve_hand_with_swivel(0, target, pitch, swivel)
                    .unwrap();
                assert_eq!(a.reachable, b.reachable);
                if a.reachable {
                    assert!(original.may_reach_contact(0, target));
                }
                assert!(distance(a.middle, b.middle) < 1e-9);
                assert!(distance(a.end, b.end) < 1e-9);
                assert!(distance(a.tip, b.tip) < 1e-9);
            }
        }
    }
}

#[test]
fn legacy_profiles_do_not_opt_into_the_new_hand_solver() {
    let profile = RigProfile::primitive();
    assert!(profile.solve_hand(0, profile.rest_tips[0], 0.0).is_none());
    assert!(profile.solve_hand(2, profile.rest_tips[2], 0.0).is_none());
}

#[test]
fn malformed_hand_frames_are_rejected_before_solving() {
    let mut valid = RigProfile::primitive();
    let frame = HandControlFrame {
        forearm: [0.0, 0.0, 1.0],
        palm: [0.0, 1.0, 0.0],
        desired_palm: [0.0, 1.0, 0.0],
    };
    valid.hand_frames = Some([frame.clone(), frame]);
    assert!(valid.validate().is_ok());
    for invalid_axis in [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [f64::NAN, 1.0, 0.0]] {
        let mut invalid = valid.clone();
        invalid.hand_frames.as_mut().unwrap()[0].palm = invalid_axis;
        assert!(invalid.validate().is_err());
    }
    let mut invalid = valid.clone();
    invalid.hand_frames.as_mut().unwrap()[0].desired_palm = [0.0, 2.0, 0.0];
    assert!(invalid.validate().is_err());
}

/// Independent asymmetric held-tool geometry, with a palm already angled in
/// the calibrated posture. Exercise local IK continuity across the front
/// workspace, including folded but reachable preparatory positions.
#[test]
fn nearby_targets_do_not_switch_elbow_or_palm_branches() {
    let mut profile = profile(0.82, [0.0, 1.0, 0.0]);
    let forearm = unit([-0.48, -0.52, 0.71]);
    let palm = projected([0.2, 0.9, 0.5], forearm);
    profile.contact_rotations[0] = [0.0, 0.0, 0.0, 1.0];
    profile.grip_offsets[0] = [-0.035, 0.005, 0.025];
    profile.tip_offsets[0] = [0.0, 0.0, 0.31];
    profile.hand_frames.as_mut().unwrap()[0] = HandControlFrame {
        forearm,
        palm,
        desired_palm: [0.0, 1.0, 0.0],
    };
    for x in -12..=12 {
        for y in -12..=4 {
            for z in 3..=14 {
                let target = add(
                    profile.roots[0],
                    [x as f64 * 0.05, y as f64 * 0.05, z as f64 * 0.05],
                );
                let a = profile.solve_hand(0, target, -0.2).unwrap();
                if !a.reachable {
                    continue;
                }
                for direction in [[1e-6, 0.0, 0.0], [0.0, 1e-6, 0.0], [0.0, 0.0, 1e-6]] {
                    let b = profile.solve_hand(0, add(target, direction), -0.2).unwrap();
                    if !b.reachable {
                        continue;
                    }
                    assert!(
                        distance(a.middle, b.middle) < 0.001,
                        "elbow jumped near {target:?}"
                    );
                    assert!(
                        distance(a.end, b.end) < 0.001,
                        "wrist jumped near {target:?}"
                    );
                    let agreement = a
                        .rotation
                        .iter()
                        .zip(b.rotation)
                        .map(|(x, y)| x * y)
                        .sum::<f64>()
                        .abs()
                        .min(1.0);
                    assert!(agreement.acos() * 2.0 < 0.01, "palm jumped near {target:?}");
                }
            }
        }
    }
}

#[test]
fn degenerate_shoulder_span_and_swivel_keep_real_lengths() {
    for right in [[-0.19, 1.235, 0.06], [-0.19, 1.5, 0.06]] {
        let mut p = profile(1.0, [0.0, 1.0, 0.0]);
        p.roots[1] = right;
        for swivel in [-0.8, 0.0, 0.8] {
            let hand = p
                .solve_hand_with_swivel(0, [-0.18, 0.79, 0.4], 0.0, swivel)
                .unwrap();
            assert!((distance(p.roots[0], hand.middle) - p.lengths[0][0]).abs() < 1e-9);
            assert!((distance(hand.middle, hand.end) - p.lengths[0][1]).abs() < 1e-9);
            assert!(distance(hand.tip, [-0.18, 0.79, 0.4]) < 1e-9);
        }
    }
}

/// Authored oblique grip over a tilted drumhead. The measured grip is retained,
/// but the tool is free to roll so a safe approach does not require a rigid
/// forearm/tool compound link or an exact world-space palm orientation.
#[test]
fn oblique_grip_uses_a_nonpenetrating_tilted_head_approach() {
    use crate::equipment::{LocalBox, PieceGeometry, Surface};
    let mut p = profile(1., [0., 1., 0.]);
    p.contact_rotations[0] = [0., 0., 0., 1.];
    p.grip_offsets[0] = [-0.015, 0., 0.025];
    p.tip_offsets[0] = [0., 0., 0.31];
    p.hand_frames.as_mut().unwrap()[0].forearm = unit([-1., 0., 1.]);
    let target = [-0.15, 0.90, 0.55];
    let normal = [0., 12_f64.to_radians().cos(), -12_f64.to_radians().sin()];
    let mut drum = crate::model::default_kit()
        .into_iter()
        .find(|piece| piece.instrument == crate::model::Instrument::Snare)
        .unwrap();
    drum.position = target;
    drum.radius = 0.22;
    drum.geometry = Some(PieceGeometry {
        version: 1,
        reference_radius: 0.22,
        source: "authored-test".into(),
        body: LocalBox {
            center: [0., -0.08, 0.],
            half_extents: [0.22, 0.12, 0.22],
        },
        surface: Surface {
            center: [0.; 3],
            normal,
            radius: 0.22,
            radial_profile: None,
            motion: None,
        },
    });
    let mut geometry = crate::equipment::kit_geometry(std::slice::from_ref(&drum));
    geometry.supports.clear();
    let ctx = crate::equipment_motion::EquipmentContext::new(vec![drum], geometry);
    let hand = crate::contact_feasibility::candidate(&p, 0, target, 0., Some(&ctx), Some("snare"))
        .expect("bounded common approach should reach this tilted head");
    assert!(crate::posture::anatomical(&p, 0, target, &hand));
    assert!(p.wrist_deviation(0, &hand) <= crate::posture::MAX_WRIST_SWING);
    assert!(distance(hand.tip, target) < 1e-9);
    let direction = rotate(hand.rotation, unit(p.tip_offsets[0]));
    assert!(
        dot(direction, normal) < -0.05,
        "shaft must arrive from above the head"
    );
    let grip = add(hand.end, rotate(hand.rotation, p.grip_offsets[0]));
    assert!(dot(sub(grip, target), normal) > 0.02);
    assert!(
        crate::equipment_motion::tool_clearance(&p, 0, &hand, &ctx, Some("snare"))
            >= crate::posture::MARGIN
    );
}
