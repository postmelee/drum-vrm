import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { OBB } from 'three/addons/math/OBB.js';
import { alignInstrumentBody, measureInstrumentGeometry } from '../src/instrument-body';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { DEFAULT_KIT } from '../src/scene';
import type { KitPiece, PieceGeometry, Vec3 } from '../src/types';

function absoluteBounds(piece: KitPiece, geometry: PieceGeometry) {
  const ratio = piece.radius / geometry.referenceRadius;
  const center = new THREE.Vector3(...geometry.body.center).multiplyScalar(ratio).add(new THREE.Vector3(...piece.position));
  const half = new THREE.Vector3(...geometry.body.halfExtents).multiplyScalar(ratio);
  const bounds = new THREE.Box3(), rotation = new THREE.Quaternion(...geometry.body.rotation ?? [0, 0, 0, 1]);
  for (const x of [-1, 1]) for (const y of [-1, 1]) for (const z of [-1, 1]) bounds.expandByPoint(new THREE.Vector3(x * half.x, y * half.y, z * half.z).applyQuaternion(rotation).add(center));
  return bounds;
}
function expectSameBounds(first: THREE.Box3, second: THREE.Box3) {
  expect(first.min.distanceTo(second.min)).toBeLessThan(1e-6);
  expect(first.max.distanceTo(second.max)).toBeLessThan(1e-6);
}

describe('measured equipment body descriptors', () => {
  it('keeps authored playing radii in the surface frame through tilt, yaw and fitted scaling', () => {
    for (const id of ['snare', 'tomHigh', 'crashRight'] as const) {
      const original = DEFAULT_KIT.find(piece => piece.id === id)!;
      let expectedRadius: number | undefined;
      for (const tilt of [0, 15, 20, 25]) for (const rotation of [0, 27]) {
        const settings = resolvePieceVisual('studio', id, { tilt, rotation });
        const model = createPieceModel(original, settings, 'studio', DEFAULT_KIT, false);
        const geometry = measureInstrumentGeometry(model.root, original, 'procedural');
        expectedRadius ??= geometry.surface.radius;
        expect(geometry.surface.radius, `${id}/${tilt}/${rotation}`).toBeCloseTo(expectedRadius, 6);
        if (geometry.surface.radialProfile) {
          const profile = geometry.surface.radialProfile;
          expect(geometry.surface.radius).toBe(profile.points.at(-1)![0]);
          const origin = new THREE.Vector3(...profile.origin).add(new THREE.Vector3(...original.position)), inverse = new THREE.Quaternion(...profile.rotation).invert();
          const mesh = model.root.getObjectByName('contactSurface') as THREE.Mesh, vertices = mesh.geometry.getAttribute('position');
          let outerRadius = 0;
          for (let index = 0; index < vertices.count; index++) {
            const point = new THREE.Vector3().fromBufferAttribute(vertices, index).applyMatrix4(mesh.matrixWorld).sub(origin).applyQuaternion(inverse);
            outerRadius = Math.max(outerRadius, Math.hypot(point.x, point.z));
          }
          expect(outerRadius).toBeCloseTo(geometry.surface.radius, 7);
        }
        disposeModel(model.root);
        const fitted = { ...original, radius: original.radius * .8, geometry };
        const display = createPieceModel(fitted, settings, 'studio', DEFAULT_KIT, false);
        expect(measureInstrumentGeometry(display.root, fitted, 'procedural')).toEqual(geometry);
        disposeModel(display.root);
      }
    }
  });

  it('measures semantic flat imports in their own plane and preserves the unknown-body fallback', () => {
    const piece = DEFAULT_KIT.find(value => value.id === 'snare')!;
    for (const tilt of [15, 20, 25]) for (const yaw of [0, 37]) {
      const root = new THREE.Group(), plane = new THREE.Mesh(new THREE.PlaneGeometry(.8, .4), new THREE.MeshBasicMaterial({ side: THREE.DoubleSide }));
      plane.rotation.x = -Math.PI / 2; plane.userData.hitResponseRole = 'surface'; root.add(plane);
      root.position.set(...piece.position); root.rotation.set(-THREE.MathUtils.degToRad(tilt), THREE.MathUtils.degToRad(yaw), 0);
      const measured = measureInstrumentGeometry(root, piece, 'mesh-bounds');
      expect(measured.surface.radius).toBeCloseTo(.2, 7);
      const normal = new THREE.Vector3(...measured.surface.normal), center = new THREE.Vector3(...measured.surface.center).add(new THREE.Vector3(...piece.position));
      const tangent = new THREE.Vector3(1, 0, 0).transformDirection(plane.matrixWorld), across = new THREE.Vector3().crossVectors(tangent, normal);
      for (let step = 0; step < 12; step++) {
        const angle = step * Math.PI / 6;
        const target = center.clone().addScaledVector(tangent, .19 * Math.cos(angle)).addScaledVector(across, .19 * Math.sin(angle));
        const hit = new THREE.Raycaster(target.clone().addScaledVector(normal, .1), normal.clone().negate()).intersectObject(plane)[0];
        expect(hit).toBeDefined(); expect(hit.point.distanceTo(target)).toBeLessThan(1e-7);
      }
      delete plane.userData.hitResponseRole;
      const unknown = measureInstrumentGeometry(root, piece, 'mesh-bounds'), worldSize = new THREE.Box3().setFromObject(root, true).getSize(new THREE.Vector3());
      expect(unknown.surface.radius).toBeCloseTo(Math.min(worldSize.x, worldSize.z) / 2, 7);
      disposeModel(root);
    }
  });

  it('tilts rack heads toward the player and measures the whole tilted shell in the same frame', () => {
    for (const id of ['tomHigh', 'tomMid'] as const) {
      const piece = DEFAULT_KIT.find(value => value.id === id)!;
      const visual = resolvePieceVisual('studio', id);
      const model = createPieceModel(piece, visual, 'studio', DEFAULT_KIT, false);
      const geometry = measureInstrumentGeometry(model.root, piece, 'procedural');
      const expectedNormal = new THREE.Vector3(0, 1, 0).applyAxisAngle(new THREE.Vector3(1, 0, 0), -THREE.MathUtils.degToRad(visual.tilt));
      expect(visual.tilt).toBe(25);
      expect(expectedNormal.z).toBeLessThan(0); // seated player is toward -Z
      expect(new THREE.Vector3(...geometry.surface.normal).distanceTo(expectedNormal)).toBeLessThan(1e-7);
      const surfaces: THREE.Object3D[] = [];
      model.root.traverse(node => { if (node.name === 'contactSurface') surfaces.push(node); });
      const center = new THREE.Vector3(...piece.position).add(new THREE.Vector3(...geometry.surface.center));
      const hit = new THREE.Raycaster(center.clone().addScaledVector(expectedNormal, .1), expectedNormal.clone().negate()).intersectObjects(surfaces, true)[0];
      expect(hit).toBeDefined(); expect(hit.point.distanceTo(center)).toBeLessThan(1e-6);
      const bounds = new THREE.Box3(new THREE.Vector3(...geometry.body.halfExtents).negate(), new THREE.Vector3(...geometry.body.halfExtents)).expandByScalar(1e-7);
      const bodyCenter = new THREE.Vector3(...geometry.body.center).add(new THREE.Vector3(...piece.position));
      const inverse = new THREE.Quaternion(...geometry.body.rotation!).invert();
      expect(new THREE.Vector3(0, 1, 0).applyQuaternion(inverse.clone().invert()).distanceTo(expectedNormal)).toBeLessThan(1e-7);
      model.root.traverse(node => {
        if (!(node instanceof THREE.Mesh) || node.userData.hitResponseRole === 'marker') return;
        const vertices = node.geometry.getAttribute('position');
        for (let index = 0; index < vertices.count; index++) expect(bounds.containsPoint(new THREE.Vector3().fromBufferAttribute(vertices, index).applyMatrix4(node.matrixWorld).sub(bodyCenter).applyQuaternion(inverse))).toBe(true);
      });
      disposeModel(model.root);
    }
  });

  it('removes empty world-box corners for a 20 degree rack tilt while retaining a real lowered shell collision', () => {
    const measured = (id: 'snare' | 'tomHigh', tilt: number, drop = 0) => {
      const piece = structuredClone(DEFAULT_KIT.find(value => value.id === id)!); piece.position[1] -= drop;
      const model = createPieceModel(piece, resolvePieceVisual('studio', id, { tilt }), 'studio', DEFAULT_KIT, false);
      const geometry = measureInstrumentGeometry(model.root, piece, 'procedural'); disposeModel(model.root);
      const rotation = new THREE.Matrix3().setFromMatrix4(new THREE.Matrix4().makeRotationFromQuaternion(new THREE.Quaternion(...geometry.body.rotation ?? [0, 0, 0, 1])));
      return { world: absoluteBounds(piece, geometry), obb: new OBB(new THREE.Vector3(...geometry.body.center).add(new THREE.Vector3(...piece.position)), new THREE.Vector3(...geometry.body.halfExtents), rotation) };
    };
    const snare = measured('snare', 4), tilted = measured('tomHigh', 20), lowered = measured('tomHigh', 12, .05);
    expect(snare.world.intersectsBox(tilted.world)).toBe(true);
    // Expand each solid by half the unchanged 5 mm equipment clearance.
    snare.obb.halfSize.addScalar(.0025); tilted.obb.halfSize.addScalar(.0025); lowered.obb.halfSize.addScalar(.0025);
    expect(snare.obb.intersectsOBB(tilted.obb)).toBe(false);
    expect(snare.obb.intersectsOBB(lowered.obb)).toBe(true);
  });

  it('contains every drawn body vertex in its measured local box across presets, rotations and fitted sizes', () => {
    for (const preset of ['studio', 'vintage', 'electronic'] as const) for (const original of DEFAULT_KIT.filter(piece => !['openHat', 'pedalHat'].includes(piece.id))) {
      const visual = resolvePieceVisual(preset, original.id, { tilt: 20, rotation: 27, scale: 1.1 });
      const base = createPieceModel(original, visual, preset, DEFAULT_KIT, false);
      const reference = measureInstrumentGeometry(base.root, original, 'procedural'); disposeModel(base.root);
      const piece = { ...original, geometry: reference, radius: original.radius * .76 };
      const model = createPieceModel(piece, visual, preset, DEFAULT_KIT, false); model.root.updateMatrixWorld(true);
      const inverse = new THREE.Quaternion(...reference.body.rotation ?? [0, 0, 0, 1]).invert();
      const center = new THREE.Vector3(...reference.body.center).multiplyScalar(.76).add(new THREE.Vector3(...piece.position));
      const half = reference.body.halfExtents.map(value => value * .76);
      let largestEscape = -Infinity, drawn = 0;
      model.root.traverse(node => {
        if (!(node instanceof THREE.Mesh)) return;
        for (let parent: THREE.Object3D | null = node; parent; parent = parent.parent) {
          if (!parent.visible || ['marker', 'hardware', 'pedal'].includes(parent.userData.hitResponseRole) || parent.userData.fixedContact) return;
          if (parent === model.root) break;
        }
        const positions = node.geometry.getAttribute('position'), indices = node.geometry.index;
        const end = Math.min(indices?.count ?? positions.count, node.geometry.drawRange.start + node.geometry.drawRange.count);
        for (let i = node.geometry.drawRange.start; i < end; i++) {
          const local = new THREE.Vector3().fromBufferAttribute(positions, indices ? indices.getX(i) : i).applyMatrix4(node.matrixWorld).sub(center).applyQuaternion(inverse);
          largestEscape = Math.max(largestEscape, ...local.toArray().map((value, axis) => Math.abs(value) - half[axis])); drawn++;
        }
      });
      expect(drawn).toBeGreaterThan(0);
      expect(largestEscape, `${preset}/${piece.id}`).toBeLessThan(1e-7); disposeModel(model.root);
    }
  });

  it.each(['studio', 'vintage', 'electronic'] as const)('scales all %s body dimensions with a fitted radius while retaining serialized reference geometry', preset => {
    for (const original of DEFAULT_KIT.filter(piece => !['openHat', 'pedalHat'].includes(piece.id))) {
      const visual = resolvePieceVisual(preset, original.id, { rotation: 17, scale: 1.1 });
      const base = createPieceModel(original, visual, preset, DEFAULT_KIT, false);
      const descriptor = measureInstrumentGeometry(base.root, original, 'procedural');
      const fitted: KitPiece = { ...original, geometry: descriptor, radius: original.radius * .8, position: original.position.map((value, axis) => value + [0.04, .03, -.05][axis]) as Vec3 };
      const display = createPieceModel(fitted, visual, preset, DEFAULT_KIT, false);
      const measured = measureInstrumentGeometry(display.root, fitted, 'procedural');
      expectSameBounds(absoluteBounds(fitted, descriptor), absoluteBounds(fitted, measured));
      expect(measured).toEqual(descriptor);
      disposeModel(base.root); disposeModel(display.root);
    }
  });

  it('retains the same fitted reference for rotated imported bodies, including the kick offset', () => {
    for (const id of ['kick', 'snare', 'hihat'] as const) {
      const original = DEFAULT_KIT.find(piece => piece.id === id)!;
      const build = (piece: KitPiece) => {
        const display = new THREE.Group(), pivot = new THREE.Group();
        display.add(new THREE.Mesh(new THREE.BoxGeometry(1, .8, .6), new THREE.MeshBasicMaterial())); pivot.add(display);
        alignInstrumentBody(display, pivot, piece, resolvePieceVisual('studio', id, { rotation: 31 })); return pivot;
      };
      const initial = build(original), descriptor = measureInstrumentGeometry(initial, original, 'mesh-bounds');
      const fitted: KitPiece = { ...original, geometry: descriptor, radius: original.radius * .83, position: original.position.map((value, axis) => value + [-.03, .02, .04][axis]) as Vec3 };
      const display = build(fitted), measured = measureInstrumentGeometry(display, fitted, 'mesh-bounds');
      expectSameBounds(absoluteBounds(fitted, descriptor), absoluteBounds(fitted, measured));
      expect(measured).toEqual(descriptor); disposeModel(initial); disposeModel(display);
    }
  });

  it('ignores selection markers and hardware and works before display-axis reflection', () => {
    const piece = DEFAULT_KIT.find(value => value.id === 'snare')!, root = new THREE.Group();
    const body = new THREE.Mesh(new THREE.BoxGeometry(.4, .2, .3), new THREE.MeshBasicMaterial());
    body.position.set(...piece.position); root.add(body);
    const unwanted = new THREE.Mesh(new THREE.BoxGeometry(10, 10, 10), new THREE.MeshBasicMaterial());
    unwanted.userData.hitResponseRole = 'marker'; root.add(unwanted);
    const hardware = unwanted.clone(); hardware.userData.hitResponseRole = 'hardware'; root.add(hardware);
    const stage = new THREE.Group(); stage.scale.x = -1; stage.add(root);
    const descriptor = measureInstrumentGeometry(root, piece, 'mesh-bounds');
    descriptor.body.halfExtents.forEach((value, axis) => expect(value).toBeCloseTo([.2, .1, .15][axis], 7));
    descriptor.body.center.forEach(value => expect(value).toBeCloseTo(0, 7)); disposeModel(stage);
  });

  it('keeps a sloped surface plane through the actual off-center stick contact', () => {
    const piece = DEFAULT_KIT.find(value => value.id === 'hihat')!, display = new THREE.Group(), pivot = new THREE.Group();
    const plane = new THREE.Mesh(new THREE.PlaneGeometry(.8, .8), new THREE.MeshBasicMaterial({ side: THREE.DoubleSide }));
    plane.rotation.x = -Math.PI / 2 + .25; plane.name = 'contactSurface'; display.add(plane); pivot.add(display);
    alignInstrumentBody(display, pivot, piece, resolvePieceVisual('studio', piece.id));
    const geometry = measureInstrumentGeometry(pivot, piece, 'mesh-bounds');
    const target = new THREE.Vector3(...piece.position), center = new THREE.Vector3(...geometry.surface.center).add(target), normal = new THREE.Vector3(...geometry.surface.normal);
    const hit = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0)).intersectObject(plane)[0];
    expect(hit.point.distanceTo(target)).toBeLessThan(1e-7);
    expect(Math.abs(normal.z)).toBeGreaterThan(.2); expect(normal.length()).toBeCloseTo(1, 7);
    expect(Math.abs(normal.dot(hit.point.clone().sub(center)))).toBeLessThan(1e-7);
    expect(Math.abs(center.y - target.y)).toBeGreaterThan(.01); disposeModel(pivot);
  });

  it('keeps numeric bell, bow and edge contacts on rendered curved cymbals after tilt and body fitting', () => {
    for(const id of ['hihat','crash','ride'] as const) for(const fit of [.72,1,1.3]) {
      const original=DEFAULT_KIT.find(piece=>piece.id===id)!;
      const visual=resolvePieceVisual('studio',id,{tilt:18,rotation:27,scale:1.1});
      const base=createPieceModel(original,visual,'studio',DEFAULT_KIT,false);
      const geometry=measureInstrumentGeometry(base.root,original,'procedural');disposeModel(base.root);
      const piece={...original,radius:original.radius*fit,geometry};
      const rendered=createPieceModel(piece,visual,'studio',DEFAULT_KIT,false);rendered.root.updateMatrixWorld(true);
      const profile=geometry.surface.radialProfile!,q=new THREE.Quaternion(...profile.rotation);
      const axis=new THREE.Vector3(0,1,0).applyQuaternion(q),origin=new THREE.Vector3(...profile.origin).multiplyScalar(fit).add(new THREE.Vector3(...piece.position));
      for(const fraction of [.12,.38,.62,.86,.88]) for(const angle of [.17,1.1,2.6]) {
        const radius=profile.points.at(-1)![0]*fraction;
        const next=Math.max(1,profile.points.findIndex(point=>point[0]>=radius));
        const a=profile.points[next-1],b=profile.points[next];
        const height=a[1]+(b[1]-a[1])*(radius-a[0])/(b[0]-a[0]);
        const point=new THREE.Vector3(Math.sin(angle)*radius,height,Math.cos(angle)*radius).multiplyScalar(fit).applyQuaternion(q).add(origin);
        const ray=new THREE.Raycaster(point.clone().addScaledVector(axis,.05),axis.clone().negate());
        const hit=ray.intersectObject(rendered.root.getObjectByName('contactSurface')!)[0];
        expect(hit,`${id} ${fit} ${fraction}`).toBeDefined();
        expect(hit.point.distanceTo(point),`${id} ${fit} ${fraction}`).toBeLessThan(.0003);
      }
      disposeModel(rendered.root);
    }
  });
  it('advertises surface travel only for separated semantic upper hi-hats', () => {
    const piece=DEFAULT_KIT.find(piece=>piece.id==='hihat')!;
    for(const preset of ['studio','electronic'] as const) {
      const model=createPieceModel(piece,resolvePieceVisual(preset,piece.id),preset,DEFAULT_KIT,false);
      const geometry=measureInstrumentGeometry(model.root,piece,'procedural');
      if(preset==='studio') expect(geometry.surface.motion).toEqual({kind:'translation',axis:[0,1,0],travelMeters:.025});
      else expect(geometry.surface.motion).toBeUndefined();
      disposeModel(model.root);
    }
    const imported=new THREE.Group();imported.add(new THREE.Mesh(new THREE.CylinderGeometry(.18,.18,.01),new THREE.MeshBasicMaterial()));
    expect(measureInstrumentGeometry(imported,piece,'mesh-bounds').surface.motion).toBeUndefined();disposeModel(imported);
  });
  it('rejects missing body geometry rather than manufacturing a clear proxy', () => {
    expect(() => measureInstrumentGeometry(new THREE.Group(), DEFAULT_KIT[0], 'mesh-bounds')).toThrow('measure the drum body geometry');
  });
});
