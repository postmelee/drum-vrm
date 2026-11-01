# Shaft count-in and motion-path follow-up

This record separates adopted follow-up work from rejected private experiments.
It does not establish publication or full natural-motion acceptance. Earlier
checkpoints remain in [elbow/grip revision evidence](elbow-grip-revision.md).

## Adopted physical shaft count-in

Planner v19 / prepared format 7 replaces tip-to-tip count-in with tangential
surface contact at each complete butt-to-tip shaft midpoint. The left stick
holds position and the right lifts 70 mm. Adjacent cues with no intervening hand
strike retain one held phrase. The ordinary source priority, event IDs, microsecond
times and velocities remain unchanged; no musical strike is invented or retimed.
Actual rendered shaft midpoints and axes produce measured surface markers. The
whole physical stick remains in collision checks without a trimmed terminal
exception. Cue entry/exit applies one deterministic orientation overlay, shared
by planning, replay and prepared validation. A 15-degree palm-down reserve is
clamped into the profile's existing roll interval; it does not relax the interval.

Native/B/Seed/Twist frozen Posture Study preparations retain 53/53 hits, unchanged
source/hit facts, layout, exact seek-order results and prepared restoration. All
six shaft contacts per rig resolve. The three original public VRM skeleton/tool
matrices match their frozen measured profiles; maximum actual shaft contact errors
are B 0.531 µm, Seed 0.472 µm and Twist 0.538 µm. These are measurement/contact
results, not full-motion acceptance.

The native Rust 120 Hz comparison finds no stick-pair or equipment penetration
inside the active count-in phrase on any of the four rigs. Elbow-fold samples below
the existing 55-degree bound change native 0→0, B 22→10, Seed 18→0 and Twist 32→7.
B/Twist still briefly reach about 50.0/52.7 degrees during entry/exit; the warnings
remain. Source records, prepared bytes and all 241 reverse-order samples remain
exact in the final native audit. This is sampled evidence, not continuous clearance.
Evidence: `.local/shaft-count-in-v19/{comparison.json,candidate-exact/}`.

The WASM Posture matrix under `.local/reference-layout/v19-posture-final/` uses
`b7f1bf813fd62695761e3bc1e4c9ff17da33d1ad229e52ebf3736cd5cf585111`.
A final custom-roll-interval clamp rebuild produces
`8e3737fd045d8ecbcc2aa380103f68896bee49cbc3f66f293ee8d5203fe862e0`.
The final binary restores all four results with identical prepared bytes and
zero pose difference at 1,561 samples per rig; actual rendered contact checks are
repeated with that binary. Evidence: `.local/shaft-count-in-v19/final-wasm-equivalence.json`
and `rendered-final/`. A one-off 12-second count-only comparison remains roughly
unchanged (native 0.56→0.52 s, B 16.95→16.33 s, Seed 9.32→9.39 s, Twist 11.91→12.26 s);
concurrent work makes these coarse observations, not a performance claim.

**All four full Posture Study runs still fail temporal tool/posture acceptance.**
The new count-in and static repair do not fix the outstanding tom/crash transitions.
The four full demo matrix and broader VRM 0.0/FBX motion validation are not expanded
before the short-window failures and owner review are resolved.

## Adopted stand-phase repair

The fitting stage now tries its existing 24 tripod orientations when the current
support obstructs another piece, even if structural legs already pass. Previously,
equipment rejection ran first and prevented this existing degree of freedom from
being used. The placement and shell stay fixed; the resulting hardware, legs and
actual tool approaches still require validation. Unrelated failures and invariant
shell collisions do not trigger more phase probes. Candidate budgets are unchanged.

Two focused regressions cover the actual repair call order, unchanged playing
surfaces, budget exhaustion/restoration and idempotence. Native/B/Seed/Twist frozen
successful fits remain complete with no geometry or structural-leg failures;
accepted proposals match the baseline exactly and add zero candidates. One native
run measured 575/572/706/806 ms before and 608/611/758/860 ms after while other work
was running. These already-clear cases do not call the changed selector; those
single timings do not establish a latency regression. Evidence:
`.local/stand-phase-v19-check/summary.json`.

## Rejected coordinated layout

A private B/Twist trial moved the seat 8 cm forward, rack row 5 cm forward and
nearby cymbals/hi-hat 8 cm forward while retaining floor tom and kick. Derived
snare advancement and existing tripod phases allowed all static checks to pass.
However, minimum thumb-up high-tom elevation barely improved: B 77.51→76.77°,
Twist 81.44→81.15°, and the count of valid thumb-up witnesses fell. The layout was
not adopted. Static clearance alone does not establish the requested relaxed arm.
Evidence: `.local/coordinated-seat-v19-audit/review.md`.

## Rejected private path experiments

The following trials use the v18 checkpoint
`2b7660a1d597099e4b84da96087288fa4cceca6b`, prepared format 7, and the frozen
Posture Study profiles/kit under `.local/reference-layout/v18-posture-final/`.
Its `{native,avatar-sample-b,twist}-posture-prepared.json` files own the baseline
motions; `inputs/*-input.json` own the measured profiles and kit. These trials
precede the separate shaft-count-in follow-up. **None was adopted in production.**
The private native Rust harnesses do not establish browser/WASM performance.

Whole-study measurements use 1,561 times from −0.5 through 12.5 seconds at 120 Hz.
Anatomy/palm counts count individual hands; physical-tool counts count frames with
penetration below −0.01 mm, matching the existing diagnostic threshold. Intentional
count-in contact is excluded only from stick-pair penetration counts. Graph trials
keep the existing 33×33 states, eight refinement passes and inspection times.
Native graph regeneration has small floating-point/tie differences from the stored
WASM track; comparisons below use each experiment's matched baseline, not mixed
counts from different executables.

### Absolute elevation preference

Replacing only the deadband-excess elevation term with absolute elevation retained
its 0.22 coefficient, 0.39 cap and all hard bounds. Native high-tom elevation fell
36.2→30.9°, but B rose 74.8→93.6° and Twist rose 70.9→97.4°. Physical-tool failing
frames increased native 1→15, B 16→31 and Twist 11→28. All 53 hits per model remained
assigned; source facts, contact and prepared restoration remained intact.
The shared cost also changes earlier contact/path choices, and its cap saturates
at about 76.3° elevation, or 68° with the 0.08 palm-down cost. This is not a useful
isolated elbow-priority repair. Evidence: `.local/elevation-ablation-v18/summary.json`
and its `source-identity.json`.

### Preparation-only orientation continuation

Each existing swivel state could retain its preceding actual orientation at the
new target only after anatomy, palm and whole-tool acceptance, and only when its
existing comfort plus actual-motion cost beat the fresh static witness. No runtime
solver or new graph dimension was added. Native tool failures improved 1→0, but
anatomy failures rose 195→222 and palm failures 397→573. B tool failures rose 16→19,
with worst penetration 2.93→4.46 mm. Twist changed 11→10 failing frames but worst
penetration increased 10.33→10.52 mm. Peak native/B tool rates barely changed:
47.4→46.9 and 92.0→92.0 rad/s. Greedily replacing one witness per state did not
preserve a satisfactory continuous family. Evidence: `.local/continuation-v18/summary.json`.

### Shaft direction and scalar palm-roll interpolation

Interpolating body-relative shaft direction and scalar palm roll preserved selected
endpoint wrists/elbows within 9×10⁻¹⁶ m and contact within 2.5×10⁻¹⁶ m. Reusing the
existing graph with this interpolation reduced palm failures to zero for all three
models. However, anatomy failures increased native 195→338, B 225→525 and Twist
324→527; tool failing frames changed 1→0, 16→28 and 11→29. B's worst penetration
increased 2.93→12.83 mm. Replacing the shaft curve discarded useful approach-relative
motion even though the palm interval stayed valid. No representation/format change
was adopted. Evidence: `.local/semantic-graph-v18/summary.json`.

The narrower native lower-elbow crash experiment also remained invalid: at
10.3–10.5 seconds, palm failures fell from 22/25 samples to zero, but two samples
exceeded the 34.38° wrist bound, reaching 34.91°. Same-smoothstep world-quaternion
SLERP retained 22 palm failures and the same two wrist failures. Evidence:
`.local/semantic-interpolation-v18/native.json`.

### Existing solver with adjacent-edge acceptance

Eight requests tested B and Twist endpoints at 6.4, 6.5, 10.4 and 10.5 seconds,
holding time, target, pitch and swivel fixed. The existing static solver's acceptance
predicate additionally checked both adjacent intervals using existing inspection
times; it evaluated 4–28 existing candidates per request. None passed, so every
candidate replacement was rejected and the original tracks remained unchanged.
B's best 6.5-second alternative reduced 11 failed samples to four but retained
41–46° wrists. Twist's best 6.4-second alternative reduced six failures to one but
retained a 1.49 mm stick-pair penetration. Both models' 10.4-second alternatives
reduced 15/17 failures to three, which still violated palm policy near impact.
Evidence: `.local/edge-witness-v18/{avatar-sample-b,twist}.json`.

## Remaining path scope

These bounded failures are not proof that no feasible global motion exists. They
show that separately fixing tip/pitch routing and then retaining one static hand
witness per swivel can lose a usable transition. The next scope should couple
path generation with actual tool orientation and its arm witness: first certify a
short preparation→impact→rebound→next-target segment, then retain that segment's
compatible witnesses through selection and replay. Check actual wrist/upper-arm
change, whole-tool/pair clearance, contact and seek/restore together. Preserve
original MIDI times and bone lengths; do not substitute further weight tuning,
extra grids or relaxed gates for that segment-level evidence. Keep owner visual
review before broader VRM 0.0/FBX validation or any new publication approval.

## Executed checks and local review

- Final source: 274 Rust tests pass. All 336 web tests pass across the full run
  plus the rerun of the temporary-debug-message failure after that string was
  removed. Warnings-denied Clippy, Rust format, typecheck, production WASM/web
  builds, docs structure and whitespace checks pass.
- Browser captures use the rebuilt local preview at
  `http://127.0.0.1:5195/drum-vrm/?lang=ko`. Native before/after and public-model
  count-in contact/rebound views are stored privately under
  `.local/shaft-count-in-v19/visual/`. The visible center crossing replaces the
  former tip meeting; these limited views do not certify uninterrupted natural
  movement for a whole song.
- Keep PR #65 draft and #18/#23 open. Owner review should focus first on the new
  held-left/right-tap count-in. Full tom/crash motion acceptance, all-demo expansion
  and VRM 0.0/FBX handoff remain subsequent work. No merge or Pages deployment was
  performed in this increment.
