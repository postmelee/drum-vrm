//! Explicit count-in presentation, independent of musical strikes and audio.
//! A held left stick and a right-hand pulse meet at their physical shaft midpoints.
use crate::{
    clearance, equipment_motion,
    interpretation::{InterpretationKind, SourceMessage},
    model::{Diagnostic, Instrument, PerformancePlan, Vec3},
    motion::{add, distance, sub},
    posture,
    rig::{HandSolution, Quat, RigProfile},
};
use std::collections::BTreeSet;

pub const WINDOW: f64 = 0.22;
const MIN_INTERVAL: f64 = 0.12;
const MAX_PHRASE_INTERVAL: f64 = 1.2;
const CONTACT_TOLERANCE: f64 = 0.002;
const WRIST_SNAP: f64 = std::f64::consts::PI / 6.; // 30 degrees about the actual wrist
const CONTACT_GAP: f64 = 0.000_001; // conservative pair-query numerical reserve

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
    exit_start: f64,
    times: Vec<f64>,
}
#[derive(Clone, Debug, Default)]
pub struct CountInTrack {
    pub cues: Vec<CountInCue>,
    pub diagnostics: Vec<Diagnostic>,
    phrases: Vec<Phrase>,
    contact: Vec3,
    tips: [Vec3; 2],
    rotations: [Quat; 2],
    swivels: [f64; 2],
    wrist_pivots: [Vec3; 2],
    shaft_directions: [Vec3; 2],
    pulse_axis: Vec3,
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
/// A struck stick rebounds quickly and slows at the top. The matching
/// downswing starts gently, then accelerates into the authored contact instant;
/// zero velocity at contact would read as placing the two sticks together.
fn rebound(t: f64) -> f64 {
    1. - (1. - t.clamp(0., 1.)).powi(3)
}
fn beat_opening(phase: f64) -> f64 {
    let phase = phase.clamp(0., 1.);
    if phase < 0.4 {
        rebound(phase / 0.4)
    } else if phase < 0.6 {
        1.
    } else {
        1. - ((phase - 0.6) / 0.4).powi(3)
    }
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

/// The complete physical profiles are always tested. Tangential mid-shaft
/// contact needs no trimmed region or penetration exemption.
pub fn contact_pair_clearance(
    profile: &RigProfile,
    left: &HandSolution,
    right: &HandSolution,
    intended: bool,
) -> clearance::PairClearance {
    let mut pair = clearance::pair_clearance(profile, left, right);
    if intended {
        pair.sticks = contact_tool_clearance(profile, left, right, true);
    }
    pair
}
pub fn contact_tool_clearance(
    profile: &RigProfile,
    left: &HandSolution,
    right: &HandSolution,
    _intended: bool,
) -> f64 {
    clearance::tool_pair_clearance(profile, left, right)
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: Vec3) -> Vec3 {
    mul(a, 1. / distance(a, [0.; 3]).max(1e-12))
}
fn axis_rotation(axis: Vec3, angle: f64) -> Quat {
    let (sin, cos) = (angle * 0.5).sin_cos();
    [axis[0] * sin, axis[1] * sin, axis[2] * sin, cos]
}
fn wrist_pulse(
    tips: [Vec3; 2],
    rotations: [Quat; 2],
    pivot: Vec3,
    axis: Vec3,
    opening: f64,
) -> ([Vec3; 2], [Quat; 2]) {
    let rotation = axis_rotation(axis, opening * WRIST_SNAP);
    (
        [
            tips[0],
            add(pivot, crate::rig::rotate(rotation, sub(tips[1], pivot))),
        ],
        [rotations[0], multiply(rotation, rotations[1])],
    )
}
fn multiply(a: Quat, b: Quat) -> Quat {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}
fn align(from: Vec3, to: Vec3) -> Quat {
    let a = unit(from);
    let b = unit(to);
    let axis = cross(a, b);
    let w = 1. + a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    let q = [axis[0], axis[1], axis[2], w];
    let n = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    q.map(|v| v / n)
}
fn shaft_rotation(profile: &RigProfile, limb: usize, direction: Vec3) -> Quat {
    let neutral = profile.contact_rotations[limb];
    let base = multiply(
        align(
            crate::rig::rotate(neutral, profile.tip_offsets[limb]),
            direction,
        ),
        neutral,
    );
    let Some(frames) = &profile.hand_frames else {
        return base;
    };
    let frame = &frames[limb];
    let actual = crate::rig::rotate(
        multiply(base, [-neutral[0], -neutral[1], -neutral[2], neutral[3]]),
        frame.palm,
    );
    let yaw = direction[0].atan2(direction[2]);
    let (sin, cos) = (yaw * 0.5).sin_cos();
    let desired = if frame.palm_reference.is_some() {
        crate::rig::rotate([0., sin, 0., cos], frame.desired_palm)
    } else {
        frame.desired_palm
    };
    let dot = |a: Vec3, b: Vec3| a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    let project = |v| unit(sub(v, mul(direction, dot(v, direction))));
    let from = project(actual);
    let to = project(desired);
    let inward = dot(direction, cross(from, to)).atan2(dot(from, to));
    // Keep a small palm-down reserve during the crossed-stick cue. An exact
    // policy-boundary roll can leave the interval on a shortest-arc transition.
    let roll = frame.palm_roll_range.map_or(0., |[min, max]| {
        let center = (min + max) * 0.5;
        (center.signum() * center.abs().min(std::f64::consts::PI / 12.)).clamp(min, max)
    });
    let angle = inward + roll;
    let (sin, cos) = (angle * 0.5).sin_cos();
    multiply(
        [
            direction[0] * sin,
            direction[1] * sin,
            direction[2] * sin,
            cos,
        ],
        base,
    )
}

pub fn shaft_midpoint(profile: &RigProfile, limb: usize, hand: &HandSolution) -> Vec3 {
    let (butt, tip) = equipment_motion::stick_segment(profile, limb, hand);
    mul(add(butt, tip), 0.5)
}
pub fn shaft_contact_points(profile: &RigProfile, hands: &[HandSolution; 2]) -> [Vec3; 2] {
    let mid = std::array::from_fn::<_, 2, _>(|i| shaft_midpoint(profile, i, &hands[i]));
    let axes = std::array::from_fn::<_, 2, _>(|i| {
        let (butt, tip) = equipment_motion::stick_segment(profile, i, &hands[i]);
        unit(sub(tip, butt))
    });
    let mut normal = unit(cross(axes[0], axes[1]));
    if normal
        .iter()
        .zip(sub(mid[1], mid[0]))
        .map(|(a, b)| a * b)
        .sum::<f64>()
        < 0.
    {
        normal = mul(normal, -1.);
    }
    let radius = crate::stick_geometry::profile().terminal.last().unwrap()[1];
    [
        add(mid[0], mul(normal, radius)),
        sub(mid[1], mul(normal, radius)),
    ]
}

/// Prepared cue orientation shared by candidate measurement and pose replay.
/// Entry/exit blends once from the underlying musical orientation; full pulses
/// retain one elbow plane while the right stick rotates about the wrist.
#[derive(Clone, Copy, Debug)]
pub struct CueOrientation {
    pub rotations: [Quat; 2],
    pub swivels: [f64; 2],
    pub weight: f64,
}
impl CueOrientation {
    pub fn apply(
        self,
        profile: &RigProfile,
        limb: usize,
        tip: Vec3,
        swivel: f64,
        hand: HandSolution,
    ) -> HandSolution {
        let rotation = crate::hand_coordination::interpolate_orientation(
            hand.rotation,
            self.rotations[limb],
            self.weight,
        );
        profile.solve_hand_with_rotation(limb, tip, rotation, swivel)
    }
}
fn cue_hands(
    profile: &RigProfile,
    tips: [Vec3; 2],
    rotations: [Quat; 2],
    equipment: Option<&equipment_motion::EquipmentContext>,
) -> Option<[HandSolution; 2]> {
    let mut hands = Vec::new();
    for limb in 0..2 {
        let mut best: Option<(f64, HandSolution)> = None;
        for i in -16..=16 {
            let h = profile.solve_hand_with_rotation(
                limb,
                tips[limb],
                rotations[limb],
                i as f64 * 0.05,
            );
            if !h.reachable
                || !posture::anatomical(profile, limb, tips[limb], &h)
                || !profile.hand_orientation_policy_valid(limb, h.rotation)
                || clearance::upper_body_core(profile, limb, &h).minimum() < 0.
                || equipment.is_some_and(|ctx| {
                    equipment_motion::tool_clearance(profile, limb, &h, ctx, None) < posture::MARGIN
                })
            {
                continue;
            }
            let score = posture::comfort_cost(profile, limb, tips[limb], &h);
            if best.as_ref().is_none_or(|(s, _)| score < *s) {
                best = Some((score, h));
            }
        }
        hands.push(best?.1);
    }
    let pair = [hands[0], hands[1]];
    (clearance::tool_pair_clearance(profile, &pair[0], &pair[1]) >= 0.).then_some(pair)
}

/// One elbow witness must support the entire wrist pulse, not independently
/// selected elbows at each phase. The temporal graph retains this witness.
fn pulse_swivels(
    profile: &RigProfile,
    tips: [Vec3; 2],
    rotations: [Quat; 2],
    pivot: Vec3,
    axis: Vec3,
    equipment: Option<&equipment_motion::EquipmentContext>,
) -> Option<[f64; 2]> {
    let phases: Vec<_> = (0..=40)
        .map(|step| wrist_pulse(tips, rotations, pivot, axis, step as f64 / 40.))
        .collect();
    let mut swivels = [0.; 2];
    for (limb, swivel) in swivels.iter_mut().enumerate() {
        let mut best = None;
        for i in -16..=16 {
            let angle = i as f64 * 0.05;
            let mut cost = 0.;
            let valid = phases.iter().all(|(tips, rotations)| {
                let hand =
                    profile.solve_hand_with_rotation(limb, tips[limb], rotations[limb], angle);
                cost += posture::comfort_cost(profile, limb, tips[limb], &hand);
                hand.reachable
                    && posture::anatomical(profile, limb, tips[limb], &hand)
                    && profile.hand_orientation_policy_valid(limb, hand.rotation)
                    && clearance::upper_body_core(profile, limb, &hand).minimum() >= 0.
                    && equipment.is_none_or(|ctx| {
                        equipment_motion::tool_clearance(profile, limb, &hand, ctx, None)
                            >= posture::MARGIN
                    })
            });
            if valid && best.is_none_or(|(old, _)| cost < old) {
                best = Some((cost, angle));
            }
        }
        *swivel = best?.1;
    }
    phases
        .iter()
        .all(|(tips, rotations)| {
            let hands = std::array::from_fn::<_, 2, _>(|i| {
                profile.solve_hand_with_rotation(i, tips[i], rotations[i], swivels[i])
            });
            clearance::tool_pair_clearance(profile, &hands[0], &hands[1]) >= 0.
        })
        .then_some(swivels)
}

fn next_preparation_start(plan: &PerformancePlan, ready: [Vec3; 2], after: f64) -> f64 {
    let mut next = [None; 2];
    for hit in plan.hits.iter().filter(|hit| {
        hit.time > after && !matches!(hit.instrument, Instrument::Kick | Instrument::PedalHat)
    }) {
        if let Some(limb) = hit.limb.filter(|limb| *limb < 2) {
            next[limb].get_or_insert(hit);
        } else {
            // An unassigned real hand hit still has source priority. Reserve
            // any hand whose next action is not already known.
            for slot in &mut next {
                slot.get_or_insert(hit);
            }
        }
        if next.iter().all(Option::is_some) {
            break;
        }
    }
    next.into_iter()
        .enumerate()
        .filter_map(|(limb, hit)| {
            hit.map(|hit| {
                hit.time
                    - crate::motion::transfer::hand_transition_duration(
                        distance(ready[limb], hit.target),
                        0.22,
                    )
            })
        })
        .fold(plan.duration + crate::model::TAIL, f64::min)
}
impl CountInTrack {
    #[cfg(test)]
    pub fn prepare(plan: &PerformancePlan, profile: &RigProfile) -> Self {
        Self::prepare_with_ready(plan, profile, [profile.rest_tips[0], profile.rest_tips[1]])
    }
    pub fn prepare_with_ready(
        plan: &PerformancePlan,
        profile: &RigProfile,
        ready: [Vec3; 2],
    ) -> Self {
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
            } else if next_preparation_start(plan, ready, time) <= time {
                result.unavailable(time, &id, "다음 실제 타격의 준비 구간이 이미 시작됩니다");
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
        let mut pose = None;
        'search: for down in [0.40, 0.30, 0.50, 0.20] {
            for front in [0.70, 0.60, 0.50, 0.40] {
                let contact = add(
                    add(center, mul(forward, arm * front)),
                    [0., -arm * down, 0.],
                );
                for yaw in [0.70_f64, 0.55, 0.85] {
                    for incline in [0.0_f64, 0.15, -0.15] {
                        let dirs = std::array::from_fn::<_, 2, _>(|i| {
                            unit(add(
                                add(
                                    mul(forward, yaw.cos()),
                                    mul(right, if i == 0 { yaw.sin() } else { -yaw.sin() }),
                                ),
                                [0., incline, 0.],
                            ))
                        });
                        let mut normal = unit(cross(dirs[0], dirs[1]));
                        if normal[1] < 0. {
                            normal = mul(normal, -1.);
                        }
                        let radius = crate::stick_geometry::profile().terminal.last().unwrap()[1];
                        let tips = std::array::from_fn(|i| {
                            add(
                                add(
                                    contact,
                                    mul(
                                        normal,
                                        if i == 0 {
                                            -radius - CONTACT_GAP * 0.5
                                        } else {
                                            radius + CONTACT_GAP * 0.5
                                        },
                                    ),
                                ),
                                mul(
                                    dirs[i],
                                    (distance(profile.tip_offsets[i], [0.; 3])
                                        + crate::stick_geometry::profile().butt_length)
                                        * 0.5,
                                ),
                            )
                        });
                        let rotations =
                            std::array::from_fn(|i| shaft_rotation(profile, i, dirs[i]));
                        if profile.hand_frames.is_none()
                            || cue_hands(profile, tips, rotations, equipment.as_deref()).is_none()
                        {
                            continue;
                        }
                        let pivots =
                            std::array::from_fn(|i| sub(tips[i], profile.offset(i, rotations[i])));
                        let axis = unit(cross(dirs[1], normal));
                        if let Some(swivels) = pulse_swivels(
                            profile,
                            tips,
                            rotations,
                            pivots[1],
                            axis,
                            equipment.as_deref(),
                        ) {
                            pose = Some((
                                contact,
                                tips,
                                pivots,
                                axis,
                                dirs,
                                rotations,
                                swivels,
                                [-incline.atan(); 2],
                            ));
                            break 'search;
                        }
                    }
                }
            }
        }
        let Some((contact, tips, pivots, axis, directions, rotations, swivels, pitches)) = pose
        else {
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
        result.tips = tips;
        result.rotations = rotations;
        result.swivels = swivels;
        result.wrist_pivots = pivots;
        result.shaft_directions = directions;
        result.pulse_axis = axis;
        result.pitches = pitches;
        for (id, time) in free {
            result.cues.push(CountInCue {
                source_event_id: id,
                time,
                contact_point: contact,
            });
            if let Some(phrase) = result.phrases.last_mut().filter(|p| {
                let previous = *p.times.last().unwrap();
                time - previous <= MAX_PHRASE_INTERVAL
                    && !hand_times.iter().any(|t| *t > previous && *t < time)
            }) {
                if phrase.times.last() != Some(&time) {
                    phrase.times.push(time);
                }
                phrase.end = time + WINDOW;
            } else {
                result.phrases.push(Phrase {
                    start: time - WINDOW,
                    end: time + WINDOW,
                    exit_start: time + WINDOW * 0.5,
                    times: vec![time],
                });
            }
        }
        let opened = result.pulse_pose(1.).0;
        let travel = (0..2)
            .map(|i| distance(opened[i], ready[i]))
            .fold(0_f64, f64::max);
        for index in 0..result.phrases.len() {
            let last = *result.phrases[index].times.last().unwrap();
            let deadline = next_preparation_start(plan, ready, last).min(
                result
                    .phrases
                    .get(index + 1)
                    .map_or(f64::INFINITY, |p| p.start),
            );
            let available = (deadline - last).max(0.);
            let rebound = (WINDOW * 0.5).min(available * 0.25);
            let settling = crate::motion::transfer::settling_duration(travel, available - rebound)
                .min(available - rebound);
            result.phrases[index].exit_start = last + rebound;
            result.phrases[index].end = last + rebound + settling;
        }
        result
    }
    pub fn transition_times(&self) -> Vec<f64> {
        self.phrases
            .iter()
            .flat_map(|p| [p.start, p.start + WINDOW * 0.5, p.exit_start, p.end])
            .collect()
    }
    /// Protect neutral-facing roots over the entire fixed-wrist interval.
    pub fn held_intervals(&self) -> Vec<[f64; 2]> {
        self.phrases
            .iter()
            .map(|p| [p.start + WINDOW * 0.5, p.exit_start])
            .collect()
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
    /// The remaining contribution of the musical/ready path during cue entry
    /// and exit. A full cue has zero weight, so planning that underlying path
    /// cannot move its exact shaft contact.
    pub fn base_weight(&self, time: f64) -> f64 {
        if !time.is_finite() {
            return 1.;
        }
        let at = self.phrases.partition_point(|p| p.end < time);
        self.phrases
            .get(at)
            .filter(|p| time > p.start && time < p.end)
            .map_or(1., |p| {
                1. - smooth((time - p.start) / (WINDOW * 0.5))
                    .min(1. - smooth((time - p.exit_start) / (p.end - p.exit_start)))
            })
    }
    fn pulse_opening(&self, time: f64) -> Option<f64> {
        let at = self.phrases.partition_point(|p| p.end < time);
        let phrase = self
            .phrases
            .get(at)
            .filter(|p| time > p.start && time < p.end)?;
        let next = phrase.times.partition_point(|t| *t < time);
        let previous = next.checked_sub(1).map(|i| phrase.times[i]);
        let following = phrase.times.get(next).copied();
        Some(if following == Some(time) {
            0.
        } else {
            match (previous, following) {
                (Some(a), Some(b)) => beat_opening((time - a) / (b - a)),
                (Some(a), None) => rebound((time - a) / (phrase.exit_start - a)),
                (None, Some(b)) => rebound((b - time) / (WINDOW * 0.5)),
                _ => 1.,
            }
        })
    }
    fn pulse_pose(&self, opening: f64) -> ([Vec3; 2], [Quat; 2], [f64; 2]) {
        let (tips, rotations) = wrist_pulse(
            self.tips,
            self.rotations,
            self.wrist_pivots[1],
            self.pulse_axis,
            opening,
        );
        let direction = crate::rig::rotate(
            axis_rotation(self.pulse_axis, opening * WRIST_SNAP),
            self.shaft_directions[1],
        );
        (
            tips,
            rotations,
            [self.pitches[0], -direction[1].clamp(-1., 1.).asin()],
        )
    }
    pub fn orientation(&self, time: f64) -> Option<CueOrientation> {
        let weight = 1. - self.base_weight(time);
        (weight > 0.).then(|| CueOrientation {
            rotations: self.pulse_pose(self.pulse_opening(time).unwrap_or(0.)).1,
            swivels: self.swivels,
            weight,
        })
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
        let Some(_) = self
            .phrases
            .get(at)
            .filter(|p| time > p.start && time < p.end)
        else {
            return sample;
        };
        let blend = 1. - self.base_weight(time);
        let opening = self.pulse_opening(time).unwrap_or(0.);
        let (cue_tips, _, cue_pitches) = self.pulse_pose(opening);
        for limb in 0..2 {
            sample.tips[limb] = add(mul(tips[limb], 1.0 - blend), mul(cue_tips[limb], blend));
            sample.pitches[limb] = pitches[limb] * (1.0 - blend) + cue_pitches[limb] * blend;
        }
        let midpoint_offset = crate::rig::rotate(
            axis_rotation(self.pulse_axis, opening * WRIST_SNAP),
            sub(self.contact, self.wrist_pivots[1]),
        );
        let contact_travel = distance(add(self.wrist_pivots[1], midpoint_offset), self.contact);
        sample.intended_stick_contact = blend >= 1. - 1e-12 && contact_travel <= CONTACT_TOLERANCE;
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
            palm_roll_range: None,
            palm_reference: None,
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
    fn shaft_centers_touch_tangentially_without_any_collision_exemption() {
        let p = profile();
        let track = CountInTrack::prepare(&plan(), &p);
        assert_eq!(track.cues.len(), 3, "{:?}", track.diagnostics);
        let ready = track.pulse_pose(1.).0;
        for step in 0..=1200 {
            let time = 0.72 + step as f64 * 0.001;
            let sample = track.sample(time, ready, track.pitches);
            let rotation = track
                .orientation(time)
                .map_or(track.pulse_pose(1.).1, |cue| {
                    std::array::from_fn(|i| {
                        crate::hand_coordination::interpolate_orientation(
                            track.pulse_pose(1.).1[i],
                            cue.rotations[i],
                            cue.weight,
                        )
                    })
                });
            let hands = cue_hands(&p, sample.tips, rotation, None)
                .expect("shaft pulse has a valid physical/anatomical witness");
            assert!(
                clearance::tool_pair_clearance(&p, &hands[0], &hands[1]) >= 0.,
                "time={time}"
            );
            if track.cues.iter().any(|c| (c.time - time).abs() < 1e-10) {
                for actual in shaft_contact_points(&p, &hands) {
                    assert!(distance(actual, track.contact) < 1e-6);
                }
                assert!(
                    distance(hands[0].tip, hands[1].tip) > 0.20,
                    "tips must visibly pass beyond the shaft crossing"
                );
            }
        }
    }
    #[test]
    fn cue_reserve_respects_custom_palm_intervals_away_from_zero() {
        for range in [[0.5, 1.0], [-1.0, -0.5], [-0.1, 0.1]] {
            let mut p = profile();
            for frame in p.hand_frames.as_mut().unwrap() {
                frame.palm_roll_range = Some(range);
            }
            for limb in 0..2 {
                let direction = unit([if limb == 0 { 0.6 } else { -0.6 }, 0.1, 0.8]);
                let rotation = shaft_rotation(&p, limb, direction);
                assert!(p.hand_orientation_policy_valid(limb, rotation));
            }
        }
    }
    #[test]
    fn consecutive_cues_hold_wrist_pivots_and_snap_right_stick_without_reset() {
        let mut plan = plan();
        for (index, event) in plan.source_events.iter_mut().enumerate() {
            event.time_us = 1_000_000 + index as u64 * 500_000;
        }
        let p = profile();
        let track = CountInTrack::prepare(&plan, &p);
        assert_eq!(track.phrases.len(), 1);
        let rest = [p.rest_tips[0], p.rest_tips[1]];
        let contact = track.sample(1., rest, [0.; 2]);
        let lifted = track.sample(1.25, rest, [0.; 2]);
        assert_eq!(track.base_weight(1.25), 0.);
        assert_eq!(contact.tips[0], lifted.tips[0]);
        assert!(
            distance(contact.tips[1], lifted.tips[1]) > 0.14,
            "the count-in needs a visible backswing, not a small tip tremor"
        );
        for time in [1., 1.125, 1.25, 1.375, 1.5] {
            let sample = track.sample(time, rest, [0.; 2]);
            let rotations = track.orientation(time).unwrap().rotations;
            for (limb, rotation) in rotations.iter().enumerate() {
                let wrist = sub(sample.tips[limb], p.offset(limb, *rotation));
                assert!(distance(wrist, track.wrist_pivots[limb]) < 1e-12);
            }
        }
        assert!(!lifted.intended_stick_contact);
        assert!(track.sample(1.5, rest, [0.; 2]).intended_stick_contact);
    }
    #[test]
    fn forceful_pulse_accelerates_into_exact_midshaft_contact_and_rebounds() {
        let mut plan = plan();
        for (index, event) in plan.source_events.iter_mut().enumerate() {
            event.time_us = 1_000_000 + index as u64 * 500_000;
        }
        let p = profile();
        let track = CountInTrack::prepare(&plan, &p);
        let rest = [p.rest_tips[0], p.rest_tips[1]];
        let tip = |time| track.sample(time, rest, [0.; 2]).tips[1];
        let early_down = distance(tip(1.31), tip(1.32));
        let impact_speed = distance(tip(1.49), tip(1.5));
        assert!(impact_speed > early_down * 10.);
        assert!(
            impact_speed > 0.015,
            "a decisive contact keeps approach speed"
        );
        assert!(distance(tip(1.5), tip(1.51)) > 0.015);
        assert!(distance(tip(1.2), tip(1.3)) < 1e-12);
        for cue in &track.cues {
            let sample = track.sample(cue.time, rest, [0.; 2]);
            let orientation = track.orientation(cue.time).unwrap();
            let hands = std::array::from_fn(|limb| {
                p.solve_hand_with_rotation(
                    limb,
                    sample.tips[limb],
                    orientation.rotations[limb],
                    orientation.swivels[limb],
                )
            });
            assert!(clearance::tool_pair_clearance(&p, &hands[0], &hands[1]) >= 0.);
            for marker in shaft_contact_points(&p, &hands) {
                assert!(distance(marker, cue.contact_point) < 1e-6);
            }
        }
    }
    #[test]
    fn held_cue_cannot_lock_a_witness_inside_the_ordinary_torso() {
        let mut p = profile();
        // Short arms with a normal shoulder span admit wrist/shaft contact
        // while a stick butt crosses the torso. A feasible hand alone must
        // not lock that invalid paired-tool witness for every cue.
        p.roots[0][0] = -0.175;
        p.roots[1][0] = 0.175;
        p.lengths[0] = [0.15, 0.135];
        p.lengths[1] = [0.15, 0.135];
        let plan = plan();
        let before = serde_json::to_string(&plan).unwrap();
        let track = CountInTrack::prepare(&plan, &p);
        if track.cues.is_empty() {
            assert_eq!(track.diagnostics.len(), 3);
        } else {
            assert_eq!(track.cues.len(), 3);
            for step in 0..=100 {
                let (tips, rotations, _) = track.pulse_pose(step as f64 / 100.);
                for limb in 0..2 {
                    let hand = p.solve_hand_with_rotation(
                        limb,
                        tips[limb],
                        rotations[limb],
                        track.swivels[limb],
                    );
                    assert!(clearance::upper_body_core(&p, limb, &hand).minimum() >= 0.);
                }
            }
        }
        assert_eq!(before, serde_json::to_string(&plan).unwrap());
    }
    #[test]
    fn settling_reserves_each_hands_next_preparation_and_ignores_later_targets() {
        let p = profile();
        let mut plan = plan();
        for (i, event) in plan.source_events.iter_mut().enumerate() {
            event.time_us = i as u64 * 500_000;
        }
        let hit = |time: f64, limb, target| crate::model::Hit {
            id: 0,
            time,
            time_us: (time * 1_000_000.) as u64,
            tick: 0,
            track: 0,
            event_index: 9,
            channel: 9,
            note: 38,
            velocity: 100,
            instrument: Instrument::Snare,
            piece_id: "snare".into(),
            articulation: "center".into(),
            source_event_id: "0:9".into(),
            limb: Some(limb),
            target,
        };
        plan.hits = vec![hit(2., 1, p.rest_tips[1])];
        let sources = serde_json::to_string(&plan.source_events).unwrap();
        let track = CountInTrack::prepare(&plan, &p);
        let phrase = &track.phrases[0];
        assert!((phrase.exit_start - 1.11).abs() < 1e-12);
        assert!((phrase.end - 1.445).abs() < 1e-12);
        assert_eq!(track.base_weight(1.11), 0.);
        assert!(track.base_weight(1.22) < 0.5);
        assert_eq!(track.base_weight(phrase.end), 1.);
        assert!(phrase.end < 2. - 0.22);
        // Both hands' first preparations matter, including simultaneous hits.
        plan.hits
            .push(hit(2., 0, add(p.rest_tips[0], [0., 0., 0.75])));
        assert!(CountInTrack::prepare(&plan, &p).phrases[0].end < phrase.end);
        // A later same-hand target transfers from its preceding contact, so
        // its hypothetical home preparation must not cancel an earlier cue.
        plan.hits = vec![
            hit(1.35, 1, p.rest_tips[1]),
            hit(1.6, 1, add(p.rest_tips[1], [0., 0., 2.])),
        ];
        let tight = CountInTrack::prepare(&plan, &p);
        assert_eq!(tight.cues.len(), 3);
        assert!(tight.phrases[0].end <= 1.35 - 0.22);
        assert_eq!(sources, serde_json::to_string(&plan.source_events).unwrap());
        // With no next hand action, the authored ending plus tail is the deadline.
        plan.hits.clear();
        plan.duration = 1.;
        let final_phrase = &CountInTrack::prepare(&plan, &p).phrases[0];
        assert!(final_phrase.end <= plan.duration + crate::model::TAIL);
        assert!(final_phrase.end > *final_phrase.times.last().unwrap());
    }
    #[test]
    fn cue_base_weight_matches_entry_exit_without_moving_cue_contacts() {
        let p = profile();
        let track = CountInTrack::prepare(&plan(), &p);
        assert!(!track.cues.is_empty());
        let tips = [p.rest_tips[0], p.rest_tips[1]];
        let offset = [[0.01, 0.025, -0.01], [-0.02, 0.03, 0.01]];
        let pitch_delta = [0.1, -0.2];
        for time in [0.7, 0.8, 0.85, 0.95, 1.0, 1.1, 1.7, 1.82, 1.9] {
            let a = track.sample(time, tips, [0.; 2]);
            let b = track.sample(
                time,
                std::array::from_fn(|i| add(tips[i], offset[i])),
                pitch_delta,
            );
            let weight = track.base_weight(time);
            for limb in 0..2 {
                assert!(
                    distance(sub(b.tips[limb], a.tips[limb]), mul(offset[limb], weight)) < 1e-12
                );
                assert!(
                    (b.pitches[limb] - a.pitches[limb] - pitch_delta[limb] * weight).abs() < 1e-12
                );
            }
        }
        for cue in &track.cues {
            assert_eq!(track.base_weight(cue.time), 0.);
            let sample = track.sample(cue.time, tips, pitch_delta);
            assert_eq!(sample.tips, track.tips);
            assert!(sample.intended_stick_contact);
        }
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
            assert_eq!(sample.tips, track.tips);
            assert!(sample.intended_stick_contact);
            assert!(cue_hands(&p, sample.tips, track.rotations, None).is_some());
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
    fn intentional_contact_does_not_exempt_shaft_overlap() {
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
            clearance::tool_pair_clearance(&p, &a, &distant)
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
    fn bounded_natural_grip_cues_restore_through_entry_and_exit() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-natural-grip30.json")).unwrap();
        let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        let track = [
            0, 0x99, 75, 100, 0x83, 0x60, 0x99, 75, 100, 0x83, 0x60, 0x99, 75, 100, 0x83, 0x60,
            0x99, 36, 100, 0x83, 0x60, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        let mut plan = crate::build_plan(
            &bytes,
            &crate::model::Selection {
                tracks: vec![0],
                count_in_note75: true,
                ..Default::default()
            },
        )
        .unwrap();
        plan.kit.retain(|piece| piece.id == "kick");
        let source = serde_json::to_string(&plan.source_events).unwrap();
        let motion = crate::motion::Motion::with_profile(plan, profile.clone()).unwrap();
        let restored = crate::motion::Motion::from_prepared_json(
            &motion.prepared_json(),
            &motion.plan.midi_hash,
            &serde_json::to_string(&profile).unwrap(),
        )
        .unwrap();
        assert_eq!(
            source,
            serde_json::to_string(&motion.plan.source_events).unwrap()
        );
        assert_eq!(motion.plan.hits.len(), 1);
        assert_eq!(motion.plan.hits[0].time, 1.5);
        assert_eq!(
            motion
                .plan
                .contact_events
                .iter()
                .filter(|event| event.tool_part == crate::contact_events::ToolPart::StickShaft)
                .count(),
            6
        );
        assert_eq!(motion.prepared_json(), restored.prepared_json());
        let contact_pose = motion.evaluate_rig(0.);
        for i in (0..=360).rev() {
            let time = -0.5 + i as f64 / 120.;
            assert_eq!(motion.evaluate_rig(time), restored.evaluate_rig(time));
            if (0. ..=1.11).contains(&time) {
                let pose = motion.evaluate_rig(time);
                assert_eq!(&pose[73..77], &[0., 0., 0., 1.]);
                for limb in 0..2 {
                    let vector = |offset: usize| {
                        std::array::from_fn(|j| pose[limb * 12 + offset + j] as f64)
                    };
                    let hand = HandSolution {
                        middle: vector(3),
                        end: vector(6),
                        tip: vector(9),
                        rotation: std::array::from_fn(|j| pose[57 + limb * 4 + j] as f64),
                        reachable: true,
                    };
                    assert!(posture::anatomical(&profile, limb, hand.tip, &hand));
                    assert_eq!(
                        &pose[limb * 12 + 3..limb * 12 + 9],
                        &contact_pose[limb * 12 + 3..limb * 12 + 9]
                    );
                }
            }
            if time > -WINDOW && time < 1. + WINDOW {
                let inspection: serde_json::Value =
                    serde_json::from_str(&motion.inspect_pose(time)).unwrap();
                assert!(inspection["toolPairClearanceMeters"].as_f64().unwrap() >= -1e-6);
            }
        }
    }
    #[test]
    fn tight_real_hit_then_cue_keeps_a_finite_graph_and_source_contact() {
        // 221 ticks at 120 BPM / 480 PPQ is 0.230208 s: just outside
        // the existing +/-0.22 s real-hit priority window.
        let track = [
            0, 0x99, 38, 100, 0x81, 0x5d, 0x99, 75, 100, 0x83, 0x60, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-natural-grip30.json")).unwrap();
        let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
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
        let motion = crate::motion::Motion::with_profile(plan, profile.clone()).unwrap();
        let restored = crate::motion::Motion::from_prepared_json(
            &motion.prepared_json(),
            &motion.plan.midi_hash,
            &serde_json::to_string(&profile).unwrap(),
        )
        .unwrap();
        assert_eq!(
            source,
            serde_json::to_string(&motion.plan.source_events).unwrap()
        );
        assert_eq!(
            motion
                .plan
                .contact_events
                .iter()
                .filter(|e| e.tool_part == crate::contact_events::ToolPart::StickShaft)
                .count(),
            2
        );
        for hit in &motion.plan.hits {
            let limb = hit.limb.unwrap();
            let pose = motion.evaluate_rig(hit.time);
            assert!(
                distance(
                    std::array::from_fn(|i| pose[limb * 12 + 9 + i] as f64),
                    hit.target
                ) < 1e-6
            );
        }
        for i in 0..=120 {
            let time = -0.5 + i as f64 / 120.;
            assert_eq!(motion.evaluate_rig(time), restored.evaluate_rig(time));
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
                let hands = std::array::from_fn(|limb| HandSolution {
                    middle: std::array::from_fn(|i| pose[limb * 12 + 3 + i] as f64),
                    end: std::array::from_fn(|i| pose[limb * 12 + 6 + i] as f64),
                    tip: std::array::from_fn(|i| pose[limb * 12 + 9 + i] as f64),
                    rotation: std::array::from_fn(|i| {
                        pose[crate::model::POSE_FLOATS + limb * 4 + i] as f64
                    }),
                    reachable: true,
                });
                for actual in shaft_contact_points(&p, &hands) {
                    assert!(distance(actual, cue.contact_point) < 2e-6);
                }
            }
        }
    }
}
