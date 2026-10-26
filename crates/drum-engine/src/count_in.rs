//! Explicit count-in presentation, independent of musical strikes and audio.
//! Tip-to-tip contact is an engineering gesture, not an inferred drum technique.
use crate::{
    clearance, equipment_motion,
    interpretation::{InterpretationKind, SourceMessage},
    model::{Diagnostic, Instrument, PerformancePlan, Vec3},
    motion::{add, distance, sub},
    posture,
    rig::{HandSolution, RigProfile},
};
use std::collections::BTreeSet;

pub const WINDOW: f64 = 0.22;
const MIN_INTERVAL: f64 = 0.12;
const CONTACT_TOLERANCE: f64 = 0.002;
const TIP_CONTACT_REGION: f64 = 0.04;

#[derive(Clone, Debug)]
pub struct CountInCue {
    pub source_event_id: String,
    pub time: f64,
    pub contact_point: Vec3,
}
#[derive(Clone, Debug)]
struct Phrase {
    start: f64,
    end: f64,
    times: Vec<f64>,
}
#[derive(Clone, Debug, Default)]
pub struct CountInTrack {
    pub cues: Vec<CountInCue>,
    pub diagnostics: Vec<Diagnostic>,
    phrases: Vec<Phrase>,
    contact: Vec3,
    opening: [Vec3; 2],
    pitches: [f64; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CountInSample {
    pub tips: [Vec3; 2],
    /// Solver angles, the negative of motion::stroke_at's wrist pitch.
    pub pitches: [f64; 2],
    pub intended_stick_contact: bool,
    pub contact_point: Option<Vec3>,
}
fn mul(v: Vec3, s: f64) -> Vec3 {
    v.map(|x| x * s)
}
fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn events(plan: &PerformancePlan) -> Vec<(String, f64)> {
    let wanted: BTreeSet<_> = plan
        .interpretations
        .iter()
        .filter(|i| i.kind == InterpretationKind::CountIn)
        .map(|i| i.source_event_id.as_str())
        .collect();
    let mut events: Vec<_> = plan
        .source_events
        .iter()
        .filter(|e| {
            wanted.contains(e.id.as_str())
                && matches!(e.message, SourceMessage::NoteOn { velocity: 1.., .. })
        })
        .map(|e| (e.id.clone(), e.time_us as f64 / 1_000_000.0))
        .collect();
    events.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    events
}
/// Independent of body feasibility so prepared-track validation has the same
/// control times even when a cue cannot be presented on a particular rig.
pub fn control_times_for_plan(plan: &PerformancePlan) -> Vec<f64> {
    let mut result: Vec<_> = events(plan)
        .into_iter()
        .flat_map(|(_, t)| [t - WINDOW, t, t + WINDOW])
        .collect();
    result.sort_by(f64::total_cmp);
    result.dedup();
    result
}

/// Deliberate contact only exempts the terminal 40 mm of two tools. Their
/// remaining complete shafts, arms and opposite hands remain obstacles.
pub fn contact_pair_clearance(
    profile: &RigProfile,
    left: &HandSolution,
    right: &HandSolution,
    intended: bool,
) -> clearance::PairClearance {
    let mut pair = clearance::pair_clearance(profile, left, right);
    if !intended || distance(left.tip, right.tip) > CONTACT_TOLERANCE {
        return pair;
    }
    let hands = [left, right];
    let full: [(Vec3, Vec3); 2] =
        std::array::from_fn(|limb| equipment_motion::stick_segment(profile, limb, hands[limb]));
    let segments: [(Vec3, Vec3); 2] = std::array::from_fn(|limb| {
        let (butt, tip) = equipment_motion::stick_segment(profile, limb, hands[limb]);
        let length = distance(butt, tip).max(1e-9);
        (
            butt,
            add(
                tip,
                mul(sub(butt, tip), TIP_CONTACT_REGION.min(length) / length),
            ),
        )
    });
    pair.sticks =
        clearance::segment_distance(segments[0].0, segments[0].1, full[1].0, full[1].1).min(
            clearance::segment_distance(full[0].0, full[0].1, segments[1].0, segments[1].1),
        ) - 0.012;
    pair
}

fn witness(
    profile: &RigProfile,
    equipment: Option<&equipment_motion::EquipmentContext>,
    tips: [Vec3; 2],
    pitches: [f64; 2],
    contact: bool,
) -> bool {
    let candidates: [Vec<_>; 2] = std::array::from_fn(|i| {
        posture::hand_candidates(profile, i, tips[i], pitches[i])
            .filter(|hand| {
                equipment.is_none_or(|ctx| {
                    equipment_motion::hand_clearance(profile, i, hand, ctx, None) >= posture::MARGIN
                })
            })
            .collect()
    });
    candidates[0].iter().any(|left| {
        candidates[1].iter().any(|right| {
            let clear = if contact {
                contact_pair_clearance(profile, left, right, true).minimum()
            } else {
                clearance::pair_clearance(profile, left, right).minimum()
            };
            clear >= posture::MARGIN
        })
    })
}

impl CountInTrack {
    pub fn prepare(plan: &PerformancePlan, profile: &RigProfile) -> Self {
        let mut result = Self::default();
        let events = events(plan);
        if events.is_empty() {
            return result;
        }
        let mut hand_times: Vec<_> = plan
            .hits
            .iter()
            .filter(|h| !matches!(h.instrument, Instrument::Kick | Instrument::PedalHat))
            .map(|h| h.time)
            .collect();
        hand_times.sort_by(f64::total_cmp);
        let mut free = Vec::new();
        for (id, time) in events {
            let next = hand_times.partition_point(|t| *t < time - WINDOW);
            if hand_times.get(next).is_some_and(|t| *t <= time + WINDOW) {
                result.unavailable(time, &id, "양손 준비·복귀 구간에 실제 손 타격이 있습니다");
            } else if free
                .last()
                .is_some_and(|(_, previous)| time > *previous && time - previous < MIN_INTERVAL)
            {
                result.unavailable(
                    time,
                    &id,
                    "이 스틱 접촉 표현의 최소 0.12초 준비 간격보다 짧습니다",
                );
            } else {
                free.push((id, time));
            }
        }
        if free.is_empty() {
            return result;
        }
        let equipment = equipment_motion::EquipmentContext::for_kit(&plan.kit);
        let center = mul(add(profile.roots[0], profile.roots[1]), 0.5);
        let shoulder = sub(profile.roots[1], profile.roots[0]);
        let width = shoulder[0].hypot(shoulder[2]).max(1e-9);
        let right = [shoulder[0] / width, 0.0, shoulder[2] / width];
        let forward = [-right[2], 0.0, right[0]];
        let arm = profile.lengths[..2]
            .iter()
            .map(|l| l[0] + l[1])
            .fold(f64::INFINITY, f64::min);
        // The contact is a tool tip, not a wrist. An arms-only forward distance
        // puts short-armed rigs' wrists behind their torso when sticks are long.
        let tool = (0..2)
            .map(|limb| {
                distance(
                    add(profile.grip_offsets[limb], profile.tip_offsets[limb]),
                    [0.0; 3],
                )
            })
            .sum::<f64>()
            * 0.5;
        let mut pose = None;
        'search: for down in [0.40, 0.30, 0.50, 0.20] {
            for front in [0.40, 0.30, 0.50, 0.20, 0.60] {
                let contact = add(
                    add(center, mul(forward, arm * front + tool * 0.80)),
                    [0.0, -arm * down, 0.0],
                );
                for pitch in [0.0, -0.15, 0.15, -0.30] {
                    let pitches = [pitch; 2];
                    let opening = [
                        add(mul(right, -0.045), [0.0, 0.045, 0.0]),
                        add(mul(right, 0.045), [0.0, 0.045, 0.0]),
                    ];
                    if !witness(profile, equipment.as_deref(), [contact; 2], pitches, true) {
                        continue;
                    }
                    // Preparation/rebound witnesses are sampled estimates, not
                    // a proof of skin or continuous swept-volume clearance.
                    if [0.4, 0.7, 1.0].iter().all(|&phase| {
                        witness(
                            profile,
                            equipment.as_deref(),
                            opening.map(|v| add(contact, mul(v, phase))),
                            pitches,
                            false,
                        )
                    }) {
                        pose = Some((contact, opening, pitches));
                        break 'search;
                    }
                }
            }
        }
        let Some((contact, opening, pitches)) = pose else {
            for (id, time) in free {
                result.unavailable(
                    time,
                    &id,
                    "공통 신체·장비 제약을 만족하는 스틱 접촉 자세를 찾지 못했습니다",
                );
            }
            return result;
        };
        result.contact = contact;
        result.opening = opening;
        result.pitches = pitches;
        for (id, time) in free {
            result.cues.push(CountInCue {
                source_event_id: id,
                time,
                contact_point: contact,
            });
            if let Some(phrase) = result.phrases.last_mut().filter(|p| time - WINDOW <= p.end) {
                if phrase.times.last() != Some(&time) {
                    phrase.times.push(time);
                }
                phrase.end = time + WINDOW;
            } else {
                result.phrases.push(Phrase {
                    start: time - WINDOW,
                    end: time + WINDOW,
                    times: vec![time],
                });
            }
        }
        result
    }
    fn unavailable(&mut self, time: f64, id: &str, reason: &str) {
        let mut diagnostic = Diagnostic::new(
            "COUNT_IN_GESTURE_UNAVAILABLE",
            format!("{time:.3}초 카운트인 {id}: {reason}. 원본 cue와 실제 타격은 유지합니다."),
        );
        diagnostic.time = Some(time);
        self.diagnostics.push(diagnostic);
    }
    pub fn is_active(&self, time: f64) -> bool {
        if !time.is_finite() {
            return false;
        }
        let at = self.phrases.partition_point(|p| p.end < time);
        self.phrases
            .get(at)
            .is_some_and(|p| time > p.start && time < p.end)
    }
    pub fn sample(&self, time: f64, tips: [Vec3; 2], pitches: [f64; 2]) -> CountInSample {
        let mut sample = CountInSample {
            tips,
            pitches,
            intended_stick_contact: false,
            contact_point: None,
        };
        if !time.is_finite() {
            return sample;
        }
        let at = self.phrases.partition_point(|p| p.end < time);
        let Some(phrase) = self
            .phrases
            .get(at)
            .filter(|p| time > p.start && time < p.end)
        else {
            return sample;
        };
        let blend = smooth((time - phrase.start) / (WINDOW * 0.5))
            .min(smooth((phrase.end - time) / (WINDOW * 0.5)));
        let next = phrase.times.partition_point(|t| *t < time);
        let previous = next.checked_sub(1).map(|i| phrase.times[i]);
        let following = phrase.times.get(next).copied();
        let opening = if following == Some(time) {
            0.0
        } else {
            match (previous, following) {
                (Some(a), Some(b)) => ((time - a) / (b - a) * std::f64::consts::PI).sin().powi(2),
                (Some(a), None) => smooth((time - a) / (WINDOW * 0.5)),
                (None, Some(b)) => smooth((b - time) / (WINDOW * 0.5)),
                _ => 1.0,
            }
        };
        for limb in 0..2 {
            let cue = add(self.contact, mul(self.opening[limb], opening));
            sample.tips[limb] = add(mul(tips[limb], 1.0 - blend), mul(cue, blend));
            sample.pitches[limb] = pitches[limb] * (1.0 - blend) + self.pitches[limb] * blend;
        }
        sample.intended_stick_contact = distance(sample.tips[0], sample.tips[1])
            <= CONTACT_TOLERANCE
            && sample
                .tips
                .iter()
                .all(|tip| distance(*tip, self.contact) <= CONTACT_TOLERANCE);
        sample.contact_point = sample.intended_stick_contact.then_some(self.contact);
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpretation::{Interpretation, SourceEvent};
    use crate::rig::HandControlFrame;
    fn profile() -> RigProfile {
        let mut p = RigProfile::primitive();
        for i in 0..2 {
            p.grip_offsets[i] = [0.0, 0.0, 0.02];
            p.tip_offsets[i] = [0.0, 0.0, 0.30];
        }
        let frame = HandControlFrame {
            forearm: [0.0, 0.0, 1.0],
            palm: [0.0, 1.0, 0.0],
            desired_palm: [0.0, 1.0, 0.0],
        };
        p.hand_frames = Some([frame.clone(), frame]);
        p
    }
    fn plan() -> PerformancePlan {
        let mut plan: PerformancePlan = serde_json::from_value(serde_json::json!({
            "schemaVersion":2,"engineVersion":"test","plannerVersion":"test","midiHash":"test","calibrationId":"test",
            "preRoll":0.5,"tail":0.25,"duration":3.0,"hits":[],"diagnostics":[],"kit":[],"tempoCount":1
        })).unwrap();
        for (index, time_us) in [1_000_000, 1_326_087, 1_652_174].into_iter().enumerate() {
            let id = format!("0:{index}");
            plan.source_events.push(SourceEvent {
                id: id.clone(),
                track: 0,
                event_index: index,
                tick: index as u64,
                time_us,
                channel: 9,
                message: SourceMessage::NoteOn {
                    key: 75,
                    velocity: 100,
                },
            });
            plan.interpretations.push(Interpretation {
                source_event_id: id,
                kind: InterpretationKind::CountIn,
                instrument: None,
                piece_id: None,
                articulation: None,
                hit_id: None,
                related_source_event_id: None,
            });
        }
        plan
    }
    #[test]
    fn explicit_cues_contact_without_mutating_source_or_creating_hits() {
        let plan = plan();
        let before = serde_json::to_string(&plan).unwrap();
        let p = profile();
        let track = CountInTrack::prepare(&plan, &p);
        assert_eq!(track.cues.len(), 3, "{:?}", track.diagnostics);
        let normal = [p.rest_tips[0], p.rest_tips[1]];
        for cue in &track.cues {
            let sample = track.sample(cue.time, normal, [0.0; 2]);
            assert_eq!(sample.tips, [cue.contact_point; 2]);
            assert!(sample.intended_stick_contact);
            assert!(witness(&p, None, sample.tips, sample.pitches, true));
            assert!(!cue.source_event_id.is_empty());
        }
        assert_eq!(before, serde_json::to_string(&plan).unwrap());
        assert_eq!(track.sample(0.0, normal, [0.1, -0.1]).tips, normal);
        assert!(!track.sample(1.163, normal, [0.0; 2]).intended_stick_contact);
        let times = [1.4, 0.78, 1.652174, 1.0, 1.872174, 1.326087];
        let first: Vec<_> = times
            .iter()
            .map(|&t| track.sample(t, normal, [0.0; 2]))
            .collect();
        for (i, &time) in times.iter().enumerate().rev() {
            assert_eq!(track.sample(time, normal, [0.0; 2]), first[i]);
        }
    }
    #[test]
    fn note_number_alone_is_not_a_cue_and_unsupported_profiles_are_diagnosed() {
        let mut plan = plan();
        let times = control_times_for_plan(&plan);
        let track = CountInTrack::prepare(&plan, &RigProfile::primitive());
        assert!(track.cues.is_empty());
        assert_eq!(track.diagnostics.len(), 3);
        assert_eq!(times, control_times_for_plan(&plan));
        for i in &mut plan.interpretations {
            i.kind = InterpretationKind::Unmapped;
        }
        assert!(CountInTrack::prepare(&plan, &profile()).cues.is_empty());
        assert!(control_times_for_plan(&plan).is_empty());
    }
    #[test]
    fn real_hand_hits_keep_priority_even_when_unassigned() {
        let mut plan = plan();
        plan.hits.push(crate::model::Hit {
            id: 0,
            time: 1.326087,
            time_us: 1_326_087,
            tick: 1,
            track: 0,
            event_index: 9,
            channel: 9,
            note: 38,
            velocity: 100,
            instrument: Instrument::Snare,
            piece_id: "snare".into(),
            articulation: "center".into(),
            source_event_id: "0:9".into(),
            limb: None,
            target: [0.0, 1.0, 0.5],
        });
        let track = CountInTrack::prepare(&plan, &profile());
        assert_eq!(track.cues.len(), 2);
        assert_eq!(track.diagnostics.len(), 1);
        let tips = [[-0.2, 1.0, 0.5], [0.2, 1.0, 0.5]];
        assert_eq!(track.sample(1.326087, tips, [0.1; 2]).tips, tips);
        assert_eq!(plan.hits[0].limb, None);
    }
    #[test]
    fn intentional_contact_does_not_exempt_shaft_overlap_or_distant_tips() {
        let p = profile();
        let hand = |x, z| HandSolution {
            middle: [x, 1.1, 0.1],
            end: [x, 1.0, z],
            tip: [0.0, 1.0, 0.7],
            rotation: [0.0, 0.0, 0.0, 1.0],
            reachable: true,
        };
        let a = hand(0.0, 0.2);
        let b = hand(0.0, 0.4);
        assert!(contact_pair_clearance(&p, &a, &b, true).sticks < 0.0);
        let mut distant = b;
        distant.tip[0] = 0.1;
        assert_eq!(
            contact_pair_clearance(&p, &a, &distant, true).sticks,
            clearance::pair_clearance(&p, &a, &distant).sticks
        );
        assert_eq!(
            contact_pair_clearance(&p, &a, &b, false).sticks,
            clearance::pair_clearance(&p, &a, &b).sticks
        );
    }
    #[test]
    fn translated_profile_and_all_interval_edges_remain_deterministic() {
        let p = profile();
        let plan = plan();
        let track = CountInTrack::prepare(&plan, &p);
        let mut translated = p.clone();
        let offset = [0.1, 0.15, 0.2];
        for v in translated
            .roots
            .iter_mut()
            .chain(translated.rest_tips.iter_mut())
        {
            *v = add(*v, offset);
        }
        translated.pelvis = add(translated.pelvis, offset);
        translated.chest = add(translated.chest, offset);
        translated.head = add(translated.head, offset);
        let moved = CountInTrack::prepare(&plan, &translated);
        assert_eq!(moved.cues.len(), track.cues.len());
        assert!(distance(moved.contact, add(track.contact, offset)) < 1e-9);
        let normal = [p.rest_tips[0], p.rest_tips[1]];
        assert!(!track.is_active(f64::NAN));
        assert!(!track.is_active(f64::INFINITY));
        for phrase in &track.phrases {
            assert!(track.is_active((phrase.start + phrase.end) * 0.5));
            assert!(track.is_active(phrase.start + 1e-6));
            assert!(track.is_active(phrase.end - 1e-6));
            for edge in [phrase.start, phrase.end] {
                assert!(!track.is_active(edge));
                let exact = track.sample(edge, normal, [0.0; 2]);
                assert_eq!(exact.tips, normal);
                for epsilon in [-1e-6, 1e-6] {
                    let adjacent = track.sample(edge + epsilon, normal, [0.0; 2]);
                    for (tip, previous) in adjacent.tips.into_iter().zip(exact.tips) {
                        assert!(distance(tip, previous) < 1e-8);
                    }
                }
            }
        }
    }
    #[test]
    fn motion_and_prepared_transfer_preserve_actual_tool_contacts() {
        let track = [
            0x83, 0x60, 0x99, 75, 100, 0x83, 0x60, 0x99, 75, 100, 0x83, 0x60, 0x99, 36, 100, 0x83,
            0x60, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        let plan = crate::build_plan(
            &bytes,
            &crate::model::Selection {
                tracks: vec![0],
                count_in_note75: true,
                ..Default::default()
            },
        )
        .unwrap();
        let source = serde_json::to_string(&plan.source_events).unwrap();
        let p = profile();
        let cues = CountInTrack::prepare(&plan, &p);
        assert_eq!(cues.cues.len(), 2);
        let motion = crate::motion::Motion::with_profile(plan, p.clone()).unwrap();
        let restored = crate::motion::Motion::from_prepared_json(
            &motion.prepared_json(),
            &motion.plan.midi_hash,
            &serde_json::to_string(&p).unwrap(),
        )
        .unwrap();
        assert_eq!(
            source,
            serde_json::to_string(&motion.plan.source_events).unwrap()
        );
        assert_eq!(motion.plan.hits.len(), 1);
        assert_eq!(motion.plan.hits[0].time, 1.5);
        for t in [1.0, 0.5, 1.72, 0.75, -0.5, 1.5, 1.0] {
            let pose = motion.evaluate_rig(t);
            assert_eq!(pose, restored.evaluate_rig(t));
            if let Some(cue) = cues.cues.iter().find(|c| c.time == t) {
                let inspection: serde_json::Value =
                    serde_json::from_str(&motion.inspect_pose(t)).unwrap();
                assert_eq!(inspection["intentionalStickContact"], true);
                assert_eq!(inspection["countIn"]["sourceEventId"], cue.source_event_id);
                assert!(inspection["stickClearanceMeters"].as_f64().unwrap() >= -1e-6);
                for limb in 0..2 {
                    let tip = std::array::from_fn(|i| pose[limb * 12 + 9 + i] as f64);
                    assert!(distance(tip, cue.contact_point) < 1e-6);
                }
            }
        }
    }
}
