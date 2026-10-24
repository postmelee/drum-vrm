// Original, generated fixture. No recorded song, MIDI performance or samples.
function variable(value: number): number[] {
  const out = [value & 127];
  while ((value >>= 7)) out.unshift((value & 127) | 128);
  return out;
}
function chunk(name: string, data: number[]): number[] {
  return [...name].map(c => c.charCodeAt(0)).concat([data.length >>> 24, data.length >>> 16 & 255, data.length >>> 8 & 255, data.length & 255], data);
}
export function makeDemo(seconds = 30): Uint8Array {
  const ppq = 480;
  const events: { tick: number; note: number; velocity: number }[] = [];
  for (let beat = 0; beat < seconds * 2; beat++) {
    const tick = beat * ppq;
    if (beat % 16 >= 14) {
      for (let j = 0; j < 4; j++) events.push({ tick: tick + j * 120, note: [38, 48, 45, 43][j], velocity: 72 + j * 12 });
    } else {
      events.push({ tick, note: beat % 2 === 0 ? 36 : 38, velocity: beat % 2 === 0 ? 104 : 112 });
      events.push({ tick, note: beat % 16 === 0 ? 49 : 42, velocity: beat % 16 === 0 ? 106 : 76 });
      events.push({ tick: tick + 240, note: beat % 8 === 7 ? 46 : 42, velocity: 58 });
      if (beat % 8 === 6) events.push({ tick: tick + 240, note: 36, velocity: 86 });
      if (beat % 16 === 12) events.push({ tick, note: 44, velocity: 75 });
    }
  }
  const messages = events.flatMap(e => [
    { tick: e.tick, bytes: [0x99, e.note, e.velocity] },
    { tick: e.tick + 30, bytes: [0x89, e.note, 0] },
  ]).sort((a, b) => a.tick - b.tick);
  const track = [0, 255, 3, 13, ...new TextEncoder().encode('Studio Groove')];
  let last = 0;
  for (const m of messages) { track.push(...variable(m.tick - last), ...m.bytes); last = m.tick; }
  track.push(...variable(seconds * 960 - last), 255, 47, 0);
  const conductor = [0, 255, 81, 3, 7, 161, 32, ...variable(seconds * 960), 255, 47, 0];
  return new Uint8Array([...chunk('MThd', [0, 1, 0, 2, 1, 224]), ...chunk('MTrk', conductor), ...chunk('MTrk', track)]);
}
