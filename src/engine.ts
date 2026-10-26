import { Performance, ready_pose } from './wasm/drum_engine';
import { initializeWasm } from './wasm-runtime';
import type { PoseInspection, KitPiece, KitFitProposal, MidiInfo, PerformancePlan, RigProfile, Selection } from './types';
import { applyKitLayout, validateKitLayout } from './kit-calibration';
import { FitClient } from './fit-client';
import type { EngineRequest, PreparedResponse } from './engine-protocol';

export class EngineClient {
  private readonly fitter = new FitClient();
  private worker: Worker | null = null;
  private reject: ((reason: Error) => void) | null = null;
  private generation = 0;
  private requestId = 0;
  private performance: Performance | null = null;
  private sourcePlan: PerformancePlan | null = null;
  private profile: RigProfile | null = null;
  private layout: KitPiece[] | null = null;
  private cache: { keys:string[]; response:PreparedResponse } | null = null;
  constructor(private readonly createWorker=()=>new Worker(new URL('./engine.worker.ts',import.meta.url),{type:'module'})) {}

  async fitKit(profile: RigProfile, kit: readonly KitPiece[]): Promise<KitFitProposal> { return this.fitter.fit(profile,validateKitLayout(kit)); }
  cancelFit() { this.fitter.cancel(); }
  private async initialize() {
    await initializeWasm();
  }
  async readyPose(profile: RigProfile, kit: readonly KitPiece[]): Promise<Float32Array> {
    await this.initialize();
    return ready_pose(JSON.stringify(profile),JSON.stringify(validateKitLayout(kit)));
  }
  setProfile(profile: RigProfile) { return this.setSetup(profile,this.layout); }
  private assertCurrent(generation:number) {
    if(generation!==this.generation)throw new DOMException('준비를 취소했습니다.','AbortError');
  }
  private stopWorker() {
    this.worker?.terminate();this.worker=null;
    const reject=this.reject;this.reject=null;reject?.(new DOMException('준비를 취소했습니다.','AbortError'));
  }
  private begin() { this.stopWorker();return ++this.generation; }
  cancel() { ++this.generation;this.stopWorker();this.cache=null; }
  private request<T>(payload:EngineRequest,generation:number):Promise<T> {
    this.assertCurrent(generation);
    const requestId=++this.requestId;
    return new Promise((resolve,reject)=>{
      const worker=this.createWorker();this.worker=worker;this.reject=reject;
      const done=()=>{worker.terminate();if(this.worker===worker){this.worker=null;this.reject=null;}};
      const fail=(error:Error)=>{if(this.worker!==worker)return;done();reject(error);};
      worker.onmessage=({data})=>{
        if(this.worker!==worker||generation!==this.generation)return;
        if(!data||data.requestId!==requestId)return;
        done();
        try{if(data.ok)resolve(JSON.parse(data.json));else reject(new Error(data.error));}catch(error){reject(error);}
      };
      worker.onerror=()=>fail(new Error('분석 엔진을 실행하지 못했습니다. 다시 시도해 주세요.'));
      worker.onmessageerror=()=>fail(new Error('분석 엔진 응답을 읽지 못했습니다. 다시 시도해 주세요.'));
      try{worker.postMessage({...payload,requestId});}catch(error){fail(error instanceof Error?error:new Error(String(error)));}
    });
  }
  inspect(bytes:Uint8Array){return this.request<MidiInfo>({kind:'inspect',bytes},this.begin());}
  async validateProjectSource(bytes:Uint8Array,selection:Selection,stored:PerformancePlan):Promise<void> {
    const generation=this.begin(),parsed=await this.request<PerformancePlan>({kind:'source',bytes,selection},generation);this.assertCurrent(generation);
    for(const key of ['midiHash','sourceEvents','tempoMap','timeSignatures','duration','ppq','mappingProfile','mappingVersion','countInNote75','hihatPolicy','hihatOpenness','controls','interpretations','omittedSourceEventIds','handOverrides'] as const){
      if(JSON.stringify(parsed[key])!==JSON.stringify(stored[key]))throw new Error('프로젝트의 원본·해석과 다시 연결한 MIDI가 일치하지 않습니다.');
    }
  }
  private key(plan:PerformancePlan,profile:RigProfile|null){return JSON.stringify([plan,profile]);}
  private async prepareEvaluator(source:PerformancePlan,profile:RigProfile|null,generation:number) {
    const key=this.key(source,profile);
    const response=this.cache?.keys.includes(key)?this.cache.response:await this.request<PreparedResponse>({kind:'prepare',plan:source,profile},generation);
    this.assertCurrent(generation);await this.initialize();this.assertCurrent(generation);
    // The opaque worker transfer is validated by Rust against this source/body.
    const candidate=Performance.from_prepared_json(response.prepared,source.midiHash??'',profile?JSON.stringify(profile):'');
    try{
      const plan=JSON.parse(candidate.plan_json()) as PerformancePlan;
      this.cache={keys:[key,this.key(plan,profile)],response};
      return {candidate,plan};
    }catch(error){candidate.free();throw error;}
  }
  /** Expensive planning completes in a worker before the scene is changed.
   * Commit is synchronous and rejects a canceled/superseded candidate. */
  async prepareSetup(profile:RigProfile,layout:readonly KitPiece[]|null) {
    const generation=this.begin(),validated=layout?validateKitLayout(layout):null,nextProfile=structuredClone(profile);
    const source=this.sourcePlan&&validated?applyKitLayout(this.sourcePlan,validated):this.sourcePlan;
    const prepared=source?await this.prepareEvaluator(source,nextProfile,generation):null;
    try{this.assertCurrent(generation);}catch(error){prepared?.candidate.free();throw error;}
    let pending=true;
    const dispose=()=>{if(pending){pending=false;prepared?.candidate.free();}};
    return {
      plan:prepared?.plan??null,
      commit:()=>{
        if(!pending)throw new Error('Prepared setup already consumed');
        try{this.assertCurrent(generation);}catch(error){dispose();throw error;}
        pending=false;this.profile=nextProfile;this.layout=validated;
        if(prepared){this.performance?.free();this.performance=prepared.candidate;}
        return prepared?.plan??null;
      },dispose,
    };
  }
  async setSetup(profile:RigProfile,layout:readonly KitPiece[]|null){return (await this.prepareSetup(profile,layout)).commit();}
  async plan(bytes:Uint8Array,selection:Selection,commit=true) {
    const generation=this.begin(),profile=this.profile?structuredClone(this.profile):null,layout=this.layout?structuredClone(this.layout):null;
    const response=await this.request<PreparedResponse>({kind:'plan',bytes,selection:structuredClone(selection),profile,layout},generation);
    this.assertCurrent(generation);await this.initialize();this.assertCurrent(generation);
    const candidate=Performance.from_prepared_json(response.prepared,response.plan.midiHash??'',profile?JSON.stringify(profile):'');
    try{
      const plan=JSON.parse(candidate.plan_json()) as PerformancePlan;
      this.cache={keys:[this.key(plan,profile)],response};
      if(commit){this.sourcePlan=structuredClone(plan);this.performance?.free();this.performance=candidate;}else candidate.free();
      return plan;
    }catch(error){candidate.free();throw error;}
  }
  /** Exact saved/source plans are rebuilt in the worker, never automatically fit.
   * Reuse an unchanged prepared candidate after file/audio preflight. */
  async restorePlan(plan:PerformancePlan,profile:RigProfile,layout:readonly KitPiece[],commit=true):Promise<PerformancePlan> {
    const generation=this.begin(),kit=validateKitLayout(layout),source=applyKitLayout(plan,kit),nextProfile=structuredClone(profile);
    const {candidate,plan:restored}=await this.prepareEvaluator(source,nextProfile,generation);
    try{
      this.assertCurrent(generation);
      if(plan.engineVersion!==restored.engineVersion||plan.plannerVersion!==restored.plannerVersion)throw new Error('프로젝트 엔진 버전이 다릅니다. 원본 MIDI를 다시 준비해 주세요.');
      if(commit){this.performance?.free();this.performance=candidate;this.sourcePlan=structuredClone(source);this.profile=nextProfile;this.layout=kit;}else candidate.free();
      return restored;
    }catch(error){candidate.free();throw error;}
  }
  evaluate(time:number){return this.profile?this.performance?.evaluate_rig_at(time):this.performance?.evaluate_at(time);}
  inspectPose(time:number):PoseInspection|null{return this.performance?JSON.parse(this.performance.inspect_pose_at(time)):null;}
  clearPerformance(){this.cancel();this.performance?.free();this.performance=null;this.sourcePlan=null;}
  dispose(){this.cancel();this.fitter.dispose();this.clearPerformance();}
}
