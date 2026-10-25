# Technical design

**English** · [한국어](README.ko.md)

These documents explain the checked-out source: its requirements, component boundaries,
coordinate and timing contracts, and the reasoning behind them. Proposed extensions are
identified separately from implemented behavior. See the [project README](../README.md)
for the public app and quick start.

| Topic | Read |
| --- | --- |
| Purpose and invariants | [Product requirements](requirements.md) |
| Components and data flow | [Architecture overview](architecture/overview.md) |
| Source events, mapping and performer intent | [MIDI interpretation](architecture/midi-interpretation.md) |
| Local save/reopen and source relinking | [Project format](architecture/project-format.md) |
| Timing, assignment and pose interface | [Engine contract](architecture/engine-contract.md) |
| Humanoid, coordinates and drum hardware | [Rig contract](architecture/rig-contract.md) |
| Audio clock and video frames | [Audio/export contract](architecture/audio-export-contract.md) |
| Body, seat, grip and contact fitting | [Calibration](architecture/calibration.md) |
| Standard rig support and measured limits | [Rig compatibility](architecture/rig-compatibility.md) |
| Technique and future movement expression | [Performance expression](architecture/performance-expression.md) |
| Runtime stack and adoption tradeoffs | [Dependency design](architecture/dependencies.md) |

The [architecture index](architecture/README.md) groups these contracts and explanations.
Original decisions remain in [docs/decisions](../docs/decisions/), with their IDs and
historical scope. [Research sources](../docs/research/references.md) and
[verification records](../docs/verification/) retain the evidence behind the design.
