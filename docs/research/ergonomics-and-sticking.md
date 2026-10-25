# Ergonomics, sticking and import interpretation

Assessment: 2026-10-25, runtime commit `954936a7fba8500986b52b5c8426b9f7480d2f1a`
in [PR #40](https://github.com/postmelee/drum-vrm/pull/40). This is follow-up research
and proposed acceptance, not implemented collision avoidance or an approved release.
It responds to the owner's rendered review without replacing the original
[measured results](../verification/midi-interpretation-foundation.md).

Current contracts remain authoritative: [calibration](../../specs/architecture/calibration.md),
[technique](../../specs/architecture/performance-expression.md) and
[MIDI interpretation](../../specs/architecture/midi-interpretation.md).
Issues own execution status; this record preserves evidence and rationale.

## What the present code explains

| Observation | Evidence at the assessed commit | Consequence |
| --- | --- | --- |
| Hi-hat and high tom intersect | Default Studio geometry: cymbal mount XZ `(-0.48, 0.55)`, tom center `(-0.26, 0.72)`. Center distance ≈0.278 m versus radii sum 0.400 m. Cymbal height envelope ≈0.945–0.989 m intersects the tom body at 0.78–1.00 m. | Inspect actual body volumes and access paths, not just target distance. |
| Ride overlaps the floor tom | Ride mount XZ `(0.70, 0.53)` is ≈0.136 m from tom center `(0.62, 0.42)`, inside its 0.23 m radius. The cymbal can sit above the head, but the vertical stand passes through the tom volume. | Top-view overlap alone is not a failure. Mount/stand routing is a separate constraint. |
| The kit looks oversized | At default procedural scale, snare/hat diameters are 44 cm (≈17.3 in), toms 36/40/46 cm, crashes 60 cm (≈23.6 in), ride 56 cm. | Start with a coherent physical-size baseline before fitting or choosing a compact variant. |
| Imported models show missing motions | `adoptRig()` applies automatic seat calibration only; `fitKit()` runs through a separate explicit button. | Separate successful VRM import from an unfitted layout and infeasible hit assignment. |
| Tom repetitions stay on one hand | `assignment_cost()` penalizes rapid same-hand repetition only for `Snare`. High tom prefers left; mid/low tom prefer right. | Extend phrase candidates across hand-played surfaces instead of adding another isolated special case. |
| Left crash interrupts the snare hand | Crash prefers left, except `crashRight`; cost has no groove-role continuity or arm/stick clearance. | Consider musical role and both moving arms, while preserving simultaneous source demands. |
| Arms pass through the torso | Limb roots stay fixed; the adapter resets normalized pose and applies limbs/head. No torso rotation channel or body/arm collision proxy exists. | A torso turn added after solving would move shoulders and invalidate contacts. |

Code locators: [kit data](../../crates/drum-engine/src/model.rs),
[mounts/hardware](../../src/kit-hardware.ts), [procedural bodies](../../src/model-procedural.ts),
[fit](../../crates/drum-engine/src/fit.rs), [planner/motion](../../crates/drum-engine/src/motion.rs),
[import flow](../../src/main.ts), [humanoid adapter](../../src/rig-humanoid.ts).
The envelope observations are analytic checks of default procedural geometry, not
triangle-level tests of every preset. Uniformly shrinking an intersecting layout
preserves its relative intersection; reach-only fitting cannot repair that problem.

## Reconcile missing hits with earlier verification

The existing final audit used the production automatic-seat path followed by explicit
kit fitting. Before → after unassigned counts were: native 0→0, twist 82→0,
Seed-san 0→0, Avatar B 82→0, controlled short arms 285→0 and long arms 50→0.
The owner's screenshot shows 82 missing motions, consistent with an unfitted tested
profile, but neither the exact build nor the fit action can be established from an
image. Do not dismiss the report as stale or call every missing motion a VRM bone-map
failure. Zero unassigned hits after fitting never certified skin/shaft clearance.

Next reproduction should record build, mapping policy, measured arm/leg lengths,
seat, layout, fit status and exact song time in a local diagnostic snapshot. Private
assets and note streams remain private. Compare default/imported, seat-only and fitted
states on the same source. Offer an initial fit preview after import; preserve manual
edits and provide apply/restore, rather than silently overwriting the user's layout.

## Fit from a playable reference posture

The proposed order is support/seat → relaxed hand-and-stick ready frames → snare
surface → hi-hat → tom/cymbal reach zones. Use actual segment and grip lengths,
not standing height or avatar identity. I01/I03 in the source table motivate this
order; their teaching examples are not universal anatomical limits.

1. Establish a physically coherent default kit. Yamaha's examples use a 14-inch
   snare/hat and smaller rack toms; sizes vary, so these are reference choices.
2. Fit a comfortable seated pose with pedal support, shoulder/elbow room and two
   hand-specific snare contacts. A target being reachable is only one constraint.
3. Arrange other playable surfaces by role, height, surface normal, reachable
   approach and body/hardware clearance. Preserve left/right physical identity.
4. Adjust spacing first, then offer bounded compact diameters where needed. Keep
   actual contact geometry, visible bodies and supports consistent. Mesh-only
   appearance scale remains a separate control.
5. Compare complete preparation/contact/rebound/transfer paths, including both
   hands at once. Unresolved constraints remain diagnostics, not shifted note times.

Keep stable stands/pedals during an appearance replacement. A deliberate layout fit
may reposition supports while preserving their roles and constraints; these are
different operations. A boom/offset mount can place a cymbal above a tom without
putting a vertical stand through it.

For snare variation, start with small hand-specific regions inside the intended
articulation zone. Later, choose bounded offsets using source event identity, an
explicit seed and planner version. Store/evaluate them deterministically; do not
draw fresh randomness per frame. Variation must pass contact/clearance checks and
must not turn a center stroke into a rimshot or disguise a poor base posture.

## Coordinate arms and torso

The mean direction toward both hand targets can provide a preferred facing direction,
but it is not a collision solver: opposed targets can cancel, crossed arms can still
penetrate the body, and rapid alternation can make the torso oscillate.

First introduce shared conservative torso/arm/stick collision proxies and crossed
playing corridors (right-hand route above the snare-hand route where appropriate).
Plan a small number of torso/shoulder candidates over a phrase, balancing clearance,
reach, comfort and movement rate. Use target/phrase context rather than a feedback
loop on the previous rendered hand vectors. Keep pelvis/support constraints explicit.

Each candidate changes shoulder roots and must re-solve both arms against unchanged
contact times and surfaces. Extend the versioned engine/rig pose contract for torso
orientation before adding a renderer-only turn. Evaluate the resulting trajectory
by absolute time for seek/export agreement. Proxy clearance does not establish that
all hair, sleeves or skin vertices are collision-free.

## Sticking is distinct from stroke mechanics

Start with alternate singles (`RLRL`), controlled doubles (`RRLL`) and single
paradiddle (`RLRR LRLL`), including opposite-hand starts. A short `LRRLR` sequence
can be a fragment of several patterns; it does not uniquely identify a named
technique. These are candidate vocabularies, not a requirement to fit every phrase
to a rudiment or copy an instructional score.

Generate candidates for snare/tom runs and mixed fills. Reject or diagnose violations
of original timing, simultaneous limb occupancy, reach and geometric clearance.
Among feasible candidates, rank groove-hand continuity, travel/recovery demands,
velocity accents and the next destination. Look ahead in beats/phrase boundaries
with a computation bound; the current 0.35-second horizon is not a universal phrase.
Provide manual sticking overrides through #14, retaining source identity.

Then select preparation and rebound states: a down stroke ends low after an accent;
an up stroke prepares a following accent; a tap stays low. For the proposed
rebound-double candidate, model a controlled rebound rather than two repeated
full-arm swings. Velocity
and adjacent accents are useful evidence, but MIDI Note On does not encode the hand
or a unique technique. Do not promise to recover the original drummer's sticking.

### Crash transition exception observed in the private input

Read-only parsing of the corrected input found 15 crash-bearing time groups in its
first 45 seconds; five also contain an open-hi-hat note at the exact same tick.
These pairs contain no snare. A single hand cannot perform both simultaneous
impacts. The first crash group is a two-crash accent. This explains why simply
forcing the hi-hat hand onto every crash would violate the source in this benchmark.

Prefer hi-hat hand → crash → hi-hat when the crash replaces that hand's groove beat.
When both events exist, compare valid two-hand paths; a left-hand crash may be the
right assignment while its current route is wrong. Keep original physical crash
identity. Any user-requested replacement/omission is an explicit derived arrangement,
not a hidden motion repair. WAV remains unchanged unless the user edits it separately.

## Groove ownership and post-fill crashes

Follow-up to the owner's review of deployed `045b78a`, assessed 2026-10-26.
This extends the earlier crash-transition proposal without changing its historical
observations. Keep ordinary groove ownership separate from the exit of a head-led fill.

- Keiichi Nagata / Yamaha, DTX402 instruction pp.5 and 7: a ready position over the
  head and right-hand hi-hat above left-hand snare motivate the default crossed
  groove. The two-handed hi-hat example on p.18 shows why that default is not a
  universal restriction on all sticking.
- Mark Wessels / Wayne Salzmann II, [Zildjian Lesson 15 PDF](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_15.pdf),
  printed p.37: a right-hand double on the floor tom prepares a left-hand crash.
  This directly motivates looking through the end of a fill to the next destination.
- Their [Lesson 11 PDF](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_11.pdf),
  printed p.29, separately practices returning from a crash to the hi-hat within an
  ongoing groove. It does not prescribe the same crash hand after every fill.
- [Lesson 14 PDF](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_14.pdf),
  printed p.35, emphasizes reachable tom placement and relaxed wrist-led movement.

Project inference: preserve the specified cymbal and event time. During a groove,
prefer the groove hand for a replacement crash; after a head-led exit, compare both
hands using the preceding fill and following hat. A left-hand crash with right-hand
hat preparation is a valid candidate, not a filename-specific correction. The
bounded phrase planner still does not prove full-path collision freedom. PAS/Vic Firth
rudiments support candidate vocabulary, not a universal choice of crash hand.

These are linked educational sources, not reusable animation/score assets. No source
images, scores, recordings or motion data are bundled. The
[engine contract](../../specs/architecture/engine-contract.md) owns implemented ranking;
issues #14/#20/#23 retain remaining style and path-review acceptance.

## Generalize import guidance without guessing intent

The private file has track/instrument-name text but no sequencer-specific metadata,
SysEx or CC evidence establishing its producer. A name such as a kit preset is weak
evidence; even a known DAW does not identify the instrument or its input map.

Current UI and engine checks restrict count-in and short-open-close policy to
`logicDkdGm`; the latter also examines raw note keys. Generalization should operate
on mapped semantic roles/capabilities. A short close can still be a real foot chick,
and GM note 75 can be claves. Do not replace ambiguity with always-on inference.
The [central interpretation contract](../../specs/architecture/midi-interpretation.md#import-guidance)
owns the proposed producer/profile distinction, conditional UI and intent precedence.

## Sequence and existing issue ownership

| Order | Scope | Existing issues and observable completion |
| --- | --- | --- |
| 0 | Reproduction and diagnostics | #23 with #14/#18: distinguish map/import/fit/assignment/clearance states; capture a reproducible local setup. |
| 1 | Physical baseline and body-first fit | #18 with #14: correct nominal sizes, default body/stand intersections, snare ready frame and both-hand reach; compare fitted and unfitted models. |
| 2 | Coordinated arm/torso paths | #20 T01/T03/T07 with #18/#23: preserve contact/support while avoiding body/arm/stick interference throughout transitions. |
| 3 | Tom/fill and crash sticking | #20 T03/T07/T09 with #14: singles/doubles/accent candidates, groove role continuity and simultaneous-hit exceptions using the same clearance evaluator. |
| Parallel | General import UI | #14: profile recommendation with uncertainty, semantic hi-hat policies and conditional count-in/control choices; paired public guidance on implementation. |
| After baseline | Persistence and optional expression | #15 records mapping/intent/seed/calibration; broader #20 expression and #21 learned-model adoption remain later work. |

These are subscopes of open issues, not new duplicate issues or a wholesale move of
advanced #20 work into the basic milestone. Basic collision/assignment quality should
precede project persistence for the current benchmark goal. #23 supplies regression
fixtures at every step, not a final testing phase.

Required fixtures include default kit/preset envelopes, small/large and short-arm
bodies, both snare hands, crossed/open-hand alternatives, tom runs, mixed fills,
accent inversions, hi-hat→crash with/without simultaneous hi-hat, two crashes, phrase
entry/exit and shuffled seek/export. Use independently authored public patterns.

## Primary sources and reuse boundaries

Accessed 2026-10-25. Only links, brief summaries and independently proposed rules are
added. No source score, image, video, sound, model or dataset is bundled.

| Source | Locator and use | Rights/evidence boundary |
| --- | --- | --- |
| I01 — Keiichi Nagata / Yamaha | [DTX402 Drum Score / Instruction](https://data.yamaha.com/files/download/other_assets/7/1224887/DTX_DrumScore_Instruction_eng.pdf), printed pp.5–7: ready/support/crossed playing; pp.13–15 and 39: sticking and fills. | Existing [source register](../performance-foundations.md); ©2018 Yamaha. Contextual teaching, not measured joint constraints. |
| I03 — Dr. Dave Gerhart / Yamaha | [Snare basics](https://hub.yamaha.com/music-educators/instruments/perc/snare-drum-basics-pedagogy/), 2021-11-09, “Snare Drum Height”: establish relaxed arms before placing the drum. | Existing register; no open-content grant established. Adapt the principle, not a fixed elbow angle for all models. |
| Riley Warren / Yamaha | [Drum Set Configuration](https://hub.yamaha.com/music-educators/instruments/perc/drum-set-configuration/), 2025-11-24, beginner/intermediate components and setup tips. | Common dimensions and reachable arrangement examples, not a mandatory universal kit. Yamaha copyright. |
| PAS International Drum Rudiment Committee, chaired by Jay Wanamaker | [International Drum Rudiments](https://pas.org/rudiments/) and [official PDF](https://pas.org/wp-content/uploads/2024/04/pas-rudiments.pdf): single/double/paradiddle vocabulary. | ©1984 PAS, all rights reserved; linked audio ©2005 Alfred. No score/audio redistribution. |
| Dr. John Wooton / Vic Firth | [Single Paradiddle](https://ae.vicfirth.com/education/40-essential-rudiments/40-essential-rudiments-single-paradiddle/), [Double Stroke Open Roll](https://ae.vicfirth.com/education/40-essential-rudiments/40-essential-rudiments-double-stroke-open-roll/). | Instruction and application references; Zildjian rights notice, no implementation/recording adoption. |
| Mark Wessels; video instructor Wayne Salzmann II / Zildjian | [Lesson 17 PDF](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_17.pdf), printed pp.40–41: down/up/tap and accent recovery; [Lesson 11 PDF](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_11.pdf), p.29: crash return and approach; [Lesson 15](https://ae.zildjian.com/education/beginner-drum-set-lessons/zildjian-drum-set-method-lesson-15/): toms/fills. | Educational references, not reusable mocap or numerical biomechanical limits. No open-content grant established. |
| MIDI Association / Apple / Roland | [SMF overview](https://midi.org/standard-midi-files), [Note On messages](https://midi.org/about-midi-part-3midi-messages), [Apple SMF](https://support.apple.com/en-il/guide/logicpro/lgcpdf6a3851/mac), [DKD maps](https://support.apple.com/en-ae/guide/logicpro/lgsia2ae90f1/mac), [Roland sound list](https://cdn.roland.com/assets/media/pdf/Fantom-G_Soundlist.pdf), p.23. | Message/metadata and profile evidence; no guarantee of producer identity or unique technique. Documentation only. |
| Google Magenta | [GMD format/mapping/controllers](https://magenta.withgoogle.com/datasets/groove). | R07 in the [research register](references.md): device-specific mapping and CC4 example. No dataset download/adoption in this review. |
