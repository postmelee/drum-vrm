import { msg } from './i18n';
import type { PerformancePlan, Selection, SourceEvent } from './types';

export type Arrangement = Required<Pick<Selection,'omittedSourceEventIds'|'handOverrides'>>;
export const arrangementOf=(value:Pick<Selection,'omittedSourceEventIds'|'handOverrides'>):Arrangement=>({omittedSourceEventIds:[...(value.omittedSourceEventIds??[])],handOverrides:{...(value.handOverrides??{})}});
export function noteSources(plan:PerformancePlan):SourceEvent[] {
  return (plan.sourceEvents??[]).filter(event=>event.message.kind==='noteOn'&&event.message.velocity>0)
    .sort((a,b)=>a.timeUs-b.timeUs||a.track-b.track||a.eventIndex-b.eventIndex);
}
export function nearestNoteGroup(plan:PerformancePlan,time:number):SourceEvent[] {
  const notes=noteSources(plan);if(!notes.length)return [];
  const nearest=notes.reduce((best,note)=>Math.abs(note.timeUs-time*1e6)<Math.abs(best.timeUs-time*1e6)?note:best);
  return notes.filter(note=>note.timeUs===nearest.timeUs);
}
export function withArrangementEdit(plan:PerformancePlan,sourceEventId:string,omitted:boolean,hand:'auto'|'0'|'1'):Arrangement {
  const source=noteSources(plan).find(event=>event.id===sourceEventId);
  if(!source)throw new Error(msg("The source note to edit was not found.", "편집할 원본 음표를 찾지 못했습니다."));
  const result=arrangementOf(plan),meaning=plan.interpretations?.find(event=>event.sourceEventId===sourceEventId);
  result.omittedSourceEventIds=result.omittedSourceEventIds.filter(id=>id!==sourceEventId);
  delete result.handOverrides[sourceEventId];
  if(omitted)result.omittedSourceEventIds.push(sourceEventId);
  else if(hand!=='auto'){
    if(meaning?.kind!=='strike'||!meaning.instrument||['kick','pedalHat'].includes(meaning.instrument))throw new Error(msg("A hand can only be assigned to a hand-played strike.", "손으로 연주하는 타격만 손을 지정할 수 있습니다."));
    result.handOverrides[sourceEventId]=Number(hand) as 0|1;
  }
  result.omittedSourceEventIds.sort();
  return result;
}
