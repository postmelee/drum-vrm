import init, { Performance, fit_kit } from './wasm/drum_engine';
import type { KitPiece, KitFitProposal, MidiInfo, PerformancePlan, RigProfile, Selection } from './types';
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
  async plan(bytes: Uint8Array, selection: Selection) {
    const plan = await this.request<PerformancePlan>('plan', bytes, selection);
    this.ready ??= init();
    try { await this.ready; } catch (error) { this.ready = null; throw error; }
    const laidOut = this.layout ? applyKitLayout(plan, this.layout) : plan;
    const performance = this.profile
      ? Performance.from_json_with_profile(JSON.stringify(laidOut), JSON.stringify(this.profile))
      : Performance.from_json(JSON.stringify(laidOut));
    const calibrated = JSON.parse(performance.plan_json()) as PerformancePlan;
    this.sourcePlan = plan;
    this.performance?.free(); this.performance = performance;
    return calibrated;
  }
  evaluate(time: number) { return this.profile ? this.performance?.evaluate_rig_at(time) : this.performance?.evaluate_at(time); }
  clearPerformance() { this.performance?.free(); this.performance = null; this.sourcePlan = null; }
  dispose() { this.cancel(); this.clearPerformance(); }
}
