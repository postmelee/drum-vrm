> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/rig-motion-baseline.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../specs/README.md) for current contracts and policy.

# Rig motion baseline and wrist/tool comparison

Date: **2026-10-24**. Issue: **#30**. Status: **implemented/unreleased working-tree measurement**, after repository base `217de933864ae47fd96afc31a67f62341b2dc612`. This does not record a deployment. The source hashes below identify the measured files even before the enclosing PR receives its final commit.

## Purpose and measurement boundary

Compare the retained `PrimitiveRigAdapter` with `Performance.from_json()` against `NativeHumanoidRigAdapter` with `Performance.from_json_with_profile()`, using the exact same independently generated MIDI bytes. The legacy adapter was put under its mirrored display root; the native rig used a positive-scale parent. Measurements use the actual renderable stick cylinder endpoint and rendered wrist sphere (legacy) or raw hand bone (native), after pose application and world-matrix updates. They do not simply read the planner's target coordinates.

This is a headless transform audit. It does not capture GPU pixels, run a video encoder, measure skin/shoe collisions, or claim human-performance realism. No third-party avatar, motion, recording or MIDI was used.

## Reproducible input and method

- Fixture: [`makeDemo(8)`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/src/demo.ts), 8.000 seconds, **52 immutable hits**, selection `tracks=[1], channel=9`.
- MIDI SHA-256: `136ed03100b56283181d6fd6accb373d8991ec12664e849ada878fe1d12785e0`.
- Reconstruct the same source plan through `from_json()` for the legacy branch. For the native branch, pass the source plan and `NativeHumanoidRigAdapter.getProfile()` to `from_json_with_profile()`.
- Sample song time across `[-0.5, 8.25]`, including preroll and tail. Derivative centers use a 120 Hz grid excluding its endpoints, plus each exact hit time: **1101 centers**, including repeated simultaneous-hit times.
- Central-difference step `h = 1/120 s`: `v(t) = (p(t+h)-p(t-h))/(2h)` and `a(t) = (p(t+h)-2p(t)+p(t-h))/h²`. Report the largest Euclidean norm over these samples for each hand/point. Values include impacts and trajectory-phase boundaries; they are not filtered.
- Read the cylinder's positive local-Y endpoint using `geometry.parameters.height/2`, transformed through its actual `matrixWorld`. For the native rig, separately compare that rendered endpoint with the hand-local contact marker.
- Check exact assigned hit times against the unchanged native-world target `[-target.x, target.y, target.z]`.
- Re-evaluate nine absolute times in a shuffled order and compare the actual tip and wrist positions. Probe hit/preparation/rebound boundaries at `t±0.00001 s` to screen for visible position jumps.

The executed private script and full numeric report are `.local/rig-motion-baseline.mjs` and `.local/rig-motion-baseline.json`; both remain ignored. The method and source/input identities are retained here so the audit can be recreated without those local files.

## Results

| Measured point | Legacy peak speed (m/s) | Native peak speed (m/s) | Legacy peak acceleration (m/s²) | Native peak acceleration (m/s²) |
|---|---:|---:|---:|---:|
| LH wrist | 5.499 | 4.713 | 819.014 | 477.782 |
| LH tip | 5.499 | 5.499 | 819.013 | 775.975 |
| RH wrist | 7.122 | 6.881 | 865.574 | 474.235 |
| RH tip | 7.122 | 7.122 | 865.574 | 846.025 |

All sampled positions, velocities and accelerations were finite. The large acceleration peaks occur around hits in the final fill: legacy LH at 7.000 s, legacy RH at 7.250 s; native LH wrist at 7.125 s and native RH wrist at 7.250 s. These are numerical baseline values for the current piecewise trajectories, not physiological thresholds or a smoothness certification.

| Contract check | Legacy | Native |
|---|---:|---:|
| Total preserved MIDI hits | 52 | 52 |
| Assigned hits / assigned hand hits | 52 / 43 | 51 / 42 |
| Maximum actual stick-tip error at an assigned hand hit | 3.016e-08 m | 4.146e-08 m |
| Actual tip/wrist difference after shuffled seek | 0 m | 0 m |
| Maximum displacement across a `±10 µs` boundary probe | 0.131216 mm | 0.133080 mm |
| Maximum wrist-to-tip direction change from rest, LH / RH | 0.000021° / 0.000023° | 39.998° / 39.987° |

Native rendered cylinder endpoint versus its hand-local contact marker differed by at most `1.570e-16 m` in the supplementary sampled frames. Both paths preserved every MIDI identity, timestamp, velocity and target field. The native plan leaves the initial crash (hit 1, note 49, time 0) unassigned because the shorter measured native arms cannot reach it; its MIDI event and audio remain present.

The legacy wrist-to-tip vector is constant apart from floating-point noise, so wrist and tip speeds/accelerations almost coincide. The native path rotates the hand/tool frame with the stroke phase, producing about 40° of direction change and separating wrist from tip motion. This establishes that the new wrist/tool rotation affects the actual rendered stick. It does not establish a physically simulated whip, elastic drum response, or expert-quality wrist-led technique.

The native fixture also has different segment lengths, seat position and assignments. Consequently the table is a before/after regression baseline, **not an isolated causal experiment** showing that wrist rotation alone reduced acceleration. Positions were continuous to the measured boundary-probe resolution, but velocity/acceleration continuity was not proven. Future motion work should retain the original contact times, compare equivalent assignments and profiles, and measure these peaks again alongside visual review.

## Additional regression coverage

`tests/rig-vrm.test.ts` now explicitly verifies that absent optional finger bones do not prevent a VRM rig from using a neutral hand socket and producing valid actual tip contacts. The same authored fixture suite covers two proportions, nonidentity raw rest bases, actual node-constraint updates, arbitrary-seek repeatability, first-person head-triangle erasure, camera layers/model visibility and nonuniform-scale rejection. It contains no downloaded model data.

Commands run after these additions:

- `node .local/rig-motion-baseline.mjs`: passed finite/contact/immutable-event/seek/position-probe checks.
- `npm test -- tests/rig-vrm.test.ts`: **5 tests passed**.
- `npm run typecheck`: passed.

No production code was changed for this measurement.

## Measured source identities

| File | SHA-256 |
|---|---|
| `crates/drum-engine/src/motion.rs` | `948348c392ac7c567f4283a142f910624fd5a3b3d75f1c392b1e6b2a55082046` |
| `crates/drum-engine/src/rig.rs` | `6844594453c5af6e31a6cdb08f8777859a2740c22f99c721d485da630a6d824f` |
| `src/scene.ts` | `c878824e6cc01534b82d2dad71592a82b8cec967b42af296d7caf4b53e70731f` |
| `src/rig-humanoid.ts` | `ca00a693d13d58d1c346d8c50010e629b3def2c33ed47f319dbf50fb0ee7729d` |
| `src/wasm/drum_engine_bg.wasm` | `368f312a03654eba90eb208818df9ac35f2e1cd18afb434df8f61e6d3200e8a4` |
