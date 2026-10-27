import * as THREE from 'three';
import { FBXLoader } from 'three/addons/loaders/FBXLoader.js';
import { imageDimensions } from './model-image';
import { imageKey, type FBXConfig } from './fbx-config';
import { disposeFBX } from './fbx-rig';
import { msg } from './i18n';

export const MAX_FBX_BYTES = 50 * 1024 * 1024;
export const MAX_FBX_EXPANDED_BYTES = 128 * 1024 * 1024;
export const MAX_FBX_TEXTURE_BYTES = 50 * 1024 * 1024;
/** Validate binary records and declared array expansion before the loader allocates.
 * Only the two binary versions covered by our fixtures are accepted. */
export function inspectFBX(bytes: ArrayBuffer): { offset: number; bytes: number; expanded: number }[] {
  const fail = (): never => { throw new Error(msg('Invalid or oversized binary FBX. Export binary FBX 7.4 or 7.5.', '바이너리 FBX 형식 또는 크기가 올바르지 않습니다. 바이너리 FBX 7.4 또는 7.5로 내보내세요.')); };
  if (bytes.byteLength < 40 || bytes.byteLength > MAX_FBX_BYTES) fail();
  const view = new DataView(bytes), data = new Uint8Array(bytes), decoder = new TextDecoder();
  if (decoder.decode(data.subarray(0, 23)) !== 'Kaydara FBX Binary  \0\x1a\0') fail();
  const version = view.getUint32(23, true); if (version !== 7400 && version !== 7500) fail();
  const wide = version === 7500, head = wide ? 25 : 13;
  const uint = (p: number) => { const n = wide ? Number(view.getBigUint64(p, true)) : view.getUint32(p, true); if (!Number.isSafeInteger(n)) fail(); return n; };
  let expanded = 0, entries = 0, nodes = 0, images = 0, textureCount = 0, materialCount = 0;
  const arrays: { offset: number; bytes: number; expanded: number }[] = [];
  const models = new Set<number>(), links: [number, number][] = [];
  const node = (start: number, limit: number, depth: number): number => {
    if (depth > 128 || ++nodes > 100000 || start + head > limit) fail();
    const end = uint(start); if (!end) return start + head;
    const count = uint(start + (wide ? 8 : 4)), size = uint(start + (wide ? 16 : 8)), nameSize = data[start + head - 1];
    let cursor = start + head + nameSize;
    const name = decoder.decode(data.subarray(start + head, cursor)), propertyEnd = cursor + size;
    if (end > limit || end < propertyEnd || count > 100000 || cursor > limit) fail();
    const scalars: (string | number)[] = [];
    for (let i = 0; i < count; i++) {
      if (cursor >= propertyEnd) fail();
      const type = String.fromCharCode(data[cursor++]);
      const scalarBytes: Record<string, number> = { Y: 2, C: 1, I: 4, F: 4, D: 8, L: 8 };
      if (scalarBytes[type]) {
        const width = scalarBytes[type]; if (cursor + width > propertyEnd) fail();
        if (type === 'L') { const n = Number(view.getBigInt64(cursor, true)); if (!Number.isSafeInteger(n)) fail(); scalars.push(n); }
        if (type === 'I') scalars.push(view.getInt32(cursor, true));
        if ((type === 'F' && !Number.isFinite(view.getFloat32(cursor, true))) || (type === 'D' && !Number.isFinite(view.getFloat64(cursor, true)))) fail();
        cursor += width;
      } else if (type === 'S' || type === 'R') {
        if (cursor + 4 > propertyEnd) fail();
        const length = view.getUint32(cursor, true); cursor += 4;
        if (cursor + length > propertyEnd) fail();
        if (type === 'S' && scalars.length < 4) scalars.push(decoder.decode(data.subarray(cursor, cursor + Math.min(length, 4096))));
        cursor += length;
      } else if ('fdlibc'.includes(type)) {
        if (cursor + 12 > propertyEnd) fail();
        const length = view.getUint32(cursor, true), encoding = view.getUint32(cursor + 4, true), compressed = view.getUint32(cursor + 8, true);
        const width = type === 'd' || type === 'l' ? 8 : type === 'f' || type === 'i' ? 4 : 1;
        expanded += length * width; entries += length;
        if (length > 4000000 || entries > 16000000 || expanded > MAX_FBX_EXPANDED_BYTES || encoding > 1 || (encoding === 0 && compressed !== length * width)) fail();
        if (encoding === 1) arrays.push({ offset: cursor + 12, bytes: compressed, expanded: length * width });
        cursor += 12 + compressed;
      } else fail();
      if (cursor > propertyEnd) fail();
    }
    if (cursor !== propertyEnd) fail();
    if (name === 'Video' && ++images > 64) fail();
    if (name === 'Texture' && ++textureCount > 256) fail();
    if (name === 'Material' && ++materialCount > 128) fail();
    if (name === 'Model' && typeof scalars[0] === 'number') models.add(scalars[0]);
    if (name === 'C' && scalars[0] === 'OO' && typeof scalars[1] === 'number' && typeof scalars[2] === 'number') links.push([scalars[1], scalars[2]]);
    while (cursor < end) { const next = node(cursor, end, depth + 1); if (next <= cursor) fail(); cursor = next; }
    return end;
  };
  let cursor = 27;
  try {
    while (cursor + head <= bytes.byteLength && uint(cursor)) cursor = node(cursor, bytes.byteLength, 0);
  } catch (error) { if (error instanceof RangeError) fail(); throw error; }
  // Model parenting must be a tree before Three.js walks it recursively.
  const parents = new Map<number, number>();
  for (const [child, parent] of links) if (models.has(child) && models.has(parent)) {
    if (parents.has(child)) fail(); parents.set(child, parent);
  }
  for (const id of models) {
    const seen = new Set<number>(); let current: number | undefined = id;
    while (current !== undefined) { if (seen.has(current) || seen.size > 128) fail(); seen.add(current); current = parents.get(current); }
  }
  return arrays;
}

/** Check actual inflated length in a streaming decoder before FBXLoader unzips it. */
export async function validateFBXCompression(bytes: ArrayBuffer, arrays = inspectFBX(bytes)): Promise<void> {
  for (const array of arrays) {
    const stream = new Blob([bytes.slice(array.offset, array.offset + array.bytes)]).stream().pipeThrough(new DecompressionStream('deflate'));
    const reader = stream.getReader(); let total = 0;
    try {
      while (true) { const chunk = await reader.read(); if (chunk.done) break; total += chunk.value.byteLength;
        if (total > array.expanded) throw new Error(msg('FBX compressed array exceeds its declared size.', 'FBX 압축 배열이 선언된 크기를 넘습니다.'));
      }
      if (total !== array.expanded) throw new Error(msg('FBX compressed array size does not match.', 'FBX 압축 배열 크기가 일치하지 않습니다.'));
    } finally { await reader.cancel().catch(() => {}); reader.releaseLock(); }
  }
}

export interface LoadedFBX {
  scene: THREE.Group; materials: THREE.MeshPhongMaterial[]; metersPerUnit: number;
  images: File[]; animationCount: number;
  applyTextures(config: FBXConfig): Promise<void>;
  release(): void;
  dispose(): void;
}
export async function loadLocalFBX(file: File, companions: File[] = []): Promise<LoadedFBX> {
  if (!/\.fbx$/i.test(file.name) || file.size > MAX_FBX_BYTES) throw new Error(msg('Select an FBX file no larger than 50 MiB.', '50 MiB 이하의 FBX 파일을 선택하세요.'));
  if (companions.length > 64 || companions.reduce((n, f) => n + f.size, 0) > MAX_FBX_TEXTURE_BYTES) throw new Error(msg('Select at most 64 PNG/JPEG images, totaling at most 50 MiB.', 'PNG/JPEG 이미지는 최대 64개, 합계 50 MiB까지 선택할 수 있습니다.'));
  const files = new Map<string, File>();
  for (const image of companions) {
    const key = imageKey(image.name);
    if (!/\.(png|jpe?g)$/.test(key) || files.has(key)) throw new Error(msg('Companion images must be PNG/JPEG with distinct filenames.', '함께 선택한 이미지는 이름이 중복되지 않는 PNG/JPEG여야 합니다.'));
    files.set(key, image);
  }
  const bytes = await file.arrayBuffer(); await validateFBXCompression(bytes);
  let pixels = 0, imageCount = 0;
  const resources = new Map<string, Promise<Uint8Array>>(), textures = new Set<THREE.Texture>(), urls = new Set<string>();
  const read = (url: string): Promise<Uint8Array> => {
    const key = /^(blob:|data:)/.test(url) ? url : imageKey(url);
    let pending = resources.get(key);
    if (!pending) {
      pending = (async () => {
        let data: Uint8Array;
        if (url.startsWith('blob:') && urls.has(url)) data = new Uint8Array(await (await fetch(url)).arrayBuffer());
        else if (/^data:image\/(png|jpeg);base64,/i.test(url)) {
          data = Uint8Array.from(atob(url.slice(url.indexOf(',') + 1)), c => c.charCodeAt(0));
        } else {
          const image = files.get(key);
          if (!image) throw new Error(msg('Select the companion PNG/JPEG image: {0}. External requests are blocked.', '함께 사용할 PNG/JPEG 이미지를 선택하세요: {0}. 외부 요청은 차단됩니다.', key));
          data = new Uint8Array(await image.arrayBuffer());
        }
        const [w, h] = imageDimensions(data); pixels += w * h;
        if (++imageCount > 64 || !w || !h || w > 4096 || h > 4096 || pixels > 64000000) throw new Error(msg('FBX images must fit 4096 pixels per side and 64M pixels total.', 'FBX 이미지는 한 변 4096픽셀, 전체 64M 픽셀 이하여야 합니다.'));
        return data;
      })();
      // The loader starts asynchronous requests during synchronous parsing.
      pending.catch(() => {}); resources.set(key, pending);
    }
    return pending;
  };
  const pending: Promise<void>[] = [];
  const decoded = new Map<string, Promise<THREE.TextureSource<HTMLImageElement>>>();
  const decode = (url: string, texture: THREE.Texture) => {
    const key = /^(blob:|data:)/.test(url) ? url : imageKey(url);
    let source = decoded.get(key);
    if (!source) {
      source = (async () => {
        const data = await read(url), objectUrl = URL.createObjectURL(new Blob([Uint8Array.from(data)])); urls.add(objectUrl);
        const image = new Image(); image.src = objectUrl;
        try { await image.decode(); return new THREE.TextureSource(image); }
        finally { URL.revokeObjectURL(objectUrl); urls.delete(objectUrl); }
      })();
      source.catch(() => {}); decoded.set(key, source);
    }
    const task = source.then(imageSource => { texture.source = imageSource; texture.needsUpdate = true; });
    task.catch(() => {}); pending.push(task); return task;
  };
  const manager = new THREE.LoadingManager();
  manager.setURLModifier(() => { throw new Error(msg('External FBX requests are blocked.', 'FBX 외부 요청은 차단됩니다.')); });
  class LocalTextureLoader extends THREE.TextureLoader {
    override load(url: string): THREE.Texture<HTMLImageElement> { const texture = new THREE.Texture<HTMLImageElement>(); textures.add(texture); void decode(url, texture); return texture; }
  }
  manager.addHandler(/.*/, new LocalTextureLoader(manager));
  let scene: THREE.Group | undefined;
  const release = () => { textures.forEach(texture => texture.dispose()); urls.forEach(url => URL.revokeObjectURL(url)); urls.clear(); };
  try {
    // Include unassigned companions in the image budget before allowing a mapping.
    await Promise.all([...files.keys()].map(read));
    // FBXLoader creates embedded blob URLs synchronously, including unused ones.
    // Capture their lifetime and restore the global immediately, even on failure.
    const create = URL.createObjectURL;
    try {
      URL.createObjectURL = blob => { const url = create.call(URL, blob); urls.add(url); return url; };
      scene = new FBXLoader(manager).parse(bytes, '');
    } finally { URL.createObjectURL = create; }
    const results = await Promise.allSettled(pending);
    const failed = results.find(result => result.status === 'rejected'); if (failed?.status === 'rejected') throw failed.reason;
    let nodes = 0, vertices = 0; const remove: THREE.Object3D[] = [], materials = new Set<THREE.MeshPhongMaterial>();
    scene.traverse(node => {
      if (++nodes > 10000) throw new Error(msg('FBX exceeds 10,000 scene nodes.', 'FBX 장면 노드가 10,000개를 넘습니다.'));
      if (![...node.position.toArray(), ...node.quaternion.toArray(), ...node.scale.toArray()].every(Number.isFinite)) throw new Error(msg('FBX contains an invalid transform.', 'FBX에 유효하지 않은 변환 값이 있습니다.'));
      if (node instanceof THREE.Camera || node instanceof THREE.Light) remove.push(node);
      if (node instanceof THREE.Mesh) {
        const count = node.geometry.getAttribute('position')?.count ?? 0; vertices += count;
        if (vertices > 1000000) throw new Error(msg('FBX exceeds 1M vertices.', 'FBX 정점이 1M개를 넘습니다.'));
        for (const attribute of Object.values(node.geometry.attributes) as Array<THREE.BufferAttribute | THREE.InterleavedBufferAttribute>) {
          for (const value of attribute.array) if (!Number.isFinite(value)) throw new Error(msg('FBX contains nonfinite geometry attributes.', 'FBX 도형 속성에 유한하지 않은 값이 있습니다.'));
        }
        // three-vrm first-person head masking requires indexed triangles. Keep
        // FBXLoader's existing vertex order and all skin/UV attributes intact.
        if (count <= 1000000 && !node.geometry.index) node.geometry.setIndex(Array.from({ length: count }, (_, i) => i));
        for (const mat of Array.isArray(node.material) ? node.material : [node.material]) if (mat instanceof THREE.MeshPhongMaterial || mat instanceof THREE.MeshLambertMaterial) materials.add(mat as THREE.MeshPhongMaterial);
      }
    });
    if (vertices > 1000000 || materials.size > 128) throw new Error(msg('FBX exceeds 1M vertices or 128 materials.', 'FBX가 정점 1M개 또는 재질 128개를 넘습니다.'));
    remove.forEach(node => node.removeFromParent());
    const animationCount = scene.animations.length; scene.animations = [];
    const assignedTextures = new Map<string, THREE.Texture>();
    const factor = Number(scene.userData.unitScaleFactor), materialList = [...materials];
    const originalMaterials = materialList.map(material => ({ map: material.map, color: material.color.clone() }));
    const result: LoadedFBX = { scene, materials: materialList, images: companions, animationCount,
      metersPerUnit: Number.isFinite(factor) && factor > 0 ? factor / 100 : .01,
      async applyTextures(config) {
        materialList.forEach((material, index) => { material.map = originalMaterials[index].map; material.color.copy(originalMaterials[index].color); material.needsUpdate = true; });
        for (const [index, key] of Object.entries(config.textures)) {
          const material = materialList[Number(index)];
          if (!material || !files.has(key)) throw new Error(msg('An FBX material or companion texture is missing.', 'FBX 재질 또는 연결할 이미지가 없습니다.'));
          let texture = assignedTextures.get(key);
          if (!texture) {
            texture = new THREE.Texture(); textures.add(texture); texture.colorSpace = THREE.SRGBColorSpace;
            await decode(key, texture); assignedTextures.set(key, texture);
          }
          material.map = texture; material.color.set(0xffffff); material.needsUpdate = true;
        }
      }, release, dispose: () => { disposeFBX(scene!); release(); },
    };
    return result;
  } catch (error) { await Promise.allSettled(pending); if (scene) disposeFBX(scene); release(); throw error; }
}
