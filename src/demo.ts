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

/** Original eight-second study, authored to expose interpretation changes.
 * 0–1 s: count-in; 1 s: two crashes; 1⅓–3 s: short open/close hats;
 * 3⅓ s: rimshot edge; 3⅔–5⅔ s: solo roll; 5⅔–7⅔ s: roll with hats.
 * No phrase, timing sequence or performance was copied from an uploaded song. */
export function makeInterpretationDemo(): Uint8Array {
  const ppq = 480, endTick = 24 * ppq;
  const events: { tick: number; note: number; velocity: number }[] = [];
  const add = (beat: number, note: number, velocity: number) => events.push({ tick: Math.round(beat * ppq), note, velocity });
  for (let beat = 0; beat < 3; beat++) add(beat, 75, 85);
  for (const beat of [3, 23]) { add(beat, 49, 110); add(beat, 57, 105); add(beat, 36, 100); }
  for (let beat = 4; beat < 10; beat++) {
    const velocity = beat % 2 === 0 ? 88 : 55;
    add(beat, 46, velocity); add(beat + .075, 44, velocity);
    add(beat, beat % 2 === 0 ? 36 : 40, beat % 2 === 0 ? 95 : 100);
  }
  add(10, 32, 100);
  for (let index = 0; index < 24; index++) {
    add(11 + index / 4, 38, index % 2 === 0 ? 110 : 60);
    add(17 + index / 4, 38, index % 2 === 0 ? 100 : 65);
    if (index % 2 === 0) add(17 + index / 4, 42, 70);
    if (index % 8 === 0) add(17 + index / 4, 36, 95);
  }
  const messages = events.flatMap(event => [
    { tick: event.tick, bytes: [0x90, event.note, event.velocity] },
    { tick: event.tick + 24, bytes: [0x80, event.note, 0] },
  ]).sort((a, b) => a.tick - b.tick);
  const name = [...new TextEncoder().encode('Interpretation Study')];
  const track = [0, 255, 3, name.length, ...name];
  let last = 0;
  for (const message of messages) { track.push(...variable(message.tick - last), ...message.bytes); last = message.tick; }
  track.push(...variable(endTick - last), 255, 47, 0);
  // 333333 us/q is the nearest integer SMF representation of 180 BPM.
  const conductor = [0, 255, 81, 3, 5, 22, 21, ...variable(endTick), 255, 47, 0];
  return new Uint8Array([...chunk('MThd', [0, 1, 0, 2, 1, 224]), ...chunk('MTrk', conductor), ...chunk('MTrk', track)]);
}

/** Independently authored regression score: count-in, sustained groove, tom
 * transfer, ride and dual crash, followed by a one-second release. */
export function makePostureDemo(): Uint8Array {
  const events: { tick: number; note: number; velocity: number }[] = [];
  const add = (seconds: number, note: number, velocity: number) => events.push({ tick: Math.round(seconds * 960), note, velocity });
  for (const time of [0, .5, 1]) add(time, 75, 80);
  for (let index = 0; index < 16; index++) {
    const time = 2 + index * .25;
    add(time, 42, index % 2 ? 55 : 90);
    if (index % 4 === 0) add(time, 36, 95);
    if (index % 4 === 2) add(time, 38, 100);
  }
  for (let index = 0; index < 16; index++) add(6 + index * .125, [38, 48, 45, 43][Math.floor(index / 4)], index % 2 ? 65 : 100);
  for (let index = 0; index < 8; index++) {
    add(8 + index * .25, index % 4 === 0 ? 53 : 51, index % 2 ? 60 : 88);
    if (index % 4 === 2) add(8 + index * .25, 38, 95);
  }
  add(10.5, 49, 110); add(10.5, 57, 110); add(10.5, 36, 105);
  const messages = events.flatMap(e => [{ tick: e.tick, bytes: [0x90, e.note, e.velocity] }, { tick: e.tick + 24, bytes: [0x80, e.note, 0] }]).sort((a, b) => a.tick - b.tick);
  const name = [...new TextEncoder().encode('Posture Study')], track = [0, 255, 3, name.length, ...name];
  let last = 0;
  for (const message of messages) { track.push(...variable(message.tick - last), ...message.bytes); last = message.tick; }
  track.push(...variable(12 * 960 - last), 255, 47, 0);
  const conductor = [0, 255, 81, 3, 7, 161, 32, ...variable(12 * 960), 255, 47, 0];
  return new Uint8Array([...chunk('MThd', [0, 1, 0, 2, 1, 224]), ...chunk('MTrk', conductor), ...chunk('MTrk', track)]);
}
