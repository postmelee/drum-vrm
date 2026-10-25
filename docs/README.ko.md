# 문서 안내

[English](README.md) · **한국어** · [Drum VRM](../README.ko.md)

[앱 사용 흐름과 로컬 실행](../README.ko.md#사용-방법)부터 시작하세요. 안내는 저장소 빌드를 설명하며, 공개 앱과의 차이는 [버전 표](../README.ko.md#어떤-버전을-사용하고-있나요)에서 확인할 수 있습니다.

## 사용과 꾸미기

| 목적 | 안내 |
| --- | --- |
| 모델을 구하고 이용 조건을 확인한 뒤 VRM 불러오기 | [캐릭터 모델](models.ko.md) |
| 좌석·손·발을 맞추고 드럼 부품 교체하기 | [보정과 드럼 부품](calibration.ko.md) |
| 설정을 저장하고 원본 파일 다시 연결하기 | [로컬 프로젝트](projects.ko.md) |
| 사용자 WAV를 맞추고 영상 출력하기 | [WAV와 영상 출력](export.ko.md) |
| 교육 자료와 저자·출처 이해하기 | [연주 표현의 원칙](performance-foundations.ko.md) |

## 설계 살펴보기

[기술 설계 안내](../specs/README.ko.md)에서 요구사항과 구현 경계를 설명합니다. [전체 구조](../specs/architecture/overview.ko.md), [체형·키트 보정](../specs/architecture/calibration.ko.md), [VRM 호환성](../specs/architecture/rig-compatibility.ko.md), [연주 표현](../specs/architecture/performance-expression.ko.md), [의존성 선택](../specs/architecture/dependencies.ko.md)이 좋은 출발점입니다.

[연구·재사용 대장](research/references.md)은 애니메이션 논문·도구·데이터를 정리합니다. [VRM 모델](research/vrm-models.md), [드럼 에셋](research/drum-assets.md), [후속 에셋 조사](research/drum-assets-followup.md)는 출처별 조사 결과와 이용 조건을 보존합니다. [설계 결정](decisions/README.md)에는 채택한 선택의 근거를 남깁니다.

## 검증

검증 기록은 특정 환경과 리비전에서의 결과이며 모든 환경 지원이나 새 릴리스를 뜻하지 않습니다.

| 범위 | 근거 |
| --- | --- |
| 공개 MIDI→영상 흐름 | [v0.2 스튜디오 검사](verification/studio-0.2.md) |
| VRM 앱과 출력 경로 | [VRM 검사](verification/vrm-studio-unreleased.md) |
| 좌석·타점·공통 하드웨어 | [보정 검사](verification/calibration-unreleased.md) |
| 손바닥 방향과 남은 손목 한계 | [손바닥 검사](verification/palm-orientation-unreleased.md) |

에셋 조건은 [포함 모델 출처](../public/models/README.ko.md)와 [배포 고지](../public/licenses.txt)를 참고하세요. 앱 자체 코드는 [MIT 라이선스](../LICENSE)를 사용합니다.
