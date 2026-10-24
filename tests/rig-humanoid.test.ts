import { describe, expect, it } from 'vitest';
import { Bone, Group, Quaternion, Vector3 } from 'three';
import { createRestRigPose, legacyPositionToNative, legacyQuaternionToNative, NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import type { KitPiece } from '../src/types';

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
      const grip = new Vector3(...legacyPositionToNative(profile.gripOffsets[side === 'left' ? 0 : 1]));
      // The curled joints surround the shaft, rather than placing a finger
      // joint on its centre line (the old longitudinal socket regression).
      for (const finger of ['Index', 'Middle', 'Ring', 'Little']) {
        const points = ['Proximal', 'Intermediate', 'Distal'].map(segment => socket.worldToLocal(rig.root.getObjectByName(`${side}${finger}${segment}`)!.getWorldPosition(new Vector3())).sub(grip));
        expect(points[0].y).toBeGreaterThan(.01); expect(points[2].y).toBeLessThan(-.01);
        expect((side === 'left' ? 1 : -1) * points[1].x).toBeGreaterThan(.01);
        expect(Math.hypot(points[1].x, points[1].y)).toBeLessThan(.03);
      }
    }
    profile.roots[0][0] = 9; expect(rig.getProfile().roots[0][0]).not.toBe(9); rig.dispose();
  });
  it('calibrates seat height and fore/aft without accumulating offsets or changing limb lengths', () => {
    const rig = new NativeHumanoidRigAdapter(), initial = rig.getProfile();
    const kit: KitPiece[] = [{id: 'pedalHat', label: 'Hi-hat pedal', position: [-.24, .07, .52], radius: .1}, {id: 'kick', label: 'Kick pedal', position: [.22, .09, .59], radius: .1}];
    const adjusted = {seatHeight: .65, seatForward: .14};
    const profile = rig.setCalibration(adjusted, kit);
    expect(profile.pelvis).toEqual([0, .65, .14]); expect(profile.lengths).toEqual(initial.lengths);
    expect(profile.restTips.slice(2)).toEqual(kit.map(piece => piece.position));
    const delta = profile.roots[0].map((value, i) => value - initial.roots[0][i]);
    for (const key of ['pelvis', 'chest', 'head'] as const) profile[key].forEach((v, i) => expect(v - initial[key][i]).toBeCloseTo(delta[i], 12));
    rig.applyPose(createRestRigPose(profile));
    expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    rig.setCalibration({seatHeight: .54, seatForward: -.1});
    expect(rig.setCalibration(adjusted, kit)).toEqual(profile);
    const read = rig.getCalibration(); read.seatHeight = 1; expect(rig.getCalibration()).toEqual(adjusted);
    expect(() => rig.setCalibration({seatHeight: NaN, seatForward: 0})).toThrow('좌석');
    expect(() => rig.setCalibration({seatHeight: .6, seatForward: .51})).toThrow('좌석');
    expect(rig.getProfile()).toEqual(profile);
    const auto = rig.getAutomaticCalibration(kit); expect(auto.seatHeight).toBeGreaterThanOrEqual(.3); expect(auto.seatForward).toBe(.06);
    rig.dispose();
  });
  it('starts with both feet on the pedals and clamps unreachable rest poses without stretching', () => {
    const rig = new NativeHumanoidRigAdapter(), initial = rig.getProfile();
    expect(initial.restTips[2]).toEqual([-.22, .08, .55]); expect(initial.restTips[3]).toEqual([.2, .08, .62]);
    for (const limb of [2, 3]) expect(rig.getContactSamples()[limb].actual).toEqual(expect.arrayContaining(legacyPositionToNative(initial.restTips[limb]).map(value => expect.closeTo(value, 6))));
    const profile = rig.setCalibration({seatHeight: 1.2, seatForward: -.5}), pose = createRestRigPose(profile);
    rig.applyPose(pose);
    for (let limb = 2; limb < 4; limb++) {
      const offset = limb * 12, points = [0, 3, 6].map(p => new Vector3().fromArray(pose, offset + p));
      expect(points[0].distanceTo(points[1])).toBeCloseTo(profile.lengths[limb][0], 6);
      expect(points[1].distanceTo(points[2])).toBeCloseTo(profile.lengths[limb][1], 6);
      expect(new Vector3().fromArray(pose, offset + 9).distanceTo(new Vector3(...profile.restTips[limb]))).toBeGreaterThan(.1);
    }
    expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true); rig.dispose();
  });
  it('keeps rest IK finite and bone lengths intact when a moved pedal aligns with the knee pole', () => {
    const rig = new NativeHumanoidRigAdapter(), profile = rig.getProfile();
    const endpoint = new Vector3(...profile.roots[2]).add(new Vector3(0, 0, .5));
    profile.restTips[2] = endpoint.add(new Vector3(...profile.gripOffsets[2])).add(new Vector3(...profile.tipOffsets[2])).toArray();
    const pose = createRestRigPose(profile), points = [24, 27, 30].map(offset => new Vector3().fromArray(pose, offset));
    expect(pose.every(Number.isFinite)).toBe(true);
    expect(points[0].distanceTo(points[1])).toBeCloseTo(profile.lengths[2][0], 6);
    expect(points[1].distanceTo(points[2])).toBeCloseTo(profile.lengths[2][1], 6); rig.dispose();
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
