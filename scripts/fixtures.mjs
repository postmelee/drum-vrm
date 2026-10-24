// Original MIDI encodings for local browser acceptance checks. No external songs.
// Run: node scripts/fixtures.mjs. Generated files stay in ignored .local/fixtures/.
import { mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = join(root, '.local/fixtures');
const ppq = 480;
const vlq = value => {
  if (!Number.isInteger(value) || value < 0 || value > 0x0fffffff) throw new Error('Invalid delta tick');
  const bytes = [value & 127];
  while ((value = Math.floor(value / 128))) bytes.unshift((value & 127) | 128);
  return bytes;
};
const chunk = (name, data) => [
  ...Buffer.from(name), data.length >>> 24, data.length >>> 16 & 255,
  data.length >>> 8 & 255, data.length & 255, ...data,
];
const event = (tick, bytes) => ({ tick, bytes });
const nameEvent = name => event(0, [0xff, 3, ...vlq(Buffer.byteLength(name)), ...Buffer.from(name)]);
const tempo = (tick, micros) => event(tick, [0xff, 0x51, 3, micros >>> 16 & 255, micros >>> 8 & 255, micros & 255]);
function notes(events, tick, channel, note, velocity = 96) {
  events.push(event(tick, [0x90 | channel, note, velocity]));
  events.push(event(tick + 30, [0x80 | channel, note, 0]));
}
function track(events, endTick) {
  let last = 0;
  const data = [];
  for (const e of events.toSorted((a, b) => a.tick - b.tick)) {
    data.push(...vlq(e.tick - last), ...e.bytes);
    last = e.tick;
  }
  data.push(...vlq(endTick - last), 0xff, 0x2f, 0);
  return chunk('MTrk', data);
}
const midi = tracks => new Uint8Array([
  ...chunk('MThd', [0, tracks.length === 1 ? 0 : 1, 0, tracks.length, ppq >>> 8, ppq & 255]),
  ...tracks.flat(),
]);

await mkdir(output, { recursive: true });
const drums = [nameEvent('Channel 2 - Two Tempos')];
for (let halfBeat = 0; halfBeat < 90; halfBeat++) {
  const tick = halfBeat * 240;
  notes(drums, tick, 1, 42, halfBeat % 2 ? 60 : 80);
  if (halfBeat % 2 === 0) notes(drums, tick, 1, halfBeat % 4 === 0 ? 36 : 38, 105);
}
const conductor = [nameEvent('Conductor'), tempo(0, 500_000), tempo(14_400, 1_000_000)];
await writeFile(join(output, 'two-tempos-channel2-30s.mid'), midi([track(conductor, 21_600), track(drums, 21_600)]));

const simultaneous = [nameEvent('Five Limbs Requested'), tempo(0, 500_000)];
for (const note of [38, 42, 49, 36, 44, 99]) notes(simultaneous, 0, 9, note);
simultaneous.push(event(240, [0x99, 51, 0])); // NoteOn velocity zero is not a hit.
await writeFile(join(output, 'simultaneous.mid'), midi([track(simultaneous, 960)]));

await writeFile(join(output, 'invalid.mid'), new Uint8Array([
  ...chunk('MThd', [0, 0, 0, 1, 1, 224]), ...Buffer.from('MTrk'), 0, 0, 0, 16, 0, 0xff, 0x2f, 0,
]));
for (const [file, endTick] of [['duration-300s.mid', 288_000], ['duration-over-300s.mid', 288_001]]) {
  const events = [nameEvent(file), tempo(0, 500_000)];
  notes(events, 0, 9, 36);
  await writeFile(join(output, file), midi([track(events, endTick)]));
}
const dense = [nameEvent('Original Dense Cancel Fixture'), tempo(0, 500_000)];
for (let i = 0; i < 5_000; i++) notes(dense, i * 48, 9, 49, 80);
await writeFile(join(output, 'dense-cancel-300s.mid'), midi([track(dense, 288_000)]));

const expected = {
  'two-tempos-channel2-30s.mid': {
    format: 1, ppq: 480, suggestedTracks: [1], selection: { tracks: [1], channel: 1 },
    duration: 30, hits: 135, instruments: { hihat: 90, kick: 23, snare: 22 }, tempoCount: 2,
    referenceTimes: { tick0: 0, tick480: 0.5, tick14400: 15, tick14880: 16, tick21360: 29.5 },
    output: { frames: 923, duration: 923 / 30 },
    note: 'UI channel 2 is wire index 1. Both channels all and 2 preserve these results; conductor track 0 supplies tempo without being selected.',
  },
  'simultaneous.mid': {
    format: 0, duration: 1, hits: 5, tempoCount: 1, unmapped: { note: 99, count: 1 },
    unresolvedHits: 1, assignedLimbs: [0, 1, 2, 3], allHitTimes: 0, ignoredVelocityZeroNote: 51,
    note: 'Five mapped source hits stay audible; only four can be shown by four limbs. Which hand hit remains unresolved follows stable planner cost.',
  },
  'invalid.mid': { errorIncludes: 'MIDI 트랙 데이터가 잘렸습니다.' },
  'duration-300s.mid': { duration: 300, hits: 1, outputFrames: 9023, note: 'Must be accepted at exact duration boundary.' },
  'duration-over-300s.mid': { errorIncludes: '300초 이하', note: '288001 ticks rounds to 300.001042 seconds and must be rejected.' },
  'dense-cancel-300s.mid': { duration: 300, hits: 5000, note: 'Cancellation exercise: cancel during synthesis, then load the 30-second file. No completion-speed guarantee for pathological cymbal density.' },
};
await writeFile(join(output, 'expected.json'), JSON.stringify(expected, null, 2) + '\n');
console.log(`Generated ${Object.keys(expected).length} original MIDI fixtures and expected.json in .local/fixtures/.`);
