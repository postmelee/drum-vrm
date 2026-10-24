# 캐릭터 모델 확보하기

[English](obtain-model.md) · **한국어**

사용 목적을 허용하는 출처를 선택하세요. [모델 출처 기록](../reference/sources/models.ko.md)은 개별 샘플과 검사한 버전을 구분합니다.

| 출처 | 활용 |
|---|---|
| [VRoid Studio](https://vroid.com/en/studio) | 편집 가능한 캐릭터를 만들고 VRM 1.0으로 내보냅니다. |
| [VRoid Hub](https://hub.vroid.com/en/) | 제작자가 다운로드를 허용한 모델을 사이트 동의 절차에 따라 받습니다. |
| [VRM 공식 샘플](https://github.com/vrm-c/vrm-specification/tree/master/samples) | 샘플별 조건에 따라 문서화된 표준 기능을 살펴봅니다. |
| [BOOTH](https://booth.pm/) | 제작자 패키지의 포함 형식·버전·이용 조건을 확인합니다. |

## 실행용 파일 준비

1. 패키지에 VRM 1.0이 포함되어 있는지 확인하거나, VRM 1.0으로 내보낼 수 있는 허용된 편집 원본을 사용합니다.
2. `.vroid`는 Studio 편집 원본, `.vrm`은 실행용 출력으로 각각 보관합니다. 다운로드한 VRM이 원본 체형 편집값을 대신하지는 않습니다. Studio의 일부 VRM 의상 편집 지원도 두 형식이 같다는 의미는 아닙니다.
3. Studio에서 VRM 내보내기를 선택하고 **VRM 1.0**을 지정합니다. [공식 내보내기 안내](https://vroid.pixiv.help/hc/en-us/articles/15760756822297-I-want-to-learn-more-about-the-VRM-export-feature)를 따르세요. 가능한 한 손가락 본을 유지하고 본 감소 설정을 적용했다면 결과를 검사합니다.
4. 이전 버전 모델은 허용된 원본에서 다시 출력하거나 [UniVRM 이전 안내](https://vrm.dev/en/univrm1/migrate_vrm0/)를 사용합니다. 확장자만 바꾸는 것으로는 부족합니다.
5. [결과 파일을 불러온 뒤](import-character.ko.md) 사용할 MIDI로 확인합니다.

편집·출력 형식의 차이는 [Studio 형식 안내](https://vroid.pixiv.help/hc/en-us/articles/23731198070809-About-file-formats-used-by-VRoid-Studio)와 [VRM 의상 편집 설명](https://vroid.pixiv.help/hc/en-us/articles/39513464329881-What-is-VRM)을 참고하세요.

## 용도별 조건 확인

| 용도 | 확인할 사항 |
|---|---|
| 로컬 불러오기·평가 | 다운로드/사용 조건, 허용 사용자, 추가 조건. |
| 연주 영상 공개 | 아바타/영상 사용, 해당하는 상업 이용, 콘텐츠 제한, 필요한 출처 표기. |
| 모델·텍스처 공유 | 수정·재배포 조건. 영상 사용 허락이 모델 파일 재배포 허락을 뜻하지는 않습니다. |

파일 메타데이터와 출처 페이지를 함께 읽으세요. [VRM Public License](https://vrm.dev/licenses/1.0/)에는 모델별 설정이 적용됩니다. 의상·텍스처·특수 프리셋의 조건이 추가될 수 있으며 [VRoid 이용 안내](https://vroid.pixiv.help/hc/en-us/articles/4405813333657-Can-I-use-the-models-created-with-VRoid-Studio-Stable-Ver-for-commercial-purposes)를 참고하세요.

무료 가격이 CC0나 재배포 허락을 의미하지는 않습니다. 출력기가 모델 출처를 자동으로 삽입하지 않으므로 공개 시 필요한 표기를 추가하세요. 외부 캐릭터 다운로드는 저장소와 공개 에셋 밖에 보관합니다.

[작업별 사용법](README.ko.md)
