# Coupled hand paths and preparation

This record continues the [v21 count-in/path measurement](shaft-count-in-and-path-followup.md).
It covers the unpublished v22 work on `codex/kit-grip-refinement`, starting from
`9e3af50aa46083ae97774c8cfae2ef339eb36ab6`. Intermediate measurements below do not
establish a final candidate, owner visual acceptance or Pages deployment.

## Fresh UI checkpoint — 2026-11-03

The next immutable candidate is `.local/common-motion-v22-candidate/source/`,
with its file hashes retained beside it. WASM SHA-256 is
`c5ad8257717d6a33f568a211e589933d1fa872fe5a236c198518317285eeb140`.
It includes the canonical coverage, common anatomy predicate, wrist-preserving
phase solver, necessary fit reach pruning and tapered radial clearance below.
Fresh current-UI fits all complete: B 47.696 s / 42,206 candidates / scale 0.96,
Seed 4.463 s / 48 / 1.0, Twist 3.935 s / 8,296 / 1.0. Each applied fit remains
unchanged on recheck. These observations are not controlled browser benchmarks.
Native uses the authored reference seat/kit without startup fitting.

The new 60 Hz short-demo matrix preserves source facts, assignment/contact,
seek and prepared restoration in all 12 cases. Motion acceptance is separate:

| Current fitted profile | Posture (53 hits) | Studio (193) | Interpretation (82) |
| --- | --- | --- | --- |
| Native | Pass | Posture and pre-roll pair | Posture |
| B | Packing-only residual | Pass | Pass |
| Seed | Pass | Posture | Posture and pair |
| Twist | Pass | Posture, equipment and pair | Posture, equipment and pair |

B Posture has no hard engine diagnostic. Its historical strict-negative browser
review fails on a −8.482 nm reconstructed tool clearance at 10.733333 s. The new
shared review classifier allows the existing 1 µm f32 packing reserve while
preserving raw minima and the planner's 2 mm free margin. Authored regression
checks reject real −4 µm/−1 mm overlaps, missing or non-finite measurements and
mixed measured/unmeasured samples. The old matrix is retained unchanged; this
classification correction is not presented as a newly executed matrix.

Evidence directories are `.local/reference-layout/v22-taper-current-ui-fit/`,
`v22-taper-native-short/` and `v22-taper-public-short/`. This checkpoint passes
334 Rust tests in release mode, the WASM/web builds, and 338 web tests across
44 files with one worker (244.16 s). The earlier concurrent web timeouts did not
recur; no timeout thresholds changed. Review-helper targeted checks also pass.
Actual adapter validation and remaining whole-catalog phase repairs continue.
This checkpoint is not ready for broader import-model handoff or publication.

## Earlier fresh UI caveat

The most recent frozen-seat Posture checks below are not the fresh model-load
workflow. A separate current-UI fit capture uses WASM
`c67c7afeb88ea5f701c839feba6bd98e79352632d16b00ba31376d35f2c5fc7c`
and the current default kit. All three fits complete and retain their applied
results on recheck. B takes 189.965 s and 42,206 candidates; Seed takes 5.207 s
and 48 candidates; Twist takes 9.765 s and 8,296 candidates. These single,
concurrent-session observations identify a load-time problem, not a benchmark.

The resulting 60 Hz Posture runs retain 53/53 hits and source/contact/seek/restore
invariants for each model. B passes sampled motion acceptance. Seed retains a
posture failure at 8.0 s; Twist retains equipment/tool failure at 10.7 s and
posture failure at 10.708333 s. The UI shows those Twist diagnostics as well.
Private records are `.local/reference-layout/v22-current-ui-fit/` and
`.local/reference-layout/v22-current-ui-posture/`. The WASM checkpoint predates
the subsequent departure-boundary guide. Broader handoff is not ready on this
evidence; final checks must use freshly fitted current UI inputs as well as the
frozen regression inputs.

The Seed source warning at 8.0 s was a wrist value of 0.6000000000000005 rad,
not a visible posture failure. Static checks, final phase validation, restoration
and inspection now share an anatomical norm tolerance of `1e−10`, while retaining
the exact squared deficit. This numerical reserve is 100 times the bounded
optimizer's `1e−12` norm convergence and below 0.000000006 degrees. It does not
change the 0.60 rad wrist limit or other joint bounds; material `1e−6` excess,
unreachable and non-finite poses remain invalid. A separate phase gate previously
allowed `1e−8` while inspection required exact zero, causing inconsistent results.

Fresh static-fit profiling also found unnecessary wrist enumeration outside the
existing conservative triangle reach envelope. Applying that necessary bound to
each impact/preparation/recovery probe reduces B's instrumented native fit from
73.284 to 23.746 s, skipping 10,216 impossible probes. Its complete proposal and
42,206 layout decisions remain identical. Native, Seed and Twist proposals also
match their baselines exactly. These are native same-input observations; final
browser latency still needs measurement. Private evidence:
`.local/fit-performance-v22/report.md`. No candidate budget or physical bound changed.

The Twist recovery failure at 10.7 s exposed a coverage gap: a fractional raw
recovery endpoint at 10.683535 s excluded the overlapping ordinary control window,
leaving its tail outside repair. Common canonical intervals now align non-source
joins outward to feasible existing controls, union overlaps and split at every
source/held-cue boundary. Raw breakpoints remain on the shared validation grid.
The focused 10.5–10.8 s witness retains the routed tips and source poses;
physical tool minimum changes from −1.010 to +0.190 mm and peak wrist speed from
5.889 to 4.347 m/s. These are isolated measurements, not full-catalog acceptance.

A departure-boundary route guide was rejected after the next WASM matrix.
Although it cleared a narrow cymbal boundary, it selected a 2.7209 m lateral
detour for Seed and caused an almost straight 179.76-degree elbow. It also changed
global source-witness choices elsewhere. The actual old Interpretation stick
already cleared the cymbal by 2.932 mm (native) / 3.869 mm (B) when measured with
the authored taper and no terminal exemption; the unrelated-body query had
instead expanded its point tip to a constant 6 mm radius. The correction belongs
in that shared physical query, not an extra route family. Private evidence:
`.local/departure-review-v22/`; source facts and the 2 mm free margin stay fixed.

## Scope and invariants

The owner requested continuation until the common motion work is ready for the
VRM 0.0/FBX import chat. The reference kit, larger shaft-middle wrist count-in,
forward left instruments and ordinary semantic body cores from v21 remain the
starting point. No avatar-name branches, stretched bones, altered source MIDI
times or clothing-dependent hard gates are introduced.

Three distinct questions are checked: assignment/contact at original hits,
intervening physical and anatomical motion, and actual rendered adapter output.
Exact marker contact is not evidence of natural posture or collision-free motion.
Native and WASM independent planning results are recorded separately; replay of
the same prepared result must remain deterministic.

## Changes under verification

- Count-in entry/exit interpolate the wrist and actual tool rotation together.
  The tip is reconstructed from the complete rotated grip/tool mount. Independent
  tip interpolation and quaternion interpolation previously bent the implied
  wrist path. The held pulse and authored source contacts remain unchanged.
- Hand assignment prefers an available ordinary-body-clear approach among the
  existing pitch/swivel/hand candidate families, after preserving assigned hits.
  Absence of a static core witness remains a diagnostic preference, not a new
  blanket rejection: torso motion can clear a static neutral-pose crossing.
  Explicit hand overrides still take precedence.
- Physical interval candidates retain the original routed tip and solve elbow,
  wrist and complete hand/tool orientation together. A measured hand-local forearm
  direction plus the complete grip/tip mount forms a virtual lever used only for
  algebraic reconstruction; both real bone lengths remain exact. This lever is
  not a reach or anatomical certificate. Each accepted interval still passes the
  existing equipment, ordinary-core, wrist/palm and opposite-hand checks.
  Endpoint/source binding and prepared restoration are checked separately.
- Existing rate costs compare actual tool rotation, wrist displacement and
  body-relative upper-arm direction. They are preferences, not physiological
  speed limits or a guarantee that every local speed decreases.

The fresh native Posture Study assignment changes only B's three floor-tom hits
at 7.5, 7.625 and 7.875 seconds from left to right hand. All four profiles retain
53/53 hits. Authored regressions separately cover a sole body-clear hand, explicit
overrides, and a neutral-pose core failure that must not discard a playable hit.

## Exact preparation optimization

The isolated comparison excludes the concurrent motion changes above. It uses a
private archive of `9e3af50`, identical numeric profiles/kit/source plans and a
native release build on arm64 macOS with rustc 1.94.1. The complete prepared JSON,
plan, sampled poses and inspections are compared, not only aggregate counts.

B's baseline short-study preparation spent 20.06 seconds in equipment routing
and 6.53 seconds in tool-pair routing. Static hand solving made 695,804 calls for
199,689 distinct observed requests. The implementation therefore reuses exact
immutable hand queries within one equipment context. Identity includes every
numeric profile field and optional shape, signed-zero bits, intended piece,
target, pitch, swivel and solve mode. Unknown pieces bypass reuse. Successful,
fallback and absent results retain the uncached result.

The transient context cache is bounded to 32,768 hand queries and 1,024 numeric
profiles, cleared on completion, and absent from prepared JSON. Reaching a bound
causes recomputation, not aliasing or a changed candidate search. A second change
evaluates existing lexicographic rank terms lazily and reuses immutable wrist/core
geometry within a solve. Candidate order, arithmetic, constraints and budgets stay
unchanged. These optimizations do not relax physical checks.

| Input | Baseline seconds | Optimized seconds | Exact comparison |
| --- | ---: | ---: | --- |
| Native Posture Study | 2.5669 | 1.4373 | Prepared, plan, 301 poses/inspections identical |
| B Posture Study | 27.6498 | 13.6287 | Prepared, plan, 301 poses/inspections identical |
| Seed Posture Study | 11.1525 | 4.3078 | Prepared, plan, 301 poses/inspections identical |
| Twist Posture Study | 12.6749 | 4.1083 | Prepared, plan, 301 poses/inspections identical |
| Native public full song | 88.8740 | 69.4391 | Prepared, plan, 2,989 poses/inspections identical |

The public full song retains 1,720 hits over 193.025623 seconds. These are single
native wall-time observations, not statistical benchmarks or browser guarantees.
Long-song preparation remains slow. Private reproducibility records live under
`.local/prepare-profile9e3/`, including `equivalence.json`, `song-equivalence.json`
and `lazy-equivalence.json`; no private model or MIDI is committed.

## Intermediate actual-adapter measurement

The first v22 WASM build was measured with newly extracted native/B/Seed/Twist
profiles at the frozen reviewed seats and reference kit. The Posture Study scan
covers 1,561 times from -0.5 through 12.5 seconds at 120 Hz. Private evidence:
`.local/wrist-count-in-v22/actual-first/`. This predates the final interval repair.

All four retain 53/53 assignments. B, Seed and Twist each retain 59 measured
contact events. Their maximum actual contact errors are respectively 0.547,
0.479 and 0.525 micrometres; marker errors are 0.223, 0.442 and 0.147 micrometres.
Reverse seek, prepared restore poses and prepared-byte equality pass for these
three adapter runs. Native adapter equality is not inferred from absent fields.

Ordinary body cores are clear at all sampled times. B's sampled equipment and
tool-pair paths are also clear. Remaining failures include Seed's crash equipment
overlap of 0.161 mm at 10.675 s, Twist's tool-pair overlap of 6.447 mm at 6.76667 s,
and Twist's crash equipment overlap of 6.498 mm at 10.68333 s. Interior wrist/palm
limits still fail in some tom transfers; clear contact markers do not supersede
those failures. The complete four-demo matrix is being checked separately.

Independent native rate checks also found local increases: Twist's left upper-arm
direction reaches 21.014 rad/s rather than 19.364 at 6.88333–6.89167 s, and native
right-wrist speed reaches 11.010 m/s rather than 10.152 at 7.88333–7.89167 s.
These discrete samples identify a transition issue; they are not clinical limits.
Endpoint pose preservation alone does not prove derivative continuity.

## Intermediate complete catalog

WASM `e7e05995f3aeb9e50fef89c529a6e1e5e4a49eedb68e56d9820349e1d59c1845`
was checked across native plus all three official VRMs and all four demos. This
inventory predates the subsequent torso, cue and continuous-hand repairs. The
same immutable bytes were used in each process. Outer samples include 30 Hz review
points, source contacts and diagnostic timestamps; they differ from the uniform
120 Hz actual-adapter scan above. All timings below are concurrent-session
observations. They are not comparable isolated speed benchmarks.

| Model/demo | Assigned | Preparation s | Tool intersecting samples | Core intersecting samples | Equipment min mm | Pair min mm | Core min mm |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| native-posture | 53/53 | 10.3 | 0/476 | 0/476 | 0.007 | 0.001 | 0.234 |
| native-studio | 193/193 | 52.0 | 0/1240 | 52/1240 | 0.069 | 3.114 | -23.425 |
| native-interpretation | 82/82 | 5.3 | 0/423 | 0/423 | 0.158 | 0.001 | 0.325 |
| native-transparent-girl | 1720/1720 | 263.5 | 1107/8980 | 1951/8980 | -5.259 | -11.990 | -74.336 |
| avatar-sample-b-posture | 53/53 | 66.4 | 1/478 | 0/478 | 0.070 | -0.620 | 75.754 |
| avatar-sample-b-studio | 193/193 | 73.8 | 0/1240 | 0/1240 | 0.001 | 6.912 | 2.229 |
| avatar-sample-b-interpretation | 82/82 | 56.6 | 4/425 | 3/425 | -6.000 | 0.001 | -0.407 |
| avatar-sample-b-transparent-girl | 1720/1720 | 624.5 | 212/8981 | 508/8981 | -8.161 | -11.984 | -14.002 |
| seed-san-posture | 53/53 | 13.0 | 1/478 | 0/478 | -0.161 | 0.001 | 0.087 |
| seed-san-studio | 193/193 | 69.4 | 0/1240 | 483/1240 | 0.004 | 18.664 | -99.355 |
| seed-san-interpretation | 82/82 | 15.3 | 3/425 | 0/425 | 0.009 | -8.686 | 0.176 |
| seed-san-transparent-girl | 1720/1720 | 512.9 | 268/8980 | 207/8980 | -19.785 | -11.927 | -29.256 |
| twist-posture | 53/53 | 13.0 | 3/478 | 0/478 | -0.879 | -6.447 | 33.812 |
| twist-studio | 193/193 | 41.7 | 5/1241 | 0/1241 | -2.604 | -1.543 | 42.923 |
| twist-interpretation | 82/82 | 25.6 | 4/426 | 21/426 | -3.075 | -1.159 | -1.723 |
| twist-transparent-girl | 1720/1720 | 431.4 | 140/8980 | 474/8980 | -15.151 | -11.890 | -20.289 |

All completed runs retain assignment/source/layout/contact/seek/restore invariants: **True**. Maximum numeric contact error across completed runs is 0.536 µm. These preserved invariants do not make the motion pass; all completed runs retain blocking motion diagnostics.


The Interpretation failure was a scheduling artifact: a home-to-target preparation
window could already have begun although the real hit remained 333 ms away.
The candidate now permits direct cue-to-moving-stroke handoff within the existing
real-hit exclusion window. A fresh native Interpretation run retains all 82 hits
and all source cues; complete candidate acceptance is still required.

Torso candidate scoring previously omitted the ordinary body core used by final
hand inspection. The candidate uses that same cost and combines forward/reverse
bounded yaw planning to anticipate upcoming targets. The native Studio comparison
retains 193 hits and removes sampled ordinary-core overlaps; Seed Studio's worst
core overlap improves from 99.355 mm to 10.521 mm, but remains a failure. The four
fresh Posture runs retain 53 strikes plus six shaft contacts and fixed cue wrists;
interior wrist/tool/pair failures remain. Do not generalize a single repaired
phrase to the catalog or call the bidirectional candidate accepted yet.

Private catalog records: `.local/reference-layout/v22-catalog-first/combined-summary.json`,
`v22-catalog-shorts/` and `v22-catalog-full-tail/`. These contain all 16 unique
prepared states for focused replay; two repeated short runs produced byte-identical
prepared results. The inventory preserves 8,192/8,192 assigned hits and every
source/layout/numeric-contact/seek/restore invariant, while all 16 still have
blocking motion findings at this intermediate build.

## Bounded contact and radial-query corrections

The nine-pitch contact schedule could reject an available fast floor-tom to crash
transfer because both individually feasible intervals lay between coarse samples.
The existing successful schedule remains the fast path. Only a failed schedule
refines adjacent supported/unsupported boundaries three times, retaining every
original candidate and adding supported witnesses within a 33-candidate limit.
The 2 rad/s change limit, source times and physical contact gates are unchanged.

Full-song right-hand schedules retain 840/840 B and 849/849 Seed contacts with no
unsupported entries. The three previously failing crash transfers at 41.704030,
74.985156 and 153.185039 seconds use rates below 1.851 rad/s. Actual stored-yaw
contact checks retain at least 48 mm ordinary-core clearance and 2.145 mm equipment
clearance. All eight already-successful Posture hand schedules remain byte-identical
to the original coarse schedule. This fixes scheduling false negatives; it does
not establish full-song temporal acceptance. Private evidence is under
`.local/segment-obstacle-audit/`.

The radial obstacle query previously stopped once its conservative lower bound
became nonnegative, although an unrelated-body caller still needed a 2 mm margin.
It now proves the caller's existing threshold: zero for intended tangent contact,
2 mm for unrelated radial obstacles. Geometry and margins are unchanged; unresolved
queries remain conservative failures within the same depth/visit bounds. This
removes false ride-corridor failures for B and Twist without moving the routed tip.
Six query regressions and the equipment-motion suite cover proof conservatism,
actual intersection rejection and the integrated near-rim case. Private query
records are under `.local/ride-corridor-audit/`.

## Coherent transfer correction

A focused independent decomposition found that optional shoulder-radius expansion
was the dominant source of the sharp Twist rack-tom preparation. Framewise obstacle
clipping removed about 70 mm of that optional offset over 1.25 ms near 6.907 s.
At that instant the old tip derivative was 65.05 m/s; the physical curve followed
it, while its own shape change contributed only 0.275 m/s to elbow motion.

Removing that redundant expansion retains the original source contacts and the
coherent chord, bounded arch or whole-interval waypoint. In the isolated
6.625–7.0 s comparison at 480 Hz, sampled tip peak falls 32.78→3.79 m/s, original
elbow speed 33.24→1.39 m/s, with ordinary-core clearance at least 49.8 mm and tool
clearance at least 2.152 mm. These are focused route measurements, not a full-demo
pass. An authored grazing-rack regression checks the raw trajectory separately.

The physical baseline also stops forcing a neutral forearm at midpoint or mixing
its shoulder-to-tip plane with the old shoulder-to-wrist swivel coordinate. Plain
measured endpoint interpolation is accepted directly when every shared dense
constraint passes; numerical correction is reserved for a remaining failure.
Paired replanning is still necessary: retaining the old opposite-hand curve in the
isolated experiment leaves a 7.97 mm stick-pair overlap. Existing left-hand
witnesses can clear that point with about 16 mm elbow displacement, which is not
itself proof that a whole paired interval passes. Evidence:
`.local/architecture-rate-v22/`.

## Intermediate coherent-route catalog

The frozen native snapshot under `.local/short-matrix-v22-coherent-route/` combines
coherent raw transfers, the repeated-piece rebound cap, source-core certification
and measured endpoint curves. It uses the actual numeric profiles extracted under
`.local/wrist-count-in-v22/virtual-shared/`; earlier frozen-rest profiles are not
byte-equivalent inputs. It precedes source-certificate preservation in pair routing,
final source-warning reconciliation, restricted wrist-preserving phase search and
the later cue/grid/quality corrections.

Across native/B/Seed/Twist and Posture/Studio/Interpretation, all 1,312 hits and
1,124 actual hand source poses retain source/layout/contact/seek/restore invariants.
Maximum measured contact error is below 0.554 micrometres. All sampled Posture
physical/anatomical/palm conditions pass for Twist. Native, B and Seed still have
small interior palm/wrist deficits; Studio and Interpretation retain material
interior failures. This is not a whole-catalog or visual pass.

## Wrist-preserving search and inspection boundaries

The three residual Posture intervals have feasible fixed endpoints. A restricted
solve of only the existing arm-plane/tool-lever twist coefficients clears them
without changing the measured local forearm curve, source quaternion tangents,
source times, tool targets or bounds. The full wrist-flex solve remains a fallback.
Independent 480 Hz witness comparisons measure:

| Interval/model | Wrist peak m/s, before → after | Hand rotation peak rad/s, before → after |
| --- | ---: | ---: |
| Native RH 3.5–3.75 s | 3.466 → 1.212 | 9.938 → 1.815 |
| B RH 8.0–8.25 s | 1.987 → 1.113 | 24.476 → 2.673 |
| Seed RH 2.0–2.25 s | 2.635 → 1.434 | 7.700 → 4.212 |

These focused witnesses preserve exact source poses and pass all five physical
conditions, with tool margin at least 2.639 mm and pair margin at least 69.664 mm.
They do not prove global derivative continuity. Evidence:
`.local/posture-residual-rates/summary.json`.

A fresh B Posture scan also exposed a real off-grid wrist violation at a known
equipment inspection time between 480 Hz samples. Planning and restoration now
share those existing inspection times as well as the absolute lattice and control
boundaries. The fresh focused B result has zero anatomical violation at that union,
maximum palm deficit 1.12e−12, all 53 source facts and successful restoration.
This repairs a sampling mismatch without relaxing a limit.

A physically safe original interval can still oscillate between static orientation
witnesses. In Twist RH 7.5–7.625 s, the original hand rotates rapidly away from and
back toward its source grip. The same restricted curve reduces the existing dense
motion objective 1.6605→0.03149, hand rotation peak 97.395→6.223 rad/s and wrist peak
4.062→1.630 m/s while passing all physical checks and retaining exact endpoints.
The zero-correction curve alone fails the palm policy and is not accepted. Evidence:
`.local/natural-safe-span-v22/twist-7.5-fitted.json`. Safe-path replacements require
a strictly lower dense motion objective as well as the unchanged physical gates;
that cost is not a physiological speed limit.

## Fresh Posture checkpoint after common corrections

The subsequent native run under `.local/posture-preserve-wrist/` combines the
restricted-first curve search, known inspection grid, exact held-cue semantics
and safe-path simplification. Each model is scanned at 480 Hz plus source/control
and inspection times (6,258–6,294 samples per profile). All four have **zero**
material ordinary-core, equipment-margin, physical-pair, anatomical or palm-policy
failures. All 53 hit facts remain unchanged and prepared restoration passes.
Preparation is 2.735 s native, 6.785 s B, 5.637 s Seed and 5.074 s Twist, observed
in this native environment rather than a browser benchmark.

Uniform 120 Hz hand-rotation peaks are 16.14/19.95/22.13/16.65 rad/s respectively;
the earlier Twist 94.66 rad/s oscillation is absent. The fast floor-tom-to-ride
transfer still has wrist peaks around 6.18–8.18 m/s. Numerical physical/posture
acceptance of this short study does not establish naturalness, complete-catalog
acceptance or owner visual approval. `current-summary.json` and per-model
`*-current.json` / `*-current-prepared.json` retain this checkpoint separately.

## Acceptance still required

Finish interval repair and remeasure the final WASM, all four demo plans, complete
source facts, prepared restoration, actual adapters and textured multi-view
playback. Keep numerical, visual and publication results distinct. Owner visual
review of the enlarged count-in and common arm motion precedes broader VRM 0.0
and FBX validation. This intermediate record does not authorize that handoff or
claim temporal acceptance.
