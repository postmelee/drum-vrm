use super::*;
use crate::equipment_motion::{EquipmentContext, PathSample};
use std::sync::Arc;

fn two_strokes() -> Motion {
    let track = [
        0, 0x99, 38, 100, 0x83, 0x60, 0x99, 38, 100, 0x83, 0x60, 0xff, 0x2f, 0,
    ];
    let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
    bytes.extend((track.len() as u32).to_be_bytes());
    bytes.extend(track);
    let plan = crate::build_plan(
        &bytes,
        &Selection {
            tracks: vec![0],
            ..Selection::default()
        },
    )
    .unwrap();
    Motion::new(plan).unwrap()
}

#[test]
fn provisional_pitch_warning_requires_a_final_actual_source_failure() {
    let mut motion = two_strokes();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-natural-grip30.json")).unwrap();
    let profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
    let target = [-0.10, 0.90, 0.65];
    let equipment = Arc::new(EquipmentContext::new(
        vec![],
        crate::equipment::kit_geometry(&[]),
    ));
    let (angle, hand) = (-16..=16)
        .find_map(|step| {
            let angle = step as f64 * 0.05;
            let hand = crate::equipment_motion::HandEnvironment {
                equipment: Some(&equipment),
                intended_piece: Some("snare"),
            }
            .solve(&profile, 0, target, 0., angle, false)?;
            crate::hand_coordination::source_pose_valid(&profile, 0, &hand, f64::INFINITY)
                .then_some((angle, hand))
        })
        .expect("the authored source should have a clear ordinary pose");
    let orientation = profile.encode_hand_orientation(0, target, 0., angle, hand.rotation);
    motion.profile = Some(profile);
    motion.equipment = Some(equipment);
    motion.limbs = [vec![0, 1], vec![], vec![], vec![]];
    for (index, hit) in motion.plan.hits.iter_mut().enumerate() {
        hit.time = index as f64 + 1.;
        hit.limb = Some(0);
        hit.target = target;
        hit.piece_id = "snare".into();
    }
    motion.hand_swivels = [1., 2.]
        .map(|time| crate::hand_coordination::SwivelSample {
            time,
            angles: [angle, 0.],
            orientations: [orientation, None],
        })
        .to_vec();
    motion.plan.diagnostics = vec![Diagnostic::new(
        "TOOL_EQUIPMENT_PATH_CLEARANCE",
        "An independently observed interior collision.".into(),
    )];
    let hits_before = serde_json::to_string(&motion.plan.hits).unwrap();
    let pose_before = motion.evaluate_rig(1.);
    assert!(motion.source_pose_is_clear(0, 1.));
    motion.reconcile_source_pitch_warnings([vec![1.], vec![]]);
    assert_eq!(motion.plan.diagnostics.len(), 1);
    assert_eq!(
        motion.plan.diagnostics[0].code,
        "TOOL_EQUIPMENT_PATH_CLEARANCE"
    );
    assert_eq!(motion.evaluate_rig(1.), pose_before);

    // A real contact obstruction must keep the provisional warning even when
    // the same hand orientation was previously valid. An unknown source time
    // likewise cannot be silently considered resolved.
    let mut geometry = crate::equipment::kit_geometry(&[]);
    geometry.bodies.push(crate::equipment::EquipmentBox {
        piece_id: "unrelated-obstacle".into(),
        center: target,
        half_extents: [0.02; 3],
        rotation: None,
        kind: "body",
    });
    motion.equipment = Some(Arc::new(EquipmentContext::new(vec![], geometry)));
    assert!(!motion.source_pose_is_clear(0, 1.));
    motion.reconcile_source_pitch_warnings([vec![1., 3.], vec![]]);
    let warning = motion.plan.diagnostics.last().unwrap();
    assert_eq!(warning.code, "CONTACT_PITCH_LIMIT");
    assert_eq!(warning.time, Some(1.));
    assert!(warning.message.starts_with("2 provisional"));
    assert_eq!(
        serde_json::to_string(&motion.plan.hits).unwrap(),
        hits_before
    );
}

#[test]
fn repeated_contact_rebound_keeps_the_source_reach_envelope() {
    let mut motion = two_strokes();
    let mut profile = RigProfile::primitive();
    profile.roots[1] = [0.09, 1.07, 0.12];
    let root = profile.roots[1];
    let target = [0.27, 1.08, 0.89];
    let radius = distance(root, target);
    motion.profile = Some(profile);
    motion.equipment = None;
    motion.ready_poses = ready_pose::ReadyPoses::default();
    motion.limbs = [vec![], vec![0, 1], vec![], vec![]];
    for hit in &mut motion.plan.hits {
        hit.target = target;
        hit.piece_id = "tomMid".into();
    }
    motion.plan.hits[0].time = 1.;
    for interval in [0.0625, 0.125, 0.25] {
        motion.plan.hits[1].time = 1. + interval;
        let mut peak_lift = 0_f64;
        for step in 0..=120 {
            let time = 1. + interval * step as f64 / 120.;
            let tip = motion.raw_stroke_at(1, time).0;
            assert!(distance(root, tip) <= radius + 1e-12);
            peak_lift = peak_lift.max(tip[1] - target[1]);
        }
        assert!(peak_lift > 0.01, "repeat rebound should remain visible");
        for hit in &motion.plan.hits {
            assert_eq!(motion.raw_stroke_at(1, hit.time).0, target);
        }
    }
}

#[test]
fn cross_kit_preparation_does_not_jump_at_a_grazing_tom_boundary() {
    // Independently authored close rack geometry. A framewise radial expansion
    // used to release and retract near the two reserved box faces, moving the
    // tip by centimetres while the source chord moved only a few millimetres.
    let mut motion = two_strokes();
    let mut profile = RigProfile::primitive();
    profile.roots[1] = [0.11, 1.10, 0.11];
    profile.rest_tips[1] = [-0.38, 0.94, 0.45];
    motion.profile = Some(profile);
    motion.ready_poses = ready_pose::ReadyPoses::default();
    motion.plan.duration = 7.5;
    motion.plan.hits[0].time = 6.375;
    motion.plan.hits[0].target = [-0.08, 0.82, 0.62];
    motion.plan.hits[0].piece_id = "snare".into();
    motion.plan.hits[1].time = 7.;
    motion.plan.hits[1].target = [0.28, 1.13, 0.86];
    motion.plan.hits[1].piece_id = "tomMid".into();
    motion.limbs = [vec![], vec![0, 1], vec![], vec![]];
    let mut geometry = crate::equipment::kit_geometry(&[]);
    for (id, center, half_extents) in [
        ("tomHigh", [-0.16, 1.05, 0.91], [0.166, 0.176, 0.200]),
        ("tomMid", [0.28, 1.02, 0.91], [0.179, 0.182, 0.212]),
    ] {
        geometry.bodies.push(crate::equipment::EquipmentBox {
            piece_id: id.into(),
            center,
            half_extents,
            rotation: None,
            kind: "body",
        });
    }
    motion.equipment = Some(Arc::new(EquipmentContext::new(vec![], geometry)));
    let mut previous = motion.raw_stroke_at(1, 6.68).0;
    let mut peak = 0_f64;
    for step in 1..=3200 {
        let time = 6.68 + step as f64 / 10_000.;
        let point = motion.raw_stroke_at(1, time).0;
        peak = peak.max(distance(previous, point) * 10_000.);
        previous = point;
    }
    // A regression bound for this authored route, not a physiological policy.
    assert!(
        peak < 6.,
        "grazing boundary produces a tip-speed spike: {peak}"
    );
    for hit in &motion.plan.hits {
        assert_eq!(motion.raw_stroke_at(1, hit.time).0, hit.target);
    }
}

#[test]
fn final_inspection_reports_an_interior_collision_between_clear_control_knots() {
    // Authored two-stroke fixture: both source impacts are clear, while the
    // rebound passes through a small overhead obstacle between stored knots.
    let mut motion = two_strokes();
    motion.profile = Some(RigProfile::primitive());
    for hit in &mut motion.plan.hits {
        hit.limb = Some(0);
    }
    motion.limbs = [vec![0, 1], vec![], vec![], vec![]];
    motion.ready_poses = ready_pose::ReadyPoses::for_plan(
        &motion.plan,
        &motion.limbs,
        motion.profile.as_ref(),
        None,
    );
    let start = motion.plan.hits[0].time;
    let end = motion.plan.hits[1].time;
    motion.equipment_path = [start, end]
        .map(|time| PathSample {
            time,
            lift: [0.; 2],
            pitch: [0.; 2],
        })
        .to_vec();
    let middle = (start + end) * 0.5;
    let center = pose_hands(&motion.evaluate_rig(middle))[0].tip;
    let mut geometry = crate::equipment::kit_geometry(&[]);
    geometry.bodies.push(crate::equipment::EquipmentBox {
        piece_id: "overhead-obstacle".into(),
        center,
        half_extents: [0.008; 3],
        rotation: None,
        kind: "body",
    });
    motion.equipment = Some(Arc::new(EquipmentContext::new(vec![], geometry)));
    let clearance = |time| {
        motion
            .pose_equipment_clearance(time, &motion.evaluate_rig(time))
            .unwrap()
            .physical_tool
    };
    for time in [start, end] {
        assert!(clearance(time) > 0., "source impact at {time} is not clear");
    }
    assert!(clearance(middle) < -0.001);
    let before = [start, middle, end].map(|time| motion.evaluate_rig(time));
    motion.plan.diagnostics.clear();
    motion.inspect_equipment_path();
    let diagnostic = motion
        .plan
        .diagnostics
        .iter()
        .find(|d| d.code == "TOOL_EQUIPMENT_PATH_CLEARANCE")
        .expect("an interior physical collision must survive final diagnostics");
    assert!(diagnostic
        .time
        .is_some_and(|time| time > start && time < end));
    assert_eq!(
        before,
        [start, middle, end].map(|time| motion.evaluate_rig(time)),
        "inspection must not repair or otherwise change the motion"
    );
}

#[test]
fn final_posture_inspection_reports_a_nonimpact_violation_without_changing_poses() {
    let mut motion = two_strokes();
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
    profile.rest_tips[0] = [-0.1, 0.95, 0.65];
    profile.rest_tips[1] = [0.1, 0.95, 0.65];
    motion.profile = Some(profile.clone());
    // An idle interval has no impacts to trigger the former impact-only check.
    motion.plan.hits.clear();
    motion.limbs = std::array::from_fn(|_| vec![]);
    motion.ready_poses = ready_pose::ReadyPoses::default();
    motion.equipment_path = [(0., 0.), (0.5, -0.6), (1., 0.)]
        .map(|(time, pitch)| PathSample {
            time,
            lift: [0.; 2],
            pitch: [pitch, 0.],
        })
        .to_vec();
    let valid = |time| {
        pose_hands(&motion.evaluate_rig(time))
            .iter()
            .enumerate()
            .all(|(limb, hand)| crate::posture::anatomical(&profile, limb, hand.tip, hand))
    };
    assert!(
        valid(0.) && valid(1.),
        "idle endpoints must be within bounds"
    );
    assert!(!valid(0.5), "the middle must exceed a common posture bound");
    let before = [0., 0.5, 1.].map(|time| motion.evaluate_rig(time));
    let sources_before = serde_json::to_string(&motion.plan.source_events).unwrap();
    motion.plan.diagnostics.clear();
    motion.inspect_posture_path();
    let diagnostics: Vec<_> = motion
        .plan
        .diagnostics
        .iter()
        .filter(|d| d.code == "POSTURE_LIMIT")
        .collect();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0]
        .time
        .is_some_and(|time| time > 0. && time < 1.));
    assert_eq!(before, [0., 0.5, 1.].map(|time| motion.evaluate_rig(time)));
    assert_eq!(
        sources_before,
        serde_json::to_string(&motion.plan.source_events).unwrap()
    );
}

#[test]
fn spare_settling_requires_both_hands_to_be_idle() {
    let mut motion = two_strokes();
    motion.plan.hits[0].limb = Some(0);
    motion.plan.hits[1].limb = Some(0);
    motion.plan.hits[1].time = 2.;
    motion.plan.duration = 2.5;
    motion.limbs = [vec![0, 1], vec![], vec![], vec![]];
    let contact = motion.plan.hits[0].target;
    let resting = rest_tip(0);
    assert!(distance(motion.raw_stroke_at(0, 0.2).0, resting) > 0.01);
    assert_eq!(motion.raw_stroke_at(0, 0.).0, contact);
    // Ongoing opposite-hand preparation keeps the original180ms recovery.
    let mut opposite = motion.plan.hits[0].clone();
    opposite.limb = Some(1);
    opposite.target = rest_tip(1);
    opposite.time = 0.25;
    motion.plan.hits.push(opposite);
    motion.limbs[1] = vec![2];
    assert_eq!(motion.raw_stroke_at(0, 0.2).0, resting);
    // A simultaneous opposite-hand contact is also still recovering.
    motion.plan.hits[2].time = 0.;
    assert_eq!(motion.raw_stroke_at(0, 0.2).0, resting);
    assert_eq!(motion.raw_stroke_at(0, 2.).0, motion.plan.hits[1].target);
}
