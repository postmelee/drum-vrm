import { outputDuration, PRE_ROLL, SAMPLE_RATE, type Hit, type Instrument, type PerformancePlan } from './types';

export interface SynthPcm {
  channels: [Float32Array<ArrayBuffer>, Float32Array<ArrayBuffer>];
  sampleRate: number;
  duration: number;
}

/** Audio and video share output time zero; the MIDI itself keeps its original time. */
export const hitSampleIndex = (songTime: number, sampleRate = SAMPLE_RATE): number =>
  Math.round((PRE_ROLL + songTime) * sampleRate);

const voices: Record<Instrument, { seconds: number; pan: number; gain: number; frequency: number }> = {
  kick: { seconds: 0.45, pan: 0, gain: 0.85, frequency: 52 },
  snare: { seconds: 0.24, pan: -0.1, gain: 0.56, frequency: 180 },
  hihat: { seconds: 0.085, pan: -0.5, gain: 0.23, frequency: 6200 },
  openHat: { seconds: 0.5, pan: -0.5, gain: 0.27, frequency: 6200 },
  pedalHat: { seconds: 0.13, pan: -0.5, gain: 0.22, frequency: 4600 },
  tomHigh: { seconds: 0.36, pan: -0.3, gain: 0.65, frequency: 205 },
  tomMid: { seconds: 0.42, pan: 0.2, gain: 0.65, frequency: 160 },
  tomLow: { seconds: 0.5, pan: 0.55, gain: 0.65, frequency: 115 },
  crash: { seconds: 1.2, pan: -0.65, gain: 0.32, frequency: 3400 },
  ride: { seconds: 0.75, pan: 0.65, gain: 0.26, frequency: 4200 },
};

// A fixed integer PRNG per source event avoids random state depending on playback/seek order.
function noiseFor(hit: Hit): () => number {
  let state = ((hit.id + 1) * 2654435761 ^ (hit.note << 12) ^ (hit.track << 4)) >>> 0;
  if (state === 0) state = 0x6d2b79f5;
  return () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return (state >>> 0) / 2147483648 - 1;
  };
}

/** Original procedural drum synthesis. No samples, soundfonts, or downloaded assets. */
export function synthesizePcm(plan: PerformancePlan, sampleRate = SAMPLE_RATE): SynthPcm {
  if (!Number.isInteger(sampleRate) || sampleRate < 8000 || sampleRate > 192000) {
    throw new Error('지원하지 않는 오디오 샘플 레이트입니다.');
  }
  const duration = outputDuration(plan);
  const length = Math.round(duration * sampleRate);
  const left = new Float32Array(length);
  const right = new Float32Array(length);
  for (const hit of plan.hits) {
    if (!Number.isFinite(hit.time) || hit.time < 0 || hit.velocity <= 0) continue;
    const voice = voices[hit.instrument];
    if (!voice) continue;
    const start = hitSampleIndex(hit.time, sampleRate);
    const count = Math.min(Math.ceil(voice.seconds * sampleRate), length - start);
    const random = noiseFor(hit);
    const velocity = Math.min(1, hit.velocity / 127);
    const level = voice.gain * Math.pow(velocity, 1.25);
    const leftGain = Math.cos((voice.pan + 1) * Math.PI / 4) * level;
    const rightGain = Math.sin((voice.pan + 1) * Math.PI / 4) * level;
    let previousNoise = 0;
    for (let i = 0; i < count; i++) {
      const time = i / sampleRate;
      const noise = random();
      const brightNoise = (noise - previousNoise) * 0.5;
      previousNoise = noise;
      let sample: number;
      if (hit.instrument === 'kick') {
        const phase = 2 * Math.PI * (voice.frequency * time + 95 * 0.025 * (1 - Math.exp(-time / 0.025)));
        sample = Math.sin(phase) * Math.exp(-time * 12) + noise * Math.exp(-time * 500) * 0.25;
      } else if (hit.instrument === 'snare') {
        sample = brightNoise * Math.exp(-time * 22) * 1.4
          + Math.sin(2 * Math.PI * voice.frequency * time) * Math.exp(-time * 35) * 0.45;
      } else if (hit.instrument.startsWith('tom')) {
        const phase = 2 * Math.PI * (voice.frequency * time + 35 * 0.035 * (1 - Math.exp(-time / 0.035)));
        sample = Math.sin(phase) * Math.exp(-time * 11)
          + brightNoise * Math.exp(-time * 150) * 0.2;
      } else {
        const metallic = (Math.sin(2 * Math.PI * voice.frequency * time)
          + Math.sin(2 * Math.PI * voice.frequency * 1.417 * time)) * 0.15;
        sample = (brightNoise * 1.5 + metallic) * Math.exp(-time * 5.5 / voice.seconds);
      }
      // Tiny fixed attack and terminal fade prevent clicks without moving the onset.
      sample *= Math.min(1, (i + 1) / (sampleRate * 0.0006))
        * Math.min(1, (count - i) / (sampleRate * 0.008));
      left[start + i] += sample * leftGain;
      right[start + i] += sample * rightGain;
    }
  }
  // An instantaneous limiter preserves event timing and handles simultaneous hits.
  for (let i = 0; i < length; i++) {
    left[i] = Math.tanh(left[i]);
    right[i] = Math.tanh(right[i]);
  }
  return { channels: [left, right], sampleRate, duration: length / sampleRate };
}

export function synthesize(plan: PerformancePlan, sampleRate = SAMPLE_RATE): AudioBuffer {
  const pcm = synthesizePcm(plan, sampleRate);
  const buffer = new AudioBuffer({ numberOfChannels: 2, length: pcm.channels[0].length, sampleRate });
  buffer.copyToChannel(pcm.channels[0], 0);
  buffer.copyToChannel(pcm.channels[1], 1);
  return buffer;
}

/** The output-time transport uses the same PCM as export and Web Audio as its clock. */
export class DrumAudio {
  private context: AudioContext | null = null;
  private gain: GainNode | null = null;
  private source: AudioBufferSourceNode | null = null;
  private offset = 0;
  private origin = 0;
  private volume = 0.8;
  private generation = 0;
  private disposed = false;

  constructor(readonly buffer: AudioBuffer) {}

  get playing(): boolean { return this.source !== null && this.context?.state === 'running'; }

  get currentTime(): number {
    if (!this.source || !this.context) return this.offset;
    let clock = this.context.currentTime;
    // The timestamp estimates audible output; do not subtract latency a second time.
    const stamp = this.context.getOutputTimestamp?.();
    if (stamp?.contextTime && stamp.performanceTime) {
      clock = Math.min(clock, stamp.contextTime + (performance.now() - stamp.performanceTime) / 1000);
    }
    return Math.min(this.buffer.duration, this.offset + Math.max(0, clock - this.origin));
  }

  async play(outputTime = this.currentTime): Promise<void> {
    if (this.disposed) throw new Error('오디오가 이미 정리되었습니다.');
    this.pause();
    this.offset = Math.max(0, Math.min(this.buffer.duration, outputTime));
    if (this.offset >= this.buffer.duration) this.offset = 0;
    if (!this.context) {
      this.context = new AudioContext({ sampleRate: this.buffer.sampleRate, latencyHint: 'interactive' });
      this.gain = this.context.createGain();
      this.gain.gain.value = this.volume;
      this.gain.connect(this.context.destination);
      this.context.onstatechange = () => {
        if (this.source && this.context?.state !== 'running') this.pause();
      };
    }
    const generation = ++this.generation;
    await this.context.resume();
    if (this.disposed || generation !== this.generation) return;
    const source = this.context.createBufferSource();
    source.buffer = this.buffer;
    source.connect(this.gain!);
    this.origin = this.context.currentTime + 0.02;
    this.source = source;
    source.onended = () => {
      if (this.source === source) {
        this.offset = this.buffer.duration;
        this.source = null;
      }
      source.disconnect();
    };
    source.start(this.origin, this.offset);
  }

  pause(): number {
    const time = this.currentTime;
    this.generation++;
    if (this.source) {
      const previous = this.source;
      this.source = null;
      previous.onended = null;
      previous.stop();
      previous.disconnect();
    }
    this.offset = time;
    return time;
  }

  seek(outputTime: number): void {
    const wasPlaying = this.playing;
    this.pause();
    this.offset = Math.max(0, Math.min(this.buffer.duration, outputTime));
    if (wasPlaying && this.offset < this.buffer.duration) void this.play(this.offset);
  }

  setVolume(value: number): void {
    this.volume = Math.max(0, Math.min(1, value));
    if (this.gain && this.context) this.gain.gain.setTargetAtTime(this.volume, this.context.currentTime, 0.015);
  }

  dispose(): void {
    this.pause();
    this.disposed = true;
    this.gain?.disconnect();
    if (this.context) {
      this.context.onstatechange = null;
      void this.context.close();
    }
    this.context = null;
    this.gain = null;
  }
}
