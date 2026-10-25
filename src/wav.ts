import { outputDuration, PRE_ROLL, SAMPLE_RATE, type PerformancePlan } from './types';

export const MAX_WAV_BYTES = 100 * 1024 * 1024;
export const MAX_WAV_DURATION = 300;
export const MAX_WAV_OFFSET = 300;

export interface WavInfo {
  channels: 1 | 2;
  sampleRate: number;
  bitsPerSample: number;
  frames: number;
  duration: number;
  encoding: 'pcm' | 'float';
}
export interface LocalWav { name: string; buffer: AudioBuffer; info: WavInfo; }
export interface WavAlignmentInfo {
  offsetSeconds: number;
  effectiveOffsetSeconds: number;
  sourceDurationSeconds: number;
  outputDurationSeconds: number;
  copiedSeconds: number;
  trimmedStartSeconds: number;
  trimmedEndSeconds: number;
  leadingSilenceSeconds: number;
  trailingSilenceSeconds: number;
}
export interface WavPcm { channels: readonly Float32Array[]; sampleRate: number; }
export interface AlignedWavPcm {
  channels: [Float32Array<ArrayBuffer>, Float32Array<ArrayBuffer>];
  sampleRate: number;
  info: WavAlignmentInfo;
}

function fail(message: string): never { throw new Error(message); }
const fourCC = (view: DataView, offset: number) => String.fromCharCode(...new Uint8Array(view.buffer, view.byteOffset + offset, 4));

/** Bound decoded size before invoking the browser decoder. Only uncompressed,
 * self-contained RIFF/WAVE is accepted; compressed WAV/RF64 are not inferred. */
export function inspectWav(bytes: ArrayBuffer): WavInfo {
  if (bytes.byteLength > MAX_WAV_BYTES) fail('WAV 파일은 100 MiB 이하로 준비해 주세요.');
  if (bytes.byteLength < 44) fail('유효한 RIFF/WAVE 파일이 아닙니다.');
  const view = new DataView(bytes);
  if (fourCC(view, 0) !== 'RIFF' || fourCC(view, 8) !== 'WAVE') fail('RIFF/WAVE 형식의 WAV 파일만 지원합니다.');
  const end = view.getUint32(4, true) + 8;
  if (end !== bytes.byteLength) fail('WAV 파일의 선언된 크기와 실제 크기가 다릅니다.');
  let format: Omit<WavInfo, 'frames' | 'duration'> | null = null;
  let blockAlign = 0, dataOffset = -1, dataLength = 0;
  for (let offset = 12; offset < end;) {
    if (offset + 8 > end) fail('WAV 청크 헤더가 잘렸습니다.');
    const id = fourCC(view, offset), size = view.getUint32(offset + 4, true);
    const start = offset + 8, next = start + size + (size & 1);
    if (next > end) fail('WAV 청크 데이터가 잘렸습니다.');
    if (id === 'fmt ') {
      if (format || size < 16) fail('WAV 오디오 형식 정보가 없거나 중복되었습니다.');
      let tag = view.getUint16(start, true);
      const channels = view.getUint16(start + 2, true), sampleRate = view.getUint32(start + 4, true);
      const byteRate = view.getUint32(start + 8, true), align = view.getUint16(start + 12, true);
      const bitsPerSample = view.getUint16(start + 14, true);
      if (tag === 0xfffe) {
        if (size < 40 || view.getUint16(start + 16, true) < 22) fail('WAV 확장 형식 정보가 잘렸습니다.');
        const validBits = view.getUint16(start + 18, true);
        const guidTail = [0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113];
        if (validBits < 1 || validBits > bitsPerSample || guidTail.some((value, i) => view.getUint8(start + 26 + i) !== value)) fail('지원하지 않는 WAV 확장 형식입니다.');
        tag = view.getUint16(start + 24, true);
      }
      if (channels !== 1 && channels !== 2) fail('WAV는 모노 또는 스테레오로 준비해 주세요.');
      if (sampleRate < 8000 || sampleRate > 192000) fail('WAV 샘플 레이트는 8–192 kHz 범위여야 합니다.');
      if (!(tag === 1 && [8, 16, 24, 32].includes(bitsPerSample)) && !(tag === 3 && bitsPerSample === 32)) fail('WAV는 PCM 8/16/24/32비트 또는 Float 32비트로 준비해 주세요.');
      if (align !== channels * bitsPerSample / 8 || byteRate !== sampleRate * align) fail('WAV 프레임 크기 또는 전송률 정보가 잘못되었습니다.');
      blockAlign = align;
      format = { channels, sampleRate, bitsPerSample, encoding: tag === 1 ? 'pcm' : 'float' };
    } else if (id === 'data') {
      if (dataOffset >= 0) fail('여러 data 청크로 나뉜 WAV는 지원하지 않습니다.');
      dataOffset = start; dataLength = size;
    }
    offset = next;
  }
  if (!format || dataOffset < 0 || !dataLength || dataLength % blockAlign) fail('WAV 형식 정보 또는 완전한 오디오 프레임이 없습니다.');
  const frames = dataLength / blockAlign, duration = frames / format.sampleRate;
  if (duration > MAX_WAV_DURATION) fail('WAV는 300초 이하로 준비해 주세요.');
  if (format.encoding === 'float') {
    for (let i = dataOffset; i < dataOffset + dataLength; i += 4) {
      if (!Number.isFinite(view.getFloat32(i, true))) fail('WAV에 유효하지 않은 Float 샘플이 있습니다.');
    }
  }
  return { ...format, frames, duration };
}

function validatePcm(source: WavPcm): number {
  if (source.sampleRate !== SAMPLE_RATE) fail('WAV를 48 kHz로 디코딩한 뒤 정렬해야 합니다.');
  if (source.channels.length !== 1 && source.channels.length !== 2) fail('WAV 채널은 1개 또는 2개여야 합니다.');
  const length = source.channels[0].length;
  if (!length || length > MAX_WAV_DURATION * SAMPLE_RATE || source.channels.some(channel => channel.length !== length)) fail('WAV 채널 길이가 다르거나 지원 범위를 벗어납니다.');
  for (const channel of source.channels) for (const value of channel) {
    if (!Number.isFinite(value)) fail('WAV에 유효하지 않은 오디오 샘플이 있습니다.');
  }
  return length;
}

/** File bytes never leave this browser. Decode once at the output sample rate;
 * the returned source buffer stays unshifted so offset edits do not accumulate. */
export async function loadLocalWav(file: File): Promise<LocalWav> {
  if (file.size > MAX_WAV_BYTES) fail('WAV 파일은 100 MiB 이하로 준비해 주세요.');
  const bytes = await file.arrayBuffer(), info = inspectWav(bytes);
  const context = new AudioContext({ sampleRate: SAMPLE_RATE });
  try {
    if (context.sampleRate !== SAMPLE_RATE) fail('이 브라우저에서는 WAV를 48 kHz로 준비할 수 없습니다.');
    let buffer: AudioBuffer;
    try { buffer = await context.decodeAudioData(bytes); }
    catch { return fail('브라우저가 WAV를 디코딩하지 못했습니다. PCM WAV로 다시 내보내 주세요.'); }
    validatePcm({ sampleRate: buffer.sampleRate, channels: Array.from({ length: buffer.numberOfChannels }, (_, i) => buffer.getChannelData(i)) });
    if (buffer.numberOfChannels !== info.channels || Math.abs(buffer.length - Math.round(info.frames * SAMPLE_RATE / info.sampleRate)) > 2) fail('디코딩한 WAV 길이·채널이 파일 정보와 다릅니다.');
    return { name: file.name, buffer, info };
  } finally { await context.close().catch(() => {}); }
}

/** Positive offset places WAV sample zero later in song time. Align in integer
 * output samples once; mono is copied equally, never gain-normalized or mixed. */
function align(source: WavPcm, plan: PerformancePlan, offsetSeconds: number, allocate: (length: number) => AlignedWavPcm['channels']): AlignedWavPcm {
  if (!Number.isFinite(offsetSeconds) || Math.abs(offsetSeconds) > MAX_WAV_OFFSET) fail('WAV 오프셋은 −300초부터 300초까지의 유한한 값이어야 합니다.');
  if (!Number.isFinite(plan.duration) || plan.duration <= 0 || plan.duration > 300) fail('WAV를 정렬할 MIDI 길이가 유효하지 않습니다.');
  const sourceLength = validatePcm(source), length = Math.round(outputDuration(plan) * SAMPLE_RATE);
  const shift = Math.round((PRE_ROLL + offsetSeconds) * SAMPLE_RATE);
  const sourceStart = Math.min(sourceLength, Math.max(0, -shift));
  const outputStart = Math.min(length, Math.max(0, shift));
  const copied = Math.max(0, Math.min(sourceLength - sourceStart, length - outputStart));
  const channels = allocate(length);
  for (let c = 0; c < 2; c++) channels[c].set(source.channels[Math.min(c, source.channels.length - 1)].subarray(sourceStart, sourceStart + copied), outputStart);
  return {
    channels, sampleRate: SAMPLE_RATE,
    info: {
      offsetSeconds, effectiveOffsetSeconds: shift / SAMPLE_RATE - PRE_ROLL,
      sourceDurationSeconds: sourceLength / SAMPLE_RATE, outputDurationSeconds: length / SAMPLE_RATE,
      copiedSeconds: copied / SAMPLE_RATE, trimmedStartSeconds: sourceStart / SAMPLE_RATE,
      trimmedEndSeconds: (sourceLength - sourceStart - copied) / SAMPLE_RATE,
      leadingSilenceSeconds: outputStart / SAMPLE_RATE,
      trailingSilenceSeconds: (length - outputStart - copied) / SAMPLE_RATE,
    },
  };
}

export function alignWavPcm(source: WavPcm, plan: PerformancePlan, offsetSeconds: number): AlignedWavPcm {
  return align(source, plan, offsetSeconds, length => [new Float32Array(length), new Float32Array(length)]);
}

/** Preview and export receive this same aligned buffer, replacing drum synthesis. */
export function alignWavToPlan(source: AudioBuffer, plan: PerformancePlan, offsetSeconds: number): { buffer: AudioBuffer; info: WavAlignmentInfo } {
  let buffer: AudioBuffer | undefined;
  const result = align({ sampleRate: source.sampleRate, channels: Array.from({ length: source.numberOfChannels }, (_, i) => source.getChannelData(i)) }, plan, offsetSeconds, length => {
    buffer = new AudioBuffer({ numberOfChannels: 2, length, sampleRate: SAMPLE_RATE });
    return [buffer.getChannelData(0), buffer.getChannelData(1)];
  });
  return { buffer: buffer!, info: result.info };
}
