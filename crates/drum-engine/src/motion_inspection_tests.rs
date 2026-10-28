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
