import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { initSync } from '../src/wasm/drum_engine';
import { Box3, BoxGeometry, Group, Line, Mesh, MeshStandardMaterial, PerspectiveCamera, Raycaster, Texture, Vector3 } from 'three';
import { cameraPresetState, createLegacyDisplayRoot, legacyPointToDisplay, PrimitiveRigAdapter, DrumScene, DEFAULT_KIT } from '../src/scene';
import { cloneModel, createPieceModel, disposeModel, disposeModelSource, resolvePieceVisual, type KitPresetId } from '../src/model-procedural';
import { validateSelfContainedGLB } from '../src/model-import';
import { NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import { applyHitResponse, bindHitResponse, createHitResponseTrack } from '../src/hit-response';
import { pieceInstrument, type KitPiece, type Hit, type Vec3 } from '../src/types';
import { KitHardwareLayer } from '../src/kit-hardware';
import { kitHardwarePreset } from '../src/kit-layout-presets';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));

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

describe('primitive-v1 display conversion and player view', () => {
  it('shows anatomical left/right correctly from the seat without changing the canonical pose', () => {
    const state = cameraPresetState('player'), camera = new PerspectiveCamera(state.fov, 16 / 9, state.near, 40);
    camera.position.copy(state.position); camera.lookAt(state.target); camera.updateMatrixWorld(true);
    const rig = new PrimitiveRigAdapter(), display = createLegacyDisplayRoot(), input = pose(), before = Array.from(input);
    display.add(rig.root); rig.applyPose(input); display.updateMatrixWorld(true);
    for (const [name, offset, sign] of [['leftStick', 9, -1], ['rightStick', 21, 1]] as const) {
      const stick = rig.root.getObjectByName(name)!;
      const actual = stick.localToWorld(new Vector3(0, 0, stick.userData.tipLength));
      const expected = new Vector3(...legacyPointToDisplay(input.slice(offset, offset + 3)));
      expect(actual.distanceTo(expected)).toBeLessThan(1e-6);
      expect(Math.sign(actual.project(camera).x)).toBe(sign);
    }
    expect(Array.from(input)).toEqual(before);
    expect(state.minDistance).toBeLessThan(state.position.distanceTo(state.target));
    expect(state.enablePan).toBe(false); expect(state.enableRotate).toBe(false);
    expect(cameraPresetState('stage').fov).toBe(35); expect(cameraPresetState('stage').minDistance).toBe(2);
    rig.setFirstPerson(true); expect(rig.root.getObjectByName('head')!.visible).toBe(false); expect(rig.root.getObjectByName('leftStick')!.visible).toBe(true);
    rig.setFirstPerson(false); expect(rig.root.getObjectByName('head')!.visible).toBe(true);
    rig.dispose();
  });
  it('raycasts the same labeled drum at its converted contact', () => {
    const piece: KitPiece = { id: 'snare', label: 'Snare', position: [-.18, .79, .40], radius: .22 };
    const built = createPieceModel(piece, resolvePieceVisual('studio', 'snare'), 'studio', [piece]);
    const display = createLegacyDisplayRoot(); display.add(built.root); display.updateMatrixWorld(true);
    const target = new Vector3(...legacyPointToDisplay(piece.position));
    const hit = new Raycaster(target.clone().add(new Vector3(0, 1, 0)), new Vector3(0, -1, 0)).intersectObject(built.root.getObjectByName('contactSurface')!, false)[0];
    expect(hit.point.distanceTo(target)).toBeLessThan(1e-7); expect(built.root.userData.pieceId).toBe('snare');
    disposeModel(display);
  });
});

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
      const stick = rig.root.getObjectByName(name)!;
      const tip = stick.localToWorld(new Vector3(0, 0, stick.userData.tipLength));
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
    { id: 'crashRight', instrument: 'crash', label: 'Crash Right', position: [.62, 1.2, .5], radius: .3 },
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
      const origin = new Vector3(...piece.position);
      const contact = new Raycaster(origin.clone().add(new Vector3(0,1,0)),new Vector3(0,-1,0)).intersectObject(built.root.getObjectByName('contactSurface')!)[0].point;
      disposeModel(built.root);
      return { head, body, contact };
    };
    const standard = bounds('standard', 1), deep = bounds('deep', 1.4), compact = bounds('compact', .65);
    expect(deep.body.getSize(new Vector3()).y).toBeGreaterThan(standard.body.getSize(new Vector3()).y);
    expect(compact.body.getSize(new Vector3()).y).toBeLessThan(standard.body.getSize(new Vector3()).y);
    expect(deep.head.getSize(new Vector3()).x).toBeGreaterThan(standard.head.getSize(new Vector3()).x);
    for (const variant of [standard, deep, compact]) {
      // Tilted heads have a raised far rim; the actual playing contact remains
      // at the same anchor rather than the highest point of an AABB.
      expect(variant.contact.distanceTo(new Vector3(...piece.position))).toBeLessThan(1e-6);
    }
  });
  it('rejects unsafe or nonfinite editor inputs before building geometry', () => {
    expect(() => resolvePieceVisual('studio', 'snare', { scale: NaN })).toThrow('65%');
    expect(() => resolvePieceVisual('studio', 'snare', { rotation: Infinity })).toThrow('Rotation');
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
      expect(() => validateSelfContainedGLB(glb({ asset: { version: '2.0' }, images: [{ uri }] }))).toThrow('external');
    }
  });
  it('rejects incomplete and corrupted GLB containers', () => {
    expect(() => validateSelfContainedGLB(new ArrayBuffer(12))).toThrow('GLB');
    const bytes = glb({ asset: { version: '2.0' } });
    new DataView(bytes).setUint32(12, bytes.byteLength, true);
    expect(() => validateSelfContainedGLB(bytes)).toThrow('chunk');
    expect(() => validateSelfContainedGLB(glb({ asset: { version: '1.0' } }))).toThrow('2.0');
  });
});


describe('scene performance lifecycle', () => {
  it('clears stale response tracks and restores the current calibrated idle pose without replacing the drum body', () => {
    const rig = new NativeHumanoidRigAdapter();
    rig.setCalibration({ seatHeight: .60, seatForward: -.05 });
    const piece: KitPiece = { id: 'snare', label: 'Snare', position: [-.18, .79, .40], radius: .22 };
    const model = createPieceModel(piece, resolvePieceVisual('studio', piece.id), 'studio', [piece], false);
    const response = bindHitResponse(model.root, piece.position); applyHitResponse(response, { scale: 1.02, tiltX: .02, tiltZ: .01 });
    model.ring.visible = true; model.ring.material.opacity = .5;
    const visual = { ...model, id: piece.id, instrument: pieceInstrument(piece), hits: [], response, responseTrack: createHitResponseTrack([], pieceInstrument(piece)) };
    // Exercise the production lifecycle method without constructing a WebGL renderer.
    const scene = Object.create(DrumScene.prototype) as DrumScene;
    const stickTrace=new Group();
    Object.assign(scene, { rig, stickTrace, plan: { hits: [] }, songTime: 1, visuals: [visual], imported: new Map(), editorMode: false, exportMode: false, reducedMotion: { matches: false }, hitResponse: { enabled: true, strength: 1 } });
    scene.setStickTrace([[[-.1,1,.3],[-.1,1.1,.4]],[[.1,1,.3],[.1,1.1,.4]]]);
    const traceDisposals=stickTrace.children.map(child=>{
      const line=child as Line,geometry=vi.fn(),material=vi.fn();
      line.geometry.addEventListener('dispose',geometry);
      (Array.isArray(line.material)?line.material:[line.material]).forEach(value=>value.addEventListener('dispose',material));
      return {geometry,material};
    });
    scene.clearPlan();
    expect(Reflect.get(scene, 'plan')).toBeNull(); expect(Reflect.get(scene, 'songTime')).toBe(-.5);
    expect(visual.root).toBe(model.root); expect(visual.responseTrack.anchors).toEqual([]);
    expect(response.pivot.scale.toArray()).toEqual([1, 1, 1]); expect(model.ring.visible).toBe(false);
    expect(rig.getContactSamples().every(sample => sample.errorMeters < 1e-6)).toBe(true);
    expect(rig.getProfile().pelvis[1]).toBeCloseTo(.60);
    expect(stickTrace.children).toHaveLength(0);
    for(const dispose of traceDisposals){expect(dispose.geometry).toHaveBeenCalledOnce();expect(dispose.material).toHaveBeenCalledOnce();}
    rig.dispose(); disposeModel(model.root);
  });
  it('replaces and hides real world-space stick paths without leaking their geometry or materials',()=>{
    const scene=Object.create(DrumScene.prototype) as DrumScene,stickTrace=new Group();
    Object.assign(scene,{stickTrace,editorMode:false,exportMode:false});
    const paths:[Vec3[],Vec3[]]=[[[-.2,1,.3],[-.1,1.1,.4]],[[.2,1,.3],[.1,1.1,.4]]];
    scene.setStickTrace(paths);
    const originals=stickTrace.children.map(child=>{
      const line=child as Line,geometry=vi.fn(),material=vi.fn();
      line.geometry.addEventListener('dispose',geometry);
      (Array.isArray(line.material)?line.material:[line.material]).forEach(value=>value.addEventListener('dispose',material));
      return {line,geometry,material};
    });
    expect(originals[0].line.geometry.getAttribute('position').getX(0)).toBeCloseTo(-.2);
    paths[0][0][0]=-9;
    expect(originals[0].line.geometry.getAttribute('position').getX(0)).toBeCloseTo(-.2);
    Reflect.set(scene,'exportMode',true);scene.setStickTrace(paths);
    expect(stickTrace.visible).toBe(false);expect(stickTrace.children).toHaveLength(2);
    for(const original of originals){expect(original.geometry).toHaveBeenCalledOnce();expect(original.material).toHaveBeenCalledOnce();expect(original.line.parent).toBeNull();}
    const replacements=stickTrace.children.map(child=>{
      const line=child as Line,dispose=vi.fn();line.geometry.addEventListener('dispose',dispose);return dispose;
    });
    scene.setStickTrace(null);scene.setStickTrace(null);
    expect(stickTrace.children).toHaveLength(0);
    replacements.forEach(dispose=>expect(dispose).toHaveBeenCalledOnce());
  });
});


describe('project drum appearance restoration',()=>{
  it('restores accepted mount metadata and hardware after an abandoned support change', async () => {
    const hardware=new KitHardwareLayer(),kitGroup=new Group(),stage=new Group();stage.add(hardware.root,kitGroup);
    const scene=Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene,{kit:structuredClone(DEFAULT_KIT),plan:null,hardware,kitGroup,scene:stage,libraries:new Map(),
      preset:'studio',overrides:new Map(),imported:new Map(),proceduralOverrides:new Set(),pieceLoadVersions:new Map(),
      selected:null,selectionBox:null,songTime:-.5,editorMode:false,exportMode:false,reducedMotion:{matches:false},
      hitResponse:{enabled:false,strength:0},loadGeneration:0,disposed:false});
    try {
      Reflect.get(scene,'rebuildKit').call(scene);await scene.whenEquipmentReady();
      const saved=scene.getKitLayout(),appearance=saved.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).map(piece=>({id:piece.id,settings:scene.getPieceVisual(piece.id)}));
      const geometry=await scene.whenEquipmentReady(),rollback=await scene.prepareProjectAppearance('studio',appearance,new Map());
      scene.setKitHardwarePreset('suspended-floor');
      expect(kitHardwarePreset(scene.getKitLayout())).toBe('suspended-floor');
      // Restore before the candidate worker necessarily resolves: late geometry
      // must not replace the accepted hardware after rollback.
      rollback.apply(saved);await scene.whenEquipmentReady();await Promise.resolve();
      expect(scene.getKitLayout()).toEqual(saved);expect(kitHardwarePreset(scene.getKitLayout())).toBe('floor-legs');
      expect(await scene.whenEquipmentReady()).toEqual(geometry);
      const roles:string[]=[];hardware.root.traverse(node=>{if(node.userData.supportAssembly)roles.push(node.userData.supportAssembly);});
      expect(roles).toContain('floor-tom-legs');expect(roles).not.toContain('stand-mounted-tom');
      rollback.dispose();
    } finally {Reflect.set(scene,'disposed',true);hardware.dispose();disposeModel(kitGroup);}
  });

  it('rejects mismatched saved depth and restores fitted geometry with saved flat and tilted surfaces', async () => {
    const hardware = new KitHardwareLayer(), kitGroup = new Group(), stage = new Group(); stage.add(hardware.root, kitGroup);
    const scene = Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene, { kit: structuredClone(DEFAULT_KIT), plan: null, hardware, kitGroup, scene: stage, libraries: new Map(),
      preset: 'studio', overrides: new Map(), imported: new Map(), proceduralOverrides: new Set(), pieceLoadVersions: new Map(),
      selected: null, selectionBox: null, songTime: -.5, editorMode: false, exportMode: false, reducedMotion: { matches: false },
      hitResponse: { enabled: false, strength: 0 }, loadGeneration: 0, disposed: false });
    try {
      Reflect.get(scene, 'rebuildKit').call(scene);
      scene.setPieceVisual('snare', { tilt: 0 }); // Explicit flat legacy appearance, not the new preset default.
      scene.setPieceVisual('crash', { tilt: 23 });
      await scene.whenEquipmentReady();
      const saved = scene.getKitLayout(), appearance = saved.filter(piece => !['openHat', 'pedalHat'].includes(piece.id)).map(piece => ({ id: piece.id, settings: scene.getPieceVisual(piece.id) }));
      const prepared = await scene.prepareProjectAppearance('studio', appearance, new Map()), shown = kitGroup.children[0];
      const corrupted = structuredClone(saved); corrupted.find(piece => piece.id === 'snare')!.geometry!.body.halfExtents[1] += .05;
      expect(() => prepared.validateLayout(corrupted)).toThrow('saved drum geometry');
      expect(kitGroup.children[0]).toBe(shown); expect(scene.getKitLayout()).toEqual(saved);
      const fitted = structuredClone(saved);
      for (const piece of fitted) if (!['kick', 'pedalHat'].includes(piece.id)) { piece.radius *= .9; piece.position[1] -= .02; piece.position[2] -= .03; }
      expect(prepared.validateLayout(fitted)).toEqual(fitted);
      expect(kitGroup.children[0]).toBe(shown);
      prepared.apply(fitted); await scene.whenEquipmentReady();
      expect(scene.getKitLayout()).toEqual(fitted);
      expect(scene.getPieceVisual('snare').tilt).toBe(0);
      expect(scene.getPieceVisual('crash').tilt).toBe(23);
      expect(() => prepared.measureLayout(saved)).toThrow('already been applied');
      const rollback = await scene.prepareProjectAppearance('studio', appearance, new Map());
      scene.setPieceVisual('snare', { model: 'deep', scale: 1.1 }); await scene.whenEquipmentReady();
      const changed = scene.getKitLayout(), candidate = scene.measureKitLayout(saved);
      expect(candidate.find(piece => piece.id === 'snare')!.geometry!.body.halfExtents[1]).toBeGreaterThan(saved.find(piece => piece.id === 'snare')!.geometry!.body.halfExtents[1]);
      expect(scene.getKitLayout()).toEqual(changed); // preflight does not mutate display
      // An abandoned worker fit restores the old source appearance and layout.
      rollback.apply(saved); await scene.whenEquipmentReady();
      expect(scene.getKitLayout()).toEqual(saved);
      expect(scene.getPieceVisual('snare').model).toBe('standard');
    } finally { Reflect.set(scene, 'disposed', true); hardware.dispose(); disposeModel(kitGroup); }
  });

  it('preserves original library colors and only restores explicitly chosen tint overrides',async()=>{
    const piece:KitPiece={id:'snare',label:'Snare',position:[-.18,.79,.4],radius:.22};
    const source=new Group(),mesh=new Mesh(new BoxGeometry(.4,.2,.4),new MeshStandardMaterial({color:'#123456'}));
    mesh.name='AuthoredDrum';source.add(mesh);
    const hardware=new KitHardwareLayer(),kitGroup=new Group(),stage=new Group();stage.add(hardware.root,kitGroup);
    const scene=Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene,{kit:[piece],plan:null,hardware,kitGroup,scene:stage,libraries:new Map([['library',{snare:source}]]),
      preset:'library',overrides:new Map(),imported:new Map(),proceduralOverrides:new Set(),pieceLoadVersions:new Map(),
      selected:null,selectionBox:null,songTime:-.5,editorMode:false,exportMode:false,reducedMotion:{matches:false},
      hitResponse:{enabled:false,strength:0},loadGeneration:0,disposed:false});
    const color=()=>((kitGroup.getObjectByName('AuthoredDrum') as Mesh).material as MeshStandardMaterial).color.getHexString();
    try {
      Reflect.get(scene,'rebuildKit').call(scene);
      await scene.whenEquipmentReady();
      const original=scene.getPieceVisual('snare');expect(original.finishOverride).toBe(false);expect(color()).toBe('123456');
      const prepared=await scene.prepareProjectAppearance('library',[{id:'snare',settings:original}],new Map());prepared.apply();
      expect(color()).toBe('123456');expect(scene.getPieceVisual('snare').finishOverride).toBe(false);
      scene.setPieceVisual('snare',{finish:'#e0c128'});const tinted=scene.getPieceVisual('snare');expect(tinted.finishOverride).toBe(true);
      scene.setPieceVisual('snare',{finish:'#000000'});
      const restored=await scene.prepareProjectAppearance('library',[{id:'snare',settings:tinted}],new Map());restored.apply();
      expect(color()).toBe('e0c128');expect(scene.getPieceVisual('snare').finishOverride).toBe(true);
      await expect(scene.prepareProjectAppearance('library',[{id:'snare',settings:{...original,source:'imported'}}],new Map())).rejects.toThrow('Relink');
      expect(color()).toBe('e0c128');
      await scene.whenEquipmentReady();
    }finally{Reflect.set(scene,'disposed',true);hardware.dispose();disposeModel(kitGroup);disposeModel(source);}
  });
});

describe('independent physical crashes', () => {
  it('shares a library voice without coupling contacts, feedback, selection or body overrides', async () => {
    const kit: KitPiece[] = [
      { id: 'crash', instrument: 'crash', label: 'Crash Left', position: [-.6, 1.15, .5], radius: .25 },
      { id: 'crashRight', instrument: 'crash', label: 'Crash Right', position: [.6, 1.1, .5], radius: .28 },
    ];
    const hit = (pieceId: 'crash'|'crashRight', id: number, time: number): Hit => ({
      id, time, tick: id * 96, track: 0, channel: 9, note: id ? 57 : 49,
      velocity: 100, instrument: 'crash', pieceId, limb: id, target: [...kit[id].position],
    });
    // A library with one authored cymbal supplies two separately cloned bodies.
    const source = createPieceModel(kit[0], resolvePieceVisual('studio', 'crash'), 'studio', kit).root;
    const hardware = new KitHardwareLayer(), kitGroup = new Group(), stage = new Group();
    stage.add(hardware.root, kitGroup);
    const scene = Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene, {
      kit, plan: { hits: [hit('crash', 0, 1), hit('crashRight', 1, 2)] }, hardware, kitGroup, scene: stage,
      libraries: new Map([['library', { crash: source }]]), preset: 'library', overrides: new Map(), imported: new Map(),
      proceduralOverrides: new Set(), pieceLoadVersions: new Map(), selected: null, selectionBox: null,
      songTime: 1.045, editorMode: false, exportMode: false, reducedMotion: { matches: false },
      hitResponse: { enabled: true, strength: 1 },
    });
    Reflect.get(scene, 'rebuildKit').call(scene);
    await scene.whenEquipmentReady();
    const visuals = Reflect.get(scene, 'visuals') as { id: string; hits: Hit[]; responseTrack: ReturnType<typeof createHitResponseTrack>; root: Group }[];
    expect(visuals.map(v => [v.id, v.hits.map(h => h.note)])).toEqual([['crash', [49]], ['crashRight', [57]]]);
    expect(visuals.every(v => v.responseTrack.cymbal)).toBe(true);
    expect(visuals[0].root).not.toBe(visuals[1].root);
    const fixed = hardware.root.children[0];
    expect(fixed).toBeDefined();
    scene.setPieceVisual('crashRight', { finish: '#ff0000' });
    await scene.whenEquipmentReady();
    expect(scene.getPieceVisual('crashRight').finish).toBe('#ff0000');
    expect(scene.getPieceVisual('crash').finish).not.toBe('#ff0000');
    expect(hardware.root.children[0]).toBe(fixed);
    const oldBounds = scene.getKitLayout().find(piece => piece.id === 'crashRight')!.geometry!;
    const pending = scene.whenEquipmentReady();
    scene.setPieceVisual('crashRight', { scale: .8 });
    const latest = await pending;
    expect(hardware.root.userData.geometry).toEqual(latest);
    expect(hardware.root.children[0]).not.toBe(fixed);
    expect(scene.getKitLayout().find(piece => piece.id === 'crashRight')!.geometry!.body.halfExtents[0]).toBeLessThan(oldBounds.body.halfExtents[0]);
    scene.selectPiece('crashRight'); expect(Reflect.get(scene, 'selected')).toBe('crashRight');
    scene.resetPieceVisual('crashRight'); expect(scene.getPieceVisual('crashRight').finish).toBe(scene.getPieceVisual('crash').finish);
    await scene.whenEquipmentReady(); Reflect.set(scene,'disposed',true);
    hardware.dispose(); disposeModel(kitGroup); disposeModel(source);
  });
});

describe('fixed comparison views', () => {
  it('looks vertically down and along both performer sides with usable portrait framing', async () => {
    const { frameCameraState } = await import('../src/scene');
    const { validateProjectView } = await import('../src/project');
    const bounds = new Box3(new Vector3(-1.05, 0, -.25), new Vector3(1.05, 1.75, 1.5));
    for (const view of ['top', 'front', 'left', 'right'] as const) {
      const preset = cameraPresetState(view), direction = preset.target.clone().sub(preset.position).normalize();
      if (view === 'top') expect(direction.toArray()).toEqual([0, -1, 0]);
      if (view === 'left') expect(direction.toArray()).toEqual([-1, 0, 0]);
      if (view === 'right') expect(direction.toArray()).toEqual([1, 0, 0]);
      if (view !== 'front') expect(preset.enableRotate).toBe(false);
      for (const aspect of [.55, 1.8]) {
        const state = frameCameraState(preset, bounds, aspect), camera = new PerspectiveCamera(state.fov, aspect, state.near, 40);
        camera.position.copy(state.position); camera.up.copy(state.up); camera.lookAt(state.target); camera.updateMatrixWorld(true);
        for (const x of [bounds.min.x, bounds.max.x]) for (const y of [bounds.min.y, bounds.max.y]) for (const z of [bounds.min.z, bounds.max.z]) {
          const point = new Vector3(x, y, z).project(camera);
          expect(Math.abs(point.x)).toBeLessThan(1); expect(Math.abs(point.y)).toBeLessThan(1); expect(point.z).toBeLessThan(1);
        }
        expect(validateProjectView({view, position: state.position.toArray(), target: state.target.toArray(), fov: state.fov, near: state.near}).view).toBe(view);
      }
    }
  });
});

describe('accepted scene setup across score changes', () => {
  it('derives repeated automatic fit from the same baseline pedals, not the last fitted positions', () => {
    const rig=new NativeHumanoidRigAdapter(), baseline=structuredClone(DEFAULT_KIT);
    const scene=Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene,{rig,kit:baseline});
    const first=scene.getAutomaticCalibration(baseline), moved=structuredClone(baseline);
    moved.find(piece=>piece.id==='pedalHat')!.position=[-.65,.08,.8];
    moved.find(piece=>piece.id==='pedalHat')!.pedalForward=[-.6,0,.8];
    Reflect.set(scene,'kit',moved);rig.setCalibration(first,moved);
    expect(scene.getAutomaticCalibration()).not.toEqual(first);
    expect(scene.getAutomaticCalibration(baseline)).toEqual(first);
    expect(scene.getKitLayout()).toEqual(moved);
    rig.dispose();
  });
  it('binds new score tracks without rebuilding or rescaling accepted equipment during load, replan and seek', async () => {
    const { createRestRigPose } = await import('../src/rig-humanoid');
    const rig = new NativeHumanoidRigAdapter(), hardware = new KitHardwareLayer(), kitGroup = new Group(), stage = new Group(), stickTrace = new Group();
    stage.add(hardware.root, kitGroup, stickTrace);
    const scene = Object.create(DrumScene.prototype) as DrumScene;
    Object.assign(scene, {rig, hardware, kitGroup, stickTrace, scene: stage, kit: structuredClone(DEFAULT_KIT), plan: null,
      preset: 'studio', libraries: new Map(), overrides: new Map(), imported: new Map(), proceduralOverrides: new Set(), pieceLoadVersions: new Map(),
      selected: null, selectionBox: null, songTime: -.5, editorMode: false, exportMode: false, reducedMotion: {matches: false},
      hitResponse: {enabled: false, strength: 0}, loadGeneration: 0, disposed: false});
    try {
      Reflect.get(scene, 'rebuildKit').call(scene); await scene.whenEquipmentReady();
      const kit = scene.getKitLayout(), exactKit = JSON.stringify(kit), seat = rig.getCalibration();
      const objects = [...kitGroup.children], supports = [...hardware.root.children], idle = createRestRigPose(rig.getProfile());
      const rebuild = vi.fn(() => { throw new Error('Score changes must not rebuild accepted equipment'); });
      Reflect.set(scene, 'rebuildKit', rebuild);
      for (const duration of [8, 30, 3]) {
        // Rust/JSON may reorder fields or differ by one representable bit.
        const roundTrip = kit.map(piece => {
          const {radius,position,...rest}=structuredClone(piece);
          return {position,radius,...rest,instrument:pieceInstrument(piece)};
        });
        roundTrip[1].radius += Number.EPSILON;
        const hit: Hit = {id: 0, time: 1, tick: 960, track: 1, channel: 9, note: 38, velocity: 90, instrument: 'snare', pieceId: 'snare', limb: 0, target: [...kit[1].position]};
        scene.setPlan({schemaVersion: 2, duration, hits: [hit], diagnostics: [], kit: roundTrip, tempoCount: 1});
        for (const time of [duration, -.5, 1, 0]) scene.applyPose(idle, time);
        expect(JSON.stringify(scene.getKitLayout())).toBe(exactKit);
        expect(rig.getCalibration()).toEqual(seat);
        expect(kitGroup.children).toEqual(objects); expect(hardware.root.children).toEqual(supports);
        expect(Reflect.get(scene, 'visuals').find((visual: {id: string}) => visual.id === 'snare').hits).toEqual([hit]);
      }
      expect(rebuild).not.toHaveBeenCalled();
    } finally { Reflect.set(scene, 'disposed', true); hardware.dispose(); disposeModel(kitGroup); rig.dispose(); }
  });
});
