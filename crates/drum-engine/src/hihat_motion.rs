//! Shared absolute-time motion of the upper hi-hat, independently of sound synthesis.
use crate::{
    equipment::HIHAT_MAX_OPENING,
    model::{Instrument, PerformancePlan},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HihatStateAnchor {
    pub time: f64,
    pub opening: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HihatMotionTrack {
    pub travel_meters: f64,
    pub anchors: Vec<HihatStateAnchor>,
}
impl Default for HihatMotionTrack {
    fn default() -> Self {
        Self {
            travel_meters: HIHAT_MAX_OPENING,
            anchors: Vec::new(),
        }
    }
}

/// Explicit short-open controls retain all three endpoints. Literal notes set a
/// full-open/closed visual state; they do not claim measured acoustic openness.
pub fn prepare(plan: &PerformancePlan) -> HihatMotionTrack {
    let controlled: BTreeSet<_> = plan
        .controls
        .iter()
        .map(|c| c.open_source_event_id.as_str())
        .collect();
    let mut events: Vec<(f64, f64)> = plan
        .hits
        .iter()
        .filter_map(|hit| {
            if controlled.contains(hit.source_event_id.as_str()) {
                return None;
            }
            match hit.instrument {
                Instrument::OpenHat => Some((hit.time, 1.)),
                Instrument::Hihat | Instrument::PedalHat => Some((hit.time, 0.)),
                _ => None,
            }
        })
        .collect();
    for control in &plan.controls {
        events.extend([
            (control.release_start_time_us as f64 / 1_000_000., 0.),
            (control.open_time_us as f64 / 1_000_000., control.openness),
            (control.time_us as f64 / 1_000_000., 0.),
        ]);
    }
    // A literal foot chick needs a release before closing even if the preceding
    // state was already closed. Do not let this release cross the previous
    // authoritative hand/foot/control endpoint.
    let mut endpoints = events.clone();
    endpoints.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.total_cmp(&a.1)));
    for hit in plan
        .hits
        .iter()
        .filter(|hit| hit.instrument == Instrument::PedalHat)
    {
        let at = endpoints.partition_point(|event| event.0 < hit.time);
        let previous = at.checked_sub(1).map(|index| &endpoints[index]);
        if previous.is_some_and(|event| event.1 > 0.) {
            continue;
        }
        let start =
            (hit.time - 0.16).max(previous.map_or(-crate::model::PRE_ROLL, |event| event.0));
        if start < hit.time {
            events.extend([
                (start, 0.),
                (
                    (start + hit.time) * 0.5,
                    0.35 + 0.65 * hit.velocity as f64 / 127.,
                ),
            ]);
        }
    }
    events.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.total_cmp(&a.1)));
    // A simultaneous closure wins instead of declaring two physical states.
    let mut targets: Vec<(f64, f64)> = Vec::new();
    for event in events {
        if let Some(last) = targets.last_mut().filter(|last| last.0 == event.0) {
            last.1 = last.1.min(event.1);
        } else {
            targets.push(event);
        }
    }
    let mut result = HihatMotionTrack::default();
    let Some(first) = targets.first() else {
        return result;
    };
    let mut previous_time = (first.0 - 0.1).min(-crate::model::PRE_ROLL);
    let mut previous_opening = 0.;
    result.anchors.push(HihatStateAnchor {
        time: previous_time,
        opening: 0.,
    });
    for (time, opening) in targets {
        if opening != previous_opening {
            let start = (time - 0.1).max(previous_time);
            if start > result.anchors.last().unwrap().time {
                result.anchors.push(HihatStateAnchor {
                    time: start,
                    opening: previous_opening,
                });
            }
        }
        if time > result.anchors.last().unwrap().time {
            result.anchors.push(HihatStateAnchor { time, opening });
        } else {
            result.anchors.last_mut().unwrap().opening = opening;
        }
        previous_time = time;
        previous_opening = opening;
    }
    result
}

pub fn evaluate(track: &HihatMotionTrack, time: f64) -> f64 {
    if !time.is_finite() {
        return 0.;
    }
    let index = track.anchors.partition_point(|anchor| anchor.time <= time);
    let Some(previous) = index.checked_sub(1).and_then(|i| track.anchors.get(i)) else {
        return 0.;
    };
    let Some(next) = track.anchors.get(index) else {
        return previous.opening;
    };
    let t = ((time - previous.time) / (next.time - previous.time)).clamp(0., 1.);
    previous.opening + (next.opening - previous.opening) * t * t * (3. - 2. * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absolute_samples_retain_endpoints_and_hold() {
        let track = HihatMotionTrack {
            travel_meters: HIHAT_MAX_OPENING,
            anchors: vec![
                HihatStateAnchor {
                    time: 0.,
                    opening: 0.,
                },
                HihatStateAnchor {
                    time: 0.1,
                    opening: 0.2,
                },
                HihatStateAnchor {
                    time: 0.12,
                    opening: 0.,
                },
            ],
        };
        for time in [0.1, 0., 0.12, 0.05, 0.11, 2., 0.1] {
            let value = evaluate(&track, time);
            assert!((0.0..=0.2).contains(&value));
        }
        assert_eq!(evaluate(&track, 0.1), 0.2);
        assert_eq!(evaluate(&track, 0.12), 0.);
        assert!((evaluate(&track, 0.05) - 0.1).abs() < 1e-12);
        assert!((evaluate(&track, 0.11) - 0.1).abs() < 1e-12);
    }
}
