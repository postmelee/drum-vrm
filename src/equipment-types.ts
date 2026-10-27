import type { Quat, Vec3 } from './types';

/** Numeric mechanical frame from the same Rust geometry used for collision. */
export interface EquipmentMechanism {
  pieceId: string;
  kind: 'kick' | 'hihat';
  pivot: Vec3;
  axle: Vec3;
  contact: Vec3;
  normal: Vec3;
  restAngle: number;
  maxOpening: number;
  motionAxis: Vec3;
  boardRotation?: Quat;
}
