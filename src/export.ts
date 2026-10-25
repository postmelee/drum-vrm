import {
  AudioBufferSource, AudioBufferSink, BufferSource, BufferTarget, CanvasSource, Input, MP4, WEBM,
  Mp4OutputFormat, Output, Quality, WebMOutputFormat,
  canEncodeAudio, canEncodeVideo,
} from 'mediabunny';
import { FPS, outputDuration, outputFrameCount, PRE_ROLL, SAMPLE_RATE, type PerformancePlan } from './types';

export const EXPORT_WIDTH = 1280;
export const EXPORT_HEIGHT = 720;
const videoQuality = new Quality({ bitrate: 4_000_000 });
const audioQuality = new Quality({ bitrate: 128_000 });

export interface ExportProfile {
  id: string;
  label: string;
  extension: 'mp4' | 'webm';
  mime: string;
  videoCodec: 'avc' | 'vp9' | 'vp8';
  audioCodec: 'aac' | 'opus' | null;
}

const profiles: ExportProfile[] = [
  { id: 'mp4-avc', label: 'MP4 · H.264 / AAC', extension: 'mp4', mime: 'video/mp4', videoCodec: 'avc', audioCodec: 'aac' },
  { id: 'webm-vp9', label: 'WebM · VP9 / Opus', extension: 'webm', mime: 'video/webm', videoCodec: 'vp9', audioCodec: 'opus' },
  { id: 'webm-vp8', label: 'WebM · VP8 / Opus', extension: 'webm', mime: 'video/webm', videoCodec: 'vp8', audioCodec: 'opus' },
];

/** The timestamps come from the frame index, never accumulated elapsed render time. */
export function frameTiming(frame: number): { timestamp: number; songTime: number; duration: number } {
  return { timestamp: frame / FPS, songTime: frame / FPS - PRE_ROLL, duration: 1 / FPS };
}

function createOutput(profile: ExportProfile) {
  const target = new BufferTarget();
  const format = profile.extension === 'mp4'
    ? new Mp4OutputFormat({ fastStart: 'in-memory' })
    : new WebMOutputFormat();
  return new Output({ target, format });
}

function videoConfig(profile: ExportProfile) {
  return { codec: profile.videoCodec, quality: videoQuality, latencyMode: 'quality' as const, keyFrameInterval: 2 };
}

function throwIfAborted(signal?: AbortSignal): void {
  if (signal?.aborted) throw new DOMException('영상 내보내기가 취소되었습니다.', 'AbortError');
}

/** A pending browser codec request must not prevent the UI from cancelling. */
function withAbort<T>(pending: Promise<T>, signal?: AbortSignal): Promise<T> {
  if (!signal) return pending;
  if (signal.aborted) return Promise.reject(new DOMException('영상 내보내기가 취소되었습니다.', 'AbortError'));
  return new Promise((resolve, reject) => {
    const abort = () => reject(new DOMException('영상 내보내기가 취소되었습니다.', 'AbortError'));
    signal.addEventListener('abort', abort, { once: true });
    pending.then(resolve, reject).finally(() => signal.removeEventListener('abort', abort));
  });
}

/** WebCodecs encoders can add platform-specific priming and tail padding. */
async function checkAudioTimeline(
  data: ArrayBuffer, expectedDuration: number, expectedOnset?: number, signal?: AbortSignal,
): Promise<boolean> {
  const input = new Input({ source: new BufferSource(data), formats: [MP4, WEBM] });
  const dispose = () => input.dispose();
  signal?.addEventListener('abort', dispose, { once: true });
  try {
    throwIfAborted(signal);
    const track = await withAbort(input.getPrimaryAudioTrack(), signal);
    if (!track) return false;
    const duration = await withAbort(track.computeDuration(), signal);
    if (Math.abs(duration - expectedDuration) > 1 / FPS + 0.000001) return false;
    if (expectedOnset === undefined) return true;
    if (!await withAbort(track.canDecode(), signal)) return false;
    const sink = new AudioBufferSink(track);
    let onset = Infinity;
    for await (const sample of sink.buffers()) {
      throwIfAborted(signal);
      const data = sample.buffer.getChannelData(0);
      const index = data.findIndex(value => Math.abs(value) > 0.005);
      if (index >= 0) {
        onset = sample.timestamp + index / sample.buffer.sampleRate;
        break;
      }
    }
    return Math.abs(onset - expectedOnset) <= 1 / FPS;
  } finally {
    signal?.removeEventListener('abort', dispose);
    input.dispose();
  }
}

/** Config support alone is insufficient: open, encode, and mux a tiny real file. */
async function trialEncode(profile: ExportProfile, signal?: AbortSignal): Promise<boolean> {
  throwIfAborted(signal);
  const canvas = document.createElement('canvas');
  canvas.width = EXPORT_WIDTH;
  canvas.height = EXPORT_HEIGHT;
  const context = canvas.getContext('2d');
  if (!context) return false;
  context.fillStyle = '#15383d';
  context.fillRect(0, 0, canvas.width, canvas.height);
  const output = createOutput(profile);
  let videoPackets = 0;
  let audioPackets = 0;
  const video = new CanvasSource(canvas, { ...videoConfig(profile), onEncodedPacket: () => { videoPackets++; } });
  output.addVideoTrack(video, { frameRate: FPS });
  const audio = profile.audioCodec ? new AudioBufferSource({
    codec: profile.audioCodec,
    quality: audioQuality,
    onEncodedPacket: () => { audioPackets++; },
  }) : null;
  if (audio) output.addAudioTrack(audio);
  const trialFrames = 6;
  const trialDuration = trialFrames / FPS;
  const trialOnset = 0.1;
  let cancellation: Promise<void> | undefined;
  const cancel = () => { cancellation ??= output.cancel().catch(() => {}); };
  signal?.addEventListener('abort', cancel, { once: true });
  let timeout: ReturnType<typeof setTimeout>;
  const deadline = new Promise<never>((_, reject) => {
    timeout = setTimeout(() => { cancel(); reject(new Error('시험 인코딩 시간이 초과되었습니다.')); }, 15_000);
  });
  try {
    const encode = (async () => {
      throwIfAborted(signal);
      await output.start();
      throwIfAborted(signal);
      if (audio) {
        const buffer = new AudioBuffer({ numberOfChannels: 2, length: SAMPLE_RATE * trialDuration, sampleRate: SAMPLE_RATE });
        for (let c = 0; c < 2; c++) {
          const data = buffer.getChannelData(c);
          for (let i = trialOnset * SAMPLE_RATE; i < data.length; i++) {
            data[i] = Math.sin(2 * Math.PI * 440 * (i / SAMPLE_RATE - trialOnset)) * 0.05;
          }
        }
        await audio.add(buffer);
        throwIfAborted(signal);
        audio.close();
      }
      for (let i = 0; i < trialFrames; i++) {
        throwIfAborted(signal);
        await video.add(i / FPS, 1 / FPS);
      }
      video.close();
      await output.finalize();
    })();
    await withAbort(Promise.race([encode, deadline]), signal);
    throwIfAborted(signal);
    if (!output.target.buffer?.byteLength || videoPackets !== trialFrames || (audio && !audioPackets)) return false;
    // Reject a drifting codec instead of applying a guessed global audio offset.
    // Unsupported AAC priming therefore selects a tested Opus/WebM combination.
    return !audio || await withAbort(Promise.race([
      checkAudioTimeline(output.target.buffer, trialDuration, trialOnset, signal), deadline,
    ]), signal);
  } catch {
    throwIfAborted(signal);
    return false;
  } finally {
    clearTimeout(timeout!);
    signal?.removeEventListener('abort', cancel);
    if (output.state !== 'finalized' && output.state !== 'canceled') cancel();
    await cancellation;
    canvas.width = canvas.height = 0;
  }
}

/** Muted combinations are returned only for the caller's explicit muted choice. */
export async function probeExport(muted = false, signal?: AbortSignal): Promise<ExportProfile[]> {
  throwIfAborted(signal);
  if (typeof VideoEncoder === 'undefined' || (!muted && typeof AudioEncoder === 'undefined')) return [];
  const available: ExportProfile[] = [];
  for (const base of profiles) {
    throwIfAborted(signal);
    const profile = muted
      ? { ...base, id: `${base.id}-muted`, audioCodec: null, label: `${base.extension.toUpperCase()} · ${base.videoCodec.toUpperCase()} · 무음` }
      : { ...base };
    try {
      const videoSupported = await withAbort(canEncodeVideo(profile.videoCodec, {
        width: EXPORT_WIDTH, height: EXPORT_HEIGHT, frameRate: FPS, quality: videoQuality, latencyMode: 'quality',
      }), signal);
      const audioSupported = !profile.audioCodec || await withAbort(canEncodeAudio(profile.audioCodec, {
        numberOfChannels: 2, sampleRate: SAMPLE_RATE, quality: audioQuality,
      }), signal);
      if (videoSupported && audioSupported && await trialEncode(profile, signal)) available.push(profile);
    } catch {
      throwIfAborted(signal);
      // Probe failure rejects a combination; it never silently drops its audio.
    }
  }
  return available;
}

export interface ExportOptions {
  canvas: HTMLCanvasElement;
  renderFrame: (songTime: number) => void | Promise<void>;
  plan: PerformancePlan;
  audio: AudioBuffer;
  profile: ExportProfile;
  signal?: AbortSignal;
  onProgress?: (progress: number) => void;
}

export interface ExportResult {
  blob: Blob;
  extension: 'mp4' | 'webm';
  mime: string;
  frames: number;
  duration: number;
}

const aborted = () => new DOMException('영상 내보내기가 취소되었습니다.', 'AbortError');
const yieldToUi = () => new Promise<void>(resolve => setTimeout(resolve, 0));

/** Memory holds the PCM and compressed output, never a sequence of raw frames. */
export async function exportVideo(options: ExportOptions): Promise<ExportResult> {
  const { canvas, renderFrame, plan, audio, profile, signal, onProgress } = options;
  if (signal?.aborted) throw aborted();
  if (canvas.width !== EXPORT_WIDTH || canvas.height !== EXPORT_HEIGHT) {
    throw new Error('내보내기 캔버스는 1280 × 720 픽셀이어야 합니다.');
  }
  const frames = outputFrameCount(plan);
  const duration = outputDuration(plan);
  const samples = Math.round(duration * SAMPLE_RATE);
  if (audio.numberOfChannels !== 2 || audio.sampleRate !== SAMPLE_RATE || audio.length !== samples) {
    throw new Error('선택한 오디오의 길이·채널·샘플 레이트가 영상 설정과 일치하지 않습니다.');
  }
  const output = createOutput(profile);
  let encodedFrames = 0;
  let encodedAudioPackets = 0;
  const video = new CanvasSource(canvas, {
    ...videoConfig(profile), onEncodedPacket: () => { encodedFrames++; },
  });
  output.addVideoTrack(video, { frameRate: FPS });
  const audioSource = profile.audioCodec
    ? new AudioBufferSource({ codec: profile.audioCodec, quality: audioQuality,
      onEncodedPacket: () => { encodedAudioPackets++; } }) : null;
  if (audioSource) output.addAudioTrack(audioSource);
  let cancellation: Promise<void> | undefined;
  const cancel = () => { cancellation ??= output.cancel().catch(() => {}); };
  signal?.addEventListener('abort', cancel, { once: true });
  try {
    await output.start();
    onProgress?.(0);
    for (let frame = 0; frame < frames; frame++) {
      if (signal?.aborted) throw aborted();
      // Feed one-second PCM chunks beside the matching video to bound mux buffering.
      if (audioSource && frame % FPS === 0) {
        const start = frame / FPS * SAMPLE_RATE;
        const length = Math.min(SAMPLE_RATE, samples - start);
        const chunk = new AudioBuffer({ numberOfChannels: 2, length, sampleRate: SAMPLE_RATE });
        for (let channel = 0; channel < 2; channel++) {
          chunk.copyToChannel(audio.getChannelData(channel).subarray(start, start + length), channel);
        }
        await audioSource.add(chunk);
        if (start + length >= samples) audioSource.close();
      }
      const timing = frameTiming(frame);
      await renderFrame(timing.songTime);
      if (signal?.aborted) throw aborted();
      await video.add(timing.timestamp, timing.duration);
      onProgress?.((frame + 1) / frames * 0.98);
      if (frame % 4 === 3) await yieldToUi();
    }
    video.close();
    if (signal?.aborted) throw aborted();
    await output.finalize();
    if (signal?.aborted) throw aborted();
    if (!output.target.buffer?.byteLength || encodedFrames !== frames) {
      throw new Error(`영상 프레임 검증 실패: ${frames}개 중 ${encodedFrames}개가 인코딩되었습니다.`);
    }
    if (audioSource && encodedAudioPackets === 0) {
      throw new Error('오디오가 인코딩되지 않았습니다. 무음 영상으로 대체하지 않습니다.');
    }
    if (audioSource && !await checkAudioTimeline(output.target.buffer, duration, undefined, signal)) {
      throw new Error('인코딩된 오디오 길이가 영상과 1프레임 이상 다릅니다. 다른 지원 형식으로 다시 시도해 주세요.');
    }
    const blob = new Blob([output.target.buffer], { type: profile.mime });
    onProgress?.(1);
    return { blob, extension: profile.extension, mime: profile.mime, frames, duration };
  } catch (error) {
    if (signal?.aborted) throw aborted();
    throw error;
  } finally {
    signal?.removeEventListener('abort', cancel);
    if (output.state !== 'finalized' && output.state !== 'canceled') await output.cancel().catch(() => {});
    await cancellation;
  }
}
