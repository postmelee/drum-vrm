> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/decisions/0006-technique-and-expression-foundation.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../README.md) for current contracts and policy.

# ADR 0006 — Technique and constrained performance expression

Date: 2026-10-24. Status: research/design foundation documented; runtime schema and controls **proposed, not implemented**. Extends [ADR 0005](0005-calibration-pedals-hardware.md). Source IDs and rights: [public EN](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/performance-foundations.md) / [KO](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/performance-foundations.ko.md). Issues: #18, #20, #14, #23, #15; learned models remain #21.

## Decision and evidence boundary

Use instructional sources as traceable starting points for independent, testable motion rules. Keep musical intent, physical calibration, playing technique and visual expression separate. Instruction is not motion-capture ground truth, a physiological joint-limit specification, or an implemented algorithm. The owner proposed an expression control ranging from economical playing to large phrase accents. That control is our design, not a feature described by Yamaha.

I01 fundamentals (printed pp. 4–7) are the first instructional baseline; lesson-specific examples qualify it. I02/I03 broaden grip and setup coverage. Preserve alternatives rather than universalizing a beginner example. Do not describe symmetric palm-up calibration as traditional grip. Copyrighted source text, images and lesson MIDI are not implementation assets; author/source links and short independent summaries suffice here. No copied motion, code or weights are introduced by this decision.

## Rule abstraction

The following contracts are independent engineering proposals. Source locators indicate the instructional motivation, not proof that the solver exists or that numerical limits are validated.

| Rule ID / source | Proposed representation | Required observation/check |
| --- | --- | --- |
| T01 Setup — I01 p. 6; I03 Drum Height | `BodyKitCalibration`: seated pelvis, measured segments, comfortable shoulder/elbow candidate, grip/sole frames, target layout | Same numeric fitting path for every body; report reach and clearance rather than stretching bones. |
| T02 Grip — I01 p. 4; I02 grip sections | `GripProfile`: left/right hand frames, fulcrum, tool transform, optional finger constraints; matched/French/traditional profiles remain separate | Real hand-local attachment and measured wrist/forearm relation; absent fingers degrade detail explicitly. |
| T03 Ready/stroke — I01 p. 5; I03 Foundation | `ReadyPolicy` + `StrokeProfile`: per-surface clearance, preparation/contact/rebound/transfer, phrase-aware waiting | Start with an adjustable 2–3 cm tip clearance; do not treat it as a universal limit. Repeated snare notes do not detour to the torso. |
| T04 Heel-down — I01 p. 6 | `PedalTechnique` with heel support, ankle-led motion, heel/sole/forefoot contact tolerances and pedal pivot | Heel support remains plausible while the forefoot follows the pedal; do not equate fixed ankle position with grounded heel. |
| T05 Heel-up — I01 pp. 6–7 | Forefoot support, allowed heel lift, coupled ankle/knee path, preparation and recovery | The heel and ankle may translate while the forefoot maintains the intended contact. This requires extending the fixed-ankle solver. |
| T06 Pedal release — I01 p. 7 and lesson p. 46 | Independent `releasePolicy` (hold/rebound) and per-foot technique; hi-hat aperture/pressure has its own track | Do not infer release from heel mode. Do not apply a kick example to every hi-hat phrase. Current pedal note-ons are not continuous CC4. |
| T07 Sticking — I01 pp. 7, 18; I02 | `StickingStyle` (crossed/open/mirrored), preferred roles, explicit overrides and allowed fallback | Shared simultaneous-hit clearance checks, preserving event time and documenting infeasible preferences. |
| T08 Contact mode — I01 p. 7 | `ToolContactSpec`: tool tip/shaft region, surface point/normal and articulation | Crash contact may require a shaft region while other contacts use a tip. Existing tip-only checks do not validate all techniques. |
| T09 Phrase context — I01 lessons pp. 12, 34, 38, 47 | Deterministic beat/phrase hints and explicit user cues referencing original event IDs | Measure simultaneous-event timing; do not copy lesson scores or infer emotional intent solely from meter. |

Current implementation at `1e79532` provides measured VRM segments, seat/target controls, generic grip, an ankle-only foot approximation and deterministic contact checks. It does not implement these complete technique profiles. The [snare assessment](../../mydocs/tech/research/2026-10-24/snare-ready-and-kit-fit.md) records the generic-rest and dynamic-wrist gaps. Existing palm orientation remains a calibration override, not a scientifically validated technique preset.

## Proposed data flow

| Layer | Owned information |
| --- | --- |
| Source events | Immutable tick, integer `timeUs`, note, velocity, track and event index. |
| Mapping | Semantic voice/articulation and **physical piece instance ID**, selected contact frame and user limb overrides. |
| Calibration | Measured semantic joints, body/tool dimensions, seat, kit and pedal mechanics. No avatar-name branches. |
| Technique | Versioned grip/sticking profiles; left/right pedal mode, release and support contacts; bounded joint contributions. |
| Expression | Amount 0–100, allowed channels, per-phrase/event cues, envelope parameters and any deterministic seed. |
| Motion plan | Resolved assignments, ready/prepare/contact/rebound/transfer phases, tool and support constraints, optional gestures and diagnostics. |
| Evaluation/adapter | Rust evaluates the same absolute time; the common adapter converts normalized/raw bones and attached tools to model space. |

These are conceptual fields, not a shipped JSON schema. Version and validate them when extending `PerformancePlan`/`RigProfile`; derived plans must identify the source events and relevant configuration versions. #15 will persist the settings and cues. Do not add post-solver mesh rotations to simulate expression: torso/shoulder changes must feed back into limb solving and contact checks. Camera motion and instrument hit feedback remain separate from performer expression.

## Meaning of expression 0–100

Let `e = clamp(amount / 100, 0, 1)` represent a **visual movement budget**, not loudness, force or measured emotion. At zero, necessary strokes, support and rebound remain. At one, the solver may use the largest feasible allowed gesture for the particular event; not every event receives maximum amplitude.

Keep source velocity independent. Use it and timing context to propose a baseline stroke; it is not a direct force measurement, and low velocity alone does not prove a ghost note. Expression may redistribute additional preparation, forearm lift, rebound and later torso/gaze motion around explicit accents. Exact numerical ranges remain unvalidated until measured on multiple bodies.

A candidate gesture envelope must respect contact at impact and continuity at phase boundaries. Do not simply scale joint angles or interpolate two poses and assume contact survives. Re-solve constraints, reducing optional motion toward a feasible baseline when needed. If the baseline is impossible, retain source audio/event data and report the failure. Do not shift times, invent strikes, silently remove notes or change velocity to accommodate animation.

Beat positions derive from the actual tempo/meter map; authored phrase/accent cues override heuristic emphasis. Downbeat, phrase start and emotional climax are different concepts. Plan gestures with time to prepare and recover before adjacent events. Seed and record any variation; no frame-history randomness or accumulated springs may make seeking differ from export. Expression changes invalidate/rebuild the relevant motion plan and any stale video.

The current 0.5 s pre-roll and 0.25 s tail also bound expression. A large first-hit preparation or final recovery must fit those windows, reducing optional amplitude if necessary. Extending them later requires a separate versioned timing contract applied together to preview, audio alignment, frame count and export; an expression setting must not silently extend or clip the performance.

## Two-crash example and mapping prerequisite

`Instrument`/`KitPiece.id` currently uses ten fixed semantic roles, and MIDI 49/52/55/57 all map to `Crash`. `kit-calibration.ts` assumes one target per role. #14 must separate role/articulation from physical instances (for example two named cymbals), preserving original notes and event identity and migrating selection/calibration/diagnostics.

Two original simultaneous events may be mapped to two available cymbals and hands, then receive an expressive preparation if reach and clearance allow. Expression alone cannot turn one event into two impacts. An optional arrangement editor may later create a separately recorded derived event with explicit user action; it must not overwrite the source. A non-contact flourish also needs an explicit gesture contract and checks against unintended apparent strikes and interference with the next note.

## Sequence and acceptance

1. #18: establish coherent instrument/tool dimensions, measured fit and support/contact frames.
2. #20: implement phrase-aware ready positions and target-dependent wrist/forearm strokes; preserve naturalness at minimum expression first.
3. #14/#20: explicit style/grip/pedal variants and physical instance mapping; heel-up extends, rather than silently violates, ADR 0005's ankle-only approximation. Appearance swaps still preserve shared hardware; deliberate pedal actuation may rotate its mechanical parts.
4. #20/#23: expose expression only after a feasible baseline, then record comparable 0/50/100 cases. #15 persists settings later; #21 can eventually provide candidate motions under the same constraints.

Acceptance fixtures use independently authored MIDI: spaced/repeated snare, low/high velocities, dense rolls, fills, simultaneous hat/snare/kick, pedal-only rests, transitions, and eventually two distinct crashes. Compare body proportions and grip/pedal variants. Record actual tool-region/foot-support contact, time error, wrist/elbow/shoulder motion, clearance, reach failures and optional-motion reduction. Test 0/50/100 and intermediate values, configuration changes, random seek and export timestamps. Contact/continuity bounds and joint-angle thresholds require explicit engineering justification, not invented attribution to I01.

Include a first event at source `t=0` and a final event at the source end for every 0/50/100 case. Inspect the actual first/last encoded frames for truncated anticipation/recovery under the existing pre-roll/tail contract.

Track source ID + locator → rule ID → parameter/solver → fixture/evidence → implementation status. Update the public EN/KO register when adopting a new source or changing a claimed capability. Research-only rules stay unchecked. No release tag, GitHub Release or Pages deployment is authorized by this document.
