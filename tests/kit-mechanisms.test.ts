import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { initSync } from '../src/wasm/drum_engine';
import { KitHardwareLayer, solveKitGeometry } from '../src/kit-hardware';
import { KitMechanismLayer, kickMechanismAmount, pedalPitchFromPose } from '../src/kit-mechanisms';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { measureInstrumentGeometry } from '../src/instrument-body';
import { DEFAULT_KIT } from '../src/scene';
import type { Hit } from '../src/types';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));

describe('deterministic instrument mechanisms', () => {
  it('hits at immutable kick times with continuous preparation/recovery and seek-order independence', () => {
    const times = [1, 1.12, 2], queries = [.3,.94,1,1.04,1.12,1.3,2,3];
    const expected = queries.map(time => kickMechanismAmount(times,time));
    expect(times.map(time => kickMechanismAmount(times,time))).toEqual([1,1,1]);
    expect(kickMechanismAmount(times,.3)).toBe(0);
    for (let i=queries.length-1;i>=0;i--) expect(kickMechanismAmount(times,queries[i])).toBe(expected[i]);
    expect(kickMechanismAmount(times,.999999)).toBeCloseTo(1,7);
    expect(kickMechanismAmount(times,1.000001)).toBeCloseTo(1,7);
  });
  it('extracts footboard pitch from semantic tool quaternions without avatar bone conventions', () => {
    const pose = new Float32Array(77), pitch = -.18;
    pose.set(new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1,0,0),pitch).toArray(),65);
    expect(pedalPitchFromPose(pose,2)).toBeCloseTo(pitch,7);
    expect(pedalPitchFromPose(undefined,2)).toBe(0);
  });
  it('inverts the lower hat and moves only its upper surface along the shared shaft', () => {
    const kit = structuredClone(DEFAULT_KIT), piece = kit.find(piece=>piece.id==='hihat')!;
    const model = createPieceModel(piece,resolvePieceVisual('studio',piece.id),'studio',kit,false);
    const lower = model.root.getObjectByName('HiHatBottom')!;
    const bottom = lower.children[0];
    expect(new THREE.Vector3(0,1,0).applyQuaternion(bottom.quaternion).y).toBeCloseTo(-1,7);
    piece.geometry = measureInstrumentGeometry(model.root,piece,'procedural');
    const hardware = new KitHardwareLayer(), geometry = solveKitGeometry(kit); hardware.setGeometry(kit,geometry);
    const mechanism = new KitMechanismLayer(hardware.root,[{id:piece.id,root:model.root}],geometry);
    model.root.updateMatrixWorld(true);
    const initialTop = model.cymbal!.getWorldPosition(new THREE.Vector3()), initialBottom = lower.getWorldPosition(new THREE.Vector3());
    mechanism.apply(1,.7); model.root.updateMatrixWorld(true);
    expect(model.cymbal!.getWorldPosition(new THREE.Vector3()).y-initialTop.y).toBeCloseTo(.025*.7,7);
    expect(lower.getWorldPosition(new THREE.Vector3()).distanceTo(initialBottom)).toBeLessThan(1e-9);
    mechanism.apply(0,0); model.root.updateMatrixWorld(true);
    expect(model.cymbal!.getWorldPosition(new THREE.Vector3()).distanceTo(initialTop)).toBeLessThan(1e-9);
    hardware.dispose();disposeModel(model.root);
  });
  it('brings the rendered beater surface to the measured batter contact at a source kick', () => {
    const kit = structuredClone(DEFAULT_KIT), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    const stage = new THREE.Group();stage.scale.x=-1;stage.add(hardware.root);hardware.setGeometry(kit,geometry);
    const mechanism = new KitMechanismLayer(hardware.root,[],geometry), hit = {instrument:'kick',limb:3,time:1} as Hit;
    mechanism.setHits([hit]);mechanism.apply(1,0);
    const expected = mechanism.contact('kick')!.point, actual = mechanism.actualContact('kick')!;
    expect(new THREE.Vector3(...actual).distanceTo(new THREE.Vector3(...expected))).toBeLessThan(1e-8);
    // Inspect independent rendered vertices: a correctly placed contact marker
    // must not conceal a smaller beater mesh floating short of the batter head.
    const beater = hardware.root.getObjectByName('KickBeaterPivot')!;
    let surfaceDistance = Infinity;
    beater.traverse(node => {
      if (!(node instanceof THREE.Mesh)) return;
      const positions = node.geometry.getAttribute('position');
      for (let i = 0; i < positions.count; i++) {
        const point = hardware.root.worldToLocal(new THREE.Vector3().fromBufferAttribute(positions, i).applyMatrix4(node.matrixWorld));
        surfaceDistance = Math.min(surfaceDistance, point.distanceTo(new THREE.Vector3(...expected)));
      }
    });
    expect(surfaceDistance).toBeLessThan(1e-7);
    mechanism.apply(0,0);expect(mechanism.actualContact('kick')).not.toEqual(actual);
    mechanism.apply(1,0);expect(mechanism.actualContact('kick')).toEqual(actual);
    const rebound = new KitMechanismLayer(hardware.root,[],geometry);
    rebound.setHits([hit]); rebound.apply(1,0);
    const markers: THREE.Object3D[] = []; hardware.root.traverse(node => { if(node.name==='BeaterSurfaceContact') markers.push(node); });
    expect(markers).toHaveLength(1);
    hardware.dispose();
  });
  it('preserves planning-axis displacement under reflected, rotated and nonuniform authored parents', () => {
    const kit = structuredClone(DEFAULT_KIT), geometry = solveKitGeometry(kit), frame = geometry.mechanisms!.find(frame=>frame.kind==='hihat')!;
    frame.motionAxis = [1,0,0]; frame.maxOpening = .025;
    const stage = new THREE.Group(); stage.scale.x=-1;
    const hardware = new THREE.Group(), body = new THREE.Group(), authored = new THREE.Group(), top = new THREE.Object3D();
    stage.add(hardware,body); body.add(authored); authored.add(top);
    authored.rotation.set(.3,.2,.4); authored.scale.set(2,3,4); top.position.set(.1,.2,.3); top.userData.mechanismRole='hihatTop';
    stage.updateMatrixWorld(true);
    const start = hardware.worldToLocal(top.getWorldPosition(new THREE.Vector3()));
    const layer = new KitMechanismLayer(hardware,[{id:frame.pieceId,root:body}],geometry);
    layer.apply(0,1); stage.updateMatrixWorld(true);
    const end = hardware.worldToLocal(top.getWorldPosition(new THREE.Vector3()));
    expect(end.sub(start).distanceTo(new THREE.Vector3(.025,0,0))).toBeLessThan(1e-10);
  });
});
