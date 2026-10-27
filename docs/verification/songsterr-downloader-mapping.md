# Songsterr Downloader MIDI compatibility

Date: 2026-10-27 (Asia/Seoul). Scope: [issue #60](https://github.com/postmelee/drum-vrm/issues/60).
Implementation tested: `b7fb49dbec95db6e654e8488343f794e016d9801`, based on
`d8055d06bc2eef085f7bebfe40bed442c32700c8`, in an isolated worktree.
This records local verification, not a public deployment or complete-motion claim.

## Source and interpretation

- Songsterr Downloader:
  [converter at 1a2c0b4](https://github.com/Metaphysics0/songsterr-downloader/blob/1a2c0b42faf716cb609ee7c0fbb855d32ba4a4a7/src/lib/server/services/converter/songsterr-to-alphatab.converter.ts),
  [lockfile](https://github.com/Metaphysics0/songsterr-downloader/blob/1a2c0b42faf716cb609ee7c0fbb855d32ba4a4a7/bun.lock),
  [MIT license](https://github.com/Metaphysics0/songsterr-downloader/blob/1a2c0b42faf716cb609ee7c0fbb855d32ba4a4a7/LICENSE).
- alphaTab 1.8.1:
  [articulation lookup](https://github.com/CoderLine/alphaTab/blob/8a2102fe7ee7dcf4b50c3a4198fbb8c0d5c0fda3/packages/alphatab/src/model/PercussionMapper.ts),
  [MIDI generator](https://github.com/CoderLine/alphaTab/blob/8a2102fe7ee7dcf4b50c3a4198fbb8c0d5c0fda3/packages/alphatab/src/midi/MidiFileGenerator.ts),
  [MPL-2.0 package declaration](https://github.com/CoderLine/alphaTab/blob/8a2102fe7ee7dcf4b50c3a4198fbb8c0d5c0fda3/packages/alphatab/package.json).

The converter puts GP7 indices into note.percussionArticulation, without filling
track.percussionArticulations. The MIDI generator first looks up that field as an
index in the track list, then as an ID in its global map. Missing IDs fall back to
the raw value. This explains the affected MIDI without treating it as a new SMF
format or assuming all Songsterr exports use these numbers.

An independently authored seven-note alphaTab 1.8.1 experiment reproduced outputs
`0,3,5,8,12,22,87`. Attaching the round-tripped articulation definitions, or using
the corresponding normal IDs, produced `38,42,46,36,45,49,41`.
The fixed [mapping contract](../../specs/architecture/midi-interpretation.md#songsterr-downloader-compatibility)
defines 21 supported drum outputs and deliberate unsupported cases. There is no
automatic exporter detection or GM fallback.

Only factual number/role correspondences are independently implemented in Rust.
Neither upstream source, runtime, test songs nor soundfonts are distributed.
alphaTab was installed only in a temporary external research directory. Product
dependencies and licenses remain unchanged.

## Executed checks

Environment: macOS, Node 24.15.0, repository-pinned Rust toolchain.

- `cargo fmt --all`; `cargo fmt --all -- --check`.
- `cargo test --workspace --locked`: 153 passing.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passing.
- `npm run build:wasm -- -- --locked`: rebuilt from this implementation.
- `npm test`: 253 passing across 37 files.
- `npm run build:web`: typecheck and Vite production build passing.
- `npm run docs:check`, `npm run licenses`, and generated-notice diff check.

WASM SHA-256:
`6fd89f4eca5945d3ad72fdc46e41ad69ba8c8ee8496912f1b0ae65b189de6b95`.
Vite retains its existing large-chunk warning; it is not a build failure.

New independently authored fixtures cover format 0/1, explicit selection, default
GM rejection, all supported outputs and unsupported half-open/choke/GM-fallback
cases, original event identities/keys/velocities, tempo-change timing, note-offs
and zero-velocity note-ons, plan tampering, project round-trip and shuffled poses.
Existing GM/Logic tests remain unchanged and passing.

## Private file and browser checks

The two owner-supplied files were read locally; no source bytes, converted files,
scores, recordings or screenshots are committed. Only aggregate results follow.

| Check | Combined format 0 | Separate named format 1 |
| --- | --- | --- |
| PPQ | 960 | 960 |
| Selected tracks (zero-based) | 0 | 5 |
| Selected channel (user-facing) | 10 | 10 |
| Positive drum note-ons / interpreted hits | 1,513 / 1,513 | 1,513 / 1,513 |
| Duration | 212.761732 s | 212.761732 s |
| Unmapped notes | 0 | 0 |
| Default GM planning | No supported hits | No supported hits |

Original file bytes were unchanged. Every interpreted hit retained its source
event ID, raw key, velocity and integer microsecond time. The per-key counts were
0:339, 3:173, 5:194, 8:532, 12:26, 22:242, 87:7.

In the actual in-app browser at a localhost Vite instance:

1. The separate-track file failed under GM, then opened MIDI settings. Only Drums
   was suggested, with channel 10.
2. Selecting the compatibility profile and applying prepared 1,513 hits.
3. Playback and pause controls worked. Importing the combined file retained the
   explicit profile, selected channel 10 and prepared the same 1,513 hits.
4. Seek input and English/Korean switches worked. The prepared summary and piano
   roll retained raw MIDI numbers with interpreted drum labels. No console errors
   were recorded.

Both browser runs used the automatically fitted native rig (90% scale). They
reported **0 unmapped and 2 unassigned motions**. Successful mapping does not fix
these planning limitations or establish natural posture/collision clearance.
This increment adds no URL fetcher, backend, merge or Pages publication.
