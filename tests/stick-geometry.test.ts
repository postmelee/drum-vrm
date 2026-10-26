import { describe, expect, it } from 'vitest';
import { Mesh, Vector3 } from 'three';
import { createStickGeometry, STICK_PROFILE } from '../src/stick-geometry';
import { createRestRigPose, NativeHumanoidRigAdapter } from '../src/rig-humanoid';

describe('shared rendered stick volume', () => {
  it('places the actual tip at the contact marker and matches every taper radius', () => {
    const length = .31, geometry = createStickGeometry(length), vertices = geometry.getAttribute('position');
    const stations = [...STICK_PROFILE.terminal, [length + STICK_PROFILE.buttLength, STICK_PROFILE.terminal.at(-1)![1]]];
    for (const [back, radius] of stations) {
      const radial: number[] = [];
      for (let i = 0; i < vertices.count; i++) if (Math.abs(vertices.getZ(i) - (length - back)) < 1e-7) radial.push(Math.hypot(vertices.getX(i), vertices.getY(i)));
      expect(radial.length).toBeGreaterThan(0);
      expect(Math.max(...radial)).toBeCloseTo(radius, 7);
    }
    geometry.computeBoundingBox();
    expect(geometry.boundingBox!.max.z).toBeCloseTo(length, 7);
    expect(geometry.boundingBox!.min.z).toBeCloseTo(-STICK_PROFILE.buttLength, 7);
    expect(() => createStickGeometry(-.01)).toThrow();
    geometry.dispose();
  });
  it('keeps mesh tips on actual hand-local markers after calibration and seek', () => {
    const rig = new NativeHumanoidRigAdapter();
    for (const palmOrientation of ['natural', 'up', 'inward', 'down'] as const) {
      rig.setCalibration({ ...rig.getCalibration(), palmOrientation, seatForward: .08 });
      rig.applyPose(createRestRigPose(rig.getProfile()));
      for (const side of ['left', 'right']) {
        const stick = rig.root.getObjectByName(`${side}Stick`) as Mesh;
        const marker = rig.root.getObjectByName(`${side}HandActualStickTip`)!;
        const endpoint = stick.localToWorld(new Vector3(0, 0, .31));
        expect(endpoint.distanceTo(marker.getWorldPosition(new Vector3()))).toBeLessThan(1e-8);
      }
    }
    rig.dispose();
  });
});
