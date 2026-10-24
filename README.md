# Drum VRM

DAW에서 만든 드럼 MIDI를 3D 드러머의 연주 영상으로 바꾸는 브라우저 도구입니다. Rust/WASM이 MIDI와 연주 동작을 계산하고 Three.js가 확장 가능한 기본 리그를 렌더링합니다.

**MVP 배포 완료 · [지금 테스트하기](https://postmelee.github.io/drum-vrm/) · [v0.1.0 릴리스](https://github.com/postmelee/drum-vrm/releases/tag/v0.1.0)**

Chrome 155/macOS에서 실제 파일 업로드부터 오디오 포함 영상 다운로드까지 확인했습니다. [검증 기록](docs/verification/mvp-0.1.md)에 환경·측정값·미검증 범위를 기록합니다.

## 첫 MVP

`MIDI 업로드 → 사전 분석·연주 계획 → 기본 리그 표시 → 재생 → 영상 다운로드`

- 직접 만든 전신 휴머노이드, 양쪽 스틱, 드럼 세트와 무대
- 킥·스네어·하이햇·탐·크래시·라이드, 오른손 주도 기본 계획
- 템포 변경을 보존한 타격, 양손·양발 배정, 준비·접촉·반동과 불가능한 타격 진단
- 재생·일시정지·처음으로·탐색, 자체 합성 드럼 소리, 카메라와 배경 조절
- 1280×720, 30fps 고정 프레임 영상과 합성 오디오; 실행 환경에서 가능한 MP4 또는 WebM
- 사용자가 고른 MIDI는 브라우저 안에서 처리하며 자동 전송하거나 저장소에 업로드하지 않음

WAV 동기화, VRM·공통 리그 캐릭터 교체, 편집, 프로젝트 저장, 고해상도·3D 출력은 M3·M4 후속 범위입니다.

## 사용 흐름

1. **30초 데모로 시작**하거나 `.mid`/`.midi` 파일을 선택합니다.
2. 준비가 끝나면 트랙·채널과 분석 알림을 확인합니다. 선택을 바꾼 경우 **선택한 트랙으로 다시 준비**를 누릅니다.
3. **재생**을 눌러 연주를 시작합니다. 타임라인으로 이동하고 드래그·스크롤 또는 시점 버튼으로 카메라를 조절합니다.
4. **영상 추출**을 누르고 완료 후 다운로드 링크로 파일을 저장합니다. 기본 출력은 합성 드럼 오디오를 포함합니다.

원본 DAW의 음색을 재현하지는 않습니다. 출력에는 시작 준비 0.5초와 끝 여운 0.25초가 포함됩니다. 정확히 30초인 곡은 923프레임, 약 30.7667초로 출력됩니다.

입력은 SMF format 0/1·PPQ 시간축, 최대 10 MiB·선택 트랙 길이 300초·512트랙·전체 200,000이벤트·선택한 지원 타격 50,000개입니다. SMF 2·SMPTE·손상된 파일은 이유를 표시하고 거부합니다. 미매핑 음표는 알림으로, 리그로 표현하지 못한 지원 타격은 원래 시각의 소리와 타점 강조로 남깁니다.

## 로컬 개발과 검사

Node.js 24 이상, [rust-toolchain.toml](rust-toolchain.toml)의 Rust 1.94.1 및 `wasm32-unknown-unknown` target, wasm-pack 0.15.0을 사용합니다. npm과 Rust 의존성은 `package-lock.json`과 `Cargo.lock`으로 고정합니다.

```sh
npm ci
cargo install wasm-pack --version 0.15.0 --locked
npm run build:wasm
npm run dev
```

개발 서버가 출력하는 로컬 주소의 `/drum-vrm/` 경로를 엽니다. `src/wasm/`은 생성물이므로 커밋하지 않습니다. Rust 코드를 변경하면 `npm run build:wasm`으로 다시 생성합니다.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm run typecheck
npm test
npm run licenses
npm run build
npm run preview
```

위 명령은 재현 절차이며 검사를 통과했다는 기록은 아닙니다. `npm run licenses`는 설치된 npm 패키지, Cargo 잠금 그래프와 원본 라이선스를 대조하므로 먼저 잠긴 Rust 의존성을 빌드하거나 가져와야 합니다. production 출력은 `dist/`, Vite base는 `/drum-vrm/`입니다.

## 실행 환경과 제한

첫 검증 대상은 WebGL·WASM·Web Audio·WebCodecs를 제공하는 데스크톱 Chromium입니다. 영상 출력은 HTTPS 또는 localhost의 secure context가 필요하며, 운영체제와 브라우저가 실제 제공하는 코덱에 따라 달라집니다. MP4(H.264/AAC), WebM(VP9/Opus), WebM(VP8/Opus)를 실제 설정과 짧은 시험 출력의 길이·decoded onset까지 확인합니다. 검증 환경에서 AAC 지연이 기준을 넘으면 VP9/Opus WebM을 선택합니다. 오디오 실패를 무음 성공으로 바꾸지 않습니다.

MVP 리그는 고정된 기본 체형과 규칙 기반 동작입니다. 밀집 타격·빠른 이동·도달 불가능한 타점은 진단하며, 모든 MIDI를 사람처럼 연주한다고 보장하지 않습니다. PCM과 압축된 출력 파일을 메모리에 보관하므로 긴 곡의 메모리 사용량은 늘어납니다. 5분 입력 상한은 모든 기기에서의 성능 보장이 아니며 Safari·Firefox·모바일 전체 호환성과 최대 길이 성능은 별도 검증 대상입니다.

## 문서와 작업 현황

| 목적 | 문서 |
| --- | --- |
| 새 작업을 시작하기 | [인수인계와 현재 상태](docs/handoff.md), [작업 지침](AGENTS.md) |
| 범위와 완료 조건 | [요구사항](docs/product-requirements.md), [개발 계획](docs/plan.md) |
| 승인 근거 | [구현·배포 승인](docs/approval.md) |
| 엔진과 리그 연결 | [아키텍처](docs/architecture.md), [설계 결정](docs/decisions/0001-engine-and-rig-boundary.md) |
| 원 대화의 맥락 | [요구사항 출처와 주제별 기록](docs/context/source-conversation.md) |
| 연구와 재사용 조건 | [연구·데이터](docs/research/references.md), [기술 조사](docs/research/dependencies.md), [라이선스 정책](docs/research/license-policy.md) |
| 실행 작업 | [GitHub Issues](https://github.com/postmelee/drum-vrm/issues), [Milestones](https://github.com/postmelee/drum-vrm/milestones) |

## 배포와 라이선스

2026-10-24 [Pages](https://postmelee.github.io/drum-vrm/) 배포 및 공개 주소 핵심 경로 검증을 완료했습니다. 배포 commit은 `9ddc2bf2450600e1eaca5bd405a4221e254ce83b`, [배포 run](https://github.com/postmelee/drum-vrm/actions/runs/40060762280)은 성공했습니다. [릴리스](https://github.com/postmelee/drum-vrm/releases/tag/v0.1.0)에 실제 정적 artifact와 SHA-256을 보존합니다. 후속 배포는 main에서 `gh workflow run pages.yml --ref main`을 실행하며 전체 CI가 먼저 통과해야 합니다.

자체 소스는 [MIT](LICENSE)입니다. 제3자 라이선스는 각각 유지하며 [배포 고지](public/licenses.txt)에 정확한 버전·소스·원문을 제공합니다. Mediabunny는 MPL-2.0이며 대응 소스의 고정 버전 URL을 고지에 포함합니다. 외부 캐릭터·음원 샘플·텍스처·모션 데이터·모델 가중치는 포함하지 않습니다. 참고 연구의 라이선스를 관련 데이터나 가중치의 사용 허가로 확대하지 않습니다.
