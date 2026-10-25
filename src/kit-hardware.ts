import * as THREE from 'three';
import { pieceInstrument, type PieceId, type KitPiece, type Vec3 } from './types';

/** Shared mounting frames are kit data, never an avatar or a model-preset rule. */
export interface PieceAnchors { contact: Vec3; mount: Vec3; }
export function pieceAnchors(piece: KitPiece): PieceAnchors {
  const [x, y, z] = piece.position;
  const cymbal = ['hihat', 'openHat', 'crash', 'ride'].includes(pieceInstrument(piece));
  return {
    contact: [...piece.position],
    mount: pieceInstrument(piece) === 'kick' ? [x, .32, z + .34]
      : cymbal ? [x, y, z + piece.radius * .5] : [x, y, z],
  };
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
    mesh.position.copy(start).addScaledVector(delta, .5); mesh.quaternion.setFromUnitVectors(up, delta.normalize()); return mesh;
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
    for (const side of [-1, 1]) rod([x + side * .24, .18, z + .4], [x + side * .35, .015, z + .49], .012);
  } else if (['hihat', 'crash', 'ride'].includes(pieceInstrument(piece))) {
    const mountZ = anchors.mount[2]; stand(x, y + .065, mountZ);
    if (pieceInstrument(piece) === 'hihat') {
      const contact = kit.find(value => value.id === 'pedalHat')?.position ?? [-.22, .08, .55];
      pedal('pedalHat', contact);
      rod([x, .04, mountZ], [contact[0], .04, contact[2]]);
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
