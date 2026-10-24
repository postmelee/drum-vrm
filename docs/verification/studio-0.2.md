# v0.2 Studio UI 검증

일자: 2026-10-24 · 이슈 #25. 환경: macOS26.5.2 / Chrome155.0.8059.86, Node24, Rust1.94.1, wasm-pack0.15.0.

## 자동 검사

- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`: 성공, Rust18개.
- `npm run typecheck`, `npm test`: 성공, 웹27개/6files. 기존 출력·오디오·WASM취소 계약을 유지하고 tempo/meter grid6개, scene10개, 실제CC0모델1개를 포함한다.
- `npm run build:wasm`, `npm run build:web`: 성공. production 기본 JS 약1.12MB(압축289KB), CC0 GLB3.36MB는 프리셋 선택 시 지연 로드한다. Vite의500KBchunk경고는 남는다.
- `npm run licenses`: 성공. npm4개/Rust21개 원문 및 CC0 모델·원문SHA-256 검증. 외부asset고지 포함.
- `git diff --check`: 성공.

## 브라우저 경로

Production preview `/drum-vrm/`에서 확인했다.

- 기본 UI의 흰 배경, 시스템 글꼴, 숨겨진 inspector, 상단 transport, 네 메뉴, 하단 피아노 롤.
- 30초 데모193타격 준비, 트랙/채널/진단 표시. 새로운 MIDI의 bar1은pre-roll 뒤에서 시작.
- 드럼 메뉴의 정면 카메라 전환/스켈레톤 숨김, 다른 메뉴/닫기 시 복원.
- Innerscene프리셋 실제로딩, mesh클릭으로High tom선택, 개별compact형태 변경.
- 검정테마 전환, 캐릭터스켈레톤 숨김/표시, 재생·일시정지. 테마는 새로고침 후 유지.
- 직접 생성한 정적GLB부품(qa-static-piece.glb)을Snare에 가져오기, 안내상태확인, 프리셋복원.
- `two-tempos-channel2-30s.mid` 실제filechooser업로드:135타격,2템포,30초. 선택한CC0모델 유지.
- 확대1/4/16배의 피아노롤 및 위치slider. ArrowRight로0.1초, End로30.7667초 이동을 브라우저에서 확인했다. 피아노롤은 편집 기능을 표방하지 않는다.
- CC0모델+스켈레톤+검정배경으로오디오포함720p30출력·다운로드.
- 해당브라우저경로 console error/warn없음.

## 실제 출력 파일

30초2템포fixture의 결과:

| 항목 | 확인값 |
|---|---|
| 비디오 | VP9,1280×720,30fps,923frames |
| 오디오 | Opus,48kHz,stereo |
| 컨테이너 길이 | 30.780000초 |
| 파일 크기 | 5,884,758bytes |
| 첫 오디오onset | 0.506542초,목표0.5초 대비+6.542ms |

`ffprobe -count_frames`, 전체`ffmpeg -v error -i FILE -f null -`decode무오류, `silencedetect=noise=-40dB:d=0.02`측정. 1초프레임을추출해원형킥과스켈레톤·실제드럼외형을시각확인했다. 임의의사용자MIDI/모델은커밋하지않았다.

## 제한

- Logic Pro의원본UI코드·모델을복제한제품이아니다. 사용자스크린샷의작업구조를참고한자체구현이다.
- 캐릭터VRM/GLB불러오기, WAV입력, MIDI편집, 프로젝트영속저장은아직없다.
- 드럼GLB는정적메시·포함텍스처만지원한다. 외부URI와스킨모델은거부한다. 임의모델bbox맞춤은정밀접촉을보장하지않는다.
- CC0키트는측정한표면과canonical타점을맞추며원본비율을보존한다. 스탠드발높이·기계적연결은모델별추가보정여지가있고, 심벌실물변형애니메이션은없다.
- 최대5분실제인코딩/peakmemory, Safari·Firefox·모바일실기기, 외부플레이어전체청취비교는미검증이다.

## 배포

PR CI와공개Pages검증결과는실행완료후아래에기록한다.

## 확대 회귀 수정

최종 QA에서 피아노롤의 긴 spacer가 CSS Grid의 암시적 최소 너비를 늘리고 ResizeObserver를 반복 실행하는 문제를 발견했다. 명시적 `minmax(0,1fr)` 열과 `min-width:0`, component `contain:inline-size`로 외부 레이아웃과 스크롤 콘텐츠의 크기를 분리했다. 수정 후1/4/16배 모두 앱1512px·캔버스bitmap3024px를 유지하고, scrollWidth만1512/5634/22122px로 변했다. 키보드 탐색과3D렌더·상단메뉴가 정상 유지됨을 확인했다.
