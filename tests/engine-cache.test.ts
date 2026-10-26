import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { EngineClient } from '../src/engine';
import { executeEngineRequest } from '../src/engine-worker-actions';
import type { EngineRequest } from '../src/engine-protocol';
import { makeDemo } from '../src/demo';
import { NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import { initSync, plan_midi } from '../src/wasm/drum_engine';
import type { PerformancePlan, Selection } from '../src/types';

beforeAll(()=>initSync({module:readFileSync(new URL('../src/wasm/drum_engine_bg.wasm',import.meta.url))}));

// Record expensive requests, not Worker construction: an idle Worker is reused.
function fixture(){
  const requests:EngineRequest['kind'][]=[];
  class LocalWorker {
    onmessage:((event:MessageEvent)=>void)|null=null;
    terminated=false;
    terminate(){this.terminated=true;}
    postMessage(payload:EngineRequest & {requestId:number}){
      const request=structuredClone(payload);requests.push(request.kind);
      queueMicrotask(()=>{
        if(this.terminated)return;
        try{this.onmessage?.({data:{requestId:request.requestId,ok:true,json:executeEngineRequest(request)}} as MessageEvent);}
        catch(error){this.onmessage?.({data:{requestId:request.requestId,ok:false,error:String(error)}} as MessageEvent);}
      });
    }
  }
  const rig=new NativeHumanoidRigAdapter(),profile=rig.getProfile();
  delete profile.bodyGeometry; // Geometry serialization has its own tests.
  const bytes=makeDemo(4),selection:Selection={tracks:[1],channel:9};
  const raw=JSON.parse(plan_midi(bytes,JSON.stringify(selection))) as PerformancePlan;
  const kit=raw.kit.map(piece=>{const next=structuredClone(piece);if(next.id!=='crashRight')delete next.instrument;return next;});
  const engine=new EngineClient(()=>new LocalWorker() as unknown as Worker);
  return {requests,rig,profile,bytes,selection,kit,engine};
}

it('installs the same finalized candidate without repeating planning after audio preflight',async()=>{
  const {requests,rig,profile,bytes,selection,kit,engine}=fixture();
  try{
    await engine.setSetup(profile,kit);
    const candidate=await engine.plan(bytes,selection,false);
    expect(engine.evaluate(.5)).toBeUndefined();
    // Resolved articulation points differ from the anchors restorePlan accepts.
    expect(candidate.hits.some(hit=>JSON.stringify(hit.target)!==JSON.stringify(kit.find(piece=>piece.id===(hit.pieceId??hit.instrument))!.position))).toBe(true);
    expect(candidate.kit.every(piece=>piece.instrument!==undefined)).toBe(true);
    const committed=await engine.restorePlan(candidate,profile,kit);
    expect(requests).toEqual(['plan']);
    expect(committed).toEqual(candidate);
    const pose=Array.from(engine.evaluate(.5)!);
    engine.evaluate(.9);engine.evaluate(.1);
    expect(Array.from(engine.evaluate(.5)!)).toEqual(pose);
  }finally{engine.dispose();rig.dispose();}
});

it('requires full source, profile and requested layout identity for candidate reuse',async()=>{
  const {requests,rig,profile,bytes,selection,kit,engine}=fixture();
  try{
    await engine.setSetup(profile,kit);
    const candidate=await engine.plan(bytes,selection,false);
    const changedSource=structuredClone(candidate);
    const hand=changedSource.hits.find(hit=>hit.limb===0||hit.limb===1)!;
    changedSource.handOverrides={...changedSource.handOverrides,[hand.sourceEventId!]:hand.limb as 0|1};
    const sourceResult=await engine.restorePlan(changedSource,profile,kit);
    expect(requests).toEqual(['plan','prepare']);
    expect(sourceResult.handOverrides).toEqual(changedSource.handOverrides);
    await engine.restorePlan(changedSource,profile,kit);
    expect(requests).toEqual(['plan','prepare']);
    const changedProfile={...structuredClone(profile),id:'different-profile'};
    await engine.restorePlan(changedSource,changedProfile,kit);
    expect(requests).toEqual(['plan','prepare','prepare']);
    const changedKit=structuredClone(kit);changedKit.find(piece=>piece.id==='snare')!.position[2]+=.01;
    await engine.restorePlan(changedSource,changedProfile,changedKit);
    expect(requests).toEqual(['plan','prepare','prepare','prepare']);
  }finally{engine.dispose();rig.dispose();}
},20_000);

it('drops candidate reuse after failed replacement or explicit cancellation',async()=>{
  const {requests,rig,profile,bytes,selection,kit,engine}=fixture();
  try{
    await engine.setSetup(profile,kit);
    const candidate=await engine.plan(bytes,selection,false);
    const invalid=structuredClone(profile);invalid.lengths[0][0]=-1;
    await expect(engine.restorePlan(candidate,invalid,kit)).rejects.toThrow();
    expect(engine.evaluate(.5)).toBeUndefined();
    await engine.restorePlan(candidate,profile,kit);
    expect(requests).toEqual(['plan','prepare','prepare']);
    const pose=Array.from(engine.evaluate(.5)!);
    engine.cancel();
    await engine.restorePlan(candidate,profile,kit);
    expect(requests).toEqual(['plan','prepare','prepare','prepare']);
    expect(Array.from(engine.evaluate(.5)!)).toEqual(pose);
  }finally{engine.dispose();rig.dispose();}
},20_000);

it('captures restore source directives before async installation and later setup replanning',async()=>{
  const {rig,profile,bytes,selection,kit,engine}=fixture();
  try{
    await engine.setSetup(profile,kit);
    const candidate=await engine.plan(bytes,selection,false);
    const expected=structuredClone(candidate.omittedSourceEventIds);
    const pending=engine.restorePlan(candidate,profile,kit);
    // Audio/project UI may retain the caller's editable plan. The pending
    // transaction must keep the same source snapshot as its prepared evaluator.
    candidate.omittedSourceEventIds!.push(candidate.hits[0].sourceEventId!);
    const restored=await pending;
    expect(restored.omittedSourceEventIds).toEqual(expected);
    const next=await engine.prepareSetup(profile,kit);
    try{expect(next.plan!.omittedSourceEventIds).toEqual(expected);}
    finally{next.dispose();}
  }finally{engine.dispose();rig.dispose();}
});
