> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/mvp-0.1.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../../../specs/README.md) for current contracts and policy.

# MVP 0.1 검증 기록

검증일: **2026-10-24 (Asia/Seoul)**. 이 기록은 실행한 자동 검사, 로컬 브라우저 동작, 내려받은 영상 파일을 구분한다. **Pages 배포와 공개 주소의 업로드→재생→영상 다운로드 검증을 완료했다.** 아래에 확인 범위와 미검증 항목을 구분한다.

## 환경과 기준

| 항목 | 확인값 |
|---|---|
| 운영체제 | macOS 26.5.2, build 25F84 |
| 브라우저 | Google Chrome 155.0.8059.86 |
| 도구 | Rust 1.94.1, wasm-pack 0.15.0, Node.js 24 계열; 버전 잠금은 저장소 파일 참조 |
| 입력 | 직접 생성한 `Studio Groove.mid` 데모 및 `scripts/fixtures.mjs` fixture |
| 기본 시간축 | pre-roll 0.5초, 곡 30초, tail 0.25초, 30fps |
| 기대 출력 | 1280×720, 923프레임, `923/30 = 30.766666…`초, 합성 오디오 |
| 승인 | [2026-10-24 사용자 구현·MIT·검증 후 Pages 배포 승인](../history/approvals.md) |
| 구현 commit / PR | [PR #24](https://github.com/postmelee/drum-vrm/pull/24) 병합; PR 및 배포 CI 성공 |
| 최종 배포 commit SHA | `9ddc2bf2450600e1eaca5bd405a4221e254ce83b` |
| 공개 테스트 URL | [postmelee.github.io/drum-vrm](https://postmelee.github.io/drum-vrm/) |

다른 운영체제·브라우저·코덱의 결과로 일반화하지 않는다. Safari·Firefox·모바일 전체 호환성과 모든 5분 입력의 실기기 성능은 검증하지 않았다.

## 실행한 자동 검사

| 검사 | 결과 | 다루는 범위 |
|---|---|---|
| Rust 단위 검사 | **16개 통과** | SMF/템포/트랙·채널, 원본 시각, 미매핑·충돌, 접촉, 임의 시각 결정성, 입력 상한·잘못된 계획 |
| `cargo fmt` / `cargo clippy` | **통과** | 포맷 및 경고를 오류로 처리한 Rust 정적 검사 |
| Vitest | **4개 파일, 14개 통과** | 실제 WASM 데모 경계, 합성 onset/결정성/transport, 923프레임, 리그 포즈·접촉·kit 계약, codec probe 취소 |
| TypeScript typecheck | **통과** | 앱·Worker·미디어·리그 타입 경계 |
| WASM 및 production build | **통과** | 고정 의존성의 브라우저 산출물 생성; 공개 호스팅 성공과 별개 |

Rust 경계 검사는 정확히 300초인 입력 허용, 300초 초과 거부, 10MiB 초과 입력과 이벤트 상한을 포함한다. 이는 **정확히 10MiB인 유효 파일의 전체 UI 처리나 5분 전체 브라우저 인코딩을 완료했다는 뜻이 아니다**. 전체 검사 이름과 fixture 생성 코드는 저장소에 남긴다.

재실행 명령:

```sh
npm ci
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run build:wasm -- -- --locked
npm test
npm run typecheck
npm run licenses
npm run build:web
```

## 실제 브라우저 확인

| 순서/입력 | 관찰 결과 |
|---|---|
| 30초 데모 시작 | **193타격·30초** 준비 완료. 사용자가 재생하기 전 준비 상태 유지 |
| 재생·일시정지·탐색 | 동작 확인. 타임라인 약7초 탐색 후 해당 시각 자세와 재생 상태 확인 |
| 카메라 | 탑뷰와 무대 시점 전환 확인 |
| `invalid.mid` 파일 업로드 | 잘린 MIDI 트랙 오류를 표시하고 앱 유지 |
| 이어서 `two-tempos-channel2-30s.mid` 업로드 | **135타격·템포2개·30초** 준비 완료. 비채널10 입력과 다른 트랙 conductor tempo 처리 확인 |
| 영상 추출 직후 즉시 취소 | codec probe 단계 취소 메시지 표시, 재생 버튼 다시 활성화 확인 |
| 30초 데모 영상 추출 | 아래 WebM 파일을 다운로드하고 metadata·전체 decode·첫 음향 onset 검사 |

분석/합성 중 취소와 본 인코딩 중 취소 후 재출력, 공개 사이트에서 같은 흐름의 반복은 아래 후속 확인 항목에 둔다. 수행하지 않은 사례를 위 결과에 포함하지 않는다.

## 출력 파일과 동기화 판정

처음 생성한 MP4는 video track 자체의 해상도·프레임 수가 맞았지만 오디오와 컨테이너 시간 기준을 넘어서 **인수 결과에서 제외했다**. 해당 환경의 AAC 지연을 전역 offset으로 임의 보정하지 않고 codec 선택 기준을 강화했다.

| 측정값 | 초기 MP4 — 제외 | 최종 확인 WebM |
|---|---|---|
| 비디오 | H.264, 1280×720, 30fps | **VP9, 1280×720, 30fps** |
| 실제 프레임 수 | 923 | **923** |
| 오디오 | AAC | **Opus, 48,000Hz, 2채널** |
| 컨테이너 길이 | 30.848초 | **30.780000초** |
| 기대 길이 대비 차이 | +81.333ms, 1프레임 초과 | **+13.333ms, 1프레임 이내** |
| 첫 음향 시작 | 0.542958초 | **0.506146초** |
| 기대 0.5초 대비 차이 | +42.958ms, 1프레임 초과 | **+6.146ms, 1프레임 이내** |
| 파일 크기 | 인수 대상에서 제외 | **8,451,529바이트** |
| 전체 파일 decode | 인수 대상에서 제외 | **ffmpeg 전체 decode 오류 없음** |
| 이미지 확인 | — | **접촉 프레임을 직접 확인, 빈 화면이 아님** |

첫 음향 측정은 `silencedetect=noise=-40dB:d=0.1`을 첫1초에 적용했다. WebM은 `silence_start=0.007`, `silence_end=0.506146`, MP4는 시작0부터 `silence_end=0.542958`이었다. 임계값 기반 첫 onset 측정이며 모든 타격의 지각적 일치나 모든 플랫폼 encoder의 정확성을 일괄 보장하지 않는다. 포즈/합성 원본 시간의 일치는 별도 자동 검사가 다룬다.

현재 런타임 codec gate는 후보마다 **6프레임·0.2초의 실제 A/V 시험 파일**을 만들고, 0.1초에 시작하는 알려진 합성 tone을 다시 demux/decode한다. audio track 길이와 decoded onset 모두 기대값 대비1프레임 이내인 조합만 제공한다. 실제 완성 출력도 audio track 길이를 재검사한다. 이 환경에서 시간 기준을 넘은 AAC 조합은 제외하고 통과한 Opus/WebM을 사용한다. MP4를 모든 환경에서 금지하거나 모든 AAC에 같은 지연을 적용하지 않는다.

## 직접 재현하기

자체 fixture는 사용자 원본 파일을 필요로 하지 않는다. 결과 파일은 Git에서 제외된 `.local/fixtures/`에 생성된다.

```sh
node scripts/fixtures.mjs
npm run dev
```

개발 서버의 `/drum-vrm/`에 접속하여 다음을 확인한다.

1. **30초 데모로 시작** →193타격·30초·준비 완료 →재생·일시정지 →약7초 탐색 →탑뷰/무대 전환.
2. **MIDI 불러오기**에서 `.local/fixtures/invalid.mid` 선택 →오류 확인 →같은 페이지에서 `two-tempos-channel2-30s.mid` 선택.
3. 트랙은 `Channel 2 - Two Tempos`, 채널은 전체 또는 UI의 **2**를 사용한다. 135타격·템포2개·30초를 확인한다. conductor 트랙을 연주 대상으로 선택하지 않아도 tempo가 반영된다.
4. 데모를 다시 준비하고 **영상 추출**을 누른다. 처음에는 즉시 취소해 취소 메시지와 재생 복구를 확인한다. 다시 추출하여 완성 파일을 다운로드한다.
5. 일반 플레이어에서 파일 전체를 재생하고 영상/소리·시작/중간/끝을 확인한다. 아래 명령으로 프레임·오디오·파일 길이와 첫 onset을 확인한다.

내려받은 파일 경로를 자신의 환경에 맞게 지정한다. `ffprobe`/`ffmpeg`는 검사 도구이며 앱의 런타임 의존성은 아니다.

```sh
VIDEO_PATH='./demo.webm'
ffprobe -v error -count_frames -select_streams v:0 \
  -show_entries stream=codec_name,width,height,r_frame_rate,nb_read_frames \
  -of json "$VIDEO_PATH"
ffprobe -v error -select_streams a:0 \
  -show_entries stream=codec_name,sample_rate,channels:format=duration,size \
  -of json "$VIDEO_PATH"
ffmpeg -v error -i "$VIDEO_PATH" -f null -
ffmpeg -hide_banner -i "$VIDEO_PATH" -t 1 \
  -af silencedetect=noise=-40dB:d=0.1 -f null -
```

추가 fixture의 기대값은 `.local/fixtures/expected.json`에 있다. `simultaneous.mid`는5개 지원 타격·미매핑99번1개·표현 불가1개를 포함한다. `duration-300s.mid`는300초 허용, `duration-over-300s.mid`는 초과 거부, `dense-cancel-300s.mid`는 합성 취소/재시도용이다. 후자는 성능 목표가 아닌 고밀도 취소 사례다.

## 수용 기준별 증거와 남은 확인

| PRD | 현재 증거 | 추가 확인/범위 |
|---|---|---|
| A01–A03 | 데모/실제 업로드 준비, 193/135타격, 템포·트랙·채널 자동 검사 | 공개 Pages에서 채널2·두 템포·30초135타격 반복 확인 |
| A04–A05 | 손상 파일 UI 복구, native 미지원/velocity0/미매핑·상한 검사 | 정확한10MiB 유효 파일과 모든 UI 경계 사례는 별도 |
| A06–A07 | 기본 연주 시각 확인, native 양손/양발 타점 접촉, adapter 스틱 끝 계약 | 일반 플레이어 전체 재생의 최종 관찰 기록 |
| A08 | 정규화/합성/프레임 시간 자동 검사, WebM 첫 decoded onset +6.146ms | 모든 타격의 지각적 동기화 측정은 아님 |
| A09–A11 | 불가능 동시 타격 보존·진단, 시각별 결정성·역방향 평가, pause/seek UI | 사용자 DAW 패턴의 품질 피드백은 후속 |
| A12 | 실제923프레임·720p30 A/V WebM, 길이/전체 decode/비어 있지 않은 접촉 프레임 | 일반 플레이어 전체 육안·청취 최종 기록 |
| A13 | codec probe 취소 UI 복구 및 취소 자동 검사 | 5000타격 합성 취소→데모 재준비, 프레임 인코딩 중 취소, 재출력 및 명시적 무음 MP4 확인 |
| A14 | 로컬 production build | 공개 Pages 새로고침·업로드·재생·출력 통과; 아래 배포 기록 참조 |
| A15 | 잠금 의존성 고지, 로컬 파일 처리 코드 경계 | 공개 라이선스 고지 SHA 일치; 파일 처리의 외부 전송 없음은 소스 경계로 확인 |
| A16 | 변경 kit 데이터/복사,57-float 검증, 스틱 접촉과 임의 순서 pose 계약 | 임의 VRM/체형 자동 보정은 후속 |

## 제한과 후속 기록

- 최종 mux/encoder flush가 시작된 단계에서는 취소 요청 후 브라우저 작업의 정리를 기다릴 수 있다. 즉시 프로세스를 강제 종료한다고 약속하지 않으며 완료 다운로드로 표시하지 않는다.
- 5분48kHz stereo Float32 PCM 한 벌은 계산상 약110MiB다. 합성/AudioBuffer 복사와 압축 출력이 추가된다. 최대5분의 실제 UI 인코딩·peak memory·완료 시간을 아직 전체 측정하지 않았다.
- Safari·Firefox·모바일, 모든 운영체제의 MP4/AAC·WebM codec 조합은 미검증이다. runtime gate가 실패하면 지원 제한을 표시한다.
- 일반 플레이어의 추가 관찰이나 새 환경 검증 시 아래 완료 기록에 결과를 추가한다. CI/Pages·취소·공개 경로 결과는 아래에 기록했다.

## 공개 배포 완료 · 2026-10-24

- URL: [Drum VRM](https://postmelee.github.io/drum-vrm/)
- 배포 commit: `9ddc2bf2450600e1eaca5bd405a4221e254ce83b` (PR #24)
- [PR CI](https://github.com/postmelee/drum-vrm/actions/runs/40060627630)·[배포 build 및 deploy](https://github.com/postmelee/drum-vrm/actions/runs/40060762280): 성공
- 실제 공개 주소를 새로고침한 뒤 `two-tempos-channel2-30s.mid` 업로드 → 135타격/2템포/30초 준비 → 재생·일시정지·15.5초 탐색 → 오디오 포함 영상 추출·다운로드 완료. console error/warn 없음.
- 공개 사이트 다운로드: VP9/Opus,1280×720,30fps,923프레임,48kHz stereo,30.780000초,8,304,632 bytes. `ffmpeg -v error -i FILE -f null -` 전체 decode 무오류. 같은 silence threshold의 첫 onset0.506542초(목표0.5초 대비+6.542ms).
- 로컬 추가 UI: simultaneous fixture의 지원5타격/미매핑99번1개/미배정1개 진단, 명시적 무음 H264 MP453프레임·1.766667초 및 audio track 없음,5000타격/300초 합성 취소 후 데모 복구, codec probe 및 본 인코딩 중 취소·재출력 통과.
- 배포된 `licenses.txt`는 로컬 생성본과 SHA-256 일치: `f706ef0e0791dd8cfcac12b4ee2e67b6324e6e7eb5b7d5712eb3572eb7d8a12c`.
- [v0.1.0 릴리스](https://github.com/postmelee/drum-vrm/releases/tag/v0.1.0)에 실제 배포 artifact를 보존. `drum-vrm-v0.1.0-pages.tar.gz` SHA-256: `629f083369e4b6d0348bced4fb246d94162a86f821aa34caba8db90710667cf0`.
- 첫 배포이므로 이전 artifact는 없다. 후속 회귀의 기준은 이 릴리스이며 main을 이 앱 소스로 복구하고 Pages workflow를 실행하거나 보존 artifact를 재배포한다.

검증의 한계: 브라우저 프리뷰와 추출 프레임의 시각 확인 및 전체 영상/오디오 decode를 수행했다. 외부 일반 플레이어의 전체 육안·청취 비교는 완료했다고 주장하지 않는다(브라우저 도구가 로컬 file URL 재생을 차단함). Safari/Firefox/모바일,5분 전체 영상 인코딩 및 모든 타격의 지각적 싱크 검증은 후속이다.
