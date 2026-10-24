# Calibration, planted pedals, grip, and shared hardware

Date: 2026-10-24, macOS / Chrome. Implemented on `codex/calibration-pedals-hardware` after main `a2105a5508ae92e49d31977e2c56b096dfd1d822`. **Unreleased**: no tag, GitHub Release or Pages dispatch. Public v0.2.0 remains unchanged. Related work: #18, #14, #20; [ADR 0005](../decisions/0005-calibration-pedals-hardware.md).

## Automated checks

- Rust: `cargo test --locked` — 26 passing; `cargo fmt --all` and `cargo clippy --locked --all-targets -- -D warnings` passed.
- Web: `npm test` — 76 passing across 14 files. Typecheck, WASM rebuild, production build, license generation and whitespace checks passed. A later helper-only move restoring lazy preset loading passed the 10 affected library/hardware tests and typecheck, followed by another production build.
- Regression coverage includes fixed ankle/knee/hip at idle and strokes, missing pedal notes, impossible contacts without bone stretching, unchanged MIDI data, coupled hi-hat targets, bounded pedal heights, arbitrary seek, seat reset, nonidentity rest bases, optional fingers, degenerate knee bend planes, scene clearing and stable hardware identity/transforms across body/preset/scale changes.
- Source GLBs and license texts are unchanged. Provenance and generated notices explain runtime removal of source hardware and normalization to common mounts. Private VRMs are excluded from source/build.

## Real model evidence

The three previously recorded private files were reused; identities/terms are in [model research](../research/vrm-models.md). An extracted-node/constraint harness tested each at automatic fitting, 0.60m seat / 0.12m forward, and deliberately unreachable 1.20m / −0.50m. Across nine combinations: maximum raw-joint error `2.3044e-7m`, attached marker error `3.9849e-7m`, assigned-hit contact error `1.0394e-7m`, random-order world-matrix delta zero. Impossible pedals produce explicit diagnostics. These measurements concern actual adapter transforms, not skin collision.

A separate browser harness loaded full original `.vrm` files through the application importer and WebGL renderer. It evaluated a 14-event fixture, all 128 output-frame poses, and shuffled frame access:

| File | Assigned events | Maximum assigned contact error | Maximum marker error | Seek matrix delta |
| --- | ---: | ---: | ---: | ---: |
| Seed-san.vrm | 12/14 | 4.590e-7m | 4.427e-7m | 0 |
| twist-three-vrm.vrm | 11/14 | 1.241e-7m | 1.792e-7m | 0 |
| AvatarSample_B-chatvrm.vrm | 11/14 | 1.131e-7m | 1.852e-7m | 0 |

All original events survived; positive transforms and finite poses passed. Unassigned outer targets remain explicit and audible, not silently repaired. Left/right Twist hand close-ups visibly show fingers curling around the transverse shaft. AvatarSample_B's sleeves obscure part of the grip. This is a common matched-grip approximation, not certification of natural wrists or zero skin penetration.

## Application workflow

On `/drum-vrm/` at local port 5177:

- Applied 60cm seat / 12cm forward before MIDI, then prepared the 193-event, 30-second demo. Later imported AvatarSample_B and Twist, changed seats, played/paused, and sought to output time 12.5s.
- Moved snare contact to `(−15, 82, 35)cm`; changed Studio to Innerscene; replaced the snare with an authored body-only test GLB and restored it. The target fields retained their values and the shared hardware remained present. Unit tests additionally verify object/matrix invariance.
- With Twist, Innerscene, the edited snare, 60cm seat / 12cm forward, the app completed synthesized-audio WebM encoding: **923 frames, 30.77s, 6.3MiB**. Calibration/model controls were disabled during encoding and restored afterward.
- The current browser provider cannot return blob downloads for an independent file-player/ffprobe check. This run proves the application's encoding/mux completion and pose path, not independent playback of the newly encoded file. The completed download remains available in the local tab.

Private screenshots, raw numeric results and source files remain in ignored `.local/`; they are not redistributable fixtures.

## Remaining scope

- Explicit crossed/open-handed/mirrored style, hand overrides and arm/stick clearance are specified in #14/#20. Existing RH-hi-hat/LH-snare cost preference may select another feasible hand.
- Shoe/skin collision, physical pedal-board/beater mechanics, exact clamp fitting, arbitrary GLB segmentation and collision-free kit layout remain unimplemented.
- Seat/target settings last only for the current session. Controlled short/tall VRoid Studio exports remain an open #18 acceptance item; the three independent samples do not replace that requirement.
- A production build passed; no public deployment or broad browser/platform compatibility was tested for this batch.
