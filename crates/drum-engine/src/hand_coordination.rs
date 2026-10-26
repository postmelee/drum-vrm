//! Deterministic whole-timeline elbow-plane planning over immutable contacts.
//! Node and interpolated-edge checks share the calibrated body proxies. This
//! bounded graph is not a guarantee of skin clearance or a continuous certificate.
#[cfg(test)]
use crate::rig::HandSolution;
use crate::{clearance, model::Vec3, rig::RigProfile};
use std::collections::HashMap;

// Control knots are coarser than validation samples. Exact impact timestamps and
// the 30 Hz sampling grid are supplied separately to inspect their interpolation.
// Exact source impacts are control knots as well, so their pose affects the graph.
pub const STEP: f64 = 0.1;
pub const MAX_SWIVEL: f64 = 0.8;
pub const MAX_SAMPLE_RATE: f64 = 2.0;
pub(crate) const ANGLE_STEP: f64 = 0.05;
const AXIS_STATES: usize = 33;
const STATES: usize = AXIS_STATES * AXIS_STATES;
const REFINEMENTS: usize = 8;
/// Separate from the MIDI parser limit: 6000 × 1089 costs (~52 MiB) plus
/// predecessor indices (~13 MiB), before modest graph/profile bookkeeping.
pub(crate) const MAX_CONTROL_KNOTS: usize = 6000;
pub(crate) fn check_control_budget(count: usize) -> Result<(), String> {
    if count > MAX_CONTROL_KNOTS {
        Err(format!("동작 계획 제어점 {count}개가 한계 {MAX_CONTROL_KNOTS}개를 초과합니다. 더 짧은 MIDI 구간을 내보내거나 선택 트랙 수를 줄여 다시 준비하세요."))
    } else {
        Ok(())
    }
}

pub struct HandCoordinationInput {
    pub time: f64,
    pub profile: RigProfile,
    pub tips: [Vec3; 2],
    pub pitches: [f64; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwivelSample {
    pub time: f64,
    pub angles: [f64; 2],
}
fn angles(state: usize) -> [f64; 2] {
    [
        (state / AXIS_STATES) as f64 * ANGLE_STEP - MAX_SWIVEL,
        (state % AXIS_STATES) as f64 * ANGLE_STEP - MAX_SWIVEL,
    ]
}
fn deficit(clearance: f64, margin: f64) -> f64 {
    (margin - clearance).max(0.0).powi(2)
}
fn collision_cost(torso: [f64; 2], pair: clearance::PairClearance) -> f64 {
    let depths = [torso[0], torso[1], pair.arms, pair.sticks, pair.cross];
    // Every sampled safe candidate is cheaper than a collision candidate before
    // neutral-pose preference. Depth still distinguishes unavoidable failures.
    depths
        .iter()
        .map(|&c| deficit(c, crate::posture::MARGIN) * 10_000.0)
        .sum::<f64>()
        + if depths.iter().any(|&c| c < crate::posture::MARGIN) {
            1.0
        } else {
            0.0
        }
}
fn pose_cost(input: &HandCoordinationInput, a: [f64; 2]) -> f64 {
    let hands = std::array::from_fn::<_, 2, _>(|limb| {
        input
            .profile
            .solve_hand_with_swivel(limb, input.tips[limb], input.pitches[limb], a[limb])
    });
    let [Some(left), Some(right)] = hands else {
        return 1e9;
    };
    if !left.reachable || !right.reachable {
        return 1e9;
    }
    collision_cost(
        [
            clearance::hand_clearance(&input.profile, 0, &left),
            clearance::hand_clearance(&input.profile, 1, &right),
        ],
        clearance::pair_clearance(&input.profile, &left, &right),
    )
}
/// Bounded joint wrist/elbow witness cost for torso and stroke-envelope planning.
/// Candidates are shared with static calibration, never tied to an avatar name.
pub(crate) fn unconstrained_cost(profile: &RigProfile, tips: [Vec3; 2], pitches: [f64; 2]) -> f64 {
    let hands: [Vec<_>; 2] = std::array::from_fn(|limb| {
        crate::posture::SWIVELS
            .iter()
            .filter_map(|&a| {
                profile
                    .solve_hand_with_swivel(limb, tips[limb], pitches[limb], a)
                    .filter(|h| h.reachable)
                    .map(|h| {
                        let c = clearance::hand_clearance(profile, limb, &h);
                        (h, c)
                    })
            })
            .collect()
    });
    let mut best: f64 = 1e9;
    for (left, lc) in &hands[0] {
        for (right, rc) in &hands[1] {
            best = best.min(collision_cost(
                [*lc, *rc],
                clearance::pair_clearance(profile, left, right),
            ));
            if best == 0.0 {
                return best;
            }
        }
    }
    best
}
fn node_costs(input: &HandCoordinationInput) -> Vec<f64> {
    let hands: [Vec<_>; 2] = std::array::from_fn(|limb| {
        (0..AXIS_STATES)
            .map(|i| {
                input
                    .profile
                    .solve_hand_with_swivel(
                        limb,
                        input.tips[limb],
                        input.pitches[limb],
                        i as f64 * ANGLE_STEP - MAX_SWIVEL,
                    )
                    .map(|h| {
                        let c = clearance::hand_clearance(&input.profile, limb, &h);
                        (h, c)
                    })
            })
            .collect()
    });
    (0..STATES)
        .map(|state| {
            let (Some((left, lc)), Some((right, rc))) =
                (hands[0][state / AXIS_STATES], hands[1][state % AXIS_STATES])
            else {
                return 1e9;
            };
            if !left.reachable || !right.reachable {
                return 1e9;
            }
            let a = angles(state);
            collision_cost(
                [lc, rc],
                clearance::pair_clearance(&input.profile, &left, &right),
            ) + 0.002 * (a[0] * a[0] + a[1] * a[1])
        })
        .collect()
}
fn graph_path(
    inputs: &[HandCoordinationInput],
    costs: &[Vec<f64>],
    edges: &[HashMap<usize, f64>],
) -> Vec<usize> {
    let mut previous = costs[0].clone();
    let mut parents = vec![vec![0u16; STATES]; inputs.len()];
    for i in 1..inputs.len() {
        let steps = ((MAX_SAMPLE_RATE * (inputs[i].time - inputs[i - 1].time) + 1e-9) / ANGLE_STEP)
            .floor() as usize;
        let steps = steps.min(AXIS_STATES - 1);
        let mut next = vec![f64::INFINITY; STATES];
        for state in 0..STATES {
            let (l, r) = (state / AXIS_STATES, state % AXIS_STATES);
            for pl in l.saturating_sub(steps)..=(l + steps).min(AXIS_STATES - 1) {
                for pr in r.saturating_sub(steps)..=(r + steps).min(AXIS_STATES - 1) {
                    let before = pl * AXIS_STATES + pr;
                    let rate = ((l as f64 - pl as f64).powi(2) + (r as f64 - pr as f64).powi(2))
                        * ANGLE_STEP.powi(2)
                        * 0.01;
                    let edge = if edges[i].is_empty() {
                        0.0
                    } else {
                        *edges[i].get(&(before * STATES + state)).unwrap_or(&0.0)
                    };
                    let cost = previous[before] + costs[i][state] + rate + edge;
                    if cost < next[state] {
                        next[state] = cost;
                        parents[i][state] = before as u16;
                    }
                }
            }
        }
        previous = next;
    }
    let mut state = (0..STATES)
        .min_by(|&a, &b| previous[a].total_cmp(&previous[b]))
        .unwrap();
    let mut path = vec![0; inputs.len()];
    for i in (0..inputs.len()).rev() {
        path[i] = state;
        state = parents[i][state] as usize;
    }
    path
}
/// Full bounded dynamic programming retains alternatives across the complete
/// phrase/timeline; a local unsafe minimum cannot pin the elbow near zero.
/// Re-evaluate selected edges using actual profiles/targets at 30 Hz and exact
/// source times. Failed edges gain their measured cost before the next search.
pub fn plan_with_checks(
    inputs: &[HandCoordinationInput],
    checks: &[HandCoordinationInput],
) -> Result<Vec<SwivelSample>, String> {
    check_control_budget(inputs.len())?;
    if inputs.is_empty() {
        return Ok(Vec::new());
    }
    let costs: Vec<_> = inputs.iter().map(node_costs).collect();
    let mut edges: Vec<HashMap<usize, f64>> = (0..inputs.len()).map(|_| HashMap::new()).collect();
    let mut best = (f64::INFINITY, Vec::new());
    for _ in 0..REFINEMENTS {
        let path = graph_path(inputs, &costs, &edges);
        let track: Vec<_> = inputs
            .iter()
            .zip(&path)
            .map(|(input, &state)| SwivelSample {
                time: input.time,
                angles: angles(state),
            })
            .collect();
        let mut updates: HashMap<(usize, usize), f64> = HashMap::new();
        let mut score = 0.0;
        for input in checks {
            let value = pose_cost(input, at(&track, input.time));
            score += value;
            if value < 1.0 {
                continue;
            }
            let i = inputs
                .partition_point(|p| p.time < input.time)
                .min(inputs.len() - 1);
            if i == 0 {
                continue;
            }
            let key = path[i - 1] * STATES + path[i];
            let old = updates.entry((i, key)).or_insert(0.0);
            *old = old.max(value);
        }
        if score < best.0 {
            best = (score, track)
        }
        let mut changed = false;
        for ((i, key), cost) in updates {
            let old = edges[i].entry(key).or_insert(0.0);
            if cost > *old + 1e-9 {
                *old = cost;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(best.1)
}
#[cfg(test)]
fn plan(inputs: &[HandCoordinationInput]) -> Vec<SwivelSample> {
    plan_with_checks(inputs, inputs).unwrap()
}
#[cfg(test)]
fn pair_cost(profile: &RigProfile, left: &HandSolution, right: &HandSolution) -> f64 {
    let pair = clearance::pair_clearance(profile, left, right);
    (deficit(pair.arms, 0.002) + deficit(pair.sticks, 0.002) + deficit(pair.cross, 0.002)) * 2000.0
}
pub fn at(track: &[SwivelSample], time: f64) -> [f64; 2] {
    if track.is_empty() {
        return [0.0; 2];
    }
    let j = track.partition_point(|s| s.time <= time);
    if j == 0 {
        return track[0].angles;
    }
    if j == track.len() {
        return track[j - 1].angles;
    }
    let (a, b) = (track[j - 1], track[j]);
    let t = ((time - a.time) / (b.time - a.time).max(1e-9)).clamp(0.0, 1.0);
    let weight = t * t * (3.0 - 2.0 * t);
    std::array::from_fn(|i| a.angles[i] + (b.angles[i] - a.angles[i]) * weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rig::HandControlFrame;
    fn profile() -> RigProfile {
        let mut p = RigProfile::primitive();
        for i in 0..2 {
            p.contact_rotations[i] = [0.0, 0.0, 0.0, 1.0];
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
    #[test]
    fn coordination_preserves_contacts_and_bounds_interpolated_motion() {
        let p = profile();
        let mut times: Vec<f64> = (0..=30).map(|i| i as f64 * STEP).collect();
        times.extend([0.517132, 0.517133, 1.431718, 2.083146]);
        times.sort_by(f64::total_cmp);
        let inputs: Vec<_> = times
            .into_iter()
            .map(|t| {
                let crossing = (t * std::f64::consts::PI / 3.0).sin();
                HandCoordinationInput {
                    time: t,
                    profile: p.clone(),
                    tips: [
                        [-0.25 + crossing * 0.55, 1.0, 0.50],
                        [0.25 - crossing * 0.55, 1.02, 0.50],
                    ],
                    pitches: [0.0; 2],
                }
            })
            .collect();
        let track = plan(&inputs);
        assert_eq!(track, plan(&inputs));
        let mut baseline = 0.0;
        let mut corrected = 0.0;
        for (input, sample) in inputs.iter().zip(&track) {
            let before: Vec<_> = (0..2)
                .map(|i| p.solve_hand(i, input.tips[i], 0.0).unwrap())
                .collect();
            let after: Vec<_> = (0..2)
                .map(|i| {
                    p.solve_hand_with_swivel(i, input.tips[i], 0.0, sample.angles[i])
                        .unwrap()
                })
                .collect();
            baseline += pair_cost(&p, &before[0], &before[1]);
            corrected += pair_cost(&p, &after[0], &after[1]);
            for (i, hand) in after.iter().enumerate() {
                assert!(sample.angles[i].abs() <= MAX_SWIVEL + 1e-10);
                assert!(crate::motion::distance(hand.tip, input.tips[i]) < 1e-8);
            }
        }
        assert!(
            corrected < baseline * 0.85,
            "pair penalty {baseline} -> {corrected}"
        );
        for pair in track.windows(2) {
            let dt = pair[1].time - pair[0].time;
            for i in 0..2 {
                assert!(
                    (pair[1].angles[i] - pair[0].angles[i]).abs() <= MAX_SAMPLE_RATE * dt + 1e-10
                );
            }
        }
        for i in 0..3000 {
            let t = i as f64 / 1000.0;
            let a = at(&track, t);
            let b = at(&track, t + 1e-5);
            for j in 0..2 {
                assert!((b[j] - a[j]).abs() <= 3.0e-5 + 1e-9);
            }
            assert_eq!(a, at(&track, t));
        }
    }
    #[test]
    fn planning_budget_is_separate_and_checked_at_the_exact_boundary() {
        assert!(check_control_budget(MAX_CONTROL_KNOTS).is_ok());
        assert!(check_control_budget(MAX_CONTROL_KNOTS + 1)
            .unwrap_err()
            .contains("MIDI"));
        assert!(check_control_budget(usize::MAX).is_err());
        const { assert!(crate::model::MAX_HITS > MAX_CONTROL_KNOTS) };
    }
    #[test]
    fn timeline_search_prepares_before_a_future_constraint() {
        let p = profile();
        let inputs: Vec<_> = (0..3)
            .map(|i| HandCoordinationInput {
                time: i as f64 * STEP,
                profile: p.clone(),
                tips: [[-0.25, 1.0, 0.5], [0.25, 1.0, 0.5]],
                pitches: [0.0; 2],
            })
            .collect();
        let center = (AXIS_STATES - 1) / 2;
        let neutral = center * AXIS_STATES + center;
        let prepare = (center - 4) * AXIS_STATES + center;
        let future = (center - 8) * AXIS_STATES + center;
        let mut costs = vec![vec![1000.0; STATES]; 3];
        costs[0][neutral] = 0.0;
        costs[1][neutral] = 0.0;
        costs[1][prepare] = 0.1;
        costs[2][future] = 0.0;
        let edges = (0..3).map(|_| HashMap::new()).collect::<Vec<_>>();
        let path = graph_path(&inputs, &costs, &edges);
        assert_eq!(path, vec![neutral, prepare, future]);
        // Greedy neutral at the middle knot cannot reach the final safe state
        // under the unchanged 2 rad/s sample bound.
        assert!((angles(neutral)[0] - angles(future)[0]).abs() > MAX_SAMPLE_RATE * STEP);
    }
    #[test]
    fn first_frame_searches_safe_swivel_without_a_fake_previous_zero() {
        let p = profile();
        let kit = crate::fit::fit_kit(&p, &crate::model::default_kit()).unwrap();
        assert!(kit.complete);
        let snare = kit.kit.iter().find(|p| p.id == "snare").unwrap();
        let hat = kit.kit.iter().find(|p| p.id == "hihat").unwrap();
        let input = HandCoordinationInput {
            time: -0.5,
            profile: p.clone(),
            tips: crate::posture::ready_pair(&p, snare, hat),
            pitches: [0.0; 2],
        };
        let costs = node_costs(&input);
        let expected = angles(
            (0..STATES)
                .min_by(|&a, &b| costs[a].total_cmp(&costs[b]))
                .unwrap(),
        );
        let hands: Vec<_> = (0..2)
            .map(|limb| {
                p.solve_hand_with_swivel(limb, input.tips[limb], 0.0, expected[limb])
                    .unwrap()
            })
            .collect();
        for (limb, hand) in hands.iter().enumerate() {
            assert!(clearance::hand_clearance(&p, limb, hand) >= crate::posture::MARGIN);
        }
        assert!(
            clearance::pair_clearance(&p, &hands[0], &hands[1]).minimum() >= crate::posture::MARGIN
        );
        assert_eq!(plan(&[input])[0].angles, expected);
    }
}
