import { describe,expect,it } from 'vitest';
import { nearestNoteGroup,withArrangementEdit } from '../src/arrangement';
import type { PerformancePlan } from '../src/types';
const plan:PerformancePlan={schemaVersion:2,duration:2,hits:[],diagnostics:[],kit:[],tempoCount:1,
  sourceEvents:[{id:'1:1',timeUs:1000000,eventIndex:1,track:1,tick:480,channel:9,message:{kind:'noteOn',key:38,velocity:100}},
    {id:'1:2',timeUs:1000000,eventIndex:2,track:1,tick:480,channel:9,message:{kind:'noteOn',key:36,velocity:100}},
    {id:'1:3',timeUs:1000001,eventIndex:3,track:1,tick:481,channel:9,message:{kind:'noteOn',key:38,velocity:0}},
    {id:'2:1',timeUs:1100000,eventIndex:1,track:2,tick:500,channel:9,message:{kind:'noteOn',key:49,velocity:110}}],
  interpretations:[{sourceEventId:'1:1',kind:'strike',instrument:'snare'},{sourceEventId:'1:2',kind:'strike',instrument:'kick'},{sourceEventId:'2:1',kind:'strike',instrument:'crash'}]};
describe('source-stable manual arrangement',()=>{
  it('groups all coincident positive source notes without losing an omitted event',()=>{
    expect(nearestNoteGroup(plan,1.01).map(event=>event.id)).toEqual(['1:1','1:2']);
    const edited={...plan,interpretations:[{sourceEventId:'1:1',kind:'omitted' as const,instrument:'snare' as const}]};
    expect(nearestNoteGroup(edited,1).map(event=>event.id)).toEqual(['1:1','1:2']);
  });
  it('omits/restores a source ID without changing MIDI evidence or preserving its stale hand override',()=>{
    const before=JSON.stringify(plan),assigned={...plan,handOverrides:{'1:1':1 as const}};
    const omitted=withArrangementEdit(assigned,'1:1',true,'auto');
    expect(omitted).toEqual({omittedSourceEventIds:['1:1'],handOverrides:{}});
    expect(withArrangementEdit({...assigned,...omitted},'1:1',false,'auto')).toEqual({omittedSourceEventIds:[],handOverrides:{}});
    expect(JSON.stringify(plan)).toBe(before);
  });
  it('permits hand choices only for mapped non-omitted hand strikes',()=>{
    expect(withArrangementEdit(plan,'1:1',false,'1').handOverrides).toEqual({'1:1':1});
    expect(()=>withArrangementEdit(plan,'1:2',false,'0')).toThrow();
    expect(()=>withArrangementEdit(plan,'1:3',false,'0')).toThrow();
    expect(()=>withArrangementEdit(plan,'missing',true,'auto')).toThrow();
  });
});
