# 캐릭터 모델 출처

[English](models.md) · **한국어**

**2026-10-24** 검토에서 확보 경로와 조건을 기록했습니다. 표시 이름이 같은 다른 파일의 호환성을 보장하지 않습니다. [원 모델 감사](../../../mydocs/tech/research/2026-10-24/vrm-models.md)는 검사 commit·SHA-256·메타데이터·체형·검증 범위를 보존합니다. 후보 아바타는 번들에 포함하지 않습니다.

| 후보 | 출처·관찰 버전 | 유지할 조건 |
|---|---|---|
| VRM1_Constraint_Twist_Sample | [고정 commit의 공식 샘플](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample); VRM 1.0 | [샘플 고지](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/VRM1_Constraint_Twist_Sample/README.md): pixiv Inc., VRM Public License 1.0과 파일 설정이며 three-vrm의 MIT가 아닙니다. |
| Seed-san | [고정 commit의 공식 샘플](https://github.com/vrm-c/vrm-specification/tree/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san); VRM 1.0 | [샘플 고지](https://github.com/vrm-c/vrm-specification/blob/94e82dd346fa6cf0337c4421728640e5252dd38e/samples/Seed-san/README.md): VirtualCast, Inc., VRM Public License 1.0이며 검사한 메타데이터는 출처 표기를 요구합니다. |
| AvatarSample_B, ChatVRM 파일 | [pixiv 보관 ChatVRM 파일](https://github.com/pixiv/ChatVRM/blob/b542aa00e19dccf9fc48ba340cf7eee011d2329a/public/AvatarSample_B.vrm); VRM 1.0, 모델 버전 1.1 | VRoid Project; VRM Public License와 [추가 샘플 조건](https://vroid.pixiv.help/hc/en-us/articles/4402394424089-VRoidPreset-A-Z). |
| Hub의 AvatarSample_A / B | [A](https://hub.vroid.com/en/characters/2843975675147313744/models/5644550979324015604), [B](https://hub.vroid.com/en/characters/7939147878897061040/models/2292219474373673889); 검토 당시 페이지는 VRM 0.0으로 표시 | Hub 바이너리는 받지 않았습니다. 다운로드/동의 절차·샘플 조건을 따르며 별도의 ChatVRM 1.0 파일과 혼동하지 않습니다. |

VRoid A/B 샘플 조건은 CC0가 아닌 별도 조건입니다. 출처 표기 의무 없이 영상 등의 상업/비상업 이용을 허용하지만 CC0로 바꾸기, 제공 샘플 데이터 유료 재배포, 일부 캐릭터 제작 서비스 이용을 제한합니다. 전체 조건과 행위 제한을 읽으세요. [공식 샘플 분류](https://vroid.pixiv.help/hc/en-us/articles/4402614652569-Do-VRoid-Studio-s-sample-models-come-with-conditions-of-use)는 이전 CC0 샘플과 A/B/C를 구분합니다.

[VRM Public License](https://vrm.dev/licenses/1.0/)에는 모델별 설정이 포함되므로 라이선스 계열명만으로 판단할 수 없습니다. 자신의 파일에 적용되는 출처 조건·메타데이터를 확인하세요. 로컬 평가·영상 공개·모델 파일 재배포는 서로 다른 용도입니다.

## 확인한 것과 확인하지 않은 것

고정된 Twist·Seed-san·ChatVRM B 파일에는 필수 본·손가락 본이 있으며 제한적인 로더·어댑터·접촉·탐색 검사를 실행했습니다. 구현·검증 경계는 [기능 범위와 정확한 근거](../availability.ko.md)를 참고하세요. 구조 검사나 접촉 검사가 모든 자세·피부 충돌 여유·출력 파일 독립 재생까지 입증하지는 않습니다.

VRoid Studio에서 통제된 짧은/긴 체형 쌍을 만드는 일은 계획된 실험입니다. [리그 호환성 설계](../../../specs/architecture/rig-compatibility.md)는 비교 항목을 설명합니다. 편집/실행 형식과 권리는 [모델 확보](../../how-to/obtain-model.ko.md), 다음 단계는 [불러오기](../../how-to/import-character.ko.md)를 따르세요.

[출처](README.ko.md)
