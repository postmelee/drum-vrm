# Pages deployment: browser performance batch

Measured on 2026-10-26 (Asia/Seoul). This record concerns one deployment, not a general compatibility guarantee.

## Approved scope

- The owner explicitly approved merging [PR #40](https://github.com/postmelee/drum-vrm/pull/40) and [PR #41](https://github.com/postmelee/drum-vrm/pull/41), then publishing Pages only. See [approval history](../approval.md).
- Deployed commit: `045b78aa56f00709471359d8e861c420222bda93`. Its complete tree equals verified candidate `5d5d7ae6363f4562209bc5c94f4e2860d8d3903d` (`git diff --exit-code` passed).
- Public URL: <https://postmelee.github.io/drum-vrm/>.
- [Pages run 40223986570](https://github.com/postmelee/drum-vrm/actions/runs/40223986570) succeeded, including Rust formatting/tests/lint, locked WASM build, browser tests, documentation/notices checks, typecheck, production build and deployment.
- No version bump, release tag or GitHub Release was created.

## Public verification

The downloaded Pages artifact contains 23 files. Public HTML, CSS, WASM and all four JavaScript bundles were fetched and compared byte-for-byte with that artifact; all seven comparisons passed.

| Public file | SHA-256 |
| --- | --- |
| `index.html` | `f21983c6673afa57dd352d87c7615996633466f1c1ac11391081af35177f60b6` |
| `assets/drum_engine_bg-QirNewUT.wasm` | `b8b2744423adb7e7fd0a17b2df61252d297629486e8867f11ba186a1828b1da2` |

The deployed WASM hash differs from the local build recorded in [implementation verification](browser-natural-performance.md). Source equality does not establish binary equality across builds. Local projects require the exact engine hash for replay; this check does not certify a project saved against the earlier local binary as portable to Pages.

In the macOS in-app Chromium browser, a fresh public tab loaded the 3D scene and prepared the authored 30-second demo: 193 original notes, 193 planned strikes, zero unassigned hits. Play advanced the clock; pause and seek to 12 seconds updated the clock and piano roll. The upload panel exposed the new mapping/intent controls, WAV input/offset, project save/open and inspection controls. No private MIDI, WAV or VRM was loaded for this public smoke test.

## Remaining review

This deployment does not repeat every local test. [Implementation evidence](browser-natural-performance.md) retains the benchmark, real-avatar contact checks and WAV/project reopen results. Fast fill transfers and arm/stick crossings still need visual refinement; contact accuracy is not proof of natural posture or skin clearance. Mobile browsers, all avatar proportions and full-length video output are not newly certified here.

Next, the owner should review the private benchmark and representative VRMs on Pages, apply explicit body fitting where needed, and compare WAV alignment. Keep observed defects in the existing M3 issues; deployment does not close their outstanding acceptance criteria.
