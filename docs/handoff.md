# Work entrypoint

Issues own live scope and acceptance. Contracts own behavior; verification records
own measured results and publication evidence.

## Current batch and next work

- [#56](https://github.com/postmelee/drum-vrm/issues/56) / [PR #58](https://github.com/postmelee/drum-vrm/pull/58):
  local binary FBX imports, guided folder/bone setup, saved material choices and
  automatic file settings, default VRM 0.0 shade compatibility, and concise menus.
  The feature branch now incorporates `main` through `6038e8e`; review the combined
  local build before integration. This is not a Pages deployment.
  Shared fit recovery now retains failure-stage evidence, refines existing elbow
  candidates and revisits seat/pedal/snare/hat anchors under the unchanged safety
  limits. Private VRM/FBX bodies still have bounded-search failures. Next: integrate
  after `codex/reference-kit-layout` establishes the new leg/garment/equipment
  geometry and layout policy, then rerun the combined cohort. Do not treat the
  standalone localhost:5180 comparison as validation of that concurrent work.
  See [fit recovery evidence](verification/fbx-import.md#shared-fit-recovery-candidate--2026-10-27).
  See [verification](verification/fbx-import.md) and the
  [import contract](../specs/architecture/rig-contract.md#fbx-import).

- [#60](https://github.com/postmelee/drum-vrm/issues/60): Songsterr Downloader MIDI
  compatibility and automatic mapping comparison on new uploads are a separate
  import increment. Conflicting ties stay visibly provisional; manual edits and
  saved-project profiles are retained.
  See the [mapping contract](../specs/architecture/midi-interpretation.md#songsterr-downloader-compatibility)
  and [verification](verification/songsterr-downloader-mapping.md).
  Review/integrate this increment separately from kit-layout work; it does not
  add URL fetching or establish a public deployment.

The detailed-hardware [PR #59](https://github.com/postmelee/drum-vrm/pull/59)
follows [PR #57](https://github.com/postmelee/drum-vrm/pull/57)
(natural posture/contact) and [PR #55](https://github.com/postmelee/drum-vrm/pull/55)
(VRM 0.0 import). The owner requested Pages publication of the completed hardware
batch, with no tag or GitHub Release. Read the
[hardware verification](verification/detailed-hardware.md) for exact tested WASM,
three-model fitting, multi-view inspection, performance regression and publication
record; do not infer public deployment from this entrypoint.

- [#18](https://github.com/postmelee/drum-vrm/issues/18): use score-observed
  simultaneous combinations and nearby transfers in bounded fitting. All three
  public models fit the 53-hit study, but the full song still loses one hand in
  the high/floor-tom chord at 10.166442 s. Preserve hardware assembly constraints.
- [#20](https://github.com/postmelee/drum-vrm/issues/20): carry validated pair/path
  witnesses through the whole motion. Some dynamic equipment-proxy penetration
  increased with the new obstacle set/layout; do not call static clearance a
  complete-motion success. Strike variation remains deferred.
- [#23](https://github.com/postmelee/drum-vrm/issues/23): retain source IDs,
  deterministic contact/pose checks and actual rendered marker evidence; inspect
  skin/clothing separately. Profile fitting latency before expanding the search.
- [#44](https://github.com/postmelee/drum-vrm/issues/44) owns automatic-fit/menu
  acceptance. Fitting/planning use cancellable Workers. [#54](https://github.com/postmelee/drum-vrm/issues/54)
  owns VRM 0.0 import; see its [verification](verification/vrm-0-support.md).

Start with [equipment/contact](../specs/architecture/equipment-contact.md),
[calibration](../specs/architecture/calibration.md),
[MIDI interpretation](../specs/architecture/midi-interpretation.md) and the
[engine boundary](../specs/architecture/engine-contract.md).
[Hardware sources](research/drum-hardware.md),
[ergonomics/sticking](research/ergonomics-and-sticking.md) and
[motion adoption gates](research/motion-adoption-gates.md) retain rationale.
[Outcome sequence](plan.md#outcome-sequence) keeps browser/WAV quality before
advanced expression and production outputs.

## Earlier evidence and provenance

[Natural posture/contact](verification/natural-contact.md),
[body clearance](verification/body-clearance-measurements.md),
[equipment/contact/count-in](verification/equipment-contact.md),
[browser performance](verification/browser-natural-performance.md) and
[groove-hand correction](verification/groove-hand-roles.md) preserve their own
measured commits and limits. The previous public batch was
`d23a9400abc9461633f26a9b805659026626a633`; its
[preset publication](verification/character-presets.md#pages-publication),
[English interface](verification/pages-english-interface.md) and
[earlier Pages batches](verification/pages-browser-performance.md) remain historical.

[Model guidance](models.md), [MIDI provenance](../public/demos/transparent-girl-1-provenance.json)
and per-model provenance under `public/characters/` own redistribution terms.
No private uploads are authorized by these exceptions.
[Documentation decisions](verification/documentation-simplification.md),
[requirements origin](context/source-conversation.md) and the
[pre-refactor handoff](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/handoff.md)
preserve earlier context. [Approval history](approval.md) and the
[release policy](release-policy.md) require explicit approval for later batches.
