import { describe, expect, it, vi } from 'vitest';
import * as THREE from 'three';
import { createFBXMaterials } from '../src/fbx-materials';
import { loadLocalFBX, inspectFBX } from '../src/fbx-import';
import { encodeFBX, fbxDocument, type FBXNode } from './fixtures/fbx';

function materialDocument(factors: (number | undefined)[]): FBXNode[] {
  const doc = fbxDocument(), objects = doc[1].children!, links = doc[2].children!;
  const array = (values: number[], kind: 'd' | 'i') => {
    const width = kind === 'd' ? 8 : 4, bytes = new Uint8Array(13 + width * values.length), view = new DataView(bytes.buffer);
    bytes[0] = kind.charCodeAt(0); view.setUint32(1, values.length, true); view.setUint32(9, width * values.length, true);
    values.forEach((v, i) => kind === 'd' ? view.setFloat64(13 + width * i, v, true) : view.setInt32(13 + width * i, v, true)); return bytes;
  };
  objects.push({ name:'Model', props:[5000n,'Triangle\0\x01Model','Mesh'] },
    {name:'Geometry',props:[6000n,'Triangle\0\x01Geometry','Mesh'],children:[
      {name:'Vertices',props:[array([0,0,0, 1,0,0, 0,1,0],'d')]},
      {name:'PolygonVertexIndex',props:[array([0,1,-3],'i')]},
    ]});
  links.push({name:'C',props:['OO',5000n,1n]},{name:'C',props:['OO',6000n,5000n]});
  factors.forEach((factor, i) => {
    const id=BigInt(7000+i);
    objects.push({name:'Material',props:[id,'SameName\0\x01Material',''],children:[
      {name:'ShadingModel',props:['phong']},
      {name:'Properties70',children:[
        {name:'P',props:['SpecularColor','Color','','A',.5,1,.25]},
        ...(factor===undefined?[]:[{name:'P',props:['SpecularFactor','Number','','A',factor]}]),
        {name:'P',props:['Shininess','Number','','A',20]},
      ]},
    ]});
    links.push({name:'C',props:['OO',id,5000n]});
  });
  return doc;
}

describe('FBX source specular factors', () => {
  it.each([7400,7500] as const)('applies binary %s factors once by ID even with duplicate names and shared mesh materials', async version => {
    const doc=materialDocument([.25,0,undefined]);
    // A second mesh shares the same material: multiplying twice would be wrong.
    doc[1].children!.push({name:'Model',props:[5001n,'Other\0\x01Model','Mesh']});
    doc[2].children!.push({name:'C',props:['OO',5001n,1n]},{name:'C',props:['OO',6000n,5001n]},{name:'C',props:['OO',7000n,5001n]});
    const loaded=await loadLocalFBX(new File([encodeFBX(doc,version)],'authored.fbx'));
    const color=new THREE.Color().setRGB(.5,1,.25,THREE.SRGBColorSpace);
    const mats=loaded.materials as THREE.MeshPhongMaterial[];
    expect(mats).toHaveLength(3); expect(mats[0].specular.toArray()).toEqual(color.clone().multiplyScalar(.25).toArray());
    expect(mats[1].specular.toArray()).toEqual([0,0,0]); expect(mats[2].specular.toArray()).toEqual(color.toArray());
    for(const mode of ['unlit','matte','original','original'] as const) await loaded.applyTextures({bones:{},metersPerUnit:.01,textures:{},materialMode:mode});
    expect(mats[0].specular.toArray()).toEqual(color.multiplyScalar(.25).toArray());loaded.dispose();
  });
  it.each([-1,1.1,NaN,Infinity])('rejects invalid factor %s before loader allocation',factor=>{
    expect(()=>inspectFBX(encodeFBX(materialDocument([factor])))).toThrow();
  });
});

it('switches reversible shared variants while preserving textures, alpha and sidedness; releases inactive materials',()=>{
  const root=new THREE.Group(), source=new THREE.MeshPhongMaterial({color:0xabcdef,opacity:.6,transparent:true,alphaTest:.4,side:THREE.DoubleSide,vertexColors:true});
  const texture=new THREE.Texture(), alpha=new THREE.Texture();source.map=texture;source.alphaMap=alpha;source.envMap=new THREE.Texture();
  const a: THREE.Mesh<THREE.BufferGeometry, THREE.Material> = new THREE.Mesh(new THREE.BufferGeometry(),source), b=new THREE.Mesh(new THREE.BufferGeometry(),[source as THREE.Material]);root.add(a,b);
  const modes=createFBXMaterials(root,{factors:new Map(),bindings:new Map()});
  modes.apply('unlit');const unlit=a.material as THREE.MeshBasicMaterial;
  expect(unlit).toBeInstanceOf(THREE.MeshBasicMaterial);expect(b.material[0]).toBe(unlit);
  expect(unlit.map).toBe(texture);expect(unlit.alphaMap).toBe(alpha);expect(unlit.color.equals(source.color)).toBe(true);
  expect([unlit.opacity,unlit.alphaTest,unlit.side,unlit.vertexColors,unlit.toneMapped,unlit.envMap]).toEqual([.6,.4,THREE.DoubleSide,true,false,null]);
  const disposeUnlit=vi.spyOn(unlit,'dispose');
  modes.apply('matte');const matte=a.material as THREE.MeshPhongMaterial;
  expect(matte.specular.toArray()).toEqual([0,0,0]);expect(matte.envMap).toBeNull();expect(matte.toneMapped).toBe(true);
  const disposeMatte=vi.spyOn(matte,'dispose');
  source.map=null;source.color.set(0xffffff);modes.apply('unlit');expect(a.material).toBe(unlit);expect(unlit.map).toBeNull();expect(unlit.color.getHex()).toBe(0xffffff);
  modes.apply('original');expect(a.material).toBe(source);expect(b.material[0]).toBe(source);expect(source.specular.getHex()).not.toBe(0);
  modes.dispose();modes.dispose();expect(disposeUnlit).toHaveBeenCalledTimes(1);expect(disposeMatte).toHaveBeenCalledTimes(1);
});
