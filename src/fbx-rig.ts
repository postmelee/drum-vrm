import * as THREE from 'three';
import { VRMHumanoid, VRMHumanBoneParentMap, VRMFirstPerson, type VRMHumanBoneName, type VRMHumanBones } from '@pixiv/three-vrm';
import { NormalizedHumanoidRigAdapter } from './rig-humanoid';
import { disposeModelSource } from './model-procedural';
import { FBX_BONES, validateFBXConfig, type FBXConfig } from './fbx-config';
import { msg } from './i18n';

export interface FBXBone { path: string; name: string; node: THREE.Bone; }
export function fbxBones(root: THREE.Object3D): FBXBone[] {
  const result: FBXBone[] = [];
  const visit = (node: THREE.Object3D, path: string) => {
    if (node instanceof THREE.Bone) result.push({ path, name: node.name, node });
    node.children.forEach((child, index) => visit(child, path ? `${path}/${index}` : String(index)));
  };
  // Wrap even a single-bone source so its path has a stable numeric identity.
  visit(root, '0'); return result;
}
const canonical = (name: string) => name.normalize('NFKC').replace(/^.*:/, '').replace(/^mixamorig/i, '').replace(/[\s_.-]/g, '').toLowerCase();
/** Conventional semantic names only. No character/model-specific branches. */
export function matchingFBXBones(candidates: FBXBone[], name: VRMHumanBoneName): FBXBone[] {
  const aliases: Partial<Record<VRMHumanBoneName, string[]>> = {
    hips: ['hips', 'hip', 'pelvis', '腰'], spine: ['spine', '上半身'], chest: ['chest', 'spine1', '上半身1'],
    upperChest: ['upperchest', 'spine2', '上半身2'], neck: ['neck', '首'], head: ['head', '頭'],
  };
  for (const side of ['left', 'right'] as const) {
    const short = side === 'left' ? 'l' : 'r', jp = side === 'left' ? '左' : '右';
    for (const [part, names] of Object.entries({ Shoulder: ['shoulder', 'clavicle', '肩'], UpperArm: ['upperarm', 'arm', '腕'], LowerArm: ['lowerarm', 'forearm', 'elbow', 'ひじ'], Hand: ['hand', 'wrist', '手首'], UpperLeg: ['upperleg', 'upleg', 'thigh', '足'], LowerLeg: ['lowerleg', 'leg', 'calf', 'knee', 'ひざ'], Foot: ['foot', 'ankle', '足首'], Toes: ['toes', 'toe', 'toebase', 'つま先', '足先ex'] })) {
      aliases[`${side}${part}` as VRMHumanBoneName] = [`${side}${part}`, ...names.flatMap(n => [`${side}${n}`, `${n}${short}`, `${short}${n}`, `${jp}${n}`])];
    }
    for (const [finger, names] of Object.entries({ Thumb: ['thumb', '親指'], Index: ['index', '人指', '人差指'], Middle: ['middle', '中指'], Ring: ['ring', '薬指'], Little: ['little', 'pinky', '小指'] })) {
      const parts = finger === 'Thumb' ? ['Metacarpal', 'Proximal', 'Distal'] : ['Proximal', 'Intermediate', 'Distal'];
      parts.forEach((part, i) => {
        const n = i + 1;
        aliases[`${side}${finger}${part}` as VRMHumanBoneName] = [`${side}${finger}${part}`, ...names.flatMap(f => /[^a-z]/i.test(f) ? [`${f}${finger === 'Thumb' ? i : n}${short}`, `${jp}${f}${finger === 'Thumb' ? i : n}`] : [`${side}hand${f}${n}`, `${f}0${n}${short}`, `${f}${n}${short}`])];
      });
    }
  }
  const names = new Set((aliases[name] ?? [name]).map(canonical));
  return candidates.filter(bone => names.has(canonical(bone.name)));
}
export function suggestFBXBones(candidates: FBXBone[]): FBXConfig['bones'] {
  const result: FBXConfig['bones'] = {};
  for (const name of FBX_BONES) {
    const matches = matchingFBXBones(candidates, name);
    if (matches.length === 1) result[name] = matches[0].path;
  }
  return result;
}
/** Keep current selection editable; other choices must be unused and fit mapped ancestry. */
export function availableFBXBones(candidates: FBXBone[], mapping: FBXConfig['bones'], name: VRMHumanBoneName): FBXBone[] {
  const ancestor = (child: VRMHumanBoneName, parent: VRMHumanBoneName) => {
    let cursor = VRMHumanBoneParentMap[child];
    while (cursor) { if (cursor === parent) return true; cursor = VRMHumanBoneParentMap[cursor]; }
    return false;
  };
  const descends = (child: THREE.Object3D, parent: THREE.Object3D) => {
    let cursor = child.parent;
    while (cursor) { if (cursor === parent) return true; cursor = cursor.parent; }
    return false;
  };
  const mapped = Object.entries(mapping).filter(([other]) => other !== name)
    .map(([other, path]) => ({ name: other as VRMHumanBoneName, bone: candidates.find(b => b.path === path) }));
  return candidates.filter(candidate => candidate.path === mapping[name] || mapped.every(({ name: other, bone }) =>
    !bone || (bone.path !== candidate.path && (!ancestor(name, other) || descends(candidate.node, bone.node)) &&
      (!ancestor(other, name) || descends(bone.node, candidate.node)))));
}
export function resolveFBXBones(root: THREE.Object3D, input: FBXConfig): VRMHumanBones {
  const config = validateFBXConfig(input), candidates = new Map(fbxBones(root).map(b => [b.path, b.node]));
  const bones = {} as VRMHumanBones;
  for (const [name, path] of Object.entries(config.bones)) {
    const node = candidates.get(path!);
    if (!node) throw new Error(msg('FBX bone no longer exists: {0}', 'FBX 본이 존재하지 않습니다: {0}', name));
    bones[name as VRMHumanBoneName] = { node };
  }
  for (const name of Object.keys(bones) as VRMHumanBoneName[]) {
    let parent = VRMHumanBoneParentMap[name];
    while (parent && !bones[parent]) parent = VRMHumanBoneParentMap[parent];
    if (!parent) continue;
    let current = bones[name]!.node.parent;
    while (current && current !== bones[parent]!.node) current = current.parent;
    if (!current) throw new Error(msg('FBX bone {0} must descend from {1}. Check the mapping.', 'FBX 본 {0}은(는) {1} 아래에 있어야 합니다. 연결을 확인하세요.', name, parent));
  }
  return bones;
}
export function disposeFBX(root: THREE.Object3D) {
  const skeletons = new Set<THREE.Skeleton>(); root.traverse(node => { if (node instanceof THREE.SkinnedMesh) skeletons.add(node.skeleton); });
  skeletons.forEach(skeleton => skeleton.dispose()); disposeModelSource(root);
}
export class FBXHumanoidRigAdapter extends NormalizedHumanoidRigAdapter {
  constructor(model: THREE.Group, config: FBXConfig, id: string, release?: () => void) {
    const bones = resolveFBXBones(model, config);
    model.updateMatrixWorld(true);
    const point = (name: VRMHumanBoneName) => bones[name]!.node.getWorldPosition(new THREE.Vector3());
    const y = point('head').sub(point('hips')).normalize();
    const x = point('leftUpperArm').sub(point('rightUpperArm')); x.addScaledVector(y, -x.dot(y)).normalize();
    if (x.lengthSq() < .9 || y.lengthSq() < .9) throw new Error(msg('FBX rest pose has no usable body axes.', 'FBX 기본 자세에서 몸의 방향을 판단할 수 없습니다.'));
    const basis = new THREE.Group(); basis.name = 'FBXMeterBasis'; basis.scale.setScalar(config.metersPerUnit);
    basis.quaternion.setFromRotationMatrix(new THREE.Matrix4().makeBasis(x, y, x.clone().cross(y))).invert();
    basis.add(model);
    const source = new THREE.Group(); source.add(basis); source.updateMatrixWorld(true);
    const humanoid = new VRMHumanoid(bones); source.add(humanoid.normalizedHumanBonesRoot);
    super(humanoid, source, id, { disposeSource: () => { disposeFBX(source); release?.(); } });
    const annotations: { meshes: THREE.Mesh[]; type: 'auto' }[] = [];
    model.traverse(node => { if (node instanceof THREE.Mesh) annotations.push({ meshes: [node], type: 'auto' }); });
    new VRMFirstPerson(humanoid, annotations).setup(); this.refreshModelVisibility();
  }
}
