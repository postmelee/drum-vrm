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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitPiece {
    pub id: Instrument,
    pub label: String,
    pub position: Vec3,
    pub radius: f64,
}

pub fn default_kit() -> Vec<KitPiece> {
    use Instrument::*;
    [
        (Kick, "Kick", [0.20, 0.08, 0.62], 0.26),
        (Snare, "Snare", [-0.18, 0.79, 0.40], 0.22),
        (Hihat, "Hi-hat", [-0.48, 0.96, 0.44], 0.22),
        (OpenHat, "Open hi-hat", [-0.48, 0.96, 0.44], 0.22),
        (PedalHat, "Hi-hat pedal", [-0.22, 0.08, 0.55], 0.12),
        (TomHigh, "High tom", [-0.26, 1.00, 0.72], 0.18),
        (TomMid, "Mid tom", [0.18, 0.98, 0.74], 0.20),
        (TomLow, "Low tom", [0.62, 0.80, 0.42], 0.23),
        (Crash, "Crash", [-0.66, 1.26, 0.68], 0.30),
        (Ride, "Ride", [0.66, 1.15, 0.67], 0.28),
    ]
    .into_iter()
    .map(|(id, label, position, radius)| KitPiece {
        id,
        label: label.to_string(),
        position,
        radius,
    })
    .collect()
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub tracks: Vec<usize>,
    pub channel: Option<u8>,
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
}
