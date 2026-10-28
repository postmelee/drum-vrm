//! Immutable ready-pose feasibility for one calibrated Motion. Repeated strokes
//! use the previous contact's exact target, including articulation-specific
//! targets on the same instrument. No playback-time or global cache is involved.
use crate::{
    equipment_motion::EquipmentContext,
    model::{Hit, PerformancePlan, Vec3},
    rig::RigProfile,
};
use std::collections::BTreeMap;

type Targets = BTreeMap<String, BTreeMap<[u64; 3], Option<Vec3>>>;
#[derive(Default, Debug, PartialEq)]
pub(super) struct ReadyPoses {
    home: [Option<Vec3>; 2],
    repeated: [Targets; 2],
}

impl ReadyPoses {
    pub(super) fn for_plan(
        plan: &PerformancePlan,
        limbs: &[Vec<usize>; 4],
        profile: Option<&RigProfile>,
        equipment: Option<&EquipmentContext>,
    ) -> Self {
        let Some(profile) = profile else {
            return Self::default();
        };
        let center =
            std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
        let mut cache = Self::default();
        for (limb, indices) in limbs.iter().enumerate().take(2) {
            let (home, articulation) = if limb == 0 {
                ("snare", "center")
            } else {
                ("hihat", "closed")
            };
            cache.home[limb] = plan
                .kit
                .iter()
                .find(|piece| piece.id == home)
                .and_then(|piece| {
                    let target = crate::contact::target(piece, articulation, center);
                    solve(profile, equipment, limb, target, home)
                });
            for pair in indices.windows(2) {
                let hit = &plan.hits[pair[0]];
                if hit.piece_id != plan.hits[pair[1]].piece_id {
                    continue;
                }
                cache.repeated[limb]
                    .entry(hit.piece_id.clone())
                    .or_default()
                    .entry(hit.target.map(f64::to_bits))
                    .or_insert_with(|| solve(profile, equipment, limb, hit.target, &hit.piece_id));
            }
        }
        cache
    }
    pub(super) fn home(&self, limb: usize) -> Option<Vec3> {
        self.home[limb]
    }
    pub(super) fn repeated(&self, limb: usize, hit: &Hit) -> Option<Vec3> {
        self.repeated[limb]
            .get(hit.piece_id.as_str())
            .and_then(|targets| targets.get(&hit.target.map(f64::to_bits)))
            .copied()
            .flatten()
    }
}

fn solve(
    profile: &RigProfile,
    equipment: Option<&EquipmentContext>,
    limb: usize,
    target: Vec3,
    intended: &str,
) -> Option<Vec3> {
    let center =
        std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
    let side = crate::motion::sub(profile.roots[limb], center);
    let width = side[0].hypot(side[2]).max(1e-9);
    let ready = crate::motion::add(
        target,
        [side[0] / width * 0.035, 0.035, side[2] / width * 0.035],
    );
    let reachable = if profile.hand_frames.is_some() && equipment.is_some() {
        crate::contact_feasibility::candidate(profile, limb, ready, 0., equipment, Some(intended))
            .is_some()
    } else {
        profile.reachable(limb, ready)
    };
    reachable.then_some(ready)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan() -> PerformancePlan {
        let track = [
            0, 0x99, 38, 100, 0x83, 0x60, 0x99, 38, 100, 0x83, 0x60, 0x99, 38, 100, 0x83, 0x60,
            0x99, 38, 100, 0x83, 0x60, 0x99, 38, 100, 0x83, 0x60, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        crate::build_plan(
            &bytes,
            &crate::model::Selection {
                tracks: vec![0],
                ..crate::model::Selection::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn repeated_contacts_keep_distinct_targets_and_negative_results() {
        let mut plan = plan();
        let targets = [
            [-0.04, 0.95, 0.5],
            [0.04, 0.95, 0.5],
            [-0.04, 0.95, 0.5],
            [8., 0.95, 8.],
            [-0.04, 0.95, 0.5],
        ];
        for (hit, target) in plan.hits.iter_mut().zip(targets) {
            hit.target = target;
            hit.limb = Some(0);
        }
        let limbs = [vec![0, 1, 2, 3, 4], vec![], vec![], vec![]];
        let profile = RigProfile::primitive();
        let cache = ReadyPoses::for_plan(&plan, &limbs, Some(&profile), None);
        assert_eq!(cache.repeated[0]["snare"].len(), 3);
        assert_eq!(
            cache.repeated(0, &plan.hits[0]),
            Some([-0.04 - 0.035, 0.95 + 0.035, 0.5])
        );
        assert_eq!(
            cache.repeated(0, &plan.hits[1]),
            Some([0.04 - 0.035, 0.95 + 0.035, 0.5])
        );
        assert_eq!(cache.repeated(0, &plan.hits[3]), None);
        // A different calibration cannot reuse the previous Motion's witness.
        let mut distant = profile.clone();
        for root in &mut distant.roots[..2] {
            root[2] -= 5.;
        }
        let other = ReadyPoses::for_plan(&plan, &limbs, Some(&distant), None);
        assert_eq!(other.repeated(0, &plan.hits[0]), None);
        assert!(cache.repeated(0, &plan.hits[0]).is_some());
        assert_eq!(
            ReadyPoses::for_plan(&plan, &limbs, None, None),
            ReadyPoses::default()
        );
    }

    #[test]
    fn prepared_restore_rebuilds_ready_cache_without_order_dependent_state() {
        let original = super::super::Motion::with_profile(plan(), RigProfile::primitive()).unwrap();
        assert!(!original.ready_poses.repeated[0].is_empty());
        let profile = serde_json::to_string(original.profile.as_ref().unwrap()).unwrap();
        let restored = super::super::Motion::from_prepared_json(
            &original.prepared_json(),
            &original.plan.midi_hash,
            &profile,
        )
        .unwrap();
        assert_eq!(original.ready_poses, restored.ready_poses);
        for time in [2.5, -0.5, 0.0, 1.25, 0.75, 1.5, 2.5, 0.0] {
            for limb in 0..4 {
                assert_eq!(
                    original.raw_stroke_with_piece(limb, time),
                    restored.raw_stroke_with_piece(limb, time)
                );
            }
            assert_eq!(original.evaluate_rig(time), restored.evaluate_rig(time));
        }
    }
}
