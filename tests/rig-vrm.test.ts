import { describe, expect, it } from 'vitest';
import { Bone, BufferGeometry, Float32BufferAttribute, Group, MeshBasicMaterial, PerspectiveCamera, Quaternion, Skeleton, SkinnedMesh, Uint16BufferAttribute, Vector3 } from 'three';
import { VRM, VRMFirstPerson, VRMHumanoid, VRMNodeConstraintManager, VRMRotationConstraint, type VRMHumanBoneName, type VRMHumanBones } from '@pixiv/three-vrm';
import { createRestRigPose } from '../src/rig-humanoid';
import { VRMHumanoidRigAdapter } from '../src/rig-vrm';

/** Authored bone-only fixture. No downloaded model, texture, skin, or motion. */
function generatedVRM(proportion = 1, withFingers = false) {
  const source = new Group(), bones: Partial<Record<VRMHumanBoneName, Bone>> = {};
  const add = (name: VRMHumanBoneName, parent: VRMHumanBoneName | null, world: [number, number, number]) => {
    const node = new Bone(), ancestor = parent ? bones[parent]! : source;
    node.name = name; ancestor.add(node); ancestor.updateWorldMatrix(true, false);
    node.position.copy(ancestor.worldToLocal(new Vector3(...world).multiplyScalar(proportion)));
    const index = Object.keys(bones).length;
    const worldQ = new Quaternion().setFromAxisAngle(new Vector3(1, .7, .3).normalize(), .13 + index * .047);
    node.quaternion.copy(ancestor.getWorldQuaternion(new Quaternion()).invert().multiply(worldQ));
    node.updateWorldMatrix(false, true); bones[name] = node;
  };
  add('hips', null, [0, .90, 0]); add('spine', 'hips', [0, 1.08, 0]);
  add('chest', 'spine', [0, 1.23, 0]); add('neck', 'chest', [0, 1.37, 0]); add('head', 'neck', [0, 1.48, 0]);
  for (const side of ['left', 'right'] as const) {
    const sign = side === 'left' ? 1 : -1;
    add(`${side}UpperArm`, 'chest', [sign * .14, 1.27, 0]);
    add(`${side}LowerArm`, `${side}UpperArm`, [sign * .39, 1.27, 0]);
    add(`${side}Hand`, `${side}LowerArm`, [sign * .61, 1.27, 0]);
    if (withFingers) {
      for (const [index, finger] of ['Index', 'Middle', 'Ring', 'Little'].entries()) {
        const prefix = `${side}${finger}`;
        add(`${prefix}Proximal` as VRMHumanBoneName, `${side}Hand`, [sign * .665, 1.27, .022 - index * .014]);
        add(`${prefix}Intermediate` as VRMHumanBoneName, `${prefix}Proximal` as VRMHumanBoneName, [sign * .693, 1.27, .022 - index * .014]);
        add(`${prefix}Distal` as VRMHumanBoneName, `${prefix}Intermediate` as VRMHumanBoneName, [sign * .713, 1.27, .022 - index * .014]);
      }
      add(`${side}ThumbMetacarpal`, `${side}Hand`, [sign * .635, 1.265, .030]);
      add(`${side}ThumbProximal`, `${side}ThumbMetacarpal`, [sign * .660, 1.265, .044]);
      add(`${side}ThumbDistal`, `${side}ThumbProximal`, [sign * .680, 1.265, .044]);
    }
    add(`${side}UpperLeg`, 'hips', [sign * .09, .85, 0]);
    add(`${side}LowerLeg`, `${side}UpperLeg`, [sign * .09, .47, 0]);
    add(`${side}Foot`, `${side}LowerLeg`, [sign * .09, .10, 0]);
    add(`${side}Toes`, `${side}Foot`, [sign * .09, .07, .14]);
  }
  source.updateMatrixWorld(true);
  const humanoid = new VRMHumanoid(Object.fromEntries(Object.entries(bones).map(([name, node]) => [name, {node}])) as unknown as VRMHumanBones);
  source.add(humanoid.normalizedHumanBonesRoot);
  const helper = new Bone(); helper.name = 'generatedConstraintHelper'; helper.position.set(.02, 0, 0); bones.leftLowerArm!.add(helper);
  const constraint = new VRMRotationConstraint(helper, bones.leftHand!); constraint.weight = .6;
  const manager = new VRMNodeConstraintManager(); manager.addConstraint(constraint); source.updateMatrixWorld(true); manager.setInitState();
  const vrm = new VRM({scene: source, humanoid, nodeConstraintManager: manager, meta: {
    metaVersion: '1', name: 'Authored regression fixture', authors: ['Drum VRM contributors'], licenseUrl: 'https://vrm.dev/licenses/1.0/',
  }});
  return {vrm, helper, bones};
}

function matrices(rig: VRMHumanoidRigAdapter): number[] {
  rig.root.updateMatrixWorld(true);
  return Object.values(rig.humanoid.rawHumanBones).flatMap(bone => bone.node.matrixWorld.elements);
}

describe('VRM normalized-to-raw rest basis and actual tool attachments', () => {
  it.each([.86, 1.12])('fits a diagonal grip from measured %s finger geometry with arbitrary raw rest bases', proportion => {
    const {vrm} = generatedVRM(proportion, true), rig = new VRMHumanoidRigAdapter(vrm, `generated-fingers-${proportion}`);
    const profile = rig.getProfile(), pose = createRestRigPose(profile);
    expect(Math.abs(profile.gripOffsets[0][0])).toBeGreaterThan(.03 * proportion);
    expect(profile.gripOffsets[0][0]).toBeCloseTo(-profile.gripOffsets[1][0], 6);
    for (const side of ['left', 'right'] as const) {
      const socket = rig.root.getObjectByName(`${side}HandToolSocket`)!;
      const proximal = socket.worldToLocal(vrm.humanoid.getRawBoneNode(`${side}IndexProximal`)!.getWorldPosition(new Vector3()));
      const distal = socket.worldToLocal(vrm.humanoid.getRawBoneNode(`${side}IndexDistal`)!.getWorldPosition(new Vector3()));
      const grip = new Vector3(...profile.gripOffsets[side === 'left' ? 0 : 1]); grip.x *= -1;
      proximal.sub(grip); distal.sub(grip);
      expect(proximal.x * distal.x + proximal.y * distal.y).toBeLessThan(0);
      expect(Math.hypot(distal.x, distal.y)).toBeGreaterThan(.005);
      expect(vrm.humanoid.getNormalizedBoneNode(`${side}ThumbMetacarpal`)!.quaternion.angleTo(new Quaternion())).toBeGreaterThan(.01);
    }
    const before = matrices(rig), calibration = rig.getCalibration(), moved = rig.setCalibration({seatHeight: .55, seatForward: .12});
    rig.applyPose(createRestRigPose(moved)); expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    rig.setCalibration(calibration); rig.applyPose(pose); expect(matrices(rig)).toEqual(before);
    expect(rig.getProfile().lengths).toEqual(profile.lengths); rig.dispose();
  });
  it.each([.86, 1.12])('preserves %s proportions, nonidentity raw rest bases and node constraints through seeking', proportion => {
    const {vrm, helper, bones} = generatedVRM(proportion);
    expect(bones.leftHand!.getWorldQuaternion(new Quaternion()).angleTo(new Quaternion())).toBeGreaterThan(.2);
    const rig = new VRMHumanoidRigAdapter(vrm, `generated-${proportion}`), scene = new Group(); scene.add(rig.root);
    const profile = rig.getProfile(), rest = createRestRigPose(profile), changed = rest.slice();
    expect(profile.lengths[0][0]).toBeCloseTo(.25 * proportion, 10);
    expect(profile.lengths[0][1]).toBeCloseTo(.22 * proportion, 10);
    const q = new Quaternion().setFromAxisAngle(new Vector3(.3, .8, .2).normalize(), -.75);
    changed.set(q.toArray(), 57);
    const offset = new Vector3(...profile.gripOffsets[0]).add(new Vector3(...profile.tipOffsets[0])).applyQuaternion(q);
    changed.set(new Vector3().fromArray(changed, 6).add(offset).toArray(), 9);
    rig.applyPose(rest); const initial = matrices(rig), initialHelper = helper.quaternion.clone();
    rig.applyPose(changed); const modified = matrices(rig);
    expect(helper.quaternion.angleTo(initialHelper)).toBeGreaterThan(.01);
    for (const pose of [rest, changed, changed, rest, changed]) {
      rig.applyPose(pose);
      const expected = pose === rest ? initial : modified;
      expect(Math.max(...matrices(rig).map((v, i) => Math.abs(v - expected[i])))).toBeLessThan(1e-10);
      expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
      expect(bones.leftUpperArm!.getWorldPosition(new Vector3()).distanceTo(bones.leftLowerArm!.getWorldPosition(new Vector3()))).toBeCloseTo(profile.lengths[0][0], 6);
      expect(bones.leftLowerArm!.getWorldPosition(new Vector3()).distanceTo(bones.leftHand!.getWorldPosition(new Vector3()))).toBeCloseTo(profile.lengths[0][1], 6);
      rig.root.traverse(node => expect(node.matrixWorld.determinant()).toBeGreaterThan(0));
    }
    expect(rig.root.getObjectByName('leftHandActualStickTip')!.parent!.parent).toBe(bones.leftHand);
    rig.dispose(); expect(rig.root.parent).toBeNull();
  });

  it('creates the first-person head-erased mesh and applies model visibility to both camera layers', () => {
    const {vrm: source, bones} = generatedVRM(), geometry = new BufferGeometry();
    // Two authored triangles: one fully head-weighted, the other hip-weighted.
    geometry.setAttribute('position', new Float32BufferAttribute([-.04, 1.45, 0, .04, 1.45, 0, 0, 1.53, 0, -.1, .9, 0, .1, .9, 0, 0, 1.1, 0], 3));
    geometry.setAttribute('skinIndex', new Uint16BufferAttribute([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4));
    geometry.setAttribute('skinWeight', new Float32BufferAttribute([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0], 4));
    geometry.setIndex([0, 1, 2, 3, 4, 5]);
    const mesh = new SkinnedMesh(geometry, new MeshBasicMaterial()); mesh.name = 'GeneratedBody';
    source.scene.add(mesh); source.scene.updateMatrixWorld(true); mesh.bind(new Skeleton([bones.hips!, bones.head!]));
    const vrm = new VRM({scene: source.scene, humanoid: source.humanoid, meta: source.meta, nodeConstraintManager: source.nodeConstraintManager,
      firstPerson: new VRMFirstPerson(source.humanoid, [{meshes: [mesh], type: 'auto'}])});
    const rig = new VRMHumanoidRigAdapter(vrm, 'generated-first-person');
    const erased = vrm.scene.getObjectByName('GeneratedBody(erase)') as SkinnedMesh;
    expect(erased).toBeInstanceOf(SkinnedMesh);
    expect([...erased.geometry.getIndex()!.array]).toEqual([3, 4, 5]);
    const player = new PerspectiveCamera(); player.layers.enable(9);
    const outside = new PerspectiveCamera(); outside.layers.enable(10);
    expect(player.layers.test(mesh.layers)).toBe(false); expect(player.layers.test(erased.layers)).toBe(true);
    expect(outside.layers.test(mesh.layers)).toBe(true); expect(outside.layers.test(erased.layers)).toBe(false);
    rig.setFirstPerson(true); expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    rig.setModelVisible(false); expect(mesh.visible).toBe(false); expect(erased.visible).toBe(false);
    rig.setModelVisible(true); expect(mesh.visible).toBe(true); expect(erased.visible).toBe(true);
    rig.dispose();
  });

  it('rejects nonuniform skeletal ancestor scale before taking ownership', () => {
    const {vrm} = generatedVRM(); vrm.scene.scale.set(1, 1.2, .8);
    expect(() => new VRMHumanoidRigAdapter(vrm, 'generated-bad-scale')).toThrow('Nonuniform');
    expect(vrm.scene.parent).toBeNull();
  });

  it('accepts absent optional fingers and keeps the neutral hand socket usable', () => {
    const {vrm} = generatedVRM();
    for (const side of ['left', 'right'] as const) {
      expect(vrm.humanoid.getNormalizedBoneNode(`${side}IndexProximal`)).toBeNull();
      expect(vrm.humanoid.getNormalizedBoneNode(`${side}ThumbDistal`)).toBeNull();
    }
    const rig = new VRMHumanoidRigAdapter(vrm, 'generated-no-fingers'), profile = rig.getProfile();
    for (const grip of profile.gripOffsets.slice(0, 2)) expect(new Vector3(...grip).length()).toBeLessThan(.09);
    rig.applyPose(createRestRigPose(profile));
    expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    expect(rig.root.getObjectByName('leftHandActualStickTip')).toBeDefined();
    expect(rig.root.getObjectByName('rightHandActualStickTip')).toBeDefined();
    rig.dispose();
  });

  it.each(['zero-segments', 'coincident-proximals', 'parallel-palm-basis'])('keeps optional %s finger geometry finite with a generic fallback', variant => {
    const {vrm} = generatedVRM(1, true);
    for (const side of ['left', 'right'] as const) for (const finger of ['Index', 'Middle', 'Ring', 'Little'] as const) {
      for (const segment of ['Proximal', 'Intermediate', 'Distal'] as const) {
        const name = `${side}${finger}${segment}` as VRMHumanBoneName;
        for (const node of [vrm.humanoid.getNormalizedBoneNode(name)!, vrm.humanoid.getRawBoneNode(name)!]) {
          if (variant === 'zero-segments' || segment === 'Proximal') node.position.set(0, 0, variant === 'parallel-palm-basis' ? .05 : 0);
        }
      }
    }
    const rig = new VRMHumanoidRigAdapter(vrm, `generated-${variant}`);
    for (const palmOrientation of ['up', 'inward', 'down'] as const) {
      const profile = rig.setCalibration({...rig.getCalibration(), palmOrientation}); rig.applyPose(createRestRigPose(profile));
      expect(matrices(rig).every(Number.isFinite)).toBe(true);
      expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    }
    rig.dispose();
  });
});
