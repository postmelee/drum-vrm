# Common fitting and motion-path review

This is the numeric and selected browser review of the common-solver worktree for
[#18](https://github.com/postmelee/drum-vrm/issues/18) and
[#23](https://github.com/postmelee/drum-vrm/issues/23), after the owner's request
to fix automatic fitting before another Pages review. It does not replace the
[`f2d3b87` baseline](public-vrm-fit.md), which measured a different implementation.

For the original checkpoint below, **all four static fits and all study hit assignments succeeded; none of the four
models passed complete temporal acceptance.** Whole-stick/equipment or
whole-stick/stick intersections remain. Marker contact and deterministic seeking
do not make those failures acceptable. The textured browser review covered 45
paused views and still found unnatural tom-transfer arm elevation; continuous
playback was not verified at that checkpoint. Its measurements alone establish
neither merge nor public deployment; later publication and v13 records are separate.

## Measured identity

- Run: `2026-10-28T17:30:41.394Z`, Node `v24.15.0`, `darwin/arm64`.
- Hardware preset: `floor-legs`; planner `phrase-beam-8-beat-horizon-24-posture-v11`.
- Git HEAD: `f2d3b8726337bd4e8bd56a812098e25037b442e0`, **with uncommitted source changes**.
  HEAD alone does not identify this measurement or a publication candidate.
- Private evidence: `.local/reference-layout/common-motion-final-review/`, containing
  `environment.json`, four input/review pairs and `summary.json`. These numeric
  records remain local; original model files were not modified.

SHA-256 identifiers captured by the harness:

| Artifact | SHA-256 |
| --- | --- |
| Source manifest aggregate | `0ac56bf52f35f77363e44020de5b0c96e84dc390ab898f8882b502a498f7a292` |
| WASM binary | `949ba98ec611b9620b49bf9c5620aa3bbd58f4abeb29e06f338222fe6e67f902` |
| Review script | `cb1e706fa78e8f19f70fb140c388817caf679b07c34157ff1d6eb1621d971ad1` |
| Landmark helper | `8ec9040c1345d00e2006601be9365925b9814581223b6c6bb8cc11b044ae251c` |
| Motion-metrics helper | `0e0aa0a442ef78ca1d51578ef315bcafcee684928279a1b65b6f651df85d20ff` |
| Posture Study MIDI | `5cbf441f1942b7346f508cc33c97a25599dfe3bac91566d04cf859202a234a8c` |
| Interpretation Study MIDI | `24384e9626431aaad7924f27e90870965b23883bbd7b281afd10ef0ba7641afe` |

The source aggregate hashes the sorted path/hash manifest for `crates/drum-engine`,
`src` and `shared`. Input hashes below are the harness's compact JSON hashes,
not hashes of the formatted JSON files. Each input includes measured profile,
seat, complete kit, model-file hash and provenance reference.

| Input | Input SHA-256 | Model provenance |
| --- | --- | --- |
| Native skeleton | `66b861302f9f28b7f13a68482743bcb79d4121e32723627ba284558fe54a65b0` | Procedural, no model file |
| AvatarSample B | `92bc24752329489a5af084d81b4c256a582b240a9fa5b356209b332558165c5d` | [Approved asset](../../public/characters/avatar-sample-b-provenance.json) |
| Seed-san | `9a7bbd384e02d14e1dee0332491e7d1a87643ba535813b6ce31eaeb8875dfa70` | [Approved asset](../../public/characters/seed-san-provenance.json) |
| Twist | `e6537ffaf94acad7c5461ef03c3b1e8f11259cd9d597f19b26154fa911158744` | [Approved asset](../../public/characters/twist-provenance.json) |

The two independently authored studies use track 1, channel 0, `logicDkdGm` and
explicit count-in note 75. Interpretation additionally uses `shortOpenClose` with
openness `0.2`. This run does not certify the user's full song, other MIDI files,
VRM 0.0 imports or FBX imports.

## Static fitting and repeated application

The implementation follows **reviewed fixed layout → common posture/seating and
motion → bounded kit repair**. It does not move the reference kit on skeleton
startup, use model-name branches, stretch bones or scale the character. The table's
scale is the bounded kit proposal, not a uniform avatar scale. Exact rules belong
to [calibration](../../specs/architecture/calibration.md), not this measured report.

| Model | Fit complete | Kit scale | Seat height / forward (m) | Initial / repeated fit (s) | Fit reapplied to accepted setup (s) |
| --- | --- | ---: | --- | --- | ---: |
| Native skeleton | Yes | 1.00 | 0.79 / 0.14 | 0.225 / 0.214 | 0.214 |
| AvatarSample B | Yes | 0.90 | 0.73 / 0.14 | 14.436 / 14.805 | 0.169 |
| Seed-san | Yes | 1.00 | 0.73 / 0.14 | 0.950 / 0.947 | 0.191 |
| Twist | Yes | 0.92 | 0.73 / 0.14 | 23.191 / 23.126 | 0.340 |

All four runs preserved input data and bone lengths, restored candidate
measurements, matched measured and applied setups, and repeated the same fit.
Reapplying each accepted setup changed no numeric layout value. No required layout
violation or unresolved equipment item remained. Measured lower-body overlap
remained advisory for B (left leg/hi-hat, both legs/snare, right leg/floor tom) and
Twist (right leg/snare); these are not certified skin-clearance passes.

**Timing is observational, not a benchmark:** Rust tests ran concurrently and
affected available CPU. The fit calls are harness measurements, not page startup
times. Preparation below likewise must not be used as a controlled performance
comparison with earlier candidates.

## Motion, contact and repeatability

Each model assigned **53/53 Posture Study hits** and **82/82 Interpretation Study
hits**, with zero unassigned hits. Every study retained all three count-in cues
and six planned hand contacts at their original times. All 59 posture and 88
interpretation non-beater contact records had actual adapter markers; none was
missing. The greatest marker/contact error across the run was `5.022e-7 m`, below
the harness's `1e-5 m` marker threshold. This is marker agreement, not proof of
physical surface contact along the complete stick.

All studies preserved source events and the fitted layout through preparation
and seeking. Repeated plans matched; ordered/reversed pose and marker differences
were zero, with no nonfinite values. The largest sampled bone-length error was
`1.032e-7 m`. The outer review evaluated 510 posture and 439 interpretation times
per model, including exact contacts and semantic landmarks.

Signed minima below are outer-review whole-tool measurements in millimetres;
negative values indicate proxy penetration. A positive outer minimum cannot erase
a failure found on the planner's different, refined sample grid.

| Model | Study | Preparation (s) | Tool/equipment min (mm) | Tool/tool min (mm) | Intersecting outer samples | First outer failure (s) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Native | Posture | 1.028 | -20.068 | 7.785 | 4 | 7.435 |
| Native | Interpretation | 0.959 | 0.168 | -0.318 | 2 | 3.666663 |
| B | Posture | 6.030 | 0.020 | -3.331 | 2 | 1.866667 |
| B | Interpretation | 2.464 | 0.020 | -1.544 | 1 | 3.393330 |
| Seed-san | Posture | 5.409 | -6.000 | 1.239 | 1 | 6.333333 |
| Seed-san | Interpretation | 1.773 | 0.089 | -1.535 | 1 | 7.999992 |
| Twist | Posture | 2.517 | -14.897 | 5.990 | 3 | 6.333333 |
| Twist | Interpretation | 1.545 | -6.101 | -6.018 | 2 | 3.273330 |

Residuals that must remain visible to the next investigation:

| Model / study | Recorded failure entry points |
| --- | --- |
| Native / Posture | Tom-transfer landmarks at 7.4375 and 7.92 s also show whole-stick/equipment penetration; the planner emitted no hard-tool diagnostic for this study, but outer review still fails. |
| Native / Interpretation | Tool-pair diagnostic from 3.666663 s; equipment diagnostic at 7.641665 s (about -0.8 mm), which the outer grid did not capture. |
| B / Posture | Tool-pair diagnostic from 1.866667 s; equipment diagnostic from 7.575 s (eight internal samples, minimum about -29.5 mm), despite the positive outer equipment minimum. |
| B / Interpretation | Outer tool-pair failure at 3.393330 s; `CONTACT_PITCH_LIMIT` at 7.666659 s preserves one unsupported impact-pitch witness. |
| Seed-san / Posture | Equipment diagnostic from 6.316667 s near tom-transfer entry; five internal samples, minimum -6.0 mm. |
| Seed-san / Interpretation | Tool-pair failure at ending, 7.999992 s; planner diagnostic is rounded to 8 s. |
| Twist / Posture | Equipment diagnostic from 6.320833 s; five internal samples, minimum about -14.9 mm. |
| Twist / Interpretation | Outer tool failures begin at 3.273330 s; absence of a planner hard-tool diagnostic does not pass the separate sampled-path gate. |

These are recorded entry points and landmarks, not exhaustive continuous failure
intervals. Advisory body/combined-equipment diagnostics are retained separately;
they neither excuse physical-tool intersections nor turn a valid tool sample into
a failure. No source hit was omitted to obtain coverage.

Sampled tip-speed peaks ranged from 6.334 to 14.125 m/s and tool angular-speed
peaks from 48.256 to 91.007 rad/s. These are central finite differences at existing
review times using ±1/240 s, not a dense 240 Hz scan or a speed certificate. The
planner's 12 m/s endpoint-assignment heuristic is not a bound on final solved tip
motion. Quantization and transitions warrant visual review rather than treating
these figures as measured human biomechanics.

## Reproduction and remaining acceptance

Rebuild WASM and use a fresh output directory; the script refuses overwrites:

```sh
npm run build:wasm
node scripts/reference-layout-review.mjs --mode review --models native,avatar-sample-b,seed-san,twist --hardware floor-legs --output .local/reference-layout/common-motion-review-next
```

The [review script](../../scripts/reference-layout-review.mjs) requires static
layout, full assignment, count-in/motion validity and sampled physical-tool paths
to pass separately. This candidate fails the latter stages for every model.
Bounded search exhaustion and retained diagnostics are not acceptance.

## Browser observations

At `http://127.0.0.1:5190/drum-vrm/?lang=ko`, the rendered review captured all
three textured public models at five Posture Study checkpoints: ready, groove
entry, the first tom transition, both crashes and ending. Each checkpoint was
selected from the actual prepared plan and viewed from front, top and left,
for **45 paused captures**. The private capture index and UI state are in
`.local/reference-layout/common-motion-final-browser/browser-records.json`;
`b-contact-sheet.png`, `seed-contact-sheet.png` and `twist-contact-sheet.png`
collect the 15 views per model. This was a separate rendered review, not work
performed by the numeric harness.

All three models displayed their textures and changed arm poses across the
checkpoints. Ready and groove-entry arms were lower than in the earlier raised-arm
baseline. Tom transitions still raised the hands/arms excessively, most visibly
on Seed-san, where the selected transition raised a hand above the head.
These observations do **not** pass naturalness acceptance or certify skin,
clothing, equipment clearance or all intervening poses.

Continuous playback was interrupted by the rebuild's hot reload and remains
unverified. Right-side views, sustained motion between checkpoints and the full
numeric sample grid were not visually certified by these captures. The next
browser check must cover uninterrupted playback and the remaining tool-path
failures after the diagnostic follow-up; preserve this capture set as its own
observation. No publication URL or accepted candidate commit is claimed here.

## Diagnostic follow-up measurement

A rebuilt run at `2026-10-28T18:11:01.802Z` records the final diagnostic correction
in `.local/reference-layout/common-motion-diagnostics-review/`. Git HEAD is still
`f2d3b8726337bd4e8bd56a812098e25037b442e0` with uncommitted changes; the new identities are:

| Artifact | SHA-256 |
| --- | --- |
| Source manifest aggregate | `d5e3f8490b4479f0b5bd5dcc9c55013b8811023a303eb89bbc9b39835e831045` |
| WASM binary | `0af09cc24732aa64d2920326e861c6d18c61a71631dddd484a17c8e458ea4116` |

The Node/platform, hardware, review/helper hashes and all four input hashes are
unchanged. Direct comparison with the earlier run found identical `fit` and
`fittedProfile` objects for all four models, and identical `sequence` and
`kinematics` results in all eight studies. The fit, all-hit assignment and sampled
motion therefore did not improve or regress in this follow-up: it corrects what
is reported. The earlier WASM's measured tables remain above without replacement.

Final inspection now includes the route's interior checks, count-in controls,
hand controls and assigned hand-impact times, and tests posture on the final
evaluated pose. Previously missed whole-stick intersections and posture failures
are retained as blocking diagnostics. This closes the observed diagnostic gap;
it does **not** repair those motion paths or certify continuous clearance.

The table gives internal diagnostic counts, reported worst penetration and first
reported time. It uses a different grid from the earlier outer-review table.
A dash means no diagnostic of that kind, not a continuous-clearance proof.

| Model / study | Equipment: count / minimum mm / first s | Other stick: count / minimum mm / first s | Posture-failing times |
| --- | --- | --- | ---: |
| Native / Posture | 2 / -17.6 / 6.339583 | — | 114 |
| Native / Interpretation | 1 / -0.8 / 7.641665 | 7 / -5.0 / 3.658331 | 75 |
| B / Posture | 32 / -31.8 / 7.575000 | 2 / -9.8 / 1.858333 | 180 |
| B / Interpretation | — | 3 / -10.8 / 1.341667 | 36 |
| Seed-san / Posture | 19 / -28.2 / 6.316667 | 2 / -11.1 / 1.816667 | 312 |
| Seed-san / Interpretation | — | 5 / -8.6 / 1.741667 | 78 |
| Twist / Posture | 20 / -21.6 / 6.317708 | — | 129 |
| Twist / Interpretation | 1 / -6.4 / 7.641665 | 3 / -8.5 / 3.116667 | 18 |

All eight studies report `POSTURE_LIMIT`; counts are inspection times with at
least one failing hand, not unassigned hit counts. B/Interpretation also retains
`CONTACT_PITCH_LIMIT` at 7.666659 s. Count-in completeness still passes, and every
study still assigns all hits. **Every model continues to fail temporal acceptance.**

Executed regression checks passed: 217 Rust tests, 313 web tests across 43 files,
and 9 review-helper tests. Passing these checks confirms the tested behavior and
diagnostic coverage, not the physical or visual acceptance that remains unmet.
The next work is common trajectory/posture repair within the fixed-reference →
motion/seating → bounded-kit order, followed by another numeric and uninterrupted
browser review. The 45 captures above remain evidence from before this diagnostic
follow-up; they are not relabeled as a new playback verification.

### Browser check of the diagnostic build

With the rebuilt `0af09…` WASM at the same local URL, AvatarSample B loaded,
completed automatic fitting at 90% kit scale and prepared all 53 Posture Study
hits. Playback started at zero and reached the displayed end without hot reload
or another interruption. Analysis displayed zero unassigned hits and the three
retained motion diagnostics (tool/equipment, tool/pair and posture), alongside
separate advisory body warnings. This is a playback smoke check, not visual
approval of every intervening frame or resolution of the failures above.

Switching back to the native skeleton restored 100% kit scale and the authored
79 cm / 14 cm seat. The private capture directory above retains
`final-b-playback-ended.png`, `final-b-diagnostics-state.txt` and
`final-native-reset-state.txt`. Continuous playback of Seed-san and Twist on
this rebuilt diagnostic artifact has not been verified. No merge or deployment
was performed.

## Approved publication integration

On 2026-10-29 the owner explicitly requested merging and publishing this
known-incomplete checkpoint; see [approval](../approval.md). The candidate
`b837104f21385376adb76c3346353067b75d0eef` was integrated with main `990c01f`,
preserving its shared notifications and concise inspectors, and the stacked
PR #58 FBX/VRM 0.0 import work. This integration changes UI feedback and records;
it does not change the Rust solver or repair the temporal failures above.

Integration checks passed: 323 web tests (44 files), 9 review-helper tests,
typecheck/production build, documentation validation (102 files, 956 links,
23 language pairs) and whitespace checks. A production preview on local port
5194 started directly in the reference skeleton layout. AvatarSample B loaded
and fitted automatically, prepared the 53-hit Posture Study with zero unassigned
hits, and played through to the end. English/Korean switching, FBX/folder import
controls, notification history and the three retained motion diagnostics were
checked; the browser reported no console errors. Import controls were checked
for availability, not a new FBX compatibility certification. Private evidence is
in `.local/pages-common-fit-20261029/`.

The [PR #64 publication record](https://github.com/postmelee/drum-vrm/pull/64)
owns the final integrated/merged commit, CI and Pages run, deployed artifact
identity and public smoke checks. This local verification does not itself
establish deployment, resolve #18/#23, or authorize a Release/tag.

## Rack tilt and bounded grip candidate — 2026-10-29

This subsequent candidate is on `codex/kit-grip-refinement`, based on published
`6dea42f043f5d7c1d1bf9c880a6556a3356183d6`. It is **not published and does not pass
temporal motion acceptance**. The owner requested player-facing rack toms and
inward/thumb-up to palm-down grip, coordinated with the elbow. Related physical
layout adjustments follow the accepted bounded baseline work, not another
avatar-specific fitting system.

### Implementation and bounded scope

- Rack heads tilt 12° toward the player, previously 8°. Their heights remain near
  the published heights; lowering both by 6 cm was rejected after reach checks.
  The left crash is only 1 cm closer; the right crash is 17 cm lower. Nominal
  radii are unchanged. The floor tom moves toward the kick with a saved 1.2 rad
  leg/bracket phase, and the ride moves 5 cm right. The accepted native fit is
  adopted as the fixed default, including its linked hi-hat/pedal assembly.
- The actual common post/T-yoke, two ball joints and L-arms remain supported and
  measured. Removing a blanket 2D head-disc-overlap rejection allows tilted rim
  overhang; 3D access corridors, bodies, supports and whole tools still gate it.
- Natural grip has a shaft-heading reference and a side-signed inward-to-down
  roll interval. Four analytic refinements and the exact elbow circle preserve
  measured bone lengths and tool attachment. Explicit preferences and old
  profiles retain their documented behavior; there are no avatar-name branches.
- Count-in reuses the existing refined swivel candidates if a hand has no valid
  coarse candidate. Unsupported impact schedules rank reach and physical tool
  deficit before anatomical deficit; identical contact queries share a cache.
  The elbow graph distinguishes missed contact from lesser posture failures.
  These are diagnosed fallbacks, not relaxed acceptance limits.
- A floating-point envelope update could overwrite locked zero impact lift by
  about `1e-16` m and fail strict Worker restoration. The lock now preserves the
  exact value. The review harness checks serialized restoration and primary poses.

The existing sticking vocabulary is retained. In the reviewed public MIDI, the
four high-tom notes at 20.438182–20.682307 s previously excluded the opposite hand's
exact contact. With the accepted native geometry/profile their assignment changes
`LLLL` → `LRLL`, while six simultaneous snare/floor pairs at 43.212412–44.050402 s
retain L/R roles. All 1,720 source hits/times/velocities are preserved in this
assignment-only check; this is not full-song temporal acceptance. Independently
authored regressions cover both cases. Private uploads are not committed.

### First checkpoint measurements

These results precede the prepared-roll implementation below.
Private output: `.local/reference-layout/kit-grip-corrected-review/`, produced by
`node scripts/reference-layout-review.mjs --mode review --output <new-directory>`.
Node 24.15.0, Darwin arm64; source manifest SHA-256
`b253936d9176c378ca2dc8789acdb92587b5f0db830cb4d319047a2ec23813b8`, WASM
`ee5b7e873103860a5c07501ce0832350616951f43a596a6aa502b7820eb152cb`, harness
`b6f52c9266e6cf2c5e4d1c76ded3e331fa64d85c55c0d2cd3f8c20a26b07db28`.
The subsequent source-manifest delta is removal of a trailing blank line in
`hand_solver_tests.rs`; runtime sources and WASM are unchanged. The harness
verifies public model hashes against their existing provenance records.

| Rig | Kit scale | Fit / repeatable adopted layout | Posture / Interpretation assignments | Maximum actual contact error, mm | Worker restoration |
| --- | --- | --- | --- | --- | --- |
| Native | 100% | pass / pass | 53/53 · 82/82 | 0.000099 | pass |
| AvatarSample B | 84% | pass / pass | 53/53 · 82/82 | 0.000165 | pass |
| Seed-san | 88% | pass / pass | 53/53 · 82/82 | 0.000496 | pass |
| Twist | 76% | pass / pass | 53/53 · 82/82 | 0.000093 | pass |

All studies retain their count-in cues, source events, deterministic preparation,
ordered/reversed/shuffled primary poses and bone lengths. Actual rendered palm
normal Y never exceeds `5.74e-7` (numeric noise) over the sampled studies; missing
or invalid normals are zero. This supports the directional grip correction,
not a universal natural-posture claim. Native's explicit fit check takes 156 ms;
imported fits take roughly 30–43 s here under concurrent checks. Startup skips
native fitting. These measurements do not establish a loading-speed improvement.

### Remaining failures and next boundary

The strict review exits 1: every rig still has temporal tool/posture failures.
No study's complete hit coverage, palm direction or marker precision overrides them.

| Rig / study | Equipment-path samples / deepest overlap mm | Tool-pair samples / deepest overlap mm | Posture-limit samples |
| --- | --- | --- | --- |
| Native / Posture | 31 / 34.8 | 12 / 10.5 | 1087 |
| Native / Interpretation | 3 / 1.1 | 153 / 11.1 | 2279 |
| B / Posture | 35 / 29.9 | 2 / 6.4 | 843 |
| B / Interpretation | 1 / 0.5 | 4 / 8.3 | 1054 |
| Seed / Posture | 65 / 21.4 | 8 / 9.0 | 813 |
| Seed / Interpretation | — | 29 / 11.4 | 1220 |
| Twist / Posture | 88 / 18.1 | 2 / 5.3 | 640 |
| Twist / Interpretation | 87 / 29.4 | — | 2001 |

Native/Interpretation, Seed/Posture and Twist/Posture each retain one
`CONTACT_PITCH_LIMIT`. The earlier trial's missing B/Seed cues and Twist's 14.3 mm
reach miss are repaired, but evaluated wrist/arm posture is **not improved overall
against the published checkpoint**. For example native/Posture maximum wrist
angles are 60.7°/54.3° and arm elevations 93.1°/107.4°, versus 45.5°/34.4° and
66.1°/84.0° in the earlier diagnostic baseline. Keeping palms in the chosen family
does not by itself resolve the coupled wrist/elbow solve. Do not merge
this as a finished natural-motion fix or close #18/#23. Next work must preserve
this evidence while solving preparation/recovery trajectories with the same grip,
time, reach and whole-tool constraints; no new broad fitting grid is justified.

Automated checks passed: 231 Rust tests, 326 web tests in 44 files, 10 review-helper
tests, Clippy with warnings denied, formatting, WASM rebuild, typecheck/production
build and documentation checks. Passing regression checks does not certify the
remaining physical paths. Browser evidence and the exact review PR own the
textured validation scope; broader private VRM 0.0/FBX full-song review is deferred.

The focused diagnosis reused the same fitted profiles and targets. At Seed's
Interpretation preparation time 0.919999 s, the prior solver returned 144.78° arm
elevation / 46.19° wrist swing. The same target, shaft pitch and elbow swivel have
an allowed roll witness with 60.73° / 11.35°. Changing the musical path is therefore
not necessary for that frame: refinement was evaluating forearm agreement against
the previous elbow before recomputing the final elbow. Narrow 240 Hz baseline
windows also expose actual solver branch jumps, not just high static angles.
These observations motivate final-pose roll scoring; they do not certify its
subsequent implementation or continuity. The checkpoint above remains unchanged.

The checkpoint's textured browser review includes B, Seed-san and Twist at ready,
groove, tom, crash and ending, with complete Posture Study playback for each and
additional Twist front/top captures. Files remain private under
`.local/kit-grip-corrected-visual/`. The 12.766 s end state is observed for all three;
warning/error console logs are empty. These captures show the revised palm family
but retain visible excessive arm elevation and do not override numeric failures.

### Rejected pointwise refinements

Final-pose roll scoring repaired the extreme angles but exposed discontinuities
between valid minima. A conditional scalar-search shortcut added larger jumps;
removing it fixed those exact pairs, but other 11–13 cm elbow jumps remained over
about 1 microsecond. Repeated seek equality alone did not detect this defect.
A fixed continuous-gradient alternative removed the sampled jumps, but lost
previously assigned closed-hi-hat contacts (Native 16, Seed-san 12) by choosing
slightly infeasible compromises. It is rejected, not an acceptable trade-off.
The preserved private comparisons are `kit-grip-continuity-after-refinement/`
and `kit-grip-smooth-continuity/` under `.local/reference-layout/`. The full review
of the intermediate optimizer and the gradient's native fit were stopped after
these blockers; neither stopped run is a passing acceptance result.


### Prepared-roll review candidate — final v13 boundary

The final implementation stores each hand's selected roll alongside the existing
elbow-swivel controls. Graph edge checks and replay interpolate the same saved
roll; replay does not independently optimize it on every frame. The existing
33×33 graph and source control times remain unchanged, with a soft roll-variation
cost and at most about 6 MiB of extra witness cache. A valid analytic static solve
returns after five wrist solves; otherwise the bounded search uses at most 21.
Fixed-roll replay uses one. This changes prepared format to 3 and planner to v13;
checksums, policy/range validation and Worker restoration cover the new controls.
It is not a new hard angular-speed guarantee or relaxed physical acceptance.

Final output: `.local/reference-layout/kit-grip-final-review-v13/`; same command,
Node/platform and harness hash as above. Source manifest SHA-256
`11b0a0fe7de09ce867b7f8cb2c953c6ad81b0ed90daf4c86db829696c2465260`, WASM
`22f94f2ca45d38071e55a17ba0a3c977ba5af133500e330851f4bd9df9dcff8e`.
Subsequent edits are verification/handoff documentation only. All four fits,
repeated/adopted layout checks, original events, 53/53 Posture and 82/82
Interpretation assignments, three count-in pairs per study, marker contacts,
bone lengths, ordered/reversed/shuffled poses and actual format-3 restoration pass.

| Rig | Kit scale | Maximum actual contact error, mm | Fit / Posture / Interpretation preparation, s |
| --- | --- | --- | --- |
| Native | 100% | 0.000099 | 0.8 / 5.3 / 2.2 |
| AvatarSample B | 76% | 0.000196 | 79.6 / 20.6 / 7.7 |
| Seed-san | 92% | 0.000378 | 106.5 / 8.1 / 4.5 |
| Twist | 76% | 0.000092 | 42.3 / 9.1 / 4.2 |

Timings include concurrent validation/build load, not a controlled performance
comparison. No loading-speed improvement is claimed. Sampled rendered palm Y is
at most `5.9e-7`, with no missing/invalid normals; visible palms stay in the
inward-to-down family. **All eight studies still fail temporal acceptance**:

| Rig / study | Equipment-path samples / overlap mm | Tool-pair samples / overlap mm | Posture-limit samples |
| --- | --- | --- | --- |
| Native / Posture | 29 / 14.7 | — | 304 |
| Native / Interpretation | 1 / 1.1 | 7 / 4.9 | 210 |
| B / Posture | 57 / 23.9 | 2 / 5.8 | 395 |
| B / Interpretation | 1 / 1.4 | 13 / 10.0 | 271 |
| Seed / Posture | 27 / 24.8 | 2 / 5.3 | 222 |
| Seed / Interpretation | — | 6 / 7.4 | 223 |
| Twist / Posture | 72 / 18.1 | — | 96 |
| Twist / Interpretation | 20 / 11.4 | 2 / 7.8 | 118 |

The strict run exits 1. Native/Interpretation and Twist/Posture each retain one
impact-pitch limit. Posture failures decrease against the first checkpoint, but
maximum wrist swing still reaches 58.12° and arm elevation 114.46°. Re-fitted
layouts also differ: B/Interpretation stick-pair overlap increases 8.3→10.0 mm,
Seed/Posture equipment overlap 21.4→24.8 mm, and Twist/Interpretation gains two
stick-pair failures. These are not isolated solver comparisons, but prevent a
claim of regression-free motion or completed #18/#23.

Focused regression output `.local/reference-layout/kit-grip-final-roll-continuity/`
reuses the earlier fixed profiles/layouts at four 240 Hz windows (243 samples).
The former microsecond branch-jump pairs now differ by only 3.1–24.1 µm at the
elbow, and reverse evaluation is exact. Seed's old cue overlap becomes +1.961 mm
clearance. That evidence is input-specific, not the newly fitted full-study
result above. Stored roll speed still reaches 68.39 rad/s; fast continuous
rotation and remaining interior posture violations need trajectory work.

Final automated checks: 237 Rust tests, 326 web tests/44 files, 10 review-helper
tests, formatting, warnings-denied Clippy, rebuilt WASM, typecheck/production
build and docs checks. Local browser evidence is recorded separately below.
The next task is bounded preparation/recovery path repair with these saved grip
controls, keeping exact impacts and physical-tool diagnostics. Broader private
VRM 0.0/FBX full-song acceptance remains deferred. This candidate is local/draft,
not a published Pages update or a complete natural-motion solution.


Final textured review at `http://127.0.0.1:5195/drum-vrm/?lang=ko` covers B,
Seed-san and Twist at ready/groove/tom/crash/ending. All three complete the
12.766 s Posture Study; Twist also has front/top and EN/KO views. Screenshots
remain private in `.local/kit-grip-final-v13-visual/`; warning/error console logs
are empty. These views confirm the inward/down palm family, but still show
raised arms at high instruments. They do not certify collision-free motion.
