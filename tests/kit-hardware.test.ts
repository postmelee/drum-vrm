import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { initSync } from '../src/wasm/drum_engine';
import { KitHardwareLayer, pieceAnchors, solveKitGeometry } from '../src/kit-hardware';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { alignInstrumentBody, measureInstrumentGeometry } from '../src/instrument-body';
import { applyHitResponse, bindHitResponse } from '../src/hit-response';
import type { KitPiece } from '../src/types';
import { DEFAULT_KIT } from '../src/scene';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));
function measuredKit(preset: 'studio' | 'vintage' | 'electronic' = 'studio'): KitPiece[] {
  return structuredClone(DEFAULT_KIT).map(piece => {
    if (['openHat', 'pedalHat'].includes(piece.id)) return piece;
    const model = createPieceModel(piece, resolvePieceVisual(preset, piece.id), preset, DEFAULT_KIT, false);
    piece.geometry = measureInstrumentGeometry(model.root, piece, 'procedural'); disposeModel(model.root); return piece;
  });
}
function snapshot(root: THREE.Object3D) {
  root.updateWorldMatrix(true, true);
  const values: { node: THREE.Object3D; matrix: number[] }[] = [];
  root.traverse(node => values.push({ node, matrix: [...node.matrixWorld.elements] })); return values;
}

describe('shared numeric hardware rendering', () => {
  it('renders every Rust support capsule and pedal box with the same physical dimensions', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    hardware.setGeometry(kit, geometry); hardware.root.updateMatrixWorld(true);
    const supports: THREE.Mesh[] = [], pedals: THREE.Mesh[] = [];
    hardware.root.traverse(node => {
      if (node instanceof THREE.Mesh && node.userData.segment) supports.push(node);
      if (node instanceof THREE.Mesh && node.userData.box) pedals.push(node);
    });
    expect(supports.length).toBe(geometry.supports.length); expect(pedals.length).toBe(geometry.pedals.length);
    for (const mesh of supports) {
      const segment = mesh.userData.segment, a = new THREE.Vector3(...segment.start), b = new THREE.Vector3(...segment.end);
      expect(mesh.getWorldPosition(new THREE.Vector3()).distanceTo(a.clone().add(b).multiplyScalar(.5))).toBeLessThan(1e-9);
      const extent = new THREE.Vector3(segment.radius, segment.radius, segment.radius);
      const allowed = new THREE.Box3(a.clone().min(b).sub(extent), a.clone().max(b).add(extent)).expandByScalar(1e-7);
      const actual = new THREE.Box3().setFromObject(mesh, true);
      expect(allowed.containsBox(actual)).toBe(true);
      if (a.distanceTo(b) > 1e-8) expect(new THREE.Vector3(0, 1, 0).applyQuaternion(mesh.quaternion).distanceTo(b.sub(a).normalize())).toBeLessThan(1e-7);
    }
    for (const mesh of pedals) {
      const descriptor = mesh.userData.box, bounds = new THREE.Box3().setFromObject(mesh);
      expect(bounds.getCenter(new THREE.Vector3()).distanceTo(new THREE.Vector3(...descriptor.center))).toBeLessThan(1e-8);
      bounds.getSize(new THREE.Vector3()).toArray().forEach((value, axis) => expect(value / 2).toBeCloseTo(descriptor.halfExtents[axis], 7));
    }
    hardware.dispose();
  });

  it('retains hardware identity during finish/hit feedback, but refreshes after a body depth change', () => {
    const kit = measuredKit(), hardware = new KitHardwareLayer(), stage = new THREE.Group();
    stage.scale.x = -1; stage.add(hardware.root); hardware.setLayout(kit);
    const before = snapshot(hardware.root);
    const snare = kit.find(piece => piece.id === 'snare')!;
    const model = createPieceModel(snare, resolvePieceVisual('studio', snare.id, { finish: '#abcdef' }), 'studio', kit, false);
    stage.add(model.root); const binding = bindHitResponse(model.root, snare.position);
    applyHitResponse(binding, { scale: 1.02, tiltX: -.03, tiltZ: .01 });
    expect(hardware.setLayout(structuredClone(kit))).toBe(false);
    snapshot(hardware.root).forEach((value, index) => { expect(value.node).toBe(before[index].node); expect(value.matrix).toEqual(before[index].matrix); });
    const next = measuredKit('vintage'); expect(hardware.setLayout(next)).toBe(true);
    expect(snapshot(hardware.root)[1].node).not.toBe(before[1].node);
    hardware.dispose(); disposeModel(model.root);
  });

  it('keeps independent pedal contacts at their planned positions', () => {
    const kit = measuredKit(), hardware = new KitHardwareLayer();
    kit.find(piece => piece.id === 'kick')!.position = [.3, .10, .70];
    kit.find(piece => piece.id === 'pedalHat')!.position = [-.30, .09, .60];
    hardware.setLayout(kit); hardware.root.updateMatrixWorld(true);
    for (const [id, name] of [['kick', 'contactSurface'], ['pedalHat', 'pedalContactSurface']] as const) {
      const target = new THREE.Vector3(...kit.find(piece => piece.id === id)!.position), board = hardware.root.getObjectByName(name)!;
      const ray = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
      expect(ray.intersectObject(board)[0].point.distanceTo(target)).toBeLessThan(1e-7);
    }
    hardware.dispose();
  });

  it('retains unresolved equipment explicitly instead of claiming a blocked pull rod is clear', () => {
    const kit = measuredKit(), hat = kit.find(piece => piece.id === 'hihat')!, tom = kit.find(piece => piece.id === 'tomHigh')!;
    tom.position = [...pieceAnchors(hat).mount];
    const geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer(); hardware.setGeometry(kit, geometry);
    expect(geometry.unresolvedPieceIds).toContain('hihat');
    expect(hardware.root.getObjectByName('Hardware/hihat')!.userData.unresolved).toBe(true);
    expect(hardware.root.userData.geometry).toEqual(geometry); hardware.dispose();
  });
});

describe('body contacts remain independent of shared hardware', () => {
  it('keeps each procedural cymbal contact at the strike point across supported sizes', () => {
    const piece = DEFAULT_KIT.find(value => value.id === 'hihat')!;
    for (const preset of ['studio', 'vintage', 'electronic'] as const) for (const scale of [.65, 1, 1.4]) {
      const body = createPieceModel(piece, resolvePieceVisual(preset, piece.id, { scale }), preset, DEFAULT_KIT, false);
      body.root.updateMatrixWorld(true);
      const ray = new THREE.Raycaster(new THREE.Vector3(...piece.position).add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
      expect(ray.intersectObject(body.root.getObjectByName('contactSurface')!)[0].point.y).toBeCloseTo(piece.position[1], 6);
      disposeModel(body.root);
    }
  });

  it('aligns an imported body while measuring its actual transformed depth', () => {
    const piece = DEFAULT_KIT.find(value => value.id === 'snare')!;
    const body = new THREE.Group(); body.add(new THREE.Mesh(new THREE.BoxGeometry(3, 2, 3), new THREE.MeshBasicMaterial()));
    const pivot = new THREE.Group(); pivot.add(body);
    alignInstrumentBody(body, pivot, piece, resolvePieceVisual('studio', 'snare', { scale: 1.2, rotation: 45 }));
    const geometry = measureInstrumentGeometry(pivot, piece, 'mesh-bounds');
    const bounds = new THREE.Box3().setFromObject(pivot);
    expect(geometry.body.halfExtents[1] * 2).toBeCloseTo(bounds.max.y - bounds.min.y, 7);
    expect(geometry.body.halfExtents[1] * 2).toBeGreaterThan(.135);
    const target = new THREE.Vector3(...piece.position), ray = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
    expect(ray.intersectObject(body, true)[0].point.distanceTo(target)).toBeLessThan(1e-7);
    expect(pivot.userData.approximateContact).toBe(false); disposeModel(pivot);
  });
});
