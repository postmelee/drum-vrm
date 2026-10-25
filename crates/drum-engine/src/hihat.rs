//! Explicit visual intent over preserved GM Standard channel events. This is not
//! a reconstruction of a source instrument's samples or hidden pedal controller.
use crate::interpretation::{InterpretationKind, MappingProfile, SourceMessage};
use crate::model::{Diagnostic, PerformancePlan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HihatPolicy {
    #[default]
    Literal,
    ShortOpenClose,
}
pub fn default_openness() -> f64 {
    0.2
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlKind {
    HihatClose,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HihatControl {
    pub kind: ControlKind,
    pub source_event_id: String,
    pub open_source_event_id: String,
    pub release_start_time_us: i64,
    pub open_time_us: u64,
    pub time_us: u64,
    pub openness: f64,
}

pub fn validate_options(
    profile: MappingProfile,
    policy: HihatPolicy,
    openness: f64,
) -> Result<(), String> {
    if !openness.is_finite() || !(0.05..=1.0).contains(&openness) {
        return Err("하이햇 개방량은 0.05–1.0 범위여야 합니다.".into());
    }
    if policy == HihatPolicy::ShortOpenClose && profile != MappingProfile::LogicDkdGm {
        return Err("짧은 오픈·닫힘 해석은 Logic Drum Kit Designer GM 매핑에서 선택하세요.".into());
    }
    Ok(())
}

/// Candidates use actual elapsed time plus tempo at the open event, so the
/// policy does not hard-code one song's note spacing or read a file/avatar name.
pub fn derive_controls(plan: &PerformancePlan) -> (Vec<HihatControl>, Vec<Diagnostic>) {
    if plan.hihat_policy == HihatPolicy::Literal {
        return (Vec::new(), Vec::new());
    }
    #[derive(Clone, Copy)]
    struct PendingOpen {
        index: usize,
        release_start_time_us: i64,
        closure_generation: u64,
    }
    let mut pending: [Option<PendingOpen>; 16] = [None; 16];
    let mut last_closure_time_us: Option<u64> = None;
    let mut closure_generation = 0;
    let mut controls = Vec::<HihatControl>::new();
    let mut diagnostics = Vec::new();
    for (index, source) in plan.source_events.iter().enumerate() {
        let Some(open_slot) = pending.get_mut(source.channel as usize) else {
            continue;
        };
        let SourceMessage::NoteOn { key, velocity } = source.message else {
            continue;
        };
        if velocity == 0 {
            continue;
        }
        match key {
            46 => {
                *open_slot = Some(PendingOpen {
                    index,
                    release_start_time_us: (source.time_us as i64 - 100_000)
                        .max(last_closure_time_us.map_or(i64::MIN, |time| time as i64)),
                    closure_generation,
                });
            }
            42 | 33 => *open_slot = None,
            44 => {
                let candidate = open_slot.take();
                // Every physical closure constrains future preparation, whether
                // this event becomes a control or remains a literal foot strike.
                let preceding_generation = closure_generation;
                last_closure_time_us = Some(source.time_us);
                closure_generation += 1;
                let Some(candidate) = candidate else {
                    continue;
                };
                let open = &plan.source_events[candidate.index];
                let Some(gap) = source.time_us.checked_sub(open.time_us) else {
                    continue;
                };
                let tempo_index = plan
                    .tempo_map
                    .partition_point(|tempo| tempo.tick <= open.tick);
                let quarter_us = tempo_index
                    .checked_sub(1)
                    .and_then(|index| plan.tempo_map.get(index))
                    .map_or(500_000, |tempo| tempo.microseconds_per_quarter)
                    as u64;
                if gap == 0 || gap > 80_000 || gap * 4 > quarter_us {
                    continue;
                }
                // One physical hi-hat cannot simultaneously follow conflicting
                // channel curves. Preserve the conflicting foot strike literally.
                if candidate.closure_generation != preceding_generation
                    || candidate.release_start_time_us >= open.time_us as i64
                    || controls
                        .last()
                        .is_some_and(|previous| open.time_us <= previous.time_us)
                {
                    let mut diagnostic = Diagnostic::new("HIHAT_INTENT_CONFLICT", format!("원본 이벤트 {}의 짧은 하이햇 닫힘이 다른 닫힘 또는 제어 구간과 충돌하여 독립 페달 타격으로 유지했습니다.", source.id));
                    diagnostic.time = Some(source.time_us as f64 / 1_000_000.0);
                    diagnostics.push(diagnostic);
                    continue;
                }
                controls.push(HihatControl {
                    kind: ControlKind::HihatClose,
                    source_event_id: source.id.clone(),
                    open_source_event_id: open.id.clone(),
                    release_start_time_us: candidate.release_start_time_us,
                    open_time_us: open.time_us,
                    time_us: source.time_us,
                    openness: plan.hihat_openness,
                });
            }
            _ => {}
        }
        if matches!(key, 42 | 33) {
            last_closure_time_us = Some(source.time_us);
            closure_generation += 1;
        }
    }
    (controls, diagnostics)
}

pub fn apply_policy(plan: &mut PerformancePlan) {
    let (controls, diagnostics) = derive_controls(plan);
    let closes: BTreeMap<_, _> = controls
        .iter()
        .map(|control| (control.source_event_id.as_str(), control))
        .collect();
    plan.hits
        .retain(|hit| !closes.contains_key(hit.source_event_id.as_str()));
    for (id, hit) in plan.hits.iter_mut().enumerate() {
        hit.id = id;
    }
    let hit_ids: BTreeMap<_, _> = plan
        .hits
        .iter()
        .map(|hit| (hit.source_event_id.as_str(), hit.id))
        .collect();
    for interpretation in &mut plan.interpretations {
        interpretation.hit_id = hit_ids
            .get(interpretation.source_event_id.as_str())
            .copied();
        if let Some(control) = closes.get(interpretation.source_event_id.as_str()) {
            interpretation.kind = InterpretationKind::Control;
            interpretation.related_source_event_id = Some(control.open_source_event_id.clone());
        }
    }
    plan.diagnostics.extend(diagnostics);
    plan.controls = controls;
}

/// Absolute-time openness: supported release, contact and closure. The 100 ms
/// preparation is shortened after a previous close, never by shifting MIDI time.
pub fn evaluate(controls: &[HihatControl], song_time: f64) -> f64 {
    if !song_time.is_finite() {
        return 0.0;
    }
    let index =
        controls.partition_point(|control| control.time_us as f64 / 1_000_000.0 < song_time);
    let Some(control) = controls.get(index) else {
        return 0.0;
    };
    let open = control.open_time_us as f64 / 1_000_000.0;
    let close = control.time_us as f64 / 1_000_000.0;
    let start = control.release_start_time_us as f64 / 1_000_000.0;
    let smooth = |value: f64| {
        let t = value.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    if song_time < start {
        0.0
    } else if song_time <= open {
        control.openness * smooth((song_time - start) / (open - start))
    } else {
        control.openness * (1.0 - smooth((song_time - open) / (close - open)))
    }
}

#[cfg(test)]
#[path = "hihat_tests.rs"]
mod tests;
