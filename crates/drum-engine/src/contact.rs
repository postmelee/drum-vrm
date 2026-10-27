//! Deterministic musical contact regions in the measured playing-surface frame.
//! A near-player direction selects a sector, not an instruction to hit every rim.
pub use crate::contact_events::{append_count_in_events, refresh_events, ContactEvent};
use crate::{
    model::*,
    motion::{add, distance, sub},
    rig::RigProfile,
};

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn mul(a: Vec3, n: f64) -> Vec3 {
    a.map(|v| v * n)
}
pub fn surface(piece: &KitPiece) -> (Vec3, Vec3, f64) {
    if let Some(g) = &piece.geometry {
        let scale = piece.radius / g.reference_radius;
        (
            add(piece.position, mul(g.surface.center, scale)),
            g.surface.normal,
            g.surface.radius * scale,
        )
    } else {
        (piece.position, [0., 1., 0.], piece.radius)
    }
}
/// Engineering regions, not dimensions prescribed by the source teaching text.
pub fn radial_fraction(instrument: Instrument, articulation: &str) -> f64 {
    match articulation {
        "bell" => 0.12,
        "inner" => 0.38,
        "outer" => 0.62,
        "edge" | "rimshotEdge" | "sideStick" => 0.86,
        "crash" | "china" | "splash" => 0.88,
        _ if matches!(instrument, Instrument::Hihat | Instrument::OpenHat) => 0.62,
        _ => 0.,
    }
}
/// Distinct implemented contact regions, shared by fit and source planning.
/// Equivalent regions (for example center/rimshot and sideStick/rimshotEdge)
/// need one geometric probe; they remain distinct musical articulations.
pub fn articulations_for_fit(piece: &KitPiece) -> &'static [&'static str] {
    match piece.instrument {
        Instrument::Snare => &["center", "rimshotEdge"],
        Instrument::Hihat | Instrument::OpenHat => &["center", "edge"],
        Instrument::Ride => &["bell", "inner", "outer", "edge"],
        Instrument::Crash => &["crash"],
        _ => &["center"],
    }
}
pub fn target(piece: &KitPiece, articulation: &str, observer: Vec3) -> Vec3 {
    if matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat) {
        return piece.position;
    }
    if let Some((profile, origin, scale)) = radial_frame(piece) {
        let local = crate::rig::rotate(inverse(profile.rotation), sub(observer, origin));
        let length = local[0].hypot(local[2]);
        let direction = if length > 1e-9 {
            [local[0] / length, 0., local[2] / length]
        } else {
            [0., 0., -1.]
        };
        let radius =
            profile.points.last().unwrap()[0] * radial_fraction(piece.instrument, articulation);
        let height = radial_height(&profile.points, radius).0;
        return add(
            origin,
            mul(
                crate::rig::rotate(
                    profile.rotation,
                    [direction[0] * radius, height, direction[2] * radius],
                ),
                scale,
            ),
        );
    }
    let (center, normal, radius) = surface(piece);
    let delta = sub(observer, center);
    let direction = sub(delta, mul(normal, dot(delta, normal)));
    let length = distance(direction, [0.; 3]);
    let direction = if length > 1e-8 {
        mul(direction, 1. / length)
    } else {
        let axis = (0..3)
            .min_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))
            .unwrap();
        let mut basis = [0.; 3];
        basis[axis] = 1.;
        let projected = sub(basis, mul(normal, dot(basis, normal)));
        mul(projected, 1. / distance(projected, [0.; 3]).max(1e-9))
    };
    add(
        center,
        mul(
            direction,
            radius * radial_fraction(piece.instrument, articulation),
        ),
    )
}

fn inverse(q: [f64; 4]) -> [f64; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}
fn radial_frame(piece: &KitPiece) -> Option<(&crate::equipment::RadialSurfaceProfile, Vec3, f64)> {
    let geometry = piece.geometry.as_ref()?;
    let profile = geometry.surface.radial_profile.as_ref()?;
    let scale = piece.radius / geometry.reference_radius;
    Some((
        profile,
        add(piece.position, mul(profile.origin, scale)),
        scale,
    ))
}
fn radial_height(points: &[[f64; 2]], radius: f64) -> (f64, f64) {
    let index = points
        .partition_point(|point| point[0] < radius)
        .clamp(1, points.len() - 1);
    let [a, b] = [points[index - 1], points[index]];
    let slope = (b[1] - a[1]) / (b[0] - a[0]);
    (a[1] + (radius - a[0]).clamp(0., b[0] - a[0]) * slope, slope)
}
/// Profile-local curved contact, including cymbal bows/bells. Unknown imported
/// surfaces retain their declared plane instead of inventing a radial profile.
pub fn normal_at(piece: &KitPiece, world_point: Vec3) -> Vec3 {
    let Some((profile, origin, scale)) = radial_frame(piece) else {
        return surface(piece).1;
    };
    let local = mul(
        crate::rig::rotate(inverse(profile.rotation), sub(world_point, origin)),
        1. / scale,
    );
    let radius = local[0].hypot(local[2]);
    let slope = radial_height(&profile.points, radius).1;
    let normal = if radius > 1e-9 {
        [-slope * local[0] / radius, 1., -slope * local[2] / radius]
    } else {
        [0., 1., 0.]
    };
    let normal = mul(normal, 1. / distance(normal, [0.; 3]));
    crate::rig::rotate(profile.rotation, normal)
}
/// Project onto the static declared surface, not onto a plane through the
/// queried tip. Curved profiles use their authored local Y axis; this is an
/// axial projection rather than a closest-triangle query. The caller owns the
/// playable-region bounds and any mechanism displacement.
pub fn project_surface(piece: &KitPiece, point: Vec3) -> (Vec3, Vec3) {
    if let Some((profile, origin, scale)) = radial_frame(piece) {
        let local = mul(
            crate::rig::rotate(inverse(profile.rotation), sub(point, origin)),
            1. / scale,
        );
        let height = radial_height(&profile.points, local[0].hypot(local[2])).0;
        let projected = add(
            origin,
            mul(
                crate::rig::rotate(profile.rotation, [local[0], height, local[2]]),
                scale,
            ),
        );
        return (projected, normal_at(piece, projected));
    }
    let (center, normal, _) = surface(piece);
    (
        sub(point, mul(normal, dot(sub(point, center), normal))),
        normal,
    )
}
/// A measured surface moves only when its authored mechanism declares that
/// capability. Unknown imported meshes and electronic pads remain fixed.
pub fn mechanism_offset(piece: &KitPiece, opening: f64) -> Vec3 {
    if !matches!(piece.instrument, Instrument::Hihat | Instrument::OpenHat) {
        return [0.; 3];
    }
    match &piece.geometry {
        None => [0., crate::equipment::HIHAT_MAX_OPENING * opening, 0.],
        Some(geometry) => geometry.surface.motion.as_ref().map_or([0.; 3], |motion| {
            mul(motion.axis, motion.travel_meters * opening)
        }),
    }
}
pub fn apply(plan: &mut PerformancePlan, profile: &RigProfile) {
    plan.hihat_motion = crate::hihat_motion::prepare(plan);
    plan.diagnostics
        .retain(|diagnostic| diagnostic.code != "HIHAT_SURFACE_STATIC");
    let observer = mul(add(profile.roots[0], profile.roots[1]), 0.5);
    let mut static_opening = plan
        .hihat_motion
        .anchors
        .iter()
        .any(|anchor| anchor.opening > 0.)
        && plan.kit.iter().any(|piece| {
            piece.instrument == Instrument::Hihat
                && piece.geometry.is_some()
                && distance(mechanism_offset(piece, 1.), [0.; 3]) < 1e-12
        });
    for hit in &mut plan.hits {
        if let Some(piece) = plan.kit.iter().find(|p| p.id == hit.piece_id) {
            hit.target = if piece.geometry.is_some() {
                target(piece, &hit.articulation, observer)
            } else {
                piece.position
            };
            if matches!(hit.instrument, Instrument::Hihat | Instrument::OpenHat) {
                let opening = crate::hihat_motion::evaluate(&plan.hihat_motion, hit.time);
                let offset = mechanism_offset(piece, opening);
                hit.target = add(hit.target, offset);
                static_opening |= opening > 0. && distance(offset, [0.; 3]) < 1e-12;
            }
        }
    }
    if static_opening {
        plan.diagnostics.push(Diagnostic::new("HIHAT_SURFACE_STATIC", "The selected hi-hat surface has no movable upper cymbal. Openness changes the pedal state; the displayed strike surface remains fixed.".into()));
    }
}
/// A restored plan may contain the legacy anchor or a measured region contact.
/// Do not permit arbitrary world-space offsets to bypass the kit contract.
pub fn valid_target(piece: &KitPiece, target: Vec3) -> bool {
    if distance(piece.position, target) < 1e-9 {
        return true;
    }
    if piece.geometry.is_none() {
        return matches!(piece.instrument, Instrument::Hihat | Instrument::OpenHat)
            && (target[0] - piece.position[0]).abs() < 1e-9
            && (target[2] - piece.position[2]).abs() < 1e-9
            && (-1e-9..=crate::equipment::HIHAT_MAX_OPENING + 1e-9)
                .contains(&(target[1] - piece.position[1]));
    }
    if matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat) {
        return false;
    }
    if let Some((profile, origin, scale)) = radial_frame(piece) {
        let local = mul(
            crate::rig::rotate(inverse(profile.rotation), sub(target, origin)),
            1. / scale,
        );
        let radius = local[0].hypot(local[2]);
        let allowance = distance(mechanism_offset(piece, 1.), [0.; 3]) / scale;
        return radius <= profile.points.last().unwrap()[0] * 0.95 + allowance + 1e-9
            && (local[1] - radial_height(&profile.points, radius).0).abs() <= allowance + 1e-6;
    }
    let (center, normal, radius) = surface(piece);
    let delta = sub(target, center);
    // A declared moving surface translates along its shared axis. This is a
    // bounded acceptance envelope; preparation re-derives the exact timed target.
    let offset = mechanism_offset(piece, 1.);
    let projection = dot(offset, normal);
    let amount = if projection.abs() > 1e-9 {
        (dot(delta, normal) / projection).clamp(0., 1.)
    } else {
        0.
    };
    let on_plane = sub(delta, mul(offset, amount));
    dot(on_plane, normal).abs() < 1e-6 && distance(on_plane, [0.; 3]) <= radius * 0.95 + 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn musical_region_is_not_a_universal_nearest_edge() {
        let kit = default_kit();
        let observer = [0., 1.2, -0.2];
        let snare = kit.iter().find(|p| p.id == "snare").unwrap();
        assert_eq!(target(snare, "center", observer), snare.position);
        let ride = kit.iter().find(|p| p.id == "ride").unwrap();
        let bell = target(ride, "bell", observer);
        let bow = target(ride, "outer", observer);
        let edge = target(ride, "edge", observer);
        assert!(distance(bell, ride.position) < distance(bow, ride.position));
        assert!(distance(bow, ride.position) < distance(edge, ride.position));
        assert!(edge[2] < ride.position[2]);
        assert_eq!(edge, target(ride, "edge", observer));
    }
    fn tilted_piece(normal: Vec3) -> KitPiece {
        let mut piece = default_kit().into_iter().find(|p| p.id == "ride").unwrap();
        piece.position = [0.4, 0.9, 0.7];
        piece.radius = 0.32;
        piece.geometry = Some(crate::equipment::PieceGeometry {
            version: 1,
            source: "mesh-bounds".into(),
            reference_radius: 0.20,
            body: crate::equipment::LocalBox {
                center: [0.08, 0.04, -0.06],
                half_extents: [0.23, 0.09, 0.23],
            },
            surface: crate::equipment::Surface {
                center: [0.08, 0.12, -0.06],
                normal,
                radius: 0.19,
                radial_profile: None,
                motion: None,
            },
        });
        piece
    }
    #[test]
    fn measured_tilt_scale_and_translation_preserve_the_playing_plane() {
        let piece = tilted_piece([0.0, 0.6, 0.8]);
        let (center, normal, radius) = surface(&piece);
        assert!(distance(center, [0.528, 1.092, 0.604]) < 1e-12);
        assert!((radius - 0.304).abs() < 1e-12);
        let observer = add(center, [0.4, 1.3, -0.8]);
        for articulation in ["bell", "inner", "outer", "edge"] {
            let hit = target(&piece, articulation, observer);
            let radial = sub(hit, center);
            assert!(dot(radial, normal).abs() < 1e-12);
            assert!(
                (distance(radial, [0.0; 3])
                    - radius * radial_fraction(piece.instrument, articulation))
                .abs()
                    < 1e-12
            );
            assert!(dot(radial, sub(observer, center)) > 0.0);
            assert!(valid_target(&piece, hit));
            let shift = [-0.7, 0.3, 0.4];
            let mut moved = piece.clone();
            moved.position = add(piece.position, shift);
            assert!(
                distance(
                    target(&moved, articulation, add(observer, shift)),
                    add(hit, shift)
                ) < 1e-12
            );
        }
    }
    #[test]
    fn curved_profile_targets_and_normals_follow_tilted_bell_bow_and_edge() {
        let mut piece = tilted_piece([0., 1., 0.]);
        let angle = 0.19_f64;
        let rotation = [(angle / 2.).sin(), 0., 0., (angle / 2.).cos()];
        let origin = [0.02, 0.04, 0.05];
        let points = vec![[0., 0.02], [0.04, 0.01], [0.12, -0.01], [0.2, -0.03]];
        piece.geometry.as_mut().unwrap().surface.radial_profile =
            Some(crate::equipment::RadialSurfaceProfile {
                origin,
                rotation,
                points: points.clone(),
            });
        let scale = piece.radius / piece.geometry.as_ref().unwrap().reference_radius;
        let world_origin = add(piece.position, mul(origin, scale));
        for articulation in ["bell", "inner", "outer", "edge"] {
            let point = target(&piece, articulation, [0., 1.2, -0.4]);
            let local = mul(
                crate::rig::rotate(inverse(rotation), sub(point, world_origin)),
                1. / scale,
            );
            let radius = local[0].hypot(local[2]);
            assert!((radius - 0.2 * radial_fraction(piece.instrument, articulation)).abs() < 1e-12);
            assert!((local[1] - radial_height(&points, radius).0).abs() < 1e-12);
            assert!(valid_target(&piece, point));
            assert!((distance(normal_at(&piece, point), [0.; 3]) - 1.).abs() < 1e-12);
            assert!(!valid_target(&piece, add(point, [0., 0.01, 0.])));
            let direction = crate::rig::rotate(rotation, [0., 1., 0.]);
            for height in [-0.08, 0.08] {
                let query = add(point, mul(direction, height));
                let (projected, normal) = project_surface(&piece, query);
                assert!(distance(projected, point) < 1e-12);
                assert_eq!(
                    dot(sub(query, projected), normal).is_sign_positive(),
                    height > 0.
                );
            }
        }
    }
    #[test]
    fn observer_aligned_with_any_plane_normal_uses_a_finite_tangent() {
        let diagonal = 1.0 / 3.0_f64.sqrt();
        for normal in [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [diagonal; 3],
        ] {
            let piece = tilted_piece(normal);
            let (center, _, radius) = surface(&piece);
            for observer in [center, add(center, mul(normal, 1.7))] {
                let hit = target(&piece, "edge", observer);
                assert!(hit.into_iter().all(f64::is_finite));
                assert!(dot(sub(hit, center), normal).abs() < 1e-12);
                assert!((distance(hit, center) - radius * 0.86).abs() < 1e-12);
                assert!(valid_target(&piece, hit));
                assert_eq!(hit, target(&piece, "edge", observer));
            }
        }
    }
    #[test]
    fn restored_target_bounds_keep_legacy_anchors_without_accepting_arbitrary_offsets() {
        let piece = tilted_piece([0.0, 0.6, 0.8]);
        let (center, normal, radius) = surface(&piece);
        assert!(valid_target(&piece, piece.position)); // Legacy anchor is off the new measured plane.
        assert!(dot(sub(piece.position, center), normal).abs() > 0.01);
        assert!(valid_target(&piece, add(center, [radius * 0.95, 0.0, 0.0])));
        assert!(!valid_target(
            &piece,
            add(center, [radius * 0.9501, 0.0, 0.0])
        ));
        assert!(valid_target(&piece, add(center, mul(normal, 0.5e-6))));
        assert!(!valid_target(&piece, add(center, mul(normal, 2.0e-6))));
        assert!(!valid_target(&piece, [f64::NAN, 0.0, 0.0]));
        assert!(!valid_target(&piece, [f64::INFINITY, 0.0, 0.0]));
        let mut legacy = piece.clone();
        legacy.geometry = None;
        assert!(valid_target(&legacy, legacy.position));
        assert!(!valid_target(&legacy, center));
        for instrument in [Instrument::Kick, Instrument::PedalHat] {
            let mut pedal = piece.clone();
            pedal.instrument = instrument;
            assert_eq!(target(&pedal, "edge", [0.0; 3]), pedal.position);
            assert!(valid_target(&pedal, pedal.position));
            assert!(!valid_target(&pedal, center));
        }
    }
}
