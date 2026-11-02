//! Deterministic whole-timeline elbow-plane planning over immutable contacts.
//! Node and interpolated-edge checks share the calibrated body proxies. This
//! bounded graph is not a guarantee of skin clearance or a continuous certificate.
use crate::{
    clearance,
    model::Vec3,
    rig::{OrientationControl, Quat, RigProfile},
};
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
/// Separate from the MIDI parser limit: 6000 × 1089 costs (~52 MB) plus
/// predecessor indices (~13 MB), per-hand orientation witnesses (~16 MB), and
/// actual tool/wrist/elbow measurements, before
/// modest graph/profile bookkeeping. Orientation does not add a graph state dimension.
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
    pub cue_orientation: Option<crate::count_in::CueOrientation>,
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
    /// Actual tool orientation relative to the current contact approach.
    /// Legacy hands retain their existing solver.
    pub orientations: [Option<OrientationControl>; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrientationSample {
    pub time: f64,
    pub orientations: [Option<OrientationControl>; 2],
}
pub struct HandPlan {
    pub swivels: Vec<SwivelSample>,
    pub orientations: Vec<OrientationSample>,
}
type NodeOrientations = [Vec<Option<Quat>>; 2];
type NodeKinematics = [Vec<Option<HandKinematics>>; 2];
/// The shared actual-source gate, used both by graph ranking and the final
/// reconciliation of provisional pitch warnings. Costume clearance is advisory.
pub(crate) fn source_pose_valid(
    profile: &RigProfile,
    limb: usize,
    hand: &crate::rig::HandSolution,
    tool_clearance: f64,
) -> bool {
    hand.reachable
        && tool_clearance >= crate::posture::MARGIN - 1e-9
        && clearance::core_penetration(profile, limb, hand) <= 1e-9
        && crate::posture::anatomical(profile, limb, hand.tip, hand)
        && profile.hand_orientation_policy_valid(limb, hand.rotation)
}
#[derive(Clone, Copy)]
struct HandKinematics {
    direction: Option<Vec3>,
    rotation: Quat,
    wrist: Vec3,
}
fn hand_kinematics(
    profile: &RigProfile,
    limb: usize,
    hand: &crate::rig::HandSolution,
) -> HandKinematics {
    HandKinematics {
        direction: elbow_direction(profile, limb, hand),
        rotation: hand.rotation,
        wrist: hand.end,
    }
}
fn kinematic_change(a: Option<HandKinematics>, b: Option<HandKinematics>, duration: f64) -> f64 {
    a.zip(b).map_or(0., |(a, b)| {
        direction_change(a.direction, b.direction, duration)
            + orientation_change(Some(a.rotation), Some(b.rotation), duration)
            + 0.1 * crate::motion::distance(a.wrist, b.wrist).powi(2) / duration.max(1e-9)
    })
}
/// Use the same actual-motion objective when choosing a continuous arm curve.
pub(crate) fn physical_motion_cost(
    before_profile: &RigProfile,
    after_profile: &RigProfile,
    limb: usize,
    before: &crate::rig::HandSolution,
    after: &crate::rig::HandSolution,
    duration: f64,
) -> f64 {
    kinematic_change(
        Some(hand_kinematics(before_profile, limb, before)),
        Some(hand_kinematics(after_profile, limb, after)),
        duration,
    )
}
/// An animation preference on integrated squared actual angular velocity, not
/// an anatomical limit. The old relative-swivel bound remains a control bound.
const ELBOW_DIRECTION_RATE_WEIGHT: f64 = 0.01;

fn elbow_direction(
    profile: &RigProfile,
    limb: usize,
    hand: &crate::rig::HandSolution,
) -> Option<Vec3> {
    let arm = crate::motion::sub(hand.middle, profile.roots[limb]);
    let length = crate::motion::distance(arm, [0.; 3]);
    if !length.is_finite() || length < 1e-9 {
        return None;
    }
    let shoulders = crate::motion::sub(profile.roots[1], profile.roots[0]);
    let width = shoulders[0].hypot(shoulders[2]);
    let right = if width > 1e-9 {
        [shoulders[0] / width, 0., shoulders[2] / width]
    } else {
        [1., 0., 0.]
    };
    // Remove coordinated torso yaw so turning the entire body does not count
    // as an independent elbow gesture. Preserve actual upper-arm direction.
    Some([
        (arm[0] * right[0] + arm[2] * right[2]) / length,
        arm[1] / length,
        (-arm[0] * right[2] + arm[2] * right[0]) / length,
    ])
}
fn direction_change(a: Option<Vec3>, b: Option<Vec3>, duration: f64) -> f64 {
    a.zip(b).map_or(0., |(a, b)| {
        let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
        let cross = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let angle = crate::motion::distance(cross, [0.; 3]).atan2(dot);
        // rate² × duration makes the same trajectory comparable across
        // irregular exact-hit knots without making a new physiological claim.
        ELBOW_DIRECTION_RATE_WEIGHT * angle.powi(2) / duration.max(1e-9)
    })
}
fn rotation_dot(a: Quat, b: Quat) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>()
}
fn orientation_change(a: Option<Quat>, b: Option<Quat>, duration: f64) -> f64 {
    a.zip(b).map_or(0., |(a, b)| {
        (2. * rotation_dot(a, b).abs().clamp(0., 1.).acos()).powi(2) * 0.01 / duration.max(1e-9)
    })
}
/// Interpolate the actual orientation residual with one shortest arc.
/// This does not rerun a pointwise wrist optimizer between control knots.
fn interpolate_control(
    a: OrientationControl,
    b: OrientationControl,
    weight: f64,
) -> OrientationControl {
    interpolate_orientation(a, b, weight)
}
pub(crate) fn interpolate_orientation(a: Quat, mut b: Quat, weight: f64) -> Quat {
    let mut cosine = rotation_dot(a, b);
    if cosine < 0. {
        b = b.map(|v| -v);
        cosine = -cosine;
    }
    let (left, right) = if cosine > 0.9995 {
        (1. - weight, weight)
    } else {
        let angle = cosine.clamp(0., 1.).acos();
        (
            ((1. - weight) * angle).sin() / angle.sin(),
            (weight * angle).sin() / angle.sin(),
        )
    };
    let value: Quat = std::array::from_fn(|i| a[i] * left + b[i] * right);
    let norm = value.iter().map(|v| v * v).sum::<f64>().sqrt();
    value.map(|v| v / norm)
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
        close_approach,
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
        clearance::body_penalty(body)
            + core_cost(&input.profile, limb, hand)
            + posture_cost(&input.profile, limb, hand.tip, hand),
    )
}
fn core_cost(profile: &RigProfile, limb: usize, hand: &crate::rig::HandSolution) -> f64 {
    let depth = clearance::core_penetration(profile, limb, hand);
    // Ordinary body volume is a physical path constraint; garment measurements
    // retain their bounded visual cost. Keep a finite diagnostic fallback when
    // the fixed endpoints admit no clear candidate, as for tool obstructions.
    if depth > 0. {
        4. + 10_000. * depth.powi(2)
    } else {
        0.
    }
}
fn posture_cost(
    profile: &RigProfile,
    limb: usize,
    target: Vec3,
    hand: &crate::rig::HandSolution,
) -> f64 {
    let violation = crate::posture::anatomical_violation(profile, limb, target, hand);
    crate::posture::comfort_cost(profile, limb, target, hand)
        + if profile.hand_orientation_policy_valid(limb, hand.rotation) {
            0.
        } else {
            1e9
        }
        + if hand.reachable { 0. } else { 1e12 }
        + if crate::posture::anatomical_violation_valid(violation) {
            0.0
        } else {
            // Retain the infeasible tier, but do not rank a severe folded/
            // raised wrist the same as a near-bound miss. This selects a less
            // harmful fallback; the shared gate and diagnostic still fail.
            1e9 + 1e6 * violation.min(100.)
        }
}
fn pose_cost_and_kinematics(
    input: &HandCoordinationInput,
    control: SwivelSample,
) -> (f64, [Option<HandKinematics>; 2]) {
    let assessed = checked_pose(input, control);
    (assessed.cost, assessed.kinematics)
}
#[derive(Clone, Copy)]
struct CheckedPose {
    cost: f64,
    kinematics: [Option<HandKinematics>; 2],
}

fn checked_pose(input: &HandCoordinationInput, control: SwivelSample) -> CheckedPose {
    let hands = std::array::from_fn::<_, 2, _>(|limb| {
        input
            .profile
            .solve_hand_with_swivel_and_orientation(
                limb,
                input.tips[limb],
                input.pitches[limb],
                control.angles[limb],
                control.orientations[limb],
            )
            .map(|hand| {
                input.cue_orientation.map_or(hand, |cue| {
                    cue.apply(
                        &input.profile,
                        limb,
                        input.tips[limb],
                        control.angles[limb],
                        hand,
                    )
                })
            })
    });
    let [Some(left), Some(right)] = hands else {
        return CheckedPose {
            cost: 2.2e12,
            kinematics: [None; 2],
        };
    };
    let (lc, lp) = hand_cost_for_pose(input, 0, &left);
    let (rc, rp) = hand_cost_for_pose(input, 1, &right);
    let cost = pair_cost_for_pose(
        &input.profile,
        &left,
        &right,
        [lc, rc],
        input.intended_stick_contact,
        input.close_tool_approach,
    ) + lp
        + rp;
    CheckedPose {
        cost,
        kinematics: [
            Some(hand_kinematics(&input.profile, 0, &left)),
            Some(hand_kinematics(&input.profile, 1, &right)),
        ],
    }
}
#[cfg(test)]
fn pose_cost(input: &HandCoordinationInput, control: SwivelSample) -> f64 {
    pose_cost_and_kinematics(input, control).0
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
                        let c =
                            clearance::body_penalty(clearance::hand_clearance(profile, limb, &h))
                                + core_cost(profile, limb, &h);
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
                    + lc
                    + rc
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
fn node_costs(
    input: &HandCoordinationInput,
    sources: [bool; 2],
) -> (Vec<f64>, NodeOrientations, NodeKinematics, Vec<u8>) {
    let hands: [Vec<_>; 2] = std::array::from_fn(|limb| {
        (0..AXIS_STATES)
            .map(|i| {
                let swivel = i as f64 * ANGLE_STEP - MAX_SWIVEL;
                let base = if let Some(cue) = input.cue_orientation.filter(|cue| cue.weight == 1.) {
                    // At a full shaft cue the fixed physical orientation owns
                    // feasibility. Do not search an unrelated drum approach
                    // only to discard its orientation immediately afterward.
                    Some(input.profile.solve_hand_with_rotation(
                        limb,
                        input.tips[limb],
                        cue.rotations[limb],
                        swivel,
                    ))
                } else {
                    crate::equipment_motion::HandEnvironment {
                        equipment: input.equipment.as_deref(),
                        intended_piece: input.intended_pieces[limb].as_deref(),
                    }
                    .solve(
                        &input.profile,
                        limb,
                        input.tips[limb],
                        input.pitches[limb],
                        swivel,
                        false,
                    )
                };
                base.map(|h| {
                    let rotation = h.rotation;
                    let h = input.cue_orientation.map_or(h, |cue| {
                        cue.apply(
                            &input.profile,
                            limb,
                            input.tips[limb],
                            i as f64 * ANGLE_STEP - MAX_SWIVEL,
                            h,
                        )
                    });
                    let (c, p) = hand_cost_for_pose(input, limb, &h);
                    (h, c, p, rotation)
                })
            })
            .collect()
    });
    // Retain only actual selected-candidate quaternions in the fixed node
    // table. Encode the selected residual after the graph chooses a path.
    let orientations = std::array::from_fn(|limb| {
        let bounded = input
            .profile
            .hand_frames
            .as_ref()
            .is_some_and(|frames| frames[limb].palm_roll_range.is_some());
        hands[limb]
            .iter()
            .map(|candidate| {
                candidate
                    .as_ref()
                    .and_then(|(_, _, _, rotation)| bounded.then_some(*rotation))
            })
            .collect()
    });
    let directions = std::array::from_fn(|limb| {
        hands[limb]
            .iter()
            .map(|candidate| {
                candidate
                    .as_ref()
                    .map(|(hand, _, _, _)| hand_kinematics(&input.profile, limb, hand))
            })
            .collect()
    });
    // Source endpoints survive every later phase repair unchanged. Rank their
    // physical feasibility before finite interior/style costs, retaining a
    // diagnostic fallback if no bounded graph path can satisfy them all.
    let source_clear: [Vec<bool>; 2] = std::array::from_fn(|limb| {
        hands[limb]
            .iter()
            .map(|candidate| {
                !sources[limb]
                    || candidate.as_ref().is_some_and(|(h, tool, _, _)| {
                        source_pose_valid(&input.profile, limb, h, *tool)
                    })
            })
            .collect()
    });
    let source_failures = (0..STATES)
        .map(|state| {
            let left = state / AXIS_STATES;
            let right = state % AXIS_STATES;
            let pair_failure = sources[0]
                && sources[1]
                && hands[0][left]
                    .as_ref()
                    .zip(hands[1][right].as_ref())
                    .is_some_and(|(a, b)| {
                        crate::count_in::contact_tool_clearance(
                            &input.profile,
                            &a.0,
                            &b.0,
                            input.intended_stick_contact,
                        ) < if input.close_tool_approach {
                            -1e-9
                        } else {
                            crate::posture::MARGIN - 1e-9
                        }
                    });
            u8::from(!source_clear[0][left])
                + u8::from(!source_clear[1][right])
                + u8::from(pair_failure)
        })
        .collect();
    let costs = (0..STATES)
        .map(|state| {
            let a = angles(state);
            if input.cue_orientation.is_some_and(|cue| {
                cue.weight == 1. && (0..2).any(|i| (a[i] - cue.swivels[i]).abs() > 1e-9)
            }) {
                return f64::INFINITY;
            }
            let (Some((left, lc, lp, _)), Some((right, rc, rp, _))) =
                (hands[0][state / AXIS_STATES], hands[1][state % AXIS_STATES])
            else {
                return 2.2e12;
            };
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
        .collect();
    (costs, orientations, directions, source_failures)
}
/// Lexicographic source feasibility cannot be purchased with a large finite
/// pose/style cost. Infinite cue-incompatible nodes remain unreachable.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PathScore {
    source_failures: u32,
    cost: f64,
}
impl PathScore {
    const UNREACHABLE: Self = Self {
        source_failures: u32::MAX,
        cost: f64::INFINITY,
    };
    fn new(source_failures: u32, cost: f64) -> Self {
        if cost.is_finite() {
            Self {
                source_failures,
                cost,
            }
        } else {
            Self::UNREACHABLE
        }
    }
    fn add(self, cost: f64) -> Self {
        Self::new(self.source_failures, self.cost + cost)
    }
    fn with_node(self, failures: u32, cost: f64) -> Self {
        Self::new(
            self.source_failures.saturating_add(failures),
            self.cost + cost,
        )
    }
    fn compare(self, other: Self) -> std::cmp::Ordering {
        self.source_failures
            .cmp(&other.source_failures)
            .then(self.cost.total_cmp(&other.cost))
    }
    fn better_than(self, other: Self) -> bool {
        self.compare(other).is_lt()
    }
}
#[cfg(test)]
fn graph_path(
    inputs: &[HandCoordinationInput],
    costs: &[Vec<f64>],
    directions: &[NodeKinematics],
    edges: &[HashMap<usize, f64>],
) -> Vec<usize> {
    graph_path_with_sources(
        inputs,
        costs,
        &vec![vec![0; STATES]; inputs.len()],
        directions,
        edges,
    )
}
fn graph_path_with_sources(
    inputs: &[HandCoordinationInput],
    costs: &[Vec<f64>],
    source_failures: &[Vec<u8>],
    directions: &[NodeKinematics],
    edges: &[HashMap<usize, f64>],
) -> Vec<usize> {
    let mut previous: Vec<_> = costs[0]
        .iter()
        .enumerate()
        .map(|(s, &cost)| PathScore::new(source_failures[0][s] as u32, cost))
        .collect();
    let mut parents = vec![vec![0u16; STATES]; inputs.len()];
    for i in 1..inputs.len() {
        let duration = inputs[i].time - inputs[i - 1].time;
        let steps = ((MAX_SAMPLE_RATE * (inputs[i].time - inputs[i - 1].time) + 1e-9) / ANGLE_STEP)
            .floor() as usize;
        let steps = steps.min(AXIS_STATES - 1);
        // One per-hand transition table for this interval, shared by both
        // graph paths. Avoid repeated angular queries inside paired states.
        let transitions: [Vec<f64>; 2] = std::array::from_fn(|limb| {
            let mut values = vec![0.; STATES];
            for current in 0..AXIS_STATES {
                for before in current.saturating_sub(steps)..=(current + steps).min(AXIS_STATES - 1)
                {
                    values[before * AXIS_STATES + current] = kinematic_change(
                        directions[i - 1][limb][before],
                        directions[i][limb][current],
                        duration,
                    );
                }
            }
            values
        });
        let mut next = vec![PathScore::UNREACHABLE; STATES];
        // The base transition is separable. Nonnegative sparse edge penalties
        // cannot displace its winner unless that particular edge is penalized.
        // Keep the existing right-then-left addition and hierarchical tie order.
        debug_assert!(edges[i].values().all(|penalty| *penalty >= 0.));
        let mut rows = vec![(PathScore::UNREACHABLE, 0usize); STATES];
        for pl in 0..AXIS_STATES {
            for r in 0..AXIS_STATES {
                let slot = pl * AXIS_STATES + r;
                for pr in r.saturating_sub(steps)..=(r + steps).min(AXIS_STATES - 1) {
                    let before = pl * AXIS_STATES + pr;
                    let value = previous[before].add(transitions[1][pr * AXIS_STATES + r]);
                    if value.better_than(rows[slot].0) {
                        rows[slot] = (value, before);
                    }
                }
            }
        }
        for state in 0..STATES {
            let (l, r) = (state / AXIS_STATES, state % AXIS_STATES);
            for pl in l.saturating_sub(steps)..=(l + steps).min(AXIS_STATES - 1) {
                let (row, before) = rows[pl * AXIS_STATES + r];
                let value = row.add(transitions[0][pl * AXIS_STATES + l]);
                if value.better_than(next[state]) {
                    next[state] = value;
                    parents[i][state] = before as u16;
                }
            }
            let selected = parents[i][state] as usize * STATES + state;
            if edges[i].get(&selected).is_some_and(|penalty| *penalty > 0.) {
                next[state] = PathScore::UNREACHABLE;
                parents[i][state] = 0;
                let mut best_tie = (usize::MAX, f64::INFINITY, usize::MAX);
                for pl in l.saturating_sub(steps)..=(l + steps).min(AXIS_STATES - 1) {
                    for pr in r.saturating_sub(steps)..=(r + steps).min(AXIS_STATES - 1) {
                        let before = pl * AXIS_STATES + pr;
                        let row = previous[before].add(transitions[1][pr * AXIS_STATES + r]);
                        let base = row.add(transitions[0][pl * AXIS_STATES + l]);
                        let edge = *edges[i].get(&(before * STATES + state)).unwrap_or(&0.);
                        let value = base.add(edge);
                        let tie = (pl, row.cost, pr);
                        // A large left cost can round distinct row costs to the
                        // same total. Retain the separable row winner in that
                        // case too, without epsilon comparisons or dropped costs.
                        if value.better_than(next[state])
                            || (value.cost.is_finite() && value == next[state] && tie < best_tie)
                        {
                            next[state] = value;
                            parents[i][state] = before as u16;
                            best_tie = tie;
                        }
                    }
                }
            }
            next[state] = next[state].with_node(source_failures[i][state] as u32, costs[i][state]);
        }
        previous = next;
    }
    let mut state = (0..STATES)
        .min_by(|&a, &b| previous[a].compare(previous[b]))
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
    source_times: &[Vec<f64>; 2],
) -> Result<HandPlan, String> {
    check_control_budget(inputs.len())?;
    if inputs.is_empty() {
        return Ok(HandPlan {
            swivels: Vec::new(),
            orientations: Vec::new(),
        });
    }
    let policy = |input: &HandCoordinationInput, limb: usize| {
        input
            .profile
            .hand_frames
            .as_ref()
            .and_then(|frames| frames[limb].palm_roll_range)
    };
    if inputs
        .iter()
        .chain(checks)
        .any(|input| (0..2).any(|limb| policy(input, limb) != policy(&inputs[0], limb)))
    {
        return Err("양손 경로의 그립 정책이 시간에 따라 변경되었습니다.".into());
    }
    let (mut costs, mut orientations, mut directions) = (Vec::new(), Vec::new(), Vec::new());
    let mut source_failures = Vec::new();
    for input in inputs {
        let sources = std::array::from_fn(|limb| {
            let times = &source_times[limb];
            let at = times.partition_point(|time| *time < input.time - 1e-9);
            times
                .get(at)
                .is_some_and(|time| (*time - input.time).abs() < 1e-9)
        });
        let (cost, orientation, direction, failures) = node_costs(input, sources);
        source_failures.push(failures);
        costs.push(cost);
        orientations.push(orientation);
        directions.push(direction);
    }
    let mut ordered_checks: Vec<_> = checks.iter().collect();
    ordered_checks.sort_by(|a, b| a.time.total_cmp(&b.time));
    ordered_checks.dedup_by(|a, b| (a.time - b.time).abs() < 1e-9);
    let mut edges: Vec<HashMap<usize, f64>> = (0..inputs.len()).map(|_| HashMap::new()).collect();
    let mut best = (PathScore::UNREACHABLE, Vec::new());
    for _ in 0..REFINEMENTS {
        let path = graph_path_with_sources(inputs, &costs, &source_failures, &directions, &edges);
        let track: Vec<_> = inputs
            .iter()
            .zip(&path)
            .enumerate()
            .map(|(index, (input, &state))| {
                let angles = angles(state);
                SwivelSample {
                    time: input.time,
                    angles,
                    orientations: std::array::from_fn(|limb| {
                        let axis = if limb == 0 {
                            state / AXIS_STATES
                        } else {
                            state % AXIS_STATES
                        };
                        orientations[index][limb][axis].and_then(|q| {
                            input.profile.encode_hand_orientation(
                                limb,
                                input.tips[limb],
                                input.pitches[limb],
                                angles[limb],
                                q,
                            )
                        })
                    }),
                }
            })
            .collect();
        if track.iter().any(|sample| {
            (0..2).any(|limb| {
                sample.orientations[limb].is_some() != policy(&inputs[0], limb).is_some()
            })
        }) {
            return Err("선택된 양손 경로의 손 방향을 복구할 수 없습니다.".into());
        }
        let mut updates: HashMap<(usize, usize), f64> = HashMap::new();
        let mut score = 0.0;
        let mut previous: Option<(f64, [Option<HandKinematics>; 2])> = None;
        for input in &ordered_checks {
            let (value, current_directions) =
                pose_cost_and_kinematics(input, at(&track, input.time));
            // Measure actual tool, wrist and elbow motion inside the interval too:
            // the independently moving target changes the IK reference frame.
            // This is a cost, not a claim that endpoint bounds certify speed.
            let rate = previous.map_or(0., |(time, directions)| {
                (0..2)
                    .map(|limb| {
                        kinematic_change(
                            directions[limb],
                            current_directions[limb],
                            input.time - time,
                        )
                    })
                    .sum::<f64>()
            });
            previous = Some((input.time, current_directions));
            score += value + rate;
            // Actual pose rates rank every inspected path. Only failed
            // poses need coupled edge refinement: rate-only entries would
            // disable the separable transition search across the whole song.
            if value < 1. {
                continue;
            }
            let penalty = value + rate;
            let i = inputs
                .partition_point(|p| p.time < input.time)
                .min(inputs.len() - 1);
            if i == 0 {
                continue;
            }
            let key = path[i - 1] * STATES + path[i];
            let old = updates.entry((i, key)).or_insert(0.0);
            *old = old.max(penalty);
        }
        let score = PathScore::new(
            path.iter()
                .enumerate()
                .map(|(i, &state)| source_failures[i][state] as u32)
                .sum(),
            score,
        );
        if score.better_than(best.0) {
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
    // Keep exactly the original prepared controls. Inserting a pointwise
    // witness near an impact can require a large rotation in a few milliseconds.
    // Unresolved interiors stay diagnosed; they cannot buy an unsafe new anchor.
    let orientations = best
        .1
        .iter()
        .map(|s| OrientationSample {
            time: s.time,
            orientations: s.orientations,
        })
        .collect();
    Ok(HandPlan {
        swivels: best.1,
        orientations,
    })
}
#[cfg(test)]
fn plan(inputs: &[HandCoordinationInput]) -> Vec<SwivelSample> {
    plan_with_checks(inputs, inputs, &[Vec::new(), Vec::new()])
        .unwrap()
        .swivels
}
pub fn at(track: &[SwivelSample], time: f64) -> SwivelSample {
    if track.is_empty() {
        return SwivelSample {
            time,
            angles: [0.; 2],
            orientations: [None; 2],
        };
    }
    let j = track.partition_point(|s| s.time <= time);
    if j == 0 {
        return track[0];
    }
    if j == track.len() {
        return track[j - 1];
    }
    let (a, b) = (track[j - 1], track[j]);
    let t = ((time - a.time) / (b.time - a.time).max(1e-9)).clamp(0.0, 1.0);
    let weight = t * t * (3.0 - 2.0 * t);
    SwivelSample {
        time,
        angles: std::array::from_fn(|i| a.angles[i] + (b.angles[i] - a.angles[i]) * weight),
        // Construction and prepared-message validation require one policy for
        // each hand across the complete track, including mixed legacy rigs.
        orientations: std::array::from_fn(|i| {
            a.orientations[i]
                .zip(b.orientations[i])
                .map(|(a, b)| interpolate_control(a, b, weight))
        }),
    }
}

/// Keep the original C1 swivel curve while refining only its tool witnesses.
/// Inserting ordinary swivel knots would change smoothstep timing and could
/// freeze the 0.05-radian graph on intervals shorter than 25 ms.
pub fn at_with_orientations(
    swivels: &[SwivelSample],
    orientations: &[OrientationSample],
    time: f64,
) -> SwivelSample {
    let mut control = at(swivels, time);
    if orientations.is_empty() {
        return control;
    }
    let j = orientations.partition_point(|sample| sample.time <= time);
    control.orientations = if j == 0 {
        orientations[0].orientations
    } else if j == orientations.len() {
        orientations[j - 1].orientations
    } else {
        let (a, b) = (orientations[j - 1], orientations[j]);
        let t = ((time - a.time) / (b.time - a.time).max(1e-9)).clamp(0., 1.);
        let weight = t * t * (3. - 2. * t);
        std::array::from_fn(|limb| {
            a.orientations[limb]
                .zip(b.orientations[limb])
                .map(|(a, b)| interpolate_control(a, b, weight))
        })
    };
    control
}

#[cfg(test)]
mod tests {
    fn control(arm: Quat) -> OrientationControl {
        arm
    }
    use super::*;
    use crate::rig::HandControlFrame;

    #[test]
    fn close_actual_orientation_endpoints_do_not_accumulate_a_full_turn() {
        // Native interpretation RH, 4.7 -> 4.749995 s. Independently choosing
        // shortest arm/grip arcs used to travel 325.7 degrees although these
        // actual residual endpoints differ by only 34.6 degrees.
        let rotations = [
            [
                -0.09213349497012681,
                -0.054277303208593405,
                0.5075150455348098,
                0.854981796307262,
            ],
            [
                -0.11299043790723766,
                -0.055385985696809845,
                0.7380962770982836,
                0.662857027769622,
            ],
        ];
        let track = [4.7, 4.749995].map(|time| SwivelSample {
            time,
            angles: [0.; 2],
            orientations: [None, Some(rotations[usize::from(time > 4.7)])],
        });
        let angle = |a, b| 2. * rotation_dot(a, b).abs().min(1.).acos();
        let shortest = angle(rotations[0], rotations[1]);
        assert!((shortest.to_degrees() - 34.6072189335).abs() < 1e-8);
        let mut previous = rotations[0];
        let mut travelled = 0.;
        for step in 1..=100 {
            let time = 4.7 + (4.749995 - 4.7) * step as f64 / 100.;
            let current = at(&track, time).orientations[1].unwrap();
            let delta = angle(previous, current);
            // The existing smoothstep parameter has maximum slope 1.5.
            assert!(
                delta <= 1.5 * shortest / 100. + 1e-10,
                "unexpected orientation loop: {delta}"
            );
            travelled += delta;
            previous = current;
        }
        assert!((travelled - shortest).abs() < 1e-8);
        let center = at(&track, 4.726662);
        for time in [4.749995, 4.700001, 4.74, 4.7] {
            at(&track, time);
        }
        assert_eq!(center, at(&track, 4.726662));
        let mut negative = track;
        negative[1].orientations[1] = Some(rotations[1].map(|v| -v));
        assert!(
            rotation_dot(
                center.orientations[1].unwrap(),
                at(&negative, 4.726662).orientations[1].unwrap()
            )
            .abs()
                > 1. - 1e-12
        );
    }

    #[test]
    fn actual_tool_and_wrist_motion_costs_do_not_depend_on_residual_controls() {
        let a = HandKinematics {
            direction: Some([0., -1., 0.]),
            rotation: [0., 0., 0., 1.],
            wrist: [0., 1., 0.5],
        };
        let b = HandKinematics {
            rotation: [0., (0.2_f64).sin(), 0., (0.2_f64).cos()],
            wrist: [0.05, 1., 0.5],
            ..a
        };
        let cost = kinematic_change(Some(a), Some(b), 0.1);
        assert!(cost > 0.);
        assert!((cost - 10. * kinematic_change(Some(a), Some(b), 1.)).abs() < 1e-12);
        assert_eq!(kinematic_change(Some(a), Some(a), 0.01), 0.);
        let opposite_sign = HandKinematics {
            rotation: b.rotation.map(|v| -v),
            ..b
        };
        assert_eq!(cost, kinematic_change(Some(a), Some(opposite_sign), 0.1));
        assert!(
            kinematic_change(
                Some(a),
                Some(HandKinematics {
                    rotation: a.rotation,
                    ..b
                }),
                0.1
            ) > 0.
        );
        assert!(
            kinematic_change(
                Some(a),
                Some(HandKinematics {
                    wrist: a.wrist,
                    ..b
                }),
                0.1
            ) > 0.
        );
    }
    #[test]
    fn orientation_only_knots_do_not_retime_the_original_swivel_curve() {
        let base = [
            SwivelSample {
                time: 0.,
                angles: [0., -0.2],
                orientations: [Some(control([0., 0., 0., 1.])), None],
            },
            SwivelSample {
                time: 0.1,
                angles: [0.2, 0.],
                orientations: [Some(control([0., 0., 0., 1.])), None],
            },
        ];
        let refined = [
            OrientationSample {
                time: 0.,
                orientations: base[0].orientations,
            },
            OrientationSample {
                time: 0.011,
                orientations: [
                    Some(control([0., (0.1_f64).sin(), 0., (0.1_f64).cos()])),
                    None,
                ],
            },
            OrientationSample {
                time: 0.1,
                orientations: base[1].orientations,
            },
        ];
        for time in [0., 0.005, 0.011, 0.033, 0.05, 0.099, 0.1, 0.033] {
            assert_eq!(
                at_with_orientations(&base, &refined, time).angles,
                at(&base, time).angles
            );
            assert_eq!(
                at_with_orientations(&base, &refined, time).orientations[1],
                None
            );
        }
        assert_eq!(
            at_with_orientations(&base, &refined, 0.011).orientations,
            refined[1].orientations
        );
        for sample in base {
            assert_eq!(at_with_orientations(&base, &refined, sample.time), sample);
        }
    }

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
            palm_roll_range: None,
            palm_reference: None,
        };
        p.hand_frames = Some([frame.clone(), frame]);
        p
    }
    #[test]
    fn bounded_orientations_are_selected_once_and_interpolated_with_legacy_hands() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        let mut p: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        p.hand_frames.as_mut().unwrap()[1].palm_roll_range = None;
        let inputs: Vec<_> = [0., 0.1, 0.2]
            .into_iter()
            .map(|time| HandCoordinationInput {
                cue_orientation: None,
                time,
                profile: p.clone(),
                tips: [[-0.35 + time * 0.01, 1.25, 0.7], [0.35, 1.25, 0.7]],
                pitches: [-0.5; 2],
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
            })
            .collect();
        let track = plan(&inputs);
        for (input, sample) in inputs.iter().zip(&track) {
            assert!(sample.orientations[0].is_some());
            assert_eq!(sample.orientations[1], None);
            let optimized = p
                .solve_hand_with_swivel(0, input.tips[0], -0.5, sample.angles[0])
                .unwrap();
            assert_eq!(
                sample.orientations[0],
                p.encode_hand_orientation(
                    0,
                    input.tips[0],
                    -0.5,
                    sample.angles[0],
                    optimized.rotation
                )
            );
            let replay = p
                .solve_hand_with_swivel_and_orientation(
                    0,
                    input.tips[0],
                    -0.5,
                    sample.angles[0],
                    sample.orientations[0],
                )
                .unwrap();
            assert!(crate::motion::distance(optimized.middle, replay.middle) < 1e-9);
            assert!(crate::motion::distance(optimized.tip, replay.tip) < 1e-9);
        }
        for pair in track.windows(2) {
            let middle = at(&track, (pair[0].time + pair[1].time) * 0.5);
            let expected = interpolate_control(
                pair[0].orientations[0].unwrap(),
                pair[1].orientations[0].unwrap(),
                0.5,
            );
            let actual = middle.orientations[0].unwrap();
            assert!(rotation_dot(actual, expected).abs() > 1. - 1e-10);
            assert_eq!(middle.orientations[1], None);
        }
        for time in [0.173, 0.013, 0.1, 0.173] {
            let control = at(&track, time);
            let tip = [-0.35 + time * 0.01, 1.25, 0.7];
            let hand = p
                .solve_hand_with_swivel_and_orientation(
                    0,
                    tip,
                    -0.5,
                    control.angles[0],
                    control.orientations[0],
                )
                .unwrap();
            let later = at(&track, time + 1e-6);
            let next = p
                .solve_hand_with_swivel_and_orientation(
                    0,
                    [-0.35 + (time + 1e-6) * 0.01, 1.25, 0.7],
                    -0.5,
                    later.angles[0],
                    later.orientations[0],
                )
                .unwrap();
            assert!(crate::motion::distance(hand.middle, next.middle) < 1e-4);
        }
        let mut changed = inputs;
        changed[1].profile.hand_frames.as_mut().unwrap()[0].palm_roll_range = None;
        assert!(plan_with_checks(&changed, &changed, &[Vec::new(), Vec::new()]).is_err());
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
                    cue_orientation: None,
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
            baseline += pose_cost(
                input,
                SwivelSample {
                    time: input.time,
                    angles: [0.; 2],
                    orientations: [None; 2],
                },
            );
            corrected += pose_cost(input, *sample);
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
                assert!((b.angles[j] - a.angles[j]).abs() <= 3.0e-5 + 1e-9);
            }
            assert_eq!(a, at(&track, t));
        }
    }
    #[test]
    fn unreachable_contact_cannot_beat_two_reachable_posture_misses() {
        let mut p = profile();
        p.lengths[0] = [0.3, 0.27];
        let target = [-0.22, 1.65, 0.3];
        let mut h = p.solve_hand_with_swivel(0, target, 0., 0.).unwrap();
        assert!(h.reachable);
        h.rotation = [0., 0., 0., 1.];
        assert!(!crate::posture::anatomical(&p, 0, target, &h));
        let reachable_cost = posture_cost(&p, 0, target, &h);
        h.reachable = false;
        assert!(posture_cost(&p, 0, target, &h) > 2. * reachable_cost);
    }
    #[test]
    fn infeasible_swivels_choose_lower_anatomical_error_without_passing_the_gate() {
        let mut p = profile();
        p.lengths[0] = [0.3, 0.27];
        let target = [-0.22, 1.65, 0.3];
        let hands: Vec<_> = (0..AXIS_STATES)
            .map(|i| {
                p.solve_hand_with_swivel(0, target, 0., i as f64 * ANGLE_STEP - MAX_SWIVEL)
                    .unwrap()
            })
            .collect();
        assert!(hands
            .iter()
            .all(|h| h.reachable && !crate::posture::anatomical(&p, 0, target, h)));
        let best = hands
            .iter()
            .min_by(|a, b| {
                posture_cost(&p, 0, target, a).total_cmp(&posture_cost(&p, 0, target, b))
            })
            .unwrap();
        let worst = hands
            .iter()
            .max_by(|a, b| {
                crate::posture::anatomical_violation(&p, 0, target, a)
                    .total_cmp(&crate::posture::anatomical_violation(&p, 0, target, b))
            })
            .unwrap();
        assert!(
            crate::posture::anatomical_violation(&p, 0, target, best)
                < crate::posture::anatomical_violation(&p, 0, target, worst) * 0.9
        );
        assert!(!crate::posture::anatomical(&p, 0, target, best));
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
                cue_orientation: None,
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
        let directions = vec![[vec![None; AXIS_STATES], vec![None; AXIS_STATES]]; inputs.len()];
        let path = graph_path(&inputs, &costs, &directions, &edges);
        assert_eq!(path, vec![neutral, prepare, future]);
        // Greedy neutral at the middle knot cannot reach the final safe state
        // under the unchanged 2 rad/s sample bound.
        assert!((angles(neutral)[0] - angles(future)[0]).abs() > MAX_SAMPLE_RATE * STEP);
    }
    #[test]
    fn source_feasibility_precedes_finite_cost_without_relaxing_rate_or_losing_fallback() {
        let p = profile();
        let inputs: Vec<_> = (0..3)
            .map(|i| HandCoordinationInput {
                cue_orientation: None,
                time: i as f64 * STEP,
                profile: p.clone(),
                tips: [[-0.25, 1., 0.5], [0.25, 1., 0.5]],
                pitches: [0.; 2],
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
            })
            .collect();
        let center = (AXIS_STATES - 1) / 2;
        let neutral = center * AXIS_STATES + center;
        let prepare = (center - 4) * AXIS_STATES + center;
        let impact = (center - 8) * AXIS_STATES + center;
        let mut costs = vec![vec![f64::INFINITY; STATES]; 3];
        costs[0][neutral] = 0.;
        costs[1][neutral] = 0.;
        costs[1][prepare] = 1e12;
        costs[2][neutral] = 0.;
        costs[2][impact] = 1e12;
        let edges = (0..3).map(|_| HashMap::new()).collect::<Vec<_>>();
        let directions = vec![[vec![None; AXIS_STATES], vec![None; AXIS_STATES]]; 3];
        let ordinary = graph_path(&inputs, &costs, &directions, &edges);
        assert_eq!(ordinary, vec![neutral; 3]);
        let mut failures = vec![vec![0; STATES]; 3];
        failures[2].fill(1);
        failures[2][impact] = 0;
        let path = graph_path_with_sources(&inputs, &costs, &failures, &directions, &edges);
        assert_eq!(path, vec![neutral, prepare, impact]);
        for pair in path.windows(2) {
            assert!(
                (angles(pair[1])[0] - angles(pair[0])[0]).abs() <= MAX_SAMPLE_RATE * STEP + 1e-9
            );
        }
        // A source-clear graph retains the exact existing objective and ties.
        failures[2][neutral] = 0;
        assert_eq!(
            graph_path_with_sources(&inputs, &costs, &failures, &directions, &edges),
            ordinary
        );
        // If the only clear impact lies outside the unchanged transition bound,
        // retain the least-cost unresolved path instead of dropping its source.
        failures[2][neutral] = 1;
        costs[1][prepare] = f64::INFINITY;
        assert_eq!(
            graph_path_with_sources(&inputs, &costs, &failures, &directions, &edges),
            ordinary
        );
    }
    #[test]
    fn simultaneous_source_tools_require_a_joint_witness_but_recovery_is_not_locked() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-natural-grip30.json")).unwrap();
        let p: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        let input = HandCoordinationInput {
            time: 0.,
            cue_orientation: None,
            profile: p,
            tips: [[-0.0756, 0.825, 0.631]; 2],
            pitches: [0.; 2],
            equipment: None,
            intended_stick_contact: false,
            close_tool_approach: false,
            intended_pieces: [None, None],
        };
        // Both exact tips occupy the same point. No pair of shaft witnesses
        // can realize two simultaneous real hits with positive tool clearance.
        let (_, _, _, simultaneous) = node_costs(&input, [true, true]);
        assert!(simultaneous.iter().all(|failures| *failures > 0));
        // If only one hand owns an immutable impact, the other may be repaired
        // by the subsequent phase curve. Do not freeze its temporary overlap.
        let (_, _, _, one_source) = node_costs(&input, [true, false]);
        assert!(one_source.contains(&0));
    }
    /// Independent joint-neighbor reference using the existing separable
    /// floating-point objective and its right-row-then-left tie order.
    fn exhaustive_graph_path(
        inputs: &[HandCoordinationInput],
        costs: &[Vec<f64>],
        directions: &[NodeKinematics],
        edges: &[HashMap<usize, f64>],
    ) -> Vec<usize> {
        let mut previous = costs[0].clone();
        let mut parents = vec![vec![0usize; STATES]; inputs.len()];
        for i in 1..inputs.len() {
            let duration = inputs[i].time - inputs[i - 1].time;
            let steps = (((MAX_SAMPLE_RATE * duration + 1e-9) / ANGLE_STEP).floor() as usize)
                .min(AXIS_STATES - 1);
            let transition: [Vec<_>; 2] = std::array::from_fn(|limb| {
                (0..STATES)
                    .map(|pair| {
                        kinematic_change(
                            directions[i - 1][limb][pair / AXIS_STATES],
                            directions[i][limb][pair % AXIS_STATES],
                            duration,
                        )
                    })
                    .collect()
            });
            let mut next = vec![f64::INFINITY; STATES];
            for state in 0..STATES {
                let (left, right) = (state / AXIS_STATES, state % AXIS_STATES);
                let mut best = (f64::INFINITY, usize::MAX, f64::INFINITY, usize::MAX);
                for (before, &prior) in previous.iter().enumerate() {
                    let (pl, pr) = (before / AXIS_STATES, before % AXIS_STATES);
                    if pl.abs_diff(left) > steps || pr.abs_diff(right) > steps {
                        continue;
                    }
                    let row = prior + transition[1][pr * AXIS_STATES + right];
                    let value = (row + transition[0][pl * AXIS_STATES + left])
                        + edges[i]
                            .get(&(before * STATES + state))
                            .copied()
                            .unwrap_or(0.);
                    let candidate = (value, pl, row, pr);
                    if value.is_finite() && candidate < best {
                        best = candidate;
                        parents[i][state] = before;
                    }
                }
                next[state] = best.0 + costs[i][state];
            }
            previous = next;
        }
        let mut state = (0..STATES)
            .min_by(|&a, &b| previous[a].total_cmp(&previous[b]))
            .unwrap();
        let mut result = vec![0; inputs.len()];
        for i in (0..inputs.len()).rev() {
            result[i] = state;
            state = parents[i][state];
        }
        result
    }
    #[test]
    fn sparse_nonnegative_edges_match_exhaustive_graph_with_ties_and_boundaries() {
        let p = profile();
        // Includes no allowed swivel step, an exact one-step interval, and the
        // full 33-state boundary. Every destination is checked by the reference.
        let inputs: Vec<_> = [0., 0.001, 0.026, 0.126, 0.926]
            .into_iter()
            .map(|time| HandCoordinationInput {
                cue_orientation: None,
                time,
                profile: p.clone(),
                tips: [[-0.25, 1., 0.5], [0.25, 1., 0.5]],
                pitches: [0.; 2],
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
            })
            .collect();
        let mut random = 0x7189_9ab5_3492_1937u64;
        let mut draw = || {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            random
        };
        for trial in 0..32 {
            let directions = vec![[vec![None; AXIS_STATES], vec![None; AXIS_STATES]]; inputs.len()];
            let mut costs: Vec<Vec<_>> = (0..inputs.len())
                .map(|_| {
                    (0..STATES)
                        .map(|_| {
                            if trial % 4 == 0 {
                                0.
                            } else {
                                (draw() % 1024) as f64 / 16.
                            }
                        })
                        .collect()
                })
                .collect();
            // Force extrema as well as random/tied interior minima.
            if trial % 4 == 1 {
                costs[0].fill(1000.);
                costs[0][0] = 0.;
                costs[4].fill(1000.);
                costs[4][STATES - 1] = 0.;
            }
            let mut edges = vec![HashMap::new(); inputs.len()];
            // Penalize actual current winners over several passes so fallback
            // must skip more than one predecessor, not only find a second best.
            for pass in 0..4 {
                let path = graph_path(&inputs, &costs, &directions, &edges);
                for i in 1..inputs.len() {
                    edges[i].insert(path[i - 1] * STATES + path[i], 8. + pass as f64);
                    for _ in 0..4 {
                        let before = draw() as usize % STATES;
                        let state = draw() as usize % STATES;
                        edges[i].insert(before * STATES + state, (draw() % 32) as f64 / 8.);
                    }
                }
            }
            assert_eq!(
                graph_path(&inputs, &costs, &directions, &edges),
                exhaustive_graph_path(&inputs, &costs, &directions, &edges),
                "trial {trial}"
            );
            for interval in &mut edges {
                for penalty in interval.values_mut() {
                    *penalty = 0.;
                }
            }
            assert_eq!(
                graph_path(&inputs, &costs, &directions, &edges),
                exhaustive_graph_path(&inputs, &costs, &directions, &edges),
                "zero penalties {trial}"
            );
        }
    }
    #[test]
    fn sparse_edge_repair_keeps_row_ties_when_left_addition_rounds_them_equal() {
        let p = profile();
        let inputs: Vec<_> = [0., 1.]
            .into_iter()
            .map(|time| HandCoordinationInput {
                cue_orientation: None,
                time,
                profile: p.clone(),
                tips: [[-0.25, 1., 0.5], [0.25, 1., 0.5]],
                pitches: [0.; 2],
                equipment: None,
                intended_stick_contact: false,
                close_tool_approach: false,
                intended_pieces: [None, None],
            })
            .collect();
        let kinematics = |wrist| {
            Some(HandKinematics {
                direction: None,
                rotation: [0., 0., 0., 1.],
                wrist,
            })
        };
        let mut directions = vec![[vec![None; AXIS_STATES], vec![None; AXIS_STATES]]; 2];
        directions[0][0].fill(kinematics([0.; 3]));
        directions[1][0].fill(kinematics([1e8, 0., 0.]));
        let mut costs = vec![vec![1e20; STATES]; 2];
        costs[0][0] = 0.00001;
        costs[0][1] = 0.;
        costs[1][0] = 0.;
        let mut edges = vec![HashMap::new(); 2];
        assert_eq!(graph_path(&inputs, &costs, &directions, &edges), vec![1, 0]);
        assert_eq!(
            graph_path(&inputs, &costs, &directions, &edges),
            exhaustive_graph_path(&inputs, &costs, &directions, &edges)
        );
        // The zero edge keeps the faster path and its exact row tie semantics.
        edges[1].insert(STATES, 0.);
        assert_eq!(graph_path(&inputs, &costs, &directions, &edges), vec![1, 0]);
        edges[1].insert(STATES, 1.);
        assert_eq!(graph_path(&inputs, &costs, &directions, &edges), vec![0, 0]);
        assert_eq!(
            graph_path(&inputs, &costs, &directions, &edges),
            exhaustive_graph_path(&inputs, &costs, &directions, &edges)
        );
    }
    #[test]
    fn separable_transition_preserves_bounded_graph_optimum() {
        let p = profile();
        let inputs: Vec<_> = [0.0, 0.037, 0.13, 0.21]
            .into_iter()
            .map(|time| HandCoordinationInput {
                cue_orientation: None,
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
        let orientations: Vec<NodeOrientations> = (0..inputs.len())
            .map(|i| {
                std::array::from_fn(|limb| {
                    (0..AXIS_STATES)
                        .map(|j| {
                            let angle = ((i * 7 + j * 11 + limb * 3) as f64 * 0.1).sin();
                            Some([0., (angle * 0.5).sin(), 0., (angle * 0.5).cos()])
                        })
                        .collect()
                })
            })
            .collect();
        let fast = vec![HashMap::new(); inputs.len()];
        let directions: Vec<NodeKinematics> = (0..inputs.len())
            .map(|i| {
                std::array::from_fn(|limb| {
                    (0..AXIS_STATES)
                        .map(|j| {
                            let angle = (i * 5 + j * 3 + limb * 7) as f64 * 0.05;
                            Some(HandKinematics {
                                direction: Some([angle.sin(), -angle.cos(), 0.]),
                                rotation: orientations[i][limb][j].unwrap(),
                                wrist: [angle * 0.02, 0., 0.],
                            })
                        })
                        .collect()
                })
            })
            .collect();
        let mut penalties = fast.clone();
        for _ in 0..3 {
            let path = graph_path(&inputs, &costs, &directions, &penalties);
            for i in 1..inputs.len() {
                penalties[i].insert(path[i - 1] * STATES + path[i], 4.);
            }
            assert_eq!(
                graph_path(&inputs, &costs, &directions, &penalties),
                exhaustive_graph_path(&inputs, &costs, &directions, &penalties)
            );
        }
        assert_eq!(
            graph_path(&inputs, &costs, &directions, &fast),
            exhaustive_graph_path(&inputs, &costs, &directions, &fast)
        );
    }
    #[test]
    fn actual_direction_cost_accounts_for_duration_and_body_facing() {
        let (a, b) = (Some([0., -1., 0.]), Some([0.6, -0.8, 0.]));
        assert!(direction_change(a, b, 0.1) > 0.);
        assert!((direction_change(a, b, 0.1) - 10. * direction_change(a, b, 1.)).abs() < 1e-12);
        assert_eq!(direction_change(a, a, 0.01), 0.);
        let mut p = profile();
        p.torso_pivot = Some(p.chest);
        let hand = p
            .solve_hand_with_swivel(0, [-0.25, 1., 0.5], 0., 0.2)
            .unwrap();
        let direction = elbow_direction(&p, 0, &hand).unwrap();
        let yaw: f64 = 0.7;
        let q = [0., (yaw * 0.5).sin(), 0., (yaw * 0.5).cos()];
        let turned = p.turned(yaw);
        let mut rotated = hand;
        rotated.middle = crate::motion::add(
            turned.roots[0],
            crate::rig::rotate(q, crate::motion::sub(hand.middle, p.roots[0])),
        );
        assert!(
            crate::motion::distance(direction, elbow_direction(&turned, 0, &rotated).unwrap())
                < 1e-9
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
            cue_orientation: None,
            time: -0.5,
            profile: p.clone(),
            tips: crate::posture::ready_pair(&p, snare, hat),
            pitches: [0.0; 2],
            equipment: None,
            intended_stick_contact: false,
            close_tool_approach: false,
            intended_pieces: [None, None],
        };
        let (costs, _, _, _) = node_costs(&input, [false; 2]);
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
