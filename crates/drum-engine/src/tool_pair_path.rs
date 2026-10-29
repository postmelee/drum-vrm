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
use std::sync::Arc;

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
    for swivel in angles {
        let Some(hand) = input
            .profile
            .solve_hand_with_swivel(limb, target, angle, swivel)
        else {
            continue;
        };
        let error = posture::anatomical_violation(&input.profile, limb, target, &hand);
        if error < violation {
            best = Some(hand);
            violation = error;
        }
        if error == 0. {
            break;
        }
    }
    let equipment = best.as_ref().map_or(-1., |hand| {
        equipment_motion::tool_clearance(
            &input.profile,
            limb,
            hand,
            &input.equipment,
            input.intended[limb].as_deref(),
        )
    });
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
/// A cone edits one limb. Keep the other limb's exact witness and obstacle result
/// instead of resolving both arms for every candidate and every interior sample.
fn measure_changed(input: &Input, path: &[PathSample], old: Metrics, limb: usize) -> Metrics {
    let (hand, violation, equipment) = measure_hand(input, path, limb);
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
            .map(|v| {
                if v == 0. {
                    0.
                } else {
                    0.01 + 0.05 * v.min(10.)
                }
            })
            .sum::<f64>()
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
    if (pitch && value.abs() > 0.60 + 1e-9) || (!pitch && !(0.0..=0.20).contains(&value)) {
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

/// The bounded repair is deliberately separate from elbow-plane optimization:
/// swivel changes posture but cannot move a target-oriented shaft's centerline.
pub(super) fn route(path: &mut Vec<PathSample>, inputs: &[Input], locks: &[[bool; 2]]) {
    if path.is_empty() {
        return;
    }
    let mut metrics: Vec<_> = inputs.iter().map(|input| measure(input, path)).collect();
    // Deterministic work bound for long songs. Remaining failures stay explicit
    // in final inspection rather than turning an unbounded search into startup.
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
                        .and_then(|next| envelope(&next, center, limb, false, lift_delta, locks))
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
                            return;
                        }
                        evaluations += 1;
                        let old = metrics[i];
                        let m = measure_changed(&inputs[i], &next, old, limb);
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
        route(&mut repeated, &inputs, &locks);
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
        route(&mut path, &inputs, &vec![[false; 2]; inputs.len()]);
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
