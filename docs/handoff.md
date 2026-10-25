# Work entrypoint

Use the relevant issue for current scope and review status. This page routes the next worker; it does not duplicate release state or measurements.

- The approved implementation sequence is MIDI interpretation/physical instances → common body/wrist/kit fitting → phrase sticking and hi-hat control → WAV synchronization → full-song reproducibility. [PR #40](https://github.com/postmelee/drum-vrm/pull/40) implements these foundations and records bounded full-song checks in [verification](verification/midi-interpretation-foundation.md). Review before merging; no new release or Pages approval.
- [Simplification verification](verification/documentation-simplification.md) preserves the documentation audit; it does not establish new motion or model compatibility results.
- The owner corrected the private benchmark's final simultaneous three-hand input. Use the corrected file for new checks and preserve the original-input evidence. Never introduce a filename/avatar-specific exception or silently omit its events.
- The owner's rendered review identifies default kit/stand intersections, unfitted-body failures, crossed-arm penetration and tom/crash sticking gaps. [Follow-up research and sequencing](research/ergonomics-and-sticking.md) separates these from PR #40's event-accounting results. Proposed next order: reproducible setup diagnostics → physical-size/layout and body-first fit (#14/#18) → coordinated arm/torso paths and phrase sticking (#20), with #23 checks throughout. General import guidance can proceed in parallel under #14; project reopen (#15) follows this basic visual quality work. No further runtime changes are authorized by this routing note.

Start with the [MIDI interpretation contract](../specs/architecture/midi-interpretation.md), then its linked engine, calibration and expression contracts. Earlier [calibration](verification/calibration-unreleased.md), [palm orientation](verification/palm-orientation-unreleased.md) and [VRM foundation](verification/vrm-studio-unreleased.md) measurements retain their original commit scope. [Source research](research/references.md) preserves provenance.

The [release policy](release-policy.md) requires separate batch approval. No release or Pages approval is conveyed by this index.

Earlier context remains in the [pre-refactor handoff](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/handoff.md), [requirements origin](context/source-conversation.md) and [approval history](approval.md). Consult them for historical evidence, not current task status.
