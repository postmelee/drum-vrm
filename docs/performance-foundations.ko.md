# 연주 표현의 원칙과 참고자료

[English](performance-foundations.md) · **한국어** · [문서 안내](README.ko.md)

드럼 교육 자료와 애니메이션 연구를 함께 참고하고, 프로젝트의 동작 규칙을 독립적으로 설계·검증합니다. 연주는 음악을 보존하고 연주자의 체형에 맞아야 하며, 선택한 주법이 동작에 드러나야 합니다.

[연주 표현 설계](../specs/architecture/performance-expression.ko.md)는 자세·그립·heel-down/heel-up·협응을 앞으로의 주법 프로필과 시각적 표현량 0–100 조절에 연결합니다. 이 조절은 지원 설정이 아닌 **계획**입니다. 현재 페달 위 발목 동작은 근사 구현이며, 사용 가능한 조절과 한계는 [보정 안내](calibration.ko.md)에서 설명합니다.

## 연구 원칙

- 원본 MIDI의 음표·시각·velocity를 보존합니다. 부가적인 시각적 강조에도 접촉·회복할 시간과 공간이 남아야 합니다.
- 실측 체형과 같은 맞춤 절차를 모든 아바타에 적용합니다. 주법 선택과 표현량은 별개입니다.
- 원자료의 교육 내용, 프로젝트의 해석, 구현된 동작과 측정 근거를 구분합니다. 교육 예시를 모든 체형의 관절 한계 측정값으로 간주하지 않습니다.

상세 규칙·완료 조건·설계 결정은 [기술 설계](../specs/architecture/performance-expression.ko.md)에 두며, 이 문서는 원자료와 저자·출처를 찾는 입구입니다.

## 연주 교육 자료 대장

자료 ID는 유지합니다. 확인일: 2026-10-24. 링크와 프로젝트 요약만 기록하며 원본 파일·사진·악보·녹음은 포함하지 않습니다.

| ID | 저자·발행·위치 | 연구에 활용할 부분과 범위 |
| --- | --- | --- |
| I01 | **Keiichi Nagata**, Yamaha, [DTX402 Series — Drum Score / Instruction](https://data.yamaha.com/files/download/other_assets/7/1224887/DTX_DrumScore_Instruction_eng.pdf). © 2018 Yamaha Corporation. 기본 설명: 인쇄 쪽수 4–7쪽, 저자 소개 51쪽, 저작권 표기 PDF 마지막 52쪽. | 준비 자세·그립·페달 주법·협응의 기초 자료입니다. 46쪽의 하이햇 구간처럼 주법 안내는 문맥에 따라 해석합니다. 이 자료가 프로젝트의 동작 계산이나 표현량 슬라이더를 정의한 것은 아닙니다. |
| I02 | **Riley Warren**, Yamaha Music, [A Guide to Proper Stick Grips](https://hub.yamaha.com/music-educators/instruments/perc/a-guide-to-proper-stick-grips/), 2025-12-19. Fulcrum·Matched·French·Traditional 절. | 그립 종류·손가락 배치·손목과 팔의 역할을 비교합니다. 양손 손바닥을 위로 돌리는 설정은 완성된 전통 그립이 아닙니다. |
| I03 | **Dr. Dave Gerhart**, Yamaha Music, [Fix It: Tips for Teaching Snare Drum Basics](https://hub.yamaha.com/music-educators/instruments/perc/snare-drum-basics-pedagogy/), 2021-11-09. Drum Height·Fulcrum·Matched Grip·Foundation 절. | 편안한 연주 자세와 악기 배치의 관계, 기본 타격 설명을 참고합니다. 교육용 예시를 실측 관절 한계 데이터로 간주하지 않습니다. |

I01에는 위 저작권 표기가 있고 I02/I03에도 Yamaha 저작권 고지가 있습니다. 이 교육 자료들에 적용되는 공개 콘텐츠 라이선스는 확인하지 못했습니다. 프로젝트 자체 구현은 MIT이며, 출처 표기가 원자료에 MIT를 적용하거나 저자·Yamaha의 공식 보증을 의미하지 않습니다.

## 연결된 참고자료

기존 [연구·재사용 대장](research/references.md)은 **DRUMS(R01), MMIP(R02), Implementation of a 3D Virtual Drummer(R03), Disney 드러머 모션 연구(R04), MIDIAnimator(R05), ARDrum(R06), Groove MIDI Dataset(R07)**의 저자·정확한 링크·권리 근거·채택 상태를 보존합니다. 연주 교육, 생성 방법, 도구, 데이터는 서로 다른 근거입니다. 외부 학습 모션 모델을 현재 구현에 편입한 것은 아닙니다.

표준·모델·현재 부족한 부분은 [리그/모션 감사](research/rig-and-studio-audit.md), [VRM 모델 대장](research/vrm-models.md), [드럼 에셋 대장](research/drum-assets.md), [스네어/키트 적합성 분석](research/snare-ready-and-kit-fit.md)에 연결합니다. 코드·논문·데이터·가중치·에셋의 출처와 이용 조건을 각각 관리합니다.

[기술 설계](../specs/architecture/performance-expression.ko.md) · [캐릭터 모델](models.ko.md) · [보정 안내](calibration.ko.md)
