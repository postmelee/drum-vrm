import { afterEach, describe, expect, it, vi } from 'vitest';
import { DrumAudio, hitSampleIndex, synthesizePcm } from '../src/audio';
import { frameTiming } from '../src/export';
import { outputDuration, outputFrameCount, type Hit, type PerformancePlan } from '../src/types';

const hit = (overrides: Partial<Hit> = {}): Hit => ({
  id: 0, time: 0, tick: 0, track: 1, channel: 9, note: 36, velocity: 100,
  instrument: 'kick', limb: 3, target: [0, 0, 0], ...overrides,
});
const plan = (hits: Hit[] = [hit()], duration = 1): PerformancePlan => ({
  schemaVersion: 1, hits, duration, diagnostics: [], kit: [], tempoCount: 1,
});

describe('shared audio/video timeline', () => {
  it('starts the first hit at 0.5 seconds, including an unresolved visual hit', () => {
    const pcm = synthesizePcm(plan([hit({ limb: null })]));
    expect(pcm.channels[0].findIndex(value => value !== 0)).toBe(24_000);
    expect(pcm.channels[1].findIndex(value => value !== 0)).toBe(24_000);
    expect(pcm.channels[0].length).toBe(Math.round(outputDuration(plan()) * 48_000));
    expect(hitSampleIndex(0.1000125)).toBe(28_801);
  });

  it('preserves nonzero MIDI hit time rather than shifting it to a video frame', () => {
    const time = 0.123456;
    const pcm = synthesizePcm(plan([hit({ time })]));
    const actual = pcm.channels[0].findIndex(value => value !== 0);
    expect(actual).toBe(Math.round((0.5 + time) * 48_000));
    expect(Math.abs(actual / 48_000 - (time + 0.5))).toBeLessThanOrEqual(0.5 / 48_000);
  });

  it('is deterministic across synthesis runs and ignores velocity-zero events', () => {
    const input = plan([hit(), hit({ id: 1, time: 0.25, note: 38, instrument: 'snare' })]);
    const first = synthesizePcm(input, 8000);
    const second = synthesizePcm(input, 8000);
    expect(first.channels).toEqual(second.channels);
    expect(synthesizePcm(plan([hit({ velocity: 0 })]), 8000).channels[0].every(v => v === 0)).toBe(true);
    expect(input.hits.map(h => h.time)).toEqual([0, 0.25]);
  });

  it('keeps every mixed sample finite and below clipping even for coincident hits', () => {
    const input = plan(Array.from({ length: 30 }, (_, id) => hit({ id, velocity: 127 })));
    const pcm = synthesizePcm(input, 8000);
    expect(pcm.channels.every(channel => channel.every(v => Number.isFinite(v) && Math.abs(v) <= 1))).toBe(true);
    const quiet = synthesizePcm(plan([hit({ velocity: 20 })]), 8000).channels[0];
    const loud = synthesizePcm(plan([hit({ velocity: 100 })]), 8000).channels[0];
    const energy = (samples: Float32Array) => samples.reduce((sum, value) => sum + value * value, 0);
    expect(energy(loud)).toBeGreaterThan(energy(quiet));
  });

  it('writes exactly 923 frame timestamps for 30 seconds plus pre-roll and tail', () => {
    const input = plan([], 30);
    expect(outputFrameCount(input)).toBe(923);
    expect(outputDuration(input)).toBe(923 / 30);
    expect(frameTiming(0)).toEqual({ timestamp: 0, songTime: -0.5, duration: 1 / 30 });
    expect(frameTiming(15).songTime).toBe(0);
    expect(frameTiming(922).timestamp + frameTiming(922).duration).toBeCloseTo(923 / 30, 12);
  });
});

describe('interpreted drum preview audio', () => {
  const energy = (samples: Float32Array) => samples.reduce((sum, value) => sum + value * value, 0);

  it('closes an open hat at the original control sample without shifting or restarting its onset', () => {
    const open = hit({ time: .000019, timeUs: 19, instrument: 'openHat', pieceId: 'openHat', note: 46, limb: 1 });
    const input = { ...plan([open]), controls: [{ kind: 'hihatClose' as const, sourceEventId: '1:1',
      openSourceEventId: '1:0', releaseStartTimeUs: -99981, openTimeUs: 19, timeUs: 25007, openness: .2 }] };
    const before = JSON.stringify(input);
    for (const rate of [8000, 44100, 48000]) {
      const literal = synthesizePcm(plan([open]), rate);
      const controlled = synthesizePcm(input, rate);
      const onset = Math.round((.5 + .000019) * rate), close = Math.round((.5 + .025007) * rate);
      for (let channel = 0; channel < 2; channel++) {
        expect(controlled.channels[channel].findIndex(value => value !== 0)).toBe(onset);
        expect(controlled.channels[channel].slice(onset, close - Math.ceil(.008 * rate)))
          .toEqual(literal.channels[channel].slice(onset, close - Math.ceil(.008 * rate)));
        expect(controlled.channels[channel].slice(close).every(value => value === 0)).toBe(true);
        expect(energy(literal.channels[channel].slice(close, close + rate / 10))).toBeGreaterThan(0);
      }
    }
    expect(JSON.stringify(input)).toBe(before);
  });

  it('does not replay a paired foot-close voice and does not let an earlier close silence the next open', () => {
    const opening = hit({ instrument: 'openHat', pieceId: 'openHat', note: 46, time: .01 });
    const next = hit({ id: 1, instrument: 'openHat', pieceId: 'openHat', note: 46, time: .2 });
    const controlled = synthesizePcm({ ...plan([opening, next]), controls: [{ kind: 'hihatClose',
      sourceEventId: '1:1', openSourceEventId: '1:0', releaseStartTimeUs: -90000, openTimeUs: 10000, timeUs: 50000, openness: .2 }] }, 8000);
    const from = Math.round(.55 * 8000), until = Math.round(.7 * 8000);
    expect(controlled.channels.every(channel => channel.slice(from, until).every(sample => sample === 0))).toBe(true);
    expect(controlled.channels[0].slice(until)).toEqual(synthesizePcm(plan([next]), 8000).channels[0].slice(until));
    const literal = synthesizePcm(plan([opening, hit({ id: 2, time: .05, note: 44, instrument: 'pedalHat' })]), 8000);
    expect(energy(literal.channels[0].slice(from, until))).toBeGreaterThan(0);
  });

  it('keeps both physical crashes audible with opposite stereo positions, even without a visual limb', () => {
    const left = hit({ instrument: 'crash', pieceId: 'crash', note: 49, time: .123456, velocity: 80, limb: null });
    const right = hit({ ...left, id: 1, pieceId: 'crashRight', note: 57 });
    const l = synthesizePcm(plan([left]), 8000), r = synthesizePcm(plan([right]), 8000);
    const combined = synthesizePcm(plan([left, right]), 8000);
    expect(energy(l.channels[0])).toBeGreaterThan(energy(l.channels[1]) * 4);
    expect(energy(r.channels[1])).toBeGreaterThan(energy(r.channels[0]) * 4);
    for (let channel = 0; channel < 2; channel++) {
      expect(combined.channels[channel].findIndex(value => value !== 0)).toBe(Math.round(.623456 * 8000));
      expect(combined.channels[channel]).not.toEqual(l.channels[channel]);
      expect(combined.channels[channel]).not.toEqual(r.channels[channel]);
    }
  });
});

describe('Web Audio transport', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('creates audio on play, pauses/resumes once, seeks without duplicate sources, and disposes', async () => {
    const sources: Array<{ buffer: AudioBuffer | null; connect: ReturnType<typeof vi.fn>; disconnect: ReturnType<typeof vi.fn>;
      start: ReturnType<typeof vi.fn>; stop: ReturnType<typeof vi.fn>; onended: (() => void) | null }> = [];
    let context: FakeAudioContext | undefined;
    class FakeAudioContext {
      currentTime = 0;
      state = 'suspended';
      destination = {};
      onstatechange: (() => void) | null = null;
      constructor() { context = this; }
      async resume() { this.state = 'running'; }
      async close() { this.state = 'closed'; }
      createGain() { return { gain: { value: 0, setTargetAtTime: vi.fn() }, connect: vi.fn(), disconnect: vi.fn() }; }
      createBufferSource() {
        const source = { buffer: null, connect: vi.fn(), disconnect: vi.fn(), start: vi.fn(), stop: vi.fn(), onended: null };
        sources.push(source);
        return source;
      }
    }
    vi.stubGlobal('AudioContext', FakeAudioContext);
    const buffer = { duration: 5, sampleRate: 48000 } as AudioBuffer;
    const transport = new DrumAudio(buffer);
    expect(context).toBeUndefined();
    await transport.play();
    expect(transport.playing).toBe(true);
    expect(sources[0].start).toHaveBeenCalledWith(0.02, 0);
    context!.currentTime = 0.52;
    expect(transport.pause()).toBeCloseTo(0.5);
    expect(sources[0].stop).toHaveBeenCalledTimes(1);
    expect(transport.playing).toBe(false);
    await transport.play();
    expect(sources[1].start).toHaveBeenCalledWith(0.54, 0.5);
    context!.currentTime = 1.04;
    expect(transport.currentTime).toBeCloseTo(1);
    transport.seek(2);
    await Promise.resolve();
    expect(sources[1].stop).toHaveBeenCalledTimes(1);
    expect(sources[2].start).toHaveBeenCalledWith(1.06, 2);
    context!.state = 'suspended';
    context!.onstatechange?.();
    expect(transport.playing).toBe(false);
    expect(sources[2].stop).toHaveBeenCalledTimes(1);
    transport.dispose();
    expect(context!.state).toBe('closed');
  });
});
