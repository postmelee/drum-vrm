# Local FBX import verification

Date: 2026-10-26–27 (Asia/Seoul). Scope: [#56](https://github.com/postmelee/drum-vrm/issues/56),
stacked on [#55](https://github.com/postmelee/drum-vrm/pull/55).
Implementation: `9d776d12e8b289ea2231ff4a4a99c817de50aca0`; resource follow-up:
`de833f976591876b0c5d3e6ce8da91dc7cec5214`; final texture API cleanup:
`db007de52802cb21068ef4da105fe490fb06609e`.
No Rust source changed. The isolated build rebuilt the base engine rather than
using concurrent changes from the owner's other checkout.

## Environment and checks

- macOS, Node 24.15.0, npm 11.12.1; pinned Three.js 0.186.1,
  three-vrm 3.5.5, Vite 8.3.5 and Vitest 5.0.5.
- `npm run build:wasm` passed. WASM SHA-256:
  `cdcce294bb1ac06c13cbe13e0003996b9d1d69a6e20450f5ba848a9937f5de2c`.
- `npm test`: 30 files / 225 tests passed. `npm run typecheck` and
  `npm run build:web` passed. After the final API cleanup, all 20 FBX/project
  boundary tests and the embedded-image browser harness passed again. Vite retains
  its large-chunk advisory.
- `npm run docs:check` and the final scoped whitespace check passed.
- Browser development validation used localhost port 5179; production preview
  used port 5180. These are local checks, not a Pages deployment.

## Independently authored regression inputs

`tests/fixtures/fbx.ts` writes binary FBX records from the repository's authored
humanoid fixture. No source model, animation or texture data was copied into it.
The real FBXLoader-to-adapter path covers binary 7400 and 7500, metre/centimetre
units, opposite source facing, nonidentity raw rest rotations, and optional fingers.
Measured upper/lower arm lengths remain 0.25/0.22 m. Actual hand-stick and
foot-sole socket errors stay below 1e-6 m; returning to a pose after other poses
restores identical raw-bone matrices.

Other cases reject unknown/ASCII/truncated files, model-parent cycles, excessive
array expansion, actual inflated data larger than its declaration, excessive
texture objects, oversized image dimensions/aggregate pixels, duplicate image
basenames and invalid/duplicate/cross-limb semantic mappings. A missing external
texture fails without any `fetch` call. Project tests preserve native/VRM schema-1
compatibility and validate FBX settings, kinds, texture references and hashes.

An ignored browser harness generated a 1×1 PNG using canvas, embedded it in an
authored binary 7500 file and referenced it through two texture objects. Actual
image decoding succeeded; two blob URLs were created (embedded data and decode),
one was revoked after decode, and both had been revoked after disposal. The two
references shared the decoded image source. This checks browser decode/lifetime,
not the visual fidelity of arbitrary embedded textures.

## Private source inspection and actual browser checks

The owner explicitly requested inspection of a local downloaded collection.
Six FBX files were inspected: five binary 7400 exports identified Blender in their
metadata, while one binary 7500 export identified Assimp. One Blender export used
Japanese/MMD-style names with helper/control bones; others used semantic names
such as `Hip`, `Arm_L`, `Elbow_L`, and `Wrist_L`. Metadata identifies the exporter,
not necessarily the model's original authoring tool. All reported centimetre
units. Five packages contained PNGs without FBX image bindings, consistent with
the owner's source-page screenshot reporting zero textures. Rigged geometry alone
does not establish a humanoid mapping or texture assignment.

Private loader/adapter checks exercised all five Blender exports (646–866 bones),
including explicit resolution of the Japanese rig's ambiguous head/ankle names.
Their rest-pose actual socket errors were below 1e-5 m. This is not full-song
visual certification for all six downloads; the Assimp file received metadata
inspection only.

Two private Blender exports were exercised in the app:

- A 704-bone / eight-material model: automatically suggested required bones,
  explicit five-image material assignment, automatic fitting and the 193-hit
  Studio Groove demo. Forward/backward seek, texture rendering and project save
  succeeded. Reopening first kept the current session while originals were
  missing. A renamed identical FBX and renamed image relinked by SHA-256; the app
  reconstructed original texture names, checked the exact measured profile and
  restored the eight-second pose/camera/settings. Production preview repeated
  this restore. An invalid ASCII FBX was rejected; cancel preserved that character
  and the current performance, which remained seekable.
- An 866-bone / six-material Japanese rig: auto-suggestions deliberately left
  head and both feet blank because alias matches were ambiguous. Confirmation
  reported these missing assignments. Selecting the actual head/ankle bones and
  five companion images enabled adoption and demo preparation. The final
  production build repeated this import with the resource follow-up included.

First-person masking was exercised on the first model. It follows mapped head
weights/descendants; unmapped accessories and nearby clothing can remain visible
or occlude the view. The helper is not a guarantee of clear vision for arbitrary
control rigs. No functional app errors appeared during the successful import/restore checks.
The resource follow-up exposed one deprecated `THREE.Source` alias warning in
the production browser; the final cleanup uses the pinned `THREE.TextureSource`
API, with a successful build, boundary tests and repeated embedded-image harness.
An unrelated private harness import-path error during development was
corrected before the embedded-image test; it was not a production failure.

## Setup usability follow-up — 2026-10-27

Implementation: `9a3cddb` (stacked on the implementation above). The owner asked
for remaining-candidate dropdowns and the proposed simpler import workflow.
No Rust, WASM, dependency version or project schema changed in this follow-up.

- Folder selection discovers models and PNG/JPEG companions in subfolders and
  asks which model to use if multiple exist. Irrelevant package files are ignored.
- Unresolved required bones are exposed first. Used/incompatible candidates are
  excluded, conventional-name matches are shown first, and other eligible bones
  are available through a checkbox. Current choices remain clearable. Assigned
  and optional bones remain editable in disclosures.
- An isolated rest-pose preview shows selected bone positions. Generic image-name
  suggestions are reversible; original texture bindings and saved choices win.
  Successful exact-FBX settings are remembered locally for 10 recent files.

Checks: full suite **31 files / 229 tests passed**; focused FBX/setup/project
checks **24 tests passed**. Typecheck, final production web build and documentation
check passed. The existing large-chunk build advisory remains. New authored tests
cover remaining-candidate exclusion/reappearance, ambiguous aliases, conservative
image suggestions, storage limits/unavailability and folder file classification.

Actual browser checks used the owner's private 866-bone model. Folder selection
collected the FBX and five images in one action. The unresolved list contained
head and both feet; filtered conventional choices were two head candidates and
one candidate per foot. Five of six materials received unique image suggestions;
the ambiguous eyebrow material remained manual. Bone markers, texture edits,
clearing a texture back to the original material, automatic fitting and successful
adoption were observed. The production preview on port 5180 repeated import and
adoption, restored successful mappings/textures, and preserved the current model
and last successful settings after a texture change was canceled. A multi-model
selection displayed an explicit chooser and could be canceled. No warning/error
console entries were captured in this production check.

The 745-pixel layout stacked the preview above controls; a temporary 1280×900
viewport verified the two-column layout and was reset afterward. Screenshots
remain in ignored local evidence; no private model, texture or screenshot is
tracked. Preview positions and filename suggestions do not establish deformation
correctness, shader fidelity or new all-song performance guarantees. Folder image
budgets are still aggregate; select a single extracted package rather than a
large collection, and resolve duplicate image basenames before import.

## Material appearance follow-up — 2026-10-27

Implementation: `1774dcd`. The owner requested selectable original/matte/unlit
appearance and correction of the omitted FBX specular intensity. New imports
start with original materials; cached settings and projects preserve an explicit
choice. Missing mode fields in older schema-1 projects retain original mode.
No Rust, WASM or dependency version changed.

The bounded binary scanner reads explicit material `SpecularFactor` values in
[0, 1]. Material IDs and ordered model connections match the pinned FBXLoader
output, avoiding name collisions. The factor multiplies the linear Phong specular
color once per shared material. Missing factors keep loader behavior. Matte
removes specular highlights and environment reflection; Unlit uses base color,
alpha and vertex colors without lighting, tone mapping or fog. Original settings
remain available when switching back; active and inactive material resources are
released. These are rendering choices, not reconstructed toon shader graphs.

Private evidence: the owner-provided seven-material FBX had white specular colors
and explicit factor 0.25. Actual loader output previously used `[1, 1, 1]`; the
corrected output uses `[0.25, 0.25, 0.25]` for all seven. A second private FBX with
six materials showed the same correction. The matching locally downloaded GLB,
whose metadata identifies the owner's linked
[Sketchfab page](https://sketchfab.com/3d-models/umamusume-gold-ship-race-2cb9610f06a34b959cde35f2009894d3),
uses `KHR_materials_unlit` for all seven materials. The observed site preview is
flat and texture-focused; this evidence motivated the owner's explicit Unlit
choice, not any filename rule in the app.

Validation:

- Full suite: **32 files / 237 tests passed**. Focused FBX/material/setup/project
  suite: **32 tests passed**. Typecheck, production web build, documentation and
  whitespace checks passed. The existing large-chunk advisory remains.
- Authored binary 7400/7500 mesh/material fixtures verify factors 0/0.25/missing,
  duplicate material names, shared materials and no repeated multiplication after
  mode switching. Invalid factors are rejected before loader allocation.
- Material tests check reversible/shared variants, alpha, sidedness, vertex colors,
  changed textures/colors and inactive-variant disposal. Project tests round-trip
  every mode, preserve missing legacy fields and reject unknown modes. The browser
  preference test now includes an explicit Unlit setting.
- Actual browser imports compared all three modes on the private model. Unlit
  visibly removed the highlights. The production preview at localhost:5180 repeated
  texture selection, Unlit adoption and the 193-hit Studio Groove preparation.
  The saved project contained `materialMode: "unlit"`; after switching to the
  default skeleton, reopening restored the textured Unlit character and showed
  its mode in Character settings. Timeline seek remained usable. No warning/error
  console entries were captured in that production check.
- A private adapter check compared the actual model's measured profile across all
  three modes and found exact equality. The model still reports automatic-fit /
  ready-pose clearance limitations and the demo has 12 unassigned hits; this work
  changes appearance only and does not claim to resolve those motion constraints.

Private models, GLB metadata, screenshots, diagnostic probes and the saved project
remain outside tracked files. This does not certify pixel-identical rendering to
Sketchfab, arbitrary toon fidelity or video output. The local preview was rebuilt;
no merge, release or public deployment was authorized or performed.

## Limits and release boundary

The implemented contract accepts binary FBX 7.4/7.5 only. It preserves supported
Phong/Lambert materials and explicit base-color assignments, not Blender/MMD toon
node graphs, constraints, physics, expressions or embedded animation playback.
Unmapped bones follow ancestors with their local rest transforms. More complex
auxiliary rigs may deform differently from the original program. Source timing,
common fit limits and diagnostics remain intact; successful loading/contact tests
do not establish natural posture, all-frame skin/clothing clearance or video
export fidelity. No new video export certification is claimed.

All downloaded files, extraction results, screenshots, saved private projects and
private probes remain outside tracked/public files. No new model distribution
rights are inferred from the source-page license label. There is no new bundled
asset or dependency. The work is for review; merge, release and Pages publication
remain unauthorized.
