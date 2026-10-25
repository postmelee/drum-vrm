import * as THREE from 'three';
import { pieceInstrument, type KitPiece } from './types';
import { cloneModel, disposeModel, type PieceVisualSettings } from './model-procedural';
import { pieceAnchors } from './kit-hardware';

/** Normalize a replacement body around shared kit frames, without importing
 * its authored scene placement, stand, pedal, or avatar-specific assumptions. */
export function alignInstrumentBody(display: THREE.Group, pivot: THREE.Group, piece: KitPiece, settings: PieceVisualSettings): void {
  display.updateMatrixWorld(true);
  const bounds = new THREE.Box3().setFromObject(display), size = bounds.getSize(new THREE.Vector3()), center = bounds.getCenter(new THREE.Vector3());
  const kick = pieceInstrument(piece) === 'kick', anchors = pieceAnchors(piece);
  const fit = (kick ? piece.radius * 2 / Math.max(size.y, .0001) : piece.radius * 2 / Math.max(size.x, size.z, .0001)) * settings.scale;
  display.position.set(-center.x, kick ? -center.y : 0, -center.z);
  pivot.position.set(anchors.mount[0], kick ? anchors.mount[1] : 0, anchors.mount[2]);
  pivot.scale.setScalar(fit); pivot.rotation.y = THREE.MathUtils.degToRad(settings.rotation);
  pivot.updateMatrixWorld(true);
  if (kick) return; // Its foot contact belongs to the shared pedal, not the shell.
  // Prefer authored playing surfaces. Unknown local bodies have no semantic
  // metadata, so use their top at the target and report approximate calibration.
  const surfaces: THREE.Object3D[] = [];
  display.traverse(node => { if (node instanceof THREE.Mesh && node.userData.hitResponseRole === 'surface') surfaces.push(node); });
  const meshes = surfaces.length ? surfaces : [display];
  const currentBounds = new THREE.Box3().setFromObject(display);
  const ray = new THREE.Raycaster(new THREE.Vector3(piece.position[0], currentBounds.max.y + 1, piece.position[2]), new THREE.Vector3(0, -1, 0));
  const intersection = ray.intersectObjects(meshes, true)[0];
  // A body with no surface beneath the shared strike point cannot be silently
  // declared exact. Local imports retain an explicit approximation warning.
  pivot.position.y = piece.position[1] - (intersection?.point.y ?? currentBounds.max.y);
  pivot.updateMatrixWorld(true);
  pivot.userData.approximateContact = !intersection;
}

/** Render only the replaceable instrument body. Original licensed binaries and
 * the source adapter inventory retain their hardware for provenance/testing. */
export function cloneLibraryBody(source: THREE.Group): THREE.Group {
  const body = cloneModel(source);
  const fixed: THREE.Object3D[] = [];
  body.traverse(node => {
    if (node.userData.hitResponseRole === 'hardware' || node.userData.hitResponseRole === 'pedal' || node.userData.fixedContact) fixed.push(node);
  });
  fixed.forEach(node => { node.removeFromParent(); disposeModel(node); });
  body.name = `${source.name}/body`; return body;
}
