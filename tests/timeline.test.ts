import { describe, expect, it } from 'vitest';
import { createTimelineGrid, createTimelineNotes, songTimeToTick, tickToSongTime } from '../src/timeline';
import type { Hit, PerformancePlan, SourceEvent } from '../src/types';

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

describe('source note evidence in the piano roll', () => {
  const source = (id: string, timeUs: number, key: number, velocity = 100): SourceEvent => ({
    id, track: Number(id.split(':')[0]), eventIndex: Number(id.split(':')[1]),
    timeUs, tick: timeUs, channel: 9, message: { kind: 'noteOn', key, velocity },
  });
  const strike = (id: number, sourceEventId: string, pieceId: Hit['pieceId'], instrument: Hit['instrument'], note: number): Hit => ({
    id, sourceEventId, pieceId, instrument, note, time: .125001, tick: 60, track: 1,
    channel: 9, velocity: 100, limb: null, target: [0, 1, 0],
  });

  it('retains every positive source note across strike, count-in, control and unmapped interpretations', () => {
    const sources = [source('1:0', 0, 75), source('1:1', 125001, 49), source('2:0', 125001, 57),
      source('1:2', 250003, 46), source('1:3', 280007, 44), source('1:4', 500009, 90),
      source('1:5', 500009, 90), source('1:6', 500010, 46, 0),
      { ...source('1:7', 500011, 44), message: { kind: 'noteOff' as const, key: 44, velocity: 64 } },
      { ...source('1:8', 500012, 0), message: { kind: 'controller' as const, controller: 4, value: 64 } }];
    const hits = [strike(0, '1:1', 'crash', 'crash', 49), strike(1, '2:0', 'crashRight', 'crash', 57),
      { ...strike(2, '1:2', 'openHat', 'openHat', 46), time: .250003, limb: 1 }];
    const input: PerformancePlan = { ...plan, schemaVersion: 2, sourceEvents: sources, hits,
      interpretations: [{ sourceEventId: '1:0', kind: 'countIn' },
        { sourceEventId: '1:1', kind: 'strike', hitId: 0 }, { sourceEventId: '2:0', kind: 'strike', hitId: 1 },
        { sourceEventId: '1:2', kind: 'strike', hitId: 2 },
        { sourceEventId: '1:3', kind: 'control', relatedSourceEventId: '1:2' },
        { sourceEventId: '1:4', kind: 'unmapped' }] };
    const before = JSON.stringify(input);
    const notes = createTimelineNotes(input);
    // Two coincident source notes and duplicate unmapped keys remain distinct.
    expect(notes.map(note => note.id)).toEqual(['1:0', '1:1', '2:0', '1:2', '1:3', '1:4', '1:5']);
    expect(notes.map(note => note.time)).toEqual([0, .125001, .125001, .250003, .280007, .500009, .500009]);
    expect(notes.map(note => note.kind)).toEqual(['countIn', 'strike', 'strike', 'strike', 'control', 'unmapped', 'unmapped']);
    expect(notes.map(note => note.name)).toEqual(['Count-in', 'Crash L', 'Crash R', 'Open hat', 'Hat close', 'Note 90', 'Note 90']);
    expect(notes[1].hit?.limb).toBeNull(); // Unassigned motion cannot hide a source strike.
    expect(notes[4].hit).toBeUndefined(); // A derived pedal control is still inspectable as evidence.
    expect(notes.map(note => note.velocity)).toEqual(Array(7).fill(100));
    expect(JSON.stringify(input)).toBe(before);
  });

  it('continues to show legacy plans without inventing source events', () => {
    const hit = { ...strike(0, 'unused', 'crashRight', 'crash', 57), time: .321987, velocity: 33 };
    const notes = createTimelineNotes({ ...plan, hits: [hit] });
    expect(notes).toEqual([{ id: '0', time: .321987, note: 57, velocity: 33, name: 'Crash R', kind: 'strike', hit }]);
  });

  it('keeps ride and snare edge articulations on the correct instrument rows', () => {
    const notes=createTimelineNotes({...plan,hits:[
      {...strike(0,'1:0','ride','ride',52),articulation:'edge'},
      {...strike(1,'1:1','snare','snare',34),articulation:'edge'},
    ]});
    expect(notes.map(note=>note.name)).toEqual(['Ride edge','Snare edge']);
  });
});
