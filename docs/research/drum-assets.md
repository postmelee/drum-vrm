# 드럼 프리셋 모델·Logic Pro 화면 참고 조사

조사일: 2026-10-24 · 관련 실행 이슈: [#25](https://github.com/postmelee/drum-vrm/issues/25)

## 채택과 구현 경계

**Innerscene의 Five-piece drum kit를 실제 배포용 CC0 시각 프리셋으로 채택한다.** 원본 제작자가 공개한 전체 키트를 로그인 없이 GLB로 내려받았고, 개별 드럼·심벌·스탠드·페달의 이름이 분리되어 있어 악기별 교체가 가능하다. 원본 파일과 라이선스·정확한 출처는 [`public/models/`](../../public/models/README.md)에 보관한다. 앱의 로딩·장면 적용은 `src/model-library.ts`와 `src/scene.ts`의 책임이다. 파일을 편입한 사실과 브라우저에서 장면·접촉·내보내기를 검증한 사실은 별개이며, 후자는 검증 기록에 남긴다.

CC0 모델의 존재는 음향이나 캐릭터 호환성을 추가하지 않는다. 현재 앱은 MIDI 타격을 **자체 합성 드럼 음색**으로 재생하며, WAV 파일 재생·오프셋 정렬은 아직 없다. 3D 프리셋을 바꾸어도 같은 합성 PCM을 사용한다. 실제 녹음 음색이 필요한 경우 WAV 동기화 이슈 #13을 이어서 구현한다.

현재 캐릭터는 앱이 만드는 기본 관절 리그이며, 임의 캐릭터 파일을 자동 리깅하는 기능이 아니다. 장기 교체 목표는 VRM 1.0 휴머노이드와 이용 조건 확인, 좌석·팔다리·그립·페달 캘리브레이션이다. GLB라는 확장자만으로 humanoid/skeleton이 보장되지 않는다. 일반 OBJ/FBX/GLB나 VRM 파일을 모두 현재 지원한다고 안내하지 않는다. 사용자가 별도로 지정한 GLB 부품 가져오기는 정적 드럼 외형 교체와 캐릭터 리깅을 구분하여 구현해야 한다.

## 후보 비교와 실제 확인

| 후보·원본 링크 | 확인한 라이선스·다운로드 | 실제 파일 구조 | 판단 |
|---|---|---|---|
| [Innerscene Five-piece drum kit](https://www.innerscene.com/tools/library/3d-parts/five-piece-drum-kit-daff10f4) | 원 제작자 `Innerscene – original model`, CC0 1.0, 로그인 없는 GLB 공개 다운로드 | 3,361,144 bytes, 109,088 triangles, 322 meshes, 9 materials, 9개 내장 2048² PNG. 외부 URI·애니메이션 없음 | **채택.** 전체 어쿠스틱 키트와 하드웨어를 제공하며 mesh names로 독립 악기를 분리할 수 있음. 정적 메시 병합·texture 축소 필요 |
| [Grab3D Drum Kit](https://grab3d.com/models/music/drum-kit/) | [원본 LICENSE](https://grab3d.com/dl/drum-kit/LICENSE.txt)는 CC0 1.0. 공개 GLB·OBJ·blend 및 [manifest](https://grab3d.com/dl/drum-kit/manifest.json) | 내려받은 GLB 1,701,176 bytes, 24,972 triangles, **하나의 mesh**, 1 material, 3개 내장 JPEG. light 버전은 manifest상 12,486 triangles·1K textures | 비교용만 보관. 가볍지만 한 메시로 합쳐져 있어 부품 교체·타점별 배치를 바로 적용하기 어려움. 제작자는 절차적 Blender 모델링과 AI 보조 코드 사용을 밝힘 |
| [Zsky Drum set / Poly Pizza](https://poly.pizza/m/qWU9Q4flfQ) | 원 페이지와 embedded model metadata에서 **CC BY 3.0** 및 author Zsky 확인. 로그인 없는 GLB URL 확인 | GLB 206,220 bytes, 2,940 triangles, **하나의 mesh**, 6 flat materials, texture 없음. 검색 시 약 2.48k downloads | 비교용만 보관. 널리 내려받은 저용량 대안이지만 단순 외형·합쳐진 메시. 배포한다면 저작자·출처·CC BY 3.0 링크·변경 사항 표시 필요 |
| [Serg.Morgun Drum Kit / Sketchfab](https://sketchfab.com/3d-models/drum-kit-99218427e7a2424ea8afc9b8c668a5c7) | 검색 색인에는 CC Attribution과 약 38.7k triangles가 보이나 원 페이지 직접 접근은 403 | 다운로드와 정확한 license version·부품 구조 미검증 | **미채택.** 검색 요약만으로 재배포 권한·실제 파일을 확정하지 않음 |
| [Kenney 공식 카탈로그](https://kenney.nl/assets), [Quaternius 공식 카탈로그](https://quaternius.com/) | 공식 카탈로그 조사 | 이번 조사에서 요구에 맞는 완성된 어쿠스틱 드럼 키트의 원본 다운로드를 확인하지 못함 | 잘 알려진 제작자라는 이유만으로 존재하지 않는 모델이나 라이선스를 추정하지 않음 |

Poly Pizza·Grab3D의 실제 비교 다운로드는 `.local/drum-assets/`에만 두며 공개 배포물에 포함하지 않는다. 인기 수치·검색 순위는 품질이나 라이선스의 증거가 아니다. Innerscene 모델은 비교적 최근 등록된 자료(원 페이지 2026-10-16)로, 오래 검증된 업계 표준 모델이라고 표현하지 않는다.

## 채택 파일의 재현 가능한 출처

- 제작자/원본 기록: [Innerscene Five-piece drum kit](https://www.innerscene.com/tools/library/3d-parts/five-piece-drum-kit-daff10f4), record `five-piece-drum-kit-daff10f4`.
- 원 다운로드: [GLB endpoint](https://www.innerscene.com/api/library/five-piece-drum-kit-daff10f4/download). 응답을 2026-10-24 내려받아 아래 해시로 고정했다.
- SHA-256: `3cf8d568b604e99ce63ee82829d7f25044d996ed6cd42c39bc7f226919d1a5a7`.
- 라이선스 근거: 원 모델 페이지의 Public domain (CC0) 표기와 [제작자 라이브러리 약관](https://www.innerscene.com/tools/library/terms). 약관은 제출자가 권리를 보유하고 CC0로 제공함을 확인하도록 한다. CC0의 상표·특허 등 예외는 [Creative Commons 원문](https://creativecommons.org/publicdomain/zero/1.0/legalcode.txt)을 따른다.
- 공개 upstream tag/commit이 없으므로 임의의 commit SHA를 만들지 않는다. source record·조회일·byte 수·SHA-256을 버전 근거로 사용한다. 원 URL은 변경·삭제될 수 있으며, 편입한 실제 바이트와 고지는 저장소에 남긴다.
- 원본 GLB는 **변경하지 않았다.** glTF 2.0 / Blender glTF exporter v5.2.40, Y-up, 전체 경계 1.8142 × 1.2389 × 1.6177 m(XYZ), 323 nodes(322 meshes와 root), 모든 이미지와 buffer 내장. 네트워크 texture·외부 모델 참조 없음.

## 장면 적용과 유지보수

`model-library.ts`는 원본 이름을 엔진 바깥에서 canonical instrument로 매핑한다. `10 inch mounted tom`, `12 inch mounted tom`, `14 inch snare`, `16 inch floor tom`, `14 inch upper/lower hi-hat`, `16 inch crash`, `20 inch ride`, `22 inch bass drum`, 페달·스탠드 이름을 사용한다. 원본은 연주자가 +Z 쪽에 있으므로 Z를 반사하고 triangle winding을 뒤집어 앱의 +Y-up/+Z-forward 체계로 바꾼다. 좌우 배치는 유지한다.

각 악기를 material별로 병합하여 수백 개의 draw calls를 줄이고, 런타임 texture를 최대512²로 축소한다. 원본 9개 2048² RGBA texture는 압축 파일 크기와 달리 mipmap까지 약192MiB의 GPU 메모리를 쓸 수 있다. 512²로 축소한 9개 texture는 같은 계산으로 약12MiB다. 이 값은 RGBA8의 이론적 계산이며 실제 브라우저 peak memory 측정이 아니다.

실제 membrane·cymbal·footboard geometry에서 아래 방향 raycast로 접촉점을 얻는다. 심벌은 고정 너트 중심 대신 연주자 쪽 반경60% bow를 사용한다. `group.userData.contact`와 계획 target의 차이를 배치에 반영하되, `PerformancePlan`의 MIDI 시각이나 타격 위치를 외형 편의 때문에 몰래 바꾸지 않는다. 하이햇 페달은 별도 group이며 open/closed hi-hat는 같은 시각 모델을 공유한다. 원본 throne은 뼈대 중심의 화면을 유지하기 위해 표시하지 않는다.

현재 장면 어댑터는 바닥과 계획 타점을 함께 맞추기 위해 악기별 Y축 비율을 보정하고, 하이햇 페달은 별도로 계획된 발 타점에 정렬한다. 이 과정에서 원본 악기의 수직 비율이 조금 달라질 수 있다. 스탠드 높이·바닥 접촉·페달 연결은 별도로 육안 확인해야 한다. 모든 메시가 분리되어 있다는 사실이 자동 캘리브레이션·실제 기계 연결·정밀 심벌 변형을 보장하지 않는다. 다른 모델을 추가하면 모델별 타점·좌표·스케일과 저작권 기록을 새로 검증한다.

2026-10-24 `npm run typecheck`와 `npx vitest run tests/library.test.ts`를 실행해 통과했다. 회귀 검사는 실제 원본 GLB의 SHA-256·외부 URI 부재·9개 독립 그룹·반사 후 triangle winding·실제 접촉면 raycast·70개 미만의 병합 mesh를 확인한다. 브라우저 외부 단위 검사에서는 texture decode를 제외하므로 렌더 품질·실제 GPU 메모리·로딩 UX의 검증을 대신하지 않는다.

## Logic Pro를 참고하는 범위

Apple이 Logic Pro UI 소스 코드나 드럼 모델 파일을 자유롭게 재배포하도록 허용하는 공개 라이선스는 이번 조사에서 확인하지 못했다. [Apple 공식 Logic Pro SLA](https://www.apple.com/legal/sla/docs/LogicPro.pdf)는 소프트웨어·콘텐츠·인터페이스 등을 라이선스 대상으로 다루며, 특정 표시된 open-source components의 조건과 제품 전체의 권리를 구분한다. 이 PDF에는 Logic Pro X/10 표기가 있으므로 현재 설치 버전의 모든 계약을 대표한다고 단정하지 않는다. [Apple의 라이선스 안내](https://www.apple.com/uk/legal/sla/) 역시 구입·설치 시 라이선스가 게시본과 다를 수 있음을 알린다.

[Apple 공식 royalty-free 콘텐츠 안내(2025-03-03)](https://support.apple.com/en-us/101908)는 포함된 오디오 등의 원작 soundtrack 이용과 개별 콘텐츠의 독립 배포·재포장 배포를 구분한다. 이를 “Logic Pro 화면·모델을 웹 앱에 추출해서 재배포할 수 있다”는 허가로 읽지 않는다. 본 프로젝트에는 Apple UI 코드·아이콘·드럼 모델·샘플을 추출하거나 편입하지 않는다.

참고하는 것은 독립 구현 가능한 화면 구성 방향이다. 어두운 중립색, compact transport, 중앙 3D 뷰, 가벼운 inspector, 접을 수 있는 패널, 시간 눈금과 명확한 재생 상태를 자체 HTML/CSS/Three.js로 구현한다. 특정 제품과 동일한 자산을 사용한다고 표시하지 않으며, 실제 포함한 외부 자산은 위 CC0 모델로 명시한다.


## Additional candidate audit — 2026-10-24

[Additional preset candidates](drum-assets-followup.md) records Batoski/smoj GLB inspection and the unverified electronic-kit acquisition path. These are candidates for [#29](https://github.com/postmelee/drum-vrm/issues/29), not shipped presets. Public v0.2.0 assets are unchanged.
