> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/decisions/0001-engine-and-rig-boundary.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../README.md) for current contracts and policy.

# 0001 연주 의도와 렌더링 리그 분리

상태: 채택, 2026-10-24 사용자 승인 후 MVP에 적용. 통합·배포 인수 검증 진행 중.

## 배경

목표는 DAW에서 만든 드럼 MIDI를 캐릭터가 연주하는 영상으로 바꾸는 것이다. MVP는 기본 리그만으로 충분하지만, 나중에 캐릭터와 체형을 바꿔도 같은 연주 의도를 유지해야 한다. 본 회전만 원본으로 저장하면 새 팔 길이와 드럼 배치에 맞춘 접촉 보정이 어렵다.

## 결정

Rust 코어는 원본 tick, 곡 시간, 악기·주법, 손발 배정, 타점과 진단을 보존하는 버전 있는 연주 계획을 만든다. 주요 동작은 임의의 시간으로 평가한다. Three.js는 이 결과를 리그 어댑터로 적용하고 화면을 그린다.

첫 어댑터는 직접 만든 기본 휴머노이드와 드럼 지오메트리다. 향후 VRM 어댑터는 별도 계층으로 추가하고 좌석·그립·체형을 재보정한다. MIDI 타격 시각은 자연스러움을 위해 이동하지 않는다. 도달하지 못하는 타격은 진단으로 노출한다.

학습 모델은 향후 후보 생성·표현 보조에만 도입 가능하며, 타격 시각과 접촉 제약은 코어에 유지한다.

## 결과

스크러빙과 고정 FPS 내보내기는 동일한 평가기를 사용한다. 엔진을 CLI나 다른 렌더러로 재사용할 수 있다. 대신 좌표계·단위·본 의미·스틱 그립과 체형 보정 계약을 먼저 문서화해야 한다. 세부 계약은 [아키텍처](../../mydocs/report/history/baseline-66951cd/architecture.md)를 따른다.

## MVP 구현과 후속 경계

Rust `PerformancePlan` schema1은 원본 event index·tick·정수 μs·손발·타점·진단을 보존하며 `evaluate_at()`은 세계 좌표57개의 소유 Float32Array를 반환한다. `PrimitiveRigAdapter`가 이를 몸통·머리·양손·양발·스틱에 적용한다. 포즈는 local bone quaternion이 아니며 매 프레임 계획 JSON을 다시 파싱하지 않는다.

기본 engine calibration은 `primitive-v1`으로 고정이다. 키트/리그 adapter를 데이터 경계로 분리했지만 VRM normalized pose와 임의 체형 자동 재보정·calibration hash 캐시는 아직 구현하지 않았다. 모델 교체 시 재계획이 필요하다는 원칙은 #17/#18에 유지한다. 변경된 kit 데이터, 스틱 끝의 실제 메시 접촉, 임의 seek 후 같은 포즈를 검사하는 계약 테스트를 제공한다. 테스트 통과와 전체 MVP 검증 결과는 별도 완료 기록으로 남긴다.
