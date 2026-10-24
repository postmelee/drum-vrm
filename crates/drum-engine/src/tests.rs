use super::*;
use model::*;
use motion::{distance, Motion};

fn vlq(mut value: u32) -> Vec<u8> {
    let mut output = vec![(value & 0x7f) as u8];
    value >>= 7;
    while value != 0 {
        output.push(((value & 0x7f) as u8) | 0x80);
        value >>= 7;
    }
    output.reverse();
    output
}

fn smf(format: u16, ppq: u16, tracks: Vec<Vec<(u32, Vec<u8>)>>) -> Vec<u8> {
    let mut output = b"MThd\0\0\0\x06".to_vec();
    output.extend(format.to_be_bytes());
    output.extend((tracks.len() as u16).to_be_bytes());
    output.extend(ppq.to_be_bytes());
    for events in tracks {
        let mut track = Vec::new();
        for (delta, data) in events {
            track.extend(vlq(delta));
            track.extend(data);
        }
        output.extend(b"MTrk");
        output.extend((track.len() as u32).to_be_bytes());
        output.extend(track);
    }
    output
}
fn note(key: u8, velocity: u8, channel: u8) -> Vec<u8> {
    vec![0x90 + channel, key, velocity]
}
fn end() -> Vec<u8> {
    vec![0xff, 0x2f, 0]
}
fn tempo(us: u32) -> Vec<u8> {
    vec![0xff, 0x51, 3, (us >> 16) as u8, (us >> 8) as u8, us as u8]
}
fn selection(tracks: &[usize], channel: Option<u8>) -> Selection {
    Selection {
        tracks: tracks.to_vec(),
        channel,
    }
}
fn plan_for(events: Vec<(u32, Vec<u8>)>) -> PerformancePlan {
    build_plan(&smf(0, 480, vec![events]), &selection(&[0], None)).unwrap()
}

#[test]
fn format_zero_note_on_zero_and_note_off_do_not_strike() {
    let plan = plan_for(vec![
        (0, note(36, 127, 9)),
        (240, note(38, 0, 9)),
        (0, vec![0x89, 36, 127]),
        (240, note(38, 64, 9)),
        (480, end()),
    ]);
    assert_eq!(plan.hits.len(), 2);
    assert_eq!(plan.hits[0].velocity, 127);
    assert_eq!(plan.hits[1].velocity, 64);
    assert_eq!(plan.hits[1].time_us, 500_000);
    assert_eq!(plan.hits[1].tick, 480);
    assert_eq!(plan.hits[1].event_index, 3);
    assert_eq!(plan.duration, 1.0);
}

#[test]
fn all_tracks_tempo_map_and_non_drum_channel_selection() {
    let bytes = smf(
        1,
        480,
        vec![
            vec![(0, tempo(500_000)), (480, tempo(1_000_000)), (480, end())],
            vec![
                (0, note(36, 100, 2)),
                (480, note(38, 110, 2)),
                (480, note(49, 100, 3)),
                (0, end()),
            ],
        ],
    );
    let info = inspect(&bytes).unwrap();
    assert_eq!(info.format, 1);
    assert_eq!(info.suggested_tracks, vec![1]);
    assert_eq!(info.tracks[1].channels, vec![2, 3]);
    let plan = build_plan(&bytes, &selection(&[1], Some(2))).unwrap();
    assert_eq!(plan.hits.len(), 2);
    assert_eq!(plan.hits[1].time_us, 500_000);
    assert_eq!(plan.duration, 1.5);
    assert_eq!(plan.tempo_count, 2);
    let all = build_plan(&bytes, &selection(&[1], None)).unwrap();
    assert_eq!(all.hits[2].time_us, 1_500_000);
}

#[test]
fn selected_track_end_excludes_other_tails_but_keeps_their_tempos() {
    let bytes = smf(
        1,
        480,
        vec![
            vec![(0, tempo(1_000_000)), (960_000, end())],
            vec![(0, note(36, 100, 9)), (480, note(38, 100, 9)), (480, end())],
            vec![(0, note(60, 100, 0)), (480_000, end())],
        ],
    );
    let plan = build_plan(&bytes, &selection(&[1, 1], Some(9))).unwrap();
    assert_eq!(plan.duration, 2.0);
    assert_eq!(plan.hits.len(), 2);
    assert_eq!(plan.hits[1].time_us, 1_000_000);
    assert!(build_plan(&bytes, &selection(&[1, 2], None)).is_err());
}

#[test]
fn musical_grid_metadata_preserves_tempos_meters_and_original_hit_times() {
    let bytes = smf(
        1,
        480,
        vec![
            vec![
                (0, tempo(500_000)),
                (0, vec![0xff, 0x58, 4, 4, 2, 24, 8]),
                (960, tempo(1_000_000)),
                (960, vec![0xff, 0x58, 4, 3, 2, 24, 8]),
                (1440, end()),
            ],
            vec![
                (0, note(36, 100, 9)),
                (960, note(38, 100, 9)),
                (960, note(42, 100, 9)),
                (1440, end()),
            ],
        ],
    );
    let plan = build_plan(&bytes, &selection(&[1], Some(9))).unwrap();
    assert_eq!(plan.ppq, Some(480));
    assert_eq!(plan.end_tick, Some(3360));
    assert_eq!(plan.tempo_map.len(), 2);
    assert_eq!(
        (
            plan.tempo_map[1].tick,
            plan.tempo_map[1].time,
            plan.tempo_map[1].microseconds_per_quarter
        ),
        (960, 1.0, 1_000_000)
    );
    assert_eq!(
        plan.time_signatures
            .iter()
            .map(|s| (s.tick, s.numerator, s.denominator))
            .collect::<Vec<_>>(),
        vec![(0, 4, 4), (1920, 3, 4)]
    );
    assert_eq!(
        plan.hits.iter().map(|h| h.time_us).collect::<Vec<_>>(),
        vec![0, 1_000_000, 3_000_000]
    );
    assert_eq!(plan.duration, 6.0);
}

#[test]
fn meter_conflicts_use_conductor_and_invalid_meters_do_not_change_hits() {
    let bytes = smf(
        1,
        480,
        vec![
            vec![
                (0, vec![0xff, 0x58, 4, 6, 3, 24, 8]),
                (480, vec![0xff, 0x58, 4, 0, 255, 24, 8]),
                (480, end()),
            ],
            vec![
                (0, vec![0xff, 0x58, 4, 3, 2, 24, 8]),
                (480, note(38, 100, 9)),
                (480, end()),
            ],
        ],
    );
    let plan = build_plan(&bytes, &selection(&[1], Some(9))).unwrap();
    assert_eq!(plan.time_signatures.len(), 1);
    assert_eq!(
        (
            plan.time_signatures[0].numerator,
            plan.time_signatures[0].denominator
        ),
        (6, 8)
    );
    assert!(plan
        .diagnostics
        .iter()
        .any(|d| d.code == "TIME_SIGNATURE_CONFLICT"));
    assert!(plan
        .diagnostics
        .iter()
        .any(|d| d.code == "INVALID_TIME_SIGNATURE"));
    assert_eq!(plan.hits[0].time_us, 500_000);
    assert_eq!(plan.tempo_map.len(), 1);
}

#[test]
fn forged_u32_chunk_lengths_are_rejected_before_pointer_arithmetic() {
    for length in [u32::MAX, 0xffff_fff8, 0xffff_fff9, 10_485_760] {
        let mut bytes = smf(0, 480, vec![vec![(0, end())]]);
        bytes[18..22].copy_from_slice(&length.to_be_bytes());
        assert!(inspect(&bytes).unwrap_err().contains("잘렸습니다"));
    }
}

#[test]
fn tempo_conflict_uses_conductor_last_event_and_warns() {
    let bytes = smf(
        1,
        480,
        vec![
            vec![(0, tempo(800_000)), (0, tempo(600_000)), (480, end())],
            vec![(0, tempo(200_000)), (480, note(38, 90, 9)), (0, end())],
        ],
    );
    let plan = build_plan(&bytes, &selection(&[1], None)).unwrap();
    assert_eq!(plan.hits[0].time_us, 600_000);
    assert!(plan.diagnostics.iter().any(|d| d.code == "TEMPO_CONFLICT"));
}

#[test]
fn tempo_rounds_once_after_exact_rational_accumulation() {
    let bytes = smf(
        1,
        3,
        vec![
            vec![
                (0, tempo(500_000)),
                (1, tempo(500_001)),
                (1, tempo(500_002)),
                (1, end()),
            ],
            vec![
                (1, note(38, 90, 9)),
                (1, note(38, 90, 9)),
                (1, note(38, 90, 9)),
                (0, end()),
            ],
        ],
    );
    let plan = build_plan(&bytes, &selection(&[1], None)).unwrap();
    assert_eq!(
        plan.hits.iter().map(|h| h.time_us).collect::<Vec<_>>(),
        vec![166_667, 333_334, 500_001]
    );
}

#[test]
fn unmapped_notes_are_reported_without_inventing_instrument() {
    let plan = plan_for(vec![
        (0, note(36, 90, 9)),
        (120, note(80, 100, 9)),
        (0, note(80, 70, 9)),
        (120, end()),
    ]);
    assert_eq!(plan.hits.len(), 1);
    assert_eq!(
        plan.diagnostics
            .iter()
            .filter(|d| d.code == "UNMAPPED_NOTE")
            .count(),
        1
    );
    assert!(plan.diagnostics.iter().any(|d| d.message.contains("2개")));
}

#[test]
fn impossible_chord_preserves_audio_hits_and_has_no_double_limb() {
    let plan = plan_for(vec![
        (0, note(38, 100, 9)),
        (0, note(42, 100, 9)),
        (0, note(49, 100, 9)),
        (0, note(36, 100, 9)),
        (480, end()),
    ]);
    assert_eq!(plan.hits.len(), 4);
    assert_eq!(plan.hits.iter().filter(|h| h.limb.is_none()).count(), 1);
    assert!(plan.diagnostics.iter().any(|d| d.code == "UNRESOLVED_HIT"));
    let mut assigned: Vec<_> = plan.hits.iter().filter_map(|h| h.limb).collect();
    assigned.sort_unstable();
    assigned.dedup();
    assert_eq!(assigned.len(), 3);
    assert!(plan.hits.iter().all(|h| h.time == 0.0));
}

#[test]
fn too_fast_kicks_remain_at_original_times_and_are_diagnosed() {
    let plan = plan_for(vec![
        (0, note(36, 100, 9)),
        (1, note(36, 100, 9)),
        (479, end()),
    ]);
    assert_eq!(plan.hits.len(), 2);
    assert_eq!(plan.hits[1].time_us, 1_042);
    assert_eq!(plan.hits[1].limb, None);
}

#[test]
fn assigned_stick_tips_and_feet_contact_every_kit_target_within_two_cm() {
    let notes = [36, 38, 42, 46, 44, 48, 45, 41, 49, 51];
    let mut events: Vec<_> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| (if i == 0 { 0 } else { 480 }, note(*n, 100, 9)))
        .collect();
    events.push((480, end()));
    let plan = plan_for(events);
    assert!(plan.hits.iter().all(|h| h.limb.is_some()));
    let motion = Motion::new(plan).unwrap();
    let mut max_error = 0.0f64;
    for hit in &motion.plan.hits {
        let pose = motion.evaluate(hit.time);
        assert_eq!(pose.len(), 57);
        let offset = hit.limb.unwrap() * 12 + 9;
        let tip = [
            pose[offset] as f64,
            pose[offset + 1] as f64,
            pose[offset + 2] as f64,
        ];
        max_error = max_error.max(distance(tip, hit.target));
        assert!(distance(tip, hit.target) < 0.02);
    }
    assert!(max_error < 0.000_001, "max contact error {max_error}");
}

#[test]
fn zero_time_preparation_and_final_rebound_are_finite_and_stateless() {
    let plan = plan_for(vec![
        (0, note(38, 110, 9)),
        (240, note(42, 100, 9)),
        (240, note(38, 60, 9)),
        (0, end()),
    ]);
    let motion = Motion::new(plan).unwrap();
    let times = [-0.5, -0.15, 0.0, 0.13, 0.25, 0.50, 0.60, 0.75];
    let forward: Vec<_> = times.iter().map(|t| motion.evaluate(*t)).collect();
    assert_ne!(forward[0], forward[1]);
    for i in [7, 2, 6, 0, 4, 1, 5, 3] {
        assert_eq!(forward[i], motion.evaluate(times[i]));
    }
    for pose in &forward {
        assert!(pose.iter().all(|v| v.is_finite()));
        for limb in 0..4 {
            let point = |p: usize| {
                let i = limb * 12 + p * 3;
                [pose[i] as f64, pose[i + 1] as f64, pose[i + 2] as f64]
            };
            assert!((distance(point(0), point(1)) - 0.43).abs() < 1e-6);
            assert!((distance(point(1), point(2)) - 0.43).abs() < 1e-6);
        }
    }
    assert_eq!(motion.evaluate(f64::NAN), motion.evaluate(-0.5));
}

#[test]
fn serialized_plan_reconstructs_identical_pose() {
    let plan = plan_for(vec![
        (0, note(38, 110, 9)),
        (240, note(42, 100, 9)),
        (240, end()),
    ]);
    let json = serde_json::to_string(&plan).unwrap();
    assert!(json.contains("schemaVersion"));
    assert!(json.contains("eventIndex"));
    let original = Motion::new(plan).unwrap();
    let reconstructed = Motion::new(serde_json::from_str(&json).unwrap()).unwrap();
    for time in [-0.5, 0.0, 0.125, 0.25, 0.6] {
        assert_eq!(original.evaluate(time), reconstructed.evaluate(time));
    }
}

#[test]
fn malformed_unsupported_and_empty_files_have_actionable_errors() {
    for bytes in [
        vec![],
        b"not midi".to_vec(),
        smf(0, 480, vec![vec![(0, note(38, 100, 9))]]),
    ] {
        assert!(inspect(&bytes).is_err());
    }
    let good = smf(0, 480, vec![vec![(0, note(38, 100, 9)), (480, end())]]);
    assert!(inspect(&good[..good.len() - 1]).is_err());
    assert!(inspect(&smf(2, 480, vec![vec![(0, end())]]))
        .unwrap_err()
        .contains("format 2"));
    assert!(inspect(&smf(0, 0xe728, vec![vec![(0, end())]]))
        .unwrap_err()
        .contains("SMPTE"));
    assert!(inspect(&smf(0, 0, vec![vec![(0, end())]])).is_err());
    assert!(build_plan(&good, &selection(&[], None)).is_err());
    assert!(build_plan(&good, &selection(&[10], None)).is_err());
    assert!(build_plan(&good, &selection(&[0], Some(16))).is_err());
    assert!(build_plan(&smf(0, 480, vec![vec![(0, end())]]), &selection(&[0], None)).is_err());
}

#[test]
fn exact_five_minute_boundary_and_file_and_event_limits() {
    let at_limit = smf(0, 480, vec![vec![(0, note(36, 100, 9)), (288_000, end())]]);
    assert_eq!(
        build_plan(&at_limit, &selection(&[0], None))
            .unwrap()
            .duration,
        300.0
    );
    let above = smf(0, 480, vec![vec![(0, note(36, 100, 9)), (288_001, end())]]);
    assert!(build_plan(&above, &selection(&[0], None))
        .unwrap_err()
        .contains("300초"));
    assert!(inspect(&vec![0; MAX_FILE_BYTES + 1])
        .unwrap_err()
        .contains("10 MiB"));
    let mut events = vec![(0, note(36, 0, 9)); MAX_EVENTS];
    events.push((0, end()));
    assert!(inspect(&smf(0, 480, vec![events]))
        .unwrap_err()
        .contains("200000"));
}

#[test]
fn malicious_plan_rejects_nonfinite_targets_and_double_assignments() {
    let mut plan = plan_for(vec![
        (0, note(38, 110, 9)),
        (240, note(42, 100, 9)),
        (240, end()),
    ]);
    plan.hits[0].target[0] = f64::NAN;
    assert!(Motion::new(plan).is_err());
    let mut plan = plan_for(vec![
        (0, note(38, 110, 9)),
        (0, note(42, 100, 9)),
        (240, end()),
    ]);
    plan.hits[1].limb = plan.hits[0].limb;
    assert!(Motion::new(plan).is_err());
}

#[test]
fn huge_simultaneous_group_is_bounded_and_preserved() {
    let mut events = vec![(0, note(38, 100, 9)); 1_000];
    events.push((480, end()));
    let plan = plan_for(events);
    assert_eq!(plan.hits.len(), 1_000);
    assert_eq!(plan.hits.iter().filter(|h| h.limb.is_some()).count(), 2);
    assert_eq!(
        plan.diagnostics
            .iter()
            .filter(|d| d.code == "UNRESOLVED_HIT")
            .count(),
        998
    );
}
