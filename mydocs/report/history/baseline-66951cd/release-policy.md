> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/release-policy.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../../specs/README.md) for current contracts and policy.

# Release policy / 릴리스 정책

Effective: 2026-10-24. This policy follows the owner's latest instruction and supersedes previous ongoing-deployment guidance. The original MVP approval remains historical evidence.

## English

Work proceeds through issues, implementation PRs, review, and validation. PR completion, merge, CI success, or milestone completion does not authorize publication. Until the owner explicitly approves the reviewed batch, do not create release tags, publish GitHub Releases, or dispatch GitHub Pages deployment. Existing authorization for requested development work remains valid; do not repeatedly ask for permission for routine implementation or review.

Before a release decision, prepare:

1. Included issues/PRs and the exact candidate commit, distinguishing implemented/unreleased changes from the current public version.
2. Relevant automated checks and browser/output evidence, with the tested environment and remaining limitations.
3. Matching English/Korean user documentation, dependency/asset provenance, and the planned release scope.

Record the owner's explicit approval and the approved scope in the release record. If that scope changes materially, present the revised batch before publishing. Once approved, publish one batch from the verified main commit, check the public application, and record the tag, release URL, deployment run, artifact/checksum, and known limits. Pages remains a manual workflow; manual dispatch alone is not evidence of owner approval.

The public baseline at the introduction of this policy is **v0.2.0**, application commit `3e8be9333cedbc9845dbc3e4cebd73961af697b4`. No new release is approved by this document.

## 한국어

작업은 이슈, 구현 PR, 리뷰와 검증으로 진행합니다. PR 완료·병합·CI 통과·마일스톤 완료는 공개 배포 승인이 아닙니다. 사용자가 검토한 변경 묶음을 명시적으로 승인하기 전에는 릴리스 태그 생성, GitHub Release 발행, GitHub Pages 배포를 실행하지 않습니다. 이미 요청받은 개발 작업의 승인은 유지하므로 일상적인 구현·리뷰마다 반복 승인받지 않습니다.

릴리스 판단 전에 다음을 준비합니다.

1. 포함할 이슈·PR과 정확한 후보 commit. 구현되었지만 미배포인 변경과 현재 공개 버전을 구분합니다.
2. 관련 자동 검사와 실제 브라우저·출력 검증 근거, 검사 환경 및 남은 제한.
3. 내용이 일치하는 영어·한국어 사용자 문서, 의존성·에셋 출처와 라이선스, 릴리스 범위.

사용자의 명시적인 승인과 승인 범위를 릴리스 기록에 남깁니다. 범위가 크게 바뀌면 수정한 묶음을 먼저 제시합니다. 승인 이후 검증한 main commit에서 한 번에 릴리스하고, 공개 앱을 확인한 뒤 태그·Release URL·배포 run·artifact/checksum·제한을 기록합니다. Pages는 수동 workflow를 유지하며 수동 실행 자체를 사용자 승인 근거로 삼지 않습니다.

이 정책 도입 시 공개 기준은 **v0.2.0**, 앱 commit `3e8be9333cedbc9845dbc3e4cebd73961af697b4`입니다. 이 문서가 새 릴리스를 승인하지는 않습니다.
