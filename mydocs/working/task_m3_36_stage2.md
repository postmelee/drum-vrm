# Stage 2 — document ownership and preservation

[Issue #36](https://github.com/postmelee/drum-vrm/issues/36) · [Implementation plan](../plans/task_m3_36_impl.md).

## Purpose and artifacts

Separate reader guidance, maintained design and commit-specific evidence without losing the project's prior reasoning or results.

| Area | Result |
| --- | --- |
| Root README and `docs/` | Concise navigation; separate linked English/Korean guidance |
| `specs/` | Generalized requirements, contracts, calibration, compatibility, expression, dependency rationale, ADRs and policies |
| `mydocs/` | Research, proposals, approvals and measured reports with explicit historical scope |
| [Migration ledger](../report/migrations/README.md) | One destination/owner record for each of 48 baseline paths |
| Scoped AGENTS files | Stable responsibility and verification rules, outside product guidance |
| Notice generator and documentation checker | Relocated dependency output, provenance-derived notices and preservation checks supporting the move |

The user-approved flexible taxonomy is retained: substantive technical explanations live in `specs/`; short reader guides link to them. Diátaxis is a navigation aid rather than a mandatory classification for every research or design record.

## Preservation scope

The baseline is `66951cd69015e43cfe132a6ba61545b175a25412`. All 35 former `docs/` paths have recorded destinations; all 48 audited document/legal paths have immutable original permalinks. Twenty-eight historical bodies retain their content and source URLs. Only a scope header and relocated relative-link targets were added to those copies. Normalized body fingerprints and original external URL sets guard that boundary.

Six ADR identifiers, research IDs R01–R07, instruction IDs I01–I03, authors, terms and measured commit/environment limitations remain connected. Eight old paths are short compatibility notices for documented callers; they hold no second active specification. Private conversations, uploads and `.local/` material were excluded.

## Verification

- `npm run docs:check`: passed before this report, checking 172 Markdown files, 812 relative links, 28 bilingual pairs and 28 preserved historical bodies; no missing targets, orphaned project pages, oversize authored pages or framework drift.
- Baseline preservation comparison: 35/35 documentation paths covered and 28/28 retained bodies matched after allowing link relocation.
- Independent English/Korean content review checked capabilities, limits, controls and source credits. It corrected preset naming, hit feedback/reduced-motion guidance and player-camera restrictions in both languages.
- Independent code/spec review corrected the shared adapter class name, distinguished derived targets/assignments from immutable MIDI events, and retained the 10,000-node model limit.
- Fourteen Mermaid blocks parsed and rendered with temporary Mermaid 11.12.1 and Chrome 155.0.8059.86; the rendered overview was inspected. Two long technical flows use vertical layout for readability. No Mermaid runtime dependency was added.
- `git diff --check`: passed.

## Residual limits and next stage

A valid link does not prove a source's claims or grant reuse rights. Historical results apply only to their documented commit, fixture and environment. The following stage records generation repeatability, checker negative cases, build verification, external response limits and remote CI.

The owner already approved the migration and PR work. Continue within that scope; merging, releasing and deploying are outside this task.
