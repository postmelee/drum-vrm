//! Bounded clearance guidance for travel between distinct contact surfaces.
//! This shapes the raw tip path; the whole-tool equipment query remains the
//! authority. A tip arc alone does not certify the wrist, shaft or elbow route.
use super::{distance, Vec3, MAX_TIP_SPEED};
use crate::equipment::EquipmentBox;

const READY_CLEARANCE: f64 = 0.035;
const MAX_ARCH_HEIGHT: f64 = 0.20;
const CORRIDOR_MARGIN: f64 = 0.008;
const TOP_MARGIN: f64 = 0.012;
// max |d/du (16 u² (1-u)²)| = 16 / (3 sqrt(3)) < 3.08.
const ARCH_SLOPE_BOUND: f64 = 3.08;

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
    if same_target || (b[0] - a[0]).hypot(b[2] - a[2]) < 1e-6 {
        return stroke_lift;
    }
    let mut height = a[1].max(b[1]) + READY_CLEARANCE;
    for body in bodies {
        let top = body.center[1] + body.half_extents[1];
        let bottom = body.center[1] - body.half_extents[1];
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
            let extent = body.half_extents[axis] + CORRIDOR_MARGIN;
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
            height = height.max(top + TOP_MARGIN);
        }
    }
    // Smoothstep's chord derivative is at most 1.5. Do not make an already
    // rapid transfer longer merely to reach an ideal clearance height.
    let budget = ((MAX_TIP_SPEED * duration - 1.5 * distance(a, b)).max(0.) / ARCH_SLOPE_BOUND)
        .min(MAX_ARCH_HEIGHT);
    let amplitude = (height - 0.5 * (a[1] + b[1])).clamp(0., budget);
    let u = phase.clamp(0., 1.);
    let arch = 16. * u.powi(2) * (1. - u).powi(2);
    stroke_lift.max(amplitude * arch)
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
