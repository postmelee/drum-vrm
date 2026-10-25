import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { KitHardwareLayer, pieceAnchors, pieceBodyEnvelope, cymbalStandRoute, segmentClearsBody } from '../src/kit-hardware';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { alignInstrumentBody } from '../src/instrument-body';
import { applyHitResponse, bindHitResponse } from '../src/hit-response';
import type { KitPiece } from '../src/types';
import { DEFAULT_KIT } from '../src/scene';

const kit: KitPiece[] = [
  { id: 'kick', label: 'Kick', position: [.2, .08, .62], radius: .26 },
  { id: 'snare', label: 'Snare', position: [-.18, .79, .4], radius: .22 },
  { id: 'hihat', label: 'Hi-hat', position: [-.48, .96, .44], radius: .22 },
  { id: 'pedalHat', label: 'Hi-hat pedal', position: [-.22, .08, .55], radius: .12 },
  { id: 'tomHigh', label: 'High tom', position: [-.26, 1, .72], radius: .18 },
];
function snapshot(root: THREE.Object3D) {
  root.updateMatrixWorld(true);
  const values: { node: THREE.Object3D; matrix: number[] }[] = [];
  root.traverse(node => values.push({ node, matrix: [...node.matrixWorld.elements] })); return values;
}
describe('shared hardware and interchangeable bodies', () => {
  it('retains hardware identities and world matrices through every procedural body/size/rotation swap and hit response', () => {
    const hardware = new KitHardwareLayer(), stage = new THREE.Group(), bodies = new THREE.Group();
    stage.scale.x = -1; stage.add(hardware.root, bodies); expect(hardware.setLayout(kit)).toBe(true);
    stage.updateMatrixWorld(true); const initial = snapshot(hardware.root);
    for (const preset of ['studio', 'vintage', 'electronic'] as const) for (const scale of [.65, 1.4]) {
      expect(hardware.setLayout(structuredClone(kit))).toBe(false);
      for (const piece of kit.filter(value => value.id !== 'pedalHat')) {
        const model = createPieceModel(piece, resolvePieceVisual(preset, piece.id, { scale, rotation: 60 }), preset, kit, false);
        bodies.add(model.root);
        expect(model.root.getObjectByName('pedalContactSurface')).toBeUndefined();
        expect(model.root.getObjectByName('Hardware/' + piece.id)).toBeUndefined();
        const binding = bindHitResponse(model.root, piece.position);
        applyHitResponse(binding, { scale: 1.02, tiltX: -.03, tiltZ: .01 });
      }
      stage.updateMatrixWorld(true);
      const current = snapshot(hardware.root); expect(current.length).toBe(initial.length);
      current.forEach((value, index) => { expect(value.node).toBe(initial[index].node); expect(value.matrix).toEqual(initial[index].matrix); });
      disposeModel(bodies);
    }
    hardware.dispose();
  });
  it('moves shared pedal contacts only on an explicit layout edit, independent of body sizes', () => {
    const hardware = new KitHardwareLayer(); hardware.setLayout(kit);
    const moved = structuredClone(kit); moved[0].position = [.3, .10, .70]; moved[3].position = [-.30, .09, .60];
    expect(hardware.setLayout(moved)).toBe(true); hardware.root.updateMatrixWorld(true);
    for (const [index, name] of [[0, 'contactSurface'], [3, 'pedalContactSurface']] as const) {
      const target = new THREE.Vector3(...moved[index].position), board = hardware.root.getObjectByName(name)!;
      const ray = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
      expect(ray.intersectObject(board)[0].point.distanceTo(target)).toBeLessThan(1e-7);
    }
    const hihat = hardware.root.getObjectByName('Hardware/hihat')!;
    expect(hihat.getObjectByName('mountAnchor')!.position.toArray()).toEqual(pieceAnchors(moved[2]).mount);
    hardware.dispose();
  });
  it('keeps cymbal mounts fixed in X/Z while fitting each supported procedural size to the strike point', () => {
    const piece = kit[2];
    for (const preset of ['studio', 'vintage', 'electronic'] as const) for (const scale of [.65, 1, 1.4]) {
      const body = createPieceModel(piece, resolvePieceVisual(preset, piece.id, { scale }), preset, kit, false);
      expect(body.cymbal!.position.x).toBe(pieceAnchors(piece).mount[0]);
      expect(body.cymbal!.position.z).toBe(pieceAnchors(piece).mount[2]);
      body.root.updateMatrixWorld(true);
      const ray = new THREE.Raycaster(new THREE.Vector3(...piece.position).add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
      expect(ray.intersectObject(body.root.getObjectByName('contactSurface')!)[0].point.y).toBeCloseTo(piece.position[1], 6);
      disposeModel(body.root);
    }
  });
  it('fits a body-only local model without changing the independent pedal, and flags missing contact geometry', () => {
    const hardware = new KitHardwareLayer(); hardware.setLayout(kit); const before = snapshot(hardware.root);
    const body = new THREE.Group(); body.add(new THREE.Mesh(new THREE.BoxGeometry(3, 1, 3), new THREE.MeshBasicMaterial()));
    const pivot = new THREE.Group(); pivot.add(body);
    alignInstrumentBody(body, pivot, kit[1], resolvePieceVisual('studio', 'snare', { scale: 1.2, rotation: 45 }));
    const target = new THREE.Vector3(...kit[1].position);
    const ray = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
    expect(ray.intersectObject(body, true)[0].point.distanceTo(target)).toBeLessThan(1e-7);
    expect(pivot.userData.approximateContact).toBe(false);
    snapshot(hardware.root).forEach((value, index) => expect(value.matrix).toEqual(before[index].matrix));
    hardware.dispose(); disposeModel(pivot);
  });
});

describe('physical baseline and cymbal support routing', () => {
  it('has separated nominal body volumes in the rendered default layout', () => {
    const bodies = DEFAULT_KIT.flatMap(piece => { const envelope = pieceBodyEnvelope(piece); return envelope ? [{ id: piece.id, ...envelope }] : []; });
    for (const [index, a] of bodies.entries()) for (const b of bodies.slice(index + 1)) {
      const verticalGap = Math.max(a.minY, b.minY) - Math.min(a.maxY, b.maxY);
      const horizontalGap = Math.hypot(a.center[0] - b.center[0], a.center[2] - b.center[2]) - a.radius - b.radius;
      expect(Math.max(verticalGap, horizontalGap), `${a.id}/${b.id} nominal clearance`).toBeGreaterThan(.005);
    }
    const diameters = Object.fromEntries(DEFAULT_KIT.map(piece => [piece.id, 2 * piece.radius / .0254]));
    expect(diameters.snare).toBeCloseTo(14); expect(diameters.hihat).toBeCloseTo(14);
    expect(diameters.kick).toBeCloseTo(20);
  });

  it('routes the ride mast outside the floor tom while retaining its cymbal mount and target', () => {
    const ride = DEFAULT_KIT.find(piece => piece.id === 'ride')!, floor = DEFAULT_KIT.find(piece => piece.id === 'tomLow')!;
    const mount = pieceAnchors(ride).mount, body = pieceBodyEnvelope(floor)!;
    expect(segmentClearsBody([mount[0], .025, mount[2]], [mount[0], mount[1] + .065, mount[2]], body)).toBe(false);
    const route = cymbalStandRoute(ride, DEFAULT_KIT);
    expect(route.clear).toBe(true); expect(route.base[0]).not.toBe(mount[0]);
    expect(route.mount).toEqual([mount[0], mount[1] + .065, mount[2]]);
    expect(segmentClearsBody(route.base, route.boom, body)).toBe(true);
    expect(segmentClearsBody(route.boom, route.mount, body)).toBe(true);
    const hardware = new KitHardwareLayer(); hardware.setLayout(DEFAULT_KIT);
    expect(hardware.root.getObjectByName('Hardware/ride')!.getObjectByName('cymbalBoom')).toBeDefined();
    hardware.dispose();
  });

  it('checks full mast/boom segments and reports an impossible straight hi-hat support', () => {
    const obstacle = { center: [0, .5, 0] as [number, number, number], radius: .2, minY: .4, maxY: .6 };
    expect(segmentClearsBody([-.4, .5, 0], [.4, .5, 0], obstacle)).toBe(false);
    expect(segmentClearsBody([-.4, .7, 0], [.4, .7, 0], obstacle)).toBe(true);
    const hat = DEFAULT_KIT.find(piece => piece.id === 'hihat')!;
    const obstructed = [...DEFAULT_KIT, { id: 'tomHigh', label: 'Obstruction', position: pieceAnchors(hat).mount, radius: .2 } satisfies KitPiece];
    const route = cymbalStandRoute(hat, obstructed);
    expect(route.clear).toBe(false);
    expect(route.base[0]).toBe(hat.position[0]); // Never quietly move a pull-rod away from its cymbal.
  });

  it('uses physical kick diameter for both procedural and imported bodies', () => {
    const kick = DEFAULT_KIT.find(piece => piece.id === 'kick')!;
    const model = createPieceModel(kick, resolvePieceVisual('studio', kick.id), 'studio', DEFAULT_KIT, false);
    model.root.updateMatrixWorld(true);
    const shell = model.root.children.find(node => node.userData.hitResponseRole === 'shell')!;
    const bounds = new THREE.Box3().setFromObject(shell);
    expect(bounds.getSize(new THREE.Vector3()).y).toBeCloseTo(2 * (kick.radius + .017), 5);
    expect(bounds.min.y).toBeGreaterThanOrEqual(0);
    const imported = new THREE.Group(); imported.add(new THREE.Mesh(new THREE.BoxGeometry(1, 1, .7), new THREE.MeshBasicMaterial()));
    const pivot = new THREE.Group(); pivot.add(imported);
    alignInstrumentBody(imported, pivot, kick, resolvePieceVisual('studio', kick.id));
    expect(new THREE.Box3().setFromObject(pivot).getSize(new THREE.Vector3()).y).toBeCloseTo(2 * kick.radius);
    disposeModel(model.root); disposeModel(pivot);
  });
});
