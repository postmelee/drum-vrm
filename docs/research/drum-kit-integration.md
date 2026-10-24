> Research record preserved from [baseline 66951cd](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/research/drum-kit-integration.md). Dated findings and statements of implementation status below belong to that record; use the [current specifications](../../specs/README.md) and [model guide](../models.md) for current behavior.

# Bundled kit integration — 2026-10-24

Issue #29 adds two different original model kits alongside Innerscene. These are distinct source geometries, not finish variants of the existing procedural kit. The preset IDs are `library` (Innerscene), `batoski`, and `smoj`. All three use the same `loadLibraryKit(preset)` adapter boundary. The engine knows instrument roles and contacts, not source object names.

| Preset | Original model | License | Original GLB bytes | SHA-256 |
|---|---|---|---:|---|
| `library` | [Five-piece drum kit — Innerscene](https://www.innerscene.com/tools/library/3d-parts/five-piece-drum-kit-daff10f4) | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | 3,361,144 | `3cf8d568b604e99ce63ee82829d7f25044d996ed6cd42c39bc7f226919d1a5a7` |
| `batoski` | [Full Drum Kit — Batoski](https://poly.pizza/m/1HvDgRF7fX) | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) | 788,112 | `1ced36a850608f146407ad9387208209c59dd4784377a41c53727632bf00f895` |
| `smoj` | [Drum Kit — smoj](https://poly.pizza/m/7FCPjy-3xmz) | [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/) | 416,052 | `4893bf0cd89542862298456be893bce24742947d31c8293230b817e91629d5de` |

Original downloaded binaries are unchanged and self-contained: no external texture/buffer URLs. Exact download URLs, license evidence, source inspection, binary and legal-text hashes, and runtime changes are in `public/models/*-provenance.json`. `public/models/NOTICE.txt` gives the model title, creator, source, license and modification notice. `scripts/licenses.mjs` verifies every recorded asset/license hash and incorporates all three into the deployed `licenses.txt` and repository notices. It refuses an unexpected model count so adding another asset requires an intentional inventory review.

## Calibration and response roles

Each kit produces kick, snare, hi-hat, hi-hat pedal, three toms, crash and ride groups. Open hi-hat uses the hi-hat visual. Surface contacts are downward ray intersections with the actual source mesh; cymbal contacts are away from the center mounting nut. Foot contacts are footboards, not bass shells. Coordinate reflection reverses triangle winding. The source scene is consumed and its geometry/materials disposed; the caller owns returned groups and shared textures.

Meshes are merged by instrument, material and response role. `surface` and `shell` meshes may respond to a hit. `hardware` and `pedal` meshes stay fixed. This preserves separate shell/cymbal response and fixed stands even after batching. Batoski combines its floor-tom legs and kick spurs into the drum meshes; welded connected components separate these into fixed hardware. Smoj uses a pinned explicit inventory of source group IDs and material primitives. Innerscene keeps its authored semantic names inside the adapter.

Batoski's original kit has one rack tom and two floor toms; those fill high/mid/low roles. Its second crash and stool are omitted because the current planner has one crash role and the rig owns seating. No independent footboards exist in this source, so the application supplies simple original footboards/bases for the kick and hi-hat. These additions are disclosed in the modification notice. Smoj's original stool and carpet are omitted; its flat colors and translucent drum surfaces are retained. Innerscene maps are reduced to at most 512 pixels at runtime; the archived source textures remain unchanged.

## Verification and limits

On 2026-10-24, `npx vitest run tests/library.test.ts` passed all 6 tests and `npm run typecheck` passed. Tests decode all three real GLB hierarchies (only browser image references are removed in the test copy), verify source/license hashes and notices, assert nine distinct finite surface contacts, ray-check each actual surface, check handedness and draw count, and bind the real meshes to hit response. With a transformed parent and a nonzero response, tagged hardware keeps the same world matrix and the contact pivot keeps the same world position.

A local Three.js browser inspector rendered Batoski and smoj with their actual source materials and the application's uniform part fitting. The geometry, distinct finishes, cymbals, drumheads and stands displayed. A raw-source comparison confirmed smoj's authored geometry/material appearance. The application preserves drum proportions and aligns contacts by translation/uniform scale. Initial rendering exposed floating floor-tom legs when the Batoski floor tom filled the higher middle-tom target. The fitting helper now stretches only explicitly tagged, floor-connected stand hardware about the source contact to reach the floor. Used indexed vertices determine the floor; elevated rack mounts, pedals and bass hardware are excluded. Regression tests verify actual stand floor coordinates, fixed surface contact and unchanged shell scaling at a changed target height and uniform scale. This is display fitting, not a physically accurate telescoping mechanism; unusual placements can still produce intersections or gaps at mounts. Batoski heads/shells remain one response group where authored together. The models are stylized, and no claim of photorealism, popularity or industry-standard status is made.

These assets are implemented for local/PR review. This record does not authorize a release or deployment. The electronic candidate in the earlier research remains unverified and is not bundled.
