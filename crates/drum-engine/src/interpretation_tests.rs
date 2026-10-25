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
    assert!(build_plan(&bytes, &selection(MappingProfile::Gm, true)).is_err());
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
