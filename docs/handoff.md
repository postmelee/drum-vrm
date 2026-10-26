# Work entrypoint

Use the relevant issue for current scope and review status. This page routes the
next worker; contracts own behavior and evidence reports own measured results.

- The approved body-clearance batch adds automatic character/kit fitting, focused
  menus with Analysis, measured skin-weight geometry, common static target-pair
  checks, and coordinated temporal torso/elbow planning. Fitting and planning run
  in cancellable Workers. See [verification and limits](verification/body-clearance-measurements.md).
- Pages publishes `d23a9400abc9461633f26a9b805659026626a633`, the owner's
  approved #47/#48/#50/#52 batch: selectable official VRM samples, public demo, equipment/contact/count-in and an
  English-default interface with live Korean switching. [Publication evidence](verification/character-presets.md#pages-publication)
  records the exact candidate, workflow and actual checks. [Approval history](approval.md)
  retains the scope. No tag or GitHub Release was requested or created.
- [#44](https://github.com/postmelee/drum-vrm/issues/44) owns automatic-fit/menu
  acceptance. [#18](https://github.com/postmelee/drum-vrm/issues/18) owns common body
  calibration, [#20](https://github.com/postmelee/drum-vrm/issues/20) owns motion
  quality, and [#23](https://github.com/postmelee/drum-vrm/issues/23) owns inspection.
  Keep their broader acceptance open: static feasibility and accurate contact do
  not certify clear skin/clothing throughout a song.
- Merged [PR #48](https://github.com/postmelee/drum-vrm/pull/48) measures displayed
  bodies, shares numeric supports with Rust, checks complete sticks and
  articulation regions, and adds body-front two-stick count-in. See the
  [contract](../specs/architecture/equipment-contact.md) and
  [verification](verification/equipment-contact.md). The owner explicitly defers
  the remaining tom/cymbal approach contacts and Twist bounded-fit failure until
  after this publication. These remain the next motion work before strike
  variation. Do not shrink proxies, stretch bones or shift MIDI times to remove
  warnings; inspect posed skin/clothing as well as numeric clearance.
- Merged [PR #47](https://github.com/postmelee/drum-vrm/pull/47) supplies the
  selectable full-song demo. [Provenance](../public/demos/transparent-girl-1-provenance.json)
  owns its exact bytes and distribution permission; this is not MIT relicensing
  or composer attribution. The [demo contract](../specs/architecture/midi-interpretation.md#bundled-demo-selections)
  preserves the score and applies the approved final-snare omission as explicit
  selection data. No VRM or WAV is bundled with it.
- Merged [PR #50](https://github.com/postmelee/drum-vrm/pull/50) localizes UI and
  diagnostics at the presentation boundary. [Locale contract](../specs/architecture/overview.md#interface-language)
  owns URL/preference behavior; source events, uploaded metadata, plans and
  playback remain independent of the language. PR #42's historical evidence was
  already incorporated by #45, so the duplicate PR was closed without restoring
  stale deployment labels.
- Merged [PR #52](https://github.com/postmelee/drum-vrm/pull/52) under [#17](https://github.com/postmelee/drum-vrm/issues/17)
  adds on-demand official AvatarSample_B, Seed-san and Twist downloads through the
  same importer/automatic-fit transaction as local VRM files. Each reviewed public
  file has `public/characters/*-provenance.json`; generated notices preserve model
  conditions and Seed-san credit. [Model guidance](models.md) describes selection
  and limits. Preset availability does not resolve Twist fitting or approach contacts.
- [Outcome sequence](plan.md#outcome-sequence) routes M3 browser quality, M4 advanced
  expression/platforms and M5 production outputs. WAV preview remains the audio
  priority; expanded video/interchange output stays deferred.

Start with [MIDI interpretation](../specs/architecture/midi-interpretation.md),
[calibration](../specs/architecture/calibration.md) and the
[engine boundary](../specs/architecture/engine-contract.md).
[Ergonomics/sticking research](research/ergonomics-and-sticking.md),
[source provenance](research/references.md), and
[motion adoption gates](research/motion-adoption-gates.md) retain rationale.

Earlier [browser performance](verification/browser-natural-performance.md),
[groove-hand correction](verification/groove-hand-roles.md) and
[Pages batches](verification/pages-browser-performance.md) retain their tested
commits and original limitations. [Documentation simplification](verification/documentation-simplification.md)
preserves the structural decisions. Historical context remains in the
[requirements origin](context/source-conversation.md) and
[pre-refactor handoff](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/handoff.md).
The [release policy](release-policy.md) requires explicit approval for each later
batch; this index does not grant future deployment permission.
