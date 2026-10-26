# Measured body clearance and temporal posture verification

Measured locally on 2026-10-26 for the body-clearance workflow candidate. Runtime
hashes identify the tested working-tree builds; this report does not establish a
release or what GitHub Pages runs. The [calibration contract](../../specs/architecture/calibration.md)
and [rig contract](../../specs/architecture/rig-contract.md) own current behavior.
The [earlier browser report](browser-natural-performance.md) used different,
skeleton-derived proxies and is not a directly comparable skin-clearance baseline.

## Candidates, inputs and scope

| Candidate | WASM SHA-256 |
| --- | --- |
| A: measured geometry and static pair fitting, before temporal refinement | `87438bfd68936b35d8ae33ee44170ebb76f51c1d931849551e91955db78d636b` |
| B: temporal elbow coordination with impact knots and joint torso scoring | `a89f565d84e2758214114e86147f9b5b84f366fc2048b504f06d21405e5b7b0d` |

Environment: macOS, Node 24.15.0 and pinned repository dependencies. Twelve cases
combine the native rig, VRM Constraint Twist Sample, Seed-san and AvatarSample B
with the authored Studio demo (193 strikes), authored interpretation demo (82),
and owner-provided benchmark (1,720). See [model sources and permissions](../research/vrm-models.md).
Uploaded models and MIDI, extracted geometry, and song-event sequences are not
public fixtures or redistributed with this report.

The benchmark SHA-256 is
`615127ef5d0bfe7c4e934222349243e8d7c9c8309fa9e1a56e0b19bb0e41f5cf`.
It uses Logic DKD GM mapping, explicit count-in interpretation, short-open-close
hi-hat controls at 20%, and the owner's approved visual omission of source event
`23:237`. The source event remains preserved. No filename or avatar-name rules
were introduced.

The harness loads the actual GLB buffers, skin weights, morph targets, semantic
humanoid hierarchy and node constraints. It removes only material/texture/image
references in memory for headless loading; original files remain unchanged.
Geometry is measured after the production matched grip has reached the raw skin
bones. Torso and limb sampling is bounded at 12,000 candidate vertices per model.
The accepted regional samples are 1,112 for Twist, 5,911 for Seed-san and 1,667 for
AvatarSample B; the native rig explicitly uses skeletal fallback geometry.
These are percentile-based estimates, not enclosing mesh certificates. Clothing
weighted to the same bones contributes, and sparsely measured regions can fall
back to skeletal estimates.

All cases use automatic seat calibration, palm-up orientation and the same
body-informed kit fitting in A and B. Fitting checks 16 hand/target combinations
and 64 ordered target pairs. Available pairs mean a sampled static witness was
found; unavailable pairs and unsampled trajectories are not certified safe.

## Preserved behavior and computational cost

Across all 12 B cases, every strike remains assigned, original source arrays and
strike times/ticks/velocities are unchanged, and shuffled seek evaluation returns
identical primary poses and rendered contact markers. Maximum contact error is
0.435 µm, rendered-marker error 0.446 µm and bone-length error `6.53e-8 m`.
No bone stretching or timestamp displacement is used to hide path problems.

| Rig | Kit scale | Available static pairs | Fit cost B | Benchmark planning A → B |
| --- | --- | --- | --- | --- |
| Native | 94% | 48/64 | 0.17 s | 0.76 → 5.94 s |
| Twist | 74% | 23/64 | 5.72 s | 0.89 → 7.20 s |
| Seed-san | 86% | 45/64 | 1.46 s | 0.76 → 7.12 s |
| AvatarSample B | 82% | 39/64 | 4.42 s | 0.85 → 6.91 s |

Fitting and planning are measured separately in Node. Identical profile/kit fit
results are reused between the three songs in the private audit; this does not
claim an app-wide persistent fit cache. These timings are not browser FPS or an
end-to-end import benchmark. Temporal planning adds several seconds of work for
the 193.026-second benchmark; UI responsiveness requires keeping that work off the
main thread.

## Sampled path conflicts, separate from impact conflicts

The full score is evaluated at 60 Hz from −0.5 seconds through duration +0.25.
Rendered markers are additionally checked at 10 Hz and every assigned impact.
Negative counts below are overlapping **proxy samples**, not distinct incidents,
confirmed mesh penetrations, or milliseconds of visible interference.

A → B, 60 Hz samples. “Tool/arm” tests each stick against the opposite arm/hand.

| Rig / score | Torso | Arm pair | Stick pair | Tool/arm |
| --- | --- | --- | --- | --- |
| Native / Studio | 23 → 0 | 0 → 0 | 9 → 9 | 0 → 0 |
| Native / Interpretation | 5 → 0 | 0 → 0 | 4 → 4 | 0 → 0 |
| Native / Benchmark | 156 → 17 | 0 → 0 | 173 → 178 | 6 → 5 |
| Twist / Studio | 44 → 27 | 18 → 18 | 0 → 0 | 24 → 24 |
| Twist / Interpretation | 18 → 15 | 10 → 9 | 1 → 1 | 0 → 0 |
| Twist / Benchmark | 549 → 183 | 198 → 194 | 118 → 125 | 49 → 46 |
| Seed-san / Studio | 67 → 27 | 42 → 27 | 8 → 3 | 27 → 18 |
| Seed-san / Interpretation | 38 → 11 | 0 → 0 | 15 → 15 | 0 → 0 |
| Seed-san / Benchmark | 1003 → 165 | 37 → 11 | 126 → 120 | 6 → 6 |
| AvatarSample B / Studio | 80 → 21 | 6 → 0 | 0 → 0 | 0 → 0 |
| AvatarSample B / Interpretation | 50 → 92 | 114 → 99 | 15 → 9 | 0 → 0 |
| AvatarSample B / Benchmark | 1692 → 719 | 1452 → 854 | 112 → 112 | 5 → 0 |

A → B, exact assigned-hit probes. Simultaneous hits repeat the same full-pose
inspection, so these counts are not unique timestamps and need not match the
app's aggregate impact-time diagnostic count.

| Rig / score | Torso | Arm pair | Stick pair | Tool/arm |
| --- | --- | --- | --- | --- |
| Native / Studio | 0 → 0 | 0 → 0 | 0 → 0 | 0 → 0 |
| Native / Interpretation | 0 → 0 | 0 → 0 | 0 → 0 | 0 → 0 |
| Native / Benchmark | 27 → 0 | 0 → 0 | 15 → 12 | 0 → 0 |
| Twist / Studio | 9 → 6 | 6 → 6 | 0 → 0 | 0 → 0 |
| Twist / Interpretation | 2 → 2 | 1 → 1 | 0 → 0 | 0 → 0 |
| Twist / Benchmark | 119 → 18 | 7 → 5 | 7 → 7 | 2 → 2 |
| Seed-san / Studio | 6 → 0 | 3 → 3 | 0 → 0 | 0 → 0 |
| Seed-san / Interpretation | 6 → 1 | 0 → 0 | 0 → 0 | 0 → 0 |
| Seed-san / Benchmark | 186 → 3 | 8 → 0 | 7 → 7 | 0 → 0 |
| AvatarSample B / Studio | 4 → 0 | 0 → 0 | 0 → 0 | 0 → 0 |
| AvatarSample B / Interpretation | 3 → 18 | 24 → 17 | 0 → 0 | 0 → 0 |
| AvatarSample B / Benchmark | 378 → 160 | 354 → 193 | 7 → 7 | 2 → 0 |

Worst exact-impact overlap depth in millimeters; zero means no negative probe.

| Rig / benchmark | Torso A → B | Arm pair A → B |
| --- | --- | --- |
| Native | 10.34 → 0.00 | 0.00 → 0.00 |
| Twist | 39.03 → 12.23 | 7.85 → 11.49 |
| Seed-san | 65.35 → 22.80 | 6.03 → 0.00 |
| AvatarSample B | 112.61 → 17.59 | 73.86 → 39.24 |

B substantially reduces body overlap but is not a universal collision-regression
pass. Inter-stick samples and some intermediate transfers remain problematic.
The engine reports unresolved impact proxy conflicts separately from a completed
static layout fit; zero unassigned strikes does not mean collision-free motion.

## Posed surface checks

Independent geometric probes evaluate posed skinned triangles in the app's
third-person layers (0 and 10), excluding first-person duplicate geometry. They
look for triangle-edge/surface crossings between semantic torso and distal arms/
hands, and between the two distal arms/hands. Sleeves contribute; mixed semantic
boundary faces are excluded. A detected crossing is evidence of intersecting
surfaces, but zero detections do not prove volume separation or coverage between
sampled frames. Counts are intersecting face pairs, not penetration depth.

The four former worst-impact frames use the same source targets and fitted
geometry in both candidates:

| Model / benchmark time | Surface pair | A crossings | B crossings |
| --- | --- | --- | --- |
| AvatarSample B / 92.578202 s | Left distal arm–torso | 51 | 0 |
| AvatarSample B / 102.824109 s | Opposite distal arms/hands | 931 | 0 |
| Twist / 100.856895 s | Right distal arm–torso | 14 | 0 |
| Seed-san / 31.628242 s | Left / right distal arm–torso | 12 / 440 | 0 / 0 |

Eight additional B probes cover new worst impact/path samples: AvatarSample B at
74.236081, 92.250333, 92.133333 and 92.55 seconds; Twist at 102.660174 and 61.666667;
Seed-san at 102.660174 and 92.65. Seven show no detected distal-surface crossings;
AvatarSample B at 92.55 retains four opposing-arm face-pair crossings. This is
bounded evidence of improvement, not a full-song mesh-clearance pass.

Including upper arms exposes additional shoulder/sleeve intersections. The
standalone fitted snare-ready pose already has 44 right-shoulder face pairs for
AvatarSample B and 34 left-shoulder pairs for Seed-san, while distal checks are
clear on all three models. Whole-arm counts therefore mix connected attachment/
garment overlap with motion-induced deformation. They must not be relabeled as
verified skin penetration or ignored as automatically harmless. The simple
proxies remain useful estimates; neither shrinking them to erase warnings nor
claiming universal clothing clearance is justified by these probes.

## Final bounded worker-transfer candidate

Candidate C uses WASM SHA-256
`36f3fb5ce7aa83096ca6ba4d198c1916483339d4c54fe4a1ce9fcfc919fbae21`.
It adds validated prepared-runtime transfer and a 6,000-control-point allocation
budget. Exact JSON float parsing also changes some near-tied temporal choices
relative to B; the following final measurements do not overwrite B's results.

All 12 cases again preserve source/timing, assign every strike and reproduce
shuffled primary poses and rendered markers exactly. Prepared transfer preserves
the plan JSON exactly and produces zero pose or numeric-inspection differences
at every 60 Hz sample and assigned-hit probe. Maximum contact, rendered-marker
and bone errors remain 0.435 µm, 0.446 µm and `6.53e-8 m`, respectively.

Encoding takes 0.53–9.71 ms, validated decoding 0.93–15.37 ms, and payloads contain
74,982–1,459,708 bytes in this local audit. This is the internal worker handoff,
not a persisted project format. It avoids repeating temporal planning on the
main thread; actual browser responsiveness remains a separate check.

Final negative probe counts use the order **torso / arm pair / stick pair /
tool-versus-opposite-arm**. Path and assigned-hit counts remain separate.

| Rig / score | 60 Hz path samples | Assigned-hit probes |
| --- | --- | --- |
| Native / Studio | 0 / 0 / 9 / 0 | 0 / 0 / 0 / 0 |
| Native / Interpretation | 0 / 0 / 4 / 0 | 0 / 0 / 0 / 0 |
| Native / Benchmark | 17 / 0 / 178 / 5 | 0 / 0 / 12 / 0 |
| Twist / Studio | 27 / 18 / 0 / 24 | 6 / 6 / 0 / 0 |
| Twist / Interpretation | 15 / 9 / 1 / 0 | 2 / 1 / 0 / 0 |
| Twist / Benchmark | 183 / 194 / 125 / 46 | 18 / 5 / 7 / 2 |
| Seed-san / Studio | 27 / 27 / 3 / 18 | 0 / 3 / 0 / 0 |
| Seed-san / Interpretation | 11 / 0 / 15 / 0 | 1 / 0 / 0 / 0 |
| Seed-san / Benchmark | 165 / 11 / 120 / 6 | 3 / 0 / 7 / 0 |
| AvatarSample B / Studio | 21 / 0 / 0 / 0 | 0 / 0 / 0 / 0 |
| AvatarSample B / Interpretation | 92 / 99 / 9 / 0 | 18 / 17 / 0 / 0 |
| AvatarSample B / Benchmark | 714 / 851 / 112 / 0 | 161 / 194 / 7 / 0 |

Candidate C repeats the four former-worst surface checks with zero detected
distal crossings at all four frames. Repeating the eight residual probes gives
zero detections at seven frames; AvatarSample B at 92.55 seconds retains 29
opposing-arm face-pair crossings (B had four). Its minimum sampled arm proxy
clearance is −60.52 mm at that frame, compared with B's −47.79 mm. This localized
change follows exact-float parsing and a different near-tied temporal path;
C therefore retains an explicit fast-transfer limitation despite the broad
improvement over A. The upper-arm/shoulder caveats above still apply.

## Verification boundary

No random strike variation, bone stretching, learned motion model, cloth physics
or video-output validation is included. Source timing and deterministic contact
are verified separately from natural posture and surface clearance. Public model
licenses remain separate from code licenses. The final prepared-runtime audit above verifies the internal numerical handoff;
browser workflow and repository/deployment checks are separate evidence below.

## Browser and repository checks

Local IAB Chromium smoke checks used the production server at
`http://127.0.0.1:5194/drum-vrm/`, including the final C build. At a 745 × 833
viewport, the five menus, collapsed advanced controls, light/dark themes,
front/player cameras, drum presets and Analysis remained usable. This is a
bounded desktop/narrow-panel check, not mobile or cross-browser certification.

- VRM-first import automatically fitted Twist without proposal/apply steps;
  cancelling a repeated fit retained the character, and repeating it did not
  compound kit shrinking. MIDI-first Studio then AvatarSample B import retained
  the 193-strike score and the loaded WAV's +0.25-second offset.
- A saved project reopened both in-session and after a fresh reload, requiring
  MIDI/WAV/VRM relinking and retaining its setup without automatic refitting.
  These workflow checks preceded the final allocation guard; final C's numerical
  restore and prepared transfer were independently rechecked above.
- Final C loaded the Studio demo, assigned all 193 strikes, played/paused and
  exposed static-pair and motion diagnostics in Analysis. Invalid GLB input
  produced a visible error while preserving the paused score and current rig.
  Escape closed the panel and returned focus to its originating menu.
- No production-preview console errors were observed. Earlier development HMR
  errors were distinguished by their old development URL/time; they were not
  treated as final-build errors or silently erased.
- Final repository checks passed: 93 Rust tests, Rust formatting and strict
  clippy, 174 browser-side tests in 25 files, TypeScript and production build,
  generated-license consistency, and documentation checks (83 Markdown files,
  21 public language pairs). Browser checks do not replace the remaining surface
  clearance acceptance in #18/#20/#23.

The final review separately checked Worker cancellation/generation guards,
transactional setup replacement, the planning-budget error, and prepared payload
validation. A different MIDI intentionally clears the previous score; preserving
an active evaluator on failure applies to setup/replan/model replacement.
