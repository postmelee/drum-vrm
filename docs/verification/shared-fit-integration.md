# Reference layout and shared fitting integration

This record covers the local integration of FBX/shared fitting candidate
`e5fafb993768f5f455358ec7d99552559b1130fd` and reference-layout checkpoint
`da026985b73abaa980c4ef702abc785f831137b2`. The former includes `main` through
`6038e8e138cbe6ddd30fcf1ce99de0c7750df9fa`; the latter started at `d8055d0`.
It is not a Pages publication or approval to merge/release.

The [reference-layout record](reference-layout.md) and
[earlier recovery evidence](fbx-import.md#shared-fit-recovery-candidate--2026-10-27)
remain historical. Their different geometry and acceptance gates must not be
combined into a claim that the integrated candidate passes.

## Preserved reference and common contract

The incoming `shared/default-kit.json` is unchanged, SHA-256
`41b1c2ba246b2d9d53e82aa60ccf5f6a700eff064dc35624dd7857e7e59e2782`.
The closer floor tom and left crash, nominal cymbal radii, direct hi-hat linkage,
tripod phases and optional suspended-floor mount all remain physical inputs.
Shell fitting has a 0.76 lower bound. No avatar identity, source format, MIDI
retiming, bone stretching or reduced clearance margin is used.

Both import formats supply the same semantic `RigProfile`. Seat candidates move
world roots, torso references and hand rest tips; local hand/leg/foot envelopes,
leg rest frames, neutral tool rotations and bone lengths stay unchanged. Each
candidate's accepted hi-hat direction updates the left knee plane, matching
adapter calibration. Final validation uses that same candidate body and kit.
The previous separate `fit_joint` recovery module has been removed.

One placement/certification pipeline has two deterministic work allocations. The
initial nominal search gets 12,000 candidates; nominal seat retries get 6,000
each within a 30,000 total allocation. The remaining request budget covers
body-anchored coarse shell families, 6,000 each and 24,000 per seat. Child quota
exhaustion does not close later sibling families. The total remains 80,000.
Up to three distinct validated snare anchors per pedal can be reconsidered when
later placement fails. This remains a bounded search, not a global optimizer.

The final certificate includes conventional instrument regions, lower-limb
envelopes across three pedal pitches, hand/body/equipment references, required
hand pairs and rigid hardware clearance. Both pedals also use the existing
55–155 degree knee range and reach annulus. This closes an incoming fast-path
gap: lower-limb collision sampling alone checked reach but not the knee range.
Equipment clearance remains 5 mm; hand/body posture clearance remains 2 mm.
The final snare-ready check uses the same refined posture search as hand/pair
certification, fixing a merge mismatch that could discard valid refined
witnesses or refit an already accepted setup.

The browser applies a successful seat and kit together before preparation.
Failed proposals retain the active layout. A failed physical mount change uses
`requireComplete` before adoption. Saved projects bypass automatic refitting and
retain their exact engine-hash contract.

## Investigation during integration

The first combined measured native/Studio run passed with a 60 mm upward seat
adjustment and 100% shell/cymbal sizes. Its repeated fit was unchanged; the two
independently authored studies had no unassigned hits. This did not establish
acceptance of other models.

The first public VRM cohort exhausted the 80,000-candidate bound on all three
models. The private VRM 0.0 and two FBX samples also remained incomplete.
Investigation found two separate search coverage issues: an original-layout
branch could consume a whole seat quota before compact shell families were
visited, and a greedily accepted snare could block later rack placement.
Hi-hat witness failures can include hand, support or lower-limb conditions;
their aggregate counter alone is not a measured penetration or proof of
physical impossibility.

Stand-phase search now rejects invariant shell/board intersections before
trying 24 rotations. This pruning uses the same lower-limb tests after removing
only the current stand's supports. Existing guards first require every detected
failure to belong to that piece. It does not assume that all later hardware
routing is invariant.

The native positive fixture was remeasured from the independently authored
application rig and procedural Studio. The old fixture held obsolete hand,
pedal, seat and layout measurements; it was not a current-adapter regression.
The refreshed fixture retains explicit success, lower-limb/layout clearance,
equipment posture, nominal cymbal and repeated-fit assertions. No imported
model data was committed.

## Final validation

Final WASM SHA-256:
`63df277ff770f9e05b85a3006c57d33318586c905037c89ca8f3c398217a9dad`.
The production preview on localhost:5180 serves the same binary. Its existing
preview process uses this worktree's rebuilt `dist`; the separate reference
checkout and localhost:5190 comparison were not changed.

Fresh Node/WASM public-adapter audit (`scripts/reference-layout-review.mjs`) and
private loader/adapter audit produced the following results. Times are observed
local runs, not an isolated performance benchmark.

| Input | Complete fit | Candidates | Observed fit time | Accepted setup |
| --- | --- | ---: | ---: | --- |
| Native + measured Studio | Yes | 26,938 | 13.0 s | Seat +60 mm Y; shell/cymbal size 100% |
| AvatarSample B | No | 80,000 | 19.7 s | Prior layout; zero seat shift |
| Seed-san | No | 80,000 | 29.2 s | Prior layout; zero seat shift |
| Twist | No | 80,000 | 19.9 s | Prior layout; zero seat shift |
| Private Jungle Pocket VRM 0.0 | No | 80,000 | 15.7 s | Prior layout; zero seat shift |
| Private Gold Ship FBX | No | 46,145 | 22.1 s | Prior layout; zero seat shift |
| Private Almond Eye FBX | No | 80,000 | 25.9 s | Prior layout; zero seat shift |

Native repeated fitting is complete and changes neither its accepted seat nor
kit. Both authored studies assign every arranged hit, preserve original source
events and accepted equipment, and reproduce ordered/shuffled poses and actual
adapter markers. Candidate measurement restores the previous setup and matches
subsequent application; lengths remain unchanged on all four public inputs.
The three failed VRMs do **not** pass motion coverage on the retained reference:
unassigned posture/interpretation study hits are B 44/68, Seed-san 26/20 and
Twist 44/68. The earlier smaller, hand-only fitting successes are not current
lower-limb/nominal-cymbal acceptance.

The private audit uses actual production loaders and semantic adapters with
texture decoding stubbed only for geometry tests. All three preserve bone
lengths, input fields, candidate restoration/application and nominal cymbals.
The failed Rust proposal may add a normalized optional `pedalHat.geometry`;
the browser uses its existing fallback kit instead of adopting that failed
proposal. No successful private fit or private-fit idempotence is claimed.
Private acceptance tests deliberately remain red because `complete` is false.

Baseline lower-limb reports include snare clearance for all six imported inputs,
plus hi-hat/floor-tom clearance on B and hi-hat clearance on the private VRM.
Search-stage traces are separate: B/private samples chiefly miss hi-hat
witnesses; Seed-san/Twist also reach snare/rack/ride placement. These are bounded
search results, not a proof that enlarging the budget would solve them.

Final checks: 182 Rust release tests and 43 web files / 312 tests pass, with no
ignored tests in the normal suites. Formatting, Clippy with warnings denied,
typecheck, production build and documentation checks pass. Regressions include
shared quota exhaustion, compact-family coverage, a synthetic refined-only
snare witness, both knee bounds, unchanged local measurement frames and
invariant-obstacle pruning. The existing bundle-size advisory remains.

The production browser completed native automatic fitting at 100%, prepared
and played the 53-hit Posture Study, and retained the accepted 79 cm / 14 cm
seat after MIDI preparation and seeking to five seconds. Side and stage views
were inspected; the browser warning/error log was empty. Two Analysis items
remain visible; this is not a claim of all-frame proxy/skin clearance. Local
proof is `.local/integration/native-browser-final.png` (not committed).

Raw public evidence is under `.local/reference-layout/integrated-final/` and
private evidence under `.local/integration/private-final/`. The private sources,
measurements and screenshots are excluded from Git. This integration is a
tested shared implementation with explicit cohort failures; #18's complete
imported-model fitting acceptance remains open.

## Limits

Static proxy clearance does not certify full motion paths or skin/clothing.
An incomplete bounded search is not proof that a model is damaged or physically
unplayable. Private assets, profiles and screenshots remain ignored in `.local/`.

Transaction tests exercise Worker, engine, preparation and rig ownership
boundaries; they do not establish every full `main.ts` UI rollback path. A
pre-existing project-restore limitation remains: after committing the restored
engine, a failure while adopting the scene/appearance/equipment cannot restore
the entire previous session. This ordering predates both integration parents
and was not changed in this batch.
