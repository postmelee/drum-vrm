//! Numeric calibration only: no renderer objects, bone names, or mutable IK state.
use crate::model::{Vec3, POSE_FLOATS};
use crate::motion::{add, distance, sub, ARM_ROOTS, LEG_ROOTS, STICK_VECTOR};
use serde::{Deserialize, Serialize};

#[path = "hand_solver.rs"]
mod hand_solver;
pub use hand_solver::HandSolution;

pub type Quat = [f64; 4]; // xyzw, local tool frame -> legacy world frame
pub const RIG_POSE_FLOATS: usize = POSE_FLOATS + 20;
pub const COORDINATE_SYSTEM: &str = "legacy-y-up-z-forward-left-negative-x";
pub const MAX_PROFILE_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandControlFrame {
    /// Neutral world directions measured from the semantic hand, not raw axes.
    pub forearm: Vec3,
    pub palm: Vec3,
    pub desired_palm: Vec3,
}

/// Numeric envelope measurements in the profile's world basis. The torso axes
/// follow shoulder-right / up / body-forward; this is not a skinned mesh.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TorsoGeometry {
    pub center: Vec3,
    pub half_extents: Vec3,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandCapsule {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BodyGeometry {
    pub version: u32,
    pub source: String,
    pub torso: TorsoGeometry,
    pub arm_radii: [[f64; 2]; 2],
    pub hand_radii: [f64; 2],
    /// Optional measured thigh/shin and footwear envelope radii. Older profiles
    /// use an explicit segment-proportion fallback rather than zero thickness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leg_radii: Option<[[f64; 2]; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foot_radii: Option<[f64; 2]>,
    /// Upper/lower-leg boxes in their segment-local orthonormal frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leg_boxes: Option<[[TorsoGeometry; 2]; 2]>,
    /// Local box basis in measured rest-world coordinates. Its +Y is the
    /// measured segment direction; transport by the renderer's minimum swing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leg_rest_frames: Option<[[Quat; 2]; 2]>,
    /// Footwear boxes in the neutral tool frame, relative to the ankle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foot_boxes: Option<[TorsoGeometry; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_centers: Option<[Vec3; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_capsules: Option<[HandCapsule; 2]>,
    pub sample_count: u32,
}

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
    /// Optional opt-in to target-dependent neutral wrists. Old profiles retain
    /// their fixed world rotation contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hand_frames: Option<[HandControlFrame; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub torso_pivot: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_geometry: Option<BodyGeometry>,
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
            hand_frames: None,
            torso_pivot: None,
            body_geometry: None,
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
        if ![self.pelvis, self.chest, self.head].iter().all(point)
            || self.torso_pivot.as_ref().is_some_and(|v| !point(v))
        {
            return Err("리그 몸통 좌표가 유한한 미터 범위를 벗어납니다.".into());
        }
        if let Some(body) = &self.body_geometry {
            let radius = |r: &f64| r.is_finite() && (0.005..=0.25).contains(r);
            if body.version != 1
                || !matches!(body.source.as_str(), "mesh-estimate" | "skeleton-fallback")
                || !point(&body.torso.center)
                || body
                    .torso
                    .half_extents
                    .iter()
                    .any(|r| !r.is_finite() || !(0.01..=1.0).contains(r))
                || !body.arm_radii.iter().flatten().all(radius)
                || !body.hand_radii.iter().all(radius)
                || body
                    .leg_radii
                    .as_ref()
                    .is_some_and(|r| !r.iter().flatten().all(radius))
                || body
                    .foot_radii
                    .as_ref()
                    .is_some_and(|r| !r.iter().all(radius))
                || body.leg_boxes.as_ref().is_some_and(|legs| {
                    legs.iter().flatten().any(|b| {
                        !point(&b.center)
                            || distance(b.center, [0.; 3]) > 3.
                            || b.half_extents
                                .iter()
                                .any(|v| !v.is_finite() || !(0.001..=1.).contains(v))
                    })
                })
                || body.foot_boxes.as_ref().is_some_and(|feet| {
                    feet.iter().any(|b| {
                        !point(&b.center)
                            || distance(b.center, [0.; 3]) > 3.
                            || b.half_extents
                                .iter()
                                .any(|v| !v.is_finite() || !(0.001..=1.).contains(v))
                    })
                })
                || body.leg_rest_frames.as_ref().is_some_and(|frames| {
                    body.leg_boxes.is_none()
                        || frames.iter().flatten().any(|q| {
                            q.iter()
                                .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
                                || (q.iter().map(|v| v * v).sum::<f64>() - 1.).abs() > 0.001
                        })
                })
                || body.hand_centers.as_ref().is_some_and(|centers| {
                    centers
                        .iter()
                        .any(|c| !point(c) || distance(*c, [0.0; 3]) > 0.5)
                })
                || body.hand_capsules.as_ref().is_some_and(|capsules| {
                    capsules.iter().any(|c| {
                        !point(&c.start)
                            || !point(&c.end)
                            || distance(c.start, [0.0; 3]) > 0.5
                            || distance(c.end, [0.0; 3]) > 0.5
                            || !radius(&c.radius)
                    })
                })
                || body.sample_count > 1_000_000
            {
                return Err("리그 충돌 형상 측정값이 지원 범위를 벗어납니다.".into());
            }
        }
        if let Some(frames) = &self.hand_frames {
            for frame in frames {
                let unit = |v: Vec3| point(&v) && (distance(v, [0.0; 3]) - 1.0).abs() < 1e-6;
                let perpendicular: f64 = frame
                    .forearm
                    .iter()
                    .zip(frame.palm)
                    .map(|(a, b)| a * b)
                    .sum();
                if !unit(frame.forearm)
                    || !unit(frame.palm)
                    || !unit(frame.desired_palm)
                    || perpendicular.abs() > 1e-6
                {
                    return Err("손목 기준 축은 유한한 직교 단위 방향이어야 합니다.".into());
                }
            }
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
            // Pedals are desired kit contacts, so user seat calibration can
            // legitimately make a foot unreachable. Motion clamps the leg IK
            // and reports that condition instead of rejecting the whole rig.
            if limb < 2 && !self.reachable(limb, self.rest_tips[limb]) {
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

    pub fn solve_hand(&self, limb: usize, tip: Vec3, pitch: f64) -> Option<HandSolution> {
        if limb >= 2 {
            return None;
        }
        self.hand_frames
            .as_ref()
            .map(|frames| hand_solver::solve(self, limb, tip, pitch, &frames[limb]))
    }
    pub fn solve_hand_with_swivel(
        &self,
        limb: usize,
        tip: Vec3,
        pitch: f64,
        swivel: f64,
    ) -> Option<HandSolution> {
        if limb >= 2 {
            return None;
        }
        self.hand_frames.as_ref().map(|frames| {
            hand_solver::solve_with_swivel(self, limb, tip, pitch, &frames[limb], swivel)
        })
    }

    pub fn reachable(&self, limb: usize, tip: Vec3) -> bool {
        self.reachable_at_pitch(limb, tip, 0.)
    }

    /// Exact annulus of the same pitched compound link used by the hand solve.
    /// Feasibility searches may compare several pitches; a committed pose must
    /// be checked with its actual pitch rather than the existence of another.
    pub fn reachable_at_pitch(&self, limb: usize, tip: Vec3, pitch: f64) -> bool {
        if limb >= 4 || !pitch.is_finite() || tip.iter().any(|value| !value.is_finite()) {
            return false;
        }
        let rotation = pitch_rotation(self.contact_rotations[limb], pitch);
        if limb < 2 {
            if let Some(frames) = &self.hand_frames {
                let lower = self.lengths[limb][1];
                let compound = add(
                    frames[limb].forearm.map(|v| v * lower),
                    self.offset(limb, rotation),
                );
                let length = distance(compound, [0.0; 3]);
                let upper = self.lengths[limb][0];
                let d = distance(self.roots[limb], tip);
                return length >= 1e-6
                    && d >= (upper - length).abs() + 1e-5
                    && d <= upper + length - 1e-4;
            }
        }
        let d = distance(self.roots[limb], sub(tip, self.offset(limb, rotation)));
        let [a, b] = self.lengths[limb];
        d >= (a - b).abs() + 1e-5 && d <= a + b - 1e-4
    }
}

impl RigProfile {
    pub fn turned(&self, yaw: f64) -> Self {
        let Some(pivot) = self.torso_pivot else {
            return self.clone();
        };
        let mut p = self.clone();
        let q = [0.0, (yaw * 0.5).sin(), 0.0, (yaw * 0.5).cos()];
        let turn = |v| add(pivot, rotate(q, sub(v, pivot)));
        for limb in 0..2 {
            p.roots[limb] = turn(p.roots[limb]);
            p.bend_poles[limb] = rotate(q, p.bend_poles[limb]);
        }
        if let Some(body) = &mut p.body_geometry {
            body.torso.center = turn(body.torso.center);
        }
        p.chest = turn(p.chest);
        p.head = turn(p.head);
        p
    }
}
