# ADR 0004 — VRM 1.0 humanoid, numeric profiles, and rendered contacts

Status: implemented, unreleased. 2026-10-24. Issues #17, #18, #30. Supersedes the fixed-body assumption in ADR 0001 for the new adapter; the legacy evaluator remains available for comparison.

## Decision

Use the VRM 1.0 semantic humanoid hierarchy and pinned `@pixiv/three-vrm` 3.5.5 normalized-to-raw retargeting. The built-in skeleton uses real `THREE.Bone` nodes, including optional finger chains, through the same normalized solver. It is a reference skeleton, not a redistributed character model. Local `.vrm` import is separate from static drum `.glb` replacement. Generic humanoid GLB mapping and VRM 0.x conversion are not implemented by this change.

The legacy engine basis places anatomical left at negative X; native VRM places it at positive X. Convert numeric positions with `[-x,y,z]` and unit quaternions with `[x,-y,-z,w]`. Only legacy kit/debug rendering uses the reflected display root. Native VRM bones remain under a positive-scale scene root. Reject nonuniform/negative skeletal transforms rather than silently stretching the avatar.

## Profile and pose boundary

`RigProfile` schema 1 records an explicit coordinate-system tag, stable identity, four roots, unequal segment lengths, bend poles, rest tips, local grip and tip offsets, contact tool quaternions, and pelvis/chest/head. The adapter measures actual rest distances and derives a pedal-aware seat height. No body segment is scaled to fake reach. User-adjustable seat, grip and kit calibration remain #18.

Rust `Performance.from_json_with_profile(planJson, profileJson)` recalculates body-dependent assignment and diagnostics while preserving every original hit identity/time/target. Profiles are supplied separately; `calibrationId` in a plan is not enough to restore its calibration without that JSON. `evaluate_rig_at(t)` returns 73 floats: the original 57 world-position values, then four xyzw tool quaternions. `evaluate_at(t)` remains a 57-value projection; legacy `from_json` retains primitive-v1 behavior.

The quaternion maps the local tool frame to engine world space, not directly to a raw VRM bone. Hand-to-tool socket orientation and raw rest basis are applied explicitly. Stick meshes and their terminal markers are children of the actual raw hand. A fixed matched-grip finger pose is applied where bones exist. Missing optional fingers reduce grip detail; they do not reject a valid humanoid. No generic skin/finger collision guarantee is made.

The contact inspector compares actual world transforms of these markers to the evaluated pose. At an assigned hit, validation separately compares them to the immutable hit target. A foot marker is a foot-local sole socket based on foot/toe geometry, not a collision measurement against shoe vertices. Unreachable hits retain sound and timing and produce diagnostics.

## Determinism

Every evaluation resets the normalized pose, solves world-to-parent-local rotations, applies the fixed grip, and updates the VRM at delta 0. Spring state resets before evaluation and is not integrated. Look-at and expressions use neutral state; node constraints are evaluated. The same adapter/evaluator is used for live playback, arbitrary seek and fixed-frame export. First-person rendering uses the VRM first-person mesh rules and a profile-derived eye camera.

Drum response is a separate optional visual layer based on absolute song time and velocity. Shell/cymbal pivots preserve the calibrated contact at impact; stand hardware is fixed. It does not alter MIDI, the body plan, or synthesized audio.

## Input, rights, and evidence

Preflight accepts self-contained VRM 1.0 GLB, up to 50 MiB, with 15 required bones and optional finger reporting. Embedded PNG/JPEG resources have image count/dimension/pixel budgets. External resources and malformed hierarchies are rejected before loading. Metadata is displayed as text; omitted optional rights show the restrictive specification default explicitly. Required metadata omissions and malformed rights produce visible warnings. The source page and additional terms still govern use.

Library code is MIT; avatars have independent licenses. No third-party avatar or source MIDI is bundled. Model hashes, terms and the limited real-model results are in [VRM model research](../research/vrm-models.md). Public user guidance is maintained in [English](../models.md) and [Korean](../models.ko.md). Validation is recorded separately from research and from release approval.

This independently implements IK/contact/grip concepts discussed in the research audit. No DRUMS/Disney code, data, motion or weights are integrated, and the change is not a reproduction of those papers or proof of natural human motion. Advanced technique, wrist dynamics, collisions and expression remain #20/#21/#23.
