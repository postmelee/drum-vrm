# Engine contract

**English** · [한국어](engine-contract.ko.md)

## Authoritative implementation

[model.rs](../../crates/drum-engine/src/model.rs),
[midi.rs](../../crates/drum-engine/src/midi.rs),
[motion.rs](../../crates/drum-engine/src/motion.rs),
[rig.rs](../../crates/drum-engine/src/rig.rs) and
[the WASM exports](../../crates/drum-engine/src/lib.rs) own the executable contract.
[src/types.ts](../../src/types.ts) mirrors its browser data boundary.

## Input and time

Accept strict SMF formats 0 and 1 with PPQ timing. Reject asynchronous format 2 and
SMPTE division with a visible diagnostic. Tempo events from all tracks form the time
map; the selected track/channel determines the performed notes. Preserve source
ticks and use integer microseconds for timing. Velocity-zero note-ons do not trigger
strikes. Note-off duration does not define arm hold time.

The [MIDI interpretation contract](midi-interpretation.md) owns source-event
retention, profile selection, physical-piece mapping and intent/control semantics.

| Bound | Value |
| --- | --- |
| MIDI file | 10 MiB |
| Tracks | 512 |
| Total parsed events | 200,000 |
| Selected supported hits | 50,000 |
| Selected duration | 300 seconds |
| Serialized plan loaded by the evaluator | 128 MiB |
| Serialized numeric rig profile | 64 KiB |

These limits constrain input and memory exposure. They are not a measured guarantee
that every maximum-size input encodes successfully on every device.

## Plan and assignment

`PerformancePlan` schema 2 includes source events and their interpretations, hits,
kit contacts, diagnostics, duration, version identifiers and tempo/meter data.
Hits retain source identity, integer time and track/channel/note/velocity; their
`pieceId` identifies a physical contact separately from the musical `instrument`.
Contact targets and limb assignment are derived values that may change during
calibration/replanning. A limb is either assigned or explicitly absent. Source-event
scope, mapping options and schema 1 migration are defined in
[MIDI interpretation](midi-interpretation.md).

The planner uses an eight-candidate beam with lookahead up to 24 events or 0.35 s;
the current simultaneous group can extend that horizon, capped at 32 events. Future
hits consume limbs in each candidate; only the current group is committed before
looking ahead again. Faster repeated snares receive an additional same-hand cost,
so available hands can alternate without forcing alternation through other strikes.
Events outside the candidate cap retain sound/diagnostics. This is a bounded heuristic,
not a globally optimal arrangement or a collision solver.
Minimum re-strike interval (45 ms) and maximum target speed (12 m/s) are engineering
heuristics, not physiological research measurements. Reach uses the supplied body.
Unsupported notes and supported-but-unassigned hits are different diagnostics.

## WASM API and pose buffers

| Entry point | Meaning |
| --- | --- |
| `inspect_midi(bytes)` | Return inspection/track metadata as JSON |
| `fit_kit(profileJson, kitJson)` | Return a bounded static kit-fit proposal without changing source events |
| `Performance(bytes, selectionJson)` | Build the retained legacy performance path |
| `Performance.from_json(planJson)` | Load the legacy primitive comparison path |
| `Performance.from_json_with_profile(planJson, profileJson)` | Reassign and evaluate for a measured numeric body |
| `plan_json()` | Return the validated resulting plan |
| `evaluate_at(songTime)` | Return 57 world-position floats |
| `evaluate_rig_at(songTime)` | Return those 57 positions plus four xyzw tool quaternions: 73 floats |
| `evaluate_hihat_at(songTime)` | Return absolute-time hi-hat openness from the selected interpretation policy |

The first 48 values contain four limbs in LH/RH/LF/RF order, each with root, middle,
end and tool-contact XYZ points. Pelvis, chest and head occupy offsets 48, 51 and 54.
The additional quaternions start at offset 57 in the same limb order. Arrays are
owned values; consumers do not retain an undocumented view into WASM memory.

`RigProfile` schema 1 contains a coordinate-system tag, roots, segment lengths,
bend poles, rest tips, grip/tip offsets, contact rotations and torso reference points.
Optional `handFrames` provide measured forearm/palm axes and a desired palm direction
for the common target-dependent wrist solver. Older profiles without them retain
the fixed-orientation behavior. The profile is supplied separately from the plan.
A plan's `calibrationId` alone cannot
reconstruct the profile. The [rig contract](rig-contract.md) defines coordinate and
attachment semantics.

## Deterministic evaluation

A pose is a function of the validated plan, profile and absolute song time.
Sequential playback, shuffled seeks and fixed-frame export must produce the same
major pose. No frame-history randomness or accumulated secondary physics may
silently change that result.

In the measured-profile path, left/right hands normally wait 3.5 cm above snare/hi-hat.
Between repetitions on one physical piece, the ready tip stays over that piece.
If the candidate is unreachable, use the measured reference rest instead. Consecutive
hits within 0.42 s transfer directly; wider gaps use 0.22 s preparation and 0.18 s
recovery windows. The legacy primitive comparison retains its generic rest.

Velocity scales lift and wrist pitch; short intervals reduce both to fit available
time. Exact impacts retain their targets and zero added stroke pitch. These are
procedural strokes, not learned push-pull, force simulation or complete joint limits.
The [calibration contract](calibration.md) explains target-dependent hand frames;
[MIDI interpretation](midi-interpretation.md) owns hi-hat state and source semantics.
Playback and export share the selected audio buffer under the
[audio/export contract](audio-export-contract.md).
