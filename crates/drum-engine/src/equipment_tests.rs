use super::*;
use crate::model::default_kit;

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
