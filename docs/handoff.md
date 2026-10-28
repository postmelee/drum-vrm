# Work entrypoint

Issues own live scope and acceptance. Contracts own behavior; verification records
own measured results and publication evidence.

## Current review and next work

The owner requested common automatic fitting (#18) and visual review of the
three bundled public VRMs (#23), followed by a Pages test deployment. Work uses
`codex/public-vrm-fit`, based on PR #58's integrated `c2b06f2`. Preserve its FBX,
VRM 0.0 and expanded MIDI support, plus the accepted compact reference layout.
The other import chat has been notified of this continuation to avoid duplicate
solver edits. No new candidate has been merged or published by this batch. The visual/numeric
review found all three public automatic fits incomplete; publication is not a
three-model acceptance result.

- [#18](https://github.com/postmelee/drum-vrm/issues/18): the common fitter still
  exhausts its bounded search for all three public models. Expanded snare/rack
  seed experiments were rejected: they did not complete these fits and raised
  native fitting latency to 138 seconds. The candidate keeps the integrated
  fitter, nominal cymbal sizes, lower-limb constraints and failure rollback.
  Next work needs a shared posture/approach and complete-kit feasibility solution,
  not another avatar-specific placement exception.
- [#23](https://github.com/postmelee/drum-vrm/issues/23): review evidence now covers
  the three public models, source-derived groove/tom/recovery landmarks, actual
  adapter contact markers and ordered/shuffled evaluations. Missing assignments
  and intersections remain explicit failures. The candidate fixes an intended
  cymbal surface query that could overlook the stick shaft crossing its bow.
- [PR #58](https://github.com/postmelee/drum-vrm/pull/58) supplies the integrated
  source baseline. Its [record](verification/shared-fit-integration.md) retains
  the native success (seat +60 mm upward) and all six imported-model failures.
  Automated baseline test success does not establish imported-model acceptance.
- [#20](https://github.com/postmelee/drum-vrm/issues/20): broader motion-path
  refinement, expression and strike variation remain after this fit/review batch.
  Existing residual motion diagnostics must remain visible.

Start with [calibration](../specs/architecture/calibration.md),
[equipment/contact](../specs/architecture/equipment-contact.md), and
[rig measurements](../specs/architecture/rig-contract.md). The
[reference-layout record](verification/reference-layout.md) owns the earlier
layout checkpoint, not proof of this candidate's public-model acceptance.
[Hardware sources](research/drum-hardware.md) and
[ergonomics/sticking](research/ergonomics-and-sticking.md) retain the rationale.

Record actual checks, exact source/WASM and browser evidence in the
[public VRM review](verification/public-vrm-fit.md) before publication.
The owner's request covers this fitting/review Pages batch, not a Release/tag or
unrelated changes; follow the [release policy](release-policy.md).

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
