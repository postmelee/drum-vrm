# 기술 설계

[English](README.md) · **한국어**

이 문서들은 현재 체크아웃한 소스의 요구사항, 구성 요소의 경계, 좌표·시간 계약과
설계 이유를 설명합니다. 향후 확장은 구현된 동작과 구분하여 표시합니다.
공개 앱과 빠른 시작은 [프로젝트 README](../README.ko.md)에서 확인할 수 있습니다.

| 주제 | 문서 |
| --- | --- |
| 목적과 불변 조건 | [제품 요구사항](requirements.ko.md) |
| 구성 요소와 데이터 흐름 | [아키텍처 개요](architecture/overview.ko.md) |
| 간결한 메뉴와 통일된 피드백 | [알림 설계](architecture/notifications.ko.md) |
| 원본 이벤트·매핑·연주 의도 | [MIDI 해석](architecture/midi-interpretation.ko.md) |
| 로컬 저장·재열기와 원본 재연결 | [프로젝트 형식](architecture/project-format.ko.md) |
| 시간·손발 배정·포즈 인터페이스 | [엔진 계약](architecture/engine-contract.ko.md) |
| 휴머노이드·좌표·드럼 하드웨어 | [리그 계약](architecture/rig-contract.ko.md) |
| 오디오 시계와 영상 프레임 | [오디오·출력 계약](architecture/audio-export-contract.ko.md) |
| 체형·좌석·그립·타점 맞추기 | [보정](architecture/calibration.ko.md) |
| 악기 본체·하드웨어·주법별 영역 | [장비와 접촉](architecture/equipment-contact.ko.md) |
| 표준 리그 지원과 측정 범위 | [리그 호환성](architecture/rig-compatibility.ko.md) |
| 주법과 향후 동작 표현 | [연주 표현](architecture/performance-expression.ko.md) |
| 실행 환경 구성과 의존성 선택 | [의존성 설계](architecture/dependencies.ko.md) |

[아키텍처 목차](architecture/README.ko.md)는 계약과 상세 설명을 나누어 안내합니다.
기존 결정은 ID와 당시의 범위를 유지한 채 [docs/decisions](../docs/decisions/)에 있습니다.
[연구 출처](../docs/research/references.md)와 [검증 기록](../docs/verification/)에는
설계 판단의 근거가 보존되어 있습니다.
