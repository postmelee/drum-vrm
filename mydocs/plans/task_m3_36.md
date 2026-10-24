# Task #36 — documentation architecture

Issue: [#36](https://github.com/postmelee/drum-vrm/issues/36) · Milestone: M3.

## Purpose

Make reader guidance easy to navigate while retaining the project's design reasoning, source attribution, and measured results. Each fact has one owner; translations and summaries link to that owner.

## Background

The baseline is commit `66951cd69015e43cfe132a6ba61545b175a25412`. Its 48 managed documentation/license files mix current contracts with dated progress and duplicate facts. Approval and subsequent refinements are recorded in [owner decisions](../feedback/task_m3_36_feedback.md).

## Scope

Included: Hyper-Waterfall follow-on documentation migration, current technical specifications, preserved history, paired English/Korean reader pages, indexes, provenance/notices, validation and issue/PR links.

Excluded: new motion features, model acquisition, merging PRs, tags, Releases and Pages deployment. PR #34 remains unchanged; this branch stacks on bootstrap PR #37.

## Design direction

Use Diataxis as a reader-needs guide, not a mandatory folder-filling exercise. Keep tutorials/how-to concise and link to attractive, detailed technical design chapters in specs. Adopted design and policies must not live in exploratory work records.

## Document location judgment

| Material | Audience / authority | Selected location | Alternative | Reason |
|---|---|---|---|---|
| Tasks and usage | Users; maintained guide | docs/ | All detail in README | Goal-oriented entry without implementation history |
| Generalized architecture/calibration/expression | Technical readers; adopted principles/contracts | specs/architecture/ | docs four-category classification | Detailed technical reading remains discoverable without forced taxonomy |
| ADRs and current policies | Maintainers; normative decisions | specs/decisions/ and specs/policies/ | mydocs/tech/ | Separate adopted authority from drafts |
| Research and experimental proposals | Maintainers; evidence/proposed | mydocs/tech/ | Current specification | Preserve uncertainty and source context |
| Past verification, approvals and requirements | Maintainers; historical evidence | mydocs/report/ | Appended current guide | Preserve measured commit/environment/limits without stale current claims |
| Source metadata and legal text | Readers and distributors | source catalogs; model provenance; LICENSE | Repeated tables in guides | One owner with linked/generated views |

## Stages

1. Preserve approved scope, baseline and ownership mapping.
2. Migrate reader documents, specifications and historical evidence; reconcile links and availability.
3. Verify paired guidance, source preservation, local links/anchors, diagrams and generated notices; publish a separate review PR.

## Verification plan

Compare all 48 baseline records with the migration ledger and immutable originals. Check document links, language counterparts, directory entrypoints and the 499-line authored-file limit. Render Mermaid using a pinned temporary tool. Generate notices twice and compare bytes; keep original legal/model files unchanged. Check relevant CI and record unverified external URLs honestly.

## Risks

Historical source bodies include old facts: retain explicit baseline scope. Public v0.2 and unpublished work differ: centralize availability. Framework-managed templates are not ordinary authored docs and must remain pinned. Translation checks verify structure; human review must assess equivalent meaning.

## Authorization

The owner approved this migration and separate PRs on 2026-10-25. Later steering approved the specs split and flexible taxonomy. Existing authorization covers these stages; no repeated stage approval or release is inferred.
