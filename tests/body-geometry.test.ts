import { describe, expect, it } from 'vitest';
import { Bone, BufferGeometry, Float32BufferAttribute, Group, MeshBasicMaterial, Quaternion, Skeleton, SkinnedMesh, Uint16BufferAttribute, Vector3 } from 'three';
import { MAX_BODY_SAMPLES, measureBodyGeometry, skeletonBodyGeometry, validateBodyGeometry } from '../src/body-geometry';
import { createRestRigPose, NativeHumanoidRigAdapter, NormalizedHumanoidRigAdapter } from '../src/rig-humanoid';
import type { BodyGeometry } from '../src/types';

const reflect = (world: Vector3) => world.setX(-world.x);
/** Independently authored volume, with unrelated head-weighted long decoration. */
function authoredSkin(rig: NativeHumanoidRigAdapter, depth: number, repeats = 1, withHand = false) {
  const profile = rig.getProfile(), baseline = skeletonBodyGeometry(profile), positions: number[] = [], indices: number[] = [], weights: number[] = [];
  const bones = [rig.humanoid.getRawBoneNode('spine')!, rig.humanoid.getRawBoneNode('head')!, rig.humanoid.getRawBoneNode('leftHand')!] as Bone[];
  const add = (x: number, y: number, z: number, bone: number) => { positions.push(-x, y, z); indices.push(bone, 0, 0, 0); weights.push(1, 0, 0, 0); };
  for (let repeat = 0; repeat < repeats; repeat++) for (let y = 0; y < 5; y++) for (let angle = 0; angle < 64; angle++) {
    const a = angle * Math.PI / 32;
    add(baseline.torso.center[0] + Math.cos(a) * .14, baseline.torso.center[1] + (y - 2) * .05, baseline.torso.center[2] + Math.sin(a) * depth, 0);
  }
  for (let i = 0; i < 100; i++) add(2, baseline.torso.center[1], 3, 1);
  if (withHand) {
    const hand = reflect(bones[2].getWorldPosition(new Vector3())).add(new Vector3(.06, 0, .03));
    for (let i = 0; i < 32; i++) for (let j = -2; j <= 2; j++) {
      const a = i * Math.PI / 16;
      add(hand.x + j * .025, hand.y + Math.cos(a) * .012, hand.z + Math.sin(a) * .012, 2);
    }
  }
  const geometry = new BufferGeometry();
  geometry.setAttribute('position', new Float32BufferAttribute(positions, 3));
  geometry.setAttribute('skinIndex', new Uint16BufferAttribute(indices, 4));
  geometry.setAttribute('skinWeight', new Float32BufferAttribute(weights, 4));
  const source = new Group(), mesh = new SkinnedMesh(geometry, new MeshBasicMaterial());
  source.add(mesh); source.updateMatrixWorld(true); mesh.bind(new Skeleton(bones));
  return { source, mesh };
}

describe('semantic sampled body geometry', () => {
  it('measures torso depth from skin geometry and ignores head-weighted decoration without avatar names', () => {
    const rig = new NativeHumanoidRigAdapter(), thin = authoredSkin(rig, .08), thick = authoredSkin(rig, .16);
    const a = measureBodyGeometry(thin.source, rig.humanoid, rig.getProfile(), reflect);
    const b = measureBodyGeometry(thick.source, rig.humanoid, rig.getProfile(), reflect);
    expect(a.source).toBe('mesh-estimate'); expect(a.sampleCount).toBe(320);
    expect(b.torso.halfExtents[2] - a.torso.halfExtents[2]).toBeCloseTo(.08, 5);
    expect(b.torso.halfExtents[0]).toBeLessThan(.16);
    expect(b.torso.halfExtents[2]).toBeLessThan(.18);
    expect(b.armRadii).toEqual(skeletonBodyGeometry(rig.getProfile()).armRadii);
    expect(() => validateBodyGeometry(b)).not.toThrow();
    rig.dispose();
  });
  it('has a bounded deterministic sampling budget across meshes', () => {
    const rig = new NativeHumanoidRigAdapter(), data = authoredSkin(rig, .12, 100);
    const first = measureBodyGeometry(data.source, rig.humanoid, rig.getProfile(), reflect);
    expect(first.source).toBe('mesh-estimate'); expect(first.sampleCount).toBeLessThanOrEqual(MAX_BODY_SAMPLES);
    expect(measureBodyGeometry(data.source, rig.humanoid, rig.getProfile(), reflect)).toEqual(first);
    rig.dispose();
  });
  it('places the sampled hand sphere at its measured center in the tool frame, not at the wrist', () => {
    const rig = new NativeHumanoidRigAdapter(), data = authoredSkin(rig, .12, 1, true);
    const rotation = new Quaternion().setFromAxisAngle(new Vector3(0, 1, 0), .7);
    const measured = measureBodyGeometry(data.source, rig.humanoid, rig.getProfile(), reflect, [rotation.toArray(), [0, 0, 0, 1]]);
    const worldOffset = new Vector3(...measured.handCenters![0]).applyQuaternion(rotation);
    expect(worldOffset.x).toBeCloseTo(.06, 5); expect(worldOffset.y).toBeCloseTo(0, 5); expect(worldOffset.z).toBeCloseTo(.03, 5);
    expect(measured.handRadii[0]).toBeGreaterThan(.05);
    const capsule = measured.handCapsules![0];
    expect(capsule.radius).toBeGreaterThan(.012); expect(capsule.radius).toBeLessThan(.02);
    expect(new Vector3(...capsule.start).distanceTo(new Vector3(...capsule.end))).toBeCloseTo(.1, 5);
    rig.dispose();
  });
  it('measures cached closed-finger geometry and keeps it aligned after changing palm orientation', () => {
    const source = new NativeHumanoidRigAdapter(), skin = authoredSkin(source, .12, 1, true);
    skin.mesh.bind(new Skeleton([source.humanoid.getRawBoneNode('spine')!, source.humanoid.getRawBoneNode('head')!, source.humanoid.getRawBoneNode('leftIndexDistal')!] as Bone[]));
    source.root.add(skin.mesh);
    const rig = new NormalizedHumanoidRigAdapter(source.humanoid, source.root, 'authored-skinned-hand');
    for (const palmOrientation of ['up', 'inward', 'down'] as const) {
      const profile = rig.setCalibration({ ...rig.getCalibration(), palmOrientation });
      rig.applyPose(createRestRigPose(profile));
      // The source includes earlier sockets; select the newly appended ones.
      const sockets = (['left', 'right'] as const).map(side => rig.humanoid.getRawBoneNode(`${side}Hand`)!.children.filter(child => child.name === `${side}HandToolSocket`).at(-1)!);
      const rotations = sockets.map(socket => { const q = socket.getWorldQuaternion(new Quaternion()); return [q.x, -q.y, -q.z, q.w]; }) as [[number, number, number, number], [number, number, number, number]];
      const measured = measureBodyGeometry(rig.root, rig.humanoid, profile, reflect, rotations);
      const oldCenter = new Vector3(...profile.bodyGeometry!.handCapsules![0].start).add(new Vector3(...profile.bodyGeometry!.handCapsules![0].end)).multiplyScalar(.5);
      const newCenter = new Vector3(...measured.handCapsules![0].start).add(new Vector3(...measured.handCapsules![0].end)).multiplyScalar(.5);
      expect(oldCenter.distanceTo(newCenter)).toBeLessThan(1e-5);
      expect(measured.handCapsules![0].radius).toBeCloseTo(profile.bodyGeometry!.handCapsules![0].radius, 5);
    }
    rig.dispose();
  });
  it('keeps measured dimensions unchanged under the seated translation and moves its center once', () => {
    const rig = new NativeHumanoidRigAdapter(), data = authoredSkin(rig, .12), original = rig.getProfile();
    const before = measureBodyGeometry(data.source, rig.humanoid, original, reflect);
    const translated = structuredClone(original);
    translated.roots.forEach(p => { p[1] += .2; p[2] -= .1; }); translated.pelvis[1] += .2; translated.pelvis[2] -= .1;
    const after = measureBodyGeometry(data.source, rig.humanoid, translated, v => reflect(v).add(new Vector3(0, .2, -.1)));
    after.torso.center.forEach((value, i) => expect(value - before.torso.center[i]).toBeCloseTo([0, .2, -.1][i], 12));
    expect(after.torso.halfExtents).toEqual(before.torso.halfExtents);
    expect(after.armRadii).toEqual(before.armRadii);
    rig.dispose();
  });
  it('measures physical widths in the shoulder frame after a uniform scale and world yaw', () => {
    const rig = new NativeHumanoidRigAdapter(), data = authoredSkin(rig, .12), profile = rig.getProfile();
    const scale = 1.25, yaw = new Quaternion().setFromAxisAngle(new Vector3(0, 1, 0), .63), offset = new Vector3(.2, .1, -.3);
    const transform = (p: Vector3) => p.multiplyScalar(scale).applyQuaternion(yaw).add(offset);
    const expectedCenter = transform(new Vector3(...profile.bodyGeometry!.torso.center));
    profile.roots = profile.roots.map(p => transform(new Vector3(...p)).toArray()) as typeof profile.roots;
    profile.pelvis = transform(new Vector3(...profile.pelvis)).toArray();
    profile.lengths = profile.lengths.map(pair => pair.map(n => n * scale)) as typeof profile.lengths;
    const measured = measureBodyGeometry(data.source, rig.humanoid, profile, world => transform(reflect(world)));
    expect(measured.source).toBe('mesh-estimate');
    expect(measured.torso.halfExtents[0]).toBeCloseTo(.14 * scale + .004, 5);
    expect(measured.torso.halfExtents[2]).toBeCloseTo(.12 * scale + .004, 5);
    expect(measured.torso.center[0]).toBeCloseTo(expectedCenter.x, 5);
    expect(measured.torso.center[2]).toBeCloseTo(expectedCenter.z, 5);
    rig.dispose();
  });
  it('uses an explicit fallback for missing or degenerate skin and preserves lengths through calibration', () => {
    const rig = new NativeHumanoidRigAdapter(), original = rig.getProfile();
    expect(original.bodyGeometry?.source).toBe('skeleton-fallback');
    expect(original.bodyGeometry?.sampleCount).toBe(0);
    expect(measureBodyGeometry(new Group(), rig.humanoid, original, reflect)).toEqual(skeletonBodyGeometry(original));
    const settings = { ...rig.getCalibration(), seatHeight: .65, seatForward: .12 }, fitted = rig.setCalibration(settings);
    expect(fitted.lengths).toEqual(original.lengths);
    expect(fitted.bodyGeometry?.torso.halfExtents).toEqual(original.bodyGeometry?.torso.halfExtents);
    expect(fitted.bodyGeometry!.torso.center[1] - original.bodyGeometry!.torso.center[1]).toBeCloseTo(fitted.pelvis[1] - original.pelvis[1]);
    expect(rig.setCalibration(settings)).toEqual(fitted);
    rig.dispose();
  });
  it('rejects malformed versions, non-finite or unbounded values before project restoration', () => {
    const rig = new NativeHumanoidRigAdapter(), source = rig.getProfile().bodyGeometry!;
    const reject = (change: (copy: BodyGeometry) => void) => { const copy = structuredClone(source); change(copy); expect(() => validateBodyGeometry(copy)).toThrow(); };
    reject(p => { p.version = 2 as 1; });
    reject(p => { p.source = 'unknown' as BodyGeometry['source']; });
    reject(p => { p.torso.halfExtents[0] = NaN; });
    reject(p => { p.torso.center[2] = Infinity; });
    reject(p => { p.armRadii[1][0] = -.1; });
    reject(p => { p.handRadii[0] = 1; });
    reject(p => { p.handCenters = [[.5, .5, .5], [0, 0, 0]]; });
    reject(p => { p.handCapsules = [{ start: [0, 0, 0], end: [0, 0, .1], radius: NaN }, { start: [0, 0, 0], end: [0, 0, .1], radius: .02 }]; });
    reject(p => { p.sampleCount = 1.5; });
    expect(() => validateBodyGeometry(source)).not.toThrow(); rig.dispose();
  });
});
