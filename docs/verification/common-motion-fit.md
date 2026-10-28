# Common fitting and motion-path review

This is the numeric and selected browser review of the common-solver worktree for
[#18](https://github.com/postmelee/drum-vrm/issues/18) and
[#23](https://github.com/postmelee/drum-vrm/issues/23), after the owner's request
to fix automatic fitting before another Pages review. It does not replace the
[`f2d3b87` baseline](public-vrm-fit.md), which measured a different implementation.

**All four static fits and all study hit assignments succeeded; none of the four
models passed complete temporal acceptance.** Whole-stick/equipment or
whole-stick/stick intersections remain. Marker contact and deterministic seeking
do not make those failures acceptable. The textured browser review covered 45
paused views and still found unnatural tom-transfer arm elevation; continuous
playback remains unverified.
This record establishes neither merge nor public deployment.

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
