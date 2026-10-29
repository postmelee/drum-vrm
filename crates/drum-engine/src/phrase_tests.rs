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

fn authored_sequence(events: &[(u32, u8, u8)]) -> PerformancePlan {
    let mut track = Vec::new();
    for &(delta, note, velocity) in events {
        track.extend(vlq(delta));
        track.extend([0x90, note, velocity]);
    }
    short_plan(&track)
}

#[test]
fn rapid_toms_and_mixed_fills_use_available_hands_without_retiming() {
    for notes in [
        vec![48; 24],
        vec![45; 24],
        vec![43; 24],
        (0..24).map(|index| [38, 48, 45, 43][index / 6]).collect(),
    ] {
        let events: Vec<_> = notes
            .iter()
            .enumerate()
            .map(|(index, &note)| (if index == 0 { 0 } else { 80 }, note, 95))
            .collect();
        let mut source = authored_sequence(&events);
        // Authored compact target fixture isolates sticking from the physical
        // default kit. Each tom is intentionally reachable by either hand.
        for piece in &mut source.kit {
            match piece.id.as_str() {
                "tomHigh" => piece.position = [-0.20, 0.95, 0.65],
                "tomMid" => piece.position = [0.20, 0.93, 0.65],
                "tomLow" => piece.position = [0.36, 0.80, 0.42],
                _ => {}
            }
        }
        for hit in &mut source.hits {
            hit.target = source
                .kit
                .iter()
                .find(|piece| piece.id == hit.piece_id)
                .unwrap()
                .position;
        }
        let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
        let hands: Vec<_> = motion
            .plan
            .hits
            .iter()
            .map(|hit| hit.limb.unwrap())
            .collect();
        assert!(max_run(&hands) <= 2, "notes={notes:?}, hands={hands:?}");
        assert!(hands.contains(&0) && hands.contains(&1));
        for (before, after) in source.hits.iter().zip(&motion.plan.hits) {
            assert_eq!(before.time_us, after.time_us);
            assert_eq!(before.source_event_id, after.source_event_id);
            assert_eq!(before.velocity, after.velocity);
        }
    }
}

/// Original native geometry supplies measured hand/tool frames; positions come
/// from the current shared Studio layout, not the compact sticking-only fixture.
fn reference_native_phrase(events: &[(u32, u8, u8)]) -> (PerformancePlan, RigProfile) {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let measured: Vec<crate::model::KitPiece> =
        serde_json::from_value(fixture["kit"].clone()).unwrap();
    let mut plan = authored_sequence(events);
    plan.kit = crate::model::default_kit();
    for piece in &mut plan.kit {
        piece.geometry = measured
            .iter()
            .find(|reference| reference.id == piece.id)
            .unwrap()
            .geometry
            .clone();
    }
    crate::contact::apply(&mut plan, &profile);
    (plan, profile)
}

#[test]
fn reference_rack_tom_fill_uses_both_hands_at_actual_head_contacts() {
    // Independently authored six-note phrase: two snare singles followed by
    // four rapid high-tom singles. Feet never consume the other hand. A raised
    // ready-pose certificate alone must not hide an unreachable impact.
    let (source, profile) = reference_native_phrase(&[
        (0, 38, 90),
        (80, 38, 80),
        (80, 50, 90),
        (80, 50, 80),
        (80, 50, 90),
        (80, 50, 80),
    ]);
    let target = source.hits[2].target;
    let equipment = crate::equipment_motion::EquipmentContext::for_kit(&source.kit);
    for limb in 0..2 {
        assert!(
            crate::contact_feasibility::hand_probe_fits(
                &profile,
                limb,
                target,
                equipment.as_deref(),
                Some("tomHigh")
            ),
            "reference high-tom contact/preparation is unavailable to hand {limb}"
        );
    }
    let mut planned = source.clone();
    crate::motion::assign_measured_contacts_for_test(&mut planned, &profile);
    let hands: Vec<_> = planned.hits[2..]
        .iter()
        .map(|hit| hit.limb.expect("all four tom hits remain assigned"))
        .collect();
    assert!(hands.contains(&0) && hands.contains(&1), "{hands:?}");
    assert!(max_run(&hands) <= 2, "{hands:?}");
    for (before, after) in source.hits.iter().zip(&planned.hits) {
        assert_eq!(before.time_us, after.time_us);
        assert_eq!(before.source_event_id, after.source_event_id);
        assert_eq!(before.velocity, after.velocity);
    }
}

#[test]
fn reference_simultaneous_snare_and_floor_tom_keep_independent_hands() {
    // A repeated two-surface chord is not an unaccompanied tom fill. Forcing
    // tom alternation here would steal the simultaneous snare's hand.
    let events: Vec<_> = (0..6)
        .flat_map(|index| {
            let velocity = if index % 2 == 0 { 75 } else { 105 };
            [
                (if index == 0 { 0 } else { 160 }, 38, velocity),
                (0, 43, velocity),
            ]
        })
        .collect();
    let (source, profile) = reference_native_phrase(&events);
    let mut planned = source.clone();
    crate::motion::assign_measured_contacts_for_test(&mut planned, &profile);
    for pair in planned.hits.chunks_exact(2) {
        assert_eq!(pair[0].time_us, pair[1].time_us);
        assert_eq!(pair[0].limb, Some(0));
        assert_eq!(pair[1].limb, Some(1));
    }
    for (before, after) in source.hits.iter().zip(&planned.hits) {
        assert_eq!(before.time_us, after.time_us);
        assert_eq!(before.source_event_id, after.source_event_id);
        assert_eq!(before.velocity, after.velocity);
    }
}

fn reachable_crashes(plan: &mut PerformancePlan) {
    // An authored compact kit makes the left cymbal reachable by either hand;
    // a preference must never make an unreachable default target look feasible.
    for (piece, target) in [
        ("crash", [-0.35, 1.12, 0.48]),
        ("crashRight", [0.38, 1.12, 0.48]),
    ] {
        plan.kit
            .iter_mut()
            .find(|entry| entry.id == piece)
            .unwrap()
            .position = target;
        for hit in plan.hits.iter_mut().filter(|hit| hit.piece_id == piece) {
            hit.target = target;
        }
    }
}

#[test]
fn crash_replacing_a_hat_beat_keeps_the_groove_hand() {
    let mut source = authored_sequence(&[
        (0, 42, 75),
        (240, 42, 70),
        (240, 49, 110),
        (240, 42, 75),
        (240, 42, 70),
    ]);
    reachable_crashes(&mut source);
    let motion = Motion::with_profile(source, RigProfile::primitive()).unwrap();
    assert!(
        motion.plan.hits.iter().all(|hit| hit.limb == Some(1)),
        "{:?}",
        motion
            .plan
            .hits
            .iter()
            .map(|hit| (hit.note, hit.limb))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_backbeat_with_a_hat_does_not_turn_the_next_crash_into_a_fill_exit() {
    for chord in [[38, 42], [42, 38]] {
        let mut source = authored_sequence(&[
            (0, 42, 75),
            (480, chord[0], 90),
            (0, chord[1], 90),
            (240, 49, 110),
            (240, 42, 75),
        ]);
        reachable_crashes(&mut source);
        let motion = Motion::with_profile(source, RigProfile::primitive()).unwrap();
        for hit in &motion.plan.hits {
            assert_eq!(hit.limb, Some(usize::from(hit.note != 38)));
        }
    }
}

#[test]
fn simultaneous_hat_crash_and_dual_crash_keep_both_original_impacts() {
    let mut source = authored_sequence(&[
        (0, 42, 75),
        (240, 42, 70),
        (0, 49, 110),
        (480, 49, 110),
        (0, 57, 115),
        (240, 42, 70),
    ]);
    reachable_crashes(&mut source);
    let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
    assert_eq!(motion.plan.hits.len(), source.hits.len());
    for (before, after) in source.hits.iter().zip(&motion.plan.hits) {
        assert_eq!(before.time_us, after.time_us);
        assert_eq!(before.source_event_id, after.source_event_id);
        assert!(after.limb.is_some());
    }
    for pair in [[1, 2], [3, 4]] {
        assert_ne!(
            motion.plan.hits[pair[0]].limb,
            motion.plan.hits[pair[1]].limb
        );
    }
    assert_eq!(motion.plan.hits[1].limb, Some(1));
    assert_eq!(motion.plan.hits[2].limb, Some(0));
    assert_eq!(motion.plan.hits[3].limb, Some(0));
    assert_eq!(motion.plan.hits[4].limb, Some(1));
}

#[test]
fn dual_crashes_favor_outward_hands_in_both_source_orders_after_a_hat_groove() {
    for notes in [[49, 57], [57, 49]] {
        let mut source = authored_sequence(&[
            (0, 42, 75),
            (240, 42, 75),
            (240, notes[0], 115),
            (0, notes[1], 115),
            (0, 36, 110),
            (240, 42, 75),
        ]);
        reachable_crashes(&mut source);
        let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
        for (before, after) in source.hits.iter().zip(&motion.plan.hits) {
            assert_eq!(before.time_us, after.time_us);
            assert_eq!(before.source_event_id, after.source_event_id);
            assert!(after.limb.is_some());
            if after.note == 49 {
                assert_eq!(after.limb, Some(0));
            }
            if after.note == 57 {
                assert_eq!(after.limb, Some(1));
            }
            if after.note == 36 {
                assert_eq!(after.limb, Some(3));
            }
        }
    }
}

#[test]
fn idle_hand_takes_a_distant_crash_after_a_fast_floor_tom_hit() {
    let mut source =
        authored_sequence(&[(0, 42, 75), (240, 43, 110), (80, 49, 120), (240, 42, 75)]);
    reachable_crashes(&mut source);
    let floor = [0.42, 1.05, 0.5];
    source
        .kit
        .iter_mut()
        .find(|piece| piece.id == "tomLow")
        .unwrap()
        .position = floor;
    source.hits[1].target = floor;
    source
        .hand_overrides
        .insert(source.hits[1].source_event_id.clone(), 1);
    let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
    assert_eq!(motion.plan.hits[1].limb, Some(1));
    assert_eq!(motion.plan.hits[2].limb, Some(0));
    assert!(motion.plan.hits.iter().all(|hit| hit.limb.is_some()));
    for (before, after) in source.hits.iter().zip(&motion.plan.hits) {
        assert_eq!(before.time_us, after.time_us);
        assert_eq!(before.velocity, after.velocity);
    }
}

#[test]
fn next_accent_changes_preparation_without_moving_either_contact() {
    let motion = |velocities: [u8; 2]| {
        let mut plan = authored_sequence(&[(0, 38, velocities[0]), (240, 38, velocities[1])]);
        // A stored feasible one-hand phrase isolates its procedural stroke from
        // the independent sticking optimizer, as manual overrides will do.
        for hit in &mut plan.hits {
            hit.limb = Some(0);
        }
        Motion::new(plan).unwrap()
    };
    let down_tap = motion([115, 45]);
    let tap_up = motion([45, 115]);
    let tip_y = |motion: &Motion, t| motion.evaluate_rig(t)[10];
    assert!(tip_y(&down_tap, 0.0825) > tip_y(&down_tap, 0.1675));
    assert!(tip_y(&tap_up, 0.1675) > tip_y(&tap_up, 0.0825));
    assert!(tip_y(&tap_up, 0.1675) > tip_y(&down_tap, 0.0825));
    for motion in [&down_tap, &tap_up] {
        for hit in &motion.plan.hits {
            let pose = motion.evaluate_rig(hit.time);
            for axis in 0..3 {
                assert!((pose[9 + axis] as f64 - hit.target[axis]).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn controlled_accent_tap_double_keeps_the_other_hand_on_its_groove() {
    let mut events = Vec::new();
    for index in 0..6 {
        events.push((if index == 0 { 0 } else { 80 }, 42, 75));
        events.push((0, 38, 110));
        events.push((80, 38, 45));
    }
    let source = authored_sequence(&events);
    let motion = Motion::with_profile(source.clone(), RigProfile::primitive()).unwrap();
    for group in motion.plan.hits.chunks(3) {
        assert_eq!(group[0].limb, Some(1));
        assert_eq!(group[1].limb, Some(0));
        assert_eq!(group[2].limb, Some(0));
    }
    for (before, after) in source.hits.iter().zip(&motion.plan.hits) {
        assert_eq!(before.velocity, after.velocity);
        assert_eq!(before.time_us, after.time_us);
    }
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

fn distant_crash_motion(events: &[(u32, u8, u8)]) -> Motion {
    let mut source = authored_sequence(events);
    for (id, target) in [
        ("crash", [-0.65, 1.45, 0.48]),
        ("crashRight", [0.42, 1.4, 0.48]),
    ] {
        source
            .kit
            .iter_mut()
            .find(|piece| piece.id == id)
            .unwrap()
            .position = target;
        for hit in source.hits.iter_mut().filter(|hit| hit.piece_id == id) {
            hit.target = target;
        }
    }
    for hit in &mut source.hits {
        hit.limb = None;
        source.hand_overrides.insert(hit.source_event_id.clone(), 0);
    }
    let original = serde_json::to_value(&source.source_events).unwrap();
    let motion = Motion::with_profile(source, RigProfile::primitive()).unwrap();
    assert!(motion.plan.hits.iter().all(|hit| hit.limb == Some(0)));
    assert_eq!(
        serde_json::to_value(&motion.plan.source_events).unwrap(),
        original
    );
    motion
}

fn left_tip(motion: &Motion, time: f64) -> [f64; 3] {
    let pose = motion.evaluate_rig(time);
    [pose[9] as f64, pose[10] as f64, pose[11] as f64]
}

fn assert_hand_continuous(motion: &Motion, time: f64) {
    let before = motion.evaluate_rig(time - 1e-5);
    let after = motion.evaluate_rig(time + 1e-5);
    for index in (0..12).chain(57..61) {
        assert!(
            (after[index] - before[index]).abs() < 1e-4,
            "jump at {time}s, component {index}: {} -> {}",
            before[index],
            after[index]
        );
    }
}

#[test]
fn final_distant_crash_recovers_gradually_then_settles_over_the_snare() {
    let motion = distant_crash_motion(&[(480, 49, 115)]);
    let hit = &motion.plan.hits[0];
    let settled = left_tip(&motion, hit.time + 0.7);
    assert!(crate::motion::distance(left_tip(&motion, hit.time), hit.target) < 1e-6);
    // A large cross-body return must not already be over at the old 180 ms cutoff.
    assert!(crate::motion::distance(left_tip(&motion, hit.time + 0.19), settled) > 0.05);
    assert_eq!(settled, left_tip(&motion, hit.time + 0.8));
    for index in 1..700 {
        assert_hand_continuous(&motion, hit.time + index as f64 / 1000.0);
    }
    let ready = motion
        .plan
        .kit
        .iter()
        .find(|piece| piece.id == "snare")
        .unwrap()
        .position;
    assert!((settled[1] - ready[1] - 0.035).abs() < 1e-6);
    // Recover above the snare into the left hand's own ready corridor,
    // leaving the shared contact column available to an alternating hand.
    assert!(settled[0] < ready[0]);
    assert!(((settled[0] - ready[0]).hypot(settled[2] - ready[2]) - 0.035).abs() < 1e-6);
}

#[test]
fn separated_recovery_and_preparation_join_the_idle_pose_without_a_jump() {
    let motion = distant_crash_motion(&[(480, 49, 115), (960, 38, 75)]);
    let first = &motion.plan.hits[0];
    let second = &motion.plan.hits[1];
    let idle = left_tip(&motion, first.time + 0.6);
    assert_eq!(idle, left_tip(&motion, first.time + 0.7));
    for index in 1..1000 {
        assert_hand_continuous(&motion, first.time + index as f64 / 1000.0);
    }
    for hit in [first, second] {
        assert!(crate::motion::distance(left_tip(&motion, hit.time), hit.target) < 1e-6);
    }
    assert_eq!(motion.plan.hits[0].time_us, 500000);
    assert_eq!(motion.plan.hits[1].time_us, 1500000);
}

#[test]
fn overlapping_long_transfer_windows_do_not_switch_to_an_unfinished_ready_pose() {
    let motion = distant_crash_motion(&[(480, 49, 115), (480, 57, 100)]);
    let first = &motion.plan.hits[0];
    let second = &motion.plan.hits[1];
    for index in 1..500 {
        assert_hand_continuous(&motion, first.time + index as f64 / 1000.0);
    }
    for hit in [first, second] {
        assert!(crate::motion::distance(left_tip(&motion, hit.time), hit.target) < 1e-6);
    }
    assert_eq!(second.time_us - first.time_us, 500000);
}
