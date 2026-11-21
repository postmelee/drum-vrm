import { describe, expect, it } from 'vitest';
import { applyKitLayout, moveKitTarget, sameKitLayout, validateKitLayout } from '../src/kit-calibration';
import { pieceInstrument, type Instrument, type KitPiece, type PerformancePlan } from '../src/types';

const ids: Instrument[]=['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
const kit=():KitPiece[]=>ids.map(id=>({id,label:id,position:[0,id==='kick'||id==='pedalHat'?.08:1,.5],radius:.2}));
describe('physical layout recalibration',()=>{
  it('accepts optional unit body frames and rejects invalid measured rotations',()=>{
    const layout=kit(), piece=layout.find(value=>value.id==='tomHigh')!;
    piece.geometry={version:1,source:'procedural',referenceRadius:piece.radius,body:{center:[.01,-.1,.02],halfExtents:[.2,.11,.2]},surface:{center:[0,0,0],normal:[0,1,0],radius:.2}};
    expect(validateKitLayout(layout)[5].geometry!.body.rotation).toBeUndefined();
    piece.geometry.body.rotation=[Math.sin(.2),0,0,Math.cos(.2)];
    expect(validateKitLayout(layout)[5].geometry).toEqual(piece.geometry);
    for(const rotation of [[0,0,0,0],[0,0,0,2],[0,0,NaN,1],[0,0,1]]){
      const invalid=structuredClone(layout); invalid[5].geometry!.body.rotation=rotation as [number,number,number,number];
      expect(()=>validateKitLayout(invalid)).toThrow();
    }
  });
  it('round-trips semantic pedal direction and rejects non-horizontal or non-unit frames',()=>{
    const layout=kit(); layout.find(piece=>piece.id==='pedalHat')!.pedalForward=[-.6,0,.8];
    const restored=JSON.parse(JSON.stringify(layout)); validateKitLayout(restored);
    expect(sameKitLayout(layout,restored)).toBe(true);
    const shifted=structuredClone(restored);delete shifted.find((piece:KitPiece)=>piece.id==='pedalHat')!.pedalForward;
    expect(sameKitLayout(layout,shifted)).toBe(false);
    for(const direction of [[0,0,0],[0,0,2],[0,.1,Math.sqrt(.99)],[NaN,0,1]]){
      const invalid=structuredClone(layout);invalid.find(piece=>piece.id==='pedalHat')!.pedalForward=direction as [number,number,number];
      expect(()=>validateKitLayout(invalid)).toThrow();
    }
    const kick=structuredClone(layout);kick.find(piece=>piece.id==='kick')!.pedalForward=[0,0,1];
    expect(()=>validateKitLayout(kick)).toThrow();
  });
  it('retains bounded tripod and floor-leg phases without accepting them on another mount',()=>{
    const layout=kit();layout.find(piece=>piece.id==='snare')!.standRotation=Math.PI/2;
    layout.find(piece=>piece.id==='tomLow')!.standRotation=-Math.PI/2;
    const restored=validateKitLayout(JSON.parse(JSON.stringify(layout)));
    expect(sameKitLayout(layout,restored)).toBe(true);
    for(const value of [NaN,Infinity,Math.PI+.01]){
      const bad=structuredClone(layout);bad.find(piece=>piece.id==='snare')!.standRotation=value;
      expect(()=>validateKitLayout(bad)).toThrow();
    }
    const bad=structuredClone(layout);bad.find(piece=>piece.id==='tomHigh')!.standRotation=0;
    expect(()=>validateKitLayout(bad)).toThrow();
    const mounted=structuredClone(layout);mounted.find(piece=>piece.id==='tomLow')!.mount={kind:'standClamp',supportPieceId:'ride'};
    expect(()=>validateKitLayout(mounted)).toThrow();
  });
  it('keeps equivalent serialized setup but detects physical and semantic edits',()=>{
    const accepted=kit(), serialized=accepted.map(piece=>({radius:piece.radius,instrument:pieceInstrument(piece),label:piece.label,position:[...piece.position] as [number,number,number],id:piece.id}));
    serialized[1].radius+=Number.EPSILON;
    expect(sameKitLayout(accepted,serialized)).toBe(true);
    const moved=structuredClone(serialized);moved[1].position[1]+=.001;
    expect(sameKitLayout(accepted,moved)).toBe(false);
    const resized=structuredClone(serialized);resized[1].radius+=.0001;
    expect(sameKitLayout(accepted,resized)).toBe(false);
    const relabeled=structuredClone(serialized);relabeled[1].label='Edited snare';
    expect(sameKitLayout(accepted,relabeled)).toBe(false);
    const nonfinite=structuredClone(serialized);nonfinite[1].radius=NaN;
    expect(sameKitLayout(accepted,nonfinite)).toBe(false);
  });
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

  it('retargets distinct crashes by physical identity without changing their shared voice',()=>{
    const layout=kit();layout.push({id:'crashRight',instrument:'crash',label:'Crash Right',position:[.6,1.1,.5],radius:.3});
    const plan:PerformancePlan={schemaVersion:2,duration:2,tempoCount:1,kit:layout,diagnostics:[],hits:['crash','crashRight'].map((pieceId,id)=>({id,time:1,tick:480,track:0,channel:0,note:id?57:49,velocity:100,instrument:'crash',pieceId:pieceId as 'crash'|'crashRight',limb:id,target:[0,0,0]}))};
    const changed=applyKitLayout(plan,moveKitTarget(layout,'crashRight',[.7,1.2,.6]));
    expect(changed.hits.map(h=>h.target)).toEqual([[0,1,.5],[.7,1.2,.6]]);
    expect(changed.hits.map(h=>h.instrument)).toEqual(['crash','crash']);
    expect(()=>applyKitLayout(plan,kit())).toThrow('drum piece');
    const mismatched=structuredClone(layout);mismatched[10].instrument='snare';
    expect(()=>validateKitLayout(mismatched)).toThrow('instrument role');
  });
  it('rejects incomplete, duplicate, nonfinite and split hi-hat contracts',()=>{
    expect(()=>validateKitLayout(kit().slice(1))).toThrow();
    const duplicate=kit();duplicate[1]=duplicate[0];expect(()=>validateKitLayout(duplicate)).toThrow();
    expect(()=>moveKitTarget(kit(),'snare',[NaN,1,.5])).toThrow();
    expect(()=>moveKitTarget(kit(),'snare',[0,-1,.5])).toThrow();
    expect(()=>moveKitTarget(kit(),'kick',[0,.03,.5])).toThrow(/Pedal/);
    expect(()=>moveKitTarget(kit(),'pedalHat',[0,.21,.5])).toThrow(/Pedal/);
    const split=kit();split[3].position[1]=1.1;expect(()=>validateKitLayout(split)).toThrow();
  });
});
