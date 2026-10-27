# Product requirements

**English** · [한국어](requirements.ko.md)

## Purpose

Turn an authored drum MIDI file into a reviewable 3D performance and an exported video.
The immediate quality target is a natural, reviewable browser performance synchronized
to the user’s WAV. The workflow is local file preparation, deterministic playback and
seeking, character/kit adjustment, and reproducible review. Video output remains
available; expanded production/interchange outputs follow this browser baseline. Rust owns musical events and
motion planning; WASM connects the browser; Three.js renders the selected rig.

This contract describes the checked-out implementation. Public availability is
tracked in the [project README](../README.md),
which identifies the deployed release separately from development work.

## Invariants

| ID | Required behavior |
| --- | --- |
| INV01 | Preserve original event identity, track, tick, note, velocity and musical time through planning. Never shift a strike to disguise a motion failure. |
| INV02 | Let users select tracks and channels. Do not assume every drum part is on MIDI channel 10. |
| INV03 | Report unsupported or infeasible events. Supported events that cannot be assigned retain their musical data and synthesized sound. |
| INV04 | Evaluate a given absolute song time consistently during playback, arbitrary seeking and fixed-frame export. |
| INV05 | Keep performance intent separate from a renderer or avatar. Use measured semantic body dimensions, not avatar-name-specific animation branches. |
| INV06 | Fit seat and targets without stretching bones. Measure actual attached tool contacts; near-zero contact error alone does not prove natural motion or absence of collision. |
| INV07 | Keep visual instrument appearance and hit feedback separate from musical targets and audio. Deliberate layout changes update related targets together. |
| INV08 | Process uploaded MIDI, WAV, drum GLB and VRM locally in the browser. Do not commit user files, private models or conversation originals. |
| INV09 | Report export failures; do not silently turn an audio export into a silent success. A silent video is an explicit choice. |
| INV10 | Keep source code, research, motion data, weights, models and sound assets under their respective rights. |

## Implemented workflow boundary

The application accepts SMF MIDI with explicit GM/Logic mapping and optional count-in/
hi-hat interpretation, preserving source evidence under the
[MIDI interpretation contract](architecture/midi-interpretation.md). A bounded phrase
planner assigns hits; playback and seeking use the piano roll and either synthesized
drums or a local WAV aligned to the same export clock.

The shared semantic humanoid path serves the built-in skeleton and imported VRM 0.0/1.0
characters, with target-dependent neutral wrists and instrument-relative ready poses.
Seat/contact calibration and explicit static kit fitting can be retained in a
[local project](architecture/project-format.md), with exact external files relinked. Drum bodies
can be replaced while retaining common hardware. Output uses fixed-frame browser
encoding and a runtime codec check.

Exact limits and coordinate/time contracts belong in the
[engine](architecture/engine-contract.md), [rig](architecture/rig-contract.md)
and [audio/export](architecture/audio-export-contract.md) specifications.
They are engineering bounds, not promises of performance on every accepted input.

## Extension boundary

Arbitrary humanoid GLB mapping, controlled body-proportion
comparisons, measured anatomical joint limits, explicit
playing styles, physically coupled pedal techniques and expressive motion remain
separate issue scopes. The existing right-hand-hi-hat/left-hand-snare preference is
not a full crossed-arm style or collision solver.

The technique/expression design preserves music, fitting, technique and expression
as separate concerns. See [ADR 0006](../docs/decisions/0006-technique-and-expression-foundation.md).
Its proposed 0–100 movement control and complete heel-up/down mechanics are not
implemented controls.

## Acceptance ownership

Each implementation issue states observable completion criteria, test cases and
remaining limitations. Verification records identify the tested commit, environment,
executed checks and untested scope. A completed PR is not proof of a public deployment.

Historical U/P/A identifiers and the original MVP scope are preserved in the
[requirements baseline](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/product-requirements.md).
They are not a second mutable list of current issue status.
