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
fn target_dependent_neutral_wrists_keep_actual_contact_and_bone_lengths() {
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
                assert!(dot(hand_forward, unit(sub(solved.end, solved.middle))) > 1.0 - 1e-10);
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
        assert!(!solved.reachable);
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
    assert!(!solved.reachable);
    assert!(solved.tip.iter().all(|v| v.is_finite()));
}

#[test]
fn bounded_wrist_pitch_does_not_reintroduce_target_dependent_wrist_swing() {
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
            let delta = multiply(solved.rotation, inverse(profile.contact_rotations[0]));
            let hand_forward = rotate(delta, [0.0, 0.0, 1.0]);
            let swing = dot(hand_forward, unit(sub(solved.end, solved.middle)))
                .clamp(-1.0, 1.0)
                .acos();
            assert!(swing <= pitch.abs() + 1e-7);
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
