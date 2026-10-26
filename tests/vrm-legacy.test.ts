import { describe, expect, it } from 'vitest';
import { Quaternion, Vector3 } from 'three';
import { inspectVRM, loadLocalVRM, MAX_VRM_BYTES } from '../src/vrm-import';
import { VRMHumanoidRigAdapter } from '../src/rig-vrm';
import { createRestRigPose } from '../src/rig-humanoid';
import { encodeGLB, humanoidDocument } from './fixtures/vrm';

const source = () => humanoidDocument('0.0') as Record<string, any>;
const inspect = (doc: unknown) => inspectVRM(encodeGLB(doc));

describe('VRM 0.0 preflight and original terms', () => {
  it('maps the legacy thumb chain and preserves original metadata without v1 defaults', () => {
    const doc = source(), meta = doc.extensions.VRM.meta;
    Object.assign(meta, { version: 'v-test', reference: 'One\nTwo', contactInformation: 'Contact text', allowedUserName: 'ExplicitlyLicensedPerson',
      violentUssageName: 'Disallow', sexualUssageName: 'Allow', commercialUssageName: 'Disallow', otherPermissionUrl: 'javascript:untrusted()',
      licenseName: 'Other', otherLicenseUrl: 'https://example.invalid/license' });
    const info = inspect(doc);
    expect(info.version).toBe('0.0'); expect(info.name).toBe(meta.title); expect(info.authors).toEqual([meta.author]);
    expect(info.required).toBe(15); expect(info.fingers).toEqual({ left: 15, right: 15 }); expect(info.missing).toEqual([]);
    expect(info.references).toEqual(['One\nTwo']); expect(info.modelVersion).toBe('v-test'); expect(info.rawMeta).toEqual(meta);
    expect(info.metadataWarnings).toEqual([]);
    expect(info.terms).toContainEqual(['Additional usage terms', 'javascript:untrusted()']);
    expect(info.terms).toContainEqual(['Commercial use', 'Disallow']);
    expect(info.terms).toContainEqual(['Redistribution / modification license', 'Other']);
    expect(info.terms.some(([, value]) => value.includes('VRM 1.0'))).toBe(false);
  });
  it('reports missing/invalid legacy terms without inventing permissions', () => {
    const doc = source(); doc.extensions.VRM.meta = { licenseName: 'Other', commercialUssageName: 'true' };
    const info = inspect(doc);
    expect(info.metadataWarnings).toHaveLength(4);
    expect(info.terms).toContainEqual(['Avatar users', 'Not specified']);
    expect(info.terms).toContainEqual(['Commercial use', 'true']);
    doc.extensions.VRM.meta = {}; expect(inspect(doc).metadataWarnings).toHaveLength(3);
  });
  it('rejects duplicated semantic names, node references, invalid entries and malformed arrays', () => {
    const mutations = [
      (doc: any) => doc.extensions.VRM.humanoid.humanBones.push({ bone: 'hips', node: doc.nodes.length - 1 }),
      (doc: any) => doc.extensions.VRM.humanoid.humanBones.push({ bone: 'leftThumbMetacarpal', node: 1 }),
      (doc: any) => doc.extensions.VRM.humanoid.humanBones[1].node = 0,
      (doc: any) => doc.extensions.VRM.humanoid.humanBones[0].node = -1,
      (doc: any) => doc.extensions.VRM.humanoid.humanBones[0].node = 9999,
      (doc: any) => doc.extensions.VRM.humanoid.humanBones[0].node = .5,
      (doc: any) => doc.extensions.VRM.humanoid.humanBones.push(null),
      (doc: any) => doc.extensions.VRM.humanoid.humanBones = {},
      (doc: any) => doc.nodes[0].children.push(0),
      (doc: any) => doc.nodes[0].rotation = [0, 1],
      (doc: any) => doc.images = [{ uri: 'https://example.invalid/image.png' }],
    ];
    for (const mutate of mutations) { const doc = source(); mutate(doc); expect(() => inspect(doc)).toThrow(); }
  });
  it('keeps the same file, image dimension, aggregate pixel and image count budgets', () => {
    expect(() => inspectVRM(new ArrayBuffer(MAX_VRM_BYTES + 1))).toThrow(/50 MiB/);
    for (const [width, height, count, valid] of [[1024, 1024, 1, true], [4097, 1, 1, false], [4096, 4096, 5, false], [8, 8, 65, false]] as const) {
      const header = new Uint8Array(24), view = new DataView(header.buffer);
      view.setUint32(0, 0x89504e47); view.setUint32(4, 0x0d0a1a0a); view.setUint32(12, 0x49484452);
      view.setUint32(16, width); view.setUint32(20, height);
      const doc = source(); doc.images = Array.from({ length: count }, () => ({ uri: `data:image/png;base64,${Buffer.from(header).toString('base64')}` }));
      if (valid) expect(inspect(doc).version).toBe('0.0'); else expect(() => inspect(doc)).toThrow(/VRM (?:image|models)/);
    }
  });
});

describe('real GLTF/VRM loader to shared adapter', () => {
  it.each([[true, .86], [true, 1.12], [false, 1]] as const)('matches v1 with legacy fingers=%s, proportion=%s and repeatable actual contacts', async (fingers, proportion) => {
    const rigs: VRMHumanoidRigAdapter[] = [];
    for (const version of ['0.0', '1.0'] as const) {
      const file = new File([encodeGLB(humanoidDocument(version, fingers, proportion))], `authored-${version}.vrm`);
      const { vrm, info } = await loadLocalVRM(file);
      expect(vrm.meta.metaVersion).toBe(version === '0.0' ? '0' : '1'); expect(info.version).toBe(version);
      const rig = new VRMHumanoidRigAdapter(vrm, `authored-${version}`); rigs.push(rig);
      const profile = rig.getProfile(), rest = createRestRigPose(profile), changed = rest.slice();
      expect(profile.roots[0][0]).toBeLessThan(0); expect(profile.roots[1][0]).toBeGreaterThan(0);
      const q = new Quaternion().setFromAxisAngle(new Vector3(.3, .8, .2).normalize(), -.75);
      changed.set(q.toArray(), 57);
      const offset = new Vector3(...profile.gripOffsets[0]).add(new Vector3(...profile.tipOffsets[0])).applyQuaternion(q);
      changed.set(new Vector3().fromArray(changed, 6).add(offset).toArray(), 9);
      const matrices = () => Object.values(vrm.humanoid.rawHumanBones).flatMap(bone => bone.node.matrixWorld.elements);
      rig.applyPose(rest); const expected = matrices();
      for (const pose of [changed, rest, changed, changed, rest]) {
        rig.applyPose(pose);
        expect(Math.max(...rig.getContactSamples().map(s => s.errorMeters))).toBeLessThan(1e-6);
        if (pose === rest) expect(matrices()).toEqual(expected);
        rig.root.traverse(node => expect(node.matrixWorld.determinant()).toBeGreaterThan(0));
      }
      expect(rig.root.getObjectByName('leftHandActualStickTip')!.parent!.parent).toBe(vrm.humanoid.getRawBoneNode('leftHand'));
      expect(vrm.humanoid.getRawBoneNode('leftToes')!.getWorldPosition(new Vector3()).z).toBeGreaterThan(vrm.humanoid.getRawBoneNode('leftFoot')!.getWorldPosition(new Vector3()).z);
    }
    const numeric = (rig: VRMHumanoidRigAdapter) => {
      const p = rig.getProfile(); return [...p.roots.flat(), ...p.lengths.flat(), ...p.gripOffsets.flat(), ...p.tipOffsets.flat(), ...p.contactRotations.flat(), ...rig.getPlayerPosition()];
    };
    const a = numeric(rigs[0]), b = numeric(rigs[1]); expect(Math.max(...a.map((n, i) => Math.abs(n - b[i])))).toBeLessThan(1e-6);
    rigs.forEach(rig => rig.dispose());
  });
});
