# Work entrypoint

Issues own live scope and acceptance. Contracts own behavior; verification records
own measured results and publication evidence.

## Current review and next work

[PR #64](https://github.com/postmelee/drum-vrm/pull/64) carries the reviewed
common-fitting/motion checkpoint on top of PR #58's FBX/VRM 0.0 work. The owner
approved merging this known-incomplete candidate and publishing it to Pages on
2026-10-29. The PR publication record owns the exact integrated commit, CI,
workflow and public checks; approval alone is not deployment evidence. No
Release/tag or strike-position variation is included.

The [common-motion review](verification/common-motion-fit.md) preserves the
`b837104` source/WASM measurements and the older `f2d3b87` failed baseline. All
four static fits and both study assignments now succeed, but every model still
fails temporal physical-tool acceptance. The
[diagnostic follow-up](verification/common-motion-fit.md#diagnostic-follow-up-measurement)
exposes previously missed tool-path and evaluated-posture failures without
changing those measured fits or motion.

The integration preserves main's shared notification/concise-inspector work
([PR #63](https://github.com/postmelee/drum-vrm/pull/63),
[contract](../specs/architecture/notifications.md),
[evidence](verification/notification-design.md#production-integration)) and
Songsterr/automatic mapping work ([PR #61](https://github.com/postmelee/drum-vrm/pull/61),
[contract](../specs/architecture/midi-interpretation.md#songsterr-downloader-compatibility),
[evidence](verification/songsterr-downloader-mapping.md)). Conflict resolution
must retain guided FBX import, shared operation feedback and the native reference
startup. These UI integrations do not establish new motion acceptance.

- **Reference first:** the native skeleton starts at the reviewed fixed kit and
  authored seat, without an automatic startup search. Imported models first try
  common posture/seating with that kit, then bounded body-informed layout repair.
  Do not add another large candidate grid or avatar-name placement exceptions.
- **Physical tools versus estimated bodies:** whole-stick/equipment and
  stick/stick intersections remain required checks. Measured arm/costume overlap
  is advisory; structural lower-limb cores and actual pedal reach remain required.
  Report measured lower-body warnings separately from fit completion. Extreme
  costume clearance is outside this batch; character scaling is not implemented.
- **Next acceptance:** preserve the successful fits and complete 53/53 posture,
  82/82 interpretation assignments while repairing common tool trajectories and
  final evaluated posture, including the retained impact-pitch failure. Do not
  restart layout search to hide these diagnostics. Marker and ordered/reversed
  checks passed; they do not
  certify intervening motion. The new 45 textured ready/groove-entry/tom/dual-crash/
  ending captures cover front/top/left views of all three models; tom transitions
  still show excessive arm elevation, especially Seed-san. A final diagnostic-build
  smoke check confirmed B playback to end and native seat restoration; Seed-san
  and Twist continuous playback on that build remain unverified. Missing motion or tool
  measurements cannot count as acceptance. The earlier 36 captures remain a
  separate failed baseline.
- **Integration:** the other import chat has been notified to avoid duplicate
  solver edits. [PR #58](https://github.com/postmelee/drum-vrm/pull/58) and its
  [integration record](verification/shared-fit-integration.md) preserve their
  own scope/results. Keep broader #20 expression and strike variation deferred.

Start with [calibration](../specs/architecture/calibration.md),
[equipment/contact](../specs/architecture/equipment-contact.md), and
[rig measurements](../specs/architecture/rig-contract.md). The
[reference-layout record](verification/reference-layout.md) owns the earlier
layout checkpoint, not proof of this candidate's public-model acceptance.
[Hardware sources](research/drum-hardware.md) and
[ergonomics/sticking](research/ergonomics-and-sticking.md) retain the rationale.

Record subsequent checks and browser evidence against the exact candidate in the
[common-motion review](verification/common-motion-fit.md); preserve the older
[public VRM review](verification/public-vrm-fit.md) as its own measurement.
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
