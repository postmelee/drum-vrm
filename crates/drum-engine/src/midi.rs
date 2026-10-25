use crate::interpretation::{self, InterpretationKind, SourceEvent, SourceMessage};
use crate::model::*;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct Tempo {
    tick: u64,
    track: usize,
    event: usize,
    micros: u32,
}

#[derive(Clone, Copy)]
struct Segment {
    tick: u64,
    numerator: u128,
    micros: u32,
}

#[derive(Clone, Copy)]
struct SignatureEvent {
    tick: u64,
    track: usize,
    event: usize,
    numerator: u8,
    denominator: u16,
}

pub struct Parsed<'a> {
    smf: Smf<'a>,
    pub info: MidiInfo,
    track_ends: Vec<u64>,
    segments: Vec<Segment>,
    diagnostics: Vec<Diagnostic>,
    tempo_count: usize,
    time_signatures: Vec<TimeSignature>,
}

/// A length preflight prevents malformed chunk lengths from hiding truncated data.
/// Allocation is additionally bounded by the 10 MiB input and 200k event ceilings.
fn preflight(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("MIDI 파일은 10 MiB 이하여야 합니다.".into());
    }
    if bytes.len() < 14 || &bytes[..4] != b"MThd" {
        return Err("올바른 Standard MIDI 파일 헤더가 없습니다.".into());
    }
    let header_len = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if header_len != 6 {
        return Err("지원하지 않는 MIDI 헤더 길이입니다 (6 바이트 필요).".into());
    }
    let expected = u16::from_be_bytes([bytes[10], bytes[11]]) as usize;
    if expected == 0 || expected > 512 {
        return Err("MIDI 트랙 수는 1–512개여야 합니다.".into());
    }
    let mut offset = 14;
    let mut tracks = 0;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err("MIDI 트랙 헤더가 잘렸습니다.".into());
        }
        if &bytes[offset..offset + 4] != b"MTrk" {
            return Err("지원하지 않는 MIDI 청크입니다. SMF로 다시 내보내세요.".into());
        }
        let length = u32::from_be_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        // Reject before adding: on wasm32 `8 + 0xfffffff8` would wrap to 0
        // and leave the scan offset unchanged, even inside checked_add(...).
        if length > bytes.len() - offset - 8 {
            return Err("MIDI 트랙 데이터가 잘렸습니다.".into());
        }
        offset += 8 + length;
        tracks += 1;
    }
    if tracks != expected {
        return Err("MIDI 헤더와 실제 트랙 수가 다릅니다.".into());
    }
    Ok(())
}

pub fn parse(bytes: &[u8]) -> Result<Parsed<'_>, String> {
    preflight(bytes)?;
    let (header, track_iter) = midly::parse(bytes).map_err(|e| format!("MIDI 구문 오류: {e}"))?;
    let mut smf = Smf {
        header,
        tracks: Vec::new(),
    };
    let mut event_count = 0;
    for track in track_iter {
        let mut events = Vec::new();
        for event in track.map_err(|e| format!("MIDI 트랙 구문 오류: {e}"))? {
            if event_count == MAX_EVENTS {
                return Err(format!(
                    "MIDI 이벤트가 {MAX_EVENTS}개를 초과합니다. 트랙을 줄여 다시 내보내세요."
                ));
            }
            events.push(event.map_err(|e| format!("MIDI 이벤트 구문 오류: {e}"))?);
            event_count += 1;
        }
        smf.tracks.push(events);
    }
    let format = match smf.header.format {
        Format::SingleTrack => 0,
        Format::Parallel => 1,
        Format::Sequential => {
            return Err("SMF format 2는 지원하지 않습니다. Format 0 또는 1로 내보내세요.".into())
        }
    };
    if format == 0 && smf.tracks.len() != 1 {
        return Err("SMF format 0에는 트랙이 하나여야 합니다.".into());
    }
    let ppq = match smf.header.timing {
        Timing::Metrical(p) if p.as_int() != 0 => p.as_int(),
        Timing::Metrical(_) => return Err("MIDI PPQ 값은 0보다 커야 합니다.".into()),
        Timing::Timecode(_, _) => {
            return Err("SMPTE 시간축은 지원하지 않습니다. PPQ 시간축으로 내보내세요.".into())
        }
    };
    let mut tracks = Vec::new();
    let mut track_ends = Vec::new();
    let mut tempos = Vec::new();
    let mut signatures = Vec::new();
    let mut diagnostics = Vec::new();
    let mut channel_ten_tracks = Vec::new();
    for (index, track) in smf.tracks.iter().enumerate() {
        let mut info = TrackInfo {
            index,
            name: format!("Track {}", index + 1),
            note_count: 0,
            drum_notes: 0,
            channels: Vec::new(),
        };
        let mut tick = 0u64;
        let mut ended = false;
        let mut has_channel_ten = false;
        for (event_index, event) in track.iter().enumerate() {
            if ended {
                return Err(format!(
                    "트랙 {}의 End of Track 뒤에 이벤트가 있습니다.",
                    index + 1
                ));
            }
            tick = tick
                .checked_add(event.delta.as_int() as u64)
                .ok_or("MIDI tick 누적 범위 초과")?;
            if tick > 9_007_199_254_740_991 {
                return Err("MIDI tick이 안전 정수 범위를 초과합니다.".into());
            }
            match event.kind {
                TrackEventKind::Meta(MetaMessage::EndOfTrack) => ended = true,
                TrackEventKind::Meta(MetaMessage::TrackName(name)) => {
                    info.name = String::from_utf8_lossy(name)
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(120)
                        .collect();
                    if info.name.is_empty() {
                        info.name = format!("Track {}", index + 1);
                    }
                }
                TrackEventKind::Meta(MetaMessage::Tempo(micros)) => {
                    if micros.as_int() == 0 {
                        return Err("MIDI 템포 값은 0보다 커야 합니다.".into());
                    }
                    tempos.push(Tempo {
                        tick,
                        track: index,
                        event: event_index,
                        micros: micros.as_int(),
                    });
                }
                TrackEventKind::Meta(MetaMessage::TimeSignature(numerator, power, _, _)) => {
                    if numerator == 0 || power > 7 {
                        diagnostics.push(Diagnostic::new("INVALID_TIME_SIGNATURE", format!("트랙 {} tick {tick}의 지원하지 않는 박자표를 무시했습니다. 타격 시각은 유지합니다.", index + 1)));
                    } else {
                        signatures.push(SignatureEvent {
                            tick,
                            track: index,
                            event: event_index,
                            numerator,
                            denominator: 1u16 << power,
                        });
                    }
                }
                TrackEventKind::Midi {
                    channel,
                    message: MidiMessage::NoteOn { key, vel },
                } if vel.as_int() > 0 => {
                    info.note_count += 1;
                    let c = channel.as_int();
                    if !info.channels.contains(&c) {
                        info.channels.push(c);
                    }
                    if map_note(key.as_int()).is_some() {
                        info.drum_notes += 1;
                    }
                    has_channel_ten |= c == 9;
                }
                _ => {}
            }
        }
        if !ended {
            return Err(format!(
                "트랙 {}에 End of Track 이벤트가 없습니다.",
                index + 1
            ));
        }
        if has_channel_ten {
            channel_ten_tracks.push(index);
        }
        info.channels.sort_unstable();
        tracks.push(info);
        track_ends.push(tick);
    }
    let suggested_tracks = if channel_ten_tracks.is_empty() {
        tracks
            .iter()
            .filter(|t| t.drum_notes > 0)
            .map(|t| t.index)
            .collect()
    } else {
        channel_ten_tracks
    };
    tempos.sort_by_key(|t| (t.tick, t.track, t.event));
    let mut chosen = Vec::new();
    let mut i = 0;
    while i < tempos.len() {
        let start = i;
        while i < tempos.len() && tempos[i].tick == tempos[start].tick {
            i += 1;
        }
        let group = &tempos[start..i];
        let selected = group
            .iter()
            .rev()
            .find(|t| t.track == 0)
            .unwrap_or(group.last().unwrap());
        if group.iter().any(|t| t.micros != selected.micros) {
            diagnostics.push(Diagnostic::new("TEMPO_CONFLICT", format!("tick {}의 템포 충돌: 트랙 {} 이벤트 {}의 {} μs/박을 선택했습니다 (트랙 1 우선, 동일 트랙 마지막 이벤트).", selected.tick, selected.track + 1, selected.event, selected.micros)));
        }
        chosen.push(*selected);
    }
    let tempo_count = chosen.len();
    let mut segments = vec![Segment {
        tick: 0,
        numerator: 0,
        micros: 500_000,
    }];
    for tempo in chosen {
        let prev = *segments.last().unwrap();
        let numerator = prev.numerator + (tempo.tick - prev.tick) as u128 * prev.micros as u128;
        if tempo.tick == prev.tick {
            segments.pop();
        }
        segments.push(Segment {
            tick: tempo.tick,
            numerator,
            micros: tempo.micros,
        });
    }
    signatures.sort_by_key(|s| (s.tick, s.track, s.event));
    let mut time_signatures = vec![TimeSignature {
        tick: 0,
        numerator: 4,
        denominator: 4,
    }];
    let mut i = 0;
    while i < signatures.len() {
        let start = i;
        while i < signatures.len() && signatures[i].tick == signatures[start].tick {
            i += 1;
        }
        let group = &signatures[start..i];
        let selected = group
            .iter()
            .rev()
            .find(|s| s.track == 0)
            .unwrap_or(group.last().unwrap());
        if group
            .iter()
            .any(|s| (s.numerator, s.denominator) != (selected.numerator, selected.denominator))
        {
            diagnostics.push(Diagnostic::new("TIME_SIGNATURE_CONFLICT", format!("tick {}의 박자표 충돌: 트랙 {}의 {}/{}를 선택했습니다 (트랙 1 우선, 동일 트랙 마지막 이벤트).", selected.tick, selected.track + 1, selected.numerator, selected.denominator)));
        }
        if time_signatures
            .last()
            .is_some_and(|s| s.tick == selected.tick)
        {
            time_signatures.pop();
        }
        time_signatures.push(TimeSignature {
            tick: selected.tick,
            numerator: selected.numerator,
            denominator: selected.denominator,
        });
    }
    Ok(Parsed {
        smf,
        info: MidiInfo {
            tracks,
            suggested_tracks,
            format,
            ppq,
        },
        track_ends,
        segments,
        diagnostics,
        tempo_count,
        time_signatures,
    })
}

impl Parsed<'_> {
    fn time_us(&self, tick: u64) -> Result<u64, String> {
        let i = self
            .segments
            .partition_point(|s| s.tick <= tick)
            .saturating_sub(1);
        let segment = self.segments[i];
        let numerator = segment.numerator + (tick - segment.tick) as u128 * segment.micros as u128;
        let micros = (numerator + self.info.ppq as u128 / 2) / self.info.ppq as u128;
        u64::try_from(micros).map_err(|_| "MIDI 시간 범위를 초과했습니다.".into())
    }

    pub fn plan(&self, bytes: &[u8], selection: &Selection) -> Result<PerformancePlan, String> {
        interpretation::validate_options(selection.mapping_profile, selection.count_in_note75)?;
        crate::hihat::validate_options(
            selection.mapping_profile,
            selection.hihat_policy,
            selection.hihat_openness,
        )?;
        if selection.tracks.is_empty() {
            return Err("분석할 드럼 트랙을 한 개 이상 선택하세요.".into());
        }
        if selection.channel.is_some_and(|c| c > 15) {
            return Err("MIDI 채널은 1–16 범위여야 합니다.".into());
        }
        let mut selected = selection.tracks.clone();
        selected.sort_unstable();
        selected.dedup();
        if selected.iter().any(|i| *i >= self.smf.tracks.len()) {
            return Err("선택한 MIDI 트랙이 존재하지 않습니다.".into());
        }
        let end_tick = selected.iter().map(|i| self.track_ends[*i]).max().unwrap();
        let duration = self.time_us(end_tick)? as f64 / 1_000_000.0;
        if duration > MAX_DURATION {
            return Err(format!(
                "선택한 트랙 길이가 {duration:.2}초입니다. 300초 이하의 MIDI를 사용하세요."
            ));
        }
        let kit = default_kit();
        let mut diagnostics = self.diagnostics.clone();
        let mut hits = Vec::new();
        let mut source_events = Vec::new();
        let mut interpretations = Vec::new();
        let mut unmapped: BTreeMap<(usize, u8, u8), usize> = BTreeMap::new();
        let omitted: BTreeSet<_> = selection
            .omitted_source_event_ids
            .iter()
            .map(String::as_str)
            .collect();
        let mut had_supported_strike = false;
        for index in selected {
            let mut tick = 0u64;
            for (event_index, event) in self.smf.tracks[index].iter().enumerate() {
                tick += event.delta.as_int() as u64;
                if let TrackEventKind::Midi { channel, message } = event.kind {
                    if selection.channel.is_some_and(|c| c != channel.as_int()) {
                        continue;
                    }
                    let source = SourceEvent {
                        id: format!("{index}:{event_index}"),
                        track: index,
                        event_index,
                        tick,
                        time_us: self.time_us(tick)?,
                        channel: channel.as_int(),
                        message: SourceMessage::from_midi(message),
                    };
                    let interpreted = interpretation::interpret(
                        &source,
                        selection.mapping_profile,
                        selection.count_in_note75,
                        omitted.contains(source.id.as_str()),
                    );
                    had_supported_strike |= interpreted.instrument.is_some();
                    if interpreted.kind == InterpretationKind::Unmapped {
                        if let SourceMessage::NoteOn { key, .. } = source.message {
                            *unmapped.entry((index, channel.as_int(), key)).or_default() += 1;
                        }
                    }
                    let is_strike = interpreted.kind == InterpretationKind::Strike;
                    interpretations.push(interpreted.clone());
                    source_events.push(source.clone());
                    if !is_strike {
                        continue;
                    }
                    let SourceMessage::NoteOn { key, velocity } = source.message else {
                        unreachable!();
                    };
                    let instrument = interpreted.instrument.unwrap();
                    let piece_id = interpreted.piece_id.unwrap();
                    if hits.len() >= MAX_HITS {
                        return Err(format!(
                            "선택한 타격이 {MAX_HITS}개를 초과합니다. 트랙이나 구간을 줄여주세요."
                        ));
                    }
                    let time_us = source.time_us;
                    hits.push(Hit {
                        id: 0,
                        time: time_us as f64 / 1_000_000.0,
                        time_us,
                        tick,
                        track: index,
                        event_index,
                        channel: channel.as_int(),
                        note: key,
                        velocity,
                        instrument,
                        piece_id: piece_id.clone(),
                        articulation: interpreted.articulation.unwrap(),
                        source_event_id: source.id,
                        limb: None,
                        target: kit.iter().find(|p| p.id == piece_id).unwrap().position,
                    });
                }
            }
        }
        for ((track, channel, note), count) in unmapped {
            diagnostics.push(Diagnostic::new("UNMAPPED_NOTE", format!("트랙 {} / 채널 {} / MIDI 음표 {note}: {count}개의 원본 이벤트를 보존했지만 선택한 매핑으로 연주할 수 없습니다.", track + 1, channel + 1)));
        }
        interpretation::validate_arrangement(
            &source_events,
            &selection.omitted_source_event_ids,
            &selection.hand_overrides,
            selection.mapping_profile,
            selection.count_in_note75,
        )?;
        if hits.is_empty() && !had_supported_strike {
            return Err("선택한 트랙/채널에 지원하는 드럼 타격이 없습니다. 트랙 선택과 GM 드럼 음표를 확인하세요.".into());
        }
        hits.sort_by_key(|h| (h.time_us, h.track, h.event_index));
        for (id, hit) in hits.iter_mut().enumerate() {
            hit.id = id;
        }
        source_events.sort_by_key(|event| (event.time_us, event.track, event.event_index));
        let hit_ids: BTreeMap<_, _> = hits
            .iter()
            .map(|hit| (hit.source_event_id.as_str(), hit.id))
            .collect();
        for interpreted in &mut interpretations {
            interpreted.hit_id = hit_ids.get(interpreted.source_event_id.as_str()).copied();
        }
        let hash = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ *b as u64).wrapping_mul(0x100000001b3)
        });
        let mut plan = PerformancePlan {
            schema_version: 2,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            planner_version: "beam-8-lookahead-2".into(),
            midi_hash: format!("fnv1a64:{hash:016x}"),
            calibration_id: "primitive-v1".into(),
            pre_roll: PRE_ROLL,
            tail: TAIL,
            duration,
            hits,
            diagnostics,
            kit,
            tempo_count: self.tempo_count,
            ppq: Some(self.info.ppq),
            end_tick: Some(end_tick),
            tempo_map: self
                .segments
                .iter()
                .map(|s| TempoAnchor {
                    tick: s.tick,
                    time: s.numerator as f64 / self.info.ppq as f64 / 1_000_000.0,
                    microseconds_per_quarter: s.micros,
                })
                .collect(),
            time_signatures: self.time_signatures.clone(),
            mapping_profile: selection.mapping_profile,
            mapping_version: interpretation::MAPPING_VERSION.into(),
            count_in_note75: selection.count_in_note75,
            source_events,
            interpretations,
            hihat_policy: selection.hihat_policy,
            hihat_openness: selection.hihat_openness,
            controls: Vec::new(),
            omitted_source_event_ids: selection.omitted_source_event_ids.clone(),
            hand_overrides: selection.hand_overrides.clone(),
        };
        crate::hihat::apply_policy(&mut plan);
        Ok(plan)
    }
}
