import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import type { Instrument, Vec3 } from './types';

type LibraryPiece = Exclude<Instrument, 'openHat'>;
type ResponseRole = 'surface' | 'shell' | 'hardware' | 'pedal';
type Part = { instrument: LibraryPiece; role: ResponseRole; contact?: boolean };
export type LibraryPreset = 'library' | 'batoski' | 'smoj';
export type LibraryKit = Partial<Record<Instrument, THREE.Group>>;
const FILES: Record<LibraryPreset, string> = {
  library: 'innerscene-drum-kit.glb', batoski: 'batoski-drum-kit.glb', smoj: 'smoj-drum-kit.glb',
};
const CONTACT_MESH: Record<LibraryPiece, string> = {
  kick: 'Bass pedal footboard', snare: '14 inch snare upper membrane',
  hihat: '14 inch upper hi-hat thin bronze cymbal', pedalHat: 'Hi-hat foot pedal',
  tomHigh: '10 inch mounted tom upper membrane', tomMid: '12 inch mounted tom upper membrane',
  tomLow: '16 inch floor tom upper membrane', crash: '16 inch crash thin bronze cymbal',
  ride: '20 inch ride thin bronze cymbal',
};

// Asset-specific names stay outside the engine. Seating is owned by the rig.
function innerscenePart(name: string): Part | null {
  let instrument: LibraryPiece;
  if (name.startsWith('10 inch mounted tom')) instrument = 'tomHigh';
  else if (name.startsWith('12 inch mounted tom')) instrument = 'tomMid';
  else if (name.startsWith('14 inch snare') || name.startsWith('Snare ')) instrument = 'snare';
  else if (name.startsWith('16 inch floor tom') || name.startsWith('Floor tom ')) instrument = 'tomLow';
  else if (name.startsWith('16 inch crash')) instrument = 'crash';
  else if (name.startsWith('20 inch ride')) instrument = 'ride';
  else if (name === 'Hi-hat foot pedal' || name === 'Hi-hat pedal linkage') instrument = 'pedalHat';
  else if (name.includes('hi-hat') || name.startsWith('Hi-hat ')) instrument = 'hihat';
  else if (name.startsWith('22 inch bass drum') || name.startsWith('Bass ')) instrument = 'kick';
  else if (name.startsWith('Attached exterior tom L mount') || name.startsWith('Tom shell receiver bracket')) instrument = name.endsWith('.001') ? 'tomMid' : 'tomHigh';
  else return null;
  const contact = name === CONTACT_MESH[instrument];
  const role: ResponseRole = /pedal|beater/i.test(name) ? 'pedal'
    : /membrane|thin bronze cymbal|fine lathe groove/.test(name) ? 'surface'
    : /finite wood shell|bearing ring|lug casing|tension bolt|tension hoop|vent grommet/.test(name) ? 'shell' : 'hardware';
  return { instrument, role, contact };
}

const BATOSKI_PARTS: Record<string, Part> = {
  BigKick: { instrument: 'kick', role: 'shell' },
  Bigsnare: { instrument: 'snare', role: 'shell', contact: true },
  Reggirul: { instrument: 'snare', role: 'hardware' },
  BigTOM: { instrument: 'tomHigh', role: 'shell', contact: true },
  Reggitom: { instrument: 'tomHigh', role: 'hardware' },
  BigFL: { instrument: 'tomMid', role: 'shell', contact: true },
  BigFL_1: { instrument: 'tomLow', role: 'shell', contact: true },
  rEGGIhh: { instrument: 'hihat', role: 'hardware' },
  HHB: { instrument: 'hihat', role: 'surface' },
  HHT: { instrument: 'hihat', role: 'surface', contact: true },
  Crash: { instrument: 'crash', role: 'surface', contact: true },
  'Asta Crash': { instrument: 'crash', role: 'hardware' },
  Ride: { instrument: 'ride', role: 'surface', contact: true },
  'Asta Ride': { instrument: 'ride', role: 'hardware' },
  // The second crash and stool are intentionally omitted: no independent
  // second-crash event role exists in the current PerformancePlan.
};

// Pinned source-node inventory. Explicit semantic grouping avoids nearest-part
// heuristics changing meaning when a user moves or replaces an instrument.
const SMOJ_PARTS = new Map<string, Part>();
function smojNames(instrument: LibraryPiece, role: ResponseRole, ids: string): void {
  ids.split(' ').forEach(id => SMOJ_PARTS.set(`group${id}`, { instrument, role }));
}
smojNames('kick', 'surface', '887072338');
smojNames('kick', 'shell', '706437995 1998573136 2128802699 2013354032 1591534448 1451281793 410413010 350255547 56324186 878792710 535822007 513085455 2023158278 307779137 1375386455 360840042 1208045665 1557875796 1228051283 707175795 4380836 1533848445 1038593656 1694807479');
smojNames('kick', 'pedal', '1522356203 1465303332 1302850767 1611367333 218248327 1865635461 1821305895 1607619771');
smojNames('kick', 'hardware', '137165110 1299911273 630199045 2106758197 1297818961');
smojNames('tomLow', 'surface', '1564768936');
smojNames('tomLow', 'shell', '1333367943 760235922 469720825 28713638 656228341 73174683 1226633016 635239078 1889351565 1367814359 623216003 1440220504 848616315 967072075 1005381260 1645684424 708842017 459653258 466805100 722085133 419243393 1380473956 2095966557 296639090');
smojNames('tomLow', 'hardware', '1662138281 546876326 577289209');
smojNames('snare', 'surface', '282742422');
smojNames('snare', 'shell', '236626127 1640730452 853210252 565789196 681233847 634043653 1244698544 676900349 1591861425 942699307 635306850 363091015 1217487814 344498584 297875909 2057032849 444395174 1027453609 1061192834 2071522131 480157691 1104148678 2051415276 1528215653');
smojNames('snare', 'hardware', '1803570891 647981632 1055204630');
smojNames('crash', 'hardware', '1147216678 1227846528 1876241811');
smojNames('ride', 'hardware', '1635506047 661505037 1639056195 1526001106');
smojNames('ride', 'surface', '370074410');
smojNames('hihat', 'surface', '763320155 1404238537');
smojNames('hihat', 'hardware', '353020347');
const SMOJ_CONTACT: Partial<Record<LibraryPiece, string>> = {
  kick: 'group1607619771', snare: 'group282742422', tomLow: 'group1564768936',
  hihat: 'group1404238537', ride: 'group370074410',
};
function smojPart(name: string, material: THREE.Material): Part | null {
  if (name === 'group654266889' || name === 'group1478113385') return {
    instrument: name === 'group654266889' ? 'tomMid' : 'tomHigh',
    role: material.name === 'mat24' ? 'surface' : 'shell', contact: material.name === 'mat24',
  };
  if (name === 'group1320678639') return { instrument: 'crash', role: material.name === 'mat19' ? 'surface' : 'hardware', contact: material.name === 'mat19' };
  if (name === 'group664528598') return material.name === 'mat15'
    ? { instrument: 'hihat', role: 'hardware' }
    : { instrument: 'pedalHat', role: 'pedal', contact: material.name === 'mat23' };
  const part = SMOJ_PARTS.get(name);
  return part ? { ...part, contact: name === SMOJ_CONTACT[part.instrument] } : null;
}

function sourceName(node: THREE.Object3D, preset: LibraryPreset): string {
  for (let current: THREE.Object3D | null = node; current; current = current.parent) {
    const name = current.userData.name as string | undefined;
    if (name && (preset !== 'smoj' || /^group\d+$/.test(name))) return name;
  }
  throw new Error(`Unnamed source node in ${preset} drum library.`);
}

function reverseWinding(geometry: THREE.BufferGeometry): void {
  const index = geometry.index;
  if (!index) throw new Error('The pinned drum libraries require indexed geometry.');
  for (let i = 0; i < index.count; i += 3) {
    const b = index.getX(i + 1); index.setX(i + 1, index.getX(i + 2)); index.setX(i + 2, b);
  }
}

/** Surface contact is measured on geometry, never the mounting nut or a box top. */
function contactOn(mesh: THREE.Mesh, cymbal: boolean): Vec3 {
  mesh.geometry.computeBoundingBox();
  const box = mesh.geometry.boundingBox!, center = box.getCenter(new THREE.Vector3());
  if (cymbal) center.z -= (box.max.x - box.min.x) * .3;
  const ray = new THREE.Raycaster(new THREE.Vector3(center.x, box.max.y + 1, center.z), new THREE.Vector3(0, -1, 0));
  const hit = ray.intersectObject(mesh, false)[0];
  if (!hit) throw new Error(`Drum library contact surface missing: ${mesh.name}`);
  return hit.point.toArray() as Vec3;
}

// Batoski stores floor-tom legs and kick spurs in the drum meshes. Separate
// connected components using welded positions, so hardware never pulses.
function splitBatoskiHardware(geometry: THREE.BufferGeometry, name: string): [THREE.BufferGeometry, ResponseRole][] {
  if (!['BigKick', 'BigFL', 'BigFL_1'].includes(name)) return [[geometry, 'shell']];
  const positions = geometry.getAttribute('position'), indices = geometry.index!;
  const parents = Array.from({ length: positions.count }, (_, index) => index), welded = new Map<string, number>();
  const find = (index: number): number => parents[index] === index ? index : (parents[index] = find(parents[index]));
  const link = (a: number, b: number): void => { parents[find(a)] = find(b); };
  for (let i = 0; i < positions.count; i++) {
    const key = [positions.getX(i), positions.getY(i), positions.getZ(i)].map(value => value.toFixed(5)).join(',');
    const previous = welded.get(key); if (previous !== undefined) link(i, previous); else welded.set(key, i);
  }
  for (let i = 0; i < indices.count; i += 3) { link(indices.getX(i), indices.getX(i + 1)); link(indices.getX(i + 1), indices.getX(i + 2)); }
  const bounds = new Map<number, THREE.Box3>(), point = new THREE.Vector3();
  for (let i = 0; i < positions.count; i++) {
    const key = find(i), box = bounds.get(key) ?? new THREE.Box3(); bounds.set(key, box);
    box.expandByPoint(point.fromBufferAttribute(positions, i));
  }
  const shell: number[] = [], hardware: number[] = [];
  for (let i = 0; i < indices.count; i += 3) {
    const box = bounds.get(find(indices.getX(i)))!, size = box.getSize(new THREE.Vector3());
    const fixed = name === 'BigKick' ? size.y > .15 && box.max.y < .30 : size.y > .3 && size.x < .05 && size.z < .05;
    (fixed ? hardware : shell).push(indices.getX(i), indices.getX(i + 1), indices.getX(i + 2));
  }
  const result: [THREE.BufferGeometry, ResponseRole][] = [];
  for (const [list, role] of [[shell, 'shell'], [hardware, 'hardware']] as const) {
    if (list.length) { const indexed = geometry.clone(); indexed.setIndex(list); const part = indexed.toNonIndexed(); part.setIndex(Array.from({ length: part.getAttribute('position').count }, (_, index) => index)); indexed.dispose(); result.push([part, role]); }
  }
  geometry.dispose(); return result;
}

// Downsample the Innerscene source maps before GPU upload. The original binary
// is retained unchanged. Other source maps already meet this maximum size.
function reduceRuntimeMaps(materials: Set<THREE.Material>): void {
  const seen = new Set<THREE.Texture>();
  for (const material of materials) {
    if (!(material instanceof THREE.MeshStandardMaterial)) continue;
    const map = material.map; if (!map || seen.has(map)) continue; seen.add(map);
    const image = map.image as CanvasImageSource & { width: number; height: number; close?: () => void };
    if (!image || Math.max(image.width, image.height) <= 512) continue;
    const scale = 512 / Math.max(image.width, image.height), canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(image.width * scale)); canvas.height = Math.max(1, Math.round(image.height * scale));
    const context = canvas.getContext('2d'); if (!context) continue;
    context.drawImage(image, 0, 0, canvas.width, canvas.height); map.image = canvas; map.needsUpdate = true; image.close?.();
  }
}

/** Caller owns the returned groups. Scene aligns userData.contact to its target. */
export async function loadLibraryKit(preset: LibraryPreset = 'library'): Promise<LibraryKit> {
  const file = FILES[preset]; if (!file) throw new Error('Unknown drum library preset.');
  const gltf = await new GLTFLoader().loadAsync(`${import.meta.env.BASE_URL}models/${file}`);
  return buildLibraryKit(gltf.scene, preset);
}

/** Consume a pinned source scene; exposed for asset/contact regression tests. */
export function buildLibraryKit(scene: THREE.Object3D, preset: LibraryPreset = 'library'): LibraryKit {
  scene.updateMatrixWorld(true);
  const groups: LibraryKit = {};
  const batches = new Map<LibraryPiece, Map<ResponseRole, Map<THREE.Material, THREE.BufferGeometry[]>>>();
  const originals = new Set<THREE.BufferGeometry>(), sourceMaterials = new Set<THREE.Material>();
  const transform = new THREE.Matrix4().makeScale(...(preset === 'smoj' ? [-1.4, 1.4, 1.4] : preset === 'batoski' ? [.42, .42, -.42] : [1, 1, -1]) as Vec3);
  if (preset === 'smoj') transform.setPosition(0, .504, 0);
  if (preset === 'batoski') transform.setPosition(0, .004, 0);
  const add = (instrument: LibraryPiece, role: ResponseRole, geometry: THREE.BufferGeometry, material: THREE.Material, name: string, contact = false): void => {
    const group = groups[instrument] ??= new THREE.Group(); group.name = `${preset}/${instrument}`;
    group.userData.sourceNames ??= []; if (!group.userData.sourceNames.includes(name)) group.userData.sourceNames.push(name);
    group.userData.asset = preset;
    const byRole = batches.get(instrument) ?? new Map(); batches.set(instrument, byRole);
    const byMaterial = byRole.get(role) ?? new Map(); byRole.set(role, byMaterial);
    const geometries = byMaterial.get(material) ?? []; geometries.push(geometry); byMaterial.set(material, geometries);
    if (contact) {
      const surface = new THREE.Mesh(geometry, material); surface.name = name;
      group.userData.contact = contactOn(surface, ['hihat', 'crash', 'ride'].includes(instrument));
    }
  };
  try {
    scene.traverse(node => {
      if (!(node instanceof THREE.Mesh)) return;
      originals.add(node.geometry);
      const materials = Array.isArray(node.material) ? node.material : [node.material]; materials.forEach(material => sourceMaterials.add(material));
      if (materials.length !== 1) throw new Error('Unexpected multi-material primitive in drum library.');
      const name = sourceName(node, preset), material = materials[0];
      const part = preset === 'library' ? innerscenePart(name) : preset === 'batoski' ? BATOSKI_PARTS[name] : smojPart(name, material);
      if (!part) return;
      const geometry = node.geometry.clone(); geometry.applyMatrix4(new THREE.Matrix4().multiplyMatrices(transform, node.matrixWorld)); reverseWinding(geometry);
      const pieces: [THREE.BufferGeometry, ResponseRole][] = preset === 'batoski' && part.role === 'shell' ? splitBatoskiHardware(geometry, name) : [[geometry, part.role]];
      pieces.forEach(([piece, role]) => add(part.instrument, role, piece, material, name, part.contact && role !== 'hardware'));
    });
    // This source has no independent footboards. Supply original application
    // geometry, disclosed in provenance, rather than calibrating feet to shells.
    if (preset === 'batoski') for (const [instrument, x, z] of [['kick', .37, .11], ['pedalHat', -.54, -.20]] as const) {
      const material = new THREE.MeshStandardMaterial({ color: 0x30343a, metalness: .45, roughness: .45 }); sourceMaterials.add(material);
      const board = new THREE.BoxGeometry(.10, .025, .22).translate(x, .0675, z);
      add(instrument, 'pedal', board, material, 'Application footboard', true);
      const base = new THREE.BoxGeometry(.13, .025, .27).translate(x, .0175, z);
      add(instrument, 'hardware', base, material, 'Application pedal base');
    }
    reduceRuntimeMaps(sourceMaterials);
    for (const [instrument, byRole] of batches) {
      const group = groups[instrument]!;
      if (!group.userData.contact) throw new Error(`Drum library contact is absent: ${instrument}`);
      for (const [role, byMaterial] of byRole) for (const [material, geometries] of byMaterial) {
        const merged = mergeGeometries(geometries, false); if (!merged) throw new Error(`Cannot merge drum library: ${preset}/${instrument}/${role}`);
        const mesh = new THREE.Mesh(merged, material.clone()); mesh.castShadow = mesh.receiveShadow = true;
        mesh.name = `${instrument}/${role}/${material.name}`; mesh.userData.hitResponseRole = role; group.add(mesh);
      }
    }
    if (Object.keys(groups).length !== Object.keys(CONTACT_MESH).length) throw new Error('Incomplete drum library asset.');
    return groups;
  } catch (error) {
    const textures = new Set<THREE.Texture>();
    sourceMaterials.forEach(material => { if (material instanceof THREE.MeshStandardMaterial && material.map) textures.add(material.map); });
    textures.forEach(texture => texture.dispose());
    Object.values(groups).forEach(group => group.traverse(node => {
      if (node instanceof THREE.Mesh) { node.geometry.dispose(); (Array.isArray(node.material) ? node.material : [node.material]).forEach(material => material.dispose()); }
    }));
    throw error;
  } finally {
    originals.forEach(geometry => geometry.dispose()); sourceMaterials.forEach(material => material.dispose());
    batches.forEach(roles => roles.forEach(materials => materials.forEach(geometries => geometries.forEach(geometry => geometry.dispose()))));
  }
}
