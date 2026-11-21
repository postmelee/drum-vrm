use super::*;

fn profile() -> RigProfile {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
    let mut p: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    for frame in p.hand_frames.as_mut().unwrap() {
        frame.palm_roll_range = None;
    }
    p
}
fn input(t: f64) -> HandCoordinationInput {
    HandCoordinationInput {
        time: t,
        profile: profile(),
        tips: [
            [-0.13 + 0.02 * t, 1.03 + 0.01 * t * t, 0.63],
            [0.13, 1.03, 0.63],
        ],
        pitches: [0.2 + if t < 0. { 0.15 * t } else { -0.08 * t }, 0.2],
        cue_orientation: None,
        equipment: None,
        intended_stick_contact: false,
        close_tool_approach: false,
        intended_pieces: [None, None],
    }
}
fn controls() -> Vec<SwivelSample> {
    [-1., 0., 1., 2.]
        .map(|time| SwivelSample {
            time,
            angles: [0.1; 2],
            orientations: [None; 2],
        })
        .to_vec()
}
fn segment(limb: usize, start: f64, end: f64, source: bool) -> Segment {
    let controls = controls();
    Segment {
        limb,
        start,
        end,
        from: key_at(start, limb, source, 1., &controls, &[], &input).unwrap(),
        to: key_at(end, limb, source, -1., &controls, &[], &input).unwrap(),
        parameters: [0.03, -0.01, 0.02, 0.01, 0.01, -0.01, 0.02, 0.],
    }
}
#[test]
fn independent_overlapping_tracks_replay_exact_tip_and_fixed_bones() {
    let controls = controls();
    let segments = vec![segment(0, 0., 1., true), segment(1, 0.25, 0.75, true)];
    for t in [0., 0.25, 0.6, 1., 0.7, 0.6] {
        let s = sample(t, &controls, &[], &input);
        for limb in 0..2 {
            let actual = at(
                &segments,
                &s.input.profile,
                limb,
                t,
                s.angles[limb],
                s.targets[limb],
            );
            if limb == 1 && !(0.25..=0.75).contains(&t) {
                assert!(actual.is_none());
                continue;
            }
            let h = actual.unwrap();
            assert!(distance(h.tip, s.hands[limb].tip) < 1e-12);
            assert!(
                (distance(s.input.profile.roots[limb], h.middle)
                    - s.input.profile.lengths[limb][0])
                    .abs()
                    < 1e-12
            );
            assert!((distance(h.middle, h.end) - s.input.profile.lengths[limb][1]).abs() < 1e-12);
            if t == segments[limb].start || t == segments[limb].end {
                assert!(distance(h.middle, s.hands[limb].middle) < 1e-11);
                assert!(distance(h.end, s.hands[limb].end) < 1e-11);
            }
        }
    }
}
#[test]
fn ordinary_join_preserves_original_inside_derivative_at_piecewise_pitch_knot() {
    let controls = controls();
    let seg = segment(0, 0., 1., false);
    let epsilon = 1e-6;
    let zero = sample(0., &controls, &[], &input);

    let right = sample(epsilon, &controls, &[], &input);
    let curve = seg
        .at(
            &right.input.profile,
            epsilon,
            right.angles[0],
            right.hands[0].tip,
        )
        .unwrap();
    let left_rate = mul(sub(right.hands[0].end, zero.hands[0].end), 1. / epsilon);
    let curve_rate = mul(sub(curve.end, zero.hands[0].end), 1. / epsilon);
    assert!(
        distance(left_rate, curve_rate) < 1e-3,
        "left {left_rate:?}, curve {curve_rate:?}"
    );
    let left_elbow = mul(
        sub(right.hands[0].middle, zero.hands[0].middle),
        1. / epsilon,
    );
    let curve_elbow = mul(sub(curve.middle, zero.hands[0].middle), 1. / epsilon);
    assert!(distance(left_elbow, curve_elbow) < 1e-3);
}
#[test]
fn source_basis_preserves_impact_pose_but_allows_feasible_departure_tangent() {
    let epsilon = 1e-6;
    assert_eq!(correction_weight(0., true, false), 0.);
    assert_eq!(correction_weight(1., true, false), 0.);
    assert!(correction_weight(epsilon, true, false) / epsilon > 1.);
    assert!(correction_weight(epsilon, false, false) / epsilon < 0.0001);
    assert!(correction_weight(1. - epsilon, true, false) / epsilon < 0.0001);
}
#[test]
fn overlapping_preparation_cannot_hide_partial_cue_exit() {
    let cue_input = |t: f64| {
        let mut x = input(t);
        if (0.2..=0.8).contains(&t) {
            let weight = if t < 0.3 {
                (t - 0.2) / 0.1
            } else if t <= 0.6 {
                1.
            } else {
                (0.8 - t) / 0.2
            };
            x.cue_orientation = Some(crate::count_in::CueOrientation {
                rotations: [
                    x.profile.contact_rotations[0],
                    x.profile.contact_rotations[1],
                ],
                swivels: [0.1; 2],
                weight,
            });
        }
        x
    };
    let spans = [vec![[0.2, 0.3], [0.4, 0.9], [0.6, 0.8]], vec![]];
    let (available, _) = available_spans(&spans[0], &cue_input);
    assert!(available.contains(&[0.6, 0.9]), "{available:?}");
    assert!(!available.iter().any(|s| s[0] < 0.6 && s[1] > 0.3));
}

#[test]
fn ordinary_phase_ends_align_to_safe_controls_without_uncovered_tails() {
    let controls: Vec<_> = (0..=10)
        .map(|i| SwivelSample {
            time: i as f64 / 10.,
            angles: [0.1; 2],
            orientations: [None; 2],
        })
        .collect();
    let quiet = |t: f64| {
        let mut x = input(0.2);
        x.time = t;
        x.tips = [x.profile.rest_tips[0], x.profile.rest_tips[1]];
        x.pitches = [0.; 2];
        x
    };
    let witness = sample(0.7, &controls, &[], &quiet);
    assert!(
        feasible_endpoint(0.7, 0, -1., &controls, &[], &quiet),
        "{:?}",
        constraints(&witness, &witness.hands, 0)
    );
    let authored = [vec![[0., 0.683_535]], vec![]];
    let sources = [vec![0., 1.], vec![]];
    let spans = canonical(&authored, &sources, &controls, &[], &quiet);
    assert!(spans[0].contains(&[0., 0.7]), "{:?}", spans[0]);
    assert!(spans[0].contains(&[0.7, 0.8]));

    let unsafe_control = |t: f64| {
        let mut x = quiet(t);
        if (t - 0.7).abs() < 0.0001 {
            x.tips[0] = x.profile.roots[0];
        }
        x
    };
    let spans = canonical(&authored, &sources, &controls, &[], &unsafe_control);
    assert!(spans[0].contains(&[0., 0.8]), "{:?}", spans[0]);
    // Outward-aligned recovery and preparation share one interval, rather than
    // dropping preparation when its original start lies in the extended tail.
    let authored = [vec![[0., 0.683_535], [0.72, 1.]], vec![]];
    let spans = canonical(&authored, &sources, &controls, &[], &unsafe_control);
    assert_eq!(spans[0], vec![[0., 1.]]);

    let sources = [vec![0., 0.75, 1.], vec![]];
    let mut controls = controls;
    controls.push(SwivelSample {
        time: 0.75,
        angles: [0.1; 2],
        orientations: [None; 2],
    });
    controls.sort_by(|a, b| a.time.total_cmp(&b.time));
    let spans = canonical(&authored, &sources, &controls, &[], &unsafe_control);
    assert!(spans[0].contains(&[0., 0.75]));
    assert!(!spans[0].iter().any(|s| s[0] < 0.75 && s[1] > 0.75));
}

#[test]
fn aligned_phase_union_preserves_source_splits_and_held_gaps() {
    let spans = merged_phase_coverage(vec![[0., 0.8], [0.6, 1.]], &[0.75]);
    assert_eq!(spans, vec![[0., 0.75], [0.75, 1.]]);
    // Disjoint entry/exit coverage cannot fill an analytically held interval.
    let spans = merged_phase_coverage(vec![[0., 0.3], [0.6, 0.8], [0.7, 1.]], &[0.3, 0.6, 0.9]);
    assert_eq!(spans, vec![[0., 0.3], [0.6, 0.9], [0.9, 1.]]);
}

#[test]
fn certified_pose_seed_maps_all_four_physical_degrees_exactly() {
    let controls = controls();
    let s = sample(0.31, &controls, &[], &input);
    let original = segment(0, 0., 1., true);
    let mut witness_curve = original.clone();
    witness_curve.parameters = [0.15, -0.03, 0.22, 0.02, 0.11, -0.08, 0.03, -0.01];
    let witness = witness_curve
        .at(&s.input.profile, s.input.time, s.angles[0], s.targets[0])
        .unwrap();
    let mut seeded = original;
    seeded.parameters = seed_from_hand(&seeded, &s, &witness).unwrap();
    let actual = seeded
        .at(&s.input.profile, s.input.time, s.angles[0], s.targets[0])
        .unwrap();
    assert!(distance(actual.middle, witness.middle) < 1e-11);
    assert!(distance(actual.end, witness.end) < 1e-11);
    assert!(
        distance(
            rotate(actual.rotation, [0.2, -0.4, 0.3]),
            rotate(witness.rotation, [0.2, -0.4, 0.3])
        ) < 1e-11
    );
}

#[test]
fn plane_and_lever_twist_repair_preserves_measured_wrist_flex_and_contacts() {
    let controls = controls();
    let mut baseline = segment(1, 0., 1., true);
    baseline.parameters = [0.; 8];
    let mut repaired = baseline.clone();
    repaired.parameters = [0.15, -0.10, -0.12, 0.05, 0., 0., 0., 0.];
    let mut elbow_changed = false;
    for step in 0..=100 {
        let time = step as f64 / 100.;
        let s = sample(time, &controls, &[], &input);
        let p = &s.input.profile;
        let before = baseline.at(p, time, s.angles[1], s.targets[1]).unwrap();
        let after = repaired.at(p, time, s.angles[1], s.targets[1]).unwrap();
        assert!((p.wrist_deviation(1, &before) - p.wrist_deviation(1, &after)).abs() < 1e-12);
        assert!(distance(after.tip, s.targets[1]) < 1e-12);
        assert!((distance(after.middle, after.end) - p.lengths[1][1]).abs() < 1e-12);
        elbow_changed |= distance(before.middle, after.middle) > 0.005;
        if step == 0 || step == 100 {
            assert!(distance(before.middle, after.middle) < 1e-12);
            assert!(distance(before.end, after.end) < 1e-12);
            assert_eq!(before.rotation, after.rotation);
        }
    }
    assert!(
        elbow_changed,
        "the preserved wrist must not disable arm-path repair"
    );
}

#[test]
fn overlapping_hand_phases_share_every_validation_time() {
    let controls = controls();
    let spans = [vec![[0.113, 0.437]], vec![[0.227, 0.519]]];
    let inspection = [0.319_791_666_666_667];
    let times = shared_sample_times(&spans, &controls, &inspection);
    assert!(times.contains(&inspection[0]));
    let left = samples(0.113, 0.437, &times, &controls, &[], &input);
    let right = samples(0.227, 0.519, &times, &controls, &[], &input);
    let overlap = |xs: Vec<Sample>| {
        xs.into_iter()
            .map(|s| s.input.time)
            .filter(|t| *t >= 0.227 && *t <= 0.437)
            .collect::<Vec<_>>()
    };
    assert_eq!(overlap(left), overlap(right));
    assert!(times
        .windows(2)
        .all(|p| p[1] - p[0] <= 1. / CHECK_HZ + 1e-12));
}

#[test]
fn held_cue_endpoints_remain_exact_while_only_held_interiors_are_excluded() {
    let controls = controls();
    let cue_input = |t: f64| {
        let mut x = input(t);
        let endpoint = t == 0. || t == 1.;
        x.cue_orientation = Some(crate::count_in::CueOrientation {
            rotations: x.profile.contact_rotations[..2].try_into().unwrap(),
            swivels: [0.1; 2],
            weight: if endpoint { 1. } else { 0.5 },
        });
        x.intended_stick_contact = endpoint;
        x
    };
    let mut samples: Vec<_> = [0., 0.5, 1.]
        .into_iter()
        .map(|t| sample(t, &controls, &[], &cue_input))
        .collect();
    assert!(!has_held_cue(&samples));
    samples[1].input.intended_stick_contact = true;
    assert!(has_held_cue(&samples));
    samples[1].input.intended_stick_contact = false;
    // A smooth release can begin arbitrarily close to weight one. Only the
    // authored exact plateau is held; numerical closeness is not a new hold.
    samples[1].input.cue_orientation.as_mut().unwrap().weight = 1. - 5e-11;
    assert!(!has_held_cue(&samples));
    samples[1].input.cue_orientation.as_mut().unwrap().weight = 1.;
    assert!(has_held_cue(&samples));
}

#[test]
fn invalid_active_other_hand_cannot_silently_fall_back_during_joint_validation() {
    let controls = controls();
    let s = sample(0.5, &controls, &[], &input);
    let left = segment(0, 0., 1., true);
    let mut right = segment(1, 0.25, 0.75, true);
    right.parameters = [0.; 8];
    right.parameters[4] = std::f64::consts::PI;
    assert!(right
        .at(&s.input.profile, 0.5, s.angles[1], s.targets[1])
        .is_none());
    assert!(candidate_hands(&s, &left, &[right]).is_none());
}

#[test]
fn ordinary_boundary_widens_for_outward_first_order_not_repairable_curvature() {
    let h = DERIVATIVE_STEP;
    assert!(outward_tangent(0., 0.6 * h, 1.2 * h));
    assert!(!outward_tangent(0., -0.6 * h, -1.2 * h));
    assert!(!outward_tangent(0., h * h, 4. * h * h));
    assert!(!outward_tangent(-0.01, -0.01 + 0.6 * h, -0.01 + 1.2 * h));
}

#[test]
fn one_adjacent_merge_never_crosses_an_own_source_contact_or_a_gap() {
    let spans = [[0., 0.2], [0.2, 0.3], [0.3, 0.6], [0.6, 0.8]];
    assert_eq!(coalesced_span(&spans, 2, &[0., 0.6, 0.8]), Some([0.2, 0.6]));
    assert_eq!(
        coalesced_span(&spans, 1, &[0., 0.3, 0.6, 0.8]),
        Some([0., 0.3])
    );
    assert_eq!(coalesced_span(&spans, 2, &[0., 0.3, 0.6, 0.8]), None);
    let gaps = [[0., 0.2], [0.3, 0.6]];
    assert_eq!(coalesced_span(&gaps, 1, &[0., 0.6]), None);
}

#[test]
fn ordinary_requested_target_does_not_inherit_legacy_ik_clamping() {
    let p = profile();
    let requested = [0.2, 1.1, 3.0];
    let hand = p.solve_hand_with_rotation(1, requested, p.contact_rotations[1], 0.);
    assert!(!hand.reachable);
    assert!(distance(hand.tip, requested) > 0.1);
    assert_eq!(effective_target(requested, false, &hand), requested);
    assert_eq!(effective_target(requested, true, &hand), hand.tip);
}

#[test]
fn safe_but_reversing_witness_motion_prefers_lower_motion_endpoint_path() {
    let inputs = |time: f64| {
        let mut i = input(0.);
        i.time = time;
        i.tips = [i.profile.rest_tips[0], i.profile.rest_tips[1]];
        i.pitches = [0.; 2];
        i.pitches[0] += 0.02 * (std::f64::consts::TAU * time).sin().powi(2);
        i
    };
    let controls = controls();
    let times: Vec<_> = (0..=480).map(|i| i as f64 / 480.).collect();
    let samples = samples(0., 1., &times, &controls, &[], &inputs);
    assert!(
        samples.iter().all(|s| physical_valid(s, &s.hands, 0)),
        "{:?}",
        samples
            .iter()
            .filter(|s| !physical_valid(s, &s.hands, 0))
            .take(3)
            .map(|s| (s.input.time, constraints(s, &s.hands, 0)))
            .collect::<Vec<_>>()
    );
    let base = make_segment(0, 0., 1., &[0., 1.], &controls, &[], &inputs).unwrap();
    let curve =
        optimize_segment(base, &samples, &[]).expect("safe witness reversals should simplify");
    let before = original_motion_cost(&samples, &[], 0);
    let after = motion_cost(
        &samples,
        &(0..samples.len()).collect::<Vec<_>>(),
        &curve,
        &[],
    );
    assert!(after < before * 0.01, "before {before}, after {after}");
    for s in &samples {
        let h = candidate_hands(s, &curve, &[]).unwrap();
        assert!(physical_valid(s, &h, 0));
        assert!(distance(h[0].tip, s.targets[0]) < 1e-12);
        if s.input.time == 0. || s.input.time == 1. {
            assert!(distance(h[0].middle, s.hands[0].middle) < 1e-12);
            assert!(distance(h[0].end, s.hands[0].end) < 1e-12);
        }
    }
}
