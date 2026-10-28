//! Conservative whole-segment queries against an authored surface of revolution.
//! A tangent plane at one contact cannot describe a cymbal's bell and bow.
use crate::{
    equipment::{EquipmentBox, RadialSurfaceProfile},
    model::Vec3,
    motion::sub,
    rig::rotate,
};

fn inverse(q: [f64; 4]) -> [f64; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// A 1-Lipschitz lower bound on distance to the body below the declared upper
/// profile, clipped to the measured body box and outer profile radius. Filling
/// the centre hole and the space below the surface is deliberately conservative:
/// approaching a playing surface from underneath cannot become a valid strike.
#[cfg(test)]
fn point_bound(
    point: Vec3,
    body: &EquipmentBox,
    profile: &RadialSurfaceProfile,
    slope: f64,
) -> f64 {
    let local = rotate(inverse(profile.rotation), sub(point, profile.origin));
    let radius = local[0].hypot(local[2]);
    let points = &profile.points;
    let index = points
        .partition_point(|p| p[0] < radius)
        .clamp(1, points.len() - 1);
    let [a, b] = [points[index - 1], points[index]];
    let height = a[1] + (radius - a[0]).clamp(0., b[0] - a[0]) * (b[1] - a[1]) / (b[0] - a[0]);
    let relative = sub(point, body.center);
    let relative = body
        .rotation
        .map_or(relative, |q| rotate(inverse(q), relative));
    let box_bound = (0..3)
        .map(|axis| relative[axis].abs() - body.half_extents[axis])
        .fold(f64::NEG_INFINITY, f64::max);
    ((local[1] - height) / (1. + slope * slope).sqrt())
        .max(radius - points.last().unwrap()[0])
        .max(box_bound)
}

/// Lower bound for a tapered segment. Endpoint samples alone could miss a bow
/// crossing. Exact radial extrema and piecewise-profile maxima certify each
/// interval; long shafts above the bow usually pass without subdivision.
/// A finite subdivision budget returns an unresolved conservative bound; it
/// never converts an uncertain interval into clearance.
pub(super) fn interval(
    start: Vec3,
    end: Vec3,
    radii: [f64; 2],
    body: &EquipmentBox,
    profile: &RadialSurfaceProfile,
) -> f64 {
    let slope = profile
        .points
        .windows(2)
        .map(|p| ((p[1][1] - p[0][1]) / (p[1][0] - p[0][0])).abs())
        .fold(0., f64::max);
    let origin = profile.origin;
    let inverse_profile = inverse(profile.rotation);
    let local = [start, end].map(|point| rotate(inverse_profile, sub(point, origin)));
    let box_local = [start, end].map(|point| {
        let relative = sub(point, body.center);
        body.rotation
            .map_or(relative, |q| rotate(inverse(q), relative))
    });
    let norm = (1. + slope * slope).sqrt();
    let point = |ends: [Vec3; 2], t: f64| -> Vec3 {
        std::array::from_fn(|axis| ends[0][axis] + (ends[1][axis] - ends[0][axis]) * t)
    };
    let height = |radius: f64| {
        let points = &profile.points;
        let index = points
            .partition_point(|p| p[0] < radius)
            .clamp(1, points.len() - 1);
        let [a, b] = [points[index - 1], points[index]];
        a[1] + (radius - a[0]).clamp(0., b[0] - a[0]) * (b[1] - a[1]) / (b[0] - a[0])
    };
    let bound = |a: f64, b: f64| {
        let [p, q] = [point(local, a), point(local, b)];
        let dx = q[0] - p[0];
        let dz = q[2] - p[2];
        let length_squared = dx * dx + dz * dz;
        let closest = if length_squared > 1e-20 {
            (-(p[0] * dx + p[2] * dz) / length_squared).clamp(0., 1.)
        } else {
            0.
        };
        let low = (p[0] + dx * closest).hypot(p[2] + dz * closest);
        let high = p[0].hypot(p[2]).max(q[0].hypot(q[2]));
        let maximum_height = profile
            .points
            .iter()
            .filter(|point| point[0] >= low && point[0] <= high)
            .map(|point| point[1])
            .fold(height(low).max(height(high)), f64::max);
        let [pa, pb] = [point(box_local, a), point(box_local, b)];
        let box_bound = (0..3)
            .flat_map(|axis| {
                [-1., 1.]
                    .map(|sign| (sign * pa[axis]).min(sign * pb[axis]) - body.half_extents[axis])
            })
            .fold(f64::NEG_INFINITY, f64::max);
        let radius =
            (radii[0] + (radii[1] - radii[0]) * a).max(radii[0] + (radii[1] - radii[0]) * b);
        ((p[1].min(q[1]) - maximum_height) / norm)
            .max(low - profile.points.last().unwrap()[0])
            .max(box_bound)
            - radius
    };
    let mut pending = vec![(0., 1., 0_u8)];
    let mut minimum = f64::INFINITY;
    let mut visits = 0;
    while let Some((a, b, depth)) = pending.pop() {
        let lower = bound(a, b);
        visits += 1;
        if lower >= 0. || bound(a, a) <= 0. || bound(b, b) <= 0. || depth == 16 || visits >= 256 {
            minimum = minimum.min(lower);
            continue;
        }
        let mid = (a + b) * 0.5;
        pending.push((mid, b, depth + 1));
        pending.push((a, mid, depth + 1));
    }
    minimum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (EquipmentBox, RadialSurfaceProfile) {
        (
            EquipmentBox {
                piece_id: "cymbal".into(),
                center: [0., -0.02, 0.],
                half_extents: [0.25, 0.05, 0.25],
                rotation: None,
                kind: "body",
            },
            RadialSurfaceProfile {
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
            },
        )
    }

    #[test]
    fn rejects_bow_crossing_between_clear_segment_endpoints() {
        let (body, profile) = fixture();
        // Both ends are outside the disk; the centre crosses its raised bell.
        assert!(
            interval(
                [-0.3, 0.01, 0.],
                [0.3, 0.01, 0.],
                [0.003; 2],
                &body,
                &profile
            ) < 0.
        );
    }

    #[test]
    fn certifies_shaft_above_bow_without_extending_bell_tangent() {
        let (body, profile) = fixture();
        assert!(
            interval(
                [0.07, 0.015, 0.],
                [0.30, 0.005, 0.],
                [0.003, 0.006],
                &body,
                &profile
            ) > 0.
        );
        assert!(
            interval(
                [0.07, -0.03, 0.],
                [0.2, -0.035, 0.],
                [0.003; 2],
                &body,
                &profile
            ) < 0.
        );
    }

    #[test]
    fn bell_contact_cannot_exempt_a_later_bow_crossing() {
        let (mut body, mut profile) = fixture();
        body.half_extents = [0.26, 0.06, 0.26];
        profile.points = vec![
            [0.008, -0.004],
            [0.03048, 0.],
            [0.06096, -0.022],
            [0.2032, -0.038],
            [0.254, -0.044],
        ];
        let tip = [-0.02194422083, 0., -0.02115376024];
        let butt = [-0.09499178237, -0.10595165694, -0.41569564666];
        // A bow crossing over 120 mm from the tip is outside the intended
        // terminal contact region, regardless of the tangent at the bell.
        let start = std::array::from_fn(|axis| tip[axis] + (butt[axis] - tip[axis]) * 0.07);
        assert!(interval(start, butt, [0.003, 0.006], &body, &profile) < 0.);
    }

    #[test]
    fn rotated_profile_and_body_preserve_the_same_certificate() {
        let (body, profile) = fixture();
        let a = [0.07, 0.015, 0.];
        let b = [0.30, 0.005, 0.];
        let before = interval(a, b, [0.003, 0.006], &body, &profile);
        let angle = 0.37_f64;
        let q = [(angle / 2.).sin(), 0., 0., (angle / 2.).cos()];
        let mut moved_body = body.clone();
        moved_body.center = rotate(q, body.center);
        moved_body.rotation = Some(q);
        let mut moved_profile = profile.clone();
        moved_profile.origin = rotate(q, profile.origin);
        moved_profile.rotation = q;
        let after = interval(
            rotate(q, a),
            rotate(q, b),
            [0.003, 0.006],
            &moved_body,
            &moved_profile,
        );
        assert!((before - after).abs() < 1e-12);
    }

    #[test]
    fn result_is_a_lower_bound_on_dense_signed_profile_samples() {
        let (body, profile) = fixture();
        let slope = 0.025 / 0.03;
        for i in 0..40 {
            let a = [-0.27, -0.04 + i as f64 * 0.003, 0.015];
            let b = [0.27, 0.045 - i as f64 * 0.001, -0.02];
            let bound = interval(a, b, [0.002, 0.006], &body, &profile);
            for step in 0..=1000 {
                let t = step as f64 / 1000.;
                let point = std::array::from_fn(|axis| a[axis] + (b[axis] - a[axis]) * t);
                assert!(
                    bound
                        <= point_bound(point, &body, &profile, slope) - (0.002 + 0.004 * t) + 1e-12
                );
            }
        }
    }
}
