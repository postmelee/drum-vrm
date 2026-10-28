//! Deterministic whole-timeline elbow-plane planning over immutable contacts.
//! Node and interpolated-edge checks share the calibrated body proxies. This
//! bounded graph is not a guarantee of skin clearance or a continuous certificate.
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
    pub equipment: Option<std::sync::Arc<crate::equipment_motion::EquipmentContext>>,
    pub intended_stick_contact: bool,
    pub close_tool_approach: bool,
    pub intended_pieces: [Option<String>; 2],
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
fn collision_cost(equipment: [f64; 2], tools: f64, close_approach: bool) -> f64 {
    let depths = [
        (equipment[0], crate::posture::MARGIN),
        (equipment[1], crate::posture::MARGIN),
        (
            tools,
            if close_approach {
                0.
            } else {
                crate::posture::MARGIN
            },
        ),
    ];
    // Only physical tool/hardware intersections receive a feasibility penalty.
    // Body proxies remain bounded visual preferences, retained in inspection.
    depths
        .iter()
        .map(|&(c, margin)| deficit(c, margin) * 10_000.0)
        .sum::<f64>()
        + if depths.iter().any(|&(c, margin)| c < margin) {
            4.0
        } else {
            0.0
        }
}
fn equipment_clearance(
    input: &HandCoordinationInput,
    limb: usize,
    hand: &crate::rig::HandSolution,
) -> crate::equipment_motion::HandClearance {
    input.equipment.as_ref().map_or(
        crate::equipment_motion::HandClearance {
            tool: f64::INFINITY,
            physical_tool: f64::INFINITY,
            body: f64::INFINITY,
        },
        |ctx| {
            crate::equipment_motion::assess_hand(
                &input.profile,
                limb,
                hand,
                ctx,
                input.intended_pieces[limb].as_deref(),
            )
        },
    )
}
fn pair_cost_for_pose(
    profile: &RigProfile,
    left: &crate::rig::HandSolution,
    right: &crate::rig::HandSolution,
    equipment: [f64; 2],
    intended_contact: bool,
    close_approach: bool,
) -> f64 {
    let pair = clearance::pair_clearance(profile, left, right);
    collision_cost(
        equipment,
        crate::count_in::contact_tool_clearance(profile, left, right, intended_contact),
        close_approach && !intended_contact,
    ) + clearance::body_penalty(pair.arms.min(pair.cross))
}
fn hand_cost_for_pose(
    input: &HandCoordinationInput,
    limb: usize,
    hand: &crate::rig::HandSolution,
) -> (f64, f64) {
    let equipment = equipment_clearance(input, limb, hand);
    let body = clearance::hand_clearance(&input.profile, limb, hand).min(equipment.body);
    (
        equipment.tool,
        clearance::body_penalty(body) + posture_cost(&input.profile, limb, input.tips[limb], hand),
    )
}
fn posture_cost(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    hand: &crate::rig::HandSolution,
) -> f64 {
    crate::posture::comfort_cost(profile, limb, target, hand)
        + if crate::posture::anatomical(profile, limb, target, hand) {
            0.0
        } else {
            1e9
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
    let (lc, lp) = hand_cost_for_pose(input, 0, &left);
    let (rc, rp) = hand_cost_for_pose(input, 1, &right);
    pair_cost_for_pose(
        &input.profile,
        &left,
        &right,
        [lc, rc],
        input.intended_stick_contact,
        input.close_tool_approach,
    ) + lp
        + rp
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
            best = best.min(
                pair_cost_for_pose(profile, left, right, [f64::INFINITY; 2], false, false)
                    + clearance::body_penalty(*lc)
                    + clearance::body_penalty(*rc)
                    + posture_cost(profile, 0, tips[0], left)
                    + posture_cost(profile, 1, tips[1], right),
            );
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
                        let (c, p) = hand_cost_for_pose(input, limb, &h);
                        (h, c, p)
                    })
            })
            .collect()
    });
    (0..STATES)
        .map(|state| {
            let (Some((left, lc, lp)), Some((right, rc, rp))) =
                (hands[0][state / AXIS_STATES], hands[1][state % AXIS_STATES])
            else {
                return 1e9;
            };
            if !left.reachable || !right.reachable {
                return 1e9;
            }
            let a = angles(state);
            pair_cost_for_pose(
                &input.profile,
                &left,
                &right,
                [lc, rc],
                input.intended_stick_contact,
                input.close_tool_approach,
            ) + lp
                + rp
                + 0.002 * (a[0] * a[0] + a[1] * a[1])
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
        if edges[i].is_empty() {
            // The unchanged graph transition is separable in left/right swivel.
            // Two bounded one-dimensional minima replace the square neighbor
            // product. Keep predecessor indices and stable ascending tie order.
            let mut rows = vec![(f64::INFINITY, 0usize); STATES];
            let rate_unit = ANGLE_STEP.powi(2) * 0.01;
            for pl in 0..AXIS_STATES {
                for r in 0..AXIS_STATES {
                    let slot = pl * AXIS_STATES + r;
                    for pr in r.saturating_sub(steps)..=(r + steps).min(AXIS_STATES - 1) {
                        let before = pl * AXIS_STATES + pr;
                        let value = previous[before] + (r as f64 - pr as f64).powi(2) * rate_unit;
                        if value < rows[slot].0 {
                            rows[slot] = (value, before);
                        }
                    }
                }
            }
            for state in 0..STATES {
                let (l, r) = (state / AXIS_STATES, state % AXIS_STATES);
                for pl in l.saturating_sub(steps)..=(l + steps).min(AXIS_STATES - 1) {
                    let (row, before) = rows[pl * AXIS_STATES + r];
                    let value = row + (l as f64 - pl as f64).powi(2) * rate_unit;
                    if value < next[state] {
                        next[state] = value;
                        parents[i][state] = before as u16;
                    }
                }
                next[state] += costs[i][state];
            }
            previous = next;
            continue;
        }
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
        let mut p = profile();
        // The old 86 cm primitive arms require anatomically folded elbows at
        // these nearby targets. Author a feasible human-scale wrist workspace.
        p.lengths[0] = [0.30, 0.27];
        p.lengths[1] = [0.30, 0.27];
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
                    equipment: None,
                    intended_stick_contact: false,
                    close_tool_approach: false,
                    intended_pieces: [None, None],
                }
            })
            .collect();
        let track = plan(&inputs);
        assert_eq!(track, plan(&inputs));
        let mut baseline = 0.0;
        let mut corrected = 0.0;
        for (input, sample) in inputs.iter().zip(&track) {
            let after: Vec<_> = (0..2)
                .map(|i| {
                    p.solve_hand_with_swivel(i, input.tips[i], 0.0, sample.angles[i])
                        .unwrap()
                })
                .collect();
            // Compare the actual production objective: anatomical/tool limits
            // are hard; estimated body overlap is a bounded preference.
            baseline += pose_cost(input, [0.; 2]);
            corrected += pose_cost(input, sample.angles);
            assert!(
                clearance::tool_pair_clearance(&p, &after[0], &after[1]) >= crate::posture::MARGIN
            );
            for (i, hand) in after.iter().enumerate() {
                assert!(sample.angles[i].abs() <= MAX_SWIVEL + 1e-10);
                assert!(crate::posture::anatomical(&p, i, input.tips[i], hand));
                assert!(crate::motion::distance(hand.tip, input.tips[i]) < 1e-8);
                assert!(
                    (crate::motion::distance(p.roots[i], hand.middle) - p.lengths[i][0]).abs()
                        < 1e-8
                );
                assert!(
                    (crate::motion::distance(hand.middle, hand.end) - p.lengths[i][1]).abs() < 1e-8
                );
            }
        }
        assert!(
            corrected < baseline,
            "shared pose objective {baseline} -> {corrected}"
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
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
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
    fn separable_transition_preserves_bounded_graph_optimum() {
        let p = profile();
        let inputs: Vec<_> = [0.0, 0.037, 0.13, 0.21]
            .into_iter()
            .map(|time| HandCoordinationInput {
                time,
                profile: p.clone(),
                tips: [[-0.25, 1.0, 0.5], [0.25, 1.0, 0.5]],
                pitches: [0.0; 2],
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
            })
            .collect();
        let costs: Vec<Vec<_>> = (0..inputs.len())
            .map(|k| {
                (0..STATES)
                    .map(|s| {
                        let a = angles(s);
                        (a[0] - k as f64 * 0.07).powi(2)
                            + (a[1] + k as f64 * 0.09).powi(2)
                            + ((s * 31 + k * 7) % 97) as f64 * 0.00001
                    })
                    .collect()
            })
            .collect();
        let fast = vec![HashMap::new(); inputs.len()];
        // A never-addressed entry forces the original general transition path
        // while leaving every transition cost exactly zero.
        let general = vec![HashMap::from([(usize::MAX, 0.0)]); inputs.len()];
        assert_eq!(
            graph_path(&inputs, &costs, &fast),
            graph_path(&inputs, &costs, &general)
        );
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
            equipment: None,
            intended_stick_contact: false,
            close_tool_approach: false,
            intended_pieces: [None, None],
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
