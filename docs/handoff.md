# Work entrypoint

Issues own live scope and acceptance. Contracts own behavior; verification records
own measured results and publication evidence.

## Current review and next work

The owner-approved kit/grip batch on `codex/kit-grip-refinement` now includes
player-facing rack toms, the bounded inward-to-down palm family and prepared
roll controls shared by edge validation and replay. It remains a **local/draft
review candidate, not published**. Read the
[final v13 evidence](verification/common-motion-fit.md#prepared-roll-review-candidate--final-v13-boundary)
before changing the solver. All four fits and 53/53 + 82/82 assignments pass;
all eight studies still fail temporal physical/posture acceptance, with some
new or deeper tool overlaps. Do not describe this as a regression-free release.
The [elbow follow-up audit](verification/elbow-motion-audit.md) confirms that
hand roll rotates the elbow reference plane outside the relative swivel bound.
Simple pole replacements failed existing valid witnesses and were reverted.
Next work is actual anatomical elbow-plane coordination with prepared grip roll,
preserving exact impacts, reach and whole-tool gates. Further crash approach and
rack tilt are not yet accepted; fast-fill choice explanations are also unfinished.
Do not add another fitting grid or force paradiddles into every fill. The
[ergonomics/sticking research](research/ergonomics-and-sticking.md) records why
opposite-hand contact feasibility comes before retuning the existing vocabulary.

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
- **Next acceptance:** preserve current successful fit/assignment/contact and
  prepared-format-3 restoration checks while repairing the remaining tool paths,
  wrist limits and excessive arm elevation. Marker precision, palm direction and
  seek equality do not certify intervening motion. Exact branch-jump regressions
  now pass, but sampled roll speeds remain too high for a natural-motion claim.
  Keep earlier captures/measurements distinct from the final candidate evidence.
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
The earlier Pages authorization covers the published PR #64 batch. A subsequent
candidate needs its own reviewed scope and approval under the
[release policy](release-policy.md); no Release/tag is implied.

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
