# 개발 계획과 GitHub 작업 지도

기준일: 2026-10-24 (Asia/Seoul). 현재 단계는 **M0·M1·M2 완료, v0.2.0 공개 유지, 후속 조사·이슈·PR 준비**이다. 실행할 이슈는 [GitHub Issues](https://github.com/postmelee/drum-vrm/issues), 범위·종료 기준은 아래 문서와 마일스톤으로 관리한다.

## Current execution order — 2026-10-24

The owner requested a research-first assessment of studio controls, additional free kits, hand/grip quality, and VRM compatibility. Application features below remain unimplemented; this update prepares issues and review documentation. Public v0.2.0 stays unchanged.

| Priority | Work | Issue / milestone | Boundary |
| --- | --- | --- | --- |
| Now | Preserve Korean README, add English, record batch-release policy and audit | [#27](https://github.com/postmelee/drum-vrm/issues/27) / M3 | Documentation/review PR; no release |
| Next, small PRs | Lucide SVG controls; player camera; optional hit response | [#28](https://github.com/postmelee/drum-vrm/issues/28) / M3 | Icons can start independently; player camera requires a left/right fixture; response must preserve contact and seek/export determinism |
| Parallel | Batoski then smoj preset adaptation; investigate electronic kit | [#29](https://github.com/postmelee/drum-vrm/issues/29) / M3 | Only verified model rights/files enter implementation; electronic file remains unverified |
| Foundation before naturalness | Rendered contact baseline, wrist/grip orientation, versioned rig profile | [#30](https://github.com/postmelee/drum-vrm/issues/30) + #17 / M3 | Pull minimal measurement out of #23 and basic grip out of #20; preserve their broader backlog |
| After the foundation | Two-body seat/grip/target/pedal calibration | #18 / M3 | Preserve source-event timing, recompute body-dependent reach/assignment as needed |
| Next production phase | WAV sync, mapping/hand editing, persistence, higher-resolution output | #13–#16 / M3 | Keep open; stabilize model/rig settings before project persistence; validate current output before raising resolution |
| Later | Advanced expression, motion baking, learned models, platform expansion, full inspection tools | #19–#23 / M4 | Literature review can happen now; model adoption follows a measured baseline and verified code/data/weight terms |

M3/M4 are scope groupings, **not automatic release triggers**. Keep the existing milestone identities; the latest priorities refine execution order rather than discard earlier user requirements. No issue is closed merely because its analysis is complete.

Research: [rig/motion audit](research/rig-and-studio-audit.md), [additional models](research/drum-assets-followup.md). Each feature PR should update user-facing English/Korean guidance and record relevant validation. After integration, prepare one reviewable batch with exact commit and evidence. Explicit owner approval is required before release tags, GitHub Releases, or Pages deployment; see [release policy](release-policy.md).

## 승인된 최초 MVP 범위 (기록)

M1·M2를 구현하여 MIDI를 올리고 기본 리그의 연주를 재생한 뒤 영상 파일을 저장하는 MVP를 GitHub Pages에 공개한다. Rust/WASM과 Three.js를 사용한다. 자체 리그·키트와 합성 드럼 소리를 사용하고 자체 소스에 MIT를 채택했다. 브라우저 인코딩과 Mediabunny는 실제 채택 버전의 라이선스·대응 소스 고지를 포함한다.

첫 지원 대상은 검증한 데스크톱 Chromium 환경이다. 입력 상한은 10MiB와 곡 길이 5분, 주 인수 사례는 자체 30초 MIDI다. 720p30 영상과 합성 오디오를 기본 출력한다. 실제 기기에서 가능한 MP4 또는 WebM을 제공하며 사용자가 명시적으로 선택하는 무음 출력 옵션을 제공한다. 모든 기기·코덱 지원을 약속하지 않는다.

첫 타격의 준비를 위해 기본 pre-roll 0.5초, 마지막 반동을 위해 tail 0.25초를 둔다. 영상·소리를 같은 양만큼 이동하고 MIDI 내부 시각은 그대로 유지한다. 전체 계약과 수치 기준은 [요구사항](product-requirements.md)과 [아키텍처](architecture.md)에 정의한다.

## 마일스톤

| 마일스톤 | 결과와 종료 조건 | 이슈 |
| --- | --- | --- |
| [M0 계획과 구현 승인](https://github.com/postmelee/drum-vrm/milestone/1) | 대화 전체 추적, 설계·연구·권리 대장, 작업 분해 및 실제 사용자 승인 기록 | #1–#3 |
| [M1 MIDI에서 기본 리그 연주까지](https://github.com/postmelee/drum-vrm/milestone/2) | 사전 분석 → 손발 계획 → 기본 리그 → 재생·일시정지·탐색, 타격 시각·접촉 검증 | #4–#8 |
| [M2 영상 출력과 Pages MVP](https://github.com/postmelee/drum-vrm/milestone/3) | 720p30 합성 A/V 다운로드, 실패·취소, 인수 기준, 실제 Pages 핵심 경로 검증 | #9–#12 |
| [M3 커버 제작과 캐릭터 교체](https://github.com/postmelee/drum-vrm/milestone/4) | WAV 싱크, 편집·프로젝트 저장, 고해상도, VRM/공통 리그 GLB·체형 보정 | #13–#18 |
| [M4 모션 품질과 제작 파이프라인](https://github.com/postmelee/drum-vrm/milestone/5) | 3D 출력, 스타일·주법, 권리가 확보된 학습 실험, 다른 실행 환경, 기록·검수 도구 | #19–#23 |

M3·M4는 장기 방향을 보존한 백로그다. 첫 MVP 승인이나 완료를 해당 전체 범위의 실행 지시로 확대하지 않는다. 사용자 승인은 2026-10-24에 완료되었다. #6 계획기와 #9 인코딩의 실제 검증 결과에 따라 후속 품질·지원 환경을 정하며, MVP 승인으로 전체 장기 백로그를 자동 착수하지 않는다.

## 최초 MVP 실행 순서와 병렬 작업 (완료 기록)

1. #1·#2의 준비 결과와 #3에 기록된 2026-10-24 승인을 유지한다. 같은 범위를 재승인 요청하지 않는다.
2. #4에서 재현 가능한 Rust/WASM·웹 빌드와 최소 CI를 구성한다.
3. #5 → #6 → #7 → #8로 MIDI에서 리그 재생까지 연결한다. 동시에 #9에서 짧은 영상·오디오 인코딩과 메모리·취소를 검증한다.
4. #7과 #9가 통과하면 #10으로 영상 출력 UI와 합성 오디오 결합을 완성한다.
5. #11에서 원본 시각·공간 오차·실제 영상·오디오·오류 경로·출처를 검증한다.
6. #12에서 Pages 배포 후 실제 URL로 업로드 → 재생 → 출력 경로를 확인하고 사용자에게 테스트 절차를 전달한다.
7. 사용자가 실제 MIDI로 확인한 피드백을 이슈화하고 M3 우선순위를 정한다.

MVP의 긴 의존 경로는 #3 → #4 → #5 → #6 → #7 → #8/#10 → #11 → #12다. #9는 #4 직후 병렬 진행하여 브라우저 인코딩 실패를 늦게 발견하지 않도록 한다.

## 완료된 구현과 증거

| 이슈 묶음 | 현재 코드 | 완료 증거 |
|---|---|---|
| #4–#6 | 고정 Rust/npm 의존성, strict 파서, 전역 정수 템포맵, beam 배정/IK, WASM Worker | Rust16개 검사·fmt·Clippy·WASM 및 CI 통과 |
| #7–#8 | PrimitiveRigAdapter, 키트, 합성, 재생/탐색 UI | 계약 검사와 실제 브라우저 업로드·pause/seek·분석 취소 복구 |
| #9–#10 | 실제 A/V round-trip probe, 고정720p30, 진행·취소·다운로드 | VP9/Opus923프레임·30.78초·onset약6ms, 명시적 무음/취소 검증 |
| #11–#12 | 검증 기록·CI·정적 배포 | 공개 Pages 업로드→재생→영상 다운로드 및 라이선스 고지 확인 |


이 표는 최초 MVP 구현과 검증 기록이며 현재 상태는 상단과 GitHub 이슈를 기준으로 한다. v0.1.0과 v0.2.0 배포 검증은 각각의 검증 문서에 보존했다. 입력 상한은 10MiB·300초·512트랙·200,000이벤트·지원 타격50,000개로 구현했다. 최대 길이 성능과 모든 브라우저 지원을 뜻하지 않는다.

## 전체 이슈

각 이슈 본문에는 선행 이슈, 체크 가능한 완료 조건, 설계 링크, 실제 검증 기록 위치가 있다. 아래는 생성 시점의 연결 지도이며 진행 상태는 GitHub에서 확인한다.

| 이슈 | 단계 | 작업 | 선행 이슈 |
| --- | --- | --- | --- |
| [#1](https://github.com/postmelee/drum-vrm/issues/1) | M0 | 계획과 원본 대화 요구 추적 확정 | — |
| [#2](https://github.com/postmelee/drum-vrm/issues/2) | M0 | 참고자료와 라이선스 대장 검증 | — |
| [#3](https://github.com/postmelee/drum-vrm/issues/3) | M0 | 사용자 MVP 범위 승인 기록 | #1, #2 |
| [#4](https://github.com/postmelee/drum-vrm/issues/4) | M1 | Rust 코어·WASM·Three.js 웹 셸 구성 | #3 |
| [#5](https://github.com/postmelee/drum-vrm/issues/5) | M1 | MIDI 파싱·템포맵·트랙 선택·입력 진단 | #4 |
| [#6](https://github.com/postmelee/drum-vrm/issues/6) | M1 | 손발 배정과 결정적 준비·접촉·반동 계획 | #5 |
| [#7](https://github.com/postmelee/drum-vrm/issues/7) | M1 | 확장 가능한 기본 리그·드럼 키트·스틱 접촉 | #4, #6 |
| [#8](https://github.com/postmelee/drum-vrm/issues/8) | M1 | 분석 상태·재생 제어·타임라인·기본 미리듣기 | #5, #7 |
| [#9](https://github.com/postmelee/drum-vrm/issues/9) | M2 | 브라우저 영상 인코딩 가능성·메모리 스파이크 | #4, #2 |
| [#10](https://github.com/postmelee/drum-vrm/issues/10) | M2 | 고정 FPS 영상 다운로드·진척도·취소 | #7, #9 |
| [#11](https://github.com/postmelee/drum-vrm/issues/11) | M2 | MVP 수용 기준·픽스처·배포 검증 | #8, #10 |
| [#12](https://github.com/postmelee/drum-vrm/issues/12) | M2 | GitHub Pages 배포와 사용자 테스트 안내 | #11, #3 |
| [#13](https://github.com/postmelee/drum-vrm/issues/13) | M3 | WAV 업로드·오프셋·오디오와 영상 동기화 | #12 |
| [#14](https://github.com/postmelee/drum-vrm/issues/14) | M3 | 매핑·손 배정·키트 편집과 문제 구간 수정 | #12 |
| [#15](https://github.com/postmelee/drum-vrm/issues/15) | M3 | 프로젝트 저장·재열기와 한 곡 완주 | #12 |
| [#16](https://github.com/postmelee/drum-vrm/issues/16) | M3 | 고해상도·고FPS·합성용 출력 | #10 |
| [#17](https://github.com/postmelee/drum-vrm/issues/17) | M3 | VRM/공통 리그 GLB 어댑터와 모델 조건 표시 | #12 |
| [#18](https://github.com/postmelee/drum-vrm/issues/18) | M3 | 체형별 좌석·스틱·타점·페달 보정 | #17 |
| [#19](https://github.com/postmelee/drum-vrm/issues/19) | M4 | GLB/VRMA 모션 베이크와 재편집 출력 | #18, #15 |
| [#20](https://github.com/postmelee/drum-vrm/issues/20) | M4 | 연주 스타일·특수 주법·표현 품질 확장 | #14, #18 |
| [#21](https://github.com/postmelee/drum-vrm/issues/21) | M4 | 학습 모델·모션 데이터의 채택 조건 연구 | #20, #2 |
| [#22](https://github.com/postmelee/drum-vrm/issues/22) | M4 | Rust 코어의 CLI·Tauri·Blender·라이브 MIDI 확장 검토 | #15 |
| [#23](https://github.com/postmelee/drum-vrm/issues/23) | M4 | 모션 기록·스틱 궤적·접촉 오차 검수 도구 | #12, #2 |

## 주요 위험과 해결 기준

| 위험 | 먼저 확인할 작업 | 판단과 대응 |
| --- | --- | --- |
| 코덱·mux·메모리 차이 | #9 | 실제 설정으로 인코딩 및 재생 확인. 미지원은 사전에 알리고 무음/누락 영상으로 성공을 가장하지 않는다. |
| 손 배정이 실제로 불가능 | #6·#7 | 시각을 고정하고 진단. 레이어·매핑·손 고정 편집은 #14로 확장한다. |
| 0초 첫 타격·출력 끝 누락 | #6·#10·#11 | pre-roll/tail과 프레임 수·오디오 시간 계약을 공통으로 검증한다. |
| 캐릭터 변경 시 접촉 이탈 | #7·#18 | 리그 어댑터와 치수·그립 계약 유지. 체형 변경 뒤 재보정한다. |
| 외부 데이터의 권리 불명확 | #2·#21·#23 | MVP는 자체 에셋·fixture. 논문/코드/데이터/가중치·에셋을 따로 추적한다. |
| Pages 하위 경로의 자산 오류 | #4·#12 | /drum-vrm/에서 WASM·Worker·자산 경로와 새로고침을 실제 확인한다. |

## 검증과 완료 정의

MVP는 PRD의 수용 기준을 실제로 실행한 뒤 완료한다. 단위 검사는 템포맵·이벤트 보존·손발 충돌·임의 시점 자세를 검증하고, 브라우저 검사는 파일 입력·오디오 제스처·탐색·취소를 검증한다. 출력 파일은 프레임 수·타임스탬프·길이·소리 정렬 및 일반 플레이어 재생을 검사한다. 테스트한 OS·브라우저 버전·commit SHA와 제한을 남긴다.

Rust MIDI/계획·WASM 연결, 기본 리그/키트, 합성 오디오, UI, codec probe와 고정 FPS export의 코드와 자동 검사가 마련되었다. 최초 MVP와 v0.2.0의 통합 인수·배포 검증은 완료했다. 후속 변경도 검사 코드가 있다는 사실이나 개별 단위 검사 성공만으로 전체 통과를 표시하지 않는다. 배포 시에는 자동 검사, 실제 브라우저 A/V 파일 검사와 공개 Pages 실사용 확인을 각각 기록한다.

## 장기 맥락을 유지하는 규칙

- 작업 시작점: [handoff](handoff.md) → 현재 이슈 → PRD·아키텍처 → 관련 ADR·참고자료.
- 결정 변경: 원 요구를 삭제하지 않고 이유·영향·대체 범위를 ADR와 해당 이슈에 기록한다.
- 출처 추가: 정확한 버전/commit과 LICENSE/NOTICE, 실제 포함 파일 및 변경 내용을 대장에 추가한다.
- 작업 종료: 이슈의 검증 결과, 다음 작업, 문서 변경, 현재 한계를 남기고 handoff를 갱신한다.
- 승인: [승인 기록](approval.md)과 #3에 2026-10-24 실제 사용자 승인을 기록했다. 이는 최초 M1·M2와 MIT 채택의 과거 승인이다. 새 기능은 이슈·PR로 추적하며, 후속 릴리스·Pages 배포는 최신 배치별 명시적 승인 정책을 따른다.

완료 근거: [v0.1.0 검증](verification/mvp-0.1.md), [배포](https://github.com/postmelee/drum-vrm/actions/runs/40060762280), [테스트 사이트](https://postmelee.github.io/drum-vrm/). M3·M4는 열린 상태로 유지한다.

## 사용자 피드백 반영 — v0.2

M3의 [#25](https://github.com/postmelee/drum-vrm/issues/25)를 우선한다. 최소 작업 화면, 네 inspector, 피아노 롤, 스켈레톤, 드럼 정면 편집과 라이선스 검증 모델 프리셋을 포함한다. [ADR0003](decisions/0003-minimal-studio-and-models.md)가 범위와 동작을 고정한다. #14의 음표/손 배정 편집과 #17/#18의 캐릭터 리타게팅 전체는 계속 후속이다. 외부 CC0 키트의 편입은 최초 MVP의 자체 에셋만 사용하던 범위를 이번 사용자 요청으로 확장한 것이다.

#25는 [PR #26](https://github.com/postmelee/drum-vrm/pull/26)으로 완료했고, v0.2.0을 [Pages](https://github.com/postmelee/drum-vrm/actions/runs/40063363035)에 배포했다. [검증 기록](verification/studio-0.2.md)과 [정적 artifact](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0)를 후속 기준으로 사용한다. M3 전체는 열린 상태를 유지한다.
