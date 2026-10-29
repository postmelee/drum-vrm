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

Derived `contactEvents` and `hihatMotion` share planned surface contact and
absolute-time mechanism state across motion, feedback and separate actual-marker
inspection. The [equipment contact contract](equipment-contact.md#shared-contact-records-and-hi-hat-state)
owns their fields, regeneration and capability limits.

Planner `phrase-beam-8-beat-horizon-24-posture-v18` uses an eight-candidate beam with lookahead
up to 24 events and two local quarter notes, with the time horizon clamped to
0.35–1.20 s. The current simultaneous
group can extend that horizon, capped at 32 events. Future hits consume limbs in each
candidate; only the current group is committed before looking ahead again.

Candidate costs retain a bounded eight-hand-event history. Rapid snare/tom runs and
mixed fills discourage extended one-hand repetition. A strong hit followed by a
weaker hit on the same surface can instead favor a two-hit rebound candidate.
Single, double and paradiddle sequences provide weak preferences, including mirrored
and cyclic variants; they do not identify the original drummer's technique.
Candidates first minimize unassigned events, then deviations from the default
right-handed groove roles, then transfer, clearance and rudiment costs. Hi-hat/open
hi-hat/ride belong to the right hand; sparse snare backbeats near a groove belong to
the left. Head strikes within 0.8 quarter notes (clamped to 0.15–0.4 s) remain fill/run
candidates, free to alternate or use doubles. These windows are engineering choices,
not recovery of a player's intended style. Explicit hand overrides take precedence;
reach, same-limb occupancy and transfer bounds still determine feasibility.

A nearby single crash continues the right-hand groove role unless the most recent
head strike occurred after the last groove strike: that head-led exit releases the
crash-hand preference. A simultaneous backbeat and hat do not break the groove.
Post-fill candidates retain both hands and compare their previous contacts and the
next destination; a left crash may use the left hand while the right prepares the
hat. Do not change a source left crash into a right crash to shorten a path.
A simultaneous two-crash chord prefers outward spatial ownership (left hand to the
left target, right hand to the right), independently of event order and kit names.
Original times and physical targets stay unchanged. The default roles are not a
universal prohibition on two-handed hi-hat or open-handed playing; explicit style
selection remains future work. [Instructional rationale](../../docs/research/ergonomics-and-sticking.md#groove-ownership-and-post-fill-crashes).

Numeric torso-clearance cost is a soft preference at candidate contacts. Events
outside the candidate cap retain sound/diagnostics. This is a bounded heuristic,
not a globally optimal arrangement or a guarantee of clear motion paths.
Minimum re-strike interval (45 ms) and maximum target-chord speed (12 m/s) are
assignment heuristics, not physiological research measurements. The speed is the
straight-line distance between successive assigned targets divided by their source
time interval; it does not bound the final curved stick-tip path or joint speed.
Reach uses the supplied body.
Unsupported notes and supported-but-unassigned hits are different diagnostics.

## WASM API and pose buffers

| Entry point | Meaning |
| --- | --- |
| `inspect_midi(bytes)` | Return inspection/track metadata as JSON |
| `plan_midi(bytes, selectionJson)` | Interpret and assign a source plan without constructing runtime motion tracks |
| `fit_kit(profileJson, kitJson)` | Return a bounded static kit-fit proposal without changing source events |
| `ready_pose(profileJson, kitJson)` | Return a validated dual-snare reference pose without a score |
| `Performance(bytes, selectionJson)` | Build the retained legacy performance path |
| `Performance.from_json(planJson)` | Load the legacy primitive comparison path |
| `Performance.from_json_with_profile(planJson, profileJson)` | Reassign and evaluate for a measured numeric body |
| `prepared_json()` | Serialize validated plan/profile and precomputed motion tracks for a Worker handoff |
| `Performance.from_prepared_json(json, expectedMidiHash, expectedProfileJson)` | Validate and install a prepared runtime without repeating planning |
| `plan_json()` | Return the validated resulting plan |
| `evaluate_at(songTime)` | Return 57 world-position floats |
| `evaluate_rig_at(songTime)` | Return 57 positions, four xyzw tool quaternions and one torso quaternion: 77 floats |
| `evaluate_hihat_at(songTime)` | Return shared absolute-time hi-hat openness from literal notes and explicit controls |
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

`fit_kit` also returns `seatOffset` (metres in profile coordinates) and bounded
`searchFailures` entries `{stage, pieceId, branches}`. Apply a nonzero seat offset
only with a complete proposal, regenerate the profile through the same adapter,
and prepare the plan before committing scene state. A failed proposal preserves
the input layout and zero seat offset; search-stage evidence does not replace
layout-specific reach/clearance diagnostics. Optional `advisoryLayoutViolations: string[]` contains
measured leg/foot envelopes and broad `head-approach:<head>:<obstacle>` warnings, excluding hard failures in
`layoutViolations`. Actual contact, whole-stick and required-pair checks still gate completion. It does not affect `complete`; an absent legacy field means unreported, not clear.
The [calibration contract](calibration.md) owns search bounds and static/motion acceptance.

## Preparation and Worker boundary

Browser fitting, initial planning, calibration replanning and project reconstruction
run in cancellable Workers. The active scene and evaluator stay intact until a
candidate is validated and committed; failed or cancelled setup/replan operations
retain the previous setup. Loading a different MIDI intentionally clears the old
score before preparation. Rendering, absolute-time evaluation and inexpensive runtime installation
remain on the main thread.

`plan_midi` parses and interprets the source without constructing a motion evaluator.
The worker uses it before applying the final measured profile and kit, so it does
not prepare an unused default-body motion first. Completed requests retain the
initialized worker for the next request; cancellation or replacement of an active
request terminates it. Request IDs and generations reject stale replies. An exact
source/profile match can reuse the single validated prepared candidate. After
MIDI planning, that candidate also owns the exact input produced by resetting its
derived targets to the captured request layout. Audio preflight can then install
the same candidate without repeating motion search. Each key includes the complete
plan and numeric profile, preserving every field of its layout; changed source
directives, geometry or body data require preparation again. Cancellation and a
failed replacement search discard candidate reuse. Rust still validates every
prepared installation. This is bounded
reuse, not a cache of arbitrary uploaded files.

[prepared.rs](../../crates/drum-engine/src/prepared.rs) owns internal transfer format
7, containing the plan, numeric profile, torso, elbow/orientation and equipment-approach tracks,
and an FNV-1a integrity
checksum. Planner version `phrase-beam-8-beat-horizon-24-posture-v18` identifies
single actual-orientation controls relative to the geometric approach; earlier prepared candidates must be rebuilt.
Installation checks engine/planner versions, expected source/profile
identity, finite values, ordering, track bounds and mandatory control times. The separate
`handOrientations` track retains every original swivel-time orientation exactly, stays within
the original endpoints and has at most 6,000 sorted anchors. Each
`SwivelSample.orientations` / `OrientationSample.orientations` value is a single finite unit quaternion (squared norm within
`1e-8` of one), or consistently `None` for a hand without a bounded palm policy. Decoded
orientations at control times must satisfy that policy. Assigned hand impacts and retained
count-in contacts are rechecked with stored/interpolated swivel and approach-relative actual
orientation, torso yaw and equipment/stroke pitch. Pedal feasibility is also retained. A reachable
alternative pose cannot validate an unreachable stored impact. Equipment tracks additionally require bounded lift/pitch,
rate limits and zero lift at each source impact. All mandatory equipment times must
survive exactly; sorted extra controls may lie only within the original endpoints.
Installation uses these validated samples without repeating equipment-path search
on the main thread. The checksum
detects stale or damaged data; it is not a
cryptographic signature. Payloads are capped at 128 MiB. Exact float round trips
preserve primary poses and numeric inspection. This ephemeral handoff is separate
from the [saved project format](project-format.md).

The joint motion planner has a separate 6,000-control-time budget, including its
0.1-second grid, exact assigned hand impacts and explicit count-in boundaries after
deduplication. Original impact times take precedence over almost-equal floating-point
grid times. Exceeding it
returns a preparation error before allocating the search tables; shorten the
selected passage or reduce selected tracks. The 50,000-hit parsing limit does not
guarantee that a score fits this motion budget. Sampling and memory bounds are
specified in [calibration](calibration.md). The independent equipment-approach track
retains its mandatory 30 Hz grid and exact assigned hand impacts. Up to three
midpoint-refinement passes add failing interiors of intervals at least 1/240 s long,
within a total of 16,000 controls. Preparation and transfer deserialization enforce
the same total bound. [Equipment contact](equipment-contact.md) defines the subsequent
bounded joint pitch/lift repair; neither search certifies the continuous path.

## Deterministic evaluation

A pose is a function of the validated plan, profile and absolute song time.
Sequential playback, shuffled seeks and fixed-frame export must produce the same
major pose. No frame-history randomness or accumulated secondary physics may
silently change that result.

Static fitting and contact candidates use [calibration](calibration.md)'s bounded optimizer:
retain the legacy anatomical arm using a feasible rigid wrist-to-tip twist, otherwise search
shaft yaw within ±45° with twelve golden-section steps. Overextended approaches may use the
existing 165° elbow reach cone via a shortest rigid tool-offset rotation, retaining exact contact
and the full posture/palm gates. Static contact/probe and ready/pair searches exhaust the existing pitch range with arm-preserving witnesses before their full fallback families; a fixed-pitch query keeps its original coarse-first full search. There is no runtime cone search.
Shared `HandEnvironment` gives contact, ready, cue, graph and route searches the intended-piece whole-tool gate before a static candidate is chosen; a colliding first witness continues through the existing bounded alternatives.
Each selected original control stores one quaternion `inverse(q_approach) × q_selected`.
Edge checks/replay use `q_currentApproach × slerp(residual)` with smoothstep weights.
Selected endpoint wrist/elbow/contact remain exact; intermediate legacy-arm or pure wrist-twist preservation is not guaranteed. No isolated anchors or graph dimensions are added.
Runtime performs one fixed-length arm solve; it does not repeat the ordinary roll refinements, project the palm/wrist or search shaft yaw. Unresolved
interpolated poses retain their diagnoses. Legacy hands keep `None` and their existing solve.
Actual tool rotation, wrist displacement and body-relative shoulder-to-elbow direction
receive integrated squared-rate soft costs at graph transitions and inspected edge samples.
The costs use actual poses, not residual quaternion changes. The 2 rad/s relative-swivel
knot limit and smoothstep's 1.5 factor remain; neither these bounds nor the soft costs certify
physiological limits on the complete arm/tool motion. There is no added graph dimension or
frame history. Inspected samples do not certify the entire continuous path.

In the measured-profile path, left/right hands normally wait 3.5 cm above snare/hi-hat.
Between repetitions on one physical piece, the ready tip stays over that piece.
If the candidate is unreachable, use the measured reference rest instead. Hand
preparation/recovery windows use contact-to-ready distance divided by 2.5 m/s,
clamped to 0.22–0.65 s and 0.18–0.65 s respectively. A gap within their sum plus
0.02 s uses one direct transfer, preserving the 0.42 s threshold for nearby targets
while preventing overlapping long moves from interrupting each other. Wider gaps
join the ready pose between separate windows. These durations are engineering
heuristics. The legacy primitive comparison retains its generic rest.
Each `Motion` precomputes immutable ready-pose feasibility, including failed
queries, keyed by limb, physical piece and exact target bits. Shared assembly
reconstructs it on prepared restoration; there is no playback-dependent cache.

Velocity scales lift and wrist pitch; short intervals reduce both to fit available
time. Current/next velocity also moves the peak of a direct transition earlier after
an accent and later before an accent, leaving both endpoints fixed. Exact impacts
retain their targets and zero added stroke pitch. These are procedural strokes, not
learned push-pull, force simulation or complete joint limits.
Transfers between distinct surfaces also use a bounded smooth tip arch informed by
nearby equipment bounds, retaining larger authored stroke lift. Its added height
is limited to 0.20 m and the available endpoint travel budget; endpoints and source
times are unchanged. This guide does not replace whole-tool collision inspection
or establish a speed bound for the final solved pose.
For hand transfers in the measured-profile path, a lifted intermediate tip is kept
at least as far from its reference shoulder as the interpolated endpoint radii.
This reduces inward folding during preparation; it is not obstacle routing.

For profiles with `torsoPivot`, planning precomputes a 12 Hz facing curve with yaw
bounded to ±0.24 radians. Candidate yaw balances both hand targets, torso/forearm
proxy clearance and change from the previous planned sample. Absolute-time smooth
interpolation selects yaw; both arms are solved again from the turned shoulders to
the original world targets. Pelvis and leg roots remain fixed. This precomputed
curve does not depend on playback order or frame rate. Every assigned hand impact
is checked using interpolated yaw; losing reach attenuates adjacent knots with
rate-bound propagation, with zero yaw as the bounded-repair fallback. The [calibration contract](calibration.md) defines
the shared target-oriented shaft/wrist solve and bounded torso planning.

For profiles with `handFrames`, a separate precomputed elbow-swivel track coordinates
both arms against torso, arm and stick proxies. It uses the turned profiles and
existing contact paths; it changes neither hit assignment nor source timing. The
[calibration contract](calibration.md) owns its sampling and angular bounds.
Absolute-time interpolation preserves seek determinism; proxy costs do not certify
collision-free motion.

`inspect_pose_at` reports `scope: "skeletal-proxies"`, `torsoClearanceMeters`,
`armClearanceMeters`, `stickClearanceMeters`, `crossClearanceMeters`,
`geometrySource`, `boneLengthErrorMeters` and `poseFloats`. Cross clearance is the
minimum stick-to-opposite-arm/hand clearance; own-hand grip attachment is permitted.
Negative measured-body clearance is an advisory numeric overlap. The historical
`equipmentClearanceMeters` remains the combined arm/hand/tool minimum; it must not
be interpreted as the physical-stick acceptance result. Optional fields
`toolEquipmentClearanceMeters` and `toolPairClearanceMeters` separately report
whole-stick/equipment and whole-stick/stick clearance, including only the explicit
playing-surface or count-in contact exceptions. Missing fields mean unmeasured, not
zero clearance. Bone error is the maximum segment-length deviation across four
limbs. These values inspect the evaluated numeric pose, not rendered attachment
markers, skin or exact triangles.

Browser review keeps the raw proxy minimum/count fields and additionally aggregates
optional `toolSamples`, `toolEquipmentMinimumMeters`, `toolPairMinimumMeters`,
`toolIntersectingSamples` and `firstToolIntersectionSeconds`. A negative value in
either tool metric counts as an intersecting tool sample. Acceptance needs present,
finite, nonnegative whole-tool measurements; advisory body minima cannot substitute
for or invalidate that result. Frame samples still do not prove the continuous path.

Final engine inspection evaluates the same quarter/mid/three-quarter interval
times as tool-route repair, plus the path knots, hand controls and count-in controls.
It checks the final pose after torso and hand coordination. `POSTURE_LIMIT`
covers sampled preparation, contact, recovery and rest, not only impact instants;
it retains the common anatomical bounds without treating body/clothing overlap
as a hard failure. All these diagnostics are sampled findings, not continuous
collision or posture certificates.

The [reference-layout review](../../scripts/reference-layout-review.mjs) separately
gates assignment coverage, tool paths and motion validity. `POSTURE_LIMIT`,
`CONTACT_PITCH_LIMIT`, `COUNT_IN_GESTURE_UNAVAILABLE`, `TOOL_EQUIPMENT_PATH_CLEARANCE`
and `TOOL_PAIR_PATH_CLEARANCE` block motion validity: clear review-grid samples
cannot erase a physical failure found at an internal refined time. Each source
count-in must retain two planned hand contacts at its original time, so omitting
an unavailable cue cannot pass. Successful fit and full hit assignment alone do
not establish temporal acceptance. Existing review
times also receive primary-pose finite-difference measurements at ±1/240 s: sampled
tip/wrist speed, tip acceleration and tool angular speed. These diagnostics are not
a dense 240 Hz scan or a continuous speed certificate; f32 quantization affects
derivatives, especially acceleration. No final-tip speed threshold is asserted.
Actual adapter bone angles and these motion metrics are summarized globally and
around semantic landmarks. Missing angle measurements remain unmeasured. The script
still requires separate textured visual review of posture, skin and clothing.

`BODY_PATH_CLEARANCE` reports advisory body overlap.
`TOOL_EQUIPMENT_PATH_CLEARANCE` and `TOOL_PAIR_PATH_CLEARANCE` identify sampled
physical-tool intersections; `EQUIPMENT_PATH_CLEARANCE` retains the combined
historical report. These diagnostics preserve source hits/times and expose an
imperfect motion result rather than asserting that every path is clear.

The [calibration contract](calibration.md) explains target-dependent hand frames;
[MIDI interpretation](midi-interpretation.md) owns hi-hat state and source semantics.
Playback and export share the selected audio buffer under the
[audio/export contract](audio-export-contract.md).
