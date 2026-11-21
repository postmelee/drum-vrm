//! Solve the original routed tip with a coupled
//! forearm/wrist/tool frame using measured bones and the complete tool mount.
//!
//! A measured lower-arm direction in hand-local coordinates determines wrist
//! swing. The lower arm plus the COMPLETE calibrated wrist-to-tip offset is a
//! virtual lever; its length is an algebraic construction, not a reach or posture
//! certificate. Every result still requires the existing physical checks.
use crate::{
    model::Vec3,
    motion::{add, distance, sub},
    rig::{rotate, HandSolution, Quat, RigProfile},
};

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn mul(a: Vec3, s: f64) -> Vec3 {
    a.map(|x| x * s)
}
fn unit(a: Vec3) -> Option<Vec3> {
    let n = distance(a, [0.; 3]);
    (n.is_finite() && n > 1e-10).then(|| mul(a, 1. / n))
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(q: Quat) -> Option<Quat> {
    let n = q.iter().map(|x| x * x).sum::<f64>().sqrt();
    (n.is_finite() && n > 1e-10).then(|| q.map(|x| x / n))
}
fn multiply(a: Quat, b: Quat) -> Quat {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}
fn around(axis: Vec3, angle: f64) -> Quat {
    let v = mul(axis, (angle * 0.5).sin());
    [v[0], v[1], v[2], (angle * 0.5).cos()]
}
fn project(v: Vec3, axis: Vec3) -> Option<Vec3> {
    unit(sub(v, mul(axis, dot(v, axis))))
}
fn shortest_arc(a: Vec3, b: Vec3, fallback: Vec3) -> Option<Quat> {
    let (a, b) = (unit(a)?, unit(b)?);
    let cosine = dot(a, b).clamp(-1., 1.);
    if cosine < -1. + 1e-9 {
        return Some(around(project(fallback, a)?, std::f64::consts::PI));
    }
    let v = cross(a, b);
    normalize([v[0], v[1], v[2], 1. + cosine])
}

fn anatomical_plane(profile: &RigProfile, limb: usize, target: Vec3) -> Option<(Vec3, Vec3)> {
    let shoulders = sub(profile.roots[1], profile.roots[0]);
    let right = unit([shoulders[0], 0., shoulders[2]]).unwrap_or([1., 0., 0.]);
    let outward = if limb == 0 { -1. } else { 1. };
    let down = unit(add(mul(right, outward * 0.45), [0., -1., 0.]))?;
    let forward = [-right[2], 0., right[0]];
    let axis = unit(sub(target, profile.roots[limb]))?;
    let pole = rotate(shortest_arc(forward, axis, down)?, down);
    Some((axis, pole))
}

/// Recover the actual endpoint arm plane around the shoulder-to-tip axis.
/// This is distinct from the existing shoulder-to-wrist swivel parameter.
pub(crate) fn source_plane(profile: &RigProfile, limb: usize, hand: &HandSolution) -> Option<f64> {
    let (axis, pole) = anatomical_plane(profile, limb, hand.tip)?;
    let actual = project(sub(hand.middle, profile.roots[limb]), axis)?;
    Some(dot(axis, cross(pole, actual)).atan2(dot(pole, actual)))
}

/// All shape inputs are continuous phase curves. `forearm_local` is measured
/// from the physical lower bone, transformed by inverse(actual hand rotation).
/// `prior_rotation` fixes the residual lever twist; `twist` is an optional
/// continuous correction around elbow-to-tip. No optimizer runs here.
pub(crate) fn solve(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    forearm_local: Vec3,
    pole_angle: f64,
    prior_rotation: Quat,
    twist: f64,
) -> Option<HandSolution> {
    let root = profile.roots[limb];
    let [upper, lower] = profile.lengths[limb];
    let forearm_local = unit(forearm_local)?;
    // The mount is wrist→grip PLUS grip→tip. Omitting either is incorrect for
    // the measured non-collinear public VRM hand/tool mounts.
    let mount = add(profile.grip_offsets[limb], profile.tip_offsets[limb]);
    let local_lever = add(mul(forearm_local, lower), mount);
    let lever_length = distance(local_lever, [0.; 3]);
    let target_distance = distance(target, root);
    if target_distance <= (upper - lever_length).abs() + 1e-8
        || target_distance >= upper + lever_length - 1e-8
    {
        return None; // Do not substitute a clamped endpoint or old bad pose.
    }
    let (axis, pole) = anatomical_plane(profile, limb, target)?;
    let bend = rotate(around(axis, pole_angle), pole);
    let along = (upper * upper - lever_length * lever_length + target_distance * target_distance)
        / (2. * target_distance);
    let radius = (upper * upper - along * along).max(0.).sqrt();
    let middle = add(add(root, mul(axis, along)), mul(bend, radius));
    let world_lever = sub(target, middle);
    let lever_axis = unit(world_lever)?;
    let rotation = normalize(multiply(
        around(lever_axis, twist),
        multiply(
            shortest_arc(rotate(prior_rotation, local_lever), world_lever, pole)?,
            prior_rotation,
        ),
    ))?;
    let end = add(middle, mul(rotate(rotation, forearm_local), lower));
    let tip = add(end, profile.offset(limb, rotation));
    // Algebraically: tip-middle = rotate(q, lower*forearm_local + mount).
    // Both real bone lengths are unchanged, and this equals target-middle.
    Some(HandSolution {
        middle,
        end,
        tip,
        rotation,
        reachable: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn inverse(q: Quat) -> Quat {
        [-q[0], -q[1], -q[2], q[3]]
    }
    fn source(profile: &RigProfile, limb: usize) -> HandSolution {
        let sign = if limb == 0 { -1. } else { 1. };
        let rotation = around(unit([0.3, sign * 0.4, 0.8]).unwrap(), 0.35);
        let upper = unit([sign * 0.16, -0.9, 0.27]).unwrap();
        let lower_local = unit([sign * 0.12, 0.23, 1.]).unwrap();
        let middle = add(profile.roots[limb], mul(upper, profile.lengths[limb][0]));
        let end = add(
            middle,
            mul(rotate(rotation, lower_local), profile.lengths[limb][1]),
        );
        HandSolution {
            middle,
            end,
            tip: add(end, profile.offset(limb, rotation)),
            rotation,
            reachable: true,
        }
    }
    fn profile() -> RigProfile {
        let mut p = RigProfile::primitive();
        p.grip_offsets[0] = [0.027, -0.019, 0.011];
        p.grip_offsets[1] = [-0.021, -0.012, 0.017];
        p.tip_offsets[0] = [0.021, 0.012, 0.315];
        p.tip_offsets[1] = [-0.014, 0.022, 0.327];
        p
    }
    #[test]
    fn measured_source_roundtrip_keeps_both_real_bones_and_complete_mount() {
        let p = profile();
        for limb in 0..2 {
            let h = source(&p, limb);
            let local = rotate(inverse(h.rotation), unit(sub(h.end, h.middle)).unwrap());
            let q = solve(
                &p,
                limb,
                h.tip,
                local,
                source_plane(&p, limb, &h).unwrap(),
                h.rotation,
                0.,
            )
            .unwrap();
            assert!(distance(q.middle, h.middle) < 1e-12);
            assert!(distance(q.end, h.end) < 1e-12);
            assert!(distance(q.tip, h.tip) < 1e-12);
            assert!(
                (dot(
                    q.rotation[..3].try_into().unwrap(),
                    h.rotation[..3].try_into().unwrap()
                ) + q.rotation[3] * h.rotation[3])
                    .abs()
                    > 1. - 1e-12
            );
            assert!((distance(p.roots[limb], q.middle) - p.lengths[limb][0]).abs() < 1e-12);
            assert!((distance(q.middle, q.end) - p.lengths[limb][1]).abs() < 1e-12);
        }
    }
    #[test]
    fn exact_solution_is_covariant_under_rigid_yaw_and_translation() {
        let p = profile();
        let yaw = around([0., 1., 0.], 0.79);
        let shift = [-0.31, 0.17, 0.22];
        let transform = |v| add(rotate(yaw, v), shift);
        let mut moved = p.clone();
        moved.roots = moved.roots.map(transform);
        for limb in 0..2 {
            let h = source(&p, limb);
            let local = rotate(inverse(h.rotation), unit(sub(h.end, h.middle)).unwrap());
            let plane = source_plane(&p, limb, &h).unwrap() + 0.13;
            let original = solve(&p, limb, h.tip, local, plane, h.rotation, -0.07).unwrap();
            let result = solve(
                &moved,
                limb,
                transform(h.tip),
                local,
                plane,
                multiply(yaw, h.rotation),
                -0.07,
            )
            .unwrap();
            assert!(distance(result.middle, transform(original.middle)) < 1e-12);
            assert!(distance(result.end, transform(original.end)) < 1e-12);
            assert!(distance(result.tip, transform(original.tip)) < 1e-12);
            assert!(
                distance(
                    rotate(result.rotation, [0.2, -0.1, 0.3]),
                    rotate(yaw, rotate(original.rotation, [0.2, -0.1, 0.3]))
                ) < 1e-12
            );
        }
    }
    #[test]
    fn impossible_and_degenerate_virtual_triangles_return_none_without_clamping() {
        let p = profile();
        for limb in 0..2 {
            let lower = [0., 0., 1.];
            let lever = distance(
                add(
                    mul(lower, p.lengths[limb][1]),
                    add(p.grip_offsets[limb], p.tip_offsets[limb]),
                ),
                [0.; 3],
            );
            for d in [
                0.,
                (p.lengths[limb][0] - lever).abs(),
                p.lengths[limb][0] + lever,
                p.lengths[limb][0] + lever + 0.1,
            ] {
                assert!(solve(
                    &p,
                    limb,
                    add(p.roots[limb], [0., 0., d]),
                    lower,
                    0.,
                    [0., 0., 0., 1.],
                    0.
                )
                .is_none());
            }
        }
    }
}
