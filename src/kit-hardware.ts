import * as THREE from 'three';
import { kit_geometry } from './wasm/drum_engine';
import { pieceInstrument, type KitPiece, type KitGeometry, type Vec3 } from './types';
import { createHardwarePart } from './hardware-parts';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';

/** Shared mounting frames are kit data, never an avatar or model-preset rule. */
export interface PieceAnchors { contact: Vec3; mount: Vec3; }
export function pieceAnchors(piece: KitPiece): PieceAnchors {
  const [x, y, z] = piece.position;
  const cymbal = ['hihat', 'openHat', 'crash', 'ride'].includes(pieceInstrument(piece));
  const ratio = piece.radius / (piece.geometry?.referenceRadius ?? piece.radius);
  return {
    contact: [...piece.position],
    mount: pieceInstrument(piece) === 'kick' ? piece.geometry ? piece.position.map((value, axis) => value + piece.geometry!.body.center[axis] * ratio) as Vec3 : [x, piece.radius + .018, z + .34]
      : cymbal ? [x, y, z + piece.radius * .5] : [x, y, z],
  };
}
const canonical = (id: string): string => id === 'openHat' || id === 'pedalHat' ? 'hihat' : id;

/** Retain semantic nodes for inspection, but draw fixed details in material
 * batches. Moving pedal/beater groups are deliberately outside these batches. */
function batchFixedSupports(root: THREE.Group): void {
  const buckets = new Map<THREE.Material, THREE.BufferGeometry[]>();
  root.updateMatrixWorld(true);
  for (const part of root.children) {
    if (!part.userData.segment) continue;
    part.userData.renderBounds = new THREE.Box3().setFromObject(part, true).toJSON();
    for (const node of [...part.children]) {
      if (!(node instanceof THREE.Mesh) || Array.isArray(node.material)) continue;
      const geometry = node.geometry.clone().applyMatrix4(node.matrixWorld);
      const bucket = buckets.get(node.material) ?? []; bucket.push(geometry); buckets.set(node.material, bucket);
      node.geometry.dispose(); part.remove(node);
    }
  }
  for (const [material, geometries] of buckets) {
    const merged = mergeGeometries(geometries, false);
    geometries.forEach(geometry => geometry.dispose());
    if (!merged) throw new Error('Cannot batch fixed hardware');
    const mesh = new THREE.Mesh(merged, material); mesh.name = 'FixedHardwareBatch';
    mesh.castShadow = mesh.receiveShadow = true; mesh.userData.hitResponseRole = 'hardware'; root.add(mesh);
  }
}

/** Requires the shared WASM runtime to have initialized. Routing, body clearance
 * and unresolved results have one implementation in Rust. */
export function solveKitGeometry(kit: readonly KitPiece[]): KitGeometry {
  return JSON.parse(kit_geometry(JSON.stringify(kit))) as KitGeometry;
}

/** Render semantic hardware inside the exact routed collision primitives. */
export function createPieceHardware(piece: KitPiece, kit: readonly KitPiece[], geometry = solveKitGeometry(kit)): THREE.Group {
  const root = new THREE.Group(); root.name = `Hardware/${piece.id}`;
  root.userData.pieceId = piece.id; root.userData.hitResponseRole = 'hardware';
  root.userData.anchors = pieceAnchors(piece);
  root.userData.unresolved = geometry.unresolvedPieceIds.some(id => canonical(id) === piece.id);
  const metal = new THREE.MeshStandardMaterial({ color: 0x9ca0a3, metalness: .76, roughness: .3 });
  const dark = new THREE.MeshStandardMaterial({ color: 0x26282b, metalness: .15, roughness: .7 });
  const rubber = new THREE.MeshStandardMaterial({ color: 0x17191b, metalness: 0, roughness: .95 });
  const add = (shape: THREE.BufferGeometry, material: THREE.Material, center: Vec3) => {
    const mesh = new THREE.Mesh(shape, material); mesh.position.set(...center);
    mesh.castShadow = mesh.receiveShadow = true; root.add(mesh); return mesh;
  };
  for (const [name, point] of Object.entries(pieceAnchors(piece))) {
    const anchor = new THREE.Object3D(); anchor.name = `${name}Anchor`; anchor.position.set(...point as Vec3); root.add(anchor);
  }
  for (const [index, segment] of geometry.supports.entries()) {
    if (canonical(segment.pieceId) !== piece.id) continue;
    const part = createHardwarePart(segment, { chrome: metal, dark, rubber });
    part.name = `support/${segment.role ?? 'rod'}/${index}`; root.add(part);
  }
  for (const [index, pedal] of geometry.pedals.entries()) {
    if (canonical(pedal.pieceId) !== piece.id) continue;
    const mesh = add(new THREE.BoxGeometry(...pedal.halfExtents.map(value => value * 2) as Vec3), dark, pedal.center);
    if (pedal.rotation) mesh.quaternion.fromArray(pedal.rotation);
    mesh.name = pedal.kind === 'board' ? pedal.pieceId === 'kick' ? 'contactSurface' : 'pedalContactSurface' : `pedalBase/${index}`;
    mesh.userData.pieceId = pedal.pieceId; mesh.userData.hitResponseRole = pedal.kind === 'board' ? 'pedal' : 'hardware';
    mesh.userData.fixedContact = true; mesh.userData.box = structuredClone(pedal);
  }
  const mechanism = geometry.mechanisms?.find(value => value.pieceId === piece.id);
  if (mechanism) {
    const board = root.children.find(node => node.userData.box?.kind === 'board');
    if (board) {
      const pivot = new THREE.Group(); pivot.name = `${piece.id}/FootboardPivot`; pivot.userData.mechanismRole = 'footboardPivot'; pivot.userData.hitResponseRole = 'hardware';
      pivot.position.set(...mechanism.pivot); if (mechanism.boardRotation) pivot.quaternion.fromArray(mechanism.boardRotation);
      root.add(pivot); root.updateMatrixWorld(true); pivot.attach(board);
    }
    if (mechanism.kind === 'kick') {
      const center = new THREE.Vector3(...mechanism.contact).addScaledVector(new THREE.Vector3(...mechanism.normal), .028);
      const parts = root.children.filter(node => {
        const segment = node.userData.segment;
        return segment && new THREE.Vector3(...segment.end).distanceTo(center) < 1e-7 &&
          (new THREE.Vector3(...segment.start).distanceTo(center) < 1e-7 || new THREE.Vector3(...segment.start).distanceTo(new THREE.Vector3(...mechanism.axle)) < 1e-7);
      });
      const pivot = new THREE.Group(); pivot.name = 'KickBeaterPivot'; pivot.userData.mechanismRole = 'beaterPivot'; pivot.userData.hitResponseRole = 'hardware';
      pivot.position.set(...mechanism.axle); root.add(pivot); root.updateMatrixWorld(true); parts.forEach(node => pivot.attach(node));
      pivot.userData.contact = mechanism.contact;
    }
  }
  root.userData.supportAssembly = pieceInstrument(piece) === 'tomHigh' || pieceInstrument(piece) === 'tomMid' ? 'bass-mounted-tom'
    : pieceInstrument(piece) === 'tomLow' ? piece.mount ? 'stand-mounted-tom' : 'floor-tom-legs' : pieceInstrument(piece) === 'hihat' ? 'direct-hihat'
      : pieceInstrument(piece) === 'snare' ? 'snare-basket' : pieceInstrument(piece) === 'kick' ? 'bass-pedal' : 'boom-cymbal';
  batchFixedSupports(root);
  const usedMaterials = new Set<THREE.Material>();
  root.traverse(node => { if (node instanceof THREE.Mesh) (Array.isArray(node.material) ? node.material : [node.material]).forEach(value => usedMaterials.add(value)); });
  [metal, dark, rubber].filter(material => !usedMaterials.has(material)).forEach(material => material.dispose());
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
  /** Finish and hit-feedback edits retain object identity when the shared
   * physical shapes have not changed. */
  setGeometry(kit: readonly KitPiece[], geometry: KitGeometry): boolean {
    const next = JSON.stringify(geometry);
    if (this.signature === next) return false;
    const replacement = new THREE.Group();
    for (const piece of kit) if (canonical(piece.id) === piece.id) replacement.add(createPieceHardware(piece, kit, geometry));
    dispose(this.root); this.root.add(...[...replacement.children]); this.signature = next;
    this.root.userData.geometry = structuredClone(geometry); return true;
  }
  /** Synchronous convenience for initialized runtimes and focused tests. */
  setLayout(kit: readonly KitPiece[]): boolean { return this.setGeometry(kit, solveKitGeometry(kit)); }
  clear(): void { dispose(this.root); this.signature = ''; delete this.root.userData.geometry; }
  dispose(): void { this.clear(); }
}
