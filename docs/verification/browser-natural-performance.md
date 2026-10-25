# Browser performance and common-rig verification

Measured locally on 2026-10-25–26 against `codex/natural-browser-performance`,
based on `c116ca105a79767381d90a74b89ab2903b9d0f5b`. The PR records the reviewed
commit; the WASM hashes below identify the measured runtime candidates. These
results do not establish a release or the version running on GitHub Pages.

The [MIDI contract](../../specs/architecture/midi-interpretation.md),
[calibration contract](../../specs/architecture/calibration.md),
[engine contract](../../specs/architecture/engine-contract.md) and
[project format](../../specs/architecture/project-format.md) own current behavior.
The [earlier MIDI report](midi-interpretation-foundation.md) retains its different
input revisions and original measurement scope.

## Inputs and measured candidates

Environment: macOS, Node 24.15.0, Rust 1.94.1 and pinned repository dependencies.
Uploaded MIDI, WAV, VRM and extracted model data are not repository fixtures.

| Identifier | SHA-256 |
| --- | --- |
| Owner-provided MIDI benchmark | `615127ef5d0bfe7c4e934222349243e8d7c9c8309fa9e1a56e0b19bb0e41f5cf` |
| v5, before coupled elbow coordination | `0f5da5492d0527929f35455c5bdb571bbfa7626a5a42e224000b97b62c02206b` |
| Isolated zero-coordination comparison build | `47117b3207606b1b7748d3eac795f070f73a09a49fcd35275fbab8c5dc5d3625` |
| v6, with coupled elbow coordination | `7ae255f6f71a0d207efb6d4606b2cc48e03db21a1d54267c52bfb9b5f930ee45` |
| v7, with distance-aware preparation/recovery | `072bd4cae0ec5ecf20439a41b7664c4d5babb5db178a377eebb83e060a0e1fa9` |

Interpretation used Logic DKD GM mapping, explicit note-75 count-in, short-open-close
hi-hat controls at 20% requested openness, and the owner's approved omission of
source event `23:237` from the visual arrangement. The original event remains in
the source data. This is an explicit arrangement setting, not a filename rule.

Six profiles were checked: the native rig; the real Twist, Seed-san and
AvatarSample B node hierarchies; and controlled 0.8×/1.2× arm proportions derived
from the Twist hierarchy. Imported profiles used the production automatic seat
calibration, palm-up setting and explicit kit fit before performance planning.
The numerical harness reconstructs real raw/normalized bones and node constraints
and uses production adapters. **It does not render or inspect skinned pixels.**

## Final v7 numerical results

The whole 193.025623-second song was evaluated at 60 Hz, with every assigned impact
also checked through the actual adapter's contact markers. Reverse-order snapshots
checked primary poses and marker positions. Checks around impacts, dense samples
near the fastest rotations, and actual preparation/recovery boundaries used
`time ± 1 μs`; the boundary calculation includes fitted geometry, ready-position
fallbacks and overlapping-window/shared-transfer decisions.

| Observation | Result across all six profiles |
| --- | --- |
| Source accounting | 3,520 retained channel messages, including 1,760 positive note-ons: 1,720 strikes, 36 close controls, 3 count-in cues and 1 explicit visual omission. Source event arrays and original strike times unchanged. |
| Mapping and assignment | Zero unmapped or unassigned strikes; all six fit proposals complete. This fit result covers the contract's static/sampled posture and nominal hardware checks. |
| Actual adapter contact | Maximum marker error `6.289e-7 m`; maximum bone-length error `8.027e-8 m`. No bone stretching was introduced. |
| Seek determinism | Reverse-order primary-pose and actual-marker differences both zero. |
| Preparation/recovery boundaries | 2,078 boundaries across the six runs. Maximum wrist change over the 2 μs interval `5.704e-6 m`; maximum hand-quaternion change `0.001436°`. |
| All sampled continuity probes | Maximum wrist change `1.512e-5 m`; maximum hand-quaternion change `0.002310°`. These finite-difference results do not impose a global movement-speed limit. |
| Local cost | Fitted planning about 443–474 ms; fitting separately about 84–589 ms. This is not an end-to-end UI or rendering-FPS benchmark. |

## What coordination improved, and what remains

The isolated zero-coordination build reproduced all six v5 minimum clearances and
their timestamps exactly. It provided the missing whole-song counts without
changing the active app. The following v5 → v6 counts isolate coordination from
the later recovery change. A negative frame means overlap of a **skeletal proxy**,
not a confirmed skin-mesh intersection; counts are samples, not distinct incidents.

| Profile | Torso-negative frames | Arm-pair-negative frames | Stick-pair-negative frames |
| --- | --- | --- | --- |
| Native | 448 → 36 | 17 → 12 | 157 → 150 |
| Twist | 622 → 47 | 730 → 12 | 4,447 → 156 |
| Seed-san | 651 → 101 | 8 → 9 | 1,127 → 158 |
| AvatarSample B | 251 → 16 | 1,574 → 23 | 190 → 126 |
| Controlled short arms | 204 → 33 | 12 → 0 | 166 → 147 |
| Controlled long arms | 594 → 32 | 18 → 10 | 143 → 134 |

The time-integrated penetration deficit decreased for every profile/proxy pair,
including Seed-san's arm pair despite its 8 → 9 frame count. Planning increased
from about 97–101 ms to 438–475 ms. Minimum depth alone would miss this improvement:
a small number of deep, fast-transfer conflicts remains.

v7 reduced rapid return-to-ready movement. AvatarSample B's maximum right-wrist
step at 60 Hz fell from 0.1315 m to 0.0783 m, and its maximum right-hand rotation
from 33.87° to 19.02° per frame. Controlled long arms fell from 0.0917 m to 0.0663 m
and 22.18° to 12.28°. This was a tradeoff: AvatarSample B's arm-negative frames
increased from 23 to 41, with integrated arm deficit increasing from 0.00481 to
0.00741 m·s, still below the v5 value of 0.09213 m·s. v7 is not a universal
collision-regression pass.

| Worst v7 sampled proxy overlap | Depth | Song time | Observed category |
| --- | --- | --- | --- |
| Torso, Seed-san | 5.43 mm | 69.617 s | Intermediate hand/arm posture |
| Arm pair, AvatarSample B | 39.30 mm | 181.083 s | Left-hand floor-tom repetition while the right hand prepares a cross-kit crash |
| Stick pair, AvatarSample B | 11.99 mm | 164.683 s | Remaining inter-stick path conflict |

A separate 0.15-second coordination look-ahead experiment was not adopted: it
reduced some torso deficits but introduced arm overlap in a previously clear
controlled profile and worsened another profile's integrated arm deficit.

## Browser observations, separate from the numerical harness

These are bounded observations from the parent implementation/review session.
They do not turn the six-profile hierarchy audit into a six-model visual pass.

- In the actual app's earlier v4 preview, the approved arrangement showed 1,760
  positive source notes, 1,720 strikes, 3 cues, 36 controls, zero unmapped and zero
  unassigned strikes. This browser observation predates the final v7 runtime.
- An independently authored 3-second WAV with a +0.125-second offset was exercised
  through pause and seek. Project download and same-session reopening retained the
  tested state. This does not cover every codec or long recording.
- A private browser harness using the actual v7 `DrumScene` imported the real Twist,
  Seed-san and AvatarSample B VRM files at song time 25 s. AvatarSample B was also
  played, paused and sought back to 25 s. Contact measurements were finite and
  screenshots were inspected. This was a three-model visual spot check, not a
  full-song video review or broad skin-clearance test.

- In the final v7 app, AvatarSample B had 82 unassigned strikes after seat-only
  calibration and zero after explicitly applying the 90% fit proposal. The source
  inspector omitted the approved final snare without removing its original event.
- A v7 project containing that fitted VRM, the corrected MIDI, a 3-second authored
  WAV at +0.125 s and a 25.5 s output playhead was downloaded. After a full reload,
  the app requested all three original files. Relinking them restored the VRM,
  layout, offset, playhead, 1,720 strikes, one omission and zero unassigned strikes.
  This is a real fresh-session relinking check for this setup, not exhaustive
  model/format or long-recording synchronization acceptance.

## Repository validation

Rust formatting, 79 Rust tests and strict Clippy passed. All 138 web tests across
20 files, TypeScript checking, the production web build and document checks passed
(80 Markdown files, 690 relative links, 21 bilingual pairs). Regenerated license
notices were unchanged; no new dependencies or bundled assets were introduced.

## Acceptance boundaries and next review

The candidate is ready for owner review of the improved browser workflow. It has
not passed a universal natural-performance or collision-free acceptance gate.

- Body fit and contact checks do not certify clothing, hair, hands or skin-mesh
  clearance. Fast coupled transfers and inter-stick paths need further work.
- Distinct contact regions, hand-specific strike spread, shaft/rim dual contact and
  accurate rimshot/sidestick geometry are not implemented or passed by these tests.
- Velocity-aware procedural motion and phrase assignment do not establish a full
  push-pull, Moeller, double-stroke or learned-style reconstruction.
- No learned model, motion dataset or weights were adopted by this verification.
  The [source-adoption gates](../research/motion-adoption-gates.md) remain applicable.
- No new full-model video/output validation, release or Pages deployment is claimed.

Private evidence remains in `.local/natural-browser-audit-v5-counts.json`,
`.local/natural-browser-audit-v6.json`, `.local/natural-browser-audit-v7.json` and
`.local/natural-browser-audit-v7-summary.json`; the private input and scripts are
not public regression fixtures. The PR records the final repository checks.
