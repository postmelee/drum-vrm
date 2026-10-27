//! Versioned input semantics. Rendering names, body proportions and musical
//! inference do not belong in a note mapping profile.
use crate::model::{Hit, Instrument, PerformancePlan, MAX_DURATION, MAX_EVENTS, MAX_HITS};
use midly::MidiMessage;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAPPING_VERSION: &str = "1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingProfile {
    #[default]
    Gm,
    LogicDkdGm,
    SongsterrDownloader,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SourceMessage {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8, velocity: u8 },
    Controller { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    PitchBend { value: u16 },
    PolyAftertouch { key: u8, pressure: u8 },
    ChannelAftertouch { pressure: u8 },
}

impl SourceMessage {
    pub fn from_midi(message: MidiMessage) -> Self {
        match message {
            MidiMessage::NoteOn { key, vel } => Self::NoteOn {
                key: key.as_int(),
                velocity: vel.as_int(),
            },
            MidiMessage::NoteOff { key, vel } => Self::NoteOff {
                key: key.as_int(),
                velocity: vel.as_int(),
            },
            MidiMessage::Controller { controller, value } => Self::Controller {
                controller: controller.as_int(),
                value: value.as_int(),
            },
            MidiMessage::ProgramChange { program } => Self::ProgramChange {
                program: program.as_int(),
            },
            MidiMessage::PitchBend { bend } => Self::PitchBend {
                value: bend.0.as_int(),
            },
            MidiMessage::Aftertouch { key, vel } => Self::PolyAftertouch {
                key: key.as_int(),
                pressure: vel.as_int(),
            },
            MidiMessage::ChannelAftertouch { vel } => Self::ChannelAftertouch {
                pressure: vel.as_int(),
            },
        }
    }
    fn valid(&self) -> bool {
        match *self {
            Self::NoteOn { key, velocity } | Self::NoteOff { key, velocity } => {
                key < 128 && velocity < 128
            }
            Self::Controller { controller, value } => controller < 128 && value < 128,
            Self::ProgramChange { program } => program < 128,
            Self::PitchBend { value } => value < 16384,
            Self::PolyAftertouch { key, pressure } => key < 128 && pressure < 128,
            Self::ChannelAftertouch { pressure } => pressure < 128,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvent {
    pub id: String,
    pub track: usize,
    pub event_index: usize,
    pub tick: u64,
    pub time_us: u64,
    pub channel: u8,
    pub message: SourceMessage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InterpretationKind {
    Strike,
    CountIn,
    Unmapped,
    Preserved,
    Control,
    Omitted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interpretation {
    pub source_event_id: String,
    pub kind: InterpretationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<Instrument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub piece_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub articulation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub related_source_event_id: Option<String>,
}

pub struct MappedNote {
    pub instrument: Instrument,
    pub piece_id: &'static str,
    pub articulation: &'static str,
}

/// GM Standard here means the fixed note mapping, not GM + ModWheel or V-Drum.
/// Known but unimplemented splash/choke/control articulations remain unmapped.
pub fn mapped_note(profile: MappingProfile, note: u8) -> Option<MappedNote> {
    use Instrument::*;
    if profile == MappingProfile::SongsterrDownloader {
        return songsterr_downloader_note(note);
    }
    let logic = profile == MappingProfile::LogicDkdGm;
    let (instrument, piece_id, articulation) = match note {
        35 | 36 => (Kick, "kick", "center"),
        32 if logic => (Snare, "snare", "rimshotEdge"),
        34 if logic => (Snare, "snare", "edge"),
        37 => (Snare, "snare", "sideStick"),
        38 => (Snare, "snare", "center"),
        40 if logic => (Snare, "snare", "rimshot"),
        40 => (Snare, "snare", "electric"),
        42 => (Hihat, "hihat", "closed"),
        46 => (OpenHat, "openHat", "open"),
        33 if logic => (PedalHat, "pedalHat", "footClose"),
        44 => (PedalHat, "pedalHat", "footClose"),
        48 | 50 => (TomHigh, "tomHigh", "center"),
        45 | 47 => (TomMid, "tomMid", "center"),
        41 | 43 => (TomLow, "tomLow", "center"),
        49 => (Crash, "crash", "crash"),
        57 => (Crash, "crashRight", "crash"),
        52 if logic => (Ride, "ride", "edge"),
        52 => (Crash, "crash", "china"),
        55 if !logic => (Crash, "crash", "splash"),
        51 => (Ride, "ride", "outer"),
        53 => (Ride, "ride", "bell"),
        59 => (Ride, "ride", "inner"),
        _ => return None,
    };
    Some(MappedNote {
        instrument,
        piece_id,
        articulation,
    })
}

/// Compatibility with the alphaTab 1.8.1 GP7-index MIDI export bug, not a
/// general Songsterr/GM map. Keep source keys intact; never fall back to GM.
/// Factual correspondence and pinned upstream evidence: MIDI interpretation spec.
fn songsterr_downloader_note(note: u8) -> Option<MappedNote> {
    use Instrument::*;
    let (instrument, piece_id, articulation) = match note {
        0 => (Snare, "snare", "center"),
        1 => (Snare, "snare", "sideStick"),
        2 => (Snare, "snare", "rimshot"),
        3 => (Hihat, "hihat", "closed"),
        5 => (OpenHat, "openHat", "open"),
        6 => (PedalHat, "pedalHat", "footClose"),
        7 | 8 => (Kick, "kick", "center"),
        9 | 10 => (TomHigh, "tomHigh", "center"),
        11 | 12 => (TomMid, "tomMid", "center"),
        13 | 87 => (TomLow, "tomLow", "center"),
        14 => (Ride, "ride", "edge"),
        15 => (Ride, "ride", "outer"),
        16 => (Ride, "ride", "bell"),
        18 => (Crash, "crash", "splash"),
        20 => (Crash, "crash", "china"),
        22 => (Crash, "crash", "crash"),
        24 => (Crash, "crashRight", "crash"),
        // Half-open (4), chokes (17/19/21/23/25), other percussion and
        // potentially lossy/colliding exporter outputs remain diagnostic.
        _ => return None,
    };
    Some(MappedNote {
        instrument,
        piece_id,
        articulation,
    })
}

pub fn interpret(
    source: &SourceEvent,
    profile: MappingProfile,
    count_in_note75: bool,
    omitted: bool,
) -> Interpretation {
    let mut result = Interpretation {
        source_event_id: source.id.clone(),
        kind: InterpretationKind::Preserved,
        instrument: None,
        piece_id: None,
        articulation: None,
        hit_id: None,
        related_source_event_id: None,
    };
    if let SourceMessage::NoteOn { key, velocity } = source.message {
        if velocity > 0 {
            if count_in_note75 && key == 75 {
                result.kind = InterpretationKind::CountIn;
            } else if let Some(mapped) = mapped_note(profile, key) {
                result.kind = InterpretationKind::Strike;
                result.instrument = Some(mapped.instrument);
                result.piece_id = Some(mapped.piece_id.into());
                result.articulation = Some(mapped.articulation.into());
            } else {
                result.kind = InterpretationKind::Unmapped;
            }
            if omitted {
                // Preserve mapping metadata as context, but never create a Hit
                // or a control for an explicitly omitted source performance event.
                result.kind = InterpretationKind::Omitted;
            }
        }
    }
    result
}

pub fn validate_options(_profile: MappingProfile, _count_in_note75: bool) -> Result<(), String> {
    // Intent is explicitly selected by the user, independent of the producer.
    Ok(())
}

/// Arrangement directives address retained positive note-ons, never indexes in a
/// derived hit array. A hand choice is meaningful only for a mapped hand strike.
pub fn validate_arrangement(
    sources: &[SourceEvent],
    omitted_ids: &[String],
    hand_overrides: &BTreeMap<String, usize>,
    profile: MappingProfile,
    count_in_note75: bool,
) -> Result<(), String> {
    if omitted_ids.len() > MAX_EVENTS || hand_overrides.len() > MAX_HITS {
        return Err("연주 편집 지시가 이벤트 한계를 초과합니다.".into());
    }
    let sources: BTreeMap<_, _> = sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect();
    let mut omitted = BTreeSet::new();
    let positive_source = |id: &str| -> Result<&SourceEvent, String> {
        sources.get(id).copied().filter(|source| {
            matches!(source.message, SourceMessage::NoteOn { velocity, .. } if velocity > 0)
        }).ok_or_else(|| format!("연주 편집의 원본 이벤트 {id}가 선택한 양의 note-on에 없습니다."))
    };
    for id in omitted_ids {
        positive_source(id)?;
        if !omitted.insert(id.as_str()) {
            return Err("제외할 원본 이벤트가 중복되었습니다.".into());
        }
    }
    for (id, hand) in hand_overrides {
        let source = positive_source(id)?;
        let mapped = interpret(source, profile, count_in_note75, false);
        if *hand > 1
            || omitted.contains(id.as_str())
            || mapped.kind != InterpretationKind::Strike
            || matches!(
                mapped.instrument,
                Some(Instrument::Kick | Instrument::PedalHat) | None
            )
        {
            return Err(format!("손 지정 {id}는 제외하지 않은 손 타격에 왼손(0) 또는 오른손(1)만 사용할 수 있습니다."));
        }
    }
    Ok(())
}

/// Fill only fields that version one did not contain. Never invent discarded raw
/// channel events or claim a legacy plan has a verified source interpretation.
pub fn normalize_legacy(plan: &mut PerformancePlan) {
    if plan.schema_version != 1 {
        return;
    }
    for hit in &mut plan.hits {
        if hit.piece_id.is_empty() {
            hit.piece_id = hit.instrument.default_piece_id().into();
        }
        if hit.source_event_id.is_empty() {
            hit.source_event_id = format!("{}:{}", hit.track, hit.event_index);
        }
        if hit.articulation.is_empty() {
            hit.articulation = "legacy".into();
        }
    }
}

fn source_hit_matches(source: &SourceEvent, hit: &Hit) -> bool {
    matches!(source.message, SourceMessage::NoteOn { key, velocity } if key == hit.note && velocity == hit.velocity)
        && source.id == hit.source_event_id
        && source.track == hit.track
        && source.event_index == hit.event_index
        && source.tick == hit.tick
        && source.time_us == hit.time_us
        && source.channel == hit.channel
}

/// Validate provenance before any plan is re-evaluated or recalibrated. Target
/// positions and assigned limbs are allowed to change; source identity is not.
pub fn validate_interpretation(plan: &PerformancePlan) -> Result<(), String> {
    if plan.schema_version == 1 {
        if !plan.source_events.is_empty()
            || !plan.interpretations.is_empty()
            || plan.mapping_profile != MappingProfile::Gm
            || plan.count_in_note75
            || plan.hihat_policy != crate::hihat::HihatPolicy::Literal
            || !plan.controls.is_empty()
            || !plan.omitted_source_event_ids.is_empty()
            || !plan.hand_overrides.is_empty()
        {
            return Err("버전 1 연주 계획은 버전 2 해석 정보를 포함할 수 없습니다.".into());
        }
        return Ok(());
    }
    if plan.schema_version != 2 || plan.mapping_version != MAPPING_VERSION {
        return Err("지원하지 않는 MIDI 해석 버전입니다.".into());
    }
    validate_options(plan.mapping_profile, plan.count_in_note75)?;
    crate::hihat::validate_options(plan.mapping_profile, plan.hihat_policy, plan.hihat_openness)?;
    if plan.tempo_map.len() > MAX_EVENTS {
        return Err("MIDI 템포 구간이 한계를 초과합니다.".into());
    }
    let Some(ppq) = plan.ppq.filter(|value| *value > 0 && *value <= 32767) else {
        return Err("버전 2 연주 계획의 PPQ 시간축이 없습니다.".into());
    };
    // Reconstruct the same exact rational accumulator used by the parser. This
    // also checks retained controllers, not only events that produced a Hit.
    let mut segments = Vec::with_capacity(plan.tempo_map.len());
    let mut numerator = 0u128;
    for (index, anchor) in plan.tempo_map.iter().enumerate() {
        if anchor.tick > 9_007_199_254_740_991
            || anchor.microseconds_per_quarter == 0
            || anchor.microseconds_per_quarter > 0x00ff_ffff
            || !anchor.time.is_finite()
        {
            return Err("MIDI 템포 시간축이 유효하지 않습니다.".into());
        }
        if index == 0 {
            if anchor.tick != 0 {
                return Err("MIDI 템포 시간축은 tick 0에서 시작해야 합니다.".into());
            }
        } else {
            let previous = &plan.tempo_map[index - 1];
            if anchor.tick <= previous.tick {
                return Err("MIDI 템포 구간의 순서가 잘못되었습니다.".into());
            }
            numerator +=
                (anchor.tick - previous.tick) as u128 * previous.microseconds_per_quarter as u128;
        }
        if (anchor.time - numerator as f64 / ppq as f64 / 1_000_000.0).abs() > 1e-9 {
            return Err("MIDI 템포 구간의 누적 시각이 다릅니다.".into());
        }
        segments.push(numerator);
    }
    if segments.is_empty() {
        return Err("MIDI 템포 시간축이 없습니다.".into());
    }
    let Some(end_tick) = plan.end_tick.filter(|tick| *tick <= 9_007_199_254_740_991) else {
        return Err("버전 2 연주 계획의 종료 tick이 없습니다.".into());
    };
    let canonical_us = |tick: u64| {
        let segment = plan
            .tempo_map
            .partition_point(|anchor| anchor.tick <= tick)
            .saturating_sub(1);
        let anchor = &plan.tempo_map[segment];
        let exact = segments[segment]
            + (tick - anchor.tick) as u128 * anchor.microseconds_per_quarter as u128;
        (exact + ppq as u128 / 2) / ppq as u128
    };
    if !plan.duration.is_finite()
        || (plan.duration - canonical_us(end_tick) as f64 / 1_000_000.0).abs() > 1e-9
    {
        return Err("MIDI 종료 tick과 연주 길이가 일치하지 않습니다.".into());
    }
    if plan.source_events.len() > MAX_EVENTS
        || plan.interpretations.len() != plan.source_events.len()
        || plan.hits.len() > MAX_HITS
    {
        return Err("원본 이벤트와 MIDI 해석 개수가 일치하지 않거나 한계를 초과합니다.".into());
    }
    let interpretations: BTreeMap<_, _> = plan
        .interpretations
        .iter()
        .map(|i| (i.source_event_id.as_str(), i))
        .collect();
    if interpretations.len() != plan.interpretations.len() {
        return Err("중복 MIDI 해석입니다.".into());
    }
    validate_arrangement(
        &plan.source_events,
        &plan.omitted_source_event_ids,
        &plan.hand_overrides,
        plan.mapping_profile,
        plan.count_in_note75,
    )?;
    let omitted: BTreeSet<_> = plan
        .omitted_source_event_ids
        .iter()
        .map(String::as_str)
        .collect();
    let (expected_controls, _) = crate::hihat::derive_controls(plan);
    if expected_controls != plan.controls {
        return Err("하이햇 제어가 원본 이벤트와 선택한 해석 규칙에 일치하지 않습니다.".into());
    }
    let closes: BTreeMap<_, _> = expected_controls
        .iter()
        .map(|control| (control.source_event_id.as_str(), control))
        .collect();
    let mut seen = BTreeSet::new();
    let mut hit_ids = BTreeSet::new();
    let mut previous = None;
    for source in &plan.source_events {
        if source.id != format!("{}:{}", source.track, source.event_index)
            || source.track >= 512
            || source.event_index >= MAX_EVENTS
            || source.channel >= 16
            || source.tick > 9_007_199_254_740_991
            || source.time_us > (MAX_DURATION * 1_000_000.0) as u64
            || source.time_us as f64 / 1_000_000.0 > plan.duration + 1e-9
            || plan.end_tick.is_some_and(|end| source.tick > end)
            || !source.message.valid()
            || !seen.insert(source.id.as_str())
        {
            return Err("유효하지 않거나 중복된 원본 MIDI 이벤트입니다.".into());
        }
        if canonical_us(source.tick) != source.time_us as u128 {
            return Err("원본 MIDI 이벤트의 tick과 시각이 일치하지 않습니다.".into());
        }
        let order = (source.time_us, source.track, source.event_index);
        if previous.is_some_and(|p| p > order) {
            return Err("원본 MIDI 이벤트의 시간 순서가 잘못되었습니다.".into());
        }
        previous = Some(order);
        let Some(actual) = interpretations.get(source.id.as_str()) else {
            return Err("원본 이벤트의 해석이 없습니다.".into());
        };
        let mut expected = interpret(
            source,
            plan.mapping_profile,
            plan.count_in_note75,
            omitted.contains(source.id.as_str()),
        );
        if let Some(control) = closes.get(source.id.as_str()) {
            expected.kind = InterpretationKind::Control;
            expected.related_source_event_id = Some(control.open_source_event_id.clone());
        }
        if expected.kind == InterpretationKind::Strike {
            let Some(id) = actual.hit_id else {
                return Err("타격 해석에 연결된 타격이 없습니다.".into());
            };
            let Some(hit) = plan.hits.get(id) else {
                return Err("해석이 존재하지 않는 타격을 참조합니다.".into());
            };
            if hit.id != id
                || !hit_ids.insert(id)
                || !source_hit_matches(source, hit)
                || expected.instrument != Some(hit.instrument)
                || expected.piece_id.as_deref() != Some(hit.piece_id.as_str())
                || expected.articulation.as_deref() != Some(hit.articulation.as_str())
            {
                return Err("타격의 원본 또는 매핑 정보가 MIDI 해석과 다릅니다.".into());
            }
            expected.hit_id = Some(id);
        }
        if **actual != expected {
            return Err("원본 이벤트와 매핑 프로필의 해석이 일치하지 않습니다.".into());
        }
    }
    if hit_ids.len() != plan.hits.len() {
        return Err("원본 이벤트에 연결되지 않은 타격입니다.".into());
    }
    Ok(())
}
