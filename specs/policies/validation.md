# Validation and evidence policy

## Choose checks by the changed boundary

Run meaningful checks appropriate to the change. A documentation-only migration needs
link/anchor, ownership, language, length, diagram and generation checks; it does not
claim new MIDI, model or media validation. Runtime changes require the applicable
unit/integration/browser checks. Rebuild WASM after Rust engine changes.

Preserve source-event timing, measured limb lengths, rendered contacts and absolute-time
repeatability when changing planning or rig behavior. Use authored public fixtures;
private acquired models may supplement local evaluation without becoming repository
or release assets.

## Keep evidence levels separate

| Evidence | Establishes | Does not establish by itself |
| --- | --- | --- |
| Unit/property checks | The exercised numeric/input contracts | Natural-looking motion on every skin |
| Render-transform harness | Actual attached markers/raw-bone transforms and repeatability | Pixel appearance, skin collision or encoded media |
| Browser workflow | Observed UI/import/play/seek behavior in that environment | Broad browser/platform compatibility |
| Encoder/mux completion | That code path produced a reported encoded result | Independent playback of the downloaded file |
| Downloaded-file inspection | Measured tracks, frames, decode/onset for that file | Every strike's perceptual realism or another build |
| Public-URL check | Observed deployed artifact/path | Another commit's correctness |

## Record a result

A report identifies the measured commit or exact source hashes, fixture identity,
environment, commands/method, results and untested scope. Distinguish a documentation
baseline from the code actually tested. If the latter was not captured, say so rather
than filling it from a nearby commit.

Retain historical test counts and values in the report. New checks create a new
record linked to the prior one. Do not copy the latest count into README, ADRs and
handoff notes or replace the old measurements with today's result.

Report unassigned hits and preserved audio separately from successfully performed
contacts. Tiny tool error does not prove safe joint angles, no collision, correct
shoe contact or a natural grip. Full body-proportion experiments are not replaced
by unrelated example characters.

## Completion

Issue acceptance criteria own completion. Link executed evidence and remaining work
from the issue/PR. A document describing a procedure is not evidence that it was run.
A build pass is not a deployment. Follow the [release policy](releases.md) separately.
