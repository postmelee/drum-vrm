import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { Hit, Instrument, KitPiece, PerformancePlan, Vec3 } from './types';

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
const GOLD = 0xc5a466;
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
function disposeObject(root: THREE.Object3D): void {
  const geometries = new Set<THREE.BufferGeometry>();
  const materials = new Set<THREE.Material>();
  const textures = new Set<THREE.Texture>();
  root.traverse(node => {
    if (!(node instanceof THREE.Mesh || node instanceof THREE.Line)) return;
    geometries.add(node.geometry);
    for (const mat of Array.isArray(node.material) ? node.material : [node.material]) {
      materials.add(mat);
      if (mat instanceof THREE.MeshStandardMaterial && mat.map) textures.add(mat.map);
    }
  });
  textures.forEach(texture => texture.dispose());
  geometries.forEach(geometry => geometry.dispose());
  materials.forEach(mat => mat.dispose());
  root.clear();
}

/** The only class mapping canonical packed joints onto a particular visual rig. */
export class PrimitiveRigAdapter implements RigAdapter {
  readonly root = new THREE.Group();
  private readonly bodyMaterial = material(0x6d9d97, .10, .63);
  private readonly jointMaterial = material(0x252d30, .3, .57);
  private readonly skinMaterial = material(0xdbd6c5, .08, .48);
  private readonly limbs: { upper: THREE.Mesh; lower: THREE.Mesh; joint: THREE.Mesh; rootJoint: THREE.Mesh; end: THREE.Mesh; effector: THREE.Mesh }[] = [];
  private readonly torso: THREE.Mesh;
  private readonly hips: THREE.Mesh;
  private readonly neck: THREE.Mesh;
  private readonly head = new THREE.Group();
  private readonly collar: THREE.Mesh;

  constructor() {
    this.root.name = 'PrimitiveRigAdapter';
    const trouserMaterial = material(0x384247, .03, .89);
    const stickMaterial = material(0xddbc83, .02, .5);
    const shoeMaterial = material(0x273234, .06, .72);
    this.torso = cylinder(.145, 1, this.bodyMaterial, this.root, .20, 32);
    this.hips = sphere(.155, trouserMaterial, this.root);
    this.hips.scale.set(1.12, .69, .86);
    this.neck = cylinder(.061, 1, this.skinMaterial, this.root);
    this.collar = cylinder(.088, .032, this.jointMaterial, this.root);
    this.head.name = 'head';
    this.root.add(this.head);
    const skull = sphere(.127, this.skinMaterial, this.head);
    skull.scale.set(.91, 1.14, 1);
    // A quiet face slit and ear cups give the abstract mannequin a readable front.
    const visor = mesh(new THREE.BoxGeometry(.16, .026, .025), this.jointMaterial, this.head);
    visor.position.set(0, .015, .119);
    const nose = sphere(.019, this.skinMaterial, this.head);
    nose.position.set(0, -.017, .127);
    for (const side of [-1, 1]) {
      const ear = cylinder(.052, .037, this.jointMaterial, this.head);
      ear.rotation.z = Math.PI / 2;
      ear.position.set(side * .123, .01, 0);
      const accent = cylinder(.037, .039, material(GOLD, .65, .4), this.head);
      accent.rotation.z = Math.PI / 2;
      accent.position.copy(ear.position);
    }
    const headband = mesh(new THREE.TorusGeometry(.132, .012, 8, 28, Math.PI), this.jointMaterial, this.head);
    headband.rotation.z = 0;
    headband.position.y = .004;

    for (let index = 0; index < 4; index++) {
      const arm = index < 2;
      const upper = cylinder(arm ? .055 : .078, 1, arm ? this.bodyMaterial : trouserMaterial, this.root, arm ? .069 : .092, 20);
      const lower = cylinder(arm ? .043 : .056, 1, arm ? this.skinMaterial : trouserMaterial, this.root, arm ? .052 : .073, 20);
      const joint = sphere(arm ? .056 : .073, this.jointMaterial, this.root);
      const rootJoint = sphere(arm ? .078 : .088, arm ? this.bodyMaterial : trouserMaterial, this.root);
      const end = sphere(arm ? .045 : .048, arm ? this.skinMaterial : shoeMaterial, this.root);
      const effector = arm
        ? cylinder(.008, 1, stickMaterial, this.root, .005, 10)
        : cylinder(.054, 1, shoeMaterial, this.root, .045, 16);
      effector.name = ['leftStick', 'rightStick', 'leftFoot', 'rightFoot'][index];
      this.limbs.push({ upper, lower, joint, rootJoint, end, effector });
    }
    this.applyPose(defaultPose());
  }

  describeRig() {
    return {
      id: 'primitive-humanoid-v1', poseFloats: 57,
      joints: ['leftShoulder', 'leftElbow', 'leftWrist', 'leftStickTip', 'rightShoulder', 'rightElbow', 'rightWrist', 'rightStickTip', 'leftHip', 'leftKnee', 'leftAnkle', 'leftToe', 'rightHip', 'rightKnee', 'rightAnkle', 'rightToe', 'pelvis', 'chest', 'head'] as const,
    };
  }
  calibrate(kit: readonly KitPiece[]): RigCalibration {
    const targetPositions: Partial<Record<Instrument, Vec3>> = {};
    kit.forEach(piece => {
      if (piece.position.length !== 3 || !piece.position.every(Number.isFinite) || !Number.isFinite(piece.radius) || piece.radius <= 0) {
        throw new Error(`Invalid rig target: ${piece.id}`);
      }
      targetPositions[piece.id] = [...piece.position];
    });
    return { id: 'primitive-humanoid-v1', coordinateSystem: '+Y-up/+Z-forward', poseFloats: 57, stickLength: Math.hypot(.09, .30), targetPositions };
  }
  applyPose(pose: Float32Array): void {
    if (pose.length !== 57 || !pose.every(Number.isFinite)) throw new Error('Rig pose must contain 57 finite world-space coordinates.');
    this.limbs.forEach((limb, index) => {
      const offset = index * 12;
      POINT.fromArray(pose, offset);
      END.fromArray(pose, offset + 3);
      limb.rootJoint.position.copy(POINT);
      limb.joint.position.copy(END);
      connect(limb.upper, POINT, END);
      POINT.copy(END);
      END.fromArray(pose, offset + 6);
      connect(limb.lower, POINT, END);
      limb.end.position.copy(END);
      POINT.copy(END);
      END.fromArray(pose, offset + 9);
      if (index < 2) {
        // Grip lies inside the wooden stick. The distal end remains exactly at the canonical tip.
        POINT.addScaledVector(MID.subVectors(POINT, END), .17);
      }
      connect(limb.effector, POINT, END);
    });
    POINT.fromArray(pose, 48);
    this.hips.position.copy(POINT);
    POINT.y += .10;
    END.fromArray(pose, 51);
    END.y += .12;
    connect(this.torso, POINT, END);
    POINT.copy(END);
    END.fromArray(pose, 54);
    END.y -= .11;
    connect(this.neck, POINT, END);
    this.collar.position.copy(POINT);
    this.head.position.fromArray(pose, 54);
  }
  setVisualOptions(options: { bodyColor?: string }): void {
    if (options.bodyColor) this.bodyMaterial.color.set(options.bodyColor);
  }
  dispose(): void { disposeObject(this.root); }
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

type KitVisual = { id: Instrument; ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>; head?: THREE.Mesh; cymbal?: THREE.Group; hits: Hit[]; };

export class DrumScene {
  readonly canvas: HTMLCanvasElement;
  readonly renderer: THREE.WebGLRenderer;
  readonly rig: RigAdapter;
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(35, 1, .05, 40);
  private readonly controls: OrbitControls;
  private readonly kitGroup = new THREE.Group();
  private visuals: KitVisual[] = [];
  private readonly floorMaterial = material(0x182321, .04, .93);
  private exportMode = false;
  private readonly resizeObserver: ResizeObserver;
  private currentBackground = '#182321';

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, preserveDrawingBuffer: true, powerPreference: 'high-performance' });
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.3;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFShadowMap;
    this.scene.background = new THREE.Color(this.currentBackground);
    this.scene.fog = new THREE.Fog(this.currentBackground, 7, 16);
    this.scene.add(new THREE.HemisphereLight(0xe7f0e9, 0x414038, 2.15));
    const key = new THREE.DirectionalLight(0xffedd3, 4.4);
    key.position.set(-3, 5.5, 3.5);
    key.castShadow = true;
    key.shadow.mapSize.set(2048, 2048);
    key.shadow.camera.left = -2.7;
    key.shadow.camera.right = 2.7;
    key.shadow.camera.top = 2.7;
    key.shadow.camera.bottom = -2.7;
    key.shadow.camera.near = .1;
    key.shadow.camera.far = 12;
    key.shadow.normalBias = .025;
    key.shadow.bias = -.0002;
    this.scene.add(key);
    const rim = new THREE.DirectionalLight(0xb3ded6, 2.8);
    rim.position.set(2.5, 3, -3);
    this.scene.add(rim);
    const front = new THREE.DirectionalLight(0xffd6af, .8);
    front.position.set(3, 1.6, 4);
    this.scene.add(front);
    this.buildStage();
    this.scene.add(this.kitGroup);
    this.rig = new PrimitiveRigAdapter();
    this.scene.add(this.rig.root);
    this.buildKit(DEFAULT_KIT);
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = false;
    this.controls.minDistance = 2;
    this.controls.maxDistance = 8;
    this.controls.maxPolarAngle = Math.PI * .49;
    this.controls.target.set(0, .80, .32);
    this.controls.enablePan = true;
    this.setCamera('stage');
    this.resizeObserver = new ResizeObserver(() => { if (!this.exportMode) this.resize(); });
    this.resizeObserver.observe(canvas);
    this.resize();
  }

  setPlan(plan: PerformancePlan): void {
    this.rig.calibrate(plan.kit);
    this.buildKit(plan.kit);
    this.visuals.forEach(visual => {
      const instruments: Instrument[] = visual.id === 'hihat' ? ['hihat', 'openHat', 'pedalHat'] : [visual.id];
      visual.hits = plan.hits.filter(hit => instruments.includes(hit.instrument)).sort((a, b) => a.time - b.time);
    });
  }
  applyPose(pose: Float32Array, songTime: number): void {
    this.rig.applyPose(pose);
    for (const visual of this.visuals) {
      // Binary search means long performances do not require scanning all hits each frame.
      let low = 0;
      let high = visual.hits.length;
      while (low < high) {
        const mid = (low + high) >>> 1;
        if (visual.hits[mid].time <= songTime) low = mid + 1;
        else high = mid;
      }
      const hit = visual.hits[low - 1];
      const age = hit ? songTime - hit.time : Infinity;
      const envelope = age >= 0 && age < .35 ? Math.exp(-age * 12) * (hit!.velocity / 127) : 0;
      visual.ring.material.opacity = envelope * .85;
      visual.ring.visible = envelope > .002;
      visual.ring.scale.setScalar(1 + (1 - envelope) * .065);
      if (visual.cymbal) {
        const motion = age >= 0 && age < .8 ? Math.sin(age * 46) * Math.exp(-age * 7) * .035 * (hit!.velocity / 127) : 0;
        visual.cymbal.rotation.x = motion;
        visual.cymbal.rotation.z = motion * .65;
      }
    }
  }
  render(): void {
    if (!this.exportMode) this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
  resize(width?: number, height?: number): void {
    const rect = this.canvas.getBoundingClientRect();
    const size = (value: number) => Number.isFinite(value) ? Math.max(1, Math.round(value)) : 1;
    const w = size(width ?? rect.width);
    const h = size(height ?? rect.height);
    this.renderer.setPixelRatio(this.exportMode ? 1 : Math.min(window.devicePixelRatio || 1, 2));
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }
  setCamera(preset: 'front' | 'stage' | 'top'): void {
    const position = { front: [0, 1.95, 4.7], stage: [2.75, 2.4, 3.75], top: [.001, 4.8, .5] }[preset];
    this.camera.position.set(position[0], position[1], position[2]);
    this.controls.target.set(0, preset === 'top' ? .5 : .72, .32);
    this.controls.update();
  }
  setBackground(color: string): void {
    this.currentBackground = color;
    this.scene.background = new THREE.Color(color);
    this.scene.fog = new THREE.Fog(color, 7, 16);
    this.floorMaterial.color.set(color);
  }
  setExportMode(enabled: boolean): void {
    this.exportMode = enabled;
    this.controls.enabled = !enabled;
    if (!enabled) this.resize();
  }
  dispose(): void {
    this.resizeObserver.disconnect();
    this.controls.dispose();
    disposeObject(this.scene);
    this.renderer.dispose();
  }

  private buildStage(): void {
    const floor = mesh(new THREE.PlaneGeometry(200, 200), this.floorMaterial, this.scene);
    floor.rotation.x = -Math.PI / 2;
    floor.position.y = -.10;
    floor.castShadow = false;
    const rug = cylinder(1.54, .055, material(0x333c3b, .05, .97), this.scene, 1.54, 96);
    rug.position.set(0, -.057, .35);
    rug.castShadow = false;
    const border = mesh(new THREE.TorusGeometry(1.48, .005, 6, 128), material(0x78847a, .1, .85), this.scene);
    border.rotation.x = Math.PI / 2;
    border.position.set(0, -.025, .35);
    // The faint concentric weave is geometry, so the stage has no external image assets.
    const weave = new THREE.LineBasicMaterial({ color: 0x45504b, transparent: true, opacity: .23 });
    for (let radius = .12; radius < 1.4; radius += .09) {
      const points = Array.from({ length: 96 }, (_, index) => {
        const angle = index / 95 * Math.PI * 2;
        return new THREE.Vector3(Math.sin(angle) * radius, -.027, .35 + Math.cos(angle) * radius);
      });
      this.scene.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(points), weave));
    }
    const steel = material(0x808e8c, .85, .3);
    const seat = cylinder(.20, .07, material(0x24302f, .05, .75), this.scene, .20);
    seat.position.set(0, .61, -.20);
    rod(this.scene, [0, .03, -.20], [0, .59, -.20], .025, steel);
    for (let i = 0; i < 3; i++) {
      const angle = i * Math.PI * 2 / 3;
      rod(this.scene, [0, .25, -.20], [Math.sin(angle) * .28, .015, -.20 + Math.cos(angle) * .28], .016, steel);
    }
  }

  private buildKit(kit: readonly KitPiece[]): void {
    disposeObject(this.kitGroup);
    this.visuals = [];
    const shell = material(0xd5cbbb, .16, .39);
    const steel = material(0x8d9b98, .83, .25);
    const dark = material(0x273031, .35, .52);
    const headMat = material(0xc6c9bb, .05, .82);
    const cymbalMat = material(GOLD, .78, .39);
    cymbalMat.side = THREE.DoubleSide;
    const cymbalLine = material(0x94723b, .66, .48);
    const stage = this.kitGroup;
    const stand = (x: number, y: number, z: number, tripod = true) => {
      rod(stage, [x, .04, z], [x, y, z], .012, steel);
      cylinder(.024, .045, dark, stage).position.set(x, y * .54, z);
      if (tripod) {
        for (let i = 0; i < 3; i++) {
          const angle = i * Math.PI * 2 / 3 + .4;
          rod(stage, [x, .28, z], [x + Math.sin(angle) * .20, .015, z + Math.cos(angle) * .20], .012, steel);
          const foot = sphere(.024, dark, stage);
          foot.position.set(x + Math.sin(angle) * .20, .012, z + Math.cos(angle) * .20);
          foot.scale.y = .5;
        }
      }
    };
    const addRing = (id: Instrument, target: Vec3, radius: number): KitVisual => {
      const mat = new THREE.MeshBasicMaterial({ color: 0xc2f0c9, transparent: true, opacity: 0, side: THREE.DoubleSide, depthWrite: false });
      const ring = new THREE.Mesh(new THREE.RingGeometry(radius * .87, radius * .98, 64), mat);
      ring.rotation.x = -Math.PI / 2;
      ring.position.set(target[0], target[1] + .007, target[2]);
      ring.visible = false;
      stage.add(ring);
      const visual = { id, ring, hits: [] };
      this.visuals.push(visual);
      return visual;
    };

    for (const piece of kit) {
      const [x, y, z] = piece.position;
      const radius = piece.radius;
      if (piece.id === 'openHat' || piece.id === 'pedalHat') continue;
      if (piece.id === 'kick') {
        const bass = new THREE.Group();
        stage.add(bass);
        const centreZ = z + .34;
        const drumRadius = Math.max(.31, radius);
        bass.position.set(x, drumRadius + .005, centreZ);
        bass.rotation.x = Math.PI / 2;
        cylinder(drumRadius, .40, shell, bass);
        for (const side of [-1, 1]) {
          cylinder(drumRadius + .016, .030, dark, bass).position.y = side * .202;
          cylinder(drumRadius, .006, side === -1 ? dark : headMat, bass).position.y = side * .220;
          for (let i = 0; i < 10; i++) {
            const angle = i / 10 * Math.PI * 2;
            const lug = cylinder(.012, .065, steel, bass, .012, 10);
            lug.position.set(Math.sin(angle) * (drumRadius + .008), side * .15, Math.cos(angle) * (drumRadius + .008));
          }
        }
        const labelCanvas = document.createElement('canvas');
        labelCanvas.width = 512;
        labelCanvas.height = 512;
        const ctx = labelCanvas.getContext('2d');
        if (ctx) {
          ctx.fillStyle = '#25302f';
          ctx.fillRect(0, 0, 512, 512);
          ctx.textAlign = 'center';
          ctx.fillStyle = '#dcd5bf';
          ctx.font = '500 51px sans-serif';
          ctx.fillText('DRUM', 256, 207);
          ctx.font = '300 37px sans-serif';
          ctx.fillText('V R M', 256, 258);
          ctx.strokeStyle = '#879d8b';
          ctx.lineWidth = 2;
          ctx.beginPath();
          ctx.moveTo(204, 286); ctx.lineTo(308, 286); ctx.stroke();
          const map = new THREE.CanvasTexture(labelCanvas);
          map.colorSpace = THREE.SRGBColorSpace;
          const badge = mesh(new THREE.CircleGeometry(drumRadius * .95, 64), new THREE.MeshStandardMaterial({ map, roughness: .8 }), stage);
          badge.position.set(x, drumRadius + .005, centreZ + .226);
        }
        for (const side of [-1, 1]) rod(stage, [x + side * .23, .2, centreZ + .1], [x + side * .36, .016, centreZ + .19], .013, steel);
        const pedal = mesh(new THREE.BoxGeometry(.12, .035, .23), dark, stage);
        pedal.position.set(x, .025, z - .02);
        for (let i = 0; i < 4; i++) {
          const stripe = mesh(new THREE.BoxGeometry(.09, .002, .005), steel, stage);
          stripe.position.set(x, .044, z - .08 + i * .036);
        }
        rod(stage, [x, .07, z + .06], [x, .30, z + .11], .008, steel);
        sphere(.029, headMat, stage).position.set(x, .30, z + .12);
        addRing(piece.id, [x, .049, z -.01], .11);
        continue;
      }
      if (['hihat', 'crash', 'ride'].includes(piece.id)) {
        // Kit coordinates denote the playable bow, not the obstructed stand mount.
        const mountZ = z + radius * .50;
        stand(x, y + .07, mountZ);
        const cymbal = new THREE.Group();
        cymbal.position.set(x, y + .0294, mountZ);
        stage.add(cymbal);
        // The bell rises behind the target; the playable bow meets its canonical height.
        const profile = [new THREE.Vector2(.008, -.004), new THREE.Vector2(radius * .12, 0), new THREE.Vector2(radius * .24, -.022), new THREE.Vector2(radius * .8, -.038), new THREE.Vector2(radius, -.044)];
        mesh(new THREE.LatheGeometry(profile, 80), cymbalMat, cymbal);
        if (piece.id === 'hihat') {
          const bottom = mesh(new THREE.LatheGeometry(profile, 64), cymbalMat, cymbal);
          bottom.position.y = -.028;
          const pedalTarget = kit.find(p => p.id === 'pedalHat')?.position ?? [-.22, .08, .55];
          rod(stage, [x, .05, mountZ], [pedalTarget[0], .04, pedalTarget[2]], .012, steel);
          const pedal = mesh(new THREE.BoxGeometry(.10, .025, .20), dark, stage);
          pedal.position.set(pedalTarget[0], .035, pedalTarget[2] - .02);
        }
        for (const fraction of [.34, .45, .57, .69, .81, .93]) {
          const line = mesh(new THREE.TorusGeometry(radius * fraction, .0007, 4, 64), cymbalLine, cymbal);
          line.rotation.x = Math.PI / 2;
          line.position.y = -.022 - ((fraction - .24) / .76) * .022 + .001;
        }
        cylinder(.016, .015, dark, cymbal).position.y = .011;
        const wing = mesh(new THREE.BoxGeometry(.048, .009, .008), steel, cymbal);
        wing.position.y = .026;
        const visual = addRing(piece.id, [x, y, z], radius * .35);
        visual.cymbal = cymbal;
        continue;
      }
      const depth = piece.id === 'snare' ? .13 : piece.id === 'tomLow' ? .29 : .22;
      const drum = new THREE.Group();
      drum.position.set(x, y - depth / 2, z);
      stage.add(drum);
      cylinder(radius, depth, shell, drum);
      const top = cylinder(radius - .006, .008, headMat, drum);
      top.position.y = depth / 2 - .004;
      // The head is exposed above the annular rim, never hidden behind a solid cap.
      const rim = mesh(new THREE.TorusGeometry(radius, .009, 8, 64), steel, drum);
      rim.rotation.x = Math.PI / 2;
      rim.position.y = depth / 2 - .002;
      const bottomRim = rim.clone();
      bottomRim.position.y = -depth / 2 + .005;
      drum.add(bottomRim);
      for (let i = 0; i < 8; i++) {
        const angle = i / 8 * Math.PI * 2;
        cylinder(.010, depth * .67, steel, drum, .010, 10).position.set(Math.sin(angle) * (radius + .006), 0, Math.cos(angle) * (radius + .006));
      }
      if (piece.id === 'tomLow') {
        for (let i = 0; i < 3; i++) {
          const angle = i / 3 * Math.PI * 2;
          rod(stage, [x + Math.sin(angle) * radius, y - depth * .7, z + Math.cos(angle) * radius], [x + Math.sin(angle) * (radius + .06), .02, z + Math.cos(angle) * (radius + .06)], .012, steel);
        }
      } else stand(x, y - depth, z, piece.id === 'snare');
      const visual = addRing(piece.id, piece.position, radius);
      visual.head = top;
    }
  }
}
