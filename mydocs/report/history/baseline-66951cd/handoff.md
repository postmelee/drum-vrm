> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/handoff.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../../specs/README.md) for current contracts and policy.

# 현재 상태와 다음 작업

## Latest foundation — instruction, technique and expression

The owner agreed with the snare/fit recommendation and asked to ground future performance expression in Yamaha instruction and prior references. Added the bilingual [performance foundation (EN)](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/performance-foundations.md) / [KO](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/performance-foundations.ko.md), README author/source credits and [ADR 0006](../../../../specs/decisions/0006-technique-and-expression-foundation.md). I01 credits Keiichi Nagata and © 2018 Yamaha Corporation; I02/I03 credit Riley Warren and Dr. Dave Gerhart. Keep source teaching, independent engineering choices and implemented behavior distinct. Rule IDs T01–T09 connect posture, grip, ready strokes, heel-down/up, pedal release, sticking, contact regions and phrase context to proposed checks. The expression budget preserves MIDI/audio and separates technique from theatrical amplitude. Physical crash instances and explicit arrangement edits must not be hidden inside that slider. Existing #18/#20/#14/#23 track the follow-ups without duplicate issues. Documentation only; no runtime feature, release or Pages deployment in this increment.

## Latest assessment — snare ready pose and kit fit

The owner requested a status report and a recommendation after observing repeated snare approaches and possible body/kit disproportion. [The engineering assessment](../../../tech/research/2026-10-24/snare-ready-and-kit-fit.md) traces the cause to the >0.42 s generic-rest return policy and fixed-direction reference wrists. VRM dimensions are preserved, while the initial kit layout/dimensions and stick length are common constants. The existing visual scale does not fit target reach. Prioritize a measured #18 placement baseline, then #20 instrument-specific ready points and target-dependent wrist/forearm motion as one solution, followed by explicit playing styles/clearance. Existing #18/#20 acceptance criteria track this proposal without a duplicate issue. No runtime changes were made for this assessment; PR #34 remains open, and release/Pages approval remains required.

갱신일: 2026-10-24 (Asia/Seoul). 상태: **v0.2.0 공개 유지, PR #33 병합 완료·미배포, 공통 보정·페달·하드웨어 PR #34 검토 중·미배포**.

## 현재 우선순위와 릴리스 경계

PR #31의 CI를 확인하고 `28a50a2c19c2b5109c490384f0b03e554f4dea1e`로 병합했다. 사용자는 #28·#29 구현 후 #30·#17 리그 개선을 지시했다. Lucide SVG, 연주자 시점, 결정적 선택형 타격 반응, Batoski/smoj 프리셋은 PR #32의 CI 성공 후 `bd1568d4a3cc89913627b04e34f2fd2153a3ffac`로 병합했다. VRM 1.0 normalized humanoid와 손목/그립·실제 렌더 접촉 계측, 로컬 파일·조건 표시는 [PR #33](https://github.com/postmelee/drum-vrm/pull/33)의 CI 성공 후 `a2105a5508ae92e49d31977e2c56b096dfd1d822`로 병합했다. 구현·수치 검증·브라우저 검증은 완료했으며 릴리스 승인 대상 묶음이다. Rust 22개/웹 63개, 타입 검사와 세 실제 모델의 128프레임·탐색 검증이 통과했다. 실제 앱 업로드·재생·1인칭과 923프레임 오디오 포함 영상 인코딩을 확인했다. 자동화 브라우저의 blob 수신 제한으로 새 다운로드 파일의 별도 플레이어/ffprobe 검증은 남아 있다. [이번 검증 기록](../../verification/vrm-studio-pr33.md)에 범위와 제한을 구분했다. #18은 체형별 검증을 추적한다. [VRM 모델 조사](../../../tech/research/2026-10-24/vrm-models.md), [영문 안내](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/models.md), [한국어 안내](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/models.ko.md)를 추가했고 #17/#18/#30의 완료 조건을 갱신했다. 외부 VRM 파일은 `.local/vrm-test-models`에만 있으며 공개 저장소/배포물에 포함하지 않는다.

**사용자 승인 전 새 태그·Release·Pages 배포 금지.** 이슈·PR·CI를 진행한 후 검토 가능한 변경 묶음으로 승인받아 한 번에 릴리스한다. 과거 MVP 승인과 아래 배포 기록은 향후 배포 권한이 아니다. [릴리스 정책](release-policy.md)을 따른다. README는 [영어](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/README.md)와 [한국어](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/README.ko.md)를 함께 유지한다.

## Calibration batch — implemented, unreleased

PR #33 passed CI and merged at `a2105a5508ae92e49d31977e2c56b096dfd1d822`. Its numerical/import/encoding checks are recorded in [the unreleased VRM verification](../../verification/vrm-studio-pr33.md); that evidence does not automatically validate the new batch below. Main and public Pages remain separate states.

The latest owner instruction is to implement shared seat height/forward and contact adjustment first, keep feet on pedals with ankle-only strokes immediately, improve the generic visible grip, then keep stands/pedals while replacing drum bodies. #18/#14/#20 completion criteria were updated without duplicate issues. [ADR 0005](../../../../specs/decisions/0005-calibration-pedals-hardware.md) and the [English](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/calibration.md)/[Korean](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/calibration.ko.md) user guides preserve the contract.

- Seat controls: 30–120 cm height, −50–50 cm forward position, and automatic fitting. Selected-role XYZ contact adjustment couples closed/open hi-hat while keeping pedalHat independent. Update kit contacts and corresponding hit targets together, then replan with the measured body. Settings are session-only.
- Feet: the Rust profile path now anchors LF/RF to pedalHat/kick even with no foot notes. Fixed hip/knee/ankle chains, small ankle rotation, exact impact contact, and unreachable-pedal diagnostics preserve body lengths and original MIDI. `cargo test --locked` passed 26 tests; formatting and clippy passed for this increment. Web 76 tests, typecheck, WASM/production builds and license generation passed; see [the integrated verification](../../verification/calibration-644fe185.md).
- Grip/hardware: generic palm-transverse socket and available finger curl; common standalone hardware and body-only preset/component replacement are implemented. Imported local GLB must have hardware removed by its author. Do not claim per-skin contact or physically exact mounts from socket metrics.
- Playing style: existing RH-hi-hat/LH-snare preference is a planner cost. Explicit crossed/open/mirrored styles, hand overrides and collision-aware arm/stick ordering remain #14/#20 follow-up work; do not present them as implemented.
- Verification: full original VRMs passed 128-frame/shuffled-seek browser checks; Twist hand close-ups show the grip. Seat/target controls, preset/GLB swaps and a 923-frame synthesized-audio WebM encode were exercised. Independent playback of the new blob file and the controlled two-export VRoid proportion pair remain open. Implementation commit: `644fe1859190b8e427a244206db27a7f5d15938e`, [PR #34](https://github.com/postmelee/drum-vrm/pull/34), branch `codex/calibration-pedals-hardware`. The PR remains open for review; consult its latest checks for CI status. No release.

## Palm direction follow-up — implemented, unreleased

PR #34 now includes the requested up-facing palm option (default), inward/down alternatives, a diagonal measured grip, reference hand/forearm alignment and forearm-carried roll. Selection persists across seat changes and model import within the session. Web 79 tests and typecheck/build passed; three actual VRMs passed 128-frame/seek browser checks. See [the separate verification](../../verification/palm-orientation-2026-10-24.md), including measured remaining 99–110° hi-hat wrist bends. Reference wrists are fixed; dynamic target-dependent wrist limits are **not**. #20 tracks that priority before claiming natural performance hands. No release or Pages deployment.

## v0.2 사용자 피드백 반영 상태

사용자는 최소 Logic Pro형 작업 화면, 흰색/검정 테마, 지정 사이트 시스템 글꼴, 캐릭터·드럼·업로드/연주정보·내보내기 네 메뉴, 스켈레톤, 드럼 모델 교체를 요청했다. **#25 / M3**로 구현했다. [ADR0003](../../../../specs/decisions/0003-minimal-studio-and-models.md), [모델·라이선스 조사](../../../tech/research/2026-10-24/drum-assets.md), [v0.2 검증](../../verification/studio-0.2.md)을 우선 읽는다.

- 상단 transport/네 inspector/하단 피아노롤. 패널은 기본 닫힘. 글꼴은 지정 사이트와 동일한 시스템 fallback.
- 기본 스켈레톤, 350ms 정면 카메라 전환/reduced-motion, 부품 클릭/폼선택, 자체3종+InnersceneCC0 프리셋, 부품별 GLB/모델/색상/크기/회전/복원.
- `timeline.ts`와 Rust메타데이터(ppq/tempoMap/timeSignatures)로 실제 템포·박자 기반 눈금. 음표편집은 없음.
- `model-procedural.ts`, `model-import.ts`, `model-library.ts`가 외형을 분리. 정적 self-containedGLB최대25MiB. 캐릭터VRM/GLB로드는 여전히 #17/#18.
- `public/models`에CC0원본·라이선스·SHA256·NOTICE를 보존. `npm run licenses`가검증한다. Apple에셋은포함하지않는다.
- 로컬 Rust18개/웹27개, 타입·빌드·라이선스·실제GLB교체·오디오포함923프레임출력 검증완료. [PR CI](https://github.com/postmelee/drum-vrm/actions/runs/40063272293)와 [Pages 배포](https://github.com/postmelee/drum-vrm/actions/runs/40063363035) 성공. 공개주소에서도 모델·16배 확대·오디오포함영상출력 확인. 배포 SHA `3e8be9333cedbc9845dbc3e4cebd73961af697b4`, [v0.2.0 artifact](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0).

## 저장소와 승인

- 공개 저장소: [postmelee/drum-vrm](https://github.com/postmelee/drum-vrm)
- [개발 계획](plan.md) · [5개 마일스톤](https://github.com/postmelee/drum-vrm/milestones) · [전체 이슈](https://github.com/postmelee/drum-vrm/issues?q=is%3Aissue)
- 사용자는 2026-10-24 계획·MIT 라이선스·MVP 구현·검증·GitHub Pages 배포와 이슈·문서 갱신을 승인했다. 근거는 [승인 기록](../approvals.md)과 #3이다. 기존 기능 개발 승인과 별개로, 최신 요청에 따라 새 릴리스·배포는 배치별 명시적 승인이 필요하다.
- M0–M2 / #1–#12를 완료했다. 다음 입력은 사용자의 실제 DAW MIDI 피드백이며 M3(#13–#18)·M4(#19–#23)는 열린 후속 백로그다.
- 테스트 URL: [https://postmelee.github.io/drum-vrm/](https://postmelee.github.io/drum-vrm/) · 배포 SHA: `3e8be9333cedbc9845dbc3e4cebd73961af697b4` · [성공 run](https://github.com/postmelee/drum-vrm/actions/runs/40063363035) · [릴리스/정적 artifact](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0).

## 구현된 경로

| 영역 | 시작 파일 | 현재 구현 |
|---|---|---|
| MIDI/계획 | `crates/drum-engine/src/midi.rs`, `motion.rs`, `model.rs` | strict SMF 0/1·PPQ, 정수 템포 누산, 트랙/채널 선택, GM 매핑, beam-8·lookahead-2 배정, 도달/이동 진단, 결정적 2-bone IK |
| WASM/Worker | `src/engine.ts`, `engine.worker.ts` | Worker에서 분석, 취소 시 종료, 검증된 계획을 메인 WASM evaluator로 전달 |
| 리그/장면 | `src/scene.ts` | 57-float canonical pose → 전신 PrimitiveRigAdapter, 자체 키트·무대, 카메라/배경, 타점과 심벌 효과 |
| 음향/재생 | `src/audio.ts`, `synthesis.ts`, `audio.worker.ts` | 자체 합성 48kHz stereo PCM, Worker 사전 합성, Web Audio 시계, pause/resume/seek |
| 내보내기 | `src/export.ts` | 실제 codec probe와 짧은 시험 인코딩, 720p30 고정 시각, 합성 오디오 mux, 진행/취소/정리 |
| 앱/데모 | `src/main.ts`, `style.css`, `demo.ts` | 업로드·트랙 선택·진단·타임라인·영상 저장 UI, 직접 생성한 30초 데모 |
| 검사/출처 | Rust `tests.rs`, `tests/`, `scripts/licenses.mjs` | 입력·시간·접촉·결정성 및 미디어/리그/취소 계약 검사, 잠금 버전 기반 고지 생성 |

구현 파일이 있다는 사실만으로 A01–A16을 모두 통과한 것은 아니다. 전체 검사 결과·브라우저·출력 파일 검증과 배포 증거는 #11/#12의 완료 시점에 기록한다.

## 다음 작업자가 이어갈 순서

1. 위 Calibration batch와 [ADR 0005](../../../../specs/decisions/0005-calibration-pedals-hardware.md), #18/#14/#20의 갱신된 완료 조건, [PR #34](https://github.com/postmelee/drum-vrm/pull/34)의 최신 검증·검토 상태를 먼저 읽는다. 좌석·타점/페달 → 공통 하드웨어/교체 구현과 로컬 검증은 완료했다. M3·M4 전체나 교차 주법 후속을 자동 착수하지 않는다.
2. `npm ci`, `npm run build:wasm` 후 `npm run dev`의 `/drum-vrm/`을 연다. Rust 1.94.1, wasm-pack 0.15.0, Node 24를 사용한다.
3. 변경에 맞는 Rust/JS 검사를 실행하고, 엔진 변경은 WASM을 다시 생성한다. 새 codec/타임라인 변경은 실제 출력 파일의 프레임·오디오 onset까지 확인한다.
4. PR과 CI를 완료하고 변경 묶음·검증 commit·미해결 제한을 정리한다. main 반영과 공개 배포를 구분하고, 승인되지 않은 배포는 실행하지 않는다.
5. 해당 배치의 사용자 릴리스 승인을 받은 경우에만 태그·Release·Pages를 진행한다. 승인 근거와 배포 SHA·artifact·checksum, 공개 URL 검증 결과를 기록한다.

자세한 설치는 [README](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/README.md)를 따른다. 첫 배포 이전 artifact는 없으며 v0.1.0의 실제 Pages artifact를 릴리스 자산으로 보존했다. 이후 회귀 시 해당 artifact 또는 태그로 복구안을 준비하고 검증한다. 공개 재배포도 명시적 사용자 지시를 따른다. 빌드 Actions artifact 보존 기간은 1일이므로 릴리스 자산을 장기 기준으로 쓴다.

## 유지할 계약과 현재 제한

- 원본 MIDI tick·트랙·event index·음표·velocity·정수 μs 시각을 보존한다. 해결이 어려워도 원본 시각을 이동하지 않는다.
- 출력 시작 준비 0.5초·끝 여운 0.25초, 30fps를 프리뷰와 오디오에 함께 적용한다. `ceil((0.5 + D + 0.25) × 30)` 프레임이며 30초 곡은 923프레임이다.
- `PerformancePlan`은 타격·타점·손발 배정·진단의 기준이다. 공개 v0.2의 기본 체형은 `primitive-v1`/57-float이다. 병합·미배포 PR #33은 별도 JSON RigProfile과 73-float 자세로 VRM 1.0 및 기본 semantic Bone 리그를 공통 처리한다. 현재 묶음은 좌석·타점 수동 보정을 구현했으며 일반 GLB 매핑·정밀 피부/신발 보정은 #17/#18에 남는다.
- 입력 상한: 10MiB, 선택 트랙 길이 300초, 512트랙, 전체 200,000이벤트, 선택한 지원 타격 50,000개. 같은 시각 그룹은 처음 32개까지 beam 후보를 확장하고 나머지는 소리/진단을 보존한다.
- 각 손발은 45ms보다 짧은 재타격, 12m/s를 넘는 타점 이동, 도달 불가능한 타격을 배정하지 않는다. 생체역학적 사실을 보장하는 수치가 아니라 MVP의 보수적 규칙이다. 충돌 회피·정밀 주법·사용자 손 편집은 후속이다.
- 미매핑 음표는 트랙/채널/음표별 개수를 알리고 합성 소리에 넣지 않는다. 지원하지만 배정 불가능한 타격은 합성 소리와 타점 강조를 유지한다.
- 기본 영상은 합성 오디오가 있는 720p30이다. 명시적 무음 선택 외에는 소리가 없는 파일을 성공으로 대체하지 않는다.
- WebCodecs는 secure context와 실제 브라우저/OS codec을 필요로 한다. Safari·Firefox·모바일 및 5분·많은 이벤트 입력의 전체 성능은 별도 측정 대상이다. 5분 stereo Float32 PCM 한 벌은 계산상 약110MiB이며 복사·압축 출력은 추가 메모리를 사용한다.
- 업로드 원본, 원 대화, 다운로드한 외부 자료는 `.local/`에만 두거나 브라우저 안에서 처리한다. 공개 fixture는 직접 생성한 자료만 사용한다.

## 장기 맥락과 출처

원 대화 두 턴의 사용자 2개·응답 2개 메시지 전체를 읽었고 원본은 `.local/source-conversation.json`에 로컬 보관했다. 공개 [맥락 문서](../requirements-origin.md)가 전체 주제와 현재 범위를 연결한다.

자체 코드는 MIT이고 원본 의존성 라이선스는 별개다. 실제 배포 고지는 `npm run licenses`가 생성한다. 외부 모션·샘플·ML 가중치는 편입하지 않았다. v0.2에서 사용자 요청으로 Innerscene CC0 드럼 모델과 포함 텍스처를 추가했다. MMIP 조건, DRUMS/Disney 공개 코드·가중치, ARDrum 활용은 연구 대장의 미확인/후속 항목을 유지한다. MIDIAnimator GPL/Blender 유래 코드를 포팅하지 않았다.

최초 v0.1.0 배포 당시 Rust 16개·웹 14개 테스트와 PR/배포 CI가 통과했다. 공개 사이트에서 채널2/두 템포의 30초 MIDI 업로드·재생·seek·923프레임 WebM 다운로드를 확인했다. 첫 오디오 onset 오차는 약6.5ms, 실제 파일은30.78초였다. 자세한 명령·한계는 검증 기록에 유지한다.

## 작업 환경 메모

로컬 작업 폴더는 `drum-vrm`이며 Git remote는 위 공개 저장소다. 샌드박스에서 GitHub DNS나 로컬 서버 listen이 제한될 수 있다. 필요한 네트워크 권한을 정상 승인 경로로 요청하며 토큰을 명령 출력·문서에 노출하지 않는다. 다른 기기에서는 기존 계정·권한을 확인한다.
