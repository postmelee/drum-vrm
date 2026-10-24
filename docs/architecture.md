# 아키텍처와 구현 경계

상태: **사용자 승인 후 MVP 구현, 통합·배포 검증 진행 중** · 기준일: 2026-10-24 (Asia/Seoul)

현재 구현은 아래 파일을 기준으로 설명한다. 수용 기준의 통과 여부는 [PRD](product-requirements.md)와 실제 검사 기록으로 판단하며, 구현된 경로와 검증이 끝난 경로를 구분한다. 후속 설계는 별도로 표시한다. 기술 조사와 원본 라이선스 근거는 [의존성 조사](research/dependencies.md)에 보존한다.

## 1. MVP 범위와 입력 상한

MVP는 MIDI 파일 선택 → 사전 분석 → 기본 휴머노이드와 드럼 표시 → 재생·일시정지·탐색 → 합성 드럼 오디오를 포함한 **1280×720, 30fps 영상** 다운로드다. 사람형 리그, 드럼 메시, 30초 데모와 합성 음향을 직접 만들었다. 재생 전에는 준비 자세를 표시한다.

| 제한 | 구현값/검사 위치 |
|---|---|
| 파일 | 10MiB(10,485,760바이트), UI 및 Rust preflight |
| 시간 | 선택 트랙의 마지막 유효 End of Track까지 300초 이하; 준비/여운 제외 |
| 트랙 | 1–512개, header와 실제 MTrk 수 일치 |
| 이벤트 | 파일 전체 200,000개 이하, event iterator 도중 검사 |
| 지원 타격 | 선택 트랙/채널에서 GM 매핑된 NoteOn 50,000개 이하 |
| 동시 후보 | 동일 μs 그룹의 처음 32개까지 beam 확장; 나머지 지원 타격은 unresolved로 보존 |
| 내부 계획 | JSON 최대32MiB, schema·시각·ID·좌표·손발 검사 후 evaluator 생성 |

이 상한은 파싱·탐색의 작업량을 제한하는 정책이며, 모든 기기에서 5분 출력 성능을 측정했다는 뜻이 아니다. 주 인수 사례는 30초다. WAV·VRM·편집·프로젝트 저장·고해상도·3D 출력은 M3·M4에 유지한다.

## 2. 실제 모듈 책임

| 파일/모듈 | 책임 | 경계 |
|---|---|---|
| `crates/drum-engine/src/midi.rs` | SMF preflight/파싱, 템포맵, 선택과 GM 매핑 | canonical 타격·진단·키트 |
| `model.rs`, `motion.rs` | 스키마·상한·기본 캘리브레이션, beam 배정, 궤적, 해석적 IK | `PerformancePlan`, 57-float pose |
| `lib.rs` | wasm-bindgen API와 JSON/포즈 경계 검사 | `inspect_midi`, `Performance` |
| `src/engine.worker.ts`, `engine.ts` | Worker에서 분석·취소, main WASM evaluator 소유 | JSON 계획, 시간별 Float32Array |
| `src/scene.ts` | PrimitiveRigAdapter, 키트·무대·조명·카메라·타점 표시 | 포즈 적용과 canvas 렌더 |
| `src/audio.ts`, `audio.worker.ts`, `synthesis.ts` | 자체 합성 PCM, Worker 합성/취소, Web Audio transport | 공통 출력 시간의 stereo PCM |
| `src/export.ts` | codec 확인·시험 인코딩, 고정 FPS 평가·인코딩·mux·취소 | MP4 또는 WebM Blob |
| `src/main.ts`, `style.css` | 파일·트랙·진단·재생·영상 저장 UI와 상태 | 사용자 입력/피드백 |
| `src/demo.ts` | 직접 작성한 30초 SMF fixture | 사용자 자료 없이 재현 가능한 데모 |

Rust 코어는 DOM·Three.js 객체 이름·브라우저 경로에 의존하지 않는다. UI가 MIDI 시간 해석이나 손발 배정을 다시 구현하지 않는다. 프로젝트 파일 저장/복원은 아직 구현하지 않았으며 #15의 후속 경계다.

Rust 1.94.1, `wasm32-unknown-unknown`, 단일 스레드 WASM을 사용한다. midly 0.5.3은 `default-features=false`, `std`/`strict`를 사용하므로 기본 Rayon 병렬 파싱을 켜지 않는다. Worker와 shared-memory WASM 멀티스레딩은 별개다. SIMD·Rayon·SharedArrayBuffer는 도입하지 않았다.

## 3. MIDI와 시간 정규화

- SMF **format 0/1, PPQN**을 지원한다. Format 2·SMPTE·0 PPQ·잘린 청크·트랙 수 불일치·End of Track 누락/이후 이벤트는 오류로 반환한다. 자동 복구로 이벤트를 숨기지 않는다.
- 채널10이 있는 트랙을 기본 후보로, 없으면 지원 드럼 음표가 있는 트랙을 제시한다. UI에서 트랙과 채널을 바꿀 수 있다. 템포는 선택하지 않은 트랙을 포함한 파일 전체에서 읽는다.
- velocity 0 NoteOn과 NoteOff는 타격으로 만들지 않는다. note length를 팔 접촉 길이로 해석하지 않는다. CC4, 정밀 주법, 레이어 병합은 후속이다.
- 미매핑 음표는 트랙·채널·음표별 개수를 `UNMAPPED_NOTE`로 알리고 합성/리그 타격에서 제외한다. 지원하지만 몸으로 표현하지 못하는 타격과 구분한다.

템포맵은 PPQN의 공통 분모를 유지한 `u128` 누산값으로 `deltaTicks × microsecondsPerQuarter`를 더한다. 마지막에 한 번 μs로 반올림한다. 첫 tempo 이전 기본값은 500,000μs/quarter다. 같은 tick의 충돌은 트랙0(conductor)을 우선하고 동일 트랙의 마지막 이벤트를 사용한다. 트랙0 값이 없으면 안정된 `(trackIndex, eventIndex)` 순서의 마지막 값을 사용하며 `TEMPO_CONFLICT`에 선택 근거를 남긴다.

MVP 타격의 `timeUs`는 음수가 아닌 `u64`, `time`은 초 단위 `f64`다. 음수 준비 시간은 타격 데이터가 아닌 evaluator 입력이다. `tick`은 JavaScript 안전 정수 범위를 검사한다. 선택 구간이 최대300초이므로 반환 타격의 μs는 안전 정수 범위다. 원본 tick·track·eventIndex·channel·note·velocity를 보존한다. 실제 DAW 음색과 연주자의 손 선택은 MIDI만으로 복원되지 않는다. [MIDI Association](https://midi.org/standard-midi-files)

## 4. PerformancePlan과 포즈 계약

`schemaVersion=1`의 현재 JSON은 다음을 보존한다.

| 데이터 | 현재 필드/의미 |
|---|---|
| 버전/식별 | `schemaVersion`, `engineVersion`, `plannerVersion`, `midiHash`, `calibrationId` |
| 시간 | `duration`, `preRoll=0.5`, `tail=0.25`, `tempoCount` |
| 타격 | `id`, `time`, `timeUs`, `tick`, `track`, `eventIndex`, `channel`, `note`, `velocity`, `instrument`, `limb`, `target` |
| 키트 | 악기별 `id`, `label`, 세계 좌표 `position`, `radius` |
| 진단 | `code`, `message`, 필요한 경우 `hitId`, `time` |

`midiHash`는 입력 식별용 FNV-1a64이며 보안 무결성 해시로 사용하지 않는다. 기본 engine calibration은 `primitive-v1`, planner 식별자는 `beam-8-lookahead-2`다. 전체 mapping/profile/calibration hash, 사용자 고정 배정, 독립 StrokeSegment 저장, project migration과 seed 편집은 아직 구현하지 않았다. 이들은 #14/#15/#18의 확장 대상이며 현재 필드가 모두 있다는 뜻으로 읽지 않는다.

프레임마다 계획 JSON을 주고받지 않는다. main WASM `Performance.from_json()`이 검증한 계획과 limb별 타격 인덱스를 보유하고, `evaluate_at(song_time)`이 소유권 있는 **Float32Array 57개 값**을 반환한다. 현재는 작은 배열을 매 호출 반환하므로 WASM memory growth 이후의 재사용 view 수명 문제를 노출하지 않는다. 추후 zero-copy 최적화 시 별도 수명 계약이 필요하다.

| Float32 offset | 의미 |
|---|---|
| 0–11 | 왼팔 root/elbow/wrist/stick tip의 XYZ |
| 12–23 | 오른팔 root/elbow/wrist/stick tip의 XYZ |
| 24–35 | 왼다리 hip/knee/ankle/toe의 XYZ |
| 36–47 | 오른다리 hip/knee/ankle/toe의 XYZ |
| 48–56 | pelvis/chest/head의 XYZ |

오른손 좌표계, 미터, +Y 위, 연주자 정면 +Z다. 현재 포즈는 세계 좌표 위치이며 본의 local quaternion 형식이 아니다. 카메라를 움직여도 이 좌표계는 변하지 않는다.

## 5. 손발 배정과 결정적 동작

기본 배정 선호는 오른발 킥, 왼발 하이햇 페달, 왼손 스네어/하이탐/크래시, 오른손 나머지 손 타격이다. 손 타격은 양손 후보를 평가한다. 동시 μs 그룹 안에서 beam 폭8을 유지하고, 다음 최대16개 이벤트 안의 손 타격2개를 비용 예측에 쓴다. 거리·회복 시간·주도 손 선호와 도달성을 반영한다. 완전한 동작 최적화나 신체 충돌 회피 모델은 아니다.

같은 limb의 재타격 최소 간격45ms, 타점 이동 속도 상한12m/s, 0.43m 두 segment의 도달 범위를 검사한다. 이는 보수적인 구현 정책이며 생체역학적으로 검증한 인간 한계가 아니다. 배정할 수 없으면 `limb=null`과 `UNRESOLVED_HIT`를 남기고 원본 시각·소리·타점 강조를 유지한다. 동시에 여러 타격을 한 손에 넣거나 시각을 밀지 않는다.

포즈는 인접 타격 사이의 smoothstep 보간과 sine arc로 절대 시각에서 평가한다. 짧은 간격은 연속 반동, 긴 간격은 준비/복귀로 처리한다. 손은 최대0.24m, 발은 최대0.045m의 준비 높이를 velocity와 간격으로 조절한다. 고정 stick vector `[0,-0.09,0.30]`에서 그립을 역산하고 2-bone IK를 적용한다. 몸통 기준점은 유지하고 머리의 작은 후속 움직임도 절대 시각으로 계산한다.

첫 타격이 0초여도 `[-0.5,0]`에서 준비한다. 기본 준비/여운은 원본 MIDI를 편집하지 않고 프리뷰·출력 시간축에만 적용한다. 입력 시각과 같은 포즈를 순서와 무관하게 얻는 것이 계약이다. GPU 픽셀이나 하드웨어 encoder 파일 바이트까지 동일하다는 보장은 아니다.

## 6. RigAdapter와 시각화

`RigAdapter`는 MIDI나 tempo를 읽지 않는다. `PrimitiveRigAdapter`는 canonical 관절 의미를 전신 원통·구·스틱에 적용한다. 외부 모델/texture를 로드하지 않는다.

| API | 현재 동작 |
|---|---|
| `describeRig()` | adapter ID `primitive-humanoid-v1`, canonical 관절19개, pose 길이57 |
| `calibrate(kit)` | 유한 좌표/양의 반지름 검사, 타점 복사, 좌표계·스틱 길이 기록 |
| `applyPose()` | 57개 유한 값 확인 후 몸통·머리·양손·양발·스틱 변환 적용 |
| `setVisualOptions()` | body 재질 색 변경; 계획/시각은 유지 |
| `dispose()` | 소유 geometry/material/texture 정리 |

현재 `calibrate()`는 다른 체형의 IK를 자동으로 다시 푸는 함수가 아니다. engine의 기본 체형은 고정되어 있고, kit 치수와 표시 adapter의 분리 계약을 제공한다. VRM normalized/raw rest basis, 체형/좌석/그립 보정과 재계획은 #17/#18에서 구현한다. [three-vrm humanoid API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html)

`DrumScene`은 `setPlan`, `applyPose`, `render`, `resize`, `setCamera`, `setBackground`, `setExportMode`를 제공한다. 무대/정면/탑뷰와 OrbitControls가 있다. 심벌 타점은 중앙 stand mount가 아닌 bow 표면에 맞추고 타격 강조·감쇠 반동을 `songTime`으로 계산한다. 출력 중에는 orbit과 preview rendering을 멈추고 pixel ratio1에서 1280×720으로 렌더한 뒤 CSS 크기로 복귀한다.

## 7. 합성 오디오와 transport

자체 파형·감쇠·event별 고정 integer PRNG 노이즈로 드럼을 합성한다. 외부 샘플이나 soundfont가 없다. 준비 단계에서 Worker가 48kHz stereo Float32 PCM을 생성하며 취소 시 Worker를 종료한다. 프리뷰와 내보내기는 같은 PCM을 사용한다. unresolved 타격도 원래 시각에 합성한다.

PCM 타격 위치는 `round((0.5 + hit.time) × sampleRate)`다. 전체 길이는 영상 프레임 수에 맞춰 trim/pad한다. preview volume slider는 미리듣기 음량이며 출력 mute는 별도의 명시적 선택이다.

transport의 시간은 출력 시간0부터 시작하고 `songTime = outputTime - 0.5`로 evaluator에 전달한다. Web Audio의 context origin/offset을 사용하고 `getOutputTimestamp()`가 유효하면 실제 출력 추정 시각을 이용한다. pause는 source를 정지하고 시간을 보존하며 resume/seek는 새 source를 만든다. AudioContext가 suspend되면 시간을 보존하고 재생을 정지한다. 탭 비활성화로 렌더 프레임이 중단되더라도 프레임 시간을 누적하지 않고 복귀 시 오디오 시계의 현재 위치를 평가한다. [Web Audio 시간 API](https://www.w3.org/TR/webaudio/#dom-audiocontext-getoutputtimestamp)

후속 WAV의 부호 계약은 `audioFileTime = songTime - audioStartInSong`을 유지한다. 예를 들어 +0.2s는 곡0.2초, 준비 포함 출력0.7초에 WAV sample0이 시작한다. WAV 로딩과 정렬은 #13이며 현재 UI에 없다.

## 8. 고정 FPS 내보내기

출력 프레임 `n`의 timestamp는 `n/30`, 평가 시각은 `n/30 - 0.5`, duration은 `1/30`이다. 프레임 수는 `ceil((0.5 + D + 0.25) × 30)`이며 30초 음악은923프레임이다. 매 프레임 평가 → 포즈 적용 → 렌더 → `CanvasSource.add(timestamp,duration)` 완료 대기 순서로 처리한다. 원시 RGBA 프레임 전체를 메모리에 쌓지 않는다.

codec 후보는 MP4 AVC/AAC, WebM VP9/Opus, WebM VP8/Opus 순서다. 실제720p30·stereo48kHz 설정으로 지원을 조회한 뒤 6프레임·0.2초 A/V 파일을 시험 인코딩한다. 이어서 파일을 demux/decode하여 audio track 길이와0.1초에 예약한 tone의 실제 onset이 각각 기대값 대비1프레임 이내인지 확인한다. API 지원과 packet 존재만으로 선택하지 않는다. 조합별15초 watchdog을 두며 capability query와 trial도 AbortSignal로 취소한다. 사용할 A/V 조합이 없으면 이유를 표시하고, 사용자의 명시적 무음 선택이 있을 때만 video-only를 허용한다. WebCodecs API 존재만으로 성공을 판단하지 않는다. [Mediabunny 지원 조회](https://mediabunny.dev/guide/supported-formats-and-codecs#querying-codec-encodability)

영상은 4Mbps, 오디오는128kbps를 요청한다. 오디오는1초 PCM chunk씩 영상과 함께 공급한다. 4프레임마다 UI에 실행 기회를 주고 진행률을 표시한다. 취소/오류에는 encoder/output을 정리하고 완료 다운로드로 표시하지 않는다. 이미 시작한 encoder flush/finalize는 브라우저 작업 정리를 기다릴 수 있으므로 마지막 단계 취소가 항상 즉시는 아니다. 성공 시 예상 video packet 수와 오디오 packet 존재를 확인하고, 완성 파일의 audio track 길이도 demux하여 영상 길이와1프레임 이내인지 검사한다. 이 검사는 실제 파일 metadata·일반 플레이어·A/V 동기화 검증을 대신하지 않는다. 2026-10-24 macOS/Chrome 검증에서 AAC priming으로 첫 onset이42.958ms 늦고 출력 길이가 기준을 넘는 조합을 발견하여 이 왕복 검사를 도입했다. 임의 전역 오디오 offset으로 보정하지 않고 통과하는 Opus/WebM 조합을 선택한다. 환경과 수치는 [MVP 검증 기록](verification/mvp-0.1.md)을 따른다.

메모리에는 PCM과 압축 출력이 남는다. 5분48kHz stereo Float32 PCM 한 벌은 약110MiB이고 합성/AudioBuffer 복사·출력 buffer는 추가 공간을 사용한다. 장시간·고밀도 입력의 peak memory와 속도는 별도 측정 대상이다. 실시간 MediaRecorder·ffmpeg.wasm·PNG 시퀀스는 현재 구현에 없다.

## 9. 정적 빌드와 Pages

사용자는 2026-10-24 구현·검증 후 공개 배포를 승인했다. 예정 주소는 `https://postmelee.github.io/drum-vrm/`이며 이 문서 갱신 시점의 공개 배포 성공은 아직 기록하지 않았다. Vite base는 `/drum-vrm/`, production 산출물은 `dist/`다. 업로드 MIDI·오디오·영상 처리에 서버 API를 사용하지 않는다.

- Node24 이상, Rust1.94.1, wasm-pack0.15.0, npm/Cargo lockfile을 사용한다. `src/wasm/`은 `npm run build:wasm` 생성물이다.
- `cargo test --locked`, clippy/format, TypeScript, Vitest, production build와 라이선스 생성 검사를 실행한다. 정확한 명령은 [README](../README.md)에 있다.
- CI/Pages workflow는 검사 성공 뒤 `dist/`만 배포하고 `github-pages` environment와 필요한 최소권한을 사용하도록 구성한다. workflow의 실제 실행 결과는 #12에 기록한다.
- SharedArrayBuffer·COOP/COEP를 전제로 하지 않는다. Worker와 단일 스레드 WASM을 `crossOriginIsolated=false`에서도 검증한다.
- HTTPS/프로젝트 하위 경로에서 WASM·Worker·라이선스와 새로고침을 검사하고, 실제 테스트 URL·SHA·브라우저 환경을 남긴다. 로컬 성공을 Pages 성공으로 표기하지 않는다.

GitHub Pages는 정적 호스팅이다. 대형 사용자 원음·모델·데이터셋을 저장소나 배포물에 넣지 않는다. 서비스 확장 시 호스팅 용도와 현재 제한을 다시 검토한다. [Pages 공식 문서](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages), [사용 제한](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)

## 10. 검증 목표와 증거

[PRD A01–A16](product-requirements.md)의 수치와 목표를 유지한다. Rust/TypeScript 검사는 정규화·접촉·임의 시점 결정성·리그 적용·합성 onset·출력 시간·취소를 다룬다. 통합 증거는 실제 브라우저의 업로드/재생/탐색/취소, 30초 A/V 파일, metadata와 playback, 공개 Pages 경로로 나누어 남긴다.

첫 지원 검증 대상은 데스크톱 Chromium이다. Safari·Firefox·모바일 전체 호환성, 모든 입력 상한의 실기기 성능은 확인 전 보장하지 않는다. 브라우저·OS·버전·검증 일시·commit SHA를 완료 기록에 적고, 수치나 범위를 변경하면 관련 이슈와 PRD를 함께 갱신한다.

## v0.2 작업 화면·드럼 모델 경계

[ADR0003](decisions/0003-minimal-studio-and-models.md)에 따라 `main.ts`는 네 inspector의 상태와 transport를 연결하고, `timeline.ts`는 tempoMap/timeSignatures 기반 피아노 롤을 표시한다. Rust 계획의 추가 metadata는 optional TS 필드로 이전 fixture와 호환한다.

`scene.ts`는 스켈레톤·카메라·부품 선택을 담당한다. `model-procedural.ts`는 자체 키트 외형, `model-library.ts`는 CC0 키트의 부품 분류/타점 정렬, `model-import.ts`는 외부 자원이 없는 로컬 GLB 검증을 담당한다. 외형은 canonical 엔진 좌표를 수정하지 않는다. 원본/라이선스/해시와 runtime 변형 기록은 `public/models/innerscene-provenance.json`에 보존하며 라이선스 생성기가 배포 때 검증한다.
