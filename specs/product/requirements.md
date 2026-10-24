# Product requirements

## Purpose

Turn an authored drum MIDI file into a reviewable 3D performance and an exported video.
The primary workflow is local file preparation, deterministic playback and seeking,
character/kit adjustment, then fixed-frame video output. Rust owns musical events and
motion planning; WASM connects the browser; Three.js renders the selected rig.

This contract describes the checked-out implementation. Public availability is
tracked in the [reader capability guide](../../docs/reference/availability.md),
which identifies the deployed release separately from development work.

## Invariants

| ID | Required behavior |
| --- | --- |
| P01 | Preserve original event identity, track, tick, note, velocity and musical time through planning. Never shift a strike to disguise a motion failure. |
| P02 | Let users select tracks and channels. Do not assume every drum part is on MIDI channel 10. |
| P03 | Report unsupported or infeasible events. Supported events that cannot be assigned retain their musical data and synthesized sound. |
| P04 | Evaluate a given absolute song time consistently during playback, arbitrary seeking and fixed-frame export. |
| P05 | Keep performance intent separate from a renderer or avatar. Use measured semantic body dimensions, not avatar-name-specific animation branches. |
| P06 | Fit seat and targets without stretching bones. Measure actual attached tool contacts; near-zero contact error alone does not prove natural motion or absence of collision. |
| P07 | Keep visual instrument appearance and hit feedback separate from musical targets and audio. Deliberate layout changes update related targets together. |
| P08 | Process uploaded MIDI, drum GLB and VRM locally in the browser. Do not commit user files, private models or conversation originals. |
| P09 | Report export failures; do not silently turn an audio export into a silent success. A silent video is an explicit choice. |
| P10 | Keep source code, research, motion data, weights, models and sound assets under their respective rights. |

## Implemented workflow boundary

The application accepts SMF MIDI, prepares a plan and synthesized audio, and supports
play/pause/seek with a piano-roll preview. The current source uses a shared semantic
humanoid path for the built-in skeleton and imported VRM 1.0 characters. Seat/contact
calibration is session-only. Drum bodies can be replaced while retaining common
hardware. Output uses fixed-frame browser encoding and a runtime codec check.

Exact limits and coordinate/time contracts belong in the
[engine](../architecture/engine-contract.md), [rig](../architecture/rig-contract.md)
and [audio/export](../architecture/audio-export-contract.md) specifications.
They are engineering bounds, not promises of performance on every accepted input.

## Extension boundary

WAV synchronization, editing and persistence, arbitrary humanoid GLB mapping,
controlled body-proportion comparisons, target-dependent wrist limits, explicit
playing styles, physically coupled pedal techniques and expressive motion remain
separate issue scopes. The existing right-hand-hi-hat/left-hand-snare preference is
not a full crossed-arm style or collision solver.

The technique/expression design preserves music, fitting, technique and expression
as separate concerns. See [ADR 0006](../decisions/0006-technique-and-expression-foundation.md).
Its proposed 0–100 movement control and complete heel-up/down mechanics are not
implemented controls.

## Acceptance ownership

Each implementation issue states observable completion criteria, test cases and
remaining limitations. The [validation policy](../policies/validation.md) defines
how results are recorded. A completed PR is not proof of a public deployment.

Historical U/P/A identifiers and the original MVP scope are preserved in the
[requirements baseline](../../mydocs/report/history/baseline-66951cd/product-requirements.md).
They are not a second mutable list of current issue status.
