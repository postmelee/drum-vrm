import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { Hit, Instrument, KitPiece, PerformancePlan, Vec3 } from './types';
import { canonicalPieceId, cloneModel, createPieceModel, disposeModel, disposeModelSource, resolvePieceVisual, type KitPresetId, type PieceVisualOptions, type PieceVisualSettings, type PieceModel } from './model-procedural';
import { loadLocalGLB } from './model-import';
export { KIT_PRESETS, type KitPresetId, type PieceVisualOptions, type PieceVisualSettings } from './model-procedural';

/** All coordinates are metres; +Y is up and the performer faces +Z. */
export interface RigCalibration {
  id: string;
  coordinateSystem: '+Y-up/+Z-forward';
  poseFloats: 57;
  stickLength: number;
  targetPositions: Partial<Record<Instrument, Vec3>>;
}
export interface RigAdapter {
  readonly root: THREE.Group;
  describeRig(): { id: string; joints: readonly string[]; poseFloats: number };
  calibrate(kit: readonly KitPiece[]): RigCalibration;
  applyPose(pose: Float32Array): void;
  setVisualOptions(options: { bodyColor?: string }): void;
  dispose(): void;
}

const UP = new THREE.Vector3(0, 1, 0);
const POINT = new THREE.Vector3();
const END = new THREE.Vector3();
const DELTA = new THREE.Vector3();
const MID = new THREE.Vector3();
const DEFAULT_KIT: KitPiece[] = [
  { id: 'kick', label: 'Kick', position: [.20, .08, .62], radius: .26 },
  { id: 'snare', label: 'Snare', position: [-.18, .79, .40], radius: .22 },
  { id: 'hihat', label: 'Hi-hat', position: [-.48, .96, .44], radius: .22 },
  { id: 'openHat', label: 'Open hi-hat', position: [-.48, .96, .44], radius: .22 },
  { id: 'pedalHat', label: 'Hi-hat pedal', position: [-.22, .08, .55], radius: .12 },
  { id: 'tomHigh', label: 'High tom', position: [-.26, 1, .72], radius: .18 },
  { id: 'tomMid', label: 'Mid tom', position: [.18, .98, .74], radius: .20 },
  { id: 'tomLow', label: 'Floor tom', position: [.62, .80, .42], radius: .23 },
  { id: 'crash', label: 'Crash', position: [-.66, 1.26, .68], radius: .30 },
  { id: 'ride', label: 'Ride', position: [.66, 1.15, .67], radius: .28 },
];

function material(color: THREE.ColorRepresentation, metalness = 0, roughness = .55): THREE.MeshStandardMaterial {
  return new THREE.MeshStandardMaterial({ color, metalness, roughness });
}
function mesh(geometry: THREE.BufferGeometry, mat: THREE.Material, parent?: THREE.Object3D): THREE.Mesh {
  const result = new THREE.Mesh(geometry, mat);
  result.castShadow = true;
  result.receiveShadow = true;
  parent?.add(result);
  return result;
}
function cylinder(radius: number, length: number, mat: THREE.Material, parent: THREE.Object3D, topRadius = radius, segments = 32): THREE.Mesh {
  return mesh(new THREE.CylinderGeometry(topRadius, radius, length, segments), mat, parent);
}
function sphere(radius: number, mat: THREE.Material, parent: THREE.Object3D): THREE.Mesh {
  return mesh(new THREE.SphereGeometry(radius, 20, 14), mat, parent);
}
function connect(node: THREE.Object3D, a: THREE.Vector3, b: THREE.Vector3): void {
  DELTA.subVectors(b, a);
  const length = DELTA.length();
  node.position.copy(a).addScaledVector(DELTA, .5);
  node.scale.y = Math.max(length, .0001);
  node.quaternion.setFromUnitVectors(UP, DELTA.normalize());
}
function rod(parent: THREE.Object3D, a: Vec3, b: Vec3, radius: number, mat: THREE.Material): THREE.Mesh {
  const node = cylinder(radius, 1, mat, parent, radius, 12);
  connect(node, new THREE.Vector3(...a), new THREE.Vector3(...b));
  return node;
}
/** The only class mapping canonical packed joints onto a particular visual rig. */
export class PrimitiveRigAdapter implements RigAdapter {
  readonly root = new THREE.Group();
  private readonly boneMaterial = new THREE.MeshBasicMaterial({ color: 0x5c626b });
  private readonly jointMaterial = new THREE.MeshBasicMaterial({ color: 0x357cdb });
  private readonly limbs: { upper: THREE.Mesh; lower: THREE.Mesh; joint: THREE.Mesh; rootJoint: THREE.Mesh; end: THREE.Mesh; effector: THREE.Mesh }[] = [];
  private readonly torso: THREE.Mesh;
  private readonly hips: THREE.Mesh;
  private readonly neck: THREE.Mesh;
  private readonly head = new THREE.Group();
  private readonly chest: THREE.Mesh;
  private readonly shoulderLinks: THREE.Mesh[] = [];
  private readonly hipLinks: THREE.Mesh[] = [];

  constructor() {
    this.root.name = 'SkeletonRigAdapter';
    this.torso = cylinder(.012, 1, this.boneMaterial, this.root, .012, 10);
    this.hips = sphere(.026, this.jointMaterial, this.root);
    this.chest = sphere(.026, this.jointMaterial, this.root);
    this.neck = cylinder(.010, 1, this.boneMaterial, this.root, .010, 10);
    this.head.name = 'head'; this.root.add(this.head);
    const headMarker = mesh(new THREE.SphereGeometry(.075, 12, 8), new THREE.MeshBasicMaterial({ color: 0x5c626b, wireframe: true }), this.head);
    headMarker.name = 'headMarker';
    for (let side = 0; side < 2; side++) {
      this.shoulderLinks.push(cylinder(.010, 1, this.boneMaterial, this.root, .010, 10));
      this.hipLinks.push(cylinder(.010, 1, this.boneMaterial, this.root, .010, 10));
    }
    const stickMaterial = new THREE.MeshBasicMaterial({ color: 0xb48642 });
    for (let index = 0; index < 4; index++) {
      const arm = index < 2;
      const upper = cylinder(.012, 1, this.boneMaterial, this.root, .012, 10);
      const lower = cylinder(.012, 1, this.boneMaterial, this.root, .012, 10);
      const joint = sphere(.021, this.jointMaterial, this.root);
      const rootJoint = sphere(.022, this.jointMaterial, this.root);
      const end = sphere(.019, this.jointMaterial, this.root);
      const effector = cylinder(arm ? .006 : .012, 1, arm ? stickMaterial : this.boneMaterial, this.root, arm ? .004 : .012, 10);
      effector.name = ['leftStick', 'rightStick', 'leftFoot', 'rightFoot'][index];
      this.limbs.push({ upper, lower, joint, rootJoint, end, effector });
    }
    this.applyPose(defaultPose());
  }
  describeRig() {
    return {
      id: 'primitive-skeleton-v1', poseFloats: 57,
      joints: ['leftShoulder', 'leftElbow', 'leftWrist', 'leftStickTip', 'rightShoulder', 'rightElbow', 'rightWrist', 'rightStickTip', 'leftHip', 'leftKnee', 'leftAnkle', 'leftToe', 'rightHip', 'rightKnee', 'rightAnkle', 'rightToe', 'pelvis', 'chest', 'head'] as const,
    };
  }
  calibrate(kit: readonly KitPiece[]): RigCalibration {
    const targetPositions: Partial<Record<Instrument, Vec3>> = {};
    kit.forEach(piece => {
      if (piece.position.length !== 3 || !piece.position.every(Number.isFinite) || !Number.isFinite(piece.radius) || piece.radius <= 0) throw new Error(`Invalid rig target: ${piece.id}`);
      targetPositions[piece.id] = [...piece.position];
    });
    return { id: 'primitive-v1', coordinateSystem: '+Y-up/+Z-forward', poseFloats: 57, stickLength: Math.hypot(.09, .30), targetPositions };
  }
  applyPose(pose: Float32Array): void {
    if (pose.length !== 57 || !pose.every(Number.isFinite)) throw new Error('Rig pose must contain 57 finite world-space coordinates.');
    this.limbs.forEach((limb, index) => {
      const offset = index * 12;
      POINT.fromArray(pose, offset); END.fromArray(pose, offset + 3);
      limb.rootJoint.position.copy(POINT); limb.joint.position.copy(END); connect(limb.upper, POINT, END);
      POINT.copy(END); END.fromArray(pose, offset + 6); connect(limb.lower, POINT, END); limb.end.position.copy(END);
      POINT.copy(END); END.fromArray(pose, offset + 9);
      if (index < 2) POINT.addScaledVector(MID.subVectors(POINT, END), .17);
      connect(limb.effector, POINT, END);
    });
    POINT.fromArray(pose, 48); END.fromArray(pose, 51);
    this.hips.position.copy(POINT); this.chest.position.copy(END); connect(this.torso, POINT, END);
    POINT.copy(END); END.fromArray(pose, 54); connect(this.neck, POINT, END); this.head.position.copy(END);
    for (let side = 0; side < 2; side++) {
      POINT.fromArray(pose, 51); END.fromArray(pose, side * 12); connect(this.shoulderLinks[side], POINT, END);
      POINT.fromArray(pose, 48); END.fromArray(pose, 24 + side * 12); connect(this.hipLinks[side], POINT, END);
    }
  }
  setVisualOptions(options: { bodyColor?: string }): void {
    if (options.bodyColor) {
      this.boneMaterial.color.set(options.bodyColor);
      (this.head.getObjectByName('headMarker') as THREE.Mesh<THREE.SphereGeometry, THREE.MeshBasicMaterial>).material.color.set(options.bodyColor);
    }
  }
  dispose(): void { disposeModel(this.root); }
}

function defaultPose(): Float32Array {
  return new Float32Array([
    -.22, 1.35, -.10, -.43, 1.10, .08, -.28, 1.15, .10, -.28, 1.06, .40,
    .22, 1.35, -.10, .43, 1.10, .08, .28, 1.15, .10, .28, 1.06, .40,
    -.14, .71, -.15, -.22, .45, .20, -.22, .105, .42, -.22, .105, .55,
    .14, .71, -.15, .22, .45, .23, .20, .105, .49, .20, .105, .62,
    0, .71, -.15, 0, 1.24, -.10, 0, 1.62, -.10,
  ]);
}

type KitVisual = PieceModel & { id: Instrument; hits: Hit[]; };
type CameraState = { position: THREE.Vector3; target: THREE.Vector3 };
type CameraTransition = { from: CameraState; to: CameraState; started: number };
type ImportedModel = { root: THREE.Group; name: string; approximate: boolean };

export class DrumScene {
  readonly canvas: HTMLCanvasElement;
  readonly renderer: THREE.WebGLRenderer;
  readonly rig: RigAdapter;
  onPieceSelect: ((id: Instrument | null) => void) | null = null;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(35, 1, .05, 40);
  private readonly controls: OrbitControls;
  private readonly kitGroup = new THREE.Group();
  private readonly grid = new THREE.GridHelper(5, 20, 0xb9bec5, 0xd9dde2);
  private readonly raycaster = new THREE.Raycaster();
  private readonly resizeObserver: ResizeObserver;
  private visuals: KitVisual[] = [];
  private plan: PerformancePlan | null = null;
  private kit: readonly KitPiece[] = DEFAULT_KIT;
  private preset: KitPresetId = 'studio';
  private readonly overrides = new Map<Instrument, PieceVisualOptions>();
  private readonly imported = new Map<Instrument, ImportedModel>();
  private readonly proceduralOverrides = new Set<Instrument>();
  private readonly pieceLoadVersions = new Map<Instrument, number>();
  private library: Partial<Record<Instrument, THREE.Group>> | null = null;
  private libraryPromise: Promise<Partial<Record<Instrument, THREE.Group>>> | null = null;
  private loadGeneration = 0;
  private disposed = false;
  private selected: Instrument | null = null;
  private selectionBox: THREE.BoxHelper | null = null;
  private editorMode = false;
  private characterVisible = true;
  private exportMode = false;
  private savedCamera: CameraState | null = null;
  private transition: CameraTransition | null = null;
  private pointerStart: { id: number; x: number; y: number; dragged: boolean } | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, preserveDrawingBuffer: true, powerPreference: 'high-performance' });
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x8c929a, 2.2));
    const key = new THREE.DirectionalLight(0xffffff, 3.0); key.position.set(-3, 5, 4); this.scene.add(key);
    const fill = new THREE.DirectionalLight(0xffffff, 1.5); fill.position.set(4, 2, -2); this.scene.add(fill);
    const gridMaterial = this.grid.material as THREE.LineBasicMaterial;
    gridMaterial.transparent = true; gridMaterial.opacity = .17;
    this.grid.position.set(0, -.02, .4); this.grid.visible = false; this.scene.add(this.grid);
    this.scene.add(this.kitGroup);
    this.rig = new PrimitiveRigAdapter(); this.scene.add(this.rig.root);
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = false;
    this.controls.minDistance = 2; this.controls.maxDistance = 8;
    this.controls.maxPolarAngle = Math.PI * .49;
    this.controls.enablePan = true;
    this.setCamera('stage'); this.setTheme('light'); this.rebuildKit();
    canvas.addEventListener('pointerdown', this.pointerDown);
    canvas.addEventListener('pointermove', this.pointerMove);
    canvas.addEventListener('pointerup', this.pointerUp);
    canvas.addEventListener('pointercancel', this.pointerCancel);
    this.resizeObserver = new ResizeObserver(() => { if (!this.exportMode) this.resize(); });
    this.resizeObserver.observe(canvas); this.resize();
  }

  setPlan(plan: PerformancePlan): void {
    this.rig.calibrate(plan.kit); this.plan = plan; this.kit = plan.kit; this.rebuildKit();
  }
  setCharacterVisible(visible: boolean): void {
    this.characterVisible = visible; this.rig.root.visible = visible && !this.editorMode;
  }
  setTheme(theme: 'light' | 'dark'): void {
    this.setBackground(theme === 'light' ? '#ffffff' : '#000000');
    this.rig.setVisualOptions({ bodyColor: theme === 'light' ? '#5c626b' : '#c3c9d1' });
    (this.grid.material as THREE.LineBasicMaterial).color.set(theme === 'light' ? 0x8b949e : 0x7d8691);
  }
  setGridVisible(visible: boolean): void { this.grid.visible = visible; }
  setEditorMode(enabled: boolean): void {
    if (this.editorMode === enabled || this.exportMode) return;
    this.updateCameraTransition();
    this.editorMode = enabled;
    this.controls.enableRotate = !enabled;
    this.controls.maxPolarAngle = enabled ? Math.PI : Math.PI * .49;
    this.rig.root.visible = this.characterVisible && !enabled;
    if (enabled) {
      this.savedCamera = this.cameraState();
      this.moveCamera({ position: new THREE.Vector3(0, 1.4, 4.1), target: new THREE.Vector3(0, .76, .50) });
    } else if (this.savedCamera) {
      this.moveCamera(this.savedCamera); this.savedCamera = null;
    }
    this.updateSelection();
  }
  async setKitPreset(preset: KitPresetId): Promise<void> {
    if (!['studio', 'vintage', 'electronic', 'library'].includes(preset)) throw new Error('지원하지 않는 드럼 키트입니다.');
    const generation = ++this.loadGeneration;
    if (preset === 'library') {
      this.libraryPromise ??= import('./model-library').then(module => module.loadLibraryKit()).then(library => {
        if (this.disposed) this.disposeLibrary(library); else this.library = library;
        return library;
      });
      try { await this.libraryPromise; }
      catch (error) { this.libraryPromise = null; throw error; }
      if (this.disposed || generation !== this.loadGeneration) return;
    }
    this.preset = preset; this.overrides.clear(); this.proceduralOverrides.clear();
    const discarded = [...this.imported.values()]; this.imported.clear(); this.rebuildKit();
    discarded.forEach(model => disposeModelSource(model.root));
  }
  getPieceVisual(id: Instrument): PieceVisualSettings {
    const canonical = canonicalPieceId(id);
    const visual = resolvePieceVisual(this.preset, canonical, this.overrides.get(canonical));
    const imported = this.imported.get(canonical);
    return { ...visual, source: imported ? 'imported' : this.preset === 'library' && !this.proceduralOverrides.has(canonical) ? 'library' : 'procedural', ...(imported ? { customModel: imported.name, approximateCalibration: imported.approximate } : {}) };
  }
  setPieceVisual(id: Instrument, options: PieceVisualOptions): void {
    const canonical = canonicalPieceId(id);
    const updated = { ...this.overrides.get(canonical), ...options };
    resolvePieceVisual(this.preset, canonical, updated);
    let discarded: ImportedModel | undefined;
    if (options.model !== undefined) {
      this.invalidatePieceLoad(canonical);
      discarded = this.imported.get(canonical);
      this.imported.delete(canonical); this.proceduralOverrides.add(canonical);
    }
    this.overrides.set(canonical, updated); this.rebuildKit();
    if (discarded) disposeModelSource(discarded.root);
  }
  resetPieceVisual(id: Instrument): void {
    const canonical = canonicalPieceId(id);
    this.invalidatePieceLoad(canonical);
    const imported = this.imported.get(canonical);
    this.imported.delete(canonical); this.overrides.delete(canonical); this.proceduralOverrides.delete(canonical);
    this.rebuildKit();
    if (imported) disposeModelSource(imported.root);
  }
  async loadPieceModel(id: Instrument, file: File): Promise<{ fileName: string; approximateCalibration: true }> {
    const canonical = canonicalPieceId(id), generation = this.loadGeneration;
    const version = this.invalidatePieceLoad(canonical);
    const root = await loadLocalGLB(file);
    if (this.disposed || generation !== this.loadGeneration || this.pieceLoadVersions.get(canonical) !== version) { disposeModelSource(root); throw new Error('모델을 불러오는 동안 키트가 변경되었습니다. 다시 선택해 주세요.'); }
    const previous = this.imported.get(canonical);
    this.proceduralOverrides.delete(canonical);
    this.imported.set(canonical, { root, name: file.name, approximate: true }); this.rebuildKit();
    if (previous) disposeModelSource(previous.root);
    return { fileName: file.name, approximateCalibration: true };
  }
  selectPiece(id: Instrument | null): void {
    const canonical = id === null ? null : canonicalPieceId(id);
    if (canonical && !this.visuals.some(visual => visual.id === canonical)) return;
    this.selected = canonical; this.updateSelection(); this.onPieceSelect?.(canonical);
  }
  applyPose(pose: Float32Array, songTime: number): void {
    this.rig.applyPose(pose);
    for (const visual of this.visuals) {
      let low = 0, high = visual.hits.length;
      while (low < high) { const mid = (low + high) >>> 1; if (visual.hits[mid].time <= songTime) low = mid + 1; else high = mid; }
      const hit = visual.hits[low - 1], age = hit ? songTime - hit.time : Infinity;
      const envelope = age >= 0 && age < .35 ? Math.exp(-age * 12) * (hit!.velocity / 127) : 0;
      visual.ring.material.opacity = this.editorMode && this.imported.has(visual.id) ? .7 : envelope * .7;
      visual.ring.visible = !this.exportMode && this.editorMode && this.imported.has(visual.id) || envelope > .002;
      if (visual.cymbal) {
        const motion = age >= 0 && age < .8 ? Math.sin(age * 46) * Math.exp(-age * 7) * .025 * (hit!.velocity / 127) : 0;
        visual.cymbal.rotation.x = this.editorMode ? 0 : motion;
        visual.cymbal.rotation.z = this.editorMode ? 0 : motion * .65;
      }
    }
  }
  render(): void {
    if (!this.exportMode) { this.updateCameraTransition(); this.controls.update(); }
    this.renderer.render(this.scene, this.camera);
  }
  resize(width?: number, height?: number): void {
    const rect = this.canvas.getBoundingClientRect();
    const size = (value: number) => Number.isFinite(value) ? Math.max(1, Math.round(value)) : 1;
    const w = size(width ?? rect.width), h = size(height ?? rect.height);
    this.renderer.setPixelRatio(this.exportMode ? 1 : Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.setSize(w, h, false); this.camera.aspect = w / h; this.camera.updateProjectionMatrix();
  }
  setCamera(preset: 'front' | 'stage' | 'top'): void {
    if (this.editorMode) return;
    this.transition = null;
    const position = { front: [0, 1.95, 4.7], stage: [2.75, 2.4, 3.75], top: [.001, 4.8, .5] }[preset];
    this.camera.position.set(...position as Vec3);
    this.controls.target.set(0, preset === 'top' ? .5 : .72, .32);
    this.controls.enabled = !this.exportMode; this.controls.update();
  }
  setBackground(color: string): void { this.scene.background = new THREE.Color(color); }
  setExportMode(enabled: boolean): void {
    if (enabled) this.finishCameraTransition();
    this.exportMode = enabled; this.controls.enabled = !enabled && !this.transition;
    this.pointerStart = null; this.updateSelection(); if (!enabled) this.resize();
  }
  dispose(): void {
    this.disposed = true; ++this.loadGeneration; this.resizeObserver.disconnect(); this.controls.dispose();
    this.canvas.removeEventListener('pointerdown', this.pointerDown); this.canvas.removeEventListener('pointermove', this.pointerMove);
    this.canvas.removeEventListener('pointerup', this.pointerUp); this.canvas.removeEventListener('pointercancel', this.pointerCancel);
    this.clearSelection(); disposeModel(this.scene);
    this.imported.forEach(model => disposeModelSource(model.root)); this.imported.clear();
    if (this.library) this.disposeLibrary(this.library);
    this.renderer.dispose();
  }

  private cameraState(): CameraState { return { position: this.camera.position.clone(), target: this.controls.target.clone() }; }
  private invalidatePieceLoad(id: Instrument): number {
    const version = (this.pieceLoadVersions.get(id) ?? 0) + 1; this.pieceLoadVersions.set(id, version); return version;
  }
  private disposeLibrary(library: Partial<Record<Instrument, THREE.Group>>): void {
    const root = new THREE.Group(); Object.values(library).forEach(group => { if (group) root.add(group); }); disposeModelSource(root);
  }
  private moveCamera(to: CameraState): void {
    this.transition = { from: this.cameraState(), to: { position: to.position.clone(), target: to.target.clone() }, started: performance.now() };
    this.controls.enabled = false;
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) this.finishCameraTransition();
  }
  private updateCameraTransition(): void {
    if (!this.transition) return;
    const t = Math.min(1, (performance.now() - this.transition.started) / 350), eased = t * t * (3 - 2 * t);
    this.camera.position.lerpVectors(this.transition.from.position, this.transition.to.position, eased);
    this.controls.target.lerpVectors(this.transition.from.target, this.transition.to.target, eased);
    if (t === 1) { this.transition = null; this.controls.enabled = !this.exportMode; }
  }
  private finishCameraTransition(): void {
    if (this.transition) { this.camera.position.copy(this.transition.to.position); this.controls.target.copy(this.transition.to.target); this.transition = null; }
    this.controls.update(); this.controls.enabled = !this.exportMode;
  }
  private clearSelection(): void {
    if (this.selectionBox) { this.scene.remove(this.selectionBox); this.selectionBox.geometry.dispose(); this.selectionBox.material.dispose(); this.selectionBox = null; }
  }
  private updateSelection(): void {
    this.clearSelection();
    if (!this.editorMode || this.exportMode || !this.selected) return;
    const model = this.visuals.find(visual => visual.id === this.selected); if (!model) return;
    model.root.updateMatrixWorld(true);
    this.selectionBox = new THREE.BoxHelper(model.root, 0x388bfd);
    this.selectionBox.material.depthTest = false; this.selectionBox.material.transparent = true; this.selectionBox.material.opacity = .8;
    this.selectionBox.renderOrder = 10; this.scene.add(this.selectionBox);
  }
  private pointerDown = (event: PointerEvent) => {
    if (event.button === 0 && this.editorMode && !this.exportMode && !this.transition) this.pointerStart = { id: event.pointerId, x: event.clientX, y: event.clientY, dragged: false };
  };
  private pointerMove = (event: PointerEvent) => {
    if (this.pointerStart && Math.hypot(event.clientX - this.pointerStart.x, event.clientY - this.pointerStart.y) > 5) this.pointerStart.dragged = true;
  };
  private pointerCancel = () => { this.pointerStart = null; };
  private pointerUp = (event: PointerEvent) => {
    const start = this.pointerStart; this.pointerStart = null;
    if (!start || start.id !== event.pointerId || start.dragged || !this.editorMode || this.exportMode || this.transition) return;
    const rect = this.canvas.getBoundingClientRect(); if (!rect.width || !rect.height) return;
    this.raycaster.setFromCamera(new THREE.Vector2((event.clientX - rect.left) / rect.width * 2 - 1, -(event.clientY - rect.top) / rect.height * 2 + 1), this.camera);
    this.kitGroup.updateMatrixWorld(true);
    const hit = this.raycaster.intersectObjects(this.visuals.map(value => value.root), true)[0];
    let object: THREE.Object3D | null = hit?.object ?? null;
    while (object && !object.userData.pieceId) object = object.parent;
    this.selectPiece(object?.userData.pieceId as Instrument ?? null);
  };
  private rebuildKit(): void {
    this.clearSelection(); disposeModel(this.kitGroup); this.visuals = [];
    for (const piece of this.kit) {
      if (canonicalPieceId(piece.id) !== piece.id) continue;
      const settings = this.getPieceVisual(piece.id);
      const built = createPieceModel(piece, settings, this.preset, this.kit);
      const source = this.imported.get(piece.id)?.root ?? (this.preset === 'library' && !this.proceduralOverrides.has(piece.id) ? this.library?.[piece.id] : undefined);
      if (source) {
        built.ring.removeFromParent(); disposeModel(built.root); built.cymbal = undefined;
        const display = cloneModel(source), pivot = new THREE.Group(); pivot.add(display); built.root.add(pivot, built.ring);
        const custom = this.imported.has(piece.id);
        if (custom) {
          display.rotation.y = THREE.MathUtils.degToRad(settings.rotation);
          display.updateMatrixWorld(true);
          const box = new THREE.Box3().setFromObject(display), size = box.getSize(new THREE.Vector3()), center = box.getCenter(new THREE.Vector3());
          const fit = piece.radius * 2 / Math.max(size.x, size.z, .0001) * settings.scale;
          pivot.scale.setScalar(fit);
          pivot.position.set(piece.position[0] - center.x * fit, piece.position[1] - box.max.y * fit, piece.position[2] - center.z * fit);
          if (piece.id === 'kick') pivot.position.y = .32 - center.y * fit;
        } else {
          this.alignLibraryPiece(display, pivot, source, piece, settings);
          if (piece.id === 'hihat' && this.library?.pedalHat) {
            const pedal = this.kit.find(item => item.id === 'pedalHat');
            if (pedal) {
              const pedalDisplay = cloneModel(this.library.pedalHat), pedalPivot = new THREE.Group();
              pedalPivot.add(pedalDisplay); built.root.add(pedalPivot);
              this.alignLibraryPiece(pedalDisplay, pedalPivot, this.library.pedalHat, pedal, settings);
            }
          }
        }
        if (this.overrides.get(piece.id)?.finish) display.traverse(node => {
          if (node instanceof THREE.Mesh) for (const material of Array.isArray(node.material) ? node.material : [node.material]) if (material instanceof THREE.MeshStandardMaterial) material.color.set(settings.finish);
        });
      } else if (settings.rotation) {
        // Rotate around the canonical contact, never around the scene origin.
        const pivot = new THREE.Group(); pivot.position.set(...piece.position); pivot.rotation.y = THREE.MathUtils.degToRad(settings.rotation);
        const model = new THREE.Group(); model.position.copy(pivot.position).multiplyScalar(-1);
        [...built.root.children].forEach(child => { if (!child.userData.fixedContact) model.add(child); }); pivot.add(model); built.root.add(pivot);
      }
      built.root.userData.pieceId = piece.id; this.kitGroup.add(built.root);
      const instruments: Instrument[] = piece.id === 'hihat' ? ['hihat', 'openHat', 'pedalHat'] : [piece.id];
      this.visuals.push({ ...built, id: piece.id, hits: this.plan?.hits.filter(hit => instruments.includes(hit.instrument)).sort((a, b) => a.time - b.time) ?? [] });
    }
    this.updateSelection();
  }
  private alignLibraryPiece(display: THREE.Group, pivot: THREE.Group, source: THREE.Group, piece: KitPiece, settings: PieceVisualSettings): void {
    const contact = new THREE.Vector3(...source.userData.contact as Vec3);
    display.position.copy(contact).multiplyScalar(-1);
    pivot.position.set(...piece.position);
    // Preserve source proportions; contact stays exact even if stand feet vary in height.
    pivot.scale.setScalar(settings.scale);
    pivot.rotation.y = THREE.MathUtils.degToRad(settings.rotation);
  }
}
