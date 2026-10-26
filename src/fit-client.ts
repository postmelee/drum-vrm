import { engineMessage } from './engine-messages';
import { msg } from './i18n';
import type { KitFitProposal, KitPiece, RigProfile } from './types';

export interface FitRequest { requestId: number; profileJson: string; kitJson: string }
export type FitResponse = { requestId: number; ok: true; json: string } | { requestId: number; ok: false; error: string };

/** Reuse the initialized WASM worker between fits. A running Rust search cannot
 * process a cancel message, so cancellation terminates that worker outright. */
export class FitClient {
  private worker: Worker | null = null;
  private nextId = 0;
  private disposed = false;
  private pending: { id: number; resolve: (proposal: KitFitProposal) => void; reject: (error: Error) => void } | null = null;

  constructor(private readonly createWorker = () => new Worker(new URL('./fit.worker.ts', import.meta.url), { type: 'module' })) {}

  async fit(profile: RigProfile, kit: readonly KitPiece[]): Promise<KitFitProposal> {
    if (this.disposed) throw new Error('The fit engine has been disposed.');
    // Capture inputs before yielding; subsequent scene edits cannot change them.
    const request: FitRequest = { requestId: ++this.nextId, profileJson: JSON.stringify(profile), kitJson: JSON.stringify(kit) };
    if (this.pending) this.cancel();
    return new Promise((resolve, reject) => {
      const worker = this.worker ?? this.startWorker();
      this.pending = { id: request.requestId, resolve, reject };
      try { worker.postMessage(request); }
      catch (error) { this.fail(worker, error instanceof Error ? error : new Error(String(error))); }
    });
  }

  cancel() {
    this.worker?.terminate(); this.worker = null;
    const pending = this.pending; this.pending = null;
    pending?.reject(new DOMException(msg("Automatic fitting canceled.", "자동 맞춤을 취소했습니다."), 'AbortError'));
  }

  dispose() { this.disposed = true; this.cancel(); }

  private startWorker() {
    const worker = this.createWorker(); this.worker = worker;
    worker.onmessage = ({ data }: MessageEvent<FitResponse>) => {
      const pending = this.pending;
      if (this.worker !== worker || !pending) return;
      if (!data || typeof data.requestId !== 'number') { this.fail(worker, new Error(msg("The automatic-fit response could not be read. Try again.", "자동 맞춤 응답을 읽을 수 없습니다. 다시 시도해 주세요."))); return; }
      if (data.requestId !== pending.id) return;
      this.pending = null;
      try {
        if (data.ok === true && typeof data.json === 'string') pending.resolve(JSON.parse(data.json));
        else if (data.ok === false && typeof data.error === 'string') pending.reject(new Error(engineMessage(data.error)));
        else throw new Error(msg("The automatic-fit response could not be read. Try again.", "자동 맞춤 응답을 읽을 수 없습니다. 다시 시도해 주세요."));
      } catch (error) { pending.reject(error instanceof Error ? error : new Error(String(error))); }
    };
    worker.onerror = () => this.fail(worker, new Error(msg("The automatic-fit engine could not start or run. Try again.", "자동 맞춤 엔진을 시작하거나 실행하지 못했습니다. 다시 시도해 주세요.")));
    worker.onmessageerror = () => this.fail(worker, new Error(msg("The automatic-fit response could not be transferred. Try again.", "자동 맞춤 응답을 전달하지 못했습니다. 다시 시도해 주세요.")));
    return worker;
  }

  private fail(worker: Worker, error: Error) {
    if (this.worker !== worker) return;
    worker.terminate(); this.worker = null;
    const pending = this.pending; this.pending = null;
    pending?.reject(error);
  }
}
