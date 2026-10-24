# Documentation policy

## Responsibility and authority

| Location | Owns |
| --- | --- |
| `docs/` | Reader-facing tutorials, task guides, references and explanations |
| `specs/product/` | Adopted product purpose, requirements and invariants |
| `specs/architecture/` | Current adopted component/data/time/rig contracts |
| `specs/decisions/` | Decision rationale, identifiers and historical scope |
| `specs/policies/` | Project maintenance rules |
| `mydocs/` | Workflow entry, active plans, drafts, research, experiments and reports |
| GitHub issues/PRs | Active scope, acceptance criteria and execution/review status |
| Release/deployment records | What was actually published and from which commit |

A fact has one responsible source. Explain or translate it where useful, but link
back rather than maintaining independent status tables or copied measurements.
Executable limits are owned by code/schema; the matching specification must change
with them. Reader lookup pages summarize that contract for their audience.

## Language and audience

Communicate with the owner in Korean. Maintain `README.md` and `README.ko.md` as paired
entry points. Reader-facing product and contributor guidance has separate English
and Korean files, reciprocal links, and equivalent capabilities/limitations in the
same PR. Use `name.md` and `name.ko.md` beside one another.

New specifications, policies, internal engineering/research, issues and PR prose are
English. Preserve existing Korean historical documents and ADR text; do not discard
source material to make the tree appear monolingual. Original legal text and generated
language-neutral data are not translated; paired guides can explain their purpose.

## Document shape

Keep substantive adopted technical design in `specs/`: generalized calibration,
architecture, performance-expression and dependency/compatibility explanations are
valuable to repository explorers. Do not compress them into shallow summaries or
force them into Diátaxis categories. Link concise user instructions to the deeper
technical chapters, with implemented and proposed behavior explicit.

Use Diátaxis for reader guidance according to the reader's need: tutorial for learning, how-to for a goal,
reference for lookup, explanation for understanding. Do not create empty categories
or copy one topic into all four forms. A directory README explains audience and routes
to its contents; it does not duplicate child documents or issue status.

Keep authored documents under 500 lines and one primary purpose. Prefer 100–250 lines
when the topic needs that much space; there is no minimum. Split by responsibility,
not arbitrary line counts. Long table cells and single-line paragraphs must not hide
multiple topics. Original legal text, generated inventories/notices and preserved
upstream framework files are explicit length exceptions.

Use Mermaid for meaningful relationships, sequences or boundaries. Provide ordinary
links and a prose explanation as well. Inspect rendering after structural changes.
AGENTS files contain scoped instructions and policy links, not product explanations.

## Time and claims

Current specifications describe the checked-out implementation and adopted constraints.
Reader availability distinguishes deployed, development and proposed features in one
[capability guide](../../docs/reference/availability.md). Individual guides state scope
briefly and link there. A merge or test pass does not establish deployment.

Record test and research results against their commit/source identities and dates.
Do not append a new current implementation beneath an obsolete current snapshot.
Create a follow-up record and connect it. ADR changes require a new decision or an
explicit supersession; preserve accepted historical rationale and scope.

## Changes and migration

1. Identify each changed fact's owner and update it before explanatory copies.
2. Update both reader languages and the relevant issue/PR links in the same change.
3. Preserve ADR/source IDs, measured commit/environment/limits and asset provenance.
4. For moves, record old path, baseline permalink, destination and content responsibility.
5. Keep an old URL only as a short relocation notice when a real caller still uses it;
   document that caller and the removal condition. Do not leave a second active body.
6. Check internal links/anchors, paired pages, orphaned documents, line limits and
   diagrams. Compare meaning in the two languages; link existence is insufficient.
7. Generated paths change with their generator and consumers. Verify regeneration
   against locked inputs rather than hand-editing generated output.

Do not migrate private `.local/` data, uploaded assets or original conversations into
public documentation. The [third-party policy](third-party.md) governs quoted or
redistributed source material.
