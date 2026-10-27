use serde::{Deserialize, Serialize};

pub type Vec3 = [f64; 3];
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_EVENTS: usize = 200_000;
pub const MAX_HITS: usize = 50_000;
pub const MAX_DURATION: f64 = 300.0;
pub const PRE_ROLL: f64 = 0.5;
pub const TAIL: f64 = 0.25;
pub const POSE_FLOATS: usize = 57;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Instrument {
    Kick,
    Snare,
    Hihat,
    OpenHat,
    PedalHat,
    TomHigh,
    TomMid,
    TomLow,
    Crash,
    Ride,
}

impl Instrument {
    pub fn default_piece_id(self) -> &'static str {
        match self {
            Self::Kick => "kick",
            Self::Snare => "snare",
            Self::Hihat => "hihat",
            Self::OpenHat => "openHat",
            Self::PedalHat => "pedalHat",
            Self::TomHigh => "tomHigh",
            Self::TomMid => "tomMid",
            Self::TomLow => "tomLow",
            Self::Crash => "crash",
            Self::Ride => "ride",
        }
    }
}

pub fn map_note(note: u8) -> Option<Instrument> {
    use Instrument::*;
    Some(match note {
        35 | 36 => Kick,
        37 | 38 | 40 => Snare,
        42 => Hihat,
        46 => OpenHat,
        44 => PedalHat,
        48 | 50 => TomHigh,
        45 | 47 => TomMid,
        41 | 43 => TomLow,
        49 | 52 | 55 | 57 => Crash,
        51 | 53 | 59 => Ride,
        _ => return None,
    })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KitPiece {
    pub id: String,
    pub instrument: Instrument,
    pub label: String,
    pub position: Vec3,
    pub radius: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<crate::equipment::PieceGeometry>,
}

impl<'de> Deserialize<'de> for KitPiece {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Fields {
            id: String,
            instrument: Option<Instrument>,
            label: String,
            position: Vec3,
            radius: f64,
            geometry: Option<crate::equipment::PieceGeometry>,
        }
        let fields = Fields::deserialize(deserializer)?;
        // Version-one layouts used the instrument enum as their piece ID.
        // Infer only that documented legacy representation, never an arbitrary name.
        let instrument = match fields.instrument {
            Some(value) => value,
            None => serde_json::from_value(serde_json::Value::String(fields.id.clone()))
                .map_err(serde::de::Error::custom)?,
        };
        Ok(Self {
            id: fields.id,
            instrument,
            label: fields.label,
            position: fields.position,
            radius: fields.radius,
            geometry: fields.geometry,
        })
    }
}

pub fn default_kit() -> Vec<KitPiece> {
    serde_json::from_str(include_str!("../../../shared/default-kit.json"))
        .expect("valid shared default kit")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub id: usize,
    pub time: f64,
    pub time_us: u64,
    pub tick: u64,
    pub track: usize,
    pub event_index: usize,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
    pub instrument: Instrument,
    #[serde(default)]
    pub piece_id: String,
    #[serde(default)]
    pub articulation: String,
    #[serde(default)]
    pub source_event_id: String,
    pub limb: Option<usize>,
    pub target: Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<f64>,
}

impl Diagnostic {
    pub fn new(code: &str, message: String) -> Self {
        Self {
            code: code.into(),
            message,
            hit_id: None,
            time: None,
        }
    }
    pub fn for_hit(code: &str, message: String, hit: &Hit) -> Self {
        Self {
            code: code.into(),
            message,
            hit_id: Some(hit.id),
            time: Some(hit.time),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub index: usize,
    pub name: String,
    pub note_count: usize,
    pub drum_notes: usize,
    pub channels: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiInfo {
    pub tracks: Vec<TrackInfo>,
    pub suggested_tracks: Vec<usize>,
    pub format: u8,
    pub ppq: u16,
    pub suggested_channel: Option<u8>,
    pub mapping_analysis: crate::interpretation::MappingAnalysis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub tracks: Vec<usize>,
    pub channel: Option<u8>,
    #[serde(default)]
    pub mapping_profile: crate::interpretation::MappingProfile,
    #[serde(default)]
    pub count_in_note75: bool,
    #[serde(default)]
    pub hihat_policy: crate::hihat::HihatPolicy,
    #[serde(default = "crate::hihat::default_openness")]
    pub hihat_openness: f64,
    #[serde(default)]
    pub omitted_source_event_ids: Vec<String>,
    #[serde(default)]
    pub hand_overrides: std::collections::BTreeMap<String, usize>,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            channel: None,
            mapping_profile: Default::default(),
            count_in_note75: false,
            hihat_policy: Default::default(),
            hihat_openness: crate::hihat::default_openness(),
            omitted_source_event_ids: Vec::new(),
            hand_overrides: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TempoAnchor {
    pub tick: u64,
    /// Unrounded segment origin in seconds, for beat-grid conversion. Hit timeUs
    /// remains the canonical once-rounded timestamp used by sound and motion.
    pub time: f64,
    pub microseconds_per_quarter: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSignature {
    pub tick: u64,
    pub numerator: u8,
    pub denominator: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformancePlan {
    pub schema_version: u32,
    pub engine_version: String,
    pub planner_version: String,
    pub midi_hash: String,
    pub calibration_id: String,
    pub pre_roll: f64,
    pub tail: f64,
    pub duration: f64,
    pub hits: Vec<Hit>,
    /// Derived after assignment. Imported copies never override validated hits or kit geometry.
    #[serde(default)]
    pub contact_events: Vec<crate::contact::ContactEvent>,
    #[serde(default)]
    pub hihat_motion: crate::hihat_motion::HihatMotionTrack,
    pub diagnostics: Vec<Diagnostic>,
    pub kit: Vec<KitPiece>,
    pub tempo_count: usize,
    #[serde(default)]
    pub ppq: Option<u16>,
    #[serde(default)]
    pub end_tick: Option<u64>,
    #[serde(default)]
    pub tempo_map: Vec<TempoAnchor>,
    #[serde(default)]
    pub time_signatures: Vec<TimeSignature>,
    #[serde(default)]
    pub mapping_profile: crate::interpretation::MappingProfile,
    #[serde(default)]
    pub mapping_version: String,
    #[serde(default)]
    pub count_in_note75: bool,
    #[serde(default)]
    pub source_events: Vec<crate::interpretation::SourceEvent>,
    #[serde(default)]
    pub interpretations: Vec<crate::interpretation::Interpretation>,
    #[serde(default)]
    pub hihat_policy: crate::hihat::HihatPolicy,
    #[serde(default = "crate::hihat::default_openness")]
    pub hihat_openness: f64,
    #[serde(default)]
    pub controls: Vec<crate::hihat::HihatControl>,
    #[serde(default)]
    pub omitted_source_event_ids: Vec<String>,
    #[serde(default)]
    pub hand_overrides: std::collections::BTreeMap<String, usize>,
}
