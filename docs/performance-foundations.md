# Performance foundations and sources

**English** · [한국어](performance-foundations.ko.md) · [Documentation](README.md)

We study drumming instruction alongside animation research, then design and test our own motion rules. A readable performance should preserve the music, fit the performer's body and make the chosen technique visible.

The [performance-expression design](../specs/architecture/performance-expression.md) connects posture, grip, heel-down/heel-up and coordinated playing to planned technique profiles and a 0–100 visual expression control. Those controls are **planned**, not supported settings. Current planted-foot ankle motion is an approximation; the [calibration guide](calibration.md) explains available controls and their limits.

## Research principles

- Preserve source MIDI notes, times and velocities. Optional visual emphasis must leave time and space for contact and recovery.
- Use measured body proportions and the same fitting process for every avatar. Technique choices and expression amount are separate.
- Distinguish source instruction, project interpretation, implemented behavior and measured evidence. Educational examples are not universal joint-limit measurements.

The detailed rules, acceptance checks and design decisions belong in the [technical design](../specs/architecture/performance-expression.md), so this page remains the entry point to the original sources and their credits.

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

[Technical design](../specs/architecture/performance-expression.md) · [Character models](models.md) · [Calibration](calibration.md)
