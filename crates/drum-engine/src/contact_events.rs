//! One derived, semantic contact record for planning, mechanisms and display.
//! A planned event is never a claim that a rendered mesh actually touched it.
use crate::{contact, count_in::CountInCue, equipment, hihat_motion, model::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolPart {
    StickTip,
    Sole,
    Beater,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContactRole {
    Primary,
    Mechanism,
    PairedTool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContactStatus {
    Planned,
    Unassigned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContactFrame {
    PieceReference,
    World,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactEvent {
    pub version: u8,
    pub id: String,
    pub source_event_id: String,
    pub hit_id: Option<usize>,
    pub time: f64,
    pub time_us: u64,
    pub velocity: u8,
    pub limb: Option<usize>,
    pub tool_part: ToolPart,
    pub role: ContactRole,
    pub piece_id: Option<String>,
    pub surface_id: String,
    pub frame: ContactFrame,
    /// Reference-scaled piece coordinates, or world coordinates for paired tools.
    pub local_point: Vec3,
    pub local_normal: Option<Vec3>,
    /// Mechanism displacement already present in this impact point, in metres.
    pub mechanism_offset: Vec3,
    pub status: ContactStatus,
}

fn make_event(
    hit: &Hit,
    piece: &KitPiece,
    surface_frame: (Vec3, Vec3),
    tool_part: ToolPart,
    surface_id: &str,
    role: ContactRole,
    opening_offset: Vec3,
) -> ContactEvent {
    let (point, normal) = surface_frame;
    let ratio = piece
        .geometry
        .as_ref()
        .map_or(1., |g| piece.radius / g.reference_radius);
    ContactEvent {
        version: 1,
        id: format!("hit:{}:{}", hit.id, surface_id),
        source_event_id: hit.source_event_id.clone(),
        hit_id: Some(hit.id),
        time: hit.time,
        time_us: hit.time_us,
        velocity: hit.velocity,
        limb: hit.limb,
        tool_part,
        role,
        piece_id: Some(piece.id.clone()),
        surface_id: surface_id.into(),
        frame: ContactFrame::PieceReference,
        local_point: std::array::from_fn(|axis| (point[axis] - piece.position[axis]) / ratio),
        local_normal: Some(normal),
        mechanism_offset: opening_offset,
        status: if hit.limb.is_some() {
            ContactStatus::Planned
        } else {
            ContactStatus::Unassigned
        },
    }
}

/// Discard imported copies and derive only from the validated plan and geometry.
/// Motion targets were resolved before assignment by contact::apply.
pub fn refresh_events(plan: &mut PerformancePlan) {
    plan.hihat_motion = hihat_motion::prepare(plan);
    let mut events = Vec::with_capacity(plan.hits.len());
    for hit in &plan.hits {
        let Some(piece) = plan.kit.iter().find(|piece| piece.id == hit.piece_id) else {
            continue;
        };
        let pedal = matches!(hit.instrument, Instrument::Kick | Instrument::PedalHat);
        let surface = if pedal {
            "pedal"
        } else {
            match hit.articulation.as_str() {
                "bell" => "bell",
                "edge" | "rimshotEdge" | "sideStick" | "crash" | "china" | "splash" => "edge",
                _ if matches!(
                    hit.instrument,
                    Instrument::Hihat | Instrument::OpenHat | Instrument::Crash | Instrument::Ride
                ) =>
                {
                    "bow"
                }
                _ => "head",
            }
        };
        let opening_offset =
            contact::mechanism_offset(piece, hihat_motion::evaluate(&plan.hihat_motion, hit.time));
        let static_target = std::array::from_fn(|axis| hit.target[axis] - opening_offset[axis]);
        events.push(make_event(
            hit,
            piece,
            (
                hit.target,
                if pedal {
                    [0., 1., 0.]
                } else {
                    contact::normal_at(piece, static_target)
                },
            ),
            if pedal {
                ToolPart::Sole
            } else {
                ToolPart::StickTip
            },
            surface,
            ContactRole::Primary,
            opening_offset,
        ));
        if hit.instrument == Instrument::Kick {
            let (point, normal) = equipment::beater_contact(piece);
            events.push(make_event(
                hit,
                piece,
                (point, normal),
                ToolPart::Beater,
                "batter",
                ContactRole::Mechanism,
                [0.; 3],
            ));
        }
    }
    plan.contact_events = events;
}

/// A pair has a common point but no instrument surface or invented surface normal.
pub fn append_count_in_events(plan: &mut PerformancePlan, cues: &[CountInCue]) {
    for cue in cues {
        let source = plan
            .source_events
            .iter()
            .find(|event| event.id == cue.source_event_id);
        let velocity = source
            .and_then(|event| match event.message {
                crate::interpretation::SourceMessage::NoteOn { velocity, .. } => Some(velocity),
                _ => None,
            })
            .unwrap_or(0);
        for limb in 0..2 {
            plan.contact_events.push(ContactEvent {
                version: 1,
                id: format!("cue:{}:{}", cue.source_event_id, limb),
                source_event_id: cue.source_event_id.clone(),
                hit_id: None,
                time: cue.time,
                time_us: source
                    .map_or_else(|| (cue.time * 1_000_000.).round() as u64, |s| s.time_us),
                velocity,
                limb: Some(limb),
                tool_part: ToolPart::StickTip,
                role: ContactRole::PairedTool,
                piece_id: None,
                surface_id: "stickTip".into(),
                frame: ContactFrame::World,
                local_point: cue.contact_point,
                local_normal: None,
                mechanism_offset: [0.; 3],
                status: ContactStatus::Planned,
            });
        }
    }
    plan.contact_events
        .sort_by(|a, b| a.time.total_cmp(&b.time).then(a.id.cmp(&b.id)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kick_separates_sole_and_beater_frames_and_unassigned_is_explicit() {
        let piece = default_kit()
            .into_iter()
            .find(|p| p.instrument == Instrument::Kick)
            .unwrap();
        let hit = Hit {
            id: 2,
            time: 1.,
            time_us: 1_000_000,
            tick: 480,
            track: 1,
            event_index: 1,
            channel: 9,
            note: 36,
            velocity: 90,
            instrument: Instrument::Kick,
            piece_id: "kick".into(),
            articulation: "center".into(),
            source_event_id: "1:1".into(),
            limb: None,
            target: piece.position,
        };
        let primary = make_event(
            &hit,
            &piece,
            (hit.target, [0., 1., 0.]),
            ToolPart::Sole,
            "pedal",
            ContactRole::Primary,
            [0.; 3],
        );
        let (batter, normal) = equipment::beater_contact(&piece);
        let secondary = make_event(
            &hit,
            &piece,
            (batter, normal),
            ToolPart::Beater,
            "batter",
            ContactRole::Mechanism,
            [0.; 3],
        );
        assert_ne!(primary.local_point, secondary.local_point);
        assert_eq!(primary.status, ContactStatus::Unassigned);
        assert_eq!(secondary.status, primary.status);
        assert_eq!(primary.time, secondary.time);
        assert_eq!(primary.source_event_id, secondary.source_event_id);
        assert_eq!(secondary.local_normal, Some([0., 0., -1.]));
    }
    #[test]
    fn piece_frame_round_trip_retains_world_contact_after_fit() {
        let mut piece = default_kit()
            .into_iter()
            .find(|p| p.instrument == Instrument::Snare)
            .unwrap();
        piece.geometry = Some(equipment::local_geometry(&piece));
        piece.radius *= 0.83;
        let point = [0.13, 0.81, 0.43];
        let hit = Hit {
            id: 0,
            time: 0.,
            time_us: 0,
            tick: 0,
            track: 1,
            event_index: 0,
            channel: 9,
            note: 38,
            velocity: 80,
            instrument: Instrument::Snare,
            piece_id: "snare".into(),
            articulation: "edge".into(),
            source_event_id: "1:0".into(),
            limb: Some(0),
            target: point,
        };
        let event = make_event(
            &hit,
            &piece,
            (point, [0., 1., 0.]),
            ToolPart::StickTip,
            "edge",
            ContactRole::Primary,
            [0.; 3],
        );
        let scale = piece.radius / piece.geometry.as_ref().unwrap().reference_radius;
        for (axis, value) in point.iter().enumerate() {
            assert!((piece.position[axis] + event.local_point[axis] * scale - value).abs() < 1e-12);
        }
        assert_eq!(event.status, ContactStatus::Planned);
    }
    #[test]
    fn explicit_hat_opening_is_shared_by_motion_target_and_event_without_source_change() {
        let mut track = vec![0, 0x99, 46, 90, 30, 0x99, 44, 70, 0, 0xff, 0x2f, 0];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.append(&mut track);
        let selection = Selection {
            tracks: vec![0],
            hihat_policy: crate::hihat::HihatPolicy::ShortOpenClose,
            ..Selection::default()
        };
        let mut plan = crate::build_plan(&bytes, &selection).unwrap();
        let source = serde_json::to_value(&plan.source_events).unwrap();
        refresh_events(&mut plan);
        let hit = &plan.hits[0];
        let piece = plan.kit.iter().find(|p| p.id == hit.piece_id).unwrap();
        assert!(
            (hit.target[1] - piece.position[1] - equipment::HIHAT_MAX_OPENING * 0.2).abs() < 1e-12
        );
        let event = &plan.contact_events[0];
        assert!((event.local_point[1] - event.mechanism_offset[1]).abs() < 1e-12);
        assert_eq!(event.source_event_id, hit.source_event_id);
        assert_eq!(event.time_us, hit.time_us);
        assert_eq!(serde_json::to_value(&plan.source_events).unwrap(), source);
        assert_eq!(hihat_motion::evaluate(&plan.hihat_motion, 0.03125), 0.);
        let mut legacy = serde_json::to_value(&plan).unwrap();
        legacy.as_object_mut().unwrap().remove("contactEvents");
        legacy.as_object_mut().unwrap().remove("hihatMotion");
        let mut restored: PerformancePlan = serde_json::from_value(legacy).unwrap();
        assert!(restored.contact_events.is_empty());
        assert!(restored.hihat_motion.anchors.is_empty());
        contact::apply(&mut restored, &crate::rig::RigProfile::primitive());
        refresh_events(&mut restored);
        assert_eq!(restored.contact_events, plan.contact_events);
        assert_eq!(restored.hihat_motion, plan.hihat_motion);
        // An electronic pad or unknown imported surface must not acquire an
        // imaginary opening merely because the score contains an open note.
        let index = plan
            .kit
            .iter()
            .position(|p| p.id == plan.hits[0].piece_id)
            .unwrap();
        plan.kit[index].geometry = Some(equipment::local_geometry(&plan.kit[index]));
        let profile = crate::rig::RigProfile::primitive();
        let observer =
            std::array::from_fn(|axis| (profile.roots[0][axis] + profile.roots[1][axis]) * 0.5);
        let fixed = contact::target(&plan.kit[index], &plan.hits[0].articulation, observer);
        contact::apply(&mut plan, &profile);
        refresh_events(&mut plan);
        assert_eq!(plan.hits[0].target, fixed);
        assert_eq!(plan.contact_events[0].mechanism_offset, [0.; 3]);
        assert_eq!(
            plan.diagnostics
                .iter()
                .filter(|d| d.code == "HIHAT_SURFACE_STATIC")
                .count(),
            1
        );
        // A declared translation carries its full vector. Mechanical travel
        // does not scale with the shell or mutate the source/impact time.
        plan.kit[index].geometry.as_mut().unwrap().surface.motion =
            Some(equipment::SurfaceMotion {
                kind: "translation".into(),
                axis: [0., 0.6, 0.8],
                travel_meters: 0.02,
            });
        contact::apply(&mut plan, &profile);
        refresh_events(&mut plan);
        for axis in 0..3 {
            let offset = [0., 0.0024, 0.0032][axis];
            assert!((plan.hits[0].target[axis] - fixed[axis] - offset).abs() < 1e-12);
            assert!((plan.contact_events[0].mechanism_offset[axis] - offset).abs() < 1e-12);
        }
        assert!(!plan
            .diagnostics
            .iter()
            .any(|d| d.code == "HIHAT_SURFACE_STATIC"));
        assert_eq!(serde_json::to_value(&plan.source_events).unwrap(), source);
    }

    #[test]
    fn sole_events_match_hinged_pedals_at_impacts_and_control_closures() {
        // Open/close, an isolated foot chick, and a simultaneous open/close.
        let track = vec![
            0, 0x99, 46, 90, 0, 0x99, 36, 100, 48, 0x99, 44, 70, 0, 0x99, 36, 90, 96, 0x99, 44, 80,
            0, 0x99, 46, 80, 96, 0x99, 36, 100, 96, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        for policy in [
            crate::hihat::HihatPolicy::Literal,
            crate::hihat::HihatPolicy::ShortOpenClose,
        ] {
            let selection = Selection {
                tracks: vec![0],
                hihat_policy: policy,
                ..Selection::default()
            };
            let plan = crate::build_plan(&bytes, &selection).unwrap();
            let mut profile = crate::rig::RigProfile::primitive();
            profile.id = "sole-contact-test".into();
            for (limb, instrument) in [(2, Instrument::PedalHat), (3, Instrument::Kick)] {
                let piece = plan
                    .kit
                    .iter()
                    .find(|piece| piece.instrument == instrument)
                    .unwrap();
                profile.roots[limb] = [
                    piece.position[0],
                    piece.position[1] + 0.65,
                    piece.position[2] - 0.2,
                ];
                profile.rest_tips[limb] = piece.position;
            }
            let motion = crate::motion::Motion::with_profile(plan, profile).unwrap();
            let sole: Vec<_> = motion
                .plan
                .contact_events
                .iter()
                .filter(|event| event.tool_part == ToolPart::Sole)
                .collect();
            assert_eq!(
                sole.iter()
                    .filter(|event| event.piece_id.as_deref() == Some("kick"))
                    .count(),
                3
            );
            assert_eq!(
                sole.iter()
                    .filter(|event| event.piece_id.as_deref() == Some("pedalHat"))
                    .count(),
                if policy == crate::hihat::HihatPolicy::Literal {
                    2
                } else {
                    1
                }
            );
            for event in sole {
                assert_eq!(event.status, ContactStatus::Planned);
                let limb = event.limb.unwrap();
                let piece = motion
                    .plan
                    .kit
                    .iter()
                    .find(|piece| Some(&piece.id) == event.piece_id.as_ref())
                    .unwrap();
                let ratio = piece
                    .geometry
                    .as_ref()
                    .map_or(1., |geometry| piece.radius / geometry.reference_radius);
                let intended: Vec3 = std::array::from_fn(|axis| {
                    piece.position[axis] + event.local_point[axis] * ratio
                });
                let pose = motion.evaluate_rig(event.time);
                let actual: Vec3 = std::array::from_fn(|axis| pose[limb * 12 + 9 + axis] as f64);
                assert!(crate::motion::distance(actual, intended) < 1e-6);
                assert_eq!(intended, equipment::pedal_contact(piece, 0.));
                let rotation = std::array::from_fn(|axis| pose[57 + limb * 4 + axis] as f64);
                assert!(
                    crate::motion::distance(
                        crate::rig::rotate(rotation, [0., 1., 0.]),
                        event.local_normal.unwrap()
                    ) < 1e-6
                );
            }
            assert_eq!(
                motion.plan.controls.len(),
                if policy == crate::hihat::HihatPolicy::Literal {
                    0
                } else {
                    1
                }
            );
            let pedal = motion
                .plan
                .kit
                .iter()
                .find(|piece| piece.instrument == Instrument::PedalHat)
                .unwrap();
            for control in &motion.plan.controls {
                let time = control.time_us as f64 / 1_000_000.;
                let pose = motion.evaluate_rig(time);
                assert_eq!(hihat_motion::evaluate(&motion.plan.hihat_motion, time), 0.);
                assert!(
                    crate::motion::distance(
                        std::array::from_fn(|axis| pose[33 + axis] as f64),
                        pedal.position
                    ) < 1e-6
                );
            }
        }
    }
}
