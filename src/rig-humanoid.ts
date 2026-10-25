import * as THREE from 'three';
import { VRMHumanoid, VRMHumanBoneParentMap, type VRMHumanBoneName, type VRMHumanBones } from '@pixiv/three-vrm';
import type { RigAdapter, RigCalibration } from './scene';
import { RIG_POSE_FLOATS, type KitPiece, type Quat, type RigProfile, type Vec3, type HandControlFrame } from './types';
import { disposeModelSource } from './model-procedural';

/** The numeric engine retains the explicitly versioned v1 basis. Native VRM
 * bones use +X anatomical left, +Y up, +Z forward and positive scales. */
export const legacyPositionToNative = (point: ArrayLike<number>): Vec3 => [-point[0], point[1], point[2]];
export const legacyQuaternionToNative = (q: ArrayLike<number>): Quat => [q[0], -q[1], -q[2], q[3]];
const toVector = (point: ArrayLike<number>) => new THREE.Vector3(...legacyPositionToNative(point));
const tuple = (point: THREE.Vector3): Vec3 => [point.x, point.y, point.z];
const CHAIN_NAMES = [
  ['leftUpperArm', 'leftLowerArm', 'leftHand'], ['rightUpperArm', 'rightLowerArm', 'rightHand'],
  ['leftUpperLeg', 'leftLowerLeg', 'leftFoot'], ['rightUpperLeg', 'rightLowerLeg', 'rightFoot'],
] as const;
export interface RigContactSample {
  limb: number; expected: Vec3; actual: Vec3; errorMeters: number;
  /** Calibration socket contacts, not a collision test against arbitrary skin. */
  source: 'hand-local-stick-tip' | 'foot-local-sole-contact';
  /** Angle between the measured hand-forward axis and the actual raw forearm. */
  wristSwingDegrees?: number;
}
export interface HumanoidRigAdapter extends RigAdapter {
  getProfile(): RigProfile;
  getCalibration(): HumanoidCalibration;
  getAutomaticCalibration(kit?: readonly KitPiece[]): HumanoidCalibration;
  setCalibration(value: HumanoidCalibration, kit?: readonly KitPiece[]): RigProfile;
  getContactSamples(): RigContactSample[];
  setSkeletonVisible(visible: boolean): void;
  setModelVisible(visible: boolean): void;
  getPlayerPosition(): Vec3;
}
/** Absolute pelvis height and forward position in metres. Bone lengths never change. */
export type PalmOrientation = 'up' | 'inward' | 'down';
export interface HumanoidCalibration { seatHeight: number; seatForward: number; palmOrientation?: PalmOrientation; }
interface HandFrame { forward: THREE.Vector3; palm: THREE.Vector3; shaft: THREE.Vector3; grip: THREE.Vector3; }
/** Map two semantic directions without assuming the model's raw bone axes. */
function directionFrame(forward: THREE.Vector3, normal: THREE.Vector3): THREE.Quaternion {
  const x = forward.clone().normalize(), y = normal.clone().addScaledVector(x, -normal.dot(x));
  if (y.lengthSq() < 1e-10) {
    const axis = x.toArray().map(Math.abs).indexOf(Math.min(...x.toArray().map(Math.abs)));
    y.set(0, 0, 0).setComponent(axis, 1).addScaledVector(x, -x.getComponent(axis));
  }
  y.normalize(); return new THREE.Quaternion().setFromRotationMatrix(new THREE.Matrix4().makeBasis(x, y, x.clone().cross(y)));
}
function referenceElbow(root: THREE.Vector3, end: THREE.Vector3, lengths: [number, number], pole: THREE.Vector3): THREE.Vector3 {
  const delta = end.clone().sub(root), distance = delta.length(), direction = delta.normalize();
  const along = (lengths[0] ** 2 - lengths[1] ** 2 + distance ** 2) / (2 * distance);
  const bend = pole.clone().addScaledVector(direction, -pole.dot(direction)).normalize();
  return root.clone().addScaledVector(direction, along).addScaledVector(bend, Math.sqrt(Math.max(0, lengths[0] ** 2 - along ** 2)));
}
const DEFAULT_PEDALS: [Vec3, Vec3] = [[-.22, .08, .55], [.20, .08, .62]];
const pedalTargets = (kit?: readonly KitPiece[]): [Vec3, Vec3] => {
  return ['pedalHat', 'kick'].map((id, index) => {
    const point = kit?.find(piece => piece.id === id)?.position ?? DEFAULT_PEDALS[index];
    if (point.length !== 3 || !point.every(Number.isFinite)) throw new Error(`Invalid pedal target: ${id}`);
    return [...point];
  }) as [Vec3, Vec3];
};
interface AdapterHooks {
  beforePose?(): void;
  afterPose?(): void;
  disposeSource?(): void;
}
interface Chain {
  nodes: [THREE.Object3D, THREE.Object3D, THREE.Object3D];
  restAxes: [THREE.Vector3, THREE.Vector3];
  handToTool: THREE.Quaternion;
  contact: THREE.Object3D;
  socket: THREE.Group;
  rawRestInverse: THREE.Quaternion;
  stick?: THREE.Mesh;
}

/** Derive a bent, reachable rest pose. This is also the initial visual before
 * MIDI preparation; the running renderer consumes Rust's 73-float pose. */
export function createRestRigPose(profile: RigProfile): Float32Array {
  const pose = new Float32Array(RIG_POSE_FLOATS);
  for (let limb = 0; limb < 4; limb++) {
    const root = new THREE.Vector3(...profile.roots[limb]), q = new THREE.Quaternion(...profile.contactRotations[limb]);
    const offset = new THREE.Vector3(...profile.gripOffsets[limb]).add(new THREE.Vector3(...profile.tipOffsets[limb])).applyQuaternion(q);
    const tip = new THREE.Vector3(...profile.restTips[limb]), end = tip.clone().sub(offset), direction = end.clone().sub(root);
    const [a, b] = profile.lengths[limb];
    const distance = THREE.MathUtils.clamp(direction.length(), Math.abs(a - b) + 1e-6, a + b - 1e-6);
    if (direction.length() < 1e-10) direction.set(0, 1, 0); else direction.normalize();
    end.copy(root).addScaledVector(direction, distance); tip.copy(end).add(offset);
    const along = (a * a - b * b + distance * distance) / (2 * distance);
    const pole = new THREE.Vector3(...profile.bendPoles[limb]); pole.addScaledVector(direction, -pole.dot(direction));
    if (pole.lengthSq() < 1e-10) {
      const values=direction.toArray().map(Math.abs), axis=values.indexOf(Math.min(...values));
      pole.set(0,0,0).setComponent(axis,1);
      pole.addScaledVector(direction, -pole.dot(direction));
    }
    pole.normalize();
    const middle = root.clone().addScaledVector(direction, along).addScaledVector(pole, Math.sqrt(Math.max(0, a * a - along * along)));
    pose.set([...root.toArray(), ...middle.toArray(), ...end.toArray(), ...tip.toArray()], limb * 12);
    pose.set(q.toArray(), 57 + limb * 4);
  }
  pose.set(profile.pelvis, 48); pose.set(profile.chest, 51); pose.set(profile.head, 54); return pose;
}

/** Shared semantic solver for the built-in Bone hierarchy and VRM 1.0 models.
 * Normalized rotations are solved in world space, then converted to each
 * parent's local basis. Rendered effectors remain children of the actual hand
 * and foot, so retargeting/constraint errors remain visible and measurable. */
export class NormalizedHumanoidRigAdapter implements HumanoidRigAdapter {
  readonly root = new THREE.Group();
  readonly humanoid: VRMHumanoid;
  readonly profile: RigProfile;
  readonly skeletonOverlay = new THREE.Group();
  protected readonly source: THREE.Object3D;
  private readonly hooks: AdapterHooks;
  private readonly chains: Chain[] = [];
  private readonly overlayLinks: { from: THREE.Object3D; to: THREE.Object3D; mesh: THREE.Mesh; head: boolean }[] = [];
  private readonly overlayJoints: { bone: THREE.Object3D; mesh: THREE.Mesh; head: boolean }[] = [];
  private readonly boneMaterial = new THREE.MeshBasicMaterial({ color: 0x59636d, depthTest: false, transparent: true, opacity: .85 });
  private readonly jointMaterial = new THREE.MeshBasicMaterial({ color: 0x357cdb, depthTest: false });
  private readonly expected = [new THREE.Vector3(), new THREE.Vector3(), new THREE.Vector3(), new THREE.Vector3()];
  private readonly originalMeshVisibility = new Map<THREE.Mesh, boolean>();
  private firstPerson = false;
  private disposed = false;
  private calibration: HumanoidCalibration;
  private readonly baseProfile: RigProfile;
  private readonly baseCalibration: HumanoidCalibration;
  private pedals: [Vec3, Vec3] = pedalTargets();
  private readonly fingerPose: { node: THREE.Object3D; rotation: THREE.Quaternion }[] = [];
  private readonly handFrames: HandFrame[];
  private readonly handMounts: THREE.Quaternion[] = [];
  private readonly referenceWrists: THREE.Vector3[] = [];

  constructor(humanoid: VRMHumanoid, source: THREE.Object3D, id: string, hooks: AdapterHooks = {}) {
    this.humanoid = humanoid; this.source = source; this.hooks = hooks;
    if (!/^[A-Za-z0-9._:-]{1,128}$/.test(id) || id === 'primitive-v1') throw new Error('유효하지 않은 리그 ID입니다.');
    source.updateMatrixWorld(true);
    const boneAncestors = new Set<THREE.Object3D>();
    Object.values(humanoid.rawHumanBones).forEach(bone => { for (let node: THREE.Object3D | null = bone.node; node; node = node.parent) boneAncestors.add(node); });
    boneAncestors.forEach(node => {
      const values = [node.scale.x, node.scale.y, node.scale.z];
      if (!values.every(value => Number.isFinite(value) && value > 0)) throw new Error('유한한 양수 크기의 VRM 본 변환만 지원합니다.');
      if (Math.max(...values) - Math.min(...values) > Math.max(...values) * 1e-5) throw new Error('축마다 크기가 다른 비균일 본 스케일은 지원하지 않습니다. 균일한 스케일의 VRM을 사용해 주세요.');
    });
    humanoid.resetNormalizedPose(); humanoid.normalizedHumanBonesRoot.updateWorldMatrix(true, true);
    const required = (name: VRMHumanBoneName) => {
      const node = humanoid.getNormalizedBoneNode(name);
      if (!node || !humanoid.getRawBoneNode(name)) throw new Error(`필수 휴머노이드 본이 없습니다: ${name}`);
      return node;
    };
    const hips = required('hips'), head = required('head'); required('spine');
    const chest = humanoid.getNormalizedBoneNode('upperChest') ?? humanoid.getNormalizedBoneNode('chest') ?? required('spine');
    const restPelvis = hips.getWorldPosition(new THREE.Vector3());
    // Derive seat height from the actual legs and the canonical pedal targets.
    // Keep a small bend reserve rather than forcing shorter avatars' knees to
    // full extension, and retain all torso/limb rest distances unchanged.
    let seatHeight = .73;
    for (const [side, target] of [['left', [.22, .08, .55]], ['right', [-.20, .08, .62]]] as const) {
      const upper = required(`${side}UpperLeg`).getWorldPosition(new THREE.Vector3()), lower = required(`${side}LowerLeg`).getWorldPosition(new THREE.Vector3()), foot = required(`${side}Foot`).getWorldPosition(new THREE.Vector3());
      const toe = humanoid.getNormalizedBoneNode(`${side}Toes`);
      const footLength = toe ? THREE.MathUtils.clamp(toe.getWorldPosition(new THREE.Vector3()).distanceTo(foot), .07, .25) : .14;
      const relative = upper.clone().sub(restPelvis), reach = (upper.distanceTo(lower) + lower.distanceTo(foot)) * .94;
      const horizontalSq = (target[0] - relative.x) ** 2 + (target[2] - footLength - (.06 + relative.z)) ** 2;
      if (horizontalSq < reach * reach) seatHeight = Math.min(seatHeight, target[1] + .025 + Math.sqrt(reach * reach - horizontalSq) - relative.y);
    }
    const seat = new THREE.Vector3(0, Math.max(.3, seatHeight), .06), translation = seat.clone().sub(restPelvis);
    this.calibration = { seatHeight: seat.y, seatForward: seat.z, palmOrientation: 'up' };
    this.baseCalibration = { ...this.calibration };
    this.handFrames = this.prepareMatchedGrip();
    const roots: Vec3[] = [], lengths: [number, number][] = [], rests: Vec3[] = [], grips: Vec3[] = [], tips: Vec3[] = [], rotations: Quat[] = [];
    const controlFrames: HandControlFrame[] = [];
    const poles: Vec3[] = [[-1, -.35, -.2], [1, -.35, -.2], [0, 0, 1], [0, 0, 1]];
    const chainData = CHAIN_NAMES.map((names, limb) => {
      const nodes = names.map(required) as Chain['nodes'];
      const points = nodes.map(node => node.getWorldPosition(new THREE.Vector3()));
      const axes: [THREE.Vector3, THREE.Vector3] = [points[1].clone().sub(points[0]), points[2].clone().sub(points[1])];
      const measures: [number, number] = [axes[0].length(), axes[1].length()];
      if (measures.some(length => !Number.isFinite(length) || length < .03 || length > 2)) throw new Error('리그 팔·다리의 실제 길이가 지원 범위를 벗어납니다.');
      const root = points[0].clone().add(translation), arm = limb < 2;
      let nativeQ = new THREE.Quaternion();
      const side = limb === 0 || limb === 2 ? 'left' : 'right';
      // The shaft crosses the finger curl planes, as in a matched grip. Its
      // centre follows measured finger geometry rather than an avatar preset.
      let grip: Vec3 = [0, -.025, 0];
      const toe = !arm ? humanoid.getNormalizedBoneNode(`${side}Toes`) : null;
      const footLength = toe ? THREE.MathUtils.clamp(toe.getWorldPosition(new THREE.Vector3()).distanceTo(points[2]), .07, .25) : .14;
      const tip: Vec3 = arm ? [0, 0, .31] : [0, 0, footLength];
      const restReach = Math.max(Math.abs(measures[0] - measures[1]) + .001, (measures[0] + measures[1]) * .78);
      const end = root.clone().add(new THREE.Vector3(arm ? (limb === 0 ? .07 : -.07) : 0, arm ? -.48 : -.68, arm ? .36 : .48).normalize().multiplyScalar(restReach));
      if (arm) {
        this.referenceWrists.push(end.clone());
        const orientation = this.handOrientation(limb, root, end, measures, toVector(poles[limb]), 'up');
        nativeQ = orientation.rotation; grip = tuple(orientation.grip); this.handMounts[limb] = orientation.mount; controlFrames.push(orientation.controlFrame);
      }
      const restTip = arm ? end.clone().add(new THREE.Vector3(...grip).add(new THREE.Vector3(...tip)).applyQuaternion(nativeQ)) : toVector(this.pedals[limb - 2]);
      roots.push(legacyPositionToNative(root.toArray())); lengths.push(measures); rests.push(legacyPositionToNative(restTip.toArray()));
      grips.push(legacyPositionToNative(grip)); tips.push(legacyPositionToNative(tip)); rotations.push(legacyQuaternionToNative(nativeQ.toArray()));
      return { nodes, restAxes: axes.map(axis => axis.normalize()) as Chain['restAxes'] };
    });
    this.profile = {
      schemaVersion: 1, id, coordinateSystem: 'legacy-y-up-z-forward-left-negative-x',
      roots: roots as RigProfile['roots'], lengths: lengths as RigProfile['lengths'], restTips: rests as RigProfile['restTips'], bendPoles: poles as RigProfile['bendPoles'],
      gripOffsets: grips as RigProfile['gripOffsets'], tipOffsets: tips as RigProfile['tipOffsets'], contactRotations: rotations as RigProfile['contactRotations'],
      handFrames: controlFrames as [HandControlFrame, HandControlFrame],
      pelvis: legacyPositionToNative(seat.toArray()), chest: legacyPositionToNative(chest.getWorldPosition(new THREE.Vector3()).add(translation).toArray()),
      head: legacyPositionToNative(head.getWorldPosition(new THREE.Vector3()).add(translation).toArray()),
    };
    this.baseProfile = structuredClone(this.profile);
    // All validation above precedes taking ownership of the source scene.
    this.root.name = id; this.root.add(source, this.skeletonOverlay);
    this.root.userData.coordinateSystem = 'native-vrm1-y-up-z-forward-left-positive-x';
    this.skeletonOverlay.name = 'SemanticBoneOverlay';
    this.refreshModelVisibility();
    for (let limb = 0; limb < 4; limb++) {
      const names = CHAIN_NAMES[limb], rawEnd = humanoid.getRawBoneNode(names[2])!;
      const restWorld = rawEnd.getWorldQuaternion(new THREE.Quaternion());
      const handToTool = limb < 2 ? this.handMounts[limb].clone() : new THREE.Quaternion();
      const socket = new THREE.Group(); socket.name = `${names[2]}ToolSocket`; socket.userData.rigTool = true;
      socket.quaternion.copy(restWorld).invert().multiply(handToTool); rawEnd.add(socket);
      // Neutralize any inherited rest scale in the socket, never in the bones.
      const scale = rawEnd.getWorldScale(new THREE.Vector3()); socket.scale.set(1 / scale.x, 1 / scale.y, 1 / scale.z);
      const grip = new THREE.Vector3(...legacyPositionToNative(grips[limb])), tip = new THREE.Vector3(...legacyPositionToNative(tips[limb]));
      const contact = new THREE.Object3D(); contact.name = limb < 2 ? `${names[2]}ActualStickTip` : `${names[2]}SoleContact`;
      contact.position.copy(grip).add(tip); socket.add(contact);
      let stick: THREE.Mesh | undefined;
      if (limb < 2) {
        stick = new THREE.Mesh(new THREE.CylinderGeometry(.0045, .006, tip.length() + .105, 12), new THREE.MeshStandardMaterial({ color: 0xb98d53, roughness: .6 }));
        stick.userData.rigTool = true; stick.name = limb === 0 ? 'leftStick' : 'rightStick';
        stick.rotation.x = Math.PI / 2; stick.position.copy(grip).addScaledVector(tip.clone().normalize(), (tip.length() - .105) / 2); socket.add(stick);
      }
      this.chains.push({ ...chainData[limb], handToTool, contact, socket, rawRestInverse: restWorld.clone().invert(), stick });
    }
    this.buildOverlay(); this.applyPose(createRestRigPose(this.profile));
  }

  getProfile(): RigProfile { return structuredClone(this.profile); }
  getCalibration(): HumanoidCalibration { return { ...this.calibration }; }
  getAutomaticCalibration(kit?: readonly KitPiece[]): HumanoidCalibration {
    const pedals = kit ? pedalTargets(kit) : this.pedals;
    let height = .73;
    const forward = .06;
    for (let limb = 2; limb < 4; limb++) {
      const relative = new THREE.Vector3(...this.baseProfile.roots[limb]).sub(new THREE.Vector3(...this.baseProfile.pelvis));
      const ankle = new THREE.Vector3(...pedals[limb - 2]).sub(new THREE.Vector3(...this.baseProfile.gripOffsets[limb])).sub(new THREE.Vector3(...this.baseProfile.tipOffsets[limb]));
      const reach = (this.baseProfile.lengths[limb][0] + this.baseProfile.lengths[limb][1]) * .94;
      const horizontalSq = (ankle.x - relative.x) ** 2 + (ankle.z - forward - relative.z) ** 2;
      if (horizontalSq < reach * reach) height = Math.min(height, ankle.y + Math.sqrt(reach * reach - horizontalSq) - relative.y);
    }
    return { seatHeight: THREE.MathUtils.clamp(height, .3, 1.2), seatForward: forward, palmOrientation: this.calibration.palmOrientation };
  }
  setCalibration(value: HumanoidCalibration, kit?: readonly KitPiece[]): RigProfile {
    if (!Number.isFinite(value.seatHeight) || value.seatHeight < .3 || value.seatHeight > 1.2 || !Number.isFinite(value.seatForward) || Math.abs(value.seatForward) > .5) {
      throw new Error('좌석 높이는 0.3–1.2m, 앞뒤 위치는 −0.5–0.5m 범위여야 합니다.');
    }
    const palmOrientation = value.palmOrientation ?? this.calibration.palmOrientation ?? 'up';
    if (!['up', 'inward', 'down'].includes(palmOrientation)) throw new Error('Invalid palm orientation.');
    const pedals = kit ? pedalTargets(kit) : this.pedals;
    const profile = structuredClone(this.baseProfile);
    const offset: Vec3 = [0, value.seatHeight - this.baseCalibration.seatHeight, value.seatForward - this.baseCalibration.seatForward];
    const translate = (point: Vec3): Vec3 => point.map((v, i) => v + offset[i]) as Vec3;
    profile.roots = profile.roots.map(translate) as RigProfile['roots'];
    profile.pelvis = translate(profile.pelvis); profile.chest = translate(profile.chest); profile.head = translate(profile.head);
    profile.restTips = [translate(profile.restTips[0]), translate(profile.restTips[1]), [...pedals[0]], [...pedals[1]]];
    for (let limb = 0; limb < 2; limb++) {
      const wrist = this.referenceWrists[limb].clone().add(toVector(offset));
      const orientation = this.handOrientation(limb, toVector(profile.roots[limb]), wrist, profile.lengths[limb], toVector(profile.bendPoles[limb]), palmOrientation);
      profile.contactRotations[limb] = legacyQuaternionToNative(orientation.rotation.toArray());
      profile.handFrames![limb] = orientation.controlFrame;
      profile.gripOffsets[limb] = legacyPositionToNative(orientation.grip.toArray());
      const toolOffset = orientation.grip.clone().add(toVector(profile.tipOffsets[limb]));
      profile.restTips[limb] = legacyPositionToNative(wrist.add(toolOffset.clone().applyQuaternion(orientation.rotation)).toArray());
      const chain = this.chains[limb]; chain.handToTool.copy(orientation.mount);
      chain.socket.quaternion.copy(chain.rawRestInverse).multiply(orientation.mount);
      chain.contact.position.copy(toolOffset);
      const tip = toVector(profile.tipOffsets[limb]); chain.stick!.position.copy(orientation.grip).addScaledVector(tip.clone().normalize(), (tip.length() - .105) / 2);
    }
    Object.assign(this.profile, profile); this.pedals = pedals; this.calibration = { ...value, palmOrientation };
    return this.getProfile();
  }
  describeRig() { return { id: this.profile.id, joints: Object.keys(this.humanoid.rawHumanBones), poseFloats: RIG_POSE_FLOATS }; }
  calibrate(kit: readonly KitPiece[]): RigCalibration {
    const targetPositions: RigCalibration['targetPositions'] = {};
    for (const piece of kit) {
      if (!piece.position.every(Number.isFinite)) throw new Error(`Invalid rig target: ${piece.id}`);
      targetPositions[piece.id] = [...piece.position];
    }
    return { id: this.profile.id, coordinateSystem: '+Y-up/+Z-forward', poseFloats: 73, stickLength: .31, targetPositions };
  }
  applyPose(pose: Float32Array): void {
    if (pose.length !== RIG_POSE_FLOATS || !pose.every(Number.isFinite)) throw new Error('Humanoid pose requires 73 finite floats.');
    for (let limb = 0; limb < 4; limb++) { const q = new THREE.Quaternion().fromArray(pose, 57 + limb * 4); if (Math.abs(q.lengthSq() - 1) > .001) throw new Error('Humanoid tool rotation must be a unit quaternion.'); }
    this.hooks.beforePose?.(); this.humanoid.resetNormalizedPose();
    const hips = this.humanoid.getNormalizedBoneNode('hips')!;
    hips.position.copy(toVector(pose.subarray(48, 51))); hips.parent?.worldToLocal(hips.position);
    this.root.updateMatrixWorld(true);
    for (let limb = 0; limb < 4; limb++) {
      const chain = this.chains[limb], points = [0, 3, 6].map(offset => toVector(pose.subarray(limb * 12 + offset, limb * 12 + offset + 3)));
      const tool = new THREE.Quaternion(...legacyQuaternionToNative(pose.subarray(57 + limb * 4, 61 + limb * 4))).normalize();
      const hand = tool.clone().multiply(chain.handToTool.clone().invert());
      for (let part = 0; part < 2; part++) {
        const direction = points[part + 1].clone().sub(points[part]).normalize();
        let world = new THREE.Quaternion().setFromUnitVectors(chain.restAxes[part], direction);
        if (limb < 2 && part === 1) {
          // Put supination/pronation in the forearm rather than adding an axial
          // wrist twist. Swing still follows the immutable tool pose.
          const palm = this.handFrames[limb].palm;
          world = directionFrame(direction, palm.clone().applyQuaternion(hand))
            .multiply(directionFrame(chain.restAxes[part], palm).invert());
        }
        this.setWorldRotation(chain.nodes[part], world);
      }
      this.setWorldRotation(chain.nodes[2], hand);
      this.expected[limb].copy(toVector(pose.subarray(limb * 12 + 9, limb * 12 + 12)));
    }
    this.applyMatchedGrip();
    const head = this.humanoid.getNormalizedBoneNode('head')!;
    const nod = THREE.MathUtils.clamp((pose[56] - this.profile.head[2]) * 8, -.15, .15); head.quaternion.setFromAxisAngle(new THREE.Vector3(1, 0, 0), nod);
    this.root.updateMatrixWorld(true);
    if (this.hooks.afterPose) this.hooks.afterPose(); else this.humanoid.update();
    this.root.updateMatrixWorld(true); this.updateOverlay();
  }
  getContactSamples(): RigContactSample[] {
    this.root.updateMatrixWorld(true);
    return this.chains.map((chain, limb) => {
      const actual = chain.contact.getWorldPosition(new THREE.Vector3());
      let wristSwingDegrees: number | undefined;
      if (limb < 2) {
        const hand = this.humanoid.getRawBoneNode(CHAIN_NAMES[limb][2])!;
        const lower = this.humanoid.getRawBoneNode(CHAIN_NAMES[limb][1])!;
        const forearm = hand.getWorldPosition(new THREE.Vector3()).sub(lower.getWorldPosition(new THREE.Vector3())).normalize();
        const forward = this.handFrames[limb].forward.clone().applyQuaternion(chain.rawRestInverse).applyQuaternion(hand.getWorldQuaternion(new THREE.Quaternion()));
        wristSwingDegrees = THREE.MathUtils.radToDeg(Math.acos(THREE.MathUtils.clamp(forward.dot(forearm), -1, 1)));
      }
      return { limb, expected: tuple(this.expected[limb]), actual: tuple(actual), errorMeters: actual.distanceTo(this.expected[limb]),
        source: limb < 2 ? 'hand-local-stick-tip' as const : 'foot-local-sole-contact' as const,
        ...(wristSwingDegrees === undefined ? {} : { wristSwingDegrees }),
      };
    });
  }
  setVisualOptions(options: { bodyColor?: string }): void { if (options.bodyColor) this.boneMaterial.color.set(options.bodyColor); }
  setSkeletonVisible(visible: boolean): void { this.skeletonOverlay.visible = visible; }
  setModelVisible(visible: boolean): void { this.originalMeshVisibility.forEach((original, mesh) => { mesh.visible = visible && original; }); }
  protected refreshModelVisibility(): void {
    this.source.traverse(node => { if (node instanceof THREE.Mesh && !node.userData.rigTool && !this.originalMeshVisibility.has(node)) this.originalMeshVisibility.set(node, node.visible); });
  }
  setFirstPerson(enabled: boolean): void { this.firstPerson = enabled; this.updateOverlay(); }
  getPlayerPosition(): Vec3 { const head = toVector(this.profile.head); return [head.x, head.y + .045, head.z + .035]; }
  dispose(): void {
    if (this.disposed) return; this.disposed = true;
    if (this.hooks.disposeSource) { this.hooks.disposeSource(); disposeModelSource(this.skeletonOverlay); }
    else disposeModelSource(this.root);
    this.root.removeFromParent(); this.root.clear();
  }
  private setWorldRotation(node: THREE.Object3D, rotation: THREE.Quaternion): void {
    const parent = node.parent?.getWorldQuaternion(new THREE.Quaternion()) ?? new THREE.Quaternion();
    node.quaternion.copy(parent.invert().multiply(rotation)); node.updateWorldMatrix(false, true);
  }
  private applyMatchedGrip(): void {
    for (const { node, rotation } of this.fingerPose) node.quaternion.copy(rotation);
  }
  private handOrientation(limb: number, root: THREE.Vector3, wrist: THREE.Vector3, lengths: [number, number], pole: THREE.Vector3, mode: PalmOrientation) {
    const frame = this.handFrames[limb], forward = wrist.clone().sub(referenceElbow(root, wrist, lengths, pole)).normalize();
    const palm = mode === 'up' ? new THREE.Vector3(0, 1, 0) : mode === 'down' ? new THREE.Vector3(0, -1, 0) : new THREE.Vector3(limb === 0 ? -1 : 1, 0, 0);
    const hand = directionFrame(forward, palm).multiply(directionFrame(frame.forward, frame.palm).invert());
    const shaft = frame.shaft.clone().applyQuaternion(hand), y = new THREE.Vector3(0, 1, 0).addScaledVector(shaft, -shaft.y).normalize();
    if (y.lengthSq() < 1e-10) y.set(0, 0, 1).addScaledVector(shaft, -shaft.z).normalize();
    const rotation = new THREE.Quaternion().setFromRotationMatrix(new THREE.Matrix4().makeBasis(y.clone().cross(shaft).normalize(), y, shaft));
    const mount = hand.clone().invert().multiply(rotation);
    const controlFrame: HandControlFrame = {
      forearm: legacyPositionToNative(forward.toArray()),
      palm: legacyPositionToNative(frame.palm.clone().applyQuaternion(hand).toArray()),
      desiredPalm: legacyPositionToNative(palm.toArray()),
    };
    return { rotation, mount, grip: frame.grip.clone().applyQuaternion(mount.clone().invert()), controlFrame };
  }
  /** Prepare one deterministic pose from semantic finger joints. No skin names,
   * avatar IDs or per-model exceptions are involved. The shaft runs across the
   * closed fingers; the thumb opposes their measured centre. */
  private prepareMatchedGrip(): HandFrame[] {
    const frames: HandFrame[] = [];
    for (const side of ['left', 'right'] as const) {
      const hand = this.humanoid.getNormalizedBoneNode(`${side}Hand`)!;
      const fingers: THREE.Object3D[][] = [];
      for (const finger of ['Index', 'Middle', 'Ring', 'Little'] as const) {
        const nodes = ['Proximal', 'Intermediate', 'Distal'].map(segment => this.humanoid.getNormalizedBoneNode(`${side}${finger}${segment}` as VRMHumanBoneName));
        if (nodes.every(node => node !== null)) {
          const positions = nodes.map(node => node!.getWorldPosition(new THREE.Vector3()));
          if (positions.every(point => point.toArray().every(Number.isFinite)) && positions[0].distanceTo(positions[1]) > 1e-5 && positions[1].distanceTo(positions[2]) > 1e-5) fingers.push(nodes as THREE.Object3D[]);
        }
      }
      const points = fingers.map(nodes => nodes.map(node => hand.worldToLocal(node.getWorldPosition(new THREE.Vector3()))));
      const forward = points.length ? points[Math.min(1, points.length - 1)][0].clone().normalize() : new THREE.Vector3(side === 'left' ? 1 : -1, 0, 0);
      if (forward.lengthSq() < 1e-8) forward.set(side === 'left' ? 1 : -1, 0, 0);
      const across = points.length > 1 ? points[0][0].clone().sub(points.at(-1)![0]) : new THREE.Vector3(0, 0, 1);
      across.addScaledVector(forward, -across.dot(forward)).normalize();
      if (across.lengthSq() < 1e-8) {
        across.set(0, 0, 1); if (Math.abs(across.dot(forward)) > .9) across.set(1, 0, 0);
        across.addScaledVector(forward, -across.dot(forward)).normalize();
      }
      const palm = forward.clone().cross(across).multiplyScalar(side === 'left' ? 1 : -1).normalize();
      const shaft = forward.clone().add(across).normalize();
      const fingerLength = points.length ? points.reduce((sum, p) => sum + p[0].distanceTo(p[1]) + p[1].distanceTo(p[2]), 0) / points.length : .054;
      const grip = points.length ? points.reduce((sum, p) => sum.add(p[0]), new THREE.Vector3()).divideScalar(points.length) : forward.clone().multiplyScalar(.06);
      grip.addScaledVector(palm, fingerLength * .5).addScaledVector(forward, -fingerLength * .08);
      frames.push({forward, palm, shaft, grip});
      // Each finger wraps in its own plane perpendicular to the diagonal shaft.
      // Preserve every bone length; use a joint-centre clearance, not a claim of
      // collision-free arbitrary skin. Missing finger chains remain neutral.
      fingers.forEach((nodes, index) => {
        const p = points[index], centre = grip.clone().addScaledVector(shaft, p[0].clone().sub(grip).dot(shaft));
        const radial = p[0].clone().sub(centre), distance = radial.length(), axis = radial.normalize();
        const tangent = shaft.clone().cross(axis).normalize(); if (tangent.dot(palm) < 0) tangent.negate();
        const a = p[0].distanceTo(p[1]), b = p[1].distanceTo(p[2]);
        const radius = Math.max(fingerLength * .225, Math.abs(distance - a) + .0001);
        const nextRadius = Math.max(fingerLength * .225, Math.abs(radius - b) + .0001);
        const angle = Math.acos(THREE.MathUtils.clamp((distance * distance + radius * radius - a * a) / (2 * distance * radius), -1, 1));
        const bend = Math.acos(THREE.MathUtils.clamp((radius * radius + nextRadius * nextRadius - b * b) / (2 * radius * nextRadius), -1, 1));
        const circle = (r: number, theta: number) => centre.clone().addScaledVector(axis, r * Math.cos(theta)).addScaledVector(tangent, r * Math.sin(theta));
        const goals = [p[0], circle(radius, angle), circle(nextRadius, angle + bend), circle(nextRadius, angle + bend + .65)];
        for (let joint = 0; joint < 3; joint++) {
          const rest = p[Math.min(joint + 1, 2)].clone().sub(p[Math.min(joint, 1)]).normalize();
          this.setWorldRotation(nodes[joint], new THREE.Quaternion().setFromUnitVectors(rest, goals[joint + 1].clone().sub(goals[joint]).normalize()));
        }
      });
      const thumb = ['Metacarpal', 'Proximal', 'Distal'].map(segment => this.humanoid.getNormalizedBoneNode(`${side}Thumb${segment}` as VRMHumanBoneName));
      if (thumb.every(node => node !== null)) {
        const joints = thumb as THREE.Object3D[], end = new THREE.Object3D();
        const distal = joints[2], axis = distal.getWorldPosition(new THREE.Vector3()).sub(joints[1].getWorldPosition(new THREE.Vector3()));
        axis.setLength(THREE.MathUtils.clamp(axis.length() * .7, .008, .03));
        end.position.copy(axis); distal.add(end);
        const target = hand.localToWorld(grip.clone().addScaledVector(palm, -.009).addScaledVector(shaft, .004));
        // Bounded CCD opposes the thumb to the shaft. The resulting local pose
        // is cached, so playback/seek/export never integrate solver state.
        for (let step = 0; step < 12; step++) for (let i = joints.length - 1; i >= 0; i--) {
          const node = joints[i]; node.updateWorldMatrix(true, true);
          const pivot = node.getWorldPosition(new THREE.Vector3()), from = end.getWorldPosition(new THREE.Vector3()).sub(pivot), to = target.clone().sub(pivot);
          if (from.lengthSq() < 1e-12 || to.lengthSq() < 1e-12) continue;
          const world = new THREE.Quaternion().setFromUnitVectors(from.normalize(), to.normalize()).multiply(node.getWorldQuaternion(new THREE.Quaternion()));
          const local = node.parent!.getWorldQuaternion(new THREE.Quaternion()).invert().multiply(world);
          node.quaternion.copy(new THREE.Quaternion().rotateTowards(local, [1.1, 1.3, 1.3][i]));
        }
        end.removeFromParent();
      }
      for (const bone of Object.entries(this.humanoid.normalizedHumanBones)) {
        if (bone[0].startsWith(side) && /Thumb|Index|Middle|Ring|Little/.test(bone[0])) this.fingerPose.push({ node: bone[1].node, rotation: bone[1].node.quaternion.clone() });
      }
    }
    this.humanoid.resetNormalizedPose(); this.humanoid.normalizedHumanBonesRoot.updateWorldMatrix(true, true);
    return frames;
  }
  private buildOverlay(): void {
    const sphere = new THREE.SphereGeometry(.012, 8, 6), cylinder = new THREE.CylinderGeometry(.006, .006, 1, 6);
    for (const [name, bone] of Object.entries(this.humanoid.rawHumanBones)) {
      const finger = /Thumb|Index|Middle|Ring|Little/.test(name), head = ['head', 'neck', 'leftEye', 'rightEye', 'jaw'].includes(name);
      const marker = new THREE.Mesh(sphere, this.jointMaterial); marker.scale.setScalar(finger ? .38 : 1); marker.renderOrder = 3; this.skeletonOverlay.add(marker);
      this.overlayJoints.push({ bone: bone.node, mesh: marker, head });
      let parentName = VRMHumanBoneParentMap[name as VRMHumanBoneName];
      while (parentName && !this.humanoid.getRawBoneNode(parentName)) parentName = VRMHumanBoneParentMap[parentName];
      if (parentName) {
        const link = new THREE.Mesh(cylinder, this.boneMaterial); link.renderOrder = 2; this.skeletonOverlay.add(link);
        link.scale.x = link.scale.z = finger ? .45 : 1;
        this.overlayLinks.push({ from: this.humanoid.getRawBoneNode(parentName)!, to: bone.node, mesh: link, head });
      }
    }
  }
  private updateOverlay(): void {
    const up = new THREE.Vector3(0, 1, 0);
    for (const link of this.overlayLinks) {
      const from = link.from.getWorldPosition(new THREE.Vector3()), to = link.to.getWorldPosition(new THREE.Vector3()), direction = to.sub(from);
      link.mesh.position.copy(from).addScaledVector(direction, .5); link.mesh.scale.y = direction.length(); link.mesh.quaternion.setFromUnitVectors(up, direction.normalize());
      link.mesh.visible = !this.firstPerson || !link.head;
    }
    for (const joint of this.overlayJoints) { joint.bone.getWorldPosition(joint.mesh.position); joint.mesh.visible = !this.firstPerson || !joint.head; }
  }
}

/** A real semantic Bone tree, including fingers, serves as the default rig. */
export class NativeHumanoidRigAdapter extends NormalizedHumanoidRigAdapter {
  readonly skeleton: THREE.Skeleton;
  constructor() {
    const source = new THREE.Group(), nodes: Partial<Record<VRMHumanBoneName, THREE.Bone>> = {};
    const add = (name: VRMHumanBoneName, parent: VRMHumanBoneName | null, position: Vec3) => {
      const bone = new THREE.Bone(); bone.name = name; bone.position.set(...position); (parent ? nodes[parent]! : source).add(bone); nodes[name] = bone;
    };
    add('hips', null, [0, .95, 0]); add('spine', 'hips', [0, .18, 0]); add('chest', 'spine', [0, .18, 0]); add('upperChest', 'chest', [0, .12, 0]);
    add('neck', 'upperChest', [0, .09, 0]); add('head', 'neck', [0, .10, 0]);
    for (const side of ['left', 'right'] as const) {
      const s = side === 'left' ? 1 : -1;
      add(`${side}Shoulder`, 'upperChest', [s * .09, .025, 0]); add(`${side}UpperArm`, `${side}Shoulder`, [s * .10, 0, 0]);
      add(`${side}LowerArm`, `${side}UpperArm`, [s * .30, 0, 0]); add(`${side}Hand`, `${side}LowerArm`, [s * .27, 0, 0]);
      add(`${side}UpperLeg`, 'hips', [s * .10, -.07, 0]); add(`${side}LowerLeg`, `${side}UpperLeg`, [0, -.43, 0]);
      add(`${side}Foot`, `${side}LowerLeg`, [0, -.43, 0]); add(`${side}Toes`, `${side}Foot`, [0, -.035, .16]);
      for (const [f, finger] of ['Index', 'Middle', 'Ring', 'Little'].entries()) {
        const name = `${side}${finger}`;
        add(`${name}Proximal` as VRMHumanBoneName, `${side}Hand`, [s * .065, 0, .025 - f * .017]);
        add(`${name}Intermediate` as VRMHumanBoneName, `${name}Proximal` as VRMHumanBoneName, [s * .032, 0, 0]);
        add(`${name}Distal` as VRMHumanBoneName, `${name}Intermediate` as VRMHumanBoneName, [s * .022, 0, 0]);
      }
      add(`${side}ThumbMetacarpal`, `${side}Hand`, [s * .025, -.005, .035]);
      add(`${side}ThumbProximal`, `${side}ThumbMetacarpal`, [s * .028, 0, .016]); add(`${side}ThumbDistal`, `${side}ThumbProximal`, [s * .023, 0, 0]);
    }
    source.updateMatrixWorld(true);
    const humanoid = new VRMHumanoid(Object.fromEntries(Object.entries(nodes).map(([name, node]) => [name, { node }])) as unknown as VRMHumanBones);
    source.add(humanoid.normalizedHumanBonesRoot);
    super(humanoid, source, 'humanoid-native-v1'); this.skeleton = new THREE.Skeleton(Object.values(nodes));
  }
  override dispose(): void { this.skeleton.dispose(); super.dispose(); }
}
