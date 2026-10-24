# 기반 기술·라이선스 조사 기록

확인일: **2026-10-24 (Asia/Seoul)** · 상태: 채택 후보 조사 완료, 구현/최종 버전 고정 전

원 대화의 라이선스 주장을 그대로 확정하지 않고 아래 공식 저장소·패키지 메타데이터·제작자 문서를 확인했다. 이 목록은 실제 배포 SBOM이 아니다. 버전/commit이 결정되면 lockfile, 출처 URL, 배포 파일, LICENSE/NOTICE, 변경 여부, 대응 소스 제공 위치를 함께 채운다. 논문·학습 데이터·캐릭터·음원은 라이브러리 라이선스와 별도 권리 단위다.

## 1. 제안 스택과 확인 근거

| 구성요소 | MVP 역할/채택 상태 | 공식 확인 라이선스 | 확인한 내용·적용 조건 |
|---|---|---|---|
| `midly` | Rust SMF 파서, 채택 제안 | **Unlicense** | 공식 배포 메타데이터 0.5.3의 license와 기능을 확인. [Cargo.toml](https://docs.rs/crate/midly/0.5.3/source/Cargo.toml.orig), [저장소](https://github.com/negamartin/midly). 설치 버전 고정은 구현 단계. |
| `glam` | 벡터·행렬·quaternion, 채택 제안 | **MIT OR Apache-2.0** | 라이선스 선택 가능. 원 저작권과 선택한 라이선스 사본을 보존하고 포함 코드의 attribution도 확인. [README/license](https://github.com/bitshifter/glam-rs#license), [ATTRIBUTION](https://github.com/bitshifter/glam-rs/blob/main/ATTRIBUTION.md). |
| `wasm-bindgen` | Rust↔JS bindings, 채택 제안 | **MIT OR Apache-2.0** | crate/CLI를 호환되는 고정 버전으로 사용. [공식 저장소 license](https://github.com/wasm-bindgen/wasm-bindgen#license), [가이드](https://wasm-bindgen.github.io/wasm-bindgen/). |
| `three` | WebGL 렌더러·장면·관절 계층, 채택 제안 | **MIT** | 라이선스/저작권 안내 유지. 예제에 등장하는 모델·texture의 조건까지 일괄 MIT라고 추정하지 않음. [LICENSE](https://github.com/mrdoob/three.js/blob/dev/LICENSE). |
| `@pixiv/three-vrm` | 후속 VRM 캐릭터 로딩, MVP 필수 제외 | **MIT** | 라이브러리의 허락과 모델의 사용/재배포 허락은 별개. [LICENSE](https://github.com/pixiv/three-vrm/blob/dev/LICENSE), [API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html). |
| `mediabunny` | 고정 프레임 인코딩·A/V mux, 채택 제안 | **MPL-2.0** | 수정 여부와 무관하게 배포한 MPL 부분의 대응 소스를 얻는 방법을 제공. 앱 코드의 라이선스는 별도로 선택 가능. [LICENSE](https://github.com/Vanilagy/mediabunny/blob/main/LICENSE), [제작자 안내](https://github.com/Vanilagy/mediabunny#license). |
| TypeScript | 브라우저 코드·정적 검사, 채택 제안 | **Apache-2.0** | build 도구와 실제 배포 파일을 구분해 inventory에 기록. [LICENSE](https://github.com/microsoft/TypeScript/blob/main/LICENSE.txt). |
| Vite | 정적 build·개발 서버, 채택 제안 | **MIT** | 최종 lockfile의 전이 의존성은 별도 확인. [LICENSE](https://github.com/vitejs/vite/blob/main/LICENSE), [배포 안내](https://vite.dev/guide/static-deploy.html). |

이 프로젝트의 소스 라이선스는 위 의존성과 별도 결정이다. `main`/`dev` URL은 조사 출처이며 배포 버전 식별자가 아니다. 실제 채택 시 tag/commit의 영구 링크로 교체하거나 함께 기록한다. 표에 없는 helper crate/npm 패키지를 추가하면 같은 절차로 등록한다.

## 2. 확인된 기능과 아직 검증하지 않은 가정

| 주제 | 확인된 사실 | 이 프로젝트가 검증해야 할 것 |
|---|---|---|
| MIDI 파싱 | midly는 SMF와 시간 형식 enum을 제공. 기본 `parallel`은 Rayon을 포함하며 비활성화 가능. strict feature도 제공. [features](https://docs.rs/midly/latest/midly/#about-features), [Timing](https://docs.rs/midly/latest/midly/enum.Timing.html) | 브라우저 단일 스레드 build, 파일별 tempo merge, 시간 반올림, 손상 입력·상한·취소. 파서는 드러머 손 배정을 제공하지 않음. |
| 수학 | glam에는 f32/f64 타입과 wasm SIMD 선택지가 있음. SIMD 이점은 문서도 보장하지 않음. [공식 README](https://github.com/bitshifter/glam-rs) | 기본 scalar 동작, 도달성·IK·수치 오차. SIMD는 성능 측정 전 기본값으로 요구하지 않음. |
| IK | Three.js CCDIKSolver는 `SkinnedMesh`를 대상으로 하며 iteration/관절 제한 등을 제공. [공식 API](https://threejs.org/docs/pages/CCDIKSolver.html) | primitive 리그에는 Rust 해석적 2-bone IK를 우선 제안. CCD 채택 시 초기화·수렴·스틱 접촉·seek 순서 독립성 검증. |
| VRM pose | three-vrm은 normalized/raw humanoid 접근과 pose 적용 API를 제공. [공식 API](https://pixiv.github.io/three-vrm/docs/classes/three-vrm.VRMHumanoid.html) | 서로 다른 팔 길이·rest basis·앉은 위치·그립의 재보정. API가 접촉 보정을 자동 해결한다고 가정하지 않음. |
| 프레임 출력 | Mediabunny CanvasSource는 timestamp/duration(초)로 프레임을 받음. [공식 API 설명](https://mediabunny.dev/guide/media-sources#canvassource) | WebGL canvas capture, 720p30 5분 입력의 메모리·취소·실제 파일 유효성. |
| codec | Mediabunny는 실제 구성에 대한 video/audio encoding 검사 제공. 컨테이너와 codec 호환 표가 있음. [공식 codec 문서](https://mediabunny.dev/guide/supported-formats-and-codecs) | AVC/AAC MP4, VP8/VP9+Opus WebM이 사용자 OS/브라우저에서 실제 가능한지 검사. 지원 표를 브라우저 보장으로 읽지 않음. |
| WebCodecs | 표준은 특정 codec을 요구하지 않으며 secure context의 encoder API를 정의. [W3C](https://www.w3.org/TR/webcodecs/) | 실제 HTTPS origin, encoder 구성·가용성·시험 인코딩·에러 후 해제. |
| 오디오 시계 | Web Audio는 currentTime 및 getOutputTimestamp를 정의. [W3C](https://www.w3.org/TR/webaudio/#dom-audiocontext-getoutputtimestamp) | 출력 지연 보정, pause/resume/seek, 합성 PCM onset과 실제 영상 접촉. |

## 3. Mediabunny/MPL-2.0 배포 준비

Mozilla의 공식 설명에 따르면 MPL은 파일 단위 copyleft다. 별도 작성한 앱 파일이 한 minified bundle에 들어간다는 이유만으로 모두 MPL이 되는 것은 아니다. 그러나 minified JavaScript는 수정에 적합한 소스 형태가 아니며, 클라이언트에 전달하는 코드는 배포로 본다. 수정하지 않은 라이브러리도 배포한 버전의 소스를 얻는 방법이 필요하다. [Mozilla MPL FAQ Q1, Q16, Q17](https://www.mozilla.org/en-US/MPL/2.0/FAQ/), [MPL 2.0 원문 §3](https://www.mozilla.org/en-US/MPL/2.0/)

배포 준비 작업은 다음과 같다.

1. `mediabunny`의 정확한 npm 버전·integrity·source tag/commit을 고정한다.
2. 대응 소스 원본을 가리키는 버전 고정 URL과 LICENSE를 배포의 “오픈소스 안내”에 넣는다. URL이 실제 수정 가능한 소스와 해당 버전으로 연결되는지 확인한다. 라이브러리를 수정했다면 수정된 MPL 파일의 소스도 제공한다.
3. 라이선스·저작권 고지와 필요한 배포 안내를 보존한다. 단순 npm package 이름이나 최신 저장소 링크만을 대응 소스 근거로 삼지 않는다.
4. 최초 릴리스의 source archive/manifest를 함께 보존하여 이후 upstream 변경에도 어떤 소스를 사용했는지 추적한다.
5. 추가 encoder extension은 별도 의존성으로 조사한다. core의 MPL 확인이 포함 codec 구현·다른 패키지·특허 조건을 일괄 확인한 것은 아니다.

MVP는 브라우저가 제공하는 encoder와 Mediabunny core를 먼저 사용한다. MPEG/codec 관련 조건은 라이브러리 소스 라이선스와 다른 검토 항목이며, 자체 encoder를 배포하거나 유료 서비스로 확장할 때 채택 범위를 기준으로 다시 검토한다.

## 4. GitHub Pages와 빌드 제약

GitHub Pages는 정적 파일 호스팅이다. Free의 public 저장소에서 사용할 수 있고 프로젝트 사이트 기본 URL은 `<owner>.github.io/<repository>`다. `postmelee/drum-vrm`의 제안 URL은 `https://postmelee.github.io/drum-vrm/`이며 아직 배포 확인 전이다. [공식 설명](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages)

| 제약/확인 사항 | 프로젝트 대응 |
|---|---|
| Pages project 하위 경로 | Vite `base: '/drum-vrm/'`; WASM·Worker·라이선스 링크까지 production 경로로 검사. [Vite 공식 안내](https://vite.dev/guide/static-deploy.html#github-pages) |
| 서버 실행 없음 | 업로드 MIDI·음원·출력 파일은 로컬 브라우저 처리. 파일 선택이 네트워크 업로드를 뜻하지 않도록 UI 문구 구분. |
| 배포 권한/환경 | Actions build artifact → deploy. 최소 `pages: write`, `id-token: write`, `needs`, `github-pages` environment. [공식 workflow](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) |
| 공유 메모리 격리 | threaded WASM은 별도 브라우저/헤더 요건이 있음. MVP는 단일 스레드이며 `crossOriginIsolated=false`에서도 통과하도록 설계. [wasm-bindgen threaded 예제](https://wasm-bindgen.github.io/wasm-bindgen/examples/raytrace.html), [격리 요건](https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated) |
| 서비스 한도 | 게시 1GB, 배포 10분, 월 100GB soft bandwidth limit; custom Actions workflow는 시간당 10 build soft limit의 예외. 대형 데이터를 번들에 넣지 않음. [공식 제한](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits) |
| 상업 서비스 확장 | Pages가 상업 거래/SaaS를 위한 무료 호스팅은 아니라는 공식 용도 제한을 고려해 별도 호스팅 결정. 같은 공식 제한 참조. |

임의 COOP/COEP 헤더를 Pages에서 설정할 수 있다고 확인한 상태가 아니다. service worker 우회나 외부 proxy를 기본 배포 조건으로 넣지 않는다. 실제 배포 성공은 승인 후 Pages URL에서 WASM 초기화·파일 분석·재생·다운로드까지 확인했을 때 기록한다.

## 5. 채택하지 않거나 후속 검토할 경로

- `mp4-muxer`: 공식 저장소가 deprecated 상태이며 Mediabunny로의 이동을 안내한다. 신규 MVP의 직접 의존성으로 채택하지 않는다. [공식 공지](https://github.com/Vanilagy/mp4-muxer#readme)
- `three-vrm` / VRM 1.0 meta: 라이브러리는 MIT이지만 모델의 권리는 별도다. 공식 schema의 `commercialUsage`, `creditNotation`, `allowRedistribution`, `modification`, `otherLicenseUrl`, `licenseUrl`과 저자·추가 라이선스를 기록한다. 영상 렌더링·수익화와 모델을 포함하는 GLB/VRM 재배포는 서로 다른 사용이므로 각각 판단한다. 영어 meta 안내는 아직 TODO이므로 **공식 schema**를 확인 근거로 사용했다. [VRM 1.0 meta schema](https://github.com/vrm-c/vrm-specification/blob/master/specification/VRMC_vrm-1.0/schema/VRMC_vrm.meta.schema.json), [후속 이슈 #17](https://github.com/postmelee/drum-vrm/issues/17)
- GLTFExporter / VRMA: Three.js의 MIT addon인 GLTFExporter에는 binary GLB와 명시적 `AnimationClip` 목록 출력 옵션이 있다. 실행 중인 IK를 자동 저장하는 계약은 없으므로 프로젝트에서는 최종 본 변환을 먼저 bake하도록 설계한다. VRMA는 glTF 기반의 humanoid·표정·시선 애니메이션 형식이며 여러 VRM 사이에 모션을 옮길 수 있다. 체형 변경 후의 스틱 접촉과 VRM 전용 extension 보존은 별도 검증 항목이다. 모델을 포함하는 출력은 모델 권리, 모션만의 출력은 모션 출처 조건을 확인한다. [GLTFExporter 공식 API](https://threejs.org/docs/pages/GLTFExporter.html), [VRM Animation 공식 설명](https://vrm.dev/en/vrma/), [후속 이슈 #19](https://github.com/postmelee/drum-vrm/issues/19)
- `ffmpeg.wasm`·추가 codec 패키지: 공식 FAQ는 **`@ffmpeg/ffmpeg` wrapper는 MIT**, **`@ffmpeg/core`는 FFmpeg 및 포함 외부 라이브러리 조건을 따른다**고 구분한다. FFmpeg 자체는 LGPL-2.1-or-later 기반이며 선택한 GPL 구성요소가 적용 범위를 바꿀 수 있다. 실제 core 버전·build flags·포함 codec·대응 소스는 아직 감사하지 않았고 전체를 MIT로 표기하지 않는다. 공식 FAQ는 native 대비 느린 실행과 멀티스레드의 추가 CPU/메모리 부담도 설명한다. 이 프로젝트의 수치로 일반화하지 않으며 기본 Pages 번들에 넣지 않는다. [ffmpeg.wasm FAQ](https://ffmpegwasm.netlify.app/docs/faq/), [FFmpeg 공식 license 설명](https://ffmpeg.org/legal.html), [codec 이슈 #9](https://github.com/postmelee/drum-vrm/issues/9)
- ONNX Runtime: 공식 저장소의 **MIT** 라이선스를 확인했다. 후속 모델 추론 후보이며 학습 모델·데이터·가중치·출력 권리가 확보되고 규칙 기반의 구체적 한계가 드러날 때 검토한다. 런타임의 MIT는 가중치·데이터의 허락을 대신하지 않으며 실제 선택할 web package와 전이 의존성도 채택 시 다시 조사한다. [ONNX Runtime LICENSE](https://github.com/microsoft/onnxruntime/blob/main/LICENSE), [후속 이슈 #21](https://github.com/postmelee/drum-vrm/issues/21)
- 샘플 팩/외부 캐릭터: MVP는 자체 primitive mesh와 자체 합성 파형을 사용한다. 실제 음악/사용자 MIDI는 자동으로 Git에 커밋하거나 공개 fixture로 배포하지 않는다.

## 6. 구현 시 남길 의존성 기록 형식

다음 필드를 릴리스마다 채운다. 현재 값이 없는 항목을 “검증 완료”로 표시하지 않는다.

| 필드 | 기록할 값 |
|---|---|
| 식별 | package/crate, 직접/전이 의존성, runtime/build/test 구분 |
| 고정값 | version, lockfile integrity/checksum, upstream commit/tag |
| 출처 | 공식 repository, package URL, 그 버전의 LICENSE/NOTICE |
| 포함 범위 | 배포 JS/WASM/asset 경로, 불포함 개발 전용 여부 |
| 권리 | SPDX, 저작권, 추가 attribution/NOTICE, source 제공 의무 |
| 변경 | 원본 사용/patch 여부, patch 위치, 수정 source URL |
| 검증 | 확인자/확인일, build·codec probe·라이선스 검사 결과 |

업그레이드 PR은 lockfile 변경과 함께 이 기록을 갱신하고, API·라이선스·전이 의존성·브라우저 출력에 달라진 점을 확인한다. 구체적인 배포가 없는 현 단계에서 “상업 이용 전체 승인”이나 “모든 브라우저 지원”이라는 판정을 내리지 않는다.
