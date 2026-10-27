# Local project format

**English** · [한국어](project-format.ko.md)

[project.ts](../../src/project.ts) owns schema 1 of `drum-vrm-project`.
It is a local JSON container, bounded to 32 MiB; saving and relinking do not transmit
files. See the [user workflow](../../docs/projects.md).

| Field | Responsibility |
| --- | --- |
| `format`, `schemaVersion`, `engineModuleSha256` | Container identity and exact WASM binary used for replay |
| `plan` | Full schema-2 performance plan, including selected source messages, interpretation, controls, original timing and assignment |
| `selection` | Tracks/channel, mapping profile, count-in choice, hi-hat policy/aperture, omitted source IDs and manual hand overrides |
| `profile`, `rig`, `calibration`, `kit` | Measured common humanoid contract, native/VRM/FBX choice, seat/palm settings and physical layout |
| `assets` | Kind, display name, SHA-256, byte size and optional drum-piece ID; never a fetch URL or embedded binary |
| `appearance` | Bundled preset and per-piece procedural/library/imported choice, finish, explicit `finishOverride`, scale and rotation. A displayed default color must not tint original library/imported materials unless it was chosen as an override. |
| `camera`, `hitResponse`, `view` | Actual camera position/target, view mode, feedback, theme/background, visibility and piano-roll zoom |
| `audio`, `outputTime` | Synthesized/WAV choice, explicit song-time WAV offset, volume and output-time playhead |

The project preserves selected MIDI messages and derived motion data. It is **not**
the privacy-minimized review report and should not be published as if it contained
only settings. It excludes raw input files, audio PCM, model geometry and asset
permissions. Hashes establish same-content relinking, not authorship or reuse rights.

## Restore transaction

The browser checks schema/version, finite bounded settings, required piece identities,
unique asset references and consistency between selected policies/layout and plan.
Missing references leave the active session intact. Relinking matches SHA-256 and
byte size; a renamed identical file is accepted, a same-name modified file is not.
MIDI is required even though the plan retains selected messages, so later track and
mapping edits still have the original input. WAV/VRM/FBX/selected images/custom GLB are required only
when used by the project. Bundled geometry is loaded from the app's own preset assets.

Before committing, the app re-parses the matched MIDI and compares retained source
events, timing maps, interpretation and controls with the saved plan. It constructs
the actual rig, reapplies calibration, checks the measured profile, asks Rust to
validate/rebuild the performance, decodes WAV and prepares all external drum geometry.
Failed preflight leaves the currently rendered session unchanged. The final settings,
scene and audio replacement occur only after these asynchronous checks complete.

Rust remains authoritative for plan/rig validity. The browser does not accept stored
limb assignments, velocities or controls merely because the JSON parser succeeded.
The source/mapping/intent boundary remains in [MIDI interpretation](midi-interpretation.md);
WAV alignment remains in [audio/export](audio-export-contract.md).


Saved profiles include optional numeric body envelopes, not mesh vertices. The browser validates their bounds and compares the restored measured profile; reopening does not rerun automatic layout fitting. Old files without geometry retain their original schema interpretation, but exact engine-hash compatibility is still required.

## FBX project settings

Schema 1 also accepts `rig: "fbx"` with a required `fbx` object containing
`bones` (semantic names to stable child-index paths), `metersPerUnit` and
`textures` (material traversal indices to normalized companion basenames). Optional
`materialMode` accepts only `original`, `matte` or `unlit`; omission in older
projects restores automatic file settings (`original`). The choice is applied with materials/textures
without changing bones, body measurements or the performance plan. Original mode
also receives the [FBX contract](rig-contract.md#fbx-import) specular-factor correction.
Native/VRM projects omit this object. The original FBX has one `fbx` asset reference;
selected PNG/JPEG files use distinct `fbxTexture` references. At most 77 total asset
references and 64 FBX images are accepted; selected image bytes total at most 50 MiB.
The [FBX contract](rig-contract.md#fbx-import) owns bone/unit/image bounds.

Reject inconsistent rig/source kinds, duplicate case-folded image names, or material
assignments without matching image references. Restore reconstructs original names
from the saved manifest after hash/size matching, so renamed identical files still
resolve texture bindings. It reruns the loader, mappings and measured-profile check;
settings are committed only after the existing restore transaction succeeds.
Existing native/VRM schema-1 projects remain accepted without FBX settings.

## Compatibility and evidence

Schema 1 requires the exact stored WASM SHA-256 and matching planner version. A mismatch
blocks exact restore with an instruction to use the original app build or prepare
the original MIDI again. There is no silent engine migration, asset substitution,
automatic download or upload. Source assets can be relinked through the local file
picker. The engine hash alone does not certify identical pixels across renderer,
browser, hardware or model-loader revisions; retain the matching app build for review.

Verification must compare original times and interpretation, arbitrary-time poses,
camera/settings and WAV alignment before/after reopening. A project reopening with
all events accounted for does not establish naturalness or skin clearance. Export
quality extensions and cross-version migration are separate work.
