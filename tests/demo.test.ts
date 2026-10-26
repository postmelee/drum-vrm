import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { makeDemo, makeInterpretationDemo } from '../src/demo';
import { sampleHihatOpening, resolveContactEvent } from '../src/contact-events';
import { fit_kit, initSync, inspect_midi, Performance } from '../src/wasm/drum_engine';
import { legacyPositionToNative, NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import { outputFrameCount, type Hit, type KitFitProposal, type MidiInfo, type PerformancePlan, type Selection } from '../src/types';

beforeAll(() => {
  initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) });
});

it('the authored interpretation study preserves source notes while making count-in and pedal intent explicit', () => {
  const bytes = makeInterpretationDemo();
  const info = JSON.parse(inspect_midi(bytes)) as MidiInfo;
  expect(info.tracks[1].noteCount).toBe(91);
  expect(info.tracks[1].channels).toEqual([0]);
  const literal = new Performance(bytes, JSON.stringify({ tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true }));
  const interpreted = new Performance(bytes, JSON.stringify({ tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true, hihatPolicy: 'shortOpenClose', hihatOpenness: .2 }));
  try {
    const sourcePlan = JSON.parse(literal.plan_json()) as PerformancePlan;
    const plan = JSON.parse(interpreted.plan_json()) as PerformancePlan;
    expect(plan.duration).toBeCloseTo(8, 4);
    expect(plan.sourceEvents).toEqual(sourcePlan.sourceEvents);
    expect(plan.interpretations?.filter(event => event.kind === 'countIn')).toHaveLength(3);
    expect(plan.controls).toHaveLength(6);
    expect(plan.hits).toHaveLength(82);
    expect(plan.hits.filter(hit => hit.note === 32)[0].articulation).toBe('rimshotEdge');
    expect(new Set(plan.hits.filter(hit => hit.instrument === 'crash').map(hit => hit.pieceId))).toEqual(new Set(['crash', 'crashRight']));
    expect(plan.diagnostics.filter(diagnostic => diagnostic.code === 'UNMAPPED_NOTE')).toHaveLength(0);
    for (const control of plan.controls ?? []) {
      expect(interpreted.evaluate_hihat_at(control.openTimeUs / 1e6)).toBeCloseTo(.2, 9);
      expect(interpreted.evaluate_hihat_at(control.timeUs / 1e6)).toBe(0);
      expect(plan.hits.some(hit => hit.sourceEventId === control.sourceEventId)).toBe(false);
    }
  } finally { literal.free(); interpreted.free(); }
});

it('the original 30-second demo crosses the actual WASM boundary with complete expected events', () => {
  const bytes = makeDemo();
  const info = JSON.parse(inspect_midi(bytes)) as MidiInfo;
  expect(info.format).toBe(1);
  expect(info.ppq).toBe(480);
  expect(info.suggestedTracks).toEqual([1]);
  expect(info.tracks[1].channels).toEqual([9]);
  // 54 ordinary beats × 3 hits + 6 fill beats × 4 hits + 4 extra kicks + 3 pedal hits.
  expect(info.tracks[1].noteCount).toBe(193);
  const performance = new Performance(bytes, JSON.stringify({ tracks: [1], channel: 9 }));
  try {
    const plan = JSON.parse(performance.plan_json()) as PerformancePlan;
    expect(plan.duration).toBe(30);
    expect(plan.hits).toHaveLength(193);
    expect(plan.hits[0].time).toBe(0);
    expect(plan.hits.every(hit => hit.channel === 9 && hit.velocity > 0 && hit.time < 30)).toBe(true);
    expect(new Set(plan.hits.map(hit => hit.instrument))).toEqual(new Set([
      'kick', 'snare', 'hihat', 'openHat', 'pedalHat', 'crash', 'tomHigh', 'tomMid', 'tomLow',
    ]));
    expect(outputFrameCount(plan)).toBe(923);
    const pose = performance.evaluate_at(0);
    expect(pose).toHaveLength(57);
    expect(pose.every(Number.isFinite)).toBe(true);
    // The JS array is owned; another evaluation must not overwrite an earlier pose.
    const original = Array.from(pose);
    performance.evaluate_at(15);
    expect(Array.from(pose)).toEqual(original);
  } finally {
    performance.free();
  }
});

function prepareNative(bytes: Uint8Array, selection: Selection, fitted = true) {
  const source = new Performance(bytes, JSON.stringify(selection));
  const original = JSON.parse(source.plan_json()) as PerformancePlan;
  source.free();
  const rig = new NativeHumanoidRigAdapter();
  const profile = JSON.stringify(rig.getProfile());
  const input = structuredClone(original);
  if (fitted) {
    const proposal = JSON.parse(fit_kit(profile, JSON.stringify(input.kit))) as KitFitProposal;
    expect(proposal.complete).toBe(true);
    input.kit = proposal.kit;
    input.hits.forEach(hit => { hit.target = [...proposal.kit.find(piece => piece.id === hit.pieceId)!.position]; });
  }
  // Unmeasured source targets retain anchors plus the physical upper-hat state;
  // a fitted input is explicitly reset and will derive its timed contacts again.
  for (const hit of input.hits) {
    const expected = [...input.kit.find(piece => piece.id === hit.pieceId)!.position];
    if (!fitted && ['hihat', 'openHat'].includes(hit.instrument)) expected[1] += sampleHihatOpening(input.hihatMotion, hit.time) * (input.hihatMotion?.travelMeters ?? 0);
    hit.target.forEach((value, axis) => expect(value).toBeCloseTo(expected[axis], 12));
  }
  const performance = Performance.from_json_with_profile(JSON.stringify(input), profile);
  const plan = JSON.parse(performance.plan_json()) as PerformancePlan;
  const sourceFacts = (hit: Hit) => [hit.sourceEventId, hit.tick, hit.timeUs, hit.time, hit.note, hit.velocity];
  expect(plan.hits.map(sourceFacts)).toEqual(original.hits.map(sourceFacts));
  for (const [index, hit] of plan.hits.entries()) {
    expect([hit.instrument, hit.pieceId]).toEqual([input.hits[index].instrument, input.hits[index].pieceId]);
    const piece = plan.kit.find(piece => piece.id === hit.pieceId)!, geometry = piece.geometry;
    const contact = plan.contactEvents!.find(event => event.hitId === hit.id && event.role === 'primary')!;
    const displacement = contact.mechanismOffset;
    if (!geometry || hit.instrument === 'kick' || hit.instrument === 'pedalHat') {
      // JSON's JS/Rust float round trip can change the last representable bit.
      hit.target.forEach((value, axis) => expect(value).toBeCloseTo(piece.position[axis] + displacement[axis], 12));
    } else if (geometry.surface.radialProfile) {
      resolveContactEvent(contact, plan.kit, plan.hihatMotion)!.position.forEach((value, axis) => expect(value).toBeCloseTo(hit.target[axis], 12));
      expect(contact.localNormal?.every(Number.isFinite)).toBe(true);
    } else {
      const scale = piece.radius / geometry.referenceRadius;
      const center = piece.position.map((value, axis) => value + geometry.surface.center[axis] * scale);
      const delta = hit.target.map((value, axis) => value - center[axis] - displacement[axis]);
      expect(Math.abs(delta.reduce((dot, value, axis) => dot + value * geometry.surface.normal[axis], 0))).toBeLessThan(1e-9);
      expect(Math.hypot(...delta)).toBeLessThanOrEqual(geometry.surface.radius * scale * .95 + 1e-9);
      // This demo uses center heads, bow hats and edgeward crash contacts.
      const fraction = ({ bell: .12, inner: .38, outer: .62, edge: .86, rimshotEdge: .86, sideStick: .86, crash: .88, china: .88, splash: .88 } as Record<string, number>)[hit.articulation ?? ""] ?? (['hihat', 'openHat'].includes(hit.instrument) ? .62 : 0);
      expect(Math.hypot(...delta)).toBeCloseTo(geometry.surface.radius * scale * fraction, 10);
    }
  }
  if (fitted) {
    const repeated = Performance.from_json_with_profile(JSON.stringify(input), profile);
    try { expect((JSON.parse(repeated.plan_json()) as PerformancePlan).hits).toEqual(plan.hits); }
    finally { repeated.free(); }
  }
  expect(plan.sourceEvents).toEqual(original.sourceEvents);
  if (fitted) expect(plan.hits.filter(hit => hit.limb === null)).toHaveLength(0);
  return { rig, performance, plan, close() { performance.free(); rig.dispose(); } };
}

it('the automatically fitted browser demo keeps right-hand hats and left-hand backbeats with the actual native rig', () => {
  const session = prepareNative(makeDemo(), { tracks: [1], channel: 9 });
  try {
    const { rig, performance, plan } = session;
    const hats = plan.hits.filter(hit => hit.instrument === 'hihat' || hit.instrument === 'openHat');
    expect(hats.length).toBeGreaterThan(0);
    expect(hats.filter(hit => hit.limb !== 1).map(hit => [hit.time, hit.limb])).toEqual([]);
    const backbeats = plan.hits.filter(hit => hit.instrument === 'snare' && (hit.tick / 480) % 16 < 14);
    expect(backbeats.length).toBeGreaterThan(0);
    expect(backbeats.filter(hit => hit.limb !== 0).map(hit => [hit.time, hit.limb])).toEqual([]);
    // These crashes follow a fast floor-tom fill, not an uninterrupted hat beat.
    // Their source instrument and timestamp remain unchanged above; fitted
    // geometry chooses an explicit contact region on that instrument.
    const fillCrashes = plan.hits.filter(hit => hit.instrument === 'crash' && hit.time > 0);
    expect(fillCrashes.map(hit => [hit.time, hit.limb])).toEqual([[8, 0], [16, 0], [24, 0]]);

    // Check anatomical hand-local contact markers, not screen-left/screen-right.
    for (const hit of [hats[0], backbeats[0], fillCrashes[0]]) {
      rig.applyPose(performance.evaluate_rig_at(hit.time));
      const sample = rig.getContactSamples()[hit.limb!];
      const target = legacyPositionToNative(hit.target);
      expect(Math.hypot(...sample.actual.map((value, axis) => value - target[axis]))).toBeLessThan(1e-6);
    }

    const snare = plan.kit.find(piece => piece.id === 'snare')!.position;
    const ready = [snare[0], snare[1] + .035, snare[2]];
    const times = [.95, 1, 1.15, 2, 5];
    const poses = times.map(time => Array.from(performance.evaluate_rig_at(time)));
    for (const index of [4, 1, 3, 0, 2]) {
      const pose = performance.evaluate_rig_at(times[index]);
      expect(Array.from(pose)).toEqual(poses[index]);
      // Left hand has recovered from its backbeat and waits above the snare.
      expect(Math.hypot(...ready.map((value, axis) => pose[9 + axis] - value))).toBeLessThan(1e-6);
    }
  } finally { session.close(); }
},60_000); // Includes two complete WASM equipment/joint plans, not just pose evaluation.

it('an unfitted kit preserves source events and reports contacts outside the native rig capability', () => {
  const session = prepareNative(makeDemo(), { tracks: [1], channel: 9 }, false);
  try {
    const unavailable = session.plan.hits.filter(hit => hit.limb === null);
    expect(unavailable.length).toBeGreaterThan(0);
    expect(session.plan.diagnostics.some(item => item.code === 'RIG_UNREACHABLE_HIT')).toBe(true);
    for (const hit of unavailable) {
      expect(session.plan.contactEvents!.filter(event => event.hitId === hit.id).every(event => event.status === 'unassigned')).toBe(true);
    }
  } finally { session.close(); }
}, 60_000);

it('the browser interpretation study preserves groove roles without disabling two-hand solo rolls', () => {
  const session = prepareNative(makeInterpretationDemo(), {
    tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true,
    hihatPolicy: 'shortOpenClose', hihatOpenness: .2,
  });
  try {
    const hats = session.plan.hits.filter(hit => hit.instrument === 'hihat' || hit.instrument === 'openHat');
    expect(hats.filter(hit => hit.limb !== 1).map(hit => [hit.time, hit.limb])).toEqual([]);
    const backbeats = session.plan.hits.filter(hit => hit.instrument === 'snare' && hit.tick < 10 * 480);
    expect(backbeats).toHaveLength(3);
    expect(backbeats.map(hit => hit.limb)).toEqual([0, 0, 0]);
    const soloRoll = session.plan.hits.filter(hit => hit.instrument === 'snare' && hit.tick >= 11 * 480 && hit.tick < 17 * 480);
    expect(soloRoll).toHaveLength(24);
    expect(new Set(soloRoll.map(hit => hit.limb))).toEqual(new Set([0, 1]));
  } finally { session.close(); }
}, 60_000);

it('an explicit source-event hand edit overrides the default groove role', () => {
  const bytes = makeDemo(3), selection = { tracks: [1], channel: 9 };
  const source = new Performance(bytes, JSON.stringify(selection));
  const sourcePlan = JSON.parse(source.plan_json()) as PerformancePlan;
  source.free();
  const hat = sourcePlan.hits.find(hit => hit.instrument === 'hihat')!;
  const session = prepareNative(bytes, { ...selection, handOverrides: { [hat.sourceEventId!]: 0 } });
  try {
    expect(session.plan.hits.find(hit => hit.sourceEventId === hat.sourceEventId)?.limb).toBe(0);
    expect(session.plan.handOverrides).toEqual({ [hat.sourceEventId!]: 0 });
  } finally { session.close(); }
}, 60_000);
