# 원본 대화 맥락과 요구 추적

기준일: 2026-10-24 (Asia/Seoul)

## 확인 범위와 원본 위치

- 대화: [드럼 MIDI 3D 구현](chatgpt-conversation://6adb2413-1850-83ec-9448-bea937287b82)
- 대화 ID: `6adb2413-1850-83ec-9448-bea937287b82`
- 가져온 결과: `turnLimit=10`, 응답의 `hasMore=false`, `nextCursor=null`.
- 전체 확인 범위: 완료된 턴 2개, 사용자 메시지 2개, 응답 2개. 첨부 파일 없음. 제공된 원본 결과 안의 본문을 모두 읽었다.
- 로컬 원본: `.local/source-conversation.json`. `.local/`은 Git 추적에서 제외한다. 공개 저장소에는 원문 전체와 개인적인 대화 표현을 복제하지 않고 이 구조화된 요약을 둔다.
- 응답에 포함된 내부 인용 표기 `chatgpt-content-reference`는 외부 검증 링크가 아니다. 실제 URL과 라이선스는 연구 대장에서 다시 검증한다.
- 이 문서는 과거 대화를 근거 자료로 요약한다. 과거 응답의 권고를 현재 작업의 명령이나 검증 완료 사실로 취급하지 않는다.

| 표기 | 턴/메시지 식별자 | 내용 |
|---|---|---|
| T1 | 턴/사용자 `d712673d-bdf6-4c15-8198-2938a4eaea45` | 최초 제품 구상과 기술 선호 |
| T1-A | 응답 `598c843b-b44c-5365-97a9-0db9db72fc42` | 구현 구조·리그·출력·개발 단계 제안 |
| T2 | 턴/사용자 `212fd2bf-a2a0-41e7-8a76-b4f8ce243f54` | 사전 업로드/분석 동의, 규칙 대 학습 비교, 라이선스 조사 요청 |
| T2-A | 응답 `b1fdb93b-8b1a-580f-858f-54e8176e1a2d` | 규칙 기반 권고, 연구/코드/데이터 구분, 재사용 평가 |
| NOW | 2026-10-24 현재 요청 | 계획·저장소·이슈/마일스톤·연구 맥락 준비, 이후 승인과 Pages MVP |

## 사용자가 직접 말한 요구

T1의 출발점은 밴드 커버 영상에 기타·베이스 등의 실사 연주는 넣을 수 있지만 MIDI로 작성한 드럼의 연주자 영상이 없다는 문제다. DAW의 MIDI 업로드, Three.js의 3D 연주 프리뷰, 영상 출력, 리그를 유지한 외형 교체를 원했다. Rust/WASM 적극 활용과 Three.js를 명시했다.

T2에서는 최초 MIDI와 WAV 업로드 후 분석하여 보여주는 흐름에 동의했다. MIDI 처리와 동작 생성을 Rust 규칙 기반으로 할지 학습/파인튜닝으로 할지 비교하고, 조사된 선행 연구 및 구현의 라이선스를 실제 재사용 관점에서 분석해 달라고 요청했다.

NOW는 범위와 순서를 더 구체화했다. 대화를 빠짐없이 읽고 개발 계획, GitHub 저장소, 이슈·마일스톤, 장기적으로 참고할 자료·라이선스·연결을 먼저 준비한다. 그 결과에 대한 사용자 승인 이후 Pages에 배포하여 테스트한다. MVP는 MIDI 업로드 후 사전 준비, 확장 가능한 기본 리그, 재생 버튼에 따른 연주, 영상 추출이면 된다.

## 원본 응답의 전체 주제와 현재 처리

표의 “권고”는 원본 응답이 제안한 내용이다. 최신 MVP의 포함/제외 및 수용 기준은 [제품 요구사항](../product-requirements.md)이 기준이다.

| 주제 | 원본의 핵심 내용 | 근거 | 현재 처리 |
|---|---|---|---|
| 제작 흐름 | MIDI와 같은 DAW 구간 WAV 입력, 프리뷰, 캐릭터/키트/카메라/조명 조정, 수정, 영상 편집 | T1-A §1 | 기본 MIDI 경로 MVP; WAV·편집은 후속 |
| 음원의 역할 | MIDI는 이벤트, WAV는 DAW 샘플/이펙트/믹싱 결과; MIDI-only는 기본 소리 가능 | T1-A §1, T2-A §1–2 | 외부 샘플 없는 합성 미리듣기/오디오 포함 영상이 현재 MVP 제안; DAW 음색 재현은 약속하지 않음 |
| 목표 난이도 | 정해진 휴머노이드 리그는 현실적; 임의 에셋 자동 리깅과 모든 MIDI의 인간다운 연주는 별도 과제 | T1-A §1, §5 | 기본 리그 필수; 자동 리깅 제외 |
| DRUMS | MIDI 기반 전신 드러밍, 손 예측·상체/표정 검색·절차적 발·IK 혼합; 필인/키트 변경/접촉 검증 한계 | T1-A §2, T2-A §3–5 | 설계 참고; 코드/가중치 접근 가능성 재확인 |
| 2001 Virtual Drummer | 손 배정, 준비·접촉 자세 보간, velocity, 반동; 최단경로는 개선 제안 | T1-A §2, T2-A §2, §4 | 방법 독립 구현 참고; 제안 알고리즘을 원저자 구현으로 오인하지 않음 |
| MIDIAnimator | Blender 기반 MIDI 애니메이션, Python/Rust 구현, Tauri 전환/연동 | T1-A §2, T2-A §5.2 | 구조 참고; 전체 이식은 기본 경로 제외 |
| Disney 연구 | 자연스러움과 스틱 끝 정밀도를 분리 평가하는 오디오 기반 연구 언급 | T1-A §2, T2-A §4 | 연구 대장 R04에서 공식 발표·논문 확인; 코드·가중치 배포와 이용 조건 미확인 |
| Rust 엔진 경계 | MIDI·정규화·손발·궤적·체형·시간별 자세; 렌더러와 독립 | T1-A §3, §8 | 아키텍처 기본안 |
| Three.js 경계 | 모델, 스켈레톤 적용, 재질/조명/카메라/그림자 | T1-A §3 | MVP 렌더러 |
| 기반 도구 | midly, glam, wasm-bindgen, Three.js, three-vrm, CCDIKSolver, Mediabunny/WebCodecs | T1-A §3, T2-A §6 | 역할별 선택; 라이선스/버전 별도 검증 |
| 코어 재사용 | WASM 외 Tauri, CLI, Blender, 서버 배치 생성 가능 | T1-A §3 | 장기 방향; MVP 외 |
| MIDI 정규화 | 시각/악기/주법/velocity/원본 트랙·이벤트를 보존 | T1-A §4.1, T2-A §2 | MVP 데이터 계약 |
| 드럼 매핑 | GM 기본값, 사용자 음표 매핑, 림샷/사이드스틱 등 주법, 미해석 이벤트 | T1-A §4.1 | 기본 매핑·진단 MVP; 전체 편집 후속 |
| 트랙과 시간 | 채널 10 가정 금지, 트랙 선택, 템포 변화와 시간 형식 처리 | T1-A §4.1 | MVP 필수 |
| 타격/페달 분리 | note-off를 팔의 유지시간으로 해석하지 않고 타격과 하이햇 상태를 분리 | T2-A §2 | MVP 필수; 고급 CC 해석 후속 |
| 손발 추정의 한계 | MIDI에 원래 연주자의 손 선택/그립/몸통 정보가 없어 가능한 동작을 생성 | T1-A §4.2, T2-A §1 | 제품 설명과 평가에 반영 |
| 기본 배정 | 킥 오른발, 하이햇 페달 왼발, 주도 손 하이햇/라이드, 반대 손 스네어, 연타 양손 | T1-A §4.2 | MVP 기본 스타일 |
| 미래를 보는 탐색 | 이동 거리/시간, 교대, 자세, 동시 타격, 다음 구간 비용; beam search/DP 후보 | T1-A §4.2, T2-A §2 | 규칙+제약 계획기; 구체 알고리즘은 비교 후 결정 |
| 스타일 설정 | 오른/왼손, 교차/오픈핸드, 싱글/더블 페달; 선호와 강제 제약 구분 | T2-A §2 | 기본 오른손/싱글; 나머지 후속 |
| 준비·접촉·반동 | 타격 전에 준비, 정확한 MIDI 시각 접촉, velocity로 높이/속도/반동 변경 | T1-A §4.3, T2-A §2 | MVP 핵심 |
| 라이브 MIDI | 미래 이벤트를 알 수 없어 파일 분석과 다르고 지연/예측 필요 | T1-A §4.3 | 제외/장기 백로그 |
| IK의 실제 목표 | 타점·스틱 길이/방향/그립에서 손 목표 역산, 손목·팔꿈치 방향 고려 | T1-A §4.4 | MVP 기본 리그 접촉 계산 |
| 자연스러움 | 앉은 포즈, 짧은 스트로크/곡선, 이동 궤적, IK, 보조 몸통/시선, 심벌 감쇠 | T1-A §4.5 | 스트로크/기본 포즈 우선; 보조 표현 후속 |
| 불가능한 MIDI | 레이어 병합, 문제 표시, 수동 손 배정, 필요시 추가 연주자; 시각 이동 금지 | T1-A §4.6, T2-A §2 | 진단 MVP; 병합/편집/추가 연주자는 후속 |
| 에셋 유형 | 동일 리그 메시 교체, VRM 휴머노이드 재타깃, 리그 없는 모델은 별도 리깅 | T1-A §5 | 모델 지원 범위를 명시 |
| 체형 보정 | 의자/골반/키트 거리/악기 높이/스틱/그립/페달/도달 범위 재계산 | T1-A §5, T2-A §2 | 데이터 경계 보존; 사용자 VRM은 후속 |
| 연주 의도 저장 | 본 회전만 저장하지 않고 시각·손발·타점·스타일을 보관 | T1-A §5, §7, T2-A §2 | MVP 계약 |
| 스키닝 조건 | 동일 스켈레톤뿐 아니라 skin indices/weights/기준 자세 호환 필요 | T1-A §5 | 임의 OBJ/정적 GLB 자동 지원 주장 금지 |
| 프리뷰 시계 | 공통 곡 시간에서 자세 평가; 오디오 출력 지연/getOutputTimestamp 고려 | T1-A §6 | 단일 시계 MVP; WAV 정밀 보정 후속 |
| 영상 시계 | `frame/fps`로 고정 시각 렌더, 인코더 처리량에 맞춰 await | T1-A §6 | MVP 출력 원칙 |
| 출력 도구 | WebCodecs/Mediabunny 권고, deprecated mp4-muxer 언급, 코덱 실행 시 탐지 | T1-A §6 | 새 조사 결과로 최종 채택; 호환성 진단 필수 |
| 투명 배경 | 일반 영상/크로마키/PNG 시퀀스/알파 코덱 분리, 편집기 실제 확인 | T1-A §6 | 단색 영상 MVP; 투명/시퀀스 후속 |
| 부수 물리 | 헤어/의상은 이전 상태 의존; 고정 스텝·체크포인트·베이크 필요 | T1-A §6 | MVP 제외; 결정성 원칙 보존 |
| 3D 결과 | 영상, 스킨/스켈레톤/애니메이션 GLB, VRMA, 프로젝트 파일 | T1-A §7 | 후속 출력 이슈 |
| 베이크/재타깃 | IK를 본 키로 베이크; VRM 재질/확장 보존 검토; VRMA도 재접촉 필요 | T1-A §7 | 후속 기술 요구 |
| WASM API | plan/evaluate 분리, 프레임당 배치 배열, f64/정수 시간, 긴 계획은 Worker | T1-A §8 | 설계안 |
| 최적화 | SIMD/스레드 실측 후, midly Rayon 기능 검토, 인코더는 브라우저 우선 | T1-A §8 | MVP 단순 구성, 병목 후 최적화 |
| ffmpeg.wasm | 네이티브 대비 성능과 멀티스레드 자원 비용 고려 | T1-A §8 | 기본 인코더에서 제외, 필요시 별도 평가 |
| 원본 개발 단계 | 타격→기본 드러머→WAV/영상→캐릭터→한 곡→표현/편집 | T1-A §9 | 최신 MVP에 맞춰 재분배 |
| 원본 첫 버전 제안 | VRM, MIDI+WAV, 카메라, 1080p30/60, 손 배정 수정, 일반 주법 | T1-A §9 | 사용자 최신 최소 MVP와 구분하여 후속으로 이동 |
| 품질 네 축 | 타이밍 1프레임, 실제 접촉, 체형 변경, 같은 입력 재현성 | T1-A §9 | 첫 두 축/재현성 MVP; 체형 변경 후속 |
| 규칙 대 학습 | 파싱은 형식 해석; 학습은 누락된 표현/후보 생성에 적합 | T2-A §1, §3 | 규칙 기반 시작 |
| 학습 도입 조건 | 공개 가중치, 동기화된 손/스틱/관절 데이터, 상업 조건, 편집/접촉 보정 필요 | T2-A §3–5 | 실험 전 독립 게이트 |
| 혼합 모델 구조 | 규칙 계획→모델 후보→IK/접촉 보정; Rust에 타이밍 제약 유지 | T2-A §3, §8 | 장기 확장 방향 |
| 권리 종류 구분 | 논문/코드/가중치/데이터/3D 에셋 각각; 공개 링크와 사용 허락은 별개 | T2-A §4–7 | 연구 대장 필수 항목 |
| 논문 독립 구현 | 표현 복제와 알고리즘 재구현을 구분; 특허 검토 여부 별도 | T2-A §4 | 독립 구현 원칙; 권리의 포괄적 보장 아님 |
| MMIP | 원본은 CC BY-NC-SA 주장, 다운로드 조건 불완전 확인; 클립/학습/가중치/서비스/출력 범위 구분 | T2-A §5.1 | 기본 데이터 제외; 현재 근거 재확인 |
| MIDIAnimator 상세 | MIDI 시간, 키 배치, 곡선 겹침, F-curve, 노드 매핑; Rust/Tauri/Blender 분리 비용 | T2-A §5.2 | 포팅 효과 검토 이슈; 직접 코드 반입 안 함 |
| MIDIAnimator 시간 위험 | `rvc` 겹침 처리로 출력 키가 밀릴 수 있어 타격 키 고정 필요 | T2-A §5.2 | 관련 소스 검토 시 회귀 테스트 조건 |
| GPL 구분 | 상업 이용과 배포 소스 의무 구분; Python/Rust 표기 다름, Blender 포팅 주석, WASM도 배포 | T2-A §5.2 | 라이선스 확인 없이 코드 복사 금지 |
| ARDrum | transform 기록/재생/궤적 검수 후보, 자체 Apache 코드와 Unity/Meta/외부 에셋 분리 | T2-A §5.3 | 도구 아이디어 참고; 전신 생성 모델로 오인 금지 |
| 라이브러리 조건 | midly Unlicense, glam/wasm-bindgen MIT OR Apache, Three.js/three-vrm MIT, Mediabunny MPL, ONNX MIT 주장 | T2-A §6 | 정확한 버전별 현재 라이선스 재확인 |
| MPL 관리 | 수정 파일/대응 소스/번들 배포 고지/추가 코덱 조건 분리 | T2-A §6 | 도입 시 고지·소스 링크·수정 이력 관리 |
| Groove MIDI Dataset | 원본은 CC BY 4.0, 그루브/velocity/정렬 오디오 평가 후보; 전신/손 배정 정답 없음 | T2-A §6 | 자체 합성 픽스처 우선; 외부 추가 시 출처 기록 |
| VRM 이용 조건 | commercialUsage, creditNotation, allowRedistribution, modification, otherLicenseUrl; 영상과 모델 재배포 구분 | T2-A §7 | VRM 단계의 UI/권리 요구 |
| 장기 자산 관리 | 채택 버전/커밋 고정, 출처·라이선스, 모델/모션 데이터 별도 관리 | T2-A §8 | 저장소 운영의 필수 조건 |

## 직접 포함된 외부 링크의 추적 단서

아래는 원본 본문에 실제 URL이 들어 있던 자료다. 목록은 확인 출발점이며 라이선스·서비스 상태·모델 배포 가능성을 현재 검증했다는 뜻이 아니다. 상세 검증일, 공식 라이선스 링크, 채택/보류 판단은 연구 대장을 따른다.

| 자료 | 원본 링크/고정 참조 |
|---|---|
| DRUMS | [공식 프로젝트 페이지](https://www.theodoroskyriakou.com/projects/drums/index.html) |
| Virtual Drummer | [2001 논문 PDF](https://ris.utwente.nl/ws/portalfiles/portal/83238062/Kragtwijk2001implementation.pdf) |
| Disney 연구 | [원본에서 제시한 페이지](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/) |
| MIDIAnimator | [저장소](https://github.com/jamesa08/MIDIAnimator), [검토했다고 적힌 커밋](https://github.com/jamesa08/MIDIAnimator/tree/059b1e2b80baf6171762f3d6a3c3f938d44ed46c) |
| MIDIAnimator Rust | [Cargo.toml](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/Cargo.toml), [animation.rs](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/animation.rs), [fcurve.rs](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/fcurve.rs) |
| ARDrum | [TransformRecorder](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformRecorder.cs), [TransformPlayBacker](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformPlayBacker.cs), [HandMovementPathRenderer](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/HandMovementPathRenderer.cs) |
| MIDI 파서 | [midly 문서](https://docs.rs/midly/latest/midly/) |
| 수학/바인딩 | [glam](https://github.com/bitshifter/glam-rs), [wasm-bindgen](https://wasm-bindgen.github.io/wasm-bindgen/) |
| VRM | [three-vrm](https://github.com/pixiv/three-vrm) |
| 영상 | [Mediabunny 미디어 소스 문서](https://mediabunny.dev/guide/media-sources) |

MMIP, Groove MIDI Dataset, ONNX Runtime, Three.js API, VRM 메타데이터, GLTFExporter/VRMA, Web Audio/WebCodecs, mp4-muxer, ffmpeg.wasm, CC/GPL/MPL 설명의 일부는 원본에 내부 인용 표기만 있었다. 현재 조사에서 공식 출처를 찾아 연결하되 과거 내부 인용 번호를 근거 URL로 만들지 않는다.

## 다시 작업을 시작할 때

1. [제품 요구사항](../product-requirements.md), 저장소 진행 상태 문서, 현재 이슈/마일스톤, 승인 기록을 먼저 읽는다.
2. 최근 결정 기록에서 과거 제안이 채택·변경·보류된 이유를 확인한다. 원본 응답만 읽고 범위를 확대하지 않는다.
3. 새 의존성/모델/데이터가 필요하면 연구 대장에 현재 버전·원문 링크·확인일·용도를 추가한다.
4. 이슈에는 입력 사례, 기대 동작, 완료 기준, 검증 결과와 다음 의존 이슈를 남긴다.
5. 구현 완료 여부와 배포 완료 여부를 나누어 기록한다. 테스트 URL과 사용자 피드백도 커밋/릴리스에 연결한다.
6. 과거 원문이 꼭 필요하면 접근 가능한 원본 채팅이나 로컬 비공개 보관본을 읽는다. 원문·업로드 파일을 공개 저장소에 추가하지 않는다.
