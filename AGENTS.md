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

The explicit owner-approved Drum VRM rules below override conflicting defaults in the framework manuals, templates, and Skills, including the stage gates, branch names, and selected-locale rule above.

### Authorization, branches, and release

- Continue implementation, verification, documentation, issue, and PR work already authorized by the owner without repeatedly asking for stage approval. Record evidence at stage boundaries. Ask when scope or an action needs new authorization; passing a stage never authorizes a release.
- Use `codex/` work branches and publish the same branch name for PRs to `main`; do not create `local/taskN` or `publish/taskN` branches. Preserve other workers' changes and worktrees.
- Both the integration and release reference are `main`. Do not create a `main`-to-`main` promotion PR. Release only a specifically approved batch and tested commit.
- Do not create release tags, publish GitHub Releases, or dispatch GitHub Pages deployment without an explicit owner instruction approving that batch. A previous MVP approval, passing CI, merging a PR, or finishing a milestone is not approval. Before requesting release approval, prepare a concrete change summary, tested commit, known limitations, and release scope. Keep the existing `docs/release-policy.md` in force until its approved migration.

### Language and document ownership

- Communicate with the owner in Korean. Write issues, PR descriptions, new internal engineering notes, research audits, and ADRs in English.
- `README.md` is English and `README.ko.md` is Korean. Maintain user-facing instructions, supported formats, and release guidance in separate English and Korean files in the same PR, with reciprocal links and equivalent facts. Preserve existing Korean material during migration.
- Distinguish proposed, implemented/unreleased, and publicly deployed behavior. A merged PR does not imply the public site changed.
- This bootstrap installs operating files only. Keep the current README and product documents unchanged; their migration belongs to the separately approved documentation PR.
- For that approved documentation redesign, adopted product requirements, architecture, decisions, and project policies belong in `specs/{product,architecture,decisions,policies}`. Reader guidance belongs in `docs/`; draft research and task evidence belong in `mydocs/`. Do not put project-owned policy files inside framework-managed `mydocs/manual/`.
- Keep framework-managed files unchanged apart from approved placeholders. On updates, enumerate files and compare the pinned source and installed hashes; preserve project-owned files and review conflicts before replacing anything. Never replace an entire populated directory blindly.

### Engineering and source boundaries

- Use Rust for MIDI interpretation and performance planning, WASM for the browser boundary, and Three.js for visualization.
- Never move original MIDI hit times to hide motion problems; return diagnostics for impossible hits.
- Preserve the `PerformancePlan` and `RigAdapter` boundary. Do not hardcode basic-rig object names into the engine. `evaluate_at(t)` must return the same primary pose regardless of playback or seek order.
- Run meaningful checks appropriate to the change and record actual results. Never claim an unrun check passed.
- Pin adopted dependencies by version or commit and license. Distinguish papers, software, data, model weights, and assets, and retain their own terms and source evidence.
- Never commit uploaded MIDI, WAV, or VRM files, raw conversations, secrets, or private external downloads. Generate demo and test fixtures locally. `.local/` remains private working material.

### Continuity and completion

- Until the documentation migration lands, resume from `docs/handoff.md`, then read `docs/product-requirements.md`, `docs/plan.md`, `docs/architecture.md`, applicable `docs/decisions/`, and relevant `docs/research/` adoption conditions. Check the current GitHub issue and milestone; issues own execution state and documents own design.
- Update affected design documents and ADRs with changed facts. Record commits or PRs, executed verification, and remaining limits on the issue. Update `docs/handoff.md` until its replacement is established by the documentation PR.
- When licenses or bundled assets change, update the provenance records and generated distribution notices. On an approved actual deployment, record the URL and deployed commit SHA.

## Required References

- [`README.md`](README.md) - project overview, setup, and build
- [`mydocs/manual/document_structure_guide.md`](mydocs/manual/document_structure_guide.md) - `mydocs/` folder roles, document filenames, external PR folder policy, Skills location policy
- [`mydocs/manual/task_workflow_guide.md`](mydocs/manual/task_workflow_guide.md) - 15 task workflow steps, commit message rules, work time rules
- [`mydocs/manual/git_workflow_guide.md`](mydocs/manual/git_workflow_guide.md) - branch policy, Git diagrams, maintainer/contributor workflow
- [`mydocs/manual/pr_process_guide.md`](mydocs/manual/pr_process_guide.md) - external contributor PR review
- [`mydocs/manual/agent_code_hyperfall_rule_conflict.md`](mydocs/manual/agent_code_hyperfall_rule_conflict.md) - conflicts between Hyper-Waterfall and default agent behavior
- `docs/handoff.md`, `docs/product-requirements.md`, `docs/plan.md`, `docs/architecture.md`, `docs/release-policy.md`, `docs/decisions/`, and `docs/research/` remain the project references until the separately approved documentation migration updates these pointers.

## Agent Skills

Fixed Hyper-Waterfall procedure points are separated into SKILL files. The source of truth is `mydocs/skills/`. Codex (`.agents/skills`) and Claude Code (`.claude/skills`) read the same content through symbolic links. Details: the "Agent Skills location policy" section in [`document_structure_guide.md`](mydocs/manual/document_structure_guide.md).

## Work Rules

- The task requester decides when work starts and ends. Agents do not propose ending work or impose time limits on their own.
