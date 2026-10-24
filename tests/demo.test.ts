import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { makeDemo } from '../src/demo';
import { initSync, inspect_midi, Performance } from '../src/wasm/drum_engine';
import { outputFrameCount, type MidiInfo, type PerformancePlan } from '../src/types';

beforeAll(() => {
  initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) });
});

it('the original 30-second demo crosses the actual WASM boundary with complete expected events', () => {
  const bytes = makeDemo();
  const info = JSON.parse(inspect_midi(bytes)) as MidiInfo;
  expect(info.format).toBe(1);
  expect(info.ppq).toBe(480);
  expect(info.suggestedTracks).toEqual([1]);
  expect(info.tracks[1].channels).toEqual([9]);
  // 54 ordinary beats × 3 hits + 6 fill beats × 4 hits + 4 extra kicks + 3 pedal hits.
  expect(info.tracks[1].noteCount).toBe(193);
  const performance = new Performance(bytes, JSON.stringify({ tracks: [1], channel: 9 }));
  try {
    const plan = JSON.parse(performance.plan_json()) as PerformancePlan;
    expect(plan.duration).toBe(30);
    expect(plan.hits).toHaveLength(193);
    expect(plan.hits[0].time).toBe(0);
    expect(plan.hits.every(hit => hit.channel === 9 && hit.velocity > 0 && hit.time < 30)).toBe(true);
    expect(new Set(plan.hits.map(hit => hit.instrument))).toEqual(new Set([
      'kick', 'snare', 'hihat', 'openHat', 'pedalHat', 'crash', 'tomHigh', 'tomMid', 'tomLow',
    ]));
    expect(outputFrameCount(plan)).toBe(923);
    const pose = performance.evaluate_at(0);
    expect(pose).toHaveLength(57);
    expect(pose.every(Number.isFinite)).toBe(true);
    // The JS array is owned; another evaluation must not overwrite an earlier pose.
    const original = Array.from(pose);
    performance.evaluate_at(15);
    expect(Array.from(pose)).toEqual(original);
  } finally {
    performance.free();
  }
});
