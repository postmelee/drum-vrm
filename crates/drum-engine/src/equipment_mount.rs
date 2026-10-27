//! Optional physical mount dependencies, separate from appearance presets.
//! A suspended low tom uses a real host mast/clamp/L-rod route. Failed bounds
//! never acquire hidden floor legs or an unbounded connector.
use super::*;
use crate::model::PieceMount;

const MAX_CLAMP_ARM: f64 = 0.38;
const MAX_CLAMP_RISE: f64 = 0.30;
const SOCKET_OFFSET: f64 = 0.05;

fn host_id(piece: &KitPiece) -> Option<&str> {
    piece.mount.as_ref().map(|mount| match mount {
        PieceMount::StandClamp { support_piece_id } => support_piece_id.as_str(),
    })
}

pub(super) fn host<'a>(piece: &KitPiece, kit: &'a [KitPiece]) -> Option<&'a KitPiece> {
    if piece.instrument != Instrument::TomLow {
        return None;
    }
    let id = host_id(piece)?;
    kit.iter().find(|host| {
        host.id == id
            && host.id != piece.id
            && host.mount.is_none()
            && matches!(host.instrument, Instrument::Ride | Instrument::Crash)
    })
}

pub(super) fn routing_order(kit: &[KitPiece]) -> Vec<&KitPiece> {
    let mut result: Vec<&KitPiece> = Vec::with_capacity(kit.len());
    for piece in kit {
        if let Some(host) = host(piece, kit) {
            if !result.iter().any(|p| p.id == host.id) {
                result.push(host);
            }
        }
        if !result.iter().any(|p| p.id == piece.id) {
            result.push(piece);
        }
    }
    result
}

pub(super) fn loads(host: &KitPiece, kit: &[KitPiece]) -> Vec<Vec3> {
    kit.iter()
        .filter(|piece| host_id(piece) == Some(host.id.as_str()))
        .filter_map(body)
        .map(|bounds| bounds.center)
        .collect()
}

fn on_mast(point: Vec3, mast: &EquipmentCapsule) -> bool {
    matches!(mast.role, "tubeLower" | "tubeMiddle" | "tubeUpper")
        && (mast.end[0] - mast.start[0]).hypot(mast.end[2] - mast.start[2]) < 1e-8
        && (point[0] - mast.start[0]).hypot(point[2] - mast.start[2]) < 1e-8
        && point[1] >= mast.start[1].min(mast.end[1]) - 1e-8
        && point[1] <= mast.start[1].max(mast.end[1]) + 1e-8
}

/// Only the clamp sleeve and its short socket may touch their named host tube.
/// The L-rod, ball and shell still check every unrelated support normally.
pub(super) fn joint(a: &EquipmentCapsule, b: &EquipmentCapsule, kit: &[KitPiece]) -> bool {
    let linked = |part: &EquipmentCapsule, mast: &EquipmentCapsule| {
        let Some(piece) = kit.iter().find(|p| p.id == part.piece_id) else {
            return false;
        };
        if host(piece, kit).is_none_or(|host| host.id != mast.piece_id) {
            return false;
        }
        match part.role {
            "standClamp" => on_mast(part.start, mast) && on_mast(part.end, mast),
            "standLink" => {
                on_mast(part.start, mast) && distance(part.start, part.end) <= SOCKET_OFFSET + 1e-8
            }
            _ => false,
        }
    };
    linked(a, b) || linked(b, a)
}

pub(super) fn route(
    piece: &KitPiece,
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
    existing: &[EquipmentCapsule],
) -> (Vec<EquipmentCapsule>, bool) {
    let Some(host) = host(piece, kit) else {
        return (Vec::new(), false);
    };
    let Some(bounds) = body(piece) else {
        return (Vec::new(), false);
    };
    let head = surface(piece);
    let mut original = Vec::new();
    for mast in existing.iter().filter(|part| {
        part.piece_id == host.id && matches!(part.role, "tubeLower" | "tubeMiddle" | "tubeUpper")
    }) {
        // Published stand-clamp pipe range; route lengths remain project bounds.
        if !(0.0159..=0.0286 + 1e-9).contains(&(mast.radius * 2.)) {
            continue;
        }
        let toward = unit_xz(sub(mast.start, bounds.center));
        let angle = toward[0].atan2(toward[2]);
        for offset in [0.0_f64, 0.35, -0.35, 0.7, -0.7] {
            let radial = [(angle + offset).sin(), 0., (angle + offset).cos()];
            let extent = (bounds.half_extents[0] / radial[0].abs().max(1e-6))
                .min(bounds.half_extents[2] / radial[2].abs().max(1e-6));
            let bracket = axis_add(bounds.center, radial, extent + 0.025);
            for rise in [0.08, 0.12, 0.18, 0.24, MAX_CLAMP_RISE] {
                let clamp = [mast.start[0], bracket[1] - rise, mast.start[2]];
                if !on_mast(add(clamp, [0., -0.025, 0.]), mast)
                    || !on_mast(add(clamp, [0., 0.025, 0.]), mast)
                {
                    continue;
                }
                let out = unit_xz(sub(bracket, clamp));
                let socket = axis_add(clamp, out, SOCKET_OFFSET);
                let bend = [bracket[0], socket[1], bracket[2]];
                let dot: f64 = (0..3).map(|axis| radial[axis] * head.normal[axis]).sum();
                let tangent = sub(radial, head.normal.map(|v| v * dot));
                let length = distance(tangent, [0.; 3]);
                let shell = axis_add(
                    bounds.center,
                    tangent.map(|v| v / length.max(1e-9)),
                    head.radius,
                );
                let rods = vec![
                    part(
                        piece,
                        "standClamp",
                        add(clamp, [0., -0.018, 0.]),
                        add(clamp, [0., 0.018, 0.]),
                        0.027,
                    ),
                    part(piece, "standLink", clamp, socket, 0.012),
                    ball(piece, "mountBall", socket, 0.026),
                    part(piece, "tomArm", socket, bend, 0.00525),
                    part(
                        piece,
                        "tomArm",
                        bend,
                        add(bracket, [0., 0.035, 0.]),
                        0.00525,
                    ),
                    part(
                        piece,
                        "tomClamp",
                        add(bracket, [0., -0.022, 0.]),
                        add(bracket, [0., 0.022, 0.]),
                        0.023,
                    ),
                    part(piece, "suspensionBracket", shell, bracket, 0.016),
                ];
                if original.is_empty() {
                    original = rods.clone();
                }
                if distance(socket, bend) < 0.025 || distance(socket, bend) > MAX_CLAMP_ARM {
                    continue;
                }
                if !rods[..5].iter().all(|rod| {
                    segment_box_clearance(rod.start, rod.end, rod.radius, &bounds) >= CLEARANCE
                }) {
                    continue;
                }
                if clear(&rods, bodies, boards, kit, existing) {
                    return (rods, true);
                }
            }
        }
    }
    (original, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::default_kit;

    fn suspended() -> Vec<KitPiece> {
        let mut kit = default_kit();
        kit.retain(|p| matches!(p.instrument, Instrument::Ride | Instrument::TomLow));
        let floor = kit
            .iter_mut()
            .find(|p| p.instrument == Instrument::TomLow)
            .unwrap();
        floor.position = [0.50, 0.76, 0.55];
        floor.radius = 0.1651;
        floor.mount = Some(PieceMount::StandClamp {
            support_piece_id: "ride".into(),
        });
        let ride = kit
            .iter_mut()
            .find(|p| p.instrument == Instrument::Ride)
            .unwrap();
        ride.position = [0.72, 1.22, 0.68];
        kit
    }

    #[test]
    fn host_is_routed_before_dependant_and_unknown_mount_stays_unresolved() {
        let mut kit = suspended();
        assert_eq!(
            routing_order(&kit)
                .iter()
                .map(|p| p.id.as_str())
                .collect::<Vec<_>>(),
            vec!["ride", "tomLow"]
        );
        kit.iter_mut().find(|p| p.id == "tomLow").unwrap().mount = Some(PieceMount::StandClamp {
            support_piece_id: "absent".into(),
        });
        let geometry = kit_geometry(&kit);
        assert!(geometry
            .unresolved_piece_ids
            .contains(&"tomLow".to_string()));
        assert!(!geometry
            .supports
            .iter()
            .any(|part| part.piece_id == "tomLow" && part.role == "floorLeg"));
    }

    #[test]
    fn only_declared_clamp_joint_can_touch_host_mast() {
        let kit = suspended();
        let floor = kit.iter().find(|p| p.id == "tomLow").unwrap();
        let ride = kit.iter().find(|p| p.id == "ride").unwrap();
        let mast = part(ride, "tubeMiddle", [0.7, 0.3, 0.7], [0.7, 0.8, 0.7], 0.0111);
        let clamp = part(
            floor,
            "standClamp",
            [0.7, 0.4, 0.7],
            [0.7, 0.436, 0.7],
            0.027,
        );
        assert!(joint(&clamp, &mast, &kit));
        let mut rogue = clamp.clone();
        rogue.role = "tomArm";
        assert!(!joint(&rogue, &mast, &kit));
        assert!(!supports_clear(&rogue, &mast, &kit));
        let mut unnamed = kit.clone();
        unnamed.iter_mut().find(|p| p.id == "tomLow").unwrap().mount = None;
        assert!(!joint(&clamp, &mast, &unnamed));
    }

    #[test]
    fn suspended_tom_has_one_bounded_connected_mount_and_no_floor_legs() {
        let kit = suspended();
        let geometry = kit_geometry(&kit);
        assert!(
            geometry.unresolved_piece_ids.is_empty(),
            "{:?}",
            geometry.unresolved_piece_ids
        );
        let mut reordered = kit.clone();
        reordered.reverse();
        let reordered_geometry = kit_geometry(&reordered);
        assert!(reordered_geometry.unresolved_piece_ids.is_empty());
        assert_eq!(
            serde_json::to_value(&geometry.supports).unwrap(),
            serde_json::to_value(&reordered_geometry.supports).unwrap()
        );
        let parts: Vec<_> = geometry
            .supports
            .iter()
            .filter(|p| p.piece_id == "tomLow")
            .collect();
        assert_eq!(parts.iter().filter(|p| p.role == "standClamp").count(), 1);
        assert_eq!(parts.iter().filter(|p| p.role == "tomArm").count(), 2);
        assert!(!parts
            .iter()
            .any(|p| matches!(p.role, "floorLeg" | "rubberFoot")));
        let arms: Vec<_> = parts.iter().filter(|p| p.role == "tomArm").collect();
        assert!(distance(arms[0].end, arms[1].start) < 1e-9);
        assert!(distance(arms[0].start, arms[0].end) <= MAX_CLAMP_ARM);
        assert!((arms[1].end[1] - arms[1].start[1]) <= MAX_CLAMP_RISE + 0.035 + 1e-9);
        let original: Vec<_> = default_kit()
            .into_iter()
            .filter(|p| p.instrument == Instrument::TomLow)
            .collect();
        assert_eq!(
            kit_geometry(&original)
                .supports
                .iter()
                .filter(|p| p.role == "floorLeg")
                .count(),
            6
        );
    }

    #[test]
    fn distant_host_is_unresolved_without_an_unbounded_replacement() {
        let mut kit = suspended();
        kit.iter_mut()
            .find(|piece| piece.id == "ride")
            .unwrap()
            .position[0] += 2.;
        let geometry = kit_geometry(&kit);
        assert!(geometry
            .unresolved_piece_ids
            .contains(&"tomLow".to_string()));
        assert!(!geometry
            .supports
            .iter()
            .any(|part| part.piece_id == "tomLow" && part.role == "floorLeg"));
    }
}
