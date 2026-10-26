import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { DEMO_CATALOG, loadDemo } from '../src/demo-catalog';
import { makeDemo, makeInterpretationDemo } from '../src/demo';
import type { MidiInfo, PerformancePlan, Selection } from '../src/types';
import { initSync, inspect_midi, Performance } from '../src/wasm/drum_engine';

const PUBLIC_MIDI_SHA256 = '615127ef5d0bfe7c4e934222349243e8d7c9c8309fa9e1a56e0b19bb0e41f5cf';
const readPublicMidi = () => new Uint8Array(readFileSync(new URL('../public/demos/transparent-girl-1.mid', import.meta.url)));
const midiFetcher = (bytes = readPublicMidi()) => vi.fn<typeof fetch>(async () => new Response(Uint8Array.from(bytes)));

beforeAll(() => {
  initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) });
});

describe('selectable MIDI demos', () => {
  it('keeps all four demos discoverable with distinct IDs and filenames', () => {
    expect(DEMO_CATALOG.map(demo => demo.id)).toEqual(['posture', 'studio', 'interpretation', 'transparent-girl']);
    expect(new Set(DEMO_CATALOG.map(demo => demo.filename)).size).toBe(4);
    expect(DEMO_CATALOG.every(demo => demo.label.trim() && demo.filename.endsWith('.mid'))).toBe(true);
  });

  it.each([
    ['studio', makeDemo],
    ['interpretation', makeInterpretationDemo],
  ] as const)('generates %s locally without fetching a bundled song', async (id, generate) => {
    const fetcher = vi.fn<typeof fetch>().mockRejectedValue(new Error('Generated demos must not fetch'));
    const demo = await loadDemo(id, '/drum-vrm/', fetcher);
    expect(fetcher).not.toHaveBeenCalled();
    expect(demo.bytes).toEqual(generate());
    expect(demo.filename).toBe(DEMO_CATALOG.find(entry => entry.id === id)!.filename);
    expect(demo.selection.omittedSourceEventIds ?? []).toEqual([]);
    expect(demo.selection.handOverrides ?? {}).toEqual({});
  });

  it.each([
    ['/drum-vrm/', '/drum-vrm/demos/transparent-girl-1.mid'],
    ['/drum-vrm', '/drum-vrm/demos/transparent-girl-1.mid'],
    ['/', '/demos/transparent-girl-1.mid'],
  ])('loads the bundled MIDI under deployment base %s', async (base, expectedUrl) => {
    const source = readPublicMidi(), fetcher = midiFetcher(source);
    const demo = await loadDemo('transparent-girl', base, fetcher);
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(String(fetcher.mock.calls[0][0])).toBe(expectedUrl);
    expect(demo.bytes).toEqual(source);
    expect(createHash('sha256').update(demo.bytes).digest('hex')).toBe(PUBLIC_MIDI_SHA256);
    expect(demo.filename).toBe(DEMO_CATALOG.find(entry => entry.id === 'transparent-girl')!.filename);
    expect(demo.selection).toMatchObject({
      mappingProfile: 'logicDkdGm', countInNote75: true,
      hihatPolicy: 'shortOpenClose', hihatOpenness: .2,
      omittedSourceEventIds: ['23:237'], handOverrides: {},
    });
  });

  it('rejects a missing demo without trying to interpret the error response body', async () => {
    const readBody = vi.fn(async () => new ArrayBuffer(0));
    const fetcher = vi.fn<typeof fetch>().mockResolvedValue({ ok: false, status: 404, arrayBuffer: readBody } as unknown as Response);
    await expect(loadDemo('transparent-girl', '/drum-vrm/', fetcher)).rejects.toThrow();
    expect(readBody).not.toHaveBeenCalled();
  });

  it('rejects changed MIDI bytes and successful HTTP responses containing non-MIDI content', async () => {
    const changed = readPublicMidi();
    changed[changed.length - 1] ^= 1;
    await expect(loadDemo('transparent-girl', '/drum-vrm/', midiFetcher(changed))).rejects.toThrow();
    const html = vi.fn<typeof fetch>(async () => new Response('<html>Not a MIDI file</html>'));
    await expect(loadDemo('transparent-girl', '/drum-vrm/', html)).rejects.toThrow();
  });

  it('does not let later arrangement edits contaminate another demo load', async () => {
    const first = await loadDemo('transparent-girl', '/drum-vrm/', midiFetcher());
    first.selection.omittedSourceEventIds!.push('0:0');
    first.selection.handOverrides!['0:0'] = 1;
    first.bytes[0] = 0;
    const second = await loadDemo('transparent-girl', '/drum-vrm/', midiFetcher());
    expect(second.selection.omittedSourceEventIds).toEqual(['23:237']);
    expect(second.selection.handOverrides).toEqual({});
    expect(createHash('sha256').update(second.bytes).digest('hex')).toBe(PUBLIC_MIDI_SHA256);
    const generated = await loadDemo('studio', '/drum-vrm/', midiFetcher());
    expect(generated.selection.omittedSourceEventIds ?? []).toEqual([]);
    expect(generated.selection.handOverrides ?? {}).toEqual({});
  });

  it('preserves the public source MIDI while applying the approved final chord through WASM', async () => {
    const demo = await loadDemo('transparent-girl', '/drum-vrm/', midiFetcher());
    const info = JSON.parse(inspect_midi(demo.bytes)) as MidiInfo;
    const selection: Selection = { tracks: info.suggestedTracks, channel: null, ...demo.selection };
    const source = new Performance(demo.bytes, JSON.stringify({ ...selection, omittedSourceEventIds: [] }));
    const arranged = new Performance(demo.bytes, JSON.stringify(selection));
    try {
      const sourcePlan = JSON.parse(source.plan_json()) as PerformancePlan;
      const plan = JSON.parse(arranged.plan_json()) as PerformancePlan;
      const sourceNotes = plan.sourceEvents!.filter(event => event.message.kind === 'noteOn' && event.message.velocity > 0);
      expect(sourceNotes).toHaveLength(1760);
      expect(plan.sourceEvents).toEqual(sourcePlan.sourceEvents);
      expect(plan.duration).toBe(sourcePlan.duration);
      expect(plan.hits).toHaveLength(1720);
      expect(sourcePlan.hits).toHaveLength(1721);
      expect(plan.controls).toHaveLength(36);
      expect(plan.interpretations!.filter(event => event.kind === 'countIn')).toHaveLength(3);
      expect(plan.interpretations!.filter(event => event.kind === 'unmapped')).toHaveLength(0);
      expect(plan.interpretations!.filter(event => event.kind === 'omitted').map(event => event.sourceEventId)).toEqual(['23:237']);

      const omitted = sourceNotes.find(event => event.id === '23:237')!;
      expect(omitted.message).toMatchObject({ kind: 'noteOn', key: 32 });
      expect(omitted.timeUs / 1e6).toBeCloseTo(191.804, 3);
      expect(plan.hits.some(hit => hit.sourceEventId === omitted.id)).toBe(false);
      const finalChord = plan.hits.filter(hit => hit.timeUs === omitted.timeUs);
      expect(finalChord.map(hit => hit.note).sort((a, b) => a - b)).toEqual([36, 49, 57]);
      expect(finalChord.map(hit => hit.pieceId).sort()).toEqual(['crash', 'crashRight', 'kick']);
      expect(Math.max(...plan.hits.map(hit => hit.timeUs!))).toBe(omitted.timeUs);

      // Compare retained evidence, not planner-dependent limb choices or large snapshots.
      const facts = (input: PerformancePlan) => input.hits
        .filter(hit => hit.sourceEventId !== omitted.id)
        .map(hit => [hit.sourceEventId, hit.tick, hit.timeUs, hit.time, hit.note, hit.velocity, hit.pieceId]);
      expect(facts(plan)).toEqual(facts(sourcePlan));
      expect(createHash('sha256').update(demo.bytes).digest('hex')).toBe(PUBLIC_MIDI_SHA256);
    } finally {
      source.free(); arranged.free();
    }
  });
});
