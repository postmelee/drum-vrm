# Documentation simplification evidence

Review date: 2026-10-25. Scope: [issue #36](https://github.com/postmelee/drum-vrm/issues/36), branch `codex/docs-simplification`, based directly on [PR #34 at 66951cd](https://github.com/postmelee/drum-vrm/tree/66951cd69015e43cfe132a6ba61545b175a25412). The PR's commit list identifies the reviewed document revision.

## Decision and retained work

The owner approved a smaller replacement for unmerged [PR #37](https://github.com/postmelee/drum-vrm/pull/37) and [PR #38](https://github.com/postmelee/drum-vrm/pull/38). The full Hyper-Waterfall installation and mandatory Diataxis directory taxonomy added more workflow and navigation than this repository needed. Their original commits remain available; this branch does not depend on either proposal.

- README again contains features, usage, supported inputs, local commands, limitations and credits. It links short model/calibration/source guides rather than requiring a tutorial hierarchy.
- Eight detailed technical chapters and their diagrams from [the prior proposal](https://github.com/postmelee/drum-vrm/tree/ccc071482390bdac87b8201f7d8586decf973f5a/specs/architecture) remain under `specs/architecture/`. Public specifications have English/Korean counterparts.
- Original ADRs, research and measured evidence keep their `docs/` paths. Only obsolete general overviews and rolling work state become short pointers to current contracts or immutable original revisions.
- Issues/PRs own live scope and review status. Project instructions are concise; separate reports are justified by evidence, not a mandatory daily/stage workflow.
- Model provenance JSON generates model attribution and distribution notices. `LICENSE` remains the sole project license text; a duplicate `LICENSE.md` is unnecessary.

## Preservation audit

Compared file contents with baseline `66951cd69015e43cfe132a6ba61545b175a25412`:

| Material | Result |
| --- | --- |
| Tracked runtime files in `src/` and `crates/` | 30/30 byte-identical |
| npm/Cargo lockfiles, Pages workflow, root LICENSE | Unchanged |
| Three GLBs, three provenance records, two original model license texts | 8/8 byte-identical; model hashes and sizes match provenance |
| Original ADRs and verification records | 6/6 and 6/6 byte-identical |
| Requirements-origin summary, approval history and release policy | 3/3 byte-identical |
| Research documents, excluding generated dependency inventory | 9/9 original bodies byte-identical; only a two-line historical-scope prefix added |
| Instruction sources I01–I03 | Original bilingual source-table rows retained verbatim; R01–R07 register unchanged below its scope prefix |
| General architecture, requirements, plan and handoff | Original paths retained as concise entrypoints, each with an immutable baseline link |
| npm, Rust and standard-library notice sections | Byte-identical to the baseline; duplicate model notice input removed |

New requirement invariants use INV01–INV10, avoiding reuse of the original U/P/A identifiers with different meanings. Historical IDs, measurements, authors, URLs and license distinctions remain available. Keeping history does not mean copying every superseded overview into a new archive tree.

## Executed checks

- `npm run docs:check`: no local-target, heading, language-counterpart, reciprocal-link or authored-length errors. Seventeen public EN/KO pairs are covered; generated legal/inventory material is excluded from the authored-size rule.
- Checker validation: 17 isolated temporary fixtures passed, covering missing files/headings/translations, reference links, Unicode/duplicate headings, code fences, escaped repository links and length exceptions. No fixed migration ledger, issue number or framework state is required by CI.
- Translation review: all five reader-guide pairs, eleven specification pairs and the model-credit pair were read for equivalent behavior, limits and rights. Specification code tokens, external URLs and table/section structures match across languages. This is editorial review, not an independent linguistic certification.
- Mermaid: all 12 diagrams rendered with Mermaid 11.12.1 and Chrome 155.0.8059.86; inspected the resulting overview for clipped or overlapping structure. Renderer dependencies and screenshots stayed outside the repository.
- `npm run licenses`: 19 npm packages, 21 Rust packages and three assets processed successfully. Repeating generation preserved the hashes of all four outputs: root notices, distributed notices, model notices and the existing dependency inventory.
- `npm run build:web`: typecheck and production build passed. Vite retains its existing warning about a JavaScript chunk over 500 kB.
- `git diff --check`: passed. No framework installation, symlinks, manifest or required stage/daily reports are introduced.

Reader walkthroughs confirmed that feature/setup information is directly in README; model and fitting instructions are one link away; detailed Korean contracts and original decisions/evidence are reachable from their topic pages. These are editorial navigation checks, not user-study results.

## Boundaries

The checker is deliberately smaller than a complete Markdown renderer. It does not prove semantic translation quality, external-site availability or reuse rights. Original research links and rights records were preserved; this pass did not revalidate every external source's availability or terms.

No new playback, VRM, contact or exported-video compatibility claim is made. Existing motion limitations and the unverified independent playback of earlier encoded media remain in their original records. CI results for the published candidate belong on the PR.

The owner will review the replacement PR before instructing a merge. No PR merge, release tag, GitHub Release or Pages deployment is part of this change.
