> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/research/snare-ready-and-kit-fit.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../../specs/README.md) for current contracts and policy.

# Snare ready pose and body-to-kit fit assessment

Date: 2026-10-24. Code assessed: `1e79532ea8c1d1e387ed052511718cd43cdf4e41` (open PR #34). This is an engineering assessment and proposed follow-up, not implemented behavior or a new release approval. Related: #18 (body/kit calibration), #20 (stroke/style), #14 (editing/hardware).

## Current evidence

The owner's screenshots show a repeated departure from the snare and apparent body/kit disproportion. Perspective alone cannot measure model size or prove the exact pose at a given event. The code independently explains the repeat-return behavior:

- `crates/drum-engine/src/motion.rs::stroke_at` connects adjacent hits directly only when the same limb's interval is at most 0.42 s. Otherwise it returns to `profile.rest_tips` over 0.18 s and begins the next approach 0.22 s before contact. It does not distinguish repeated snare notes from different-instrument transitions.
- The normal 120 BPM demo backbeat has snare notes 1 s apart. Unless intervening events assigned to that hand alter its interval, each backbeat therefore returns to the generic pose.
- `src/rig-humanoid.ts` derives the reference wrist from measured arm reach (78% of total length) along a fixed direction, not from the snare/hi-hat. Seat adjustment translates that reference; it does not make it instrument-specific.
- The hand tip arc reaches up to 0.24 m, scaled by 35–100% of MIDI velocity. The main wrist pitch envelope is independently multiplied by a fixed 0.70 rad. Wrist-first motion with bounded forearm contribution is not implemented.
- Palm-direction calibration improved reference alignment, but the [measured dynamic fixture](../../../report/verification/palm-orientation-2026-10-24.md) still has 99–110° left-hi-hat hand/forearm angles. Exact rendered contact and deterministic seeking do not establish natural posture.

## Size and fit are separate from the standard rig

The shared VRM 1.0 semantic hierarchy does not impose a fixed character height. Actual upper/lower arm and leg lengths are measured and preserved. No avatar-name-specific motion branches are needed.

The default kit has fixed world-space positions and radii (`src/scene.ts::DEFAULT_KIT`, matching the engine defaults). For example, snare radius 0.22 m means a nominal 0.44 m diameter, with a contact height of 0.79 m. Sticks currently have a fixed 0.31 m forward tip offset plus 0.105 m rear shaft. These dimensions need a coherent physical-size baseline before fitting very different bodies.

Automatic seat calibration considers leg reach to the pedals; it does not optimize arm reach, shoulder elevation, wrist bends, inter-instrument clearance or a complete seated playing posture. Manual seat and contact XYZ controls already exist. The 75–125% piece scale changes the body mesh around the existing contact; it does not move musical targets, fit kit spacing or reposition shared hardware. It is not a body-to-kit fit solution.

## Recommended sequence

1. **#18: establish a measured fit baseline.** Audit nominal drum/stick dimensions and record seated shoulder height, arm/leg segments, grip offset and both pedal anchors. Use the existing seat/XYZ controls to establish reachable reference fixtures; then implement a bounded common fit from these measurements. Keep body dimensions first and adjust seat, snare/hat height and instrument distances. Do not infer all proportions from standing height or silently stretch bones.
2. **#20: implement phrase-aware ready points and a joint-aware stroke together.** In a repeated snare phrase, keep the stick tip slightly above its contact plane between hits. Derive the wrist behind the tip from the actual grip/stick. Do not lock the palm over the drum center. Separate same-instrument rebound from transfers to toms, cymbals or another role. Use a long-rest policy only when the musical context warrants it. Solve target-dependent tool orientation, bounded wrist bend and velocity-dependent wrist/forearm contribution together; moving only the generic rest point leaves the known dynamic wrist problem.
3. **#14/#20: explicit playing style and clearance.** Make right-hand hi-hat / left-hand snare a selectable crossed style with a documented infeasibility fallback. Preserve open/mirrored alternatives; verify simultaneous-hit forearm/stick clearance. Existing cost preference is not this feature.
4. **#18/#14: compact kit controls if ordinary placement cannot fit.** Separate layout width/depth/height from physical shell/cymbal diameter. An explicit compact-fit action may change both, with bounded overlap and reach checks. Recompute actual targets, stands and pedal anchors atomically. Body-only model changes continue to preserve supports; an explicit layout edit may reposition or resize those same supports. Do not use a mesh-only scene scale as physical calibration.

Perform shared calculations from semantic joints, measured lengths, target frames and numeric style parameters. Keep Rust planning/evaluation responsible for the motion and immutable MIDI events; keep the adapter responsible for model-space conversion and actual attached tools. Full learned motion and more presets are lower priority than this baseline. Broader M3/M4 work is not started by this assessment.

## Proposed completion checks

- Same-hand snare repetitions at 0.5 s and 1 s intervals stay near the snare during the phrase; a long rest and a snare-to-tom-to-crash fill have intentional, continuous transitions.
- Low/high velocity cases show separately measured wrist and forearm contribution; a ghost note does not inherit the full common arm arc. Velocity is an animation input, not a direct physical-force measurement.
- Seat and snare XYZ edits update ready points, tool orientation, reachability and attached rendered contact together.
- Compare at least the three existing private VRMs and the still-pending controlled pair of VRoid Studio exports. Record hand/forearm angles, elbow/shoulder travel, contact error, foot anchors, clearances and unassigned events; contact accuracy alone is insufficient.
- Preserve bone lengths, original event identity/time/velocity and sound, including explicitly diagnosed infeasible hits. No avatar-specific exceptions or detached contact markers.
- Verify the same pose at each timestamp during sequential playback, arbitrary seeking and video-frame evaluation. Independently play a new encoded file before claiming the whole output path verified.

## Primary instructional references

- [Yamaha DTX402 Drum Score / Instruction, printed pp. 4–7](https://usa.yamaha.com/files/download/other_assets/7/1224887/DTX_DrumScore_Instruction_eng.pdf): a basic ready position holds stick tips 2–3 cm above the head; describes forearm alignment, rebound and a right-hand-hi-hat/left-hand-snare starting position. Use the clearance as an adjustable starting point, not a universal technique constraint.
- [Dr. Dave Gerhart / Yamaha: Snare Drum Basics](https://hub.yamaha.com/music-educators/instruments/perc/snare-drum-basics-pedagogy/): establish a comfortable arm position and adjust drum height to it; basic wrist stroke instruction. This is instructional evidence, not a validated runtime fitting algorithm.

Accessed 2026-10-24. These are copyrighted instructional references, not MIT motion/code/model assets. Link and summarize only; no images, motion data or source material are bundled. The proposed algorithms above are independent engineering choices, not a claim to reproduce a research model or measured biomechanics.
