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
                    let base = projected(reference.desired_palm, shaft);
                    let actual = projected(palm, shaft);
                    let roll = dot(shaft, cross(base, actual)).atan2(dot(base, actual));
                    assert!(
                        roll >= range[0] - 1e-9 && roll <= range[1] + 1e-9,
                        "roll {roll} outside {range:?}"
                    );
                    assert!(palm[1] < 0.15, "matched grip turned palm up: {palm:?}");
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
                    let a = profile.solve_hand(0, target, -0.2).unwrap();
                    if !a.reachable {
                        continue;
                    }
                    for direction in [[1e-6, 0.0, 0.0], [0.0, 1e-6, 0.0], [0.0, 0.0, 1e-6]] {
                        let roll = profile.hand_roll(0, target, -0.2, &a);
                        let b = profile
                            .solve_hand_with_swivel_and_roll(
                                0,
                                add(target, direction),
                                -0.2,
                                0.,
                                roll,
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

/// The old coordinate update optimizes against the preceding elbow, not the
/// final elbow produced from that orientation. Retained only as a regression
/// witness for the coupled pose objective.
fn lagged_roll_solution(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    pitch: f64,
    swivel: f64,
) -> HandSolution {
    let reference = &profile.hand_frames.as_ref().unwrap()[limb];
    let base = approach_rotation(profile, limb, target, pitch, reference);
    let mut rotation = base;
    let shaft = rotate(base, unit(profile.tip_offsets[limb]));
    let neutral = rotate(
        multiply(base, inverse(profile.contact_rotations[limb])),
        reference.forearm,
    );
    let a = sub(neutral, mul(shaft, dot(neutral, shaft)));
    let [minimum, maximum] = reference.palm_roll_range.unwrap();
    for _ in 0..ROLL_REFINEMENTS {
        let hand = solve_wrist(profile, limb, target, rotation, swivel);
        let actual = unit(sub(hand.end, hand.middle));
        let b = sub(actual, mul(shaft, dot(actual, shaft)));
        let roll = bounded_roll(dot(shaft, cross(a, b)), dot(a, b), minimum, maximum);
        rotation = normalize(multiply(axis_rotation(shaft, roll), base));
    }
    solve_wrist(profile, limb, target, rotation, swivel)
}

#[test]
fn final_coupled_roll_preserves_contact_and_avoids_lagged_elbow_extremes() {
    // This is the app-authored procedural rig, not imported avatar geometry.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    for limb in 0..2 {
        let sign = if limb == 0 { -1. } else { 1. };
        let target = [sign * 0.35, 1.25, 0.7];
        let swivel = -sign * -0.2;
        let lagged = lagged_roll_solution(&profile, limb, target, -0.5, swivel);
        assert!(profile.wrist_deviation(limb, &lagged).to_degrees() > 55.);
        assert!(
            crate::posture::arm_posture(&profile, limb, &lagged)
                .elevation
                .to_degrees()
                > 120.
        );
        let hand = profile
            .solve_hand_with_swivel(limb, target, -0.5, swivel)
            .unwrap();
        assert!(crate::posture::anatomical(&profile, limb, target, &hand));
        assert!(profile.wrist_deviation(limb, &hand).to_degrees() < 12.);
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
        let shaft = rotate(hand.rotation, unit(profile.tip_offsets[limb]));
        assert!(
            distance(
                shaft,
                rotate(lagged.rotation, unit(profile.tip_offsets[limb]))
            ) < 1e-10
        );
        let reference = &profile.hand_frames.as_ref().unwrap()[limb];
        let palm = rotate(
            multiply(hand.rotation, inverse(profile.contact_rotations[limb])),
            reference.palm,
        );
        assert!(palm[1] <= 1e-10);
        // Re-evaluation remains stateless and this local swivel neighbourhood
        // cannot jump between the old high elbow and the repaired branch.
        let mut previous = hand;
        let roll = profile.hand_roll(limb, target, -0.5, &hand);
        for step in 1..=40 {
            let next = profile
                .solve_hand_with_swivel_and_roll(
                    limb,
                    target,
                    -0.5,
                    swivel + sign * step as f64 * 0.0005,
                    roll,
                )
                .unwrap();
            assert!(crate::posture::anatomical(&profile, limb, target, &next));
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
fn fixed_roll_replay_is_continuous_at_static_minimum_switches() {
    // App-authored native Posture Study recovery. The first pair exposed a
    // skipped interior search (26 cm elbow jump); the second exposed global
    // argmin switching between valid branches (11 cm). Both changes are tiny.
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
        let roll = profile.hand_roll(0, a, pitch[0], &anchor);
        let mut previous: Option<HandSolution> = None;
        for step in 0..=100 {
            let t = step as f64 / 100.;
            let target = add(mul(a, 1. - t), mul(b, t));
            let hand = profile
                .solve_hand_with_swivel_and_roll(
                    0,
                    target,
                    pitch[0] * (1. - t) + pitch[1] * t,
                    swivel[0] * (1. - t) + swivel[1] * t,
                    roll,
                )
                .unwrap();
            assert!(crate::posture::anatomical(&profile, 0, target, &hand));
            assert!(distance(hand.tip, target) < 1e-10);
            if let Some(previous) = previous {
                assert!(distance(previous.middle, hand.middle) < 0.002);
                assert!(
                    dot(
                        rotate(previous.rotation, [1., 0., 0.]),
                        rotate(hand.rotation, [1., 0., 0.])
                    ) > 0.9999
                );
            }
            previous = Some(hand);
        }
    }
}

#[test]
fn selected_roll_round_trips_exact_pose_and_rejects_invalid_controls() {
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
                let roll = profile.hand_roll(limb, target, pitch, &hand).unwrap();
                let replay = profile
                    .solve_hand_with_swivel_and_roll(limb, target, pitch, 0.4, Some(roll))
                    .unwrap();
                assert!(distance(hand.middle, replay.middle) < 1e-10);
                assert!(distance(hand.end, replay.end) < 1e-10);
                assert_eq!(hand.reachable, replay.reachable);
                let mut negative = hand;
                negative.rotation = negative.rotation.map(|v| -v);
                assert!(
                    (profile.hand_roll(limb, target, pitch, &negative).unwrap() - roll).abs()
                        < 1e-10
                );
            }
        }
        let target = profile.rest_tips[limb];
        assert!(profile
            .solve_hand_with_swivel_and_roll(limb, target, 0., 0., Some(f64::NAN))
            .is_none());
        assert!(profile
            .solve_hand_with_swivel_and_roll(limb, target, 0., 0., Some(4.))
            .is_none());
    }
    for palm in [[0., 1., 0.], [0., -1., 0.]] {
        let mut explicit = self::profile(1., palm);
        explicit.hand_frames.as_mut().unwrap()[0].palm_roll_range = Some([-0.15, 0.15]);
        let target = [-0.2, 0.95, 0.5];
        let hand = explicit.solve_hand(0, target, 0.1).unwrap();
        let roll = explicit.hand_roll(0, target, 0.1, &hand).unwrap();
        let replay = explicit
            .solve_hand_with_swivel_and_roll(0, target, 0.1, 0., Some(roll))
            .unwrap();
        assert!(distance(hand.middle, replay.middle) < 1e-10);
        assert!(distance(hand.end, replay.end) < 1e-10);
    }
    let legacy = self::profile(1., [0., 1., 0.]);
    let target = legacy.rest_tips[0];
    let hand = legacy.solve_hand(0, target, 0.).unwrap();
    assert!(legacy.hand_roll(0, target, 0., &hand).is_none());
    assert!(legacy
        .solve_hand_with_swivel_and_roll(0, target, 0., 0., Some(0.))
        .is_none());
    assert_eq!(
        legacy
            .solve_hand_with_swivel_and_roll(0, target, 0., 0., None)
            .unwrap()
            .middle,
        hand.middle
    );
}

#[test]
fn valid_static_witness_keeps_exact_contact_and_physical_tool_segment() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let mut accepted = 0;
    for limb in 0..2 {
        for target in [
            [-0.4, 0.9, 0.4],
            [0.3, 1.1, 0.65],
            [-0.35, 1.25, 0.7],
            [0.35, 1.25, 0.7],
        ] {
            for pitch in [-0.5, -0.2, 0.0] {
                for swivel in [-0.4, 0., 0.4] {
                    let witness = lagged_roll_solution(&profile, limb, target, pitch, swivel);
                    if !crate::posture::anatomical(&profile, limb, target, &witness) {
                        continue;
                    }
                    accepted += 1;
                    let selected = profile
                        .solve_hand_with_swivel(limb, target, pitch, swivel)
                        .unwrap();
                    assert_eq!(selected.middle, witness.middle);
                    assert_eq!(selected.rotation, witness.rotation);
                    assert!(distance(selected.tip, target) < 1e-10);
                    let (butt, tip) =
                        crate::equipment_motion::stick_segment(&profile, limb, &selected);
                    let range = profile.hand_frames.as_ref().unwrap()[limb]
                        .palm_roll_range
                        .unwrap();
                    for roll in [range[0], (range[0] + range[1]) * 0.5, range[1]] {
                        let other = profile
                            .solve_hand_with_swivel_and_roll(
                                limb,
                                target,
                                pitch,
                                swivel,
                                Some(roll),
                            )
                            .unwrap();
                        if other.reachable {
                            let (other_butt, other_tip) =
                                crate::equipment_motion::stick_segment(&profile, limb, &other);
                            assert!(distance(butt, other_butt) < 1e-10);
                            assert!(distance(tip, other_tip) < 1e-10);
                        }
                    }
                }
            }
        }
    }
    assert!(
        accepted > 8,
        "independently authored rig must exercise valid witnesses"
    );
}
