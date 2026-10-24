# 현재 상태와 다음 작업

갱신일: 2026-10-24 (Asia/Seoul). 상태: **M0·M1·M2 완료, v0.1.0 공개 배포 및 사용자 테스트 준비 완료**.

## 저장소와 승인

- 공개 저장소: [postmelee/drum-vrm](https://github.com/postmelee/drum-vrm)
- [개발 계획](plan.md) · [5개 마일스톤](https://github.com/postmelee/drum-vrm/milestones) · [전체 이슈](https://github.com/postmelee/drum-vrm/issues?q=is%3Aissue)
- 사용자는 2026-10-24 계획·MIT 라이선스·MVP 구현·검증·GitHub Pages 배포와 이슈·문서 갱신을 승인했다. 근거는 [승인 기록](approval.md)과 #3이다. 같은 범위를 다시 승인받을 필요가 없다.
- M0–M2 / #1–#12를 완료했다. 다음 입력은 사용자의 실제 DAW MIDI 피드백이며 M3(#13–#18)·M4(#19–#23)는 열린 후속 백로그다.
- 테스트 URL: [https://postmelee.github.io/drum-vrm/](https://postmelee.github.io/drum-vrm/) · 배포 SHA: `9ddc2bf2450600e1eaca5bd405a4221e254ce83b` · [성공 run](https://github.com/postmelee/drum-vrm/actions/runs/40060762280) · [릴리스/정적 artifact](https://github.com/postmelee/drum-vrm/releases/tag/v0.1.0).

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

1. [검증 기록](verification/mvp-0.1.md)과 사용자 피드백을 읽고 재현할 MIDI 사례/관련 이슈를 정한다. M3·M4를 자동으로 모두 착수하지 않는다.
2. `npm ci`, `npm run build:wasm` 후 `npm run dev`의 `/drum-vrm/`을 연다. Rust 1.94.1, wasm-pack 0.15.0, Node 24를 사용한다.
3. 변경에 맞는 Rust/JS 검사를 실행하고, 엔진 변경은 WASM을 다시 생성한다. 새 codec/타임라인 변경은 실제 출력 파일의 프레임·오디오 onset까지 확인한다.
4. PR CI 후 main에 반영하고 `gh workflow run pages.yml --ref main`으로 배포한다. 공개 URL에서 실제 업로드·출력 경로를 확인한다.
5. 새 릴리스의 배포 SHA·artifact·checksum과 검증 결과, 관련 이슈 및 이 문서를 갱신한다.

자세한 설치는 [README](../README.md)를 따른다. 첫 배포 이전 artifact는 없으며 v0.1.0의 실제 Pages artifact를 릴리스 자산으로 보존했다. 이후 회귀 시 해당 artifact를 재배포하거나 해당 태그의 앱 소스로 main을 복구한 뒤 검증·배포한다. 빌드 Actions artifact 보존 기간은 1일이므로 릴리스 자산을 장기 기준으로 쓴다.

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

Rust 16개·웹 14개 테스트와 PR/배포 CI가 통과했다. 공개 사이트에서 채널2/두 템포의 30초 MIDI 업로드·재생·seek·923프레임 WebM 다운로드를 확인했다. 첫 오디오 onset 오차는 약6.5ms, 실제 파일은30.78초였다. 자세한 명령·한계는 검증 기록에 유지한다.

## 작업 환경 메모

로컬 작업 폴더는 `drum-vrm`이며 Git remote는 위 공개 저장소다. 샌드박스에서 GitHub DNS나 로컬 서버 listen이 제한될 수 있다. 필요한 네트워크 권한을 정상 승인 경로로 요청하며 토큰을 명령 출력·문서에 노출하지 않는다. 다른 기기에서는 기존 계정·권한을 확인한다.
