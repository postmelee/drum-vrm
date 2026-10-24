import { describe, expect, it } from 'vitest';
import { Vector3 } from 'three';
import { PrimitiveRigAdapter } from '../src/scene';
import type { KitPiece } from '../src/types';

const pose = () => new Float32Array([
  -.22, 1.35, -.10, -.43, 1.10, .08, -.28, 1.15, .10, -.28, 1.06, .40,
  .22, 1.35, -.10, .43, 1.10, .08, .28, 1.15, .10, .28, 1.06, .40,
  -.14, .71, -.15, -.22, .45, .20, -.22, .105, .42, -.22, .105, .55,
  .14, .71, -.15, .22, .45, .23, .20, .105, .49, .20, .105, .62,
  0, .71, -.15, 0, 1.24, -.10, 0, 1.62, -.10,
]);

const capture = (rig: PrimitiveRigAdapter) => rig.root.children.map(node => ({
  position: node.position.toArray(), rotation: node.quaternion.toArray(), scale: node.scale.toArray(),
}));

describe('PrimitiveRigAdapter canonical pose contract', () => {
  it('places the distal ends of both sticks at the supplied canonical contact points', () => {
    const rig = new PrimitiveRigAdapter();
    const input = pose();
    // Change the grip and contact independently to exercise orientation, not only translation.
    input.set([-.31, 1.07, .14, -.48, .96, .44], 6);
    input.set([.31, .91, .12, .18, .98, .74], 18);
    rig.applyPose(input);
    rig.root.updateMatrixWorld(true);
    for (const [name, offset] of [['leftStick', 9], ['rightStick', 21]] as const) {
      const tip = rig.root.getObjectByName(name)!.localToWorld(new Vector3(0, .5, 0));
      expect(tip.distanceTo(new Vector3().fromArray(input, offset))).toBeLessThan(1e-6);
    }
    expect(rig.root.getObjectByName('head')!.position.toArray()).toEqual(Array.from(input.slice(54)));
    rig.dispose();
  });

  it('returns the same complete visual pose after arbitrary seek order', () => {
    const rig = new PrimitiveRigAdapter();
    const first = pose();
    rig.applyPose(first);
    const initial = capture(rig);
    const second = first.map((value, index) => value + [0.4, .12, -.2][index % 3]);
    rig.applyPose(second);
    const translated = capture(rig);
    translated.forEach((node, index) => {
      node.position.forEach((value, axis) => {
        expect(value - initial[index].position[axis]).toBeCloseTo([.4, .12, -.2][axis], 5);
      });
    });
    rig.applyPose(first);
    expect(capture(rig)).toEqual(initial);
    expect(Array.from(first)).toEqual(Array.from(pose()));
    rig.dispose();
  });

  it('accepts changed kit dimensions as data and does not alias caller-owned targets', () => {
    const rig = new PrimitiveRigAdapter();
    const kit: KitPiece[] = [{ id: 'snare', label: 'Custom snare', position: [-.4, .85, .5], radius: .25 }];
    const calibration = rig.calibrate(kit);
    expect(calibration.targetPositions.snare).toEqual([-.4, .85, .5]);
    kit[0].position[0] = -.8;
    expect(calibration.targetPositions.snare?.[0]).toBe(-.4);
    expect(rig.calibrate(kit).targetPositions.snare?.[0]).toBe(-.8);
    expect(rig.describeRig().joints).toHaveLength(19);
    expect(rig.describeRig().poseFloats).toBe(57);
    rig.dispose();
  });

  it('rejects non-finite and incomplete poses before corrupting the displayed rig', () => {
    const rig = new PrimitiveRigAdapter();
    const initial = capture(rig);
    expect(() => rig.applyPose(new Float32Array(56))).toThrow('57');
    const invalid = pose(); invalid[4] = NaN;
    expect(() => rig.applyPose(invalid)).toThrow('finite');
    expect(capture(rig)).toEqual(initial);
    expect(() => rig.calibrate([{ id: 'snare', label: 'Invalid', position: [0, NaN, 0], radius: .2 }])).toThrow('Invalid rig target');
    rig.dispose();
  });
});
