//! Short, independently authored state transitions rather than uploaded MIDI.
use super::*;
use crate::interpretation::{self, InterpretationKind, MappingProfile};
use crate::model::{PerformancePlan, Selection};
use crate::{build_plan, load_motion};

fn file(events: &[&[u8]]) -> Vec<u8> {
    let mut track = events.concat();
    track.extend([0, 0xff, 0x2f, 0]);
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    bytes.extend((track.len() as u32).to_be_bytes());
    bytes.extend(track);
    bytes
}
fn selected(policy: HihatPolicy) -> Selection {
    Selection {
        tracks: vec![0],
        mapping_profile: MappingProfile::LogicDkdGm,
        hihat_policy: policy,
        ..Selection::default()
    }
}
fn plan(events: &[&[u8]]) -> PerformancePlan {
    build_plan(&file(events), &selected(HihatPolicy::ShortOpenClose)).unwrap()
}

#[test]
fn explicit_policy_preserves_open_and_close_provenance_without_extra_foot_hit() {
    let bytes = file(&[
        &[0, 0x90, 46, 83],
        &[30, 0x90, 44, 83],
        &[120, 0x90, 38, 90],
    ]);
    let literal = build_plan(&bytes, &selected(HihatPolicy::Literal)).unwrap();
    let interpreted = build_plan(&bytes, &selected(HihatPolicy::ShortOpenClose)).unwrap();
    assert_eq!(literal.hits.len(), 3);
    assert_eq!(interpreted.hits.len(), 2);
    assert_eq!(
        serde_json::to_value(&literal.source_events).unwrap(),
        serde_json::to_value(&interpreted.source_events).unwrap()
    );
    assert_eq!(interpreted.controls.len(), 1);
    assert_eq!(interpreted.controls[0].source_event_id, "0:1");
    assert_eq!(interpreted.controls[0].open_source_event_id, "0:0");
    assert_eq!(interpreted.controls[0].release_start_time_us, -100_000);
    assert_eq!(interpreted.controls[0].open_time_us, 0);
    assert_eq!(interpreted.controls[0].time_us, 31_250);
    assert_eq!(
        interpreted.interpretations[1].kind,
        InterpretationKind::Control
    );
    assert_eq!(
        interpreted.interpretations[1]
            .related_source_event_id
            .as_deref(),
        Some("0:0")
    );
    assert_eq!(interpreted.interpretations[1].hit_id, None);
    assert_eq!(interpreted.hits[1].id, 1);
    assert_eq!(interpreted.interpretations[2].hit_id, Some(1));
    assert_eq!(interpreted.hits[1].time_us, literal.hits[2].time_us);
    interpretation::validate_interpretation(&interpreted).unwrap();
    load_motion(&serde_json::to_string(&interpreted).unwrap(), None).unwrap();
}

#[test]
fn nearest_open_same_channel_and_closed_interruption_bound_pairing() {
    let repeated = plan(&[&[0, 0x90, 46, 80], &[10, 0x90, 46, 70], &[10, 0x90, 44, 60]]);
    assert_eq!(repeated.controls[0].open_source_event_id, "0:1");
    let interrupted = plan(&[&[0, 0x90, 46, 80], &[10, 0x90, 42, 70], &[10, 0x90, 44, 60]]);
    assert!(interrupted.controls.is_empty());
    let other_channel = plan(&[&[0, 0x90, 46, 80], &[10, 0x91, 44, 60]]);
    assert!(other_channel.controls.is_empty());
    let simultaneous = plan(&[&[0, 0x90, 46, 80], &[0, 0x90, 44, 60]]);
    assert!(simultaneous.controls.is_empty());
}

#[test]
fn same_channel_controls_can_span_selected_tracks_with_stable_source_ids() {
    let open_track = file(&[&[0, 0x90, 46, 80]]);
    let close_track = file(&[&[30, 0x90, 44, 60]]);
    let mut bytes = b"MThd\0\0\0\x06\0\x01\0\x02\x01\xe0".to_vec();
    bytes.extend(&open_track[14..]);
    bytes.extend(&close_track[14..]);
    let mut selection = selected(HihatPolicy::ShortOpenClose);
    selection.tracks = vec![1, 0];
    let plan = build_plan(&bytes, &selection).unwrap();
    assert_eq!(plan.controls.len(), 1);
    assert_eq!(plan.controls[0].open_source_event_id, "0:0");
    assert_eq!(plan.controls[0].source_event_id, "1:0");
    assert_eq!(plan.hits.len(), 1);
    interpretation::validate_interpretation(&plan).unwrap();
}

#[test]
fn gap_bound_uses_seconds_and_quarter_duration_at_the_open() {
    let slow = plan(&[&[0, 0x90, 46, 80], &[79, 0x90, 44, 60]]); //82.292ms at120 BPM.
    assert!(slow.controls.is_empty());
    let fast = plan(&[
        &[0, 0xff, 0x51, 3, 0x03, 0x0d, 0x40],
        &[0, 0x90, 46, 80],
        &[121, 0x90, 44, 60],
    ]); //200000us/q; >quarter/4.
    assert!(fast.controls.is_empty());
    let accepted = plan(&[
        &[0, 0xff, 0x51, 3, 0x03, 0x0d, 0x40],
        &[0, 0x90, 46, 80],
        &[120, 0x90, 44, 60],
    ]);
    assert_eq!(accepted.controls.len(), 1);
    assert_eq!(accepted.controls[0].time_us, 50_000);
}

#[test]
fn openness_is_continuous_bounded_and_independent_of_seek_order() {
    let plan = plan(&[
        &[0, 0x90, 46, 80],
        &[30, 0x90, 44, 60],
        &[60, 0x90, 46, 70],
        &[30, 0x90, 44, 50],
    ]);
    assert_eq!(plan.controls.len(), 2);
    assert_eq!(evaluate(&plan.controls, -0.1), 0.0);
    assert_eq!(evaluate(&plan.controls, 0.0), 0.2);
    assert_eq!(evaluate(&plan.controls, 0.03125), 0.0);
    assert_eq!(evaluate(&plan.controls, 0.09375), 0.2);
    assert_eq!(evaluate(&plan.controls, 0.125), 0.0);
    for t in [-0.2, -0.05, 0.01, 0.03125, 0.07, 0.09375, 0.12, 0.3] {
        let expected = evaluate(&plan.controls, t);
        assert!((0.0..=0.2).contains(&expected));
        evaluate(&plan.controls, t + 0.1);
        evaluate(&plan.controls, t - 1.0);
        assert_eq!(expected, evaluate(&plan.controls, t));
        assert!((evaluate(&plan.controls, t + 1e-9) - expected).abs() < 1e-7);
    }
}

#[test]
fn competing_channels_keep_the_conflicting_close_as_a_literal_strike() {
    let plan = plan(&[
        &[0, 0x90, 46, 80],
        &[5, 0x91, 46, 70],
        &[5, 0x90, 44, 60],
        &[5, 0x91, 44, 60],
    ]);
    assert_eq!(plan.controls.len(), 1);
    assert_eq!(plan.controls[0].source_event_id, "0:2");
    assert!(plan.hits.iter().any(|hit| hit.source_event_id == "0:3"));
    assert!(plan
        .diagnostics
        .iter()
        .any(|d| d.code == "HIHAT_INTENT_CONFLICT"));
    interpretation::validate_interpretation(&plan).unwrap();
}

#[test]
fn literal_closure_bounds_the_next_release_and_keeps_its_foot_contact() {
    for close_note in [33, 42, 44] {
        let plan = plan(&[
            &[0, 0x90, close_note, 80],
            &[48, 0x90, 46, 80],
            &[24, 0x90, 44, 60],
        ]);
        assert_eq!(plan.controls.len(), 1);
        assert_eq!(plan.controls[0].release_start_time_us, 0);
        assert_eq!(evaluate(&plan.controls, 0.0), 0.0);
        assert_eq!(evaluate(&plan.controls, 0.05), 0.2);
        assert_eq!(evaluate(&plan.controls, 0.075), 0.0);
        interpretation::validate_interpretation(&plan).unwrap();
        if close_note != 42 {
            let motion =
                crate::motion::Motion::with_profile(plan, crate::rig::RigProfile::primitive())
                    .unwrap();
            let hit = motion
                .plan
                .hits
                .iter()
                .find(|hit| hit.note == close_note)
                .unwrap();
            assert_eq!(hit.limb, Some(2));
            let pose = motion.evaluate_rig(0.0);
            for axis in 0..3 {
                assert!((pose[2 * 12 + 9 + axis] as f64 - hit.target[axis]).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn paired_close_bounds_later_literal_preparation_without_a_pose_jump() {
    let plan = plan(&[&[0, 0x90, 46, 80], &[30, 0x90, 44, 60], &[66, 0x90, 44, 80]]);
    let profile = crate::rig::RigProfile::primitive();
    let motion = crate::motion::Motion::with_profile(plan, profile).unwrap();
    let pedal = motion
        .plan
        .kit
        .iter()
        .find(|piece| piece.id == "pedalHat")
        .unwrap()
        .position;
    let times = [0.03125 - 1e-9, 0.03125, 0.03125 + 1e-9, 0.065625, 0.1];
    let poses: Vec<_> = times
        .iter()
        .map(|&time| motion.evaluate_rig(time))
        .collect();
    // A future literal foot strike must not override the exact control close,
    // nor introduce a step immediately before or after that boundary.
    for pose in &poses[..3] {
        for axis in 0..3 {
            assert!((pose[33 + axis] as f64 - pedal[axis]).abs() < 1e-6);
        }
        assert!(pose[65].abs() < 1e-6);
        assert!((pose[68] - 1.0).abs() < 1e-6);
    }
    assert!(poses[3][34] > pedal[1] as f32);
    for axis in 0..3 {
        assert!((poses[4][33 + axis] as f64 - pedal[axis]).abs() < 1e-6);
    }
    for index in [4, 1, 3, 0, 2] {
        assert_eq!(poses[index], motion.evaluate_rig(times[index]));
    }
}

#[test]
fn closures_on_another_channel_and_zero_preparation_remain_literal() {
    for close_note in [33, 42, 44] {
        let plan = plan(&[
            &[0, 0x90, 46, 80],
            &[12, 0x91, close_note, 80],
            &[12, 0x90, 44, 60],
        ]);
        assert!(plan.controls.is_empty());
        assert_eq!(plan.hits.len(), 3);
        assert!(plan
            .diagnostics
            .iter()
            .any(|d| d.code == "HIHAT_INTENT_CONFLICT"));
        interpretation::validate_interpretation(&plan).unwrap();
    }
    let simultaneous = plan(&[&[0, 0x90, 44, 80], &[0, 0x90, 46, 80], &[24, 0x90, 44, 60]]);
    assert!(simultaneous.controls.is_empty());
    assert_eq!(simultaneous.hits.len(), 3);
    assert!(simultaneous
        .diagnostics
        .iter()
        .any(|d| d.code == "HIHAT_INTENT_CONFLICT"));
}

#[test]
fn intent_fields_and_control_tampering_are_validated_on_load() {
    let original = plan(&[&[0, 0x90, 46, 80], &[30, 0x90, 44, 60]]);
    for case in 0..7 {
        let mut plan = original.clone();
        match case {
            0 => plan.controls[0].time_us += 1,
            1 => plan.controls[0].openness = 0.5,
            2 => plan.controls[0].open_source_event_id = "0:1".into(),
            3 => plan.hihat_policy = HihatPolicy::Literal,
            4 => plan.hihat_openness = 0.0,
            5 => plan.controls[0].release_start_time_us += 1,
            _ => plan.interpretations[1].related_source_event_id = None,
        }
        assert!(
            load_motion(&serde_json::to_string(&plan).unwrap(), None).is_err(),
            "case {case}"
        );
    }
    let mut invalid = selected(HihatPolicy::ShortOpenClose);
    invalid.mapping_profile = MappingProfile::Gm;
    assert!(build_plan(&file(&[&[0, 0x90, 46, 80]]), &invalid).is_err());
}
