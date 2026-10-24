# MIDI and engine contract

## Authoritative implementation

[model.rs](../../crates/drum-engine/src/model.rs),
[midi.rs](../../crates/drum-engine/src/midi.rs),
[motion.rs](../../crates/drum-engine/src/motion.rs),
[rig.rs](../../crates/drum-engine/src/rig.rs) and
[the WASM exports](../../crates/drum-engine/src/lib.rs) own the executable contract.
[src/types.ts](../../src/types.ts) mirrors its browser data boundary.

## Input interpretation

Accept strict SMF formats 0 and 1 with PPQ timing. Reject asynchronous format 2 and
SMPTE division with a visible diagnostic. Tempo events from all tracks form the time
map; the selected track/channel determines the performed notes. Preserve source
ticks and use integer microseconds for timing. Velocity-zero note-ons do not trigger
strikes. Note-off duration does not define arm hold time.

| Bound | Value |
| --- | --- |
| MIDI file | 10 MiB |
| Tracks | 512 |
| Total parsed events | 200,000 |
| Selected supported hits | 50,000 |
| Selected duration | 300 seconds |
| Serialized plan loaded by the evaluator | 32 MiB |
| Serialized numeric rig profile | 64 KiB |

These limits constrain input and memory exposure. They are not a measured guarantee
that every maximum-size input encodes successfully on every device.

## Plan and assignment

`PerformancePlan` schema 1 includes hits, kit contacts, diagnostics, duration,
metadata/version identifiers and tempo/meter data. Hits preserve source identity,
integer time and track/channel/note/velocity. Contact targets and limb assignment are
derived values that may change during calibration/replanning. A limb is either assigned
or explicitly absent. Current semantic roles are kick, snare, closed/open/pedal hi-hat,
three toms, crash and ride. Distinct physical instances are not yet a separate schema.

The planner uses a bounded beam and short lookahead. Simultaneous groups expand at
most 32 events into assignment candidates; remaining events retain sound/diagnostics.
Minimum re-strike interval (45 ms) and maximum target speed (12 m/s) are engineering
heuristics, not physiological research measurements. Reach uses the supplied body.
Unsupported notes and supported-but-unassigned hits are different diagnostics.

## WASM API and pose buffers

| Entry point | Meaning |
| --- | --- |
| `inspect_midi(bytes)` | Return inspection/track metadata as JSON |
| `Performance(bytes, selectionJson)` | Build the retained legacy performance path |
| `Performance.from_json(planJson)` | Load the legacy primitive comparison path |
| `Performance.from_json_with_profile(planJson, profileJson)` | Reassign and evaluate for a measured numeric body |
| `plan_json()` | Return the validated resulting plan |
| `evaluate_at(songTime)` | Return 57 world-position floats |
| `evaluate_rig_at(songTime)` | Return those 57 positions plus four xyzw tool quaternions: 73 floats |

The first 48 values contain four limbs in LH/RH/LF/RF order, each with root, middle,
end and tool-contact XYZ points. Pelvis, chest and head occupy offsets 48, 51 and 54.
The additional quaternions start at offset 57 in the same limb order. Arrays are
owned values; consumers do not retain an undocumented view into WASM memory.

`RigProfile` schema 1 contains a coordinate-system tag, roots, segment lengths,
bend poles, rest tips, grip/tip offsets, contact rotations and torso reference points.
It is supplied separately from the plan. A plan's `calibrationId` alone cannot
reconstruct the profile. The [rig contract](rig-contract.md) defines coordinate and
attachment semantics.

## Deterministic evaluation

A pose is a function of the validated plan, profile and absolute song time.
Sequential playback, shuffled seeks and fixed-frame export must produce the same
major pose. No frame-history randomness or accumulated secondary physics may
silently change that result.

The implemented hand trajectory still returns to a generic rest for sufficiently
spaced hits (the current direct-transition threshold is 0.42 s). Instrument-specific
waiting poses and target-dependent wrist limits are proposals, not guarantees of
this interface. Preserve that limitation in motion-quality claims.
