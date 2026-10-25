use super::*;
use crate::model::default_kit;
use crate::rig::HandControlFrame;
fn profile(scale: f64) -> RigProfile {
    let mut p = RigProfile::primitive();
    let y = 1.235 - (1.0 - scale) * 0.25;
    for side in 0..2 {
        let sign = if side == 0 { -1.0 } else { 1.0 };
        p.roots[side] = [sign * 0.19, y, 0.06];
        p.lengths[side] = [0.3 * scale, 0.27 * scale];
        let elbow = [
            p.roots[side][0] + sign * 0.24 * scale,
            y - 0.18 * scale,
            0.06,
        ];
        let wrist = [elbow[0], elbow[1], elbow[2] + 0.27 * scale];
        p.rest_tips[side] = crate::motion::add(wrist, p.offset(side, p.contact_rotations[side]));
    }
    let frame = HandControlFrame {
        forearm: [0.0, 0.0, 1.0],
        palm: [0.0, 1.0, 0.0],
        desired_palm: [0.0, 1.0, 0.0],
    };
    p.hand_frames = Some([frame.clone(), frame]);
    p
}
#[test]
fn bounded_common_fit_keeps_pedals_unchanged_and_is_idempotent() {
    for scale in [0.65, 0.8, 1.0] {
        let profile = profile(scale);
        let original = default_kit();
        let before = serde_json::to_string(&original).unwrap();
        let fitted = fit_kit(&profile, &original).unwrap();
        assert!(
            fitted.complete,
            "{scale} {:?}",
            fitted.unreachable_piece_ids
        );
        assert!((0.6..=1.0).contains(&fitted.scale));
        assert!(fitted.clearance_piece_ids.is_empty());
        assert!(kit_clearance_ids(&fitted.kit).is_empty());
        assert!((-0.25..=0.10).contains(&fitted.translation[2]));
        assert_eq!(before, serde_json::to_string(&original).unwrap());
        for (old, new) in original.iter().zip(&fitted.kit) {
            if hand_piece(old) {
                assert!((new.radius - old.radius * fitted.scale).abs() < 1e-12);
                let uniform =
                    transform(std::slice::from_ref(old), fitted.scale, fitted.translation);
                let shift = fitted
                    .piece_adjustments
                    .iter()
                    .find(|change| change.piece_id == old.id)
                    .map_or([0.0; 3], |change| change.translation);
                assert!(shift[0].abs() <= 0.30 + 1e-12);
                assert!((-0.25 - 1e-12..=0.25 + 1e-12).contains(&shift[1]));
                assert!((-0.3 - 1e-12..=0.3 + 1e-12).contains(&shift[2]));
                for (axis, offset) in shift.iter().enumerate() {
                    assert!(
                        (new.position[axis] - uniform[0].position[axis] - offset).abs() < 1e-10
                    );
                }
            } else {
                assert_eq!(
                    serde_json::to_string(old).unwrap(),
                    serde_json::to_string(new).unwrap()
                );
            }
        }
        let snare = fitted.kit.iter().find(|piece| piece.id == "snare").unwrap();
        assert!(snare_ready_fits(&profile, snare));
        for piece in &fitted.kit {
            assert!(piece_posture_fits(&profile, piece), "{}", piece.id);
        }
        for limb in 0..2 {
            for (lift, pitch) in [(0.0, 0.0), (0.055, -0.2), (0.11, -0.42)] {
                let target = [
                    snare.position[0],
                    snare.position[1] + lift,
                    snare.position[2],
                ];
                let solved = profile.solve_hand(limb, target, pitch).unwrap();
                assert!(solved.reachable);
                if target[1] <= profile.roots[limb][1] {
                    assert!(
                        solved.middle[1]
                            <= profile.roots[limb][1] + profile.lengths[limb][0] * 0.1 + 1e-9
                    );
                }
                assert!(
                    solved.end[2]
                        >= profile.roots[limb][2] + profile.lengths[limb][1] * 0.15 - 1e-9
                );
                assert!(crate::motion::distance(solved.tip, target) < 1e-9);
            }
        }
        let repeated = fit_kit(&profile, &fitted.kit).unwrap();
        assert_eq!(repeated.scale, 1.0);
        assert_eq!(repeated.translation, [0.0; 3]);
        assert!(repeated.piece_adjustments.is_empty());
        assert_eq!(
            serde_json::to_string(&repeated.kit).unwrap(),
            serde_json::to_string(&fitted.kit).unwrap()
        );
    }
}

#[test]
fn nominal_geometry_distinguishes_above_head_overlap_from_body_intersection() {
    let kit = default_kit();
    assert!(kit_clearance_ids(&kit).is_empty());
    let ride = body_envelope(kit.iter().find(|p| p.id == "ride").unwrap()).unwrap();
    let floor = body_envelope(kit.iter().find(|p| p.id == "tomLow").unwrap()).unwrap();
    assert!(
        (ride.center[0] - floor.center[0]).hypot(ride.center[2] - floor.center[2])
            < ride.radius + floor.radius
    );
    assert!(bodies_clear(ride, floor)); // A raised cymbal may project over a tom.
    let mut intersecting = kit;
    let tom = intersecting
        .iter()
        .find(|p| p.id == "tomHigh")
        .unwrap()
        .position;
    for hat in intersecting
        .iter_mut()
        .filter(|p| matches!(p.instrument, Instrument::Hihat | Instrument::OpenHat))
    {
        hat.position = [tom[0], tom[1] - 0.01, tom[2] - hat.radius * 0.5];
    }
    let ids = kit_clearance_ids(&intersecting);
    assert!(ids.iter().any(|id| id == "hihat"));
    assert!(ids.iter().any(|id| id == "tomHigh"));
    let fitted = fit_kit(&profile(1.0), &intersecting).unwrap();
    assert!(
        fitted.complete,
        "{:?} {:?}",
        fitted.unreachable_piece_ids, fitted.clearance_piece_ids
    );
    assert!(kit_clearance_ids(&fitted.kit).is_empty());
}
#[test]
fn annulus_reach_is_not_accepted_as_a_safe_near_snare_posture() {
    let profile = profile(1.0);
    let target = [-0.18, 1.05, 0.35];
    assert!(profile.reachable(0, target));
    assert!(!hand_posture_fits(&profile, 0, target));
    let mut kit = default_kit();
    kit.iter_mut()
        .find(|piece| piece.id == "snare")
        .unwrap()
        .position = target;
    let result = fit_kit(&profile, &kit).unwrap();
    assert!(result.complete);
    assert!(result
        .piece_adjustments
        .iter()
        .any(|change| change.piece_id == "snare"));
    let again = fit_kit(&profile, &result.kit).unwrap();
    assert_eq!(
        serde_json::to_string(&again.kit).unwrap(),
        serde_json::to_string(&result.kit).unwrap()
    );
    assert!(again.piece_adjustments.is_empty());
    assert_eq!(again.scale, 1.0);
    assert_eq!(again.translation, [0.0; 3]);
}
#[test]
fn incomplete_fit_remains_explicit_and_invalid_layouts_fail() {
    let mut profile = profile(0.65);
    profile.roots[1][0] = 3.0;
    profile.rest_tips[1][0] += 3.0 - 0.19;
    let fitted = fit_kit(&profile, &default_kit()).unwrap();
    assert!(!fitted.complete);
    assert!(!fitted.unreachable_piece_ids.is_empty());
    assert_eq!(
        serde_json::to_string(&fitted.kit).unwrap(),
        serde_json::to_string(&default_kit()).unwrap()
    );
    assert!(fitted.piece_adjustments.is_empty());
    assert!(fit_kit(&profile, &[]).is_err());
    let mut invalid = default_kit();
    invalid[0].position[0] = f64::NAN;
    assert!(fit_kit(&profile, &invalid).is_err());
    let mut uncoupled = default_kit();
    uncoupled
        .iter_mut()
        .find(|piece| piece.id == "openHat")
        .unwrap()
        .position[2] += 0.1;
    assert!(fit_kit(&profile, &uncoupled).is_err());
}
