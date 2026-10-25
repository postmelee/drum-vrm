//! Deterministic default groove roles and soft technique candidates, not a
//! transcription of the original drummer or physiological limits. Reach, timing
//! and simultaneous limb occupancy are checked separately by the planner.
use crate::model::{Hit, Instrument, PerformancePlan};

#[derive(Clone, Copy)]
pub(super) struct Context {
    pub preferred: usize,
    /// Default right-handed groove ownership, below feasibility/explicit edits
    /// but above geometric convenience. Fills remain free to use both hands.
    pub role: Option<usize>,
    pub quarter: f64,
    pub horizon: f64,
}

fn groove(instrument: Instrument) -> bool {
    matches!(
        instrument,
        Instrument::Hihat | Instrument::OpenHat | Instrument::Ride
    )
}

fn head(instrument: Instrument) -> bool {
    matches!(
        instrument,
        Instrument::Snare | Instrument::TomHigh | Instrument::TomMid | Instrument::TomLow
    )
}

pub(super) fn contexts(plan: &PerformancePlan) -> Vec<Context> {
    let mut next_groove = vec![None; plan.hits.len()];
    let mut next_head = vec![None; plan.hits.len()];
    let mut next = None;
    let mut following_head = None;
    for (index, hit) in plan.hits.iter().enumerate().rev() {
        next_head[index] = following_head;
        if head(hit.instrument) {
            following_head = Some(hit.time);
        }
        if groove(hit.instrument) {
            next = Some(hit.time);
        }
        next_groove[index] = next;
    }
    let mut previous: Option<f64> = None;
    let mut previous_head: Option<f64> = None;
    plan.hits
        .iter()
        .enumerate()
        .map(|(index, hit)| {
            if groove(hit.instrument) {
                previous = Some(hit.time);
            }
            let tempo_at = plan
                .tempo_map
                .partition_point(|tempo| tempo.tick <= hit.tick);
            let quarter = tempo_at.checked_sub(1).map_or(0.5, |at| {
                plan.tempo_map[at].microseconds_per_quarter as f64 / 1e6
            });
            let groove_window = (quarter * 2.0).clamp(0.35, 1.2);
            let near_groove = previous
                .into_iter()
                .chain(next_groove[index])
                .any(|time| (time - hit.time).abs() <= groove_window);
            // A fill ending on a drum head breaks cymbal-groove ownership. The
            // next hi-hat must not pull its preceding crash into the right hand
            // across a completed tom/snare run.
            let after_fill = previous_head.is_some_and(|time| {
                hit.time > time
                    && hit.time - time <= groove_window
                    && previous.is_none_or(|groove_time| time > groove_time)
            });
            let preferred = if hit.instrument == Instrument::Crash
                && (hit.piece_id == "crashRight" || (near_groove && !after_fill))
            {
                1
            } else {
                super::preferred_limb(hit.instrument)
            };
            let fill_window = (quarter * 0.8).clamp(0.15, 0.4);
            let in_head_run = previous_head
                .into_iter()
                .chain(next_head[index])
                .any(|time| {
                    let dt = (time - hit.time).abs();
                    dt > 0.0 && dt <= fill_window
                });
            let role = if groove(hit.instrument)
                || (hit.instrument == Instrument::Crash && near_groove && !after_fill)
            {
                Some(1)
            } else if hit.instrument == Instrument::Snare && near_groove && !in_head_run {
                Some(0)
            } else {
                None
            };
            if head(hit.instrument) {
                previous_head = Some(hit.time);
            }
            Context {
                preferred,
                role,
                quarter,
                horizon: (quarter * 2.0).clamp(0.35, 1.2),
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
struct HandEvent {
    limb: usize,
    time: f64,
    instrument: Instrument,
    target: [f64; 3],
    velocity: u8,
}

#[derive(Clone, Default)]
pub(super) struct History {
    events: Vec<HandEvent>,
}

impl History {
    pub fn record(&mut self, limb: usize, hit: &Hit) {
        if limb >= 2 {
            return;
        }
        if self.events.len() == 8 {
            self.events.remove(0);
        }
        self.events.push(HandEvent {
            limb,
            time: hit.time,
            instrument: hit.instrument,
            target: hit.target,
            velocity: hit.velocity,
        });
    }

    pub fn cost(&self, limb: usize, hit: &Hit, context: Context) -> f64 {
        if limb >= 2 || !head(hit.instrument) {
            return 0.0;
        }
        let Some(previous) = self.events.last() else {
            return 0.0;
        };
        // One phrase's candidate vocabulary does not propagate through a cymbal
        // groove, a simultaneous chord or a long rest.
        let window = (context.quarter * 0.8).clamp(0.15, 0.4);
        let dt = hit.time - previous.time;
        if !head(previous.instrument) || dt <= 0.0 || dt > window {
            return 0.0;
        }
        let mut cost = 0.0;
        if previous.limb == limb {
            let third = self.events.iter().rev().nth(1).is_some_and(|before| {
                head(before.instrument)
                    && before.limb == limb
                    && previous.time > before.time
                    && previous.time - before.time <= window
            });
            let same_surface = super::distance(previous.target, hit.target) < 0.025;
            let rebound_tap = same_surface && hit.velocity as f64 <= previous.velocity as f64 * 0.8;
            // A controlled accent→tap double can be cheaper than changing hands;
            // an extended one-hand run stays expensive, including across toms.
            cost += if third {
                3.0 + 0.08 / dt
            } else if rebound_tap {
                0.12
            } else {
                0.65 + 0.025 / dt
            };
        }
        let mut hands = vec![limb];
        let mut time = hit.time;
        for event in self.events.iter().rev() {
            if !head(event.instrument) || time <= event.time || time - event.time > window {
                break;
            }
            hands.push(event.limb);
            time = event.time;
            if hands.len() == 8 {
                break;
            }
        }
        hands.reverse();
        // All rotations and both lead hands remain candidates. The weak cost
        // admits singles, doubles and paradiddles without forcing every fill
        // into a rudiment or treating an incomplete fragment as proof of intent.
        let patterns: [&[usize]; 3] = [&[0, 1], &[0, 0, 1, 1], &[0, 1, 0, 0, 1, 0, 1, 1]];
        let mismatch = patterns
            .iter()
            .flat_map(|pattern| {
                (0..pattern.len())
                    .flat_map(move |phase| (0..2).map(move |mirror| (*pattern, phase, mirror)))
            })
            .map(|(pattern, phase, mirror)| {
                hands
                    .iter()
                    .enumerate()
                    .filter(|(index, hand)| {
                        **hand != (pattern[(index + phase) % pattern.len()] ^ mirror)
                    })
                    .count()
            })
            .min()
            .unwrap_or(0);
        cost + mismatch as f64 * 0.12
    }
}

/// Accent-aware transfer envelope: a low following tap stays low after an
/// accent, while a following accent receives later preparation. The endpoints
/// remain zero and its apex stays bounded by the normal velocity lift.
pub(super) fn stroke_envelope(phase: f64, previous: u8, next: u8) -> f64 {
    let phase = phase.clamp(0.0, 1.0);
    let peak = if next > previous.saturating_add(20) {
        0.67
    } else if previous > next.saturating_add(20) {
        0.33
    } else {
        0.5
    };
    let local = if phase <= peak {
        phase / peak
    } else {
        (1.0 - phase) / (1.0 - peak)
    };
    // A squared sine has zero velocity at contacts and at the peak.
    (local * std::f64::consts::FRAC_PI_2).sin().powi(2)
}
