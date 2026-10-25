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
  if (keys.has(32) || keys.has(34)) evidence.push('32·34번 계열 노트가 있습니다. Drum Kit Designer GM 매핑의 스네어 표현인지 확인하세요.');
  if (keys.has(52)) evidence.push('52번은 General MIDI에서 차이나, Drum Kit Designer GM에서 라이드 가장자리입니다. 사용한 음원을 기준으로 선택하세요.');
  if (!evidence.length) evidence.push('노트만으로 원래 음원이나 제작 프로그램을 확정할 수 없습니다. 사용한 드럼 음원의 매핑을 선택하세요.');
  return { countInCandidates, open, foot, paired, hasHihatChoice: open > 0 && foot + paired > 0, evidence };
}
