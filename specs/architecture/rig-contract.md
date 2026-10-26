# Humanoid, calibration and drum appearance contract

**English** · [한국어](rig-contract.ko.md)

For detailed reasoning, see [calibration](calibration.md) and [rig compatibility](rig-compatibility.md).

## Semantic body boundary

The built-in skeleton and VRM 1.0 import share
[NormalizedHumanoidRigAdapter](../../src/rig-humanoid.ts). Measure semantic rest joints
and actual segment lengths; do not branch animation on an avatar's name, model
filename or creator. Source-specific drum extraction belongs in appearance adapters.

Use normalized humanoid bones for solving and explicit normalized-to-raw conversion.
Attach sticks and contact markers to actual raw hands. Reject negative/nonuniform
skeletal transforms rather than silently stretching a character to reach a target.
Missing optional fingers reduce grip detail without rejecting a valid body.

## Coordinate and tool contract

The numeric engine basis is metres, +Y up, +Z forward, anatomical left at negative X.
Native VRM left is positive X. Position conversion is `[-x, y, z]`; xyzw quaternion
conversion is `[x, -y, -z, w]`. Native skeletal roots remain positive-scale. Only the
legacy kit/debug display uses its reflected root.

The four tool quaternions transform local tool frames into engine world space. They
are not raw bone quaternions. Hand/socket orientation and the model's rest basis are
applied explicitly. The 77-float pose appends a coordinated torso quaternion at
offset 73; it describes spine facing, not a fifth tool. The humanoid adapter accepts
the older 73-float buffer without that rotation. The [engine contract](engine-contract.md)
owns the complete buffer layout.

Measure the transformed stick endpoint or foot-local sole socket against the target;
this does not measure every skin, glove or shoe vertex. Numeric skeletal-proxy
inspection is a separate boundary from these rendered attachment measurements.

## Numeric body geometry

`RigProfile.bodyGeometry` is an optional version-1 numeric contract. It carries
`source` (`mesh-estimate` or `skeleton-fallback`), seated engine-world torso
`center`, shoulder-right/up/forward `halfExtents`, left/right upper/lower
`armRadii`, `handRadii`, optional `handCenters`/`handCapsules`, and `sampleCount`, all lengths in metres. Hand centers and capsule endpoints are wrist-relative in the tool frame; the solver transforms them with the actual solved wrist and tool rotation. Capsules take precedence over offset spheres. Missing optional hand geometry uses the wrist-centered sphere fallback.
[body-geometry.ts](../../src/body-geometry.ts) derives it without avatar or material
name branches; Rust validates it independently. Seat translation moves the center,
and torso facing rotates it with the same shoulders used by the solver.

The adapter samples at most 12,000 vertices deterministically across skinned
meshes. A semantic region needs at least 65% of a vertex's valid skin weight;
extra bones inherit the nearest mapped semantic ancestor. Head/leg regions are
excluded. At least 24 torso samples within the pelvis-to-shoulder height band are
needed; otherwise the shoulder/segment-based fallback remains explicit. Torso
width/depth use the 1st–99th percentile plus 4 mm padding; limb radii use the 98th
percentile plus 3 mm, bounded to 5–250 mm. Sparse limb regions retain their
skeletal estimate even when the torso is mesh-derived. Hand vertices are sampled after the normalized grip is applied to raw bones. A deterministic principal-axis capsule uses the 1st–99th percentile along its axis and the 98th-percentile radial distance plus 3 mm. Palm-direction changes transport the descriptor into the new tool frame; they do not remeasure an unrelated open-hand pose.

These are sampled **surface envelopes**, not an anatomical body segmentation or
containment proof. Torso/arm-weighted clothing can enlarge them, excluded outliers
can fall outside them, and skinning changes with pose. The raw mesh and textures
stay outside the Rust profile and saved project. Rendering/visual checks remain
separate from proxy clearance; see [calibration](calibration.md).

## Model import

[vrm-import.ts](../../src/vrm-import.ts) and
[model-import.ts](../../src/model-import.ts) own preflight checks.

| Input | Accepted boundary |
| --- | --- |
| Character | One self-contained VRM 1.0 `.vrm`, at most 50 MiB |
| Required body | 15 required semantic bones, at most 10,000 nodes, valid references/hierarchy and finite transforms |
| Character images | Embedded PNG/JPEG; at most 64 images, 4096 pixels per side, 64 million pixels total |
| Drum component | Self-contained static `.glb`, at most 25 MiB; body-only geometry expected |

VRM 0.x is detected and rejected with conversion guidance. `.vroid` is an editing
source, not an imported runtime model. Generic humanoid GLB/FBX mapping is not
implemented. External resource references are rejected; metadata links are displayed
without automatically fetching them. File metadata and source terms govern asset
use, independent of the application's MIT license.

## Calibration

[kit-calibration.ts](../../src/kit-calibration.ts) validates contact layouts;
[rig-humanoid.ts](../../src/rig-humanoid.ts) measures and translates the seated body.

- Seat height: 0.30–1.20 m. Forward/back adjustment: −0.50–0.50 m.
- Automatic seat fitting gives a pedal-aware starting position, not collision-free fitting.
- Explicit kit fitting adjusts hand-played contact positions/radii together; kick and
  pedal contacts remain fixed. [Calibration](calibration.md) owns search bounds and
  the limited meaning of a complete proposal.
- Target X is bounded to ±2 m, Y to 0.02–2.50 m and Z to −0.50–2.50 m.
- Kick and hi-hat pedal height is further bounded to 0.04–0.20 m.
- Closed and open hi-hat share a surface contact; pedal hi-hat is independent.
- Kick's engine contact is the foot pedal, not the visible front drum head.
- Changing layout updates `plan.kit` and matching `hit.target` together, then replans
  with the measured profile. Preserve original MIDI data and audio.

Calibration is unavailable during conflicting operations.
Seat translation alone cannot accommodate every arm-to-leg ratio; retain reach
failures rather than altering limb lengths or source times.

## Hands, feet and repeatability

The grip uses a measured diagonal hand-local mount and available finger curl.
Palm up/inward/down is a common calibration choice; symmetric palms-up is not a
complete traditional grip. Optional measured `handFrames` let the common solver
orient the wrist for each target through a compound forearm/tool link, instead of a
fixed world rotation. Forearm roll carries palm orientation. This preserves segment
lengths and the calibrated grip. The bend plane is transported continuously across
the front workspace, and palm roll is a bounded preference rather than an exact
constraint. This does not establish anatomical limits or skin clearance. Profiles
without hand frames retain the previous fixed-orientation path.
With hand frames, a bounded, precomputed two-arm swivel track adjusts the elbow
planes while retaining the calibrated hand/tool relationship. It evaluates existing
contact paths against numeric proxies; its limits and remaining collision scope
are defined in [calibration](calibration.md).

Optional `torsoPivot` in the numeric profile is the measured normalized spine
position, translated with the seat. The evaluator uses it to coordinate shoulder
roots with a precomputed, bounded torso-facing curve, then solves both arms again
to world targets. Pelvis and leg roots remain stable. Profiles without the pivot
retain zero torso yaw. Exact bounds and proxy limitations belong to
[calibration](calibration.md).

The measured path uses instrument-relative ready tips and velocity-dependent lift/
pitch under the [engine contract](engine-contract.md). Phrase assignment considers
available hands for snare/tom fills, accent-to-tap doubles and groove-to-crash role
continuity. It preserves simultaneous source demands and does not recover an
original sticking style or guarantee clear arm paths.

Feet rest on calibrated kick/hi-hat pedal contacts even without pedal notes. In the
profile path, hip/knee/ankle positions remain fixed during the simple ankle stroke.
A time-based foot pitch prepares and returns the sole socket to contact. Unreachable
idle pedals produce diagnostics. Explicit hi-hat controls can drive the left-foot
pitch instead of paired close strikes; [MIDI interpretation](midi-interpretation.md)
owns that policy. This is not full heel-up/down, measured sole support or pedal mechanics.

Each evaluation resets the normalized pose and spring state, applies torso rotation
before the solved limb pose, and updates VRM at delta zero. Expressions/look-at are neutral; node constraints are
applied. Dynamic hair/clothing integration is outside this deterministic contract.

## Shared hardware and replaceable bodies

[equipment.rs](../../crates/drum-engine/src/equipment.rs) defines common numeric
bodies, supports and pedal boxes; [kit-hardware.ts](../../src/kit-hardware.ts)
renders those returned shapes. Appearance adapters replace shell/head/cymbal bodies
while preserving their role and hardware identity. Displayed static bounds are
measured for planning; appearance replacement may therefore require a new layout
transaction, while original MIDI identity and audio remain unchanged. Deliberate
calibration can move/rebuild hardware with the bodies. The
[equipment/contact contract](equipment-contact.md) owns geometry validation,
playable-region targets, whole-tool queries and bounded routing. A completed fit
does not certify exact triangles, support-to-support crossings, skin or every frame.

Bundled adapters may use source geometry identifiers to normalize a model. Those
identifiers must not enter the Rust planner or avatar solver. Local GLB authors must
remove source stands/pedals; arbitrary geometry cannot be segmented reliably.

Optional hit feedback depends on absolute song time/velocity and preserves contact
at impact. It is separate from performer expression. Accurate clamps, telescoping
stands, skin collision, explicit crossed-arm styles and full physical pedal linkage
are not established by this contract.
