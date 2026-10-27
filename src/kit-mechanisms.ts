import * as THREE from 'three';
import type { EquipmentMechanism } from './equipment-types';
import type { Hit, KitGeometry, Vec3 } from './types';

interface MovingNode { node: THREE.Object3D; rest: THREE.Vector3; direction: THREE.Vector3; }
interface MechanismBinding { frame: EquipmentMechanism; board?: THREE.Object3D; beater?: THREE.Object3D; marker?: THREE.Object3D; top: MovingNode[]; }
const up = new THREE.Vector3(0, 1, 0);
const smooth = (value: number) => { const x = Math.max(0, Math.min(1, value)); return x * x * (3 - 2 * x); };

/** Preparation/contact/recovery envelope, sampled by song time, never accumulated. */
export function kickMechanismAmount(times: readonly number[], time: number): number {
  let lo = 0, hi = times.length;
  while (lo < hi) { const mid = (lo + hi) >>> 1; if (times[mid] <= time) lo = mid + 1; else hi = mid; }
  const previous = times[lo - 1], next = times[lo];
  const recovery = previous === undefined ? 0 : 1 - smooth((time - previous) / .10);
  const approach = next === undefined ? 0 : smooth((time - next + .07) / .07);
  return Math.max(recovery, approach);
}

/** Tool quaternions use a semantic +Z foot axis, independent of raw VRM bones. */
export function pedalPitchFromPose(pose: ArrayLike<number> | undefined, limb: 2 | 3): number {
  if (!pose || pose.length < 73) return 0;
  const i = 57 + limb * 4, q = new THREE.Quaternion(pose[i], pose[i + 1], pose[i + 2], pose[i + 3]);
  if (!Number.isFinite(q.lengthSq()) || q.lengthSq() < .5) return 0;
  const forward = new THREE.Vector3(0, 0, 1).applyQuaternion(q.normalize());
  return Math.atan2(-forward.y, Math.hypot(forward.x, forward.z));
}

/** Bind only semantic moving parts. Imported bodies without separated hat
 * cymbals remain static and report no false mechanical compatibility. */
export class KitMechanismLayer {
  private readonly bindings: MechanismBinding[] = [];
  private readonly pitchRotation = new THREE.Quaternion();
  private kickTimes: number[] = [];
  constructor(private readonly hardware: THREE.Object3D, bodies: readonly { id: string; root: THREE.Object3D }[], geometry: KitGeometry) {
    hardware.updateWorldMatrix(true, true);
    for (const frame of geometry.mechanisms ?? []) {
      const owner = hardware.getObjectByName(`Hardware/${frame.pieceId}`);
      const binding: MechanismBinding = { frame, top: [] };
      owner?.traverse(node => {
        if (node.userData.mechanismRole === 'footboardPivot') binding.board = node;
        if (node.userData.mechanismRole === 'beaterPivot') binding.beater = node;
      });
      if (binding.beater) {
        const marker = binding.beater.getObjectByName('BeaterSurfaceContact') ?? new THREE.Object3D(); marker.name = 'BeaterSurfaceContact';
        marker.position.set(...frame.contact).sub(new THREE.Vector3(...frame.axle)); binding.beater.add(marker); binding.marker = marker;
      }
      const body = bodies.find(value => value.id === frame.pieceId)?.root;
      body?.updateWorldMatrix(true, true);
      body?.traverse(node => {
        if (node.userData.mechanismRole !== 'hihatTop') return;
        // Convert the planning +Y shaft travel into the authored parent frame;
        // hit-response transforms remain an independent visual layer.
        const direction = frame.motionAxis ? new THREE.Vector3(...frame.motionAxis) : up.clone();
        // A displacement is a vector, not a normalized direction: preserve the
        // inverse authored scale and the display reflection in every axis.
        direction.applyMatrix3(new THREE.Matrix3().setFromMatrix4(hardware.matrixWorld));
        if (node.parent) direction.applyMatrix3(new THREE.Matrix3().setFromMatrix4(node.parent.matrixWorld.clone().invert()));
        binding.top.push({ node, rest: node.position.clone(), direction });
      });
      this.bindings.push(binding);
    }
  }
  setHits(hits: readonly Hit[]): void {
    this.kickTimes = hits.filter(hit => hit.instrument === 'kick' && hit.limb !== null).map(hit => hit.time).sort((a, b) => a - b);
  }
  apply(songTime: number, hihatOpening: number, pose?: ArrayLike<number>): void {
    const opening = Math.max(0, Math.min(1, Number.isFinite(hihatOpening) ? hihatOpening : 0));
    for (const { frame, board, beater, top } of this.bindings) {
      if (board) {
        const pitch = pedalPitchFromPose(pose, frame.kind === 'kick' ? 3 : 2);
        board.quaternion.fromArray(frame.boardRotation ?? [0, 0, 0, 1]);
        this.pitchRotation.set(Math.sin(pitch * .5), 0, 0, Math.cos(pitch * .5));
        board.quaternion.multiply(this.pitchRotation);
      }
      if (beater) beater.rotation.x = -frame.restAngle * (1 - kickMechanismAmount(this.kickTimes, songTime));
      for (const moving of top) moving.node.position.copy(moving.rest).addScaledVector(moving.direction, frame.maxOpening * opening);
    }
  }
  /** Return the impact contact; the animated beater's surface reaches this
   * point at every actual kick time. This is not the footboard/sole target. */
  contact(pieceId: string): { point: Vec3; normal: Vec3 } | undefined {
    const frame = this.bindings.find(binding => binding.frame.pieceId === pieceId)?.frame;
    return frame ? { point: [...frame.contact], normal: [...frame.normal] } : undefined;
  }
  /** Measured rendered beater surface in the original planning frame. */
  actualContact(pieceId: string): Vec3 | undefined {
    const marker = this.bindings.find(binding => binding.frame.pieceId === pieceId)?.marker;
    if (!marker) return undefined;
    this.hardware.updateWorldMatrix(true, true);
    return this.hardware.worldToLocal(marker.getWorldPosition(new THREE.Vector3())).toArray() as Vec3;
  }
    get hasMovableHihat(): boolean { return this.bindings.some(binding => binding.frame.kind === 'hihat' && binding.top.length > 0); }
}
