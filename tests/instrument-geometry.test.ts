import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { alignInstrumentBody, measureInstrumentGeometry } from '../src/instrument-body';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { DEFAULT_KIT } from '../src/scene';
import type { KitPiece, PieceGeometry, Vec3 } from '../src/types';

function absoluteBounds(piece: KitPiece, geometry: PieceGeometry) {
  const ratio = piece.radius / geometry.referenceRadius;
  const center = new THREE.Vector3(...geometry.body.center).multiplyScalar(ratio).add(new THREE.Vector3(...piece.position));
  const half = new THREE.Vector3(...geometry.body.halfExtents).multiplyScalar(ratio);
  return new THREE.Box3(center.clone().sub(half), center.clone().add(half));
}
function expectSameBounds(first: THREE.Box3, second: THREE.Box3) {
  expect(first.min.distanceTo(second.min)).toBeLessThan(1e-6);
  expect(first.max.distanceTo(second.max)).toBeLessThan(1e-6);
}

describe('measured equipment body descriptors', () => {
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

  it('rejects missing body geometry rather than manufacturing a clear proxy', () => {
    expect(() => measureInstrumentGeometry(new THREE.Group(), DEFAULT_KIT[0], 'mesh-bounds')).toThrow('measure the drum body geometry');
  });
});
