# Drum VRM project instructions

## Start and scope

- Read the relevant GitHub issue/PR, [work entrypoint](docs/handoff.md), and the affected [technical specification](specs/README.md). Read linked decisions/evidence when needed; do not load the entire archive by default.
- Continue within the owner's approved scope without repeated routine approvals. Preserve other workers' changes. Use `codex/` branches.
- Keep scope and acceptance on the issue; explain the final change, executed checks and limitations in the PR. Complex work may need one linked plan or evidence report; scale the process to the change.
- Do not merge a PR without the owner's instruction. Release tags, GitHub Releases and Pages deployment require separate explicit approval of the reviewed batch and exact candidate commit. CI success and previous approvals are insufficient. Follow the [release policy](docs/release-policy.md).

## Documents and language

- Speak with the owner in Korean. Write new issues, PRs and internal records in English.
- Maintain English/Korean pairs for README, public guides and public technical specifications: `name.md` and `name.ko.md`, with reciprocal links and equivalent facts in the same PR.
- Internal research, ADRs and verification records may be English only. Preserve existing Korean historical text, original legal text and generated inventories without forced translation.
- Keep useful features, usage and local commands in README. Put brief task guidance in existing topic documents under `docs/`; detailed current contracts and explanations live in `specs/`.
- Preserve ADR IDs and supersession relationships in `docs/decisions/`, research sources in `docs/research/`, and commit/environment-specific results in `docs/verification/`. New evidence does not rewrite an old measured result.
- An executable limit is owned by code; update its matching specification with the implementation. Issues own live status. Guides summarize and link rather than maintain independent status histories.
- Prefer one coherent topic over many short pages. Keep authored documents under 500 lines; original legal text and generated inventories are exceptions. Add directory indexes only when they improve navigation.
- Use Mermaid for relationships that benefit from a diagram; check rendering when diagrams change. Preserve meaningful detail and source credits, not only short summaries.
- Keep current behavior, proposed design and public deployment distinct. A specification or merged PR does not establish what the public app runs.

## Engineering and provenance

- Keep ordinary inspectors focused on controls and current values. Do not add paragraphs of mapping rationale, implementation disclaimers, candidate counts or repeated success/status messages beneath controls. Use the shared notification system for operation feedback, Analysis for evidence, and collapsed contextual help for reference material; follow the [notification design](specs/architecture/notifications.md). Use `NotificationFeedback` / `NotificationStore`; do not introduce another local toast/status mechanism.
- Retain visible field labels, units, concise actionable validation, unresolved-state badges and required attribution. Do not remove screen-reader descriptions to reduce visible copy. Every UI change must review notification duplication and both language layouts.

- Rust owns MIDI interpretation and planning; WASM is the browser boundary; Three.js renders through `RigAdapter`. Keep `PerformancePlan` independent of scene object names.
- Do not shift original MIDI hit times to conceal motion problems. Diagnose impossible hits. `evaluate_at(t)` must return the same primary pose regardless of playback/seek order.
- Fit through semantic humanoid measurements and common numeric constraints, without avatar-name branches or stretching bones. Actual stick/sole-marker contact is not proof of natural posture or skin collision clearance.
- Run meaningful checks appropriate to the changed boundary before claiming success. Rebuild WASM for Rust changes. Media changes need actual output evidence, not just encoder completion.
- Pin dependency/source versions and preserve applicable licenses. Distinguish papers, code, data, weights and assets; a license for one does not cover the others. Model provenance JSON owns attribution metadata; generate notices with `npm run licenses`.
- Never commit uploaded MIDI/WAV/VRM, private downloads, secrets or raw conversations without explicit permission for the exact public asset and recorded distribution rights. The reviewed MIDI in `public/demos/*-provenance.json` and official samples in `public/characters/*-provenance.json` are recorded exceptions; they do not authorize other uploads or change asset licenses to MIT. `.local/` stays private. Author other public demo/test fixtures independently.

## Finish

- Update affected contracts, both public languages, relevant issue/PR evidence and the short work entrypoint if the next action changed.
- Run `npm run docs:check` for document changes. It checks structure, not semantic translation, source authenticity or source reuse rights.
- Keep one-time migration audits in their report/PR rather than making a past directory layout a permanent CI requirement.
- At an approved deployment, record the URL, exact commit and actual validation. Do not infer release approval from completion of this checklist.
