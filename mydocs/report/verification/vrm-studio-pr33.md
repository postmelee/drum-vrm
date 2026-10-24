> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/vrm-studio-unreleased.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../specs/README.md) for current contracts and policy.

# Studio and VRM batch verification

Date: 2026-10-24. macOS, Chrome 155. **Unreleased**: no tag, GitHub Release or Pages deployment. Public v0.2.0 remains at `3e8be9333cedbc9845dbc3e4cebd73961af697b4`.

## Changes and automated checks

PR #31 passed CI and merged at `28a50a2c19c2b5109c490384f0b03e554f4dea1e`. PR #32 completed #28/#29 and merged at `bd1568d4a3cc89913627b04e34f2fd2153a3ffac`; its final [CI run](https://github.com/postmelee/drum-vrm/actions/runs/40068065398) passed.

The subsequent rig change adds VRM 1.0 local import, a shared semantic humanoid skeleton, measured-body Rust profiles, rotating hand-local sticks, matched finger pose, and actual socket measurement. [ADR 0004](../../../specs/decisions/0004-vrm-humanoid-and-grip.md) records the contract.

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --locked`: **22 passed**.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `npm test`: **63 passed across 12 files**.
- `npm run typecheck`: passed.
- `npm run build:wasm -- -- --locked`: passed using the installed pinned toolchain.
- `npm run licenses`: checked 19 production npm packages, 21 Rust dependencies and 3 bundled drum-model provenance records.
- `npm run build:web`: passed. Vite reports the existing large-chunk warning; this is not a performance certification.

The suites cover self-contained resources, malformed VRM/bone metadata, image budgets, missing optional fingers, nonidentity rest bases, two authored proportions, actual constraints, positive skeletal transforms, first-person mesh handling, source MIDI preservation, profile reach, grip/contact and seek determinism. No third-party character file is in the public tests or build.

## Local browser checks

Studio controls: SVG transport/theme controls in both themes; player viewpoint and return from drum editing; Batoski and smoj loading; response toggle and disabled strength while off; play/pause. Batoski floor-tom stand fitting was checked against the actual imported GLB, preserving shell dimensions/contact while adjusting only hardware height. Material/geometry assumptions and limitations are in [drum integration](../../tech/research/2026-10-24/drum-kit-integration.md).

Actual VRM files were selected through a local file chooser. A private harness used the production importer/adapter, Rust WASM and actual materials in WebGL. It evaluated the same 14-hit MIDI on three models, 128 fixed 30-fps timestamps, then shuffled the timestamps. Actual raw-bone matrices and hand-local/sole contacts were compared. All three preserved the original hits, had finite positive transforms and exact repeated pose/matrix values. Assigned-hit maximum error was `3.90e-7 m`; all-frame marker maximum was `4.32e-7 m`.

**Each model assigned 11/14 hits.** Notes 41, 49 and 51 were unreachable for the fixed kit and were diagnosed, with their original MIDI/audio retained. Kick and hat pedal were assigned. This is a measured subset, not full performance compatibility. Exact file hashes, model rights, constraints and per-model values are in [VRM evidence](../../tech/research/2026-10-24/vrm-models.md#actual-browser-harness-results--2026-10-24).

In the actual app, the Twist sample loaded after the 30-second demo was prepared. The panel displayed model/version, required bones 15/15, fingers 15/15 on each hand, and creator. Playback, player-view head handling, model-only and skeleton-only visibility, and profile-driven diagnostic refresh worked. The demo preserved 193 hits; 10 were unassigned for this model. The built-in semantic rig leaves 4 unassigned rather than stretching its bones to hide misses.

## Export evidence and its limit

The new smoj preset with player view and hit response completed a 923-frame, 30.77-second WebM with synthesized audio; the app reported 10.3 MiB. The actual Twist avatar, front view and model-only display completed a second **923-frame, 30.77-second WebM with synthesized audio**, reported as 7.3 MiB. The same pose evaluator and adapter render the frames; the UI restored after completion.

These results establish the encoder/mux completion path and reported output contract. The current browser automation provider does not support retrieval of blob downloads, so a newly downloaded file was **not independently inspected with ffprobe or replayed in a separate player** in this run. An older file in Downloads was deliberately excluded after checking its timestamp. Prior v0.2 file verification must not be presented as proof of this new avatar output. A manual download/playback check is part of the release review, alongside the pending full performance checks.

Local-only evidence: `.local/vrm-browser-evidence.txt`, `.local/vrm-export-review.png`, and the numeric baseline scripts/reports. Avatar-containing evidence is not added to public assets. [Rendered motion baseline](rig-motion-baseline.md) separately measures actual stick endpoints, wrist speeds/accelerations and stroke direction; it does not claim physiological realism.

## Remaining scope

- #17: explicit mapping for generic humanoid GLB; VRM 0.x is conversion-only.
- #18: two controlled body variants created in VRoid Studio, visible wrists/legs, user seat/grip/kit calibration and actual shoe/skin collision. The three independent samples do not replace that controlled pair.
- #20/#23: motion smoothness, instrument technique, collision avoidance and full trajectory recording. Secondary hair/clothing physics remain deliberately static for deterministic playback/seek/export.
- No DRUMS/Disney weights, motion datasets or source code were integrated. No model redistribution permission is inferred from a video-use permission.
- Release/deployment remains blocked on explicit owner approval of the batch, not on CI alone.
