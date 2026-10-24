import { describe, expect, it } from 'vitest';
import { Bone, Group, Quaternion, Vector3 } from 'three';
import { createRestRigPose, legacyPositionToNative, legacyQuaternionToNative, NativeHumanoidRigAdapter } from '../src/rig-humanoid';

describe('semantic humanoid and hand-local tools', () => {
  it('uses a real positive-scale hierarchy with fingers and measured lengths', () => {
    const rig = new NativeHumanoidRigAdapter();
    expect(rig.humanoid.getRawBoneNode('leftHand')).toBeInstanceOf(Bone);
    expect(rig.humanoid.getRawBoneNode('leftIndexDistal')!.parent).toBe(rig.humanoid.getRawBoneNode('leftIndexIntermediate'));
    const profile = rig.getProfile(); expect(profile.lengths[0][0]).toBeCloseTo(.30); expect(profile.lengths[0][1]).toBeCloseTo(.27);
    expect(profile.lengths[0][0]).not.toBe(.43);
    rig.root.traverse(node => { expect(Math.min(node.scale.x, node.scale.y, node.scale.z)).toBeGreaterThan(0); });
    expect(rig.humanoid.getNormalizedBoneNode('leftIndexIntermediate')!.quaternion.angleTo(new Quaternion())).toBeGreaterThan(.5);
    expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    for (const side of ['left', 'right']) {
      const socket = rig.root.getObjectByName(`${side}HandToolSocket`)!;
      const shaft = socket.getWorldDirection(new Vector3());
      const finger = rig.humanoid.getRawBoneNode(`${side}IndexIntermediate` as 'leftIndexIntermediate' | 'rightIndexIntermediate')!.getWorldPosition(new Vector3());
      const tip = rig.root.getObjectByName(`${side}HandActualStickTip`)!.getWorldPosition(new Vector3());
      expect(finger.sub(tip).cross(shaft).length()).toBeLessThan(.006);
    }
    profile.roots[0][0] = 9; expect(rig.getProfile().roots[0][0]).not.toBe(9); rig.dispose();
  });
  it('renders actual parented stick tips without stretching limbs and repeats after seek', () => {
    const rig = new NativeHumanoidRigAdapter(), profile = rig.getProfile(), pose = createRestRigPose(profile), initial = rig.getContactSamples();
    const changed = pose.slice();
    // Change the hand orientation and corresponding tool target, keeping the wrist fixed.
    const q = new Quaternion().setFromAxisAngle(new Vector3(1, 0, 0), -.3);
    changed.set(q.toArray(), 57);
    const offset = new Vector3(...profile.gripOffsets[0]).add(new Vector3(...profile.tipOffsets[0])).applyQuaternion(q);
    changed.set(new Vector3().fromArray(changed, 6).add(offset).toArray(), 9);
    rig.applyPose(changed);
    expect(rig.getContactSamples()[0].errorMeters).toBeLessThan(1e-6);
    expect(rig.getContactSamples()[0].actual).not.toEqual(initial[0].actual);
    expect(rig.root.getObjectByName('leftHandActualStickTip')!.parent!.parent).toBe(rig.humanoid.getRawBoneNode('leftHand'));
    rig.applyPose(pose); expect(rig.getContactSamples()).toEqual(initial);
    const raw = rig.humanoid.getRawBoneNode('leftUpperArm')!, middle = rig.humanoid.getRawBoneNode('leftLowerArm')!;
    expect(raw.getWorldPosition(new Vector3()).distanceTo(middle.getWorldPosition(new Vector3()))).toBeCloseTo(.3, 6);
    const before = rig.getContactSamples(); changed[3] = NaN; expect(() => rig.applyPose(changed)).toThrow('73 finite'); expect(rig.getContactSamples()).toEqual(before);
    rig.dispose();
  });
  it('converts rotations and points consistently without reflecting native bones', () => {
    const point = new Vector3(.1, -.2, .3), q = new Quaternion().setFromAxisAngle(new Vector3(.4, .2, .1).normalize(), .7);
    const a = new Vector3(...legacyPositionToNative(point.clone().applyQuaternion(q).toArray()));
    const b = new Vector3(...legacyPositionToNative(point.toArray())).applyQuaternion(new Quaternion(...legacyQuaternionToNative(q.toArray())));
    expect(a.distanceTo(b)).toBeLessThan(1e-12);
    const scene = new Group(), rig = new NativeHumanoidRigAdapter(); scene.add(rig.root); scene.updateMatrixWorld(true);
    expect(rig.root.matrixWorld.determinant()).toBeGreaterThan(0); expect(rig.getContactSamples()[0].actual[0]).toBeGreaterThan(0);
    rig.dispose();
  });
});
