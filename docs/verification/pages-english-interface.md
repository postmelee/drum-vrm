# English interface and Pages batch

## Scope and approval

The owner requested English as the website's initial language, a simple Korean
switch, language-specific README links, then merge and Pages publication. The
same instruction explicitly defers unresolved tom/cymbal approach contacts and
Twist automatic fitting until after publication. No release/tag is authorized.

The batch combines the permission-cleared full-song demo [#47](https://github.com/postmelee/drum-vrm/pull/47),
equipment/contact/count-in [#48](https://github.com/postmelee/drum-vrm/pull/48), and
English UI [#50](https://github.com/postmelee/drum-vrm/pull/50).
Issue [#49](https://github.com/postmelee/drum-vrm/issues/49) owns language acceptance.

## Candidate and validation

- UI head: `28b2c246fc24ca9236283a825bf73256569cbca9`.
- Integrated main: `2f9ca06cfa9fa19b493608e3e8f6881ac41257dc`.
  The trees are identical (`git diff --exit-code` passed).
- [UI CI run 40281095091](https://github.com/postmelee/drum-vrm/actions/runs/40281095091)
  passed. Local TypeScript/production build, 199 tests across 27 files, and the
  documentation check (87 files, 801 links, 22 bilingual pairs) passed.
- Locale tests cover initial/default/URL precedence, literal user text, live DOM
  bindings, cache eviction, repeated message composition, and Worker/engine
  errors. A source-coverage test checks all 113 current Rust message templates.
  Rust plans and diagnostic evidence are preserved; translation is a UI adapter.
- Local production preview in the Codex in-app browser: prepare the Studio demo,
  switch EN/KO, retain a paused 12.5 s position, and switch during playback.
  The actual engine diagnostic and timeline accessibility summary translated.
- With a local VRM and WAV loaded, switching retained the active model, score,
  paused position, model selection and WAV offset. Source filenames and model
  metadata were preserved; labels and audio guidance translated. No console
  errors were observed. The local files are not part of this distribution.

## Pages publication

[Pages run 40281668719](https://github.com/postmelee/drum-vrm/actions/runs/40281668719)
succeeded for `2f9ca06cfa9fa19b493608e3e8f6881ac41257dc`. Its build repeated Rust
format/tests/lint, rebuilt WASM and passed browser tests, notices, documentation
and production checks before publishing.

- [English app](https://postmelee.github.io/drum-vrm/?lang=en)
  and [Korean app](https://postmelee.github.io/drum-vrm/?lang=ko) were verified in
  the actual public browser. The UI switch also updates the URL in place.
- All 23 served non-source-map files matched the workflow artifact byte-for-byte:
  HTML, CSS, JavaScript/Workers, WASM, models/provenance/licenses and demo files.
- Artifact ID: `12632721048`; archive digest:
  `sha256:2654beeebc06c4ed852143d39aa638f7acfda8871fcbdcec95126b3e67dd1d90`.
- Shipped WASM SHA-256:
  `688482e465bd9c2e18d6124caf6b8b776a678cfd44c42db95c6cceeac4d73a37`.
  This is the CI artifact hash, not an assumption that local toolchain output is identical.
- Public full-song demo: 1,760 source notes and 1,720 arranged hits; prepare,
  seek, play, pause and EN→KO→EN switching passed. The paused 24.000 s position
  and demo selection survived switching. The original public MIDI filename
  stayed unchanged while the demo label and timeline descriptions translated.
- No browser console errors were observed. No release/tag was created.

## Remaining limits

[Equipment/contact evidence](equipment-contact.md) retains the measured native
approach contacts and failed Twist bounded fit. The UI translation does not
change fitting, motion, MIDI timings, or collision tolerances. Keep #18/#20/#23
open; static feasibility and tip contact do not certify dynamic skin/clothing
clearance. Random strike variation remains deferred.

PR #42 was closed as superseded: its original deployment evidence is already
preserved by #45. Merging its stale README/handoff edits would have restored an
older public-version description. Earlier [body-clearance publication](body-clearance-measurements.md#pages-publication)
and [browser-performance publications](pages-browser-performance.md) retain their
original measured scope.
