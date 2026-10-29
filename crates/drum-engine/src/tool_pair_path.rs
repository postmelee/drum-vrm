//! Joint tool/equipment routing in the existing numeric pitch/lift track.
//! Local Lipschitz envelopes preserve source contacts and rate bounds; every
//! changed knot and interior check is tested against both tools and equipment.
use crate::{
    count_in,
    equipment_motion::{self, EquipmentContext, PathSample},
    model::Vec3,
    posture,
    rig::{HandSolution, RigProfile},
};
use std::{collections::HashMap, sync::Arc};

/// The route and its final inspection share the same bounded observation grid.
/// Clear control knots alone do not certify the interpolated whole shafts.
pub(super) fn check_times(path: &[PathSample]) -> Vec<f64> {
    let mut times: Vec<_> = path.iter().map(|sample| sample.time).collect();
    times.extend(path.windows(2).flat_map(|window| {
        [0.25, 0.5, 0.75].map(|phase| window[0].time + (window[1].time - window[0].time) * phase)
    }));
    times.sort_by(f64::total_cmp);
    times.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    times
}

pub(super) struct Input {
    pub time: f64,
    pub profile: RigProfile,
    /// Affine base of the additive path; `weight` is zero at full cue contact.
    pub tips: [Vec3; 2],
    pub pitches: [f64; 2],
    pub equipment: Arc<EquipmentContext>,
    pub intended: [Option<String>; 2],
    pub weight: f64,
    pub intentional_contact: bool,
    pub close_tool_approach: bool,
}
#[derive(Clone, Copy)]
struct Metrics {
    pair: f64,
    equipment: [f64; 2],
    anatomy: [bool; 2],
    violation: [f64; 2],
    hands: [Option<HandSolution>; 2],
}
struct Repair {
    score: f64,
    path: Vec<PathSample>,
    metrics: Vec<(usize, Metrics)>,
}
fn measure_hand(
    input: &Input,
    path: &[PathSample],
    limb: usize,
) -> (Option<HandSolution>, f64, f64) {
    let (lift, pitch) = equipment_motion::sample(path, input.time, limb);
    let mut target = input.tips[limb];
    target[1] += lift * input.weight;
    let angle = input.pitches[limb] + pitch * input.weight;
    let angles = posture::SWIVELS.into_iter().chain(
        (-15_i32..=15)
            .filter(|i| i % 8 != 0)
            .map(|i| i as f64 * 0.05),
    );
    let mut best = None;
    let mut violation = f64::INFINITY;
    let mut equipment = -1.;
    let mut score = f64::INFINITY;
    for swivel in angles {
        let Some(hand) = equipment_motion::HandEnvironment {
            equipment: Some(&input.equipment),
            intended_piece: input.intended[limb].as_deref(),
        }
        .solve(&input.profile, limb, target, angle, swivel, false) else {
            continue;
        };
        let error = posture::anatomical_violation(&input.profile, limb, target, &hand);
        let body_cost = anatomy_penalty(error);
        if body_cost >= score {
            continue;
        }
        // A bounded wrist solve can select a different shaft heading for each
        // elbow candidate. Keep the actual tool witness with its posture, rather
        // than stopping at the first clear arm and measuring its shaft later.
        let clear = equipment_motion::tool_clearance(
            &input.profile,
            limb,
            &hand,
            &input.equipment,
            input.intended[limb].as_deref(),
        );
        let cost = body_cost + penalty(clear);
        if cost < score {
            best = Some(hand);
            violation = error;
            equipment = clear;
            score = cost;
        }
        if score == 0. {
            break;
        }
    }
    (best, violation, equipment)
}

fn pair(input: &Input, hands: [Option<HandSolution>; 2]) -> f64 {
    match hands {
        [Some(left), Some(right)] => count_in::contact_tool_clearance(
            &input.profile,
            &left,
            &right,
            input.intentional_contact,
        ),
        _ => -1.,
    }
}
fn measure(input: &Input, path: &[PathSample]) -> Metrics {
    let values = std::array::from_fn::<_, 2, _>(|limb| measure_hand(input, path, limb));
    let hands = values.map(|v| v.0);
    Metrics {
        pair: pair(input, hands),
        hands,
        anatomy: values.map(|v| v.1 == 0.),
        violation: values.map(|v| v.1),
        equipment: values.map(|v| v.2),
    }
}
type HandMeasurement = (Option<HandSolution>, f64, f64);
const MAX_HAND_CACHE_ENTRIES: usize = 16_384;

/// Inputs/profiles/obstacles are immutable for one route invocation. Only the
/// effective height and pitch vary. Cache the one-hand witness, never a pair
/// metric that would retain an obsolete opposite-hand pose after a repair.
struct HandMemo {
    entries: HashMap<(usize, usize, u64, u64), HandMeasurement>,
    capacity: usize,
    #[cfg(test)]
    hits: usize,
}
impl HandMemo {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity,
            #[cfg(test)]
            hits: 0,
        }
    }
    fn measure(
        &mut self,
        index: usize,
        input: &Input,
        path: &[PathSample],
        limb: usize,
    ) -> HandMeasurement {
        let (lift, pitch) = equipment_motion::sample(path, input.time, limb);
        let height = input.tips[limb][1] + lift * input.weight;
        let angle = input.pitches[limb] + pitch * input.weight;
        let key = (index, limb, height.to_bits(), angle.to_bits());
        if let Some(&value) = self.entries.get(&key) {
            #[cfg(test)]
            {
                self.hits += 1;
            }
            return value;
        }
        let value = measure_hand(input, path, limb);
        // Deterministic bounded storage. Eviction only repeats the same query;
        // candidate order, work-budget accounting and selection do not change.
        if self.entries.len() == self.capacity {
            self.entries.clear();
        }
        self.entries.insert(key, value);
        value
    }
}

/// A cone edits one limb. Always combine its exact cached/new witness with the
/// current other hand, then recompute their pair clearance.
fn measure_changed(input: &Input, old: Metrics, limb: usize, value: HandMeasurement) -> Metrics {
    let (hand, violation, equipment) = value;
    let mut next = old;
    next.hands[limb] = hand;
    next.anatomy[limb] = violation == 0.;
    next.violation[limb] = violation;
    next.equipment[limb] = equipment;
    next.pair = pair(input, next.hands);
    next
}
fn pair_margin(input: &Input) -> f64 {
    if input.close_tool_approach && !input.intentional_contact {
        0.
    } else {
        posture::MARGIN
    }
}
fn total_penalty(input: &Input, metrics: Metrics) -> f64 {
    (pair_margin(input) - metrics.pair).max(0.).powi(2)
        + metrics.equipment.into_iter().map(penalty).sum::<f64>()
        + metrics
            .violation
            .into_iter()
            .map(anatomy_penalty)
            .sum::<f64>()
}

fn anatomy_penalty(violation: f64) -> f64 {
    if violation == 0. {
        0.
    } else {
        0.01 + 0.05 * violation.min(10.)
    }
}

fn penalty(clearance: f64) -> f64 {
    (posture::MARGIN - clearance).max(0.).powi(2)
}

/// Raising/lowering one control value extends its minimal rate-limited cone
/// through its neighbours. Max/min of Lipschitz curves preserves the bound.
fn envelope(
    path: &[PathSample],
    center: usize,
    limb: usize,
    pitch: bool,
    delta: f64,
    locks: &[[bool; 2]],
) -> Option<Vec<PathSample>> {
    let mut next = path.to_vec();
    if delta.abs() < 1e-12 {
        return Some(next);
    }
    let value = if pitch {
        path[center].pitch[limb]
    } else {
        path[center].lift[limb]
    } + delta;
    if (pitch && value.abs() > 0.60 + 1e-9)
        || (!pitch && !(0.0..=crate::equipment_motion::MAX_LIFT).contains(&value))
    {
        return None;
    }
    for (i, sample) in next.iter_mut().enumerate() {
        let distance = (sample.time - path[center].time).abs();
        let old = if pitch {
            &mut sample.pitch[limb]
        } else {
            &mut sample.lift[limb]
        };
        let rate = if pitch {
            equipment_motion::PITCH_RATE
        } else {
            equipment_motion::LIFT_RATE
        };
        let changed = if delta > 0. {
            old.max(value - rate * distance)
        } else {
            old.min(value + rate * distance)
        };
        if !pitch && locks[i][limb] {
            if (changed - *old).abs() > 1e-10 {
                return None;
            }
            // A cone that meets the impact within roundoff still preserves the
            // exact source anchor. Writing a tiny nonzero lift would invalidate
            // the worker transfer's exact contact lock.
            continue;
        }
        *old = changed;
    }
    Some(next)
}

/// An interior failure can be closest to an impact, or a nearly coincident
/// grid control whose rate budget is exhausted by that impact. Its other
/// bounding control may carry the same lift without moving the contact.
fn lift_envelope(
    path: &[PathSample],
    locks: &[[bool; 2]],
    time: f64,
    center: usize,
    limb: usize,
    delta: f64,
) -> Option<Vec<PathSample>> {
    if let Some(next) = envelope(path, center, limb, false, delta, locks) {
        return Some(next);
    }
    let right = path.partition_point(|sample| sample.time < time);
    if right == 0 || right == path.len() || time == path[right].time {
        return None;
    }
    let other = [right - 1, right]
        .into_iter()
        .find(|&index| index != center && !locks[index][limb])?;
    envelope(path, other, limb, false, delta, locks)
}

/// The bounded repair chooses pitch/lift routes from complete static hand/tool
/// witnesses. The later elbow planner still owns the continuous hand controls.
pub(super) fn route(path: &mut Vec<PathSample>, inputs: &[Input], locks: &[[bool; 2]]) {
    route_with_reuse::<true>(path, inputs, locks);
}

fn route_with_reuse<const REUSE: bool>(
    path: &mut Vec<PathSample>,
    inputs: &[Input],
    locks: &[[bool; 2]],
) -> usize {
    if path.is_empty() {
        return 0;
    }
    let ceilings: Vec<[f64; 2]> = inputs
        .iter()
        .map(|input| {
            std::array::from_fn(|limb| {
                if input.weight == 0. {
                    equipment_motion::MAX_LIFT
                } else {
                    (equipment_motion::lift_ceiling(
                        &input.equipment,
                        input.tips[limb],
                        input.intended[limb].as_deref(),
                    ) / input.weight)
                        .min(equipment_motion::MAX_LIFT)
                }
            })
        })
        .collect();
    let mut metrics: Vec<_> = inputs.iter().map(|input| measure(input, path)).collect();
    // Deterministic work bound for long songs. Remaining failures stay explicit
    // in final inspection rather than turning an unbounded search into startup.
    let mut memo = HandMemo::new(MAX_HAND_CACHE_ENTRIES);
    let mut evaluations = 0usize;
    const MAX_REPAIR_EVALUATIONS: usize = 250_000;
    for _ in 0..4 {
        let mut changed = false;
        let failing: Vec<_> = metrics
            .iter()
            .enumerate()
            .filter(|(i, m)| total_penalty(&inputs[*i], **m) > 1e-14)
            .map(|(i, _)| i)
            .collect();
        for index in failing {
            if total_penalty(&inputs[index], metrics[index]) <= 1e-14 || inputs[index].weight < 1e-9
            {
                continue;
            }
            let at = path.partition_point(|s| s.time < inputs[index].time);
            let center = if at == 0 {
                0
            } else if at == path.len()
                || inputs[index].time - path[at - 1].time <= path[at].time - inputs[index].time
            {
                at - 1
            } else {
                at
            };
            let mut best: Option<Repair> = None;
            for limb in [1, 0] {
                for (pitch_delta, lift_delta) in [
                    (-0.1, 0.),
                    (0.1, 0.),
                    (-0.2, 0.),
                    (0.2, 0.),
                    (0., -0.025),
                    (0., 0.025),
                    (0., -0.05),
                    (0., 0.05),
                    (-0.1, -0.025),
                    (-0.1, 0.025),
                    (0.1, -0.025),
                    (0.1, 0.025),
                    (-0.2, -0.05),
                    (-0.2, 0.05),
                    (0.2, -0.05),
                    (0.2, 0.05),
                ] {
                    let Some(next) = envelope(path, center, limb, true, pitch_delta, locks)
                        .and_then(|next| {
                            lift_envelope(
                                &next,
                                locks,
                                inputs[index].time,
                                center,
                                limb,
                                lift_delta,
                            )
                        })
                    else {
                        continue;
                    };
                    let from = next
                        .iter()
                        .zip(path.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(center);
                    let to = next
                        .iter()
                        .zip(path.iter())
                        .rposition(|(a, b)| a != b)
                        .unwrap_or(center);
                    let start = path[from.saturating_sub(1)].time;
                    let end = path[(to + 1).min(path.len() - 1)].time;
                    let first = inputs.partition_point(|s| s.time < start - 1e-9);
                    let last = inputs.partition_point(|s| s.time <= end + 1e-9);
                    let mut changes = Vec::new();
                    let mut improvement = 0.;
                    let mut accepted = true;
                    for i in first..last {
                        if evaluations == MAX_REPAIR_EVALUATIONS {
                            return evaluations;
                        }
                        evaluations += 1;
                        let old = metrics[i];
                        let controls = equipment_motion::sample(&next, inputs[i].time, limb);
                        let original_controls =
                            equipment_motion::sample(path, inputs[i].time, limb);
                        // Keep the selected underpass. If an interpolated old
                        // sample already exceeds its cap, a repair may reduce
                        // that excess but cannot add to it. The shaft query
                        // still decides whether the resulting pose is feasible.
                        if controls.0 > ceilings[i][limb].max(original_controls.0) + 1e-10 {
                            accepted = false;
                            break;
                        }
                        let m = if REUSE {
                            if inputs[i].weight == 0. || controls == original_controls {
                                old
                            } else {
                                let value = memo.measure(i, &inputs[i], &next, limb);
                                measure_changed(&inputs[i], old, limb, value)
                            }
                        } else {
                            let value = measure_hand(&inputs[i], &next, limb);
                            measure_changed(&inputs[i], old, limb, value)
                        };
                        if (0..2).any(|j| {
                            (!m.anatomy[j] && old.anatomy[j])
                                || (old.equipment[j] >= posture::MARGIN
                                    && m.equipment[j] < posture::MARGIN - 1e-7)
                        }) || (old.pair >= pair_margin(&inputs[i])
                            && m.pair < pair_margin(&inputs[i]) - 1e-7)
                        {
                            accepted = false;
                            break;
                        }
                        improvement +=
                            total_penalty(&inputs[i], old) - total_penalty(&inputs[i], m);
                        changes.push((i, m));
                    }
                    if !accepted || improvement <= 1e-10 {
                        continue;
                    }
                    // Prefer a smaller route when equal physical improvement is available.
                    let score = improvement - (pitch_delta.abs() + lift_delta.abs()) * 1e-8;
                    if best.as_ref().is_none_or(|b| score > b.score) {
                        best = Some(Repair {
                            score,
                            path: next,
                            metrics: changes,
                        });
                    }
                }
            }
            if let Some(repair) = best {
                *path = repair.path;
                for (i, m) in repair.metrics {
                    metrics[i] = m;
                }
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    evaluations
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> RigProfile {
        let mut profile = RigProfile::primitive();
        profile.roots[0] = [-0.2, 1.3, 0.1];
        profile.roots[1] = [0.2, 1.3, 0.1];
        for limb in 0..2 {
            profile.lengths[limb] = [0.3, 0.27];
            profile.grip_offsets[limb] = [0., 0., 0.03];
            profile.tip_offsets[limb] = [0., 0., 0.31];
            profile.contact_rotations[limb] = [0., 0., 0., 1.];
        }
        let frame = crate::rig::HandControlFrame {
            forearm: [0., 0., 1.],
            palm: [0., 1., 0.],
            desired_palm: [0., 1., 0.],
            palm_roll_range: None,
            palm_reference: None,
        };
        profile.hand_frames = Some([frame.clone(), frame]);
        profile
    }
    fn assert_hand_measurement_eq(a: HandMeasurement, b: HandMeasurement) {
        assert_eq!(a.1.to_bits(), b.1.to_bits());
        assert_eq!(a.2.to_bits(), b.2.to_bits());
        match (a.0, b.0) {
            (Some(a), Some(b)) => {
                assert_eq!(a.middle.map(f64::to_bits), b.middle.map(f64::to_bits));
                assert_eq!(a.end.map(f64::to_bits), b.end.map(f64::to_bits));
                assert_eq!(a.tip.map(f64::to_bits), b.tip.map(f64::to_bits));
                assert_eq!(a.rotation.map(f64::to_bits), b.rotation.map(f64::to_bits));
                assert_eq!(a.reachable, b.reachable);
                assert!(a
                    .middle
                    .into_iter()
                    .chain(a.end)
                    .chain(a.tip)
                    .chain(a.rotation)
                    .all(f64::is_finite));
            }
            (None, None) => {}
            _ => panic!("cached and direct hand availability differ"),
        }
    }
    #[test]
    fn hand_memo_is_exact_bounded_and_recombines_the_current_other_hand() {
        let input = Input {
            time: 0.,
            profile: profile(),
            tips: [[-0.1, 0.95, 0.65], [-0.25, 1.03, 0.5]],
            pitches: [0.; 2],
            equipment: Arc::new(EquipmentContext::new(
                vec![],
                crate::equipment::kit_geometry(&[]),
            )),
            intended: [None, None],
            weight: 1.,
            intentional_contact: false,
            close_tool_approach: false,
        };
        let path = [PathSample {
            time: 0.,
            lift: [0.; 2],
            pitch: [0.125, -0.3],
        }];
        let mut memo = HandMemo::new(3);
        let left = memo.measure(0, &input, &path, 0);
        assert_hand_measurement_eq(left, measure_hand(&input, &path, 0));
        assert_hand_measurement_eq(left, memo.measure(0, &input, &path, 0));
        assert_eq!(memo.hits, 1);
        // Adjacent floating values are separate keys, with no quantization.
        let mut adjacent = path;
        adjacent[0].pitch[0] = f64::from_bits(path[0].pitch[0].to_bits() + 1);
        assert_hand_measurement_eq(
            memo.measure(0, &input, &adjacent, 0),
            measure_hand(&input, &adjacent, 0),
        );
        assert_eq!(memo.hits, 1);
        assert_eq!(memo.entries.len(), 2);
        adjacent = path;
        adjacent[0].lift[0] = f64::from_bits(input.tips[0][1].to_bits() + 1) - input.tips[0][1];
        assert_hand_measurement_eq(
            memo.measure(0, &input, &adjacent, 0),
            measure_hand(&input, &adjacent, 0),
        );
        assert_eq!(memo.entries.len(), 3);
        assert_eq!(memo.hits, 1);
        // A different immutable input index cannot inherit the previous query.
        assert_hand_measurement_eq(memo.measure(1, &input, &path, 0), left);
        assert_eq!(memo.entries.len(), 1);
        assert_eq!(memo.hits, 1);
        assert_hand_measurement_eq(memo.measure(0, &input, &path, 0), left);
        assert_eq!(memo.entries.len(), 2);
        let original = measure(&input, &path);
        let mut changed = path;
        changed[0].pitch[1] += 0.2;
        let other = memo.measure(0, &input, &changed, 1);
        let current = measure_changed(&input, original, 1, other);
        let cached_left = memo.measure(0, &input, &changed, 0);
        assert_eq!(memo.hits, 2);
        let recombined = measure_changed(&input, current, 0, cached_left);
        let direct = measure(&input, &changed);
        assert_eq!(recombined.pair.to_bits(), direct.pair.to_bits());
        assert_ne!(original.pair.to_bits(), direct.pair.to_bits());
        for limb in 0..2 {
            assert_hand_measurement_eq(
                (
                    recombined.hands[limb],
                    recombined.violation[limb],
                    recombined.equipment[limb],
                ),
                (
                    direct.hands[limb],
                    direct.violation[limb],
                    direct.equipment[limb],
                ),
            );
        }
    }
    #[test]
    fn route_memo_matches_sequential_repairs_of_both_limbs_and_work_budget() {
        let profile = profile();
        let mut geometry = crate::equipment::kit_geometry(&[]);
        for x in [-0.1, 0.1] {
            geometry.bodies.push(crate::equipment::EquipmentBox {
                piece_id: format!("overhead-{x}"),
                center: [x, 1.06, 0.65],
                half_extents: [0.07, 0.025, 0.07],
                rotation: None,
                kind: "body",
            });
        }
        let equipment = Arc::new(EquipmentContext::new(vec![], geometry));
        let original: Vec<_> = (0..=20)
            .map(|i| PathSample {
                time: i as f64 * 0.025,
                lift: [0.1; 2],
                pitch: [0.; 2],
            })
            .collect();
        let inputs: Vec<_> = original
            .iter()
            .map(|s| Input {
                time: s.time,
                profile: profile.clone(),
                tips: [[-0.1, 0.95, 0.65], [0.1, 0.95, 0.65]],
                pitches: [0.; 2],
                equipment: equipment.clone(),
                intended: [None, None],
                weight: 1.,
                intentional_contact: false,
                close_tool_approach: false,
            })
            .collect();
        let locks = vec![[false; 2]; original.len()];
        let mut cached = original.clone();
        let mut direct = original.clone();
        let cached_work = route_with_reuse::<true>(&mut cached, &inputs, &locks);
        let direct_work = route_with_reuse::<false>(&mut direct, &inputs, &locks);
        assert_eq!(cached, direct);
        assert_eq!(cached_work, direct_work);
        assert!(cached_work > 0);
        for limb in 0..2 {
            assert!(cached
                .iter()
                .zip(&original)
                .any(|(a, b)| a.lift[limb] != b.lift[limb] || a.pitch[limb] != b.pitch[limb]));
        }
        assert_eq!(
            cached.iter().map(|s| s.time).collect::<Vec<_>>(),
            original.iter().map(|s| s.time).collect::<Vec<_>>()
        );
        for input in &inputs {
            let checked = measure(input, &cached);
            assert!(checked.anatomy.into_iter().all(|v| v));
            assert!(checked
                .equipment
                .into_iter()
                .all(|v| v >= posture::MARGIN - 1e-7));
            assert!(checked.pair >= posture::MARGIN - 1e-7);
        }
    }
    #[test]
    fn near_impact_shaft_repair_uses_the_free_interval_endpoint() {
        let profile = profile();
        let path: Vec<_> = (0..3)
            .map(|i| PathSample {
                time: i as f64 / 30.,
                lift: [0.; 2],
                pitch: [0.; 2],
            })
            .collect();
        let locks = [[true; 2], [false; 2], [true; 2]];
        let target = [-0.1, 0.94, 0.65];
        let hand = profile.solve_hand_with_swivel(0, target, 0., 0.).unwrap();
        let (butt, tip) = equipment_motion::stick_segment(&profile, 0, &hand);
        let mut center = std::array::from_fn(|axis| (butt[axis] + tip[axis]) * 0.5);
        center[1] -= 0.006;
        let mut geometry = crate::equipment::kit_geometry(&[]);
        geometry.bodies.push(crate::equipment::EquipmentBox {
            piece_id: "shaft-obstacle".into(),
            center,
            half_extents: [0.01, 0.001, 0.01],
            rotation: None,
            kind: "body",
        });
        let equipment = Arc::new(EquipmentContext::new(vec![], geometry));
        for (time, nearest) in [(1. / 120., 0), (7. / 120., 2)] {
            let input = Input {
                time,
                profile: profile.clone(),
                tips: [target, [0.25, 0.95, 0.65]],
                pitches: [0.; 2],
                equipment: equipment.clone(),
                intended: [None, None],
                weight: 1.,
                intentional_contact: false,
                close_tool_approach: false,
            };
            assert!(measure_hand(&input, &path, 0).2 < 0.);
            assert!(envelope(&path, nearest, 0, false, 0.025, &locks).is_none());
            let repaired = lift_envelope(&path, &locks, time, nearest, 0, 0.025).unwrap();
            let lifted_hand = measure_hand(&input, &repaired, 0);
            assert!(
                lifted_hand.2 >= posture::MARGIN,
                "time {time}, clearance {}, lift {:?}",
                lifted_hand.2,
                equipment_motion::sample(&repaired, time, 0)
            );
            assert_eq!(repaired[0], path[0]);
            assert_eq!(repaired[2], path[2]);
            assert!(repaired.windows(2).all(|w| {
                (w[1].lift[0] - w[0].lift[0]).abs()
                    <= equipment_motion::LIFT_RATE * (w[1].time - w[0].time)
            }));
        }
        for time in [0., 2. / 30.] {
            let nearest = if time == 0. { 0 } else { 2 };
            assert_eq!(lift_envelope(&path, &locks, time, nearest, 0, 0.025), None);
        }
        assert_eq!(
            lift_envelope(&path, &[[true; 2]; 3], 1. / 120., 0, 0, 0.025),
            None
        );
        let mut near_grid = path.clone();
        near_grid.insert(
            1,
            PathSample {
                time: 0.000_001,
                ..path[0]
            },
        );
        let locks = [[true; 2], [false; 2], [false; 2], [true; 2]];
        assert!(envelope(&near_grid, 1, 0, false, 0.025, &locks).is_none());
        let repaired = lift_envelope(&near_grid, &locks, 1. / 120., 1, 0, 0.025).unwrap();
        assert_eq!(repaired[0], near_grid[0]);
        assert_eq!(repaired[3], near_grid[3]);
        assert!(equipment_motion::sample(&repaired, 1. / 120., 0).0 > 0.006);
    }
    #[test]
    fn bounded_wrist_route_retains_a_clear_tool_witness() {
        let mut profile = profile();
        let frame = &mut profile.hand_frames.as_mut().unwrap()[0];
        frame.palm = [1., 0., 0.];
        frame.desired_palm = [1., 0., 0.];
        frame.palm_roll_range = Some([-std::f64::consts::FRAC_PI_2, 0.]);
        frame.palm_reference = Some(crate::rig::PalmReference::ShaftHeading);
        let target = [-0.1, 0.95, 0.65];
        let initial = profile.solve_hand_with_swivel(0, target, 0., 0.).unwrap();
        assert!(posture::anatomical(&profile, 0, target, &initial));
        let (butt, tip) = equipment_motion::stick_segment(&profile, 0, &initial);
        let center = std::array::from_fn(|i| butt[i] * 0.65 + tip[i] * 0.35);
        let mut geometry = crate::equipment::kit_geometry(&[]);
        geometry.bodies.push(crate::equipment::EquipmentBox {
            piece_id: "narrow-obstacle".into(),
            center,
            half_extents: [0.003; 3],
            rotation: None,
            kind: "body",
        });
        let equipment = Arc::new(EquipmentContext::new(vec![], geometry));
        assert!(equipment_motion::tool_clearance(&profile, 0, &initial, &equipment, None) < 0.);
        let input = Input {
            time: 0.,
            profile,
            tips: [target, [0.1, 0.95, 0.65]],
            pitches: [0.; 2],
            equipment,
            intended: [None, None],
            weight: 1.,
            intentional_contact: false,
            close_tool_approach: false,
        };
        let path = [PathSample {
            time: 0.,
            lift: [0.; 2],
            pitch: [0.; 2],
        }];
        let (hand, violation, clearance) = measure_hand(&input, &path, 0);
        assert_eq!(violation, 0.);
        assert!(clearance >= posture::MARGIN, "tool clearance: {clearance}");
        assert!(crate::motion::distance(hand.unwrap().tip, target) < 1e-9);
    }
    #[test]
    fn crossing_contact_approaches_use_joint_pitch_without_moving_contacts() {
        let profile = profile();
        let equipment = Arc::new(EquipmentContext::new(
            vec![],
            crate::equipment::kit_geometry(&[]),
        ));
        let mut path: Vec<_> = (0..9)
            .map(|i| PathSample {
                time: i as f64 / 30.,
                lift: [0.; 2],
                pitch: [0., -0.3],
            })
            .collect();
        let inputs: Vec<_> = path
            .iter()
            .map(|s| Input {
                time: s.time,
                profile: profile.clone(),
                tips: [[-0.1, 0.95, 0.65], [-0.25, 1.03, 0.5]],
                pitches: [0.; 2],
                equipment: equipment.clone(),
                intended: [None, None],
                weight: 1.,
                intentional_contact: false,
                close_tool_approach: false,
            })
            .collect();
        assert!(measure(&inputs[0], &path).pair < -0.002);
        let original = path.clone();
        // All knots are immutable contacts: only approach pitch may change.
        let locks = vec![[true; 2]; path.len()];
        route(&mut path, &inputs, &locks);
        for input in &inputs {
            let clear = measure(input, &path);
            assert!(clear.pair >= posture::MARGIN - 1e-7, "{}", clear.pair);
            assert!(clear.anatomy.iter().all(|v| *v));
        }
        assert!(path.iter().all(|s| s.lift == [0.; 2]));
        assert_eq!(
            path.iter().map(|s| s.time).collect::<Vec<_>>(),
            original.iter().map(|s| s.time).collect::<Vec<_>>()
        );
        let mut repeated = original;
        route_with_reuse::<false>(&mut repeated, &inputs, &locks);
        assert_eq!(path, repeated);
    }
    #[test]
    fn overhead_obstacle_can_remove_an_unsafe_lift_without_breaking_clear_tools() {
        let profile = profile();
        let mut geometry = crate::equipment::kit_geometry(&[]);
        geometry.bodies.push(crate::equipment::EquipmentBox {
            piece_id: "overhead".into(),
            center: [-0.1, 1.06, 0.65],
            half_extents: [0.07, 0.025, 0.07],
            rotation: None,
            kind: "body",
        });
        let equipment = Arc::new(EquipmentContext::new(vec![], geometry));
        let mut path: Vec<_> = (0..=20)
            .map(|i| PathSample {
                time: i as f64 * 0.025,
                lift: [0.1, 0.],
                pitch: [0.; 2],
            })
            .collect();
        let inputs: Vec<_> = path
            .iter()
            .map(|s| Input {
                time: s.time,
                profile: profile.clone(),
                tips: [[-0.1, 0.95, 0.65], [0.1, 0.95, 0.65]],
                pitches: [0.; 2],
                equipment: equipment.clone(),
                intended: [None, None],
                weight: 1.,
                intentional_contact: false,
                close_tool_approach: false,
            })
            .collect();
        let before = measure(&inputs[10], &path);
        assert!(before.equipment[0] < 0.);
        assert!(before.pair >= posture::MARGIN);
        let mut uncached = path.clone();
        route_with_reuse::<false>(&mut uncached, &inputs, &vec![[false; 2]; inputs.len()]);
        route(&mut path, &inputs, &vec![[false; 2]; inputs.len()]);
        assert_eq!(path, uncached);
        for input in &inputs {
            let m = measure(input, &path);
            assert!(
                m.equipment.iter().all(|v| *v >= posture::MARGIN - 1e-7),
                "{:?}",
                m.equipment
            );
            assert!(m.pair >= posture::MARGIN - 1e-7);
            assert!(
                m.anatomy.iter().all(|v| *v),
                "time={} violations={:?}",
                input.time,
                m.violation
            );
        }
        assert!(path.iter().any(|s| s.lift[0] < 0.1));
    }
    #[test]
    fn minimal_envelopes_preserve_rates_and_exact_contact_locks() {
        let path: Vec<_> = (0..21)
            .map(|i| PathSample {
                time: i as f64 * 0.025,
                lift: [0.; 2],
                pitch: [0.; 2],
            })
            .collect();
        let mut locks = vec![[false; 2]; 21];
        locks[0] = [true; 2];
        locks[20] = [true; 2];
        let lifted = envelope(&path, 10, 1, false, 0.1, &locks).unwrap();
        assert_eq!(lifted[0].lift[1], 0.);
        assert_eq!(lifted[20].lift[1], 0.);
        assert_eq!(lifted[10].lift[1], 0.1);
        assert!(lifted
            .windows(2)
            .all(|w| (w[1].lift[1] - w[0].lift[1]).abs() <= 1.2 * (w[1].time - w[0].time) + 1e-10));
        assert!(envelope(&path, 0, 1, false, 0.1, &locks).is_none());
        let pitched = envelope(&path, 10, 0, true, -0.2, &locks).unwrap();
        assert!(
            pitched
                .windows(2)
                .all(|w| (w[1].pitch[0] - w[0].pitch[0]).abs()
                    <= 2. * (w[1].time - w[0].time) + 1e-10)
        );
        assert_eq!(
            pitched.iter().map(|s| s.time).collect::<Vec<_>>(),
            path.iter().map(|s| s.time).collect::<Vec<_>>()
        );
    }

    #[test]
    fn cone_roundoff_does_not_write_a_nonzero_impact_lift() {
        let path = [2.25, 2.2916666666666665].map(|time| PathSample {
            time,
            lift: [0.; 2],
            pitch: [0.; 2],
        });
        let locks = [[true; 2], [false; 2]];
        let lifted = envelope(&path, 1, 1, false, 0.05, &locks).unwrap();
        assert_eq!(lifted[0].lift[1].to_bits(), 0_f64.to_bits());
        assert_eq!(lifted[1].lift[1], 0.05);
        assert!(
            lifted[1].lift[1] <= equipment_motion::LIFT_RATE * (path[1].time - path[0].time) + 1e-9
        );
    }
}
