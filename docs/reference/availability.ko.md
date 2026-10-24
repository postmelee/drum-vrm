# 기능 제공 범위

[English](availability.md) · **한국어**

독자 안내에 적용되는 기능 구분입니다. [공개 앱](https://postmelee.github.io/drum-vrm/)은 [v0.2.0](https://github.com/postmelee/drum-vrm/releases/tag/v0.2.0), 앱 commit `3e8be9333cedbc9845dbc3e4cebd73961af697b4`에 해당합니다. 소스 브랜치 변경만으로 배포된 앱이 바뀌지는 않습니다.

| 기능 | 공개 v0.2.0 | 이 문서가 설명하는 개발 빌드 |
|---|---|---|
| MIDI 준비·재생·탐색·합성 오디오·영상 출력 | 제공 | 제공 |
| 기본 스켈레톤·라이트/다크 화면·피아노 롤 | 제공 | 제공 |
| 정면 드럼 편집·로컬 정적 GLB 외형 교체 | 제공 | 제공 |
| 자체 드럼 스타일·Innerscene 키트 | 제공 | 제공 |
| Batoski·smoj 키트, Lucide 컨트롤, 연주자 시점, 선택형 타격 반응 | 이 버전에 없음 | 구현 |
| VRM 1.0 불러오기·실측 표준 휴머노이드 어댑터 | 이 버전에 없음 | 구현; PR #33 병합 |
| 좌석·타점 조절, 페달 위 발목 동작, 손바닥 방향, 고정 드럼 하드웨어 | 이 버전에 없음 | PR #34에 구현; 문서 기준점에서 미병합 |
| WAV 동기화·프로젝트 저장·수동 음표/손 편집 | 미제공 | 계획 |
| 완전한 heel-down/heel-up 프로필·충돌 없는 교차 연주·표현량 0–100 | 미제공 | 계획 |

개발 문서의 앱 기준 commit은 `66951cd69015e43cfe132a6ba61545b175a25412`입니다. 양언어 안내는 그 동작을 설명하며 새 릴리스를 알리는 문서가 아닙니다. 현재 검토 상태는 [PR #33](https://github.com/postmelee/drum-vrm/pull/33), [PR #34](https://github.com/postmelee/drum-vrm/pull/34), [이슈](https://github.com/postmelee/drum-vrm/issues)에서 확인합니다.

## 근거와 한계

- [공개 v0.2 검증](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/studio-0.2.md)은 실제 환경과 산출물 검사를 기록합니다.
- [VRM 통합 근거](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/vrm-studio-unreleased.md)와 [보정 근거](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/calibration-unreleased.md)는 접촉·탐색, 앱, 인코딩 검사를 구분합니다.
- [손바닥 방향 근거](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/verification/palm-orientation-unreleased.md)는 남은 동적 손목 한계를 보존합니다.

형식 검사를 통과해도 자연스러운 동작·피부 충돌 방지·모든 기기의 성능을 보장하지 않습니다. 통제된 짧은/긴 체형 VRoid 쌍은 아직 검사하지 않았습니다. 일부 VRM 영상은 인코딩을 마쳤지만 내려받은 파일의 독립 재생을 확인하지 못했으므로, 인코딩을 전체 재생 검증으로 간주하지 말고 해당 기록을 확인하세요.

[참조](README.ko.md)
