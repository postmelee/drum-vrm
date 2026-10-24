# 현재 상태와 다음 작업

갱신일: 2026-10-24 (Asia/Seoul). 상태: **사용자 승인 완료, M1·M2 MVP 구현 및 통합 검증 진행 중**.

## 저장소와 승인

- 공개 저장소: [postmelee/drum-vrm](https://github.com/postmelee/drum-vrm)
- [개발 계획](plan.md) · [5개 마일스톤](https://github.com/postmelee/drum-vrm/milestones) · [전체 이슈](https://github.com/postmelee/drum-vrm/issues?q=is%3Aissue)
- 사용자는 2026-10-24 계획·MIT 라이선스·MVP 구현·검증·GitHub Pages 배포와 이슈·문서 갱신을 승인했다. 근거는 [승인 기록](approval.md)과 #3이다. 같은 범위를 다시 승인받을 필요가 없다.
- 현재 집중 이슈는 #4–#10 구현 통합, #11 인수 검증, #12 배포다. 소스 구현과 이슈의 최종 완료 판정은 구분한다.
- 이 문서 갱신 시점에는 공개 배포 URL·배포 SHA를 확정하지 않았다. 예정 URL을 테스트 가능한 서비스로 안내하지 않는다.

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

1. 최신 사용자 메시지, 현재 브랜치/변경 파일, GitHub #4–#12 상태를 확인한다. 이 문서와 관련 PRD·ADR를 읽고 진행 중인 수정과 겹치지 않게 한다.
2. Node 24 이상, Rust 1.94.1/wasm target, wasm-pack 0.15.0을 확인한다. `npm ci` 후 `npm run build:wasm`을 실행한다. `src/wasm/`은 생성물이며 fresh checkout에 없다.
3. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `npm run typecheck`, `npm test`를 실행하고 실제 결과를 남긴다.
4. `npm run licenses`로 잠금 버전·원본 고지를 대조한 뒤 `npm run build`로 production 산출물을 만든다. 소스·공개 고지·라이선스 대장의 일치와 `.local/` 미포함을 확인한다.
5. `npm run dev` 또는 `npm run preview`의 `/drum-vrm/`에서 자체 데모와 파일 업로드를 확인한다. 30초 A/V 출력, 일반 플레이어 재생·923프레임·길이·오디오 onset, 분석/codec probe/출력 취소 후 재시도를 검증한다.
6. Pages 설정·검증 workflow를 마치고 승인된 배포를 실행한다. 실제 HTTPS URL에서 새로고침, WASM/Worker 로드, 업로드 → 재생 → 다운로드 경로를 다시 확인한다.
7. 실제 배포 URL·commit SHA·OS/브라우저 버전·검증 결과·남은 제한을 README, 이 문서, #11/#12와 해당 구현 이슈에 기록한다. M3·M4는 후속 백로그로 유지한다.

자세한 설치와 명령은 [README](../README.md)를 따른다. 추가 기능을 시작하기 전에 사용자의 실제 MIDI 피드백과 [계획](plan.md)의 후속 이슈 우선순위를 확인한다.

## 유지할 계약과 현재 제한

- 원본 MIDI tick·트랙·event index·음표·velocity·정수 μs 시각을 보존한다. 해결이 어려워도 원본 시각을 이동하지 않는다.
- 출력 시작 준비 0.5초·끝 여운 0.25초, 30fps를 프리뷰와 오디오에 함께 적용한다. `ceil((0.5 + D + 0.25) × 30)` 프레임이며 30초 곡은 923프레임이다.
- `PerformancePlan`은 타격·타점·손발 배정·진단의 기준이고, 리그는 세계 좌표 57개 Float32 값을 적용한다. 기본 체형은 `primitive-v1`으로 고정되어 있으며 VRM와 임의 체형 자동 보정은 아직 없다.
- 입력 상한: 10MiB, 선택 트랙 길이 300초, 512트랙, 전체 200,000이벤트, 선택한 지원 타격 50,000개. 같은 시각 그룹은 처음 32개까지 beam 후보를 확장하고 나머지는 소리/진단을 보존한다.
- 각 손발은 45ms보다 짧은 재타격, 12m/s를 넘는 타점 이동, 도달 불가능한 타격을 배정하지 않는다. 생체역학적 사실을 보장하는 수치가 아니라 MVP의 보수적 규칙이다. 충돌 회피·정밀 주법·사용자 손 편집은 후속이다.
- 미매핑 음표는 트랙/채널/음표별 개수를 알리고 합성 소리에 넣지 않는다. 지원하지만 배정 불가능한 타격은 합성 소리와 타점 강조를 유지한다.
- 기본 영상은 합성 오디오가 있는 720p30이다. 명시적 무음 선택 외에는 소리가 없는 파일을 성공으로 대체하지 않는다.
- WebCodecs는 secure context와 실제 브라우저/OS codec을 필요로 한다. Safari·Firefox·모바일 및 5분·많은 이벤트 입력의 전체 성능은 별도 측정 대상이다. 5분 stereo Float32 PCM 한 벌은 계산상 약110MiB이며 복사·압축 출력은 추가 메모리를 사용한다.
- 업로드 원본, 원 대화, 다운로드한 외부 자료는 `.local/`에만 두거나 브라우저 안에서 처리한다. 공개 fixture는 직접 생성한 자료만 사용한다.

## 장기 맥락과 출처

원 대화 두 턴의 사용자 2개·응답 2개 메시지 전체를 읽었고 원본은 `.local/source-conversation.json`에 로컬 보관했다. 공개 [맥락 문서](context/source-conversation.md)가 전체 주제와 현재 범위를 연결한다.

자체 코드는 MIT이고 원본 의존성 라이선스는 별개다. 실제 배포 고지는 `npm run licenses`가 생성한다. 외부 모션·모델·샘플·가중치는 편입하지 않았다. MMIP 조건, DRUMS/Disney 공개 코드·가중치, ARDrum 활용은 연구 대장의 미확인/후속 항목을 유지한다. MIDIAnimator GPL/Blender 유래 코드를 포팅하지 않았다.

현재 공개 사이트의 검증 결과와 SHA는 아직 이 문서에 기록하지 않았다. 단위 검사, 실제 브라우저 인코딩, 공개 Pages 실사용 확인을 서로 구분하여 완료 증거를 추가한다.

## 작업 환경 메모

로컬 작업 폴더는 `drum-vrm`이며 Git remote는 위 공개 저장소다. 샌드박스에서 GitHub DNS나 로컬 서버 listen이 제한될 수 있다. 필요한 네트워크 권한을 정상 승인 경로로 요청하며 토큰을 명령 출력·문서에 노출하지 않는다. 다른 기기에서는 기존 계정·권한을 확인한다.
