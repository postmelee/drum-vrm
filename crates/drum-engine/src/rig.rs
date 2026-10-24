//! Numeric calibration only: no renderer objects, bone names, or mutable IK state.
use crate::model::{Vec3, POSE_FLOATS};
use crate::motion::{add, distance, sub, ARM_ROOTS, LEG_ROOTS, STICK_VECTOR};
use serde::{Deserialize, Serialize};

pub type Quat = [f64; 4]; // xyzw, local tool frame -> legacy world frame
pub const RIG_POSE_FLOATS: usize = POSE_FLOATS + 16;
pub const COORDINATE_SYSTEM: &str = "legacy-y-up-z-forward-left-negative-x";
pub const MAX_PROFILE_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigProfile {
    pub schema_version: u32,
    pub id: String,
    pub coordinate_system: String,
    /// LH, RH, LF, RF; upper-arm/upper-leg world origins in meters.
    pub roots: [Vec3; 4],
    pub lengths: [[f64; 2]; 4],
    pub rest_tips: [Vec3; 4],
    /// World direction from the root toward the intended bend side.
    pub bend_poles: [Vec3; 4],
    /// End joint -> grip/sole and grip/sole -> actual rendered contact point.
    /// Both are in the local tool frame and rotate together.
    pub grip_offsets: [Vec3; 4],
    pub tip_offsets: [Vec3; 4],
    pub contact_rotations: [Quat; 4],
    pub pelvis: Vec3,
    pub chest: Vec3,
    pub head: Vec3,
}

pub fn rotate(q: Quat, v: Vec3) -> Vec3 {
    let u = [q[0], q[1], q[2]];
    let cross = |a: Vec3, b: Vec3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let t = cross(u, v).map(|x| x * 2.0);
    let c = cross(u, t);
    std::array::from_fn(|i| v[i] + q[3] * t[i] + c[i])
}

/// Baseline world orientation composed with a local X wrist/ankle pitch.
pub fn pitch_rotation(base: Quat, pitch: f64) -> Quat {
    let (s, c) = (pitch * 0.5).sin_cos();
    [
        base[0] * c + base[3] * s,
        base[1] * c + base[2] * s,
        base[2] * c - base[1] * s,
        base[3] * c - base[0] * s,
    ]
}

impl RigProfile {
    /// Numeric reference for tests/adapters; legacy constructors do not opt into
    /// its rotating-tool motion, preserving the original v1 position contract.
    pub fn primitive() -> Self {
        Self {
            schema_version: 1,
            id: "primitive-v2".into(),
            coordinate_system: COORDINATE_SYSTEM.into(),
            roots: [ARM_ROOTS[0], ARM_ROOTS[1], LEG_ROOTS[0], LEG_ROOTS[1]],
            lengths: [[0.43, 0.43]; 4],
            rest_tips: std::array::from_fn(crate::motion::rest_tip),
            bend_poles: [
                [-1.0, -0.5, -0.25],
                [1.0, -0.5, -0.25],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            grip_offsets: [[0.0; 3]; 4],
            tip_offsets: [
                STICK_VECTOR,
                STICK_VECTOR,
                [0.0, 0.0, 0.20],
                [0.0, 0.0, 0.20],
            ],
            contact_rotations: [[0.0, 0.0, 0.0, 1.0]; 4],
            pelvis: [0.0, 0.71, -0.15],
            chest: [0.0, 1.24, -0.10],
            head: [0.0, 1.62, -0.10],
        }
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > MAX_PROFILE_BYTES {
            return Err("리그 프로필이 64 KiB 한계를 초과합니다.".into());
        }
        let profile: Self =
            serde_json::from_str(json).map_err(|e| format!("리그 프로필 형식 오류: {e}"))?;
        profile.validate()?;
        Ok(profile)
    }

    pub fn validate(&self) -> Result<(), String> {
        let point = |v: &Vec3| v.iter().all(|x| x.is_finite() && x.abs() <= 10.0);
        if self.schema_version != 1
            || self.coordinate_system != COORDINATE_SYSTEM
            || self.id.is_empty()
            || self.id.len() > 128
            || self.id == "primitive-v1"
            || !self
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c))
        {
            return Err("지원하지 않는 리그 프로필 버전·좌표계·ID입니다.".into());
        }
        if ![self.pelvis, self.chest, self.head].iter().all(point) {
            return Err("리그 몸통 좌표가 유한한 미터 범위를 벗어납니다.".into());
        }
        for limb in 0..4 {
            let q = self.contact_rotations[limb];
            let q_norm: f64 = q.iter().map(|x| x * x).sum();
            if !point(&self.roots[limb])
                || !point(&self.rest_tips[limb])
                || !point(&self.bend_poles[limb])
                || distance(self.bend_poles[limb], [0.0; 3]) < 1e-5
                || self.lengths[limb]
                    .iter()
                    .any(|x| !x.is_finite() || !(0.03..=2.0).contains(x))
                || [self.grip_offsets[limb], self.tip_offsets[limb]]
                    .iter()
                    .any(|v| !point(v) || distance(*v, [0.0; 3]) > 2.0)
                || !q_norm.is_finite()
                || (q_norm - 1.0).abs() > 1e-6
            {
                return Err(format!(
                    "리그 손발 {limb}의 길이·그립·단위 회전이 유효하지 않습니다."
                ));
            }
            if !self.reachable(limb, self.rest_tips[limb]) {
                return Err(format!(
                    "리그 손발 {limb}의 준비 자세가 도달 범위를 벗어납니다."
                ));
            }
        }
        Ok(())
    }

    pub fn offset(&self, limb: usize, rotation: Quat) -> Vec3 {
        rotate(
            rotation,
            add(self.grip_offsets[limb], self.tip_offsets[limb]),
        )
    }

    pub fn reachable(&self, limb: usize, tip: Vec3) -> bool {
        let d = distance(
            self.roots[limb],
            sub(tip, self.offset(limb, self.contact_rotations[limb])),
        );
        let [a, b] = self.lengths[limb];
        d >= (a - b).abs() + 1e-5 && d <= a + b - 1e-4
    }
}
