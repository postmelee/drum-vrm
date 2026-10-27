import { Bone, Group, Quaternion, Vector3 } from 'three';

/** Independently authored bone-only GLB. No third-party model or animation data. */
export function humanoidDocument(version: '0.0' | '1.0', withFingers = true, proportion = 1) {
  const root = new Group(), bones = new Map<string, Bone>();
  const facing = new Quaternion().setFromAxisAngle(new Vector3(0, 1, 0), version === '0.0' ? Math.PI : 0);
  const add = (name: string, parent: string | null, world: number[]) => {
    const node = new Bone(), ancestor = parent ? bones.get(parent)! : root;
    node.name = name; ancestor.add(node); ancestor.updateWorldMatrix(true, false);
    node.position.copy(ancestor.worldToLocal(new Vector3(...world).multiplyScalar(proportion).applyQuaternion(facing)));
    const rest = new Quaternion().setFromAxisAngle(new Vector3(1, .7, .3).normalize(), .13 + bones.size * .047).premultiply(facing);
    node.quaternion.copy(ancestor.getWorldQuaternion(new Quaternion()).invert().multiply(rest));
    node.updateWorldMatrix(false, true); bones.set(name, node);
  };
  add('hips', null, [0, .9, 0]); add('spine', 'hips', [0, 1.08, 0]);
  add('chest', 'spine', [0, 1.23, 0]); add('neck', 'chest', [0, 1.37, 0]); add('head', 'neck', [0, 1.48, .02]);
  for (const side of ['left', 'right']) {
    const sign = side === 'left' ? 1 : -1;
    add(`${side}UpperArm`, 'chest', [sign * .14, 1.27, 0]);
    add(`${side}LowerArm`, `${side}UpperArm`, [sign * .39, 1.27, 0]);
    add(`${side}Hand`, `${side}LowerArm`, [sign * .61, 1.27, 0]);
    if (withFingers) {
      for (const [i, finger] of ['Index', 'Middle', 'Ring', 'Little'].entries()) {
        add(`${side}${finger}Proximal`, `${side}Hand`, [sign * .665, 1.27, .022 - i * .014]);
        add(`${side}${finger}Intermediate`, `${side}${finger}Proximal`, [sign * .693, 1.27, .022 - i * .014]);
        add(`${side}${finger}Distal`, `${side}${finger}Intermediate`, [sign * .713, 1.27, .022 - i * .014]);
      }
      add(`${side}ThumbMetacarpal`, `${side}Hand`, [sign * .635, 1.265, .03]);
      add(`${side}ThumbProximal`, `${side}ThumbMetacarpal`, [sign * .660, 1.265, .044]);
      add(`${side}ThumbDistal`, `${side}ThumbProximal`, [sign * .680, 1.265, .044]);
    }
    add(`${side}UpperLeg`, 'hips', [sign * .09, .85, 0]);
    add(`${side}LowerLeg`, `${side}UpperLeg`, [sign * .09, .47, 0]);
    add(`${side}Foot`, `${side}LowerLeg`, [sign * .09, .1, 0]);
    add(`${side}Toes`, `${side}Foot`, [sign * .09, .07, .14]);
  }
  const entries = [...bones.entries()], indices = new Map(entries.map(([name], i) => [name, i]));
  const nodes = entries.map(([name, node]) => ({ name, translation: node.position.toArray(), rotation: node.quaternion.toArray(), children: node.children.map(child => indices.get(child.name)!) }));
  const v0Name = (name: string) => name.replace('ThumbProximal', 'ThumbIntermediate').replace('ThumbMetacarpal', 'ThumbProximal');
  const extension = version === '0.0' ? 'VRM' : 'VRMC_vrm';
  return { asset: { version: '2.0' }, scene: 0, scenes: [{ nodes: [0] }], nodes, extensionsUsed: [extension], extensions: {
    [extension]: { specVersion: version, humanoid: { humanBones: version === '0.0'
      ? entries.map(([name], node) => ({ bone: v0Name(name), node }))
      : Object.fromEntries(entries.map(([name], node) => [name, { node }])) },
    meta: version === '0.0' ? { title: 'Authored legacy fixture', author: 'Drum VRM tests', licenseName: 'Redistribution_Prohibited' }
      : { name: 'Authored current fixture', authors: ['Drum VRM tests'], licenseUrl: 'https://vrm.dev/licenses/1.0/' },
    },
  } };
}

export function encodeGLB(json: unknown): ArrayBuffer {
  const data = new TextEncoder().encode(JSON.stringify(json)), length = Math.ceil(data.length / 4) * 4;
  const bytes = new ArrayBuffer(20 + length), view = new DataView(bytes);
  view.setUint32(0, 0x46546c67, true); view.setUint32(4, 2, true); view.setUint32(8, bytes.byteLength, true);
  view.setUint32(12, length, true); view.setUint32(16, 0x4e4f534a, true);
  new Uint8Array(bytes, 20).fill(32); new Uint8Array(bytes, 20, data.length).set(data);
  return bytes;
}
