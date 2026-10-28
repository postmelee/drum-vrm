# Reference layout and fitting boundary

This record preserves the findings from the local reference-layout work started
from `d8055d0`. It supplements the earlier
[detailed-hardware verification](detailed-hardware.md); it does not replace that
record or transfer its acceptance to a different fitter.

## Owner scope and integration boundary

The owner stopped further generalized fitting development in this increment.
The remaining local changes are the left knee facing the hi-hat pedal, the floor
tom closer to the kick, and the left crash closer to the performer. The preferred
arrangement must remain recognizable in front, side and top views. This is not an
approval to expand motion, resume a larger search or publish the current tree.

The separate unified-fitting task owns [PR #58](https://github.com/postmelee/drum-vrm/pull/58),
candidate `e5fafb9` atop `main` `6038e8e`. Its source line and this local tree must
be reconciled explicitly. The generalized layout/clearance candidate described
below remains unfinished. Its contracts do not establish three-model acceptance
or public deployment.

## Comparison points

| Candidate | Evidence boundary | Result |
| --- | --- | --- |
| Earlier PR #59 | Exact commits/WASM and checks in [detailed hardware](detailed-hardware.md) | Historical accepted hardware study; retains documented full-song and dynamic-path limits |
| Intermediate `e23c4925…` WASM | Native plus three public models, earlier sole markers, scaled cymbals, no new lower-limb gate | Layout/study checks passed then; not acceptance for the stricter candidate |
| New lower-limb/generalized fit | Nominal cymbal diameters, shell factor bounded at 0.76, measured leg/foot geometry | Incomplete; results below precede the final rest-frame correction |
| Narrow owner-requested candidate | Compact coordinates and pedal-aligned left knee; final WASM below; actual native top/front/left views | Static equipment passes; native complete fit still fails snare/leg checks; no current three-model pass claimed |

The intermediate `e23c4925…` evidence remains under
`.local/reference-layout/candidate-e23c-20261027/`. Its native/B/Seed/Twist factors
were 92/76/84/76%, respectively. Those values included a different size policy and
must not be presented as current shell-only fitting results.

## Findings retained for integration

- Independent instrument repairs can pass local checks yet destroy the selected
  kit arrangement. The candidate groups the rack pair and constrains its row
  relative to the snare, floor tom, hi-hat and cymbal sectors. These are constraints
  of a chosen conventional layout family, not universal rules for every drummer.
- Choosing a leg-reachable hi-hat pedal before evaluating the complete assembly
  can lock an unsuitable mast position. Bounded pedal/mast/hand candidates are
  preserved, but their search strategy and runtime are not accepted yet.
- A 300 mm snare tripod intersected the native right shin/foot at a hand-valid
  snare position across all tested phases. A common 225 mm snare footprint with
  a rearward leg clears that reference case. Other stand footprints remain
  unchanged; no leg-to-hardware collision exemption was introduced.
- Some rendered clothing creates offset lower-limb volume that a circular radius
  models poorly. Referenced, visible mesh vertices now feed segment-local boxes
  alongside a skeleton core. Large clothing dimensions were not discarded merely
  to make fitting succeed.
- Reprojecting body-right after bending introduced axial roll relative to the
  rendered bone's minimum-swing rotation. Optional measured `legRestFrames`
  transport the box in the same frame as the adapter. Measured shoe soles also
  replace a fixed sole depth where available. These corrections require fresh
  adapter inputs; older collision measurements cannot prove their final result.
- Physically clear snare witnesses for Seed/Twist still failed hand/preparation
  checks. Nonintersection alone is not proof of a reachable, natural performance.
  The owner-authorized knee change is limited to the left leg and hi-hat frame.

## Preserved experimental implementation

`fit.rs` contains the grouped layout, linked hi-hat candidates, exact support-phase
repair, mount dependency order and shared final certificate. Its global candidate
budget remains 80,000, including phase probes. `fit_layout.rs` owns relational
layout checks. `fit_lower_limb.rs` samples seated/pedal extrema against shells,
supports and pedal bodies using measured boxes plus core capsules. Intended own
shoe-to-board contact is distinguished from thigh/shin collision.

Related schemas and renderer changes must be integrated together: `pedalForward`,
`standRotation`, lower-limb geometry/rest frames, measured sole markers and optional
stand-clamped floor-tom hardware. Preserve the current implementations for the
separate fitting task rather than restarting or adding avatar-specific branches.

The private detailed handoff is
`.local/reference-layout/HANDOFF-fit-20261027.md`. No private model or input file is
newly authorized for redistribution by this report.

## Executed checks and incomplete acceptance

The last probes with the measured v5 inputs produced the following observations
on the development machine. These inputs include corrected sole markers but
**predate `legRestFrames` and the latest left-knee policy**. They describe an
unfinished iteration, not a benchmark or current compatibility guarantee.

| Probe | Complete | Candidates | Time | Boundary |
| --- | --- | --- | --- | --- |
| Native v7 | Yes | 3,983 | 7.92 s | Nominal shell factor; before later hi-hat branch/frame changes |
| AvatarSample_B v9 | No | 12,180 | 1.15 s | Direct hi-hat precheck rejected the candidate |
| Seed-san v10 | No | 78,313 | 10.11 s | No complete coupled placement found |
| Twist v10 | No | 80,000 | 8.57 s | Search budget exhausted without complete placement |

The focused command
`cargo test -p drum-engine --release fit::lower_limb -- --nocapture` passed six
tests after the rest-frame correction and trace cleanup. These cover geometry,
frame transport, metadata validation and sampled pedal positions; they do not
establish full-fitter acceptance. An intermediate whole Rust run still had seven
failures across default geometry, fitting, first-frame swivel and paired-close
continuity. Reproduce those on the final candidate before classifying or changing
the fixtures; no full-suite pass is claimed here.

The frozen integration WASM was
`9e72898445ce9b80e8ae49df50190f2211d766926753aaacf4428b236bc7457b`.
Its existence is not a passing review. The final narrow candidate is recorded below.

## Reproduction and next acceptance

`scripts/reference-layout-review.mjs --mode inputs --output <fresh-dir>` exports
fresh actual-adapter inputs. `--mode review --output <fresh-dir>` checks repeated
fitting, two independently authored studies, immutable layouts and deterministic
seek/marker results. Invoke the script with `node`; `--hardware suspended-floor`
selects the optional hardware variant. Keep previous evidence in place.

For the narrow increment, record the final source, actual rendered left knee and
compact instrument positions from multiple views. The separate fitting task must
regenerate inputs, resolve coupled posture/clearance failures and run the complete
suite before claiming native plus three-public-model acceptance. Layout, assigned
hits, actual contact, dynamic equipment clearance and skin/clothing clearance are
separate claims. See the [work entrypoint](../handoff.md) for current ownership.

## Final narrow baseline

The source snapshot containing this report is an integration checkpoint, not a
merge-ready complete fitter. Local review runs at port 5190.

| Piece | Engine position in metres | Change from the preceding local seed |
| --- | --- | --- |
| Floor tom | `[0.59, 0.80, 0.48]` | 30 mm inward and 60 mm toward the kick |
| Left crash | `[-0.55, 1.26, 0.50]` | 110 mm inward and 180 mm toward the player |
| Right crash | `[0.58, 1.30, 0.64]` | 20 mm higher to clear ride/tilter interference |
| Ride | `[0.70, 1.15, 0.39]` | Unchanged |

All nominal radii are retained. The more aggressive floor X=0.55 trial overlapped
the middle-tom shell; left crash `[-0.48,1.26,0.58]` blocked the high-tom approach.
The selected positions pass both the fallback hardware regression and measured
Studio static geometry: 9 bodies, 143 support segments, 4 pedal/mechanism boxes,
no unresolved equipment IDs. This does not certify hands, legs or dynamic paths.

The left knee follows horizontal `pedalForward`; right-leg policy and pedal
contacts stay fixed. Candidate calibration now restores the accepted pedal frame,
including after a failed/temporary measurement. Native top/front/left browser
views were inspected; local screenshots are `compact-top.png`, `compact-front.png`
and `compact-left.png` under `.local/reference-layout/`.

- Final kit JSON SHA-256: `41b1c2ba246b2d9d53e82aa60ccf5f6a700eff064dc35624dd7857e7e59e2782`.
- Final runtime WASM SHA-256: `8d5f0e25994d1869f2c4e2a802b0608f019180140fb6bc356a427b37c1d78c07`.
- Fresh native input: `.local/reference-layout/compact-approved-inputs/native-input.json`.
- Native fit: `compact-approved-fit/fit.json`, 3.179 s; incomplete with
  `left-leg-clearance:snare` and `right-leg-clearance:snare`, static equipment clear.
  The browser retains the reference and displays an incomplete-fit warning.
- `npm run build:wasm` and `npm run build:web` passed.
- Full `npm test`: 267 passed / 4 failed. The calibration-rollback failure was
  then fixed and its 17-test adoption/humanoid subset passed. Three demo tests
  remain blocked by `measuredNativeFit` requiring a complete fit. No assertion
  was relaxed to make the candidate pass.
- Full `cargo test -p drum-engine --release`: 163 passed / 5 failed. The relational
  layout fixture was then corrected to put the floor explicitly on the wrong
  side of the snare; its focused test passed. Four fitter-dependent failures remain.

Remaining Rust tests: `annulus_reach_is_not_accepted_as_a_safe_near_snare_posture`,
`bounded_common_fit_preserves_kick_and_fits_linked_hihat_idempotently`,
`measured_body_pair_capability_and_ready_pose_share_the_fit_contract`, and
`first_frame_searches_safe_swivel_without_a_fake_previous_zero`.
Full logs are `.local/reference-layout/final-rust-tests.log` and
`final-web-tests.log`. Their counts precede the two focused repairs described above.
The coordinated PR #58 work must reproduce these failures and unify search/seat
constraints before requesting merge. No merge, Pages deployment or release occurred.
