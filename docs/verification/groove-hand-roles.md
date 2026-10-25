# Groove-hand ownership and post-fill crash verification

Assessment: 2026-10-26. Baseline: deployed `045b78a`. Candidate: planner
`phrase-beam-8-beat-horizon-24-v4`, local WASM SHA-256
`034ac279dd85cd882bdd328cbdc5c6190cc03a78e2b57814b11fda91e32f7ce4`.
These measurements describe the implementation candidate. Its later approved Pages publication is recorded separately in the [deployment evidence](pages-browser-performance.md#groove-hand-correction-follow-up).

## Cause and boundary

The real browser native rig, before fitting, assigned 77 of the Studio demo's 104
hi-hat strikes to the left hand. Its right-hand contact's static torso-proxy cost
was about 2.046, exceeding the 0.65 wrong-hand penalty. The left hand consequently
traveled between hat and snare; the existing ready-pose logic followed that assignment.
This was not a camera mirroring or left/right label error. Fitting masked the issue.

The fix ranks feasible event coverage, then default groove ownership, then movement
costs. It releases crash ownership after a head-led fill and leaves rolls/doubles
free to use both hands. Explicit source-event hand edits and feasibility constraints
remain authoritative. The [engine contract](../../specs/architecture/engine-contract.md)
owns these rules; [instructional rationale](../research/ergonomics-and-sticking.md#groove-ownership-and-post-fill-crashes)
distinguishes Yamaha's crossed groove and Zildjian's left-hand post-fill crash example.

## Results

Both independently authored browser fixtures were evaluated through production WASM
and `NativeHumanoidRigAdapter`, before and after explicit kit fitting.

| Check | Observed result |
| --- | --- |
| Studio hi-hat | All 100 closed and 4 open hits use RH, before and after fitting |
| Studio snare | All 33 use LH; sparse backbeats are explicitly regression-tested |
| Verification pattern | All 12 closed and 6 open hats use RH; solo snare roll still uses both hands |
| Studio fill exit at song 8/16/24 s | RH floor tom → LH left crash → RH hat at +0.25 s |
| Idle LH during groove | Tip stays 3.5 cm above the snare at sampled idle times; shuffled seeks match |
| Event preservation | Original IDs, timing, velocity, physical pieces and intended contacts preserved; zero unassigned in all four fixture/setup combinations |
| Explicit hand override | A source-addressed LH hat edit remains LH |

At fitted Studio fill boundaries (7.5–8.5, 15.5–16.5, 23.5–24.5 song seconds),
120 Hz samples had minimum proxy clearances of 8.63 mm torso, 154 mm arms and
13.03 mm sticks. Maximum impact error was approximately 4.16×10⁻⁸ m. These are
bounded fixture measurements, not arbitrary-avatar collision certification.

The default **unfitted** first 6.5 seconds still had 11.23 mm torso-proxy penetration.
Correct hand ownership does not resolve the remaining body/kit and path-clearance
work in #18/#20/#23. The eight-candidate search is not a global optimum guarantee.

Validation: 80 Rust tests, 142 web tests, Rust fmt/clippy, rebuilt WASM, TypeScript
and production web build passed. Browser inspection used the local app, authored
Studio demo, player/top views and seeks. Preview includes 0.5 seconds of preparation:
the song's 8-second crash is at output time 8.5 seconds. No private input/model or
external instructional media is included in the regression fixtures.
