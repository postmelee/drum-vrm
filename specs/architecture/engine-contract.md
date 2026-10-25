# Engine contract

**English** · [한국어](engine-contract.ko.md)

## Authoritative implementation

[model.rs](../../crates/drum-engine/src/model.rs),
[midi.rs](../../crates/drum-engine/src/midi.rs),
[motion.rs](../../crates/drum-engine/src/motion.rs),
[sticking.rs](../../crates/drum-engine/src/sticking.rs),
[body_motion.rs](../../crates/drum-engine/src/body_motion.rs),
[hand_coordination.rs](../../crates/drum-engine/src/hand_coordination.rs),
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

Planner `phrase-beam-8-beat-horizon-24-v3` uses an eight-candidate beam with lookahead
up to 24 events and two local quarter notes, with the time horizon clamped to
0.35–1.20 s. The current simultaneous
group can extend that horizon, capped at 32 events. Future hits consume limbs in each
candidate; only the current group is committed before looking ahead again.

Candidate costs retain a bounded eight-hand-event history. Rapid snare/tom runs and
mixed fills discourage extended one-hand repetition. A strong hit followed by a
weaker hit on the same surface can instead favor a two-hit rebound candidate.
Single, double and paradiddle sequences provide weak preferences, including mirrored
and cyclic variants; they do not identify the original drummer's technique.
Hi-hat/ride context gives nearby crashes a right-hand preference so a crash can
continue the groove role. It remains a preference: simultaneous hi-hat/crash events
still compete for separate available hands. A simultaneous two-crash chord instead
strongly prefers outward spatial ownership (left hand to the left target, right hand
to the right), independently of event order and kit names. Squared transfer-speed
effort also favors an available hand over a rapid cross-kit reach, even below the
hard speed ceiling. These remain soft costs and respect explicit hand overrides.
No source hit is deleted or retimed to enforce a style.

Numeric torso-clearance cost is a soft preference at candidate contacts. Events
outside the candidate cap retain sound/diagnostics. This is a bounded heuristic,
not a globally optimal arrangement or a guarantee of clear motion paths.
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
| `evaluate_rig_at(songTime)` | Return 57 positions, four xyzw tool quaternions and one torso quaternion: 77 floats |
| `evaluate_hihat_at(songTime)` | Return absolute-time hi-hat openness from the selected interpretation policy |
| `inspect_pose_at(songTime)` | Return numeric skeletal-proxy clearance and bone-length measurements as JSON |

The first 48 values contain four limbs in LH/RH/LF/RF order, each with root, middle,
end and tool-contact XYZ points. Pelvis, chest and head occupy offsets 48, 51 and 54.
Tool quaternions start at offset 57 in the same limb order; the torso quaternion
starts at offset 73. The retained 57-position API truncates this buffer. The humanoid
adapter also accepts the older 73-float buffer, with no torso rotation. Arrays are
owned values; consumers do not retain an undocumented view into WASM memory.

`RigProfile` schema 1 contains a coordinate-system tag, roots, segment lengths,
bend poles, rest tips, grip/tip offsets, contact rotations and torso reference points.
Optional `handFrames` provide measured forearm/palm axes and a desired palm direction
for the common target-dependent wrist solver. Older profiles without them retain
the fixed-orientation behavior. Optional `torsoPivot` supplies the measured spine
pivot for coordinated facing; without it, torso yaw remains zero. Both fields keep
schema 1 compatible. The profile is supplied separately from the plan.
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
If the candidate is unreachable, use the measured reference rest instead. Hand
preparation/recovery windows use contact-to-ready distance divided by 2.5 m/s,
clamped to 0.22–0.65 s and 0.18–0.65 s respectively. A gap within their sum plus
0.02 s uses one direct transfer, preserving the 0.42 s threshold for nearby targets
while preventing overlapping long moves from interrupting each other. Wider gaps
join the ready pose between separate windows. These durations are engineering
heuristics. The legacy primitive comparison retains its generic rest.

Velocity scales lift and wrist pitch; short intervals reduce both to fit available
time. Current/next velocity also moves the peak of a direct transition earlier after
an accent and later before an accent, leaving both endpoints fixed. Exact impacts
retain their targets and zero added stroke pitch. These are procedural strokes, not
learned push-pull, force simulation or complete joint limits.
For hand transfers in the measured-profile path, a lifted intermediate tip is kept
at least as far from its reference shoulder as the interpolated endpoint radii.
This reduces inward folding during preparation; it is not obstacle routing.

For profiles with `torsoPivot`, planning precomputes a 12 Hz facing curve with yaw
bounded to ±0.24 radians. Candidate yaw balances both hand targets, torso/forearm
proxy clearance and change from the previous planned sample. Absolute-time smooth
interpolation selects yaw; both arms are solved again from the turned shoulders to
the original world targets. Pelvis and leg roots remain fixed. This precomputed
curve does not depend on playback order or frame rate. Every assigned hand impact
is checked using the interpolated yaw; losing reach attenuates the entire curve,
with zero yaw as the fallback. The [calibration contract](calibration.md) defines
the continuous neutral-wrist solve and bounded torso planning.

For profiles with `handFrames`, a separate precomputed elbow-swivel track coordinates
both arms against torso, arm and stick proxies. It uses the turned profiles and
existing contact paths; it changes neither hit assignment nor source timing. The
[calibration contract](calibration.md) owns its sampling and angular bounds.
Absolute-time interpolation preserves seek determinism; proxy costs do not certify
collision-free motion.

`inspect_pose_at` reports `scope: "skeletal-proxies"`, `torsoClearanceMeters`,
`armClearanceMeters`, `stickClearanceMeters`, `boneLengthErrorMeters` and `poseFloats`.
Negative clearance means overlap of the numeric proxies. Bone error is the maximum
segment-length deviation across four limbs. These values inspect the evaluated
numeric pose; they neither measure rendered attachment markers nor establish skin,
clothing, imported instrument geometry or continuous-path clearance.

The [calibration contract](calibration.md) explains target-dependent hand frames;
[MIDI interpretation](midi-interpretation.md) owns hi-hat state and source semantics.
Playback and export share the selected audio buffer under the
[audio/export contract](audio-export-contract.md).
