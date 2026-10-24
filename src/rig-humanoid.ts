import * as THREE from 'three';
import { VRMHumanoid, VRMHumanBoneParentMap, type VRMHumanBoneName, type VRMHumanBones } from '@pixiv/three-vrm';
import type { RigAdapter, RigCalibration } from './scene';
import { RIG_POSE_FLOATS, type KitPiece, type Quat, type RigProfile, type Vec3 } from './types';
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
}
export interface HumanoidRigAdapter extends RigAdapter {
  getProfile(): RigProfile;
  getContactSamples(): RigContactSample[];
  setSkeletonVisible(visible: boolean): void;
  setModelVisible(visible: boolean): void;
  getPlayerPosition(): Vec3;
}
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
}

/** Derive a bent, reachable rest pose. This is also the initial visual before
 * MIDI preparation; the running renderer consumes Rust's 73-float pose. */
export function createRestRigPose(profile: RigProfile): Float32Array {
  const pose = new Float32Array(RIG_POSE_FLOATS);
  for (let limb = 0; limb < 4; limb++) {
    const root = new THREE.Vector3(...profile.roots[limb]), q = new THREE.Quaternion(...profile.contactRotations[limb]);
    const offset = new THREE.Vector3(...profile.gripOffsets[limb]).add(new THREE.Vector3(...profile.tipOffsets[limb])).applyQuaternion(q);
    const tip = new THREE.Vector3(...profile.restTips[limb]), end = tip.clone().sub(offset), direction = end.clone().sub(root);
    const distance = direction.length(); direction.normalize();
    const [a, b] = profile.lengths[limb], along = (a * a - b * b + distance * distance) / (2 * distance);
    const pole = new THREE.Vector3(...profile.bendPoles[limb]); pole.addScaledVector(direction, -pole.dot(direction)).normalize();
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
    const seat = new THREE.Vector3(0, Math.max(.45, seatHeight), .06), translation = seat.clone().sub(restPelvis);
    const roots: Vec3[] = [], lengths: [number, number][] = [], rests: Vec3[] = [], grips: Vec3[] = [], tips: Vec3[] = [], rotations: Quat[] = [];
    const poles: Vec3[] = [[-1, -.35, -.2], [1, -.35, -.2], [0, 0, 1], [0, 0, 1]];
    const chainData = CHAIN_NAMES.map((names, limb) => {
      const nodes = names.map(required) as Chain['nodes'];
      const points = nodes.map(node => node.getWorldPosition(new THREE.Vector3()));
      const axes: [THREE.Vector3, THREE.Vector3] = [points[1].clone().sub(points[0]), points[2].clone().sub(points[1])];
      const measures: [number, number] = [axes[0].length(), axes[1].length()];
      if (measures.some(length => !Number.isFinite(length) || length < .03 || length > 2)) throw new Error('리그 팔·다리의 실제 길이가 지원 범위를 벗어납니다.');
      const root = points[0].clone().add(translation), arm = limb < 2;
      const nativeQ = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), arm ? .29 : 0);
      const side = limb === 0 || limb === 2 ? 'left' : 'right';
      const index = arm ? humanoid.getNormalizedBoneNode(`${side}IndexProximal`) : null;
      const indexOffset = index?.getWorldPosition(new THREE.Vector3()).sub(points[2]);
      // Put the shaft in the index finger's curl plane, rather than passing
      // through the palm centre. Optional fingers fall back to a neutral socket.
      const grip: Vec3 = arm ? [indexOffset ? (limb === 0 ? -1 : 1) * indexOffset.z : 0, -.023, .035] : [0, -.025, 0];
      const toe = !arm ? humanoid.getNormalizedBoneNode(`${side}Toes`) : null;
      const footLength = toe ? THREE.MathUtils.clamp(toe.getWorldPosition(new THREE.Vector3()).distanceTo(points[2]), .07, .25) : .14;
      const tip: Vec3 = arm ? [0, 0, .31] : [0, 0, footLength];
      const restReach = Math.max(Math.abs(measures[0] - measures[1]) + .001, (measures[0] + measures[1]) * .78);
      const end = root.clone().add(new THREE.Vector3(arm ? (limb === 0 ? .07 : -.07) : 0, arm ? -.48 : -.68, arm ? .36 : .48).normalize().multiplyScalar(restReach));
      const restTip = end.add(new THREE.Vector3(...grip).add(new THREE.Vector3(...tip)).applyQuaternion(nativeQ));
      roots.push(legacyPositionToNative(root.toArray())); lengths.push(measures); rests.push(legacyPositionToNative(restTip.toArray()));
      grips.push(legacyPositionToNative(grip)); tips.push(legacyPositionToNative(tip)); rotations.push(legacyQuaternionToNative(nativeQ.toArray()));
      return { nodes, restAxes: axes.map(axis => axis.normalize()) as Chain['restAxes'] };
    });
    this.profile = {
      schemaVersion: 1, id, coordinateSystem: 'legacy-y-up-z-forward-left-negative-x',
      roots: roots as RigProfile['roots'], lengths: lengths as RigProfile['lengths'], restTips: rests as RigProfile['restTips'], bendPoles: poles as RigProfile['bendPoles'],
      gripOffsets: grips as RigProfile['gripOffsets'], tipOffsets: tips as RigProfile['tipOffsets'], contactRotations: rotations as RigProfile['contactRotations'],
      pelvis: legacyPositionToNative(seat.toArray()), chest: legacyPositionToNative(chest.getWorldPosition(new THREE.Vector3()).add(translation).toArray()),
      head: legacyPositionToNative(head.getWorldPosition(new THREE.Vector3()).add(translation).toArray()),
    };
    // All validation above precedes taking ownership of the source scene.
    this.root.name = id; this.root.add(source, this.skeletonOverlay);
    this.root.userData.coordinateSystem = 'native-vrm1-y-up-z-forward-left-positive-x';
    this.skeletonOverlay.name = 'SemanticBoneOverlay';
    this.refreshModelVisibility();
    for (let limb = 0; limb < 4; limb++) {
      const names = CHAIN_NAMES[limb], rawEnd = humanoid.getRawBoneNode(names[2])!;
      const restWorld = rawEnd.getWorldQuaternion(new THREE.Quaternion());
      const handToTool = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 1, 0), limb < 2 ? (limb === 0 ? 1 : -1) * Math.PI / 2 : 0);
      const socket = new THREE.Group(); socket.name = `${names[2]}ToolSocket`; socket.userData.rigTool = true;
      socket.quaternion.copy(restWorld).invert().multiply(handToTool); rawEnd.add(socket);
      // Neutralize any inherited rest scale in the socket, never in the bones.
      const scale = rawEnd.getWorldScale(new THREE.Vector3()); socket.scale.set(1 / scale.x, 1 / scale.y, 1 / scale.z);
      const grip = new THREE.Vector3(...legacyPositionToNative(grips[limb])), tip = new THREE.Vector3(...legacyPositionToNative(tips[limb]));
      const contact = new THREE.Object3D(); contact.name = limb < 2 ? `${names[2]}ActualStickTip` : `${names[2]}SoleContact`;
      contact.position.copy(grip).add(tip); socket.add(contact);
      if (limb < 2) {
        const stick = new THREE.Mesh(new THREE.CylinderGeometry(.0045, .006, tip.length() + .105, 12), new THREE.MeshStandardMaterial({ color: 0xb98d53, roughness: .6 }));
        stick.userData.rigTool = true; stick.name = limb === 0 ? 'leftStick' : 'rightStick';
        stick.rotation.x = Math.PI / 2; stick.position.copy(grip).addScaledVector(tip.clone().normalize(), (tip.length() - .105) / 2); socket.add(stick);
      }
      this.chains.push({ ...chainData[limb], handToTool, contact });
    }
    this.buildOverlay(); this.applyPose(createRestRigPose(this.profile));
  }

  getProfile(): RigProfile { return structuredClone(this.profile); }
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
      for (let part = 0; part < 2; part++) {
        const direction = points[part + 1].clone().sub(points[part]).normalize();
        const world = new THREE.Quaternion().setFromUnitVectors(chain.restAxes[part], direction);
        this.setWorldRotation(chain.nodes[part], world);
      }
      const tool = new THREE.Quaternion(...legacyQuaternionToNative(pose.subarray(57 + limb * 4, 61 + limb * 4))).normalize();
      this.setWorldRotation(chain.nodes[2], tool.multiply(chain.handToTool.clone().invert()));
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
    return this.chains.map((chain, limb) => { const actual = chain.contact.getWorldPosition(new THREE.Vector3()); return {
      limb, expected: tuple(this.expected[limb]), actual: tuple(actual), errorMeters: actual.distanceTo(this.expected[limb]),
      source: limb < 2 ? 'hand-local-stick-tip' : 'foot-local-sole-contact',
    }; });
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
    for (const side of ['left', 'right'] as const) {
      const sign = side === 'left' ? -1 : 1;
      for (const finger of ['Index', 'Middle', 'Ring', 'Little'] as const) for (const [index, segment] of ['Proximal', 'Intermediate', 'Distal'].entries()) {
        const node = this.humanoid.getNormalizedBoneNode(`${side}${finger}${segment}` as VRMHumanBoneName);
        node?.quaternion.setFromAxisAngle(new THREE.Vector3(0, 0, 1), sign * [.82, 1.05, .7][index]);
      }
      for (const [index, segment] of ['Metacarpal', 'Proximal', 'Distal'].entries()) {
        const node = this.humanoid.getNormalizedBoneNode(`${side}Thumb${segment}` as VRMHumanBoneName);
        if (node) node.quaternion.setFromEuler(new THREE.Euler(index === 0 ? -.4 : 0, sign * .35, sign * [.5, .55, .45][index]));
      }
    }
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
