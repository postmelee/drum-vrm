import { msg } from './i18n';
import * as THREE from 'three';
import { pieceInstrument, type PieceId, type KitPiece } from './types';
import { createPieceHardware, pieceAnchors } from './kit-hardware';

export type KitPresetId = 'studio' | 'vintage' | 'electronic' | 'library' | 'batoski' | 'smoj';
export type PieceModelId = 'standard' | 'deep' | 'compact';
export interface PieceVisualOptions { finish?: string; model?: PieceModelId; scale?: number; rotation?: number; tilt?: number; }
export interface PieceVisualSettings { finish: string; model: PieceModelId; scale: number; rotation: number; tilt: number; finishOverride?: boolean; source?: 'procedural' | 'library' | 'imported'; customModel?: string; approximateCalibration?: boolean; }
export const KIT_PRESETS = [
  { id: 'studio', label: 'Studio Acoustic' }, { id: 'vintage', label: 'Vintage Maple' },
  { id: 'electronic', label: 'Electronic' }, { id: 'library', label: 'Innerscene Acoustic · CC0' },
  { id: 'batoski', label: 'Batoski Acoustic · CC BY 3.0' }, { id: 'smoj', label: 'smoj Low-poly · CC BY 3.0' },
] as const;
export const canonicalPieceId = (id: PieceId): PieceId => id === 'openHat' || id === 'pedalHat' ? 'hihat' : id;
export function resolvePieceVisual(preset: KitPresetId, id: PieceId, override: PieceVisualOptions = {}): PieceVisualSettings {
  const cymbal = ['hihat', 'crash', 'crashRight', 'ride'].includes(canonicalPieceId(id));
  const defaults = {
    studio: { finish: cymbal ? '#ba995b' : '#e1e1dd', model: 'standard' as const },
    vintage: { finish: cymbal ? '#a18346' : '#8e4d2e', model: 'deep' as const },
    electronic: { finish: '#33363a', model: 'compact' as const },
    library: { finish: '#9b582f', model: 'standard' as const },
    batoski: { finish: '#9b582f', model: 'standard' as const },
    smoj: { finish: '#8d4554', model: 'standard' as const },
  }[preset];
  const scale = override.scale ?? 1;
  if (!Number.isFinite(scale) || scale < .65 || scale > 1.4) throw new Error(msg("Scale must be between 65% and 140%.", "크기는 65%–140% 범위여야 합니다."));
  if (override.finish !== undefined && !/^#[0-9a-f]{6}$/i.test(override.finish)) throw new Error(msg("Finish color must use #RRGGBB format.", "마감 색상은 #RRGGBB 형식이어야 합니다."));
  if (override.model !== undefined && !['standard', 'deep', 'compact'].includes(override.model)) throw new Error(msg("Unsupported drum shape.", "지원하지 않는 드럼 형태입니다."));
  const rotation = override.rotation ?? 0;
  if (!Number.isFinite(rotation) || rotation < -360 || rotation > 360) throw new Error(msg("Rotation must be between -360° and 360°.", "회전은 -360°–360° 범위여야 합니다."));
  const instrument = canonicalPieceId(id);
  const tilt = override.tilt ?? (instrument === 'crash' || instrument === 'crashRight' ? 12 : instrument === 'ride' ? 10 : instrument === 'tomHigh' || instrument === 'tomMid' ? 8 : instrument === 'snare' ? 4 : 0);
  if (!Number.isFinite(tilt) || tilt < -30 || tilt > 30) throw new Error(msg("Tilt must be between -30° and 30°.", "기울기는 -30°–30° 범위여야 합니다."));
  return { ...defaults, ...override, scale, rotation, tilt };
}

export function disposeModel(root: THREE.Object3D): void {
  const geometries = new Set<THREE.BufferGeometry>();
  const materials = new Set<THREE.Material>();
  const textures = new Set<THREE.Texture>();
  root.traverse(node => {
    if (!(node instanceof THREE.Mesh || node instanceof THREE.Line || node instanceof THREE.LineSegments)) return;
    geometries.add(node.geometry);
    for (const mat of Array.isArray(node.material) ? node.material : [node.material]) {
      materials.add(mat);
      Object.values(mat).forEach(value => { if (value instanceof THREE.Texture) textures.add(value); });
    }
  });
  textures.forEach(value => value.dispose()); geometries.forEach(value => value.dispose()); materials.forEach(value => value.dispose()); root.clear();
}
/** Final cached-source disposal only: render clones share these decoded images. */
export function disposeModelSource(root: THREE.Object3D): void {
  const images = new Set<{ close: () => void }>();
  const collect = (value: unknown): void => {
    if (Array.isArray(value)) { value.forEach(collect); return; }
    if (value && typeof value === 'object' && 'close' in value && typeof value.close === 'function') images.add(value as { close: () => void });
  };
  root.traverse(node => {
    if (!(node instanceof THREE.Mesh)) return;
    for (const material of Array.isArray(node.material) ? node.material : [node.material]) {
      Object.values(material).forEach(value => { if (value instanceof THREE.Texture) collect(value.image); });
    }
  });
  disposeModel(root);
  images.forEach(image => image.close());
}
export function cloneModel(root: THREE.Group): THREE.Group {
  const clone = root.clone(true);
  clone.traverse(node => {
    if (!(node instanceof THREE.Mesh)) return;
    node.geometry = node.geometry.clone();
    const copy = (source: THREE.Material) => {
      const mat = source.clone();
      for (const [key, value] of Object.entries(mat)) {
        if (value instanceof THREE.Texture) (mat as unknown as Record<string, unknown>)[key] = value.clone();
      }
      return mat;
    };
    node.material = Array.isArray(node.material) ? node.material.map(copy) : copy(node.material);
  });
  return clone;
}

const mat = (color: THREE.ColorRepresentation, metalness = 0, roughness = .6) => new THREE.MeshStandardMaterial({ color, metalness, roughness });
function mesh(root: THREE.Object3D, geometry: THREE.BufferGeometry, material: THREE.Material) {
  const value = new THREE.Mesh(geometry, material); root.add(value); return value;
}
function cylinder(root: THREE.Object3D, radius: number, height: number, material: THREE.Material, segments = 48) {
  return mesh(root, new THREE.CylinderGeometry(radius, radius, height, segments), material);
}
export interface PieceModel { root: THREE.Group; ring: THREE.Mesh<THREE.RingGeometry, THREE.MeshBasicMaterial>; cymbal?: THREE.Group; }

/** Procedural models keep the canonical playing surface fixed while changing their silhouette. */
export function createPieceModel(piece: KitPiece, visual: PieceVisualSettings, preset: KitPresetId, kit: readonly KitPiece[], includeHardware = true): PieceModel {
  const currentPiece = piece;
  const fitScale = piece.radius / (piece.geometry?.referenceRadius ?? piece.radius);
  // A measured reference shape scales as a whole during body fitting; shell
  // depth, rims and lugs must follow the same radius ratio as collision bounds.
  if (fitScale !== 1) piece = { ...piece, radius: piece.geometry!.referenceRadius };
  const root = new THREE.Group(); root.name = piece.id; root.userData.pieceId = piece.id;
  const finish = (model: PieceModel): PieceModel => {
    if (visual.rotation || visual.tilt) {
      // Tilt around the canonical contact so the measured playing point remains
      // fixed. Bodies and their semantic mounts share this transformed frame.
      const pivot = new THREE.Group(); pivot.position.set(...(pieceInstrument(piece) === 'kick' ? pieceAnchors(piece).mount : piece.position));
      pivot.rotation.set(-THREE.MathUtils.degToRad(visual.tilt), THREE.MathUtils.degToRad(visual.rotation), 0);
      const body = new THREE.Group(); body.position.copy(pivot.position).multiplyScalar(-1);
      [...root.children].filter(child => child.userData.hitResponseRole !== 'marker' && child.userData.hitResponseRole !== 'hardware').forEach(child => body.add(child));
      pivot.add(body); root.add(pivot);
    }
    if (fitScale !== 1) {
      const pivot = new THREE.Group(); pivot.position.set(...piece.position); pivot.scale.setScalar(fitScale);
      const body = new THREE.Group(); body.position.copy(pivot.position).multiplyScalar(-1);
      [...root.children].filter(child => child.userData.hitResponseRole !== 'hardware').forEach(child => body.add(child));
      pivot.add(body); root.add(pivot);
    }
    return model;
  };
  const [x, y, z] = piece.position, instrument = pieceInstrument(piece);
  if (includeHardware) root.add(createPieceHardware(currentPiece, kit));
  const radius = piece.radius * visual.scale;
  const electronic = preset === 'electronic';
  const shell = mat(visual.finish, electronic ? .05 : .16, electronic ? .83 : .4);
  const metal = mat(preset === 'vintage' ? 0x777876 : 0x9ca0a3, .76, .3);
  const dark = mat(0x26282b, .15, .7);
  const head = mat(electronic ? 0x303238 : 0xd9dbda, .02, .82);
  const modelDepth = visual.model === 'deep' ? 1.38 : visual.model === 'compact' ? .53 : 1;
  const ringMat = new THREE.MeshBasicMaterial({ color: 0x388bfd, transparent: true, opacity: 0, side: THREE.DoubleSide, depthWrite: false });
  const ring = new THREE.Mesh(new THREE.RingGeometry(radius * .81, radius * .94, 48), ringMat);
  ring.userData.hitResponseRole = 'marker';
  ring.rotation.x = -Math.PI / 2; ring.position.set(x, y + .008, z); ring.visible = false; root.add(ring);
  if (instrument === 'kick') {
    const bass = new THREE.Group(); bass.userData.hitResponseRole = 'shell'; root.add(bass);
    const drumRadius = electronic ? Math.min(.17, piece.radius) * visual.scale : piece.radius * visual.scale;
    const depth = electronic ? .075 : .4 * modelDepth;
    bass.position.set(...pieceAnchors(piece).mount); bass.rotation.x = Math.PI / 2;
    cylinder(bass, drumRadius, depth, shell);
    for (const side of [-1, 1]) {
      cylinder(bass, drumRadius + .012, .018, metal).position.y = side * depth / 2;
      cylinder(bass, drumRadius - .004, .008, dark).position.y = side * (depth / 2 + .012);
      if (!electronic) for (let i = 0; i < 10; i++) {
        const angle = i / 10 * Math.PI * 2;
        cylinder(bass, .009, .07, metal, 10).position.set(Math.sin(angle) * (drumRadius + .008), side * depth * .34, Math.cos(angle) * (drumRadius + .008));
      }
    }
    ring.geometry.dispose(); ring.geometry = new THREE.RingGeometry(.08, .10, 36); ring.position.set(x, y + .008, z);
    return finish({ root, ring });
  }
  if (['hihat', 'crash', 'ride'].includes(instrument)) {
    const mountZ = pieceAnchors(piece).mount[2];
    const cymbal = new THREE.Group(); cymbal.userData.hitResponseRole = 'surface'; root.add(cymbal);
    if (instrument === 'hihat' && !electronic) cymbal.userData.mechanismRole = 'hihatTop';
    if (electronic) {
      cymbal.position.set(x, y - .013, mountZ);
      const pad = cylinder(cymbal, radius, .026, shell); pad.name = 'contactSurface';
      const edge = mesh(cymbal, new THREE.TorusGeometry(radius, .008, 8, 64), dark); edge.rotation.x = Math.PI / 2;
    } else {
      // Width changes preserve the shared mount X/Z and the exact strike height.
      const fraction = (mountZ - z) / radius;
      const bowY = fraction <= .8 ? -.022 - (fraction - .24) / .56 * .016 : -.038 - (fraction - .8) / .2 * .006;
      cymbal.position.set(x, y - bowY, mountZ);
      shell.metalness = .78; shell.side = THREE.DoubleSide;
      const profile = [new THREE.Vector2(.008, -.004), new THREE.Vector2(radius * .12, 0), new THREE.Vector2(radius * .24, -.022), new THREE.Vector2(radius * .8, -.038), new THREE.Vector2(radius, -.044)];
      const bow = mesh(cymbal, new THREE.LatheGeometry(profile, 80), shell); bow.name = 'contactSurface';
      bow.userData.radialProfile = profile.map(point => [point.x, point.y]);
      if (instrument === 'hihat') {
        const lower = new THREE.Group(); lower.name = 'HiHatBottom'; lower.userData.hitResponseRole = 'surface'; lower.userData.mechanismRole = 'hihatBottom';
        lower.position.copy(cymbal.position); root.add(lower);
        const bottom = mesh(lower, new THREE.LatheGeometry(profile, 64), shell);
        // Bells face away; only the two broad rims face one another. Keep a
        // 2 mm resting rim gap, independent of the top cymbal's opening travel.
        bottom.rotation.x = Math.PI; bottom.position.y = -.090;
      }
      const etched = mat(0x88713f, .68, .46);
      for (const fraction of [.36, .52, .68, .84]) {
        const line = mesh(cymbal, new THREE.TorusGeometry(radius * fraction, .0006, 4, 64), etched);
        line.rotation.x = Math.PI / 2; line.position.y = -.022 - (fraction - .24) / .76 * .022;
      }
    }
    cylinder(cymbal, .015, .015, dark).position.y = electronic ? .022 : .011;
    ring.geometry.dispose(); ring.geometry = new THREE.RingGeometry(radius * .24, radius * .31, 40);
    return finish({ root, ring, cymbal });
  }
  const depth = electronic ? .04 : (instrument === 'snare' ? .13 : instrument === 'tomLow' ? .29 : .22) * modelDepth;
  const drum = new THREE.Group(); drum.userData.hitResponseRole = 'shell'; drum.position.set(x, y - depth / 2, z); root.add(drum);
  cylinder(drum, radius, depth, shell);
  const surface = cylinder(drum, radius - .007, .008, head); surface.position.y = depth / 2 - .004; surface.name = 'contactSurface';
  for (const side of [1, -1]) {
    const rim = mesh(drum, new THREE.TorusGeometry(radius, electronic ? .009 : .007, 8, 64), electronic ? dark : metal);
    rim.rotation.x = Math.PI / 2; rim.position.y = side * depth / 2 - .003;
  }
  if (!electronic) for (let i = 0; i < (preset === 'vintage' ? 6 : 8); i++) {
    const angle = i / (preset === 'vintage' ? 6 : 8) * Math.PI * 2;
    cylinder(drum, .009, depth * .7, metal, 10).position.set(Math.sin(angle) * (radius + .005), 0, Math.cos(angle) * (radius + .005));
  }
  return finish({ root, ring });
}
