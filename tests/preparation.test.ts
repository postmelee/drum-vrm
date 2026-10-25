import { describe, expect, it, vi } from 'vitest';
import { preparePerformance } from '../src/preparation';
import type { PerformancePlan } from '../src/types';

const plan = (duration: number): PerformancePlan => ({
  schemaVersion: 2, duration, hits: [], diagnostics: [], kit: [], tempoCount: 1,
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(accept => { resolve = accept; });
  return { promise, resolve };
}

function session() {
  const previous = { plan: plan(8), audio: 'previous audio', time: 3.25 };
  const active = { ...previous };
  const replacement = plan(12);
  const commit = vi.fn(async (candidate: PerformancePlan) => {
    active.plan = candidate;
    return candidate;
  });
  return { previous, active, replacement, commit };
}

describe('performance preparation transaction', () => {
  it('keeps the existing plan, audio and clock when source directives are invalid', async () => {
    const state = session();
    const audio = vi.fn(() => 'new audio');
    await expect(preparePerformance({
      candidate: async () => { throw new Error('Hand override references an omitted source event'); },
      audio, commit: state.commit, current: () => true,
    })).rejects.toThrow('omitted source event');
    expect(audio).not.toHaveBeenCalled();
    expect(state.commit).not.toHaveBeenCalled();
    expect(state.active).toEqual(state.previous);
  });

  it('does not replace the evaluator if preparing its audio fails', async () => {
    const state = session();
    await expect(preparePerformance({
      candidate: async () => state.replacement,
      audio: async () => { throw new Error('Audio preparation failed'); },
      commit: state.commit, current: () => true,
    })).rejects.toThrow('Audio preparation failed');
    expect(state.commit).not.toHaveBeenCalled();
    expect(state.active).toEqual(state.previous);
  });

  it('discards a candidate superseded before its analysis finishes', async () => {
    const state = session();
    const pending = deferred<PerformancePlan>();
    const audio = vi.fn(() => 'new audio');
    let current = true;
    const result = preparePerformance({
      candidate: () => pending.promise, audio, commit: state.commit, current: () => current,
    });
    current = false;
    pending.resolve(state.replacement);
    expect(await result).toBeNull();
    expect(audio).not.toHaveBeenCalled();
    expect(state.commit).not.toHaveBeenCalled();
    expect(state.active).toEqual(state.previous);
  });

  it('discards a completed audio buffer after its pending request is canceled', async () => {
    const state = session();
    const pending = deferred<string>();
    const started = deferred<void>();
    let current = true;
    const result = preparePerformance({
      candidate: async () => state.replacement,
      audio: candidate => {
        expect(candidate).toBe(state.replacement);
        started.resolve();
        return pending.promise;
      },
      commit: state.commit, current: () => current,
    });
    await started.promise;
    current = false;
    pending.resolve('discarded audio');
    expect(await result).toBeNull();
    expect(state.commit).not.toHaveBeenCalled();
    expect(state.active).toEqual(state.previous);
  });

  it('commits only when matching audio is ready and preserves the caller clock', async () => {
    const state = session();
    const pending = deferred<string>();
    const started = deferred<void>();
    const result = preparePerformance({
      candidate: async () => state.replacement,
      audio: candidate => {
        expect(candidate).toBe(state.replacement);
        started.resolve();
        return pending.promise;
      },
      commit: state.commit, current: () => true,
    });
    await started.promise;
    expect(state.active).toEqual(state.previous);
    expect(state.commit).not.toHaveBeenCalled();
    pending.resolve('matching new audio');
    expect(await result).toEqual({ plan: state.replacement, audio: 'matching new audio' });
    expect(state.commit).toHaveBeenCalledExactlyOnceWith(state.replacement);
    expect(state.active.plan).toBe(state.replacement);
    expect(state.active.time).toBe(state.previous.time);
  });
});
