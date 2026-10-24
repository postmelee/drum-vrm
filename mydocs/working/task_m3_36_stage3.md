# Stage 3 — repeatable verification and PR preparation

[Issue #36](https://github.com/postmelee/drum-vrm/issues/36) · [Implementation plan](../plans/task_m3_36_impl.md) · [Machine-readable evidence](../report/migrations/task_m3_36_checks.json).

## Purpose and artifacts

Make the migrated structure checkable, preserve legal provenance and prepare a separate reviewable PR. The checker and notice generator landed with Stage 2 so its document moves had working consumers; this stage adds the CI gate and records integrated evidence. The root diagram now shows the `docs/`, `specs/`, `mydocs/` responsibility split directly.

## Verification results

Executed on 2026-10-25 Asia/Seoul with Node 24.15.0 on macOS. This is documentation/tooling verification, not a new avatar or performance validation run.

| Boundary | Executed check | Result |
| --- | --- | --- |
| Structure and preservation | `npm run docs:check` | Passed: language pairs, links/anchors, indexes, reachability, length, historical bodies/URLs, framework hashes and symlinks |
| Checker failures | Nine isolated fixtures | All expected outcomes: valid baseline, missing/undefined reference, mismatched fence, inline-code example, outside symlink, missing docs root, untracked/tracked generated files |
| Notice generation | `npm run licenses` twice, compare SHA-256 of four outputs | Identical; 19 npm packages, 21 Rust packages, three model assets |
| Legal and asset inputs | Compare nine files with baseline bytes | MIT and two model license texts, three models and three provenance records unchanged |
| CI drift guard | `git ls-files --error-unmatch` and `git diff --exit-code` for all four generated outputs | Passed; also rejects an untracked generated file in the negative fixture |
| Browser source | `npm run typecheck`; `npm run build:web` | Passed; existing Vite chunk-size warning remains |
| Diagrams | Parse and render all 14 blocks using Mermaid 11.12.1 / Chrome 155.0.8059.86 | Passed; visual overview inspected; dependency installed only in temporary test tooling |
| External sources | 84 bounded, TLS-verified HTTP requests | 74 HTTP 200; eight HTTP 403; two certificate-chain failures; no 404/410 |
| Change boundary | Diff `src/`, `crates/`, package lock and Pages workflow against baseline | Unchanged |
| Whitespace | `git diff --check` | Passed |

The evidence JSON records generated hashes, individual external response classifications, tested checker identity and Mermaid source hashes. Source availability is independent of source correctness and reuse rights. The new CI step checks structure and generated drift; it does not render Mermaid or audit translations semantically.

## Preservation and limits

Root `LICENSE` is the canonical MIT text; a duplicate `LICENSE.md` is unnecessary. `THIRD_PARTY_NOTICES.md` adds the pinned workflow's repository-tooling attribution. Runtime `public/licenses.txt` retains dependency/model license texts and excludes tooling that is not shipped in the browser. `public/models/NOTICE.txt` is generated from the original provenance records.

The eight HTTP 403 responses affect VRoid Studio/Hub (including the two sample pages), BOOTH, two DOI destinations and the Twente paper PDF. Both MMIP pages failed local certificate-chain verification. Those ten responses remain unverified; none is called a dead link or silently deleted. TLS checks were not disabled.

## Next boundary

Local checks are complete. The [final report](../report/task_m3_36_report.md) records review scope and remaining limits. Remote CI for the new PR is reported in the PR at its exact head after publication; no prospective pass is claimed here. Existing owner approval covers PR publication. Merge, release and Pages deployment remain outside this task.
