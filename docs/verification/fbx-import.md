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
