import { msg, raw } from './i18n';
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { hitPieceId, pieceInstrument, type Hit, type Instrument, type PieceId, type KitPiece, type KitGeometry, type PerformancePlan, type RigProfile, type Vec3 } from './types';
import { createRestRigPose, NativeHumanoidRigAdapter, type HumanoidRigAdapter, type RigContactSample } from './rig-humanoid';
import { validateKitLayout } from './kit-calibration';
import { canonicalPieceId, disposeModel, disposeModelSource, resolvePieceVisual, type KitPresetId, type PieceVisualOptions, type PieceVisualSettings, type PieceModel } from './model-procedural';
import { loadLocalGLB } from './model-import';
import { KitHardwareLayer, solveKitGeometry } from './kit-hardware';
import { createMeasuredInstrumentBody } from './instrument-body';
import { initializeWasm } from './wasm-runtime';
import { applyHitResponse, bindHitResponse, createHitResponseTrack, evaluateHitResponse, validateHitResponse, type HitResponseBinding, type HitResponseSettings, type HitResponseTrack } from './hit-response';
export { KIT_PRESETS, type KitPresetId, type PieceVisualOptions, type PieceVisualSettings } from './model-procedural';
export type { HitResponseSettings } from './hit-response';

export type CameraView = 'front' | 'stage' | 'top' | 'player';
export interface SceneViewState { view:CameraView;position:Vec3;target:Vec3;fov:number;near:number; }
/** primitive-v1 labels have LH at -X. Display +Z-facing anatomy requires LH at +X.
 * This is a renderer-only v1 compatibility basis, never a plan/assignment mutation.
 * Future VRM adapters must convert their numeric pose basis, not reflect VRM bones. */
export const legacyPointToDisplay = (point: ArrayLike<number>): Vec3 => [-point[0], point[1], point[2]];
export const legacyDirectionToDisplay = legacyPointToDisplay;
export function createLegacyDisplayRoot(): THREE.Group {
  const root = new THREE.Group(); root.name = 'PrimitiveV1DisplayBasis'; root.scale.x = -1;
  root.userData.coordinateConversion = 'primitive-v1-to-display-v1:reflect-x'; return root;
}
export interface CameraState {
  position: THREE.Vector3; target: THREE.Vector3; view: CameraView;
  fov: number; near: number; minDistance: number; maxDistance: number; maxPolarAngle: number;
  enableRotate: boolean; enablePan: boolean; enableZoom: boolean;
}
export function cameraPresetState(view: CameraView): CameraState {
  const player = view === 'player';
  const positions: Record<CameraView, Vec3> = { front: [0, 1.95, 4.7], stage: [2.75, 2.4, 3.75], top: [.001, 4.8, .5], player: [0, 1.55, -.30] };
  return {
    view, position: new THREE.Vector3(...positions[view]), target: new THREE.Vector3(0, player ? .90 : view === 'top' ? .5 : .72, player ? .70 : .32),
    fov: player ? 65 : 35, near: player ? .025 : .05, minDistance: player ? .2 : 2, maxDistance: player ? 2 : 8,
    maxPolarAngle: player ? Math.PI : Math.PI * .49, enableRotate: !player, enablePan: !player, enableZoom: !player,
  };
}
const isLibraryPreset = (preset: KitPresetId): preset is 'library' | 'batoski' | 'smoj' => ['library', 'batoski', 'smoj'].includes(preset);

/** All coordinates are metres; +Y is up and the performer faces +Z. */
export interface RigCalibration {
  id: string;
  coordinateSystem: '+Y-up/+Z-forward';
  poseFloats: 57 | 73 | 77;
  stickLength: number;
  targetPositions: Partial<Record<PieceId, Vec3>>;
}
export interface RigAdapter {
  readonly root: THREE.Group;
  describeRig(): { id: string; joints: readonly string[]; poseFloats: number };
  calibrate(kit: readonly KitPiece[]): RigCalibration;
  applyPose(pose: Float32Array): void;
  setVisualOptions(options: { bodyColor?: string }): void;
  setFirstPerson?(enabled: boolean): void;
  dispose(): void;
}

const UP = new THREE.Vector3(0, 1, 0);
const POINT = new THREE.Vector3();
const END = new THREE.Vector3();
const DELTA = new THREE.Vector3();
const MID = new THREE.Vector3();
export const DEFAULT_KIT: KitPiece[] = [
  { id: 'kick', label: 'Kick', position: [.20, .08, .62], radius: .254 },
  { id: 'snare', label: 'Snare', position: [-.18, .79, .36], radius: .1778 },
  { id: 'hihat', label: 'Hi-hat', position: [-.52, .96, .35], radius: .1778 },
  { id: 'openHat', label: 'Open hi-hat', position: [-.52, .96, .35], radius: .1778 },
  { id: 'pedalHat', label: 'Hi-hat pedal', position: [-.22, .08, .55], radius: .12 },
  { id: 'tomHigh', label: 'High tom', position: [-.26, 1, .72], radius: .1524 },
  { id: 'tomMid', label: 'Mid tom', position: [.18, .98, .74], radius: .1651 },
  { id: 'tomLow', label: 'Floor tom', position: [.62, .80, .42], radius: .2032 },
  { id: 'crash', instrument: 'crash', label: 'Crash Left', position: [-.66, 1.26, .68], radius: .2032 },
  { id: 'crashRight', instrument: 'crash', label: 'Crash Right', position: [.58, 1.20, .64], radius: .2286 },
  { id: 'ride', label: 'Ride', position: [.70, .99, .39], radius: .254 },
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
    const targetPositions: Partial<Record<PieceId, Vec3>> = {};
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
  setFirstPerson(enabled: boolean): void {
    // Preserve arms, wrist markers and sticks; hide geometry around the seated eye.
    [this.head, this.neck, this.torso, this.hips, this.chest, ...this.shoulderLinks, ...this.hipLinks].forEach(node => { node.visible = !enabled; });
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

type KitVisual = PieceModel & { id: PieceId; instrument: Instrument; hits: Hit[]; response: HitResponseBinding; responseTrack: HitResponseTrack; };
type CameraTransition = { from: CameraState; to: CameraState; started: number };
type ImportedModel = { root: THREE.Group; name: string; approximate: boolean };

export class DrumScene {
  readonly canvas: HTMLCanvasElement;
  readonly renderer: THREE.WebGLRenderer;
  rig: HumanoidRigAdapter;
  onPieceSelect: ((id: PieceId | null) => void) | null = null;
  private readonly scene = new THREE.Scene();
  private readonly displayRoot = createLegacyDisplayRoot();
  private readonly camera = new THREE.PerspectiveCamera(35, 1, .05, 40);
  private readonly controls: OrbitControls;
  private readonly kitGroup = new THREE.Group();
  private readonly stickTrace = new THREE.Group();
  private readonly hardware = new KitHardwareLayer();
  private readonly grid = new THREE.GridHelper(5, 20, 0xb9bec5, 0xd9dde2);
  private readonly raycaster = new THREE.Raycaster();
  private readonly resizeObserver: ResizeObserver;
  private visuals: KitVisual[] = [];
  private plan: PerformancePlan | null = null;
  private kit: readonly KitPiece[] = DEFAULT_KIT;
  private preset: KitPresetId = 'studio';
  private readonly overrides = new Map<PieceId, PieceVisualOptions>();
  private readonly imported = new Map<PieceId, ImportedModel>();
  private readonly proceduralOverrides = new Set<PieceId>();
  private readonly pieceLoadVersions = new Map<PieceId, number>();
  private readonly libraries = new Map<KitPresetId, Partial<Record<Instrument, THREE.Group>>>();
  private readonly libraryPromises = new Map<KitPresetId, Promise<Partial<Record<Instrument, THREE.Group>>>>();
  private loadGeneration = 0;
  private disposed = false;
  private equipmentGeneration = 0;
  private equipmentKey = '';
  private equipmentPromise: Promise<KitGeometry> | null = null;
  private equipmentResult: KitGeometry | null = null;
  private equipmentError: string | null = null;
  private selected: PieceId | null = null;
  private selectionBox: THREE.BoxHelper | null = null;
  private editorMode = false;
  private characterVisible = true;
  private skeletonVisible = true;
  private modelVisible = true;
  private theme: 'light' | 'dark' = 'light';
  private exportMode = false;
  private cameraView: CameraView = 'stage';
  private readonly reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  private hitResponse: HitResponseSettings = { enabled: !this.reducedMotion.matches, strength: .65 };
  private songTime = -.5;
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
    this.scene.add(this.displayRoot,this.stickTrace); this.displayRoot.add(this.kitGroup, this.hardware.root);
    this.rig = new NativeHumanoidRigAdapter(); this.scene.add(this.rig.root);
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
    this.reducedMotion.addEventListener('change', this.motionPreferenceChanged);
  }

  setPlan(plan: PerformancePlan): void {
    this.setStickTrace(null);
    this.rig.calibrate(plan.kit); this.plan = plan; this.kit = plan.kit; this.rebuildKit();
  }
  clearPlan(): void {
    this.setStickTrace(null);
    this.plan = null; this.songTime = -.5;
    for (const visual of this.visuals) { visual.hits = []; visual.responseTrack = createHitResponseTrack([], visual.instrument); }
    this.rig.applyPose(createRestRigPose(this.rig.getProfile())); this.updateHitResponses();
  }
  setRig(adapter: HumanoidRigAdapter): void {
    if (adapter === this.rig) return;
    // Configure the candidate before transferring ownership. A calibration or
    // pose failure must not dispose the currently displayed character.
    adapter.calibrate(this.kit);
    if (!this.plan) adapter.applyPose(createRestRigPose(adapter.getProfile()));
    adapter.root.visible = this.characterVisible && !this.editorMode;
    adapter.setSkeletonVisible(this.skeletonVisible); adapter.setModelVisible(this.modelVisible);
    adapter.setVisualOptions({ bodyColor: this.theme === 'light' ? '#5c626b' : '#c3c9d1' });
    adapter.setFirstPerson?.(!this.editorMode && this.cameraView === 'player');
    const previous = this.rig;
    this.scene.add(adapter.root); this.rig = adapter;
    this.setStickTrace(null); previous.dispose();
    if (!this.editorMode && this.cameraView === 'player') this.setCamera('player');
  }
  /** Common-engine idle pose; never replace a prepared performance's pose. */
  applyReadyPose(pose: Float32Array): void {
    if(this.plan)return;
    if(pose.length!==77||!pose.every(Number.isFinite))throw new Error(msg("Invalid ready-pose data.", "기본 자세 데이터가 잘못되었습니다."));
    this.rig.applyPose(pose);this.songTime=-.5;this.updateHitResponses();
  }
  getRigProfile(): RigProfile { return this.rig.getProfile(); }
  getKitLayout(): KitPiece[] { return structuredClone(this.kit) as KitPiece[]; }
  /** Measure a proposed layout against the currently selected appearances before
   * passing it to planning. This never changes the displayed kit. */
  measureKitLayout(kit: readonly KitPiece[]): KitPiece[] {
    const measured = validateKitLayout(kit), library = this.libraries.get(this.preset);
    for (const piece of measured) {
      if (canonicalPieceId(piece.id) !== piece.id) continue;
      const source = this.imported.get(piece.id)?.root ?? (isLibraryPreset(this.preset) && !this.proceduralOverrides.has(piece.id) ? library?.[pieceInstrument(piece)] : undefined);
      const built = createMeasuredInstrumentBody(piece, this.getPieceVisual(piece.id), this.preset, measured, source, this.imported.has(piece.id));
      piece.geometry = built.geometry; disposeModel(built.root);
    }
    const hihat = measured.find(piece => piece.id === 'hihat')!, open = measured.find(piece => piece.id === 'openHat')!;
    open.geometry = structuredClone(hihat.geometry); return measured;
  }
  /** Actual body descriptors are synchronous; routed supports await WASM. */
  refreshEquipmentGeometry(): Promise<KitGeometry> {
    const kit = this.getKitLayout(), key = JSON.stringify(kit);
    if (key === this.equipmentKey && this.equipmentPromise) return this.equipmentPromise;
    const generation = this.equipmentGeneration = (this.equipmentGeneration ?? 0) + 1;
    this.equipmentKey = key; this.equipmentError = null;
    this.equipmentPromise = initializeWasm().then(() => solveKitGeometry(kit)).then(geometry => {
      if (!this.disposed && generation === this.equipmentGeneration) {
        this.hardware.setGeometry(kit, geometry); this.equipmentResult = geometry;
      }
      return geometry;
    }).catch(error => {
      if (!this.disposed && generation === this.equipmentGeneration) {
        this.equipmentError = error instanceof Error ? error.message : String(error);
        this.equipmentResult = null; this.hardware.clear(); this.equipmentKey = '';
      }
      throw error;
    });
    return this.equipmentPromise;
  }
  async whenEquipmentReady(): Promise<KitGeometry> {
    for (;;) {
      const pending = this.refreshEquipmentGeometry(), result = await pending;
      if (pending === this.equipmentPromise) return structuredClone(result);
    }
  }
  getEquipmentDiagnostics(): string[] {
    return this.equipmentError ? [msg('Equipment geometry calculation failed: {0}', '장비 형상 계산 실패: {0}', this.equipmentError)] : [...(this.equipmentResult?.unresolvedPieceIds ?? [])];
  }
  setKitLayout(kit: readonly KitPiece[]): void {
    this.kit = validateKitLayout(kit); this.rebuildKit();
  }
  getCalibration() { return this.rig.getCalibration(); }
  getAutomaticCalibration() { return this.rig.getAutomaticCalibration(this.kit); }
  /** Measure a candidate without leaving the displayed rig in that calibration. */
  measureCalibration(value: Parameters<HumanoidRigAdapter['setCalibration']>[0], kit: readonly KitPiece[]): RigProfile {
    const previous=this.rig.getCalibration();
    try{return this.rig.setCalibration(value,kit);}
    finally{this.rig.setCalibration(previous,this.kit);}
  }
  setCalibration(value: Parameters<HumanoidRigAdapter['setCalibration']>[0]): RigProfile {
    const profile = this.rig.setCalibration(value, this.kit);
    if (!this.plan) this.rig.applyPose(createRestRigPose(profile));
    if (!this.editorMode && this.cameraView === 'player') this.setCamera('player');
    return profile;
  }
  getRigContactSamples(): RigContactSample[] { return this.rig.getContactSamples(); }
  /** World-space trajectories measured from actual hand-local stick markers. */
  setStickTrace(paths:readonly [readonly Vec3[],readonly Vec3[]]|null):void {
    for(const child of [...this.stickTrace.children]){this.stickTrace.remove(child);disposeModel(child);}
    if(paths)paths.forEach((path,index)=>{
      if(path.length<2||path.length>121||!path.every(point=>point.length===3&&point.every(Number.isFinite)))throw new Error(msg("Invalid stick trajectory samples.", "스틱 궤적 표본이 잘못되었습니다."));
      const geometry=new THREE.BufferGeometry().setFromPoints(path.map(point=>new THREE.Vector3(...point)));
      const line=new THREE.Line(geometry,new THREE.LineBasicMaterial({color:index===0?0xc47b00:0x167bd1,depthTest:false}));
      line.renderOrder=20;this.stickTrace.add(line);
    });
    this.stickTrace.visible=!this.exportMode&&!this.editorMode;
  }
  setSkeletonVisible(visible: boolean): void { this.skeletonVisible = visible; this.rig.setSkeletonVisible(visible); }
  setModelVisible(visible: boolean): void { this.modelVisible = visible; this.rig.setModelVisible(visible); }
  setCharacterVisible(visible: boolean): void {
    this.characterVisible = visible; this.rig.root.visible = visible && !this.editorMode;
  }
  setTheme(theme: 'light' | 'dark'): void {
    this.theme = theme;
    this.setBackground(theme === 'light' ? '#ffffff' : '#000000');
    this.rig.setVisualOptions({ bodyColor: theme === 'light' ? '#5c626b' : '#c3c9d1' });
    (this.grid.material as THREE.LineBasicMaterial).color.set(theme === 'light' ? 0x8b949e : 0x7d8691);
  }
  setGridVisible(visible: boolean): void { this.grid.visible = visible; }
  setHitResponse(settings: HitResponseSettings): void {
    this.hitResponse = validateHitResponse(settings); this.updateHitResponses();
  }
  getHitResponse(): HitResponseSettings { return { ...this.hitResponse }; }
  setEditorMode(enabled: boolean): void {
    if (this.editorMode === enabled || this.exportMode) return;
    this.updateCameraTransition();this.stickTrace.visible=!enabled;
    const beforeEditor = this.cameraState();
    this.editorMode = enabled;
    this.rig.root.visible = this.characterVisible && !enabled;
    if (enabled) {
      this.savedCamera ??= beforeEditor;
      this.moveCamera({ ...cameraPresetState('front'), position: new THREE.Vector3(0, 1.4, 4.1), target: new THREE.Vector3(0, .76, .50), maxPolarAngle: Math.PI, enableRotate: false });
    } else if (this.savedCamera) {
      this.moveCamera(this.savedCamera);
    }
    this.rig.setFirstPerson?.(!enabled && this.cameraView === 'player'); this.updateHitResponses(); this.updateSelection();
  }
  async setKitPreset(preset: KitPresetId): Promise<void> {
    if (!['studio', 'vintage', 'electronic', 'library', 'batoski', 'smoj'].includes(preset)) throw new Error(msg("Unsupported drum kit.", "지원하지 않는 드럼 키트입니다."));
    const generation = ++this.loadGeneration;
    if (isLibraryPreset(preset)) {
      if (!this.libraryPromises.has(preset)) this.libraryPromises.set(preset, import('./model-library').then(module => module.loadLibraryKit(preset)).then(library => {
        if (this.disposed) this.disposeLibrary(library); else this.libraries.set(preset, library);
        return library;
      }));
      try { await this.libraryPromises.get(preset); }
      catch (error) { this.libraryPromises.delete(preset); throw error; }
      if (this.disposed || generation !== this.loadGeneration) return;
    }
    this.preset = preset; this.overrides.clear(); this.proceduralOverrides.clear();
    const discarded = [...this.imported.values()]; this.imported.clear(); this.rebuildKit();
    discarded.forEach(model => disposeModelSource(model.root));
    await this.whenEquipmentReady();
  }
  /** Load all external geometry before committing a project appearance. A failed
   * file or bundled preset leaves the currently displayed scene intact. */
  async prepareProjectAppearance(preset:KitPresetId,pieces:readonly {id:PieceId;settings:PieceVisualSettings}[],files:ReadonlyMap<PieceId,File>) {
    const imported=new Map<PieceId,ImportedModel>();
    try {
      if(isLibraryPreset(preset)&&!this.libraries.has(preset)) {
        const library=await import('./model-library').then(module=>module.loadLibraryKit(preset));
        if(this.disposed){this.disposeLibrary(library);throw new Error(msg("The scene has been disposed.", "화면이 종료되었습니다."));}
        this.libraries.set(preset,library);
      }
      for(const piece of pieces) {
        resolvePieceVisual(preset,piece.id,piece.settings);
        if(piece.settings.source==='imported') {
          const file=files.get(piece.id);if(!file)throw new Error(msg("Relink the drum model file.", "드럼 모델 파일을 다시 연결해 주세요."));
          imported.set(piece.id,{root:await loadLocalGLB(file),name:file.name,approximate:true});
        }
      }
    } catch(error){imported.forEach(model=>disposeModelSource(model.root));throw error;}
    let consumed=false;
    const measureLayout = (kit: readonly KitPiece[]) => {
      if (consumed) throw new Error(msg("The project appearance has already been applied or disposed.", "프로젝트 외형은 이미 적용하거나 정리했습니다."));
      const measured = validateKitLayout(kit), library = this.libraries.get(preset);
      for (const piece of measured) {
        if (canonicalPieceId(piece.id) !== piece.id) continue;
        const saved = pieces.find(value => value.id === piece.id)?.settings;
        if (!saved) throw new Error(msg('Drum appearance settings missing: {0}', '드럼 외형 설정이 누락되었습니다: {0}', piece.id));
        const settings = { ...resolvePieceVisual(preset, piece.id, saved), finishOverride: saved.finishOverride };
        const source = imported.get(piece.id)?.root ?? (saved.source !== 'procedural' && isLibraryPreset(preset) ? library?.[pieceInstrument(piece)] : undefined);
        const built = createMeasuredInstrumentBody(piece, settings, preset, measured, source, imported.has(piece.id));
        piece.geometry = built.geometry; disposeModel(built.root);
      }
      const hihat = measured.find(piece => piece.id === 'hihat')!, open = measured.find(piece => piece.id === 'openHat')!;
      open.geometry = structuredClone(hihat.geometry);
      return measured;
    };
    return {
      measureLayout,
      validateLayout: (kit: readonly KitPiece[]) => {
        const measured = measureLayout(kit);
        for (const piece of kit) {
          if (canonicalPieceId(piece.id) !== piece.id) continue;
          if (!piece.geometry || JSON.stringify(piece.geometry) !== JSON.stringify(measured.find(value => value.id === piece.id)!.geometry)) throw new Error(msg('The saved drum geometry differs from the source model: {0}. Prepare the original MIDI again.', '저장한 드럼 형상과 원본 모델이 다릅니다: {0}. 원본 MIDI로 다시 준비하세요.', raw(piece.label)));
        }
        return measured;
      },
      dispose:()=>{if(!consumed){imported.forEach(model=>disposeModelSource(model.root));consumed=true;}},
      apply:(kit?: readonly KitPiece[])=>{
        if(consumed)throw new Error(msg("The project appearance can only be applied once.", "프로젝트 외형은 한 번만 적용할 수 있습니다."));
        const target = kit ? validateKitLayout(kit) : undefined;
        ++this.loadGeneration;
        const discarded=[...this.imported.values()];this.imported.clear();this.overrides.clear();this.proceduralOverrides.clear();this.preset=preset;
        for(const piece of pieces){const {finish,finishOverride,model,scale,rotation}=piece.settings;this.overrides.set(piece.id,{...(finishOverride?{finish}:{}),model,scale,rotation});if(piece.settings.source==='procedural')this.proceduralOverrides.add(piece.id);}
        imported.forEach((model,id)=>this.imported.set(id,model));consumed=true;
        if (target) this.kit = target;
        this.rebuildKit();discarded.forEach(model=>disposeModelSource(model.root));
      },
    };
  }
  getPieceVisual(id: PieceId): PieceVisualSettings {
    const canonical = canonicalPieceId(id);
    const visual = resolvePieceVisual(this.preset, canonical, this.overrides.get(canonical));
    const imported = this.imported.get(canonical);
    return { ...visual,finishOverride:this.overrides.get(canonical)?.finish!==undefined, source: imported ? 'imported' : isLibraryPreset(this.preset) && !this.proceduralOverrides.has(canonical) ? 'library' : 'procedural', ...(imported ? { customModel: imported.name, approximateCalibration: imported.approximate } : {}) };
  }
  setPieceVisual(id: PieceId, options: PieceVisualOptions): void {
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
  resetPieceVisual(id: PieceId): void {
    const canonical = canonicalPieceId(id);
    this.invalidatePieceLoad(canonical);
    const imported = this.imported.get(canonical);
    this.imported.delete(canonical); this.overrides.delete(canonical); this.proceduralOverrides.delete(canonical);
    this.rebuildKit();
    if (imported) disposeModelSource(imported.root);
  }
  async loadPieceModel(id: PieceId, file: File): Promise<{ fileName: string; approximateCalibration: true }> {
    const canonical = canonicalPieceId(id), generation = this.loadGeneration;
    const version = this.invalidatePieceLoad(canonical);
    const root = await loadLocalGLB(file);
    if (this.disposed || generation !== this.loadGeneration || this.pieceLoadVersions.get(canonical) !== version) { disposeModelSource(root); throw new Error(msg("The kit changed while the model was loading. Select it again.", "모델을 불러오는 동안 키트가 변경되었습니다. 다시 선택해 주세요.")); }
    const previous = this.imported.get(canonical);
    this.proceduralOverrides.delete(canonical);
    this.imported.set(canonical, { root, name: file.name, approximate: true }); this.rebuildKit();
    if (previous) disposeModelSource(previous.root);
    await this.whenEquipmentReady();
    return { fileName: file.name, approximateCalibration: true };
  }
  selectPiece(id: PieceId | null): void {
    const canonical = id === null ? null : canonicalPieceId(id);
    if (canonical && !this.visuals.some(visual => visual.id === canonical)) return;
    this.selected = canonical; this.updateSelection(); this.onPieceSelect?.(canonical);
  }
  applyPose(pose: Float32Array, songTime: number): void {
    this.rig.applyPose(pose); this.songTime = songTime; this.updateHitResponses();
  }
  private updateHitResponses(): void {
    for (const visual of this.visuals) {
      let low = 0, high = visual.hits.length;
      while (low < high) { const mid = (low + high) >>> 1; if (visual.hits[mid].time <= this.songTime) low = mid + 1; else high = mid; }
      const hit = visual.hits[low - 1], age = hit ? this.songTime - hit.time : Infinity;
      const envelope = this.hitResponse.enabled && age >= 0 && age < .35 ? Math.exp(-age * 12) * (hit!.velocity / 127) * this.hitResponse.strength : 0;
      visual.ring.material.opacity = this.editorMode && this.imported.has(visual.id) ? .7 : envelope * .7;
      visual.ring.visible = !this.exportMode && this.editorMode && this.imported.has(visual.id) || envelope > .002;
      applyHitResponse(visual.response, evaluateHitResponse(visual.responseTrack, this.songTime, this.hitResponse, this.reducedMotion.matches || this.editorMode, visual.response.fallback));
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
  setCamera(preset: CameraView): void {
    if (this.editorMode) return;
    this.transition = null; this.savedCamera = null;
    const state = cameraPresetState(preset);
    if (preset === 'player') state.position.set(...this.rig.getPlayerPosition());
    this.applyCameraState(state); this.rig.setFirstPerson?.(preset === 'player');
    this.controls.enabled = !this.exportMode; this.controls.update();
  }
  getViewState():SceneViewState {
    const state=this.editorMode&&this.savedCamera?this.savedCamera:this.cameraState();
    return {view:state.view,position:state.position.toArray(),target:state.target.toArray(),fov:state.fov,near:state.near};
  }
  setViewState(state:SceneViewState):void {
    if(!['front','stage','top','player'].includes(state.view)||state.position.length!==3||state.target.length!==3||![...state.position,...state.target,state.fov,state.near].every(Number.isFinite)||state.fov<15||state.fov>100||state.near<.005||state.near>.5)throw new Error(msg("Invalid camera settings.", "카메라 설정이 잘못되었습니다."));
    this.transition=null;this.savedCamera=null;
    this.applyCameraState({...cameraPresetState(state.view),position:new THREE.Vector3(...state.position),target:new THREE.Vector3(...state.target),fov:state.fov,near:state.near});
    this.rig.setFirstPerson?.(state.view==='player');this.controls.enabled=!this.exportMode;this.controls.update();
  }
  setBackground(color: string): void { this.scene.background = new THREE.Color(color); }
  setExportMode(enabled: boolean): void {
    if (enabled) this.finishCameraTransition();
    this.exportMode = enabled;this.stickTrace.visible=!enabled&&!this.editorMode; this.controls.enabled = !enabled && !this.transition;
    this.pointerStart = null; this.updateSelection(); if (!enabled) this.resize();
  }
  dispose(): void {
    this.disposed = true; ++this.loadGeneration; this.resizeObserver.disconnect(); this.controls.dispose();
    this.reducedMotion.removeEventListener('change', this.motionPreferenceChanged);
    this.canvas.removeEventListener('pointerdown', this.pointerDown); this.canvas.removeEventListener('pointermove', this.pointerMove);
    this.canvas.removeEventListener('pointerup', this.pointerUp); this.canvas.removeEventListener('pointercancel', this.pointerCancel);
    this.clearSelection(); this.rig.dispose(); disposeModel(this.scene);
    this.imported.forEach(model => disposeModelSource(model.root)); this.imported.clear();
    this.libraries.forEach(library => this.disposeLibrary(library)); this.libraries.clear();
    this.renderer.dispose();
  }

  private cameraState(): CameraState {
    return { position: this.camera.position.clone(), target: this.controls.target.clone(), view: this.cameraView, fov: this.camera.fov, near: this.camera.near,
      minDistance: this.controls.minDistance, maxDistance: this.controls.maxDistance, maxPolarAngle: this.controls.maxPolarAngle,
      enableRotate: this.controls.enableRotate, enablePan: this.controls.enablePan, enableZoom: this.controls.enableZoom };
  }
  private applyCameraState(state: CameraState): void {
    this.camera.position.copy(state.position); this.controls.target.copy(state.target); this.cameraView = state.view;
    this.camera.fov = state.fov; this.camera.near = state.near; this.camera.updateProjectionMatrix();
    this.camera.layers.set(0); this.camera.layers.enable(state.view === 'player' ? 9 : 10);
    for (const key of ['minDistance', 'maxDistance', 'maxPolarAngle', 'enableRotate', 'enablePan', 'enableZoom'] as const) {
      (this.controls as unknown as Record<string, unknown>)[key] = state[key];
    }
  }
  private invalidatePieceLoad(id: PieceId): number {
    const version = (this.pieceLoadVersions.get(id) ?? 0) + 1; this.pieceLoadVersions.set(id, version); return version;
  }
  private disposeLibrary(library: Partial<Record<Instrument, THREE.Group>>): void {
    const root = new THREE.Group(); Object.values(library).forEach(group => { if (group) root.add(group); }); disposeModelSource(root);
  }
  private moveCamera(to: CameraState): void {
    this.transition = { from: this.cameraState(), to: { ...to, position: to.position.clone(), target: to.target.clone() }, started: performance.now() };
    this.cameraView = to.view;
    this.controls.minDistance = 0; this.controls.maxDistance = Infinity; this.controls.maxPolarAngle = Math.PI;
    this.controls.enabled = false;
    if (this.reducedMotion.matches) this.finishCameraTransition();
  }
  private updateCameraTransition(): void {
    if (!this.transition) return;
    const t = Math.min(1, (performance.now() - this.transition.started) / 350), eased = t * t * (3 - 2 * t);
    this.camera.position.lerpVectors(this.transition.from.position, this.transition.to.position, eased);
    if (this.transition.from.view === 'player' || this.transition.to.view === 'player') this.camera.position.y += .45 * Math.sin(Math.PI * eased);
    this.controls.target.lerpVectors(this.transition.from.target, this.transition.to.target, eased);
    this.camera.fov = THREE.MathUtils.lerp(this.transition.from.fov, this.transition.to.fov, eased); this.camera.updateProjectionMatrix();
    if (t === 1) { this.applyCameraState(this.transition.to); this.transition = null; this.controls.enabled = !this.exportMode; if (!this.editorMode) this.savedCamera = null; }
  }
  private finishCameraTransition(): void {
    if (this.transition) { this.applyCameraState(this.transition.to); this.transition = null; }
    if (!this.editorMode) this.savedCamera = null;
    this.controls.update(); this.controls.enabled = !this.exportMode;
  }
  private motionPreferenceChanged = (): void => { if (this.reducedMotion.matches) this.finishCameraTransition(); this.updateHitResponses(); };
  private clearSelection(): void {
    if (this.selectionBox) { this.scene.remove(this.selectionBox); this.selectionBox.geometry.dispose(); this.selectionBox.material.dispose(); this.selectionBox = null; }
  }
  private updateSelection(): void {
    this.clearSelection();
    if (!this.editorMode || this.exportMode || !this.selected) return;
    const model = this.visuals.find(visual => visual.id === this.selected); if (!model) return;
    model.root.updateWorldMatrix(true, true);
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
    this.displayRoot.updateMatrixWorld(true);
    const hit = this.raycaster.intersectObjects([...this.visuals.map(value => value.root), this.hardware.root], true)[0];
    let object: THREE.Object3D | null = hit?.object ?? null;
    while (object && !object.userData.pieceId) object = object.parent;
    this.selectPiece(object?.userData.pieceId as PieceId ?? null);
  };
  private rebuildKit(): void {
    const nextKit = this.getKitLayout(), nextGroup = new THREE.Group(), visuals: KitVisual[] = [];
    const library = this.libraries.get(this.preset);
    try { for (const piece of nextKit) {
      if (canonicalPieceId(piece.id) !== piece.id) continue;
      const settings = this.getPieceVisual(piece.id);
      const source = this.imported.get(piece.id)?.root ?? (isLibraryPreset(this.preset) && !this.proceduralOverrides.has(piece.id) ? library?.[pieceInstrument(piece)] : undefined);
      const built = createMeasuredInstrumentBody(piece, settings, this.preset, nextKit, source, this.imported.has(piece.id));
      nextGroup.add(built.root); piece.geometry = built.geometry;
      const response = bindHitResponse(built.root, piece.position, this.imported.has(piece.id));
      built.root.userData.pieceId = piece.id;
      // Different physical cymbals may share a library body and sound voice, but
      // their selection, hardware, contact and feedback are always independent.
      const instrument = pieceInstrument(piece);
      const hits = this.plan?.hits.filter(hit => canonicalPieceId(hitPieceId(hit)) === piece.id).sort((a, b) => a.time - b.time) ?? [];
      visuals.push({ ...built, id: piece.id, instrument, hits, response, responseTrack: createHitResponseTrack(hits, instrument) });
    }} catch (error) { disposeModel(nextGroup); throw error; }
    const hihat = nextKit.find(piece => piece.id === 'hihat'), openHat = nextKit.find(piece => piece.id === 'openHat');
    if (hihat?.geometry && openHat) openHat.geometry = structuredClone(hihat.geometry);
    this.clearSelection(); disposeModel(this.kitGroup); this.kitGroup.add(...[...nextGroup.children]);
    this.kit = nextKit; this.visuals = visuals;
    // Rejections remain observable through whenEquipmentReady/diagnostics even
    // for synchronous appearance edits; never report an unhandled background error.
    void this.refreshEquipmentGeometry().catch(() => {});
    this.updateHitResponses(); this.updateSelection();
  }
}
