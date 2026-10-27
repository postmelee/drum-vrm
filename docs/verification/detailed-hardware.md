# Detailed drum hardware verification

This record covers [PR #59](https://github.com/postmelee/drum-vrm/pull/59),
the tom-holder/boom-stand increment following PR #57.
The [equipment contract](../../specs/architecture/equipment-contact.md) owns current
rules and [manufacturer research](../research/drum-hardware.md) preserves sources.
The owner requested implementation, multi-view inspection and Pages publication
of this batch; no release/tag or randomized strike variation is included.

## Changes and review findings

- One bass-mounted socket/post/yoke feeds side-clamped L-rods. The rod has separate
  horizontal and insertion sections, with bounded reach rather than an arbitrary
  diagonal connector to the bottom head. Suspension brackets touch the shell.
- Cymbal stands have nested tubes, collars, memory locks, boom joints and tails,
  separate end tilters, sleeves/felts, braced legs and rubber feet. Native spindle
  axes use the cymbal's radial frame, not the different bow-contact normal.
- The same numerical geometry owns rendering, placement checks and motion
  obstacles. Decorative vertices remain inside their capsule envelope. Fixed
  parts are merged by material; the beater retains its moving pivot.
- Independent review found and corrected a disconnected offset snare mast,
  floating tom brackets, a shortened visible beater sphere, and a tilted cymbal
  spindle using the wrong normal. These are covered by direct geometry checks.
- Static checks now include unrelated support crossings and the cymbal-centre
  projection inside the tripod polygon. This is not a structural load simulation.

## Regression boundary

The previous exact candidate was `64240552eecb608255efef569b6793a828698da5`.
Its recorded WASM was
`9b2b04ab45456257a83f48a77a7a8f19e11623a488052bb23fcd4e4f6d484584`.
Measured fit/study preparation on this machine was B 1.48/2.46 s, Seed-san
3.34/2.67 s and Twist 2.25/2.83 s. These are observations, not performance budgets.

The first new topology passed the raw default seed but failed the actual measured
Studio seed and all three fresh VRM fits. Inspecting only the front view or raw
JSON would have missed this regression. Tests now use the same tilted, measured
bodies as the app. `scripts/public-posture-review.mjs` also fails on incomplete
fit, unresolved equipment or unassigned posture-study hits, rather than only
checking deterministic evaluation and source preservation.

## Acceptance evidence

The local final WASM SHA256 is
`46d04abd65ecf631735770546ced56633d858592b2ef2798273fb75488ddb4ab`, planner
`phrase-beam-8-beat-horizon-24-posture-v10`. Rust 151 tests, web 249 tests,
Clippy with warnings denied, TypeScript and production web build pass. Generated
notices retain the existing dependencies/assets; no manufacturer asset is bundled.

The first PR CI run timed out in three browser-demo tests (60 s each). Those
fixtures repeatedly fitted absent geometry instead of the measured Studio bodies
used by the app. They now measure the actual models and reuse a cloned fit keyed
by the complete profile and kit. Contact/source/assignment assertions remain, the
positive hand-override case uses a reachable solo-roll event, and the 60 s limit
is unchanged. The six demo tests pass locally in 20.65 s after this correction;
the native absent-geometry fallback retains its separate Rust regression coverage.

The fresh measured Studio kit has no unresolved hardware. Independent geometry
checks on all three fitted public samples found no unrelated support crossings
or support/body/pedal clearance violations. All 15 tom suspension brackets reach
their actual shell meshes; all nine crash/ride spindles match the radial frame's
mounting origin and axis. Rubber feet are within 0.36 mm of the ground. Minimum
cymbal projection margin inside the tripod is 29.8 mm; the longest tom horizontal
arm is 369 mm within its 380 mm limit. Six study bell contacts remain in the
declared bell region, outside the felt including conservative stick-tip thickness.

Local Node/WASM observations on the shared development machine:

| Public sample | Fit scale | Fit / study preparation | Unassigned study hits | Replay / shuffled seek |
| --- | --- | --- | --- | --- |
| AvatarSample_B | 68% | 6.17 / 1.44 s | 0 of 53 | Equal |
| Seed-san | 78% | 13.67 / 1.70 s | 0 of 53 | Equal |
| Twist | 66% | 7.49 / 1.18 s | 0 of 53 | Equal |

More restrictive hardware routing increases initial fitting time relative to the
baseline. Study preparation improves, but total first-use latency is **not** an
overall speedup. Workers remain cancellable. These are not mobile benchmarks.

The browser review uses the built-in posture study, automatic public-model
selection, front/top/orbit side views and close-ups of the shared holder and boom
joints. Local screenshots and numeric reports are retained under
`.local/detailed-hardware/` and `.local/hardware-layout/`; they are not newly
redistributed character assets. [PR #59](https://github.com/postmelee/drum-vrm/pull/59)
records the exact reviewed commit and
subsequent Pages workflow, integrated commit and served-artifact verification.

## Full-song boundary

A separate regression run on the measured profiles/layouts uses intermediate
WASM `435b57f813d5cb12bcbb6c33521b417d65101bdc9eb6c4ff04481c8f8b3f880f`.
The final native-fallback fix does not alter those explicit measured descriptors.
It retains 1,760 source notes, 3,520 source events and 1,720 arranged hits, including
only the approved last-snare omission `23:237`. Each model passes finite-pose and
playback/seek equality over 9,985 timestamps, preserving timing and velocity.

| Sample | Unassigned before → after | Preparation before → after | Maximum sampled equipment-proxy penetration before → after |
| --- | --- | --- | --- |
| AvatarSample_B | 1 → 1 | 46.5 → 28.4 s | 16.0 → 27.4 mm |
| Seed-san | 1 → 1 | 49.9 → 31.5 s | 48.2 → 41.4 mm |
| Twist | 1 → 1 | 54.4 → 23.8 s | 40.8 → 63.1 mm |

The high/floor-tom chord at 10.166442 s still loses one assigned hand: high tom
`2:10` for B/Twist, floor tom `2:9` for Seed-san. This is a feasible two-hand
musical combination and an unresolved planner/layout limitation, not bad MIDI.
Dynamic proxy overlaps are not uniformly improved; the obstacle set and fitted
positions both changed. Twist also retains four impact-pitch diagnostics. The
strict full-song checker exits 1 because it requires zero unassigned hits. This
is an explicit failed acceptance boundary, not a claimed complete-song pass.

## Limits retained

Capsules and body boxes are conservative approximations, not exact arbitrary-model
triangle collision. Same-assembly construction, joint attachment, load stability,
full moving cymbal sweeps and rendered clothing require separate review. Clear
static hardware and a fully assigned independent study do not certify all fast
fills or simultaneous chords in the public full song. #18/#20/#23 stay open for
those broader motion-quality requirements.
