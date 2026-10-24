# Implementation plan — task #36

[Task plan](task_m3_36.md) · [Issue #36](https://github.com/postmelee/drum-vrm/issues/36) · M3.

| Stage | Output | Verification |
|---|---|---|
| 1 | Approved boundaries, baseline, file ownership | 48-file audit coverage; original SHA |
| 2 | docs/specs/mydocs migration and source continuity | Link checks; historical body comparison; source/code review |
| 3 | Generated notices, validation tools, report and PR | Repeat generation; language/link/diagram checks; CI |

## Stage 1

Record the owner's decisions and the document location judgment. Use `codex/documentation-architecture` stacked on the independently committed bootstrap. Work artifacts are English; reader-facing pages use separate English/Korean pairs.

Check: `git diff --check`; confirm baseline commit and clean bootstrap separation. Commit the approved plan, board and stage evidence.

## Stage 2

Apply scoped reader and internal-document packages, preserving source IDs, historical bodies and tested commits. Update project placeholder references without editing framework-managed manuals. Replace mixed current/historical prose with generalized specs and immutable reports. Remove obsolete active duplicates; retain the few necessary compatibility entrypoints. Keep rich technical explanations directly reachable from README.

Check: `npm run docs:check`; original-document migration inventory comparison; review current limits against code; human review of English/Korean scope and limitations. Stage report records only completed checks.

## Stage 3

Move the dependency inventory with its generator, derive model NOTICE from provenance, and preserve full license texts. Add CI checks for documentation and generated drift. Render all project-owned Mermaid blocks using a pinned temporary Mermaid installation. Record source URL response failures separately from source authenticity.

Check: `npm run docs:check`; `npm run licenses` twice with checksum comparison; notice invariants; `npm run typecheck`; `npm run build:web`; `git diff --check`. No new motion tests are required when runtime source is unchanged. CI performs the existing complete suite.

Publish the report and open a PR against `codex/hyper-waterfall-adoption`. Keep issues open until integration; no merge or deployment.

## Location and continuity

The [task plan](task_m3_36.md) owns the location judgment. The migration ledger owns old-to-new mappings; specs own maintained rules, not this execution plan. Each stage includes a short evidence record. Existing release restrictions remain binding.

## Stage evidence

- [Stage 1: baseline and authority](../working/task_m3_36_stage1.md).
- [Stage 2: ownership and preservation](../working/task_m3_36_stage2.md).
