import { describe, expect, it, vi } from 'vitest';
import { deflateSync } from 'node:zlib';
import { Bone, Group, Vector3 } from 'three';
import { inspectFBX, loadLocalFBX, validateFBXCompression, MAX_FBX_BYTES } from '../src/fbx-import';
import { FBXHumanoidRigAdapter, fbxBones, resolveFBXBones, suggestFBXBones } from '../src/fbx-rig';
import { validateFBXConfig, FBX_REQUIRED } from '../src/fbx-config';
import { createRestRigPose } from '../src/rig-humanoid';
import { encodeFBX, fbxDocument } from './fixtures/fbx';

describe('bounded local FBX import', () => {
  it.each([7400, 7500] as const)('loads authored binary %s with nonidentity rest axes, units and consistent contacts', async version => {
    for (const units of [1, 100]) for (const facing of ['0.0', '1.0'] as const) {
      const file = new File([encodeFBX(fbxDocument(units, facing), version)], 'authored.fbx'), loaded = await loadLocalFBX(file);
      const bones = suggestFBXBones(fbxBones(loaded.scene));
      expect(FBX_REQUIRED.filter(name => !bones[name as keyof typeof bones])).toEqual([]);
      expect(loaded.metersPerUnit).toBeCloseTo(1 / units);
      const config = validateFBXConfig({ bones, metersPerUnit: loaded.metersPerUnit, textures: {} });
      const rig = new FBXHumanoidRigAdapter(loaded.scene, config, 'fbx:authored', loaded.release);
      const profile = rig.getProfile();
      expect(profile.lengths[0][0]).toBeCloseTo(.25, 6); expect(profile.lengths[0][1]).toBeCloseTo(.22, 6);
      expect(profile.roots[0][0]).toBeLessThan(0); expect(profile.roots[1][0]).toBeGreaterThan(0);
      const rest = createRestRigPose(profile), moved = rest.slice(); for (let i = 0; i < 57; i += 3) moved[i] += .05;
      const matrices = () => Object.values(rig.humanoid.rawHumanBones).flatMap(b => b.node.matrixWorld.elements);
      rig.applyPose(rest); const expected = matrices();
      for (const pose of [moved, rest, moved, moved, rest]) {
        rig.applyPose(pose);
        expect(Math.max(...rig.getContactSamples().map(s => s.errorMeters))).toBeLessThan(1e-6);
        if (pose === rest) expect(matrices()).toEqual(expected);
      }
      const foot = rig.humanoid.getRawBoneNode('leftFoot')!, toes = rig.humanoid.getRawBoneNode('leftToes')!;
      expect(toes.getWorldPosition(new Vector3()).z).toBeGreaterThan(foot.getWorldPosition(new Vector3()).z);
      rig.dispose();
    }
  });
  it('rejects oversized/unsupported/truncated records and cyclic model parenting before loading', () => {
    expect(() => inspectFBX(new ArrayBuffer(MAX_FBX_BYTES + 1))).toThrow();
    expect(() => inspectFBX(new TextEncoder().encode('; ASCII FBX').buffer)).toThrow();
    const good = encodeFBX(fbxDocument()); expect(() => inspectFBX(good.slice(0, 60))).toThrow();
    const bad = good.slice(0); new DataView(bad).setUint32(23, 7600, true); expect(() => inspectFBX(bad)).toThrow();
    const doc = fbxDocument(); doc[2].children!.push({ name: 'C', props: ['OO', 1n, 2n] });
    expect(() => inspectFBX(encodeFBX(doc))).toThrow();
  });
  it('checks actual compressed size as well as declared array budgets', async () => {
    const compressed = deflateSync(new Uint8Array(1024));
    const array = new Uint8Array(13 + compressed.length), view = new DataView(array.buffer);
    array[0] = 'b'.charCodeAt(0); view.setUint32(1, 1024, true); view.setUint32(5, 1, true); view.setUint32(9, compressed.length, true); array.set(compressed, 13);
    const bytes = () => encodeFBX([{ name: 'Array', props: [array] }]);
    await expect(validateFBXCompression(bytes())).resolves.toBeUndefined();
    view.setUint32(1, 1, true); await expect(validateFBXCompression(bytes())).rejects.toThrow(/exceeds/);
    view.setUint32(1, 5000000, true); expect(() => inspectFBX(bytes())).toThrow();
  });
  it('never fetches missing external texture references and rejects ambiguous companions', async () => {
    const doc = fbxDocument(), objects = doc[1].children!, connections = doc[2].children!;
    objects.push({ name: 'Video', props: [900n, 'Image\0\x01Video', 'Clip'], children: [{ name: 'RelativeFilename', props: ['https://example.invalid/image.png'] }] },
      { name: 'Texture', props: [901n, 'Image\0\x01Texture', ''], children: [{ name: 'FileName', props: ['image.png'] }] });
    connections.push({ name: 'C', props: ['OO', 900n, 901n] });
    const fetchSpy = vi.spyOn(globalThis, 'fetch');
    try {
      await expect(loadLocalFBX(new File([encodeFBX(doc)], 'authored.fbx'))).rejects.toThrow(/companion/);
      expect(fetchSpy).not.toHaveBeenCalled();
      await expect(loadLocalFBX(new File([encodeFBX(fbxDocument())], 'authored.fbx'), [new File(['x'], 'a.png'), new File(['y'], 'A.PNG')])).rejects.toThrow(/distinct/);
    } finally { fetchSpy.mockRestore(); }
  });
});

describe('explicit semantic mapping', () => {
  it('suggests generic, Mixamo and Japanese aliases without guessing duplicate names', () => {
    const root = new Group();
    for (const name of ['mixamorig:Hips', 'Arm_L', 'Elbow_R', '手首.L', '親指０.R', 'Head', '頭']) { const b = new Bone(); b.name = name; root.add(b); }
    const map = suggestFBXBones(fbxBones(root));
    expect(map.hips).toBeDefined(); expect(map.leftUpperArm).toBeDefined(); expect(map.rightLowerArm).toBeDefined(); expect(map.leftHand).toBeDefined(); expect(map.rightThumbMetacarpal).toBeDefined(); expect(map.head).toBeUndefined();
  });
  it('rejects duplicate, missing and cross-limb hierarchy assignments before adoption', async () => {
    const loaded = await loadLocalFBX(new File([encodeFBX(fbxDocument())], 'authored.fbx'));
    const valid = { bones: suggestFBXBones(fbxBones(loaded.scene)), metersPerUnit: .01, textures: {} };
    expect(() => resolveFBXBones(loaded.scene, valid)).not.toThrow();
    for (const mutate of [(c: typeof valid) => { delete c.bones.hips; }, (c: typeof valid) => { c.bones.leftHand = c.bones.rightHand; }, (c: typeof valid) => { [c.bones.leftHand, c.bones.rightHand] = [c.bones.rightHand, c.bones.leftHand]; }, (c: typeof valid) => { c.metersPerUnit = NaN; }]) {
      const config = structuredClone(valid); mutate(config); expect(() => resolveFBXBones(loaded.scene, config)).toThrow();
    }
    loaded.dispose();
  });
});

it('checks companion image dimensions and the aggregate decode budget before image allocation', async()=>{
  const png=(w:number,h:number,name:string)=>{
    const b=new ArrayBuffer(24),v=new DataView(b);v.setUint32(0,0x89504e47);v.setUint32(4,0x0d0a1a0a);v.setUint32(12,0x49484452);v.setUint32(16,w);v.setUint32(20,h);return new File([b],name);
  };
  const file=new File([encodeFBX(fbxDocument())],'authored.fbx');
  await expect(loadLocalFBX(file,[png(4097,1,'large.png')])).rejects.toThrow(/4096/);
  await expect(loadLocalFBX(file,Array.from({length:5},(_,i)=>png(4096,4096,`${i}.png`)))).rejects.toThrow(/64M/);
});
