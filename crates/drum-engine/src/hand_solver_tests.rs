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
        palm_roll_range: None,
        palm_reference: None,
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
    for (palm, range) in [
        ([0., 1., 0.], None),
        ([1., 0., 0.], Some([-std::f64::consts::FRAC_PI_2, 0.])),
    ] {
        let mut original = profile(0.88, palm);
        original.hand_frames.as_mut().unwrap()[0].palm_roll_range = range;
        original.hand_frames.as_mut().unwrap()[0].palm_reference =
            range.map(|_| crate::rig::PalmReference::ShaftHeading);
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
        palm_roll_range: None,
        palm_reference: None,
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
    for range in [[f64::NAN, 0.], [-4., 0.], [0., 4.], [0.1, 0.2], [-2., 2.]] {
        let mut invalid = valid.clone();
        invalid.hand_frames.as_mut().unwrap()[0].palm_roll_range = Some(range);
        assert!(invalid.validate().is_err());
    }
}

#[test]
fn bounded_roll_checks_the_periodic_arc_endpoints() {
    let angle = 170_f64.to_radians();
    assert_eq!(
        bounded_roll(angle.sin(), angle.cos(), -std::f64::consts::FRAC_PI_2, 0.),
        -std::f64::consts::FRAC_PI_2
    );
    assert_eq!(
        bounded_roll(-angle.sin(), angle.cos(), 0., std::f64::consts::FRAC_PI_2),
        std::f64::consts::FRAC_PI_2
    );
}

#[test]
fn shaft_heading_keeps_thumb_up_reference_for_side_and_cross_body_targets() {
    for side in [-1., 1.] {
        let mut p = profile(1., [-side, 0., 0.]);
        p.roots[0][0] = side * 0.19;
        let f = &mut p.hand_frames.as_mut().unwrap()[0];
        f.palm_roll_range = Some(if side < 0. {
            [-std::f64::consts::FRAC_PI_2, 0.]
        } else {
            [0., std::f64::consts::FRAC_PI_2]
        });
        f.palm_reference = Some(crate::rig::PalmReference::ShaftHeading);
        for x in [-0.6, -0.3, 0., 0.3, 0.6] {
            for y in [0.8, 1.1, 1.4] {
                let target = [x, y, 0.5];
                for pitch in [-0.45, 0., 0.3] {
                    let h = p.solve_hand(0, target, pitch).unwrap();
                    let palm = rotate(
                        multiply(h.rotation, inverse(p.contact_rotations[0])),
                        p.hand_frames.as_ref().unwrap()[0].palm,
                    );
                    assert!(
                        palm[1] <= 1e-9,
                        "palm turned upward at {target:?}: {palm:?}"
                    );
                    assert!((distance(p.roots[0], h.middle) - p.lengths[0][0]).abs() < 1e-9);
                    assert!((distance(h.middle, h.end) - p.lengths[0][1]).abs() < 1e-9);
                }
            }
        }
    }
}

#[test]
fn matched_grip_keeps_inward_to_down_roll_without_changing_contact_or_bones() {
    for side in [-1., 1.] {
        let mut p = profile(1., [-side, 0., 0.]);
        p.roots[0][0] = side * 0.19;
        let range = if side < 0. {
            [-std::f64::consts::FRAC_PI_2, 0.]
        } else {
            [0., std::f64::consts::FRAC_PI_2]
        };
        p.hand_frames.as_mut().unwrap()[0].palm_roll_range = Some(range);
        p.hand_frames.as_mut().unwrap()[0].palm_reference =
            Some(crate::rig::PalmReference::ShaftHeading);
        let reference = &p.hand_frames.as_ref().unwrap()[0];
        for target in [
            [side * 0.16, 0.86, 0.48],
            [side * 0.25, 1.08, 0.58],
            [side * 0.45, 1.35, 0.6],
        ] {
            for pitch in [-0.42, 0., 0.3] {
                for swivel in [-0.8, 0., 0.8] {
                    let hand = p.solve_hand_with_swivel(0, target, pitch, swivel).unwrap();
                    let delta = multiply(hand.rotation, inverse(p.contact_rotations[0]));
                    let palm = rotate(delta, reference.palm);
                    let shaft = rotate(hand.rotation, unit(p.tip_offsets[0]));
                    let desired = rotate(
                        axis_rotation([0., 1., 0.], shaft[0].atan2(shaft[2])),
                        reference.desired_palm,
                    );
                    let base = projected(desired, shaft);
                    let actual = projected(palm, shaft);
                    let roll = dot(shaft, cross(base, actual)).atan2(dot(base, actual));
                    assert!(
                        roll >= range[0] - 1e-9 && roll <= range[1] + 1e-9,
                        "roll {roll} outside {range:?}"
                    );
                    assert!(palm[1] <= 1e-9, "matched grip turned palm up: {palm:?}");
                    assert!((distance(p.roots[0], hand.middle) - p.lengths[0][0]).abs() < 1e-9);
                    assert!((distance(hand.middle, hand.end) - p.lengths[0][1]).abs() < 1e-9);
                    if hand.reachable {
                        assert!(distance(hand.tip, target) < 1e-9);
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_palm_up_preference_does_not_acquire_matched_grip_policy() {
    let mut p = profile(1., [0., 1., 0.]);
    p.hand_frames.as_mut().unwrap()[0].palm_roll_range = Some([-0.15, 0.15]);
    for target in [[-0.16, 0.86, 0.48], [-0.25, 1.08, 0.58], [-0.45, 1.35, 0.6]] {
        let hand = p.solve_hand(0, target, 0.).unwrap();
        let delta = multiply(hand.rotation, inverse(p.contact_rotations[0]));
        assert!(rotate(delta, p.hand_frames.as_ref().unwrap()[0].palm)[1] > 0.8);
    }
}

/// Independent asymmetric held-tool geometry, with a palm already angled in
/// the calibrated posture. Exercise local IK continuity across the front
/// workspace, including folded but reachable preparatory positions.
#[test]
fn nearby_targets_do_not_switch_elbow_or_palm_branches() {
    for (desired_palm, range) in [
        ([0., 1., 0.], None),
        ([1., 0., 0.], Some([-std::f64::consts::FRAC_PI_2, 0.])),
    ] {
        let mut profile = profile(0.82, [0.0, 1.0, 0.0]);
        let forearm = unit([-0.48, -0.52, 0.71]);
        let palm = projected([0.2, 0.9, 0.5], forearm);
        profile.contact_rotations[0] = [0.0, 0.0, 0.0, 1.0];
        profile.grip_offsets[0] = [-0.035, 0.005, 0.025];
        profile.tip_offsets[0] = [0.0, 0.0, 0.31];
        profile.hand_frames.as_mut().unwrap()[0] = HandControlFrame {
            forearm,
            palm,
            desired_palm,
            palm_roll_range: range,
            palm_reference: range.map(|_| crate::rig::PalmReference::ShaftHeading),
        };
        for x in -12..=12 {
            for y in -12..=4 {
                for z in 3..=14 {
                    let target = add(
                        profile.roots[0],
                        [x as f64 * 0.05, y as f64 * 0.05, z as f64 * 0.05],
                    );
                    let anchor = profile.solve_hand(0, target, -0.2).unwrap();
                    let orientation = range.and_then(|_| {
                        profile.encode_hand_orientation(0, target, -0.2, 0., anchor.rotation)
                    });
                    let a = profile
                        .solve_hand_with_swivel_and_orientation(0, target, -0.2, 0., orientation)
                        .unwrap();
                    if !a.reachable {
                        continue;
                    }
                    for direction in [[1e-6, 0.0, 0.0], [0.0, 1e-6, 0.0], [0.0, 0.0, 1e-6]] {
                        let b = profile
                            .solve_hand_with_swivel_and_orientation(
                                0,
                                add(target, direction),
                                -0.2,
                                0.,
                                orientation,
                            )
                            .unwrap();
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
            rotation: None,
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

#[test]
fn axial_forearm_uses_perpendicular_fallback_without_stretching() {
    let mut p = profile(1., [1., 0., 0.]);
    p.hand_frames.as_mut().unwrap()[0].palm_roll_range = Some([-std::f64::consts::FRAC_PI_2, 0.]);
    let rotation = p.contact_rotations[0];
    let target = add(add(p.roots[0], [0., 0., 0.4]), p.offset(0, rotation));
    let h = solve_wrist(&p, 0, target, rotation, 0.);
    assert!((distance(p.roots[0], h.middle) - p.lengths[0][0]).abs() < 1e-9);
    assert!((distance(h.middle, h.end) - p.lengths[0][1]).abs() < 1e-9);
    assert!(distance(h.tip, target) < 1e-9);
}

/// The published arm style is the priority when a physical hand/tool twist can
/// satisfy the palm policy without moving any of its joints.
#[test]
fn bounded_orientation_preserves_valid_legacy_arm_witnesses() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let mut accepted = 0;
    for limb in 0..2 {
        let reference = &profile.hand_frames.as_ref().unwrap()[limb];
        let mut legacy = reference.clone();
        legacy.palm_roll_range = None;
        legacy.palm_reference = None;
        legacy.desired_palm = unit([reference.desired_palm[0], -1., 0.]);
        for target in [
            [-0.4, 0.9, 0.4],
            [0.3, 1.1, 0.65],
            [-0.35, 1.25, 0.7],
            [0.35, 1.25, 0.7],
        ] {
            for pitch in [-0.5, -0.2, 0.] {
                for swivel in [-0.4, 0., 0.4] {
                    let old = solve_legacy(&profile, limb, target, pitch, &legacy, swivel);
                    let Some(witness) =
                        preserve_arm_with_twist(&profile, limb, target, reference, old)
                    else {
                        continue;
                    };
                    accepted += 1;
                    let selected = profile
                        .solve_hand_with_swivel(limb, target, pitch, swivel)
                        .unwrap();
                    assert_eq!(selected.middle, old.middle);
                    assert_eq!(selected.end, old.end);
                    assert_eq!(selected.tip, old.tip);
                    assert_eq!(selected.rotation, witness.rotation);
                    assert!(crate::posture::anatomical(
                        &profile, limb, target, &selected
                    ));
                    assert!(profile.hand_orientation_policy_valid(limb, selected.rotation));
                    assert!(
                        distance(
                            add(selected.end, profile.offset(limb, selected.rotation)),
                            target
                        ) < 1e-10
                    );
                    let (butt, tip) =
                        crate::equipment_motion::stick_segment(&profile, limb, &selected);
                    assert!(distance(tip, target) < 1e-10);
                    assert!(distance(butt, tip) > 0.31);
                    let replay = profile
                        .solve_hand_with_swivel_and_orientation(
                            limb,
                            target,
                            pitch,
                            swivel,
                            profile.encode_hand_orientation(
                                limb,
                                target,
                                pitch,
                                swivel,
                                selected.rotation,
                            ),
                        )
                        .unwrap();
                    assert!(distance(replay.middle, old.middle) < 1e-10);
                    assert!(distance(replay.end, old.end) < 1e-10);
                }
            }
        }
    }
    assert!(
        accepted > 8,
        "native rig must exercise arm-preserving witnesses"
    );
}

#[test]
fn bounded_yaw_preserves_contact_without_the_hand_driven_high_elbow() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    for limb in 0..2 {
        let sign = if limb == 0 { -1. } else { 1. };
        let target = [sign * 0.35, 1.25, 0.7];
        let swivel = -sign * -0.2;
        let hand = profile
            .solve_hand_with_swivel(limb, target, -0.5, swivel)
            .unwrap();
        assert!(crate::posture::anatomical(&profile, limb, target, &hand));
        assert!(profile.hand_orientation_policy_valid(limb, hand.rotation));
        assert!(
            crate::posture::arm_posture(&profile, limb, &hand)
                .elevation
                .to_degrees()
                < 60.
        );
        assert!(distance(hand.tip, target) < 1e-10);
        assert!(
            (distance(profile.roots[limb], hand.middle) - profile.lengths[limb][0]).abs() < 1e-10
        );
        assert!((distance(hand.middle, hand.end) - profile.lengths[limb][1]).abs() < 1e-10);
        let mut previous = hand;
        for step in 1..=40 {
            let next = profile
                .solve_hand_with_swivel_and_orientation(
                    limb,
                    target,
                    -0.5,
                    swivel + sign * step as f64 * 0.0005,
                    profile.encode_hand_orientation(limb, target, -0.5, swivel, hand.rotation),
                )
                .unwrap();
            assert!(
                crate::posture::anatomical(&profile, limb, target, &next),
                "limb={limb} step={step} initial_wrist={} next_wrist={} elevation={}",
                profile.wrist_deviation(limb, &hand),
                profile.wrist_deviation(limb, &next),
                crate::posture::arm_posture(&profile, limb, &next).elevation
            );
            assert!(distance(previous.middle, next.middle) < 0.003);
            previous = next;
        }
        assert_eq!(
            hand.middle,
            profile
                .solve_hand_with_swivel(limb, target, -0.5, swivel)
                .unwrap()
                .middle
        );
    }
}

#[test]
fn fixed_orientation_replay_is_continuous_at_static_minimum_switches() {
    // Frozen authored recovery frames that previously exposed 26 cm and 11 cm
    // pointwise-minimum jumps. Runtime keeps the chosen full physical rotation.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    for (yaw, a, b, pitch, swivel) in [
        (
            0.14,
            [-0.07353261858224869, 0.839480996131897, 0.6434189677238464],
            [-0.07353269308805466, 0.8394802212715149, 0.6434183120727539],
            [-0.13058535905039745, -0.1305784930222758],
            [-0.3310060585348149, -0.3310079403313726],
        ),
        (
            0.13,
            [-0.1019628569483757, 0.9209684133529663, 0.6591596007347107],
            [-0.10196259617805481, 0.9209681749343872, 0.6591601371765137],
            [-0.4578140888675401, -0.45781602787619247],
            [-0.5253058371996288, -0.5253040873007924],
        ),
    ] {
        let profile = profile.turned(yaw);
        let anchor = profile
            .solve_hand_with_swivel(0, a, pitch[0], swivel[0])
            .unwrap();
        let mut previous: Option<HandSolution> = None;
        for step in 0..=100 {
            let t = step as f64 / 100.;
            let target = add(mul(a, 1. - t), mul(b, t));
            let hand = profile
                .solve_hand_with_swivel_and_orientation(
                    0,
                    target,
                    pitch[0] * (1. - t) + pitch[1] * t,
                    swivel[0] * (1. - t) + swivel[1] * t,
                    profile.encode_hand_orientation(0, a, pitch[0], swivel[0], anchor.rotation),
                )
                .unwrap();
            assert!(crate::posture::anatomical(&profile, 0, target, &hand));
            assert!(distance(hand.tip, target) < 1e-10);
            if let Some(previous) = previous {
                assert!(distance(previous.middle, hand.middle) < 0.002);
                assert!(
                    previous
                        .rotation
                        .iter()
                        .zip(hand.rotation)
                        .map(|(a, b)| a * b)
                        .sum::<f64>()
                        .abs()
                        > 0.999999
                );
            }
            previous = Some(hand);
        }
    }
}

#[test]
fn selected_orientation_round_trips_exact_pose_and_rejects_invalid_controls() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    for limb in 0..2 {
        for target in [
            [-0.35, 1.25, 0.7],
            [0.35, 1.25, 0.7],
            [-0.4, 0.9, 0.4],
            [0.3, 1.1, 0.65],
        ] {
            for pitch in [-0.65, -0.2, 0.15] {
                let hand = profile
                    .solve_hand_with_swivel(limb, target, pitch, 0.4)
                    .unwrap();
                let replay = profile
                    .solve_hand_with_swivel_and_orientation(
                        limb,
                        target,
                        pitch,
                        0.4,
                        profile.encode_hand_orientation(limb, target, pitch, 0.4, hand.rotation),
                    )
                    .unwrap();
                assert!(distance(hand.middle, replay.middle) < 1e-10);
                assert!(distance(hand.end, replay.end) < 1e-10);
                assert_eq!(hand.reachable, replay.reachable);
                let negative = hand.rotation.map(|v| -v);
                let opposite = profile
                    .solve_hand_with_swivel_and_orientation(
                        limb,
                        target,
                        pitch,
                        0.4,
                        profile.encode_hand_orientation(limb, target, pitch, 0.4, negative),
                    )
                    .unwrap();
                assert!(distance(hand.middle, opposite.middle) < 1e-10);
                assert!(distance(hand.end, opposite.end) < 1e-10);
            }
        }
        let target = profile.rest_tips[limb];
        for swivel in [f64::NAN, f64::INFINITY] {
            assert!(profile
                .encode_hand_orientation(limb, target, 0., swivel, [0., 0., 0., 1.])
                .is_none());
        }
        for rotation in [[f64::NAN, 0., 0., 1.], [0.; 4], [0., 0., 0., 2.]] {
            assert!(profile
                .solve_hand_with_swivel_and_orientation(limb, target, 0., 0., Some(rotation))
                .is_none());
        }
        assert!(profile
            .solve_hand_with_swivel_and_orientation(limb, target, 0., 0., None)
            .is_none());
    }
    let legacy = self::profile(1., [0., 1., 0.]);
    let target = legacy.rest_tips[0];
    let hand = legacy.solve_hand(0, target, 0.).unwrap();
    assert!(legacy.hand_roll(0, target, 0., &hand).is_none());
    assert!(legacy
        .solve_hand_with_swivel_and_orientation(0, target, 0., 0., Some(hand.rotation))
        .is_none());
    assert_eq!(
        legacy
            .solve_hand_with_swivel_and_orientation(0, target, 0., 0., None)
            .unwrap()
            .middle,
        hand.middle
    );
}

#[test]
fn orientation_policy_checks_actual_heading_and_explicit_preferences() {
    for (palm, range, heading) in [
        ([1., 0., 0.], [-std::f64::consts::FRAC_PI_2, 0.], true),
        ([-1., 0., 0.], [0., std::f64::consts::FRAC_PI_2], true),
        ([0., 1., 0.], [-0.15, 0.15], false),
        ([0., -1., 0.], [-0.15, 0.15], false),
    ] {
        let mut p = profile(1., palm);
        let frame = &mut p.hand_frames.as_mut().unwrap()[0];
        frame.palm_roll_range = Some(range);
        frame.palm_reference = heading.then_some(crate::rig::PalmReference::ShaftHeading);
        let reference = p.hand_frames.as_ref().unwrap()[0].clone();
        for target in [[-0.5, 1., 0.5], [0.3, 1.2, 0.6]] {
            let base = approach_rotation(&p, 0, target, -0.2, &reference);
            let shaft = rotate(base, unit(p.tip_offsets[0]));
            for roll in [range[0], (range[0] + range[1]) * 0.5, range[1]] {
                let rotation = normalize(multiply(axis_rotation(shaft, roll), base));
                assert!(p.hand_orientation_policy_valid(0, rotation));
                let hand = solve_with_roll(&p, 0, target, -0.2, &reference, 0., roll);
                assert!((p.hand_roll(0, target, -0.2, &hand).unwrap() - roll).abs() < 1e-9);
            }
            for roll in [range[0] - 0.1, range[1] + 0.1] {
                let rotation = normalize(multiply(axis_rotation(shaft, roll), base));
                assert!(!p.hand_orientation_policy_valid(0, rotation));
            }
        }
    }
}

#[test]
fn prepared_actual_orientation_preserves_selected_endpoints_and_contact_with_moving_controls() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let mut covered = [0; 2];
    for (limb, count) in covered.iter_mut().enumerate() {
        let reference = &profile.hand_frames.as_ref().unwrap()[limb];
        for target in [[-0.36, 0.9, 0.36], [-0.15, 0.92, 0.58], [0.3, 1.1, 0.65]] {
            for pitch in [-0.45, -0.3, -0.15] {
                for swivel in [-0.4, 0., 0.4] {
                    let moved = add(target, [0.008, 0.01, -0.006]);
                    let end_pitch = pitch + 0.025;
                    let end_swivel = swivel + 0.04;
                    let (Some(first), Some(last)) = (
                        solve_preserving_arm_with_swivel(
                            &profile, limb, target, pitch, reference, swivel,
                        ),
                        solve_preserving_arm_with_swivel(
                            &profile, limb, moved, end_pitch, reference, end_swivel,
                        ),
                    ) else {
                        continue;
                    };
                    *count += 1;
                    let mut track = [crate::hand_coordination::SwivelSample {
                        time: 0.,
                        angles: [0.; 2],
                        orientations: [None; 2],
                    }; 2];
                    for (i, (tip, pitch, swivel, hand)) in [
                        (target, pitch, swivel, first),
                        (moved, end_pitch, end_swivel, last),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        track[i].time = i as f64;
                        track[i].angles[limb] = swivel;
                        track[i].orientations[limb] = profile.encode_hand_orientation(
                            limb,
                            tip,
                            pitch,
                            swivel,
                            hand.rotation,
                        );
                    }
                    let evaluate = |t: f64| {
                        let control = crate::hand_coordination::at(&track, t);
                        let tip = add(mul(target, 1. - t), mul(moved, t));
                        let pitch = pitch * (1. - t) + end_pitch * t;
                        let hand = profile
                            .solve_hand_with_swivel_and_orientation(
                                limb,
                                tip,
                                pitch,
                                control.angles[limb],
                                control.orientations[limb],
                            )
                            .unwrap();
                        // Exact selected joints are retained at controls. Between
                        // controls, the actual orientation follows one arc;
                        // contact and fixed bone lengths remain the invariants.
                        if t == 0. || t == 1. {
                            let selected = if t == 0. { first } else { last };
                            assert!(distance(hand.middle, selected.middle) < 1e-9);
                            assert!(distance(hand.end, selected.end) < 1e-9);
                        }
                        assert!(
                            (distance(profile.roots[limb], hand.middle) - profile.lengths[limb][0])
                                .abs()
                                < 1e-9
                        );
                        assert!(
                            (distance(hand.middle, hand.end) - profile.lengths[limb][1]).abs()
                                < 1e-9
                        );
                        assert!(
                            distance(add(hand.end, profile.offset(limb, hand.rotation)), tip)
                                < 1e-9
                        );
                        let (_, actual_tip) =
                            crate::equipment_motion::stick_segment(&profile, limb, &hand);
                        assert!(distance(actual_tip, tip) < 1e-9);
                        hand
                    };
                    for step in 0..=100 {
                        evaluate(step as f64 / 100.);
                    }
                    let before_seek = evaluate(0.37);
                    for t in [0.9, 0.03, 1., 0.] {
                        evaluate(t);
                    }
                    let after_seek = evaluate(0.37);
                    assert_eq!(before_seek.middle, after_seek.middle);
                    assert_eq!(before_seek.rotation, after_seek.rotation);
                    let restored: [crate::hand_coordination::SwivelSample; 2] =
                        serde_json::from_str(&serde_json::to_string(&track).unwrap()).unwrap();
                    assert_eq!(restored[1].orientations, track[1].orientations);
                }
            }
        }
    }
    assert!(
        covered.into_iter().all(|count| count >= 2),
        "both hands need feasible independently moving witnesses: {covered:?}"
    );
}

#[test]
fn prepared_replay_keeps_smooth_orientation_and_reports_policy_failures() {
    let mut p = profile(1., [1., 0., 0.]);
    let reference = &mut p.hand_frames.as_mut().unwrap()[0];
    reference.palm_roll_range = Some([-std::f64::consts::FRAC_PI_2, 0.]);
    reference.palm_reference = Some(crate::rig::PalmReference::ShaftHeading);
    let target = [-0.24, 0.95, 0.69];
    let base = p.contact_rotations[0];
    let raw = solve_wrist(&p, 0, target, base, 0.);
    assert!(crate::posture::anatomical(&p, 0, target, &raw));
    assert!(p.hand_orientation_policy_valid(0, raw.rotation));
    let same = solve_with_rotation(&p, 0, target, base, 0.);
    assert_eq!(same.rotation, raw.rotation);
    assert_eq!(same.middle, raw.middle);
    assert_eq!(same.end, raw.end);
    let mut previous: Option<HandSolution> = None;
    for step in -100..=100 {
        let angle = step as f64 * 0.0002;
        let q = normalize(multiply(axis_rotation([0., 0., 1.], angle), base));
        let unprojected = solve_wrist(&p, 0, target, q, 0.);
        let corrected = solve_with_rotation(&p, 0, target, q, 0.);
        assert_eq!(corrected.rotation, q);
        assert!(crate::posture::anatomical(&p, 0, target, &corrected));
        assert_eq!(corrected.middle, unprojected.middle);
        assert_eq!(corrected.end, unprojected.end);
        assert_eq!(corrected.tip, unprojected.tip);
        assert!(distance(add(corrected.end, p.offset(0, corrected.rotation)), target) < 1e-10);
        if angle > 0.001 {
            assert!(!p.hand_orientation_policy_valid(0, q));
            assert!(!p.hand_orientation_policy_valid(0, corrected.rotation));
        }
        if let Some(previous) = previous {
            let cosine = previous
                .rotation
                .iter()
                .zip(corrected.rotation)
                .map(|(a, b)| a * b)
                .sum::<f64>()
                .abs()
                .min(1.);
            assert!(2. * cosine.acos() < 0.00021);
        }
        let residual = p
            .encode_hand_orientation(0, target, -APPROACH_INCLINE, 0., q)
            .unwrap();
        let replay = p
            .solve_hand_with_swivel_and_orientation(
                0,
                target,
                -APPROACH_INCLINE,
                0.,
                Some(residual),
            )
            .unwrap();
        assert!(distance(replay.middle, corrected.middle) < 1e-10);
        assert!(distance(replay.end, corrected.end) < 1e-10);
        assert!(
            distance(
                rotate(replay.rotation, [1., 0., 0.]),
                rotate(corrected.rotation, [1., 0., 0.])
            ) < 1e-9
        );
        previous = Some(corrected);
    }
}

#[test]
fn reach_cone_restores_contact_with_real_mount_and_replayable_orientation() {
    let mut p = profile(0.8, [1., 0., 0.]);
    let reference = &mut p.hand_frames.as_mut().unwrap()[0];
    reference.palm_roll_range = Some([-std::f64::consts::FRAC_PI_2, 0.]);
    reference.palm_reference = Some(crate::rig::PalmReference::ShaftHeading);
    let reference = reference.clone();
    let target = [-0.19, 1.35, 0.84];
    let pitch = -0.15;
    let swivel = 0.;
    let mut legacy = reference.clone();
    legacy.palm_roll_range = None;
    legacy.palm_reference = None;
    legacy.desired_palm = unit([1., -1., 0.]);
    let old = solve_legacy(&p, 0, target, pitch, &legacy, swivel);
    assert!(
        !old.reachable,
        "authored approach misses the reachable cone"
    );
    let hand = project_reach_cone(&p, 0, target, &reference, swivel, old, &|_| true)
        .expect("the same measured arm and mount have a valid nearby approach");
    assert!(crate::posture::anatomical(&p, 0, target, &hand));
    assert!(p.hand_orientation_policy_valid(0, hand.rotation));
    assert!(distance(hand.tip, target) < 1e-10);
    assert!(distance(add(hand.end, p.offset(0, hand.rotation)), target) < 1e-10);
    assert!((distance(p.roots[0], hand.middle) - p.lengths[0][0]).abs() < 1e-10);
    assert!((distance(hand.middle, hand.end) - p.lengths[0][1]).abs() < 1e-10);
    // Reject the first orientation while accepting a distinct twist on the
    // same reachable arm. The cone's internal wrist repair must retain this
    // predicate, rather than filtering only its already-discarded winner.
    let accepts_alternative = |candidate: &HandSolution| {
        candidate
            .rotation
            .iter()
            .zip(hand.rotation)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            .abs()
            < 0.9999
    };
    let alternative =
        project_reach_cone(&p, 0, target, &reference, swivel, old, &accepts_alternative)
            .expect("an accepted rigid-twist alternative remains on the cone");
    assert!(accepts_alternative(&alternative));
    assert!(crate::posture::anatomical(&p, 0, target, &alternative));
    assert!(p.hand_orientation_policy_valid(0, alternative.rotation));
    assert!(distance(alternative.middle, hand.middle) < 1e-10);
    assert!(distance(alternative.end, hand.end) < 1e-10);
    assert!(
        distance(
            add(alternative.end, p.offset(0, alternative.rotation)),
            target
        ) < 1e-10
    );
    assert!(project_reach_cone(&p, 0, target, &reference, swivel, old, &|_| false).is_none());
    let selected = p.solve_hand_with_swivel(0, target, pitch, swivel).unwrap();
    assert!(distance(selected.end, hand.end) < 1e-10);
    let residual = p
        .encode_hand_orientation(0, target, pitch, swivel, selected.rotation)
        .unwrap();
    let restored = p
        .solve_hand_with_swivel_and_orientation(0, target, pitch, swivel, Some(residual))
        .unwrap();
    assert!(distance(restored.middle, selected.middle) < 1e-10);
    assert!(distance(restored.end, selected.end) < 1e-10);
    assert!(distance(restored.tip, target) < 1e-10);
    assert!(project_reach_cone(
        &p,
        0,
        add(target, [0., 0., 2.]),
        &reference,
        swivel,
        old,
        &|_| true
    )
    .is_none());
}

#[test]
fn natural_grip_prefers_inward_without_moving_an_already_valid_arm() {
    let mut p = profile(1., [1., 0., 0.]);
    let reference = &mut p.hand_frames.as_mut().unwrap()[0];
    reference.palm_roll_range = Some([-std::f64::consts::FRAC_PI_2, 0.]);
    reference.palm_reference = Some(crate::rig::PalmReference::ShaftHeading);
    let reference = reference.clone();
    let target = [-0.24, 0.95, 0.69];
    let rotation = multiply(axis_rotation([0., 0., 1.], -0.8), p.contact_rotations[0]);
    let old = solve_wrist(&p, 0, target, rotation, 0.);
    assert!(crate::posture::anatomical(&p, 0, target, &old));
    let hand = preserve_arm_with_twist(&p, 0, target, &reference, old).unwrap();
    let (old_roll, _) = palm_orientation(&p, 0, &reference, old.rotation);
    let (roll, _) = palm_orientation(&p, 0, &reference, hand.rotation);
    assert!(old_roll.abs() > 0.7);
    assert!(roll.abs() < 1e-8, "unnecessary pronation remained: {roll}");
    // Pitch, fit and temporal selection must rank the same preference, rather
    // than losing it after the static solver has selected a hand orientation.
    assert!(
        p.hand_grip_preference_cost(0, hand.rotation)
            < p.hand_grip_preference_cost(0, old.rotation)
    );
    assert!(
        crate::posture::comfort_cost(&p, 0, target, &hand)
            < crate::posture::comfort_cost(&p, 0, target, &old)
    );
    assert!(crate::posture::comfort_cost(&p, 0, target, &old) <= 0.39);
    assert_eq!(hand.middle, old.middle);
    assert_eq!(hand.end, old.end);
    assert_eq!(hand.tip, old.tip);
    assert!(crate::posture::anatomical(&p, 0, target, &hand));
    assert!(p.hand_orientation_policy_valid(0, hand.rotation));
}

#[test]
fn prepared_actual_orientation_avoids_legacy_roll_feedback_between_existing_controls() {
    // Authored native count-in rest contact: with constant target and swivel,
    // the old four roll iterations amplify a 2 rad/s pitch change near -0.1.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let profile = profile.turned(-0.02);
    let target = [-0.1106, 0.8600195115817388, 0.6312790258949765];
    let swivel = 0.35;
    let reference = &profile.hand_frames.as_ref().unwrap()[0];
    let legacy = legacy_reference(reference);
    let mut track = [crate::hand_coordination::SwivelSample {
        time: 0.,
        angles: [swivel, 0.],
        orientations: [None; 2],
    }; 2];
    for (i, time) in [0.72, 0.78].into_iter().enumerate() {
        let pitch = -0.1 + 2. * (time - 0.75);
        let hand = profile
            .solve_hand_with_swivel(0, target, pitch, swivel)
            .unwrap();
        track[i].time = time;
        track[i].orientations[0] =
            profile.encode_hand_orientation(0, target, pitch, swivel, hand.rotation);
        let restored = profile
            .solve_hand_with_swivel_and_orientation(
                0,
                target,
                pitch,
                swivel,
                track[i].orientations[0],
            )
            .unwrap();
        assert!(distance(restored.middle, hand.middle) < 1e-10);
        assert!(distance(restored.end, hand.end) < 1e-10);
    }
    let evaluate = |time: f64| {
        let pitch = -0.1 + 2. * (time - 0.75);
        let control = crate::hand_coordination::at(&track, time);
        profile
            .solve_hand_with_swivel_and_orientation(
                0,
                target,
                pitch,
                swivel,
                control.orientations[0],
            )
            .unwrap()
    };
    let times = [0.75 - 1. / 240., 0.75 + 1. / 240.];
    let angular_speed = |a: Quat, b: Quat| {
        2. * a
            .iter()
            .zip(b)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            .abs()
            .min(1.)
            .acos()
            / (times[1] - times[0])
    };
    let old = times.map(|time| {
        solve_legacy(
            &profile,
            0,
            target,
            -0.1 + 2. * (time - 0.75),
            &legacy,
            swivel,
        )
    });
    let prepared = times.map(evaluate);
    let old_speed = angular_speed(old[0].rotation, old[1].rotation);
    let prepared_speed = angular_speed(prepared[0].rotation, prepared[1].rotation);
    assert!(
        old_speed > 100.,
        "fixture must expose unstable legacy feedback: {old_speed}"
    );
    assert!(
        prepared_speed < old_speed * 0.5,
        "prepared orientation retained legacy amplification: {prepared_speed} versus {old_speed}"
    );
    for step in 0..=100 {
        let time = 0.72 + 0.06 * step as f64 / 100.;
        let hand = evaluate(time);
        assert!(distance(hand.tip, target) < 1e-10);
        assert!(distance(add(hand.end, profile.offset(0, hand.rotation)), target) < 1e-10);
    }
    let center = evaluate(0.75);
    evaluate(0.78);
    evaluate(0.72);
    assert_eq!(center.rotation, evaluate(0.75).rotation);
}
