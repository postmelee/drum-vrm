import { LatheGeometry, Vector2 } from 'three';
import dimensions from '../shared/stick-profile.json?raw';

export const STICK_PROFILE = JSON.parse(dimensions) as Readonly<{ buttLength: number; terminal: readonly (readonly [number, number])[] }>;

/** Grip at the origin, contact at +Z * tipLength. The shaft continues behind
 * the grip without stretching; the terminal tapers to the contact marker. */
export function createStickGeometry(tipLength: number): LatheGeometry {
  if (!Number.isFinite(tipLength) || tipLength < 0 || tipLength + STICK_PROFILE.buttLength <= STICK_PROFILE.terminal.at(-1)![0]) throw new Error('Stick length must exceed the taper.');
  const shaftRadius = STICK_PROFILE.terminal.at(-1)![1];
  const points = [new Vector2(0, -STICK_PROFILE.buttLength), new Vector2(shaftRadius, -STICK_PROFILE.buttLength),
    ...[...STICK_PROFILE.terminal].reverse().map(([back, radius]) => new Vector2(radius, tipLength - back))];
  const geometry = new LatheGeometry(points, 16);
  geometry.rotateX(Math.PI / 2);
  return geometry;
}
