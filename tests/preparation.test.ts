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

// This boundary uses the same snapshot operation as MIDI/demo load and replan.
// The callback is intentionally asynchronous so a later UI setup cannot leak
// into an earlier accepted request.
describe('accepted setup during score preparation', () => {
  it('keeps kit dimensions, measured geometry and calibrated body fixed across different scores', async () => {
    const { preparePerformanceInSetup } = await import('../src/preparation');
    const { NativeHumanoidRigAdapter } = await import('../src/rig-humanoid');
    const { DEFAULT_KIT } = await import('../src/kit-default');
    const rig = new NativeHumanoidRigAdapter();
    const kit = structuredClone(DEFAULT_KIT);
    kit[1].geometry = { version: 1, source: 'procedural', referenceRadius: kit[1].radius,
      body: {center: [0, -.06, 0], halfExtents: [.18, .06, .18]}, surface: {center: [0, 0, 0], normal: [0, 1, 0], radius: .18} };
    const setup = { profile: rig.getProfile(), kit }, before = structuredClone(setup), seat = rig.getCalibration();
    const seen: unknown[] = [];
    try {
      for (const duration of [8, 30, 3]) {
        const source = plan(duration), pending = deferred<PerformancePlan>();
        const result = preparePerformanceInSetup({setup, candidate: () => pending.promise, audio: () => 'audio', current: () => true,
          commit: async (candidate, profile, fixedKit) => {
            seen.push(structuredClone({profile, kit: fixedKit}));
            // Simulate downstream preparation attaching per-score data.
            fixedKit[1].radius = .01;
            return {...candidate, kit: structuredClone(before.kit)};
          },
        });
        pending.resolve(source);
        expect((await result)!.plan.duration).toBe(duration);
        expect(setup).toEqual(before);
      }
      expect(seen).toEqual([before, before, before]);
      expect(rig.getCalibration()).toEqual(seat);
    } finally { rig.dispose(); }
  });

  it('captures setup before analysis starts and never commits a canceled request', async () => {
    const { preparePerformanceInSetup } = await import('../src/preparation');
    const { DEFAULT_KIT } = await import('../src/kit-default');
    const setup = {profile: {id: 'original'} as import('../src/types').RigProfile, kit: structuredClone(DEFAULT_KIT)};
    const before = structuredClone(setup), pending = deferred<PerformancePlan>(), commit = vi.fn(async (candidate, profile, kit) => ({...candidate, kit, profile}));
    const result = preparePerformanceInSetup({setup, candidate: () => pending.promise, audio: () => 'audio', commit, current: () => true});
    setup.profile.id = 'later character'; setup.kit[1].radius *= .8;
    pending.resolve(plan(8)); await result;
    expect(commit).toHaveBeenCalledWith(expect.anything(), before.profile, before.kit);
    const canceledCommit = vi.fn(async (candidate: PerformancePlan) => candidate);
    expect(await preparePerformanceInSetup({setup, candidate: async () => plan(30), audio: () => 'audio', commit: canceledCommit, current: () => false})).toBeNull();
    expect(canceledCommit).not.toHaveBeenCalled();
  });
});
