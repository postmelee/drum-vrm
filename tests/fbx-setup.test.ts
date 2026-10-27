import { describe, expect, it, vi } from 'vitest';
import { Bone, Group } from 'three';
import { availableFBXBones, fbxBones, matchingFBXBones, suggestFBXBones } from '../src/fbx-rig';
import { savedFBXSetup, rememberFBXSetup, suggestFBXTexture } from '../src/fbx-preferences';
import { characterFiles } from '../src/character-files';
import { loadLocalFBX } from '../src/fbx-import';
import { encodeFBX, fbxDocument } from './fixtures/fbx';

describe('remaining bone dropdown choices', () => {
  it('excludes used and incompatible ancestors/descendants; releasing a selection returns it', () => {
    const root = new Group(), hips = new Bone(), neck = new Bone(), head = new Bone(), alt = new Bone(), leg = new Bone();
    hips.name = 'Hips'; neck.name = 'Neck'; head.name = 'Head'; alt.name = '頭'; leg.name = 'LeftUpperLeg';
    root.add(hips); hips.add(neck, leg); neck.add(head, alt);
    const bones = fbxBones(root), path = (node: Bone) => bones.find(b => b.node === node)!.path;
    const mapping = { hips: path(hips), neck: path(neck) };
    expect(matchingFBXBones(bones, 'head').map(b => b.name)).toEqual(['Head', '頭']);
    expect(suggestFBXBones(bones).head).toBeUndefined();
    expect(availableFBXBones(bones, mapping, 'head').map(b => b.node)).toEqual([head, alt]);
    expect(availableFBXBones(bones, {...mapping, head:path(head)}, 'neck').map(b => b.node)).toEqual([neck]);
    expect(availableFBXBones(bones, {...mapping, leftEye:path(alt)}, 'rightEye').map(b => b.node)).not.toContain(alt);
    expect(availableFBXBones(bones, mapping, 'rightEye').map(b => b.node)).toContain(alt);
    // Keep an invalid current choice visible, so it can be cleared/repaired.
    expect(availableFBXBones(bones, {...mapping, head:path(leg)}, 'head').map(b => b.node)).toContain(leg);
  });
});

describe('conservative image suggestions', () => {
  it('uses generic names, ignores shadow/normal maps and does not choose ties or numeric-only matches', () => {
    expect(suggestFBXTexture('mtl_body_12', ['tex_body_12_diff.png','tex_body_12_shad_c.png','tex_body_12_normal.png'])).toBe('tex_body_12_diff.png');
    expect(suggestFBXTexture('face.002', ['tex_chr77_face.png','tex_chr77_eye.png'])).toBe('tex_chr77_face.png');
    expect(suggestFBXTexture('face', ['first_face.png','other_face.png'])).toBeUndefined();
    expect(suggestFBXTexture('12', ['body_12.png'])).toBeUndefined();
    expect(suggestFBXTexture('brow', ['face.png'])).toBeUndefined();
  });
});

it('remembers only validated settings for exact hashes, bounds history and tolerates unavailable storage', async () => {
  const data = new Map<string,string>();
  vi.stubGlobal('localStorage', { getItem:(k:string)=>data.get(k)??null, setItem:(k:string,v:string)=>data.set(k,v) });
  const loaded = await loadLocalFBX(new File([encodeFBX(fbxDocument())], 'authored.fbx'));
  const config = { bones:suggestFBXBones(fbxBones(loaded.scene)), metersPerUnit:.01, textures:{}, materialMode:'unlit' as const };
  try {
    for (let i=0;i<12;i++) rememberFBXSetup(i.toString(16).padStart(64,'0'),config);
    expect(savedFBXSetup('0'.repeat(64))).toBeUndefined();
    expect(savedFBXSetup('b'.padStart(64,'0'))).toEqual(config);
    expect(JSON.parse([...data.values()][0])).toHaveProperty('b'.padStart(64,'0'));
    expect(Object.keys(JSON.parse([...data.values()][0]))).toHaveLength(10);
    vi.stubGlobal('localStorage', {getItem:()=>{throw Error('blocked');},setItem:()=>{throw Error('blocked');}});
    expect(savedFBXSetup('a'.repeat(64))).toBeUndefined();
    expect(()=>rememberFBXSetup('a'.repeat(64),config)).not.toThrow();
  } finally { loaded.dispose(); vi.unstubAllGlobals(); }
});

it('collects folder models and companion images while ignoring unrelated package files', () => {
  const files=['first.fbx','second.VRM','body.PNG','license.txt','original.blend','preview.webp'].map(name=>new File([],name));
  const result=characterFiles(files);expect(result.models.map(f=>f.name)).toEqual(['first.fbx','second.VRM']);expect(result.images.map(f=>f.name)).toEqual(['body.PNG']);
});
