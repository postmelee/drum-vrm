import { describe, expect, it } from 'vitest';
import { createTimelineGrid, songTimeToTick, tickToSongTime } from '../src/timeline';
import type { PerformancePlan } from '../src/types';

const plan: PerformancePlan = {
  schemaVersion: 1, duration: 6, hits: [], diagnostics: [], kit: [], tempoCount: 2,
  ppq: 480, endTick: 3360,
  tempoMap: [
    { tick: 0, time: 0, microsecondsPerQuarter: 500_000 },
    { tick: 960, time: 1, microsecondsPerQuarter: 1_000_000 },
  ],
  timeSignatures: [{ tick: 0, numerator: 4, denominator: 4 }, { tick: 1920, numerator: 3, denominator: 4 }],
};

describe('MIDI piano-roll musical time', () => {
  it('keeps source ticks correct across tempo changes in both directions', () => {
    const samples = [[0, 0], [480, .5], [960, 1], [1440, 2], [1920, 3], [3360, 6]];
    for (const [tick, time] of samples) {
      expect(tickToSongTime(plan, tick)).toBeCloseTo(time, 10);
      expect(songTimeToTick(plan, time)).toBeCloseTo(tick, 7);
    }
  });

  it('places bar one after pre-roll and respects a 4/4 to 3/4 meter change', () => {
    const grid = createTimelineGrid(plan);
    expect(grid.filter(mark => mark.major).map(mark => [mark.bar, mark.tick, mark.outputTime])).toEqual([
      [1, 0, .5], [2, 1920, 3.5], [3, 3360, 6.5],
    ]);
    expect(grid.find(mark => mark.tick === 1440)).toMatchObject({ bar: 1, beat: 4, outputTime: 2.5 });
    expect(grid.find(mark => mark.tick === 1920)?.signature).toBe('3/4');
    const endsAtChange = createTimelineGrid({ ...plan, duration: 3, endTick: 1920 });
    expect(endsAtChange.filter(mark => mark.tick === 1920)).toHaveLength(1);
    expect(endsAtChange.find(mark => mark.tick === 1920)?.signature).toBe('3/4');
  });

  it('counts a partial bar once when the MIDI changes meter off a bar boundary', () => {
    const irregular = { ...plan, timeSignatures: [{ tick: 0, numerator: 4, denominator: 4 }, { tick: 1440, numerator: 6, denominator: 8 }] };
    const grid = createTimelineGrid(irregular);
    expect(grid.find(mark => mark.tick === 1440)).toMatchObject({ bar: 2, beat: 1, signature: '6/8' });
    expect(grid.find(mark => mark.tick === 1680)).toMatchObject({ bar: 2, beat: 2, outputTime: 3 });
    expect(grid.find(mark => mark.tick === 2880)).toMatchObject({ bar: 3, beat: 1 });
  });

  it('keeps the absolute bar number while horizontally scrolled', () => {
    const grid = createTimelineGrid(plan, 3.4, 5.6);
    expect(grid[0]).toMatchObject({ bar: 2, beat: 1, tick: 1920 });
    expect(grid.every(mark => mark.outputTime >= 3.4 && mark.outputTime <= 5.6)).toBe(true);
  });

  it('bounds grid allocation for an extreme but legal MIDI tempo', () => {
    const dense = { ...plan, duration: 300, endTick: 144_000_000_000, tempoMap: [{ tick: 0, time: 0, microsecondsPerQuarter: 1 }], timeSignatures: [{ tick: 0, numerator: 4, denominator: 4 }] };
    const grid = createTimelineGrid(dense, 0, 301, 80);
    expect(grid.length).toBeLessThanOrEqual(82);
    expect(grid.every(mark => mark.major)).toBe(true);
  });

  it('uses seconds for older plans without pretending they have a known tempo', () => {
    const legacy = { ...plan, ppq: undefined, tempoMap: undefined, timeSignatures: undefined };
    const grid = createTimelineGrid(legacy);
    expect(grid[0]).toEqual({ outputTime: .5, label: '0s', major: true });
    expect(grid.every(mark => mark.bar === undefined)).toBe(true);
  });
});
