> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/palm-orientation-unreleased.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../specs/README.md) for current contracts and policy.

# Palm orientation and reference wrist alignment

2026-10-24, local macOS / Chrome. Follow-up on [PR #34](https://github.com/postmelee/drum-vrm/pull/34), not a release or Pages deployment. Previous implementation/verification is preserved in [the calibration record](calibration-644fe185.md); its video-encoding evidence predates this follow-up.

## Change and checks

The previous identity hand-to-tool frame kept T-pose hand axes sideways while the forearms reached forward. The new common adapter measures a diagonal hand-local stick mount, aligns the reference hand with the seated forearm, and carries palm roll in the forearm. Up is the owner-requested default; inward and down are also available. The actual stick remains parented to the raw hand. No Rust logic, WASM schema, dependencies or external assets changed.

- Web tests: **79 passing across 14 files**; typecheck, production build and `git diff --check` passed. Focused rig tests cover palm directions, neutral reference wrists, fixed lengths, parented contacts, repeated seeks, arbitrary raw rest bases, and degenerate optional fingers.
- Actual app: selected inward before importing Twist; selection survived import and automatic fitting. Changed up/down during the prepared 193-hit demo; playback paused, diagnostics refreshed, and up survived seat 60cm / forward 12cm and seeking to 12.5s. Up left 10 unassigned events; down left 126 with that kit/pose, so orientation selection materially affects reach and must not hide diagnostics.
- The real-file browser harness loaded Seed, AvatarSample_B and Twist through the actual importer. Each evaluated 128 fixed output frames and shuffled frame order. All passed finite/positive transforms, immutable event data, attached contact and deterministic seeking; all assigned 11/14 fixture events. Maximum assigned contact errors: Seed `4.039e-7m`, Avatar B `1.665e-7m`, Twist `1.316e-7m`; maximum frame-marker error `4.590e-7m`, seek matrix delta zero.

## Wrist measurements and limits

A separate private harness uses extracted real nodes/constraints from the same three files, with the same five-hit MIDI fixture before and after. All three palm modes on all three bodies (nine cases) passed the contact, joint-position, fixed-length and seek contracts. Maximum contact error `4.336e-7m`, raw-joint error `3.361e-7m`, length error `1.852e-7m`, random-seek matrix delta zero. Reference hand-longitudinal/forearm angle improved from **114–122° to below 0.000007°**. Chosen palm-direction dot product was at least `0.842`; this is the upward/inward/downward direction projected perpendicular to the hand, not a claim of an exactly horizontal palm.

**Dynamic wrist limits are not solved.** At the left-hand hi-hat contact at song time 0.5s, palms-up hand/forearm angles remain approximately **99.08° (Twist), 104.58° (Seed), 109.90° (Avatar B)**. The tool orientation is still a calibrated reference plus stroke pitch. This is a real unresolved reach-pose problem despite near-zero tip error; #20 retains target-dependent tool orientation, wrist constraints and crossed-arm clearance. It must not be described as natural wrists throughout a performance.

The Twist rest-pose close-ups visibly show the upward-facing grip and straighter wrist; they do not certify collision-free skin, glove thickness or anatomical finger limits. Symmetric palm-up is not the asymmetric finger arrangement of traditional grip. The [research note](../../tech/research/2026-10-24/rig-and-studio-audit.md#palm-orientation-correction--2026-10-24) links the instructional source and distinguishes these terms.

Private VRMs, extracted metadata, reports and screenshots stay in ignored `.local/`. No new video was encoded for this follow-up: fixed-frame pose evaluation was tested, not a new media file. Existing large-chunk build warning remains.
