import { joinMessages, msg } from './i18n';
import type { MappingAnalysis, MappingProfile, MidiInfo, Selection } from './types';

export const mappingLabel = (profile: MappingProfile = 'gm'): string =>
  ({ gm: 'General MIDI', logicDkdGm: 'Logic DKD · GM', songsterrDownloader: 'Songsterr Downloader' })[profile];

/** Only new uploads use this choice. Manual edits, demos and saved projects keep
 * their explicit selection. Rust owns candidate scoring and semantic ambiguity. */
export function automaticImportSelection(info: MidiInfo): Selection {
  return {
    tracks: [...info.suggestedTracks], channel: info.suggestedChannel,
    mappingProfile: info.mappingAnalysis.selected ?? 'gm',
    countInNote75: false, hihatPolicy: 'literal', hihatOpenness: .2,
    omittedSourceEventIds: [], handOverrides: {},
  };
}

export function automaticMappingMessage(analysis: MappingAnalysis): string {
  const counts = joinMessages(analysis.candidates.map(candidate =>
    msg('{0}: {1}/{2}', '{0}: {1}/{2}', mappingLabel(candidate.profile), candidate.mapped, analysis.totalNotes)), ' · ');
  if (!analysis.selected) return msg(
    'No mapping candidate supports these notes. Check tracks, channel and mapping. {0}',
    '이 음표를 지원하는 매핑 후보가 없습니다. 트랙·채널·매핑을 확인하세요. {0}', counts);
  const summary = analysis.ambiguous ? msg(
    'Provisional choice: {0}. Equally supported mappings disagree on instruments or articulations. Review MIDI settings.',
    '임시 선택: {0}. 같은 수의 음표를 지원하는 후보들의 악기·주법 해석이 다릅니다. MIDI 설정에서 확인하세요.', mappingLabel(analysis.selected)) : msg(
    'Automatically selected {0} by supported note count. You can change it in MIDI settings; this does not identify the exporter.',
    '지원하는 음표 수를 비교한 자동 선택: {0}. MIDI 설정에서 변경할 수 있으며 제작 프로그램을 식별한 결과는 아닙니다.', mappingLabel(analysis.selected));
  return joinMessages([summary, counts], ' ');
}
