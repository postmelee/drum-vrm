# Work entrypoint

Use the relevant issue for current scope and review status. This page routes the next worker; it does not duplicate release state or measurements.

- The approved implementation sequence is MIDI interpretation/physical instances → common body/wrist/kit fitting → phrase sticking and hi-hat control → WAV synchronization → full-song reproducibility. The candidate on `codex/midi-interpretation-foundation` implements and verifies the first four boundaries; [verification](verification/midi-interpretation-foundation.md) records source accounting, visual/media evidence and remaining limits. Review before merging; no new release or Pages approval.
- [Simplification verification](verification/documentation-simplification.md) preserves the documentation audit; it does not establish new motion or model compatibility results.
- The owner corrected the private benchmark's final simultaneous three-hand input. Use the corrected file for new checks and preserve the original-input evidence. Never introduce a filename/avatar-specific exception or silently omit its events.
- Next review focus: rendered wrist/shoulder comfort and short-arm transfer speed ([#18](https://github.com/postmelee/drum-vrm/issues/18), [#20](https://github.com/postmelee/drum-vrm/issues/20)); then versioned project reopen and asset relinking ([#15](https://github.com/postmelee/drum-vrm/issues/15)). Full collision clearance, rim/shaft contact, pedal mechanics and expression 0–100 remain broader work, not completed by zero unassigned hits.

Start with the [MIDI interpretation contract](../specs/architecture/midi-interpretation.md), then its linked engine, calibration and expression contracts. Earlier [calibration](verification/calibration-unreleased.md), [palm orientation](verification/palm-orientation-unreleased.md) and [VRM foundation](verification/vrm-studio-unreleased.md) measurements retain their original commit scope. [Source research](research/references.md) preserves provenance.

The [release policy](release-policy.md) requires separate batch approval. No release or Pages approval is conveyed by this index.

Earlier context remains in the [pre-refactor handoff](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/handoff.md), [requirements origin](context/source-conversation.md) and [approval history](approval.md). Consult them for historical evidence, not current task status.
