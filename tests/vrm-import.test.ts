import { describe, expect, it, vi } from 'vitest';
import { inspectVRM, MAX_VRM_BYTES, MAX_VRM_IMAGES, REQUIRED_BONES } from '../src/vrm-import';

type Document = Record<string, any>;
function source(): Document {
  return { asset: { version: '2.0' }, nodes: REQUIRED_BONES.map(name => ({ name })), extensions: { VRMC_vrm: {
    specVersion: '1.0', humanoid: { humanBones: Object.fromEntries(REQUIRED_BONES.map((name, node) => [name, { node }])) },
    meta: { name: 'Original test humanoid', authors: ['Drum VRM tests'], licenseUrl: 'https://vrm.dev/licenses/1.0/' },
  } } };
}
function glb(json: Document, binary?: Uint8Array): ArrayBuffer {
  const encoded = new TextEncoder().encode(JSON.stringify(json));
  const jsonLength = Math.ceil(encoded.length / 4) * 4, binLength = Math.ceil((binary?.length ?? 0) / 4) * 4;
  const result = new ArrayBuffer(20 + jsonLength + (binary ? 8 + binLength : 0));
  const view = new DataView(result); view.setUint32(0, 0x46546c67, true); view.setUint32(4, 2, true); view.setUint32(8, result.byteLength, true);
  view.setUint32(12, jsonLength, true); view.setUint32(16, 0x4e4f534a, true);
  new Uint8Array(result, 20, jsonLength).fill(0x20); new Uint8Array(result, 20, encoded.length).set(encoded);
  if (binary) { view.setUint32(20 + jsonLength, binLength, true); view.setUint32(24 + jsonLength, 0x004e4942, true); new Uint8Array(result, 28 + jsonLength, binary.length).set(binary); }
  return result;
}
function png(width: number, height: number): Uint8Array {
  // A generated header for size inspection only; no downloaded image or avatar.
  const bytes = new Uint8Array(24), view = new DataView(bytes.buffer);
  view.setUint32(0, 0x89504e47); view.setUint32(4, 0x0d0a1a0a); view.setUint32(8, 13); view.setUint32(12, 0x49484452); view.setUint32(16, width); view.setUint32(20, height); return bytes;
}
function imageSource(width: number, height: number, count = 1): [Document, Uint8Array] {
  const json = source(), bytes = png(width, height);
  json.buffers = [{ byteLength: bytes.length }]; json.bufferViews = [{ buffer: 0, byteOffset: 0, byteLength: bytes.length }];
  json.images = Array.from({ length: count }, () => ({ bufferView: 0, mimeType: 'image/png' })); return [json, bytes];
}

describe('VRM inspection before browser loading', () => {
  it('reports required bones, absent fingers, and explicit omitted-license defaults', () => {
    const info = inspectVRM(glb(source()));
    expect(info.required).toBe(15); expect(info.missing).toEqual([]); expect(info.fingers).toEqual({ left: 0, right: 0 });
    expect(info.metadataWarnings).toEqual([]);
    expect(info.terms).toContainEqual(['Credit', 'Not specified (VRM 1.0 default: required)']);
    expect(info.terms).toContainEqual(['Model redistribution', 'Not specified (VRM 1.0 default: Not allowed)']);
  });
  it('preserves supplied rights, references, additional text and incomplete metadata separately', () => {
    const json = source(); json.extensions.VRMC_vrm.meta = { name: '<script>untrusted</script>', version: 'test-v1', authors: [], references: ['https://example.invalid/original'], creditNotation: 'required', allowRedistribution: false, thirdPartyLicenses: 'Line one\nLine two', otherLicenseUrl: 'javascript:untrusted()' };
    const info = inspectVRM(glb(json));
    expect(info.name).toBe('<script>untrusted</script>'); expect(info.modelVersion).toBe('test-v1');
    expect(info.references).toEqual(['https://example.invalid/original']); expect(info.rawMeta).toEqual(json.extensions.VRMC_vrm.meta);
    expect(info.terms).toContainEqual(['Model redistribution', 'Not allowed']); expect(info.terms).toContainEqual(['Third-party terms', 'Line one\nLine two']);
    expect(info.metadataWarnings.length).toBe(2);
  });
  it('rejects legacy/unknown formats with an actionable conversion message', () => {
    const json = source(); json.extensions = { VRM: { specVersion: '0.0' } };
    expect(() => inspectVRM(glb(json))).toThrow(/0.x.*VRM 1.0/);
    json.extensions = {}; expect(() => inspectVRM(glb(json))).toThrow(/VRoid Studio.*UniVRM/);
  });
  it('flags malformed rights values without applying permissive fallback', () => {
    const json = source(); json.extensions.VRMC_vrm.meta.allowRedistribution = 'true'; json.extensions.VRMC_vrm.meta.avatarPermission = 'unrestricted';
    const info = inspectVRM(glb(json)); expect(info.metadataWarnings).toHaveLength(2);
    expect(info.terms).toContainEqual(['Avatar users', 'unrestricted']); expect(info.terms).toContainEqual(['Model redistribution', 'true']);
  });
  it.each(['https://example.invalid/private.png', './local.png', 'blob:foreign', 'file:///private.png'])('blocks resource URI %s without requesting anything', uri => {
    const fetch = vi.spyOn(globalThis, 'fetch'); const json = source(); json.images = [{ uri }];
    expect(() => inspectVRM(glb(json))).toThrow(/external files or URLs/); expect(fetch).not.toHaveBeenCalled(); fetch.mockRestore();
  });
  it('rejects malformed headers, oversized files, hierarchy cycles and invalid bone references', () => {
    expect(() => inspectVRM(new ArrayBuffer(12))).toThrow(/GLB/);
    expect(() => inspectVRM(new ArrayBuffer(MAX_VRM_BYTES + 1))).toThrow(/50 MiB/);
    const cases: Document[] = [];
    let json = source(); json.nodes[0].children = [0]; cases.push(json);
    json = source(); json.nodes[0].translation = [0, 1]; cases.push(json);
    json = source(); json.nodes[0].children = [-1]; cases.push(json);
    json = source(); json.extensions.VRMC_vrm.humanoid.humanBones.head.node = 0; cases.push(json);
    json = source(); delete json.extensions.VRMC_vrm.humanoid.humanBones.head; cases.push(json);
    json = source(); json.extensions.VRMC_vrm.humanoid.humanBones.head.node = 99; cases.push(json);
    for (const value of cases) expect(() => inspectVRM(glb(value))).toThrow(/VRM/);
  });
  it('admits bounded embedded PNG/JPEG dimensions without decoding or network', () => {
    const [json, bytes] = imageSource(2048, 2048, 4); expect(inspectVRM(glb(json, bytes)).version).toBe('1.0');
    const jpeg = new Uint8Array([0xff,0xd8,0xff,0xc2,0,8,8,4,0,8,0,1]);
    json.images = [{ uri: `data:image/jpeg;base64,${Buffer.from(jpeg).toString('base64')}` }];
    expect(inspectVRM(glb(json)).version).toBe('1.0');
  });
  it('rejects dimensions, aggregate pixels, image count and corrupt image ranges before decoding', () => {
    for (const [width, height, count] of [[4097, 1, 1], [0, 10, 1], [4096, 4096, 5], [8, 8, MAX_VRM_IMAGES + 1]]) {
      const [json, bytes] = imageSource(width, height, count); expect(() => inspectVRM(glb(json, bytes))).toThrow(/VRM (?:image|models)/);
    }
    const [json, bytes] = imageSource(1024, 1024); json.bufferViews[0].byteLength = 1000;
    expect(() => inspectVRM(glb(json, bytes))).toThrow(/range/);
    json.bufferViews[0].byteLength = 24; bytes[0] = 0;
    expect(() => inspectVRM(glb(json, bytes))).toThrow(/PNG or JPEG/);
  });
});
