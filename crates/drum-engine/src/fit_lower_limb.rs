//! Seated lower-limb envelopes are part of layout feasibility. Static hand
//! reach cannot certify a floor tom or stand that intersects the player's leg.
//! These are measured/fallback capsules, not a skinned-mesh collision proof.
use crate::{
    clearance, equipment,
    model::{Instrument, KitPiece, Vec3},
    motion::{add, distance, sub, two_bone_lengths},
    rig::{pitch_rotation, rotate, RigProfile},
};
#[cfg(test)]
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct Segment {
    start: Vec3,
    end: Vec3,
    radius: f64,
    foot: bool,
    #[cfg(test)]
    pitch: f64,
    #[cfg(test)]
    kind: &'static str,
}

#[derive(Debug)]
struct UnreachableProbe {
    #[cfg(test)]
    pitch: f64,
    #[cfg(test)]
    clearance: f64,
}

struct Samples {
    capsules: Vec<Segment>,
    boxes: Vec<(equipment::EquipmentBox, bool, f64, &'static str)>,
}

fn samples(
    profile: &RigProfile,
    kit: &[KitPiece],
    limb: usize,
) -> Result<Samples, UnreachableProbe> {
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
            return Err(UnreachableProbe {
                #[cfg(test)]
                pitch,
                #[cfg(test)]
                clearance: (reach - ((upper - lower).abs() + 1e-5))
                    .min(upper + lower - 1e-4 - reach),
            });
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
                    pitch,
                    if part == 0 { "thigh-box" } else { "shin-box" },
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
                pitch,
                "foot-box",
            ));
        }
        segments.extend([
            Segment {
                start: profile.roots[limb],
                end: knee,
                radius: radii[0],
                foot: false,
                #[cfg(test)]
                pitch,
                #[cfg(test)]
                kind: "thigh-capsule",
            },
            Segment {
                start: knee,
                end: shin_end,
                radius: radii[1],
                foot: false,
                #[cfg(test)]
                pitch,
                #[cfg(test)]
                kind: "shin-capsule",
            },
            Segment {
                start: actual_ankle,
                end: toe_axis,
                radius: foot_radius,
                foot: true,
                #[cfg(test)]
                pitch,
                #[cfg(test)]
                kind: "foot-capsule",
            },
        ]);
    }
    Ok(Samples {
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

/// Detailed failing samples for diagnosis. Signed capsule clearance and SAT
/// axis gaps are proxy metrics, not exact skinned-mesh penetration depths.
#[cfg(test)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LowerLimbDiagnostic {
    pub violation: String,
    pub side: &'static str,
    /// Pedal preparation pitch in radians.
    pub pitch: f64,
    pub proxy_kind: &'static str,
    pub piece_id: String,
    pub obstacle_group: &'static str,
    pub obstacle_kind: &'static str,
    /// Index in bodies chained with pedals, or in supports; absent for reach.
    pub obstacle_index: Option<usize>,
    pub clearance_meters: f64,
    pub required_clearance_meters: f64,
    pub metric: &'static str,
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum ProbeClearance<'a> {
    Scalar(f64, f64, &'static str),
    Boxes(&'a equipment::EquipmentBox, &'a equipment::EquipmentBox),
}
struct ProbeFailure<'a> {
    side: &'static str,
    #[cfg(test)]
    pitch: f64,
    #[cfg(test)]
    proxy_kind: &'static str,
    piece_id: &'a str,
    obstacle_group: &'static str,
    #[cfg(test)]
    obstacle_kind: &'static str,
    #[cfg(test)]
    obstacle_index: Option<usize>,
    #[cfg(test)]
    clearance: ProbeClearance<'a>,
}
impl ProbeFailure<'_> {
    fn code(&self) -> String {
        if self.obstacle_group == "reach" {
            format!("{}-leg-unreachable", self.side)
        } else {
            format!("{}-leg-clearance:{}", self.side, self.piece_id)
        }
    }
    #[cfg(test)]
    fn diagnostic(&self) -> LowerLimbDiagnostic {
        let (clearance_meters, required_clearance_meters, metric) = match self.clearance {
            ProbeClearance::Scalar(value, required, metric) => (value, required, metric),
            ProbeClearance::Boxes(a, b) => (
                box_axis_gap(a, b),
                equipment::CLEARANCE - 1e-9,
                "sat-axis-gap",
            ),
        };
        LowerLimbDiagnostic {
            violation: self.code(),
            side: self.side,
            pitch: self.pitch,
            proxy_kind: self.proxy_kind,
            piece_id: self.piece_id.into(),
            obstacle_group: self.obstacle_group,
            obstacle_kind: self.obstacle_kind,
            obstacle_index: self.obstacle_index,
            clearance_meters,
            required_clearance_meters,
            metric,
        }
    }
}

/// Report the same normalized-axis gap used by equipment::boxes_clear. This
/// diagnostic runs only when requested; the acceptance still calls boxes_clear.
#[cfg(test)]
fn box_axis_gap(a: &equipment::EquipmentBox, b: &equipment::EquipmentBox) -> f64 {
    let axes = |box_: &equipment::EquipmentBox| {
        [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]
            .map(|axis| box_.rotation.map_or(axis, |q| rotate(q, axis)))
    };
    let aa = axes(a);
    let ba = axes(b);
    let dot = |a: Vec3, b: Vec3| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
    let cross = |a: Vec3, b: Vec3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let gap = |axis: Vec3| {
        let length = distance(axis, [0.; 3]);
        if length < 1e-10 {
            return f64::NEG_INFINITY;
        }
        let axis = axis.map(|v| v / length);
        let projected = |basis: [Vec3; 3], half: Vec3| {
            (0..3)
                .map(|i| dot(axis, basis[i]).abs() * half[i])
                .sum::<f64>()
        };
        dot(sub(a.center, b.center), axis).abs()
            - projected(aa, a.half_extents)
            - projected(ba, b.half_extents)
    };
    aa.into_iter()
        .chain(ba)
        .chain(
            aa.into_iter()
                .flat_map(|x| ba.into_iter().map(move |y| cross(x, y))),
        )
        .map(gap)
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Exact failing comparisons, retaining side, pitch, body proxy and obstacle.
/// No candidates, margins, exceptions or acceptance outcomes are changed.
#[cfg(test)]
pub(super) fn diagnostics(
    profile: &RigProfile,
    kit: &[KitPiece],
    geometry: &equipment::EquipmentGeometry,
) -> Vec<LowerLimbDiagnostic> {
    let mut out = Vec::new();
    visit_failures(profile, kit, geometry, |failure| {
        out.push(failure.diagnostic())
    });
    out
}

/// Stable diagnostics identify the physical obstacle and side. Every support
/// remains included, including supports belonging to the contacted instrument.
pub(super) fn violations(
    profile: &RigProfile,
    kit: &[KitPiece],
    geometry: &equipment::EquipmentGeometry,
) -> Vec<String> {
    let mut out = BTreeSet::new();
    visit_failures(profile, kit, geometry, |failure| {
        out.insert(failure.code());
    });
    out.into_iter().collect()
}

fn visit_failures(
    profile: &RigProfile,
    kit: &[KitPiece],
    geometry: &equipment::EquipmentGeometry,
    mut emit: impl FnMut(ProbeFailure<'_>),
) {
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
        let segments = match samples(profile, kit, limb) {
            Ok(samples) => samples,
            Err(_probe) => {
                emit(ProbeFailure {
                    side,
                    #[cfg(test)]
                    pitch: _probe.pitch,
                    #[cfg(test)]
                    proxy_kind: "ankle-reach",
                    piece_id: own_pedal.map_or("", |p| p.id.as_str()),
                    obstacle_group: "reach",
                    #[cfg(test)]
                    obstacle_kind: "annulus",
                    #[cfg(test)]
                    obstacle_index: None,
                    #[cfg(test)]
                    clearance: ProbeClearance::Scalar(_probe.clearance, 0., "reach-bound-gap"),
                });
                continue;
            }
        };
        for segment in &segments.capsules {
            for (index, body) in geometry.bodies.iter().chain(&geometry.pedals).enumerate() {
                if segment.foot
                    && body.kind == "board"
                    && own_pedal.is_some_and(|p| p.id == body.piece_id)
                {
                    continue;
                }
                let gap = equipment::segment_box_clearance(
                    segment.start,
                    segment.end,
                    segment.radius,
                    body,
                );
                if gap < equipment::CLEARANCE {
                    emit(ProbeFailure {
                        side,
                        #[cfg(test)]
                        pitch: segment.pitch,
                        #[cfg(test)]
                        proxy_kind: segment.kind,
                        piece_id: &body.piece_id,
                        obstacle_group: if index < geometry.bodies.len() {
                            "body"
                        } else {
                            "pedal"
                        },
                        #[cfg(test)]
                        obstacle_kind: body.kind,
                        #[cfg(test)]
                        obstacle_index: Some(index),
                        #[cfg(test)]
                        clearance: ProbeClearance::Scalar(
                            gap,
                            equipment::CLEARANCE,
                            "capsule-box-distance",
                        ),
                    });
                }
            }
            for support in &geometry.supports {
                let gap = clearance::segment_distance(
                    segment.start,
                    segment.end,
                    support.start,
                    support.end,
                ) - segment.radius
                    - support.radius;
                if gap < equipment::CLEARANCE {
                    emit(ProbeFailure {
                        side,
                        #[cfg(test)]
                        pitch: segment.pitch,
                        #[cfg(test)]
                        proxy_kind: segment.kind,
                        piece_id: &support.piece_id,
                        obstacle_group: "support",
                        #[cfg(test)]
                        obstacle_kind: support.role,
                        #[cfg(test)]
                        obstacle_index: geometry
                            .supports
                            .iter()
                            .position(|candidate| std::ptr::eq(candidate, support)),
                        #[cfg(test)]
                        clearance: ProbeClearance::Scalar(
                            gap,
                            equipment::CLEARANCE,
                            "capsule-capsule-distance",
                        ),
                    });
                }
            }
        }
        for (limb_box, foot, _pitch, _kind) in &segments.boxes {
            for (index, body) in geometry.bodies.iter().chain(&geometry.pedals).enumerate() {
                if *foot && body.kind == "board" && own_pedal.is_some_and(|p| p.id == body.piece_id)
                {
                    continue;
                }
                if !equipment::boxes_clear(limb_box, body) {
                    emit(ProbeFailure {
                        side,
                        #[cfg(test)]
                        pitch: *_pitch,
                        #[cfg(test)]
                        proxy_kind: _kind,
                        piece_id: &body.piece_id,
                        obstacle_group: if index < geometry.bodies.len() {
                            "body"
                        } else {
                            "pedal"
                        },
                        #[cfg(test)]
                        obstacle_kind: body.kind,
                        #[cfg(test)]
                        obstacle_index: Some(index),
                        #[cfg(test)]
                        clearance: ProbeClearance::Boxes(limb_box, body),
                    });
                }
            }
            for support in &geometry.supports {
                let gap = equipment::segment_box_clearance(
                    support.start,
                    support.end,
                    support.radius,
                    limb_box,
                );
                if gap < equipment::CLEARANCE {
                    emit(ProbeFailure {
                        side,
                        #[cfg(test)]
                        pitch: *_pitch,
                        #[cfg(test)]
                        proxy_kind: _kind,
                        piece_id: &support.piece_id,
                        obstacle_group: "support",
                        #[cfg(test)]
                        obstacle_kind: support.role,
                        #[cfg(test)]
                        obstacle_index: geometry
                            .supports
                            .iter()
                            .position(|candidate| std::ptr::eq(candidate, support)),
                        #[cfg(test)]
                        clearance: ProbeClearance::Scalar(
                            gap,
                            equipment::CLEARANCE,
                            "capsule-box-distance",
                        ),
                    });
                }
            }
        }
    }
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
    fn detailed_failures_match_stable_codes_and_locate_the_shin_obstacle() {
        let (profile, kit, mut geometry) = fixture();
        let shin = samples(&profile, &kit, 3).unwrap().capsules[1];
        let centre = std::array::from_fn(|i| (shin.start[i] + shin.end[i]) * 0.5);
        geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "authored-obstacle".into(),
            role: "tripodLeg",
            start: centre,
            end: add(centre, [0.1, 0., 0.]),
            radius: 0.012,
        });
        let details = diagnostics(&profile, &kit, &geometry);
        let codes: BTreeSet<_> = details.iter().map(|d| d.violation.clone()).collect();
        assert_eq!(
            codes.into_iter().collect::<Vec<_>>(),
            violations(&profile, &kit, &geometry)
        );
        let flat_shin = details
            .iter()
            .find(|d| d.side == "right" && d.pitch == 0. && d.proxy_kind == "shin-capsule")
            .unwrap();
        assert_eq!(flat_shin.obstacle_group, "support");
        assert_eq!(flat_shin.obstacle_kind, "tripodLeg");
        assert_eq!(flat_shin.obstacle_index, Some(0));
        assert_eq!(flat_shin.metric, "capsule-capsule-distance");
        assert!((flat_shin.clearance_meters + shin.radius + 0.012).abs() < 1e-9);
        assert_eq!(flat_shin.required_clearance_meters, equipment::CLEARANCE);
        assert!(details
            .iter()
            .all(|d| d.clearance_meters < d.required_clearance_meters));
        serde_json::to_string(&details).unwrap();
        geometry.supports[0].start[0] += 2.;
        geometry.supports[0].end[0] += 2.;
        assert!(diagnostics(&profile, &kit, &geometry).is_empty());
    }
    #[test]
    fn reported_box_gap_keeps_sat_threshold_and_rotated_box_acceptance() {
        let a = equipment::EquipmentBox {
            piece_id: "a".into(),
            kind: "body",
            center: [0.; 3],
            half_extents: [0.1, 0.2, 0.3],
            rotation: None,
        };
        for rotation in [None, Some(pitch_rotation([0., 0., 0., 1.], 0.4))] {
            for x in [0., 0.199, 0.204999998, 0.205, 0.4] {
                let b = equipment::EquipmentBox {
                    piece_id: "b".into(),
                    kind: "body",
                    center: [x, 0., 0.],
                    half_extents: [0.1, 0.2, 0.3],
                    rotation,
                };
                assert_eq!(
                    equipment::boxes_clear(&a, &b),
                    box_axis_gap(&a, &b) >= equipment::CLEARANCE - 1e-9
                );
            }
        }
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
        let details = diagnostics(&profile, &kit, &geometry);
        let unreachable = details
            .iter()
            .find(|d| d.violation == "right-leg-unreachable")
            .unwrap();
        assert_eq!(unreachable.pitch, 0.);
        assert_eq!(unreachable.metric, "reach-bound-gap");
        assert!(unreachable.clearance_meters < 0.);
        assert_eq!(unreachable.required_clearance_meters, 0.);
    }
}
