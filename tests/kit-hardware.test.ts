import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { initSync } from '../src/wasm/drum_engine';
import { KitHardwareLayer, pieceAnchors, solveKitGeometry } from '../src/kit-hardware';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { alignInstrumentBody, measureInstrumentGeometry } from '../src/instrument-body';
import { applyHitResponse, bindHitResponse } from '../src/hit-response';
import type { KitPiece } from '../src/types';
import { createHardwarePart } from '../src/hardware-parts';
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
  it('renders a connected suspended floor tom from the shared mount graph without hidden legs', () => {
    const kit = structuredClone(DEFAULT_KIT).filter(piece=>piece.id==='tomLow'||piece.id==='ride');
    const floor=kit.find(piece=>piece.id==='tomLow')!, ride=kit.find(piece=>piece.id==='ride')!;
    floor.position=[.50,.76,.55];floor.radius=.1651;floor.mount={kind:'standClamp',supportPieceId:'ride'};delete floor.standRotation;
    ride.position=[.72,1.22,.68];
    const geometry=solveKitGeometry(kit), layer=new KitHardwareLayer();
    expect(geometry.unresolvedPieceIds).toEqual([]);
    layer.setGeometry(kit,geometry);
    const root=layer.root.getObjectByName('Hardware/tomLow')!;
    expect(root.userData.supportAssembly).toBe('stand-mounted-tom');
    const segments=geometry.supports.filter(segment=>segment.pieceId==='tomLow');
    expect(segments.filter(segment=>segment.role==='standClamp')).toHaveLength(1);
    expect(segments.filter(segment=>segment.role==='tomArm')).toHaveLength(2);
    expect(segments.some(segment=>segment.role==='floorLeg'||segment.role==='rubberFoot')).toBe(false);
    for(const segment of segments) expect(root.children.some(child=>child.userData.segment?.role===segment.role)).toBe(true);
    layer.dispose();
  });
  it('routes the measured Studio kit and connects its snare basket to the mast', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    hardware.setGeometry(kit, geometry);
    const snare = hardware.root.getObjectByName('Hardware/snare')!;
    const segments = snare.children.flatMap(node => node.userData.segment ? [node.userData.segment] : []);
    const mast = segments.find(segment => segment.role === 'tubeUpper');
    const neck = segments.find(segment => segment.role === 'basketNeck');
    const basket = segments.filter(segment => segment.role === 'basketArm');
    const grips = segments.filter(segment => segment.role === 'basketGrip');
    expect(mast).toBeDefined(); expect(neck).toBeDefined(); expect(basket).toHaveLength(3);
    expect(grips).toHaveLength(3);
    expect(neck.start).toEqual(mast.end);
    expect(new THREE.Vector3(...neck.start).distanceTo(new THREE.Vector3(...neck.end))).toBeLessThanOrEqual(.10);
    const normal = new THREE.Vector3(...kit.find(piece => piece.id === 'snare')!.geometry!.surface.normal);
    for (const [index, arm] of basket.entries()) {
      expect(arm.start).toEqual(neck.end);
      const grip = grips[index], start = new THREE.Vector3(...grip.start), end = new THREE.Vector3(...grip.end);
      expect(end.clone().sub(start).normalize().distanceTo(normal)).toBeLessThan(1e-7);
      expect(new THREE.Line3(start, end).closestPointToPoint(new THREE.Vector3(...arm.end), true, new THREE.Vector3()).distanceTo(new THREE.Vector3(...arm.end))).toBeLessThan(1e-7);
    }
    hardware.dispose();
    expect(geometry.unresolvedPieceIds).toEqual([]);
  });

  it('connects both player-facing rack toms to one T holder through bounded horizontal and vertical L arms', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit);
    expect(geometry.unresolvedPieceIds).toEqual([]);
    const posts = geometry.supports.filter(part => part.role === 'tomPost');
    const yokes = geometry.supports.filter(part => part.role === 'tomYoke');
    const balls = geometry.supports.filter(part => part.role === 'tomBall');
    expect(posts).toHaveLength(1); expect(yokes).toHaveLength(1); expect(balls).toHaveLength(2);
    expect(posts[0].end[0]).toBe(posts[0].start[0]); expect(posts[0].end[2]).toBe(posts[0].start[2]);
    expect(yokes[0].end[1]).toBe(yokes[0].start[1]); expect(yokes[0].end[2]).toBe(yokes[0].start[2]);
    for (const id of ['tomHigh', 'tomMid']) {
      const piece = kit.find(value => value.id === id)!;
      const normal = new THREE.Vector3(...piece.geometry!.surface.normal);
      expect(normal.z).toBeLessThan(-.2);
      const arms = geometry.supports.filter(part => part.pieceId === id && part.role === 'tomArm');
      expect(arms).toHaveLength(2);
      expect(balls.some(ball => ball.start.every((value, axis) => Math.abs(value - arms[0].start[axis]) < 1e-8))).toBe(true);
      expect(arms[0].end[1]).toBe(arms[0].start[1]);
      expect(new THREE.Vector3(...arms[0].end).distanceTo(new THREE.Vector3(...arms[0].start))).toBeLessThanOrEqual(.38);
      expect(arms[1].start).toEqual(arms[0].end);
      expect(arms[1].end[0]).toBe(arms[1].start[0]); expect(arms[1].end[2]).toBe(arms[1].start[2]);
      const bracket = geometry.supports.find(part => part.pieceId === id && part.role === 'suspensionBracket')!;
      const body = geometry.bodies.find(value => value.pieceId === id)!;
      const radial = new THREE.Vector3(...bracket.start).sub(new THREE.Vector3(...body.center));
      expect(Math.abs(radial.dot(normal))).toBeLessThan(1e-7);
      expect(radial.length()).toBeCloseTo(piece.geometry!.surface.radius, 7);
    }
  });

  it('aligns native tilted cymbal spindles with the measured radial axis, not the bow normal', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    hardware.setGeometry(kit, geometry); hardware.root.updateMatrixWorld(true);
    for (const id of ['crash', 'crashRight', 'ride']) {
      const piece = kit.find(value => value.id === id)!, profile = piece.geometry!.surface.radialProfile!;
      expect(profile).toBeDefined();
      const axis = new THREE.Vector3(0, 1, 0).applyQuaternion(new THREE.Quaternion(...profile.rotation));
      expect(axis.distanceTo(new THREE.Vector3(0, 1, 0))).toBeGreaterThan(.1);
      expect(axis.distanceTo(new THREE.Vector3(...piece.geometry!.surface.normal))).toBeGreaterThan(.01);
      const sleeve = geometry.supports.find(segment => segment.pieceId === id && segment.role === 'cymbalSleeve')!;
      expect(sleeve).toBeDefined();
      const start = new THREE.Vector3(...sleeve.start), end = new THREE.Vector3(...sleeve.end);
      expect(end.clone().sub(start).normalize().distanceTo(axis)).toBeLessThan(1e-7);
      const ratio = piece.radius / piece.geometry!.referenceRadius;
      const mount = new THREE.Vector3(...profile.origin).multiplyScalar(ratio).add(new THREE.Vector3(...piece.position));
      expect(new THREE.Line3(start, end).closestPointToPoint(mount, true, new THREE.Vector3()).distanceTo(mount)).toBeLessThan(1e-7);
      const rendered = hardware.root.getObjectByName(`Hardware/${id}`)!.children.find(node => node.userData.segment?.role === 'cymbalSleeve')!;
      expect(new THREE.Vector3(0, 1, 0).applyQuaternion(rendered.getWorldQuaternion(new THREE.Quaternion())).distanceTo(axis)).toBeLessThan(1e-7);
    }
    hardware.dispose();
  });

  it('renders every Rust support inside its shared collision capsule and keeps pedal dimensions', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    hardware.setGeometry(kit, geometry); hardware.root.updateMatrixWorld(true);
    const supports: THREE.Object3D[] = [], pedals: THREE.Mesh[] = [];
    hardware.root.traverse(node => {
      if (node.userData.segment) supports.push(node);
      if (node instanceof THREE.Mesh && node.userData.box) pedals.push(node);
    });
    expect(supports.length).toBe(geometry.supports.length); expect(pedals.length).toBe(geometry.pedals.length);
    for (const mesh of supports) {
      const segment = mesh.userData.segment, a = new THREE.Vector3(...segment.start), b = new THREE.Vector3(...segment.end);
      expect(mesh.getWorldPosition(new THREE.Vector3()).distanceTo(a.clone().add(b).multiplyScalar(.5))).toBeLessThan(1e-9);
      const extent = new THREE.Vector3(segment.radius, segment.radius, segment.radius);
      const allowed = new THREE.Box3(a.clone().min(b).sub(extent), a.clone().max(b).add(extent)).expandByScalar(1e-7);
      const saved = mesh.userData.renderBounds;
      const actual = saved ? new THREE.Box3(new THREE.Vector3(...saved.min), new THREE.Vector3(...saved.max)) : new THREE.Box3().setFromObject(mesh, true);
      expect(allowed.containsBox(actual)).toBe(true);
      if (a.distanceTo(b) > 1e-8) expect(new THREE.Vector3(0, 1, 0).applyQuaternion(mesh.quaternion).distanceTo(b.sub(a).normalize())).toBeLessThan(1e-7);
    }
    for (const mesh of pedals) {
      const descriptor = mesh.userData.box, bounds = new THREE.Box3().setFromObject(mesh);
      expect(bounds.getCenter(new THREE.Vector3()).distanceTo(new THREE.Vector3(...descriptor.center))).toBeLessThan(1e-8);
      mesh.geometry.computeBoundingBox();
      mesh.geometry.boundingBox!.getSize(new THREE.Vector3()).toArray().forEach((value, axis) => expect(value / 2).toBeCloseTo(descriptor.halfExtents[axis], 7));
      const expectedRotation=new THREE.Quaternion(...(descriptor.rotation??[0,0,0,1]));
      expect(mesh.getWorldQuaternion(new THREE.Quaternion()).angleTo(expectedRotation)).toBeLessThan(1e-7);
      const expectedBounds=new THREE.Box3();
      for(const x of [-1,1])for(const y of [-1,1])for(const z of [-1,1])expectedBounds.expandByPoint(new THREE.Vector3(x*descriptor.halfExtents[0],y*descriptor.halfExtents[1],z*descriptor.halfExtents[2]).applyQuaternion(expectedRotation).add(new THREE.Vector3(...descriptor.center)));
      expect(bounds.min.distanceTo(expectedBounds.min)).toBeLessThan(1e-8);
      expect(bounds.max.distanceTo(expectedBounds.max)).toBeLessThan(1e-8);
    }
    hardware.dispose();
  });

  it('batches fixed support detail while keeping mechanisms independently movable', () => {
    const kit = measuredKit(), geometry = solveKitGeometry(kit), hardware = new KitHardwareLayer();
    hardware.setGeometry(kit, geometry);
    hardware.root.traverse(node => {
      if (!node.name.startsWith('Hardware/')) return;
      const batch = node.children.filter(child => child.name === 'FixedHardwareBatch');
      expect(batch.length).toBeLessThanOrEqual(3);
      const fixed = node.children.filter(child => child.userData.renderBounds);
      if (fixed.length) {
        const source = new THREE.Box3();
        for (const part of fixed) source.union(new THREE.Box3(new THREE.Vector3(...part.userData.renderBounds.min), new THREE.Vector3(...part.userData.renderBounds.max)));
        const merged = new THREE.Box3();
        batch.forEach(mesh => merged.union(new THREE.Box3().setFromObject(mesh, true)));
        expect(source.min.distanceTo(merged.min)).toBeLessThan(1e-6);
        expect(source.max.distanceTo(merged.max)).toBeLessThan(1e-6);
      }
    });
    const beater = hardware.root.getObjectByName('KickBeaterPivot')!;
    expect(beater.children.length).toBeGreaterThan(0);
    expect(beater.children.every(child => !child.userData.renderBounds)).toBe(true);
    hardware.dispose();
  });

  it('keeps every mechanical detail inside its capsule, including joints and double braces', () => {
    const material = new THREE.MeshBasicMaterial();
    const roles = ['tubeLower', 'tubeMiddle', 'tubeUpper', 'collar', 'memoryLock', 'tripodLeg', 'tripodBrace', 'rubberFoot', 'boom', 'boomTail', 'boomPivot', 'cymbalTilter', 'cymbalSleeve', 'tomPost', 'tomSocket', 'tomYoke', 'tomBall', 'mountBall', 'standClamp', 'standLink', 'tomArm', 'tomClamp', 'basketNeck', 'basketTilter', 'basketArm', 'basketGrip', 'suspensionBracket', 'floorLeg', 'cymbalFelt', 'pedalLink', 'rod'];
    for (const role of roles) for (const length of [0, .025, .35]) {
      const part = createHardwarePart({pieceId: 'crash', role, start: [0, 0, 0], end: [0, length, 0], radius: .025}, {chrome: material, dark: material, rubber: material});
      part.updateMatrixWorld(true);
      part.traverse(node => {
        if (!(node instanceof THREE.Mesh)) return;
        const vertices = node.geometry.getAttribute('position');
        for (let i = 0; i < vertices.count; i++) {
          const point = new THREE.Vector3().fromBufferAttribute(vertices, i).applyMatrix4(node.matrixWorld);
          const closest = new THREE.Vector3(0, THREE.MathUtils.clamp(point.y, 0, length), 0);
          expect(point.distanceTo(closest), `${role}/${length}`).toBeLessThanOrEqual(.02500001);
        }
        node.geometry.dispose();
      });
    }
    material.dispose();
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
    const rotation = new THREE.Matrix4().makeRotationFromQuaternion(new THREE.Quaternion(...geometry.body.rotation ?? [0, 0, 0, 1])).elements;
    const worldHeight = 2 * (Math.abs(rotation[1]) * geometry.body.halfExtents[0] + Math.abs(rotation[5]) * geometry.body.halfExtents[1] + Math.abs(rotation[9]) * geometry.body.halfExtents[2]);
    expect(worldHeight).toBeCloseTo(bounds.max.y - bounds.min.y, 7);
    expect(geometry.body.halfExtents[1] * 2).toBeGreaterThan(.135);
    const target = new THREE.Vector3(...piece.position), ray = new THREE.Raycaster(target.clone().add(new THREE.Vector3(0, 1, 0)), new THREE.Vector3(0, -1, 0));
    expect(ray.intersectObject(body, true)[0].point.distanceTo(target)).toBeLessThan(1e-7);
    expect(pivot.userData.approximateContact).toBe(false); disposeModel(pivot);
  });
});
