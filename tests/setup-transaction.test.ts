import { readFileSync } from 'node:fs';
import { beforeAll, expect, it, vi } from 'vitest';
import { executeEngineRequest } from '../src/engine-worker-actions';
import type { EngineRequest } from '../src/engine-protocol';
import { EngineClient } from '../src/engine';
import { makeInterpretationDemo } from '../src/demo';
import { NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import { initSync, Performance } from '../src/wasm/drum_engine';
import type { PerformancePlan, RigProfile } from '../src/types';

beforeAll(()=>initSync({module:readFileSync(new URL('../src/wasm/drum_engine_bg.wasm',import.meta.url))}));
// Exercise the real worker action and transfer contract while controlling delivery.
class LocalWorker {
  onmessage: ((event:MessageEvent)=>void)|null=null;
  onerror: (()=>void)|null=null;
  onmessageerror: (()=>void)|null=null;
  terminated=false;
  terminate(){this.terminated=true;}
  postMessage(request:EngineRequest & {requestId:number}){
    const snapshot=structuredClone(request);
    queueMicrotask(()=>{
      if(this.terminated)return;
      try{this.onmessage?.({data:{requestId:snapshot.requestId,ok:true,json:executeEngineRequest(snapshot)}} as MessageEvent);}
      catch(error){this.onmessage?.({data:{requestId:snapshot.requestId,ok:false,error:String(error)}} as MessageEvent);}
    });
  }
}
const localWorker=()=>new LocalWorker() as unknown as Worker;
function fixture(){
  const rig=new NativeHumanoidRigAdapter(),profile=rig.getProfile();
  // The optional geometry extension is exercised separately at its own boundary.
  delete profile.bodyGeometry;
  const selection={tracks:[1],channel:0,mappingProfile:'logicDkdGm',countInNote75:true,hihatPolicy:'shortOpenClose',hihatOpenness:.2};
  const source=new Performance(makeInterpretationDemo(),JSON.stringify(selection));
  const raw=JSON.parse(source.plan_json()) as PerformancePlan;source.free();
  const omitted=raw.hits.find(h=>h.instrument==='snare')!.sourceEventId!;
  const arranged=new Performance(makeInterpretationDemo(),JSON.stringify({...selection,omittedSourceEventIds:[omitted]}));
  const calibrated=Performance.from_json_with_profile(arranged.plan_json(),JSON.stringify(profile));arranged.free();
  const plan=JSON.parse(calibrated.plan_json()) as PerformancePlan;calibrated.free();
  return {rig,profile,plan};
}

it('leaves the active evaluator unchanged while a replacement is prepared or abandoned',async()=>{
  const {rig,profile,plan}=fixture(),engine=new EngineClient(localWorker);
  try{
    await engine.restorePlan(plan,profile,plan.kit);
    const before=Array.from(engine.evaluate(.5)!);
    const changed:RigProfile={...structuredClone(profile),id:'replacement'};
    changed.roots[0][1]+=.02;
    const staged=await engine.prepareSetup(changed,plan.kit);
    expect(Array.from(engine.evaluate(.5)!)).toEqual(before);
    staged.dispose();
    expect(Array.from(engine.evaluate(.5)!)).toEqual(before);
    expect(()=>staged.commit()).toThrow('consumed');
  }finally{engine.dispose();rig.dispose();}
});

it('keeps exact source directives through fit commit and restores saved setup without another fit',async()=>{
  const {rig,profile,plan}=fixture(),engine=new EngineClient(localWorker);
  try{
    await engine.restorePlan(plan,profile,plan.kit);
    const savedPose=Array.from(engine.evaluate(.75)!);
    const moved=structuredClone(plan.kit);moved.find(p=>p.id==='snare')!.position[2]+=.025;
    const staged=await engine.prepareSetup(profile,moved),next=staged.commit()!;
    expect(next.sourceEvents).toEqual(plan.sourceEvents);
    expect(next.omittedSourceEventIds).toEqual(plan.omittedSourceEventIds);
    expect(next.hits.map(h=>[h.sourceEventId,h.time,h.velocity])).toEqual(plan.hits.map(h=>[h.sourceEventId,h.time,h.velocity]));
    staged.dispose(); // a committed evaluator stays owned by the engine
    expect(engine.evaluate(.75)!.length).toBe(77);
    await engine.restorePlan(plan,profile,plan.kit);
    expect(Array.from(engine.evaluate(.75)!)).toEqual(savedPose);
  }finally{engine.dispose();rig.dispose();}
});

it('rejects invalid setup before replacing a playable performance',async()=>{
  const {rig,profile,plan}=fixture(),engine=new EngineClient(localWorker);
  try{
    await engine.restorePlan(plan,profile,plan.kit);
    const before=Array.from(engine.evaluate(1)!);
    const invalid=structuredClone(profile);invalid.lengths[0][0]=-1;
    await expect(engine.prepareSetup(invalid,plan.kit)).rejects.toThrow();
    expect(Array.from(engine.evaluate(1)!)).toEqual(before);
  }finally{engine.dispose();rig.dispose();}
});


it('reuses a validated candidate when audio or project preflight is followed by commit',async()=>{
  const {rig,profile,plan}=fixture(),factory=vi.fn(localWorker),engine=new EngineClient(factory);
  try{
    const preflight=await engine.restorePlan(plan,profile,plan.kit,false);
    expect(engine.evaluate(.5)).toBeUndefined();
    await engine.restorePlan(plan,profile,plan.kit);
    expect(factory).toHaveBeenCalledOnce();
    expect(engine.evaluate(.5)?.length).toBe(77);
    expect(preflight.sourceEvents).toEqual(plan.sourceEvents);
  }finally{engine.dispose();rig.dispose();}
});

it('cannot commit a prepared setup after cancellation or supersession',async()=>{
  const {rig,profile,plan}=fixture(),engine=new EngineClient(localWorker);
  try{
    await engine.restorePlan(plan,profile,plan.kit);
    const before=Array.from(engine.evaluate(.5)!);
    const staged=await engine.prepareSetup(profile,plan.kit);
    engine.cancel();
    expect(()=>staged.commit()).toThrow('취소');
    expect(Array.from(engine.evaluate(.5)!)).toEqual(before);
    staged.dispose();
  }finally{engine.dispose();rig.dispose();}
});

it('cancels background planning before it can replace the active evaluator',async()=>{
  const {rig,profile,plan}=fixture(),engine=new EngineClient(localWorker);
  try{
    await engine.restorePlan(plan,profile,plan.kit);
    const before=Array.from(engine.evaluate(.5)!);
    const changed=structuredClone(profile);changed.id='candidate';
    const pending=engine.prepareSetup(changed,plan.kit);
    engine.cancel();
    await expect(pending).rejects.toMatchObject({name:'AbortError'});
    expect(Array.from(engine.evaluate(.5)!)).toEqual(before);
  }finally{engine.dispose();rig.dispose();}
});

it('prepares MIDI and body motion once, then installs the identical candidate after audio preflight',async()=>{
  const {rig,profile,plan}=fixture(),factory=vi.fn(localWorker),engine=new EngineClient(factory);
  try{
    await engine.setSetup(profile,plan.kit);
    const candidate=await engine.plan(makeInterpretationDemo(),{tracks:[1],channel:0,mappingProfile:'logicDkdGm',countInNote75:true,hihatPolicy:'shortOpenClose',hihatOpenness:.2},false);
    expect(engine.evaluate(.5)).toBeUndefined();
    const committed=await engine.restorePlan(candidate,profile,plan.kit);
    expect(factory).toHaveBeenCalledOnce();
    expect(committed).toEqual(candidate);
    expect(engine.evaluate(.5)?.length).toBe(77);
  }finally{engine.dispose();rig.dispose();}
});
