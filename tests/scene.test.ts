import { describe, expect, it, vi } from 'vitest';
import { Box3, BoxGeometry, Group, Mesh, MeshStandardMaterial, Raycaster, Texture, Vector3 } from 'three';
import { PrimitiveRigAdapter } from '../src/scene';
import { cloneModel, createPieceModel, disposeModel, disposeModelSource, resolvePieceVisual, type KitPresetId } from '../src/model-procedural';
import { validateSelfContainedGLB } from '../src/model-import';
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

describe('editable procedural models preserve playing contacts', () => {
  const pieces: KitPiece[] = [
    { id: 'kick', label: 'Kick', position: [.2, .08, .62], radius: .26 },
    { id: 'snare', label: 'Snare', position: [-.18, .79, .40], radius: .22 },
    { id: 'hihat', label: 'Hi-hat', position: [-.48, .96, .44], radius: .22 },
    { id: 'pedalHat', label: 'Pedal', position: [-.22, .08, .55], radius: .12 },
    { id: 'tomHigh', label: 'High tom', position: [-.26, 1, .72], radius: .18 },
    { id: 'ride', label: 'Ride', position: [.66, 1.15, .67], radius: .28 },
  ];
  it('keeps the real surface at every canonical target across all procedural presets and width limits', () => {
    for (const preset of ['studio', 'vintage', 'electronic'] as KitPresetId[]) {
      for (const scale of [.65, 1, 1.4]) for (const piece of pieces.filter(piece => piece.id !== 'pedalHat')) {
        const model = createPieceModel(piece, resolvePieceVisual(preset, piece.id, { scale }), preset, pieces);
        model.root.updateMatrixWorld(true);
        const surface = model.root.getObjectByName('contactSurface')!;
        expect(surface, `${preset} ${piece.id}`).toBeDefined();
        const ray = new Raycaster(new Vector3(piece.position[0], piece.position[1] + 1, piece.position[2]), new Vector3(0, -1, 0));
        const contact = ray.intersectObject(surface, false)[0];
        expect(contact, `${preset} ${piece.id} ×${scale}`).toBeDefined();
        expect(contact.point.y).toBeCloseTo(piece.position[1], 5);
        if (piece.id === 'hihat') {
          const pedal = pieces.find(piece => piece.id === 'pedalHat')!;
          const pedalRay = new Raycaster(new Vector3(pedal.position[0], 1, pedal.position[2]), new Vector3(0, -1, 0));
          expect(pedalRay.intersectObject(model.root.getObjectByName('pedalContactSurface')!, false)[0].point.y).toBeCloseTo(pedal.position[1], 6);
        }
        disposeModel(model.root);
      }
    }
  });
  it('changes drum depth and width without translating its playing head', () => {
    const piece = pieces[1];
    const bounds = (model: 'standard' | 'deep' | 'compact', scale: number) => {
      const built = createPieceModel(piece, resolvePieceVisual('studio', piece.id, { model, scale }), 'studio', pieces);
      built.root.updateMatrixWorld(true);
      const head = new Box3().setFromObject(built.root.getObjectByName('contactSurface')!);
      const body = new Box3().setFromObject(built.root.getObjectByName('contactSurface')!.parent!);
      disposeModel(built.root);
      return { head, body };
    };
    const standard = bounds('standard', 1), deep = bounds('deep', 1.4), compact = bounds('compact', .65);
    expect(deep.body.getSize(new Vector3()).y).toBeGreaterThan(standard.body.getSize(new Vector3()).y);
    expect(compact.body.getSize(new Vector3()).y).toBeLessThan(standard.body.getSize(new Vector3()).y);
    expect(deep.head.getSize(new Vector3()).x).toBeGreaterThan(standard.head.getSize(new Vector3()).x);
    for (const variant of [standard, deep, compact]) {
      expect(variant.head.max.y).toBeCloseTo(piece.position[1], 6);
      expect(variant.head.getCenter(new Vector3()).x).toBeCloseTo(piece.position[0], 6);
      expect(variant.head.getCenter(new Vector3()).z).toBeCloseTo(piece.position[2], 6);
    }
  });
  it('rejects unsafe or nonfinite editor inputs before building geometry', () => {
    expect(() => resolvePieceVisual('studio', 'snare', { scale: NaN })).toThrow('65%');
    expect(() => resolvePieceVisual('studio', 'snare', { rotation: Infinity })).toThrow('회전');
    expect(() => resolvePieceVisual('studio', 'snare', { finish: 'url(https://example.com)' })).toThrow('#RRGGBB');
  });
});

function glb(json: object): ArrayBuffer {
  const raw = new TextEncoder().encode(JSON.stringify(json));
  const length = Math.ceil(raw.length / 4) * 4;
  const buffer = new ArrayBuffer(20 + length), view = new DataView(buffer);
  view.setUint32(0, 0x46546c67, true); view.setUint32(4, 2, true); view.setUint32(8, buffer.byteLength, true);
  view.setUint32(12, length, true); view.setUint32(16, 0x4e4f534a, true);
  new Uint8Array(buffer, 20).fill(32); new Uint8Array(buffer, 20).set(raw);
  return buffer;
}
describe('local model privacy validation', () => {
  it('keeps shared decoded images alive during visual rebuild and closes them once when the source is discarded', () => {
    const bitmap = { close: vi.fn(), width: 1, height: 1 };
    const texture = new Texture(bitmap);
    const root = new Group();
    root.add(new Mesh(new BoxGeometry(), new MeshStandardMaterial({ map: texture })));
    root.add(new Mesh(new BoxGeometry(), new MeshStandardMaterial({ map: texture })));
    const rendered = cloneModel(root);
    disposeModel(rendered);
    expect(bitmap.close).not.toHaveBeenCalled();
    disposeModelSource(root);
    expect(bitmap.close).toHaveBeenCalledTimes(1);
  });
  it('accepts embedded GLB resources and blocks external references before any loader request', () => {
    expect(() => validateSelfContainedGLB(glb({ asset: { version: '2.0' }, buffers: [{ byteLength: 12 }] }))).not.toThrow();
    expect(() => validateSelfContainedGLB(glb({ asset: { version: '2.0' }, images: [{ uri: 'data:image/png;base64,AAAA' }] }))).not.toThrow();
    for (const uri of ['https://example.com/image.png', '//example.com/image.png', '../image.png', 'file:///private/image.png', 'blob:outside', 'data:text/html;base64,AAAA']) {
      expect(() => validateSelfContainedGLB(glb({ asset: { version: '2.0' }, images: [{ uri }] }))).toThrow('외부');
    }
  });
  it('rejects incomplete and corrupted GLB containers', () => {
    expect(() => validateSelfContainedGLB(new ArrayBuffer(12))).toThrow('GLB');
    const bytes = glb({ asset: { version: '2.0' } });
    new DataView(bytes).setUint32(12, bytes.byteLength, true);
    expect(() => validateSelfContainedGLB(bytes)).toThrow('청크');
    expect(() => validateSelfContainedGLB(glb({ asset: { version: '1.0' } }))).toThrow('2.0');
  });
});
