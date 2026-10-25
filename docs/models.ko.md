# 캐릭터 모델 준비하기

[English](models.md) · **한국어** · [문서 안내](README.ko.md)

로컬 VRM 1.0 캐릭터를 불러오고, 이용 조건을 확인한 뒤 드럼 키트에 맞춥니다. 이 안내는 저장소 빌드를 설명하므로 공개 앱을 사용한다면 [버전별 제공 범위](../README.ko.md#어떤-버전을-사용하고-있나요)를 먼저 확인하세요. 외부 아바타는 배포물에 포함하지 않습니다.

## 지원 파일

| 항목 | 앱의 제한 |
| --- | --- |
| 캐릭터 형식 | 리소스를 포함한 VRM 1.0 단일 `.vrm`, 최대 50 MiB |
| 휴머노이드 | 필수 시맨틱 본 15개, 선택적인 손가락 본 수 표시 |
| 장면 | 최대 10,000노드, 올바른 본 참조·계층, 유한한 변환 값 |
| 이미지 | 내장 PNG/JPEG 최대 64개, 한 변 최대 4096픽셀, 전체 6400만 픽셀 |
| 리소스 | 버퍼·텍스처 내장, 외부 파일·URL 참조 불가 |

본 참조는 유효하고 중복되지 않아야 하며 계층에 순환이나 여러 부모가 있으면 안 됩니다. 이는 입력 허용 범위이며 최대 크기 모델이 모든 기기에서 원활하게 표시된다는 뜻은 아닙니다. 기술 상세는 [리그 입력 계약](../specs/architecture/rig-contract.ko.md)을 참고하세요.

VRM 0.x는 모델 조건이 허용하는 범위에서 VRM 1.0으로 변환하거나 다시 내보내야 합니다. 파일 이름만 바꿔서는 변환되지 않습니다. VRoid Studio의 편집 원본이나 [UniVRM 이전 안내](https://vrm.dev/en/univrm1/migrate_vrm0/)를 이용하세요. `.vroid`는 편집 원본이며 실행용 불러오기 형식이 아닙니다. 일반 캐릭터 GLB·FBX·OBJ·BLEND·GLTF 매핑은 지원하지 않습니다. 정적 드럼 GLB 교체는 최대 25 MiB인 [별도 작업](calibration.ko.md#받침대페달을-유지하며-드럼-몸체-교체하기)입니다.

## 불러오고 확인하기

1. **캐릭터**에서 **VRM 불러오기…**를 누르고 `.vrm` 하나를 고릅니다. 앱은 파일을 업로드하지 않고 브라우저에서 처리합니다.
2. 모델명·제작자·버전·필수 본·손가락 본 수와 **모델 이용 조건**을 읽습니다. 선택적인 권리 항목이 빠졌다면 미기재 상태와 VRM 1.0 명세의 기본값을 표시하며, 누락을 허용으로 해석하지 않습니다. 메타데이터 링크는 자동으로 열거나 조회하지 않습니다.
3. 데모 또는 MIDI를 열고 모델/뼈대의 개별 표시 옵션을 확인합니다. 어댑터는 팔다리 길이를 측정하고 기본 스켈레톤과 같은 시맨틱 본 계산을 사용합니다. 손가락 본이 있으면 그립 자세를 적용하며, 선택적인 손가락 본이 없어도 불러올 수 있습니다.
4. [좌석과 타점을 조절](calibration.ko.md)하고 재생·앞뒤 임의 탐색을 비교합니다. 실제 그립, 손에 붙은 스틱 끝, 발과 페달을 확인하세요. 접촉 계측은 도구·발 소켓 기준이며 모든 피부·장갑·신발 정점을 검사하지 않습니다.
5. **기본 리그**를 누르면 스켈레톤으로 돌아갑니다. 불러오기와 복원은 원본 타격 시각을 유지하면서 선택한 체형에 맞춰 현재 MIDI를 다시 계획합니다.

탐색과 출력의 재현성을 위해 매 자세 평가에서 스프링 상태를 초기화합니다. 불러오기 성공만으로 동적인 머리카락·의상 시뮬레이션, 충돌 없는 착석, 정밀 피부·발바닥 보정이나 모든 타점에서 자연스러운 손목이 검증되지는 않습니다. 모델·보정 선택을 다음 세션에도 유지하려면 [로컬 프로젝트](projects.ko.md)를 사용하고 같은 모델 파일을 다시 연결하세요.

## 모델을 구할 곳

| 출처 | 활용 방법 |
| --- | --- |
| [VRoid Studio](https://vroid.com/en/studio) | 캐릭터를 제작하고 VRM 1.0으로 내보냅니다. 통제된 체형 비교에 유용합니다. |
| [VRoid Hub](https://hub.vroid.com/en/) | 다운로드 가능한 모델을 찾고 제작자의 다운로드·동의 절차를 따릅니다. 파일별 버전과 조건을 확인합니다. |
| [VRM 공식 샘플](https://github.com/vrm-c/vrm-specification/tree/master/samples) | 문서화된 표준 기능을 테스트합니다. 모델별 고지와 내장 설정이 따로 있습니다. |
| [BOOTH](https://booth.pm/) | 제작자 에셋을 찾습니다. 포함 파일과 조건을 확인하세요. 무료라는 사실만으로 재배포 허용이나 호환성이 확인되지는 않습니다. |

확인했던 Hub의 [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604)와 [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889)는 VRM 0.0이었으며, CC0가 아닌 [별도 샘플 조건](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z)이 적용됩니다. 실제 내려받은 파일의 버전을 확인하세요.

개발용으로는 고정된 [ChatVRM AvatarSample_B](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm), [Twist 샘플](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample), [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san)이 VRM 1.0 테스트 후보입니다. 검사한 파일에는 필수 본과 양손 손가락 본이 있습니다. Twist에는 별도 VRM Public License 설정이 적용되며 three-vrm 코드의 MIT를 따르는 모델이 아닙니다. Seed-san은 크레딧 표시가 필요합니다. 정확한 파일 식별값·조건·검사 결과는 [모델 조사 기록](research/vrm-models.md)에 보존합니다.

## 편집 원본과 실행 파일 보관하기

체형 파라미터 편집용 `.vroid`와 실행용 `.vrm`을 함께 보관하세요. 내려받은 `.vrm`은 원래 편집 원본을 대신하지 않습니다. Studio의 dress-up 기능은 VRM 데이터를 열 수 있습니다. [공식 파일 형식](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio)과 [dress-up VRM 지원](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM)을 참고하세요.

체형 적합성을 비교할 통제된 두 모델은 다음처럼 준비합니다.

1. 편집이 허용된 원본에서 시작하고 의상·텍스처 조건을 보관합니다.
2. 의상은 비슷하게 유지하면서 키·팔다리 비율이 다른 작은 체형과 큰 체형의 `.vroid`를 저장합니다. 손목과 다리가 보이는 의상을 우선하고 체형 설정값·Studio 버전을 기록합니다.
3. [공식 내보내기 안내](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature)에 따라 모델·제작자 정보를 입력하고 둘 다 **VRM 1.0**으로 내보냅니다.
4. 본 축소 옵션을 사용했다면 내보낸 손·손가락 본을 확인합니다. 같은 MIDI·접촉·좌석 맞춤·임의 탐색·출력 프레임으로 두 모델을 비교합니다.

이는 문서화된 검증 절차이며 완료된 호환성 결과는 아닙니다.

## 사용 목적별 조건 확인하기

| 하려는 일 | 확인할 내용 |
| --- | --- |
| 로컬에서 불러와 평가 | 다운로드·사용 조건, 사용 가능한 사람, 추가 조건 |
| 렌더링한 영상 공개 | 아바타·영상 이용, 해당되는 경우 상업 이용, 표현 제한과 필요한 크레딧 |
| `.vrm`·텍스처·수정 모델 공유 | 별도의 재배포·수정 조건. 영상 이용 허용이 모델 재배포 허용을 뜻하지 않습니다. |

내장 조건과 배포 페이지를 함께 읽으세요. [VRM Public License](https://vrm.dev/licenses/1.0/)에는 모델별 설정이 적용되며, 의상·텍스처·프리셋에 추가 조건이 있을 수 있습니다. [VRoid 모델 이용 안내](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes)도 참고하세요. 프로젝트의 MIT는 아바타에 적용되지 않습니다. 내보내기는 모델 크레딧을 자동 삽입하지 않으므로 조건에 따라 공개 영상이나 함께 제공하는 출처 표기에 필요한 크레딧을 넣으세요.

확보한 모델은 저장소의 추적·공개 파일에 넣지 마세요. `.local/vrm-test-models/`는 Git에서 제외한 로컬 보관 위치이며 재배포 가능한 fixture 보관소가 아닙니다.

## 무엇을 검증했나요?

식별값을 고정한 Twist·Seed-san·ChatVRM AvatarSample_B 파일로 실제 로더·VRM 어댑터·WebGL 렌더러·Rust/WASM 평가기를 검사했습니다. 기록에는 배정된 타격과 도달 불가능한 타격, 스틱·발 소켓 접촉 측정값, 앞뒤 탐색 시 재현성이 있습니다. 모든 타격, 피부·의상의 겹침 해소나 이름이 같은 다른 파일까지 인증하는 결과는 아닙니다.

- [원본 모델 검사와 브라우저 측정](research/vrm-models.md)
- [앱 불러오기·재생·출력 검사](verification/vrm-studio-unreleased.md)
- [좌석·타점·공통 하드웨어 검사](verification/calibration-unreleased.md)
- [손바닥 방향 검사와 손목 한계](verification/palm-orientation-unreleased.md)

기록된 VRM 출력 실행은 인코딩·mux를 완료했지만 브라우저 제공자가 blob 다운로드를 반환하지 못해 별도 플레이어/ffprobe로 확인하지 못했습니다. 통제된 작은 체형/큰 체형 VRoid 쌍은 미검증입니다. 자신의 모델과 사용할 MIDI로 확인하세요. 형식 검사 통과만으로 연주·영상 호환성이 보장되지는 않습니다.

[체형과 키트 맞추기](calibration.ko.md) · [VRM 호환성 설계](../specs/architecture/rig-compatibility.ko.md)
