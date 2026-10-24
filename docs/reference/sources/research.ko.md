# 드러머 동작 연구·도구·데이터

[English](research.md) · **한국어**

**2026-10-24** 출처 감사의 R01–R07을 보존합니다. 여기에 인용했다는 이유만으로 학습 드러머 모델·연구 모션 데이터·코드를 편입한 것은 아닙니다. 전체 원 감사는 [역사적 연구 기록](../../../mydocs/tech/research/2026-10-24/references.md)에 남깁니다.

## R01 — DRUMS: Drummer Reconstruction Using Midi Sequences

**Theodoros Kyriakou, Panayiotis Charalambous, Andreas Aristidou**, MIG 2025, 7쪽. [DOI](https://doi.org/10.1145/3769047.3769066), [저자 프로젝트](https://www.theodoroskyriakou.com/projects/drums/index.html), [저자 PDF](https://www.theodoroskyriakou.com/projects/drums/Paper.pdf).

PDF는 논문에 CC BY 4.0을 명시합니다. 데이터·가중치 허락의 근거는 아닙니다. BiLSTM 손 궤적, 상체/표정 검색, 절차적 발 동작과 IK를 결합하며 MMIP 드럼 데이터 약 35분을 사용합니다(§3.1). §5는 복잡한 필인·미학습 패턴·고정 키트 의존성을 지적하며 드럼 지오메트리가 없어 표면 접촉 정확도를 평가하지 못했다고 설명합니다.

모듈 분리와 평가의 한계를 참고합니다. 속도/MIDI 정렬은 실제 스틱 접촉 오차와 다릅니다. 검사한 프로젝트 페이지에서 논문·DOI·영상 링크를 확인했으나 실행 코드·가중치 다운로드는 확정하지 못했습니다. [CC BY 4.0 조건](https://creativecommons.org/licenses/by/4.0/).

## R02 — Multi-Modal Instrument Performances (MMIP)

[공식 데이터셋](https://mmip.cs.ucy.ac.cy/), [라이선스 페이지](https://mmip.cs.ucy.ac.cy/License.html), [논문 DOI](https://doi.org/10.1111/cgf.70025), [기관 논문 레코드](https://zenodo.org/records/15830081).

권리는 부분 검증입니다. 공식 라이선스 페이지의 검색 색인에서 CC BY-NC-SA 4.0과 상업 라이선스 절을 확인했지만 직접 접근은 시간 초과였습니다. 다운로드 동의·실제 데이터·예외는 검사하지 못했습니다. Zenodo 논문 PDF는 데이터셋 자체나 이용 허락의 증거가 아닙니다.

MMIP 모션 클립·학습 데이터는 채택하지 않았습니다. 후속 사용에는 정확한 파일·버전과 학습·배포·출력 목적별 조건 확인이 필요합니다. 모든 가중치·영상에 같은 라이선스가 자동 적용된다고 추정하지 않습니다. [CC BY-NC-SA 4.0 설명](https://creativecommons.org/licenses/by-nc-sa/4.0/).

## R03 — Implementation of a 3D Virtual Drummer

**Martijn Kragtwijk, Anton Nijholt, Job Zwiers**, Computer Animation and Simulation 2001, 15–26쪽. [DOI](https://doi.org/10.1007/978-3-7091-6240-8_2), [University of Twente 레코드](https://research.utwente.nl/en/publications/implementation-of-a-3d-virtual-drummer/), [저자 PDF](https://research.utwente.nl/files/83238062/Kragtwijk2001implementation.pdf), [출판사](https://link.springer.com/chapter/10.1007/978-3-7091-6240-8_2).

출판 권리는 © 2001 Springer-Verlag Wien입니다. 저자본 열람이 재사용 가능한 소스 코드나 공개 라이선스를 의미하지는 않습니다. 이벤트 분류·손 선호/교대·준비/접촉/반동·velocity와 탄성에 따른 동작을 다룹니다.

저자 PDF §4.2.3(PDF 8쪽)은 최단경로 방식이 아직 구현되지 않았다고 명시합니다. 프로젝트의 beam 계획기는 독립 구현이며 논문의 이벤트 삭제/변환 정책을 원본 이벤트 보존 대신 적용하지 않습니다. 재사용 가능한 저자 소스 패키지·라이선스는 확인하지 못했습니다.

## R04 — Generalized Audio-Driven Synthesis of Precise Drummer Motion

**Álvaro G. Iñesta, Mattia Ryffel, Amit H. Bermano, Robert W. Sumner, Martin Guay**. [Disney Research Studios 발표, 2026-08-18](https://studios.disneyresearch.com/2026/08/18/generalized-audio-driven-synthesis-of-precise-drummer-motion/), [arXiv 2608.19055v1, 2026-08-19](https://arxiv.org/abs/2608.19055v1), [DOI](https://doi.org/10.48550/arXiv.2608.19055).

오디오 조건 확산 모델에서 신체 자연스러움과 스틱 정밀도를 분리하고 타점 거리·오디오-동작 측정을 다룹니다. MIDI 엔진의 즉시 대체품은 아닙니다. arXiv는 [비독점 배포 라이선스](https://arxiv.org/licenses/nonexclusive-distrib/1.0/license.html)에 연결되며 Disney는 허락 없는 논문 재게시를 제한합니다. 코드·데이터·가중치의 포괄적 사용권은 아닙니다.

감사에서는 논문·영상 링크를 확인했으며 허가된 실행 파일·가중치·데이터셋 패키지를 확인한 것은 아닙니다. 평가 관점을 참고하되 논문이 보고한 정밀도를 앱의 결과로 주장하지 않습니다.

## R05 — MIDIAnimator

검토 commit: [059b1e2b80baf6171762f3d6a3c3f938d44ed46c](https://github.com/jamesa08/MIDIAnimator/tree/059b1e2b80baf6171762f3d6a3c3f938d44ed46c).

- [LICENSE.txt](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/LICENSE.txt): GPL version 3 원문.
- [Python 고지](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/MIDIAnimator/__init__.py#L1-L12): version 3 또는 이후 버전.
- [Tauri manifest](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/Cargo.toml): `GPL-3.0` 선언. [Blender 유래 곡선 고지](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/fcurve.rs#L1-L4), [겹침 계산](https://github.com/jamesa08/MIDIAnimator/blob/059b1e2b80baf6171762f3d6a3c3f938d44ed46c/rust-impl/motionkeys/src-tauri/src/utils/animation.rs#L514-L557).

검사한 Rust 코드는 실제로 존재하며 MIDI/velocity 키프레임·겹침을 처리하지만 데스크톱/Blender 구조의 브라우저·전신 동작을 검증한 것은 아닙니다. rest-value 계산이 접촉 키를 늦출 수 있다는 판단은 정적 분석이며 실행 벤치마크가 아닙니다. 직접 복사·번역 포팅은 채택하지 않았습니다. 저장소 전체를 GPL-only/or-later 하나로 단정하거나 WASM으로 옮기면 라이선스가 해결된다고 보지 않습니다.

## R06 — ARDrum-VirtualHand

검토 commit: [dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4](https://github.com/wayne0419/ARDrum-VirtualHand/tree/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4). 저장소 [Apache-2.0 라이선스](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/LICENSE).

MR.Drum/CHI 2025와 관련된 Unity 기록·학습 시스템입니다. [TransformRecorder](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformRecorder.cs), [TransformPlayBacker](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/TransformPlayBacker.cs), [HandMovementPathRenderer](https://github.com/wayne0419/ARDrum-VirtualHand/blob/dafb6b1a7ab0cc1ddc7301ac1d846b9c159cfac4/Assets/HandMovementPathRenderer.cs)를 확인했습니다.

손·발의 추적 transform은 전신 정답 데이터나 MIDI→신체 모델이 아닙니다. SDK·Unity 패키지·모델·음원·녹음 전체를 감사하지는 않았습니다. 기록·디버그 도구 후보이며 편입된 코드가 아닙니다. 채택 시 [Apache-2.0 §4](https://www.apache.org/licenses/LICENSE-2.0)에 따라 파일별 출처·변경·라이선스·제공된 NOTICE를 보존해야 합니다.

## R07 — Groove MIDI Dataset (GMD)

[Google Magenta 원출처](https://magenta.tensorflow.org/datasets/groove)는 Google LLC 데이터에 CC BY 4.0을 명시합니다. 실제 연주의 MIDI와 정렬된 합성 오디오를 제공하며 전신 관절·손 배정 정답은 아닙니다.

자료: [groove-v1.0.0-midionly.zip](https://storage.googleapis.com/magentadata/datasets/groove/groove-v1.0.0-midionly.zip). 게시 SHA-256: `651cbc524ffb891be1a3e46d89dc82a1cecb09a57c748c7b45b844c4841dcc1e`. 감사에서는 공식 URL·해시를 확인했으나 ZIP을 내려받아 내용을 확인하지는 않았습니다. MIDI+WAV 번들은 별도입니다.

Roland TD-11 음표 매핑은 GM과 다를 수 있고 CC4 하이햇 정보가 있습니다. 공개 데모는 독립 생성물이며 GMD는 후속 테스트 후보입니다. 채택 시 파일별 출처·split·해시·변환 기록이 필요합니다. 자연스러운 모션의 정답 데이터는 아닙니다.

[출처](README.ko.md)
