# Release policy

## Authority

Implementation, verification, issue updates and review PRs can proceed within the
owner's approved work scope. Publishing is a separate decision: do not create release
tags, publish GitHub Releases or dispatch Pages without explicit owner approval of
the concrete batch being released.

Passing CI, merging a PR, finishing a milestone and a historical deployment approval
are not approval for another batch. The initial MVP approval is retained as a
[historical record](../../mydocs/report/history/approvals.md), not ongoing permission.

## Prepare the reviewable batch

Before asking for a release decision, finish authorized implementation and checks.
Present the exact candidate commit, included issues/PRs, executed evidence, remaining
limitations and deployment scope. Check reader instructions in both languages and
third-party notices. Do not describe unexecuted checks as passing.

## After explicit batch approval

1. Confirm that the approved candidate and intended artifact still match.
2. Build from the reviewed revision with locked dependencies and required notices.
3. Run the publication workflow for that approved batch.
4. Verify the public URL and relevant uploaded-file/playback/export path.
5. Record approval evidence, source commit, workflow run, artifact identity/checksum,
   public URL and observed limitations in the release report.

Keep release/deployment records as the authority for public versions. The reader
[availability guide](../../docs/reference/availability.md) links to them; ordinary
specifications do not carry a copied list of historical deployment hashes.

Rollback or republishing also requires an explicit instruction for that action.
Preserved release artifacts are recovery inputs, not automatic authority to deploy.
