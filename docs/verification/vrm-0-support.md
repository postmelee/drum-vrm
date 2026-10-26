# VRM 0.0 import verification

Scope: [#54](https://github.com/postmelee/drum-vrm/issues/54), under #17.
Runtime implementation: `a4e0309` (base `0d012e8d384c9b30292ff8cd8d7171b98e4b1956`).
Checked on 2026-10-26, macOS, Node 24.15.0, pinned three-vrm 3.5.5 and Three.js
0.186.1. No dependency or Rust implementation changes belong to this batch.

## Implementation and rationale

The importer accepts explicit VRM 0.0 and 1.0 extensions with shared resource,
image, hierarchy and transform checks. Legacy array entries and thumb names are
normalized during preflight, with duplicate semantic/node assignments rejected.
Original v0 metadata remains separate from v1 license defaults.

The [official migration guide](https://pixiv.github.io/three-vrm/docs/documents/migration-guide-1.0.html)
explains the opposing forward axes and `rotateVRM0`. This adapter also requires
identity-oriented normalized bones. Accordingly, it rotates a wrapper around the
source roots before the VRM plugin captures rest transforms, leaving the scene at
identity for the normalized root added afterward. It does not rotate the completed
normalized rig or change bone scales. The pinned loader's v0 thumb conversion and
[original metadata schema](https://github.com/vrm-c/vrm-specification/blob/master/specification/0.0/schema/vrm.meta.schema.json)
were checked against the preflight mapping.

## Executed checks

- `npm test`: 29 files / 213 tests passed before the final additional image-budget
  case. `npm test -- tests/vrm-legacy.test.ts`: all 7 legacy tests passed afterward
  (214 tests in the final suite). `npm run typecheck` passed on the final source.
- `npm run build:web`: passed; Vite retained its large-chunk advisory.
- `npm run docs:check` and `git diff --check`: passed.
- Real GLTFLoader + VRMLoaderPlugin + shared adapter integration uses independently
  authored GLBs: two body scales (0.86, 1.12) with fingers and one without fingers.
  Nonidentity raw rest rotations, v0 thumb conversion, anatomical sides, forward
  toes, body/player measurements, positive determinants and raw-hand tool parenting
  are checked. v0/v1 numeric profiles agree within 1e-6; actual contact error is
  below 1e-6 m, and repeated pose ordering restores identical raw world matrices.
- Legacy metadata tests retain original values/URLs as text, report invalid or
  missing terms, and avoid v1 defaults. Invalid versions, conflicting extensions,
  duplicate bones, malformed transforms/hierarchies, external images and the
  existing size/image budgets are rejected.
- Local in-app Chromium browser: authored v0 import and automatic fit succeeded;
  Studio Groove prepared 193 hits, playback/pause and seeks to 12.5 s then 3 s
  worked. Model details showed VRM 0.0 and 15/15 fingers on both hands. English/
  Korean switching retained playback position. Replacing v0 with the official
  AvatarSample_B v1 preset succeeded. No browser warning/error logs were captured.

The browser used the existing local WASM artifact with SHA-256
`cdcce294bb1ac06c13cbe13e0003996b9d1d69a6e20450f5ba848a9937f5de2c`.
No Rust change or WASM rebuild is part of this feature. Concurrent, unrelated
Rust working-tree edits are excluded from the commits and validation claims.

## Supplemental local material diagnosis

An owner-supplied private VRM 0.0 file imported and rendered, but appeared white.
All five MToon materials supplied main textures and white shade colors without
shade textures. A separate local comparison copy assigned each existing main
texture to its shade slot; binary payload and all other JSON fields were unchanged.
With identical application lighting, the hair, face and clothing colors returned.
No browser warnings/errors were captured. The original and comparison copy both
retained explicit incomplete-fit warnings. This is a source material configuration
case, not evidence for changing every imported model's materials or global lights.
No private avatar bytes, textures, metadata or screenshots are included in this PR.
The [model guide](../models.md#a-legacy-model-looks-washed-out) documents the general diagnosis.

## Limits

Authored integration fixtures are bone-only; they do not certify arbitrary legacy
materials, skin, clothing, blend shapes or dynamic spring motion. Existing fitting,
contact and natural-motion limitations remain. No video export certification is
claimed. The new feature is implemented locally and requires owner review before
merge or publication; the recorded Pages deployment remains unchanged.
