//! MIDI parsing, immutable performance planning and random-access rig evaluation.
//! Public JSON fields match src/types.ts; the pose buffer is an owned Float32Array.

mod body_motion;
pub mod clearance;
mod fit;
mod hand_coordination;
pub mod hihat;
pub mod interpretation;
mod midi;
pub mod model;
pub mod motion;
pub mod rig;

use model::{MidiInfo, PerformancePlan, Selection};
use motion::Motion;
use rig::RigProfile;
use wasm_bindgen::prelude::*;

pub fn inspect(bytes: &[u8]) -> Result<MidiInfo, String> {
    Ok(midi::parse(bytes)?.info)
}

pub fn build_plan(bytes: &[u8], selection: &Selection) -> Result<PerformancePlan, String> {
    let parsed = midi::parse(bytes)?;
    let mut plan = parsed.plan(bytes, selection)?;
    motion::assign(&mut plan);
    Ok(plan)
}

/// Pure Rust boundary shared by native regression tests and the WASM exports.
pub fn load_motion(plan_json: &str, profile_json: Option<&str>) -> Result<Motion, String> {
    if plan_json.len() > 128 * 1024 * 1024 {
        return Err("연주 계획 크기가 한계를 초과합니다.".into());
    }
    let mut plan: PerformancePlan =
        serde_json::from_str(plan_json).map_err(|e| format!("연주 계획 형식 오류: {e}"))?;
    interpretation::normalize_legacy(&mut plan);
    match profile_json {
        Some(json) => Motion::with_profile(plan, RigProfile::from_json(json)?),
        None => Motion::new(plan),
    }
}

#[wasm_bindgen]
pub fn inspect_midi(bytes: &[u8]) -> Result<String, JsValue> {
    inspect(bytes)
        .and_then(|info| serde_json::to_string(&info).map_err(|e| e.to_string()))
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn fit_kit(profile_json: &str, kit_json: &str) -> Result<String, JsValue> {
    fit::fit_kit_json(profile_json, kit_json).map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen]
pub struct Performance {
    motion: Motion,
}

#[wasm_bindgen]
impl Performance {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8], selection_json: &str) -> Result<Performance, JsValue> {
        let result = (|| {
            let selection: Selection = serde_json::from_str(selection_json)
                .map_err(|e| format!("트랙 선택 형식 오류: {e}"))?;
            Motion::new(build_plan(bytes, &selection)?)
        })();
        result
            .map(|motion| Self { motion })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn from_json(plan_json: &str) -> Result<Performance, JsValue> {
        load_motion(plan_json, None)
            .map(|motion| Self { motion })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn from_json_with_profile(
        plan_json: &str,
        profile_json: &str,
    ) -> Result<Performance, JsValue> {
        load_motion(plan_json, Some(profile_json))
            .map(|motion| Self { motion })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn plan_json(&self) -> String {
        // Every serializable number is validated before constructing Performance.
        serde_json::to_string(&self.motion.plan).expect("validated finite performance plan")
    }

    pub fn evaluate_at(&self, song_time: f64) -> Vec<f32> {
        self.motion.evaluate(song_time)
    }

    pub fn evaluate_rig_at(&self, song_time: f64) -> Vec<f32> {
        self.motion.evaluate_rig(song_time)
    }

    pub fn inspect_pose_at(&self, song_time: f64) -> String {
        self.motion.inspect_pose(song_time)
    }

    pub fn evaluate_hihat_at(&self, song_time: f64) -> f64 {
        hihat::evaluate(&self.motion.plan.controls, song_time)
    }
}

#[cfg(test)]
mod interpretation_tests;
#[cfg(test)]
mod phrase_tests;
#[cfg(test)]
mod tests;
