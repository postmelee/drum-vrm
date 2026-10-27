//! Mechanical hardware topology. Numeric capsules own both clearance and the
//! renderer's detail envelope; individual screws do not multiply motion queries.
use super::*;
#[path = "equipment_mount.rs"]
mod mounts;

pub(super) fn routing_order(kit: &[KitPiece]) -> Vec<&KitPiece> {
    mounts::routing_order(kit)
}

pub const MAX_TOM_ARM: f64 = 0.38;
const MAX_TOM_RISE: f64 = 0.30;
const MAX_BOOM: f64 = 0.45;
const TRIPOD_RADIUS: f64 = 0.30;
// Snare baskets sit between the seated legs. A compact base is a distinct
// mechanical family, not a uniform copy of the wider boom-cymbal stand.
const SNARE_TRIPOD_RADIUS: f64 = 0.225;

fn part(piece: &KitPiece, role: &'static str, a: Vec3, b: Vec3, radius: f64) -> EquipmentCapsule {
    let mut rod = capsule(piece, a, b, radius);
    rod.role = role;
    rod
}

fn ball(piece: &KitPiece, role: &'static str, at: Vec3, radius: f64) -> EquipmentCapsule {
    part(piece, role, at, at, radius)
}

fn axis_add(point: Vec3, axis: Vec3, amount: f64) -> Vec3 {
    add(point, axis.map(|v| v * amount))
}

fn unit_xz(v: Vec3) -> Vec3 {
    let length = v[0].hypot(v[2]);
    if length < 1e-6 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / length, 0.0, v[2] / length]
    }
}

fn rack_frame(kit: &[KitPiece]) -> Option<(&KitPiece, Vec3, Vec3)> {
    let bass_piece = kit.iter().find(|p| p.instrument == Instrument::Kick)?;
    let bass = body(bass_piece)?;
    let start = [
        bass.center[0],
        bass.center[1] + bass.half_extents[1] + 0.018,
        bass.center[2] + bass.half_extents[2] * 0.35,
    ];
    let lowest = kit
        .iter()
        .filter(|p| matches!(p.instrument, Instrument::TomHigh | Instrument::TomMid))
        .filter_map(body)
        .map(|b| b.center[1] - b.half_extents[1])
        .reduce(f64::min)?;
    // Keep the shared yoke below the shells, but never collapse a telescopic
    // holder to an invisible zero-length post. Unfit layouts remain diagnosed.
    let top = [
        start[0],
        (lowest - 0.045).clamp(start[1] + 0.07, start[1] + 0.24),
        start[2],
    ];
    Some((bass_piece, start, top))
}

pub(super) fn rack_holder(kit: &[KitPiece]) -> Vec<EquipmentCapsule> {
    let Some((bass, start, top)) = rack_frame(kit) else {
        return Vec::new();
    };
    let middle = [start[0], start[1] + (top[1] - start[1]) * 0.5, start[2]];
    vec![
        part(
            bass,
            "tomSocket",
            add(start, [0., -0.055, 0.]),
            add(start, [0., 0.012, 0.]),
            0.022,
        ),
        part(bass, "tomPost", start, middle, 0.0127),
        part(bass, "tubeUpper", middle, top, 0.0105),
        part(
            bass,
            "collar",
            axis_add(middle, [0., 1., 0.], -0.008),
            axis_add(middle, [0., 1., 0.], 0.008),
            0.025,
        ),
        part(
            bass,
            "tomYoke",
            add(top, [-0.045, 0., 0.]),
            add(top, [0.045, 0., 0.]),
            0.018,
        ),
        ball(bass, "tomBall", add(top, [-0.045, 0., 0.]), 0.026),
        ball(bass, "tomBall", add(top, [0.045, 0., 0.]), 0.026),
    ]
}

/// Only physically connected rack components may meet across piece ownership.
/// A random shared endpoint between unrelated stands is not an exemption.
fn rack_joint(a: &EquipmentCapsule, b: &EquipmentCapsule, kit: &[KitPiece]) -> bool {
    let linked = |a: &EquipmentCapsule, b: &EquipmentCapsule| {
        matches!(a.role, "tomYoke" | "tomBall")
            && b.role == "tomArm"
            && kit
                .iter()
                .any(|p| p.id == a.piece_id && p.instrument == Instrument::Kick)
            && kit.iter().any(|p| {
                p.id == b.piece_id
                    && matches!(p.instrument, Instrument::TomHigh | Instrument::TomMid)
            })
            && [a.start, a.end]
                .iter()
                .any(|&p| distance(p, b.start) < 1e-8)
    };
    linked(a, b) || linked(b, a)
}

pub(super) fn supports_clear(a: &EquipmentCapsule, b: &EquipmentCapsule, kit: &[KitPiece]) -> bool {
    if same_owner(&a.piece_id, &b.piece_id, kit)
        || rack_joint(a, b, kit)
        || mounts::joint(a, b, kit)
    {
        return true;
    }
    let clearance = a.radius + b.radius + CLEARANCE;
    // Conservative broad phase; expensive segment distance only for overlapping
    // expanded boxes. No diagnostic or candidate order changes with this gate.
    if (0..3).any(|axis| {
        a.start[axis].min(a.end[axis]) - b.start[axis].max(b.end[axis]) >= clearance
            || b.start[axis].min(b.end[axis]) - a.start[axis].max(a.end[axis]) >= clearance
    }) {
        return true;
    }
    crate::clearance::segment_distance(a.start, a.end, b.start, b.end) >= clearance
}

fn clear(
    rods: &[EquipmentCapsule],
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
    existing: &[EquipmentCapsule],
) -> bool {
    rods_clear(rods, bodies, boards, kit)
        && rods
            .iter()
            .all(|a| existing.iter().all(|b| supports_clear(a, b, kit)))
}

fn stand(piece: &KitPiece, base: Vec3, top: Vec3, rotation: f64) -> Vec<EquipmentCapsule> {
    let radius = if piece.instrument == Instrument::Snare {
        SNARE_TRIPOD_RADIUS
    } else {
        TRIPOD_RADIUS
    };
    let lower_end = [base[0], top[1] * 0.43, base[2]];
    let middle_end = [base[0], top[1] * 0.73, base[2]];
    let mut rods = vec![
        part(piece, "tubeLower", base, lower_end, 0.0143),
        part(piece, "tubeMiddle", lower_end, middle_end, 0.0111),
        part(piece, "tubeUpper", middle_end, top, 0.00955),
        part(
            piece,
            "collar",
            add(middle_end, [0., -0.010, 0.]),
            add(middle_end, [0., 0.010, 0.]),
            0.021,
        ),
        part(
            piece,
            "collar",
            add(lower_end, [0., -0.012, 0.]),
            add(lower_end, [0., 0.012, 0.]),
            0.025,
        ),
        part(
            piece,
            "memoryLock",
            add(lower_end, [0., 0.021, 0.]),
            add(lower_end, [0., 0.029, 0.]),
            0.017,
        ),
    ];
    let leg_hub = [base[0], (top[1] * 0.55).min(0.20), base[2]];
    for leg in 0..3 {
        let angle = leg as f64 * std::f64::consts::TAU / 3.0 + rotation;
        let foot = [
            base[0] + angle.sin() * radius,
            0.018,
            base[2] + angle.cos() * radius,
        ];
        let knee = std::array::from_fn(|axis| leg_hub[axis] * 0.35 + foot[axis] * 0.65);
        rods.push(part(piece, "tripodLeg", leg_hub, foot, 0.013));
        rods.push(part(
            piece,
            "tripodBrace",
            [base[0], 0.075, base[2]],
            knee,
            0.008,
        ));
        rods.push(part(
            piece,
            "rubberFoot",
            foot,
            add(foot, [angle.sin() * 0.018, 0., angle.cos() * 0.018]),
            0.018,
        ));
    }
    rods
}

/// Keep the conservative vertical cymbal-load projection within the tripod.
/// This is a geometric placement rule, not a certification of physical COM.
fn load_projection_supported(base: Vec3, load: Vec3, rotation: f64) -> bool {
    let feet: [Vec3; 3] = std::array::from_fn(|i| {
        let angle = i as f64 * std::f64::consts::TAU / 3. + rotation;
        [
            base[0] + angle.sin() * TRIPOD_RADIUS,
            0.,
            base[2] + angle.cos() * TRIPOD_RADIUS,
        ]
    });
    let mut sign = 0.;
    for i in 0..3 {
        let a = feet[i];
        let b = feet[(i + 1) % 3];
        let cross = (b[0] - a[0]) * (load[2] - a[2]) - (b[2] - a[2]) * (load[0] - a[0]);
        if i == 0 {
            sign = cross.signum();
        }
        if cross * sign < 1e-6 {
            return false;
        }
    }
    true
}

fn rack_arm(
    piece: &KitPiece,
    kit: &[KitPiece],
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    existing: &[EquipmentCapsule],
) -> Option<(Vec<EquipmentCapsule>, bool)> {
    let (_, _, top) = rack_frame(kit)?;
    let bounds = body(piece)?;
    let side = if piece.instrument == Instrument::TomHigh {
        -1.0
    } else {
        1.0
    };
    let joint = add(top, [side * 0.045, 0., 0.]);
    let toward = unit_xz(sub(joint, bounds.center));
    let start_angle = toward[0].atan2(toward[2]);
    let mut original = Vec::new();
    for offset in [
        0.0_f64,
        0.35,
        -0.35,
        0.7,
        -0.7,
        1.2,
        -1.2,
        std::f64::consts::PI,
    ] {
        let angle = start_angle + offset;
        let radial = [angle.sin(), 0., angle.cos()];
        // Exit the conservative shell box along this radial direction. The
        // short clamp enters only its own shell; the L-rod remains outside it.
        let radius = (bounds.half_extents[0] / radial[0].abs().max(1e-6))
            .min(bounds.half_extents[2] / radial[2].abs().max(1e-6));
        let bracket = axis_add(bounds.center, radial, radius + 0.025);
        let bend = [bracket[0], joint[1], bracket[2]];
        let head = surface(piece);
        let dot: f64 = (0..3).map(|axis| radial[axis] * head.normal[axis]).sum();
        let side_axis = sub(radial, head.normal.map(|v| v * dot));
        let side_length = distance(side_axis, [0.; 3]);
        let shell_attachment = axis_add(
            bounds.center,
            side_axis.map(|v| v / side_length.max(1e-9)),
            head.radius,
        );
        let rods = vec![
            part(piece, "tomArm", joint, bend, 0.00525),
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
            part(piece, "suspensionBracket", shell_attachment, bracket, 0.016),
        ];
        if original.is_empty() {
            original = rods.clone();
        }
        let horizontal = distance(joint, bend);
        let rise = bracket[1] - joint[1];
        if horizontal <= MAX_TOM_ARM
            && (0.025..=MAX_TOM_RISE).contains(&rise)
            && rods[..2].iter().all(|rod| {
                segment_box_clearance(rod.start, rod.end, rod.radius, &bounds) >= CLEARANCE
            })
            && clear(&rods, bodies, boards, kit, existing)
        {
            return Some((rods, true));
        }
    }
    Some((original, false))
}

pub(super) fn route(
    piece: &KitPiece,
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
    existing: &[EquipmentCapsule],
) -> (Vec<EquipmentCapsule>, bool) {
    if piece.mount.is_some() {
        return mounts::route(piece, bodies, boards, kit, existing);
    }
    let physical = body(piece).expect("routed equipment has body");
    let cymbal = matches!(
        piece.instrument,
        Instrument::Hihat | Instrument::Crash | Instrument::Ride
    );
    if matches!(piece.instrument, Instrument::TomHigh | Instrument::TomMid) {
        if let Some(routed) = rack_arm(piece, kit, bodies, boards, existing) {
            return routed;
        }
    }
    if piece.instrument == Instrument::TomLow {
        let mut original = Vec::new();
        for rotation in [0.4, 0.8, 1.2, 1.6] {
            let mut rods = Vec::new();
            for leg in 0..3 {
                let angle = leg as f64 * std::f64::consts::TAU / 3.0 + rotation;
                let radial = [angle.sin(), 0., angle.cos()];
                let radius = physical.half_extents[0].max(physical.half_extents[2]) + 0.02;
                let start = axis_add(physical.center, radial, radius);
                let bend = [start[0], (start[1] * 0.42).max(0.055), start[2]];
                let foot = [
                    start[0] + radial[0] * 0.07,
                    0.018,
                    start[2] + radial[2] * 0.07,
                ];
                rods.push(part(
                    piece,
                    "tomClamp",
                    add(start, [0., -0.015, 0.]),
                    add(start, [0., 0.015, 0.]),
                    0.022,
                ));
                rods.push(part(
                    piece,
                    "suspensionBracket",
                    axis_add(physical.center, radial, surface(piece).radius),
                    start,
                    0.013,
                ));
                rods.push(part(piece, "floorLeg", start, bend, 0.0105));
                rods.push(part(piece, "floorLeg", bend, foot, 0.0105));
                rods.push(ball(piece, "rubberFoot", foot, 0.018));
            }
            if original.is_empty() {
                original = rods.clone();
            }
            if clear(&rods, bodies, boards, kit, existing) {
                return (rods, true);
            }
        }
        return (original, false);
    }
    let head = surface(piece);
    // Drum bodies are rotational shells around the measured head normal. Their
    // opposite rim is the reflection of the head center through the body center.
    let lower_rim_center = sub(physical.center.map(|v| 2. * v), head.center);
    // A bow contact normal is not the cymbal symmetry axis. Use the measured
    // radial profile's local +Y transformed into world space for the spindle.
    let (normal, mount) = if let Some(profile) = &head.radial_profile {
        let [x, y, z, w] = profile.rotation;
        let axis = [
            2. * (x * y - w * z),
            1. - 2. * (x * x + z * z),
            2. * (y * z + w * x),
        ];
        (axis, profile.origin)
    } else if cymbal {
        (
            [0., 1., 0.],
            [
                physical.center[0],
                physical.center[1] + physical.half_extents[1] - 0.004,
                physical.center[2],
            ],
        )
    } else {
        (head.normal, axis_add(lower_rim_center, head.normal, -0.035))
    };
    let attached_loads = mounts::loads(piece, kit);
    let build = |dx: f64, dz: f64, rotation: f64, boom_drop: f64| {
        let base = [mount[0] + dx, 0.025, mount[2] + dz];
        let boom = cymbal && piece.instrument != Instrument::Hihat;
        let tilter = if boom {
            axis_add(mount, normal, -0.075)
        } else {
            mount
        };
        let top = [
            base[0],
            (tilter[1]
                - if boom {
                    boom_drop
                } else if cymbal {
                    0.0
                } else {
                    0.04
                })
            .max(0.12),
            base[2],
        ];
        let phase = piece.stand_rotation.unwrap_or_else(|| {
            if piece.instrument == Instrument::Hihat {
                // A leg points away from the heel; the two rear legs straddle the
                // shoe/footboard corridor rather than cutting across its centre.
                let direction = kit
                    .iter()
                    .find(|p| p.instrument == Instrument::PedalHat)
                    .map(pedal_forward)
                    .unwrap_or([0., 0., 1.]);
                direction[0].atan2(direction[2]) + rotation
            } else if boom && dx.hypot(dz) > 1e-9 {
                (-dx).atan2(-dz) + rotation
            } else {
                rotation
            }
        });
        let mut rods = stand(piece, base, top, phase);
        if boom {
            let direction = sub(tilter, top);
            let length = distance(top, tilter);
            let unit = direction.map(|v| v / length.max(1e-9));
            rods.push(ball(piece, "boomPivot", top, 0.027));
            rods.push(part(piece, "boom", top, tilter, 0.0105));
            rods.push(part(
                piece,
                "boomTail",
                axis_add(top, unit, -0.10),
                top,
                0.0105,
            ));
            rods.push(part(
                piece,
                "cymbalTilter",
                axis_add(tilter, [1., 0., 0.], -0.015),
                axis_add(tilter, [1., 0., 0.], 0.015),
                0.019,
            ));
            rods.push(part(
                piece,
                "cymbalSleeve",
                tilter,
                axis_add(mount, normal, 0.018),
                0.007,
            ));
            rods.push(part(
                piece,
                "cymbalFelt",
                axis_add(mount, normal, -0.018),
                axis_add(mount, normal, -0.006),
                CYMBAL_FELT_RADIUS,
            ));
        } else if cymbal {
            rods.push(part(
                piece,
                "cymbalSleeve",
                [mount[0], piece.position[1] - 0.02, mount[2]],
                add(mount, [0., 0.018, 0.]),
                0.006,
            ));
        } else {
            rods.push(ball(piece, "basketTilter", top, 0.020));
            rods.push(part(piece, "basketNeck", top, mount, 0.012));
            for i in 0..3 {
                let angle = i as f64 * std::f64::consts::TAU / 3.0;
                let radial = [angle.sin(), 0., angle.cos()];
                let dot: f64 = (0..3).map(|axis| radial[axis] * normal[axis]).sum();
                let tangent = sub(radial, normal.map(|v| v * dot));
                let length = distance(tangent, [0.; 3]);
                let receiver = axis_add(
                    lower_rim_center,
                    tangent.map(|v| v / length.max(1e-9)),
                    head.radius,
                );
                rods.push(part(piece, "basketArm", mount, receiver, 0.010));
                rods.push(part(
                    piece,
                    "basketGrip",
                    axis_add(receiver, normal, -0.007),
                    axis_add(receiver, normal, 0.012),
                    0.012,
                ));
            }
        }
        if piece.instrument == Instrument::Hihat {
            if let Some(pedal) = kit.iter().find(|p| p.instrument == Instrument::PedalHat) {
                rods.push(part(
                    piece,
                    "pedalLink",
                    pedal_frame_point(pedal, [0., 0.04 - pedal.position[1], HIHAT_PEDAL_TO_AXIS]),
                    pedal_frame_point(pedal, [0., 0.04 - pedal.position[1], 0.08]),
                    0.011,
                ));
            }
        }
        let valid = if boom {
            distance(top, tilter) <= MAX_BOOM
                && load_projection_supported(base, mount, phase)
                && attached_loads
                    .iter()
                    .all(|&load| load_projection_supported(base, load, phase))
        } else if !cymbal {
            distance(top, mount) <= 0.10
        } else {
            true
        };
        (rods, valid)
    };
    let outward = if mount[0] < 0.0 { -1.0 } else { 1.0 };
    // A rear-facing snare leg leaves the front sector open for both pedal
    // strokes. The phase remains part of the shared physical routing search.
    let initial_phase = if piece.instrument == Instrument::Snare {
        std::f64::consts::PI
    } else {
        0.4
    };
    let (original, valid) = if cymbal && piece.instrument != Instrument::Hihat {
        build(outward * 0.20, 0.12, 0., 0.10)
    } else if piece.instrument == Instrument::Hihat {
        build(0., 0., 0., 0.10)
    } else {
        build(0., 0., initial_phase, 0.10)
    };
    if valid && clear(&original, bodies, boards, kit, existing) {
        return (original, true);
    }
    for step in 0..=6 {
        if (piece.instrument == Instrument::Hihat && step > 0) || (!cymbal && step > 1) {
            break;
        }
        for direction in 0..if step == 0 { 1 } else { 24 } {
            let angle = direction as f64 * std::f64::consts::TAU / 24.;
            for &phase_delta in if piece.stand_rotation.is_some() {
                &[0.][..]
            } else {
                &[0., 0.2, -0.2, 0.4, -0.4][..]
            } {
                let rotation = if piece.instrument == Instrument::Snare {
                    initial_phase + phase_delta
                } else {
                    phase_delta
                };
                for &drop in if cymbal && piece.instrument != Instrument::Hihat {
                    &[0.10, 0.04, 0.06, 0.18, 0.26, 0.34][..]
                } else {
                    &[0.10][..]
                } {
                    let (rods, valid) = build(
                        outward * angle.cos() * step as f64 * 0.05,
                        angle.sin() * step as f64 * 0.05,
                        rotation,
                        drop,
                    );
                    if valid && clear(&rods, bodies, boards, kit, existing) {
                        return (rods, true);
                    }
                }
            }
        }
    }
    (original, false)
}

#[cfg(test)]
mod hardware_family_tests {
    use super::*;

    #[test]
    fn explicit_tripod_phase_is_absolute_for_snare_boom_and_hihat() {
        for instrument in [Instrument::Snare, Instrument::Ride, Instrument::Hihat] {
            let mut kit = crate::model::default_kit();
            kit.retain(|piece| {
                piece.instrument == instrument || piece.instrument == Instrument::PedalHat
            });
            for angle in [-1.0_f64, 0.7] {
                kit.iter_mut()
                    .find(|piece| piece.instrument == instrument)
                    .unwrap()
                    .stand_rotation = Some(angle);
                let geometry = kit_geometry(&kit);
                let id = instrument.default_piece_id();
                let base = geometry
                    .supports
                    .iter()
                    .find(|part| part.piece_id == id && part.role == "tubeLower")
                    .unwrap()
                    .start;
                let leg = geometry
                    .supports
                    .iter()
                    .find(|part| part.piece_id == id && part.role == "tripodLeg")
                    .unwrap();
                let actual = (leg.end[0] - base[0]).atan2(leg.end[2] - base[2]);
                assert!(
                    (actual - angle).abs() < 1e-9,
                    "{instrument:?}: {actual} != {angle}"
                );
            }
        }
    }

    #[test]
    fn compact_snare_base_preserves_the_pedal_and_seated_foot_corridor() {
        let mut kit = crate::model::default_kit();
        kit.retain(|piece| {
            matches!(
                piece.instrument,
                Instrument::Snare | Instrument::Kick | Instrument::PedalHat
            )
        });
        let snare = kit
            .iter_mut()
            .find(|piece| piece.instrument == Instrument::Snare)
            .unwrap();
        snare.position = [-0.0156, 0.825, 0.511];
        // Independently generated Studio snare measurement, not imported art.
        snare.geometry=Some(serde_json::from_value(serde_json::json!({
            "version":1,"referenceRadius":0.1778,"source":"procedural",
            "body":{"center":[0.,-0.0678343554,0.0045341708],"halfExtents":[0.1913595082,0.0842273128,0.1945067039]},
            "surface":{"center":[0.,0.0000195116,0.0002790259],"normal":[0.,0.9975640502598242,-0.0697564737441253],"radius":0.1706629661}
        })).unwrap());
        let geometry = kit_geometry(&kit);
        assert!(!geometry.unresolved_piece_ids.contains(&"snare".to_string()));
        let supports: Vec<_> = geometry
            .supports
            .iter()
            .filter(|part| part.piece_id == "snare")
            .collect();
        let base = supports
            .iter()
            .find(|part| part.role == "tubeLower")
            .unwrap()
            .start;
        let feet: Vec<_> = supports
            .iter()
            .filter(|part| part.role == "tripodLeg")
            .collect();
        assert_eq!(feet.len(), 3);
        assert!(feet
            .iter()
            .all(|part| ((part.end[0] - base[0]).hypot(part.end[2] - base[2])
                - SNARE_TRIPOD_RADIUS)
                .abs()
                < 1e-9));
        // Foot capsule through flat and raised toe phases: the former 300 mm
        // boom-style base entered this corridor by more than 35 mm.
        for (heel, toe) in [
            ([0.2, 0.105, 0.4562166], [0.2, 0.12, 0.62]),
            ([0.2, 0.09835399, 0.45013833], [0.2, 0.14243372, 0.60859014]),
        ] {
            for part in &supports {
                let gap = crate::clearance::segment_distance(heel, toe, part.start, part.end)
                    - 0.04
                    - part.radius;
                assert!(
                    gap >= CLEARANCE,
                    "{} enters the foot corridor: {gap}",
                    part.role
                );
            }
        }
    }
}
