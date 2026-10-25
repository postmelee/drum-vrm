import init, { Performance, fit_kit } from './wasm/drum_engine';
import type { PoseInspection, KitPiece, KitFitProposal, MidiInfo, PerformancePlan, RigProfile, Selection } from './types';
import { applyKitLayout, validateKitLayout } from './kit-calibration';
export class EngineClient {
  private worker: Worker | null = null;
  private reject: ((reason: Error) => void) | null = null;
  private ready: Promise<unknown> | null = null;
  private performance: Performance | null = null;
  private sourcePlan: PerformancePlan | null = null;
  private profile: RigProfile | null = null;
  private layout: KitPiece[] | null = null;
  async fitKit(profile: RigProfile, kit: readonly KitPiece[]): Promise<KitFitProposal> {
    this.ready ??= init();
    try{await this.ready;}catch(error){this.ready=null;throw error;}
    return JSON.parse(fit_kit(JSON.stringify(profile),JSON.stringify(kit)));
  }
  setProfile(profile: RigProfile): PerformancePlan | null {
    return this.setSetup(profile, this.layout);
  }
  setSetup(profile: RigProfile, layout: readonly KitPiece[] | null): PerformancePlan | null {
    // Construct before committing: a rejected profile retains the previous evaluator.
    const validated = layout ? validateKitLayout(layout) : null;
    const source = this.sourcePlan && validated ? applyKitLayout(this.sourcePlan, validated) : this.sourcePlan;
    const next = source ? Performance.from_json_with_profile(JSON.stringify(source), JSON.stringify(profile)) : null;
    const plan = next ? JSON.parse(next.plan_json()) as PerformancePlan : null;
    this.profile = structuredClone(profile); this.layout = validated;
    if (next) { this.performance?.free(); this.performance = next; }
    return plan;
  }
  cancel() {
    this.worker?.terminate(); this.worker = null;
    this.reject?.(new DOMException('분석을 취소했습니다.', 'AbortError'));
    this.reject = null;
  }
  private request<T>(kind: 'inspect'|'plan', bytes: Uint8Array, selection?: Selection): Promise<T> {
    this.cancel();
    return new Promise((resolve, reject) => {
      const worker = new Worker(new URL('./engine.worker.ts', import.meta.url), {type: 'module'});
      this.worker = worker; this.reject = reject;
      const done = () => { worker.terminate(); if (this.worker === worker) { this.worker = null; this.reject = null; } };
      worker.onmessage = ({data}) => { done(); try { if (data.ok) resolve(JSON.parse(data.json)); else reject(new Error(data.error)); } catch (error) { reject(error); } };
      worker.onerror = () => { done(); reject(new Error('분석 엔진을 시작하지 못했습니다. 페이지를 새로고침해 주세요.')); };
      worker.postMessage({kind, bytes, selection});
    });
  }
  inspect(bytes: Uint8Array) { return this.request<MidiInfo>('inspect', bytes); }
  async validateProjectSource(bytes:Uint8Array,selection:Selection,stored:PerformancePlan):Promise<void> {
    const parsed=await this.request<PerformancePlan>('plan',bytes,selection);
    for(const key of ['midiHash','sourceEvents','tempoMap','timeSignatures','duration','ppq','mappingProfile','mappingVersion','countInNote75','hihatPolicy','hihatOpenness','controls','interpretations','omittedSourceEventIds','handOverrides'] as const){
      if(JSON.stringify(parsed[key])!==JSON.stringify(stored[key]))throw new Error('프로젝트의 원본·해석과 다시 연결한 MIDI가 일치하지 않습니다.');
    }
  }
  async plan(bytes: Uint8Array, selection: Selection,commit=true) {
    const plan = await this.request<PerformancePlan>('plan', bytes, selection);
    this.ready ??= init();
    try { await this.ready; } catch (error) { this.ready = null; throw error; }
    const laidOut = this.layout ? applyKitLayout(plan, this.layout) : plan;
    const performance = this.profile
      ? Performance.from_json_with_profile(JSON.stringify(laidOut), JSON.stringify(this.profile))
      : Performance.from_json(JSON.stringify(laidOut));
    const calibrated = JSON.parse(performance.plan_json()) as PerformancePlan;
    if(commit){this.sourcePlan = plan;this.performance?.free(); this.performance = performance;}
    else performance.free();
    return calibrated;
  }
  /** Validate first; callers can preflight an entire local project before changing
   * the rendered rig. Commit replaces the evaluator only after Rust accepts it. */
  async restorePlan(plan:PerformancePlan,profile:RigProfile,layout:readonly KitPiece[],commit=true):Promise<PerformancePlan> {
    this.ready??=init();
    try{await this.ready;}catch(error){this.ready=null;throw error;}
    const kit=validateKitLayout(layout),source=applyKitLayout(plan,kit);
    const candidate=Performance.from_json_with_profile(JSON.stringify(source),JSON.stringify(profile));
    try {
      const restored=JSON.parse(candidate.plan_json()) as PerformancePlan;
      if(plan.engineVersion!==restored.engineVersion||plan.plannerVersion!==restored.plannerVersion)throw new Error('프로젝트 엔진 버전이 다릅니다. 원본 MIDI를 다시 준비해 주세요.');
      if(commit){this.performance?.free();this.performance=candidate;this.sourcePlan=structuredClone(source);this.profile=structuredClone(profile);this.layout=kit;}
      else candidate.free();
      return restored;
    } catch(error){candidate.free();throw error;}
  }
  evaluate(time: number) { return this.profile ? this.performance?.evaluate_rig_at(time) : this.performance?.evaluate_at(time); }
  inspectPose(time: number): PoseInspection | null { return this.performance ? JSON.parse(this.performance.inspect_pose_at(time)) : null; }
  clearPerformance() { this.performance?.free(); this.performance = null; this.sourcePlan = null; }
  dispose() { this.cancel(); this.clearPerformance(); }
}
