import init, { Performance } from './wasm/drum_engine';
import type { MidiInfo, PerformancePlan, Selection } from './types';
export class EngineClient {
  private worker: Worker | null = null;
  private reject: ((reason: Error) => void) | null = null;
  private ready: Promise<unknown> | null = null;
  private performance: Performance | null = null;
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
    const performance = Performance.from_json(JSON.stringify(plan));
    this.performance?.free(); this.performance = performance;
    return plan;
  }
  evaluate(time: number) { return this.performance?.evaluate_at(time); }
  dispose() { this.cancel(); this.performance?.free(); this.performance = null; }
}
