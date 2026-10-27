# MVP 구현과 배포 승인

현재 상태: **후속 작업은 이슈·PR로 진행하며 새 릴리스·배포는 배치별 사용자 승인 필요**. 아래 MVP 승인은 과거 기록이다. 기록 기준일: 2026-10-24 (Asia/Seoul).

초기 준비 범위는 개발 계획, 공개 GitHub 저장소 생성, 이슈와 마일스톤 구성, 장기 작업 문서 및 출처·라이선스 조사였다. 저장소 공개는 사용자가 2026-10-24에 선택했다.

2026-10-24 사용자가 계획과 MIT 라이선스를 승인하고 MVP 구현·검증·GitHub Pages 배포를 요청했다. 다음 범위를 진행한다.

- M1과 M2 구현: MIDI 업로드·사전 계산, 확장 가능한 기본 휴머노이드 리그, 재생·일시정지·탐색, 자체 합성 드럼 음향, 720p30 영상 다운로드.
- Rust/WASM + Three.js + 지원 브라우저의 WebCodecs 및 Mediabunny 구성.
- 테스트 통과 후 `postmelee/drum-vrm`의 GitHub Pages에 공개 MVP 배포.
- 자체 소스의 MIT 라이선스 채택. 제3자 구성요소는 각각의 원래 라이선스를 유지한다.

승인 근거: 이 채팅의 사용자 메시지 “좋아, 이 계획과 MIT 라이선스로 진행해줘. MVP를 구현하고 검증한 뒤 GitHub Pages에 배포해서 테스트 링크를 알려줘. 진행하면서 이슈와 문서도 갱신해줘.”

담당 이슈: #3 승인 완료, #4–#12 구현·검증·배포. 해당 승인 범위에서 반복 승인 없이 진행한다.

M3와 M4는 장기 로드맵이며 첫 MVP 완료 조건이 아니다.

## 2026-10-24 사용자 UI 후속 요청

사용자가 Logic Pro형 최소 UI, 라이트/다크, 지정 사이트 글꼴, 메뉴로 여는 네 설정 화면, 스켈레톤 표시, 드럼 정면 편집/모델 교체와 무료 모델 조사를 요청했다. 이슈 #25로 추적하며 기존 구현·검증·Pages 배포 및 문서 갱신 승인에 따라 반영한다. 전체 WAV/VRM/프로젝트 저장 백로그를 자동으로 승인한 것으로 해석하지 않는다.


## 2026-10-24 latest direction — batch release approval required

The owner explicitly requested issue/PR work only unless instructed otherwise, followed by one batch release after approval. This supersedes the ongoing deployment interpretation in the historical UI follow-up section above.

- Continue requested analysis, implementation, verification, issue updates, documentation, and review PRs.
- Do not create release tags, publish Releases, or deploy Pages without explicit approval of the reviewed batch. A PR merge is not that approval.
- Present the exact commit, included changes, test evidence, and remaining limitations before a release decision.
- The current public baseline remains v0.2.0, app commit `3e8be9333cedbc9845dbc3e4cebd73961af697b4`, until a new release is approved.
- Public user documentation must be maintained in English and Korean; existing Korean documents are preserved. Engineering-only material may be English.

See [release policy / 릴리스 정책](release-policy.md). This record documents the owner's instruction; it does not itself grant a future release approval.

## 2026-10-25 — merge and Pages-only deployment approval

The owner instructed: “남은 PR도 검토후 문제없다면 병합하고 현재까지 진행된 것을 Pages에 배포해줘. 릴리즈는 하지 말아줘.”

This authorizes review and merge of the remaining PRs #34/#39 and one Pages deployment of the verified integrated main commit. Record that exact commit, workflow run and public checks on the PRs and #36. It does **not** authorize a version bump, release tag or GitHub Release. The earlier v0.2.0 public-state statements above remain historical. Further feature work waits for the owner's next instruction; this is not ongoing deployment approval.


## 2026-10-26 — browser performance batch, merge and Pages only

The owner requested Pages access to the current changes while away from the local server, then explicitly approved merging PRs #40/#41 and deploying the tested integrated commit by replying “승인할게” to that concrete scope. Both PRs were merged and commit `045b78aa56f00709471359d8e861c420222bda93` was deployed. Its tree exactly matches tested candidate `5d5d7ae6363f4562209bc5c94f4e2860d8d3903d`.

This approval covers that batch only, with no version bump, release tag or GitHub Release. [Deployment evidence](verification/pages-browser-performance.md) records the successful workflow, public artifact comparison and browser smoke checks. It does not authorize future deployments or declare remaining motion-quality issues complete.


## 2026-10-26 — groove-hand correction Pages approval

After reviewing the PR #43 change summary, test evidence and explicit undeployed status, the owner requested: “Pages 배포해서 내가 변경을 확인할 수 있게 해줘.” This authorizes publication of that reviewed correction. PR #43 was merged as `22e182a8153e3f409ac4a93bb49c9021e96ed15c`; the integrated tree equals tested head `09639306a0c87e9436ca42a144f2d1275b3f766b`. [Deployment evidence](verification/pages-browser-performance.md#groove-hand-correction-follow-up) records the successful Pages run. No tag or GitHub Release was created. This does not authorize subsequent batches.

## 2026-10-26 — automatic fitting and measured-clearance batch

The owner explicitly requested continuing through the stage before randomized
strike-location variation, reorganizing/updating issues, merging PRs where needed,
and deploying the result: “이것까지 고려해서 위치 분산 추가 작업 전까지 진행하고 배포해줘.
이슈 재정립과 업데이트, pr 병합이 필요하다면 진행하며 계속해줘.”

This authorizes the tested automatic-fit/menu and measured body-clearance batch,
including shared reference-pose/target-pair constraints, temporal torso/elbow
coordination and responsive preparation. Record the exact reviewed commit and
Pages workflow in [batch evidence](verification/body-clearance-measurements.md).
It does not authorize randomized strike variation, version changes, release tags
or GitHub Releases. Remaining fast-transfer and skin/clothing limits stay open;
this is not continuing deployment approval for later features.

The approved batch was merged as `dc581d203f7416b34388bca309f496c07e8d241d` through PR #45
and published by [Pages run 40261433421](https://github.com/postmelee/drum-vrm/actions/runs/40261433421).
The tested head and merge tree match; [publication verification](verification/body-clearance-measurements.md#pages-publication)
records actual artifact and browser checks. No tag or GitHub Release was created.

## 2026-10-26 — English interface, demo and contact batch

The owner explicitly requested an English-default website, a simple language
switch and matching English/Korean README links, then merge and Pages publication:
“그 다음 병합 및 pages 배포해줘.” The same instruction defers unresolved tom/cymbal
approach contacts and Twist automatic fitting until after deployment.

This covers #47 (public demo), #48 (equipment/contact/count-in) and #50 (English
interface). Tested UI head `28b2c246fc24ca9236283a825bf73256569cbca9` and integrated
candidate `2f9ca06cfa9fa19b493608e3e8f6881ac41257dc` have identical trees. The
[batch record](verification/pages-english-interface.md) records checks, publication
and retained limits. This is one batch's Pages approval, not permission for a
version bump, release tag, GitHub Release or later deployment. #18/#20/#23 remain
open for the deferred motion-quality work.

The approved main candidate was published by [Pages run 40281668719](https://github.com/postmelee/drum-vrm/actions/runs/40281668719).
The publication record verifies the served files against that workflow's artifact.

## 2026-10-26 — Official character preset batch

The owner explicitly requested that the previously local test VRMs be made
selectable from a website dropdown and deployed on Pages. The reviewed scope is
[PR #52](https://github.com/postmelee/drum-vrm/pull/52): unchanged official
AvatarSample_B, Seed-san and Twist samples, their recorded distribution terms,
on-demand selection through the existing importer, and bilingual guidance.
Other private/user models are not authorized by this artifact review.

Tested head `e16fee1a700cc851a18786cf415722239b108ac6` and integrated main candidate
`d23a9400abc9461633f26a9b805659026626a633` have identical trees. Publication evidence
belongs to the [preset verification record](verification/character-presets.md).
This approval does not cover a release/tag or further motion changes; Twist fit
and approach-contact limitations remain explicit.

## 2026-10-27 — Detailed hardware Pages batch

The owner requested realistic tom holders and crash stands, multi-view layout
inspection, and publication: “완료 후 내가 pages에서 확인 가능하게 배포해줘.”
This covers the detailed hardware increment and its pending prerequisites,
PR #55 (VRM 0.0 imports) and PR #57 (natural posture/contact), reviewed together
on the hardware branch. The [verification record](verification/detailed-hardware.md)
records tests and the explicitly incomplete full-song motion boundary. Its linked
PR publication record identifies the exact integrated commit, CI and served
artifact after manual deployment. No release/tag or randomized strike variation
is included, and this does not authorize later unrelated deployment.
