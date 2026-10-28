//! Common conservative equipment geometry for rendering, fitting and motion.
//! Bounds describe rigid equipment, not exact triangles. Every generated support
//! is returned to the renderer; failed routing is explicit and never hidden.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::motion::{add, distance, sub};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CLEARANCE: f64 = 0.005;
pub const CYMBAL_FELT_RADIUS: f64 = 0.017;
pub const APPROACH_HEIGHT: f64 = 0.12;
pub const HIHAT_MAX_OPENING: f64 = 0.025;
pub const HIHAT_PEDAL_TO_AXIS: f64 = 0.12;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalBox {
    pub center: Vec3,
    pub half_extents: Vec3,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadialSurfaceProfile {
    /// Optional semantic outer bell radius in the profile's local metric frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bell_radius: Option<f64>,
    pub origin: Vec3,
    pub rotation: [f64; 4],
    /// Increasing radii and their surface heights in the local +Y frame.
    pub points: Vec<[f64; 2]>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceMotion {
    pub kind: String,
    pub axis: Vec3,
    /// Mechanical travel is absolute metres, independent of fitted shell scale.
    pub travel_meters: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Surface {
    pub center: Vec3,
    pub normal: Vec3,
    pub radius: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radial_profile: Option<RadialSurfaceProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion: Option<SurfaceMotion>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceGeometry {
    pub version: u32,
    pub reference_radius: f64,
    pub source: String,
    pub body: LocalBox,
    pub surface: Surface,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentBox {
    pub piece_id: String,
    pub center: Vec3,
    pub half_extents: Vec3,
    /// Local box axes to world; absent for axis-aligned body bounds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<crate::rig::Quat>,
    pub kind: &'static str,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentCapsule {
    pub piece_id: String,
    /// Semantic part used by rendering; the capsule remains the collision envelope.
    pub role: &'static str,
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentMechanism {
    pub piece_id: String,
    pub kind: &'static str,
    /// Rear footboard hinge, transverse beater axle, and physical surface contact.
    pub pivot: Vec3,
    pub axle: Vec3,
    pub contact: Vec3,
    pub normal: Vec3,
    pub rest_angle: f64,
    pub max_opening: f64,
    pub motion_axis: Vec3,
    /// Local footboard +Z toe / +Y up to the shared planning frame.
    pub board_rotation: crate::rig::Quat,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentGeometry {
    pub version: u32,
    pub bodies: Vec<EquipmentBox>,
    pub supports: Vec<EquipmentCapsule>,
    pub pedals: Vec<EquipmentBox>,
    pub mechanisms: Vec<EquipmentMechanism>,
    pub unresolved_piece_ids: Vec<String>,
}

pub fn valid_piece_geometry(piece: &KitPiece) -> bool {
    piece.stand_rotation.is_none_or(|angle| {
        matches!(
            piece.instrument,
            Instrument::Snare | Instrument::Hihat | Instrument::Crash | Instrument::Ride
        ) && angle.is_finite()
            && angle.abs() <= std::f64::consts::PI
    }) && piece.pedal_forward.is_none_or(|direction| {
        piece.instrument == Instrument::PedalHat
            && direction.iter().all(|value| value.is_finite())
            && direction[1].abs() <= 1e-8
            && (distance(direction, [0.; 3]) - 1.).abs() <= 1e-6
    }) && piece.geometry.as_ref().is_none_or(|g| {
        let finite = |v: Vec3, max: f64| v.iter().all(|n| n.is_finite() && n.abs() <= max);
        g.version == 1
            && matches!(g.source.as_str(), "mesh-bounds" | "procedural")
            && g.reference_radius.is_finite()
            && (0.01..=1.0).contains(&g.reference_radius)
            && finite(g.body.center, 3.0)
            && g.body
                .half_extents
                .iter()
                .all(|n| n.is_finite() && (0.001..=2.0).contains(n))
            && finite(g.surface.center, 3.0)
            && finite(g.surface.normal, 1.0)
            && (distance(g.surface.normal, [0.0; 3]) - 1.0).abs() < 1e-5
            && g.surface.radius.is_finite()
            && (0.01..=1.0).contains(&g.surface.radius)
            && g.surface.motion.as_ref().is_none_or(|motion| {
                motion.kind == "translation"
                    && finite(motion.axis, 1.0)
                    && (distance(motion.axis, [0.0; 3]) - 1.0).abs() < 1e-5
                    && motion.travel_meters.is_finite()
                    && (0.0..=HIHAT_MAX_OPENING).contains(&motion.travel_meters)
            })
            && g.surface.radial_profile.as_ref().is_none_or(|profile| {
                profile.bell_radius.is_none_or(|r| {
                    r.is_finite() && r > 0.0 && profile.points.last().is_some_and(|p| r <= p[0])
                }) && finite(profile.origin, 3.0)
                    && profile.rotation.iter().all(|v| v.is_finite())
                    && (profile.rotation.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-5
                    && (2..=32).contains(&profile.points.len())
                    && profile.points.iter().all(|point| {
                        point[0].is_finite()
                            && point[1].is_finite()
                            && (0.0..=1.0).contains(&point[0])
                            && point[1].abs() <= 1.0
                    })
                    && profile.points.windows(2).all(|p| p[0][0] < p[1][0])
            })
            && (0.05..=5.0).contains(&(piece.radius / g.reference_radius))
    })
}

/// Legacy layouts receive a documented procedural descriptor, not a guessed
/// asset identity. Measured descriptors replace these dimensions completely.
pub fn local_geometry(piece: &KitPiece) -> PieceGeometry {
    if let Some(geometry) = &piece.geometry {
        return geometry.clone();
    }
    use Instrument::*;
    let r = piece.radius;
    let cymbal = matches!(piece.instrument, Hihat | OpenHat | Crash | Ride);
    let (center, half_extents) = if piece.instrument == Kick {
        (
            [0.0, r + 0.018 - piece.position[1], 0.34],
            [r + 0.017, r + 0.017, 0.22],
        )
    } else if cymbal {
        let low = if matches!(piece.instrument, Hihat | OpenHat) {
            -0.045
        } else {
            -0.02
        };
        (
            [0.0, (low + 0.05) * 0.5, r * 0.5],
            [r + 0.001, (0.05 - low) * 0.5, r + 0.001],
        )
    } else {
        let depth = match piece.instrument {
            Snare => 0.135,
            TomLow => 0.295,
            _ => 0.225,
        };
        (
            [0.0, (0.008 - depth) * 0.5, 0.0],
            [r + 0.014, (depth + 0.008) * 0.5, r + 0.014],
        )
    };
    PieceGeometry {
        version: 1,
        reference_radius: r,
        source: "procedural".into(),
        body: LocalBox {
            center,
            half_extents,
        },
        surface: Surface {
            // The authored fallback is an analytic flat cymbal surface. Its
            // semantic bell region lets contact selection avoid mounting felt
            // after compact fitting, just like a measured native surface.
            radial_profile: cymbal.then_some(RadialSurfaceProfile {
                bell_radius: Some(r * 0.24),
                origin: [0.0, 0.0, r * 0.5],
                rotation: [0.0, 0.0, 0.0, 1.0],
                points: vec![[0.0, 0.0], [r, 0.0]],
            }),
            motion: None,
            center: if cymbal {
                [0.0, 0.0, r * 0.5]
            } else if matches!(piece.instrument, Kick | PedalHat) {
                [0.0, 0.0, -0.02]
            } else {
                [0.0; 3]
            },
            normal: [0.0, 1.0, 0.0],
            radius: if matches!(piece.instrument, Kick | PedalHat) {
                0.05
            } else {
                r
            },
        },
    }
}
pub fn surface(piece: &KitPiece) -> Surface {
    let g = local_geometry(piece);
    let scale = piece.radius / g.reference_radius;
    Surface {
        motion: g.surface.motion.clone(),
        radial_profile: g
            .surface
            .radial_profile
            .as_ref()
            .map(|profile| RadialSurfaceProfile {
                bell_radius: profile.bell_radius.map(|r| r * scale),
                origin: add(piece.position, profile.origin.map(|v| v * scale)),
                rotation: profile.rotation,
                points: profile
                    .points
                    .iter()
                    .map(|point| [point[0] * scale, point[1] * scale])
                    .collect(),
            }),
        center: add(piece.position, g.surface.center.map(|v| v * scale)),
        normal: g.surface.normal,
        radius: g.surface.radius * scale,
    }
}
pub fn body(piece: &KitPiece) -> Option<EquipmentBox> {
    if matches!(piece.instrument, Instrument::OpenHat | Instrument::PedalHat) {
        return None;
    }
    let g = local_geometry(piece);
    let scale = piece.radius / g.reference_radius;
    Some(EquipmentBox {
        piece_id: piece.id.clone(),
        center: add(piece.position, g.body.center.map(|v| v * scale)),
        half_extents: g.body.half_extents.map(|v| v * scale),
        rotation: None,
        kind: "body",
    })
}

/// Exact minimum distance from a finite segment to an axis-aligned box. Squared
/// distance is piecewise quadratic; breakpoints are the six slab crossings.
/// Inside the box, the result is zero, so radius gives signed penetration of a
/// capsule into the conservative box (not triangle penetration depth).
pub fn segment_box_clearance(start: Vec3, end: Vec3, radius: f64, box_: &EquipmentBox) -> f64 {
    let a = box_local_vector(box_, sub(start, box_.center));
    let d = box_local_vector(box_, sub(end, start));
    let h = box_.half_extents;
    let mut knots = [0.0; 8];
    knots[1] = 1.0;
    let mut count = 2;
    for axis in 0..3 {
        if d[axis].abs() > 1e-12 {
            for sign in [-1.0, 1.0] {
                let t = (sign * h[axis] - a[axis]) / d[axis];
                if t > 0.0 && t < 1.0 {
                    knots[count] = t;
                    count += 1;
                }
            }
        }
    }
    knots[..count].sort_by(f64::total_cmp);
    let at = |t: f64| {
        (0..3)
            .map(|axis| ((a[axis] + d[axis] * t).abs() - h[axis]).max(0.0).powi(2))
            .sum::<f64>()
    };
    let mut best = at(0.0).min(at(1.0));
    for interval in knots[..count].windows(2) {
        let middle = (interval[0] + interval[1]) * 0.5;
        let (mut linear, mut quadratic) = (0.0, 0.0);
        for axis in 0..3 {
            let p = a[axis] + d[axis] * middle;
            if p.abs() > h[axis] {
                linear += d[axis] * (a[axis] - p.signum() * h[axis]);
                quadratic += d[axis] * d[axis];
            }
        }
        let t = if quadratic > 1e-14 {
            (-linear / quadratic).clamp(interval[0], interval[1])
        } else {
            middle
        };
        best = best.min(at(t)).min(at(interval[0])).min(at(interval[1]));
    }
    best.sqrt() - radius
}
pub fn boxes_clear(a: &EquipmentBox, b: &EquipmentBox) -> bool {
    let aa = box_axes(a);
    let ba = box_axes(b);
    let separates = |axis: Vec3| {
        let length = distance(axis, [0.; 3]);
        if length < 1e-10 {
            return false;
        }
        let axis = axis.map(|v| v / length);
        let projected = |axes: [Vec3; 3], half: Vec3| -> f64 {
            (0..3).map(|i| dot(axis, axes[i]).abs() * half[i]).sum()
        };
        dot(sub(a.center, b.center), axis).abs()
            - projected(aa, a.half_extents)
            - projected(ba, b.half_extents)
            >= CLEARANCE - 1e-9
    };
    aa.into_iter().chain(ba).any(separates)
        || aa
            .iter()
            .any(|&left| ba.iter().any(|&right| separates(cross(left, right))))
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    (0..3).map(|axis| a[axis] * b[axis]).sum()
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn box_axes(box_: &EquipmentBox) -> [Vec3; 3] {
    [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]
        .map(|axis| box_.rotation.map_or(axis, |q| crate::rig::rotate(q, axis)))
}
fn box_local_vector(box_: &EquipmentBox, vector: Vec3) -> Vec3 {
    box_.rotation.map_or(vector, |[x, y, z, w]| {
        crate::rig::rotate([-x, -y, -z, w], vector)
    })
}
fn box_world_half_extents(box_: &EquipmentBox) -> Vec3 {
    let axes = box_axes(box_);
    std::array::from_fn(|axis| {
        (0..3)
            .map(|i| axes[i][axis].abs() * box_.half_extents[i])
            .sum()
    })
}
/// Conservative distance lower bound to the contacted body's box clipped by
/// the inward playing half-space. This handles tilted measured surfaces without
/// pretending the entire AABB bell/rim top is a flat physical obstruction. The
/// allowed region and impact-tip attachment are selected by the caller.
pub fn segment_surface_box_clearance(
    start: Vec3,
    end: Vec3,
    radius: f64,
    box_: &EquipmentBox,
    surface: &Surface,
) -> f64 {
    let delta = sub(end, start);
    let value = |t: f64| {
        let p = add(start, delta.map(|v| v * t));
        let box_distance = (0..3)
            .map(|axis| (p[axis] - box_.center[axis]).abs() - box_.half_extents[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        let plane_distance = (0..3)
            .map(|axis| (p[axis] - surface.center[axis]) * surface.normal[axis])
            .sum::<f64>();
        box_distance.max(plane_distance)
    };
    if start == end {
        return value(0.0) - radius;
    }
    // max(convex box-distance bound, plane distance) is convex and 1-Lipschitz.
    let (mut lo, mut hi) = (0.0, 1.0);
    const R: f64 = 0.618_033_988_749_894_9;
    let (mut x, mut y) = (hi - R * (hi - lo), lo + R * (hi - lo));
    let (mut vx, mut vy) = (value(x), value(y));
    for _ in 0..18 {
        if vx < vy {
            hi = y;
            y = x;
            vy = vx;
            x = hi - R * (hi - lo);
            vx = value(x);
        } else {
            lo = x;
            x = y;
            vx = vy;
            y = lo + R * (hi - lo);
            vy = value(y);
        }
    }
    value((lo + hi) * 0.5) - (hi - lo) * distance(start, end) * 0.5 - radius
}
fn head(instrument: Instrument) -> bool {
    matches!(
        instrument,
        Instrument::Snare | Instrument::TomHigh | Instrument::TomMid | Instrument::TomLow
    )
}
/// Drumheads require distinct projected playing regions, regardless of height.
/// Cymbals may overhang another body only outside its bounded head-approach lane.
pub fn playable_pair(a: &KitPiece, b: &KitPiece) -> bool {
    if matches!(a.instrument, Instrument::OpenHat | Instrument::PedalHat)
        || matches!(b.instrument, Instrument::OpenHat | Instrument::PedalHat)
    {
        return true;
    }
    let (sa, sb) = (surface(a), surface(b));
    if head(a.instrument)
        && head(b.instrument)
        && (sa.center[0] - sb.center[0]).hypot(sa.center[2] - sb.center[2])
            < sa.radius + sb.radius + CLEARANCE
    {
        return false;
    }
    let access = |piece: &KitPiece, other: &KitPiece| {
        if !head(piece.instrument) {
            return true;
        }
        let Some(obstacle) = body(other) else {
            return true;
        };
        let s = surface(piece);
        // Reserve a useful central striking region, not just one unobstructed
        // mathematical point. Longer sideways transfers are checked by motion.
        let radius = (s.radius * 0.45).min(0.09);
        let start = add(s.center, s.normal.map(|v| v * (radius + 0.008)));
        let end = add(s.center, s.normal.map(|v| v * (APPROACH_HEIGHT + radius)));
        segment_box_clearance(start, end, radius, &obstacle) >= CLEARANCE
    };
    access(a, b) && access(b, a)
}

fn capsule(piece: &KitPiece, start: Vec3, end: Vec3, radius: f64) -> EquipmentCapsule {
    EquipmentCapsule {
        piece_id: piece.id.clone(),
        role: "rod",
        start,
        end,
        radius,
    }
}
/// Bounded setup uses anatomical leg roots, not model names, to turn a pedal.
pub fn horizontal_direction(from: Vec3, to: Vec3) -> Vec3 {
    let delta = sub(to, from);
    let length = delta[0].hypot(delta[2]);
    if length.is_finite() && length > 1e-9 {
        [delta[0] / length, 0., delta[2] / length]
    } else {
        [0., 0., 1.]
    }
}
pub fn pedal_forward(piece: &KitPiece) -> Vec3 {
    piece.pedal_forward.unwrap_or([0., 0., 1.])
}
pub fn pedal_board_rotation(piece: &KitPiece) -> crate::rig::Quat {
    let direction = pedal_forward(piece);
    let (sin, cos) = (direction[0].atan2(direction[2]) * 0.5).sin_cos();
    [0., sin, 0., cos]
}
/// Local +X is the transverse hinge, +Z points from heel toward the mast.
pub fn pedal_frame_point(piece: &KitPiece, offset: Vec3) -> Vec3 {
    add(
        piece.position,
        crate::rig::rotate(pedal_board_rotation(piece), offset),
    )
}
pub fn pedal_rotation(piece: &KitPiece, base: crate::rig::Quat, pitch: f64) -> crate::rig::Quat {
    let [x, y, z, w] = crate::rig::pitch_rotation(base, pitch);
    let [_, s, _, c] = pedal_board_rotation(piece);
    [c * x + s * z, c * y + s * w, c * z - s * x, c * w - s * y]
}
/// Keep a direct-drive hi-hat above the pedal's toe-side mechanism. The
/// contact anchor may differ from its actual measured central mast axis.
pub fn anchored_hihat_position(piece: &KitPiece, kit: &[KitPiece]) -> Vec3 {
    let Some(pedal) = kit.iter().find(|p| p.instrument == Instrument::PedalHat) else {
        return piece.position;
    };
    let Some(bounds) = body(piece) else {
        return piece.position;
    };
    let axis = pedal_frame_point(pedal, [0., 0., HIHAT_PEDAL_TO_AXIS]);
    [
        axis[0] - (bounds.center[0] - piece.position[0]),
        piece.position[1],
        axis[2] - (bounds.center[2] - piece.position[2]),
    ]
}

/// Sole contact follows the same rear footboard hinge as the visible board.
/// Zero pitch is the original immutable strike contact, not a shifted MIDI hit.
pub fn pedal_contact(piece: &KitPiece, pitch: f64) -> Vec3 {
    let (sin, cos) = pitch.sin_cos();
    pedal_frame_point(
        piece,
        [
            0.,
            -0.012 + 0.012 * cos - 0.13 * sin,
            -0.13 + 0.012 * sin + 0.13 * cos,
        ],
    )
}

/// Physical bass-drum batter contact, distinct from the foot's pedal anchor.
/// The front/back plane is approximate for arbitrary imported body meshes.
pub fn beater_contact(piece: &KitPiece) -> (Vec3, Vec3) {
    let bounds = body(piece).expect("kick has a physical body");
    (
        [
            bounds.center[0],
            bounds.center[1],
            bounds.center[2] - bounds.half_extents[2],
        ],
        [0.0, 0.0, -1.0],
    )
}
fn beater_center(piece: &KitPiece) -> Vec3 {
    let (point, normal) = beater_contact(piece);
    add(point, normal.map(|v| v * 0.028))
}
fn mechanisms(kit: &[KitPiece]) -> Vec<EquipmentMechanism> {
    kit.iter()
        .filter_map(|piece| {
            let [x, y, z] = piece.position;
            if piece.instrument == Instrument::Kick {
                let (contact, normal) = beater_contact(piece);
                Some(EquipmentMechanism {
                    piece_id: piece.id.clone(),
                    kind: "kick",
                    pivot: [x, y - 0.012, z - 0.13],
                    axle: [x, y + 0.04, z + 0.08],
                    contact,
                    normal,
                    rest_angle: 0.45,
                    max_opening: 0.0,
                    motion_axis: [0.0, 1.0, 0.0],
                    board_rotation: pedal_board_rotation(piece),
                })
            } else if piece.instrument == Instrument::Hihat {
                let pedal = kit.iter().find(|p| p.instrument == Instrument::PedalHat)?;
                let s = surface(piece);
                Some(EquipmentMechanism {
                    piece_id: piece.id.clone(),
                    kind: "hihat",
                    pivot: pedal_frame_point(pedal, [0., -0.012, -0.13]),
                    axle: pedal_frame_point(pedal, [0., 0.04, HIHAT_PEDAL_TO_AXIS]),
                    contact: s.center,
                    normal: s.normal,
                    rest_angle: 0.10,
                    max_opening: piece.geometry.as_ref().map_or(HIHAT_MAX_OPENING, |g| {
                        g.surface.motion.as_ref().map_or(0.0, |m| m.travel_meters)
                    }),
                    motion_axis: piece
                        .geometry
                        .as_ref()
                        .and_then(|g| g.surface.motion.as_ref())
                        .map_or([0.0, 1.0, 0.0], |m| m.axis),
                    board_rotation: pedal_board_rotation(pedal),
                })
            } else {
                None
            }
        })
        .collect()
}

fn pedals(kit: &[KitPiece]) -> Vec<EquipmentBox> {
    kit.iter()
        .filter(|p| matches!(p.instrument, Instrument::Kick | Instrument::PedalHat))
        .flat_map(|p| {
            [
                EquipmentBox {
                    piece_id: p.id.clone(),
                    center: pedal_frame_point(p, [0., -0.012, -0.02]),
                    half_extents: [0.05, 0.012, 0.11],
                    rotation: Some(pedal_board_rotation(p)),
                    kind: "board",
                },
                EquipmentBox {
                    piece_id: p.id.clone(),
                    center: pedal_frame_point(p, [0., 0.018 - p.position[1], -0.02]),
                    half_extents: [0.065, 0.0125, 0.135],
                    rotation: Some(pedal_board_rotation(p)),
                    kind: "base",
                },
            ]
        })
        .collect()
}
fn pedal_hardware(piece: &KitPiece) -> Vec<EquipmentCapsule> {
    let [x, y, z] = piece.position;
    let mut rods = Vec::new();
    for sign in [-1.0, 1.0] {
        rods.push(capsule(
            piece,
            [x + sign * 0.065, 0.03, z + 0.08],
            [x + sign * 0.065, y + 0.04, z + 0.08],
            0.008,
        ));
    }
    rods.push(capsule(
        piece,
        [x - 0.065, y + 0.04, z + 0.08],
        [x + 0.065, y + 0.04, z + 0.08],
        0.008,
    ));
    if piece.instrument == Instrument::Kick {
        rods.push(capsule(
            piece,
            [x, y + 0.04, z + 0.08],
            beater_center(piece),
            0.008,
        ));
        rods.push(capsule(
            piece,
            beater_center(piece),
            beater_center(piece),
            0.028,
        ));
        for sign in [-1.0, 1.0] {
            rods.push(capsule(
                piece,
                [x + sign * piece.radius * 0.78, piece.radius * 0.58, z + 0.4],
                [x + sign * (piece.radius + 0.04), 0.015, z + 0.49],
                0.012,
            ));
        }
    }
    if piece.pedal_forward.is_some() {
        for rod in &mut rods {
            rod.start = pedal_frame_point(piece, sub(rod.start, piece.position));
            rod.end = pedal_frame_point(piece, sub(rod.end, piece.position));
        }
    }
    rods
}
fn same_owner(a: &str, b: &str, kit: &[KitPiece]) -> bool {
    a == b
        || kit.iter().find(|p| p.id == a).is_some_and(|a| {
            matches!(
                a.instrument,
                Instrument::Hihat | Instrument::OpenHat | Instrument::PedalHat
            )
        }) && kit.iter().find(|p| p.id == b).is_some_and(|b| {
            matches!(
                b.instrument,
                Instrument::Hihat | Instrument::OpenHat | Instrument::PedalHat
            )
        })
}
fn rod_box_clear(rod: &EquipmentCapsule, obstacle: &EquipmentBox) -> bool {
    // A separating coordinate proves full capsule clearance. Keep the exact
    // segment/box query for overlapping expanded boxes only.
    let half = box_world_half_extents(obstacle);
    (0..3).any(|axis| {
        rod.start[axis].min(rod.end[axis]) - obstacle.center[axis] - half[axis]
            >= rod.radius + CLEARANCE
            || obstacle.center[axis] - half[axis] - rod.start[axis].max(rod.end[axis])
                >= rod.radius + CLEARANCE
    }) || segment_box_clearance(rod.start, rod.end, rod.radius, obstacle) >= CLEARANCE
}
fn rods_clear(
    rods: &[EquipmentCapsule],
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
) -> bool {
    rods.iter().all(|rod| {
        bodies.iter().chain(boards).all(|obstacle| {
            same_owner(&rod.piece_id, &obstacle.piece_id, kit) || rod_box_clear(rod, obstacle)
        })
    })
}
#[path = "equipment_hardware.rs"]
mod hardware;

pub fn kit_geometry(kit: &[KitPiece]) -> EquipmentGeometry {
    let bodies: Vec<_> = kit.iter().filter_map(body).collect();
    let pedals = pedals(kit);
    let mechanisms = mechanisms(kit);
    let mut supports: Vec<_> = kit
        .iter()
        .filter(|p| matches!(p.instrument, Instrument::Kick | Instrument::PedalHat))
        .flat_map(pedal_hardware)
        .collect();
    supports.extend(hardware::rack_holder(kit));
    let mut unresolved = BTreeSet::new();
    // A direct hi-hat cannot be disconnected from its pedal. Report mismatches;
    // the common fitting transaction is responsible for moving the body anchor.
    for hat in kit.iter().filter(|p| p.instrument == Instrument::Hihat) {
        let aligned = anchored_hihat_position(hat, kit);
        if (aligned[0] - hat.position[0]).hypot(aligned[2] - hat.position[2]) > 0.003 {
            unresolved.insert(hat.id.clone());
        }
    }
    for body in &bodies {
        for pedal in &pedals {
            if !same_owner(&body.piece_id, &pedal.piece_id, kit) && !boxes_clear(body, pedal) {
                unresolved.insert(body.piece_id.clone());
                unresolved.insert(pedal.piece_id.clone());
            }
        }
    }
    for (i, a) in pedals.iter().enumerate() {
        for b in &pedals[i + 1..] {
            if a.piece_id != b.piece_id && !boxes_clear(a, b) {
                unresolved.insert(a.piece_id.clone());
                unresolved.insert(b.piece_id.clone());
            }
        }
    }
    for (i, a) in kit.iter().enumerate() {
        if let Some(a_body) = body(a) {
            if a_body.center[1] - a_body.half_extents[1] < -0.005 {
                unresolved.insert(a.id.clone());
            }
            for b in &kit[i + 1..] {
                if let Some(b_body) = body(b) {
                    if !boxes_clear(&a_body, &b_body) || !playable_pair(a, b) {
                        unresolved.insert(a.id.clone());
                        unresolved.insert(b.id.clone());
                    }
                }
            }
        }
    }
    // Physical dependencies are routed before their attached instruments;
    // source array ordering cannot create a different support graph.
    for a in hardware::routing_order(kit) {
        if matches!(
            a.instrument,
            Instrument::Kick | Instrument::PedalHat | Instrument::OpenHat
        ) {
            continue;
        }
        let (rods, clear) = hardware::route(a, &bodies, &pedals, kit, &supports);
        if !clear {
            unresolved.insert(a.id.clone());
        }
        supports.extend(rods);
    }
    // Fixed kick/pedal supports are not exempt from other instruments. Their
    // own mount/footboard attachments are the only intended overlaps.
    for rod in &supports {
        for obstacle in bodies.iter().chain(&pedals) {
            if !same_owner(&rod.piece_id, &obstacle.piece_id, kit) && !rod_box_clear(rod, obstacle)
            {
                unresolved.insert(rod.piece_id.clone());
                unresolved.insert(obstacle.piece_id.clone());
            }
        }
    }
    for (i, a) in supports.iter().enumerate() {
        for b in &supports[i + 1..] {
            if !hardware::supports_clear(a, b, kit) {
                unresolved.insert(a.piece_id.clone());
                unresolved.insert(b.piece_id.clone());
            }
        }
    }
    EquipmentGeometry {
        version: 1,
        bodies,
        supports,
        pedals,
        mechanisms,
        unresolved_piece_ids: unresolved.into_iter().collect(),
    }
}
pub fn kit_geometry_json(kit_json: &str) -> Result<String, String> {
    if kit_json.len() > 128 * 1024 {
        return Err("키트 배치 크기가 한계를 초과합니다.".into());
    }
    let kit: Vec<KitPiece> =
        serde_json::from_str(kit_json).map_err(|e| format!("키트 배치 형식 오류: {e}"))?;
    if !crate::fit::valid(&kit) {
        return Err("키트 배치가 지원 범위를 벗어납니다.".into());
    }
    serde_json::to_string(&kit_geometry(&kit)).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "equipment_tests.rs"]
mod tests;
