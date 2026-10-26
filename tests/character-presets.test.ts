import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { describe, expect, it, vi } from 'vitest';
import { CHARACTER_PRESETS, characterAssetUrl, loadCharacterPreset } from '../src/character-presets';
import { inspectVRM } from '../src/vrm-import';

const readModel = (file: string) => Uint8Array.from(readFileSync(new URL(`../public/characters/${file}`, import.meta.url)));

describe('public character presets', () => {
  it('provides unique, pinned models with distribution links', () => {
    expect(CHARACTER_PRESETS).toHaveLength(3);
    expect(new Set(CHARACTER_PRESETS.map(preset => preset.id)).size).toBe(CHARACTER_PRESETS.length);
    expect(new Set(CHARACTER_PRESETS.map(preset => preset.sha256)).size).toBe(CHARACTER_PRESETS.length);
    for (const preset of CHARACTER_PRESETS) {
      expect(preset.label.trim()).not.toBe('');
      expect(preset.creator.trim()).not.toBe('');
      expect(preset.source).toMatch(/^https:\/\//);
      expect(preset.licenseUrl).toMatch(/^https:\/\//);
      expect(preset.sha256).toMatch(/^[0-9a-f]{64}$/);
      expect(preset.file).toMatch(/^[\w.-]+\.vrm$/);
    }
  });

  it.each(CHARACTER_PRESETS)('checks and imports the actual $title bytes under the Pages base', async preset => {
    const bytes = readModel(preset.file), fetcher = vi.fn<typeof fetch>(async () => new Response(bytes));
    const file = await loadCharacterPreset(preset.id, undefined, '/drum-vrm/', fetcher);
    expect(fetcher.mock.calls[0][0]).toBe(`/drum-vrm/characters/${preset.file}`);
    expect(file.name).toBe(preset.file);
    expect(file.size).toBe(preset.bytes);
    const actual = await file.arrayBuffer();
    expect(createHash('sha256').update(new Uint8Array(actual)).digest('hex')).toBe(preset.sha256);
    const info = inspectVRM(actual);
    expect(info.version).toBe('1.0');
    expect(info.required).toBe(15);
    expect(info.missing).toEqual([]);
  });

  it('resolves both root and repository deployments without a duplicated slash', () => {
    expect(characterAssetUrl('test.vrm', '/')).toBe('/characters/test.vrm');
    expect(characterAssetUrl('test.vrm', '/drum-vrm')).toBe('/drum-vrm/characters/test.vrm');
  });

  it('rejects HTTP errors before reading their body and rejects unknown IDs', async () => {
    const body = vi.fn(), fetcher = vi.fn<typeof fetch>(async () => ({ ok: false, status: 404, arrayBuffer: body } as unknown as Response));
    await expect(loadCharacterPreset(CHARACTER_PRESETS[0].id, undefined, '/', fetcher)).rejects.toThrow();
    expect(body).not.toHaveBeenCalled();
    fetcher.mockClear();
    await expect(loadCharacterPreset('missing', undefined, '/', fetcher)).rejects.toThrow();
    expect(fetcher).not.toHaveBeenCalled();
  });

  it('rejects truncated or modified assets rather than importing them', async () => {
    const preset = CHARACTER_PRESETS[0], bytes = readModel(preset.file);
    await expect(loadCharacterPreset(preset.id, undefined, '/', async () => new Response('<html>not a model</html>'))).rejects.toThrow();
    bytes[bytes.length - 1] ^= 1;
    await expect(loadCharacterPreset(preset.id, undefined, '/', async () => new Response(bytes))).rejects.toThrow();
  });

  it('honors cancellation before a request and after the download resolves', async () => {
    const preset = CHARACTER_PRESETS[0], controller = new AbortController();
    controller.abort();
    const unused = vi.fn<typeof fetch>();
    await expect(loadCharacterPreset(preset.id, controller.signal, '/', unused)).rejects.toMatchObject({ name: 'AbortError' });
    expect(unused).not.toHaveBeenCalled();
    const duringBody = new AbortController();
    const fetcher = vi.fn<typeof fetch>(async (_url, options) => {
      expect(options?.signal).toBe(duringBody.signal);
      return { ok: true, arrayBuffer: async () => { duringBody.abort(); return new ArrayBuffer(0); } } as Response;
    });
    await expect(loadCharacterPreset(preset.id, duringBody.signal, '/', fetcher)).rejects.toMatchObject({ name: 'AbortError' });
  });
});
