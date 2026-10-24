# ADR 0005 — Shared calibration, planted pedals, and replaceable drum bodies

Status: accepted; implemented and locally verified, unreleased. 2026-10-24. Issues [#18](https://github.com/postmelee/drum-vrm/issues/18), [#14](https://github.com/postmelee/drum-vrm/issues/14), and [#20](https://github.com/postmelee/drum-vrm/issues/20). Extends [ADR 0004](0004-vrm-humanoid-and-grip.md). PR #33 merged at `a2105a5508ae92e49d31977e2c56b096dfd1d822`; it is not a Pages deployment.

## Context and execution order

The owner requested feet on their pedals before playback, seat and target adjustment for different proportions, a visible stick grip, and a common hardware layer that survives drum-body changes. The order is (1) shared body/seat/target calibration and planted feet, (2) stable hardware with replaceable shells/cymbals, then (3) explicit playing styles and collision-aware crossed arms. No model-name-specific animation branch is permitted. These are separate from release approval.

## Calibration contract

- Measure semantic humanoid joints and fixed segment lengths once from the imported rest pose. Translate the seated body rather than scaling bones to achieve reach. Default skeleton and VRM 1.0 use the same adapter path.
- Expose seat height **0.30–1.20 m** and forward position **−0.50–0.50 m** in centimetres in the UI. An automatic action derives a pedal-aware seat from measured proportions; it is a starting point rather than a proof that every target is reachable.
- Expose XYZ contact positions for the selected kit role. Closed/open hi-hat share one surface position; the hi-hat foot pedal is a separate contact. Kick's engine target is its foot-pedal contact, not its visible front head.
- Calibration changes update `plan.kit` and all corresponding `hit.target` values together, then rebuild the body-dependent `Performance` evaluator. Preserve original IDs, ticks, track/event indices, notes, velocities and times. Audio remains unchanged. A visual appearance change does not alter the performance plan.
- Re-evaluate the current absolute song time after changes. Preview, arbitrary seek and fixed-frame export must consume the same calibrated plan/profile. A failed update must not leave a partially applied body/plan pair.
- Store settings for the current session only. Reload persistence/project migration remains #15. Keep adjustments unavailable while a conflicting preparation/import/export operation is active.

Seat translation alone cannot fit every arm-to-leg ratio while a kit remains fixed. Users can adjust seat and target layout together. The solver must report unreachable contacts rather than conceal the condition by stretching limbs or changing MIDI times. Skin/sole offsets, collision-aware fitting and a controlled pair of Studio body exports remain follow-up validation/calibration work.

## Planted pedal motion

For the orientation-capable profile path, limb 2 rests at `plan.kit.pedalHat` and limb 3 at `plan.kit.kick`, even if there are no pedal notes. The baseline ankle is the pedal anchor minus the model's baseline ankle-to-contact offset. Hip, knee and ankle stay fixed during a stroke; a small foot pitch around the ankle supplies preparation and depression. At impact and at rest the sole socket returns to the contact anchor.

The current envelope uses a maximum **0.18 rad** toe-up preparation, scaled by velocity, with a squared-sine curve. Isolated hits use a 0.16 s preparation window; repetitions separated by at most 0.32 s share the interval. This is a simple ankle-only animation, not a biomechanical model of heel-up, heel-toe or double-pedal techniques. Hardware is not yet a physically coupled linkage.

Evaluation depends only on absolute time. No integration history, spring state, avatar identity, or rewritten note time controls the pedal pose. A foot hit whose target disagrees with its fixed kit contact is rejected as an inconsistent plan. Unreachable idle contacts generate `RIG_UNREACHABLE_PEDAL`, including songs with no foot hits; IK clamps within actual leg lengths. Foot `restTips` may remain desired contacts outside reach, while hand rest tips must remain reachable. The legacy primitive-v1 comparison path retains its historical motion.

## Stable hardware and appearance adapters

The kit owns standalone stand/pedal groups and calibrated mount/contact frames. A preset/component appearance replacement swaps only the playable drum body/head/cymbal beneath that shared structure. Scale, finish, appearance rotation and optional hit response affect replaceable bodies; they must not mutate hardware transforms or the plan's contact positions. Explicit target/layout calibration may move or rebuild the affected hardware to follow the new contact arrangement.

Bundled-asset adapters identify source geometry and normalize size, orientation and contact/mount alignment. Per-source extraction metadata is acceptable; source mesh names must not enter the Rust planner or avatar solver. Original source files and their license notices remain unchanged. A local GLB is treated as a body-only replacement: arbitrary stand geometry cannot be identified reliably, so users must remove it in an editor before import. No new third-party model is included merely by adding this architecture.

"Fixed hardware" means stable across appearance replacement and strike feedback. It does not prohibit deliberate layout changes or a future mechanically coupled pedal animation. This batch does not claim accurate clamps, telescoping joints or a physically exact hi-hat linkage.

## Grip and playing-style boundary

A hand-local socket aligned across the measured palm, plus finger curl around the stick where semantic finger bones exist, improves the visible grip through the same generic adapter. Keep the stick attached to the actual raw hand and retain rendered-tip measurements. Missing optional fingers reduce detail. A convincing result on one skin is not proof of collision-free finger surfaces on every mesh; per-model animation branches are not an acceptable fix.

The owner's wrist follow-up adds a common palm direction calibration: up (requested default), inward/thumbs-up, or down. The old identity hand-to-tool frame incorrectly coupled a T-pose hand's lateral axis to a forward-pointing stick. Use a diagonal measured hand-local mount and a seated-forearm reference orientation, with forearm roll following the hand while keeping joint positions fixed. Recalibration uses the existing atomic profile/plan path and invalidates an old video. Preserve the chosen direction across seat reset and character replacement within the session. This bounded reference-pose correction does not solve dynamic wrist limits at every target, and symmetric palm-up must not be presented as an anatomical traditional grip. See the [research note](../research/rig-and-studio-audit.md#palm-orientation-correction--2026-10-24).

The existing planner prefers right-hand hi-hat/open-hat and left-hand snare. It is a cost preference: reach, timing and alternation can choose another hand. This is not an explicit cross-handed playing-style mode. The #14/#20 acceptance criteria now distinguish future right-handed crossed, open-handed and mirrored left-handed policies, user overrides, forearm/stick separation, and source-time-preserving diagnostics. A style must not be described as implemented merely because default hand assignments resemble it.

## Validation and release boundary

The Rust pedal increment has passed `cargo test --locked` (26 tests), `cargo fmt --all`, and `cargo clippy --locked --all-targets -- -D warnings`. Its authored two-proportion tests cover planted idle feet without foot notes, fixed ankle chains, rendered contact reconstruction, shuffled seeks, unreachable-contact diagnostics and consistent kit/hit contacts. The [integrated verification record](../verification/calibration-unreleased.md) adds UI, full model import, hardware, seek and encoding evidence, with skin and independent video-file playback limits.

Web/type/build/browser checks are recorded in that verification document. Existing issues track remaining scope and the implementation proceeds through a reviewable PR. No tag, Release or Pages deployment is authorized. User instructions are paired in [English](../calibration.md) and [Korean](../calibration.ko.md).
