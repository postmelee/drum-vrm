import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { KitHardwareLayer, pieceAnchors } from '../src/kit-hardware';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { alignInstrumentBody } from '../src/instrument-body';
import { applyHitResponse, bindHitResponse } from '../src/hit-response';
import type { KitPiece } from '../src/types';

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
