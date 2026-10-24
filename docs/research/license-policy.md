> Research record preserved from [baseline 66951cd](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/research/license-policy.md). Dated findings and statements of implementation status below belong to that record; use the [current specifications](../../specs/README.md) and [model guide](../models.md) for current behavior.

# 라이선스와 출처 관리

기준일: 2026-10-24. 현재 저장소에는 계획 문서만 있으며 외부 소스·모델·모션·데이터를 포함하지 않는다. 자체 코드에는 MIT 채택을 제안하며, 실제 라이선스 파일 추가는 구현 승인 때 확정한다.

## 채택 기준

연구를 읽고 알고리즘을 독립 구현하는 것, 소스를 복사·포팅하는 것, 모션·에셋을 배포하는 것은 별도 기록한다. 논문의 공개 라이선스를 코드·가중치·데이터에 자동 적용하지 않는다. 저장소가 공개되어 있다는 이유만으로 재사용 허가를 추정하지 않는다.

| 구분 | 첫 MVP 방침 |
|---|---|
| 자체 리그·키트·드럼 소리 | 코드로 직접 생성하고 제작 출처를 기록 |
| Rust 및 Three.js 기반 라이브러리 | 실제 채택 버전 고정, LICENSE와 관련 NOTICE 유지 |
| Mediabunny | MPL 대상 버전에 대응하는 소스 취득 링크, 고지, 수정 내역 제공 |
| MIDIAnimator | 기본 의존성으로 가져오거나 코드를 포팅하지 않음. GPL 채택은 별도 결정 |
| MMIP 및 조건 미확인 모션·가중치 | 기본 데이터로 포함하지 않음 |
| Groove MIDI Dataset | 후속 평가 후보. 사용 시 원본 파일 출처·CC BY 4.0 표시·변경 사항 기록 |
| 사용자 MIDI/WAV/VRM | 로컬 처리. 사용자 파일을 저장소·서버·분석 서비스에 전송하지 않음 |

## 채택할 때 남길 기록

이름, 역할, 버전 또는 commit SHA, 공식 저장소, 해당 버전의 LICENSE/NOTICE URL, 실제 포함 파일, 변경 내역, 배포되는 소스 취득 방법을 기록한다. 소프트웨어와 샘플 파일·3D 모델·오디오의 조건을 분리한다. VRM은 모델의 메타데이터와 추가 조건도 확인한다.

`docs/research/dependencies.md`는 후보 라이브러리의 조사 대장이고 `docs/research/references.md`는 연구·데이터 대장이다. 도입 시 `THIRD_PARTY_NOTICES.md`와 라이선스 사본을 추가한다. 아직 설치하지 않은 후보를 배포 의존성이라고 표시하지 않는다.

## 릴리스 때 확인

- lockfile과 실제 번들·WASM 의존성 목록이 고지와 일치하는지 확인한다.
- 정확한 버전의 원문 라이선스와 필요한 attribution/NOTICE를 포함한다.
- 수정된 MPL 파일이 있다면 그 소스와 변경 내역을 제공한다.
- 자체 fixture와 외부 테스트 데이터의 출처를 구분한다.
- 연구의 라이선스·배포 링크가 바뀌었으면 날짜와 근거를 갱신한다.

이 문서는 프로젝트의 채택 정책이며 개별 데이터·모델·배포물에 대한 포괄적 법률 판단을 대신하지 않는다. 미확인 조건은 배포 전 해당 권리자 자료로 확인한다.
