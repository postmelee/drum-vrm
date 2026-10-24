> Historical source record. Preserved from [the audited baseline `66951cd6`](https://github.com/postmelee/drum-vrm/blob/66951cd69015e43cfe132a6ba61545b175a25412/docs/decisions/0003-minimal-studio-and-models.md).
> Original wording and recorded scope are retained; only relative link targets were relocated.
> Statements such as “current”, “unreleased” and approval instructions below apply to that record, not to new work.
> Use [adopted specifications](../README.md) for current contracts and policy.

# 0003 — 최소 작업 화면과 교체 가능한 드럼 외형

상태: 채택. 2026-10-24 사용자 UI 피드백, 이슈 #25.

## 결정

화면은 상단 재생 제어/네 메뉴, 중앙 3D 화면, 하단 피아노 롤로 구성한다. 캐릭터·드럼·업로드/연주정보·내보내기는 한 번에 하나의 우측 패널로 연다. 사용자 문장에는 “세 가지”라고 적혀 있지만 열거된 네 기능을 각각 유지했다. 기본 테마는 흰색이며 검정색 테마로 전환할 수 있다. 텍스트·기본 폼·얇은 경계선 외 장식은 제거한다.

지정 참고 사이트의 실제 computed font-family는 `-apple-system, system-ui, "Segoe UI", "Apple SD Gothic Neo", "Noto Sans KR", sans-serif`였다. 영문/한글 모두 이 스택을 그대로 사용한다. macOS의 시스템 글꼴과 한국어 fallback, 다른 OS의 해당 fallback을 사용하며 웹폰트를 별도 배포하지 않는다. 확인한 사이트: https://codex-usage-profile-stage5.meleeisdeveloping.chatgpt.site/ (2026-10-24).

기본 캐릭터는 관절과 뼈대, 스틱만 표시한다. 57-float canonical pose와 엔진의 고정 체형 계산은 유지한다. 실제 VRM/휴머노이드 파일의 스키닝과 리타게팅은 #17/#18이다. UI는 미지원 업로드를 가능하다고 표시하지 않는다.

드럼 설정 진입 시 350ms 카메라 이동으로 정면을 보고 캐릭터를 숨긴다. 닫으면 기존 카메라/캐릭터 표시를 복원한다. reduced-motion 환경은 이동을 즉시 완료한다. 클릭과 회전 드래그는 구분하며, 부품을 폼에서도 선택할 수 있다. 영상 출력은 전환을 완료하고 편집 강조를 숨긴다.

## 에셋 경계

자체 절차형 Studio/Vintage/Electronic와 CC0 Innerscene 키트를 프리셋으로 제공한다. CC0 원본 GLB·원문·출처·체크섬은 `public/models`에서 함께 관리한다. 자세한 비교와 채택 근거는 [드럼 에셋 조사](../../mydocs/tech/research/2026-10-24/drum-assets.md)를 따른다. Logic Pro 앱 자산이나 Apple 로고·텍스처는 포함하지 않는다.

외부 단일 GLB 2.0은 선택한 드럼 부품의 외형으로만 불러온다. 파일은 최대25MiB이며 외부 URL/파일 참조를 사전 검사하고 차단한다. 텍스처는 포함되어야 한다. GLTF/FBX/OBJ/BLEND나 Draco/KTX 전용 디코더에 의존하는 파일 지원은 약속하지 않는다. 사용자 파일을 서버로 전송하거나 자동 저장하지 않는다.

모델·색상·크기·회전은 시각적 변경이다. MIDI의 원본 시각, canonical 타점과 손발 계획, 합성 음향은 바꾸지 않는다. 임의 GLB는 bounding box로 자동 맞춤하므로 실제 표면과의 정밀 접촉은 보장하지 않는다. 프리셋 변경은 개별 외형을 초기화하며 새 MIDI 준비는 외형을 유지한다. 프로젝트 영속 저장은 #15이다.

## 피아노 롤

Rust가 PPQ·정수 기반 템포앵커·박자표를 계획에 추가한다. 원본 hit 시간은 변경하지 않는다. 1마디는 output 0.5초(시작 준비 뒤)에서 시작하며 템포 변경에 따라 실제 시간 간격이 달라진다. 박자 변경은 새 마디를 시작한다. 메타데이터 없는 이전 계획은 초 눈금을 사용한다. 음표 클릭/키보드 탐색·확대·스크롤을 제공하되 편집은 #14에 남긴다.
