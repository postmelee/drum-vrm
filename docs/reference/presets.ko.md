# 드럼 프리셋

[English](presets.md) · **한국어**

프리셋의 제공 범위는 [기능 제공 범위](availability.ko.md)에 있습니다. 프리셋은 외형을 바꾸며 합성 드럼 소리나 음악 음표 매핑은 바꾸지 않습니다.

| 프리셋 종류 | 제작자·조건 | 에셋 정보 원본 |
|---|---|---|
| Studio·Vintage·Electronic 자체 스타일 | 프로젝트 자체 도형; MIT | [자체 모델 코드](../../src/model-procedural.ts) |
| Five-piece drum kit | Innerscene; CC0-1.0 | [출처 기록](../../public/models/innerscene-provenance.json) |
| Full Drum Kit | Batoski; CC BY 3.0 | [출처 기록](../../public/models/batoski-provenance.json) |
| Drum Kit | smoj; CC BY 3.0 | [출처 기록](../../public/models/smoj-provenance.json) |

기본·깊은·컴팩트는 개별 부품 형태의 선택값입니다. 자체 키트 스타일은 프로젝트 도형의 변형이며 별도로 확보한 모델 세 개가 아닙니다. 외부 원본 바이너리는 보존하며 실행 시 어댑터가 부품 분류·좌표 정규화·연주 표면 맞춤을 합니다. 개발의 공통 하드웨어 경로는 원본의 모든 부품을 복제하는 대신 앱 받침대·페달과 몸체를 표시합니다.

정확한 제작자 링크·확보 파일 해시·라이선스 근거·변형은 위 provenance와 [에셋 고지](../../public/models/NOTICE.txt)에 있습니다. [라이선스 원문](../../public/models/README.ko.md)은 파일 옆에 보존합니다. 해당 에셋의 CC BY 출처 표기·원출처 조건은 MIT로 대체되지 않습니다.

임의의 로컬 GLB 몸체는 준비와 외형 확인이 필요합니다. [드럼 부품 교체](../how-to/replace-drum-parts.ko.md)를 따르세요. 앱의 크래시 역할은 하나이며 원본의 추가 심벌이 독립된 크래시 음표 매핑을 만들지는 않습니다.

외부 모델 전자 드럼과 다른 조사 후보는 설치된 프리셋이 아닌 연구 대상입니다. 원 조사는 [에셋 연구 기록](../../mydocs/tech/research/2026-10-24/drum-assets-followup.md)에 보존합니다.

[참조](README.ko.md)
