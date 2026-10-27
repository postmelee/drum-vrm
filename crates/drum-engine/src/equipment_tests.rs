use super::*;
use crate::model::default_kit;

#[test]
fn shared_default_kit_has_clear_fallback_geometry() {
    let g = kit_geometry(&default_kit());
    assert!(
        g.unresolved_piece_ids.is_empty(),
        "{:?}",
        g.unresolved_piece_ids
    );
}

#[test]
fn segment_box_checks_middle_parallel_corner_and_degenerate_cases() {
    let b = EquipmentBox {
        piece_id: "box".into(),
        center: [0.0; 3],
        half_extents: [0.5; 3],
        kind: "body",
    };
    assert!(
        (segment_box_clearance([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, &b) + 0.01).abs() < 1e-12
    );
    assert!(
        (segment_box_clearance([-1.0, 0.6, 0.0], [1.0, 0.6, 0.0], 0.01, &b) - 0.09).abs() < 1e-12
    );
    assert!(
        (segment_box_clearance([0.6, 0.6, 0.0], [0.6, 0.6, 0.0], 0.01, &b)
            - (0.02_f64.sqrt() - 0.01))
            .abs()
            < 1e-12
    );
    assert!(
        (segment_box_clearance([-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], 0.01, &b) + 0.01).abs() < 1e-12
    );
    assert_eq!(segment_box_clearance([0.0; 3], [0.0; 3], 0.0, &b), 0.0);
}

#[test]
fn tilted_playing_plane_clips_only_its_own_conservative_top() {
    let b = EquipmentBox {
        piece_id: "hat".into(),
        center: [0.0, -0.04, 0.0],
        half_extents: [0.2, 0.10, 0.2],
        kind: "body",
    };
    let n = 0.5_f64.sqrt();
    let surface = Surface {
        radial_profile: None,
        motion: None,
        center: [0.0; 3],
        normal: [0.0, n, -n],
        radius: 0.18,
    };
    let a = [0.0, 0.07, 0.0];
    let end = [0.0, 0.04, 0.0];
    assert!(segment_box_clearance(a, end, 0.006, &b) < 0.0);
    assert!(segment_surface_box_clearance(a, end, 0.006, &b, &surface) > 0.02);
    assert!(
        segment_surface_box_clearance([0.0, 0.02, 0.0], [0.0, -0.04, 0.0], 0.006, &b, &surface)
            < 0.0
    );
}

#[test]
fn vertically_disjoint_heads_are_not_a_playable_stack() {
    let kit = default_kit();
    let mut lower = kit.iter().find(|p| p.id == "tomHigh").unwrap().clone();
    let mut upper = kit.iter().find(|p| p.id == "tomMid").unwrap().clone();
    lower.position = [0.0, 0.8, 0.6];
    upper.position = [0.0, 1.04, 0.6];
    assert!(boxes_clear(&body(&lower).unwrap(), &body(&upper).unwrap()));
    assert!(!playable_pair(&lower, &upper));
    let geometry = kit_geometry(&[lower, upper]);
    assert_eq!(geometry.unresolved_piece_ids, vec!["tomHigh", "tomMid"]);
}

#[test]
fn actual_depth_and_radius_scale_together_and_invalid_geometry_is_rejected() {
    let mut p = default_kit().into_iter().find(|p| p.id == "snare").unwrap();
    p.geometry = Some(PieceGeometry {
        version: 1,
        reference_radius: p.radius,
        source: "mesh-bounds".into(),
        body: LocalBox {
            center: [0.0, -0.15, 0.0],
            half_extents: [0.2, 0.15, 0.2],
        },
        surface: Surface {
            radial_profile: None,
            motion: None,
            center: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            radius: 0.17,
        },
    });
    assert!(valid_piece_geometry(&p));
    p.radius *= 0.7;
    let b = body(&p).unwrap();
    assert!((b.half_extents[1] - 0.105).abs() < 1e-12);
    assert!((b.center[1] - (p.position[1] - 0.105)).abs() < 1e-12);
    assert!((surface(&p).radius - 0.119).abs() < 1e-12);
    p.geometry.as_mut().unwrap().surface.normal = [0.0; 3];
    assert!(!valid_piece_geometry(&p));
}

#[test]
fn generated_hardware_includes_tripods_baskets_pedals_and_fixed_contact_planes() {
    let kit = default_kit();
    let g = kit_geometry(&kit);
    assert_eq!(g.bodies.len(), 9);
    assert_eq!(g.pedals.len(), 4);
    for p in kit
        .iter()
        .filter(|p| matches!(p.instrument, Instrument::Kick | Instrument::PedalHat))
    {
        let board = g
            .pedals
            .iter()
            .find(|b| b.piece_id == p.id && b.kind == "board")
            .unwrap();
        assert!((board.center[1] + board.half_extents[1] - p.position[1]).abs() < 1e-12);
        assert!(g.supports.iter().filter(|s| s.piece_id == p.id).count() >= 3);
    }
    let snare: Vec<_> = g
        .supports
        .iter()
        .filter(|s| s.piece_id == "snare")
        .collect();
    assert!(snare.len() >= 8); // pole, collar, three feet, three basket fingers
    assert!(snare.iter().all(|s| s.radius > 0.0));
    assert_eq!(
        serde_json::to_string(&g).unwrap(),
        serde_json::to_string(&kit_geometry(&kit)).unwrap()
    );
}

#[test]
fn measured_tall_body_is_used_by_support_routing_and_inspection() {
    let mut kit = default_kit();
    let tom = kit.iter_mut().find(|p| p.id == "tomLow").unwrap();
    let mut descriptor = local_geometry(tom);
    descriptor.body.half_extents[1] = 0.50;
    descriptor.body.center[1] = -0.5;
    descriptor.source = "mesh-bounds".into();
    tom.geometry = Some(descriptor);
    let actual = kit_geometry(&kit);
    assert_eq!(
        actual
            .bodies
            .iter()
            .find(|b| b.piece_id == "tomLow")
            .unwrap()
            .half_extents[1],
        0.5
    );
    assert!(actual.unresolved_piece_ids.iter().any(|p| p == "tomLow"));
}

#[test]
fn semantic_tom_supports_and_booms_replace_universal_tripods() {
    let kit = default_kit();
    let g = kit_geometry(&kit);
    for id in ["tomHigh", "tomMid"] {
        let rods: Vec<_> = g.supports.iter().filter(|rod| rod.piece_id == id).collect();
        assert_eq!(rods.iter().filter(|rod| rod.role == "tomArm").count(), 2);
        let arms: Vec<_> = rods.iter().filter(|rod| rod.role == "tomArm").collect();
        assert_eq!(
            arms[0].start[1], arms[0].end[1],
            "L-arm first section is horizontal"
        );
        assert_eq!(arms[1].start[0], arms[1].end[0]);
        assert_eq!(arms[1].start[2], arms[1].end[2]);
        assert!(
            arms[1].end[1] > arms[1].start[1],
            "L-arm enters receiver vertically"
        );
        assert!(distance(arms[0].start, arms[0].end) <= hardware::MAX_TOM_ARM);
        assert!(rods.iter().any(|rod| rod.role == "suspensionBracket"));
        assert!(rods
            .iter()
            .all(|rod| rod.start[1] > 0.2 && rod.end[1] > 0.2));
    }
    let floor: Vec<_> = g
        .supports
        .iter()
        .filter(|rod| rod.piece_id == "tomLow")
        .collect();
    assert_eq!(
        floor.iter().filter(|rod| rod.role == "rubberFoot").count(),
        3
    );
    assert_eq!(
        floor
            .iter()
            .filter(|rod| rod.role == "suspensionBracket")
            .count(),
        3
    );
    let crash: Vec<_> = g
        .supports
        .iter()
        .filter(|rod| rod.piece_id == "crash")
        .collect();
    assert!(crash.iter().any(|rod| rod.start[1] > 0.5
        && rod.end[1] > rod.start[1]
        && (rod.end[0] - rod.start[0]).abs() > 0.1));
}

#[test]
fn shared_rack_holder_branches_clear_the_other_shell_at_unequal_heights() {
    let mut kit: Vec<_> = default_kit()
        .into_iter()
        .filter(|p| {
            matches!(
                p.instrument,
                Instrument::Kick | Instrument::TomHigh | Instrument::TomMid
            )
        })
        .collect();
    for piece in &mut kit {
        match piece.instrument {
            Instrument::TomHigh => {
                piece.position = [-0.26, 1.0, 0.72];
                piece.radius = 0.1524;
            }
            Instrument::TomMid => {
                piece.position = [0.18, 0.98, 0.74];
                piece.radius = 0.1651;
            }
            _ => {}
        }
    }
    let g = kit_geometry(&kit);
    for rod in g
        .supports
        .iter()
        .filter(|rod| rod.piece_id == "tomHigh" || rod.piece_id == "tomMid")
    {
        for other in g.bodies.iter().filter(|body| body.piece_id != rod.piece_id) {
            assert!(
                segment_box_clearance(rod.start, rod.end, rod.radius, other) >= CLEARANCE,
                "{} branch crosses {}",
                rod.piece_id,
                other.piece_id
            );
        }
    }
    assert_eq!(
        g.supports
            .iter()
            .filter(|rod| rod.role == "tomPost")
            .count(),
        1
    );
    assert_eq!(
        g.supports
            .iter()
            .filter(|rod| rod.role == "tomYoke")
            .count(),
        1
    );
    assert!(g
        .supports
        .iter()
        .filter(|rod| rod.role == "tomPost" || rod.role == "tomYoke")
        .all(|rod| rod.piece_id == "kick"));
}

#[test]
fn direct_hihat_alignment_and_pedal_contact_share_mechanical_frames() {
    let mut kit = default_kit();
    let index = kit
        .iter()
        .position(|piece| piece.instrument == Instrument::Hihat)
        .unwrap();
    kit[index].position = anchored_hihat_position(&kit[index], &kit);
    let axis = body(&kit[index]).unwrap().center;
    let pedal = kit
        .iter()
        .find(|piece| piece.instrument == Instrument::PedalHat)
        .unwrap();
    assert!((axis[0] - pedal.position[0]).abs() < 1e-12);
    assert!((axis[2] - pedal.position[2] - HIHAT_PEDAL_TO_AXIS).abs() < 1e-12);
    assert_eq!(pedal_contact(pedal, 0.0), pedal.position);
    let raised = pedal_contact(pedal, -0.18);
    assert!(raised[1] > pedal.position[1]);
    let hinge = [
        pedal.position[0],
        pedal.position[1] - 0.012,
        pedal.position[2] - 0.13,
    ];
    assert!((distance(raised, hinge) - distance(pedal.position, hinge)).abs() < 1e-12);
    let original = kit[index].position;
    kit[index].position[0] += 0.05;
    assert!(kit_geometry(&kit)
        .unresolved_piece_ids
        .contains(&kit[index].id));
    assert_eq!(anchored_hihat_position(&kit[index], &kit), original);
}

#[test]
fn beater_impact_surface_is_distinct_from_sole_and_matches_head_tangent() {
    let kit = default_kit();
    let kick = kit
        .iter()
        .find(|piece| piece.instrument == Instrument::Kick)
        .unwrap();
    let (contact, normal) = beater_contact(kick);
    let bounds = body(kick).unwrap();
    assert!((contact[2] - (bounds.center[2] - bounds.half_extents[2])).abs() < 1e-12);
    assert_eq!(normal, [0.0, 0.0, -1.0]);
    assert!(distance(contact, kick.position) > 0.1);
    let center = beater_center(kick);
    assert!((distance(center, contact) - 0.028).abs() < 1e-12);
    let geometry = kit_geometry(&kit);
    let frame = geometry
        .mechanisms
        .iter()
        .find(|frame| frame.kind == "kick")
        .unwrap();
    assert_eq!(frame.contact, contact);
    assert!(geometry.supports.iter().any(|rod| rod.piece_id == kick.id
        && rod.start == center
        && rod.end == center
        && rod.radius == 0.028));
}

#[test]
fn unrelated_hardware_crossings_are_not_joint_exemptions() {
    let kit = default_kit();
    let mut a = capsule(&kit[0], [-1., 0.2, 0.], [1., 0.2, 0.], 0.01);
    a.piece_id = "crash".into();
    let mut b = capsule(&kit[0], [0., 0., 0.], [0., 1., 0.], 0.01);
    b.piece_id = "ride".into();
    assert!(!hardware::supports_clear(&a, &b, &kit));
    b.start = a.end;
    b.end = [1., 1., 0.];
    assert!(
        !hardware::supports_clear(&a, &b, &kit),
        "unrelated shared endpoints are still crossings"
    );
    let g = kit_geometry(&kit);
    for (i, a) in g.supports.iter().enumerate() {
        for b in &g.supports[i + 1..] {
            assert!(
                hardware::supports_clear(a, b, &kit),
                "{}:{} crosses {}:{}",
                a.piece_id,
                a.role,
                b.piece_id,
                b.role
            );
        }
    }
}

#[test]
fn unreachable_rack_rod_is_reported_instead_of_accepted_as_long_diagonal() {
    let mut kit = default_kit();
    kit.iter_mut().find(|p| p.id == "tomHigh").unwrap().position[0] = -1.5;
    assert!(kit_geometry(&kit)
        .unresolved_piece_ids
        .iter()
        .any(|id| id == "tomHigh"));
}

#[test]
fn cymbal_spindle_uses_radial_axis_not_bow_contact_normal() {
    let mut piece = default_kit().into_iter().find(|p| p.id == "crash").unwrap();
    let mut geometry = local_geometry(&piece);
    let tilt = 0.3_f64;
    geometry.surface.normal = [0., (0.6_f64).cos(), -(0.6_f64).sin()];
    geometry.surface.radial_profile = Some(RadialSurfaceProfile {
        bell_radius: None,
        origin: [0.04, 0.02, 0.03],
        rotation: [(tilt / 2.).sin(), 0., 0., (tilt / 2.).cos()],
        points: vec![[0.008, 0.], [piece.radius, -0.04]],
    });
    piece.geometry = Some(geometry);
    let g = kit_geometry(&[piece.clone()]);
    let sleeve = g
        .supports
        .iter()
        .find(|rod| rod.role == "cymbalSleeve")
        .unwrap();
    let expected = [0., tilt.cos(), tilt.sin()];
    let d = sub(sleeve.end, sleeve.start);
    let n = distance(d, [0.; 3]);
    for axis in 0..3 {
        assert!((d[axis] / n - expected[axis]).abs() < 1e-10);
    }
    let mount = add(piece.position, [0.04, 0.02, 0.03]);
    assert!(crate::clearance::segment_distance(mount, mount, sleeve.start, sleeve.end) < 1e-10);
}

#[test]
fn boom_load_projection_stays_within_oriented_tripod() {
    let kit = default_kit();
    let g = kit_geometry(&kit);
    for piece in kit
        .iter()
        .filter(|p| matches!(p.instrument, Instrument::Crash | Instrument::Ride))
    {
        let rods: Vec<_> = g
            .supports
            .iter()
            .filter(|rod| rod.piece_id == piece.id)
            .collect();
        let feet: Vec<_> = rods
            .iter()
            .filter(|rod| rod.role == "rubberFoot")
            .map(|rod| rod.start)
            .collect();
        let sleeve = rods.iter().find(|rod| rod.role == "cymbalSleeve").unwrap();
        let load = sleeve.end;
        let crosses: Vec<_> = (0..3)
            .map(|i| {
                let a = feet[i];
                let b = feet[(i + 1) % 3];
                (b[0] - a[0]) * (load[2] - a[2]) - (b[2] - a[2]) * (load[0] - a[0])
            })
            .collect();
        assert!(
            crosses.iter().all(|v| *v > 0.) || crosses.iter().all(|v| *v < 0.),
            "{} tripod",
            piece.id
        );
        assert!(rods.iter().any(|rod| rod.role == "boomTail"));
        for role in ["tubeLower", "tubeMiddle", "tubeUpper"] {
            assert!(rods.iter().any(|rod| rod.role == role));
        }
    }
}

#[test]
fn basket_neck_connects_the_routed_mast_to_its_receiver() {
    let kit = default_kit();
    let g = kit_geometry(&kit);
    let neck = g
        .supports
        .iter()
        .find(|s| s.piece_id == "snare" && s.role == "basketNeck")
        .unwrap();
    let mast = g
        .supports
        .iter()
        .find(|s| s.piece_id == "snare" && s.role == "tubeUpper")
        .unwrap();
    assert_eq!(mast.end, neck.start);
    assert!(distance(neck.start, neck.end) <= 0.10);
    for arm in g
        .supports
        .iter()
        .filter(|s| s.piece_id == "snare" && s.role == "basketArm")
    {
        assert_eq!(arm.start, neck.end);
    }
}

#[test]
fn rod_box_broad_phase_preserves_exact_clearance_predicate() {
    let obstacle = EquipmentBox {
        piece_id: "shell".into(),
        center: [0.1, 0.6, -0.2],
        half_extents: [0.2, 0.14, 0.3],
        kind: "body",
    };
    let piece = &default_kit()[0];
    for i in -6..=6 {
        for j in -6..=6 {
            let rod = capsule(
                piece,
                [i as f64 * 0.08, 0.3, j as f64 * 0.07],
                [j as f64 * 0.09, 0.8, i as f64 * 0.06],
                0.017,
            );
            assert_eq!(
                rod_box_clear(&rod, &obstacle),
                segment_box_clearance(rod.start, rod.end, rod.radius, &obstacle) >= CLEARANCE
            );
        }
    }
}

#[test]
fn tilted_snare_basket_receivers_follow_the_lower_rim_plane() {
    let mut piece = default_kit()
        .into_iter()
        .find(|p| p.instrument == Instrument::Snare)
        .unwrap();
    let normal = [0., 0.3_f64.cos(), 0.3_f64.sin()];
    let mut geometry = local_geometry(&piece);
    geometry.body.center = normal.map(|v| -0.065 * v);
    geometry.body.half_extents = [
        piece.radius + 0.014,
        0.065 * normal[1] + piece.radius * normal[2],
        piece.radius * normal[1] + 0.065 * normal[2],
    ];
    geometry.surface.normal = normal;
    piece.geometry = Some(geometry);
    let g = kit_geometry(&[piece.clone()]);
    let center = sub(
        body(&piece).unwrap().center.map(|v| 2. * v),
        surface(&piece).center,
    );
    for arm in g.supports.iter().filter(|s| s.role == "basketArm") {
        let offset = sub(arm.end, center);
        let plane_distance: f64 = offset.iter().zip(normal).map(|(a, b)| a * b).sum();
        assert!(plane_distance.abs() < 1e-10);
        assert!((distance(arm.end, center) - piece.radius).abs() < 1e-10);
    }
    assert_eq!(
        g.supports.iter().filter(|s| s.role == "basketGrip").count(),
        3
    );
}

#[test]
fn authored_fallback_bell_semantics_survive_compact_fitting_without_guessing_imports() {
    let mut ride = default_kit()
        .into_iter()
        .find(|p| p.instrument == Instrument::Ride)
        .unwrap();
    ride.geometry = Some(local_geometry(&ride));
    let descriptor = ride.geometry.clone().unwrap();
    assert_eq!(
        descriptor
            .surface
            .radial_profile
            .as_ref()
            .unwrap()
            .bell_radius,
        Some(ride.radius * 0.24)
    );
    for scale in [1.0, 0.72, 0.60] {
        ride.radius = descriptor.reference_radius * scale;
        assert!(valid_piece_geometry(&ride));
        let surface = surface(&ride);
        let observer = add(surface.center, [0.0, 0.5, -0.6]);
        let target = crate::contact::target(&ride, "bell", observer);
        let radius = distance(target, surface.center);
        let tool_radius = crate::stick_geometry::profile().terminal.last().unwrap()[1];
        assert!(radius + 1e-12 >= CYMBAL_FELT_RADIUS + tool_radius + crate::posture::MARGIN);
        assert!(radius <= surface.radial_profile.unwrap().bell_radius.unwrap() + 1e-12);
    }
    // An explicit planar asset descriptor carries no authored bell semantics.
    // Importing it must not silently replace the caller's surface contract.
    ride.geometry.as_mut().unwrap().source = "mesh-bounds".into();
    ride.geometry.as_mut().unwrap().surface.radial_profile = None;
    assert!(local_geometry(&ride).surface.radial_profile.is_none());
}
