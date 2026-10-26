# Articulation, approach and stick contact

This record separates teaching evidence from the project's numerical choices.
The current [equipment/contact contract](../../specs/architecture/equipment-contact.md)
owns implemented behavior; [ergonomics/sticking](ergonomics-and-sticking.md) retains
earlier investigations. No photographs, instructional scores, videos or source
animations are copied into the application.

## Primary teaching sources

| Source and locator | Relevant teaching | Engineering consequence |
| --- | --- | --- |
| Keiichi Nagata / Yamaha, [DTX402 instruction](https://data.yamaha.com/files/download/other_assets/7/1224887/DTX_DrumScore_Instruction_eng.pdf), printed pp.5, 7 | Relaxed ready posture with tips above the head centre; tip-led ride and shoulder-led crash examples | Establish a comfortable head-centred reference, then choose a tool part and playing region by articulation |
| Dr. Dave Gerhart / Yamaha, [Snare Basics](https://hub.yamaha.com/music-educators/instruments/perc/snare-drum-basics-pedagogy/), posture/grip sections | Establish relaxed arms and drum height together; leave elbow room and use wrist-led strokes | Do not repair every shaft collision by lifting the elbow or forcing the same anatomical angle on all bodies |
| Mark Wessels / Wayne Salzmann II, Zildjian [Lesson 7](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_7.pdf), printed pp.20–21 | Hi-hat setup permits top/tip and edge/shoulder strokes; crossed and open-hand alternatives exist | Opening state, striking surface and sticking style are different dimensions |
| Same authors, Zildjian [Lesson 11](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_11.pdf), pp.28–29 | Bell/bow versus edge strokes and a withdrawing crash approach | A player-facing sector is useful, but a universal nearest-edge target loses articulation |
| Same authors, Zildjian [Lesson 13](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_13.pdf), p.32 | Ride placement reachable by hand rotation from the snare; bow, bell and edge produce distinct responses | Keep the elbow relaxed when geometry permits; normalized contact regions need explicit semantics |
| Same authors, Zildjian [Lesson 10](https://ae.zildjian.com/wp-content/uploads/Zildjian_Drum_Method_Lesson_10.pdf), p.27 | Cross-stick keeps the tip on the head while the butt strikes the rim | This needs two contact relationships; moving a tip target near the rim does not implement it |
| Yamaha, [The Five Different Kinds of Snare Drums Explained](https://hub.yamaha.com/drums/stage/the-five-different-kinds-of-snare-drums-explained/), rimshot description | Rimshot uses simultaneous head/rim contact | A label and one planar point cannot certify rimshot mechanics |

The sources motivate different contact regions and relaxed setup, not the
application's fractions, metre limits, pitch grid or collision algorithm. Their
examples are contextual teaching, not measured anatomical limits. In particular,
the ride placement example does not prohibit elbow motion during all performance.
Original teaching rights remain with their authors/publishers; this is citation
and independent analysis, not an open-content license.

## Critical application

**Near-player sector:** preferable to aiming blindly at the centre of a cymbal when
the intended articulation is an edge crash. It is not a replacement for centre
snare strokes or bell/bow ride articulations. Surface direction and radius must be
evaluated in the same frame as the displayed body. A plane is still an approximation
to a curved bell, tilted cymbal or raised rim.

**Whole tool before elbow correction:** two poses can have identical tip error
while one shaft penetrates a drum shell. Check the calibrated wrist/grip, butt,
shaft and intended contact segment against body and hardware envelopes. Try common
orientation and approach candidates; preserve source impact time. If none clears,
report the failure rather than calling the target reachable and the motion solved.

**Count-in:** the cited pages do not establish a universal body-centred stick-click
technique. The project chooses an explicit tip-to-tip presentation cue as a narrow
first implementation. It requires both hands free, preserves the source, and does
not add an audio note. A physically simulated shaft-click technique is a separate
tool-contact mode, not an inferred property of MIDI note 75.

## Verification boundaries

- Compare centre/bow/bell/edge targets, exact timing and source identity separately.
- Test a reachable tip with an obstructed shaft, including the butt and the same
  instrument's body; do not exempt an entire intended-contact body.
- Test short arms, both palm directions, multiple drum shapes and the common
  posture reference. A successful fit on one model is insufficient.
- Keep true rimshot/cross-stick/shoulder-edge mechanics distinct from planar
  target approximations until the required multiple contact constraints exist.
- Test count-in source-only semantics, unavailable hands, close cues, exact tip
  contact, arbitrary-time seeking and prepared-worker restoration. Intentional
  stick contact must not exempt arms, opposing hands or nonterminal shaft overlap.
- Compare posed skin and rendered tools at difficult times. Static boxes and
  sampled temporal proxies cannot establish continuous triangle-level clearance.

Reusable private verification scripts retain their original inputs and scope in
`.local/`. Full-skinned loading and actual marker evaluation are stronger evidence
than a bone-node-only reconstruction. Published reports record aggregates and
versions without redistributing the private source assets.
