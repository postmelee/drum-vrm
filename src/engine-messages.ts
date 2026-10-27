import { msg } from './i18n';

/** Compact labels use diagnostic codes; complete engine messages remain in details. */
export function diagnosticTitle(code: string): string {
  const labels: Record<string, [string, string]> = {
    RIG_UNREACHABLE_HIT: ['Target out of reach', '타점 도달 불가'],
    RIG_UNREACHABLE_PEDAL: ['Pedal out of reach', '페달 도달 불가'],
    UNRESOLVED_HIT: ['Motion unassigned', '동작 미배정'],
    UNMAPPED_NOTE: ['Instrument unmapped', '악기 미매핑'],
    HAND_OVERRIDE_INFEASIBLE: ['Hand assignment unavailable', '지정한 손으로 연주 불가'],
    BODY_PATH_CLEARANCE: ['Body path overlap', '신체 경로 겹침'],
    EQUIPMENT_PATH_CLEARANCE: ['Equipment path overlap', '악기 경로 겹침'],
    EQUIPMENT_LAYOUT_CLEARANCE: ['Equipment layout overlap', '악기 배치 겹침'],
    COUNT_IN_GESTURE_UNAVAILABLE: ['Count-in motion unavailable', '카운트인 동작 불가'],
    HIHAT_INTENT_CONFLICT: ['Hi-hat interpretation conflict', '하이햇 해석 충돌'],
    TEMPO_CONFLICT: ['Tempo conflict', '템포 충돌'],
    TIME_SIGNATURE_CONFLICT: ['Time signature conflict', '박자표 충돌'],
    INVALID_TIME_SIGNATURE: ['Invalid time signature', '잘못된 박자표'],
  };
  return labels[code] ? msg(...labels[code]) : msg('Diagnostic: {0}', '진단: {0}', code);
}

/** Presentation adapter for the current Rust diagnostic text protocol.
 * The engine plan and its original diagnostic messages are never rewritten.
 * Keep exact templates covered by the Rust-message coverage test until the
 * engine exposes structured diagnostic arguments at this boundary. */
export const ENGINE_MESSAGE_TEMPLATES: readonly (readonly [string, string])[] = [
  [
    "{0} impact pitches have no safe posture/equipment witness within the bounded pitch-rate schedule. Original contact times are preserved; inspect the marked interval.",
    "타격 피치 {0}개는 제한된 피치 변화율 안에서 안전한 자세·장비 접촉 후보를 유지하지 못했습니다. 원래 타격 시각은 유지합니다. 표시된 구간을 확인하세요."
  ],
  [
    "The selected hi-hat surface has no movable upper cymbal. Openness changes the pedal state; the displayed strike surface remains fixed.",
    "선택한 하이햇 표면에는 움직일 수 있는 상단 심벌이 없습니다. 개방량은 페달 상태에 반영하며 화면의 타격 표면은 고정합니다."
  ],
  [
    "{0} hand-impact poses exceed the common posture limits. Contact timing is preserved; adjust the layout and inspect the arms.",
    "손 타격 자세 {0}개가 공통 자세 한계를 벗어납니다. 원래 타격 시각은 유지합니다. 배치를 조정하고 팔을 확인하세요."
  ],
  [
    "MIDI PPQ must be greater than zero.",
    "MIDI PPQ 값은 0보다 커야 합니다."
  ],
  [
    "MIDI tick accumulation overflow.",
    "MIDI tick 누적 범위 초과"
  ],
  [
    "MIDI ticks exceed the safe integer range.",
    "MIDI tick이 안전 정수 범위를 초과합니다."
  ],
  [
    "MIDI syntax error: {0}",
    "MIDI 구문 오류: {0}"
  ],
  [
    "MIDI time exceeds the supported range.",
    "MIDI 시간 범위를 초과했습니다."
  ],
  [
    "MIDI event syntax error: {0}",
    "MIDI 이벤트 구문 오류: {0}"
  ],
  [
    "The MIDI contains more than {0} events. Export fewer tracks.",
    "MIDI 이벤트가 {0}개를 초과합니다. 트랙을 줄여 다시 내보내세요."
  ],
  [
    "The MIDI end tick does not match the performance duration.",
    "MIDI 종료 tick과 연주 길이가 일치하지 않습니다."
  ],
  [
    "MIDI channel must be between 1 and 16.",
    "MIDI 채널은 1–16 범위여야 합니다."
  ],
  [
    "MIDI tempo must be greater than zero.",
    "MIDI 템포 값은 0보다 커야 합니다."
  ],
  [
    "The accumulated MIDI tempo time is inconsistent.",
    "MIDI 템포 구간의 누적 시각이 다릅니다."
  ],
  [
    "MIDI tempo segments are out of order.",
    "MIDI 템포 구간의 순서가 잘못되었습니다."
  ],
  [
    "The MIDI contains too many tempo segments.",
    "MIDI 템포 구간이 한계를 초과합니다."
  ],
  [
    "The MIDI tempo timeline must start at tick 0.",
    "MIDI 템포 시간축은 tick 0에서 시작해야 합니다."
  ],
  [
    "The MIDI tempo timeline is missing.",
    "MIDI 템포 시간축이 없습니다."
  ],
  [
    "Invalid MIDI tempo timeline.",
    "MIDI 템포 시간축이 유효하지 않습니다."
  ],
  [
    "MIDI track syntax error: {0}",
    "MIDI 트랙 구문 오류: {0}"
  ],
  [
    "The MIDI track data is truncated.",
    "MIDI 트랙 데이터가 잘렸습니다."
  ],
  [
    "MIDI must contain between 1 and 512 tracks.",
    "MIDI 트랙 수는 1–512개여야 합니다."
  ],
  [
    "The MIDI track header is truncated.",
    "MIDI 트랙 헤더가 잘렸습니다."
  ],
  [
    "MIDI files must be no larger than 10 MiB.",
    "MIDI 파일은 10 MiB 이하여야 합니다."
  ],
  [
    "The MIDI header does not match the actual track count.",
    "MIDI 헤더와 실제 트랙 수가 다릅니다."
  ],
  [
    "SMF format 0 must contain exactly one track.",
    "SMF format 0에는 트랙이 하나여야 합니다."
  ],
  [
    "SMF format 2 is not supported. Export format 0 or 1.",
    "SMF format 2는 지원하지 않습니다. Format 0 또는 1로 내보내세요."
  ],
  [
    "SMPTE timing is not supported. Export with PPQ timing.",
    "SMPTE 시간축은 지원하지 않습니다. PPQ 시간축으로 내보내세요."
  ],
  [
    "Time-signature conflict at tick {0}: selected {2}/{3} from track {1} (track 1 takes priority; the last event wins within a track).",
    "tick {0}의 박자표 충돌: 트랙 {1}의 {2}/{3}를 선택했습니다 (트랙 1 우선, 동일 트랙 마지막 이벤트)."
  ],
  [
    "Tempo conflict at tick {0}: selected {3} μs/beat from track {1}, event {2} (track 1 takes priority; the last event wins within a track).",
    "tick {0}의 템포 충돌: 트랙 {1} 이벤트 {2}의 {3} μs/박을 선택했습니다 (트랙 1 우선, 동일 트랙 마지막 이벤트)."
  ],
  [
    "{0}s, MIDI note {1}: {2}. Source events and audio retain their timing; unassigned strikes have no contact feedback.",
    "{0}초 MIDI 음표 {1}: {2}. 원본 이벤트와 소리의 시각은 유지하며 미배정 타격의 접촉 효과는 표시하지 않습니다."
  ],
  [
    "{0}s, count-in {1}: {2}. The original cue and actual strikes are preserved.",
    "{0}초 카운트인 {1}: {2}. 원본 cue와 실제 타격은 유지합니다."
  ],
  [
    "The foot cannot reach the {0} pedal. Adjust the seat or pedal position. Leg length is not stretched.",
    "{0} 페달에 발이 닿지 않습니다. 좌석 또는 페달 위치를 조절하세요. 다리 길이는 늘리지 않습니다."
  ],
  [
    "Simultaneous strikes are assigned to the same limb.",
    "같은 손발에 동시 타격이 배정되어 있습니다."
  ],
  [
    "No stick-contact pose satisfied the common body and equipment constraints",
    "공통 신체·장비 제약을 만족하는 스틱 접촉 자세를 찾지 못했습니다"
  ],
  [
    "No limb could be assigned within the simultaneous-strike, travel-speed and reach limits",
    "동시 타격·이동 속도·도달 한계로 손발을 배정하지 못했습니다"
  ],
  [
    "The motion plan has {0} control points, exceeding the limit of {1}. Export a shorter MIDI segment or select fewer tracks.",
    "동작 계획 제어점 {0}개가 한계 {1}개를 초과합니다. 더 짧은 MIDI 구간을 내보내거나 선택 트랙 수를 줄여 다시 준비하세요."
  ],
  [
    "Rig torso coordinates are outside the finite meter range.",
    "리그 몸통 좌표가 유한한 미터 범위를 벗어납니다."
  ],
  [
    "Rig limb {0} has an invalid length, grip or unit rotation.",
    "리그 손발 {0}의 길이·그립·단위 회전이 유효하지 않습니다."
  ],
  [
    "Rig limb {0}'s ready pose is outside its reach.",
    "리그 손발 {0}의 준비 자세가 도달 범위를 벗어납니다."
  ],
  [
    "Rig collision measurements are outside the supported range.",
    "리그 충돌 형상 측정값이 지원 범위를 벗어납니다."
  ],
  [
    "Rig profile format error: {0}",
    "리그 프로필 형식 오류: {0}"
  ],
  [
    "The rig profile exceeds the 64 KiB limit.",
    "리그 프로필이 64 KiB 한계를 초과합니다."
  ],
  [
    "A version 1 performance plan cannot contain version 2 interpretation data.",
    "버전 1 연주 계획은 버전 2 해석 정보를 포함할 수 없습니다."
  ],
  [
    "The version 2 performance plan has no PPQ timeline.",
    "버전 2 연주 계획의 PPQ 시간축이 없습니다."
  ],
  [
    "The version 2 performance plan has no end tick.",
    "버전 2 연주 계획의 종료 tick이 없습니다."
  ],
  [
    "Select at least one drum track to analyze.",
    "분석할 드럼 트랙을 한 개 이상 선택하세요."
  ],
  [
    "The selected MIDI track does not exist.",
    "선택한 MIDI 트랙이 존재하지 않습니다."
  ],
  [
    "The selection contains more than {0} strikes. Select fewer tracks or a shorter segment.",
    "선택한 타격이 {0}개를 초과합니다. 트랙이나 구간을 줄여주세요."
  ],
  [
    "The selected tracks are {0} seconds long. Use MIDI no longer than 300 seconds.",
    "선택한 트랙 길이가 {0}초입니다. 300초 이하의 MIDI를 사용하세요."
  ],
  [
    "The selected tracks/channels contain no supported drum strikes. Check the track selection and GM drum notes.",
    "선택한 트랙/채널에 지원하는 드럼 타격이 없습니다. 트랙 선택과 GM 드럼 음표를 확인하세요."
  ],
  [
    "Hand override {0} must assign left (0) or right (1) to a non-omitted hand strike.",
    "손 지정 {0}는 제외하지 않은 손 타격에 왼손(0) 또는 오른손(1)만 사용할 수 있습니다."
  ],
  [
    "Wrist reference axes must be finite, orthogonal unit directions.",
    "손목 기준 축은 유한한 직교 단위 방향이어야 합니다."
  ],
  [
    "The snare reference target is missing.",
    "스네어 기준 타점이 없습니다."
  ],
  [
    "Instrument/support layout needs review: {0}. The displayed layout has not been verified as collision-free.",
    "악기·받침대 배치 확인 필요: {0}. 표시된 배치는 충돌 없음이 확인되지 않았습니다."
  ],
  [
    "An actual hand strike overlaps the two-stick preparation or recovery interval",
    "양손 준비·복귀 구간에 실제 손 타격이 있습니다"
  ],
  [
    "The performance plan exceeds the size limit.",
    "연주 계획 크기가 한계를 초과합니다."
  ],
  [
    "Performance plan format error: {0}",
    "연주 계획 형식 오류: {0}"
  ],
  [
    "Invalid limb assignments in the performance plan.",
    "연주 계획의 손발 배정이 유효하지 않습니다."
  ],
  [
    "Invalid kit, diagnostic or tempo data in the performance plan.",
    "연주 계획의 키트·진단·템포 수치가 유효하지 않습니다."
  ],
  [
    "Invalid strike time, ID, coordinates or MIDI values in the performance plan.",
    "연주 계획의 타격 시각·ID·좌표·MIDI 값이 유효하지 않습니다."
  ],
  [
    "The performance plan exceeds the input limits.",
    "연주 계획이 입력 한계를 벗어납니다."
  ],
  [
    "Performance edit instructions exceed the event limit.",
    "연주 편집 지시가 이벤트 한계를 초과합니다."
  ],
  [
    "Edited source event {0} is not a selected positive-velocity note-on.",
    "연주 편집의 원본 이벤트 {0}가 선택한 양의 note-on에 없습니다."
  ],
  [
    "right",
    "오른쪽"
  ],
  [
    "No valid Standard MIDI file header was found.",
    "올바른 Standard MIDI 파일 헤더가 없습니다."
  ],
  [
    "left",
    "왼쪽"
  ],
  [
    "The source MIDI event tick and time do not match.",
    "원본 MIDI 이벤트의 tick과 시각이 일치하지 않습니다."
  ],
  [
    "Source MIDI events are out of time order.",
    "원본 MIDI 이벤트의 시간 순서가 잘못되었습니다."
  ],
  [
    "The short hi-hat close at source event {0} conflicts with another close or control interval and remains a separate pedal strike.",
    "원본 이벤트 {0}의 짧은 하이햇 닫힘이 다른 닫힘 또는 제어 구간과 충돌하여 독립 페달 타격으로 유지했습니다."
  ],
  [
    "A strike is not connected to a source event.",
    "원본 이벤트에 연결되지 않은 타격입니다."
  ],
  [
    "Source event and MIDI interpretation counts differ or exceed the limits.",
    "원본 이벤트와 MIDI 해석 개수가 일치하지 않거나 한계를 초과합니다."
  ],
  [
    "The source event interpretation does not match the mapping profile.",
    "원본 이벤트와 매핑 프로필의 해석이 일치하지 않습니다."
  ],
  [
    "A source event has no interpretation.",
    "원본 이벤트의 해석이 없습니다."
  ],
  [
    "Invalid or duplicate source MIDI event.",
    "유효하지 않거나 중복된 원본 MIDI 이벤트입니다."
  ],
  [
    "The interval is shorter than the 0.12-second minimum for this stick-contact gesture",
    "이 스틱 접촉 표현의 최소 0.12초 준비 간격보다 짧습니다"
  ],
  [
    "This performance plan requires its matching rig profile.",
    "이 연주 계획에는 해당 리그 프로필이 필요합니다."
  ],
  [
    "The equipment path has {0} samples, exceeding the limit of {1}. Prepare a shorter MIDI segment or fewer tracks.",
    "장비 경로 표본 {0}개가 한계 {1}개를 초과합니다. 더 짧은 MIDI 구간이나 적은 트랙으로 다시 준비하세요."
  ],
  [
    "The source-event omission list contains duplicates.",
    "제외할 원본 이벤트가 중복되었습니다."
  ],
  [
    "Arm/stick and instrument/support proxies still overlap at {0} preparation, strike, recovery or ending samples (up to {1} mm). Original timing and targets are preserved.",
    "준비·타격·회수·종료 경로 표본 {0}곳에서 팔·스틱과 악기/받침대의 대리 형상 겹침이 남았습니다(최대 {1} mm). 원본 시각과 타점은 유지합니다."
  ],
  [
    "The prepared performance transfer exceeds the size limit.",
    "준비된 연주 전달 크기가 한계를 초과합니다."
  ],
  [
    "Prepared performance format error: {0}",
    "준비된 연주 형식 오류: {0}"
  ],
  [
    "The prepared performance requires a rig profile.",
    "준비된 연주에 리그 프로필이 필요합니다."
  ],
  [
    "The prepared performance rig profile does not match the request.",
    "준비된 연주의 리그 프로필이 요청과 일치하지 않습니다."
  ],
  [
    "Invalid torso path in the prepared performance.",
    "준비된 연주의 몸통 경로가 유효하지 않습니다."
  ],
  [
    "The prepared torso rotation does not preserve original strike contact.",
    "준비된 연주의 몸통 회전이 원본 타격 접촉을 보존하지 않습니다."
  ],
  [
    "The prepared performance version, source or checksum does not match.",
    "준비된 연주의 버전·원본·체크섬이 일치하지 않습니다."
  ],
  [
    "The prepared performance calibration ID does not match.",
    "준비된 연주의 보정 ID가 일치하지 않습니다."
  ],
  [
    "Invalid hand paths in the prepared performance.",
    "준비된 연주의 양손 경로가 유효하지 않습니다."
  ],
  [
    "Invalid equipment approach path in the prepared performance.",
    "준비된 연주의 장비 접근 경로가 유효하지 않습니다."
  ],
  [
    "The prepared performance pedal contacts do not match.",
    "준비된 연주의 페달 접촉 위치가 일치하지 않습니다."
  ],
  [
    "Duplicate MIDI interpretation.",
    "중복 MIDI 해석입니다."
  ],
  [
    "Duplicate drum piece ID.",
    "중복된 드럼 부품 ID입니다."
  ],
  [
    "Unsupported MIDI chunk. Export the file again as SMF.",
    "지원하지 않는 MIDI 청크입니다. SMF로 다시 내보내세요."
  ],
  [
    "Unsupported MIDI interpretation version.",
    "지원하지 않는 MIDI 해석 버전입니다."
  ],
  [
    "Unsupported MIDI header length (6 bytes required).",
    "지원하지 않는 MIDI 헤더 길이입니다 (6 바이트 필요)."
  ],
  [
    "Unsupported rig profile version, coordinate system or ID.",
    "지원하지 않는 리그 프로필 버전·좌표계·ID입니다."
  ],
  [
    "Unsupported preparation or tail duration.",
    "지원하지 않는 준비/여운 시간입니다."
  ],
  [
    "The assigned hand cannot strike at the original time. The other hand is not substituted automatically",
    "지정한 손으로 원래 시각에 타격할 수 없습니다. 자동으로 다른 손을 사용하지 않습니다"
  ],
  [
    "The kit layout exceeds the size limit.",
    "키트 배치 크기가 한계를 초과합니다."
  ],
  [
    "Kit layout format error: {0}",
    "키트 배치 형식 오류: {0}"
  ],
  [
    "The kit layout is outside the supported range.",
    "키트 배치가 지원 범위를 벗어납니다."
  ],
  [
    "Body/stick proxies still overlap at {0} strike times (up to {1} mm). Review motion paths separately from layout fitting.",
    "타격 시각 {0}곳에서 신체·스틱 대리 형상 겹침이 남았습니다(최대 {1} mm). 배치 완료와 별도로 연주 경로를 확인하세요."
  ],
  [
    "A strike interpretation has no connected strike.",
    "타격 해석에 연결된 타격이 없습니다."
  ],
  [
    "Strike source or mapping information differs from its MIDI interpretation.",
    "타격의 원본 또는 매핑 정보가 MIDI 해석과 다릅니다."
  ],
  [
    "Track {0} / channel {1} / MIDI note {2}: {3} source events were preserved but cannot be played with the selected mapping.",
    "트랙 {0} / 채널 {1} / MIDI 음표 {2}: {3}개의 원본 이벤트를 보존했지만 선택한 매핑으로 연주할 수 없습니다."
  ],
  [
    "An unsupported time signature at track {0}, tick {1} was ignored. Strike timing is preserved.",
    "트랙 {0} tick {1}의 지원하지 않는 박자표를 무시했습니다. 타격 시각은 유지합니다."
  ],
  [
    "Track {0} has no End of Track event.",
    "트랙 {0}에 End of Track 이벤트가 없습니다."
  ],
  [
    "Track {0} contains events after End of Track.",
    "트랙 {0}의 End of Track 뒤에 이벤트가 있습니다."
  ],
  [
    "Track selection format error: {0}",
    "트랙 선택 형식 오류: {0}"
  ],
  [
    "Pedal strikes do not match the kit's fixed contact positions.",
    "페달 타격과 키트의 고정 접촉 위치가 일치하지 않습니다."
  ],
  [
    "Hi-hat openness must be between 0.05 and 1.0.",
    "하이햇 개방량은 0.05–1.0 범위여야 합니다."
  ],
  [
    "Hi-hat controls do not match the source events and selected interpretation policy.",
    "하이햇 제어가 원본 이벤트와 선택한 해석 규칙에 일치하지 않습니다."
  ],
  [
    "An interpretation refers to a nonexistent strike.",
    "해석이 존재하지 않는 타격을 참조합니다."
  ],
  [
    "No two-hand ready pose with clearance was found in the current layout.",
    "현재 배치에서 충돌 여유가 있는 양손 준비 자세를 찾지 못했습니다."
  ],
  [
    "The target could not be reached with the current body, seat and grip",
    "현재 체형·좌석·그립에서 타점에 도달하지 못했습니다"
  ]
];

const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const templates = ENGINE_MESSAGE_TEMPLATES.map(([en, ko]) => ({
  en, ko, patterns: [ko, en].map(source => new RegExp('^' + source.split(/\{\d+\}/).map(escape).join('([\\s\\S]*?)') + '$')),
}));

/** Convert only known engine messages to canonical English. Unknown vendor
 * details are preserved verbatim; original plans remain serialization-stable. */
export function engineMessage(source: string): string {
  if (source.startsWith('Error: ')) return engineMessage(source.slice(7));
  for (const { en, ko, patterns } of templates) {
    const values = patterns.map(pattern => pattern.exec(source)).find(Boolean);
    if (values) return msg(en, ko, ...values.slice(1).map(value => engineMessage(value)));
  }
  return source;
}
