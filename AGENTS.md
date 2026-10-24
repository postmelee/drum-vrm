# AGENTS.md

Operational rules for every coding agent working in this repository, including Codex and Claude Code. This file is loaded into the system prompt on every turn, so keep only the policies, constraints, and indexes that must always be present. Put detailed procedures in manuals and SKILL files.

## Project Overview

Drum VRM converts uploaded MIDI into deterministic 3D drum performances in the browser. Rust plans the performance, WASM connects the engine, and Three.js renders interchangeable rigs and drum assets.

## Core Hyper-Waterfall Rules

This project uses the **Hyper-Waterfall** methodology. Read these rules carefully because they can conflict with an agent's default behavior, especially fast execution and autonomous edits. Details: [`agent_code_hyperfall_rule_conflict.md`](mydocs/manual/agent_code_hyperfall_rule_conflict.md).

- Request task requester approval before modifying source files.
- Track work by GitHub Issue.
- Never skip the order for new features, bug fixes, or structural changes: `Issue -> branch -> daily task board -> plan -> implementation -> verification -> final report -> PR`.
- Do not move to the next stage without approval after completing the current stage.
- If the scope is unclear or could conflict with existing work, ask first.
- Do not revert changes made by the user or another worker.
- Close an Issue only after task requester approval or after confirming the PR was merged.
- Before editing documentation, read the existing content first, change only the necessary parts, and add content only when needed.
- When creating, moving, or editing product, user, contributor, external integration, API, architecture, or roadmap documentation, record a document location judgment in the task plan and get approval.
- `mydocs/manual` is not the target project's product documentation location. The target project must explicitly choose its official documentation root, such as `docs/`, `specs/`, `site/`, `website/`, or `adr/`, in a separate task.
- After work is complete, clean up local and remote artifacts that are not needed for the next task.
- After PR merge and Issue close, return to `main` and remove unneeded `local/task{number}` branches and temporary worktrees.

**Approval assumption rule**: Treat the stage as approved only when the task requester explicitly says to continue in the same thread, such as "continue" or "proceed to the next stage."

## Naming Rules

- Milestone: `M{version}` (for example, M100=v1.0.0, M05x=v0.5.x). Document filenames use lowercase `m{number}` (for example, `m100`).
- Branches: `local/task{issue_number}` for work, `publish/task{issue_number}` for PR publication to `main`.
- Commit messages:
  - Basic: `Task #{number}: summary`
  - Stage: `Task #{number} Stage {N}: summary`
  - Substage: `Task #{number} [Stage {N.M}]: summary`
  - Report bundle: `Task #{number} Stage {N} + final report: summary`
- Document filename: `task_{milestone}_{issue_number}{_impl|_stage{N}|_report}?.md`. New documents must include the milestone. Details: [`document_structure_guide.md`](mydocs/manual/document_structure_guide.md).
- Write all documents in the selected Hyper-Waterfall locale for this repository.

## Mandatory Rules Before Changes

The owner-approved project rules below take precedence over conflicting framework defaults, including stage approvals, branch names, language and document naming.

### Authorization and workflow

- Continue work within the owner's approved scope without repeated stage permission. Record evidence; ask for new authorization when scope or an action requires it.
- Use `codex/` branches. Stacked PRs may target their prerequisite branch, then retarget to `main` after reviewed integration. Do not create `local/taskN` or `publish/taskN`, rename existing milestones, or open `main`-to-`main` promotion PRs.
- Do not create release tags, publish Releases, or dispatch Pages without explicit approval of a reviewed batch and tested commit. Passing CI, a merge, or historical MVP approval is insufficient. See [release procedure](specs/policies/releases.md).
- Keep scope, acceptance and review state on GitHub; record verification and limitations against the tested commit. Preserve other workers' changes.

### Documents and continuity

- Communicate with the owner in Korean. New issues, PRs, specifications and internal records use English. Reader guidance uses equivalent separate English/Korean files with reciprocal links; preserve existing Korean historical material.
- Resume from [work records](mydocs/README.md), then the relevant issue/plan and [specifications](specs/README.md). Read [document ownership](specs/policies/documentation.md) before creating or moving documents.
- `docs/` owns concise reader guidance; `specs/` owns adopted requirements, architecture, ADRs and policies; `mydocs/` owns task evidence, research and proposals. Keep rich generalized technical explanations in specs and link reader guidance to them.
- Milestone/issue filename conventions apply to task artifacts, not stable specifications or reader guides. Preserve existing ADR/source IDs. Never append new current status to old evidence.
- Follow scoped AGENTS. Update affected contracts and both reader languages with a change; link the relevant issue to actual verification and limits. Keep the work index short.
- Framework manuals/templates/skills retain their pinned text except approved placeholders. Compare source and installed hashes before updates; preserve project-owned files and never replace a populated directory blindly.

### Engineering and provenance

- Rust owns MIDI interpretation and planning; WASM is the browser boundary; Three.js owns visualization. Preserve `PerformancePlan`/`RigAdapter` boundaries without engine-specific scene object names.
- Never move original MIDI hit times to hide motion problems. Diagnose impossible hits. `evaluate_at(t)` must give the same primary pose regardless of playback or seek order.
- Use common semantic-body measurements; do not introduce avatar-name branches. Distinguish actual tool contact from skin/collision realism.
- Run meaningful checks before claiming success. Pin dependencies/sources and keep code, papers, data, weights and assets under their own terms. Preserve original legal notices and provenance.
- Uploaded MIDI/WAV/VRM, raw conversations, secrets and private downloads must not be committed. Generate original demo/test fixtures; `.local/` remains private.
- Proposed, implemented and publicly deployed are different states. An actual approved deployment record must identify URL and commit.

## Required References

- [`README.md`](README.md) - project overview, setup, and build
- [`mydocs/manual/document_structure_guide.md`](mydocs/manual/document_structure_guide.md) - `mydocs/` folder roles, document filenames, external PR folder policy, Skills location policy
- [`mydocs/manual/task_workflow_guide.md`](mydocs/manual/task_workflow_guide.md) - 15 task workflow steps, commit message rules, work time rules
- [`mydocs/manual/git_workflow_guide.md`](mydocs/manual/git_workflow_guide.md) - branch policy, Git diagrams, maintainer/contributor workflow
- [`mydocs/manual/pr_process_guide.md`](mydocs/manual/pr_process_guide.md) - external contributor PR review
- [`mydocs/manual/agent_code_hyperfall_rule_conflict.md`](mydocs/manual/agent_code_hyperfall_rule_conflict.md) - conflicts between Hyper-Waterfall and default agent behavior
- [Specifications](specs/README.md), [documentation policy](specs/policies/documentation.md), [work records](mydocs/README.md), and the relevant issue-specific plan provide project authority and continuity.

## Agent Skills

Fixed Hyper-Waterfall procedure points are separated into SKILL files. The source of truth is `mydocs/skills/`. Codex (`.agents/skills`) and Claude Code (`.claude/skills`) read the same content through symbolic links. Details: the "Agent Skills location policy" section in [`document_structure_guide.md`](mydocs/manual/document_structure_guide.md).

## Work Rules

- The task requester decides when work starts and ends. Agents do not propose ending work or impose time limits on their own.
