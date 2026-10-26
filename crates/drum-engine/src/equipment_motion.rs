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
    (sub(grip, mul(axis, 0.105 / len)), hand.tip)
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
        // Only the last 20 mm may touch the designated measured surface. The
        // remaining shaft (including the butt) still checks that SAME body.
        let surface = ctx
            .kit
            .iter()
            .find(|p| p.id == body.piece_id)
            .map(contact::surface);
        let approach = intended_body == Some(body.piece_id.as_str())
            && surface.is_some_and(|(center, normal, radius)| {
                let d = sub(tip, center);
                let height = d.into_iter().zip(normal).map(|(a, b)| a * b).sum::<f64>();
                height >= -0.012 && distance(sub(d, mul(normal, height)), [0.; 3]) <= radius * 0.96
            });
        let contact = approach
            && surface.is_some_and(|(center, normal, _)| {
                sub(tip, center)
                    .into_iter()
                    .zip(normal)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    .abs()
                    <= 0.012
            });
        let end = if contact {
            add(
                tip,
                mul(sub(butt, tip), 0.02 / distance(butt, tip).max(0.02)),
            )
        } else {
            tip
        };
        let value = if approach {
            let (center, normal, radius) = surface.expect("approach surface");
            equipment::segment_surface_box_clearance(
                butt,
                end,
                0.006,
                body,
                &equipment::Surface {
                    center,
                    normal,
                    radius,
                },
            )
        } else {
            equipment::segment_box_clearance(butt, end, 0.006, body)
        };
        minimum = minimum.min(value);
    }
    for pedal in &ctx.geometry.pedals {
        for &(a, b, r) in arm.iter().chain(std::iter::once(&(butt, tip, 0.006))) {
            minimum = minimum.min(equipment::segment_box_clearance(a, b, r, pedal));
        }
    }
    for rod in &ctx.geometry.supports {
        for &(a, b, r) in arm.iter().chain(std::iter::once(&(butt, tip, 0.006))) {
            minimum =
                minimum.min(clearance::segment_distance(a, b, rod.start, rod.end) - r - rod.radius);
        }
    }
    minimum
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
    let mut best = (f64::INFINITY, 0., 0.);
    let lifts = if impact {
        &[0.][..]
    } else {
        &[0., 0.025, 0.05, 0.09, 0.14, 0.20][..]
    };
    for &lift in lifts {
        for delta in PITCH_OFFSETS {
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
                    + delta.abs() * 0.005;
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
                unresolved_piece_ids: Vec::new(),
            },
        };
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        ctx.geometry.bodies.clear();
        ctx.geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "stand".into(),
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
