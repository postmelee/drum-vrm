# Bundled character preset verification

## Scope and provenance

The owner requested on 2026-10-26 that the previously local test VRMs be
selectable from a dropdown and published on Pages. The preceding public batch
`2f9ca06cfa9fa19b493608e3e8f6881ac41257dc` did not include character VRMs.
This change is scoped to three unchanged official sample artifacts; it does not
publish other user models or change their licenses to MIT.

`public/characters/*-provenance.json` owns the exact source commits, original
hashes, embedded rights settings, notices and the distribution decision. All
three permit redistribution. Seed-san requires credit. AvatarSample_B retains
additional VRoid sample conditions, including restrictions on paid sample
redistribution and character-creation-service reuse. The application performs
transient posing; it does not generate or export modified avatar files.

## Implementation boundary

- Character offers the native skeleton and three samples. Model bytes download
  only after selection, from the application's own deployment base.
- Size and SHA-256 must match provenance before the existing VRM 1.0 preflight,
  semantic adapter, automatic fitting and staged adoption run.
- Download errors/cancellation retain the committed character; the UI returns
  to its previous selection. Local VRM import remains available.
- Model creators and source/license/provenance links are visible in Character.
  Language changes preserve the model, score and playback position.
- No Rust planner, source MIDI times, fitting bounds or collision tolerances
  changed. Fresh-session project restore retains its existing file-relink policy.

## Checks

- 207 tests across 28 files passed; TypeScript and production build passed.
- Preset tests use all three real public binaries and the actual VRM inspector.
  They cover deployment-base URLs, size/hash mismatch, HTTP errors, unknown IDs,
  and cancellation before/after download.
- `npm run licenses` verifies every bundled model/legal-file hash and compares
  recorded rights against the binary's embedded VRM metadata. Generated notices
  include the full license, original source notices and additional sample terms.
- Documentation check and `git diff --check` passed. Independent review found
  no blocking selector, state-retention or provenance regression.
- Actual local browser: import AvatarSample_B from the dropdown, prepare and
  play the 30-second Studio demo, pause/seek to 12.5 s, switch to Seed-san, then
  switch English to Korean. The score retained 193 notes/193 hits and 12.5 s;
  both models reported an applied automatic fit. Seed-san attribution appeared.
- Twist loaded through the same selector and displayed the existing bounded-fit
  failure, retaining the prior kit and the 12.5 s position. The dropdown labels
  this sample's fit limitation; import success is not claimed as fit success.

## Remaining limits

Preset distribution makes testing easier; it does not complete #17/#18/#23.
Twist automatic fitting, remaining tom/cymbal approach contacts and visible
skin/clothing interference retain their existing evidence and follow-up scope.
See [equipment contact evidence](equipment-contact.md), [model guidance](../models.md)
and [rig contract](../../specs/architecture/rig-contract.md).
