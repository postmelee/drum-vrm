# Feature availability

**English** · [한국어](availability.ko.md)

This is the availability boundary for the reader guides. The [public app](https://postmelee.github.io/drum-vrm/) corresponds to [v0.2.0](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0), application commit `3e8be9333cedbc9845dbc3e4cebd73961af697b4`. A source-branch change does not change that deployed app.

| Capability | Public v0.2.0 | Development build documented here |
|---|---|---|
| MIDI preparation, playback, seeking, synthesized audio and video export | Available | Available |
| Basic skeleton, light/dark workspace and piano roll | Available | Available |
| Front drum editing, local static GLB appearance replacement | Available | Available |
| Procedural drum styles and Innerscene kit | Available | Available |
| Batoski and smoj kits, Lucide controls, player camera, optional hit response | Not in this release | Implemented |
| VRM 1.0 import and measured semantic humanoid adapter | Not in this release | Implemented; PR #33 merged |
| Seat/contact controls, planted ankle strokes, palm orientation and persistent drum hardware | Not in this release | Implemented in PR #34; not merged at the documentation baseline |
| WAV synchronization, project persistence, manual note/hand editing | Not available | Planned |
| Full heel-down/heel-up profiles, collision-free crossed-hand style, expression 0–100 | Not available | Planned |

The development documentation baseline is application commit `66951cd69015e43cfe132a6ba61545b175a25412`. The paired guides describe that behavior; they do not announce a new release. Current review states belong to [PR #33](https://github.com/postmelee/drum-vrm/pull/33), [PR #34](https://github.com/postmelee/drum-vrm/pull/34) and the [issue tracker](https://github.com/postmelee/drum-vrm/issues).

## Evidence and boundaries

- [Public v0.2 verification](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/studio-0.2.md) records the actual environment and artifact checks.
- [VRM integration evidence](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/vrm-studio-unreleased.md) and [calibration evidence](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/calibration-unreleased.md) distinguish contact/seek, application and encoding checks.
- [Palm evidence](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/palm-orientation-unreleased.md) preserves remaining dynamic wrist limitations.

A successful format check does not establish natural motion, collision-free skin or performance on every device. The controlled short/tall VRoid pair is still untested. Some VRM video encoding was completed without independent downloaded-file playback; consult the exact record rather than treating encoding as complete playback validation.

[Reference](README.md)
