//! Opaque, version-bound worker transfer. This is not a persistent project format.
//! The checksum detects stale/mixed/accidentally edited messages; it is not an
//! authenticity signature. Independent validation remains mandatory on import.
use super::{validate_plan, Motion, PLANNER_VERSION};
use crate::{
    body_motion, equipment_motion::PathSample, hand_coordination, model::*, rig::RigProfile,
};
use hand_coordination::{OrientationSample, SwivelSample};
use serde::{Deserialize, Serialize};
const FORMAT_VERSION: u32 = 7;
const MAX_BYTES: usize = 128 * 1024 * 1024;
pub(super) const MAX_EQUIPMENT_SAMPLES: usize = 16_000;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Prepared {
    format_version: u32,
    checksum: String,
    profile_hash: String,
    plan: PerformancePlan,
    profile: Option<RigProfile>,
    #[serde(deserialize_with = "bounded_track")]
    body_turns: Vec<f64>,
    #[serde(deserialize_with = "bounded_track")]
    hand_swivels: Vec<SwivelSample>,
    #[serde(deserialize_with = "bounded_track")]
    hand_orientations: Vec<OrientationSample>,
    #[serde(deserialize_with = "bounded_equipment_track")]
    equipment_path: Vec<PathSample>,
}
fn bounded_track<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    bounded_values::<D, T, { hand_coordination::MAX_CONTROL_KNOTS }>(deserializer)
}
fn bounded_equipment_track<'de, D>(deserializer: D) -> Result<Vec<PathSample>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    bounded_values::<D, PathSample, MAX_EQUIPMENT_SAMPLES>(deserializer)
}
fn bounded_values<'de, D, T, const LIMIT: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct TrackVisitor<T, const LIMIT: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> serde::de::Visitor<'de>
        for TrackVisitor<T, LIMIT>
    {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a bounded numeric motion track")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<T>, A::Error> {
            let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(LIMIT));
            while let Some(value) = seq.next_element()? {
                if values.len() >= LIMIT {
                    return Err(serde::de::Error::custom(
                        "prepared motion track exceeds planning budget",
                    ));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(TrackVisitor::<T, LIMIT>(std::marker::PhantomData))
}
fn fingerprint(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("validated finite worker transfer");
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x100000001b3)
    });
    format!("fnv1a64:{hash:016x}")
}
impl Prepared {
    fn checksum(&self) -> String {
        fingerprint(&(
            self.format_version,
            &self.profile_hash,
            &self.plan,
            &self.profile,
            &self.body_turns,
            &self.hand_swivels,
            &self.hand_orientations,
            &self.equipment_path,
        ))
    }
}
/// Same exact time grid for worker preparation and transfer validation.
fn merge_times(mut samples: Vec<(f64, u8)>) -> Vec<f64> {
    samples.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut result: Vec<(f64, u8)> = Vec::with_capacity(samples.len());
    for sample in samples {
        if let Some(previous) = result.last_mut().filter(|p| (p.0 - sample.0).abs() < 1e-9) {
            if sample.1 > previous.1 {
                *previous = sample;
            }
        } else {
            result.push(sample);
        }
    }
    result.into_iter().map(|s| s.0).collect()
}
pub(super) fn equipment_times(plan: &PerformancePlan) -> Result<Vec<f64>, String> {
    let count = ((plan.duration + PRE_ROLL + TAIL) * 30.0).ceil() as usize + 1;
    let mut samples: Vec<_> = (0..count)
        .map(|i| (i as f64 / 30.0 - PRE_ROLL, 0))
        .collect();
    samples.extend(
        plan.hits
            .iter()
            .filter(|h| h.limb.is_some_and(|l| l < 2))
            .map(|h| (h.time, 2)),
    );
    let times = merge_times(samples);
    if times.len() > MAX_EQUIPMENT_SAMPLES {
        return Err(format!("장비 경로 표본 {}개가 한계 {MAX_EQUIPMENT_SAMPLES}개를 초과합니다. 더 짧은 MIDI 구간이나 적은 트랙으로 다시 준비하세요.", times.len()));
    }
    Ok(times)
}
pub(super) fn control_times(plan: &PerformancePlan) -> Result<Vec<f64>, String> {
    let count = ((plan.duration + PRE_ROLL + TAIL) / hand_coordination::STEP).ceil() as usize + 1;
    let mut samples: Vec<_> = (0..count)
        .map(|i| (i as f64 * hand_coordination::STEP - PRE_ROLL, 0))
        .collect();
    samples.extend(
        plan.hits
            .iter()
            .filter(|h| h.limb.is_some_and(|l| l < 2))
            .map(|h| (h.time, 2)),
    );
    samples.extend(
        crate::count_in::control_times_for_plan(plan)
            .into_iter()
            .map(|t| (t, 1)),
    );
    let times = merge_times(samples);
    hand_coordination::check_control_budget(times.len())?;
    Ok(times)
}
impl Motion {
    pub fn prepared_json(&self) -> String {
        let mut prepared = Prepared {
            format_version: FORMAT_VERSION,
            checksum: String::new(),
            profile_hash: fingerprint(&self.profile),
            plan: self.plan.clone(),
            profile: self.profile.clone(),
            body_turns: self.body_turns.clone(),
            hand_swivels: self.hand_swivels.clone(),
            hand_orientations: self.hand_orientations.clone(),
            equipment_path: self.equipment_path.clone(),
        };
        prepared.checksum = prepared.checksum();
        serde_json::to_string(&prepared).expect("validated finite worker transfer")
    }
    pub fn from_prepared_json(
        json: &str,
        midi_hash: &str,
        profile_json: &str,
    ) -> Result<Self, String> {
        if json.len() > MAX_BYTES {
            return Err("준비된 연주 전달 크기가 한계를 초과합니다.".into());
        }
        let p: Prepared =
            serde_json::from_str(json).map_err(|e| format!("준비된 연주 형식 오류: {e}"))?;
        if p.format_version != FORMAT_VERSION
            || p.plan.planner_version != PLANNER_VERSION
            || p.plan.engine_version != env!("CARGO_PKG_VERSION")
            || p.plan.midi_hash != midi_hash
            || p.checksum != p.checksum()
        {
            return Err("준비된 연주의 버전·원본·체크섬이 일치하지 않습니다.".into());
        }
        let expected_profile = if profile_json.is_empty() {
            None
        } else {
            Some(RigProfile::from_json(profile_json)?)
        };
        if p.profile_hash != fingerprint(&p.profile)
            || p.profile_hash != fingerprint(&expected_profile)
        {
            return Err("준비된 연주의 리그 프로필이 요청과 일치하지 않습니다.".into());
        }
        validate_plan(&p.plan)?;
        if let Some(profile) = &p.profile {
            profile.validate()?;
            if p.plan.calibration_id != profile.id {
                return Err("준비된 연주의 보정 ID가 일치하지 않습니다.".into());
            }
            for limb in 2..4 {
                let anchor = super::pedal_contact(&p.plan, profile, limb);
                let instrument = if limb == 2 {
                    Instrument::PedalHat
                } else {
                    Instrument::Kick
                };
                if p.plan.hits.iter().any(|hit| {
                    hit.instrument == instrument && super::distance(hit.target, anchor) > 1e-6
                }) {
                    return Err("준비된 연주의 페달 접촉 위치가 일치하지 않습니다.".into());
                }
            }
        } else if p.plan.calibration_id != "primitive-v1" {
            return Err("준비된 연주에 리그 프로필이 필요합니다.".into());
        }
        let body_count = if p
            .profile
            .as_ref()
            .is_some_and(|profile| profile.torso_pivot.is_some())
        {
            ((p.plan.duration + PRE_ROLL + TAIL) / body_motion::STEP).ceil() as usize + 1
        } else {
            0
        };
        if p.body_turns.len() != body_count
            || p.body_turns
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 0.24 + 1e-9)
            || p.body_turns.first().is_some_and(|v| v.abs() > 0.06 + 1e-9)
            || p.body_turns
                .windows(2)
                .any(|w| (w[1] - w[0]).abs() > 0.06 + 1e-9)
        {
            return Err("준비된 연주의 몸통 경로가 유효하지 않습니다.".into());
        }
        let times = if p
            .profile
            .as_ref()
            .is_some_and(|profile| profile.hand_frames.is_some())
        {
            control_times(&p.plan)?
        } else {
            Vec::new()
        };
        if p.hand_swivels.len() != times.len()
            || p.hand_swivels.iter().zip(times).any(|(s, time)| {
                !s.time.is_finite()
                    || s.time.to_bits() != time.to_bits()
                    || s.angles.iter().any(|a| {
                        !a.is_finite()
                            || a.abs() > hand_coordination::MAX_SWIVEL + 1e-9
                            || (a / hand_coordination::ANGLE_STEP
                                - (a / hand_coordination::ANGLE_STEP).round())
                            .abs()
                                > 1e-8
                    })
                    || (0..2).any(|limb| {
                        let range = p
                            .profile
                            .as_ref()
                            .and_then(|profile| profile.hand_frames.as_ref())
                            .and_then(|frames| frames[limb].palm_roll_range);
                        match (s.orientations[limb], range) {
                            (None, None) => false,
                            (Some(q), Some(_)) => !crate::rig::valid_orientation(q, 1e-8),
                            _ => true,
                        }
                    })
            })
            || p.hand_swivels.windows(2).any(|w| {
                w[1].time <= w[0].time
                    || (0..2).any(|i| {
                        (w[1].angles[i] - w[0].angles[i]).abs()
                            > hand_coordination::MAX_SAMPLE_RATE * (w[1].time - w[0].time) + 1e-9
                    })
            })
        {
            return Err("준비된 연주의 양손 경로가 유효하지 않습니다.".into());
        }
        if (p.hand_swivels.is_empty() && !p.hand_orientations.is_empty())
            || p.hand_orientations.len() < p.hand_swivels.len()
            || p.hand_orientations.len() > hand_coordination::MAX_CONTROL_KNOTS
            || p.hand_orientations
                .windows(2)
                .any(|w| w[1].time <= w[0].time)
            || p.hand_orientations.iter().any(|s| {
                !s.time.is_finite()
                    || p.hand_swivels
                        .first()
                        .is_none_or(|first| s.time < first.time)
                    || p.hand_swivels.last().is_none_or(|last| s.time > last.time)
                    || (0..2).any(|limb| {
                        let bounded = p
                            .profile
                            .as_ref()
                            .and_then(|p| p.hand_frames.as_ref())
                            .is_some_and(|frames| frames[limb].palm_roll_range.is_some());
                        match (s.orientations[limb], bounded) {
                            (None, false) => false,
                            (Some(q), true) => !crate::rig::valid_orientation(q, 1e-8),
                            _ => true,
                        }
                    })
            })
            || p.hand_swivels.iter().any(|original| {
                p.hand_orientations
                    .binary_search_by(|s| s.time.total_cmp(&original.time))
                    .map_or(true, |i| {
                        p.hand_orientations[i].orientations != original.orientations
                    })
            })
        {
            return Err("준비된 연주의 손 방향 제어점이 유효하지 않습니다.".into());
        }
        let equipment_times =
            if p.profile.is_some() && p.plan.kit.iter().any(|piece| piece.geometry.is_some()) {
                equipment_times(&p.plan)?
            } else {
                Vec::new()
            };
        if p.equipment_path.len() > MAX_EQUIPMENT_SAMPLES
            || (equipment_times.is_empty() && !p.equipment_path.is_empty())
            || equipment_times.iter().any(|time| {
                p.equipment_path
                    .binary_search_by(|s| s.time.total_cmp(time))
                    .map_or(true, |i| {
                        p.equipment_path[i].time.to_bits() != time.to_bits()
                    })
            })
            || p.equipment_path.iter().any(|s| {
                !s.time.is_finite()
                    || equipment_times.first().is_none_or(|start| s.time < *start)
                    || equipment_times.last().is_none_or(|end| s.time > *end)
                    || s.lift.iter().any(|v| {
                        !v.is_finite() || *v < 0.0 || *v > crate::equipment_motion::MAX_LIFT + 1e-9
                    })
                    || s.pitch
                        .iter()
                        .any(|v| !v.is_finite() || v.abs() > 0.60 + 1e-9)
            })
            || p.equipment_path.windows(2).any(|w| {
                let dt = w[1].time - w[0].time;
                dt <= 0.0
                    || (0..2).any(|i| {
                        (w[1].lift[i] - w[0].lift[i]).abs()
                            > crate::equipment_motion::LIFT_RATE * dt + 1e-9
                            || (w[1].pitch[i] - w[0].pitch[i]).abs()
                                > crate::equipment_motion::PITCH_RATE * dt + 1e-9
                    })
            })
            || p.plan.hits.iter().any(|hit| {
                hit.limb.filter(|&l| l < 2).is_some_and(|limb| {
                    !p.equipment_path.is_empty()
                        && p.equipment_path
                            .binary_search_by(|s| s.time.total_cmp(&hit.time))
                            .map_or(true, |index| p.equipment_path[index].lift[limb] != 0.0)
                })
            })
        {
            return Err("준비된 연주의 장비 접근 경로가 유효하지 않습니다.".into());
        }
        if let Some(profile) = &p.profile {
            for hit in &p.plan.hits {
                if let Some(limb) = hit.limb.filter(|&limb| limb < 2) {
                    let pitch =
                        crate::equipment_motion::sample(&p.equipment_path, hit.time, limb).1;
                    let turned = profile.turned(body_motion::at(&p.body_turns, hit.time));
                    let hands = hand_coordination::at_with_orientations(
                        &p.hand_swivels,
                        &p.hand_orientations,
                        hit.time,
                    );
                    let reachable = turned
                        .solve_hand_with_swivel_and_orientation(
                            limb,
                            hit.target,
                            pitch,
                            hands.angles[limb],
                            hands.orientations[limb],
                        )
                        .map_or_else(
                            || {
                                turned.hand_frames.is_none()
                                    && turned.reachable_at_pitch(limb, hit.target, pitch)
                            },
                            |hand| hand.reachable && super::distance(hand.tip, hit.target) < 1e-5,
                        );
                    if !reachable {
                        return Err(
                            "준비된 연주의 양손 경로가 원본 타격 접촉을 보존하지 않습니다.".into(),
                        );
                    }
                }
            }
        }
        // Reuse the complete plan/assignment/index validation path. Only expensive
        // torso/DP planning is skipped, using the already validated numeric tracks.
        let motion = Self::assemble_with_tracks(
            p.plan,
            p.profile,
            Some((
                p.body_turns,
                p.hand_swivels,
                p.hand_orientations,
                p.equipment_path,
            )),
        )?;
        // Paired-tool cues are source contacts too, although they are not Hits.
        if let Some(profile) = &motion.profile {
            for sample in &motion.hand_orientations {
                let controls = hand_coordination::at_with_orientations(
                    &motion.hand_swivels,
                    &motion.hand_orientations,
                    sample.time,
                );
                let turned = profile.turned(body_motion::at(&motion.body_turns, sample.time));
                for limb in 0..2 {
                    let (tip, pitch) = motion.stroke_at(limb, controls.time);
                    if controls.orientations[limb].is_some()
                        && turned
                            .solve_hand_with_swivel_and_orientation(
                                limb,
                                tip,
                                -pitch,
                                controls.angles[limb],
                                controls.orientations[limb],
                            )
                            .is_none_or(|hand| {
                                !turned.hand_orientation_policy_valid(limb, hand.rotation)
                            })
                    {
                        return Err("준비된 연주의 손 방향이 그립 정책과 일치하지 않습니다.".into());
                    }
                }
            }
            for cue in &motion.count_in.cues {
                let turned = profile.turned(body_motion::at(&motion.body_turns, cue.time));
                let controls = hand_coordination::at_with_orientations(
                    &motion.hand_swivels,
                    &motion.hand_orientations,
                    cue.time,
                );
                for limb in 0..2 {
                    let (tip, pitch) = motion.stroke_at(limb, cue.time);
                    if turned
                        .solve_hand_with_swivel_and_orientation(
                            limb,
                            tip,
                            -pitch,
                            controls.angles[limb],
                            controls.orientations[limb],
                        )
                        .is_none_or(|hand| {
                            !hand.reachable || super::distance(hand.tip, cue.contact_point) > 1e-5
                        })
                    {
                        return Err("준비된 연주의 카운트인 접촉이 보존되지 않았습니다.".into());
                    }
                }
            }
        }
        Ok(motion)
    }
}

#[cfg(test)]
mod tests {
    fn control(arm: crate::rig::Quat) -> crate::rig::OrientationControl {
        arm
    }
    use super::*;
    use crate::{build_plan, rig::HandControlFrame};
    fn sample(with_profile: bool) -> Motion {
        let track = [
            0, 0x99, 38, 100, 0x81, 0x70, 0x99, 42, 90, 0x81, 0x70, 0xff, 0x2f, 0,
        ];
        let mut bytes = b"MThd\0\0\0\x06\0\0\0\x01\x01\xe0MTrk".to_vec();
        bytes.extend((track.len() as u32).to_be_bytes());
        bytes.extend(track);
        let plan = build_plan(
            &bytes,
            &Selection {
                tracks: vec![0],
                ..Selection::default()
            },
        )
        .unwrap();
        if !with_profile {
            return Motion::new(plan).unwrap();
        }
        let mut profile = RigProfile::primitive();
        profile.torso_pivot = Some([0., 0.9, -0.1]);
        let frame = HandControlFrame {
            forearm: [0., 0., 1.],
            palm: [0., 1., 0.],
            desired_palm: [0., 1., 0.],
            palm_roll_range: None,
            palm_reference: None,
        };
        profile.hand_frames = Some([frame.clone(), frame]);
        Motion::with_profile(plan, profile).unwrap()
    }
    fn profile_json(m: &Motion) -> String {
        m.profile
            .as_ref()
            .map_or(String::new(), |p| serde_json::to_string(p).unwrap())
    }
    fn sample_equipment() -> Motion {
        let original = sample(true);
        let mut plan = original.plan;
        plan.kit.retain(|p| p.id == "snare" || p.id == "hihat");
        for piece in &mut plan.kit {
            piece.geometry = Some(crate::equipment::local_geometry(piece));
        }
        Motion::with_profile(plan, original.profile.unwrap()).unwrap()
    }
    fn sample_orientations(mixed: bool) -> Motion {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        let mut profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        if mixed {
            profile.hand_frames.as_mut().unwrap()[1].palm_roll_range = None;
        }
        let mut plan = sample(false).plan;
        crate::contact::apply(&mut plan, &profile);
        Motion::with_profile(plan, profile).unwrap()
    }
    #[test]
    fn worker_orientation_track_roundtrip_and_policy_validation() {
        for mixed in [false, true] {
            let original = sample_orientations(mixed);
            let text = original.prepared_json();
            let profile = profile_json(&original);
            let hash = &original.plan.midi_hash;
            let restored = Motion::from_prepared_json(&text, hash, &profile).unwrap();
            for time in [0.417, -0.5, 0., 0.25, 0.125, 0.75, -0.1, 0.417] {
                assert_eq!(original.evaluate_rig(time), restored.evaluate_rig(time));
            }
            let prepared: Prepared = serde_json::from_str(&text).unwrap();
            assert_eq!(original.hand_orientations, restored.hand_orientations);
            for case in 0..6 {
                let mut edited = prepared.clone();
                match case {
                    0 => {
                        edited.hand_orientations.remove(0);
                    }
                    1 => edited.hand_orientations[1].time = edited.hand_orientations[0].time,
                    2 => edited.hand_orientations[0].orientations[0] = None,
                    3 => {
                        edited.hand_orientations[0].orientations[0] =
                            Some(control([0., 0., 0., 2.]))
                    }
                    4 => edited.hand_orientations[0].orientations[0] = Some([0.; 4]),
                    _ => edited.hand_orientations.last_mut().unwrap().time += 1.,
                }
                edited.checksum = edited.checksum();
                assert!(Motion::from_prepared_json(
                    &serde_json::to_string(&edited).unwrap(),
                    hash,
                    &profile
                )
                .is_err());
            }
            assert!(prepared
                .hand_swivels
                .iter()
                .all(|s| s.orientations[0].is_some() && s.orientations[1].is_none() == mixed));
            let mut stale = prepared.clone();
            stale.hand_swivels[0].orientations[0] = Some(control([0.12345, 0., 0., 1.]));
            assert_ne!(stale.checksum(), prepared.checksum);
            assert!(Motion::from_prepared_json(
                &serde_json::to_string(&stale).unwrap(),
                hash,
                &profile
            )
            .is_err());
            for orientation in [
                None,
                Some(control([0., 0., 0., 10.])),
                Some(control([f64::NAN, 0., 0., 1.])),
            ] {
                let mut edited = prepared.clone();
                edited.hand_swivels[0].orientations[0] = orientation;
                edited.checksum = edited.checksum();
                assert!(Motion::from_prepared_json(
                    &serde_json::to_string(&edited).unwrap(),
                    hash,
                    &profile
                )
                .is_err());
            }
            if mixed {
                let mut edited = prepared.clone();
                edited.hand_swivels[0].orientations[1] = Some(control([0., 0., 0., 1.]));
                edited.checksum = edited.checksum();
                assert!(Motion::from_prepared_json(
                    &serde_json::to_string(&edited).unwrap(),
                    hash,
                    &profile
                )
                .is_err());
            }
            let mut missing: serde_json::Value = serde_json::from_str(&text).unwrap();
            missing["handSwivels"][0]
                .as_object_mut()
                .unwrap()
                .remove("orientations");
            assert!(Motion::from_prepared_json(&missing.to_string(), hash, &profile).is_err());
        }
    }
    #[test]
    fn exact_source_times_take_priority_over_nearby_sampling_grid() {
        let mut plan = sample(false).plan;
        plan.hits[0].time = 0.1;
        plan.hits[0].time_us = 100_000;
        assert_ne!(18.0f64 / 30.0 - PRE_ROLL, plan.hits[0].time);
        for times in [
            equipment_times(&plan).unwrap(),
            control_times(&plan).unwrap(),
        ] {
            assert!(times
                .iter()
                .any(|t| t.to_bits() == plan.hits[0].time.to_bits()));
            assert_eq!(times.iter().filter(|t| (**t - 0.1).abs() < 1e-9).count(), 1);
        }
    }
    #[test]
    fn refined_equipment_controls_preserve_mandatory_times_and_worker_poses() {
        let original = sample_equipment();
        let text = original.prepared_json();
        let mut prepared: Prepared = serde_json::from_str(&text).unwrap();
        let time = (prepared.equipment_path[0].time + prepared.equipment_path[1].time) * 0.5;
        let controls: [_; 2] = std::array::from_fn(|limb| {
            crate::equipment_motion::sample(&prepared.equipment_path, time, limb)
        });
        prepared.equipment_path.insert(
            1,
            PathSample {
                time,
                lift: controls.map(|s| s.0),
                pitch: controls.map(|s| s.1),
            },
        );
        prepared.checksum = prepared.checksum();
        let restored = Motion::from_prepared_json(
            &serde_json::to_string(&prepared).unwrap(),
            &original.plan.midi_hash,
            &profile_json(&original),
        )
        .unwrap();
        for t in [time, 0.0, 0.1, -0.5, 0.417, 0.75] {
            assert_eq!(original.evaluate_rig(t), restored.evaluate_rig(t));
        }
        // Adding controls is allowed; removing even one mandatory control is not.
        prepared.equipment_path.remove(0);
        prepared.checksum = prepared.checksum();
        assert!(Motion::from_prepared_json(
            &serde_json::to_string(&prepared).unwrap(),
            &original.plan.midi_hash,
            &profile_json(&original),
        )
        .is_err());
    }
    #[test]
    fn equipment_tracks_transfer_exactly_and_reject_recomputed_checksum_tampering() {
        let original = sample_equipment();
        assert!(!original.equipment_path.is_empty());
        let text = original.prepared_json();
        let profile = profile_json(&original);
        let hash = &original.plan.midi_hash;
        let restored = Motion::from_prepared_json(&text, hash, &profile).unwrap();
        assert_eq!(original.equipment_path, restored.equipment_path);
        for t in [0.0, 0.1, 0.25, -0.5, 0.417, 0.75, 0.1] {
            assert_eq!(original.evaluate_rig(t), restored.evaluate_rig(t));
            assert_eq!(original.inspect_pose(t), restored.inspect_pose(t));
        }
        let prepared: Prepared = serde_json::from_str(&text).unwrap();
        for change in 0..8 {
            let mut edited = prepared.clone();
            match change {
                0 => {
                    edited.equipment_path.pop();
                }
                1 => edited.equipment_path[0].time += 0.00001,
                2 => edited.equipment_path[0].lift[0] = -0.01,
                3 => edited.equipment_path[0].lift[0] = 0.21,
                4 => edited.equipment_path[0].pitch[0] = 0.61,
                5 => {
                    edited.equipment_path[0].pitch[0] = 0.6;
                    edited.equipment_path[1].pitch[0] = -0.6;
                }
                6 => {
                    for s in &mut edited.equipment_path {
                        s.lift = [0.01; 2];
                    }
                }
                _ => {
                    edited.equipment_path =
                        vec![edited.equipment_path[0]; MAX_EQUIPMENT_SAMPLES + 1]
                }
            }
            edited.checksum = edited.checksum();
            assert!(
                Motion::from_prepared_json(
                    &serde_json::to_string(&edited).unwrap(),
                    hash,
                    &profile
                )
                .is_err(),
                "tamper {change}"
            );
        }
    }
    #[test]
    fn worker_transfer_preserves_exact_poses_and_shuffled_evaluation() {
        for with_profile in [false, true] {
            let original = sample(with_profile);
            let restored = Motion::from_prepared_json(
                &original.prepared_json(),
                &original.plan.midi_hash,
                &profile_json(&original),
            )
            .unwrap();
            assert_eq!(
                serde_json::to_string(&original.plan).unwrap(),
                serde_json::to_string(&restored.plan).unwrap()
            );
            assert_eq!(original.body_turns, restored.body_turns);
            assert_eq!(original.hand_swivels, restored.hand_swivels);
            for t in [0.417, -0.5, 0.0, 0.25, 0.125, 0.75, -0.1, 0.417] {
                assert_eq!(original.evaluate_rig(t), restored.evaluate_rig(t));
                assert_eq!(original.inspect_pose(t), restored.inspect_pose(t));
            }
        }
    }
    #[test]
    fn oversized_runtime_tracks_stop_during_deserialization() {
        let original = sample(false);
        let mut p: Prepared = serde_json::from_str(&original.prepared_json()).unwrap();
        p.body_turns = vec![0.0; hand_coordination::MAX_CONTROL_KNOTS + 1];
        p.checksum = p.checksum();
        let error = Motion::from_prepared_json(
            &serde_json::to_string(&p).unwrap(),
            &original.plan.midi_hash,
            "",
        )
        .err()
        .unwrap();
        assert!(error.contains("planning budget"));
    }
    #[test]
    fn transfer_rejects_identity_checksum_and_structural_tampering() {
        let original = sample(true);
        let text = original.prepared_json();
        let profile = profile_json(&original);
        let hash = &original.plan.midi_hash;
        assert!(Motion::from_prepared_json(&text, "wrong-source", &profile).is_err());
        assert!(Motion::from_prepared_json(&text, hash, "").is_err());
        let prepared: Prepared = serde_json::from_str(&text).unwrap();
        // Neither earlier quaternion interpretation may be restored as the
        // new independent arm/grip control, even with a recomputed checksum.
        for version in [4, 5, 6] {
            let mut old_frame = prepared.clone();
            old_frame.format_version = version;
            old_frame.checksum = old_frame.checksum();
            assert!(Motion::from_prepared_json(
                &serde_json::to_string(&old_frame).unwrap(),
                hash,
                &profile
            )
            .is_err());
        }
        let mut edited = prepared.clone();
        edited.hand_swivels[0].angles[0] += 0.05;
        assert!(Motion::from_prepared_json(
            &serde_json::to_string(&edited).unwrap(),
            hash,
            &profile
        )
        .is_err());
        for change in 0..7 {
            let mut edited = prepared.clone();
            match change {
                0 => edited.format_version += 1,
                1 => edited.body_turns.pop().map(|_| ()).unwrap(),
                2 => edited.hand_swivels[0].angles[0] = 0.81,
                3 => edited.hand_swivels[1].time = edited.hand_swivels[0].time,
                4 => edited.plan.hits[0].limb = Some(3),
                5 => edited.plan.planner_version = "different-planner".into(),
                _ => {
                    edited.hand_swivels[0].angles[0] = -0.8;
                    edited.hand_swivels[1].angles[0] = 0.8;
                }
            }
            edited.checksum = edited.checksum();
            assert!(
                Motion::from_prepared_json(
                    &serde_json::to_string(&edited).unwrap(),
                    hash,
                    &profile
                )
                .is_err(),
                "tamper {change}"
            );
        }
    }
}
