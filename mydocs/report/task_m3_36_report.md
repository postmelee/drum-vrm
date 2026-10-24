# Documentation architecture migration — final report

[Issue #36](https://github.com/postmelee/drum-vrm/issues/36) · M3 · [Plan](../plans/task_m3_36.md) · [Implementation plan](../plans/task_m3_36_impl.md).

## Outcome

Three stages separate reader documentation, adopted technical design and historical/work evidence. The bootstrap is independently reviewable in [PR #37](https://github.com/postmelee/drum-vrm/pull/37); this migration is stacked on it. The original rig work in [PR #34](https://github.com/postmelee/drum-vrm/pull/34) is unchanged.

Current specifications explain stable responsibilities and constraints. Historical records preserve what was measured, decided or proposed at a particular commit. Reader guidance links to both without copying running status or presenting proposals as completed features.

## Location and change scope

| Responsibility | Planned and actual home | Evidence |
| --- | --- | --- |
| Product entry and user/contributor guidance | Paired root README and `docs/` | [English](../../docs/README.md) / [Korean](../../docs/README.ko.md) |
| Detailed calibration, compatibility, expression, dependencies and contracts | `specs/architecture/` | [Technical design index](../../specs/architecture/README.md) |
| Requirements, decisions and maintenance rules | `specs/product/`, `decisions/`, `policies/` | [Specification index](../../specs/README.md) |
| Research, approvals, test results and unimplemented proposals | `mydocs/tech/`, `report/`, `feedback/` | [Work-record index](../README.md) |
| Agent instructions | Scoped AGENTS, linked policies | [Document ownership](../../specs/policies/documentation.md) |
| Licenses and provenance | Original legal texts, model JSON and generated notices | [Notices](../../THIRD_PARTY_NOTICES.md), [model provenance](../../public/models/README.md) |

This matches the approved location judgment. Rich technical explanation remains in specs rather than being forced into the four reader categories. There are no app, model-binary, dependency-lock or deployment-workflow changes.

## Preservation and measured result

| Metric | Result |
| --- | --- |
| Audited baseline paths | 48/48 have owners, destinations, original hashes and immutable permalinks |
| Historical bodies retained | 28/28 passed content and original-source URL preservation checks |
| Original `docs/` paths accounted for | 35/35 |
| Legacy entrypoints | Eight short caller-backed relocation notices, with removal conditions |
| Paired reader documents | 28 English/Korean pairs; semantic review and reciprocal links |
| Root README length | 111 lines per language before; 35 after |
| Largest authored project Markdown | 156 lines at migration verification; under the 500-line limit |
| Meaningful diagrams | 14 parsed, rendered and visually inspected |

The [migration ledger](migrations/README.md) owns detailed old-to-new mappings. Original Korean research/ADR text, six decision IDs, R01–R07 and I01–I03 source IDs, author credits and measurement limitations are preserved. Historical link relocation does not replace source references. No private uploads or conversation transcript was published.

## Verification evidence

- [Stage 1](../working/task_m3_36_stage1.md): approved boundaries and complete baseline inventory.
- [Stage 2](../working/task_m3_36_stage2.md): migration, source continuity, code/spec review and bilingual semantic review.
- [Stage 3](../working/task_m3_36_stage3.md): repeat generation, nine checker fixtures, typecheck/build, rendered diagrams, external responses and CI gate.
- [Machine-readable checks](migrations/task_m3_36_checks.json): hashes, fixture outcomes and bounded external-link results.

`npm run docs:check`, repeated `npm run licenses`, generated-file tracking/drift checks, `npm run typecheck`, `npm run build:web` and `git diff --check` passed. CI runs the existing complete Rust/WASM/web suite plus the new documentation checks after PR publication; its exact head/result belongs in the PR, not an assumed pass in this report.

## Limits and follow-up

The HTTP check reached 74 of 84 external URLs. Eight returned 403 and two MMIP requests failed certificate-chain verification; these remain unverified rather than being declared broken. No 404/410 was observed. Link reachability does not establish licensing or model compatibility.

The checker covers the repository's authored Markdown conventions, not every possible Markdown extension. Automated bilingual pairing cannot replace semantic review. The browser build retains the existing large-chunk warning. No new avatar imports, naturalness tests or independent media playback were performed in this documentation change.

After review/integration, resume the existing [fit #18](https://github.com/postmelee/drum-vrm/issues/18), [stroke/style #20](https://github.com/postmelee/drum-vrm/issues/20) and [measurement #23](https://github.com/postmelee/drum-vrm/issues/23) criteria. Preserve existing #14/#17/#30 scope; this migration does not close their remaining feature work. Subsequent records should link the responsible specification and prior evidence rather than append another current snapshot to multiple pages.

## Review boundary

Implementation and PR publication were approved by the owner. The two documentation/workflow PRs remain separate and unmerged for review. No release tag, GitHub Release or Pages deployment is included or authorized by this report.
