# Drum VRM

DAW에서 만든 드럼 MIDI를 3D 드러머의 연주 영상으로 바꾸는 브라우저 도구입니다. Rust/WASM이 MIDI와 연주 동작을 계산하고 Three.js가 확장 가능한 리그를 렌더링합니다.

**현재 상태: 계획 준비 완료 후 사용자 구현 승인 대기. 아직 실행 가능한 MVP나 배포 사이트는 없습니다.**

## 첫 MVP

`MIDI 업로드 → 사전 분석·연주 계획 → 기본 리그 표시 → 재생 → 영상 다운로드`

- 직접 만든 기본 휴머노이드, 스틱, 드럼 세트
- 킥·스네어·하이햇·탐·크래시·라이드, 오른손 주도 기본 스타일
- 템포 변경을 보존한 타격, 양손·양발 계획, 준비·접촉·반동
- 재생·일시정지·탐색, 합성 드럼 소리, 카메라 조절
- 720p30 고정 프레임 영상 출력, 환경에 따라 MP4 또는 WebM
- 사용자 MIDI는 브라우저에서 처리

WAV 동기화, VRM·동일 리그 캐릭터 교체, 편집, 프로젝트 저장, 고해상도·3D 출력은 후속 마일스톤입니다.

## 문서와 작업 현황

| 목적 | 문서 |
| --- | --- |
| 새 작업을 시작하기 | [인수인계와 현재 상태](docs/handoff.md), [작업 지침](AGENTS.md) |
| 범위와 완료 조건 | [요구사항](docs/product-requirements.md), [개발 계획](docs/plan.md) |
| 이번 승인 대상 | [구현·배포 승인](docs/approval.md) |
| 엔진과 리그 연결 | [아키텍처](docs/architecture.md), [설계 결정](docs/decisions/0001-engine-and-rig-boundary.md) |
| 원 대화의 맥락 | [요구사항 출처와 주제별 기록](docs/context/source-conversation.md) |
| 연구와 재사용 조건 | [연구·데이터](docs/research/references.md), [라이브러리](docs/research/dependencies.md), [라이선스 정책](docs/research/license-policy.md) |
| 실행 작업 | [GitHub Issues](https://github.com/postmelee/drum-vrm/issues), [Milestones](https://github.com/postmelee/drum-vrm/milestones) |

## 구현과 배포

사용자가 계획을 승인하면 M1·M2를 진행합니다. GitHub Pages에는 테스트를 통과한 정적 앱을 배포할 예정입니다. 예상 주소는 `https://postmelee.github.io/drum-vrm/`이며 현재 접속 가능한 서비스로 안내하지 않습니다.

계획 단계에는 빌드 명령·의존성·CI가 아직 없습니다. 구현 시 버전을 고정하고 설치·개발·검증 명령을 이 문서에 추가합니다.

## 라이선스

자체 소스는 MIT 채택을 제안하며 구현 승인 때 확정합니다. 라이브러리·데이터·모델의 라이선스는 각각 유지합니다. 현재 외부 코드·모션·모델·음원을 포함하지 않았습니다. 참고 연구의 공개 라이선스를 관련 데이터나 가중치의 사용 허가로 간주하지 않습니다.
