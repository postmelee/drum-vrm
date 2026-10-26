# Natural posture and shared contact verification

This report belongs to the implementation of #18, #20 and #23 after the public
sample review. It records a candidate, not a new public deployment.

## Scope and method

The owner approved the following order: shared rendered/numeric equipment and
mechanical attachments; comfortable snare-first posture and layout; grip and
pedal mechanics; one contact record for motion, measurement and effects; then
three-sample visual and deterministic playback checks. Random strike variation
and expanded video output remain deferred.

The acceptance score is an independently authored 12-second posture study with
count-in, sustained snare/hat groove, a tom transfer, ride, simultaneous crashes
and recovery. AvatarSample_B, Seed-san and Twist must use the same numeric rules,
without avatar names, bone stretching, source retiming or smaller body proxies.

Review distinguishes assignment, requested endpoint, actual adapter marker,
posture angles, equipment/body proxy clearance and visible skin/clothing. A
passing marker comparison alone does not certify natural movement or surface
contact. Reference sources and implementation choices are documented in the
[technique research](../research/ergonomics-and-sticking.md) and
[equipment contract](../../specs/architecture/equipment-contact.md).

## Baseline

Baseline code: `0d012e8d384c9b30292ff8cd8d7171b98e4b1956`.
A local Node/WASM audit evaluated the existing eight-second interpretation study
with recorded measured profiles and layouts; these are not fresh-fit timings.
Source construction took 8.6 ms. Profile motion preparation took 1,351 ms for
AvatarSample_B, 1,312 ms for Seed-san and 1,784 ms for Twist. At 2 seconds,
left/right upper-arm elevation from downward vertical was 74°/92°, 89°/91° and
106°/87° respectively. This reproduces the reported high-elbow silhouette.

Raw local inputs and timing records remain in `.local/natural-contact/`. Final
candidate measurements must identify their own WASM hash and fresh-fit inputs;
this baseline is not retroactively replaced with candidate results.

## Candidate results

The candidate is stacked on the separate VRM 0.0 import PR (#55, base
`994ec128323e6f033cae1f5b4d73ea90d62c1e9d`). Its review branch is
`codex/natural-posture-review`. This report does not claim a new Pages version.

### Implemented boundaries

- Rust and Three.js use the same nominal kit dimensions and stick taper data.
  Rack toms share a bass-mounted holder; floor toms use three legs; snare uses a
  basket; cymbals use angled booms. The direct hi-hat mast is linked to its pedal.
- Static fit, source assignment and runtime planning share articulation-aware
  posture/contact feasibility. Snare supports both hands, hats/ride their groove
  hand, and toms/crashes at least one hand. Dual crashes require one consistent
  assignment at both contact and ready poses. Fit success is not a whole-path
  collision certificate.
- Canonical contact records identify source, time, limb, tool part, surface,
  point/normal and assignment. Effects and actual-marker inspection resolve the
  same records. Sole pressure does not create a drumhead flash; the derived kick
  beater contact does. Unassigned notes have no fabricated impact effect.
- The opposing hi-hat surfaces, pedal boards and beater evaluate absolute song
  time. Hi-hat opening is included in planned hand contact, rather than moving
  only the rendered cymbal. Imported static surfaces report their limitation.
- The oblique grip is the new default. Thumb fitting uses measured finger joints,
  pad position and shaft alignment, without stretching bones. Explicit saved
  palm directions remain unchanged.
- The study and semantic checkpoints expose ready, groove, tom, ride, dual-crash
  and ending views. Analysis separates actual markers, posture angles and proxy
  clearance; its report remains local and excludes source notes/assets.

### Isolated preparation-cache comparison

A same-input comparison on intermediate WASM
`91fa3e53e75374814778a22d680f08d111e393a6689d3dac400d6d71de26fccd`
isolated duplicate planning from fitting and later posture changes. The source,
profiles and layouts were identical in each before/after pair.

| Recorded profile | Before: plan + commit | After: plan + commit |
| --- | ---: | ---: |
| AvatarSample_B | 6,320 ms | 3,254 ms |
| Seed-san | 5,853 ms | 2,932 ms |
| Twist | 5,866 ms | 2,968 ms |

The former commit stage issued a second Worker preparation. Reusing its exact
validated candidate reduced that stage from about 3 seconds to 4–9 ms. Plan,
121 sampled poses and 25 inspection records retained identical hashes per model.
The cache includes the complete plan/profile/layout, captures caller-owned data
before asynchronous work, and invalidates canceled/replaced candidates. Tests
exercise cancellation, failed replacement and mutated inputs. These numbers are
an isolated cache experiment, not a final browser-load or whole-song benchmark.

### Independent challenge checks

A review found that exempting an upper-arm attachment solely by shoulder height
could hide a rear-to-front torso crossing. The attachment must also project
inside the torso's top ellipse in its moving frame. The counterexample changes
from a false +56.973 mm clearance to the correct −64.281 mm; mirrored depth and
rotated-torso cases are regression tests. Other arms, sticks and equipment do
not receive that attachment exemption.

Measured physical surface probes and sole/board checks are separate from skin
validation. Intermediate checks cover 135 rendered surface probes within 0.3 mm,
and 81 sole strike contacts plus 18 interpreted closures across three profiles
and three studies within 1.22e−8 m. These validate the implemented kinematic
contact markers, not shoe deformation or arbitrary imported foot orientations.

### Final numeric candidate identity and automated checks

The following results use WASM SHA-256
`78246ccc13104597fc27fbbebb1b78d1375208317962d1d1b4c32c4b1fd0ded5`
and planner `phrase-beam-8-beat-horizon-24-posture-v9`. The branch/base above
identifies the reviewed work before its PR commit is assigned. It does not replace
the historical Pages commit recorded in the [work entrypoint](../handoff.md).

| Check | Result |
| --- | --- |
| `cargo test -p drum-engine --release --lib --no-default-features` | 138 passed |
| `npm test` | 245 passed across 36 files |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy -p drum-engine --all-targets --no-default-features -- -D warnings` | Passed |
| `npm run typecheck` | Passed |
| `npm run build` | WASM and web production builds passed |
| `npm run docs:check` | Passed |

The cases include source/prepared-plan invariants, actual-pitch reach, shared
surface/contact frames, pedal hinges, equipment support geometry, shoulder
attachment counterexamples, deterministic evaluation and cache invalidation.
Passing these checks does not close the full-song motion-quality acceptance.

### Three public models: 12-second posture study

`node scripts/public-posture-review.mjs` verifies each public model against its
provenance hash, loads its humanoid/skin geometry without textures, performs fresh
automatic fitting, and recalibrates the adapter to the fitted kit before preparing
the score. No previous fitted layout is substituted for this run.

| Model | Fit / kit scale | Fit | Prepare | Unassigned | Maximum actual-marker error |
| --- | --- | ---: | ---: | ---: | ---: |
| AvatarSample_B | Complete / 70% | 1,484 ms | 4,155 ms | 0 / 53 | 1.71e−7 m |
| Seed-san | Complete / 78% | 3,235 ms | 5,141 ms | 0 / 53 | 3.89e−7 m |
| Twist | Complete / 70% | 2,309 ms | 4,891 ms | 0 / 53 | 1.40e−7 m |

Every model preserved the study's source events. The 475 primary-pose samples
(30 fps, exact impacts and neighboring times) had zero order-dependent differences
and no nonfinite values under a reproducible shuffled order. Reverse evaluation of
the six semantic checkpoints also reproduced the primary pose and actual adapter
contact markers exactly. Each model has 59 actual-marker comparisons: assigned
stick/sole contacts and paired count-in contacts; the derived beater is excluded
from this adapter-marker count.

Upper-arm elevation is measured from downward vertical: 0° hangs down and 90° is
horizontal. These are recorded poses, not clinical comfort limits.

| Model | Ready, L / R | Groove at 2 s, L / R | Ending, L / R |
| --- | ---: | ---: | ---: |
| AvatarSample_B | 47° / 56° | 39° / 54° | 39° / 56° |
| Seed-san | 29° / 34° | 34° / 49° | 22° / 35° |
| Twist | 44° / 41° | 44° / 60° | 43° / 40° |

This is a fresh 12-second study with new fitting; the eight-second historical
baseline above is a different input/layout and is not a controlled timing or
frame-by-frame posture comparison. Elevated cymbal actions remain: the selected
ride checkpoint reaches about 77° on B's right arm. The diagnostics remain visible:

| Model | Pitch-rate contact failures | Equipment-path samples / greatest proxy overlap | Impact poses outside posture limits |
| --- | ---: | ---: | ---: |
| AvatarSample_B | 3 | 4 / 6.0 mm | 1 |
| Seed-san | None reported | 40 / 21.8 mm | None reported |
| Twist | None reported | 25 / 13.2 mm | 1 |

These counts come from the planner's specific diagnostic passes. A separate raw
475-sample proxy summary has a different sampling set and includes intentional
count-in stick contact; its negative-clearance count is not an interchangeable
count of failed notes. Public model provenance and original license terms remain
in `public/characters/*-provenance.json`. Detailed private outputs are
`.local/public-posture-review/{avatar-sample-b,seed-san,twist}.json`.

### Public full-song regression

The private `.local/natural-contact/full-song-review.mjs` reuses those final
measured profiles and fitted kits; it does **not** rerun the avatar importer or
prove new rendered skin clearance. It applies all 24 exported tracks and the exact
[public demo selection](../../public/demos/transparent-girl-1-provenance.json),
including the approved omission of the extra final snare. MIDI SHA-256 is
`615127ef5d0bfe7c4e934222349243e8d7c9c8309fa9e1a56e0b19bb0e41f5cf`.

All three runs preserve **1,760 positive note-ons, 3,520 complete source messages
and 1,720 arranged hits**, including their original IDs, timestamps, notes and
velocities. The exported all-track timeline ends at **226.545223 seconds**;
the previously discussed final musical chord near 191.804 seconds is not used
to truncate that ending. Source-only planning took 50.6 ms in this run.

| Model | Prepare | Sampled review | Unassigned | Maximum engine endpoint error |
| --- | ---: | ---: | ---: | ---: |
| AvatarSample_B | 78.4 s | 4.1 s | 1 / 1,720 | 3.32e−8 m |
| Seed-san | 95.3 s | 15.2 s | 1 / 1,720 | 5.84e−8 m |
| Twist | 91.6 s | 13.4 s | 1 / 1,720 | 4.37e−8 m |

These timings were collected on a shared development machine during parallel
browser QA. They are not isolated benchmarks or import/download timings, but
the observed 78–95-second preparation is a remaining usability problem. The
earlier same-build cache experiment does not establish acceptable full-song latency.

Each model's **9,985** samples include the bounded 30 fps grid, every exact impact,
±60 ms impact neighborhoods, count-in boundaries and explicit checkpoints at
0, 25, 30, 92.55, 191.804, 226.545223 and 226.795223 seconds. Ordered, shuffled
and reversed checkpoint evaluations were identical with no nonfinite values.
The endpoint errors above compare the engine's returned tool/sole coordinates
with its planned targets; unlike the 12-second table, they are **not independent
measurements of the actual adapter's transformed marker**.

| Model | Unassigned original event | Score context |
| --- | --- | --- |
| AvatarSample_B | `2:10`, 10.166442 s, note 48, high tom | Simultaneous low/high toms, followed by dual crashes 326 ms later |
| Seed-san | `12:105`, 92.660170 s, note 49, left crash | Low tom + kick + crash at the end of a fast fill |
| Twist | `12:105`, 92.660170 s, note 49, left crash | Same fill-ending chord |

The fitted capability matrices report both hand assignments for the affected
tom/tom or crash/floor-tom ready combinations as `not-found`. This is evidence of
a bounded score-specific combination limitation, not proof that a human cannot
play the chord. A fit can support each piece and the required snare/groove/dual-crash
set while failing another chord in the score. No source hit is deleted or retimed
to hide it; an unassigned hit receives no fabricated contact effect.

| Model | Pitch-rate contact failures | Equipment samples / greatest proxy overlap | Body-overlap impacts / greatest overlap | Posture-limit impacts |
| --- | ---: | ---: | ---: | ---: |
| AvatarSample_B | 27 | 150 / 16.0 mm | 2 / 4.3 mm | 17 |
| Seed-san | 31 | 520 / 47.9 mm | 47 / 33.3 mm | 8 |
| Twist | Two reports: 35 and 31 | 491 / 40.8 mm | 72 / 36.9 mm | None reported |

The two Twist pitch reports are separate diagnostic passes and are not asserted
to identify 66 unique failures. Accurate endpoints and deterministic seeking
coexist with these unresolved posture/path problems. Full results, exact saved
input hashes, diagnostics and checkpoints remain under
`.local/natural-contact/full-song-review/`; its exit code is 1 because one hit
remains unassigned per model, not because source preservation or determinism failed.

### Final result-preserving optimization candidate

The final rebuild is WASM SHA-256
`9b2b04ab45456257a83f48a77a7a8f19e11623a488052bb23fcd4e4f6d484584`.
The `78246ccc…` results above remain the measurements of that earlier candidate.
Two numeric hot paths changed: surface projection is evaluated only for the
intended body, and equipment correction skips a lift/pitch candidate only when
its nonnegative cost lower bound cannot beat the current strict winner. Collision
rules, candidate order, source timing and posture constraints are unchanged.
An exhaustive-versus-pruned regression checks that the selected result is identical.

After rebuilding, both the fresh three-model study and the full-song review were
rerun. A JSON-exact comparison against the preserved `78246ccc…` outputs found
**no behavioral differences on any of the three models**:

- Study: plans, calibrated profiles, fitted kits, search counts, capabilities,
  semantic poses/markers, all impact measurements, 475-sample sequence summaries,
  source preservation and diagnostics are identical.
- Full song: source facts/selection, unassigned event identities, diagnostics,
  explicit checkpoints, 9,985-sample sequence summaries and engine-contact errors
  are identical. Only build identity, elapsed timings and record metadata differ.

| Model | Study prepare: before → final | Full-song prepare: before → final | Final full-song review |
| --- | ---: | ---: | ---: |
| AvatarSample_B | 4,155 → 2,462 ms | 78.4 → 46.5 s | 3.8 s |
| Seed-san | 5,141 → 2,665 ms | 95.3 → 49.9 s | 14.3 s |
| Twist | 4,891 → 2,829 ms | 91.6 → 54.4 s | 12.6 s |

These before/final observations share inputs but were collected on a development
machine running other tests and browser QA; they are not an isolated benchmark.
The final full-song preparation of 46–54 seconds remains too long to call the
responsiveness problem solved. The one unassigned note per model and the precise
motion-quality limitations in the preceding tables are unchanged.

Final source checks passed: `cargo test --workspace --locked` **139 tests**,
`npm test` **245 tests in 36 files**, formatting, Clippy, TypeScript checking,
WASM build, production web build and document checks. The additional Rust test
covers the optimized search's equivalence to exhaustive enumeration.

The previous binary and measurements are preserved privately under
`.local/natural-contact/before-optimization-78246ccc/`. Final study and full-song
files use the output directories described above; exact compared-field hashes
and timing pairs are in `.local/natural-contact/optimized-review-comparison.json`.
Both full-song runs exit with code 1 for their known unassigned hit; the exact
comparison passes. This optimization is not evidence that those motion defects
have been repaired.

### Next acceptance work

1. Feed observed simultaneous score combinations and nearby transfers into
   bounded layout/planner validation, without demanding every theoretical pair
   or adding per-song/avatar exceptions. Keep source facts and uncertainty visible.
2. Preserve feasible posture/contact witnesses through constrained pitch-rate,
   torso and paired-arm trajectories; check the reported fill and cymbal intervals
   against actual adapter markers and visible skin/clothing.
3. Profile full-song planning stages and reduce their measured cost before
   treating import/preparation responsiveness as complete.

Strike-position variation remains deferred. No merge or deployment approval is
inferred from these results; browser observations are recorded separately below.


### Browser review

Local in-app Chromium review used `http://127.0.0.1:5189/drum-vrm/` at
1280 × 720. All three unmodified public presets were imported through Character,
automatically fitted, and inspected at Ready, Groove, Tom transfer, Ride, Both
crashes and Ending. Front/side views show the lowered sustained-groove silhouette
and semantic mounts; these views do not establish whole-song skin or clothing
clearance. The initial checkpoint screenshots use the `78246ccc…` candidate;
the exact numeric equivalence above covers the subsequent optimization.

The final `9b2b04ab…` browser run checked preset replacement, MIDI-first and
VRM-first preparation, playback/pause, checkpoint seeking and live English/Korean
switching without losing the prepared score. Replacing AvatarSample_B with
Seed-san cleared the previous posture/contact readout, trace and checkpoint
selection. No automatic inspection is performed merely to refresh the UI.

Twist's final browser song review reports 475 times, zero seek-order difference,
zero invalid values and 40 samples with possible proxy intersections (including
intentional count-in contact). This matches the independent numeric review; it
is not a claim of zero collisions. No warning/error console entries were present
in the final browser session. Local proof screenshots are retained in
`.local/natural-contact/browser/`, including `final-twist-review.png`; they are
not published with this PR. The local server remains available for owner review.
