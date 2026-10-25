# 아키텍처

[English](README.md) · **한국어**

[개요](overview.ko.md)를 먼저 읽고, 변경하려는 경계의 계약을 확인합니다.

| 계약 | 책임 |
| --- | --- |
| [MIDI 해석](midi-interpretation.ko.md) | 원본 이벤트 근거, 매핑 프로필, 연주 의도와 제어의 경계 |
| [엔진](engine-contract.ko.md) | 입력 제한, 시간, 손발 배정과 프로필·포즈 API |
| [리그](rig-contract.ko.md) | 의미 기반 본, 좌표 변환, 보정, 렌더링된 접촉과 드럼 외형 |
| [오디오·출력](audio-export-contract.ko.md) | 곡·출력 시각, PCM, 재생 시계와 인코딩 |

상세 문서는 [체형·도구·키트 보정](calibration.ko.md),
[휴머노이드 호환성](rig-compatibility.ko.md), [주법과 표현](performance-expression.ko.md),
[의존성 선택](dependencies.ko.md)의 이유와 한계를 설명합니다.

이 계약은 소스의 동작을 설명하며 배포나 새 검증 결과를 뜻하지 않습니다.
[기존 결정](../../docs/decisions/)에는 설계 이유와 당시의 범위가 보존되어 있습니다.

[기술 설계](../README.ko.md)
