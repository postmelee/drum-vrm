//! Deterministic elbow-plane coordination over an already planned contact path.
//! Conservative proxies guide the candidate; this is not a mesh collision solver.
use crate::{
    clearance,
    model::Vec3,
    rig::{HandSolution, RigProfile},
};

pub const STEP: f64 = 1.0 / 30.0;
pub const MAX_SWIVEL: f64 = 0.8;
// Smoothstep's peak derivative is 1.5, so interpolation stays below 3 rad/s.
pub const MAX_SAMPLE_RATE: f64 = 2.0;

pub struct HandCoordinationInput {
    pub time: f64,
    pub profile: RigProfile,
    pub tips: [Vec3; 2],
    pub pitches: [f64; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwivelSample {
    pub time: f64,
    pub angles: [f64; 2],
}

fn deficit(clearance: f64, margin: f64) -> f64 {
    (margin - clearance).max(0.0).powi(2)
}
fn pair_cost(profile: &RigProfile, left: &HandSolution, right: &HandSolution) -> f64 {
    let mut arm: f64 = 0.0;
    for (a, b) in [(profile.roots[0], left.middle), (left.middle, left.end)] {
        for (c, d) in [(profile.roots[1], right.middle), (right.middle, right.end)] {
            arm = arm.max(deficit(clearance::segment_distance(a, b, c, d), 0.042));
        }
    }
    let stick = deficit(
        clearance::segment_distance(left.end, left.tip, right.end, right.tip),
        0.014,
    );
    arm * 2000.0 + stick * 2000.0
}

fn candidate(
    input: &HandCoordinationInput,
    current: [f64; 2],
    previous: Option<SwivelSample>,
    next: Option<SwivelSample>,
) -> [f64; 2] {
    let mut options: [Vec<(f64, HandSolution, f64)>; 2] = std::array::from_fn(|_| Vec::new());
    for (limb, states) in options.iter_mut().enumerate() {
        let mut low = -MAX_SWIVEL;
        let mut high = MAX_SWIVEL;
        for neighbor in [previous, next].into_iter().flatten() {
            let change = MAX_SAMPLE_RATE * (neighbor.time - input.time).abs();
            low = low.max(neighbor.angles[limb] - change);
            high = high.min(neighbor.angles[limb] + change);
        }
        // The incoming track already satisfies the rate bound. Numerical
        // roundoff at coincident samples must not make an invalid clamp range.
        high = high.max(low);
        for angle in [
            low,
            high,
            (low + high) * 0.5,
            current[limb].clamp(low, high),
            0.0_f64.clamp(low, high),
        ] {
            if states.iter().any(|(a, _, _)| (a - angle).abs() < 1e-10) {
                continue;
            }
            if let Some(hand) = input.profile.solve_hand_with_swivel(
                limb,
                input.tips[limb],
                input.pitches[limb],
                angle,
            ) {
                let torso = deficit(
                    clearance::hand_clearance(&input.profile, limb, &hand),
                    0.002,
                );
                let mut cost = torso * 2500.0 + angle * angle * 0.03;
                for neighbor in [previous, next].into_iter().flatten() {
                    cost += (angle - neighbor.angles[limb]).powi(2) * 0.1;
                }
                states.push((angle, hand, cost));
            }
        }
    }
    let mut best = (f64::INFINITY, current);
    for (a, left, left_cost) in &options[0] {
        for (b, right, right_cost) in &options[1] {
            let cost = left_cost + right_cost + pair_cost(&input.profile, left, right);
            if cost < best.0 - 1e-12 {
                best = (cost, [*a, *b]);
            }
        }
    }
    best.1
}

/// Include contact timestamps as well as the uniform grid. The caller supplies
/// already turned profiles, preserving exactly the same torso/target contract.
/// Forward/backward sweeps anticipate nearby conflicts without render history.
pub fn plan(inputs: &[HandCoordinationInput]) -> Vec<SwivelSample> {
    let mut track: Vec<_> = inputs
        .iter()
        .map(|f| SwivelSample {
            time: f.time,
            angles: [0.0; 2],
        })
        .collect();
    if inputs.len() < 2 {
        return track;
    }
    // First construct a feasible rate-bounded track; reverse/forward refinement
    // can move an avoidance gesture earlier while respecting both neighbors.
    for i in 1..inputs.len() {
        track[i].angles = candidate(&inputs[i], track[i - 1].angles, Some(track[i - 1]), None);
    }
    for reverse in [true, false, true, false] {
        for index in 1..inputs.len() {
            let i = if reverse { inputs.len() - index } else { index };
            let next = track.get(i + 1).copied();
            track[i].angles = candidate(&inputs[i], track[i].angles, Some(track[i - 1]), next);
        }
    }
    track
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
    let a = track[j - 1];
    let b = track[j];
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
        let mut times: Vec<f64> = (0..=90).map(|i| i as f64 * STEP).collect();
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
}
