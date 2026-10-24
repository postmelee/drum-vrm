import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const state = vi.hoisted(() => ({ supportPending: false, starts: 0, cancellations: 0 }));
vi.mock('mediabunny', async importOriginal => {
  const original = await importOriginal<typeof import('mediabunny')>();
  return {
    ...original,
    canEncodeVideo: () => state.supportPending ? new Promise<boolean>(() => {}) : Promise.resolve(true),
    canEncodeAudio: () => Promise.resolve(true),
    Output: class {
      state = 'pending';
      target = { buffer: null };
      addVideoTrack() {}
      addAudioTrack() {}
      start() { state.starts++; return new Promise<void>(() => {}); }
      async cancel() { state.cancellations++; this.state = 'canceled'; }
    },
    CanvasSource: class {},
    AudioBufferSource: class {},
  };
});

import { probeExport } from '../src/export';

beforeEach(() => {
  state.supportPending = false; state.starts = 0; state.cancellations = 0;
  vi.stubGlobal('VideoEncoder', class {});
  vi.stubGlobal('AudioEncoder', class {});
  vi.stubGlobal('document', {
    createElement: () => ({ width: 0, height: 0, getContext: () => ({ fillStyle: '', fillRect() {} }) }),
  });
});
afterEach(() => vi.unstubAllGlobals());

describe('export capability-probe cancellation', () => {
  it('stops before creating an encoder for an already cancelled request', async () => {
    const controller = new AbortController(); controller.abort();
    await expect(probeExport(false, controller.signal)).rejects.toMatchObject({ name: 'AbortError' });
    expect(state.starts).toBe(0);
  });

  it('returns cancellation even when the browser support query has not settled', async () => {
    state.supportPending = true;
    const controller = new AbortController();
    const result = probeExport(false, controller.signal);
    controller.abort();
    await expect(result).rejects.toMatchObject({ name: 'AbortError' });
    expect(state.starts).toBe(0);
  });

  it('cancels an active trial output and does not fall through to another codec', async () => {
    const controller = new AbortController();
    const result = probeExport(false, controller.signal);
    await vi.waitFor(() => expect(state.starts).toBe(1));
    controller.abort();
    await expect(result).rejects.toMatchObject({ name: 'AbortError' });
    expect(state.cancellations).toBe(1);
    expect(state.starts).toBe(1);
  });
});
