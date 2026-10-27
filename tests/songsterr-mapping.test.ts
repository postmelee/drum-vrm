import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { initSync, inspect_midi, plan_midi, Performance } from '../src/wasm/drum_engine';
import { makeSongsterrStudy } from './songsterr-fixture';
import type { PerformancePlan } from '../src/types';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));

it.each([0, 1] as const)('imports format %i with explicit mapping and retains raw MIDI evidence', format => {
  const bytes = makeSongsterrStudy(format), before = Uint8Array.from(bytes);
  const info = JSON.parse(inspect_midi(bytes));
  expect(info.suggestedTracks).toEqual([format]);
  const selection = { tracks: info.suggestedTracks, channel: 9 };
  expect(() => plan_midi(bytes, JSON.stringify(selection))).toThrow();
  const plan: PerformancePlan = JSON.parse(plan_midi(bytes, JSON.stringify({ ...selection, mappingProfile: 'songsterrDownloader' })));
  expect(plan.hits.map(h => h.instrument)).toEqual(['snare', 'hihat', 'openHat', 'kick', 'tomMid', 'crash', 'tomLow']);
  expect(plan.hits.map(h => [h.note, h.timeUs, h.velocity])).toEqual(
    [0, 3, 5, 8, 12, 22, 87].map((key, i) => [key, i * 500_000, 80 + i]));
  expect(plan.sourceEvents).toHaveLength(14);
  expect(plan.sourceEvents!.filter(e => e.message.kind === 'noteOff')).toHaveLength(7);
  expect(plan.diagnostics.filter(d => d.code === 'UNMAPPED_NOTE')).toEqual([]);
  expect(bytes).toEqual(before);
  const run = Performance.from_json(JSON.stringify(plan));
  try {
    const times = [0, .5, 1.25, 2.8], poses = times.map(t => run.evaluate_at(t));
    for (const index of [3, 0, 2, 1]) expect(run.evaluate_at(times[index])).toEqual(poses[index]);
    expect(JSON.parse(run.plan_json()).sourceEvents).toEqual(plan.sourceEvents);
  } finally { run.free(); }
});
