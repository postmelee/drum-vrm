import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import type { KitGeometry } from './types';

type Support = KitGeometry['supports'][number];
export interface HardwareMaterials { chrome: THREE.Material; dark: THREE.Material; rubber: THREE.Material; }
const up = new THREE.Vector3(0, 1, 0);

/** Mechanical details live inside the shared collision capsule. The endpoints
 * and role come from Rust; the renderer never routes or extends a support. */
export function createHardwarePart(segment: Support, materials: HardwareMaterials): THREE.Group {
  const root = new THREE.Group();
  const start = new THREE.Vector3(...segment.start), end = new THREE.Vector3(...segment.end);
  const delta = end.clone().sub(start), length = delta.length(), r = segment.radius;
  root.position.copy(start).add(end).multiplyScalar(.5);
  if (length > 1e-8) root.quaternion.setFromUnitVectors(up, delta.divideScalar(length));
  root.userData.segment = structuredClone(segment);
  root.userData.hardwareRole = segment.role ?? 'rod';
  root.userData.pieceId = segment.pieceId;
  root.userData.hitResponseRole = 'hardware';
  const geometries = new Map<THREE.Material, THREE.BufferGeometry[]>();
  const add = (geometry: THREE.BufferGeometry, material = materials.chrome, position = new THREE.Vector3(), rotation = new THREE.Euler()) => {
    geometry.applyMatrix4(new THREE.Matrix4().compose(position, new THREE.Quaternion().setFromEuler(rotation), new THREE.Vector3(1, 1, 1)));
    const list = geometries.get(material) ?? []; list.push(geometry); geometries.set(material, list);
  };
  const tube = (radius: number, height: number, y = 0, material = materials.chrome) =>
    add(new THREE.CylinderGeometry(radius, radius, Math.max(height, .0001), 16), material, new THREE.Vector3(0, y, 0));
  const sphere = (radius: number, material = materials.chrome) => add(new THREE.SphereGeometry(radius, 16, 10), material);
  const role = segment.role ?? 'rod';
  const bodyLength = Math.max(length, r);
  // A cylindrical collar and a transverse wing screw make each actual
  // adjustment distinct. The screw fits inside the capsule's circular section.
  const clamp = (ball = false) => {
    if (ball) sphere(r * .76, materials.dark);
    else tube(r * .66, bodyLength);
    tube(r * .77, Math.min(r * .25, bodyLength * .2), -bodyLength * .35);
    tube(r * .77, Math.min(r * .25, bodyLength * .2), bodyLength * .35);
    add(new THREE.CylinderGeometry(r * .13, r * .13, r * 1.8, 10), materials.chrome, new THREE.Vector3(), new THREE.Euler(0, 0, Math.PI / 2));
    for (const sign of [-1, 1]) add(new THREE.BoxGeometry(r * .48, r * .16, r * .32), materials.chrome, new THREE.Vector3(sign * r * .60, 0, 0), new THREE.Euler(0, 0, sign * .18));
  };
  if (['tomBall', 'mountBall', 'boomPivot', 'cymbalTilter', 'basketTilter'].includes(role)) {
    clamp(true);
    // Side discs make a tilter's axle visible; no rendered joint is larger
    // than the conservative sphere used by fitting and stick inspection.
    for (const sign of [-1, 1]) add(new THREE.CylinderGeometry(r * .55, r * .55, r * .12, 16), materials.chrome, new THREE.Vector3(sign * r * .67, 0, 0), new THREE.Euler(0, 0, Math.PI / 2));
  } else if (['collar', 'memoryLock', 'tomClamp', 'tomSocket', 'standClamp'].includes(role)) {
    clamp();
  } else if (role === 'rubberFoot' || role === 'basketGrip') {
    // The collision capsule is tangent to the floor. A smaller decorative
    // cylinder would leave a visible gap below the rubber boot.
    add(new THREE.CapsuleGeometry(r, length, 4, 12), materials.rubber);
  } else if (role === 'tripodLeg') {
    // Two parallel flat bars, with a gap between them, represent double-braced
    // construction inside one conservative collision envelope.
    for (const sign of [-1, 1]) add(new THREE.BoxGeometry(r * .32, Math.max(length, .001), r * .92), materials.chrome, new THREE.Vector3(sign * r * .52, 0, 0));
    for (const sign of [-1, 1]) add(new THREE.CylinderGeometry(r * .37, r * .37, r * 1.7, 10), materials.chrome, new THREE.Vector3(0, sign * length * .46, 0), new THREE.Euler(0, 0, Math.PI / 2));
  } else if (role === 'tripodBrace' || role === 'suspensionBracket') {
    add(new THREE.BoxGeometry(r * 1.35, Math.max(length, .001), r * .5));
    if (role === 'suspensionBracket') for (const sign of [-1, 1])
      add(new THREE.CylinderGeometry(r * .30, r * .30, r * .8, 8), materials.chrome, new THREE.Vector3(0, sign * length * .4, 0), new THREE.Euler(Math.PI / 2, 0, 0));
  } else if (role === 'cymbalFelt') {
    tube(r * .85, bodyLength, 0, materials.rubber);
  } else if (role === 'cymbalSleeve') {
    tube(r * .32, bodyLength, 0, materials.dark);
    tube(r * .92, Math.min(.006, bodyLength * .2), -bodyLength * .26, materials.rubber);
    tube(r * .88, Math.min(.006, bodyLength * .2), bodyLength * .24, materials.rubber);
    tube(r * .60, Math.min(.004, bodyLength * .15), bodyLength * .40);
  } else if (role === 'tomYoke') {
    tube(r * .86, bodyLength);
    tube(r * .96, Math.min(.014, bodyLength * .25));
  } else {
    if (length < 1e-8) sphere(r);
    else tube(r * .96, length);
  }
  // Batch decorative subparts by material; screws do not each add a draw call.
  for (const [material, shapes] of geometries) {
    const flat = shapes.map(shape => shape.toNonIndexed());
    const merged = mergeGeometries(flat, false);
    flat.forEach(shape => shape.dispose());
    shapes.forEach(shape => shape.dispose());
    if (!merged) throw new Error(`Cannot build hardware role ${role}`);
    const mesh = new THREE.Mesh(merged, material); mesh.castShadow = mesh.receiveShadow = true;
    mesh.userData.hitResponseRole = 'hardware'; root.add(mesh);
  }
  return root;
}
