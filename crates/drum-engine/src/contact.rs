//! Deterministic musical contact regions in the measured playing-surface frame.
//! A near-player direction selects a sector, not an instruction to hit every rim.
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
pub fn target(piece: &KitPiece, articulation: &str, observer: Vec3) -> Vec3 {
    if matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat) {
        return piece.position;
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
pub fn apply(plan: &mut PerformancePlan, profile: &RigProfile) {
    let observer = mul(add(profile.roots[0], profile.roots[1]), 0.5);
    for hit in &mut plan.hits {
        if let Some(piece) = plan
            .kit
            .iter()
            .find(|p| p.id == hit.piece_id && p.geometry.is_some())
        {
            hit.target = target(piece, &hit.articulation, observer);
        }
    }
}
/// A restored plan may contain the legacy anchor or a measured region contact.
/// Do not permit arbitrary world-space offsets to bypass the kit contract.
pub fn valid_target(piece: &KitPiece, target: Vec3) -> bool {
    if distance(piece.position, target) < 1e-9 {
        return true;
    }
    if piece.geometry.is_none()
        || matches!(piece.instrument, Instrument::Kick | Instrument::PedalHat)
    {
        return false;
    }
    let (center, normal, radius) = surface(piece);
    let delta = sub(target, center);
    dot(delta, normal).abs() < 1e-6 && distance(delta, [0.; 3]) <= radius * 0.95 + 1e-9
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
