use super::*;
use crate::model::default_kit;
use crate::rig::HandControlFrame;

#[test]
fn final_snare_miss_check_preserves_a_refined_only_ready_witness() {
    // Independently authored body dimensions, not measurements from an avatar.
    // This ready pose needs an intermediate elbow swivel: contact probes pass,
    // but the coarse five-angle pair search has no witness. Final certification
    // must preserve the same refined witness as the runtime posture solver.
    let mut p = profile(1.0);
    p.roots[0][0] = -0.24;
    p.roots[1][0] = 0.24;
    p.body_geometry = Some(
        serde_json::from_value(serde_json::json!({
            "version": 1, "source": "skeleton-fallback", "sampleCount": 0,
            "torso": {"center": [0., 0.95, 0.06], "halfExtents": [0.13, 0.24, 0.08]},
            "armRadii": [[0.025, 0.02], [0.025, 0.02]], "handRadii": [0.025, 0.025]
        }))
        .unwrap(),
    );
    p.validate().unwrap();
    let mut snare = default_kit()
        .into_iter()
        .find(|piece| piece.instrument == Instrument::Snare)
        .unwrap();
    snare.position = [-0.1, 0.78, 0.30];
    let kit = [snare];
    let before = serde_json::to_string(&(&p, &kit)).unwrap();
    assert!(!snare_ready_fits_cached(&p, &kit[0], &FitCache::default()));
    let refined = FitCache::default();
    refined.refine.set(true);
    assert!(snare_ready_fits_cached(&p, &kit[0], &refined));
    assert!(piece_targets(&p, &kit[0])
        .iter()
        .all(|&target| { (0..2).all(|limb| hand_posture_fits(&p, limb, target)) }));
    assert!(snare_ready_fits(&p, &kit[0]));
    assert!(missed(&p, &kit, true).is_empty());
    assert!(missed(&p, &kit, true).is_empty());
    assert_eq!(serde_json::to_string(&(&p, &kit)).unwrap(), before);
}

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
fn bounded_common_fit_preserves_kick_and_fits_linked_hihat_idempotently() {
    for scale in [1.0, 0.8, 0.65] {
        let profile = profile(scale);
        let original = default_kit();
        let before = serde_json::to_string(&original).unwrap();
        let fitted = fit_kit(&profile, &original).unwrap();
        assert!(fitted.search_candidates <= MAX_FIT_LAYOUT_CANDIDATES);
        if !fitted.complete {
            // Adding physical equipment can make a formerly reachable skeleton
            // fixture infeasible within the bounded compact-layout search.
            assert!(scale < 1.0, "reference-size fixture must fit");
            assert!(
                !fitted.unreachable_piece_ids.is_empty() || !fitted.clearance_piece_ids.is_empty()
            );
            for (old, new) in original.iter().zip(&fitted.kit) {
                assert_eq!(old.position, new.position);
                assert_eq!(old.radius, new.radius);
            }
            assert!(fitted.piece_adjustments.is_empty());
            continue;
        }
        // The accepted transaction adopts both the seat and pedal frame before
        // any posture check or subsequent fit consumes the resulting kit.
        let profile = fit_profile::candidate(&profile, fitted.seat_offset, &fitted.kit);
        assert!((MIN_FIT_SCALE..=1.0).contains(&fitted.scale));
        assert!(fitted.clearance_piece_ids.is_empty());
        assert!(kit_clearance_ids(&fitted.kit).is_empty());
        assert!((-0.25..=0.10).contains(&fitted.translation[2]));
        assert_eq!(before, serde_json::to_string(&original).unwrap());
        for (old, new) in original.iter().zip(&fitted.kit) {
            if hand_piece(old) {
                let expected_radius = if matches!(
                    old.instrument,
                    Instrument::Snare
                        | Instrument::TomHigh
                        | Instrument::TomMid
                        | Instrument::TomLow
                ) {
                    old.radius * fitted.scale
                } else {
                    old.radius
                };
                assert!((new.radius - expected_radius).abs() < 1e-12);
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
                assert_eq!(old.radius, new.radius);
                if old.instrument == Instrument::Kick {
                    assert_eq!(old.position, new.position);
                } else {
                    assert!((old.position[0] - new.position[0]).abs() <= 0.25);
                    assert!((old.position[2] - new.position[2]).abs() <= 0.25);
                    for pitch in [0.0, -0.09, -0.18] {
                        let q = crate::equipment::pedal_rotation(
                            new,
                            profile.contact_rotations[2],
                            pitch,
                        );
                        let ankle = crate::motion::sub(
                            crate::equipment::pedal_contact(new, pitch),
                            profile.offset(2, q),
                        );
                        let d = crate::motion::distance(profile.roots[2], ankle);
                        assert!(d < profile.lengths[2].iter().sum::<f64>());
                    }
                }
            }
        }
        let hat = fitted
            .kit
            .iter()
            .find(|p| p.instrument == Instrument::Hihat)
            .unwrap();
        assert!(
            crate::motion::distance(
                hat.position,
                crate::equipment::anchored_hihat_position(hat, &fitted.kit)
            ) < 1e-9
        );
        let snare = fitted.kit.iter().find(|piece| piece.id == "snare").unwrap();
        assert!(snare_ready_fits(&profile, snare));
        for piece in &fitted.kit {
            assert!(
                piece_posture_fits(&profile, piece, &FitCache::default()),
                "{}",
                piece.id
            );
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
        assert!(repeated.complete);
        assert_eq!(repeated.seat_offset, [0.0; 3]);
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
fn unchanged_pedal_fit_requires_knee_posture_as_well_as_reach_and_clearance() {
    // Independently authored pedal-only rigs isolate knee posture from hand and
    // arrangement search. Every case has reachable real bones and clear hardware.
    // The low/high hips violate opposite knee bounds; the middle case is valid.
    for (limb, instrument, x) in [(2, Instrument::PedalHat, -1.), (3, Instrument::Kick, 1.)] {
        for (hip_height, expected_complete) in [(0.30, false), (0.66, true), (0.91, false)] {
            let mut profile = RigProfile::primitive();
            // Keep the untested leg's idle foot reachable at every preparation
            // pitch as well. The primitive's rearward hip otherwise reaches its
            // outer annulus at -0.18 even though this kit has no pedal for it.
            for side in 2..4 {
                profile.roots[side][1] = 0.66;
                profile.roots[side][2] = 0.14;
                profile.grip_offsets[side] = [0., -0.025, 0.];
                profile.tip_offsets[side] = [0., 0., 0.16];
            }
            profile.roots[limb] = [x, hip_height, 0.14];
            let mut pedal = default_kit()
                .into_iter()
                .find(|p| p.instrument == instrument)
                .unwrap();
            pedal.position = [x, 0.08, 0.55];
            pedal.pedal_forward = None;
            let kit = vec![pedal];
            let profile = fit_profile::candidate(&profile, [0.; 3], &kit);
            let geometry = crate::equipment::kit_geometry(&kit);
            assert!(geometry.unresolved_piece_ids.is_empty());
            assert!(
                lower_limb::violations(&profile, &kit, &geometry).is_empty(),
                "hip={hip_height} limb={limb}: {:?}",
                lower_limb::diagnostics(&profile, &kit, &geometry)
            );
            assert_eq!(fit_profile::feet_fit(&profile, &kit), expected_complete);

            let result = fit_kit(&profile, &kit).unwrap();
            assert_eq!(
                result.complete, expected_complete,
                "hip={hip_height} limb={limb}"
            );
            assert_eq!(result.seat_offset, [0.; 3]);
            assert_eq!(result.kit[0].position, kit[0].position);
            assert_eq!(result.kit[0].radius, kit[0].radius);
            if expected_complete {
                let again = fit_kit(&profile, &result.kit).unwrap();
                assert!(again.complete);
                assert_eq!(again.seat_offset, [0.; 3]);
                assert_eq!(again.search_candidates, 0);
            } else {
                assert!(result.unreachable_piece_ids.contains(&kit[0].id));
            }
        }
    }
}

#[test]
fn stand_phase_pruning_preserves_invariant_shell_rejection_without_spending_budget() {
    // An independently authored negative case places the upper thigh axes in
    // the snare shell. Turning the tripod cannot move that shell or either hip.
    let mut profile = RigProfile::primitive();
    profile.roots[2] = [-0.1, 0.66, 0.14];
    profile.roots[3] = [0.1, 0.66, 0.14];
    for limb in 2..4 {
        profile.grip_offsets[limb] = [0., -0.025, 0.];
        profile.tip_offsets[limb] = [0., 0., 0.16];
    }
    let mut snare = default_kit()
        .into_iter()
        .find(|p| p.instrument == Instrument::Snare)
        .unwrap();
    snare.position = [0., 0.79, 0.32];
    snare.stand_rotation = Some(0.7);
    let mut kit = vec![snare];
    let original = serde_json::to_string(&kit).unwrap();
    let mut shell_only = crate::equipment::kit_geometry(&kit);
    shell_only.supports.clear();
    assert_eq!(
        lower_limb::violations(&profile, &kit, &shell_only),
        vec!["left-leg-clearance:snare", "right-leg-clearance:snare"]
    );

    // Exhaust the unpruned phase set using full routed hardware and the exact
    // lower-limb check, independently of the new early-rejection branch.
    for step in 0..24 {
        let mut phase_kit = kit.clone();
        phase_kit[0].stand_rotation =
            Some(-std::f64::consts::PI + step as f64 * std::f64::consts::PI / 12.);
        let full_geometry = crate::equipment::kit_geometry(&phase_kit);
        let failures = lower_limb::violations(&profile, &phase_kit, &full_geometry);
        assert_eq!(
            failures,
            vec!["left-leg-clearance:snare", "right-leg-clearance:snare"],
            "phase step={step}"
        );
    }

    let cache = FitCache::default();
    assert!(!choose_leg_clear_stand_phase(
        &profile,
        &mut kit,
        0,
        &[],
        &cache
    ));
    assert_eq!(cache.candidates.get(), 0);
    assert!(!cache.exhausted.get());
    assert_eq!(serde_json::to_string(&kit).unwrap(), original);
}

#[test]
fn geometry_rejects_obstructed_approaches_without_applying_unsafe_partial_repair() {
    let kit = default_kit();
    // The shared default seed is now physically clear. An authored obstruction
    // must still be rejected; this test does not depend on obsolete defaults.
    assert!(kit_clearance_ids(&kit).is_empty());
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
    if fitted.complete {
        assert!(kit_clearance_ids(&fitted.kit).is_empty());
    } else {
        assert!(!fitted.clearance_piece_ids.is_empty());
        for (old, new) in intersecting.iter().zip(&fitted.kit) {
            assert_eq!(old.position, new.position);
            assert_eq!(old.radius, new.radius);
        }
        assert!(fitted.piece_adjustments.is_empty());
    }
    assert!(fitted.search_candidates <= MAX_FIT_LAYOUT_CANDIDATES);
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
    assert!(
        result.complete,
        "misses={:?} clearance={:?} budget={} candidates={}",
        result.unreachable_piece_ids,
        result.clearance_piece_ids,
        result.search_limit_reached,
        result.search_candidates
    );
    let profile = fit_profile::candidate(&profile, result.seat_offset, &result.kit);
    let fitted_snare = result.kit.iter().find(|p| p.id == "snare").unwrap();
    assert_ne!(fitted_snare.position, target);
    assert!(snare_ready_fits(&profile, fitted_snare));
    let again = fit_kit(&profile, &result.kit).unwrap();
    assert!(again.complete);
    assert_eq!(again.seat_offset, [0.0; 3]);
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
        fitted
            .kit
            .iter()
            .map(|p| (&p.id, p.position, p.radius))
            .collect::<Vec<_>>(),
        default_kit()
            .iter()
            .map(|p| (&p.id, p.position, p.radius))
            .collect::<Vec<_>>()
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

#[test]
fn measured_body_pair_capability_and_ready_pose_share_the_fit_contract() {
    let mut profile = profile(1.0);
    profile.body_geometry = Some(crate::rig::BodyGeometry {
        version: 1,
        source: "mesh-estimate".into(),
        torso: crate::rig::TorsoGeometry {
            center: [0.0, 0.98, 0.06],
            half_extents: [0.15, 0.22, 0.12],
        },
        arm_radii: [[0.03, 0.024]; 2],
        hand_radii: [0.03; 2],
        leg_radii: None,
        foot_radii: None,
        leg_boxes: None,
        leg_rest_frames: None,
        foot_boxes: None,
        hand_centers: None,
        hand_capsules: None,
        sample_count: 1000,
    });
    let result = fit_kit(&profile, &default_kit()).unwrap();
    assert!(
        result.complete,
        "{:?} {:?}",
        result.unreachable_piece_ids, result.capabilities
    );
    let profile = fit_profile::candidate(&profile, result.seat_offset, &result.kit);
    let audit = result.capabilities.as_ref().unwrap();
    assert_eq!(audit.geometry_source, "mesh-estimate");
    assert!(audit.required_pairs_available);
    assert_eq!(audit.pair_checks, 64); // Eight physical hand-played surfaces.
    assert_eq!(audit.hand_target_checks, 16);
    assert!(audit.available_pairs > 0);
    let crashes: Vec<_> = result
        .kit
        .iter()
        .filter(|p| p.instrument == Instrument::Crash)
        .collect();
    let equipment = crate::equipment_motion::EquipmentContext::for_kit(&result.kit);
    assert!(crashes.len() >= 2);
    assert!(
        [false, true].into_iter().any(|swap| {
            let a = crashes[usize::from(swap)];
            let b = crashes[usize::from(!swap)];
            let observer =
                std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
            let targets = [
                crate::contact::target(a, "crash", observer),
                crate::contact::target(b, "crash", observer),
            ];
            crate::posture::pair_solution_with_equipment(
                &profile,
                targets,
                [0.; 2],
                equipment.as_deref(),
                [Some(&a.id), Some(&b.id)],
            )
            .is_some()
        }),
        "complete fitting must support simultaneous crash impacts, not only raised ready tips"
    );
    let pose = crate::posture::ready_pose(&profile, &result.kit).unwrap();
    let inspected = crate::clearance::inspect(&profile, &pose);
    assert!(inspected.torso_clearance_meters >= crate::posture::MARGIN - 1e-6);
    assert!(inspected.arm_clearance_meters >= crate::posture::MARGIN - 1e-6);
    assert!(inspected.stick_clearance_meters >= crate::posture::MARGIN - 1e-6);
    assert!(inspected.cross_clearance_meters >= crate::posture::MARGIN - 1e-6);
    assert!(inspected.bone_length_error_meters < 1e-6);
    let snare = result.kit.iter().find(|p| p.id == "snare").unwrap();
    let expected = crate::posture::ready_pair(&profile, snare, snare);
    for (limb, target) in expected.iter().enumerate() {
        let actual = [
            pose[limb * 12 + 9] as f64,
            pose[limb * 12 + 10] as f64,
            pose[limb * 12 + 11] as f64,
        ];
        assert!(crate::motion::distance(actual, *target) < 1e-6);
    }
    assert_eq!(
        pose,
        crate::posture::ready_pose(&profile, &result.kit).unwrap()
    );
    let old = serde_json::to_string(&result.kit).unwrap();
    let repeated = fit_kit(&profile, &result.kit).unwrap();
    assert!(repeated.complete);
    assert_eq!(repeated.seat_offset, [0.0; 3]);
    assert_eq!(repeated.scale, 1.0);
    assert_eq!(repeated.translation, [0.0; 3]);
    assert!(repeated.piece_adjustments.is_empty());
    assert_eq!(old, serde_json::to_string(&repeated.kit).unwrap());
}

#[test]
fn measured_body_is_validated_and_turns_without_changing_extents() {
    let mut p = profile(1.0);
    p.torso_pivot = Some([0.0, 0.9, 0.06]);
    p.body_geometry = Some(crate::rig::BodyGeometry {
        version: 1,
        source: "mesh-estimate".into(),
        torso: crate::rig::TorsoGeometry {
            center: [0.02, 1.0, 0.10],
            half_extents: [0.14, 0.20, 0.09],
        },
        arm_radii: [[0.03, 0.02]; 2],
        hand_radii: [0.025; 2],
        leg_radii: None,
        foot_radii: None,
        leg_boxes: None,
        leg_rest_frames: None,
        foot_boxes: None,
        hand_centers: None,
        hand_capsules: None,
        sample_count: 100,
    });
    p.validate().unwrap();
    let turned = p.turned(0.2);
    assert_ne!(
        turned.body_geometry.as_ref().unwrap().torso.center,
        p.body_geometry.as_ref().unwrap().torso.center
    );
    assert_eq!(
        turned.body_geometry.as_ref().unwrap().torso.half_extents,
        p.body_geometry.as_ref().unwrap().torso.half_extents
    );
    for field in [
        "version",
        "source",
        "sampleCount",
        "armRadii",
        "handRadii",
        "legRadii",
        "footRadii",
        "legBoxes",
        "footBoxes",
    ] {
        let mut value = serde_json::to_value(&p).unwrap();
        value["bodyGeometry"][field] = match field {
            "version" => serde_json::json!(2),
            "source" => serde_json::json!("unverified"),
            "sampleCount" => serde_json::json!(1000001),
            "armRadii" | "legRadii" => serde_json::json!([[0.0, 0.02], [0.03, 0.02]]),
            "legBoxes" => serde_json::json!(vec![
                vec![
                    serde_json::json!({"center":[0.,0.,0.],"halfExtents":[0.,0.1,0.1]});
                    2
                ];
                2
            ]),
            "footBoxes" => serde_json::json!(vec![
                serde_json::json!({"center":[4.,0.,0.],"halfExtents":[0.1,0.1,0.1]});
                2
            ]),
            _ => serde_json::json!([0.9, 0.025]),
        };
        assert!(
            crate::rig::RigProfile::from_json(&value.to_string()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn native_ready_pose_fits_real_equipment_without_stacked_heads_or_buried_shafts() {
    // Generated by reference-layout-review.mjs --mode inputs --models native.
    // This current-default fixture includes current hand frames, measured leg/foot
    // envelopes, the yawed hi-hat pedal and the approved Studio arrangement. Old palm-up frames
    // and untilted hardware do not represent the current adapter's measurements.
    // No downloaded avatar or private media is bundled.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-equipment.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let kit: Vec<KitPiece> = serde_json::from_value(fixture["kit"].clone()).unwrap();
    let result = fit_kit(&profile, &kit).unwrap();
    assert!(
        result.complete,
        "{:?} {:?} budget={}",
        result.unreachable_piece_ids, result.clearance_piece_ids, result.search_limit_reached
    );
    let profile = fit_profile::candidate(&profile, result.seat_offset, &result.kit);
    assert!(
        result.scale >= 0.8,
        "native kit should not require toy-sized drums"
    );
    assert!(result.search_candidates <= MAX_FIT_LAYOUT_CANDIDATES);
    let ctx = crate::equipment_motion::EquipmentContext {
        kit: result.kit.clone(),
        geometry: crate::equipment::kit_geometry(&result.kit),
    };
    assert!(ctx.geometry.unresolved_piece_ids.is_empty());
    assert!(layout::violations(&profile, &result.kit).is_empty());
    assert!(lower_limb::violations(&profile, &result.kit, &ctx.geometry).is_empty());
    for original in kit.iter().filter(|p| {
        matches!(
            p.instrument,
            Instrument::Hihat | Instrument::OpenHat | Instrument::Crash | Instrument::Ride
        )
    }) {
        let fitted = result.kit.iter().find(|p| p.id == original.id).unwrap();
        assert_eq!(
            fitted.radius, original.radius,
            "cymbal diameters remain nominal"
        );
    }
    assert!(result
        .kit
        .iter()
        .all(|p| equipment_posture_fits(&profile, p, &ctx)));
    assert!(crate::posture::required_pairs(&profile, &result.kit).is_empty());
    assert!(crate::posture::ready_pose(&profile, &result.kit).is_ok());
    let again = fit_kit(&profile, &result.kit).unwrap();
    assert!(again.complete);
    assert_eq!(again.seat_offset, [0.0; 3]);
    assert_eq!(again.translation, [0.0; 3]);
    assert!(again.piece_adjustments.is_empty());
    assert_eq!(again.scale, 1.0);
    assert_eq!(
        serde_json::to_string(&result.kit).unwrap(),
        serde_json::to_string(&again.kit).unwrap()
    );
}
#[test]
fn fit_budget_stops_before_allocating_more_candidate_work() {
    let cache = FitCache::default();
    cache.candidates.set(MAX_FIT_LAYOUT_CANDIDATES - 1);
    assert!(cache.candidate());
    assert!(!cache.candidate());
    assert_eq!(cache.candidates.get(), MAX_FIT_LAYOUT_CANDIDATES);
    assert!(cache.exhausted.get());
}

#[test]
fn required_pair_checks_reuse_geometry_without_changing_results() {
    let mut kit = default_kit();
    for piece in &mut kit {
        piece.geometry = Some(crate::equipment::local_geometry(piece));
    }
    let ctx = crate::equipment_motion::EquipmentContext::for_kit(&kit).unwrap();
    for scale in [1.0, 0.8] {
        let profile = profile(scale);
        assert_eq!(
            crate::posture::required_pairs(&profile, &kit),
            crate::posture::required_pairs_with_equipment(&profile, &kit, Some(&ctx))
        );
    }
}

#[test]
fn reference_layout_rejects_reachable_but_reordered_parts() {
    let profile = profile(1.0);
    let kit = default_kit();
    assert!(layout::violations(&profile, &kit).is_empty());
    let snare_x = kit.iter().find(|p| p.id == "snare").unwrap().position[0];
    let floor_x = kit.iter().find(|p| p.id == "tomLow").unwrap().position[0];
    // Move the floor tom to the wrong side of the snare, independently of the seed depth.
    let floor_wrong_sector = [snare_x - floor_x - 0.2, 0., 0.];
    for (id, translation, expected) in [
        ("tomMid", [0., -0.17, -0.32], "rack-row"),
        ("tomHigh", [0., 0., -0.4], "tomHigh-ahead-of-snare"),
        ("tomLow", floor_wrong_sector, "floor-flank"),
        ("snare", [0.2, 0., 0.], "snare-body-center"),
        ("hihat", [0.6, 0., 0.], "hihat-left-above-snare"),
        ("ride", [-0.8, 0., 0.], "ride-right-sector"),
        ("crashRight", [-0.8, 0., 0.], "crashRight-outer-sector"),
    ] {
        let mut altered = kit.clone();
        let p = altered.iter_mut().find(|p| p.id == id).unwrap();
        for (axis, offset) in translation.iter().enumerate() {
            p.position[axis] += offset;
        }
        assert!(
            layout::violations(&profile, &altered)
                .iter()
                .any(|v| v == expected),
            "{id}"
        );
    }
}

#[test]
fn compact_reference_layout_preserves_relationships_and_body_centre() {
    let mut profile = profile(1.0);
    let translation = [0.35, 0.1, 0.15];
    let mut kit = default_kit();
    for p in &mut kit {
        p.radius *= MIN_FIT_SCALE;
        p.position = std::array::from_fn(|i| p.position[i] * MIN_FIT_SCALE + translation[i]);
    }
    profile.pelvis = std::array::from_fn(|i| profile.pelvis[i] * MIN_FIT_SCALE + translation[i]);
    assert!(layout::violations(&profile, &kit).is_empty());
}

#[test]
fn nonstandard_tom_roles_do_not_panic_in_group_search() {
    let mut kit = default_kit();
    kit.iter_mut()
        .find(|p| p.instrument == Instrument::TomMid)
        .unwrap()
        .instrument = Instrument::TomHigh;
    assert!(fit_kit(&profile(1.0), &kit).is_ok());
}

#[test]
fn family_quota_exhaustion_keeps_siblings_within_the_shared_budget() {
    let cache = FitCache::default();
    cache.limit.set(7);
    for expected in [3, 6, 7] {
        cache.scoped(3, || {
            while cache.candidate() {}
            assert!(cache.exhausted.get());
        });
        assert_eq!(cache.candidates.get(), expected);
        assert_eq!(cache.exhausted.get(), expected == 7);
        assert_eq!(cache.limit.get(), 7);
    }
    assert!(!cache.candidate());
    assert_eq!(cache.candidates.get(), 7);
}

#[test]
fn incomplete_compact_search_visits_all_coarse_shell_families() {
    let mut p = profile(1.0);
    p.lengths[0] = [0.05, 0.05];
    p.lengths[1] = [0.05, 0.05];
    for limb in 0..2 {
        let wrist = crate::motion::add(p.roots[limb], [0., -0.06, 0.04]);
        p.rest_tips[limb] = crate::motion::add(wrist, p.offset(limb, p.contact_rotations[limb]));
    }
    p.validate().unwrap();
    let cache = FitCache::default();
    cache.refine.set(true);
    cache.limit.set(24_000);
    let result = uniform_fit(&p, &default_kit(), &cache, false).unwrap();
    assert!(!result.complete);
    assert!(cache
        .family_visits
        .borrow()
        .starts_with(&[1.0, 0.92, 0.84, MIN_FIT_SCALE]));
    assert!(cache.candidates.get() <= 24_000);
}
