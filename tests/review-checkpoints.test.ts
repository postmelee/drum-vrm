import { beforeAll, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { initSync, Performance } from '../src/wasm/drum_engine';
import { makePostureDemo } from '../src/demo';
import { reviewCheckpoints } from '../src/review-checkpoints';
import type { PerformancePlan } from '../src/types';
beforeAll(()=>initSync({module:readFileSync(new URL('../src/wasm/drum_engine_bg.wasm',import.meta.url))}));
it('provides every requested review phase through real source interpretation',()=>{
  const run=new Performance(makePostureDemo(),JSON.stringify({tracks:[1],channel:0,mappingProfile:'logicDkdGm',countInNote75:true}));
  try {
    const plan=JSON.parse(run.plan_json()) as PerformancePlan;
    expect(plan.duration).toBe(12);
    expect(plan.interpretations?.filter(event=>event.kind==='countIn')).toHaveLength(3);
    expect(plan.hits.filter(hit=>hit.time===10.5).map(hit=>hit.note).sort((a,b)=>a-b)).toEqual([36,49,57]);
    expect(reviewCheckpoints(plan).map(point=>point.time)).toEqual([-.25,2,6.5,8,10.5,12]);
    expect(plan.diagnostics.filter(item=>item.code==='UNMAPPED_NOTE')).toHaveLength(0);
  } finally {run.free();}
});
it('does not invent a missing phrase and keeps failed assignments inspectable',()=>{
  expect(reviewCheckpoints({duration:4,hits:[]} as unknown as PerformancePlan).map(point=>point.time)).toEqual([-.25,4]);
  const plan={duration:4,hits:[{id:0,instrument:'crash',pieceId:'crash',time:2,limb:null},{id:1,instrument:'crash',pieceId:'crashRight',time:2,limb:1}]} as unknown as PerformancePlan;
  expect(reviewCheckpoints(plan).map(point=>point.time)).toEqual([-.25,2,4]);
});
