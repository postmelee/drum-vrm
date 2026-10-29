# Elbow and grip revision evidence

This is an unpublished follow-up to [PR #65](https://github.com/postmelee/drum-vrm/pull/65),
starting from `68ad43db83978ff38da0d44c376f7a5e4e068874`. It does not supersede the
published #64 result or establish complete natural-motion acceptance. The owner
requested preservation of the earlier arm motion, an inward palm/thumb-up default,
player-facing rack toms, and review before the VRM 0.0/FBX handoff.

The sections below retain distinct measured checkpoints. A later implementation
does not retroactively turn an earlier temporal failure into a pass.

## Measured checkpoint: 20-degree toms and prepared orientation continuity

WASM SHA-256: `00c7cadd09fb7f1faf826dc2ea0cef1a0747f57d6c1a93ad4de8746c84dc9639`.
Planner v14, prepared format 4. Node 24/macOS arm64. The private fit inventory
records the source-file hashes, unchanged licensed model hashes, input geometry,
fitted profiles and exact candidate results. Private inputs and raw motion streams
remain under `.local/`; no uploaded asset is added to the repository.

Implementation at this checkpoint:

- The common anatomical elbow pole no longer rotates with hand pronation. The
  first grip candidate preserves the previous arm through a rigid wrist-to-tip
  twist. A bounded static shaft/reach search remains when that cannot produce a
  valid witness; therefore this is not an unconditional old-arm preservation claim.
- Existing hand graph transitions rank actual tool rotation, wrist translation
  and body-relative upper-arm direction using duration-aware costs. These are
  preferences, not physiological speed limits or continuous-motion certificates.
- Prepared replay uses its selected orientations without a pointwise projection.
  Adaptive orientation-only insertion was removed after it produced rapid flips.
  Invalid interior poses remain visible in diagnostics.
- Both rack toms tilt 20 degrees toward the player instead of 12 degrees; their
  heights are unchanged. The left crash moves 4 cm toward the player. Lowering
  the toms another 2 cm was rejected because it overlaps the snare envelope.
- Nonradial drum bodies use measured surface-aligned boxes containing every drawn
  vertex. Radial cymbals retain their tight world AABBs and curved surface query.
  The same bounds feed fitting, lower-limb checks and whole-tool queries. A rotated
  box has smaller empty regions in some directions but is not necessarily a subset
  of the former world AABB; arbitrary previously accepted layouts are not guaranteed
  to retain their result. No physical clearance margin was relaxed.
- Transfer routing and exact local measurement reuse were corrected; intended
  contacts, original MIDI times, source identities and fixed bone lengths remain.

## Static fitting

Fresh actual adapters and newly measured geometry were used. All four fits pass
layout, generated hardware, structural lower-limb, ready-pose and required-pair
checks; applying the fit again is idempotent. Measured clothing warnings remain
advisory and are not described as a hard-body collision pass.

| Rig | Fit time | Candidates | Kit scale | Notable adjustment |
| --- | ---: | ---: | ---: | --- |
| Native skeleton | 1.35 s | 0 | 100% | None |
| AvatarSample B | 104.81 s | 42,165 | 90% | Rack height retained |
| Seed-san | 4.96 s | 51 | 100% | Common local repair |
| Twist | 50.70 s | 24,253 | 100% | Seat +6 cm; rack +5 cm and left/back 5 cm |

The slow fit cases are not accepted performance. An exact search-order trace
reproduces the same results: B spends 42,000 probes in failed branches before
successful 77- and 88-probe compact branches; Twist spends 24,000 before its
successful 253-probe seat branch. A future shallow-before-deep schedule should
reuse the existing families and gates instead of adding another search grid.
Simply lowering the accepted Twist rack by 5 cm creates two snare/tom overlaps
and reduces its static contact certificates from 21 to 17.

Private static evidence:
`.local/reference-layout/tilt20-obb-fresh-static-v14/summary.json` and
`.local/fit-search-order-audit/review.md`.

## Why fewer posture warnings did not mean better motion

An intermediate adaptive track reduced invalid pose samples but inserted about
75 degrees of tool rotation within 4.17 ms. The runtime nearest-feasible projection
also jumped when an admissible interval appeared or disappeared. Both mechanisms
were removed from this checkpoint rather than hiding the resulting failures.

On the same native short studies, the earlier intermediate thumb-preference
checkpoint had peak sampled tool rates of 157.5 and 120.7 rad/s. The 20-degree,
fixed-control checkpoint measures 38.1 and 53.0 rad/s, with wrist speed reduced
from 17.7/13.6 to 8.2/7.7 m/s. These peaks still do not justify a naturalness claim.
The comparison changes layout as well as continuity handling and is not an
isolated causal estimate for either change.

The native Posture Study no longer reports an equipment/tool path penetration,
but still reports two stick-pair samples and 539 posture samples. Exact f64
classification confirms the latter is not Float32 noise: 194 are grip-policy only,
206 anatomy only and 139 both. Most policy-only overshoots are small, but wrist
excesses up to roughly 20 degrees and elbow-fold/behind-body violations remain.
Increasing a reporting tolerance would not solve those motions.

At the native dual-crash impact, the chosen right-arm swivel is +0.65 rad with
about 88 degrees of upper-arm elevation. Swivel zero lowers the elbow but exceeds
the wrist limit. A lower existing pitch can lower the elbow further but fails the
whole-tool clearance check. This remains an explicit unresolved posture; the
current graph cannot fix it merely by increasing its gravity preference.

## Checks and limits

- Rust release suite: 258 passed.
- Changed web geometry/calibration/default/hardware/project/cache/message tests:
  60 passed; additional actual scene-geometry restoration regression: 1 passed.
- Typecheck and production web/WASM builds passed. The existing large-chunk build
  warning remains. Document structure/link/pair check passed.
- The earlier full web run with the first OBB attempt had 320 passes and three
  default/hardware failures. The 20-degree/radial-AABB correction passes those
  failures; that earlier result is not relabeled as a clean full-suite run.
- The 30-second native fitted demo takes about 74 seconds for one preparation at
  this checkpoint. This is not compatible with the existing 60-second test budget;
  the timeout was not increased.

Local browser review uses `http://127.0.0.1:5195/drum-vrm/?lang=ko` and the rebuilt
production bundle. Native tom and dual-crash side views plus actual textured B
views are saved under `.local/elbow-revision/visual/`. The tom pose improves, while
the right crash still shows an undesirable elbow position. Other camera views,
numeric contacts and deterministic replay do not override that visible finding.

The all-demo matrix uses four catalog demos and three actual public profiles,
plus the native reference. Every arranged source event is retained, including the
reviewed full-song omission. Full results are recorded separately per rig under
`.local/reference-layout/continuity-tilt20-<rig>-v14/`.

All 6,144 public-model hits are assigned. All 16 native/public runs preserve source
facts, have zero seek-order difference, and restore prepared data successfully.
The maximum reconstructed contact error is 0.0000673 mm. **None of the 16 runs
passes temporal whole-tool/posture acceptance.** These contact results are not a
claim about natural wrists, elbows or collision-free movement.

| Rig | Posture prep | Studio prep | Interpretation prep | Full-song prep |
| --- | ---: | ---: | ---: | ---: |
| Native | 8 s | 74 s | 5 s | 259 s |
| AvatarSample B | 145 s | 86 s | 159 s | 708 s |
| Seed-san | 33 s | 192 s | 16 s | 577 s |
| Twist | 65 s | 177 s | 14 s | 299 s |

Public-model runs overlapped on the same host; these are observed end-to-end
latencies, not isolated throughput benchmarks. The costs are nevertheless too
large for the intended interactive workflow. Do not describe this checkpoint as
an optimization success. It remains unready for the broader model handoff.

## Playable-region and wrist-first checkpoint

WASM SHA-256: `0876a17caa5f31b7e3a4c58d39d9e947fe57d763792d8b655bc47a3a4a41ba58`.
This checkpoint still uses planner v14 / prepared format 4, before the later
legacy-arm-relative orientation change and approach-corridor classification.

Two inconsistent coordinate systems had invalidated tilted-cymbal impacts.
The measured surface radius used the foreshortened horizontal world box, while
the contact target used the full radial profile. The intended-contact predicate
also measured distance in a tangent plane on the curved bow. At 15 degrees,
an actual valid radial target exceeded that shrunken region and lost its terminal
contact allowance. Surface measurement now retains the authored surface-frame
radius; radial target validation and intended-contact queries share radial-local
containment. Existing region fractions, terminal length and physical margins
remain unchanged. Tests independently compare actual mesh vertices and rigid
tilt/yaw/scale transformations, including 15, 20 and 25 degrees.

The static search tries all existing wrist-only pitch/swivel candidates before
allowing the arm-changing fallback. On the same four frozen profile inputs,
complete contact-certificate counts remain 18/20/17/17; ready and required pairs
remain feasible. Exact old-algorithm wrist/elbow preservation among selected
single-contact witnesses increases from 9/54, 3/62, 4/52, 9/51 to 47/54, 43/62,
41/52, 42/51. This is not preservation of every published animation frame or a
speedup claim; those measurements concern static witnesses.

Actual native Posture Study preparation, at the 10.5-second dual crash:

| Right-crash tilt | Selected pitch / swivel | Upper-arm elevation | Wrist swing |
| --- | --- | ---: | ---: |
| 12 degrees | -0.15 / +0.65 rad | 87.973 degrees | 34.373 degrees |
| 15 degrees | -0.30 / +0.30 rad | 72.754 degrees | 34.358 degrees |
| 20 degrees | -0.30 / +0.30 rad | 72.137 degrees | 34.339 degrees |

Torso yaw remains +0.09 radians and the left hand is identical in all three.
The actual selected right-hand poses satisfy contact, anatomy and palm policy;
all use the allowed palm-down endpoint. The common default adopts 15 degrees:
20 degrees gives only another 0.62 degrees of elbow lowering. The whole study
still fails interior posture and post-crash stick-pair acceptance in every variant.
Evidence is private under `radial-frontier-native{12,15,20}-v14` and
`wrist-frontier-production/actual{12,15,20}.json`.

The authored 45-degree shaft/hand-forward mount was also audited independently.
A horizontal shaft and parallel horizontal forearm with an inward palm cannot
both satisfy the 34.4-degree wrist bound. That does not make the inward grip
impossible: an existing tip-up approach represents low-elbow inward poses for
all four profiles at their reference rest targets. These are not the separate
dual-snare ready targets and do not certify equipment clearance. The mount and
wrist bound were retained; the audit is `.local/grip-concept-probe/review.md`.

## Rejected baseline-relative replay and two-component experiment

The intermediate v15 / format-5 engine (`c2a3e699d04b0ae80b1743972bf7321c27943e42b82f62b30ad1256dfa4da4ae`)
recomputed a legacy wrist baseline during replay. Although exact rigid-twist
unit cases preserved arm joints, the legacy four-step roll solve was sensitive
between controls. Native Posture Study at 0.75 s reached 208.9 rad/s sampled tool
rotation and 11.3 m/s wrist speed. This was rejected as a review candidate.
All 16 demo/model runs still failed temporal acceptance. Static fits, assigned
contacts and deterministic restoration did not establish acceptable motion.

A sparse nonnegative-edge graph optimization (`ea85eaf6d3fb677549aff9fe64356bddfa03325700fcab5def3f4fc1f9434064`)
retained exactly equal diagnostics, kinematics and contacts on three native
studies. Native Studio preparation remained about 73 s and the existing 60 s
browser test still timed out. Its budget was not increased.

The following v16 / format-6 experiment (`1b423d090dd0de914e26b2da600a526268beb8e8c4a1238500abb6a2e9eb0af8`)
stored both baseline-arm and grip components, eliminating replay-time legacy
optimization. It also used 25-degree toms and a 15-degree right crash. Native
Posture peak rotation dropped to 38.0 rad/s, but Interpretation exposed another
failure: independently choosing each component's shortest arc can combine into
a near-full-turn wrist motion despite nearby endpoint orientations. At 4.7–4.75 s,
the right-hand peak was 168.1 rad/s. This experiment is not a completed solution.
Its full Rust suite passed 267 tests; these numerical interpolation failures
illustrate why unit tests alone do not establish natural motion.

## Static rack-grip feasibility and preparation bottleneck

On the measured native geometry, 20-degree high toms have no strict inward-grip
witness in the existing pitch/swivel families. Both 25 and 30 degrees have such
witnesses while retaining body/support/leg, ready and required-pair acceptance.
Twenty-five degrees is the first tested angle with that capability. This is a
static feasibility result, not proof that a whole performance selects that grip.

The actual high-tom selection exposed a solver ordering problem. At an existing
pitch/swivel, the arm-preserving inward witness intersects the tool margin by
0.112 mm. A bounded alternative at the same pitch/swivel clears it by 0.698 mm
with a 34.03-degree wrist angle. Returning the first hand before checking equipment
hid that alternative. The physical margin must remain; candidate selection needs
to consume the same actual whole-tool predicate.

Private native phase profiling identifies tool-pair route repair as the dominant
cost: 25.077 s of a 30.127 s preparation (83.2%). In detailed instrumentation,
98.1% of route time was static hand solving: 1,255,558 swivel solves across 151,178
hand measurements. Of these, 115,759 hand queries repeated identical effective
inputs. The graph's eight passes totaled only 0.090 s. MIDI parsing and final
inspection were not the main bottlenecks. Instrumented runs produced equal final
plans and diagnostics. This is a native snapshot measurement, not a browser speed
claim or evidence for a later implementation.

A separate gravity-pole experiment was rejected. Replacing the established
anatomical pole with projected vertical gravity barely changed the best tom/crash
arm elevations (under 0.2 degrees) but reduced feasible inward-tom and natural-crash
candidates by moving their equivalent swivels toward the existing bounds. It did
not repair the underlying grip/approach selection. At the current crash geometry,
inward-only left-hand witnesses fail physical shaft clearance and the right-hand
minimum wrist angle exceeds its bound. Conditional palm-down crashes remain valid
policy; this does not justify forcing palm-down toms. No production pole change
was adopted from that experiment.

## Single-orientation and exact-cache checkpoint (v17)

WASM SHA-256: `23182743391f028d428a366cd2a012b89c57ed9fadca6b95b7470cdcf98b1ecf`.
Prepared format 7 retains a single quaternion relative to the moving contact
approach. The previously failing two-component interval had a 34.61-degree
endpoint separation but accumulated 325.75 degrees. A single shortest arc
removes that component-composition loop. This does not certify anatomical
validity inside an arbitrary quaternion interpolation.

The common `HandEnvironment` applies the actual equipment predicate while
selecting static witnesses, instead of rejecting only the first returned pose.
The final source also propagates that predicate into the reach-cone correction's
internal rigid-twist search. A regression rejects the original cone orientation,
accepts another with identical arm/contact, and rejects the entire family when
no candidate passes. The latter correction is included in v18's binary, not in
the v17 WASM hash above.

The route cache retains exact per-hand results across its bounded repair loop.
Keys include the input/profile context, limb and exact effective height/pitch
bits; there is no quantization. Every paired-stick query uses the current other
hand. The 16,384-entry bound only causes recomputation when evicted. Sparse
nonnegative graph-edge penalties reuse the exact separable minimum unless its
winning edge needs rechecking; candidate families and physical bounds are unchanged.

An isolated serial native-release comparison on the same source, profile and
193-hit Studio input measured 33.032 s without the cache and 11.047 s with it.
Route time fell from 26.448 to 4.525 s. Both executed 157,528 route evaluations and
produced byte-identical prepared data (SHA-256
`a63461c0741162dd42d6d7c5756e84330c73baa90999f39239dbd4469813711c`).
This is an exact native benchmark, not a claim that all browser/model preparation
is now fast. Evidence: `.local/route-memo-v17/comparison.json`.

The fresh v17 static fits passed for native/B/Seed/Twist at 100% kit scale.
Observed fit times were 1.81/85.27/6.48/10.24 s. All 16 four-demo runs preserved
source facts, prepared restore and zero seek-order differences. All 8,192 hits
including native were assigned; maximum numeric contact error was 77.31 nm.
**Zero runs passed full temporal acceptance.** Public full-song preparation took
approximately 18.2/9.6/11.1 minutes for B/Seed/Twist on overlapping host runs.
Those are observed latencies, not isolated performance comparisons.
The private matrix summary is `.local/elbow-revision/v17-matrix-summary.json`;
per-run evidence is under `.local/reference-layout/inward-<rig>-v17/`.

## Shared physical grip and lower right crash (v18)

WASM SHA-256: `8ee13095fa077349437e9a0f8d503dccac8f359d178626fe9b82b4012f1331b3`.
Planner v18 retains prepared format 7. The physical shaft now crosses the measured
hand-forward/across plane at 30 degrees instead of the authored 45 degrees.
This is a common animation grip, not a universal anatomical measurement. Actual
finger wrapping, thumb fitting, socket rotation, tool offsets and calibration
are all recomputed from that shaft; the renderer and engine do not use different
grips. Bone lengths and the 0.6-radian wrist bound remain unchanged.

The old 45-degree construction itself exceeded the wrist bound for an inward
palm with a horizontal shaft and parallel forearm. A controlled actual-adapter
native comparison, on the same 25-degree kit, changes the first high-tom impact
at engine time 6.5 s from 70.98-degree pronation to inward, while upper-arm
elevation falls from 38.98 to 34.18 degrees. Wrist swing changes from 22.04 to
24.49 degrees. At -0.5 s both ready hands are inward, with approximately
21/19-degree arm elevation and 20/22-degree wrist swing. Time 0 is a count-in
stick cue, not a resting-pose sample. These native comparisons use the frozen
cone-corrected native executable with v17 metadata; they are distinct from the
final v18 WASM run.

The right crash moves from Y=1.38 to 1.30 m. With the 30-degree grip, the native
dual-crash right arm drops from 78.31 to 67.22 degrees and wrist swing from 29.76
to 26.86 degrees. A 13 cm lowering was rejected: it created real layout/tool
conflicts, and fitting raised it back by 5 cm. Rack height stays unchanged;
25 degrees is retained because 30-degree heads alone did not improve the actual
first-tom grip. Left-crash approach remains 4 cm closer than the starting batch.

Fresh actual 30-degree adapters, the lowered kit, common fitting, then the 53-hit
Posture Study were checked for all three public models. Actual rendered socket
markers were also evaluated at 55 planned contacts, including count-in cues.

| Rig | Fit / required pairs | Hits assigned | Actual adapter contact max | Right-crash elevation before / after lowering |
| --- | --- | ---: | ---: | ---: |
| AvatarSample B | Pass | 53/53 | 0.186 µm | 112.74 / 99.82 degrees |
| Seed-san | Pass | 53/53 | 0.400 µm | 105.59 / 89.61 degrees |
| Twist | Pass | 53/53 | 0.101 µm | 114.98 / 106.54 degrees |

Source facts, fixed bone lengths and prepared restoration were preserved.
These public comparisons include the fresh common fit; they do not isolate the
grip from layout/seating changes. The first high tom remains pronated for B
(90 degrees) and Twist (73.79 degrees), while Seed is inward. All three still
report temporal tool/posture violations. Their tool-equipment sample counts are
17/2/11, worst penetrations 2.9/3.2/10.3 mm; stick-pair counts are 5/0/1.
**This is a useful local review candidate, not completion of the all-model,
all-demo motion request.** Owner review still precedes the VRM 0.0/FBX handoff.
Evidence: `.local/actual-grip-v17-native/`,
`.local/crash-height-v18/`, `.local/actual-grip-v18-public-lower8/`.

The native browser-demo test was also corrected to exercise the actual app
boundary: authored reference seat, `plan_midi`, one measured-profile preparation,
then `from_prepared_json` restoration. Previously the helper lowered the native
seat from 0.79 to 0.73 m and prepared two whole performances; that exercised a
different input and took about 100 s. The corrected v17 test passes in 31.64 s
under its unchanged 60 s budget. Explicit auto-fit/idempotence and legacy
constructor coverage remain. This test-fixture correction is not an application
speedup claim.

### Final v18 boundary checks

The rebuilt v18 WASM above prepared the same frozen measured/fitted inputs again
for native, B, Seed and Twist. All four retained 53/53 hits, source identities and
original times, exact seek-order equality and prepared restoration. Maximum
profile-reconstructed contact error was 64.22 nm. Preparation took approximately
6.58/51.59/22.34/26.59 s; these were not isolated fit/throughput benchmarks.
All four still fail temporal acceptance. The final numeric records are under
`.local/reference-layout/v18-posture-final/`; the earlier four-demo matrix is
not promoted to a v18 all-demo pass.

Executed checks on the v18 source/bundle:

- Rust release suite: 271 passed; warnings-denied Clippy and format check passed.
- Full web suite: 333 passed across 44 files, with unchanged time budgets.
- Measured grip/thumb/socket tests: 24 passed, including arbitrary raw rest bases
  and proportions. A new independent socket-to-hand-bone check catches the former
  forced wrist-angle incompatibility without prescribing a universal human angle.
- Typecheck, rebuilt WASM and production web build passed. The existing large
  JavaScript chunk warning remains. Documentation/link/language-pair checks pass.
- Local native high-tom and dual-crash side views use the rebuilt bundle at
  `http://127.0.0.1:5195/drum-vrm/?lang=ko`; captures are private under
  `.local/elbow-revision/visual/native-v18-*.png`.

The source remains a draft owner-review checkpoint. It does not merge PR #65,
change Pages, close #18/#23, or authorize the deferred VRM 0.0/FBX validation.
