import { afterEach, expect, it, vi } from 'vitest';
import { ClampToEdgeWrapping, Texture, TextureLoader } from 'three';
import { MToonMaterial, VRMUtils } from '@pixiv/three-vrm';
import { applyLegacyShadeCompatibility } from '../src/vrm-materials';
import { loadLocalVRM } from '../src/vrm-import';
import { encodeGLB, humanoidDocument } from './fixtures/vrm';

afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); });

function document(version: '0.0' | '1.0' = '0.0'): Record<string, any> {
  const doc: Record<string, any> = humanoidDocument(version);
  // Independently authored PNG header; image decode is mocked only in Node.
  const png=new Uint8Array(24), view=new DataView(png.buffer);
  view.setUint32(0,0x89504e47); view.setUint32(4,0x0d0a1a0a); view.setUint32(12,0x49484452); view.setUint32(16,1); view.setUint32(20,1);
  doc.images=[{uri:`data:image/png;base64,${Buffer.from(png).toString('base64')}`}];
  doc.samplers=[{wrapS:33071,wrapT:33071}]; doc.textures=[{source:0},{source:0,sampler:0}];
  doc.materials=Array.from({length:4},(_,i)=>({name:`authored-${i}`}));
  if(version==='0.0') doc.extensions.VRM.materialProperties=[
    {shader:'VRM/MToon',textureProperties:{_MainTex:0},vectorProperties:{_ShadeColor:[1,1,1,1]}},
    {shader:'VRM/MToon',textureProperties:{_MainTex:0,_ShadeTexture:1}},
    {shader:'VRM/MToon',textureProperties:{_ShadeTexture:0}},
    {shader:'VRM/UnlitTexture',textureProperties:{_MainTex:0}},
  ];
  if(version==='1.0') { doc.extensionsUsed.push('VRMC_materials_mtoon'); doc.materials[0].pbrMetallicRoughness={baseColorTexture:{index:0}}; doc.materials[0].extensions={VRMC_materials_mtoon:{specVersion:'1.0',shadeColorFactor:[1,1,1]}}; }
  doc.accessors=[{componentType:5126,count:3,type:'VEC3',min:[0,0,0],max:[0,0,0]}];
  doc.meshes=[{primitives:doc.materials.map((_:unknown,material:number)=>({attributes:{POSITION:0},material}))}];
  doc.scenes[0].nodes.push(doc.nodes.length); doc.nodes.push({mesh:0});
  return doc;
}

it('limits migration to absent shade maps on legacy MToon, preserves explicit maps and is idempotent',()=>{
  const doc=document(), original=structuredClone(doc);
  expect(applyLegacyShadeCompatibility(doc)).toBe(1);
  original.extensions.VRM.materialProperties[0].textureProperties._ShadeTexture=0;
  expect(doc).toEqual(original); expect(applyLegacyShadeCompatibility(doc)).toBe(0);
  const modern=document('1.0'), before=structuredClone(modern);
  expect(applyLegacyShadeCompatibility(modern)).toBe(0); expect(modern).toEqual(before);
});

it.each([-1,2,0.5,null,'0'])('does not repair an invalid main texture index %s',main=>{
  const doc=document(); doc.extensions.VRM.materialProperties[0].textureProperties._MainTex=main;
  expect(applyLegacyShadeCompatibility(doc)).toBe(0);
  expect(doc.extensions.VRM.materialProperties[0].textureProperties._ShadeTexture).toBeUndefined();
});

it('real loader migrates before MToon conversion and leaves source bytes and VRM 1 unchanged',async()=>{
  vi.stubGlobal('self',globalThis);
  vi.spyOn(TextureLoader.prototype,'load').mockImplementation((_url,onLoad)=>{
    const texture=new Texture({width:1,height:1} as HTMLImageElement); queueMicrotask(()=>onLoad?.(texture)); return texture;
  });
  for(const version of ['0.0','1.0'] as const){
    const bytes=encodeGLB(document(version)), file=new File([bytes],`authored-${version}.vrm`);
    const loaded=await loadLocalVRM(file);
    expect(await file.arrayBuffer()).toEqual(bytes);
    expect(loaded.info.legacyShadeTextures).toBe(version==='0.0'?1:0);
    if(version==='0.0'){
      const material=loaded.vrm.materials!.find(m=>m.name==='authored-0') as MToonMaterial;
      expect(material.map).toBeTruthy(); expect(material.shadeMultiplyTexture).toBe(material.map);
      expect(material.shadeColorFactor.toArray()).toEqual([1,1,1]);
      const explicit=loaded.vrm.materials!.find(m=>m.name==='authored-1') as MToonMaterial;
      expect(explicit.shadeMultiplyTexture?.wrapS).toBe(ClampToEdgeWrapping); expect(explicit.map?.wrapS).not.toBe(ClampToEdgeWrapping);
      expect(explicit.shadeMultiplyTexture).toBeTruthy();
    } else { const material=loaded.vrm.materials!.find(m=>m.name==='authored-0') as MToonMaterial; expect(material.map).toBeTruthy(); expect(material.shadeMultiplyTexture).toBeNull(); }
    VRMUtils.deepDispose(loaded.vrm.scene);
  }
});
