# 아키텍처와 구현 경계

상태: **구현 승인 전 설계안** · 기준일: 2026-10-24 (Asia/Seoul)

이 문서는 원 대화 전체의 방향을 현재 MVP 범위로 구체화한다. 아래 인터페이스, 수치, 성능 기준은 **구현 제안/인수 목표**이며 실행 검증 결과가 아니다. 라이브러리의 공개 기능과 라이선스 확인 결과는 [의존성 조사](research/dependencies.md)에 분리한다. 원 대화의 코드 조각은 완성된 구현으로 취급하지 않는다.

## 1. MVP와 확장 범위

MVP는 MIDI 파일 선택 → 사전 분석 → 기본 휴머노이드 리그와 드럼 표시 → 재생·일시정지·탐색 → 합성 드럼 소리가 포함된 **1280×720, 30fps 영상** 다운로드를 완성한다. 사람형 리그는 직접 만든 구·원통·관절로 구성하며 외부 캐릭터·모션·샘플 없이 시험할 수 있어야 한다. 재생 버튼을 누르기 전에는 연주하지 않는다.

MIDI 입력의 초기 상한은 **10 MiB(10,485,760바이트), 곡 시간 5분**으로 제안한다. 주된 MVP 인수 시나리오는 30초 연주다. 파싱한 전체 이벤트 수·트랙 수·한 시점의 밀집도에도 별도 상한을 두고, 승인 후 첫 측정에서 수치를 고정한다. 파싱 도중 상한을 검사하며 초과 입력은 이유를 표시하고 취소할 수 있어야 한다. 이 제한은 5분의 모든 파일에서 목표 성능을 이미 달성했다는 뜻이 아니다.

원래 목적은 DAW에서 완성한 음원과 동기화된 커버 영상이다. 선택 WAV 입력·오프셋 보정·원음 결합은 **M3 후속 범위**로 보존한다. MVP의 합성음은 타격 확인용이며 DAW의 음색·이펙트를 재현한다고 표현하지 않는다. VRM 교체, 편집용 GLB/VRMA, 손 배정 편집, 1080p/60fps, 투명 출력, 학습 모델, 라이브 MIDI는 후속 확장이다.

## 2. 모듈 책임

| 모듈 | 책임 | 외부에 넘기는 결과 |
|---|---|---|
| `engine-core` (Rust) | SMF 파싱 결과 정규화, 템포맵, 매핑, 손발 배정, 궤적, 도달성, 시간별 운동학 | `PerformancePlan`, `PoseBuffer`, 진단 |
| `engine-wasm` (Rust + wasm-bindgen) | 바이트 입력, WASM 수명·핸들, 배열 경계, 오류 전달 | JS에서 호출할 작은 API |
| `planner-worker` (TypeScript) | 한 개의 Worker에서 분석 실행, 단계별 진행·취소 | 직렬화된 계획과 분석 보고 |
| `transport` | 곡 시간, 오디오 기준 시계, 재생·일시정지·탐색 | 절대 곡 시간 |
| `audio` | MIDI 타격 기반 합성 PCM, 프리뷰 재생; 추후 WAV 정렬 | 동일 타임라인의 PCM |
| `rig` | 기본 리그 생성, 캘리브레이션, canonical pose를 실제 노드에 적용 | Three.js 장면 노드 |
| `viewer` | 카메라·조명·키트·렌더링, 시간별 리그와 악기 상태 표시 | canvas |
| `export` | 설정 검사, 고정 FPS 평가·렌더·인코딩, 오디오 mux, 취소·정리 | MP4 또는 WebM |
| `project` | 버전·입력 해시·설정·계획의 저장/복원 경계 | 확장 가능한 프로젝트 기록 |

Rust 핵심은 DOM, Three.js, 브라우저 파일 경로에 의존하지 않는다. UI는 MIDI 해석이나 손 배정의 두 번째 구현을 만들지 않는다. 프리뷰와 영상은 같은 `evaluateAt(songTime)`를 사용한다.

기준 빌드는 `wasm32-unknown-unknown`, stable Rust, **단일 스레드**다. Worker 한 개에서 계산하는 것과 WASM 공유 메모리 멀티스레딩은 별개다. `midly`는 기본 `parallel`을 끄고 `std`/`alloc`만 필요한 범위에서 켠다. SIMD·Rayon·SharedArrayBuffer는 측정 후 별도 결정한다. midly의 기본 parallel 기능은 Rayon을 사용한다. [공식 features](https://docs.rs/midly/latest/midly/#about-features)

## 3. 곡 시간과 MIDI 정규화

### 입력 정책

- MVP는 Standard MIDI File **format 0/1, PPQN**을 지원한다. format 2의 독립 시퀀스와 SMPTE timecode는 감지 후 구체적인 미지원 메시지를 낸다. BPM으로 잘못 해석하지 않는다. `midly::Timing`은 두 시간 형식을 구별한다. [공식 API](https://docs.rs/midly/latest/midly/enum.Timing.html)
- 채널 10을 기본 드럼 후보로 제시하되 트랙/채널 선택을 허용한다. 선택하지 않은 트랙의 템포 메타 이벤트를 제거하지 않는다. GM 기본 매핑과 미매핑 음표 목록을 제공한다.
- velocity 0의 NoteOn은 타격으로 만들지 않는다. NoteOff의 길이를 팔의 접촉 시간으로 쓰지 않는다. 하이햇 타격과 페달 상태는 별도 데이터다.
- MVP 키트는 킥·스네어·닫힌/열린/페달 하이햇·탐·크래시·라이드다. 지원하지 않는 주법은 진단과 원본 참조를 남긴다. 겹친 음표를 임의로 삭제하지 않는다.
- 잘못된 파일을 조용히 일부 복구하는 대신 strict 파싱을 기본 후보로 삼는다. 복구 모드를 추가한다면 사용자에게 유실 내용을 보여주는 별도 기능으로 둔다.

### 시간 계산

원본 `absoluteTick`, 트랙, 이벤트 순서, tempo segment를 보존한다. PPQN 시간 변환은 구간마다 `deltaTicks × microsecondsPerQuarter / PPQN`을 누적한다. 구간마다 반올림하여 오차를 쌓지 말고 Rust의 넓은 정수 누산기와 유리수 표현으로 보관한 뒤 경계에서만 변환한다. 템포 이벤트 전 기본값은 SMF 관례의 500,000μs/quarter(120 BPM)로 구현하고 fixture로 검증한다.

canonical 이벤트의 `timeUs`는 부호 있는 정수 μs이며 동일 규칙으로 한 번 반올림한다. 원 tick과 유리수 원본을 함께 남겨 재계산할 수 있게 한다. JS 경계에서는 안전 정수 범위를 검사하고, 큰 정수의 원본 값은 십진 문자열로 직렬화한다. 포즈 평가·보간은 `f64` 초, 렌더용 위치·회전은 `f32` 배열을 허용한다.

같은 tick의 충돌하는 tempo 값은 출처와 함께 경고한다. 기본 선택 규칙은 format 1의 conductor track 우선, 그 안에서는 파일 순서의 마지막 값으로 명시하고 해당 선택을 프로젝트에 저장한다. 다른 트랙에만 존재하는 템포는 안정된 `(trackIndex, eventIndex)` 순서로 통합하되 충돌을 표시한다. 템포 선택은 재실행마다 달라지면 안 된다.

원본 타격 시각은 동작을 자연스럽게 보이게 하려는 이유로 이동하지 않는다. MIDI 자료는 이벤트·시간·트랙·템포를 담으며, 연주자 손 선택이나 DAW 음색은 이 정보만으로 복원되지 않는다. [MIDI Association 설명](https://midi.org/standard-midi-files)

## 4. Canonical PerformancePlan

원본은 MIDI 바이트와 프로젝트 설정이며, 계획은 버전이 있는 파생 데이터다. 외형에 종속된 본 회전만 저장하지 않는다.

| 데이터 | 필수 내용 |
|---|---|
| `PlanIdentity` | schema/engine/planner version, MIDI hash, mapping/kit/profile/calibration hash, deterministic seed |
| `SourceRef` | track, channel, eventIndex, absoluteTick, note, velocity; 병합 시 원본 참조 목록 |
| `HitIntent` | stable event ID, `timeUs`, instrument/articulation, target ID, velocity, 허용 effector, 사용자 고정 여부 |
| `AssignedHit` | 의도 ID, 선택된 left/right hand/foot, 타점·표면 법선, stick/grip, 해결 상태 |
| `StrokeSegment` | prepare/contact/rebound 시각, 곡선 제어점, 접근 방향, 이어지는 타격 ID |
| `PerformerProfile` | 주도 손, 교차손/오픈핸드, 페달 구성, 이동/관절 한계와 선호 가중치 |
| `RigCalibration` | 좌석·골반, 팔/다리 길이, rest basis, 그립, 스틱 길이, 도달 범위 |
| `Diagnostic` | 코드·심각도, 구간·원 이벤트, 원인, 적용한 근사와 남은 문제 |

좌표계는 오른손 좌표, 미터, +Y 위, 연주자 정면 +Z로 고정한다. 키트 타점은 kit-local 좌표와 공통 world transform으로 표현한다. quaternion은 `(x,y,z,w)`, 관절 회전은 local-space임을 스키마에 적는다. 카메라 회전은 키트·연주 좌표를 바꾸지 않는다.

외형을 교체하면 `RigCalibration`과 `RealizedMotion`을 다시 만든다. `HitIntent`와 사용자 고정 배정은 유지하지만, 새 체형에서 배정이 불가능하면 재계획 또는 충돌 진단이 필요하다. 오래된 calibration hash의 포즈 캐시는 무효화한다.

첫 MVP는 JSON으로 계획을 디버깅하고 테스트한다. 프레임 호출에서는 계획 전체 JSON을 왕복하지 않는다. WASM 인스턴스가 계획을 보유하고 `evaluateAt()` 한 번으로 뼈와 스틱 변환 배열을 반환한다. 재사용 버퍼의 수명과 WASM memory growth 후 view 갱신 규칙을 API 계약에 포함한다.

## 5. 손발 배정과 타격 궤적

기본 선호는 오른발 킥·왼발 하이햇 페달, 주도 손 하이햇/라이드, 반대 손 스네어다. 이것은 손 후보를 금지하는 규칙이 아니다. 동시 타격 그룹과 앞뒤 이벤트를 고려하는 제한된 lookahead/beam search로 후보를 평가한다. 비용은 거리, 남은 시간, 도달 범위, 교대 패턴, 팔 교차, 다음 구간 연결을 구분하며 선택 이유를 진단 가능하게 한다. beam 폭과 lookahead는 설정에 기록한다.

타격 접촉 시각은 고정하고 준비·접근·반동 구간을 조정한다. velocity는 높이·속도·반동·음량에 반영하되 타점은 유지한다. 타격점과 스틱 방향에서 그립 위치를 역산한 후 손목 목표를 정한다. 두 관절 팔/다리의 해석적 IK를 Rust의 기본 후보로 둔다. Three.js CCDIKSolver를 비교 실험에 사용할 수 있지만 `SkinnedMesh`용이며 이전 프레임을 초기값으로 삼으면 탐색 순서 의존성이 생기므로, 사용할 경우 매 평가마다 같은 기준 포즈로 초기화한다. [공식 CCDIKSolver](https://threejs.org/docs/pages/CCDIKSolver.html)

**첫 타격**이 곡 0초여도 준비를 음수 시간에 계획한다. 기본 pre-roll `P=0.5초`와 tail `T=0.25초`를 프로젝트·출력 설정에 기록한다. 프리뷰와 영상 모두 곡 `-P`부터 시작하고, MIDI 0초의 첫 타격은 출력 `P`초에 접촉한다. 초기 준비는 `[-P, 0]` 안에 계산하고 그 시간으로 도달이 불가능하면 진단을 남긴다. 합성 오디오와 추후 WAV 모두 같은 `P`를 적용하며 MIDI 타격 시각 자체는 변경하지 않는다. 원음과 편집할 때 사용할 pre-roll 길이를 출력 요약에도 표시한다.

### 불가능하거나 모호한 입력

같은 손의 동시 두 타격, 도달 불가능한 타점, 이동 시간이 부족한 구간을 `unresolved`로 남긴다. 동일 타점의 레이어 병합은 매핑 프로필에서 명시적으로 같은 물리 타격이라고 지정한 경우에만 하며 원 MIDI 소리는 보존한다. 자동 배정이 막히면 안정된 우선순위로 가능한 타격만 실제 리그에 배정하고 나머지는 타점 강조로 표시한다. 화면과 분석 보고에 “리그로 표현하지 못한 타격 수”를 보여 주어 완전한 신체 연주처럼 숨기지 않는다. 원본 이벤트를 보존하므로 후속 손 편집·키트 이동·추가 연주자로 다시 해결할 수 있다.

## 6. RigAdapter 계약

`RigAdapter`는 MIDI나 템포를 읽지 않는다. 기본 리그와 추후 VRM이 같은 canonical 관절 의미와 캘리브레이션 경계를 따르게 한다.

| 동작 | 입력/출력과 계약 |
|---|---|
| `describeRig()` | canonical bone 이름, parent, rest transform, 길이, 그립 anchor, 선택 기능 반환 |
| `calibrate(kit, performer)` | `RigCalibration` 반환; 필수 팔·다리/그립 누락은 명시적 오류 |
| `applyPose(poseBuffer)` | pelvis/spine/head/upper/lower arm/hand/leg/foot와 stick의 변환을 실제 노드에 적용 |
| `setVisualOptions(options)` | 재질·표시·색 변경; 연주 계획과 시각은 변경하지 않음 |
| `dispose()` | geometry/material/texture/object URL 등 소유 자원 해제 |

MVP의 `PrimitiveRigAdapter`는 본 계층에 원통과 구를 붙여 상체·팔·다리·발·양쪽 스틱이 관절을 따라 움직이게 한다. 모델 교체용 seam을 만들되 VRM 로더 자체는 MVP 필수 의존성에서 제외한다. 미래 `VrmRigAdapter`는 normalized humanoid 본과 모델의 rest basis 차이를 변환하고 좌석·그립을 재보정한다. three-vrm에는 normalized/raw 본·포즈 API가 있으나 그것만으로 드럼 접촉까지 보장되지는 않는다. [공식 VRMHumanoid API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html)

## 7. 프리뷰 오디오와 동기화

`AudioSource`는 MVP의 `SynthDrums`와 M3의 `DecodedWav`를 교체할 수 있게 한다. 합성 드럼은 자체 파형·감쇠·고정 seed 노이즈로 만들고, 모든 타격을 공통 이벤트 시간에서 PCM sample index로 변환한다. 프리뷰와 내보내기에 같은 PCM 또는 같은 합성 규칙을 사용한다. 합성음은 unresolved 시각 타격도 원 MIDI 시각에 재생한다.

프리뷰의 시계는 Web Audio를 기준으로 둔다. 재생 시작 시 `(songOrigin, contextOrigin)`을 기록하고 `songTime = songOrigin + (audioContextTime - contextOrigin)`으로 계산한다. `requestAnimationFrame` 경과 시간을 누적하지 않는다. 출력 지연을 다룰 때는 지원되는 `getOutputTimestamp()`의 context/performance 쌍으로 실제 출력 시각을 추정하며 임의의 고정 지연을 두 번 빼지 않는다. [Web Audio 시간 API](https://www.w3.org/TR/webaudio/#dom-audiocontext-getoutputtimestamp)

일시정지 시 곡 시간을 저장하고 source를 정지한다. 재개·seek는 새 source를 해당 offset에서 예약하고 이전 예약을 취소한다. pre-roll이나 WAV 시작 전처럼 계산된 `audioFileTime`이 음수이면 음수 값을 `AudioBufferSource.start()`의 offset으로 전달하지 않는다. offset 0으로 미래 시작 시각을 예약하고 그전에는 무음을 유지한다. AudioContext suspend/탭 비활성 상태를 감지해 오디오와 transport 상태가 다르게 흘러가지 않게 한다. 프리뷰 무음 모드에서는 monotonic clock과 같은 원점 모델을 사용한다.

M3 WAV의 부호 규칙은 **`audioFileTime = songTime - audioStartInSong`**이다. 예를 들어 `audioStartInSong=+0.2s`는 WAV sample 0을 곡 0.2초, 기본 pre-roll 포함 출력 0.7초에서 시작한다는 뜻이다. 앞 무음·음수 offset crop·끝 padding을 프리뷰와 출력에 동일 적용한다. 오프셋으로 원 MIDI를 수정하지 않는다.

## 8. 고정 FPS 영상 내보내기

출력은 실시간 캔버스 녹화가 아닌 오프라인 평가다. 30fps에서 프레임 `n`의 곡 시간은 `n/30 - P`, 파일 timestamp는 `n/30`이다. `P=0.5초`, 곡 길이 `D`, tail `T=0.25초`일 때 총 프레임 수는 **`ceil((P + D + T) × fps)`**다. duration은 프레임 인덱스에서 계산하여 덧셈 오차를 쌓지 않고 마지막 프레임에 맞춰 오디오를 trim/pad한다. 합성 오디오의 타격 `t`는 출력 `P+t`에 배치한다. 후속 WAV도 `P+audioStartInSong`을 원점으로 사용한다.

순서는 평가 → RigAdapter 적용 → 렌더 → timestamp/duration으로 `CanvasSource.add()` → Promise 완료 대기다. 메인 스레드 UI에 진행률·취소 기회를 주며 preview render와 export render의 동시 상태 변경을 막는다. 원시 RGBA 모든 프레임을 메모리에 쌓지 않는다. Mediabunny가 canvas와 timestamp/duration을 받는 API를 제공하는 점은 확인되었고, 이 프로젝트의 실제 WebGL 캡처·속도·품질은 구현 후 검증한다. [공식 CanvasSource](https://mediabunny.dev/guide/media-sources#canvassource)

| 실행 시 검사 | 동작 |
|---|---|
| AVC/H.264 + AAC 및 MP4가 실제 720p30/오디오 설정에서 가능 | 기본 MP4, 합성 오디오 포함 |
| 위 조합이 불가하고 VP8/VP9 + Opus 및 WebM 가능 | WebM 대체 옵션과 실제 형식 표시 |
| 영상 codec은 가능하지만 요청한 오디오 codec 불가 | 가능한 A/V 조합부터 제시; 사용자 선택 없는 무음 전환 금지 |
| 사용자가 명시적으로 무음 출력을 선택했고 영상 codec 가능 | video-only 출력, 무음 선택과 실제 형식을 요약에 표시; 합성 A/V 인수 통과와 구분 |
| 사용할 A/V 조합이 없음 | 프리뷰는 유지, 내보내기 미지원 이유·브라우저 전환 안내 |
| 인코딩 중 실패/취소/메모리 부족 | source·encoder·URL 정리, 완료 파일로 표시하지 않음 |

Mediabunny의 `canEncodeVideo`/`canEncodeAudio` 또는 WebCodecs `isConfigSupported()`를 실제 해상도·FPS·채널·sample rate로 호출하고 짧은 시험 인코딩까지 수행한다. WebCodecs 표준은 특정 codec 지원을 보장하지 않는다. API 존재만으로 MP4 성공을 약속하지 않는다. [Mediabunny 지원 조회](https://mediabunny.dev/guide/supported-formats-and-codecs#querying-codec-encodability), [W3C WebCodecs](https://www.w3.org/TR/webcodecs/)

실시간 MediaRecorder를 추가하더라도 “실시간 녹화”로 따로 표시하고 고정 FPS 인수 기준의 통과로 세지 않는다. PNG sequence·추가 WASM codec·ffmpeg.wasm은 후속 범위다. 머리카락·의상 물리는 MVP에 없으며 추후 도입 시 고정 step/체크포인트/베이크 정책을 별도로 결정한다.

결정성은 동일 엔진·설정·seed의 **주요 연주 계획과 포즈/프레임 시각**에 대한 계약이다. GPU 픽셀, 부동소수점의 플랫폼별 마지막 비트, 하드웨어 인코더가 만든 파일 바이트까지 동일하다고 주장하지 않는다.

## 9. GitHub Pages·CI 배포

예상 URL은 `https://postmelee.github.io/drum-vrm/`이다. Pages 설정과 배포가 아직 검증되지 않았으며 **사용자 승인 후** 구현·배포한다. Pages는 HTML/CSS/JS 정적 호스팅이고 프로젝트 사이트는 저장소 하위 경로를 사용한다. 서버 변환 API 없이 업로드 바이트·분석·오디오·영상 생성을 브라우저 안에서 처리한다. [공식 Pages 설명](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages)

- TypeScript + Vite의 프로젝트 `base`를 `/drum-vrm/`로 설정한다. WASM·Worker·아이콘·라이선스 자산도 이 경로 아래에서 로드되는지 production build로 확인한다. router가 필요하면 hash 경로 또는 단일 페이지 상태를 사용한다. [Vite Pages 가이드](https://vite.dev/guide/static-deploy.html#github-pages)
- 개발 서버에서만 성공한 결과를 배포 성공으로 간주하지 않는다. `/drum-vrm/` 로컬 preview와 실제 HTTPS URL에서 MIME·WASM 초기화·콘솔 오류·샘플 MIDI·한 번의 영상 다운로드를 검사한다.
- 기본 기능은 `crossOriginIsolated=false`에서도 동작해야 한다. COOP/COEP 응답 헤더를 전제로 하는 공유 메모리·threaded WASM을 Pages 기준 경로에 넣지 않는다. 정적 호스팅에서 필요한 헤더의 제공 가능성은 별도 검증 항목이지 현재 확인된 기능이 아니다. [격리 조건](https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated)
- Rust toolchain, wasm-bindgen crate/CLI, npm 패키지와 Actions 버전을 고정한다. `Cargo.lock`·npm lockfile을 커밋하고 CI에서 locked install/build를 사용한다. 라이선스 명세와 배포물이 같은 버전을 가리키게 한다.
- CI는 Rust 단위 테스트·WASM 빌드, TypeScript typecheck, production build, 브라우저 fixture smoke test를 수행한다. 배포 job은 검증 job 성공 뒤에만 실행하고 `github-pages` environment와 최소 `pages: write`·`id-token: write` 권한을 사용한다. Actions SHA와 업로드할 `dist`를 검토한다. [공식 workflow 조건](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
- 첫 승인 전에는 배포 workflow 실행이나 Pages 활성화를 하지 않는다. 승인 후 초기 배포는 명시적인 workflow dispatch로 시작하고 성공 URL·commit·브라우저·검증 결과를 기록한다.

Pages에는 사이트 크기·배포 시간·대역폭 제한이 있다. 현재 공식 문서는 게시 사이트 1GB, 배포 10분 제한, 월 100GB soft bandwidth limit을 설명한다. 대형 원음·모델·데이터셋을 저장소나 Pages 번들에 넣지 않는다. 유료 SaaS로 발전하면 Pages의 용도 제한에 맞는 별도 호스팅을 결정한다. [공식 사용 제한](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)

## 10. 검증 목표와 증거

| 항목 | 승인 후 인수 목표 | 남길 증거 |
|---|---|---|
| 시간 해석 | 기본/변경 템포, 다른 트랙의 템포, 동시 타격, velocity 0, 빈/손상/미지원 파일을 명시적으로 처리 | 작은 직접 제작 MIDI fixture와 기대 tick/time |
| 접촉 | 타격 시각 평가에서 스틱 끝과 목표점의 거리 ≤ 2cm (기본 리그·도달 가능 타격); 5mm는 선택적 개선 목표 | 실제 타격별 오차 보고와 실패 이벤트 |
| 첫 타격 | tick 0 타격에서도 준비가 계산되고 접촉 시각이 밀리지 않음 | pre-roll/0초/seek 비교 |
| 시간 일치 | 30fps 출력의 시각 접촉과 오디오 onset 차이 ≤ 1 frame | 출력 프레임/오디오를 다시 검사한 결과 |
| 결정성 | 순차 평가·역방향 seek·랜덤 순서 평가에서 같은 시각의 주요 포즈가 허용 오차 내 일치 | 동일 plan hash와 수치 비교 |
| 완주 | 기본 30초와 경계값 5분 fixture의 시작·중간·끝, pause/resume/seek 정상 | 기기·OS·브라우저·버전·시간·메모리 기록 |
| 내보내기 | 최소 지원 브라우저에서 720p30, 합성 오디오, 예상 프레임 수/길이·형식, 실제 재생 가능 | 파일 metadata + 재생 확인 |
| 오류/취소 | 분석·출력 중 취소 후 새 파일 재시도, 초과 입력/codec 미지원 명시 | 재현 절차와 결과 |
| 배포 | Pages 하위 경로에서 새로고침 후 WASM·Worker 로드와 end-to-end 다운로드 | 실제 배포 URL·commit·검증 일시 |

초기 지원 목표는 데스크톱 Chromium 계열이며 Safari·Firefox는 업로드/프리뷰/codec probe 결과를 각각 기록한다. **현재 어떤 브라우저에서도 프로젝트 동작을 검증한 상태가 아니다.** CI의 headless encoder 결과와 실제 사용자의 OS/codec 환경은 구분한다. 수치 목표를 바꾸면 관련 이슈·인수 기준·이 문서를 함께 갱신한다.
