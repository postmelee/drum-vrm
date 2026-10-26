import './worker-messages';
import { engineMessage } from './engine-messages';
import { msg } from './i18n';
import type { SynthPcm } from './audio';
import type { PerformancePlan } from './types';
/** Terminating this worker makes even a dense five-minute synthesis cancellable. */
export class SynthesisClient {
  private worker: Worker | null = null;
  private reject: ((error: Error) => void) | null = null;
  cancel() {
    this.worker?.terminate(); this.worker = null;
    this.reject?.(new DOMException(msg("Preparation canceled.", "준비를 취소했습니다."), 'AbortError')); this.reject = null;
  }
  run(plan: PerformancePlan): Promise<AudioBuffer> {
    this.cancel();
    return new Promise((resolve, reject) => {
      const worker = new Worker(new URL('./audio.worker.ts', import.meta.url), {type: 'module'});
      this.worker = worker; this.reject = reject;
      const done = () => { worker.terminate(); if (this.worker === worker) { this.worker = null; this.reject = null; } };
      worker.onmessage = ({data}) => {
        done();
        try {
          if (!data.ok) throw new Error(engineMessage(data.error));
          const pcm = data.pcm as SynthPcm;
          const buffer = new AudioBuffer({length: pcm.channels[0].length, sampleRate: pcm.sampleRate, numberOfChannels: 2});
          pcm.channels.forEach((channel, i) => buffer.copyToChannel(channel, i));
          resolve(buffer);
        } catch (error) { reject(error); }
      };
      worker.onerror = () => { done(); reject(new Error(msg("Audio preparation failed.", "오디오 준비 중 오류가 발생했습니다."))); };
      worker.postMessage(plan);
    });
  }
}
