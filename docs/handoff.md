# Work entrypoint

Issues own live scope and acceptance. Contracts own behavior; verification records
own measured results and publication evidence.

## Current batch and next work

- [#56](https://github.com/postmelee/drum-vrm/issues/56) / [PR #58](https://github.com/postmelee/drum-vrm/pull/58):
  local binary FBX imports, guided folder/bone setup, saved material choices,
  automatic file settings, default VRM 0.0 shade compatibility and concise menus.
  The feature branch incorporates `main` through `6038e8e`. The current local
  integration combines its `e5fafb9` fit recovery with reference-layout checkpoint
  `da026985b73abaa980c4ef702abc785f831137b2`, which started from `d8055d0`.
  See the [import contract](../specs/architecture/rig-contract.md#fbx-import) and
  [import/recovery evidence](verification/fbx-import.md).

- [#18](https://github.com/postmelee/drum-vrm/issues/18): the integrated fitter uses
  one placement/certification pipeline with a nominal-seat allocation capped at
  30,000 candidates and a reserved compact-family allocation within 80,000 total.
  Initial nominal work gets 12,000, seat retries 6,000; compact families get 6,000
  each and 24,000 per seat. Up to three validated snare anchors per pedal are
  reconsidered when later placement fails. It retains nominal cymbal diameters,
  the 0.76 minimum shell scale,
  grouped rack toms, measured lower-limb frames, pedal direction and physical mount
  metadata. Every returned layout uses the same arrangement, posture and clearance
  gates. Preserve body dimensions, hardware constraints and exact contact frames;
  report a failed fit rather than weakening them or adding avatar exceptions.
  The [reference-layout record](verification/reference-layout.md) retains the
  narrower knee/floor-tom/crash checkpoint and its incomplete acceptance.

The integrated candidate passes 182 Rust and 312 web tests. Native/Studio fits at
100% with a 60 mm upward seat adjustment, repeats unchanged and passes both authored
studies; localhost:5180 serves the verified build. The three public VRMs and three
private VRM 0.0/FBX samples remain incomplete under the stronger constraints.
Next: resolve their coupled hi-hat/snare/rack and lower-limb failures within the
same fitter; retain the failed setup and distinct search diagnostics. Do not treat
budget exhaustion as physical impossibility or weaken the gates to match earlier
hand-only results. The [combined evidence](verification/shared-fit-integration.md)
records actual outcomes, latency and UI/rollback limits; full-cohort acceptance
remains open.

- [#60](https://github.com/postmelee/drum-vrm/issues/60): retain Songsterr Downloader
  MIDI compatibility and automatic mapping comparison from the incorporated main
  changes. Conflicting ties stay provisional; manual edits and saved-project
  profiles remain intact. See the [mapping contract](../specs/architecture/midi-interpretation.md#songsterr-downloader-compatibility)
  and [verification](verification/songsterr-downloader-mapping.md). This adds no URL fetching.
- [#20](https://github.com/postmelee/drum-vrm/issues/20): carry validated pair/path
  witnesses through the whole motion after placement is accepted. Static clearance
  does not establish complete-motion or skin/clothing clearance. Expanded motion
  and strike variation remain separate work.
- [#23](https://github.com/postmelee/drum-vrm/issues/23): retain source IDs,
  deterministic contact/pose checks and actual rendered marker evidence; inspect
  skin/clothing separately. Profile fitting latency before expanding the search.
- [#44](https://github.com/postmelee/drum-vrm/issues/44) owns automatic-fit/menu
  acceptance. Fitting/planning use cancellable Workers. [#54](https://github.com/postmelee/drum-vrm/issues/54)
  owns VRM 0.0 import; see its [verification](verification/vrm-0-support.md).

The earlier detailed-hardware [PR #59](https://github.com/postmelee/drum-vrm/pull/59)
and [hardware verification](verification/detailed-hardware.md) retain their tested
WASM, multi-view evidence and limitations. Their successful study results do not
validate the newer lower-limb gate, measured sole frames or nominal cymbal sizes.
This integration is local work. Previous publication approvals do not authorize a
new Pages deployment, release or tag; follow the [release policy](release-policy.md).

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
