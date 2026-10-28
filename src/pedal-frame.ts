import { Quaternion } from 'three';
import type { KitPiece, Quat, Vec3 } from './types';

/** Heel-to-toe axis shared by the board, sole and seated left-knee bend plane. */
export function pedalForwardDirection(piece: KitPiece | undefined): Vec3 {
  return [...(piece?.pedalForward ?? [0, 0, 1])];
}

/** Semantic sole orientation: local +Z runs from heel toward the mast.
 * Profile rotations remain neutral; layout yaw is composed only at evaluation.
 * The same quaternion composition is owned by Rust's equipment::pedal_rotation. */
export function pedalToolRotation(piece: KitPiece | undefined, base: Quat, pitch = 0): Quaternion {
  const direction = pedalForwardDirection(piece);
  const yaw = Math.atan2(direction[0], direction[2]) * .5;
  const halfPitch = pitch * .5;
  return new Quaternion(0, Math.sin(yaw), 0, Math.cos(yaw))
    .multiply(new Quaternion(...base))
    .multiply(new Quaternion(Math.sin(halfPitch), 0, 0, Math.cos(halfPitch)));
}
