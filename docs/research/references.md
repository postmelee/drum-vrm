> Research record preserved from [baseline 66951cd](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/research/references.md). Dated findings and statements of implementation status below belong to that record; use the [current specifications](../../specs/README.md) and [model guide](../models.md) for current behavior.

# 선행 연구·구현·데이터 참고자료

검토일: **2026-10-24 (Asia/Seoul)**. 대상 대화: `6adb2413-1850-83ec-9448-bea937287b82`, 「드럼 MIDI 3D 구현」의 두 사용자 요청과 두 답변 전체.

이 문서는 계획의 근거를 다시 찾을 수 있도록 원저자·기관·저장소의 링크와 확인 범위를 기록한다. 대화의 주장 자체를 증거로 삼지 않았다. 논문, 소스 코드, 학습 가중치, 데이터, 모델·텍스처·음원은 서로 다른 배포물이므로 각각 조건을 확인한다. 아래 **채택 판단은 이 프로젝트의 구현 결정**이며, 미확인은 부재를 뜻하지 않는다.

## Performance instruction and expression foundation

The bilingual [performance source register (EN)](../performance-foundations.md) / [연주 자료 대장 (KO)](../performance-foundations.ko.md) adds instructional references I01–I03 with author credits, source locators, rights boundaries and current/planned status. [ADR 0006](../decisions/0006-technique-and-expression-foundation.md) maps instruction to independently proposed technique/support/contact rules and a future constrained 0–100 expression policy. R01–R07 below remain the canonical paper/code/data register; do not duplicate their license/adoption claims into a new unmaintained catalog. A source is not a reproduced implementation or a bundled asset.

## 이번 단계의 결정

- MVP는 독립적인 규칙 기반 Rust/WASM 엔진과 자체 제작한 기본 리그·드럼 지오메트리·스트로크로 시작한다.
- 선행 연구는 설계·평가 근거로 인용한다. 논문 전문, 그림, 시연 영상, 연구용 모션을 저장소나 Pages에 복제하지 않는다.
- 아래 외부 코드는 현재 **편입하지 않았다**. MIDIAnimator의 직접 포팅은 보류하고, ARDrum은 후속 기록·검수 도구 후보로 남긴다.
- Groove MIDI Dataset은 후속 테스트 후보다. 기본 공개 데모는 직접 생성한 MIDI로 구성하고, GMD 파일 채택 시 파일 단위 출처·해시·수정 내용을 등록한다.
- MMIP 모션과 출처·조건이 불명확한 가중치는 MVP에 포함하지 않는다. 해당 자료 확보를 MVP의 선행 조건으로 만들지 않는다.

## 검증 목록

| ID | 대상·배포물 | 확인된 조건·상태 | 프로젝트에서의 사용 |
| --- | --- | --- | --- |
| R01 | DRUMS (MIG 2025) 논문 | 논문 CC BY 4.0 직접 확인. 실행 코드·가중치 배포 미확인 | 손·상체·발 분리 설계, 한계와 평가 참고 |
| R02 | MMIP 데이터셋 | 공식 도메인 검색 색인의 CC BY-NC-SA 4.0 확인. 직접 페이지·다운로드 약관은 미검증 | 모션 편입·학습 보류 |
| R03 | 3D Virtual Drummer (2001) 논문 | Springer 저작권 확인. 원저자 소스 패키지·코드 라이선스 미확인 | 손 배정·준비·접촉·반동 개념을 독립 구현 |
| R04 | Disney 드러머 모션 연구 (2026) | 공식 발표·arXiv v1 확인. 논문 재게시 제한, 코드·가중치·데이터 조건 미확인 | 타격 공간 오차와 시간 정렬의 분리 평가 |
| R05 | MIDIAnimator 지정 커밋 | GPLv3 본문, Python의 or-later 고지, Rust의 `GPL-3.0` 표기 확인 | 직접 코드 편입 보류 |
| R06 | ARDrum-VirtualHand 지정 커밋 | 저장소 Apache-2.0 및 C# 코드 확인. SDK·에셋 전체 감사 미실시 | 후속 기록·재생·궤적 검수 후보 |
| R07 | Groove MIDI Dataset v1.0.0 | Google 공식 페이지 CC BY 4.0, 파일 URL·해시 확인 | 명시적 출처를 가진 입력 테스트 후보 |

### R01. DRUMS: Drummer Reconstruction Using Midi Sequences

- 저자: Theodoros Kyriakou, Panayiotis Charalambous, Andreas Aristidou. MIG 2025, 7쪽. [DOI: 10.1145/3769047.3769066](https://doi.org/10.1145/3769047.3769066).
- 원자료: [저자 프로젝트 페이지](https://www.theodoroskyriakou.com/projects/drums/index.html), [저자 배포 PDF](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf).
- 라이선스 증거: PDF 1쪽 하단에 **Creative Commons Attribution 4.0 International License** 명시. 이 고지는 논문에 대한 증거이며 데이터·가중치의 이용허락을 증명하지 않는다.
- 방법: 손 궤적 BiLSTM, 상체·표정 검색, 절차적 발 동작, IK를 결합한다. §3.1은 MMIP 드럼 데이터 약 35분을 사용했다고 설명한다.
- 한계: §5는 복잡한 필인과 미학습 패턴, 고정된 키트 배치 의존성을 명시한다. 드럼 지오메트리가 없어 드럼 표면 접촉 정확도를 검증하지 못했다고 밝힌다. 손 속도 피크와 MIDI의 정렬을 실제 접촉 오차로 동일시하지 않는다.
- 확보 상태: 프로젝트 페이지의 Paper·DOI·영상 링크는 확인했다. 해당 페이지에서 실행 코드·학습 가중치 다운로드 링크는 찾지 못했다. 공개 모델을 바로 채택할 수 있다는 근거는 없다.
- 활용: 모듈 분리와 평가 항목을 참고하되, MIDI 시각과 실제 스틱 접촉을 별도로 검증한다. 논문 내용 재사용 시 [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)에 따른 저자·원문·라이선스·변경 고지를 유지한다.

### R02. Multi-Modal Instrument Performances (MMIP)

- 원자료: [데이터셋](https://mmip.cs.ucy.ac.cy/), [공식 License.html](https://mmip.cs.ucy.ac.cy/License.html), [소개 논문 DOI: 10.1111/cgf.70025](https://doi.org/10.1111/cgf.70025), [기관 논문 보관 레코드](https://zenodo.org/records/15830081).
- **부분 검증:** 공식 License.html의 검색 색인에 저장소 콘텐츠가 **CC BY-NC-SA 4.0**라는 문구와 상업 라이선스 섹션이 있다. 그러나 이번 검토에서 직접 열기는 두 차례 타임아웃이었다. 실제 데이터 파일, 다운로드 동의, 개별 예외·추가 조건을 확인하지 못했다. 따라서 완전한 라이선스 감사로 표시하지 않는다.
- 자료 구분: Zenodo 레코드의 파일은 소개 **논문 PDF**이며 모션 데이터셋 자체의 배포·허락 증거로 대체하지 않는다.
- 활용 결정: MVP의 기본 모션, 잘라낸 스트로크 클립, 학습 데이터로 사용하지 않는다. 재검토 시 데이터 버전·파일과 약관을 확보하고, 필요한 상업 이용·학습·가중치 배포·출력 용도를 명확히 한다.
- 조건 참고: [CC BY-NC-SA 4.0 공식 설명](https://creativecommons.org/licenses/by-nc-sa/4.0/)은 출처 표시, 비상업 조건, 변경물의 동일조건 배포를 설명한다. 모델 가중치나 모든 생성 영상이 자동으로 같은 라이선스를 갖는다고 이 문서에서 단정하지 않는다.

### R03. Implementation of a 3D Virtual Drummer (2001)

- 저자: Martijn Kragtwijk, Anton Nijholt, Job Zwiers. Computer Animation and Simulation 2001, 출판본 15–26쪽. [DOI: 10.1007/978-3-7091-6240-8_2](https://doi.org/10.1007/978-3-7091-6240-8_2).
- 원자료: [University of Twente 서지·저자본 링크](https://research.utwente.nl/en/publications/implementation-of-a-3d-virtual-drummer/), [저자본 PDF, 10쪽](https://research.utwente.nl/files/83238062/Kragtwijk2001implementation.pdf), [Springer 출판 정보](https://link.springer.com/chapter/10.1007/978-3-7091-6240-8_2).
- 권리 증거: Springer 페이지의 Copyright information은 **© 2001 Springer-Verlag Wien**. 대학에서 저자본을 열람할 수 있다는 사실만으로 재배포 가능한 코드나 오픈 라이선스가 생기지는 않는다.
- 구현 참고: MIDI 이벤트 분류, 손 선호·교대, 준비/접촉/반동 키프레임, velocity·탄성에 따른 동작 크기를 다룬다.
- **정확성 주의:** 저자본 §4.2.3, PDF 8쪽은 최단경로 방식이 **아직 구현되지 않았음**을 명시한다. 우리 beam search/동적 계획법은 독립적인 신규 구현 항목이다. 동시 이벤트 삭제·변환 등의 원 논문 정책은 우리의 원본 이벤트 보존 정책에 그대로 적용하지 않는다.
- 확보 상태·활용: 이 서지 페이지와 논문에서 재사용 가능한 원저자 소스 배포물·코드 라이선스는 확인하지 못했다. 설명을 읽고 자체 요구사항에 맞춰 설계하며, 원 코드·그림·전문을 복사하지 않는다.

### R04. Generalized Audio-Driven Synthesis of Precise Drummer Motion (2026)

- 저자: Álvaro G. Iñesta, Mattia Ryffel, Amit H. Bermano, Robert W. Sumner, Martin Guay.
- 원자료: [Disney Research Studios 공식 발표, 2026-08-18](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/), [arXiv:2608.19055v1, 2026-08-19](https://arxiv.org/abs/2608.19055v1), [DOI](https://doi.org/10.48550/arXiv.2608.19055).
- 방법: 오디오 조건 확산 모델에서 몸의 자연스러운 구조와 스틱 정밀도를 별도 목적함수로 다룬다. 타격점 거리와 오디오–동작 상관을 제안한다. MIDI 입력 기반인 우리 엔진의 즉시 대체품으로 보지 않는다.
- 권리 증거: arXiv의 view license는 [비독점 배포 라이선스](https://arxiv.org/licenses/nonexclusive-distrib/1.0/license.html)로 연결된다. 이는 독자에게 코드·데이터의 포괄적 사용권을 주는 라이선스가 아니다. Disney 페이지는 권리를 저작권자에게 유보하고, 명시적 허락 없는 논문 재게시를 제한한다.
- 확보 상태·활용: 공식 페이지에서 PDF와 영상은 확인했다. 실행 코드·가중치·자체 데이터셋의 공개 다운로드와 이용허락은 확인하지 못했다. 평가 관점을 인용하고 원문을 링크한다. 발표의 정밀도 주장을 우리 구현의 달성 성능으로 사용하지 않는다.

### R05. MIDIAnimator

검토 기준은 대화에 나온 고정 커밋 [`059b1e2b80baf6171762f3d6a3c3f938d44ed46c`](https://github.com/jamesa08/MIDIAnimator/tree/059b1e2b80baf6171762f3d6a3c3f938d44ed46c)이다. 최신 브랜치 상태와 혼동하지 않는다.

| 증거 파일 | 확인한 내용 |
| --- | --- |
| [LICENSE.txt](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/LICENSE.txt) | GNU GPL version 3 본문 |
| [Python `__init__.py` 1–12행](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/MIDIAnimator/__init__.py#L1-L12) | GPL version 3 또는 이후 버전 고지 |
| [Rust Cargo.toml](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/Cargo.toml) | `license = "GPL-3.0"`, Tauri·Tokio·파일/네트워크 등 의존성 |
| [`fcurve.rs` 시작 부분](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/fcurve.rs#L1-L4) | Blender의 curve.cc·fcurve.cc·easing.cc에서 포팅했다는 출처 주석 |
| [`animation.rs` rvc 처리](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/animation.rs#L514-L557) | 이전 곡선의 rest 도달 시각까지 다음 키프레임 시각을 미루는 계산 |

Rust 구현은 실제 코드이며 MIDI 시각·velocity 기반 키프레임 생성과 겹침 처리가 존재한다. 다만 데스크톱·Blender 중심 구조이고, 이 검토에서 브라우저 빌드나 전신 드러머 기능을 검증하지 않았다. rvc가 접촉 키까지 지연시킬 수 있다는 판단은 해당 코드의 정적 분석이며 실행 검증 결과는 아니다.

**채택 결정:** 직접 코드·번역 포팅을 MVP에 넣지 않는다. Rust 엔진과 곡선은 별도 설계로 작성한다. 추후 편입 시 파일별 라이선스와 Blender 원출처까지 추적하고, GPL 배포·대응 소스 조건을 설계에 반영한다. 저장소 전체를 근거 없이 `GPL-3.0-only` 또는 `GPL-3.0-or-later` 하나로 단정하지 않는다. 별도 crate나 WASM으로 옮기는 것 자체를 라이선스 해소 근거로 삼지 않는다.

### R06. ARDrum-VirtualHand

- 검토 커밋: [`dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4`](https://github.com/wayne0419/ARDrum-VirtualHand/tree/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4).
- 출처·권리: [저장소 설명](https://github.com/wayne0419/ARDrum-VirtualHand), [해당 커밋 LICENSE](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/LICENSE)에서 **Apache License 2.0** 확인. MR.Drum/CHI 2025 연구의 Unity 기반 기록·학습 시스템이다.
- 코드 확인: [`TransformRecorder.cs`](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformRecorder.cs)는 세 대상의 transform·타임스탬프·타격을 기록한다. [`TransformPlayBacker.cs`](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformPlayBacker.cs)는 저장된 데이터 재생 구조다. [`HandMovementPathRenderer.cs`](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/HandMovementPathRenderer.cs)는 스틱 끝 기준 경로·주요 지점을 다룬다.
- 범위: 두 손·오른발 등의 기록 transform을 전신 모캡 정답이나 MIDI→전신 생성 모델로 취급하지 않는다. SDK·Unity 패키지·음원·모델·기록 데이터의 개별 권리는 이번 검토에서 모두 확인한 것이 아니다.
- 활용: 후속 모션 수집·디버그 도구의 선별 후보. 코드 편입 시 파일별 출처·수정 내용을 남기고, [Apache-2.0 §4](https://www.apache.org/licenses/LICENSE-2.0)의 라이선스 사본, 변경 표시, 관련 고지와 제공된 NOTICE 보존 조건을 적용한다. 저장소 전체를 일괄 복사하지 않는다.

### R07. Groove MIDI Dataset (GMD)

- 원자료·라이선스: [Google Magenta 공식 GMD 페이지](https://magenta.tensorflow.org/datasets/groove)의 License 섹션은 Google LLC 제공 **CC BY 4.0**를 명시한다. 실제 연주에서 얻은 MIDI와 정렬된 **합성 오디오**를 제공하며 전신 관절·손 배정 정답 데이터는 아니다.
- 특정 아티팩트: [`groove-v1.0.0-midionly.zip`](https://storage.googleapis.com/magentadata/datasets/groove/groove-v1.0.0-midionly.zip). 공식 게시 SHA-256: `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`.
- 확인 범위: 다운로드 URL과 공식 해시는 확인했지만 이번 단계에서 ZIP을 내려받아 로컬 해시·파일 내용 검사를 하지는 않았다. MIDI+WAV 전체 번들은 공식 페이지가 별도로 제공한다.
- 입력 주의: Roland TD-11의 일부 노트 매핑은 General MIDI와 다르며 CC4 하이햇 페달 정보가 있다. 무조건 GM 매핑만 적용해 실패를 엔진 오류로 판단하지 않는다.
- 채택 시: 공식 CC BY 출처 표기, 라이선스 링크, 원본 파일 경로, 사용한 split, SHA-256, 절단·변환 내역을 테스트 자료 목록에 기록한다. 움직임 자연스러움의 정답 검증에는 별도 관찰·측정이 필요하다.

## 원 대화의 인용 토큰을 대체하는 방법

원 대화에는 주소가 보존되지 않은 `chatgpt-content-reference` 번호가 있다. 번호는 답변마다 다시 시작하므로 전역 식별자로 사용하지 않는다. 아래는 **원 인용의 정확한 URL 복원이 아니라 동일 주장을 독립적으로 확인한 대체 근거**다. 라이브러리·웹 API의 근거는 별도의 기술·의존성 문서에서 관리한다.

| 답변 | 선행 연구 관련 원 인덱스 | 이번 검증 근거 |
| --- | --- | --- |
| 첫 답변: 구현 가능성·구조 | 1, 2 | R01 DRUMS |
| 첫 답변 | 3 | R03 Virtual Drummer |
| 첫 답변 | 4 | R05 MIDIAnimator |
| 첫 답변 | 5 | R04 Disney |
| 후속 답변: 규칙/학습·라이선스 | 1, 5 | R03 Virtual Drummer |
| 후속 답변 | 2, 3 | R01 DRUMS |
| 후속 답변 | 4, 10 | R02 MMIP; 논문과 데이터의 구분은 R01도 참고 |
| 후속 답변 | 6 | R04 Disney |
| 후속 답변 | 7 | R05 MIDIAnimator |
| 후속 답변 | 8, 13 | R06 ARDrum |
| 후속 답변 | 23 | R07 GMD |

## 장기 작업에서 유지할 구분

1. **검증됨과 미검증을 구분한다.** MMIP는 색인 확인에 머문다. DRUMS·Disney는 논문을 확인했지만 실행 가능한 모델 패키지를 확보한 것이 아니다.
2. **설계 목표와 연구 결과를 구분한다.** 1프레임 이내 타격 싱크, 접촉 오차 목표, 재현성은 우리 테스트로 입증해야 한다.
3. **아이디어 참고와 코드 편입을 구분한다.** 편입·번역·모션 절단을 했으면 원 파일·커밋·라이선스·수정 사항을 적는다. 이 문서에 링크를 추가한 것만으로 편입 승인이 되지는 않는다.
4. **논문·데이터·가중치·에셋별로 기록한다.** 후속 자료가 공개되면 기존 행을 조용히 덮어쓰지 말고 확인일, 새 버전, 증거 URL, 채택 결정과 관련 이슈를 갱신한다.
5. **공개 데모의 실제 배포물을 다시 확인한다.** 데이터셋 후보와 실제 포함 파일을 구분하고, 마지막 배포 전에 고정된 버전·라이선스 안내·출처를 일치시킨다.

현재 이 문서에는 외부 자료의 링크와 자체 요약만 포함했다. 외부 논문·코드·데이터·모델 파일은 배포물로 편입하지 않았다.


## Implementation comparison — 2026-10-24

The [rig and motion audit](rig-and-studio-audit.md) compares current code with R01/R03/R04 and VRM 1.0. It explicitly separates independent design ideas from model/code adoption. No DRUMS/Disney learned model, weights, or motion data have been integrated. Basic grip/contact work is tracked in #30 with #17/#18, ahead of advanced #20/#21 work.

## Ergonomics and sticking follow-up

The [follow-up assessment](ergonomics-and-sticking.md) records default kit/hardware
intersections, fitted versus unfitted body evidence, coordinated arm/torso proposals,
PAS/Vic Firth/Zildjian sticking instruction and general import guidance. It preserves
source credits and reuse boundaries without incorporating scores, recordings or
datasets. Its proposed completion criteria extend #14/#18/#20/#23; they are not a
claim that PR #40 implements full-body clearance or infers the original technique.
