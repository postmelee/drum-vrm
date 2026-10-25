# MIDI interpretation, fitting and audio verification

Measured on 2026-10-25 against `codex/midi-interpretation-foundation`, based on
`d8dfab38cefc3bffd6edc0b88643d02ff829af73`. The PR records the final tested commit.
These are local candidate results, not a release or Pages deployment.

## What changed

The [MIDI interpretation contract](../../specs/architecture/midi-interpretation.md)
owns source → mapping → intent → control. It retains selected channel messages and
original timing, adds explicit Logic DKD GM/count-in/short-open-close choices, and
separates left/right crash contacts. Body fitting and phrase assignment consume that
interpretation; neither changes the original events to hide an unreachable strike.

The common hand solver uses measured forearm/tool frames. Explicit kit fitting
adjusts hand-played surfaces together while keeping kick and pedal contacts fixed.
Phrase lookahead, same-instrument ready positions and velocity-dependent lift/pitch
replace the previous isolated-hit behavior. Uploaded WAV replaces synthesized audio
on the shared preview/export timeline.

## Evidence and reproducibility

Environment: macOS, Node 24.15.0, Rust 1.94.1, Chrome 155, pinned repository dependencies.
Public automated fixtures are independently authored; no uploaded MIDI, WAV or VRM
is committed. Private input hashes, model metadata and detailed measurements remain
under ignored `.local/` or the temporary directory.

| Check | Result and scope |
| --- | --- |
| Public interpretation pattern | 91 positive note-ons remain visible: 82 strikes, 6 close controls, 3 count-in cues. Includes two crashes, rim-edge classification and accented/occupied-hand rolls. |
| Original private song | 1,776 positive note-ons → 1,731 strikes, 42 controls and 3 count-in cues; zero unmapped. One simultaneous three-hand demand remains explicitly unassigned after fitting. |
| Owner-corrected private song | 1,774 positive note-ons → 1,729 strikes, 42 controls and 3 count-in cues. All six fitted profiles report zero unmapped, unassigned or diagnostics. The owner corrected the MIDI; no song-specific exception or suppressed event was introduced. |
| Whole-song common rig audit | Native, three real model hierarchies and two controlled arm proportions preserve all source timestamps and produce identical shuffled-seek poses/markers. Numerical fitting/contact results are separate from rendered skin inspection. |
| Source and import validation | Raw selected messages, mapping provenance, control derivation, schema migration, limb roles, timing and transfer constraints have regression coverage. |
| WAV formats | Actual Chrome decoding passed PCM 8/16/24/32, float32, extensible PCM24 and extensible float32. Unit tests cover bounds, malformed chunks, offsets, trimming and mono/stereo behavior. |
| Independent short output | An authored WAV onset at 0.1000208 s with +0.25 s offset appears at 0.8500208 s after the shared 0.5 s pre-roll in independently decoded output. VP9/Opus, 1280×720, 48 kHz stereo. |
| Full-song rendered output | Actual Seed-san VRM, corrected MIDI, silent H.264 MP4: 5,814 frames, 30 fps, 1280×720, 193.800 s. `ffprobe -count_frames` confirmed those values; a full `ffmpeg -f null` decode completed without errors. A decoded frame was visually inspected against the local preview. |
| Browser controls | Real local MIDI/VRM import, explicit fitting, seek, WAV offset, WAV removal and malformed replacement passed without page exceptions. An initial test-harness attempt to transfer a large blob as a number array exhausted Node memory; using the browser's normal download path succeeded. |
| Additional rendered VRMs | Twist sample and AvatarSample B loaded, fitted and prepared with zero unmapped/unassigned hits. Close/rear-oblique views showed curled fingers and approximately straight visible impact wrists on the twist sample. Avatar B's cuffs/hair obscure the grip; these views do not establish skin clearance. |
| Automated checks | Rust 54 tests, web 106 tests, formatting, clippy, pinned WASM rebuild, typecheck/production build and generated-notice comparison passed. The build retains the existing large-chunk advisory. |
| Documentation | Paired links and authored-document limits checked; eight Mermaid diagrams rendered and inspected with Mermaid 11.12.1 in Chrome 155. |

The original-input measurement is intentionally retained. Replacing the private
source file does not retroactively turn its impossible event into a successful plan.
The corrected case is a separate input and result.

## Contact and posture limits

The original-input six-profile fit audit measured assigned stick-tip contact below
`4.84e-7 m`, impact wrist swing below `0.027°`, and zero differences after shuffled
evaluation. The remaining unassigned event was the same three-hand collision in
every profile. Fits used common numeric geometry; no avatar-name branch or bone
stretching was added.

Repeating on the corrected input measured maximum strike-contact error
`4.8371e-7 m`, impact wrist swing `0.026586°`, and in-motion wrist swing `18.292°`.
All six fitted profiles had zero unassigned hits. Without fitting, the smaller and
different-proportion profiles still had unreachable contacts, so the explicit fit
step remains material to these results.

Short-arm transfers still reached approximately 71–82° per 60 fps frame despite
continuous motion. Dense checks around the worst intervals did not reveal a branch
jump, but continuity does not establish comfortable movement. `complete` in a fit
proposal certifies preferred-hand static reach only. It does not certify shoulder
limits, forearm/stick separation, skin clearance or naturalness throughout a phrase.

Other explicit limits:

- Rimshot, rim-edge and sidestick are identified, but currently share simplified
  snare tip contact. Shaft/rim dual contact is not implemented.
- Rendered crossed-arm regions remain crowded; a stick shaft can intersect a
  tilted cymbal even when its tip reaches the target. Surface-normal and clearance
  constraints remain open in T07/T08 rather than being counted as a contact pass.
- Short open/close controls drive the existing ankle approximation. Cymbal gap,
  heel/sole support and a physical pedal linkage remain future work.
- Count-in events are retained cues; they do not yet animate a stick click.
- Velocity influences basic motion and synthesized sound. It is not a complete
  push-pull/Moeller technique or an inferred emotion.
- WAV synchronization is an explicit offset. There is no automatic beat alignment,
  time stretching, VST hosting or reproduction of Logic's sampler.
- Session state is not a saved project. Versioned reopen/asset relinking remains
  [#15](https://github.com/postmelee/drum-vrm/issues/15).

## Local visual review

Run the README's local build/dev commands and use **Upload / performance →
Interpretation check** (`연주 검증 패턴`) for a redistributable comparison. Inspect
the two crashes, accented snare runs and blue retained hi-hat control notes.

For a private Logic export, explicitly select **Logic · Drum Kit Designer (GM)**.
Enable note-75 count-in only when that is the file's intent. Enable short-open-close
interpretation only when intended, then prepare again. Import a local VRM and use
the explicit kit-fit button when needed. Diagnostics link to the original song time;
the transport includes 0.5 s of pre-roll.

Review playback, arbitrary seek and the encoded result separately. Zero mapping or
assignment errors is an event-accounting milestone, not a claim of perfect playing.
