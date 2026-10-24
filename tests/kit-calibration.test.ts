import { describe, expect, it } from 'vitest';
import { applyKitLayout, moveKitTarget, validateKitLayout } from '../src/kit-calibration';
import type { Instrument, KitPiece, PerformancePlan } from '../src/types';

const ids: Instrument[]=['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
const kit=():KitPiece[]=>ids.map(id=>({id,label:id,position:[0,id==='kick'||id==='pedalHat'?.08:1,.5],radius:.2}));
describe('physical layout recalibration',()=>{
  it('couples hi-hat surfaces while preserving the independently placed pedal',()=>{
    const original=kit(), moved=moveKitTarget(original,'hihat',[-.4,.9,.6]);
    expect(moved.find(p=>p.id==='openHat')!.position).toEqual([-.4,.9,.6]);
    expect(moved.find(p=>p.id==='pedalHat')).toEqual(original.find(p=>p.id==='pedalHat'));
    expect(original.find(p=>p.id==='hihat')!.position).toEqual([0,1,.5]);
  });
  it('retargets all hits without changing source events, timing or diagnostic provenance',()=>{
    const original: PerformancePlan={schemaVersion:1,duration:2,tempoCount:1,kit:kit(),diagnostics:[{code:'UNMAPPED_NOTE',message:'retained'}],hits:ids.map((instrument,id)=>({id,time:id/10,timeUs:id*100000,tick:id*96,track:1,eventIndex:id,channel:9,note:36+id,velocity:85,instrument,limb:null,target:[0,0,0]}))};
    const before=structuredClone(original), layout=moveKitTarget(original.kit,'kick',[.2,.09,.7]);
    const changed=applyKitLayout(original,layout);
    const immutable=(p:PerformancePlan)=>p.hits.map(({target,...event})=>event);
    expect(immutable(changed)).toEqual(immutable(original));
    expect(changed.hits[0].target).toEqual([.2,.09,.7]);
    changed.hits[0].target[0]=8;changed.kit[0].position[1]=9;
    expect(original).toEqual(before);expect(layout[0].position).toEqual([.2,.09,.7]);
    expect(changed.diagnostics).toEqual(original.diagnostics);
  });
  it('rejects incomplete, duplicate, nonfinite and split hi-hat contracts',()=>{
    expect(()=>validateKitLayout(kit().slice(1))).toThrow();
    const duplicate=kit();duplicate[1]=duplicate[0];expect(()=>validateKitLayout(duplicate)).toThrow();
    expect(()=>moveKitTarget(kit(),'snare',[NaN,1,.5])).toThrow();
    expect(()=>moveKitTarget(kit(),'snare',[0,-1,.5])).toThrow();
    expect(()=>moveKitTarget(kit(),'kick',[0,.03,.5])).toThrow(/페달/);
    expect(()=>moveKitTarget(kit(),'pedalHat',[0,.21,.5])).toThrow(/페달/);
    const split=kit();split[3].position[1]=1.1;expect(()=>validateKitLayout(split)).toThrow();
  });
});
