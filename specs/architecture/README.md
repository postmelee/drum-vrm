# Architecture

**English** · [한국어](README.ko.md)

Start with the [overview](overview.md), then read the boundary relevant to the change.

| Contract | Owns |
| --- | --- |
| [Engine](engine-contract.md) | MIDI interpretation, source events, assignment and the profile/pose API |
| [Rig](rig-contract.md) | Semantic bones, coordinate conversion, calibration, rendered contacts and drum appearance |
| [Audio and export](audio-export-contract.md) | Song/output time, PCM, playback clock and encoding |

The detailed chapters explain [body/tool/kit calibration](calibration.md),
[humanoid compatibility](rig-compatibility.md), [technique and expression](performance-expression.md)
and [dependency choices](dependencies.md).

These contracts describe the source, not a deployment or a new test result. The
[original decisions](../../docs/decisions/) preserve design rationale and historical scope.

[Technical design](../README.md)
