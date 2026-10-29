//! Whole-tool/equipment queries shared by path preparation and elbow planning.
//! Conservative measured boxes and rendered support capsules, not mesh collision.
use crate::{
    clearance, contact, equipment,
    model::*,
    motion::{add, distance, sub},
    rig::{rotate, HandSolution, RigProfile},
};
use std::sync::Arc;

#[path = "radial_clearance.rs"]
mod radial_clearance;

pub const PITCH_OFFSETS: [f64; 9] = [0., -0.15, 0.15, -0.30, 0.30, -0.45, 0.45, -0.60, 0.60];
pub(crate) const LIFT_RATE: f64 = 1.2;
pub(crate) const MAX_LIFT: f64 = 0.20;
pub(crate) const PITCH_RATE: f64 = 2.0;

pub struct EquipmentContext {
    pub kit: Vec<KitPiece>,
    pub geometry: equipment::EquipmentGeometry,
    surfaces: std::collections::HashMap<String, equipment::Surface>,
}
impl EquipmentContext {
    pub fn new(kit: Vec<KitPiece>, geometry: equipment::EquipmentGeometry) -> Self {
        let surfaces = kit
            .iter()
            .map(|p| (p.id.clone(), equipment::surface(p)))
            .collect();
        Self {
            kit,
            geometry,
            surfaces,
        }
    }
    pub fn for_kit(kit: &[KitPiece]) -> Option<Arc<Self>> {
        kit.iter()
            .any(|p| p.geometry.is_some())
            .then(|| Arc::new(Self::new(kit.to_vec(), equipment::kit_geometry(kit))))
    }
}

/// Physical equipment gate shared by static contacts, preparation and graph
/// nodes. It participates in selection, before a single hand witness is kept.
#[derive(Clone, Copy, Default)]
pub(crate) struct HandEnvironment<'a> {
    pub equipment: Option<&'a EquipmentContext>,
    pub intended_piece: Option<&'a str>,
}
impl HandEnvironment<'_> {
    pub(crate) fn solve(
        self,
        profile: &RigProfile,
        limb: usize,
        tip: Vec3,
        pitch: f64,
        swivel: f64,
        preserve_arm: bool,
    ) -> Option<HandSolution> {
        let Some(equipment) = self.equipment else {
            return if preserve_arm {
                profile.solve_hand_preserving_arm_with_swivel(limb, tip, pitch, swivel)
            } else {
                profile.solve_hand_with_swivel(limb, tip, pitch, swivel)
            };
        };
        profile.solve_hand_with_swivel_where(limb, tip, pitch, swivel, preserve_arm, &|hand| {
            tool_clearance(profile, limb, hand, equipment, self.intended_piece)
                >= crate::posture::MARGIN
        })
    }
}
fn intended_body<'a>(ctx: &'a EquipmentContext, intended_piece: Option<&str>) -> Option<&'a str> {
    let piece = ctx
        .kit
        .iter()
        .find(|p| Some(p.id.as_str()) == intended_piece)?;
    if piece.instrument == Instrument::OpenHat {
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
}

/// Preserve a raw underpass when adding the scalar vertical correction. A
/// separate overhead body's vertical entry bounds the lift; an intended playing
/// surface or an already-overlapping raw point does not invent an underpass.
/// This preparation guide never replaces the complete-shaft feasibility query.
pub fn lift_ceiling(ctx: &EquipmentContext, tip: Vec3, intended_piece: Option<&str>) -> f64 {
    let intended = intended_body(ctx, intended_piece);
    let margin =
        crate::stick_geometry::profile().terminal.last().unwrap()[1] + crate::posture::MARGIN;
    let mut ceiling = MAX_LIFT;
    for body in &ctx.geometry.bodies {
        if intended == Some(body.piece_id.as_str()) {
            continue;
        }
        let inverse = body.rotation.map(|q| [-q[0], -q[1], -q[2], q[3]]);
        let point = inverse.map_or(sub(tip, body.center), |q| rotate(q, sub(tip, body.center)));
        let up = inverse.map_or([0., 1., 0.], |q| rotate(q, [0., 1., 0.]));
        let interval = |expand: f64| {
            let (mut enter, mut leave) = (f64::NEG_INFINITY, f64::INFINITY);
            for axis in 0..3 {
                let extent = body.half_extents[axis] + expand;
                if up[axis].abs() < 1e-12 {
                    if point[axis].abs() > extent {
                        return None;
                    }
                } else {
                    let a = (-extent - point[axis]) / up[axis];
                    let b = (extent - point[axis]) / up[axis];
                    enter = enter.max(a.min(b));
                    leave = leave.min(a.max(b));
                }
            }
            (enter <= leave).then_some((enter, leave))
        };
        // Include the tool radius before its centre enters the footprint. A
        // raw point inside the physical body still needs an escape, rather
        // than being classified as an already-valid underpass.
        if let Some((enter, leave)) = interval(margin) {
            if leave > 0. && (enter >= 0. || interval(0.).is_some_and(|(start, _)| start >= 0.)) {
                ceiling = ceiling.min(enter.max(0.));
            }
        }
    }
    ceiling
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
    if let Some(profile) = &surface.radial_profile {
        return radial_clearance::interval(start, end, radii, body, profile);
    }
    // The measured box axes need not be world axes. Clip its six support
    // planes and the playing-surface plane in the same rigid local frame.
    let (start, end, center, surface_center, normal) = if let Some(q) = body.rotation {
        let inverse = [-q[0], -q[1], -q[2], q[3]];
        (
            rotate(inverse, sub(start, body.center)),
            rotate(inverse, sub(end, body.center)),
            [0.; 3],
            rotate(inverse, sub(surface.center, body.center)),
            rotate(inverse, surface.normal),
        )
    } else {
        (start, end, body.center, surface.center, surface.normal)
    };
    let mut lines = [(0.0, 0.0); 7];
    for axis in 0..3 {
        for side in 0..2 {
            let sign = if side == 0 { -1.0 } else { 1.0 };
            lines[axis * 2 + side] = (
                sign * (start[axis] - center[axis]) - body.half_extents[axis] - radii[0],
                sign * (end[axis] - start[axis]) - (radii[1] - radii[0]),
            );
        }
    }
    let dot = |a: Vec3, b: Vec3| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    lines[6] = (
        dot(sub(start, surface_center), normal) - radii[0],
        dot(sub(end, start), normal) - (radii[1] - radii[0]),
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
fn support_minimum<const N: usize>(
    segments: [(Vec3, Vec3, f64); N],
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

/// Equipment intersections have separate semantics: exact tool geometry is a
/// feasibility constraint; estimated skin/clothing volumes are advisory.
#[derive(Clone, Copy, Debug)]
pub struct HandClearance {
    /// Feasibility score: intended tangent surfaces include the planner margin.
    pub tool: f64,
    /// Signed physical bound, with no added free-obstacle margin.
    pub physical_tool: f64,
    pub body: f64,
}
impl HandClearance {
    pub fn minimum(self) -> f64 {
        self.tool.min(self.body)
    }
}
/// Regression helper for the complete raw inspection metric.
#[cfg(test)]
fn hand_clearance(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
    intended_piece: Option<&str>,
) -> f64 {
    assess_hand(profile, limb, hand, ctx, intended_piece).minimum()
}
pub fn assess_hand(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
    intended_piece: Option<&str>,
) -> HandClearance {
    let mut result = query_tool(profile, limb, hand, ctx, intended_piece);
    result.body = body_clearance(profile, limb, hand, ctx);
    result
}
/// The full tapered shaft, including the butt, against all drum hardware.
/// Body thickness cannot change this result or grant a contact exception.
pub fn tool_clearance(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
    intended_piece: Option<&str>,
) -> f64 {
    query_tool(profile, limb, hand, ctx, intended_piece).tool
}
fn body_clearance(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
) -> f64 {
    let radii = clearance::arm_radii(profile, limb);
    let (hs, he, hr) = clearance::hand_volume(profile, limb, hand);
    let arm = [
        (profile.roots[limb], hand.middle, radii[0]),
        (hand.middle, hand.end, radii[1]),
        (hs, he, hr),
    ];
    let mut minimum = f64::INFINITY;
    for body in ctx.geometry.bodies.iter().chain(&ctx.geometry.pedals) {
        for &(a, b, r) in &arm {
            minimum = minimum.min(equipment::segment_box_clearance(a, b, r, body));
        }
    }
    support_minimum(arm, &ctx.geometry.supports, minimum)
}
fn query_tool(
    profile: &RigProfile,
    limb: usize,
    hand: &HandSolution,
    ctx: &EquipmentContext,
    intended_piece: Option<&str>,
) -> HandClearance {
    let (butt, tip) = stick_segment(profile, limb, hand);
    let shaft_radius = crate::stick_geometry::profile()
        .terminal
        .last()
        .expect("tool profile")[1];
    let intended_body = intended_body(ctx, intended_piece);
    let mut minimum = f64::INFINITY;
    let mut physical_minimum = f64::INFINITY;
    for body in &ctx.geometry.bodies {
        // An intended striking surface permits physical tangent contact, while
        // every unrelated body retains the free-obstacle margin. Test the
        // authored tapered tool, including its butt. Only the existing 20 mm
        // terminal contact region is exempt for an exact intended impact.
        // Contact exceptions are target-specific, but physical shape is not.
        // An unrelated cymbal retains its measured bow rather than becoming a
        // solid bounding box when a hand transitions to another instrument.
        let piece = if intended_body == Some(body.piece_id.as_str()) {
            ctx.kit.iter().find(|p| p.id == body.piece_id)
        } else {
            None
        };
        let surface = piece.map(|piece| {
            let (center, normal) = contact::project_surface(piece, tip);
            (center, normal)
        });
        let approach = intended_body == Some(body.piece_id.as_str())
            && surface.is_some_and(|(point, normal)| {
                let actual_height = sub(tip, point)
                    .into_iter()
                    .zip(normal)
                    .map(|(a, b)| a * b)
                    .sum::<f64>();
                actual_height >= -0.000_000_1
                    && contact::inside_playable_region(piece.expect("surface owner"), tip, 0.96, 0.)
            });
        let physical = if approach {
            let (center, normal) = surface.expect("approach surface");
            let contact_height = sub(tip, center)
                .iter()
                .zip(normal)
                .map(|(a, b)| a * b)
                .sum::<f64>();
            let clear = tapered_surface_clearance(
                butt,
                tip,
                body,
                ctx.surfaces
                    .get(&body.piece_id)
                    .expect("approach surface owner"),
                contact_height.abs() <= 0.012,
            );
            // The 0.1 nm numerical allowance handles a tangent point represented
            // through different coordinate transforms; it cannot hide a mesh gap.
            if clear.abs() < 1e-10 {
                0.0
            } else {
                clear
            }
        } else if let Some(surface) = ctx
            .surfaces
            .get(&body.piece_id)
            .filter(|surface| surface.radial_profile.is_some())
        {
            // No terminal exemption or contact margin on any unrelated body.
            // Keep the complete shaft's maximum radius, including the tip.
            // Flat heads keep their full box: their rim may protrude above it.
            tapered_surface_interval(butt, tip, [shaft_radius; 2], body, surface)
        } else {
            equipment::segment_box_clearance(butt, tip, shaft_radius, body)
        };
        physical_minimum = physical_minimum.min(physical);
        minimum = minimum.min(
            physical
                + if approach {
                    crate::posture::MARGIN
                } else {
                    0.0
                },
        );
    }
    for pedal in &ctx.geometry.pedals {
        let physical = equipment::segment_box_clearance(butt, tip, shaft_radius, pedal);
        physical_minimum = physical_minimum.min(physical);
        minimum = minimum.min(physical);
    }
    // Physical minima cannot exceed the feasibility minimum. A single support
    // query against the latter therefore preserves both exact minima.
    let tool = support_minimum([(butt, tip, shaft_radius)], &ctx.geometry.supports, minimum);
    HandClearance {
        tool,
        physical_tool: physical_minimum.min(tool),
        body: f64::INFINITY,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSample {
    pub time: f64,
    pub lift: [f64; 2],
    pub pitch: [f64; 2],
}
#[cfg(test)]
fn limit_rates(track: &mut [PathSample], impacts: &[Vec<f64>; 2]) {
    limit_rates_with_caps(track, impacts, &[]);
}
pub fn limit_rates_with_caps(
    track: &mut [PathSample],
    impacts: &[Vec<f64>; 2],
    ceilings: &[[f64; 2]],
) {
    assert!(ceilings.is_empty() || ceilings.len() == track.len());
    // First cap the requested lift by the distance to immutable zero-lift
    // contacts. Then extend preparation into neighbouring time samples instead
    // of flattening a necessary obstacle-clearing peak into its previous zero.
    for (limb, contacts) in impacts.iter().enumerate() {
        let mut caps: Vec<f64> = track
            .iter()
            .enumerate()
            .map(|(index, s)| {
                if contacts.binary_search_by(|t| t.total_cmp(&s.time)).is_ok() {
                    0.0
                } else {
                    ceilings
                        .get(index)
                        .map_or(MAX_LIFT, |cap| cap[limb].clamp(0., MAX_LIFT))
                }
            })
            .collect();
        // Only immutable impact knots are hard zeros here; raised
        // neighbours are propagated below within the same finite rate budget.
        for i in 1..caps.len() {
            caps[i] = caps[i].min(caps[i - 1] + LIFT_RATE * (track[i].time - track[i - 1].time));
        }
        for i in (0..caps.len().saturating_sub(1)).rev() {
            caps[i] = caps[i].min(caps[i + 1] + LIFT_RATE * (track[i + 1].time - track[i].time));
        }
        for (s, cap) in track.iter_mut().zip(&caps) {
            s.lift[limb] = s.lift[limb].min(*cap);
        }
        for i in 1..track.len() {
            track[i].lift[limb] = track[i].lift[limb]
                .max(track[i - 1].lift[limb] - LIFT_RATE * (track[i].time - track[i - 1].time))
                .min(caps[i]);
        }
        for i in (0..track.len().saturating_sub(1)).rev() {
            track[i].lift[limb] = track[i].lift[limb]
                .max(track[i + 1].lift[limb] - LIFT_RATE * (track[i + 1].time - track[i].time))
                .min(caps[i]);
        }
        for i in 1..track.len() {
            let dt = track[i].time - track[i - 1].time;
            track[i].pitch[limb] = track[i].pitch[limb].clamp(
                track[i - 1].pitch[limb] - PITCH_RATE * dt,
                track[i - 1].pitch[limb] + PITCH_RATE * dt,
            );
        }
        for i in (0..track.len().saturating_sub(1)).rev() {
            let dt = track[i + 1].time - track[i].time;
            track[i].pitch[limb] = track[i].pitch[limb].clamp(
                track[i + 1].pitch[limb] - PITCH_RATE * dt,
                track[i + 1].pitch[limb] + PITCH_RATE * dt,
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
        &[0., 0.025, 0.05, 0.09, 0.14, MAX_LIFT][..]
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
            let fixed = lift * 0.1 + delta.abs() * 0.005;
            let mut consider = |hand: HandSolution, need_physical: bool| {
                let base = fixed + crate::posture::comfort_cost(profile, limb, target, &hand);
                if PRUNE && base >= best.0 {
                    // Whether a coarse physical witness exists determines the
                    // refined search domain. Preserve that decision even when
                    // this candidate cannot win; omit expensive arm/body work.
                    let safe = need_physical
                        && tool_clearance(profile, limb, &hand, ctx, intended_piece)
                            >= crate::posture::MARGIN;
                    return (safe, false);
                }
                let clear = tool_clearance(profile, limb, &hand, ctx, intended_piece);
                let penalty = (crate::posture::MARGIN - clear).max(0.);
                let hard_cost =
                    penalty * penalty * 10_000. + if penalty > 0. { 1. } else { 0. } + base;
                // Advisory body penalties are nonnegative. Measure them only
                // when the exact shaft and comfort score can still win.
                if PRUNE && hard_cost >= best.0 {
                    return (penalty == 0., false);
                }
                let body = body_clearance(profile, limb, &hand, ctx)
                    .min(clearance::hand_clearance(profile, limb, &hand));
                let cost = hard_cost + clearance::body_penalty(body);
                if cost < best.0 {
                    best = (cost, lift, delta);
                }
                (penalty == 0., cost == 0.)
            };
            let mut coarse_safe = false;
            let environment = HandEnvironment {
                equipment: Some(ctx),
                intended_piece,
            };
            for hand in crate::posture::hand_candidates_in(
                profile,
                limb,
                target,
                pitch + delta,
                crate::posture::HandSearch::Full,
                environment,
            ) {
                let (safe, zero) = consider(hand, !coarse_safe);
                coarse_safe |= safe;
                if zero {
                    return (0., 0.);
                }
            }
            // Wrist-first solving can change shaft heading with swivel. An
            // anatomical coarse pose alone cannot suppress finer tool witnesses.
            if !coarse_safe {
                for hand in crate::posture::refined_hand_candidates_in(
                    profile,
                    limb,
                    target,
                    pitch + delta,
                    crate::posture::HandSearch::Full,
                    environment,
                ) {
                    if consider(hand, false).1 {
                        return (0., 0.);
                    }
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
    fn equipment_gate_retains_an_oblique_approach_hidden_by_the_first_hand() {
        // Authored native rig and a 25-degree procedural head. The ordinary
        // fixed-arm inward grip clips the near rim by a fraction of a mm;
        // existing bounded orientations at the same pitch/swivel can clear it.
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        let profile = profile.turned(-0.06);
        let mut drum: KitPiece = serde_json::from_value(
            fixture["kit"]
                .as_array()
                .unwrap()
                .iter()
                .find(|piece| piece["id"] == "tomHigh")
                .unwrap()
                .clone(),
        )
        .unwrap();
        drum.position = [-0.11, 1.105, 0.91];
        let angle = 25_f64.to_radians();
        let rotation = [-(angle * 0.5).sin(), 0., 0., (angle * 0.5).cos()];
        drum.geometry = Some(equipment::PieceGeometry {
            version: 1,
            reference_radius: drum.radius,
            source: "authored-test".into(),
            body: equipment::LocalBox {
                center: [0., -0.10241277993514153, 0.04775586357669892],
                half_extents: [0.1659595082491636, 0.11700000021606699, 0.1663999996125698],
                rotation: Some(rotation),
            },
            surface: equipment::Surface {
                center: [0., 0.0007882807373340572, 0.0016904730469629037],
                normal: [0., angle.cos(), -angle.sin()],
                radius: 0.1454000025987624,
                radial_profile: None,
                motion: None,
            },
        });
        let target = add(
            drum.position,
            drum.geometry.as_ref().unwrap().surface.center,
        );
        let mut geometry = equipment::kit_geometry(std::slice::from_ref(&drum));
        geometry.supports.clear();
        let ctx = EquipmentContext::new(vec![drum], geometry);
        let old = profile
            .solve_hand_with_swivel(0, target, -0.45, -0.2)
            .unwrap();
        assert!(crate::posture::anatomical(&profile, 0, target, &old));
        assert!(tool_clearance(&profile, 0, &old, &ctx, Some("tomHigh")) < crate::posture::MARGIN);
        let hand = HandEnvironment {
            equipment: Some(&ctx),
            intended_piece: Some("tomHigh"),
        }
        .solve(&profile, 0, target, -0.45, -0.2, false)
        .unwrap();
        assert!(crate::posture::anatomical(&profile, 0, target, &hand));
        assert!(profile.hand_orientation_policy_valid(0, hand.rotation));
        assert!(
            tool_clearance(&profile, 0, &hand, &ctx, Some("tomHigh")) >= crate::posture::MARGIN
        );
        assert!(distance(add(hand.end, profile.offset(0, hand.rotation)), target) < 1e-10);
        let control = profile.encode_hand_orientation(0, target, -0.45, -0.2, hand.rotation);
        let replay = profile
            .solve_hand_with_swivel_and_orientation(0, target, -0.45, -0.2, control)
            .unwrap();
        assert!(distance(hand.middle, replay.middle) < 1e-10);
        assert!(distance(hand.end, replay.end) < 1e-10);
        assert!(distance(hand.tip, replay.tip) < 1e-10);
        // An empty environment must preserve the original ordinary selection.
        let empty = EquipmentContext::new(vec![], equipment::kit_geometry(&[]));
        let unchanged = HandEnvironment {
            equipment: Some(&empty),
            intended_piece: None,
        }
        .solve(&profile, 0, target, -0.45, -0.2, false)
        .unwrap();
        assert_eq!(unchanged.rotation, old.rotation);
        assert_eq!(unchanged.middle, old.middle);
        assert_eq!(unchanged.end, old.end);
        // Exhausting the bounded family cannot turn an impossible tool path
        // into a successful contact or replace the real physical tool marker.
        let mut blocked = ctx;
        blocked.geometry.bodies[0].half_extents = [10.; 3];
        let rejected = HandEnvironment {
            equipment: Some(&blocked),
            intended_piece: None,
        }
        .solve(&profile, 0, target, -0.45, -0.2, false)
        .unwrap();
        assert!(tool_clearance(&profile, 0, &rejected, &blocked, None) < crate::posture::MARGIN);
        assert!(crate::contact_feasibility::at_pitch(
            &profile,
            0,
            target,
            -0.45,
            Some(&blocked),
            None
        )
        .is_none());
    }

    #[test]
    fn underpass_lift_cap_preserves_contacts_and_rotated_overhead_bodies() {
        let mut geometry = equipment::kit_geometry(&[]);
        geometry.bodies.push(equipment::EquipmentBox {
            piece_id: "crash".into(),
            center: [0., 1.2, 0.],
            half_extents: [0.2, 0.1, 0.2],
            rotation: None,
            kind: "body",
        });
        let mut ctx = EquipmentContext::new(default_kit(), geometry);
        let tip = [0., 0.98, 0.];
        assert!((lift_ceiling(&ctx, tip, None) - 0.112).abs() < 1e-12);
        assert!((lift_ceiling(&ctx, [0.204, 0.98, 0.], None) - 0.112).abs() < 1e-12);
        assert_eq!(lift_ceiling(&ctx, tip, Some("crash")), MAX_LIFT);
        assert!(lift_ceiling(&ctx, tip, Some("snare")) < MAX_LIFT);
        for point in [[0.3, 0.98, 0.], [0., 1.35, 0.], [0., 1.2, 0.]] {
            assert_eq!(lift_ceiling(&ctx, point, None), MAX_LIFT);
        }
        let angle = std::f64::consts::FRAC_PI_4;
        ctx.geometry.bodies[0].rotation = Some([(angle / 2.).sin(), 0., 0., (angle / 2.).cos()]);
        let cap = lift_ceiling(&ctx, tip, None);
        assert!(cap > 0. && cap < MAX_LIFT);
        assert!(
            equipment::segment_box_clearance(
                tip,
                add(tip, [0., cap, 0.]),
                0.006,
                &ctx.geometry.bodies[0],
            ) >= crate::posture::MARGIN - 1e-10
        );
    }

    #[test]
    fn underpass_caps_propagate_before_lift_rate_smoothing() {
        let mut path: Vec<_> = (0..=8)
            .map(|i| PathSample {
                time: i as f64 * 0.025,
                lift: [0.15; 2],
                pitch: [0.; 2],
            })
            .collect();
        path[0].lift = [0.; 2];
        path[8].lift = [0.; 2];
        let mut caps = vec![[MAX_LIFT; 2]; path.len()];
        caps[4][0] = 0.04;
        limit_rates_with_caps(&mut path, &[vec![0., 0.2], vec![0., 0.2]], &caps);
        assert_eq!(path[0].lift, [0.; 2]);
        assert_eq!(path[8].lift, [0.; 2]);
        assert!(path[4].lift[0] <= 0.04);
        assert!(path[3].lift[0] <= 0.04 + LIFT_RATE * 0.025 + 1e-12);
        assert!(path[5].lift[0] <= 0.04 + LIFT_RATE * 0.025 + 1e-12);
        for pair in path.windows(2) {
            for limb in 0..2 {
                assert!(
                    (pair[1].lift[limb] - pair[0].lift[limb]).abs()
                        <= LIFT_RATE * (pair[1].time - pair[0].time) + 1e-12
                );
            }
        }
    }

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
    fn body_envelopes_are_reported_without_changing_the_hard_tool_query() {
        let mut profile = RigProfile::primitive();
        profile.grip_offsets[0] = [0.; 3];
        let hand = HandSolution {
            middle: [-0.3, 1.1, -0.2],
            end: [0., 1., 0.],
            tip: [0., 1., 0.5],
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        let mut ctx = EquipmentContext::new(
            vec![],
            equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: "near-arm".into(),
                    center: hand.middle,
                    half_extents: [0.03; 3],
                    rotation: None,
                    kind: "body",
                }],
                supports: vec![],
                pedals: vec![],
                mechanisms: vec![],
                unresolved_piece_ids: vec![],
            },
        );
        let before = tool_clearance(&profile, 0, &hand, &ctx, None);
        assert!(before > crate::posture::MARGIN);
        assert!(hand_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        // A large garment estimate cannot reject a tool that is physically clear.
        profile.body_geometry = Some(
            serde_json::from_value(serde_json::json!({
                "version":1, "source":"mesh-estimate", "sampleCount":100,
                "torso":{"center":[0.,1.,0.],"halfExtents":[0.4,0.4,0.4]},
                "armRadii":[[0.2,0.2],[0.2,0.2]],"handRadii":[0.2,0.2]
            }))
            .unwrap(),
        );
        let assessment = assess_hand(&profile, 0, &hand, &ctx, None);
        assert_eq!(assessment.tool, before);
        assert!(assessment.body <= -0.19);
        assert_eq!(
            assessment.minimum(),
            hand_clearance(&profile, 0, &hand, &ctx, None)
        );
        // The same policy must still reject a drum crossing the shaft midpoint.
        ctx.geometry.bodies[0].center = [0., 1., 0.25];
        assert!(tool_clearance(&profile, 0, &hand, &ctx, None) < 0.);
    }

    #[test]
    fn unrelated_cymbal_uses_its_bow_but_never_a_contact_exception() {
        let mut cymbal = default_kit()
            .into_iter()
            .find(|p| p.instrument == Instrument::Crash)
            .unwrap();
        cymbal.position = [0.; 3];
        cymbal.radius = 0.25;
        let mut shape = equipment::local_geometry(&cymbal);
        shape.reference_radius = 0.25;
        shape.body = equipment::LocalBox {
            center: [0., -0.02, 0.],
            half_extents: [0.25, 0.05, 0.25],
            rotation: None,
        };
        shape.surface = equipment::Surface {
            center: [0.; 3],
            normal: [0., 1., 0.],
            radius: 0.25,
            motion: None,
            radial_profile: Some(equipment::RadialSurfaceProfile {
                bell_radius: Some(0.06),
                origin: [0.; 3],
                rotation: [0., 0., 0., 1.],
                points: vec![
                    [0., 0.],
                    [0.03, 0.02],
                    [0.06, -0.005],
                    [0.20, -0.025],
                    [0.25, -0.03],
                ],
            }),
        };
        cymbal.geometry = Some(shape);
        let body = equipment::body(&cymbal).unwrap();
        let ctx = EquipmentContext::new(
            vec![cymbal],
            equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![body.clone()],
                supports: vec![],
                pedals: vec![],
                mechanisms: vec![],
                unresolved_piece_ids: vec![],
            },
        );
        let mut profile = RigProfile::primitive();
        profile.grip_offsets[0] = [0.; 3];
        let mut hand = HandSolution {
            middle: [0.1, 0.2, -0.5],
            end: [0.1, 0.015, -0.23],
            tip: [0.1, 0.015, 0.23],
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        let (butt, tip) = stick_segment(&profile, 0, &hand);
        assert!(equipment::segment_box_clearance(butt, tip, 0.006, &body) < 0.);
        let clear = assess_hand(&profile, 0, &hand, &ctx, None);
        assert!(
            clear.physical_tool > 0.,
            "the shaft is above the curved bow"
        );
        assert_eq!(clear.physical_tool, clear.tool, "no target margin");
        hand.end[0] = 0.;
        hand.tip[0] = 0.;
        assert!(
            tool_clearance(&profile, 0, &hand, &ctx, None) < 0.,
            "the raised bell still blocks the middle of the shaft"
        );
        hand.end[0] = 0.1;
        hand.tip[0] = 0.1;
        hand.end[1] = -0.03;
        hand.tip[1] = -0.03;
        assert!(
            tool_clearance(&profile, 0, &hand, &ctx, None) < 0.,
            "lowering the same shaft crosses the bow"
        );
    }

    #[test]
    fn intended_radial_approach_uses_the_same_tilted_playable_region() {
        let mut profile = RigProfile::primitive();
        profile.grip_offsets[0] = [0.; 3];
        let mut expected: Option<(f64, f64)> = None;
        for degrees in [0_f64, 15., 20., 25.] {
            let half_angle = degrees.to_radians() / 2.;
            let rotation = [half_angle.sin(), 0., 0., half_angle.cos()];
            let origin = [0.3, 0.9, 0.7];
            let transform = |point| add(origin, rotate(rotation, point));
            let mut cymbal = default_kit()
                .into_iter()
                .find(|p| p.instrument == Instrument::Crash)
                .unwrap();
            cymbal.position = origin;
            cymbal.radius = 0.25;
            let mut shape = equipment::local_geometry(&cymbal);
            shape.reference_radius = 0.25;
            shape.body = equipment::LocalBox {
                center: rotate(rotation, [0., -0.02, 0.]),
                half_extents: [0.25, 0.05, 0.25],
                rotation: Some(rotation),
            };
            shape.surface = equipment::Surface {
                center: [0.; 3],
                normal: rotate(rotation, [0., 1., 0.]),
                // A legacy sampled plane must not shrink the authored profile.
                radius: 0.18,
                motion: None,
                radial_profile: Some(equipment::RadialSurfaceProfile {
                    origin: [0.; 3],
                    rotation,
                    bell_radius: None,
                    points: vec![[0., 0.], [0.25, 0.]],
                }),
            };
            cymbal.geometry = Some(shape);
            let ctx = EquipmentContext::new(
                vec![cymbal.clone()],
                equipment::EquipmentGeometry {
                    version: 1,
                    bodies: vec![equipment::body(&cymbal).unwrap()],
                    supports: vec![],
                    pedals: vec![],
                    mechanisms: vec![],
                    unresolved_piece_ids: vec![],
                },
            );
            let hand = |fraction: f64| HandSolution {
                middle: transform([0., 0.3, -0.5]),
                end: transform([0., 0.12, -0.5]),
                tip: transform([0., 0., -0.25 * fraction]),
                rotation,
                reachable: true,
            };
            let intended = assess_hand(&profile, 0, &hand(0.88), &ctx, Some(&cymbal.id));
            assert!(intended.tool >= crate::posture::MARGIN);
            if let Some((tool, physical)) = expected {
                assert!((intended.tool - tool).abs() < 1e-10);
                assert!((intended.physical_tool - physical).abs() < 1e-10);
            } else {
                expected = Some((intended.tool, intended.physical_tool));
            }
            let unrelated = assess_hand(&profile, 0, &hand(0.88), &ctx, None);
            assert!(
                unrelated.tool < 0.,
                "no terminal exception on unrelated bodies"
            );
            let outside = assess_hand(&profile, 0, &hand(0.98), &ctx, Some(&cymbal.id));
            assert!(
                outside.tool < 0.,
                "the .96 region is not expanded to the rim"
            );
            let mut below = hand(0.88);
            below.tip = sub(below.tip, rotate(rotation, [0., 0.001, 0.]));
            assert!(assess_hand(&profile, 0, &below, &ctx, Some(&cymbal.id)).tool < 0.);
        }
    }

    #[test]
    fn physical_tool_inspection_does_not_hide_penetration_under_the_contact_margin() {
        let mut profile = RigProfile::primitive();
        profile.grip_offsets[0] = [0.; 3];
        let hand = HandSolution {
            middle: [-0.2, 1.1, -0.2],
            end: [0., 1.06, 0.],
            tip: [0., 1., 0.5],
            rotation: [0., 0., 0., 1.],
            reachable: true,
        };
        let mut drum = default_kit()
            .into_iter()
            .find(|p| p.instrument == Instrument::TomHigh)
            .unwrap();
        drum.position = hand.tip;
        drum.radius = 0.10;
        drum.geometry = None;
        let ctx = EquipmentContext::new(
            vec![drum.clone()],
            equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: drum.id.clone(),
                    center: [0., 0.97, 0.5],
                    half_extents: [0.10, 0.03, 0.10],
                    rotation: None,
                    kind: "body",
                }],
                supports: vec![],
                pedals: vec![],
                mechanisms: vec![],
                unresolved_piece_ids: vec![],
            },
        );
        // A shallow shaft intersects the head outside the declared terminal
        // contact region. Its physical deficit is smaller than the 2 mm margin.
        let clear = assess_hand(&profile, 0, &hand, &ctx, Some(&drum.id));
        assert!(clear.physical_tool < 0. && clear.physical_tool > -crate::posture::MARGIN);
        assert!(clear.tool > 0. && clear.tool < crate::posture::MARGIN);
        assert!((clear.tool - clear.physical_tool - crate::posture::MARGIN).abs() < 1e-12);
        // An unrelated obstacle has no intended-contact adjustment; a global
        // subtraction would incorrectly inflate its reported penetration.
        let unrelated = assess_hand(&profile, 0, &hand, &ctx, None);
        assert_eq!(unrelated.physical_tool, unrelated.tool);
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
        let mut ctx = EquipmentContext::new(
            Vec::new(),
            equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: "other-tom".into(),
                    center: [0., 1., 0.25],
                    half_extents: [0.02, 0.02, 0.02],
                    rotation: None,
                    kind: "body",
                }],
                supports: Vec::new(),
                pedals: Vec::new(),
                mechanisms: Vec::new(),
                unresolved_piece_ids: Vec::new(),
            },
        );
        assert!(tool_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        ctx.geometry.bodies.clear();
        ctx.geometry.supports.push(equipment::EquipmentCapsule {
            piece_id: "stand".into(),
            role: "rod",
            start: [-0.1, 1., -0.08],
            end: [0.1, 1., -0.08],
            radius: 0.01,
        });
        assert!(tool_clearance(&profile, 0, &hand, &ctx, None) < 0.);
        ctx.geometry.supports[0].start[1] = 0.5;
        ctx.geometry.supports[0].end[1] = 0.5;
        assert!(tool_clearance(&profile, 0, &hand, &ctx, None) > 0.);
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
        let mut ctx = EquipmentContext::new(
            vec![hat.clone(), open, unrelated],
            equipment::EquipmentGeometry {
                version: 1,
                bodies: vec![equipment::EquipmentBox {
                    piece_id: hat.id.clone(),
                    center: [0.0, 0.97, 0.5],
                    half_extents: [0.10, 0.03, 0.10],
                    rotation: None,
                    kind: "body",
                }],
                supports: vec![],
                pedals: vec![],
                mechanisms: vec![],
                unresolved_piece_ids: vec![],
            },
        );
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
            rotation: None,
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
    fn tapered_surface_bound_follows_the_measured_oriented_box() {
        let body = equipment::EquipmentBox {
            piece_id: "tom".into(),
            center: [0., -0.08, 0.],
            half_extents: [0.17, 0.08, 0.11],
            rotation: None,
            kind: "body",
        };
        let surface = equipment::Surface {
            center: [0.; 3],
            normal: [0., 1., 0.],
            radius: 0.16,
            radial_profile: None,
            motion: None,
        };
        let angle = 0.73_f64;
        let axis = [0.48, 0.64, 0.6];
        let rotation = [
            axis[0] * (angle / 2.).sin(),
            axis[1] * (angle / 2.).sin(),
            axis[2] * (angle / 2.).sin(),
            (angle / 2.).cos(),
        ];
        let translation = [0.4, 1.1, -0.3];
        let transform = |point| add(rotate(rotation, point), translation);
        let mut moved_body = body.clone();
        moved_body.center = transform(body.center);
        moved_body.rotation = Some(rotation);
        let moved_surface = equipment::Surface {
            center: transform(surface.center),
            normal: rotate(rotation, surface.normal),
            ..surface.clone()
        };
        for (a, b, radii) in [
            ([0., 0.09, -0.35], [0., 0.003, 0.], [0.006, 0.003]),
            ([0., -0.025, -0.3], [0., -0.025, 0.3], [0.006, 0.006]),
            ([0.18, -0.05, -0.3], [0.18, -0.05, 0.3], [0.003, 0.006]),
        ] {
            let original = tapered_surface_interval(a, b, radii, &body, &surface);
            let moved = tapered_surface_interval(
                transform(a),
                transform(b),
                radii,
                &moved_body,
                &moved_surface,
            );
            assert!((original - moved).abs() < 1e-12, "{original} -> {moved}");
        }
    }
    #[test]
    fn bounded_search_preserves_exhaustive_winners_for_impacts_and_transfers() {
        let mut profile = RigProfile::primitive();
        for limb in 0..2 {
            profile.roots[limb] = [if limb == 0 { -0.19 } else { 0.19 }, 1.235, 0.06];
            profile.lengths[limb] = [0.3, 0.27];
        }
        let kit = default_kit();
        let geometry = equipment::kit_geometry(&kit);
        let ctx = EquipmentContext::new(kit, geometry);
        let mut non_neutral = 0;
        for bounded in [false, true] {
            profile.hand_frames = Some(std::array::from_fn(|limb| {
                let palm = if bounded {
                    [if limb == 0 { 1. } else { -1. }, 0., 0.]
                } else {
                    [0., 1., 0.]
                };
                crate::rig::HandControlFrame {
                    forearm: [0., 0., 1.],
                    palm,
                    desired_palm: palm,
                    palm_roll_range: bounded.then_some([-std::f64::consts::FRAC_PI_2, 0.]),
                    palm_reference: bounded.then_some(crate::rig::PalmReference::ShaftHeading),
                }
            }));
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
                    "{id}, limb {limb}, impact {impact}, intended {intended:?}, bounded {bounded}"
                );
                    non_neutral += usize::from(expected != (0.0, 0.0));
                    if impact {
                        assert_eq!(actual.0, 0.0);
                    }
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
