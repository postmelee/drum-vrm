# Performance foundations and sources

**English** · [한국어](performance-foundations.ko.md)

Updated 2026-10-24. This is the project's research direction, not a list of supported controls. Technique profiles and a 0–100 expression control are **planned**. Current implementation and limitations are recorded in [the handoff](handoff.md) and [calibration guide](calibration.md).

## Principles

We study drumming instruction alongside animation research, then design and test our own motion rules. A readable performance should preserve the music, fit the performer's body, and make the chosen technique visible. More expression must remain compatible with contact, timing and the next action.

- Preserve source MIDI notes, times and velocities. Visual expression does not change the sound.
- Fit seat, instruments and contact points from measured body proportions, using the same process for every avatar.
- Keep required preparation, contact, rebound and balance at every expression setting.
- Select grip, sticking style and pedal technique separately from expressive movement.
- Measure actual attached tools, joint posture and clearance; attractive motion or accurate contact alone is insufficient.
- Distinguish what a source teaches, our interpretation, implemented behavior and measured results.

Instructional examples are starting points for particular contexts, not universal joint limits. The [engineering contract](decisions/0006-technique-and-expression-foundation.md) maps these principles to planned data and checks.

## Planned expression control

| Amount | Intended visual behavior |
| --- | --- |
| 0 | Economical movement, with all motion needed to play the notes intact. |
| 50 | Moderate preparation, rebound and phrase emphasis. |
| 100 | Larger feasible gestures around selected accents, with time and space for recovery. |

These are design targets, not a measured emotion scale or implemented presets. A quiet note keeps its musical role; 100 does not make every stroke large. When space or time is insufficient, reduce optional movement first. Grip and heel-down/heel-up are separate technique choices; the slider does not automatically switch them.

An emphasized first beat may receive a larger preparation or recovery, informed by the score and explicit user cues. A bar's first beat is not automatically a phrase climax. Two existing simultaneous crash events can eventually drive two hands and two cymbals. Adding a second strike to one event requires an explicit arrangement edit. The current kit maps crashes to one role, so distinct physical cymbals need additional mapping work.

## Instructional source register

Source IDs are stable. Access date: 2026-10-24. These entries contain links and project summaries; the source files, photos, scores and recordings are not bundled.

| ID | Author, publication and locator | Research use and boundary |
| --- | --- | --- |
| I01 | **Keiichi Nagata**, Yamaha, [DTX402 Series — Drum Score / Instruction](https://data.yamaha.com/files/download/other_assets/7/1224887/DTX_DrumScore_Instruction_eng.pdf). © 2018 Yamaha Corporation. Fundamentals: printed pp. 4–7; author profile p. 51; copyright final PDF page 52. | Foundational instruction for ready posture, grip, pedal techniques and coordinated playing. Lesson-specific advice remains contextual; p. 46 qualifies pedal choice for a hi-hat passage. It does not define our solver or expression slider. |
| I02 | **Riley Warren**, Yamaha Music, [A Guide to Proper Stick Grips](https://hub.yamaha.com/music-educators/instruments/perc/a-guide-to-proper-stick-grips/), 2025-12-19. Sections: fulcrum, matched, French, traditional. | Compare grip families, finger placement and wrist/arm roles. A symmetric palms-up setting is not a complete traditional grip. |
| I03 | **Dr. Dave Gerhart**, Yamaha Music, [Fix It: Tips for Teaching Snare Drum Basics](https://hub.yamaha.com/music-educators/instruments/perc/snare-drum-basics-pedagogy/), 2021-11-09. Sections: drum height, fulcrum, matched grip, foundation. | Relate setup to a comfortable playing posture and assess basic stroke instruction. Teaching examples are not measured joint-limit data. |

I01 carries the copyright credit above. I02/I03 carry Yamaha copyright notices. We have not established an open-content license for these instructional materials. Our original implementation remains MIT; attribution does not place their content under MIT or imply endorsement by the authors or Yamaha.

## Related references

The existing [research and reuse register](research/references.md) preserves authors, exact links, rights evidence and adoption status for **DRUMS (R01), MMIP (R02), Implementation of a 3D Virtual Drummer (R03), Disney's drummer-motion research (R04), MIDIAnimator (R05), ARDrum (R06), and Groove MIDI Dataset (R07)**. They cover different kinds of evidence: instruction, generation methods, tools and data are not interchangeable. No external learned-motion model is integrated.

Use the [rig/motion audit](research/rig-and-studio-audit.md), [VRM model register](research/vrm-models.md), [drum asset register](research/drum-assets.md), and [snare/kit-fit assessment](research/snare-ready-and-kit-fit.md) for standards, models and current gaps. Code, papers, data, weights and assets retain separate provenance and permissions.

## Delivery order and evidence

1. **#18:** measured body/kit fit and reliable ready/contact frames.
2. **#20:** natural basic strokes, target-dependent wrists and instrument-specific waiting positions.
3. **#14/#20:** explicit grips/sticking/pedal techniques, physical instrument instances and coordinated supports.
4. **#20/#23:** constrained expression 0–100 and comparison records; **#15** later saves the settings and cues.

Compare restrained/expressive cases on the same MIDI and different bodies, measuring time, contact, posture, reach and clearance. Sequential playback, random seeking and exported frames must agree. Current planted-foot ankle motion is an approximation; full heel-down/heel-up mechanics and expression control are not implemented. This documentation does not authorize release or Pages deployment.
