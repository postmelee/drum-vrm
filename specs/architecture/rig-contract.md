# Humanoid, calibration and drum appearance contract

**English** · [한국어](rig-contract.ko.md)

For detailed reasoning, see [calibration](calibration.md) and [rig compatibility](rig-compatibility.md).

## Semantic body boundary

The built-in skeleton, VRM 0.0/1.0 and mapped FBX imports share
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
`armRadii`, `handRadii`, optional `legRadii` (left/right thigh/shin), `footRadii`,
`legBoxes`/`footBoxes`, `legRestFrames`, `handCenters`/`handCapsules`, and `sampleCount`, all lengths in metres. Hand centers and capsule endpoints are wrist-relative in the tool frame; the solver transforms them with the actual solved wrist and tool rotation. Capsules take precedence over offset spheres. Missing optional hand geometry uses the wrist-centered sphere fallback.
[body-geometry.ts](../../src/body-geometry.ts) derives it without avatar or material
name branches; Rust validates it independently. Seat translation moves the center,
and torso facing rotates it with the same shoulders used by the solver.

The adapter samples at most 12,000 vertices deterministically across skinned
meshes, considering only indices in the active draw ranges and visible material groups. Shared but unreferenced attribute vertices do not contribute. A semantic region needs at least 65% of a vertex's valid skin weight;
extra bones inherit the nearest mapped semantic ancestor. Head/hair are excluded;
legs and feet are measured separately. At least 24 torso samples within the pelvis-to-shoulder height band are
needed; otherwise the shoulder/segment-based fallback remains explicit. Torso
width/depth use the 1st–99th percentile plus 4 mm padding; limb radii use the 98th
percentile plus 3 mm, bounded to 5–250 mm. Sparse limb regions retain their
skeletal estimate even when another region is mesh-derived. Lower-limb measurements
are retained even if torso coverage is insufficient. Missing leg radii use 14% of
thigh length (25–90 mm) and 10% of shin length (20–70 mm); missing foot radii use
40 mm. Foot width is measured around the ankle-to-toes axis, or a sampled principal
axis when that segment has insufficient samples. These are explicit proxy defaults,
not measured shoe dimensions. Hand vertices are sampled after the normalized grip is applied to raw bones. A deterministic principal-axis capsule uses the 1st–99th percentile along its axis and the 98th-percentile radial distance plus 3 mm. Palm-direction changes transport the descriptor into the new tool frame; they do not remeasure an unrelated open-hand pose.

Measured lower limbs additionally provide offset boxes so hanging fabric behind a
leg does not expand an equal-radius tube in every direction. `legBoxes` pairs
left/right thigh/shin descriptors, each `{center, halfExtents}`. The origin is the
proximal joint; local Y points toward the distal joint, X is shoulder-right projected
perpendicular to Y, and Z is X × Y. `footBoxes` are ankle-relative in the neutral
tool frame. Every axis uses 1st–99th percentile bounds plus 3 mm. Centers have length
at most 3 m and half extents are 1–1000 mm. All four leg regions (or both feet) need
sufficient samples before the corresponding optional field is emitted.
`legRestFrames` optionally records the four measurement-frame unit quaternions
in the same side/segment order. The seated frame applies the minimum swing from
that frame's Y axis to the solved leg direction, then the original frame rotation:
`Qposed = Qswing(Yrest → Yposed) × Qrest`. This matches the adapter's leg transport
instead of independently projecting body-right again after bending. Older profiles
without rest frames retain the projected body-right fallback. Foot boxes use the
actual tool rotation directly. The fit retains a small segment-based core. Missing boxes retain the radius-based
fallback. No garment region is removed to make a candidate pass.

These are sampled **surface envelopes**, not an anatomical body segmentation or
containment proof. Torso/limb-weighted clothing can enlarge them, excluded outliers
can fall outside them, and skinning changes with pose. The raw mesh and textures
stay outside the Rust profile and saved project. Rendering/visual checks remain
separate from proxy clearance; see [calibration](calibration.md).

## Model import

[vrm-import.ts](../../src/vrm-import.ts) and
[model-import.ts](../../src/model-import.ts) own preflight checks.

| Input | Accepted boundary |
| --- | --- |
| Character | One self-contained VRM 0.0 or 1.0 `.vrm`, at most 50 MiB |
| Required body | 15 required semantic bones, at most 10,000 nodes, valid references/hierarchy and finite transforms |
| Character images | Embedded PNG/JPEG; at most 64 images, 4096 pixels per side, 64 million pixels total |
| Drum component | Self-contained static `.glb`, at most 25 MiB; body-only geometry expected |

Only explicit `VRM.specVersion = "0.0"` or `VRMC_vrm.specVersion = "1.0"` is accepted; unknown versions and files containing both extensions are rejected. `.vroid` is an editing
source, not an imported runtime model. Generic humanoid GLB mapping is not implemented. FBX uses the separate boundary below. External resource references are rejected; metadata links are displayed
without automatically fetching them. File metadata and source terms govern asset
use, independent of the application's MIT license.

VRM 0.0's `humanBones` array is checked before loading, including duplicate names
and node assignments. Legacy thumb `Proximal`/`Intermediate` map to normalized
`Metacarpal`/`Proximal`; `Distal` is unchanged. A positive-scale +Y half-turn wrapper
aligns the source roots from −Z to +Z **before** three-vrm captures rest transforms.
The scene stays at identity for the normalized rig subsequently added by the loader,
so both formats use the same coordinate, grip, fitting and evaluation path.

The inspection retains original metadata. VRM 0.0 uses `title`, `author`, `reference`,
`allowedUserName`, `violentUssageName`, `sexualUssageName`, `commercialUssageName`,
`otherPermissionUrl`, `licenseName` and `otherLicenseUrl`; no VRM 1.0 license defaults
are applied. Missing/invalid metadata is reported without inventing permissions.
Original strings and URLs are displayed as text and never executed or fetched.

Before three-vrm converts legacy materials, `vrm-materials.ts` applies UniVRM's
[documented migration](https://vrm.dev/en/univrm1/migrate_vrm0/feature/) to VRM 0.0
`VRM/MToon` only: a valid `_MainTex` index becomes `_ShadeTexture` when that
property is absent. Explicit shade maps (including index 0), colors, UV transforms,
non-MToon materials and VRM 1.0 are untouched. The parser JSON changes in memory;
source bytes/hashes remain identical. Invalid main indices are never repaired.
The migration is idempotent, reported in model details, and shared by local,
preset and project imports. MToon 1 shading remains in use; `v0CompatShade` is not
also enabled, avoiding two overlapping compatibility policies.

### FBX import

[fbx-import.ts](../../src/fbx-import.ts), [fbx-config.ts](../../src/fbx-config.ts) and
[fbx-rig.ts](../../src/fbx-rig.ts) own the binary FBX boundary. The pinned Three.js
0.186.1 loader handles binary 7.4/7.5 here; ASCII and other versions are rejected.
No new dependency or third-party model is bundled.

| Resource | Limit / validation |
| --- | --- |
| FBX source | 50 MiB; bounded binary record offsets and safe integer IDs |
| Binary structure | 100,000 records, depth 128, 100,000 properties per record, 256 texture objects and 128 material objects; model-parent cycles and multiple parents rejected |
| Arrays | 4M entries per array, 16M aggregate entries, 128 MiB aggregate declared expansion; actual deflate output must match each declared size before loader allocation |
| Loaded scene | 10,000 nodes, 1M mesh position vertices, 128 materials; imported cameras/lights removed |
| Images | At most 64 selected PNG/JPEG companions totaling 50 MiB; selected and used embedded images share a 64-image / 64M-pixel budget, at most 4096 pixels per side |
| Unit mapping | Finite metres per source unit in [0.000001, 10], suggested as `UnitScaleFactor / 100` (fallback 0.01); common measured-limb limits still apply |
| Mapping identity | Known humanoid semantic names to child-index paths (at most 512 characters); no missing required, duplicate or invalid-ancestry assignments |

All selected companion basenames must be distinct after NFC normalization and case
folding. Referenced images resolve only to explicit local companions or embedded
PNG/JPEG data; no external fetch is allowed. Repeated image references share a decoded source. Decoded textures and embedded blob
URLs are disposed on failure/cancel/replacement. Manual base-color assignment is
by stable material traversal index, uses sRGB and a white diffuse tint, and is
saved with the original image identity. Original mode preserves other supported Phong/Lambert properties.
This does not reconstruct authoring-tool toon/node graphs or layered materials.

`fbx.materialMode` accepts `original` (default), `matte` or `unlit`; omission in
older settings means `original`. Original preserves supported Phong/Lambert
properties. Matte removes direct specular highlights and environment reflections.
Unlit displays base color/alpha and vertex colors without lighting, tone mapping
or fog. Opacity, alpha test, sidedness, depth and blending remain. Switching is
reversible; variants are shared across meshes and inactive variants are disposed
on cancellation/replacement. Preview, adoption and project restore apply the same
choice, also retained in the browser's successful-setting cache.

Explicit `Material/Properties70/SpecularFactor` must be finite in [0, 1]. The
bounded scanner collects material IDs and mesh connection order; the importer
multiplies the Phong specular color in linear space once, without confusing
same-named materials or multiplying shared materials repeatedly. Missing factors
preserve loader behavior (multiplier 1). Source bytes are unchanged. This is a
loader correction, not inference of toon graphs or Sketchfab viewer settings;
there are no avatar-name defaults.

Folder selection collects VRM/FBX models and PNG/JPEG companions recursively,
ignores unrelated files and asks which model to import when several exist. The
same aggregate resource limits apply; ZIP extraction is outside this importer.
Setup exposes unresolved required bones first. Dropdowns exclude used paths and
incompatible mapped ancestry, preserve the current selection for repair, and
show name-matching choices before an opt-in list of other eligible bones.
A separate rest-pose preview marks the selected bone without changing source
transforms. It owns and disposes its own renderer and controls; the source model
remains owned by the staged import. Preview failure does not prevent manual setup.

Unique generic material/image-name matches may propose base-color assignments;
ties and recognized non-color map tokens remain unassigned. Original texture
bindings and valid saved choices take precedence. Every preview update restores
original maps/tints before applying current overrides, so clearing a choice is
reversible. Adoption still validates and applies the explicit resulting config.
After successful adoption only, browser storage remembers the most recent 10
setups keyed by FBX SHA-256. Records contain settings and image names, never
asset bytes. Restored settings pass schema and scene mapping validation; absent
companions remain unassigned. Storage failure is nonfatal; project files remain
the portable record. Cancel/failure does not update this cache.

Conventional humanoid, Mixamo, suffix and Japanese bone names are suggestions,
never model-specific rules. Ambiguous names remain unassigned. Validate the 15
required bones and optional hierarchy before adopting. The rest head-to-hips and
right-to-left upper-arm directions establish an orthonormal +Y/+X body basis;
reject degenerate axes. A positive unit-scale/rotation wrapper is applied before
`VRMHumanoid` captures rest transforms. Its normalized root remains under an
identity scene. The shared solver receives only measured semantic geometry,
retains segment lengths and drives the actual hand/foot sockets.

Unmapped bones keep their local rest transforms and follow ancestors. FBX animation
clips, MMD/Blender constraints, expressions and secondary-motion physics are not
played. Head-descendant masking uses the same first-person helper; non-indexed
triangles receive identity indices without changing UVs, skin weights or order.
Complex auxiliary rigs can still deform differently from their source tool.
The existing fit/adoption transaction preserves the active character on cancellation
or failure. The [project contract](project-format.md) owns saved mappings/relinking.

The UI calls `original` **File settings (automatic)**. The persisted key stays
unchanged for existing projects/preferences. Automatic honors supported explicit
file properties and version compatibility; it does not infer a missing website
shader from avatar names, image filenames or a neighboring GLB. Explicit saved
matte/unlit choices remain overrides.

### Bundled character selection

`public/characters/*-provenance.json` owns each bundled VRM's exact file identity,
creator, source and distribution terms. The preset selector fetches only the chosen
same-origin file, verifies its byte size and SHA-256, and passes it through the same
preflight, adapter and automatic-fit transaction as a local import. Bundled status does not bypass size/resource checks,
alter bone lengths, or select avatar-specific motion rules. The default skeleton
requires no VRM download; local uploads remain local.

AvatarSample_B, Seed-san and the Twist sample retain their embedded VRM Public
License settings and any additional terms. Model details expose those conditions;
Seed-san requires credit in applicable published outputs. The application does not
export modified avatar data. A preset is a reproducible input, not a compatibility
certificate: the Twist bounded-fit limitation and approach-contact warnings remain
visible. The [model guide](../../docs/models.md#bundled-character-presets) links the
individual provenance records.

## Calibration

[kit-calibration.ts](../../src/kit-calibration.ts) validates contact layouts;
[rig-humanoid.ts](../../src/rig-humanoid.ts) measures and translates the seated body.

- Seat height: 0.30–1.20 m. Forward/back adjustment: −0.50–0.50 m.
- Automatic seat fitting gives a pedal-aware starting position, not collision-free fitting.
- Explicit kit fitting adjusts hand-played contact positions/radii together; the kick
  remains fixed while the direct hi-hat moves as a coupled pedal/mast/cymbal assembly.
  [Calibration](calibration.md) owns search bounds and
  the limited meaning of a complete proposal.
- Target X is bounded to ±2 m, Y to 0.02–2.50 m and Z to −0.50–2.50 m.
- Kick and hi-hat pedal height is further bounded to 0.04–0.20 m.
- Closed and open hi-hat share one movable upper surface; its opening and the
  mechanically connected pedal use the same deterministic control state.
- Kick's engine contact is the foot pedal, not the visible front drum head.
- Changing layout updates `plan.kit` and matching `hit.target` together, then replans
  with the measured profile. Preserve original MIDI data and audio.

Calibration is unavailable during conflicting operations.
Seat translation alone cannot accommodate every arm-to-leg ratio; retain reach
failures rather than altering limb lengths or source times.

## Hands, feet and repeatability

The grip uses a measured diagonal hand-local mount and available finger curl.
The default `natural` palm preference is oblique inward/downward (45° between
those directions), with up/inward/down retained as explicit alternatives. This is
an engineering starting pose, not a claim that one grip suits every drummer.
Symmetric palms-up is not a complete traditional grip. The thumb solver jointly
minimizes pad-to-shaft distance and distal-axis misalignment with bounded rotations
about the model's rest joints; it never extends finger bones. Missing optional
fingers retain reduced grip detail. Optional measured `handFrames` let the common solver
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

Feet follow the calibrated footboard hinge/contact even without pedal notes.
The leg solves the moving sole marker without stretching bones; the renderer uses
the same foot quaternion for its board. One absolute-time hi-hat state controls
the foot, upper cymbal and stick target. Kick beater and sole contacts are separate.
[Equipment/contact](equipment-contact.md) owns the mechanism frames;
[calibration](calibration.md#feet-and-support) describes their support limitations.
This is kinematics, not full heel-up/down dynamics or measured shoe-surface support.

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
stands, skin collision, explicit crossed-arm styles and physical force simulation
are not established by this contract.
