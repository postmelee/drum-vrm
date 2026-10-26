import * as THREE from 'three';
import { pieceInstrument, type KitPiece, type PieceGeometry, type Vec3 } from './types';
import { cloneModel, createPieceModel, disposeModel, type KitPresetId, type PieceModel, type PieceVisualSettings } from './model-procedural';
import { pieceAnchors } from './kit-hardware';

/** Normalize a replacement body around shared kit frames, without importing
 * its authored scene placement, stand, pedal, or avatar-specific assumptions. */
export function alignInstrumentBody(display: THREE.Group, pivot: THREE.Group, piece: KitPiece, settings: PieceVisualSettings): void {
  const fitScale = piece.radius / (piece.geometry?.referenceRadius ?? piece.radius);
  if (fitScale !== 1) piece = { ...piece, radius: piece.geometry!.referenceRadius };
  const applyReferenceScale = () => {
    if (fitScale !== 1) {
      const origin = new THREE.Vector3(...piece.position);
      pivot.position.sub(origin).multiplyScalar(fitScale).add(origin); pivot.scale.multiplyScalar(fitScale);
      pivot.updateMatrixWorld(true);
    }
  };
  display.updateMatrixWorld(true);
  const bounds = new THREE.Box3().setFromObject(display), size = bounds.getSize(new THREE.Vector3()), center = bounds.getCenter(new THREE.Vector3());
  const kick = pieceInstrument(piece) === 'kick', anchors = pieceAnchors(piece);
  const fit = (kick ? piece.radius * 2 / Math.max(size.y, .0001) : piece.radius * 2 / Math.max(size.x, size.z, .0001)) * settings.scale;
  display.position.set(-center.x, kick ? -center.y : 0, -center.z);
  pivot.position.set(anchors.mount[0], kick ? anchors.mount[1] : 0, anchors.mount[2]);
  pivot.scale.setScalar(fit); pivot.rotation.y = THREE.MathUtils.degToRad(settings.rotation);
  pivot.updateMatrixWorld(true);
  if (kick) {
    // Center on the actual transformed vertices, including asymmetric imported
    // shells; using transformed source boxes can drift after a yaw rotation.
    const actual = new THREE.Box3().setFromObject(display, true).getCenter(new THREE.Vector3());
    pivot.position.add(new THREE.Vector3(...anchors.mount).sub(actual)); pivot.updateMatrixWorld(true);
    applyReferenceScale(); return; // Foot contact belongs to the shared pedal.
  }
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
  applyReferenceScale();
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

/** One body-building path serves display, project preflight and numeric geometry. */
export function createMeasuredInstrumentBody(piece: KitPiece, settings: PieceVisualSettings, preset: KitPresetId, kit: readonly KitPiece[], source?: THREE.Group, imported = false): PieceModel & { geometry: PieceGeometry } {
  const built = createPieceModel(piece, settings, preset, kit, false);
  try {
    if (source) {
      built.ring.removeFromParent(); disposeModel(built.root); built.cymbal = undefined;
      const display = imported ? cloneModel(source) : cloneLibraryBody(source), pivot = new THREE.Group(); pivot.add(display); built.root.add(pivot, built.ring);
      alignInstrumentBody(display, pivot, piece, settings);
      if (settings.finishOverride) display.traverse(node => {
        if (node instanceof THREE.Mesh) for (const material of Array.isArray(node.material) ? node.material : [node.material]) if (material instanceof THREE.MeshStandardMaterial) material.color.set(settings.finish);
      });
    }
    return { ...built, geometry: measureInstrumentGeometry(built.root, piece, source ? 'mesh-bounds' : 'procedural') };
  } catch (error) { disposeModel(built.root); throw error; }
}

/** Measure the displayed, static body before binding cosmetic hit response.
 * Coordinates are relative to the kit's planning frame, before display reflection.
 * All drawn vertices contribute; markers, pedals and imported hardware do not. */
export function measureInstrumentGeometry(root: THREE.Object3D, piece: KitPiece, source: PieceGeometry['source']): PieceGeometry {
  root.updateWorldMatrix(true, true);
  const frame = root.parent?.matrixWorld.clone().invert() ?? new THREE.Matrix4();
  const bounds = new THREE.Box3(), surfaceBounds = new THREE.Box3();
  const surfaces: THREE.Mesh[] = [], bodies: THREE.Mesh[] = [];
  const point = new THREE.Vector3(), transform = new THREE.Matrix4();
  root.traverse(node => {
    if (!(node instanceof THREE.Mesh)) return;
    let surface = node.name === 'contactSurface';
    for (let ancestor: THREE.Object3D | null = node; ancestor; ancestor = ancestor.parent) {
      const role = ancestor.userData.hitResponseRole;
      if (!ancestor.visible || ['marker', 'hardware', 'pedal'].includes(role) || ancestor.userData.fixedContact) return;
      surface ||= role === 'surface';
      if (ancestor === root) break;
    }
    const position = node.geometry.getAttribute('position');
    if (!position) return;
    bodies.push(node); if (surface) surfaces.push(node);
    transform.multiplyMatrices(frame, node.matrixWorld);
    const index = node.geometry.index, count = index?.count ?? position.count;
    const start = Math.max(0, node.geometry.drawRange.start), end = Math.min(count, start + node.geometry.drawRange.count);
    for (let i = start; i < end; i++) {
      point.fromBufferAttribute(position, index ? index.getX(i) : i).applyMatrix4(transform);
      if (![point.x, point.y, point.z].every(Number.isFinite)) throw new Error('드럼 모델의 형상 좌표가 잘못되었습니다.');
      bounds.expandByPoint(point); if (surface) surfaceBounds.expandByPoint(point);
    }
  });
  if (bounds.isEmpty()) throw new Error(`드럼 본체 형상을 측정하지 못했습니다: ${piece.label}`);
  const referenceRadius = piece.geometry?.referenceRadius ?? piece.radius, ratio = piece.radius / referenceRadius;
  const origin = new THREE.Vector3(...piece.position);
  const offset = (value: THREE.Vector3): Vec3 => value.sub(origin).divideScalar(ratio).toArray() as Vec3;
  const size = bounds.getSize(new THREE.Vector3());
  const playing = surfaceBounds.isEmpty() ? bounds : surfaceBounds;
  const playingCenter = playing.getCenter(new THREE.Vector3()), playingSize = playing.getSize(new THREE.Vector3());
  playingCenter.y = piece.position[1];
  const kick = pieceInstrument(piece) === 'kick';
  const normal = new THREE.Vector3(0, 1, 0);
  if (!kick) {
    const toWorld = root.parent?.matrixWorld ?? new THREE.Matrix4();
    const ray = new THREE.Raycaster(new THREE.Vector3(piece.position[0], bounds.max.y + 1, piece.position[2]).applyMatrix4(toWorld), new THREE.Vector3(0, -1, 0).transformDirection(toWorld));
    const hit = ray.intersectObjects(surfaces.length ? surfaces : bodies, false)[0];
    if (hit?.face) {
      // Smooth authored normals keep the same contact stable when a translated
      // ray lands on either triangle of a mesh seam. Flat faces still use their
      // own normals; meshes without normal attributes use the face fallback.
      normal.copy(hit.normal ?? hit.face.normal).applyNormalMatrix(new THREE.Matrix3().getNormalMatrix(hit.object.matrixWorld)).applyNormalMatrix(new THREE.Matrix3().getNormalMatrix(frame));
      if (normal.y < 0) normal.negate();
      normal.normalize();
      const contact = hit.point.clone().applyMatrix4(frame);
      if (Math.abs(normal.y) > 1e-6) playingCenter.y = contact.y - (normal.x * (playingCenter.x - contact.x) + normal.z * (playingCenter.z - contact.z)) / normal.y;
      else playingCenter.copy(contact);
    }
  }
  const measured: PieceGeometry = {
    version: 1, referenceRadius, source,
    body: { center: offset(bounds.getCenter(new THREE.Vector3())), halfExtents: size.multiplyScalar(.5 / ratio).toArray().map(value => Math.max(value, .001)) as Vec3 },
    surface: kick ? structuredClone(piece.geometry?.surface ?? { center: [0, 0, -.02], normal: [0, 1, 0], radius: .05 })
      : { center: offset(playingCenter), normal: normal.toArray() as Vec3, radius: Math.min(playingSize.x, playingSize.z) / 2 / ratio },
  };
  // Rebuilding the same appearance after a fitted radius change must retain its
  // serialized reference shape instead of accumulating floating-point drift.
  const previous = piece.geometry;
  const numbers = (value: PieceGeometry) => [value.referenceRadius, ...value.body.center, ...value.body.halfExtents, ...value.surface.center, ...value.surface.normal, value.surface.radius];
  if (previous?.source === source && numbers(previous).every((value, index) => Math.abs(value - numbers(measured)[index]) < 1e-6)) return structuredClone(previous);
  return measured;
}
