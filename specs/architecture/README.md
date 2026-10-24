# Architecture contracts

Read [the overview](overview.md) first, then the contract for the boundary being changed.

| Contract | Owns |
| --- | --- |
| [Engine](engine-contract.md) | Input interpretation, immutable events, planning, profile/pose API |
| [Rig](rig-contract.md) | Semantic bones, coordinate conversion, calibration, rendered contacts and kit appearance |
| [Audio and export](audio-export-contract.md) | Song/output time, PCM, playback clock, codec validation and encoding |

## Detailed design chapters

- [Calibration](calibration.md): what fitting can and cannot solve.
- [Performance expression](performance-expression.md): technique, immutable music and optional gestures.
- [Dependencies](dependencies.md): implementation responsibilities and runtime tradeoffs.
- [Rig compatibility](rig-compatibility.md): semantic standard, adapter choices and evidence.

The chapters provide maintained technical explanation alongside the exact contracts;
Diátaxis does not require compressing them into short user guides.

These are current source contracts, not deployment or test reports.
The [decision index](../decisions/README.md) preserves rationale and historical scope.
[Specification index](../README.md)
