# Elbow motion and remaining scope audit

## Measured candidate

2026-10-29 follow-up to the owner's elbow-motion report, on source
`2640f652698e045b27b327261b12ca18951d1fcb` (draft PR #65), with WASM SHA-256
`22f94f2ca45d38071e55a17ba0a3c977ba5af133500e330851f4bd9df9dcff8e`.
This is an investigation, **not an implemented elbow correction**. The two
solver experiments below were rejected and restored. The local preview and
published app were not replaced with those experiments.

The controlled numeric replay reused the exact fitted profiles and kits from
the [final v13 review](common-motion-fit.md#prepared-roll-review-candidate--final-v13-boundary),
without running the fitter again. It sampled primary poses at 60 Hz plus exact
hits, and compared reverse-order replay and prepared-message restoration.
All four rigs retained 53/53 Posture and 82/82 Interpretation assignments;
actual tip errors remained below 0.000061 mm, with identical reverse/restored
poses. These checks do not establish natural motion or clearance. All eight
runs still emit temporal posture/tool failures.

Private reproducibility artifacts: `.local/elbow-revision/audit.mjs`,
`before.wasm`, `before/summary.json`, per-study prepared messages and
`visual/`; additional isolated pole probes are in
`.local/reference-layout/elbow-regression-audit/`. These contain derived local
measurement data, not newly authorized public model assets.

## Confirmed failure mechanism

For bounded palm policies, `hand_solver::solve_wrist` replaces the transported
body-relative elbow pole with a projection of the **rotated hand's** neutral
forearm. Thus palm roll rotates the reference elbow plane. The coordinator's
relative swivel bounds do not bound the resulting anatomical elbow-plane
change. Preparing roll values removed pointwise optimizer jumps, but did not
make this coupled motion natural.

- Twist Interpretation at 0.9333333333 s: right upper-arm elevation 114.458°,
  wrist deviation 25.157°. The preferred pole is −132.661° from the anatomical
  reference and the actual elbow plane −117.700°, although relative swivel is
  only +14.961°.
- Twist Posture from 7.100 to 7.125 s: left prepared roll changes from
  −1.0425565 rad to zero (59.733° in 25 ms). Relative swivel changes only
  −0.50 to −0.55 rad. The browser reproduces the abrupt change during tom hits.
- A centered 1/240 s finite-difference probe over the sampled timeline estimates
  peak elbow speeds of 20.50 m/s (Twist Posture) and 22.57 m/s (Interpretation).
  These are local numerical indicators, not certified continuous maxima or
  physiological acceptance thresholds.

Two minimal replacements were tested, then **reverted**:

1. Always use the transported anatomical pole: 16/19 focused tests passed;
   previously valid native static witnesses and a recovery witness failed.
2. Use the unrolled approach's pole: 18/20 focused tests passed, including an
   extra investigative test; an existing anatomical gate still failed.

Neither experiment changed thresholds or accepted a failing witness. A separate
read-only native grip experiment reduced the fixed shaft/hand angle from 45°
to 30° and 20°. It did not improve readiness: the fitter moved the seat back
8 cm, and the 20° case reduced available reference pairs from 32/64 to 25/64.
This does not support changing the grip mount as a standalone fix.

## Layout proposals

Re-measured Three.js geometry was checked through the existing WASM geometry
and fitting queries. These are native static probes, not three-VRM acceptance.

| Candidate | Result |
| --- | --- |
| Current rack tilt 12°, left crash z=0.69 m | No unresolved hardware; snare/high-tom proxy separation 6.947 mm |
| Left crash z=0.65 m, 4 cm closer | Static hardware remains valid, but native fit moves the seat back 8 cm |
| Rack tilt 16°, positions unchanged | Snare/high-tom separation 0.510 mm, below the existing 5 mm margin |
| Rack tilt 20°, positions unchanged | Snare/high-tom proxy overlap 4.646 mm |

The 2 cm crash approach also moved the native seat back 8 cm. Therefore the
closer crash is a useful candidate to retry after elbow coordination, not a
verified new fixed startup. Increasing rack tilt alone is not a safe correction
for the stick path. No new default coordinates or tilt were adopted here.

## Actual status of the four requested steps

1. **Reference layout: partial.** Player-facing 12° tom tilt, lower right crash
   and floor-leg rotation exist. Left crash approached only 1 cm in PR #65;
   further approach and final multi-view acceptance remain open. The shared
   T-yoke/ball-joint/L-arm support was retained, not replaced with a T-only bar.
2. **Fast-fill assignment: partial.** Opposite-hand contact feasibility and
   two authored regressions were added. `sticking.rs` and `phrase.rs` pattern
   logic/costs are unchanged; per-hit hand/pattern choice explanations in
   Analysis are not implemented. Paradiddles are not forced into every fill.
3. **Stick paths and tom-transfer posture: incomplete.** The confirmed elbow
   failure and prior tool/posture failures keep #18/#23 acceptance open.
4. **Full songs and additional VRM 0.0/FBX: incomplete.** Import support is
   separate from full-song natural-motion acceptance; broader review is deferred.

The next implementation should retain the valid static contact witnesses while
coordinating and interpolating the **actual anatomical elbow plane**, together
with grip roll, on the existing bounded graph. It must handle disconnected paths
explicitly rather than selecting an infinite-cost/default-parent path. The
current 2 rad/s limit is a knot-difference bound; smoothstep interpolation can
reach 1.5 times that secant rate. Any new continuous-rate claim must account for
that distinction. Verify the reproduced intervals first, then the same frozen
four-rig inputs, before changing layout, expanding model coverage or claiming
natural-motion acceptance. No new fitting grid or relaxed physical gate is
justified by this audit.
