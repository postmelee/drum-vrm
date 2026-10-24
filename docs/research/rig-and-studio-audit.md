# Rig and studio engineering audit

Audit date: **2026-10-24 (Asia/Seoul)**.

- Repository baseline: [`e1b90358f2b51b99444b959c1e56a161253ee917`](https://github.com/postmelee/drum-vrm/commit/e1b90358f2b51b99444b959c1e56a161253ee917).
- Deployed v0.2.0 application: [`3e8be9333cedbc9845dbc3e4cebd73961af697b4`](https://github.com/postmelee/drum-vrm/commit/3e8be9333cedbc9845dbc3e4cebd73961af697b4).
- Status: **analysis and recommendations only**. This audit implements no feature, changes no runtime dependency or asset, and creates no application release or deployment. Recommendations below are proposed work, not supported product behavior or a new implementation/release approval.

This internal note compares the existing rig and motion implementation with the prior research register and primary VRM/three-vrm sources. It covers character and motion boundaries within the studio; UI and drum-model product planning is maintained separately. Read it alongside [architecture](../architecture.md), [ADR 0001](../decisions/0001-engine-and-rig-boundary.md), [research references](references.md), and [v0.2 verification](../verification/studio-0.2.md).

## Current implementation: observed facts

| Area | Evidence at the application baseline | Consequence |
|---|---|---|
| Pose representation | [`src/types.ts`](https://github.com/postmelee/drum-vrm/blob/3e8be9333cedbc9845dbc3e4cebd73961af697b4/src/types.ts) specifies 57 Float32 values: four limbs with root/middle/end/tip positions, plus pelvis/chest/head | These are **19 world-space points**, not 57 bones, local rotations, or a VRM pose. |
| Display rig | [`PrimitiveRigAdapter`](https://github.com/postmelee/drum-vrm/blob/3e8be9333cedbc9845dbc3e4cebd73961af697b4/src/scene.ts#L72) connects independent cylinder and sphere meshes | There is no anatomical `Bone` hierarchy, `Skeleton`, skin binding, or finger chain in this adapter. Its joint names are descriptive labels. |
| Head | The adapter creates a wireframe sphere named `headMarker`, then moves its containing group to the head point | This is a debug marker. Changing its shape does not establish or break VRM compatibility. A mesh-center marker is also not automatically the correct location of a VRM head joint. |
| Body calibration | [`motion.rs`](https://github.com/postmelee/drum-vrm/blob/3e8be9333cedbc9845dbc3e4cebd73961af697b4/crates/drum-engine/src/motion.rs#L5) fixes shoulder/hip roots and uses 0.43 m for every upper/lower arm and leg segment | Different character proportions are not measured or solved. The solver is a specific primitive body, not a general humanoid calibration. |
| Sticks and wrists | `STICK_VECTOR = [0, -0.09, 0.30]`; evaluation subtracts that world-space offset to obtain the wrist, then adds it back for the tip | Both sticks retain the same world orientation. There is no independent wrist stroke, palm frame, grip pivot, forearm twist, or finger motion. |
| Assignment | Beam width 8 inside simultaneous groups; bounded two-hit lookahead; reach, minimum interval, and tip-travel constraints | The independent planner preserves event timing and diagnoses unresolved hits. It does not model limb/kit collisions or establish biomechanical validity. |
| Motion | Smoothstep position interpolation plus a sine arc; velocity changes preparation/transition height; two-bone IK with fixed poles | Preparation/contact/recovery exists. Instrument-specific elasticity, articulated stick rotation, and coordinated torso/shoulder motion do not. |
| Secondary motion | Pelvis/chest points remain fixed; the head receives a small event-driven positional offset | This is not head orientation, gaze, facial animation, or a retrieved full-body performance. |
| Adapter calibration | `calibrate(kit)` validates/copies target data; `Motion::new` only accepts `primitive-v1` | The adapter interface is an extension boundary, but importing a VRM alone cannot make the current evaluator fit that model. |

The current implementation is useful as a deterministic contact/timing baseline. Existing contact and seek tests do not establish natural human performance, arbitrary-body retargeting, or correctness of every imported drum surface. The public visual is a skeleton-style diagnostic rig; describing it as an already implemented VRM-standard skeleton would overstate its capability.

## VRM requirements and compatibility gaps

The [VRM 1.0 humanoid specification](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/humanoid.md) maps semantic bones to glTF nodes and defines their hierarchy. Hips, spine, head, and the principal arm/leg chains are required. Fingers, shoulders, neck, and some torso bones are optional. A valid VRM can therefore lack the fingers needed for an articulated drum grip. A product-level grip requirement must be checked separately from basic VRM validity.

The standard's [T-pose definition](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/tpose.md) establishes +Z facing, grounded feet, arm/hand orientation, and positive uniform scale. It does not prescribe a wireframe head shape. Feet need a mesh-relative sole offset: the ankle node alone does not identify the ground-contact surface.

### Coordinate and anatomical-side migration

The existing contract declares +Y up and +Z forward, but its left arm/leg roots have negative X and right roots positive X. VRM's +Z-facing T-pose places the anatomical left hand toward +X and the right toward -X. The standard explicitly describes the corresponding thumb directions. Thus the current `left`/`right` labels must not be copied directly to VRM bones without resolving this discrepancy.

**Recommendation:** document a versioned legacy-to-canonical conversion policy before adding the VRM adapter. Transform/rebuild the related body, kit, targets, and limb identities consistently; do not silently reinterpret old plans. Test that the intended right foot still operates the kick and the intended left foot the hi-hat pedal. Do not fix this by applying a negative scale to humanoid bones; the VRM transform constraints prohibit that shortcut. Preserve MIDI event identity and timing through the conversion.

### Normalized bones, raw bones, and rest pose

VRM 1.0 permits model-specific rest rotations. The official [pose compatibility explanation](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm_animation-1.0/how_to_transform_human_pose.md) describes hips translation plus local bone rotations and explains conversion across different rest bases and optional-bone layouts. That explanation is explicitly non-normative guidance, not an additional mandatory specification.

The [three-vrm humanoid API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html) offers normalized proxies and the model's raw bones. Its default update transfers the normalized pose to the raw rig. `setNormalizedPose()` takes local transforms relative to rest; `normalizedRestPose` contains non-relative local transforms and cannot be passed directly to that setter. World positions/quaternions are not interchangeable with these local values.

The [official normalized-rig implementation](https://github.com/pixiv/three-vrm/blob/dev/packages/three-vrm-core/src/humanoid/VRMHumanoidRig.ts) records the original rest basis and converts normalized rotations back to the raw rig. This solves orientation conventions; it does **not** solve task-specific reach or preserve drum contact across different limb lengths. Simply applying the 57 world points as normalized bone translations is not a valid migration strategy.

## Proposed contract and migration

The following is a proposed design, not an implemented API. Preserve the existing `PerformancePlan`/`RigAdapter` responsibility split: Rust owns timing, assignment, motion, and diagnostics; the adapter interprets model metadata and applies the result to renderer objects. Rust must not depend on arbitrary Three.js object names.

### RigProfile

Create a serializable, versioned calibration input containing:

- A profile version and identity, model identity/hash, canonical coordinate convention, units, and anatomical-side convention.
- Semantic humanoid mapping/capabilities, required and optional bones, parent relationships, local/world rest bases, and measured segment lengths.
- Seat/root placement, shoulder and hip locations, joint limits/pole preferences, and permitted torso/shoulder adjustments.
- Per-hand grip translation/orientation, stick grip-to-tip transform and dimensions, and optional finger-pose profiles.
- Foot/sole geometry offsets, pedal contact locations/pivots, and kit target frames including surface orientation.

The model adapter should extract this information once; the Rust planner/evaluator should consume the numeric profile. Include model/profile/kit/solver versions in the calibration identity so a model or kit change invalidates cached motion. The current `primitive-v1` ID must retain its old interpretation for reproducibility.

### Versioned rotation and contact contract

Retain immutable source-event facts (`tick`, `timeUs`, source event identity, instrument, velocity). Separate those from calibrated results such as hand assignment, reach diagnostics, stroke paths, and solved pose. Recalculate calibrated results for a new body or kit; an old assignment is a preference unless explicitly locked by the user. An impossible contact remains a diagnostic and must not move the MIDI event in time.

A proposed pose v2 should represent hips/root placement and semantic, normalized local bone rotations, with explicit hand/foot orientation and tool/contact data. The exact packed format remains a design decision. Merely adding more world points cannot uniquely define bone twist or a palm frame; rest bases and a stable orientation policy are also necessary. Keep the v1 point output as a debug/compatibility projection while adapters migrate, rather than changing its 57-float meaning in place.

Maintain stateless `evaluate_at(t)`: the same plan, profile, and time must produce the same primary pose after arbitrary seeks and during fixed-frame export. Any future secondary simulation needs a deterministic sampling/reset policy before it can join that contract.

### Grip and stick attachment

Attach each rendered stick to a calibrated hand-local socket, with its real tip defined in stick coordinates. A practical adapter can use the final raw hand node after normalized pose application; its local grip transform must account for the model's rest basis. This is a project recommendation based on [Three.js local/world transform semantics](https://threejs.org/docs/pages/Object3D.html), not a VRM-defined drum-grip feature.

The intended relationship is `worldStick = worldHand × handGrip × gripStick`. Compute contact from the transformed mesh/tool tip, and solve hand position **and orientation** to satisfy it. Replacing a world-space offset with a local tool transform permits wrist rotation without detaching the stick from the hand. Do not independently overwrite the visible tip after the skeleton is solved: that would conceal attachment/contact errors.

Start with a calibrated static matched grip for models that provide finger bones. Then add wrist articulation and rebound; dynamic finger control can follow. Missing optional fingers need an explicit fallback or capability notice. A standard humanoid bone overlay can replace the current mesh-only diagnostic display independently of whether a skinned character is visible.

## Prior research: adoption audit

These comparisons distinguish similar ideas, independent implementation, and actual model/code adoption. No external research model, motion dataset, weights, or research source code was added by this audit or by the current motion implementation.

| Research | Implemented here | Missing / not a reproduction |
|---|---|---|
| [Implementation of a 3D Virtual Drummer, 2001](https://research.utwente.nl/files/83238062/Kragtwijk2001implementation.pdf) | Independent hand preference/alternation and preparation-contact-recovery concepts; velocity-dependent preparation height | Per-instrument elasticity, delayed elbow/wrist/stick interpolation, and gaze are absent. The paper itself says shortest-path assignment was not implemented; our bounded beam planner is not a reproduction of an implemented shortest-path method. Its event substitution/removal policies are not adopted because this project preserves source events. |
| [DRUMS, 2025](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf) | Modular separation of hand/foot/expression concerns; procedural feet and IK at a broad architectural level | No BiLSTM hand position/quaternion predictor, MIDI-matched upper-body/facial retrieval, MMIP training, or 57-bone performance system. Our **57 floats are 19 points**, unrelated to that bone count. The paper reports missing kit geometry as a limitation on verifying actual surface contact. |
| [Generalized Audio-Driven Synthesis of Precise Drummer Motion, 2026](https://arxiv.org/html/2608.19055v1) | The evaluation distinction between spatial contact and temporal alignment is relevant to our independent checks | No audio-conditioned diffusion, dual body-rotation/stick-position loss, research training pipeline, Impact Point Deviation evaluation, or Percussive Alignment Score implementation. Exact-time target-contact tests are not a reproduction of these metrics or evidence of equivalent motion quality. |

The most direct next motion improvements are calibrated wrist/tool orientation, instrument-dependent recovery, coordinated joint timing, and constrained torso/shoulder participation. Those can be evaluated before selecting an ML model. Learned or retrieved expression should remain a candidate generator subject to the same event-time and contact constraints.

## Proposed issue order and acceptance evidence

The following issues were open when checked on 2026-10-24. Follow-up planning split the minimum #23 measurement and basic #20 grip work into [#30](https://github.com/postmelee/drum-vrm/issues/30) in M3, alongside the #17 contract. The remaining #20/#23 scope stays in M4. This is a recommended refinement of their execution order, not an assertion that their scope is approved or complete. Keep the existing [#14 editing](https://github.com/postmelee/drum-vrm/issues/14) dependency when addressing user-controlled styles in #20.

| Order | Issue | Suggested scope and evidence |
|---|---|---|
| Begin alongside #17 | [#30 Minimum contact/grip baseline](https://github.com/postmelee/drum-vrm/issues/30), extracted from #23 | Establish a baseline before changing the solver. Record rendered stick-tip world transforms, target frames, event time, assigned limb, contact error, and trajectory samples. Compare identical seeks and export times. Complete richer recording/playback later if needed. |
| First compatibility step | [#17 VRM/common-rig adapter](https://github.com/postmelee/drum-vrm/issues/17) | Specify supported VRM 1.0 bones/rest pose, coordinate/side migration, normalized/raw conversion, optional-finger behavior, and model condition display. Verify actual deformation and tool attachment with a supported model. Common GLB needs its own explicit mapping/profile; a GLB extension alone is insufficient. |
| Then calibration | [#18 Body/seat/stick/target/pedal calibration](https://github.com/postmelee/drum-vrm/issues/18) | Apply the same immutable musical intent to at least two different proportions. Replan reach as necessary; measure actual tool contact, hand attachment, feet/pedals, and missing/unreachable targets. Establish profile/cache invalidation. |
| Then performance quality | [#20 Styles and expression](https://github.com/postmelee/drum-vrm/issues/20) | Add wrist-led strokes, instrument-dependent rebound, coordinated elbow/wrist timing, gaze, and constrained body motion. Compare motion continuity, limits/collisions, and contact before/after; retain original note timing. Prioritize special techniques separately. |
| Then model adoption | [#21 Learned models and motion data](https://github.com/postmelee/drum-vrm/issues/21) | Define remaining failures of the calibrated rule baseline; obtain reproducible candidate packages and separate code/data/weight/output permissions. Compare a constrained candidate against the same measured baseline before product adoption. |

For the #30 minimum baseline and the later #23 inspection tools, measure the **rendered tool endpoint**, not a second copy of the planner's target. Report spatial error separately from event-time error and motion quality. Include exact event times and a denser trajectory sample around impacts, because a 30 fps export does not necessarily sample the instant of contact. Contact tolerance and perceptual quality thresholds still need explicit acceptance decisions; this audit establishes no new measured performance claim.

## Sources, licenses, and adoption boundaries

Primary sources below were read on 2026-10-24. Live `master`/`dev` URLs describe the reviewed documents; they are not dependency pins. Before implementation, pin the adopted package release/commit and update the dependency and license registers.

| Material | Primary evidence | Rights/adoption status |
|---|---|---|
| VRM 1.0 specification and pose guidance | [Humanoid](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/humanoid.md), [T-pose](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/tpose.md), [pose compatibility](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm_animation-1.0/how_to_transform_human_pose.md) | Referenced and summarized; no specification files or sample model copied into the application. A specification is not a license for a user's avatar. |
| three-vrm source/library | [API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html), [normalized-rig source](https://github.com/pixiv/three-vrm/blob/dev/packages/three-vrm-core/src/humanoid/VRMHumanoidRig.ts), [MIT license](https://github.com/pixiv/three-vrm/blob/dev/LICENSE) | MIT, copyright pixiv. Not installed by this audit. Retain its notice if adopted; model/texture permissions remain separate. |
| Three.js transform API | [Object3D](https://threejs.org/docs/pages/Object3D.html) | Documentation reference for proposed attachment design. Existing Three.js version and notice remain in [dependency-lock.md](dependency-lock.md). |
| 2001 paper | [University author PDF](https://research.utwente.nl/files/83238062/Kragtwijk2001implementation.pdf), [publisher rights record](https://link.springer.com/chapter/10.1007/978-3-7091-6240-8_2) | Publisher records copyright 2001 Springer-Verlag Wien. No reusable author code package/license established. Concepts are independently implemented; paper/artwork/source are not bundled. |
| DRUMS paper and model | [Author project](https://www.theodoroskyriakou.com/projects/drums/index.html), [paper](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf) | Paper identifies CC BY 4.0. The inspected author page exposes paper/DOI/video, not a verified executable model/weight package and license. Paper permission does not authorize MMIP or weights. |
| Disney 2026 paper and model | [Official announcement](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/), [arXiv v1](https://arxiv.org/abs/2608.19055v1) | Official page restricts paper reposting; arXiv uses its non-exclusive distribution license. No code, weights, or dataset package with verified product-use terms was established from these sources. |
| MMIP motion data | [Prior R02 evidence and limits](references.md#r02-multi-modal-instrument-performances-mmip) | The earlier register records only partial terms verification. This audit does not upgrade that evidence or approve training/redistribution. No MMIP data is added. |

The application's own MIT license does not replace any of these separate rights. Avatar import must expose the relevant model metadata/conditions and distinguish rendered-video use from model redistribution. External research files, user avatars, and source MIDI stay outside the public repository unless separately approved and documented.

## Audit verification and limits

Completed work: source inspection at the stated baseline; comparison with the existing architecture/research records; read-only checks of issues #17/#18/#20/#21/#23; and primary-source review of VRM, three-vrm, and the three papers. This is not a new browser test, naturalness study, VRM runtime test, or research reproduction. Existing application verification remains in its dated verification reports.

Only this engineering document is produced by this audit task. No feature code, package, model, release tag, GitHub Release, or Pages deployment is part of it.


## Implementation follow-up (unreleased)

The historical audit above describes v0.2.0. [ADR 0004](../decisions/0004-vrm-humanoid-and-grip.md) records the implemented VRM 1.0 normalized/raw adapter, real semantic Bone default skeleton, measured segment/seat profile, wrist/tool orientation, fixed matched grip, and actual hand-local stick markers. [Model evidence](vrm-models.md) distinguishes preflight, headless bones, actual browser rendering/seek checks, and pending validation. three-vrm 3.5.5 is now adopted with its original MIT notice; separate avatar terms remain in force. No learned research model or external motion dataset is integrated.
