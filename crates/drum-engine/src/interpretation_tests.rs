//! Independently authored MIDI cases, not excerpts from an uploaded song.
use crate::interpretation::{self, InterpretationKind, MappingProfile, SourceMessage};
use crate::model::{Instrument, Selection};
use crate::{build_plan, load_motion};

fn file(events: &[&[u8]]) -> Vec<u8> {
    let mut track = events.concat();
    track.extend([0, 0xff, 0x2f, 0]);
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    bytes.extend((track.len() as u32).to_be_bytes());
    bytes.extend(track);
    bytes
}
fn selection(profile: MappingProfile, count_in: bool) -> Selection {
    Selection {
        tracks: vec![0],
        channel: None,
        mapping_profile: profile,
        count_in_note75: count_in,
        ..Selection::default()
    }
}

#[test]
fn source_messages_survive_mapping_without_becoming_strikes() {
    let bytes = file(&[
        &[0, 0x90, 38, 100],
        &[1, 0x80, 38, 64],
        &[1, 0x90, 38, 0],
        &[1, 0xb0, 4, 85],
        &[1, 0xb0, 1, 12],
        &[1, 0xc0, 8],
        &[1, 0xe0, 0, 64],
        &[1, 0xa0, 38, 31],
        &[1, 0xd0, 20],
        &[1, 0x90, 99, 111],
    ]);
    let plan = build_plan(&bytes, &selection(MappingProfile::Gm, false)).unwrap();
    assert_eq!(plan.schema_version, 2);
    assert_eq!(plan.hits.len(), 1);
    assert_eq!(plan.source_events.len(), 10);
    assert_eq!(plan.interpretations.len(), 10);
    assert_eq!(
        plan.source_events[1].message,
        SourceMessage::NoteOff {
            key: 38,
            velocity: 64
        }
    );
    assert_eq!(
        plan.source_events[2].message,
        SourceMessage::NoteOn {
            key: 38,
            velocity: 0
        }
    );
    assert_eq!(
        plan.source_events[3].message,
        SourceMessage::Controller {
            controller: 4,
            value: 85
        }
    );
    assert_eq!(
        plan.source_events[6].message,
        SourceMessage::PitchBend { value: 8192 }
    );
    assert_eq!(plan.source_events[9].id, "0:9");
    assert_eq!(plan.source_events[9].tick, 9);
    assert_eq!(plan.source_events[9].time_us, 9375);
    assert_eq!(plan.interpretations[9].kind, InterpretationKind::Unmapped);
    assert!(plan.diagnostics.iter().any(|d| d.code == "UNMAPPED_NOTE"));
    interpretation::validate_interpretation(&plan).unwrap();
    let json = serde_json::to_string(&plan).unwrap();
    let loaded = load_motion(&json, None).unwrap();
    assert_eq!(
        loaded.plan.source_events[3].message,
        plan.source_events[3].message
    );
    assert_eq!(loaded.plan.interpretations, plan.interpretations);
}

#[test]
fn logic_mapping_and_explicit_count_in_are_separate_from_gm() {
    let bytes = file(&[
        &[0, 0x90, 75, 100],
        &[120, 0x90, 32, 90],
        &[120, 0x90, 40, 70],
        &[120, 0x90, 52, 80],
    ]);
    let gm = build_plan(&bytes, &selection(MappingProfile::Gm, false)).unwrap();
    assert_eq!(gm.hits.len(), 2);
    assert_eq!(gm.hits[0].articulation, "electric");
    assert_eq!(gm.hits[1].instrument, Instrument::Crash);
    let logic = build_plan(&bytes, &selection(MappingProfile::LogicDkdGm, false)).unwrap();
    assert_eq!(logic.hits.len(), 3);
    assert_eq!(logic.hits[0].articulation, "rimshotEdge");
    assert_eq!(logic.hits[1].articulation, "rimshot");
    assert_eq!(logic.hits[2].instrument, Instrument::Ride);
    assert_eq!(logic.interpretations[0].kind, InterpretationKind::Unmapped);
    let cue = build_plan(&bytes, &selection(MappingProfile::LogicDkdGm, true)).unwrap();
    assert_eq!(cue.interpretations[0].kind, InterpretationKind::CountIn);
    assert_eq!(cue.source_events[0].id, "0:0");
    assert_eq!(cue.hits.len(), logic.hits.len());
    assert!(cue.diagnostics.iter().all(|d| d.code != "UNMAPPED_NOTE"));
    let gm_intent = build_plan(&bytes, &selection(MappingProfile::Gm, true)).unwrap();
    assert!(gm_intent
        .interpretations
        .iter()
        .any(|i| i.kind == InterpretationKind::CountIn));
    assert!(serde_json::from_str::<Selection>(
        r#"{"tracks":[0],"channel":null,"mappingProfile":"logicDkdModWheel"}"#
    )
    .is_err());
}

#[test]
fn two_crashes_have_distinct_instances_and_unchanged_source_times() {
    let bytes = file(&[
        &[0, 0x90, 49, 120],
        &[0, 0x90, 57, 90],
        &[120, 0x90, 36, 80],
    ]);
    for profile in [MappingProfile::Gm, MappingProfile::LogicDkdGm] {
        let plan = build_plan(&bytes, &selection(profile, false)).unwrap();
        assert_eq!(plan.hits[0].instrument, Instrument::Crash);
        assert_eq!(plan.hits[1].instrument, Instrument::Crash);
        assert_eq!(plan.hits[0].piece_id, "crash");
        assert_eq!(plan.hits[1].piece_id, "crashRight");
        assert_ne!(plan.hits[0].target, plan.hits[1].target);
        assert_eq!(plan.hits[0].time_us, plan.hits[1].time_us);
        assert_eq!(plan.hits[0].source_event_id, "0:0");
        assert_eq!(plan.hits[1].source_event_id, "0:1");
        interpretation::validate_interpretation(&plan).unwrap();
    }
}

#[test]
fn source_and_interpretation_tampering_is_rejected() {
    let bytes = file(&[
        &[0, 0x90, 32, 90],
        &[120, 0xb0, 4, 10],
        &[120, 0x90, 75, 110],
    ]);
    let original = build_plan(&bytes, &selection(MappingProfile::LogicDkdGm, true)).unwrap();
    for case in 0..11 {
        let mut plan = original.clone();
        match case {
            0 => plan.hits[0].piece_id = "crash".into(),
            1 => plan.hits[0].velocity = 89,
            2 => plan.source_events[0].tick += 1,
            3 => plan.interpretations[2].kind = InterpretationKind::Unmapped,
            4 => plan.mapping_profile = MappingProfile::Gm,
            5 => {
                plan.source_events[1].message = SourceMessage::Controller {
                    controller: 128,
                    value: 10,
                }
            }
            6 => plan.source_events[1].id = plan.source_events[0].id.clone(),
            7 => plan.interpretations[0].hit_id = Some(99),
            8 => plan.duration += 0.1,
            9 => plan.end_tick = plan.end_tick.map(|tick| tick + 1),
            _ => plan.source_events[1].time_us += 1,
        }
        assert!(
            interpretation::validate_interpretation(&plan).is_err(),
            "case {case}"
        );
        assert!(
            load_motion(&serde_json::to_string(&plan).unwrap(), None).is_err(),
            "load case {case}"
        );
    }
}

#[test]
fn legacy_plan_migration_does_not_invent_discarded_source() {
    let bytes = file(&[&[0, 0x90, 57, 90], &[120, 0x90, 38, 60]]);
    let plan = build_plan(&bytes, &selection(MappingProfile::Gm, false)).unwrap();
    let mut value = serde_json::to_value(plan).unwrap();
    value["schemaVersion"] = 1.into();
    for field in [
        "mappingProfile",
        "mappingVersion",
        "countInNote75",
        "sourceEvents",
        "interpretations",
    ] {
        value.as_object_mut().unwrap().remove(field);
    }
    let kit = value["kit"].as_array_mut().unwrap();
    kit.retain(|piece| piece["id"] != "crashRight");
    for piece in kit {
        piece.as_object_mut().unwrap().remove("instrument");
    }
    let left_target = value["kit"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "crash")
        .unwrap()["position"]
        .clone();
    value["hits"][0]["target"] = left_target;
    value["hits"][0]["limb"] = 0.into();
    for hit in value["hits"].as_array_mut().unwrap() {
        for field in ["pieceId", "articulation", "sourceEventId"] {
            hit.as_object_mut().unwrap().remove(field);
        }
    }
    let motion = load_motion(&value.to_string(), None).unwrap();
    assert_eq!(motion.plan.schema_version, 1);
    assert!(motion.plan.source_events.is_empty());
    assert!(motion.plan.interpretations.is_empty());
    assert_eq!(motion.plan.hits[0].piece_id, "crash");
    assert_eq!(motion.plan.hits[0].articulation, "legacy");
}

#[test]
fn source_selection_keeps_track_event_identity_across_channel_filtering() {
    let bytes = file(&[
        &[0, 0x91, 38, 100],
        &[120, 0xb1, 4, 60],
        &[0, 0x90, 38, 110],
    ]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.channel = Some(0);
    let plan = build_plan(&bytes, &selected).unwrap();
    assert_eq!(plan.source_events.len(), 1);
    assert_eq!(plan.source_events[0].id, "0:2");
    assert_eq!(plan.hits[0].source_event_id, "0:2");
    assert_eq!(plan.hits[0].time_us, 125_000);
}

#[test]
fn explicit_omission_preserves_source_and_only_changes_arranged_events() {
    let bytes = file(&[
        &[0, 0x90, 32, 100],
        &[0, 0x90, 49, 110],
        &[0, 0x90, 57, 105],
        &[0, 0x90, 36, 90],
        &[120, 0x80, 32, 64],
    ]);
    let original = build_plan(&bytes, &selection(MappingProfile::LogicDkdGm, false)).unwrap();
    let mut selected = selection(MappingProfile::LogicDkdGm, false);
    selected.omitted_source_event_ids = vec!["0:0".into()];
    let arranged = build_plan(&bytes, &selected).unwrap();
    assert_eq!(
        serde_json::to_value(&arranged.source_events).unwrap(),
        serde_json::to_value(&original.source_events).unwrap()
    );
    assert_eq!(arranged.duration, original.duration);
    assert_eq!(arranged.midi_hash, original.midi_hash);
    assert_eq!(arranged.hits.len(), 3);
    assert_eq!(
        arranged.interpretations[0].kind,
        InterpretationKind::Omitted
    );
    assert_eq!(
        arranged.interpretations[0].piece_id.as_deref(),
        Some("snare")
    );
    assert_eq!(
        arranged.interpretations[0].articulation.as_deref(),
        Some("rimshotEdge")
    );
    assert_eq!(arranged.interpretations[0].hit_id, None);
    assert_eq!(
        arranged.interpretations[4].kind,
        InterpretationKind::Preserved
    );
    for hit in &arranged.hits {
        let original_hit = original
            .hits
            .iter()
            .find(|old| old.source_event_id == hit.source_event_id)
            .unwrap();
        assert_eq!(hit.time_us, original_hit.time_us);
        assert_eq!(hit.velocity, original_hit.velocity);
        assert_eq!(hit.note, original_hit.note);
    }
    let loaded = load_motion(&serde_json::to_string(&arranged).unwrap(), None).unwrap();
    assert_eq!(
        loaded.plan.omitted_source_event_ids,
        selected.omitted_source_event_ids
    );
    assert_eq!(loaded.plan.interpretations, arranged.interpretations);
    // Removing the explicit instruction cannot silently preserve its derived omission.
    let mut forged = arranged.clone();
    forged.omitted_source_event_ids.clear();
    assert!(load_motion(&serde_json::to_string(&forged).unwrap(), None).is_err());
    let mut forged = original;
    forged.omitted_source_event_ids = selected.omitted_source_event_ids;
    assert!(load_motion(&serde_json::to_string(&forged).unwrap(), None).is_err());
}

#[test]
fn arrangement_rejects_unknown_nonpositive_conflicting_and_foot_directives() {
    let bytes = file(&[
        &[0, 0x90, 38, 100],
        &[24, 0x80, 38, 64],
        &[24, 0x90, 38, 0],
        &[24, 0xb0, 4, 60],
        &[24, 0x90, 36, 90],
        &[24, 0x90, 75, 100],
        &[24, 0x90, 99, 100],
    ]);
    for ids in [
        vec!["0:99"],
        vec!["0:1"],
        vec!["0:2"],
        vec!["0:3"],
        vec!["0:0", "0:0"],
    ] {
        let mut selected = selection(MappingProfile::Gm, true);
        selected.omitted_source_event_ids = ids.iter().map(|id| id.to_string()).collect();
        assert!(build_plan(&bytes, &selected).is_err(), "omitted {ids:?}");
    }
    for (id, hand) in [
        ("0:0", 2),
        ("0:1", 0),
        ("0:2", 1),
        ("0:3", 1),
        ("0:4", 0),
        ("0:5", 0),
        ("0:6", 0),
        ("0:99", 1),
    ] {
        let mut selected = selection(MappingProfile::Gm, true);
        selected.hand_overrides.insert(id.into(), hand);
        assert!(build_plan(&bytes, &selected).is_err(), "hand {id} {hand}");
    }
    let mut selected = selection(MappingProfile::Gm, false);
    selected.omitted_source_event_ids.push("0:0".into());
    selected.hand_overrides.insert("0:0".into(), 0);
    assert!(build_plan(&bytes, &selected).is_err());
    selected.hand_overrides.clear();
    selected.channel = Some(1);
    assert!(build_plan(&bytes, &selected).is_err());
}

#[test]
fn omission_recomputes_hihat_pairs_and_preparation_from_remaining_events() {
    let bytes = file(&[
        &[0, 0x90, 36, 90],
        &[48, 0x90, 44, 70],
        &[48, 0x90, 46, 85],
        &[24, 0x90, 44, 65],
        &[120, 0x90, 38, 90],
    ]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.hihat_policy = crate::hihat::HihatPolicy::ShortOpenClose;
    let original = build_plan(&bytes, &selected).unwrap();
    assert_eq!(original.controls.len(), 1);
    assert_eq!(original.controls[0].release_start_time_us, 50_000);
    for omitted in ["0:2", "0:3"] {
        selected.omitted_source_event_ids = vec![omitted.into()];
        let arranged = build_plan(&bytes, &selected).unwrap();
        assert!(arranged.controls.is_empty());
        assert!(arranged
            .interpretations
            .iter()
            .all(|i| i.kind != InterpretationKind::Control));
        assert!(arranged
            .interpretations
            .iter()
            .any(|i| i.source_event_id == omitted && i.kind == InterpretationKind::Omitted));
        interpretation::validate_interpretation(&arranged).unwrap();
        let mut forged = arranged;
        forged.controls = original.controls.clone();
        assert!(load_motion(&serde_json::to_string(&forged).unwrap(), None).is_err());
    }
    selected.omitted_source_event_ids = vec!["0:1".into()];
    let arranged = build_plan(&bytes, &selected).unwrap();
    assert_eq!(arranged.controls.len(), 1);
    assert_eq!(arranged.controls[0].release_start_time_us, 0);
    assert_eq!(arranged.controls[0].open_source_event_id, "0:2");
    assert_eq!(arranged.controls[0].source_event_id, "0:3");
    interpretation::validate_interpretation(&arranged).unwrap();
}

#[test]
fn all_strikes_can_be_explicitly_omitted_and_old_plans_default_to_no_edits() {
    let bytes = file(&[&[0, 0x90, 38, 100], &[120, 0x80, 38, 0]]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.omitted_source_event_ids.push("0:0".into());
    let arranged = build_plan(&bytes, &selected).unwrap();
    assert!(arranged.hits.is_empty());
    assert_eq!(arranged.source_events.len(), 2);
    load_motion(&serde_json::to_string(&arranged).unwrap(), None).unwrap();

    let original = build_plan(&bytes, &selection(MappingProfile::Gm, false)).unwrap();
    let mut old_json = serde_json::to_value(&original).unwrap();
    old_json
        .as_object_mut()
        .unwrap()
        .remove("omittedSourceEventIds");
    old_json.as_object_mut().unwrap().remove("handOverrides");
    let loaded = load_motion(&old_json.to_string(), None).unwrap();
    assert!(loaded.plan.omitted_source_event_ids.is_empty());
    assert!(loaded.plan.hand_overrides.is_empty());
    let mut legacy = original;
    legacy.schema_version = 1;
    legacy.source_events.clear();
    legacy.interpretations.clear();
    legacy.omitted_source_event_ids.push("0:0".into());
    assert!(interpretation::validate_interpretation(&legacy).is_err());
    legacy.omitted_source_event_ids.clear();
    legacy.hand_overrides.insert("0:0".into(), 0);
    assert!(interpretation::validate_interpretation(&legacy).is_err());
}

#[test]
fn explicit_hands_survive_roundtrip_and_competing_choices_remain_diagnostic() {
    let bytes = file(&[&[0, 0x90, 38, 100], &[120, 0x90, 38, 70]]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.hand_overrides.insert("0:0".into(), 1);
    selected.hand_overrides.insert("0:1".into(), 0);
    let plan = build_plan(&bytes, &selected).unwrap();
    assert_eq!(
        plan.hits.iter().map(|hit| hit.limb).collect::<Vec<_>>(),
        [Some(1), Some(0)]
    );
    let loaded = load_motion(&serde_json::to_string(&plan).unwrap(), None).unwrap();
    assert_eq!(loaded.plan.hand_overrides, selected.hand_overrides);
    assert_eq!(
        loaded
            .plan
            .hits
            .iter()
            .map(|hit| hit.limb)
            .collect::<Vec<_>>(),
        [Some(1), Some(0)]
    );
    let mut forged = plan;
    forged.hits[0].limb = Some(0);
    assert!(load_motion(&serde_json::to_string(&forged).unwrap(), None).is_err());

    let bytes = file(&[
        &[0, 0x90, 38, 100],
        &[0, 0x90, 42, 80],
        &[120, 0x90, 36, 90],
    ]);
    selected.hand_overrides.insert("0:1".into(), 1);
    let plan = build_plan(&bytes, &selected).unwrap();
    assert_eq!(
        plan.hits[..2]
            .iter()
            .filter(|hit| hit.limb == Some(1))
            .count(),
        1
    );
    assert_eq!(
        plan.hits[..2]
            .iter()
            .filter(|hit| hit.limb.is_none())
            .count(),
        1
    );
    assert!(plan.hits[..2]
        .iter()
        .all(|hit| hit.time_us == 0 && hit.limb != Some(0)));
    assert_eq!(
        plan.diagnostics
            .iter()
            .filter(|d| d.code == "HAND_OVERRIDE_INFEASIBLE")
            .count(),
        1
    );
    let loaded = load_motion(&serde_json::to_string(&plan).unwrap(), None).unwrap();
    assert_eq!(
        loaded
            .plan
            .hits
            .iter()
            .map(|hit| hit.limb)
            .collect::<Vec<_>>(),
        plan.hits.iter().map(|hit| hit.limb).collect::<Vec<_>>()
    );
}

#[test]
fn unreachable_forced_hand_does_not_fall_back_to_a_reachable_other_hand() {
    let bytes = file(&[&[0, 0x90, 38, 100], &[120, 0x80, 38, 0]]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.hand_overrides.insert("0:0".into(), 1);
    let plan = build_plan(&bytes, &selected).unwrap();
    let mut profile = crate::rig::RigProfile::primitive();
    profile.id = "independent-asymmetric-reach-fixture".into();
    profile.rest_tips[1][0] += 2.5 - profile.roots[1][0];
    profile.roots[1][0] = 2.5;
    assert!(profile.reachable(0, plan.hits[0].target));
    assert!(!profile.reachable(1, plan.hits[0].target));
    let motion = crate::motion::Motion::with_profile(plan, profile.clone()).unwrap();
    assert_eq!(motion.plan.hits[0].limb, None);
    assert_eq!(motion.plan.hits[0].time_us, 0);
    assert!(motion
        .plan
        .diagnostics
        .iter()
        .any(|d| d.code == "HAND_OVERRIDE_INFEASIBLE"));
    let loaded = load_motion(
        &serde_json::to_string(&motion.plan).unwrap(),
        Some(&serde_json::to_string(&profile).unwrap()),
    )
    .unwrap();
    assert_eq!(loaded.plan.hits[0].limb, None);
    assert_eq!(
        loaded
            .plan
            .diagnostics
            .iter()
            .filter(|d| d.code == "HAND_OVERRIDE_INFEASIBLE")
            .count(),
        1
    );
}

#[test]
fn unmapped_note33_does_not_close_a_gm_hihat() {
    let bytes = file(&[&[0, 0x90, 46, 80], &[12, 0x91, 33, 80], &[12, 0x90, 44, 65]]);
    let mut selected = selection(MappingProfile::Gm, false);
    selected.hihat_policy = crate::hihat::HihatPolicy::ShortOpenClose;
    let gm = build_plan(&bytes, &selected).unwrap();
    assert_eq!(gm.controls.len(), 1);
    assert_eq!(gm.controls[0].source_event_id, "0:2");
    assert_eq!(gm.controls[0].release_start_time_us, -100_000);
    assert_eq!(gm.interpretations[1].kind, InterpretationKind::Unmapped);
    interpretation::validate_interpretation(&gm).unwrap();
    selected.mapping_profile = MappingProfile::LogicDkdGm;
    let logic = build_plan(&bytes, &selected).unwrap();
    assert!(logic.controls.is_empty());
    assert!(logic
        .diagnostics
        .iter()
        .any(|d| d.code == "HIHAT_INTENT_CONFLICT"));
    interpretation::validate_interpretation(&logic).unwrap();
}
