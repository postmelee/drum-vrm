use super::{
    BodyGeometry, HandCapsule, HandControlFrame, PalmReference, RigProfile, TorsoGeometry,
};

// Exact numeric identity for preparation-local query reuse. Metadata strings
// cannot affect numeric solving; every numeric value and optional shape is
// included, including signed zero. Exhaustive destructuring makes new profile
// fields an explicit cache-contract decision rather than a silent omission.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct NumericProfileKey {
    values: [u64; 256],
    len: usize,
}
impl std::hash::Hash for NumericProfileKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.values[..self.len].hash(state);
    }
}
impl RigProfile {
    pub(crate) fn numeric_key(&self) -> NumericProfileKey {
        let Self {
            schema_version,
            id: _,
            coordinate_system: _,
            roots,
            lengths,
            rest_tips,
            bend_poles,
            grip_offsets,
            tip_offsets,
            contact_rotations,
            hand_frames,
            torso_pivot,
            body_geometry,
            pelvis,
            chest,
            head,
        } = self;
        let mut key = NumericProfileKey {
            values: [0; 256],
            len: 0,
        };
        let mut push = |value| {
            key.values[key.len] = value;
            key.len += 1;
        };
        push(u64::from(*schema_version));
        for points in [roots, rest_tips, bend_poles, grip_offsets, tip_offsets] {
            for value in points.iter().flatten() {
                push(value.to_bits());
            }
        }
        for value in lengths
            .iter()
            .flatten()
            .chain(contact_rotations.iter().flatten())
        {
            push(value.to_bits());
        }
        push(u64::from(hand_frames.is_some()));
        if let Some(frames) = hand_frames {
            for HandControlFrame {
                forearm,
                palm,
                desired_palm,
                palm_roll_range,
                palm_reference,
            } in frames
            {
                for value in forearm.iter().chain(palm).chain(desired_palm) {
                    push(value.to_bits());
                }
                push(u64::from(palm_roll_range.is_some()));
                if let Some(range) = palm_roll_range {
                    for value in range {
                        push(value.to_bits());
                    }
                }
                push(match palm_reference {
                    None => 0,
                    Some(PalmReference::ShaftHeading) => 1,
                });
            }
        }
        push(u64::from(torso_pivot.is_some()));
        if let Some(pivot) = torso_pivot {
            for value in pivot {
                push(value.to_bits());
            }
        }
        for value in pelvis.iter().chain(chest).chain(head) {
            push(value.to_bits());
        }
        push(u64::from(body_geometry.is_some()));
        if let Some(BodyGeometry {
            version,
            source: _,
            torso,
            arm_radii,
            hand_radii,
            leg_radii,
            foot_radii,
            leg_boxes,
            leg_rest_frames,
            foot_boxes,
            hand_centers,
            hand_capsules,
            sample_count,
        }) = body_geometry
        {
            push(u64::from(*version));
            let TorsoGeometry {
                center,
                half_extents,
            } = torso;
            for value in center
                .iter()
                .chain(half_extents)
                .chain(arm_radii.iter().flatten())
                .chain(hand_radii)
            {
                push(value.to_bits());
            }
            push(u64::from(leg_radii.is_some()));
            if let Some(radii) = leg_radii {
                for value in radii.iter().flatten() {
                    push(value.to_bits());
                }
            }
            push(u64::from(foot_radii.is_some()));
            if let Some(radii) = foot_radii {
                for value in radii {
                    push(value.to_bits());
                }
            }
            push(u64::from(leg_boxes.is_some()));
            if let Some(boxes) = leg_boxes {
                for TorsoGeometry {
                    center,
                    half_extents,
                } in boxes.iter().flatten()
                {
                    for value in center.iter().chain(half_extents) {
                        push(value.to_bits());
                    }
                }
            }
            push(u64::from(leg_rest_frames.is_some()));
            if let Some(frames) = leg_rest_frames {
                for value in frames.iter().flatten().flatten() {
                    push(value.to_bits());
                }
            }
            push(u64::from(foot_boxes.is_some()));
            if let Some(boxes) = foot_boxes {
                for TorsoGeometry {
                    center,
                    half_extents,
                } in boxes
                {
                    for value in center.iter().chain(half_extents) {
                        push(value.to_bits());
                    }
                }
            }
            push(u64::from(hand_centers.is_some()));
            if let Some(centers) = hand_centers {
                for value in centers.iter().flatten() {
                    push(value.to_bits());
                }
            }
            push(u64::from(hand_capsules.is_some()));
            if let Some(capsules) = hand_capsules {
                for HandCapsule { start, end, radius } in capsules {
                    for value in start.iter().chain(end).chain(std::iter::once(radius)) {
                        push(value.to_bits());
                    }
                }
            }
            push(u64::from(*sample_count));
        }
        key
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn identity_includes_every_numeric_field_and_optional_shape() {
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/native-reference.json")).unwrap();
        let mut profile: RigProfile = serde_json::from_value(fixture["profile"].clone()).unwrap();
        let shape = TorsoGeometry {
            center: [0.1, 0.2, 0.3],
            half_extents: [0.2; 3],
        };
        profile.body_geometry = Some(BodyGeometry {
            version: 1,
            source: "skeleton-fallback".into(),
            torso: shape.clone(),
            arm_radii: [[0.02; 2]; 2],
            hand_radii: [0.02; 2],
            leg_radii: Some([[0.04; 2]; 2]),
            foot_radii: Some([0.03; 2]),
            leg_boxes: Some(std::array::from_fn(|_| {
                std::array::from_fn(|_| shape.clone())
            })),
            leg_rest_frames: Some([[[0., 0., 0., 1.]; 2]; 2]),
            foot_boxes: Some([shape.clone(), shape]),
            hand_centers: Some([[0.01; 3]; 2]),
            hand_capsules: Some(std::array::from_fn(|_| HandCapsule {
                start: [0.; 3],
                end: [0.1; 3],
                radius: 0.02,
            })),
            sample_count: 10,
        });
        let value = serde_json::to_value(&profile).unwrap();
        fn numeric_paths(value: &Value, prefix: String, paths: &mut Vec<String>) {
            match value {
                Value::Object(fields) => {
                    for (field, child) in fields {
                        numeric_paths(child, format!("{prefix}/{field}"), paths);
                    }
                }
                Value::Array(values) => {
                    for (index, child) in values.iter().enumerate() {
                        numeric_paths(child, format!("{prefix}/{index}"), paths);
                    }
                }
                Value::Number(_) => paths.push(prefix),
                _ => {}
            }
        }
        let mut paths = Vec::new();
        numeric_paths(&value, String::new(), &mut paths);
        let original = profile.numeric_key();
        assert!(paths.len() > 180);
        for path in paths {
            let mut changed = value.clone();
            let number = changed.pointer_mut(&path).unwrap();
            *number = if let Some(integer) = number.as_u64() {
                Value::from(integer + 1)
            } else {
                Value::from(number.as_f64().unwrap() + 0.001)
            };
            let next: RigProfile = serde_json::from_value(changed).unwrap();
            assert!(
                next.numeric_key() != original,
                "ignored numeric field {path}"
            );
        }
        for path in [
            "/handFrames",
            "/torsoPivot",
            "/bodyGeometry/handCapsules",
            "/bodyGeometry/legRestFrames",
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(path).unwrap() = Value::Null;
            let next: RigProfile = serde_json::from_value(changed).unwrap();
            assert!(
                next.numeric_key() != original,
                "ignored optional shape {path}"
            );
        }
        profile.roots[0][0] = 0.;
        let positive_zero = profile.numeric_key();
        profile.roots[0][0] = -0.;
        assert!(profile.numeric_key() != positive_zero);
    }
}
