# Drum VRM

**English** · [한국어](README.ko.md)

A browser tool that turns drum MIDI from a DAW into video of a 3D drummer. Rust/WASM parses MIDI and plans motion; Three.js renders an extensible reference rig.

**v0.2.0 is deployed · [Try the app](https://postmelee.github.io/drum-vrm/) · [Release](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0)**

File upload through video download with audio has been verified in Chrome 155 on macOS. The [verification record](docs/verification/studio-0.2.md) documents the environment, measurements, and untested limits.

## MVP

`Upload MIDI → analyze and plan → display the reference rig → play → download video`

- Skeleton with joints, bones, and sticks; three procedural drum styles and an Innerscene CC0 acoustic kit.
- White/black workspace, four settings menus, and a piano roll with bars, beats, and notes.
- Front-facing drum editing, part selection, color/scale/rotation, and local GLB appearance replacement.
- Kick, snare, hi-hat, toms, crash, and ride with a basic right-hand-led plan.
- Tempo-preserving hit times, hand/foot assignment, preparation/contact/rebound, and diagnostics for unplayable hits.
- Play, pause, reset, seek, synthesized drum sound, and camera/background controls.
- Fixed-frame 1280×720, 30 fps video with synthesized audio; MP4 or WebM depending on the available codecs.
- Selected MIDI is processed in the browser without automatic transmission or repository upload.

WAV synchronization, VRM/humanoid character replacement, editing, project persistence, higher resolutions, and 3D motion export remain M3/M4 backlog items.

## Usage

1. Open **업로드 · 연주정보** (Upload / Performance) and choose **30초 데모** (30-second demo) or a `.mid`/`.midi` file.
2. Review tracks, channels, and analysis messages after preparation. After changing the selection, press **선택한 트랙으로 다시 준비** (Prepare selected tracks).
3. Press **재생** (Play). Seek on the timeline and adjust the camera with dragging, scrolling, or the viewpoint buttons.
4. Open **내보내기** (Export), press **영상 추출** (Export video), and save the completed download. Synthesized drum audio is included by default.

The app does not reproduce the source DAW's instrument sound. Export includes a 0.5-second preparation interval and a 0.25-second tail. An exactly 30-second song produces 923 frames, approximately 30.7667 seconds.

Input supports SMF formats 0/1 with PPQ timing: up to 10 MiB, 300 seconds across selected tracks, 512 tracks, 200,000 total events, and 50,000 selected supported hits. SMF 2, SMPTE timing, and malformed files are rejected with a reason. Unmapped notes are reported; supported hits the rig cannot perform retain their original audio time and target highlight.

## Model formats

Drum parts accept a self-contained static `.glb` file (GLB 2.0, up to 25 MiB) with embedded textures. External URLs/separate resource references and skinned models are rejected. Arbitrary models are fitted approximately; exact contact between the target and visible surface requires calibration. Changing appearance does not change the synthesized sound. Convert GLTF, FBX, OBJ, or BLEND assets to GLB first.

Character upload is not implemented. The current character control shows/hides a reference skeleton; VRM 1.0 and humanoid GLB adapters are tracked in [#17](https://github.com/postmelee/drum-vrm/issues/17)/[#18](https://github.com/postmelee/drum-vrm/issues/18). Drum appearance settings last for the current session; project persistence is [#15](https://github.com/postmelee/drum-vrm/issues/15).

## Local development and checks

Use Node.js 24 or newer, Rust 1.94.1 from [rust-toolchain.toml](rust-toolchain.toml) with the `wasm32-unknown-unknown` target, and wasm-pack 0.15.0. npm and Rust dependencies are pinned in `package-lock.json` and `Cargo.lock`.

```sh
npm ci
cargo install wasm-pack --version 0.15.0 --locked
npm run build:wasm
npm run dev
```

Open `/drum-vrm/` on the local URL printed by the server. `src/wasm/` is generated and is not committed. Rebuild it after Rust changes with `npm run build:wasm`.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm run typecheck
npm test
npm run licenses
npm run build
npm run preview
```

These commands describe reproduction steps, not evidence of a successful test run. `npm run licenses` compares installed npm packages, the Cargo lock graph, and original license texts; fetch or build the locked Rust dependencies first. Production output is `dist/`, and Vite's base path is `/drum-vrm/`.

## Environment and limitations

The initial verification target is desktop Chromium with WebGL, WASM, Web Audio, and WebCodecs. Export requires HTTPS or localhost and depends on the codecs the browser/OS actually provides. MP4 (H.264/AAC), WebM (VP9/Opus), and WebM (VP8/Opus) are probed with the real configuration and a short export, including duration and decoded audio onset. If AAC delay exceeds the threshold, the verified environment selects VP9/Opus WebM. Audio failure is not silently treated as successful silent output.

The MVP has a fixed reference body and rule-based motion. Dense hits, fast transitions, and unreachable targets are diagnosed; humanlike performance of every MIDI is not guaranteed. PCM and encoded output are kept in memory, so longer songs consume more memory. The five-minute input limit is not a performance guarantee on every device. Safari, Firefox, mobile compatibility, and maximum-length performance need separate verification.

## Documentation and work tracking

User-facing guidance is maintained in English and Korean in the same PR. Existing Korean documents are preserved; internal engineering/research documents may be English only. Proposed features are distinguished from publicly deployed behavior.

| Purpose | Reference |
| --- | --- |
| Resume work | [Handoff](docs/handoff.md), [agent instructions](AGENTS.md) |
| Scope and acceptance | [Requirements](docs/product-requirements.md), [development plan](docs/plan.md) |
| Authorization and releases | [Approval history](docs/approval.md), [release policy / 릴리스 정책](docs/release-policy.md) |
| Engine/rig boundary | [Architecture](docs/architecture.md), [design decision](docs/decisions/0001-engine-and-rig-boundary.md) |
| Original requirements | [Context and topic record](docs/context/source-conversation.md) |
| Research and reuse | [Research/data](docs/research/references.md), [dependencies](docs/research/dependencies.md), [license policy](docs/research/license-policy.md) |
| Execution | [Issues](https://github.com/postmelee/drum-vrm/issues), [milestones](https://github.com/postmelee/drum-vrm/milestones) |

## Releases and licenses

[Pages](https://postmelee.github.io/drum-vrm/) was deployed and its key paths verified on 2026-10-24. The deployed application commit is `3e8be9333cedbc9845dbc3e4cebd73961af697b4`; the [deployment run](https://github.com/postmelee/drum-vrm/actions/runs/40063363035) succeeded. The [release](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0) preserves the actual static artifact and its SHA-256.

Follow-up work proceeds through issues and PRs. **Do not create release tags, publish GitHub Releases, or deploy Pages until the owner explicitly approves a reviewable batch.** Merging a PR or passing CI is not deployment approval. After approval, publish the batch from a verified commit on main using the [release procedure](docs/release-policy.md).

Original source is [MIT](LICENSE). Third-party licenses remain separate; [distribution notices](public/licenses.txt) provide exact versions, sources, and license texts. Mediabunny is MPL-2.0, with a pinned corresponding-source URL in the notices. The Innerscene CC0 drum GLB and embedded textures are distributed with provenance, terms, and SHA-256 in `public/models`. No external character, sound samples, motion datasets, or ML weights are included. See the [drum asset research](docs/research/drum-assets.md). A paper's license is not assumed to cover its datasets or model weights.
