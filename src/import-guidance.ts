import { msg } from './i18n';
import type { PerformancePlan } from './types';

/** Describes evidence from the prepared selection; it does not identify a DAW,
 * choose a profile, or reinterpret an event. Rust owns those boundaries. */
export function importGuidance(plan: PerformancePlan) {
  const notes = (plan.sourceEvents ?? []).filter(event => event.message.kind === 'noteOn' && event.message.velocity > 0);
  const keys = new Set(notes.map(event => event.message.kind === 'noteOn' ? event.message.key : -1));
  const interpreted = plan.interpretations ?? [];
  const open = interpreted.filter(event => event.kind === 'strike' && event.instrument === 'openHat').length;
  const paired = interpreted.filter(event => event.kind === 'control').length;
  const foot = interpreted.filter(event => event.kind === 'strike' && event.instrument === 'pedalHat').length;
  const countInCandidates = notes.filter(event => event.message.kind === 'noteOn' && event.message.key === 75).length;
  const evidence: string[] = [];
  if (plan.mappingProfile === 'songsterrDownloader') {
    evidence.push(msg("Songsterr Downloader compatibility mapping is selected. Original MIDI numbers are retained. Half-open hi-hat, chokes and other unsupported percussion remain unmapped; this does not identify the exporter automatically.", "Songsterr Downloader 호환 매핑을 선택했습니다. 원본 MIDI 번호는 보존되며 반개방 하이햇·초크·기타 미지원 타악기는 미매핑으로 남습니다. 제작 프로그램을 자동 판별한 결과가 아닙니다."));
    return { countInCandidates, open, foot, paired, hasHihatChoice: open > 0 && foot + paired > 0, evidence };
  }
  if (keys.has(32) || keys.has(34)) evidence.push(msg("Notes 32 or 34 are present. Check whether they represent snare articulations in Drum Kit Designer GM.", "32·34번 계열 노트가 있습니다. Drum Kit Designer GM 매핑의 스네어 표현인지 확인하세요."));
  if (keys.has(52)) evidence.push(msg("Note 52 is a china cymbal in General MIDI and a ride edge in Drum Kit Designer GM. Select the mapping used by your drum instrument.", "52번은 General MIDI에서 차이나, Drum Kit Designer GM에서 라이드 가장자리입니다. 사용한 음원을 기준으로 선택하세요."));
  if (!evidence.length) evidence.push(msg("Notes alone cannot identify the original drum instrument or DAW. Select the mapping used by your drum instrument.", "노트만으로 원래 음원이나 제작 프로그램을 확정할 수 없습니다. 사용한 드럼 음원의 매핑을 선택하세요."));
  return { countInCandidates, open, foot, paired, hasHihatChoice: open > 0 && foot + paired > 0, evidence };
}
