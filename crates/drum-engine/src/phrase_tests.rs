//! Authored phrase cases verify observable scheduling, rather than a particular
//! cost formula or the contents of a user's MIDI file.
use crate::interpretation::MappingProfile;
use crate::model::{PerformancePlan, Selection};
use crate::motion::Motion;
use crate::{build_plan, load_motion, rig::RigProfile};

fn vlq(mut value: u32) -> Vec<u8> {
    let mut out = vec![(value & 127) as u8];
    while value > 127 {
        value >>= 7;
        out.push((value as u8 & 127) | 128);
    }
    out.reverse();
    out
}
fn fixture(with_hat: bool) -> Vec<u8> {
    let mut track = vec![0, 0xff, 0x51, 3, 0x04, 0xf9, 0xc7]; // 326087 us/q, approximately184 BPM.
    for index in 0..24 {
        track.extend(vlq(if index == 0 { 0 } else { 120 }));
        track.extend([
            0x90,
            if index % 3 == 0 { 40 } else { 38 },
            if index % 2 == 0 { 110 } else { 60 },
        ]);
        if with_hat && index % 2 == 0 {
            track.extend([0, 0x90, 42, 72]);
        }
    }
    track.extend(vlq(480));
    track.extend([0xff, 0x2f, 0]);
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    bytes.extend((track.len() as u32).to_be_bytes());
    bytes.extend(track);
    bytes
}
fn plan(with_hat: bool) -> PerformancePlan {
    build_plan(
        &fixture(with_hat),
        &Selection {
            tracks: vec![0],
            mapping_profile: MappingProfile::LogicDkdGm,
            ..Selection::default()
        },
    )
    .unwrap()
}
fn max_run(limbs: &[usize]) -> usize {
    let mut longest = 0;
    let mut run = 0;
    let mut previous = None;
    for &limb in limbs {
        run = if previous == Some(limb) { run + 1 } else { 1 };
        longest = longest.max(run);
        previous = Some(limb);
    }
    longest
}

#[test]
fn unaccompanied_fast_snare_uses_both_hands_with_original_accents_and_times() {
    let source = plan(false);
    let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
    assert!(motion.plan.hits.iter().all(|hit| hit.limb.is_some()));
    let hands: Vec<_> = motion
        .plan
        .hits
        .iter()
        .map(|hit| hit.limb.unwrap())
        .collect();
    assert!(
        hands.iter().filter(|&&hand| hand == 0).count() >= 8,
        "{hands:?}"
    );
    assert!(
        hands.iter().filter(|&&hand| hand == 1).count() >= 8,
        "{hands:?}"
    );
    assert!(max_run(&hands) <= 2, "{hands:?}");
    for (index, hit) in motion.plan.hits.iter().enumerate() {
        assert_eq!(hit.tick, index as u64 * 120);
        assert_eq!(hit.time_us, (hit.tick * 326087 + 240) / 480);
        assert_eq!(hit.velocity, if index % 2 == 0 { 110 } else { 60 });
        assert_eq!(hit.source_event_id, source.hits[index].source_event_id);
        assert_eq!(hit.articulation, source.hits[index].articulation);
    }
}

#[test]
fn occupied_other_hand_is_respected_without_dropping_a_feasible_roll() {
    let source = plan(true);
    let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
    assert!(
        motion.plan.hits.iter().all(|hit| hit.limb.is_some()),
        "{:?}",
        motion.plan.diagnostics
    );
    assert_eq!(motion.plan.hits.len(), 36);
    for (index, hit) in motion.plan.hits.iter().enumerate() {
        assert_eq!(hit.time_us, source.hits[index].time_us);
        assert_eq!(hit.velocity, source.hits[index].velocity);
        for other in motion
            .plan
            .hits
            .iter()
            .filter(|other| other.id != hit.id && other.time_us == hit.time_us)
        {
            assert_ne!(hit.limb, other.limb);
        }
    }
    for hand in 0..2 {
        let assigned: Vec<_> = motion
            .plan
            .hits
            .iter()
            .filter(|hit| hit.limb == Some(hand))
            .collect();
        for pair in assigned.windows(2) {
            assert!(pair[1].time - pair[0].time >= 0.045 - 1e-9);
        }
    }
}

#[test]
fn phrase_replanning_and_shuffled_seeks_survive_round_trip() {
    let source = plan(true);
    let profile = RigProfile::primitive();
    let motion = Motion::with_profile(source.clone(), profile.clone()).unwrap();
    let repeated = Motion::with_profile(source, profile.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&motion.plan).unwrap(),
        serde_json::to_value(&repeated.plan).unwrap()
    );
    let restored = load_motion(
        &serde_json::to_string(&motion.plan).unwrap(),
        Some(&serde_json::to_string(&profile).unwrap()),
    )
    .unwrap();
    let times = [-0.5, -0.1, 0.0, 0.03, 0.081522, 0.45, 0.98, 1.43, 1.9, 2.1];
    let forward: Vec<_> = times
        .iter()
        .map(|&time| motion.evaluate_rig(time))
        .collect();
    for index in [8, 0, 4, 2, 9, 1, 6, 7, 3, 5] {
        assert_eq!(forward[index], restored.evaluate_rig(times[index]));
        assert!(forward[index].iter().all(|value| value.is_finite()));
    }
}

fn short_plan(events: &[u8]) -> PerformancePlan {
    let mut track = events.to_vec();
    track.extend([0x83, 0x60, 0xff, 0x2f, 0]);
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    bytes.extend((track.len() as u32).to_be_bytes());
    bytes.extend(track);
    build_plan(
        &bytes,
        &Selection {
            tracks: vec![0],
            ..Selection::default()
        },
    )
    .unwrap()
}

#[test]
fn restored_assignments_cannot_use_a_hand_for_a_kick_or_a_foot_for_a_snare() {
    for (note, limb, target) in [(36, 0, [-0.2, 1.0, 0.45]), (38, 3, [0.20, 0.105, 0.62])] {
        let mut plan = short_plan(&[0, 0x90, note, 80]);
        let hit = &mut plan.hits[0];
        hit.limb = Some(limb);
        hit.target = target;
        plan.kit
            .iter_mut()
            .find(|piece| piece.id == hit.piece_id)
            .unwrap()
            .position = target;
        assert!(load_motion(&serde_json::to_string(&plan).unwrap(), None).is_err());
    }
}

#[test]
fn restored_assignments_cannot_bypass_interval_or_transfer_limits() {
    let mut fast = short_plan(&[0, 0x90, 38, 80, 1, 0x90, 38, 80]);
    assert!(load_motion(&serde_json::to_string(&fast).unwrap(), None).is_ok());
    for hit in &mut fast.hits {
        hit.limb = Some(0);
    }
    assert!(load_motion(&serde_json::to_string(&fast).unwrap(), None).is_err());

    let mut transfer = short_plan(&[0, 0x90, 49, 80, 48, 0x90, 57, 80]);
    for (hit, target) in transfer
        .hits
        .iter_mut()
        .zip([[-0.45, 1.0, 0.45], [0.45, 1.0, 0.45]])
    {
        hit.target = target;
        hit.limb = Some(0);
        transfer
            .kit
            .iter_mut()
            .find(|piece| piece.id == hit.piece_id)
            .unwrap()
            .position = target;
    }
    assert!(load_motion(&serde_json::to_string(&transfer).unwrap(), None).is_err());
    // Both targets are reachable individually, and the exact same source can
    // satisfy the transfer using one hand per cymbal without changing timing.
    transfer.hits[1].limb = Some(1);
    assert!(load_motion(&serde_json::to_string(&transfer).unwrap(), None).is_ok());
}
