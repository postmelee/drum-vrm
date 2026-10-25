import * as THREE from 'three';
import { pieceInstrument, type PieceId, type KitPiece, type Vec3 } from './types';

/** Shared mounting frames are kit data, never an avatar or a model-preset rule. */
export interface PieceAnchors { contact: Vec3; mount: Vec3; }
export function pieceAnchors(piece: KitPiece): PieceAnchors {
  const [x, y, z] = piece.position;
  const cymbal = ['hihat', 'openHat', 'crash', 'ride'].includes(pieceInstrument(piece));
  return {
    contact: [...piece.position],
    mount: pieceInstrument(piece) === 'kick' ? [x, piece.radius + .018, z + .34]
      : cymbal ? [x, y, z + piece.radius * .5] : [x, y, z],
  };
}
/** Conservative nominal body proxies for mounting, not skinned/triangle collision.
 * Cosmetic width/depth overrides deliberately do not reroute shared hardware. */
export interface BodyEnvelope { center: Vec3; radius: number; minY: number; maxY: number; }
export function pieceBodyEnvelope(piece: KitPiece): BodyEnvelope | undefined {
  const instrument = pieceInstrument(piece), mount = pieceAnchors(piece).mount;
  if (instrument === 'pedalHat' || instrument === 'openHat') return undefined;
  if (instrument === 'kick') return { center: mount, radius: Math.hypot(piece.radius + .017, .22), minY: .0, maxY: mount[1] + piece.radius + .017 };
  const cymbal = ['hihat', 'crash', 'ride'].includes(instrument);
  return {
    center: mount, radius: piece.radius + (cymbal ? .001 : .014),
    minY: piece.position[1] - (cymbal ? instrument === 'hihat' ? .045 : .02 : instrument === 'snare' ? .135 : instrument === 'tomLow' ? .295 : .225),
    maxY: piece.position[1] + (cymbal ? .05 : .008),
  };
}

export interface StandRoute { base: Vec3; boom: Vec3; mount: Vec3; clear: boolean; }
const SUPPORT_MARGIN = .025;
/** A segment is checked against expanded cylindrical proxies, including its
 * intermediate height. The expansion covers rod thickness and a small air gap. */
export function segmentClearsBody(a: Vec3, b: Vec3, body: BodyEnvelope, margin = SUPPORT_MARGIN): boolean {
  const dy = b[1] - a[1], lower = body.minY - margin, upper = body.maxY + margin;
  let start = 0, end = 1;
  if (Math.abs(dy) < 1e-10) {
    if (a[1] < lower || a[1] > upper) return true;
  } else {
    const t0 = (lower - a[1]) / dy, t1 = (upper - a[1]) / dy;
    start = Math.max(0, Math.min(t0, t1)); end = Math.min(1, Math.max(t0, t1));
    if (start > end) return true;
  }
  const dx = b[0] - a[0], dz = b[2] - a[2], lengthSquared = dx * dx + dz * dz;
  const nearest = lengthSquared < 1e-12 ? start : Math.max(start, Math.min(end, ((body.center[0] - a[0]) * dx + (body.center[2] - a[2]) * dz) / lengthSquared));
  return Math.hypot(a[0] + dx * nearest - body.center[0], a[2] + dz * nearest - body.center[2]) >= body.radius + margin;
}

/** Keep the cymbal mount independent of the vertical stand. Boom routing fixes
 * a stand through a tom without moving the calibrated musical contact. */
export function cymbalStandRoute(piece: KitPiece, kit: readonly KitPiece[]): StandRoute {
  const mount = pieceAnchors(piece).mount, top: Vec3 = [mount[0], mount[1] + .065, mount[2]];
  const obstacles = kit.filter(other => canonical(other.id) !== canonical(piece.id)).map(pieceBodyEnvelope).filter((value): value is BodyEnvelope => !!value);
  const candidate = (x: number, z: number): StandRoute => {
    const base: Vec3 = [x, .025, z], boom: Vec3 = [x, top[1], z];
    const clear = obstacles.every(body => segmentClearsBody(base, boom, body) && segmentClearsBody(boom, top, body));
    return { base, boom, mount: top, clear };
  };
  const original = candidate(mount[0], mount[2]);
  // A hi-hat needs a straight pull rod. Its clearance is a layout constraint.
  if (original.clear || pieceInstrument(piece) === 'hihat') return original;
  const outward = mount[0] < 0 ? -1 : 1;
  // Nearest distance first; mirrored angular order favors the outside of the kit.
  for (let step = 1; step <= 24; step++) for (let direction = 0; direction < 16; direction++) {
    const angle = (direction % 2 ? -1 : 1) * Math.ceil(direction / 2) * Math.PI / 8;
    const route = candidate(mount[0] + outward * Math.cos(angle) * step * .025, mount[2] + Math.sin(angle) * step * .025);
    if (route.clear) return route;
  }
  return original;
}
const up = new THREE.Vector3(0, 1, 0);
const canonical = (id: PieceId): PieceId => id === 'openHat' || id === 'pedalHat' ? 'hihat' : id;

/** Original application hardware. Only a layout edit replaces these objects;
 * body changes, finishes and hit feedback never inherit into their transforms. */
export function createPieceHardware(piece: KitPiece, kit: readonly KitPiece[]): THREE.Group {
  const root = new THREE.Group(); root.name = `Hardware/${piece.id}`;
  root.userData.pieceId = piece.id; root.userData.hitResponseRole = 'hardware';
  const metal = new THREE.MeshStandardMaterial({ color: 0x9ca0a3, metalness: .76, roughness: .3 });
  const dark = new THREE.MeshStandardMaterial({ color: 0x26282b, metalness: .15, roughness: .7 });
  const add = (geometry: THREE.BufferGeometry, material = metal, position: Vec3 = [0, 0, 0]) => {
    const mesh = new THREE.Mesh(geometry, material); mesh.position.set(...position); mesh.castShadow = mesh.receiveShadow = true; root.add(mesh); return mesh;
  };
  const rod = (a: Vec3, b: Vec3, radius = .011) => {
    const start = new THREE.Vector3(...a), delta = new THREE.Vector3(...b).sub(start);
    const mesh = add(new THREE.CylinderGeometry(radius, radius, delta.length(), 12));
    mesh.position.copy(start).addScaledVector(delta, .5); mesh.quaternion.setFromUnitVectors(up, delta.normalize());
    mesh.userData.segment = { start: [...a], end: [...b], radius }; return mesh;
  };
  const stand = (x: number, y: number, z: number) => {
    rod([x, .025, z], [x, y, z]);
    add(new THREE.CylinderGeometry(.023, .023, .04, 12), dark, [x, y * .55, z]);
    for (let i = 0; i < 3; i++) {
      const angle = i * Math.PI * 2 / 3 + .4;
      rod([x, Math.min(.25, y * .6), z], [x + Math.sin(angle) * .19, .012, z + Math.cos(angle) * .19]);
    }
  };
  const pedal = (id: 'kick' | 'pedalHat', contact: Vec3) => {
    const [x, y, z] = contact;
    const board = add(new THREE.BoxGeometry(.10, .024, .22), dark, [x, y - .012, z - .02]);
    board.name = id === 'kick' ? 'contactSurface' : 'pedalContactSurface';
    board.userData.hitResponseRole = 'pedal'; board.userData.fixedContact = true;
    board.userData.pieceId = id;
    add(new THREE.BoxGeometry(.13, .025, .27), dark, [x, .018, z - .02]);
    rod([x - .065, .03, z + .08], [x - .065, y + .04, z + .08], .008);
    rod([x + .065, .03, z + .08], [x + .065, y + .04, z + .08], .008);
    rod([x - .065, y + .04, z + .08], [x + .065, y + .04, z + .08], .008);
  };
  const anchors = pieceAnchors(piece), [x, y, z] = piece.position;
  for (const [name, point] of Object.entries(anchors)) {
    const anchor = new THREE.Object3D(); anchor.name = `${name}Anchor`; anchor.position.set(...point as Vec3); root.add(anchor);
  }
  root.userData.anchors = anchors;
  if (pieceInstrument(piece) === 'kick') {
    pedal('kick', piece.position);
    rod([x, y + .04, z + .08], [x, .28, z + .14], .008);
    add(new THREE.SphereGeometry(.028, 16, 12), dark, [x, .28, z + .15]);
    for (const side of [-1, 1]) rod([x + side * piece.radius * .78, piece.radius * .58, z + .4], [x + side * (piece.radius + .04), .015, z + .49], .012);
  } else if (['hihat', 'crash', 'ride'].includes(pieceInstrument(piece))) {
    const route = cymbalStandRoute(piece, kit); root.userData.standRoute = route;
    stand(route.base[0], route.boom[1], route.base[2]);
    if (Math.hypot(route.boom[0] - route.mount[0], route.boom[2] - route.mount[2]) > 1e-8) {
      const boom = rod(route.boom, route.mount); boom.name = 'cymbalBoom';
      const clamp = add(new THREE.SphereGeometry(.022, 12, 8), dark, route.boom); clamp.name = 'boomClamp';
    }
    rod([anchors.mount[0], y - .02, anchors.mount[2]], route.mount, .006);
    if (pieceInstrument(piece) === 'hihat') {
      const contact = kit.find(value => value.id === 'pedalHat')?.position ?? [-.22, .08, .55];
      pedal('pedalHat', contact);
      rod([x, .04, anchors.mount[2]], [contact[0], .04, contact[2]]);
    }
  } else {
    const support = Math.max(.09, y - .42); stand(x, support, z);
    for (let i = 0; i < 3; i++) {
      const angle = i * Math.PI * 2 / 3;
      rod([x, support, z], [x + Math.sin(angle) * piece.radius * .9, y - .10, z + Math.cos(angle) * piece.radius * .9], .009);
    }
  }
  return root;
}
function dispose(root: THREE.Object3D): void {
  const materials = new Set<THREE.Material>();
  root.traverse(node => { if (node instanceof THREE.Mesh) { node.geometry.dispose(); (Array.isArray(node.material) ? node.material : [node.material]).forEach(value => materials.add(value)); } });
  materials.forEach(value => value.dispose()); root.clear();
}
export class KitHardwareLayer {
  readonly root = new THREE.Group();
  private signature = '';
  constructor() { this.root.name = 'SharedKitHardware'; }
  /** Returns false when only body/preset settings changed; all object identities remain stable. */
  setLayout(kit: readonly KitPiece[]): boolean {
    const next = JSON.stringify(kit.map(piece => [piece.id, pieceInstrument(piece), piece.position, piece.radius]));
    if (this.signature === next) return false;
    const replacement = new THREE.Group();
    for (const piece of kit) if (canonical(piece.id) === piece.id) replacement.add(createPieceHardware(piece, kit));
    dispose(this.root); this.root.add(...replacement.children); this.signature = next; return true;
  }
  dispose(): void { dispose(this.root); this.signature = ''; }
}
