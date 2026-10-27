//! Seated lower-limb envelopes are part of layout feasibility. Static hand
//! reach cannot certify a floor tom or stand that intersects the player's leg.
//! These are measured/fallback capsules, not a skinned-mesh collision proof.
use crate::{
    clearance, equipment,
    model::{Instrument, KitPiece, Vec3},
    motion::{add, distance, sub, two_bone_lengths},
    rig::{pitch_rotation, rotate, RigProfile},
};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct Segment {
    start: Vec3,
    end: Vec3,
    radius: f64,
    foot: bool,
}

struct Samples {
    capsules: Vec<Segment>,
    boxes: Vec<(equipment::EquipmentBox, bool)>,
}

fn samples(profile: &RigProfile, kit: &[KitPiece], limb: usize) -> Option<Samples> {
    let instrument = if limb == 2 {
        Instrument::PedalHat
    } else {
        Instrument::Kick
    };
    let pedal = kit.iter().find(|p| p.instrument == instrument);
    let [upper, lower] = profile.lengths[limb];
    let measured_legs = profile
        .body_geometry
        .as_ref()
        .and_then(|b| b.leg_boxes.as_ref());
    let measured_feet = profile
        .body_geometry
        .as_ref()
        .and_then(|b| b.foot_boxes.as_ref());
    let radii = profile
        .body_geometry
        .as_ref()
        .and_then(|b| measured_legs.is_none().then_some(b.leg_radii).flatten())
        .map_or(
            [
                (upper * 0.14).clamp(0.025, 0.09),
                (lower * 0.10).clamp(0.02, 0.07),
            ],
            |r| r[limb - 2],
        );
    let foot_radius = profile
        .body_geometry
        .as_ref()
        .and_then(|b| measured_feet.is_none().then_some(b.foot_radii).flatten())
        .map_or(0.04, |r| r[limb - 2]);
    let mut segments = Vec::with_capacity(9);
    let mut boxes = Vec::with_capacity(9);
    // Include the flat strike/ready position and both sampled preparation
    // extrema. No score-dependent layout or hidden retiming is introduced.
    for pitch in [0., -0.09, -0.18] {
        let rotation = pedal.map_or_else(
            || pitch_rotation(profile.contact_rotations[limb], pitch),
            |p| equipment::pedal_rotation(p, profile.contact_rotations[limb], pitch),
        );
        let tip = pedal.map_or(profile.rest_tips[limb], |p| {
            equipment::pedal_contact(p, pitch)
        });
        let ankle = sub(tip, profile.offset(limb, rotation));
        let reach = distance(profile.roots[limb], ankle);
        if reach < (upper - lower).abs() + 1e-5 || reach > upper + lower - 1e-4 {
            return None;
        }
        let (knee, actual_ankle) = two_bone_lengths(
            profile.roots[limb],
            ankle,
            profile.bend_poles[limb],
            profile.lengths[limb],
        );
        let normal = rotate(rotation, [0., 1., 0.]);
        // The distal footwear axis sits one radius above the sole contact.
        // Its intended board contact is handled separately from every shell,
        // unrelated board and stand. Thigh/shin never receive that exception.
        let toe_axis = add(tip, normal.map(|v| v * foot_radius));
        // A finite shin ends at its ankle joint; the rounded distal cap may not
        // extend another full shin radius into the connected footwear/board.
        // Shorten only the cap axis, never by more than half the real segment.
        // All bodies, supports and boards still test this same bounded capsule.
        let shin_axis = sub(knee, actual_ankle);
        let shin_length = distance(knee, actual_ankle).max(1e-9);
        let cap_trim = radii[1].min(shin_length * 0.5);
        let shin_end = add(actual_ankle, shin_axis.map(|v| v * cap_trim / shin_length));
        if let Some(legs) = measured_legs {
            let body_right = sub(profile.roots[3], profile.roots[2]);
            for (part, (local, start, end)) in [
                (&legs[limb - 2][0], profile.roots[limb], knee),
                (&legs[limb - 2][1], knee, actual_ankle),
            ]
            .into_iter()
            .enumerate()
            {
                let rotation = profile
                    .body_geometry
                    .as_ref()
                    .and_then(|b| b.leg_rest_frames)
                    .map_or_else(
                        || basis_rotation(segment_basis(start, end, body_right)),
                        |frames| transport_frame(frames[limb - 2][part], sub(end, start)),
                    );
                boxes.push((
                    equipment::EquipmentBox {
                        piece_id: "body".into(),
                        kind: "leg",
                        center: add(start, rotate(rotation, local.center)),
                        half_extents: local.half_extents,
                        rotation: Some(rotation),
                    },
                    false,
                ));
            }
        }
        if let Some(feet) = measured_feet {
            let local = &feet[limb - 2];
            boxes.push((
                equipment::EquipmentBox {
                    piece_id: "body".into(),
                    kind: "foot",
                    center: add(actual_ankle, rotate(rotation, local.center)),
                    half_extents: local.half_extents,
                    rotation: Some(rotation),
                },
                true,
            ));
        }
        segments.extend([
            Segment {
                start: profile.roots[limb],
                end: knee,
                radius: radii[0],
                foot: false,
            },
            Segment {
                start: knee,
                end: shin_end,
                radius: radii[1],
                foot: false,
            },
            Segment {
                start: actual_ankle,
                end: toe_axis,
                radius: foot_radius,
                foot: true,
            },
        ]);
    }
    Some(Samples {
        capsules: segments,
        boxes,
    })
}

/// The same body-right projected frame is used while measuring each raw bone
/// and while reconstructing the seated limb. Off-axis clothing remains offset;
/// it is not inflated into a solid circular leg. A core capsule remains present.
fn segment_basis(start: Vec3, end: Vec3, body_right: Vec3) -> [Vec3; 3] {
    let unit = |v: Vec3| {
        let length = distance(v, [0.; 3]).max(1e-12);
        v.map(|x| x / length)
    };
    let y = unit(sub(end, start));
    let project = |axis: Vec3| {
        let dot = (0..3).map(|i| axis[i] * y[i]).sum::<f64>();
        std::array::from_fn(|i| axis[i] - y[i] * dot)
    };
    let mut x = project(body_right);
    if distance(x, [0.; 3]) < 1e-8 {
        x = project([1., 0., 0.]);
    }
    if distance(x, [0.; 3]) < 1e-8 {
        x = project([0., 0., 1.]);
    }
    let x = unit(x);
    let z = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    [x, y, z]
}
#[cfg(test)]
fn frame_point(local: Vec3, origin: Vec3, axes: [Vec3; 3]) -> Vec3 {
    std::array::from_fn(|i| origin[i] + (0..3).map(|j| axes[j][i] * local[j]).sum::<f64>())
}
/// Transport the measured rest box using exactly the adapter's minimum-swing
/// bone rotation. Reprojecting world-right after bending adds an unintended
/// axial roll and can move an offset garment envelope by centimetres.
fn transport_frame(frame: crate::rig::Quat, direction: Vec3) -> crate::rig::Quat {
    let from = rotate(frame, [0., 1., 0.]);
    let length = distance(direction, [0.; 3]).max(1e-12);
    let to = direction.map(|v| v / length);
    let w = 1. + (0..3).map(|i| from[i] * to[i]).sum::<f64>();
    let mut swing = if w < 1e-8 {
        if from[0].abs() > from[2].abs() {
            [-from[1], from[0], 0., 0.]
        } else {
            [0., -from[2], from[1], 0.]
        }
    } else {
        [
            from[1] * to[2] - from[2] * to[1],
            from[2] * to[0] - from[0] * to[2],
            from[0] * to[1] - from[1] * to[0],
            w,
        ]
    };
    let norm = swing.iter().map(|v| v * v).sum::<f64>().sqrt();
    swing = swing.map(|v| v / norm);
    let [ax, ay, az, aw] = swing;
    let [bx, by, bz, bw] = frame;
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}
fn basis_rotation([x, y, z]: [Vec3; 3]) -> crate::rig::Quat {
    let trace = x[0] + y[1] + z[2];
    if trace > 0. {
        let s = (trace + 1.).sqrt() * 2.;
        [
            (y[2] - z[1]) / s,
            (z[0] - x[2]) / s,
            (x[1] - y[0]) / s,
            s * 0.25,
        ]
    } else if x[0] > y[1] && x[0] > z[2] {
        let s = (1. + x[0] - y[1] - z[2]).sqrt() * 2.;
        [
            s * 0.25,
            (y[0] + x[1]) / s,
            (z[0] + x[2]) / s,
            (y[2] - z[1]) / s,
        ]
    } else if y[1] > z[2] {
        let s = (1. + y[1] - x[0] - z[2]).sqrt() * 2.;
        [
            (y[0] + x[1]) / s,
            s * 0.25,
            (z[1] + y[2]) / s,
            (z[0] - x[2]) / s,
        ]
    } else {
        let s = (1. + z[2] - x[0] - y[1]).sqrt() * 2.;
        [
            (z[0] + x[2]) / s,
            (z[1] + y[2]) / s,
            s * 0.25,
            (x[1] - y[0]) / s,
        ]
    }
}

/// Stable diagnostics identify the physical obstacle and side. Every support
/// remains included, including supports belonging to the contacted instrument.
pub(super) fn violations(
    profile: &RigProfile,
    kit: &[KitPiece],
    geometry: &equipment::EquipmentGeometry,
) -> Vec<String> {
    let mut out = BTreeSet::new();
    for limb in 2..4 {
        let side = if limb == 2 { "left" } else { "right" };
        let own_pedal = kit.iter().find(|p| {
            p.instrument
                == if limb == 2 {
                    Instrument::PedalHat
                } else {
                    Instrument::Kick
                }
        });
        let Some(segments) = samples(profile, kit, limb) else {
            out.insert(format!("{side}-leg-unreachable"));
            continue;
        };
        for segment in &segments.capsules {
            for body in geometry.bodies.iter().chain(&geometry.pedals) {
                if segment.foot
                    && body.kind == "board"
                    && own_pedal.is_some_and(|p| p.id == body.piece_id)
                {
                    continue;
                }
                if equipment::segment_box_clearance(
                    segment.start,
                    segment.end,
                    segment.radius,
                    body,
                ) < equipment::CLEARANCE
                {
                    out.insert(format!("{side}-leg-clearance:{}", body.piece_id));
                }
            }
            for support in &geometry.supports {
                if clearance::segment_distance(
                    segment.start,
                    segment.end,
                    support.start,
                    support.end,
                ) - segment.radius
                    - support.radius
                    < equipment::CLEARANCE
                {
                    out.insert(format!("{side}-leg-clearance:{}", support.piece_id));
                }
            }
        }
        for (limb_box, foot) in &segments.boxes {
            for body in geometry.bodies.iter().chain(&geometry.pedals) {
                if *foot && body.kind == "board" && own_pedal.is_some_and(|p| p.id == body.piece_id)
                {
                    continue;
                }
                if !equipment::boxes_clear(limb_box, body) {
                    out.insert(format!("{side}-leg-clearance:{}", body.piece_id));
                }
            }
            for support in &geometry.supports {
                if equipment::segment_box_clearance(
                    support.start,
                    support.end,
                    support.radius,
                    limb_box,
                ) < equipment::CLEARANCE
                {
                    out.insert(format!("{side}-leg-clearance:{}", support.piece_id));
                }
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (RigProfile, Vec<KitPiece>, equipment::EquipmentGeometry) {
        let mut profile = RigProfile::primitive();
        profile.roots[2] = [-0.1, 0.66, 0.14];
        profile.roots[3] = [0.1, 0.66, 0.14];
        for side in 2..4 {
            profile.grip_offsets[side] = [0., -0.025, 0.];
            profile.tip_offsets[side] = [0., 0., 0.164];
        }
        let kit = crate::model::default_kit();
        let geometry = equipment::EquipmentGeometry {
            version: 1,
            bodies: Vec::new(),
            supports: Vec::new(),
            pedals: Vec::new(),
            mechanisms: Vec::new(),
            unresolved_piece_ids: Vec::new(),
        };
        (profile, kit, geometry)
    }
    #[test]
    fn a_clear_hand_target_cannot_certify_a_shell_or_rod_through_the_leg() {
        let (profile, kit, mut geometry) = fixture();
        let leg = samples(&profile, &kit, 3).unwrap().capsules;
        let centre = std::array::from_fn(|i| (leg[1].start[i] + leg[1].end[i]) * 0.5);
        geometry.bodies.push(equipment::EquipmentBox {
            piece_id: "floor".into(),
            center: centre,
            half_extents: [0.05; 3],
            rotation: None,
            kind: "body",
        });
        assert!(violations(&profile, &kit, &geometry).contains(&"right-leg-clearance:floor".into()));
        geometry.bodies[0].center[0] += 1.;
        assert!(violations(&profile, &kit, &geometry).is_empty());
        geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "floor".into(),
            role: "tripodLeg",
            start: centre,
            end: add(centre, [0.1, 0., 0.]),
            radius: 0.012,
        });
        assert!(violations(&profile, &kit, &geometry).contains(&"right-leg-clearance:floor".into()));
    }
    #[test]
    fn finite_shin_cap_keeps_board_contact_but_not_obstacle_exemptions() {
        let (profile, kit, mut geometry) = fixture();
        let kick = kit
            .iter()
            .find(|p| p.instrument == Instrument::Kick)
            .unwrap();
        geometry.pedals.push(equipment::EquipmentBox {
            piece_id: kick.id.clone(),
            center: add(kick.position, [0., -0.012, -0.02]),
            half_extents: [0.05, 0.012, 0.11],
            rotation: None,
            kind: "board",
        });
        assert!(violations(&profile, &kit, &geometry).is_empty());
        let shin = samples(&profile, &kit, 3).unwrap().capsules[1];
        geometry.pedals[0].center = std::array::from_fn(|i| (shin.start[i] + shin.end[i]) * 0.5);
        assert!(
            violations(&profile, &kit, &geometry).contains(&"right-leg-clearance:kick".into()),
            "the own board is not exempt from the shin volume"
        );
    }
    #[test]
    fn segment_frames_preserve_offset_geometry_under_seated_bending() {
        for end in [[0., -0.4, 0.], [0.1, -0.2, 0.3], [-0.3, 0.2, 0.1]] {
            let start = [0.05, 0.7, 0.14];
            let basis = segment_basis(start, add(start, end), [1., 0., 0.]);
            let rotation = basis_rotation(basis);
            for local in [[0.2, 0., 0.], [0., 0.3, 0.], [0., 0., 0.1]] {
                assert!(
                    distance(
                        frame_point(local, start, basis),
                        add(start, rotate(rotation, local))
                    ) < 1e-9
                );
            }
            assert!((rotation.iter().map(|q| q * q).sum::<f64>() - 1.).abs() < 1e-9);
        }
    }
    #[test]
    fn measured_rest_frame_uses_minimum_swing_without_added_axial_roll() {
        let rest = basis_rotation(segment_basis([0.; 3], [0.02, -0.4, 0.04], [1., 0., 0.]));
        for target in [[0.1, -0.3, 0.2], [-0.1, -0.2, 0.3], [0., 0.4, 0.]] {
            let q = transport_frame(rest, target);
            let n = distance(target, [0.; 3]);
            assert!(distance(rotate(q, [0., 1., 0.]), target.map(|v| v / n)) < 1e-9);
            assert!((q.iter().map(|v| v * v).sum::<f64>() - 1.).abs() < 1e-9);
        }
        assert!(
            distance(
                rotate(
                    transport_frame(rest, rotate(rest, [0., 1., 0.])),
                    [0.2, 0.1, -0.15]
                ),
                rotate(rest, [0.2, 0.1, -0.15])
            ) < 1e-9
        );
    }
    #[test]
    fn offset_measured_box_keeps_its_empty_space_and_detects_its_actual_volume() {
        let (mut profile, kit, mut geometry) = fixture();
        let box_ = crate::rig::TorsoGeometry {
            center: [0., 0.2, 0.15],
            half_extents: [0.025, 0.07, 0.025],
        };
        profile.body_geometry = Some(crate::rig::BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            torso: crate::rig::TorsoGeometry {
                center: [0., 1., 0.],
                half_extents: [0.1; 3],
            },
            arm_radii: [[0.03; 2]; 2],
            hand_radii: [0.04; 2],
            leg_radii: Some([[0.2; 2]; 2]),
            foot_radii: None,
            leg_boxes: Some([[box_.clone(), box_.clone()], [box_.clone(), box_]]),
            leg_rest_frames: None,
            foot_boxes: None,
            hand_centers: None,
            hand_capsules: None,
            sample_count: 100,
        });
        let mut validated = profile.clone();
        validated.body_geometry.as_mut().unwrap().leg_rest_frames =
            Some([[[0., 0., 0., 1.]; 2]; 2]);
        validated.validate().unwrap();
        validated
            .body_geometry
            .as_mut()
            .unwrap()
            .leg_rest_frames
            .as_mut()
            .unwrap()[0][0][3] = 2.;
        assert!(validated.validate().is_err());
        validated
            .body_geometry
            .as_mut()
            .unwrap()
            .leg_rest_frames
            .as_mut()
            .unwrap()[0][0][3] = f64::NAN;
        assert!(validated.validate().is_err());
        validated.body_geometry.as_mut().unwrap().leg_rest_frames =
            Some([[[0., 0., 0., 1.]; 2]; 2]);
        validated.body_geometry.as_mut().unwrap().leg_boxes = None;
        assert!(validated.validate().is_err());
        let parts = samples(&profile, &kit, 3).unwrap();
        let lower = &parts.boxes[1].0;
        let core = &parts.capsules[1];
        assert!(
            core.radius < 0.1,
            "measured box replaces inflated legacy radius but retains skeleton core"
        );
        geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "floor".into(),
            role: "floorLeg",
            start: lower.center,
            end: add(lower.center, [0., 0.015, 0.]),
            radius: 0.005,
        });
        assert!(violations(&profile, &kit, &geometry).contains(&"right-leg-clearance:floor".into()));
        geometry.supports[0].start[0] += 1.;
        geometry.supports[0].end[0] += 1.;
        assert!(violations(&profile, &kit, &geometry).is_empty());
    }
    #[test]
    fn pedal_extrema_retain_real_segments_and_unreachable_is_explicit() {
        let (mut profile, kit, geometry) = fixture();
        let parts = samples(&profile, &kit, 3).unwrap().capsules;
        assert_eq!(parts.len(), 9);
        assert_ne!(parts[2].end, parts[8].end);
        for sample in parts.chunks_exact(3) {
            assert!(
                (distance(sample[0].start, sample[0].end) - profile.lengths[3][0]).abs() < 1e-9
            );
            assert!(distance(sample[1].start, sample[1].end) <= profile.lengths[3][1]);
        }
        profile.lengths[3] = [0.04; 2];
        assert!(violations(&profile, &kit, &geometry).contains(&"right-leg-unreachable".into()));
    }
}
