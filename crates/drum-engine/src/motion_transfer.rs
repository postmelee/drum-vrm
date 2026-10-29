//! Bounded clearance guidance for travel between distinct contact surfaces.
//! This shapes the raw tip path; the whole-tool equipment query remains the
//! authority. A tip arc alone does not certify the wrist, shaft or elbow route.
use super::{distance, Vec3, MAX_TIP_SPEED};
use crate::equipment::{box_world_half_extents, EquipmentBox};

const READY_CLEARANCE: f64 = 0.035;
const MAX_ARCH_HEIGHT: f64 = 0.20;
const CORRIDOR_MARGIN: f64 = 0.008;
const TOP_MARGIN: f64 = 0.012;
// max |d/du (16 u² (1-u)²)| = 16 / (3 sqrt(3)) < 3.08.
const ARCH_SLOPE_BOUND: f64 = 3.08;

fn in_box(point: Vec3, body: &EquipmentBox) -> bool {
    let extents = box_world_half_extents(body);
    (0..3).all(|axis| (point[axis] - body.center[axis]).abs() <= extents[axis] + CORRIDOR_MARGIN)
}

fn box_interval(a: Vec3, b: Vec3, body: &EquipmentBox) -> Option<(f64, f64)> {
    box_interval_bounds(a, b, body.center, box_world_half_extents(body))
}

fn box_interval_bounds(a: Vec3, b: Vec3, center: Vec3, half_extents: Vec3) -> Option<(f64, f64)> {
    let (mut enter, mut leave) = (0_f64, 1_f64);
    for axis in 0..3 {
        let delta = b[axis] - a[axis];
        let extent = half_extents[axis] + CORRIDOR_MARGIN;
        if delta.abs() < 1e-9 {
            if (a[axis] - center[axis]).abs() >= extent {
                return None;
            }
        } else {
            let near = (center[axis] - extent - a[axis]) / delta;
            let far = (center[axis] + extent - a[axis]) / delta;
            enter = enter.max(near.min(far));
            leave = leave.min(near.max(far));
        }
    }
    (enter < leave).then_some((enter, leave))
}

fn crosses_box(a: Vec3, b: Vec3, body: &EquipmentBox) -> bool {
    box_interval(a, b, body).is_some()
}

fn signed_box_distance(point: Vec3, center: Vec3, extents: Vec3) -> f64 {
    let q: Vec3 =
        std::array::from_fn(|i| (point[i] - center[i]).abs() - extents[i] - CORRIDOR_MARGIN);
    q.iter().map(|v| v.max(0.).powi(2)).sum::<f64>().sqrt()
        + q.into_iter().fold(f64::NEG_INFINITY, f64::max).min(0.)
}

/// Shoulder-radius preservation may expand a free chord into nearby equipment.
/// Keep only the free part of that optional correction, including the vertical
/// lift that will be applied next and the later bounded equipment lift. Reserve
/// that upward corridor without reading a path currently being prepared, which
/// would make raw travel depend recursively on its own correction. Existing
/// endpoint/route overlaps remain the whole-tool solver's responsibility.
pub(super) fn limit_projection(
    base: Vec3,
    projected: Vec3,
    lift: f64,
    reserve_lift: f64,
    bodies: &[EquipmentBox],
) -> Vec3 {
    let mut from = base;
    let mut to = projected;
    from[1] += lift;
    to[1] += lift;
    let mut amount = 1_f64;
    for body in bodies {
        let mut center = body.center;
        // Routing remains a conservative world-space guide. The final tool
        // queries use the measured oriented box, without these empty corners.
        let mut extents = box_world_half_extents(body);
        center[1] -= reserve_lift * 0.5;
        extents[1] += reserve_lift * 0.5;
        let start = signed_box_distance(from, center, extents);
        if start <= 0. {
            // A raw path can already enter a source/target body's bounding
            // envelope. Fade its optional correction by actual outward gain;
            // do not abruptly restore the entire shift at that boundary.
            let end = signed_box_distance(to, center, extents);
            let outward = if start < 0. {
                ((end - start) / -start).clamp(0., 1.)
            } else {
                f64::from(end >= 0.)
            };
            amount = amount.min(outward);
            continue;
        }
        if let Some((enter, _)) = box_interval_bounds(from, to, center, extents) {
            amount = amount.min(enter);
        }
    }
    super::lerp(base, projected, amount)
}

/// Optional complete tip route around the edge of a higher contact surface.
/// Selection depends only on immutable endpoints, duration and equipment; phase
/// merely evaluates the selected two-leg C1 path. It already contains clearance
/// travel, so callers must not add another arch or project its points afterward.
/// Endpoint boxes contain intended surface contacts; the final whole-tool query
/// still checks those surfaces, the shaft and every support without exemptions.
pub(super) fn clearance_waypoint(
    a: Vec3,
    b: Vec3,
    phase: f64,
    duration: f64,
    same_target: bool,
    bodies: &[EquipmentBox],
) -> Option<Vec3> {
    if same_target || (a[1] - b[1]).abs() < TOP_MARGIN {
        return None;
    }
    let (lower, upper) = if a[1] < b[1] { (a, b) } else { (b, a) };
    let mut best: Option<(f64, Vec3)> = None;
    for body in bodies {
        let extents = box_world_half_extents(body);
        let bottom = body.center[1] - extents[1];
        let top = body.center[1] + extents[1];
        if lower[1] >= bottom - TOP_MARGIN
            || upper[1] < bottom - TOP_MARGIN
            || upper[1] > top + TOP_MARGIN
            || [0, 2].into_iter().any(|axis| {
                (upper[axis] - body.center[axis]).abs() > extents[axis] + CORRIDOR_MARGIN
            })
        {
            continue;
        }
        let height = upper[1].max(top + TOP_MARGIN);
        if height - upper[1] > MAX_ARCH_HEIGHT {
            continue;
        }
        for axis in [0, 2] {
            for sign in [-1., 1.] {
                let edge = body.center[axis] + sign * (extents[axis] + CORRIDOR_MARGIN);
                let inside = (sign * (edge - lower[axis])).max(0.);
                // Leave the footprint before rising to its underside. This
                // also handles a floor tom sitting slightly beneath a ride.
                let offset = inside * (height - lower[1]) / (bottom - TOP_MARGIN - lower[1]);
                let mut corner = [lower[0], height, lower[2]];
                corner[axis] += sign * offset;
                let length = distance(a, corner) + distance(corner, b);
                if 1.5 * length > MAX_TIP_SPEED * duration
                    || best.is_some_and(|(old, _)| length >= old)
                    || bodies.iter().any(|obstacle| {
                        in_box(corner, obstacle)
                            || [(a, corner), (corner, b)].into_iter().any(|(from, to)| {
                                !in_box(from, obstacle)
                                    && !in_box(to, obstacle)
                                    && crosses_box(from, to, obstacle)
                            })
                    })
                {
                    continue;
                }
                best = Some((length, corner));
            }
        }
    }
    let (length, corner) = best?;
    let split = distance(a, corner) / length;
    let phase = phase.clamp(0., 1.);
    Some(if phase <= split {
        super::lerp(a, corner, super::smooth(phase / split))
    } else {
        super::lerp(corner, b, super::smooth((phase - split) / (1. - split)))
    })
}

/// Retain the authored stroke lift and add only a bounded, smooth travel arch.
/// The extra guide uses the remaining chord-speed budget. Later shoulder-radius
/// projection and equipment correction still require sampled velocity checks;
/// the planner's endpoint speed limit is not a full-trajectory certificate.
pub(super) fn lift(
    a: Vec3,
    b: Vec3,
    phase: f64,
    stroke_lift: f64,
    duration: f64,
    same_target: bool,
    bodies: &[EquipmentBox],
) -> f64 {
    let distinct = !same_target && (b[0] - a[0]).hypot(b[2] - a[2]) >= 1e-6;
    let mut height = a[1].max(b[1]) + READY_CLEARANCE;
    let mut crossing_amplitude = 0_f64;
    let mut ceiling = f64::INFINITY;
    for body in bodies {
        let extents = box_world_half_extents(body);
        let top = body.center[1] + extents[1];
        let bottom = body.center[1] - extents[1];
        // Only the tip's bounded travel corridor informs this guide. This is
        // not an obstacle exemption: the final whole-tool query keeps every
        // body, including equipment above this corridor and all support rods.
        if top < a[1].min(b[1]) || bottom > a[1].max(b[1]) + MAX_ARCH_HEIGHT {
            continue;
        }
        let mut enter = 0_f64;
        let mut leave = 1_f64;
        for axis in [0, 2] {
            let delta = b[axis] - a[axis];
            let extent = extents[axis] + CORRIDOR_MARGIN;
            if delta.abs() < 1e-9 {
                if (a[axis] - body.center[axis]).abs() > extent {
                    leave = -1.;
                }
            } else {
                let near = (body.center[axis] - extent - a[axis]) / delta;
                let far = (body.center[axis] + extent - a[axis]) / delta;
                enter = enter.max(near.min(far));
                leave = leave.min(near.max(far));
            }
        }
        if enter <= leave {
            // Keep a route that passes below an overhead surface. Test height
            // at this body's XZ entry/exit, not at remote route endpoints: a
            // high crash can descend below the other crash before reaching it.
            // Vaulting every nearby body instead lifts that safe chord into it.
            let corridor_height = [enter, leave]
                .map(|weight| a[1] + (b[1] - a[1]) * weight)
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max);
            let room = bottom - TOP_MARGIN - corridor_height;
            if room > 0. {
                ceiling = ceiling.min(room);
                continue;
            }
            height = height.max(top + TOP_MARGIN);
            // The tip follows smoothstep in XZ, not linear time. Clearing the
            // obstacle only at the arch's midpoint can still enter its near
            // wall below the head. Size the same bounded arch at both corridor
            // boundaries, converted back to stroke phase. Endpoint boundaries
            // remain exact source contacts; the whole-tool query still owns
            // acceptance there and throughout the path.
            for weight in [enter, leave] {
                if weight > 0. && weight < 1. {
                    let u = 0.5 - ((1. - 2. * weight).asin() / 3.).sin();
                    let arch = 16. * u.powi(2) * (1. - u).powi(2);
                    let y = a[1] + (b[1] - a[1]) * weight;
                    crossing_amplitude = crossing_amplitude.max((top + TOP_MARGIN - y) / arch);
                }
            }
        }
    }
    // Smoothstep's chord derivative is at most 1.5. Do not make an already
    // rapid transfer longer merely to reach an ideal clearance height.
    let budget = ((MAX_TIP_SPEED * duration - 1.5 * distance(a, b)).max(0.) / ARCH_SLOPE_BOUND)
        .min(MAX_ARCH_HEIGHT);
    let amplitude = (height - 0.5 * (a[1] + b[1]))
        .max(crossing_amplitude)
        .clamp(0., budget);
    let u = phase.clamp(0., 1.);
    let arch = 16. * u.powi(2) * (1. - u).powi(2);
    let requested = if distinct {
        stroke_lift.max(amplitude * arch)
    } else {
        stroke_lift
    };
    if ceiling.is_finite() {
        // Smooth compression preserves zero lift and does not introduce a hard
        // velocity corner when the rebound meets the corridor ceiling.
        ceiling * (requested / ceiling).tanh()
    } else {
        requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn body(center: Vec3, half_extents: Vec3) -> EquipmentBox {
        EquipmentBox {
            piece_id: "independent-obstacle".into(),
            center,
            half_extents,
            rotation: None,
            kind: "body",
        }
    }
    fn point(a: Vec3, b: Vec3, u: f64, duration: f64, bodies: &[EquipmentBox]) -> Vec3 {
        let mut p = super::super::lerp(a, b, super::super::smooth(u));
        p[1] += lift(a, b, u, 0., duration, false, bodies);
        p
    }
    #[test]
    fn oriented_bodies_keep_conservative_world_space_transfer_guides() {
        let q = [(0.4_f64).sin(), 0., 0., (0.4_f64).cos()];
        let mut oriented = body([0.1, 1.2, 0.3], [0.12, 0.04, 0.25]);
        oriented.rotation = Some(q);
        let bounds = body(oriented.center, box_world_half_extents(&oriented));
        // The broad guide encloses every oriented corner. It may discourage
        // an empty AABB corner, but cannot certify clearance inside real metal.
        for x in [-1., 1.] {
            for y in [-1., 1.] {
                for z in [-1., 1.] {
                    let local = [x * 0.12, y * 0.04, z * 0.25];
                    let point = super::super::add(crate::rig::rotate(q, local), oriented.center);
                    assert!(in_box(point, &oriented));
                }
            }
        }
        let a = [-0.4, 0.8, 0.2];
        let b = [0.4, 1.25, 0.35];
        for step in 0..=20 {
            let u = step as f64 / 20.;
            assert_eq!(
                lift(a, b, u, 0.04, 0.5, false, &[oriented.clone()]),
                lift(a, b, u, 0.04, 0.5, false, std::slice::from_ref(&bounds)),
            );
            assert_eq!(
                clearance_waypoint(a, b, u, 0.5, false, &[oriented.clone()]),
                clearance_waypoint(a, b, u, 0.5, false, std::slice::from_ref(&bounds)),
            );
        }
        assert_eq!(
            limit_projection(a, b, 0.1, 0.2, &[oriented]),
            limit_projection(a, b, 0.1, 0.2, &[bounds]),
        );
    }
    #[test]
    fn shoulder_projection_preserves_the_free_lifted_corridor() {
        let base = [-0.3, 1., 0.5];
        let projected = [-0.1, 1., 0.7];
        let bodies = [body([0., 1.1, 0.7], [0.2, 0.05, 0.2])];
        // Without the subsequent lift this optional correction is below the
        // obstacle. Its future lift is part of the actual clearance decision.
        assert_eq!(
            limit_projection(base, projected, 0., 0., &bodies),
            projected
        );
        let clipped = limit_projection(base, projected, 0.1, 0., &bodies);
        assert!((clipped[0] + 0.208).abs() < 1e-12);
        assert_eq!(clipped[1], base[1]);
        assert_eq!(limit_projection(base, projected, 0., 0.2, &bodies), clipped);
        assert_eq!(limit_projection(base, base, 0.1, 0.2, &bodies), base);
        let mut previous = base;
        for step in 1..=1000 {
            let candidate = super::super::lerp(base, projected, step as f64 / 1000.);
            let point = limit_projection(base, candidate, 0.1, 0., &bodies);
            assert!(distance(previous, point) <= distance(base, projected) / 1000. + 1e-12);
            previous = point;
        }
    }
    #[test]
    fn projection_fades_continuously_as_base_enters_and_leaves_a_reserved_body() {
        let bodies = [body([0., 1.1, 0.], [0.2, 0.05, 0.2])];
        for shift in [-0.1, 0.1] {
            let mut previous = None;
            for step in 0..=2000 {
                let x = -0.25 + step as f64 * 0.00005;
                let point = limit_projection([x, 1., 0.], [x + shift, 1., 0.], 0., 0.2, &bodies);
                if let Some(old) = previous {
                    assert!(
                        distance(old, point) < 0.001,
                        "projection jump: {old:?} -> {point:?}"
                    );
                }
                previous = Some(point);
            }
        }
    }
    #[test]
    fn single_corner_leaves_a_cymbal_footprint_before_rising_to_its_contact() {
        let a = [0., 0.8, 0.48];
        let b = [0., 1.2, 0.7];
        let bodies = [body([0., 1.15, 0.7], [0.3, 0.05, 0.25])];
        let duration = 0.25;
        let mut previous = a;
        for step in 0..=1000 {
            let u = step as f64 / 1000.;
            let point = clearance_waypoint(a, b, u, duration, false, &bodies).unwrap();
            let reverse = clearance_waypoint(b, a, 1. - u, duration, false, &bodies).unwrap();
            assert!(distance(point, reverse) < 1e-12);
            // The rendered flat upper surface is an intended contact only at
            // the endpoint. Interior travel must stay outside the solid plate.
            assert!(
                point[0].abs() >= 0.3
                    || (point[2] - 0.7).abs() >= 0.25
                    || point[1] <= 1.10
                    || point[1] >= 1.20 - 1e-12,
                "crossed cymbal: {point:?}"
            );
            if step > 0 {
                assert!(distance(previous, point) * 1000. / duration <= MAX_TIP_SPEED + 1e-8);
            }
            previous = point;
        }
        assert_eq!(
            clearance_waypoint(a, b, 0., duration, false, &bodies),
            Some(a)
        );
        assert_eq!(
            clearance_waypoint(a, b, 1., duration, false, &bodies),
            Some(b)
        );
        assert!(clearance_waypoint(a, b, 0.5, 0.01, false, &bodies).is_none());
        assert!(clearance_waypoint(a, b, 0.5, duration, true, &bodies).is_none());
    }
    #[test]
    fn clears_an_intervening_body_without_moving_either_contact() {
        let a = [-0.3, 1., 0.];
        let b = [0.3, 1., 0.];
        let bodies = [body([0., 0.94, 0.], [0.10, 0.10, 0.10])];
        assert_eq!(point(a, b, 0., 0.5, &bodies), a);
        assert_eq!(point(a, b, 1., 0.5, &bodies), b);
        for step in 0..=240 {
            let u = step as f64 / 240.;
            let p = point(a, b, u, 0.5, &bodies);
            if p[0].abs() <= 0.10 {
                assert!(p[1] > 1.042, "obstacle crossing at {p:?}");
            }
        }
    }
    #[test]
    fn high_to_low_transfer_recovers_above_source_before_descending() {
        let a = [-0.3, 1.2, 0.];
        let b = [0.3, 0.9, 0.];
        assert!(point(a, b, 0.2, 0.5, &[])[1] > a[1]);
        assert!(point(a, b, 0.5, 0.5, &[])[1] > a[1]);
        assert_eq!(point(a, b, 1., 0.5, &[]), b);
    }
    #[test]
    fn off_center_obstacle_is_cleared_when_the_tip_enters_its_corridor() {
        // The old midpoint-only arch passed the middle of this route but
        // struck the near wall of the obstacle in either travel direction.
        let endpoints = [[-0.3, 1., 0.], [0.3, 1., 0.]];
        let bodies = [body([-0.18, 1., 0.], [0.03, 0.05, 0.05])];
        for (a, b) in [(endpoints[0], endpoints[1]), (endpoints[1], endpoints[0])] {
            assert_eq!(point(a, b, 0., 0.5, &bodies), a);
            assert_eq!(point(a, b, 1., 0.5, &bodies), b);
            let mut crossed = false;
            for step in 0..=1000 {
                let p = point(a, b, step as f64 / 1000., 0.5, &bodies);
                if (p[0] + 0.18).abs() <= 0.03 + CORRIDOR_MARGIN {
                    crossed = true;
                    assert!(p[1] >= 1.05 + TOP_MARGIN - 1e-10, "wall crossing: {p:?}");
                }
            }
            assert!(crossed);
        }
    }
    #[test]
    fn overhead_surface_keeps_tom_transfer_and_same_surface_rebound_below_it() {
        let bodies = [body([0.15, 1.15, 0.], [0.25, 0.05, 0.25])];
        let a = [0., 1.03, 0.];
        let b = [0.3, 0.82, 0.];
        for (a, b, same) in [(a, b, false), (b, a, false), (a, a, true)] {
            for step in 0..=1000 {
                let u = step as f64 / 1000.;
                let rebound = 0.11 * (std::f64::consts::PI * u).sin();
                let clearance_lift = lift(a, b, u, rebound, 0.25, same, &bodies);
                let y = a[1] + (b[1] - a[1]) * super::super::smooth(u) + clearance_lift;
                assert!(y < 1.10 - TOP_MARGIN + 1e-10, "overhead crossing: {y}");
                assert!(clearance_lift >= 0.);
            }
            assert_eq!(lift(a, b, 0., 0., 0.25, same, &bodies), 0.);
            assert_eq!(lift(a, b, 1., 0., 0.25, same, &bodies), 0.);
        }
    }
    #[test]
    fn descending_crash_to_hat_keeps_the_underpass_reached_before_the_overhead_crash() {
        let a = [0.8, 1.4, 0.];
        let b = [-0.3, 0.8, 0.];
        let bodies = [body([-0.25, 1.1, 0.], [0.15, 0.05, 0.2])];
        for (a, b) in [(a, b), (b, a)] {
            for step in 0..=1000 {
                let u = step as f64 / 1000.;
                let mut point = super::super::lerp(a, b, super::super::smooth(u));
                point[1] += lift(
                    a,
                    b,
                    u,
                    0.11 * (std::f64::consts::PI * u).sin(),
                    0.5,
                    false,
                    &bodies,
                );
                if (point[0] + 0.25).abs() <= 0.15 + CORRIDOR_MARGIN {
                    assert!(point[1] < 1.05 - TOP_MARGIN, "overhead crossing: {point:?}");
                }
            }
        }
    }
    #[test]
    fn added_arch_obeys_its_duration_budget_and_preserves_same_surface_strokes() {
        let a = [-0.3, 1., 0.];
        let b = [0.3, 1., 0.];
        let bodies = [body([0., 1.05, 0.], [0.1, 0.15, 0.1])];
        for duration in [0.08, 0.125, 0.25, 0.5] {
            let mut previous = point(a, b, 0., duration, &bodies);
            for step in 1..=1000 {
                let u = step as f64 / 1000.;
                let next = point(a, b, u, duration, &bodies);
                assert!(distance(previous, next) * 1000. / duration <= MAX_TIP_SPEED + 1e-6);
                previous = next;
                let authored = (std::f64::consts::PI * u).sin() * 0.09;
                assert_eq!(lift(a, b, u, authored, duration, true, &bodies), authored);
            }
        }
        // An impossible-time chord gets no additional detour or timing shift.
        assert_eq!(lift(a, b, 0.5, 0., 0.02, false, &bodies), 0.);
    }
    #[test]
    fn remote_equipment_does_not_change_the_travel_guide() {
        let a = [-0.3, 1., 0.];
        let b = [0.3, 1., 0.];
        let bodies = [body([3., 1.1, 3.], [0.5, 0.5, 0.5])];
        assert_eq!(
            lift(a, b, 0.5, 0., 0.4, false, &bodies),
            lift(a, b, 0.5, 0., 0.4, false, &[])
        );
    }
}
