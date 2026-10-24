import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import type { Instrument, Vec3 } from './types';

type LibraryPiece = Exclude<Instrument, 'openHat'>;
export type LibraryKit = Partial<Record<Instrument, THREE.Group>>;

const CONTACT_MESH: Record<LibraryPiece, string> = {
  kick: 'Bass pedal footboard', snare: '14 inch snare upper membrane',
  hihat: '14 inch upper hi-hat thin bronze cymbal', pedalHat: 'Hi-hat foot pedal',
  tomHigh: '10 inch mounted tom upper membrane', tomMid: '12 inch mounted tom upper membrane',
  tomLow: '16 inch floor tom upper membrane', crash: '16 inch crash thin bronze cymbal',
  ride: '20 inch ride thin bronze cymbal',
};

// This adapter is specific to the pinned, unmodified CC0 source. The engine
// never depends on these names. Throne meshes are omitted: the rig owns seating.
function instrumentFor(name: string): LibraryPiece | null {
  if (name.startsWith('10 inch mounted tom')) return 'tomHigh';
  if (name.startsWith('12 inch mounted tom')) return 'tomMid';
  if (name.startsWith('14 inch snare') || name.startsWith('Snare ')) return 'snare';
  if (name.startsWith('16 inch floor tom') || name.startsWith('Floor tom ')) return 'tomLow';
  if (name.startsWith('16 inch crash')) return 'crash';
  if (name.startsWith('20 inch ride')) return 'ride';
  if (name === 'Hi-hat foot pedal' || name === 'Hi-hat pedal linkage') return 'pedalHat';
  if (name.includes('hi-hat') || name.startsWith('Hi-hat ')) return 'hihat';
  if (name.startsWith('22 inch bass drum') || name.startsWith('Bass ')) return 'kick';
  if (name.startsWith('Attached exterior tom L mount') || name.startsWith('Tom shell receiver bracket')) {
    return name.endsWith('.001') ? 'tomMid' : 'tomHigh';
  }
  return null;
}

function reverseWinding(geometry: THREE.BufferGeometry): void {
  const index = geometry.index;
  if (!index) throw new Error('The pinned drum library requires indexed geometry.');
  for (let i = 0; i < index.count; i += 3) {
    const b = index.getX(i + 1);
    index.setX(i + 1, index.getX(i + 2));
    index.setX(i + 2, b);
  }
}

/** World-coordinate contact on the actual head/bow/footboard surface. */
function contactOn(mesh: THREE.Mesh, cymbal: boolean): Vec3 {
  mesh.geometry.computeBoundingBox();
  const box = mesh.geometry.boundingBox!;
  const center = box.getCenter(new THREE.Vector3());
  if (cymbal) center.z -= (box.max.x - box.min.x) * .3;
  const ray = new THREE.Raycaster(new THREE.Vector3(center.x, box.max.y + 1, center.z), new THREE.Vector3(0, -1, 0));
  const hit = ray.intersectObject(mesh, false)[0];
  if (!hit) throw new Error(`Drum library contact surface missing: ${mesh.name}`);
  return hit.point.toArray() as Vec3;
}

// The source embeds nine 2048² PNGs. Retain its unmodified source on disk but
// resample runtime maps before GPU upload (512², about 12 MiB including mipmaps).
function reduceRuntimeMaps(materials: Set<THREE.Material>): void {
  const seen = new Set<THREE.Texture>();
  for (const material of materials) {
    if (!(material instanceof THREE.MeshStandardMaterial)) continue;
    const map = material.map;
    if (!map || seen.has(map)) continue;
    seen.add(map);
    const image = map.image as CanvasImageSource & { width: number; height: number; close?: () => void };
    if (!image || Math.max(image.width, image.height) <= 512) continue;
    const scale = 512 / Math.max(image.width, image.height);
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(image.width * scale));
    canvas.height = Math.max(1, Math.round(image.height * scale));
    const context = canvas.getContext('2d');
    if (!context) continue;
    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    map.image = canvas;
    map.needsUpdate = true;
    image.close?.();
  }
}

/**
 * Returns independently placeable instruments. Geometry is expressed in the
 * source layout, converted to +Y up/+Z performer forward, with identity roots.
 * Set group.position = plannedTarget - group.userData.contact to align a part.
 * All groups belong to the caller; dispose their geometries/materials/textures.
 */
export async function loadLibraryKit(): Promise<LibraryKit> {
  const gltf = await new GLTFLoader().loadAsync(`${import.meta.env.BASE_URL}models/innerscene-drum-kit.glb`);
  return buildLibraryKit(gltf.scene);
}

/** Adapt the pinned source scene; also used by the asset/contact regression test. */
export function buildLibraryKit(scene: THREE.Object3D): LibraryKit {
  scene.updateMatrixWorld(true);
  const groups: LibraryKit = {};
  const batches = new Map<LibraryPiece, Map<THREE.Material, THREE.BufferGeometry[]>>();
  const originals = new Set<THREE.BufferGeometry>();
  const sourceMaterials = new Set<THREE.Material>();
  const reflection = new THREE.Matrix4().makeScale(1, 1, -1);
  try {
    scene.traverse(node => {
      if (!(node instanceof THREE.Mesh)) return;
      originals.add(node.geometry);
      const materials = Array.isArray(node.material) ? node.material : [node.material];
      materials.forEach(material => sourceMaterials.add(material));
      // GLTFLoader sanitizes names. glTF extras preserve the source name here.
      const name = node.userData.name ?? node.name.replaceAll('_', ' ');
      const instrument = instrumentFor(name);
      if (!instrument) return;
      if (materials.length !== 1) throw new Error('Unexpected multi-material drum library mesh.');
      const geometry = node.geometry.clone();
      geometry.applyMatrix4(new THREE.Matrix4().multiplyMatrices(reflection, node.matrixWorld));
      reverseWinding(geometry);
      const group = groups[instrument] ??= new THREE.Group();
      group.name = `Innerscene/${instrument}`;
      group.userData.sourceNames ??= [];
      group.userData.sourceNames.push(name);
      group.userData.asset = 'innerscene-five-piece-daff10f4';
      if (name === CONTACT_MESH[instrument]) {
        const surface = new THREE.Mesh(geometry, materials[0]);
        surface.name = name;
        group.userData.contact = contactOn(surface, ['hihat', 'crash', 'ride'].includes(instrument));
      }
      const byMaterial = batches.get(instrument) ?? new Map<THREE.Material, THREE.BufferGeometry[]>();
      batches.set(instrument, byMaterial);
      const geometries = byMaterial.get(materials[0]) ?? [];
      geometries.push(geometry);
      byMaterial.set(materials[0], geometries);
    });
    reduceRuntimeMaps(sourceMaterials);
    for (const [instrument, byMaterial] of batches) {
      const group = groups[instrument]!;
      if (!group.userData.contact) throw new Error(`Drum library contact is absent: ${instrument}`);
      for (const [material, geometries] of byMaterial) {
        const merged = mergeGeometries(geometries, false);
        if (!merged) throw new Error(`Cannot merge drum library meshes: ${instrument}`);
        const mesh = new THREE.Mesh(merged, material.clone());
        mesh.castShadow = mesh.receiveShadow = true;
        group.add(mesh);
      }
    }
    if (Object.keys(groups).length !== Object.keys(CONTACT_MESH).length) throw new Error('Incomplete drum library asset.');
    return groups;
  } catch (error) {
    const textures = new Set<THREE.Texture>();
    sourceMaterials.forEach(material => {
      if (material instanceof THREE.MeshStandardMaterial && material.map) textures.add(material.map);
    });
    textures.forEach(texture => texture.dispose());
    Object.values(groups).forEach(group => group.traverse(node => {
      if (node instanceof THREE.Mesh) {
        node.geometry.dispose();
        const materials = Array.isArray(node.material) ? node.material : [node.material];
        materials.forEach(material => material.dispose());
      }
    }));
    throw error;
  } finally {
    originals.forEach(geometry => geometry.dispose());
    sourceMaterials.forEach(material => material.dispose());
    batches.forEach(batch => batch.forEach(geometries => geometries.forEach(geometry => geometry.dispose())));
  }
}
