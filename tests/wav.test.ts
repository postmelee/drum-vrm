import { afterEach, describe, expect, it, vi } from 'vitest';
import { alignWavPcm, alignWavToPlan, inspectWav, loadLocalWav, MAX_WAV_BYTES } from '../src/wav';
import { outputDuration, SAMPLE_RATE, type PerformancePlan } from '../src/types';

const plan = (duration = 1): PerformancePlan => ({ schemaVersion: 1, duration, hits: [], diagnostics: [], kit: [], tempoCount: 1 });
const pcm = (samples: number[], right?: number[]) => ({ sampleRate: SAMPLE_RATE, channels: right ? [Float32Array.from(samples), Float32Array.from(right)] : [Float32Array.from(samples)] });

/** Independently authored RIFF fixture: no uploaded audio or third-party samples. */
function wav({ channels = 1, rate = 48000, bits = 16, frames = 4, tag = 1, extensible = false } = {}): ArrayBuffer {
  const fmtSize = extensible ? 40 : 16, dataSize = frames * channels * bits / 8;
  const bytes = new ArrayBuffer(12 + 8 + fmtSize + 8 + dataSize + (dataSize & 1)), view = new DataView(bytes);
  const text = (offset: number, value: string) => [...value].forEach((character, i) => view.setUint8(offset + i, character.charCodeAt(0)));
  text(0, 'RIFF'); view.setUint32(4, bytes.byteLength - 8, true); text(8, 'WAVE'); text(12, 'fmt ');
  view.setUint32(16, fmtSize, true); view.setUint16(20, extensible ? 0xfffe : tag, true);
  view.setUint16(22, channels, true); view.setUint32(24, rate, true);
  view.setUint32(28, rate * channels * bits / 8, true); view.setUint16(32, channels * bits / 8, true); view.setUint16(34, bits, true);
  if (extensible) {
    view.setUint16(36, 22, true); view.setUint16(38, bits, true);
    new Uint8Array(bytes, 44, 16).set([tag, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113]);
  }
  const data = 20 + fmtSize; text(data, 'data'); view.setUint32(data + 4, dataSize, true);
  return bytes;
}

describe('bounded WAV preflight', () => {
  it('accepts ordinary PCM, finite float and their extensible encodings', () => {
    for (const bits of [8, 16, 24, 32]) {
      expect(inspectWav(wav({ channels: 2, bits, rate: 44100 }))).toMatchObject({ channels: 2, sampleRate: 44100, bitsPerSample: bits, frames: 4, encoding: 'pcm' });
    }
    expect(inspectWav(wav({ tag: 3, bits: 32 }))).toMatchObject({ encoding: 'float', frames: 4 });
    expect(inspectWav(wav({ extensible: true, bits: 24 }))).toMatchObject({ encoding: 'pcm', bitsPerSample: 24 });
    expect(inspectWav(wav({ extensible: true, tag: 3, bits: 32 }))).toMatchObject({ encoding: 'float' });
  });

  it('rejects disguised, truncated and malformed files before decoding', () => {
    const wrong = wav(); new Uint8Array(wrong)[0] = 0;
    expect(() => inspectWav(wrong)).toThrow(/RIFF/);
    expect(() => inspectWav(wav().slice(0, -1))).toThrow(/크기/);
    const overrun = wav(); new DataView(overrun).setUint32(40, 9999, true);
    expect(() => inspectWav(overrun)).toThrow(/청크/);
    const badAlign = wav(); new DataView(badAlign).setUint16(32, 1, true);
    expect(() => inspectWav(badAlign)).toThrow(/프레임/);
    const badGuid = wav({ extensible: true }); new Uint8Array(badGuid)[48] = 1;
    expect(() => inspectWav(badGuid)).toThrow(/확장/);
  });

  it('rejects compressed/multichannel/out-of-range and empty sources', () => {
    expect(() => inspectWav(wav({ tag: 2 }))).toThrow(/PCM/);
    expect(() => inspectWav(wav({ channels: 3 }))).toThrow(/모노/);
    expect(() => inspectWav(wav({ rate: 4000 }))).toThrow(/레이트/);
    expect(() => inspectWav(wav({ frames: 0 }))).toThrow(/프레임/);
    expect(() => inspectWav(wav({ rate: 8000, bits: 8, frames: 8000 * 301 }))).toThrow(/300초/);
  });

  it('rejects nonfinite source float samples before the browser can sanitize them', () => {
    for (const value of [NaN, Infinity, -Infinity]) {
      const bytes = wav({ tag: 3, bits: 32 }); new DataView(bytes).setFloat32(44, value, true);
      expect(() => inspectWav(bytes)).toThrow(/Float/);
    }
  });
});

describe('WAV time alignment', () => {
  it('adds the shared pre-roll once and copies mono equally without changing it', () => {
    const source = pcm([0.25, -0.5, 0.75]);
    const result = alignWavPcm(source, plan(), 0);
    expect(result.channels[0].length).toBe(Math.round(outputDuration(plan()) * SAMPLE_RATE));
    expect(Array.from(result.channels[0].slice(24000, 24003))).toEqual([0.25, -0.5, 0.75]);
    expect(result.channels[0]).toEqual(result.channels[1]);
    expect(result.channels[0][23999]).toBe(0);
    expect(result.channels[0][24003]).toBe(0);
    expect(Array.from(source.channels[0])).toEqual([0.25, -0.5, 0.75]);
    expect(result.info.leadingSilenceSeconds).toBe(0.5);
    expect(result.info.trimmedEndSeconds).toBe(0);
  });

  it('moves positive offsets later and preserves independent stereo channels', () => {
    const result = alignWavPcm(pcm([0.2, 0.4], [-0.2, -0.4]), plan(), 0.25);
    expect(result.channels[0][36000]).toBeCloseTo(0.2);
    expect(result.channels[1][36000]).toBeCloseTo(-0.2);
    expect(result.info.effectiveOffsetSeconds).toBe(0.25);
    expect(result.info.leadingSilenceSeconds).toBe(0.75);
  });

  it('uses negative offsets inside pre-roll before trimming anything at output zero', () => {
    const inside = alignWavPcm(pcm([1, 0]), plan(), -0.25);
    expect(inside.channels[0][12000]).toBe(1);
    expect(inside.info.trimmedStartSeconds).toBe(0);
    const trimmed = alignWavPcm(pcm([0.1, 0.2, 0.3, 0.4]), plan(), -0.5 - 2 / SAMPLE_RATE);
    expect(trimmed.channels[0][0]).toBeCloseTo(0.3);
    expect(trimmed.info.trimmedStartSeconds).toBe(2 / SAMPLE_RATE);
    expect(trimmed.info.copiedSeconds).toBe(2 / SAMPLE_RATE);
  });

  it('trims the end at the fixed video boundary and reports it', () => {
    const source = new Float32Array(SAMPLE_RATE * 3).fill(0.1), p = plan(0.1);
    const result = alignWavPcm({ sampleRate: SAMPLE_RATE, channels: [source] }, p, 0);
    const expectedCopied = result.channels[0].length - 24000;
    expect(result.info.copiedSeconds).toBe(expectedCopied / SAMPLE_RATE);
    expect(result.info.trimmedEndSeconds).toBe((source.length - expectedCopied) / SAMPLE_RATE);
    expect(result.info.trailingSilenceSeconds).toBe(0);
    expect(result.channels[0].at(-1)).toBeCloseTo(0.1);
  });

  it('reports complete non-overlap and returns silence at either offset extreme', () => {
    const late = alignWavPcm(pcm([1]), plan(), 300), early = alignWavPcm(pcm([1]), plan(), -300);
    expect(late.info.copiedSeconds).toBe(0); expect(early.info.copiedSeconds).toBe(0);
    expect(late.channels[0].every(sample => sample === 0)).toBe(true);
    expect(early.channels[0].every(sample => sample === 0)).toBe(true);
    expect(late.info.trimmedEndSeconds).toBe(1 / SAMPLE_RATE);
    expect(early.info.trimmedStartSeconds).toBe(1 / SAMPLE_RATE);
  });

  it('rounds once to the nearest output sample and is repeatable', () => {
    const first = alignWavPcm(pcm([1]), plan(), 0.3 / SAMPLE_RATE);
    const second = alignWavPcm(pcm([1]), plan(), 0.3 / SAMPLE_RATE);
    expect(first.info.effectiveOffsetSeconds).toBe(0);
    expect(first.channels).toEqual(second.channels);
  });

  it('rejects invalid offsets, plans, PCM rates, channel lengths and nonfinite samples', () => {
    for (const offset of [NaN, Infinity, -Infinity, 300.001, -300.001]) expect(() => alignWavPcm(pcm([1]), plan(), offset)).toThrow(/오프셋/);
    for (const duration of [NaN, Infinity, -1, 0, 301]) expect(() => alignWavPcm(pcm([1]), plan(duration), 0)).toThrow(/MIDI/);
    expect(() => alignWavPcm({ sampleRate: 44100, channels: [Float32Array.of(1)] }, plan(), 0)).toThrow(/48 kHz/);
    expect(() => alignWavPcm(pcm([1], [1, 2]), plan(), 0)).toThrow(/길이/);
    expect(() => alignWavPcm(pcm([NaN]), plan(), 0)).toThrow(/샘플/);
  });
});

class TestAudioBuffer {
  readonly channels: Float32Array<ArrayBuffer>[];
  readonly sampleRate: number;
  readonly length: number;
  readonly numberOfChannels: number;
  constructor(options: { sampleRate: number; length: number; numberOfChannels: number }) {
    Object.assign(this, options); this.sampleRate = options.sampleRate; this.length = options.length; this.numberOfChannels = options.numberOfChannels;
    this.channels = Array.from({ length: options.numberOfChannels }, () => new Float32Array(options.length));
  }
  get duration() { return this.length / this.sampleRate; }
  getChannelData(index: number) { return this.channels[index]; }
  copyToChannel(channel: Float32Array, index: number) { this.channels[index].set(channel); }
}
afterEach(() => vi.unstubAllGlobals());

describe('browser WAV boundary', () => {
  it('closes the decoder after success and exposes the unshifted source metadata', async () => {
    const close = vi.fn().mockResolvedValue(undefined);
    const decoded = new TestAudioBuffer({ sampleRate: 48000, length: 4, numberOfChannels: 1 });
    vi.stubGlobal('AudioContext', class { sampleRate = 48000; decodeAudioData = vi.fn().mockResolvedValue(decoded); close = close; });
    const result = await loadLocalWav(new File([wav()], 'authored.wav'));
    expect(result.buffer).toBe(decoded); expect(result.name).toBe('authored.wav'); expect(result.info.frames).toBe(4);
    expect(close).toHaveBeenCalledOnce();
  });

  it('closes a rejected decoder and does not attempt to decode malformed/oversized files', async () => {
    const close = vi.fn().mockResolvedValue(undefined), decode = vi.fn().mockRejectedValue(new Error('decoder'));
    const Context = vi.fn(function () { return { sampleRate: 48000, decodeAudioData: decode, close }; });
    vi.stubGlobal('AudioContext', Context);
    await expect(loadLocalWav(new File([wav()], 'authored.wav'))).rejects.toThrow(/디코딩/);
    expect(close).toHaveBeenCalledOnce();
    await expect(loadLocalWav(new File(['not a WAV'], 'bad.wav'))).rejects.toThrow(/RIFF/);
    const oversized = { size: MAX_WAV_BYTES + 1, arrayBuffer: vi.fn() } as unknown as File;
    await expect(loadLocalWav(oversized)).rejects.toThrow(/100 MiB/);
    expect(Context).toHaveBeenCalledOnce(); expect(oversized.arrayBuffer).not.toHaveBeenCalled();
  });

  it('rejects decoded length/channel mismatches and validates finite PCM', async () => {
    const decoded = new TestAudioBuffer({ sampleRate: 48000, length: 12, numberOfChannels: 1 });
    const close = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal('AudioContext', class { sampleRate = 48000; decodeAudioData = vi.fn().mockResolvedValue(decoded); close = close; });
    await expect(loadLocalWav(new File([wav()], 'authored.wav'))).rejects.toThrow(/파일 정보/);
    decoded.channels[0][0] = NaN;
    await expect(loadLocalWav(new File([wav()], 'authored.wav'))).rejects.toThrow(/샘플/);
    expect(close).toHaveBeenCalledTimes(2);
  });

  it('provides exactly the aligned stereo buffer required by preview/export', () => {
    vi.stubGlobal('AudioBuffer', TestAudioBuffer);
    const source = new TestAudioBuffer({ sampleRate: 48000, length: 2, numberOfChannels: 1 });
    source.channels[0][0] = 0.5;
    const result = alignWavToPlan(source as unknown as AudioBuffer, plan(), 0);
    expect(result.buffer.sampleRate).toBe(48000); expect(result.buffer.numberOfChannels).toBe(2);
    expect(result.buffer.length).toBe(Math.round(outputDuration(plan()) * 48000));
    expect(result.buffer.getChannelData(1)[24000]).toBe(0.5);
    expect(source.getChannelData(0)[0]).toBe(0.5);
  });
});
