//! Whole-tool/equipment queries shared by path preparation and elbow planning.
//! Conservative measured boxes and rendered support capsules, not mesh collision.
use crate::{
    clearance, contact, equipment,
    model::*,
    motion::{add, distance, sub},
    rig::{rotate, HandSolution, RigProfile},
};
use std::sync::Arc;

pub const PITCH_OFFSETS: [f64; 9] = [0., -0.15, 0.15, -0.30, 0.30, -0.45, 0.45, -0.60, 0.60];

pub struct EquipmentContext {
    pub kit: Vec<KitPiece>,
    pub geometry: equipment::EquipmentGeometry,
}
impl EquipmentContext {
    pub fn for_kit(kit: &[KitPiece]) -> Option<Arc<Self>> {
        kit.iter().any(|p| p.geometry.is_some()).then(|| {
            Arc::new(Self {
                kit: kit.to_vec(),
                geometry: equipment::kit_geometry(kit),
            })
        })
    }
}
fn mul(a: Vec3, s: f64) -> Vec3 {
    a.map(|v| v * s)
}
pub fn stick_segment(profile: &RigProfile, limb: usize, hand: &HandSolution) -> (Vec3, Vec3) {
    let grip = add(hand.end, rotate(hand.rotation, profile.grip_offsets[limb]));
    let axis = sub(hand.tip, grip);
    let len = distance(axis, [0.; 3]).max(1e-9);
    (
        sub(
            grip,
            mul(axis, crate::stick_geometry::profile().butt_length / len),
        ),
        hand.tip,
    )
}
/// Exact minimum of the clipped-box support-plane bound minus a linear
/// taper radius. Each interval is a maximum of seven affine functions, so its
/// minimum occurs at an endpoint or at an intersection of two functions.
fn tapered_surface_interval(
    start: Vec3,
    end: Vec3,
    radii: [f64; 2],
    body: &equipment::EquipmentBox,
    surface: &equipment::Surface,
) -> f64 {
    let mut lines = [(0.0, 0.0); 7];
    for axis in 0..3 {
        for side in 0..2 {
            let sign = if side == 0 { -1.0 } else { 1.0 };
            lines[axis * 2 + side] = (
                sign * (start[axis] - body.center[axis]) - body.half_extents[axis] - radii[0],
                sign * (end[axis] - start[axis]) - (radii[1] - radii[0]),
            );
        }
    }
    let dot = |a: Vec3, b: Vec3| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    lines[6] = (
        dot(sub(start, surface.center), surface.normal) - radii[0],
        dot(sub(end, start), surface.normal) - (radii[1] - radii[0]),
    );
    let value = |t: f64| {
        lines
            .iter()
            .map(|(a, b)| a + b * t)
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let mut best = value(0.).min(value(1.));
    for i in 0..lines.len() {
        for j in i + 1..lines.len() {
            let denominator = lines[i].1 - lines[j].1;
            if denominator.abs() > 1e-12 {
                let t = (lines[j].0 - lines[i].0) / denominator;
                if (0.0..=1.0).contains(&t) {
                    best = best.min(value(t));
                }
            }
        }
    }
    best
}
fn tapered_surface_clearance(
    butt: Vec3,
    tip: Vec3,
    body: &equipment::EquipmentBox,
    surface: &equipment::Surface,
    terminal_contact: bool,
) -> f64 {
    let length = distance(butt, tip).max(1e-9);
    let axis = mul(sub(butt, tip), 1.0 / length);
    let profile = &crate::stick_geometry::profile().terminal;
    let mut best = f64::INFINITY;
    // Existing axis-endpoint pose format approximates a rounded commercial tip.
    // Its last 20 mm is the declared intended-contact region, not a certificate
    // of tangent contact for that cap. The remaining measured taper is tested.
    let terminal = if terminal_contact { 0.02 } else { 0.0 };
    for pair in profile.windows(2) {
        if pair[0][0] >= length {
            break;
        }
        let start = pair[0][0].max(terminal);
        let end = pair[1][0].min(length);
        if start >= end {
            continue;
        }
        let radius = |d: f64| {
            pair[0][1] + (pair[1][1] - pair[0][1]) * (d - pair[0][0]) / (pair[1][0] - pair[0][0])
        };
        best = best.min(tapered_surface_interval(
            add(tip, mul(axis, start)),
            add(tip, mul(axis, end)),
            [radius(start), radius(end)],
            body,
            surface,
        ));
    }
    let last = profile.last().expect("tool profile");
    if length > last[0] {
        best = best.min(tapered_surface_interval(
            add(tip, mul(axis, last[0])),
            butt,
            [last[1]; 2],
            body,
            surface,
        ));
    }
    best
}

fn segment_bounds(start: Vec3, end: Vec3) -> [Vec3; 2] {
    [
        std::array::from_fn(|axis| start[axis].min(end[axis])),
        std::array::from_fn(|axis| start[axis].max(end[axis])),
    ]
}

/// The boxes enclose the segment axes, so their distance is a lower bound
/// on capsule clearance. Prune against the running minimum, never a fixed
/// collision threshold: negative penetration depths and pose costs must survive.
fn support_minimum(
    segments: [(Vec3, Vec3, f64); 4],
    supports: &[equipment::EquipmentCapsule],
    mut minimum: f64,
) -> f64 {
    let bounds = segments.map(|(start, end, _)| segment_bounds(start, end));
    for rod in supports {
        let obstacle = segment_bounds(rod.start, rod.end);
        for (index, &(start, end, radius)) in segments.iter().enumerate() {
            // Keep a conservative 1 nm band for floating-point boundary cases.
            // This only requests more exact queries; it never changes clearance.
            let threshold = minimum + radius + rod.radius + 1e-9;
            let lower_squared: f64 = (0..3)
                .map(|axis| {
                    (bounds[index][0][axis] - obstacle[1][axis])
                        .max(obstacle[0][axis] - bounds[index][1][axis])
                        .max(0.0)
                        .powi(2)
                })
                .sum();
            if threshold <= 0.0 || lower_squared >= threshold * threshold {
                continue;
            }
            minimum = minimum.min(
                clearance::segment_distance(start, end, rod.start, rod.end) - radius - rod.radius,
            );
        }
    }
    minimum
}

pub fn hand_clearance(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
    intended_piece: Option<&str>,
) -> f64 {
    let radii = clearance::arm_radii(profile, limb);
    let (hs, he, hr) = clearance::hand_volume(profile, limb, hand);
    let (butt, tip) = stick_segment(profile, limb, hand);
    let shaft_radius = crate::stick_geometry::profile()
        .terminal
        .last()
        .expect("tool profile")[1];
    let arm = [
        (profile.roots[limb], hand.middle, radii[0]),
        (hand.middle, hand.end, radii[1]),
        (hs, he, hr),
    ];
    let intended_body = intended_piece.and_then(|id| {
        let piece = ctx.kit.iter().find(|p| p.id == id)?;
        if piece.instrument == Instrument::OpenHat {
            // Open/closed articulations share a body only at the same physical
            // anchor/radius. Do not exempt an arbitrary second hi-hat.
            ctx.kit
                .iter()
                .find(|p| {
                    p.instrument == Instrument::Hihat
                        && distance(p.position, piece.position) < 1e-9
                        && (p.radius - piece.radius).abs() < 1e-9
                })
                .map(|p| p.id.as_str())
        } else {
            Some(piece.id.as_str())
        }
    });
    let mut minimum = f64::INFINITY;
    for body in &ctx.geometry.bodies {
        for &(a, b, r) in &arm {
            minimum = minimum.min(equipment::segment_box_clearance(a, b, r, body));
        }
        // An intended striking surface permits physical tangent contact, while
        // every unrelated body retains the free-obstacle margin. Test the
        // authored tapered tool, including its butt. Only the existing 20 mm
        // terminal contact region is exempt for an exact intended impact.
        // Only the intended body can use a surface-contact exception. Avoid
        // projecting the tip onto every unrelated radial/tilted surface; their
        // complete-shaft box query below is independent of that projection.
        let piece = if intended_body == Some(body.piece_id.as_str()) {
            ctx.kit.iter().find(|p| p.id == body.piece_id)
        } else {
            None
        };
        let surface = piece.map(|piece| {
            let (center, normal) = contact::project_surface(piece, tip);
            let (_, _, radius) = contact::surface(piece);
            (center, normal, radius)
        });
        let approach = intended_body == Some(body.piece_id.as_str())
            && surface.is_some_and(|(point, normal, radius)| {
                let (center, _, _) = contact::surface(piece.expect("surface owner"));
                let d = sub(tip, center);
                let height = d.into_iter().zip(normal).map(|(a, b)| a * b).sum::<f64>();
                let actual_height = sub(tip, point)
                    .into_iter()
                    .zip(normal)
                    .map(|(a, b)| a * b)
                    .sum::<f64>();
                actual_height >= -0.000_000_1
                    && distance(sub(d, mul(normal, height)), [0.; 3]) <= radius * 0.96
            });
        let value = if approach {
            let (center, normal, radius) = surface.expect("approach surface");
            let contact_height = sub(tip, center)
                .iter()
                .zip(normal)
                .map(|(a, b)| a * b)
                .sum::<f64>();
            let clear = tapered_surface_clearance(
                butt,
                tip,
                body,
                &equipment::Surface {
                    center,
                    normal,
                    radius,
                    radial_profile: None,
                    motion: None,
                },
                contact_height.abs() <= 0.012,
            );
            // The 0.1 nm numerical allowance handles a tangent point represented
            // through different coordinate transforms; it cannot hide a mesh gap.
            (if clear.abs() < 1e-10 { 0.0 } else { clear }) + crate::posture::MARGIN
        } else {
            equipment::segment_box_clearance(butt, tip, shaft_radius, body)
        };
        minimum = minimum.min(value);
    }
    for pedal in &ctx.geometry.pedals {
        for &(a, b, r) in arm
            .iter()
            .chain(std::iter::once(&(butt, tip, shaft_radius)))
        {
            minimum = minimum.min(equipment::segment_box_clearance(a, b, r, pedal));
        }
    }
    support_minimum(
        [arm[0], arm[1], arm[2], (butt, tip, shaft_radius)],
        &ctx.geometry.supports,
        minimum,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSample {
    pub time: f64,
    pub lift: [f64; 2],
    pub pitch: [f64; 2],
}
pub fn limit_rates(track: &mut [PathSample], impacts: &[Vec<f64>; 2]) {
    // First cap the requested lift by the distance to immutable zero-lift
    // contacts. Then extend preparation into neighbouring time samples instead
    // of flattening a necessary obstacle-clearing peak into its previous zero.
    for (limb, contacts) in impacts.iter().enumerate() {
        let mut caps: Vec<f64> = track
            .iter()
            .map(|s| {
                if contacts.binary_search_by(|t| t.total_cmp(&s.time)).is_ok() {
                    0.0
                } else {
                    0.20
                }
            })
            .collect();
        // Only immutable impact knots are hard zeros here; raised
        // neighbours are propagated below within the same finite rate budget.
        for i in 1..caps.len() {
            caps[i] = caps[i].min(caps[i - 1] + 1.2 * (track[i].time - track[i - 1].time));
        }
        for i in (0..caps.len().saturating_sub(1)).rev() {
            caps[i] = caps[i].min(caps[i + 1] + 1.2 * (track[i + 1].time - track[i].time));
        }
        for (s, cap) in track.iter_mut().zip(&caps) {
            s.lift[limb] = s.lift[limb].min(*cap);
        }
        for i in 1..track.len() {
            track[i].lift[limb] = track[i].lift[limb]
                .max(track[i - 1].lift[limb] - 1.2 * (track[i].time - track[i - 1].time))
                .min(caps[i]);
        }
        for i in (0..track.len().saturating_sub(1)).rev() {
            track[i].lift[limb] = track[i].lift[limb]
                .max(track[i + 1].lift[limb] - 1.2 * (track[i + 1].time - track[i].time))
                .min(caps[i]);
        }
        for i in 1..track.len() {
            let dt = track[i].time - track[i - 1].time;
            track[i].pitch[limb] = track[i].pitch[limb].clamp(
                track[i - 1].pitch[limb] - 2.0 * dt,
                track[i - 1].pitch[limb] + 2.0 * dt,
            );
        }
        for i in (0..track.len().saturating_sub(1)).rev() {
            let dt = track[i + 1].time - track[i].time;
            track[i].pitch[limb] = track[i].pitch[limb].clamp(
                track[i + 1].pitch[limb] - 2.0 * dt,
                track[i + 1].pitch[limb] + 2.0 * dt,
            );
        }
    }
}

pub fn sample(track: &[PathSample], time: f64, limb: usize) -> (f64, f64) {
    if track.is_empty() {
        return (0., 0.);
    }
    let index = track.partition_point(|s| s.time < time);
    let a = track[index.saturating_sub(1)];
    let b = track[index.min(track.len() - 1)];
    let w = if b.time > a.time {
        ((time - a.time) / (b.time - a.time)).clamp(0., 1.)
    } else {
        0.
    };
    (
        a.lift[limb] * (1. - w) + b.lift[limb] * w,
        a.pitch[limb] * (1. - w) + b.pitch[limb] * w,
    )
}
/// Bounded preparation search. Impact lift is always zero: source timing and
/// musical contact survive a failed route, which is reported by inspection.
pub fn correction(
    profile: &RigProfile,
    ctx: &EquipmentContext,
    limb: usize,
    tip: Vec3,
    pitch: f64,
    impact: bool,
    intended_piece: Option<&str>,
) -> (f64, f64) {
    correction_search::<true>(profile, ctx, limb, tip, pitch, impact, intended_piece)
}

// The unpruned instantiation is used by the differential regression to verify
// that pruning retains the original exhaustive winner and tie order.
fn correction_search<const PRUNE: bool>(
    profile: &RigProfile,
    ctx: &EquipmentContext,
    limb: usize,
    tip: Vec3,
    pitch: f64,
    impact: bool,
    intended_piece: Option<&str>,
) -> (f64, f64) {
    let mut best = (f64::INFINITY, 0., 0.);
    let lifts = if impact {
        &[0.][..]
    } else {
        &[0., 0.025, 0.05, 0.09, 0.14, 0.20][..]
    };
    for &lift in lifts {
        for delta in PITCH_OFFSETS {
            // Collision and comfort terms are nonnegative. A candidate whose
            // fixed cost cannot beat the current strict-< winner cannot change
            // the result, including exact ties. No geometry check is weakened.
            if PRUNE && lift * 0.1 + delta.abs() * 0.005 >= best.0 {
                continue;
            }
            let target = add(tip, [0., lift, 0.]);
            for swivel in crate::posture::SWIVELS {
                let Some(hand) =
                    profile.solve_hand_with_swivel(limb, target, pitch + delta, swivel)
                else {
                    continue;
                };
                if !crate::posture::anatomical(profile, limb, target, &hand) {
                    continue;
                }
                let clear = hand_clearance(profile, limb, &hand, ctx, intended_piece)
                    .min(clearance::hand_clearance(profile, limb, &hand));
                let penalty = (0.002 - clear).max(0.);
                let cost = penalty * penalty * 10_000.
                    + if penalty > 0. { 1. } else { 0. }
                    + lift * 0.1
                    + delta.abs() * 0.005
                    + crate::posture::comfort_cost(profile, limb, target, &hand);
                if cost < best.0 {
                    best = (cost, lift, delta);
                }
                if cost == 0. {
                    return (0., 0.);
                }
            }
        }
    }
    (best.1, best.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exhaustive_support_minimum(
        segments: [(Vec3, Vec3, f64); 4],
        supports: &[equipment::EquipmentCapsule],
        mut minimum: f64,
    ) -> f64 {
        for rod in supports {
            for (start, end, radius) in segments {
                minimum = minimum.min(
                    clearance::segment_distance(start, end, rod.start, rod.end)
                        - radius
                        - rod.radius,
                );
            }
        }
        minimum
    }

    #[test]
    fn support_broadphase_preserves_exact_minima_for_seeded_and_degenerate_segments() {
        let mut seed = 0x71e8_a34d_2f69_u64;
        let mut sample = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((seed >> 32) as f64 / u32::MAX as f64) * 4. - 2.
        };
        for case in 0..256 {
            let segments = std::array::from_fn(|i| {
                let start = [sample(), sample(), sample()];
                let end = if (case + i) % 7 == 0 {
                    start
                } else {
                    add(start, [sample() * 0.25, sample() * 0.25, sample() * 0.25])
                };
                (start, end, sample().abs() * 0.02 + 0.003)
            });
            let supports: Vec<_> = (0..224)
                .map(|i| {
                    let start = [sample(), sample(), sample()];
                    let end = match i % 5 {
                        0 => start,
                        1 => add(start, [0., sample(), 0.]),
                        _ => add(start, [sample(), sample(), sample()]),
                    };
                    equipment::EquipmentCapsule {
                        piece_id: "test-hardware".into(),
                        role: "rod",
                        start,
                        end,
                        radius: sample().abs() * 0.015 + 0.005,
                    }
                })
                .collect();
            for minimum in [f64::INFINITY, 0.2, 0.005, 0., -0.01, -0.10] {
                assert_eq!(
                    support_minimum(segments, &supports, minimum),
                    exhaustive_support_minimum(segments, &supports, minimum),
                    "case {case}, initial minimum {minimum}"
                );
            }
        }
    }

    #[test]
    fn support_broadphase_keeps_tangent_near_tangent_and_deeper_negative_witnesses() {
        let segments = [
            ([-0.5, 0., 0.], [0.5, 0., 0.], 0.02),
            ([2., -0.5, 0.], [2., 0.5, 0.], 0.015),
            ([2.25, 0., 0.], [2.25, 0., 0.], 0.03),
            ([0.5, 0., 0.], [-0.5, 0., 0.], 0.006),
        ];
        let mut supports = Vec::new();
        for separation in [0.05 + 1e-10, 0.05, 0.05 - 1e-10, 0.02, 0.] {
            supports.push(equipment::EquipmentCapsule {
                piece_id: "test-hardware".into(),
                role: "rod",
                start: [-0.5, separation, 0.],
                end: [0.5, separation, 0.],
                radius: 0.03,
            });
            for minimum in [f64::INFINITY, 0., -0.001, -0.04, -0.10] {
                assert_eq!(
                    support_minimum(segments, &supports, minimum),
                    exhaustive_support_minimum(segments, &supports, minimum)
                );
            }
        }
        assert_eq!(support_minimum(segments, &supports, -0.01), -0.05);
        supports.reverse();
        assert_eq!(
            support_minimum(segments, &supports, f64::INFINITY),
            exhaustive_support_minimum(segments, &supports, f64::INFINITY)
        );
    }

    #[test]
    fn shaft_middle_and_butt_are_obstacles_even_when_the_tip_is_clear() {
        let mut profile = RigProfile::primitive();
        profile.roots[0] = [-0.2, 1.3, -0.4];
        profile.grip_offsets[0] = [0.; 3];
        let hand = HandSolution {
            middle: [-0.2, 1.1, -0.2],
            end: [0., 1., 0.],
            tip: [0., 1., 0.5],
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        let mut ctx = EquipmentContext {
            kit: Vec::new(),
            geometry: equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: "other-tom".into(),
                    center: [0., 1., 0.25],
                    half_extents: [0.02, 0.02, 0.02],
                    kind: "body",
                }],
                supports: Vec::new(),
                pedals: Vec::new(),
                mechanisms: Vec::new(),
                unresolved_piece_ids: Vec::new(),
            },
        };
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        ctx.geometry.bodies.clear();
        ctx.geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "stand".into(),
            role: "rod",
            start: [-0.1, 1., -0.08],
            end: [0.1, 1., -0.08],
            radius: 0.01,
        });
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        ctx.geometry.supports[0].start[1] = 0.5;
        ctx.geometry.supports[0].end[1] = 0.5;
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) > 0.);
    }
    #[test]
    fn surface_contact_requires_the_intended_physical_piece() {
        let mut profile = RigProfile::primitive();
        profile.roots[0] = [-0.2, 1.5, -0.4];
        profile.grip_offsets[0] = [0.0; 3];
        let hand = HandSolution {
            middle: [-0.2, 1.4, -0.2],
            end: [0.0, 1.4, 0.0],
            tip: [0.0, 1.0, 0.5],
            rotation: [0.0, 0.0, 0.0, 1.0],
            reachable: true,
        };
        let mut hat = default_kit()
            .into_iter()
            .find(|p| p.instrument == Instrument::Hihat)
            .unwrap();
        hat.id = "main-hat".into();
        hat.position = hand.tip;
        hat.radius = 0.10;
        let mut open = hat.clone();
        open.id = "open-articulation".into();
        open.instrument = Instrument::OpenHat;
        let mut unrelated = hat.clone();
        unrelated.id = "unrelated-tom".into();
        unrelated.instrument = Instrument::TomHigh;
        let mut ctx = EquipmentContext {
            kit: vec![hat.clone(), open, unrelated],
            geometry: equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: hat.id.clone(),
                    center: [0.0, 0.97, 0.5],
                    half_extents: [0.10, 0.03, 0.10],
                    kind: "body",
                }],
                supports: vec![],
                pedals: vec![],
                mechanisms: vec![],
                unresolved_piece_ids: vec![],
            },
        };
        assert!(hand_clearance(&profile, 0, &hand, &ctx, Some("main-hat")) > 0.002);
        // A cue has no drum target; a different target at the same coordinates
        // must not acquire this body's terminal-contact exception either.
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) < -0.005);
        assert!(hand_clearance(&profile, 0, &hand, &ctx, Some("unrelated-tom")) < -0.005);
        assert!(hand_clearance(&profile, 0, &hand, &ctx, Some("unknown")) < -0.005);
        assert!(hand_clearance(&profile, 0, &hand, &ctx, Some("open-articulation")) > 0.002);
        // An OpenHat label alone cannot alias another instrument's location.
        ctx.kit[1].position[0] += 0.5;
        assert!(hand_clearance(&profile, 0, &hand, &ctx, Some("open-articulation")) < -0.005);
    }
    #[test]
    fn intended_taper_accepts_shallow_approach_without_exempting_other_bodies() {
        let body = equipment::EquipmentBox {
            piece_id: "snare".into(),
            center: [0., -0.1, 0.],
            half_extents: [0.2, 0.1, 0.2],
            kind: "body",
        };
        let surface = equipment::Surface {
            center: [0.; 3],
            normal: [0., 1., 0.],
            radius: 0.18,
            radial_profile: None,
            motion: None,
        };
        let angle = 14_f64.to_radians();
        let butt = [0., 0.4 * angle.sin(), -0.4 * angle.cos()];
        let tip = [0.; 3];
        assert!(tapered_surface_clearance(butt, tip, &body, &surface, true) > 0.0005);
        // The tip convention remains an approximation; demanding the pointed
        // cap itself tangent would reject this shallow real stick approach.
        assert!(tapered_surface_clearance(butt, tip, &body, &surface, false) < 0.);
        // An unrelated body receives the unchanged complete-shaft test.
        assert!(equipment::segment_box_clearance(butt, tip, 0.006, &body) < 0.);
        let buried = [0., -0.03, 0.];
        assert!(tapered_surface_clearance(butt, buried, &body, &surface, true) < -0.01);
    }
    #[test]
    fn bounded_search_preserves_exhaustive_winners_for_impacts_and_transfers() {
        let mut profile = RigProfile::primitive();
        for limb in 0..2 {
            profile.roots[limb] = [if limb == 0 { -0.19 } else { 0.19 }, 1.235, 0.06];
            profile.lengths[limb] = [0.3, 0.27];
        }
        let frame = crate::rig::HandControlFrame {
            forearm: [0.0, 0.0, 1.0],
            palm: [0.0, 1.0, 0.0],
            desired_palm: [0.0, 1.0, 0.0],
        };
        profile.hand_frames = Some([frame.clone(), frame]);
        let kit = default_kit();
        let ctx = EquipmentContext {
            geometry: equipment::kit_geometry(&kit),
            kit,
        };
        let mut non_neutral = 0;
        for (id, limb, impact, pitch) in [
            ("snare", 0, true, 0.0),
            ("snare", 1, false, -0.2),
            ("hihat", 1, true, 0.0),
            ("tomMid", 1, false, -0.2),
            ("tomLow", 0, false, 0.0),
            ("crash", 0, true, -0.2),
        ] {
            let piece = ctx.kit.iter().find(|piece| piece.id == id).unwrap();
            let target = contact::target(piece, "center", profile.chest);
            for intended in [Some(id), None] {
                let expected = correction_search::<false>(
                    &profile, &ctx, limb, target, pitch, impact, intended,
                );
                let actual = correction(&profile, &ctx, limb, target, pitch, impact, intended);
                assert_eq!(
                    actual, expected,
                    "{id}, limb {limb}, impact {impact}, intended {intended:?}"
                );
                non_neutral += usize::from(expected != (0.0, 0.0));
                if impact {
                    assert_eq!(actual.0, 0.0);
                }
            }
        }
        assert!(
            non_neutral > 0,
            "exercise more than the zero-cost early exit"
        );
    }
    #[test]
    fn bounded_corrections_keep_exact_impact_lift_and_random_access() {
        let mut path = vec![
            PathSample {
                time: 0.,
                lift: [0.; 2],
                pitch: [0.; 2],
            },
            PathSample {
                time: 0.05,
                lift: [0.2; 2],
                pitch: [0.6; 2],
            },
            PathSample {
                time: 0.1,
                lift: [0.; 2],
                pitch: [-0.6; 2],
            },
        ];
        limit_rates(&mut path, &[vec![0.0, 0.1], vec![0.0, 0.1]]);
        assert_eq!(sample(&path, 0., 0).0, 0.);
        assert_eq!(sample(&path, 0.1, 1).0, 0.);
        for pair in path.windows(2) {
            assert!(
                (pair[1].lift[0] - pair[0].lift[0]).abs()
                    <= 1.2 * (pair[1].time - pair[0].time) + 1e-9
            );
            assert!(
                (pair[1].pitch[0] - pair[0].pitch[0]).abs()
                    <= 2. * (pair[1].time - pair[0].time) + 1e-9
            );
        }
        let before = sample(&path, 0.031, 0);
        let _ = sample(&path, 0.09, 1);
        assert_eq!(before, sample(&path, 0.031, 0));
    }
}
