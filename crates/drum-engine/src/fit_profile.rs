//! Candidate seating and pedal frames shared by every static fit branch.
//! Local measured envelopes remain local; only world references are translated.
use crate::{
    equipment,
    model::{Instrument, KitPiece, Vec3},
    motion::{distance, sub},
    rig::RigProfile,
};

pub(super) fn candidate(profile: &RigProfile, offset: Vec3, kit: &[KitPiece]) -> RigProfile {
    let mut p = profile.clone();
    let translate = |point: &mut Vec3| {
        for (value, delta) in point.iter_mut().zip(offset) {
            *value += delta;
        }
    };
    for root in &mut p.roots {
        translate(root);
    }
    for tip in &mut p.rest_tips[..2] {
        translate(tip);
    }
    for point in [&mut p.pelvis, &mut p.chest, &mut p.head] {
        translate(point);
    }
    if let Some(pivot) = &mut p.torso_pivot {
        translate(pivot);
    }
    if let Some(body) = &mut p.body_geometry {
        translate(&mut body.torso.center);
    }
    for (limb, instrument) in [(2, Instrument::PedalHat), (3, Instrument::Kick)] {
        if let Some(pedal) = kit.iter().find(|piece| piece.instrument == instrument) {
            p.rest_tips[limb] = pedal.position;
        }
    }
    // Match HumanoidRigAdapter.setCalibration: the accepted heel-to-toe axis
    // controls the left knee plane. Foot rotations remain neutral; layout yaw
    // is composed exactly once by equipment::pedal_rotation during evaluation.
    p.bend_poles[2] = kit
        .iter()
        .find(|piece| piece.instrument == Instrument::PedalHat)
        .map_or([0., 0., 1.], equipment::pedal_forward);
    p
}

pub(super) fn foot_fits(profile: &RigProfile, piece: &KitPiece, limb: usize) -> bool {
    if !(2..4).contains(&limb) {
        return false;
    }
    [0., -0.09, -0.18].into_iter().all(|pitch| {
        let rotation = equipment::pedal_rotation(piece, profile.contact_rotations[limb], pitch);
        let ankle = sub(
            equipment::pedal_contact(piece, pitch),
            profile.offset(limb, rotation),
        );
        let distance = distance(profile.roots[limb], ankle);
        let [a, b] = profile.lengths[limb];
        let knee = ((a * a + b * b - distance * distance) / (2. * a * b))
            .clamp(-1., 1.)
            .acos();
        distance >= (a - b).abs() + 1e-5
            && distance <= a + b - 1e-4
            && (55_f64.to_radians()..=155_f64.to_radians()).contains(&knee)
    })
}

pub(super) fn feet_fit(profile: &RigProfile, kit: &[KitPiece]) -> bool {
    [(2, Instrument::PedalHat), (3, Instrument::Kick)]
        .into_iter()
        .all(|(limb, instrument)| {
            kit.iter()
                .find(|piece| piece.instrument == instrument)
                .is_none_or(|piece| foot_fits(profile, piece, limb))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig::{BodyGeometry, TorsoGeometry};

    #[test]
    fn seating_moves_world_references_without_moving_local_measurements() {
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0., 0.8, 0.1]);
        profile.contact_rotations[2] = [0.6, 0., 0., 0.8];
        let leg_box = TorsoGeometry {
            center: [0.025, 0.2, -0.045],
            half_extents: [0.04, 0.16, 0.075],
        };
        profile.body_geometry = Some(BodyGeometry {
            version: 1,
            source: "mesh-estimate".into(),
            torso: TorsoGeometry {
                center: [0., 1., 0.1],
                half_extents: [0.15, 0.2, 0.1],
            },
            arm_radii: [[0.03, 0.025]; 2],
            hand_radii: [0.025; 2],
            leg_radii: Some([[0.055, 0.04]; 2]),
            foot_radii: Some([0.04; 2]),
            leg_boxes: Some([
                [leg_box.clone(), leg_box.clone()],
                [leg_box.clone(), leg_box],
            ]),
            // Non-identity rest frames catch accidental neutralization.
            leg_rest_frames: Some([[[0.6, 0., 0., 0.8]; 2]; 2]),
            foot_boxes: Some([
                TorsoGeometry {
                    center: [0.01, -0.035, 0.08],
                    half_extents: [0.045, 0.03, 0.12],
                },
                TorsoGeometry {
                    center: [-0.01, -0.04, 0.09],
                    half_extents: [0.05, 0.035, 0.13],
                },
            ]),
            hand_centers: Some([[0.01, 0., 0.02]; 2]),
            hand_capsules: None,
            sample_count: 100,
        });
        let source = serde_json::to_string(&profile).unwrap();
        let kit = crate::model::default_kit();
        let offset = [0., 0.06, 0.08];
        let moved = candidate(&profile, offset, &kit);
        let shifted = |point: Vec3| std::array::from_fn(|axis| point[axis] + offset[axis]);
        for limb in 0..4 {
            assert_eq!(moved.roots[limb], shifted(profile.roots[limb]));
        }
        for limb in 0..2 {
            assert_eq!(moved.rest_tips[limb], shifted(profile.rest_tips[limb]));
        }
        for (old, new) in [
            (profile.pelvis, moved.pelvis),
            (profile.chest, moved.chest),
            (profile.head, moved.head),
            (profile.torso_pivot.unwrap(), moved.torso_pivot.unwrap()),
        ] {
            assert_eq!(new, shifted(old));
        }
        let mut expected_body = profile.body_geometry.clone().unwrap();
        expected_body.torso.center = shifted(expected_body.torso.center);
        assert_eq!(
            serde_json::to_value(moved.body_geometry.as_ref().unwrap()).unwrap(),
            serde_json::to_value(expected_body).unwrap()
        );
        assert_eq!(moved.lengths, profile.lengths);
        assert_eq!(moved.grip_offsets, profile.grip_offsets);
        assert_eq!(moved.tip_offsets, profile.tip_offsets);
        assert_eq!(moved.contact_rotations, profile.contact_rotations);
        assert_eq!(serde_json::to_string(&profile).unwrap(), source);
    }

    #[test]
    fn each_candidate_pedal_updates_only_the_left_knee_direction_and_foot_targets() {
        let mut profile = RigProfile::primitive();
        profile.bend_poles[2] = [0.6, 0., 0.8];
        profile.bend_poles[3] = [-0.6, 0., 0.8];
        let mut kit = crate::model::default_kit();
        let index = kit
            .iter()
            .position(|piece| piece.instrument == Instrument::PedalHat)
            .unwrap();
        kit[index].pedal_forward = Some([-0.8, 0., 0.6]);
        kit[index].position = [-0.3, 0.08, 0.5];
        let moved = candidate(&profile, [0.; 3], &kit);
        assert_eq!(moved.bend_poles[2], [-0.8, 0., 0.6]);
        assert_eq!(moved.bend_poles[3], profile.bend_poles[3]);
        assert_eq!(moved.bend_poles[..2], profile.bend_poles[..2]);
        assert_eq!(moved.contact_rotations, profile.contact_rotations);
        assert_eq!(moved.rest_tips[2], kit[index].position);
        assert_eq!(
            moved.rest_tips[3],
            kit.iter()
                .find(|piece| piece.instrument == Instrument::Kick)
                .unwrap()
                .position
        );
        kit[index].pedal_forward = None;
        assert_eq!(
            candidate(&profile, [0.; 3], &kit).bend_poles[2],
            [0., 0., 1.]
        );
        assert_eq!(
            candidate(&profile, [0.; 3], &[]).bend_poles[2],
            [0., 0., 1.]
        );
    }

    #[test]
    fn foot_reach_uses_the_same_yawed_sole_frame_as_the_board() {
        let mut profile = RigProfile::primitive();
        profile.roots[2] = [-0.25, 0.43, 0.];
        profile.lengths[2] = [0.25, 0.25];
        profile.grip_offsets[2] = [0.; 3];
        profile.tip_offsets[2] = [0., 0., 0.25];
        profile.contact_rotations[2] = [0., 0., 0., 1.];
        let mut pedal = crate::model::default_kit()
            .into_iter()
            .find(|piece| piece.instrument == Instrument::PedalHat)
            .unwrap();
        pedal.position = [0., 0.08, 0.];
        pedal.pedal_forward = Some([1., 0., 0.]);
        assert!(foot_fits(&profile, &pedal, 2));
        assert!(feet_fit(&profile, std::slice::from_ref(&pedal)));
        pedal.pedal_forward = None;
        assert!(!foot_fits(&profile, &pedal, 2));
        assert!(!feet_fit(&profile, std::slice::from_ref(&pedal)));
        assert!(!foot_fits(&profile, &pedal, 0));
        assert!(!foot_fits(&profile, &pedal, 4));
    }
}
