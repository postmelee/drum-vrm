# Architecture

**English** · [한국어](README.ko.md)

Start with the [overview](overview.md), then read the boundary relevant to the change.

| Contract | Owns |
| --- | --- |
| [MIDI interpretation](midi-interpretation.md) | Source event evidence, mapping profiles, performer intent and control boundaries |
| [Engine](engine-contract.md) | Input limits, timing, assignment and the profile/pose API |
| [Rig](rig-contract.md) | Semantic bones, coordinate conversion, calibration, rendered contacts and drum appearance |
| [Equipment and contact](equipment-contact.md) | Measured bodies, generated hardware, playable regions, complete-tool paths and count-in gestures |
| [Audio and export](audio-export-contract.md) | Song/output time, PCM, playback clock and encoding |
| [Local project format](project-format.md) | Versioned settings, source evidence, exact file relinking and restore validation |
| [Notifications](notifications.md) | Concise inspector copy, unified toast placement, lifecycle, motion and accessibility |

The detailed chapters explain [body/tool/kit calibration](calibration.md),
[humanoid compatibility](rig-compatibility.md), [technique and expression](performance-expression.md)
and [dependency choices](dependencies.md).

These contracts describe the source, not a deployment or a new test result. The
[original decisions](../../docs/decisions/) preserve design rationale and historical scope.

[Technical design](../README.md)
