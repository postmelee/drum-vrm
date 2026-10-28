//! Conservative full-tool pair clearance from the shared rendered taper. Each
//! linear radius band is enclosed by capsules. Adaptive subdivision tightens
//! uncertain tapered terminals; its finite budget may reject, never certify an
//! unresolved overlap. Constant-radius shaft crossings do not need refinement.
use super::{add, distance, mul, segment_distance, sub, Vec3};
const MAX_DEPTH: usize = 8;
const RADIUS_RESOLUTION: f64 = 0.000_05;
#[derive(Clone, Copy)]
struct Band {
    start: Vec3,
    end: Vec3,
    start_radius: f64,
    end_radius: f64,
}
impl Band {
    fn radius(self) -> f64 {
        self.start_radius.max(self.end_radius)
    }
    fn variation(self) -> f64 {
        (self.end_radius - self.start_radius).abs()
    }
    fn split(self) -> [Self; 2] {
        let middle = mul(add(self.start, self.end), 0.5);
        let radius = (self.start_radius + self.end_radius) * 0.5;
        [
            Self {
                end: middle,
                end_radius: radius,
                ..self
            },
            Self {
                start: middle,
                start_radius: radius,
                ..self
            },
        ]
    }
}
fn bands((butt, tip): (Vec3, Vec3), trim: f64) -> Vec<Band> {
    let length = distance(butt, tip);
    if length <= 1e-10 {
        return vec![Band {
            start: tip,
            end: tip,
            start_radius: 0.,
            end_radius: 0.,
        }];
    }
    let point = |d: f64| add(tip, mul(sub(butt, tip), d / length));
    let terminal = &crate::stick_geometry::profile().terminal;
    let mut result = Vec::with_capacity(terminal.len());
    let start = trim.clamp(0., length);
    for (i, &[from, radius]) in terminal.iter().enumerate() {
        let [to, next_radius] = terminal.get(i + 1).copied().unwrap_or([length, radius]);
        let a = start.max(from);
        let b = length.min(to);
        if b <= a || to <= from {
            continue;
        }
        let radius_at = |d: f64| radius + (next_radius - radius) * (d - from) / (to - from);
        result.push(Band {
            start: point(a),
            end: point(b),
            start_radius: radius_at(a),
            end_radius: radius_at(b),
        });
    }
    result
}
// Terminal subdivisions can be tens of micrometres long. Rebase and scale
// before the shared segment query so its absolute degeneracy tolerance cannot
// turn a short crossing into an endpoint distance and overestimate clearance.
fn axis_distance(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> f64 {
    let scale = distance(a, b).max(distance(c, d));
    if scale <= 1e-12 {
        return distance(a, c);
    }
    let local = |p| mul(sub(p, a), 1. / scale);
    // In normalized coordinates the shared query's 1e-14 degeneracy/parallel
    // cutoff permits at most sqrt(1e-14) perpendicular displacement. Subtract
    // that bound instead of treating its endpoint approximation as exact.
    ((segment_distance([0.; 3], local(b), local(c), local(d)) - 1e-7) * scale).max(0.)
}
fn bound(a: Band, b: Band, depth: usize) -> f64 {
    let lower = axis_distance(a.start, a.end, b.start, b.end) - a.radius() - b.radius();
    let variation = a.variation() + b.variation();
    // Refining a clearly intersecting proxy cannot turn the bound into a safe
    // witness. Keep its conservative value; do not spend the terminal budget.
    if lower >= crate::posture::MARGIN + 1e-9
        || lower + variation < -1e-9
        || depth == MAX_DEPTH
        || variation <= RADIUS_RESOLUTION
    {
        return lower;
    }
    if a.variation() >= b.variation() {
        a.split()
            .into_iter()
            .map(|part| bound(part, b, depth + 1))
            .fold(f64::INFINITY, f64::min)
    } else {
        b.split()
            .into_iter()
            .map(|part| bound(a, part, depth + 1))
            .fold(f64::INFINITY, f64::min)
    }
}
/// `trim` excludes only a named terminal length; callers own whether that
/// exclusion is an explicit intentional contact. Untrimmed queries cover butt,
/// full shaft, taper and terminal on both tools.
pub(super) fn clearance(a: (Vec3, Vec3), b: (Vec3, Vec3), trim: [f64; 2]) -> f64 {
    let max_radius = crate::stick_geometry::profile().terminal.last().unwrap()[1];
    let coarse = axis_distance(a.0, a.1, b.0, b.1) - 2. * max_radius;
    if coarse >= crate::posture::MARGIN + 1e-9 {
        return coarse;
    }
    let left = bands(a, trim[0]);
    let right = bands(b, trim[1]);
    left.into_iter()
        .flat_map(|a| right.iter().map(move |&b| bound(a, b, 0)))
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_scale_does_not_turn_a_crossing_into_endpoint_separation() {
        assert!(
            axis_distance(
                [-1e-5, 0., 0.],
                [1e-5, 0., 0.],
                [0., -1e-5, 0.],
                [0., 1e-5, 0.]
            ) < 1e-12
        );
    }
    #[test]
    fn facing_point_terminals_are_not_full_shaft_caps() {
        let a = ([-0.4, 0., 0.], [0.; 3]);
        let b = ([0.404, 0., 0.], [0.004, 0., 0.]);
        let gap = clearance(a, b, [0.; 2]);
        assert!((crate::posture::MARGIN..=0.004).contains(&gap));
        assert!(clearance(a, ([0.401, 0., 0.], [0.001, 0., 0.]), [0.; 2]) < crate::posture::MARGIN);
    }
    #[test]
    fn whole_shafts_and_butt_crossings_remain_hard() {
        let a = ([-0.2, 0., 0.], [0.2, 0., 0.]);
        for x in [-0.19, 0.] {
            let b = ([x, 0., -0.2], [x, 0., 0.2]);
            assert!(clearance(a, b, [0.; 2]) < -0.01);
            assert!(clearance(a, b, [0.04, 0.]) < -0.01);
        }
    }
    #[test]
    fn pair_bound_is_rigid_transform_and_exchange_invariant() {
        let q = [0., 0.5, 0., 3_f64.sqrt() / 2.];
        let transform = |p| add(crate::rig::rotate(q, p), [0.4, 1., -0.3]);
        for (a, b) in [
            (
                ([-0.4, 0., 0.], [0.; 3]),
                ([0.404, 0., 0.], [0.004, 0., 0.]),
            ),
            (
                ([-0.2, 0., 0.], [0.2, 0., 0.]),
                ([0., 0., -0.2], [0., 0., 0.2]),
            ),
        ] {
            let expected = clearance(a, b, [0.; 2]);
            assert!((clearance(b, a, [0.; 2]) - expected).abs() < 1e-9);
            assert!(
                (clearance(
                    (transform(a.0), transform(a.1)),
                    (transform(b.0), transform(b.1)),
                    [0.; 2]
                ) - expected)
                    .abs()
                    < 1e-9
            );
        }
    }
}
