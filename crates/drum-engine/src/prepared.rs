//! Opaque, version-bound worker transfer. This is not a persistent project format.
//! The checksum detects stale/mixed/accidentally edited messages; it is not an
//! authenticity signature. Independent validation remains mandatory on import.
use super::{validate_plan, Motion, PLANNER_VERSION};
use crate::{
    body_motion, equipment_motion::PathSample, hand_coordination, model::*, rig::RigProfile,
};
use hand_coordination::SwivelSample;
use serde::{Deserialize, Serialize};
const FORMAT_VERSION: u32 = 2;
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
        let equipment_times =
            if p.profile.is_some() && p.plan.kit.iter().any(|piece| piece.geometry.is_some()) {
                equipment_times(&p.plan)?
            } else {
                Vec::new()
            };
        if p.equipment_path.len() != equipment_times.len()
            || p.equipment_path
                .iter()
                .zip(equipment_times)
                .any(|(s, time)| {
                    !s.time.is_finite()
                        || s.time.to_bits() != time.to_bits()
                        || s.lift
                            .iter()
                            .any(|v| !v.is_finite() || *v < 0.0 || *v > 0.20 + 1e-9)
                        || s.pitch
                            .iter()
                            .any(|v| !v.is_finite() || v.abs() > 0.60 + 1e-9)
                })
            || p.equipment_path.windows(2).any(|w| {
                let dt = w[1].time - w[0].time;
                dt <= 0.0
                    || (0..2).any(|i| {
                        (w[1].lift[i] - w[0].lift[i]).abs() > 1.2 * dt + 1e-9
                            || (w[1].pitch[i] - w[0].pitch[i]).abs() > 2.0 * dt + 1e-9
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
                    if !profile
                        .turned(body_motion::at(&p.body_turns, hit.time))
                        .reachable(limb, hit.target)
                    {
                        return Err(
                            "준비된 연주의 몸통 회전이 원본 타격 접촉을 보존하지 않습니다.".into(),
                        );
                    }
                }
            }
        }
        // Reuse the complete plan/assignment/index validation path. Only expensive
        // torso/DP planning is skipped, using the already validated numeric tracks.
        Self::assemble_with_tracks(
            p.plan,
            p.profile,
            Some((p.body_turns, p.hand_swivels, p.equipment_path)),
        )
    }
}

#[cfg(test)]
mod tests {
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
