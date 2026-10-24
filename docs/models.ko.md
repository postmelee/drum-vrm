# 캐릭터 모델 준비하기

[English](models.md) · [README](../README.ko.md) · [엔지니어링 근거](research/vrm-models.md)

확인일: **2026-10-24**. #17/#18/#30의 VRM 작업을 위한 준비 안내입니다. 공개 v0.2.0 앱은 아직 자체 스켈레톤을 사용하며 **캐릭터 VRM 파일을 불러오지 못합니다.** 아래 샘플은 로컬에서 메타데이터를 점검했으며 Drum VRM에서의 연주·영상 출력 호환성은 아직 테스트하지 않았습니다. 이 프로젝트에는 외부 아바타를 포함하지 않습니다.

## 모델을 구할 곳

| 출처 | 활용 방법 |
|---|---|
| [VRoid Studio](https://vroid.com/en/studio) | 편집 가능한 캐릭터를 만들고 필요한 VRM 버전으로 내보냅니다. 같은 기준에서 체형 두 개를 준비하기에 적합합니다. |
| [VRoid Hub](https://hub.vroid.com/en/) | 모델을 찾고 제작자가 제공한 다운로드·동의 절차를 이용합니다. 모델별 형식과 조건을 확인합니다. |
| [VRM 공식 샘플](https://github.com/vrm-c/vrm-specification/tree/master/samples) | 개발용 공식 샘플을 구합니다. 각 모델의 고지와 내장 설정이 별도로 있습니다. |
| [BOOTH](https://booth.pm/) | 제작자 에셋을 찾습니다. 실제 포함 파일과 개별 약관을 확인합니다. 무료라는 사실만으로 재배포 허용이나 VRM 호환성이 확인되지는 않습니다. |

공식 Hub의 [AvatarSample_A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604)와 [AvatarSample_B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889)는 현재 **VRM 0.0**으로 표시됩니다. 준비하려는 VRM 1.0 파일과 구분해야 합니다. 두 모델의 [샘플 이용 조건](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z)은 CC0가 아닌 별도 조건입니다.

개발자용으로는 pixiv의 보관된 공식 [ChatVRM 저장소의 AvatarSample_B](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm)가 별도의 VRM 1.0 파일입니다. [Twist 샘플](https://github.com/vrm-c/vrm-specification/tree/master/samples/VRM1_Constraint_Twist_Sample)과 [Seed-san](https://github.com/vrm-c/vrm-specification/tree/master/samples/Seed-san)도 1.0 후보입니다. 검사한 파일에는 필수 몸통·팔다리 본과 양손 손가락 본이 있습니다. 이는 형식과 구성 확인 결과입니다. Twist에는 자체 VRM Public License 설정이 적용되며 three-vrm 코드의 MIT 라이선스를 따르는 모델이 아닙니다. Seed-san은 크레딧 표시가 필요합니다. 파일 식별값과 출처별 조건은 [엔지니어링 기록](research/vrm-models.md)에 정리했습니다.

## 편집 원본과 실행 파일을 함께 보관하기

`.vroid`는 VRoid Studio의 편집 원본, `.vrm`은 실행 앱용으로 내보낸 모델입니다. 둘 다 보관하세요. 내려받은 `.vrm`이 원래 체형 파라미터를 편집하는 원본을 대신하지는 않습니다. 최신 Studio의 dress-up 기능은 VRM 데이터를 열 수 있으므로, 이 구분을 “Studio에서는 어떤 VRM도 열 수 없다”는 뜻으로 이해하면 안 됩니다. [공식 파일 형식 안내](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio), [dress-up VRM 지원](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM)

후속 검증용 체형 두 개는 다음과 같이 준비합니다.

1. Studio에서 새 모델이나 편집이 허용된 원본을 엽니다. 의상·텍스처의 조건도 보관합니다.
2. `.vroid`를 두 파일로 저장합니다. 의상은 비슷하게 유지하고, 하나는 작고 하나는 크게 하되 신체 비율도 다르게 설정합니다. 체형 설정값과 Studio 버전을 기록합니다.
3. **Export VRM**에서 **VRM 1.0**을 고르고 모델·제작자 정보를 입력합니다. `.vroid` 원본 두 개도 유지합니다. [공식 내보내기 안내](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature)를 참고하세요.
4. 그립 확인에 필요한 손·손가락 본을 유지합니다. 본 축소 옵션을 사용했다면 내보낸 파일에서 실제로 남아 있는지 확인합니다.

이번 조사에서는 위 절차를 문서화했으며 체형 두 파일을 직접 생성하지는 않았습니다.

## 사용 목적별 조건 확인하기

| 하려는 일 | 확인할 항목 |
|---|---|
| 로컬에서 불러와 평가 | 출처의 다운로드·사용 조건, 사용 가능한 사람, 추가 조건 |
| 렌더링한 연주 영상 공개 | 영상·아바타·모델 이용, 해당되는 경우 상업 이용, 표현 제한, 필요한 크레딧 |
| `.vrm`·텍스처·수정 모델 공유 | 별도의 재배포·수정 조건. 영상 이용 허용만으로 원본 모델의 배포까지 허용되는 것은 아닙니다. |

모델에 내장된 라이선스 링크와 배포 페이지를 함께 읽으세요. [VRM Public License](https://vrm.dev/licenses/1.0/)에는 모델별 설정이 함께 적용됩니다. 외부 의상·텍스처·특정 프리셋에는 추가 조건이 있을 수 있습니다. [VRoid 모델 이용 안내](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes)도 참고하세요. 이 저장소의 MIT 라이선스는 자체 코드에 적용되며 캐릭터 에셋의 조건을 바꾸지 않습니다.

확보한 모델은 Git에 넣지 않습니다. 개발자는 무시되는 `.local/vrm-test-models/`를 사용하며, 외부 아바타 바이너리를 공개 에셋·테스트 fixture·릴리스 첨부 파일에 넣지 않습니다.

## 다음 구현에서 확인할 것

예정된 모델 정보 패널은 감지한 VRM 버전, 필수 본, 손가락 지원, 모델 조건, 실제 호환성 상태를 구분해야 합니다. 누락된 정보는 알 수 없는 상태로 표시해야 합니다.

캐릭터 불러오기를 지원하고 검증한 빌드가 준비되면 두 체형에 같은 MIDI를 적용해 좌석 위치, 손의 그립, 실제 렌더링된 스틱 끝의 접촉, 발·페달, 도달할 수 없는 타점을 비교합니다. 앞뒤로 탐색한 같은 시각과 고정 시각 영상 출력도 비교합니다. 파싱에 성공한 모델이 이 검사를 통과한 것은 아닙니다. 공개 앱에서 캐릭터를 불러오기 전에 해당 빌드의 실제 지원 범위를 릴리스 안내에서 확인하세요.
