# Drum VRM 작업 지침

## 다시 시작할 때

1. `docs/handoff.md`에서 현재 상태와 다음 작업을 읽는다.
2. `docs/product-requirements.md`, `docs/plan.md`, `docs/architecture.md`를 읽는다.
3. GitHub의 해당 이슈와 마일스톤 상태를 확인한다. 이슈가 실행 작업의 기준이고 문서는 설계의 기준이다.
4. `docs/decisions/`의 결정과 `docs/research/`의 채택 조건을 확인한다.

## Current authorization and release policy

The owner approved the original MVP implementation and deployment on 2026-10-24. That historical approval is recorded in `docs/approval.md` and does not authorize future releases.

The owner's latest instruction supersedes earlier release guidance: work through issues and PRs, then release an approved batch once. Do not create release tags, publish GitHub Releases, or dispatch Pages deployment without an explicit owner instruction approving that batch. Passing CI, merging a PR, finishing a milestone, or a previous deployment is not release approval. Continue authorized implementation, verification, documentation, and PR work without asking again for each reversible step. Prepare a concrete change summary, tested commit, known limitations, and release scope before requesting release approval. See `docs/release-policy.md`.

## Documentation languages

- Communicate with the owner in Korean.
- `README.md` is English; `README.ko.md` is Korean. Preserve existing Korean documentation.
- Maintain user-facing instructions, supported-format information, and release guidance in English and Korean in the same PR, with reciprocal links or paired sections. Keep both versions factually equivalent.
- Internal engineering notes, research audits, ADRs, issues, and PR descriptions may be English only. Existing Korean engineering documents need not be destructively replaced.
- Distinguish proposed, implemented/unreleased, and publicly deployed behavior. A merged PR does not mean the public site has changed.

## 개발 원칙

- 핵심 MIDI 해석과 연주 계획은 Rust, 브라우저 연결은 WASM, 시각화는 Three.js로 구현한다.
- 원본 타격 시각을 이동해 동작 문제를 숨기지 않는다. 불가능한 타격은 진단으로 반환한다.
- `PerformancePlan`과 `RigAdapter` 경계를 유지한다. 기본 리그의 객체 이름을 엔진에 하드코딩하지 않는다.
- `evaluate_at(t)`는 재생 순서와 관계없이 같은 주요 자세를 반환해야 한다.
- 신규 작업 브랜치는 기본적으로 `codex/` 접두어를 사용한다.
- 구현 변경에 맞는 의미 있는 검사만 실행하고 결과를 기록한다. 테스트 실행 전 통과했다고 쓰지 않는다.
- 의존성은 도입 시 버전·커밋·라이선스를 고정한다. 논문, 소스, 데이터, 가중치, 에셋을 구분한다.
- 업로드 MIDI/WAV/VRM과 대화 원문은 커밋하지 않는다. 기본 데모와 테스트 fixture는 직접 생성한다.
- `.local/`은 개인 보관·작업 자료다. 비밀 정보, 원본 대화, 외부 다운로드를 공개 저장소에 올리지 않는다.

## 작업을 마칠 때

- 변경한 설계와 사실을 관련 문서·ADR에 갱신한다.
- 이슈에 변경 커밋/PR, 실행한 검증, 남은 제한을 남긴다.
- `docs/handoff.md`에 완료 사항, 현재 이슈, 다음 실행 순서, 막힌 점을 갱신한다.
- 라이선스가 달라지거나 새 에셋을 넣었다면 연구 대장과 배포 고지를 갱신한다.
- 실제 배포 시 테스트 URL과 배포 commit SHA를 기록한다.
