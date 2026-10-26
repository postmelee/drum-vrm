//! Common conservative equipment geometry for rendering, fitting and motion.
//! Bounds describe rigid equipment, not exact triangles. Every generated support
//! is returned to the renderer; failed routing is explicit and never hidden.
use crate::model::{Instrument, KitPiece, Vec3};
use crate::motion::{add, distance, sub};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CLEARANCE: f64 = 0.005;
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
    pub kind: &'static str,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentCapsule {
    pub piece_id: String,
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
    piece.geometry.as_ref().is_none_or(|g| {
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
                finite(profile.origin, 3.0)
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
            radial_profile: None,
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
        kind: "body",
    })
}

/// Exact minimum distance from a finite segment to an axis-aligned box. Squared
/// distance is piecewise quadratic; breakpoints are the six slab crossings.
/// Inside the box, the result is zero, so radius gives signed penetration of a
/// capsule into the conservative box (not triangle penetration depth).
pub fn segment_box_clearance(start: Vec3, end: Vec3, radius: f64, box_: &EquipmentBox) -> f64 {
    let a = sub(start, box_.center);
    let d = sub(end, start);
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
    (0..3).any(|axis| {
        (a.center[axis] - b.center[axis]).abs() - a.half_extents[axis] - b.half_extents[axis]
            >= CLEARANCE - 1e-9
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
        start,
        end,
        radius,
    }
}
fn stand(piece: &KitPiece, base: Vec3, top: Vec3, rotation: f64) -> Vec<EquipmentCapsule> {
    let mut rods = vec![capsule(piece, base, top, 0.011)];
    // The height-adjustment collar is a short thick capsule and participates in
    // the same checks as the structural tubing.
    let collar = [base[0], top[1] * 0.55, base[2]];
    rods.push(capsule(
        piece,
        add(collar, [0.0, -0.01, 0.0]),
        add(collar, [0.0, 0.01, 0.0]),
        0.023,
    ));
    for leg in 0..3 {
        let angle = leg as f64 * std::f64::consts::TAU / 3.0 + rotation;
        rods.push(capsule(
            piece,
            [base[0], (top[1] * 0.6).min(0.25), base[2]],
            [
                base[0] + angle.sin() * 0.19,
                0.012,
                base[2] + angle.cos() * 0.19,
            ],
            0.011,
        ));
    }
    rods
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
    [
        pedal.position[0] - (bounds.center[0] - piece.position[0]),
        piece.position[1],
        pedal.position[2] + HIHAT_PEDAL_TO_AXIS - (bounds.center[2] - piece.position[2]),
    ]
}

/// Sole contact follows the same rear footboard hinge as the visible board.
/// Zero pitch is the original immutable strike contact, not a shifted MIDI hit.
pub fn pedal_contact(piece: &KitPiece, pitch: f64) -> Vec3 {
    let [x, y, z] = piece.position;
    let (sin, cos) = pitch.sin_cos();
    [
        x,
        y - 0.012 + 0.012 * cos - 0.13 * sin,
        z - 0.13 + 0.012 * sin + 0.13 * cos,
    ]
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
                })
            } else if piece.instrument == Instrument::Hihat {
                let pedal = kit.iter().find(|p| p.instrument == Instrument::PedalHat)?;
                let [px, py, pz] = pedal.position;
                let s = surface(piece);
                Some(EquipmentMechanism {
                    piece_id: piece.id.clone(),
                    kind: "hihat",
                    pivot: [px, py - 0.012, pz - 0.13],
                    axle: [px, py + 0.04, pz + HIHAT_PEDAL_TO_AXIS],
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
            let [x, y, z] = p.position;
            [
                EquipmentBox {
                    piece_id: p.id.clone(),
                    center: [x, y - 0.012, z - 0.02],
                    half_extents: [0.05, 0.012, 0.11],
                    kind: "board",
                },
                EquipmentBox {
                    piece_id: p.id.clone(),
                    center: [x, 0.018, z - 0.02],
                    half_extents: [0.065, 0.0125, 0.135],
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
fn rods_clear(
    rods: &[EquipmentCapsule],
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
) -> bool {
    rods.iter().all(|rod| {
        bodies.iter().chain(boards).all(|obstacle| {
            same_owner(&rod.piece_id, &obstacle.piece_id, kit)
                || segment_box_clearance(rod.start, rod.end, rod.radius, obstacle) >= CLEARANCE
        })
    })
}
fn route(
    piece: &KitPiece,
    bodies: &[EquipmentBox],
    boards: &[EquipmentBox],
    kit: &[KitPiece],
) -> (Vec<EquipmentCapsule>, bool) {
    let physical = body(piece).expect("routed equipment has body");
    let cymbal = matches!(
        piece.instrument,
        Instrument::Hihat | Instrument::Crash | Instrument::Ride
    );
    let mount = if cymbal {
        [
            physical.center[0],
            physical.center[1] + physical.half_extents[1] + 0.012,
            physical.center[2],
        ]
    } else {
        [
            physical.center[0],
            physical.center[1] - physical.half_extents[1] - 0.014,
            physical.center[2],
        ]
    };
    if piece.instrument == Instrument::TomLow {
        let mut rods = Vec::new();
        for leg in 0..3 {
            let angle = leg as f64 * std::f64::consts::TAU / 3.0 + 0.4;
            let radial = [angle.sin(), 0.0, angle.cos()];
            let r = physical.half_extents[0].max(physical.half_extents[2]) + 0.012;
            let start = [
                physical.center[0] + radial[0] * r,
                (physical.center[1] - physical.half_extents[1] * 0.3).max(0.08),
                physical.center[2] + radial[2] * r,
            ];
            let bend = [start[0], (start[1] * 0.42).max(0.04), start[2]];
            let foot = [
                start[0] + radial[0] * 0.07,
                0.012,
                start[2] + radial[2] * 0.07,
            ];
            rods.push(capsule(piece, start, bend, 0.011));
            rods.push(capsule(piece, bend, foot, 0.011));
        }
        let clear = rods_clear(&rods, bodies, boards, kit);
        return (rods, clear);
    }
    if matches!(piece.instrument, Instrument::TomHigh | Instrument::TomMid) {
        if let Some(kick) = kit.iter().find(|p| p.instrument == Instrument::Kick) {
            let bass = body(kick).expect("kick body");
            let start = [
                bass.center[0],
                bass.center[1] + bass.half_extents[1] + 0.017,
                bass.center[2],
            ];
            // Shared central holder with independent L-arms. Never add a separate
            // tripod below each rack tom or pass a post through another shell.
            // Both branches share a holder below the lowest rack shell. Raising
            // each holder to its own tom's underside made the higher branch
            // cross a neighbouring lower shell, despite a valid tom layout.
            let lowest_mount = kit
                .iter()
                .filter(|p| matches!(p.instrument, Instrument::TomHigh | Instrument::TomMid))
                .filter_map(body)
                .map(|b| b.center[1] - b.half_extents[1] - 0.014)
                .fold(mount[1], f64::min);
            let holder_height = (start[1] + 0.12).min(lowest_mount - 0.03).max(start[1]);
            let elbow = [start[0], holder_height, start[2]];
            let rods = vec![
                capsule(piece, start, elbow, 0.011),
                capsule(piece, elbow, mount, 0.010),
            ];
            let clear = rods_clear(&rods, bodies, boards, kit);
            return (rods, clear);
        }
        // A standalone rack tom may use a dedicated stand when no bass assembly
        // exists; this is a semantic attachment fallback, never an avatar rule.
    }
    let build = |dx: f64, dz: f64, rotation: f64| {
        let base = [mount[0] + dx, 0.025, mount[2] + dz];
        let top = [
            base[0],
            (mount[1]
                - if cymbal && piece.instrument != Instrument::Hihat && dx.hypot(dz) > 0.01 {
                    0.12
                } else {
                    0.0
                })
            .max(0.09),
            base[2],
        ];
        let mut rods = stand(piece, base, top, rotation);
        if distance(top, mount) > 1e-9 {
            rods.push(capsule(piece, top, mount, 0.011));
        }
        if cymbal {
            rods.push(capsule(
                piece,
                [mount[0], piece.position[1] - 0.02, mount[2]],
                mount,
                0.006,
            ));
        } else {
            // Basket fingers attach to the underside. The shaft never extends
            // through a lower shell as the old fixed y-.42 support could do.
            for i in 0..3 {
                let angle = i as f64 * std::f64::consts::TAU / 3.0;
                rods.push(capsule(
                    piece,
                    mount,
                    [
                        mount[0] + angle.sin() * piece.radius * 0.8,
                        mount[1] + 0.014,
                        mount[2] + angle.cos() * piece.radius * 0.8,
                    ],
                    0.009,
                ));
            }
        }
        if piece.instrument == Instrument::Hihat {
            if let Some(pedal) = kit.iter().find(|p| p.instrument == Instrument::PedalHat) {
                rods.push(capsule(
                    piece,
                    [
                        pedal.position[0],
                        0.04,
                        pedal.position[2] + HIHAT_PEDAL_TO_AXIS,
                    ],
                    [pedal.position[0], 0.04, pedal.position[2] + 0.08],
                    0.011,
                ));
            }
        }
        rods
    };
    let original = if cymbal && piece.instrument != Instrument::Hihat {
        build(if mount[0] < 0.0 { -0.16 } else { 0.16 }, 0.10, 0.4)
    } else {
        build(0.0, 0.0, 0.4)
    };
    if rods_clear(&original, bodies, boards, kit) {
        return (original, true);
    }
    // Straight hi-hat linkage cannot silently turn into a remote boom stand.
    // Other stands use bounded offsets and tripod rotations, nearest first.
    for step in 0..=5 {
        if piece.instrument == Instrument::Hihat && step > 0 {
            break;
        }
        for direction in 0..if step == 0 { 1 } else { 12 } {
            let outward = if mount[0] < 0.0 { -1.0 } else { 1.0 };
            let angle = (direction as f64) * std::f64::consts::TAU / 12.0;
            let offset = step as f64 * 0.1;
            for rotation in [0.4, 0.8, 1.2] {
                let rods = build(
                    outward * angle.cos() * offset,
                    angle.sin() * offset,
                    rotation,
                );
                if rods_clear(&rods, bodies, boards, kit) {
                    return (rods, true);
                }
            }
        }
    }
    (original, false)
}

pub fn kit_geometry(kit: &[KitPiece]) -> EquipmentGeometry {
    let bodies: Vec<_> = kit.iter().filter_map(body).collect();
    let pedals = pedals(kit);
    let mechanisms = mechanisms(kit);
    let mut supports = Vec::new();
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
        if matches!(a.instrument, Instrument::Kick | Instrument::PedalHat) {
            supports.extend(pedal_hardware(a));
        }
        if matches!(
            a.instrument,
            Instrument::Kick | Instrument::PedalHat | Instrument::OpenHat
        ) {
            continue;
        }
        let (rods, clear) = route(a, &bodies, &pedals, kit);
        if !clear {
            unresolved.insert(a.id.clone());
        }
        supports.extend(rods);
    }
    // Fixed kick/pedal supports are not exempt from other instruments. Their
    // own mount/footboard attachments are the only intended overlaps.
    for rod in &supports {
        for obstacle in bodies.iter().chain(&pedals) {
            if !same_owner(&rod.piece_id, &obstacle.piece_id, kit)
                && segment_box_clearance(rod.start, rod.end, rod.radius, obstacle) < CLEARANCE
            {
                unresolved.insert(rod.piece_id.clone());
                unresolved.insert(obstacle.piece_id.clone());
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
