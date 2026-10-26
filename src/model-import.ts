import { msg } from './i18n';
import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { disposeModelSource } from './model-procedural';

export const MAX_MODEL_BYTES = 25 * 1024 * 1024;
/** Inspect the actual GLB before GLTFLoader can resolve any referenced resources. */
export function validateSelfContainedGLB(buffer: ArrayBuffer, maxBytes = MAX_MODEL_BYTES): void {
  if (buffer.byteLength > maxBytes) throw new Error(msg('3D models must be no larger than {0} MiB.', '3D 모델은 {0} MiB 이하여야 합니다.', maxBytes / 1024 / 1024));
  if (buffer.byteLength < 20) throw new Error(msg("Not a valid GLB 2.0 file.", "올바른 GLB 2.0 파일이 아닙니다."));
  const view = new DataView(buffer);
  if (view.getUint32(0, true) !== 0x46546c67 || view.getUint32(4, true) !== 2 || view.getUint32(8, true) !== buffer.byteLength) {
    throw new Error(msg("Not a valid GLB 2.0 file.", "올바른 GLB 2.0 파일이 아닙니다."));
  }
  let offset = 12;
  let json: unknown;
  while (offset < buffer.byteLength) {
    if (offset + 8 > buffer.byteLength) throw new Error(msg("The GLB chunk is truncated.", "GLB 청크가 잘렸습니다."));
    const length = view.getUint32(offset, true), kind = view.getUint32(offset + 4, true);
    if (length % 4 || offset + 8 + length > buffer.byteLength) throw new Error(msg("Invalid GLB chunk length.", "GLB 청크 길이가 올바르지 않습니다."));
    if (offset === 12 && kind !== 0x4e4f534a) throw new Error(msg("The GLB JSON chunk is missing.", "GLB JSON 청크가 없습니다."));
    if (kind === 0x4e4f534a) {
      if (json !== undefined) throw new Error(msg("The GLB JSON chunk is duplicated.", "GLB JSON 청크가 중복되었습니다."));
      try { json = JSON.parse(new TextDecoder().decode(new Uint8Array(buffer, offset + 8, length))); }
      catch { throw new Error(msg("The GLB JSON data could not be read.", "GLB JSON 데이터를 읽을 수 없습니다.")); }
    }
    offset += 8 + length;
  }
  if (!json || typeof json !== 'object' || (json as { asset?: { version?: string } }).asset?.version !== '2.0') throw new Error(msg("GLB 2.0 asset information is missing.", "GLB 2.0 asset 정보가 없습니다."));
  const pending: unknown[] = [json];
  while (pending.length) {
    const current = pending.pop();
    if (!current || typeof current !== 'object') continue;
    for (const [key, value] of Object.entries(current)) {
      if (key === 'uri') {
        if (typeof value !== 'string' || !/^data:(?:image\/(?:png|jpeg|webp)|application\/(?:octet-stream|gltf-buffer));base64,/i.test(value)) {
          throw new Error(msg("Models cannot reference external files or URLs. Export a single GLB with embedded textures.", "외부 파일이나 URL을 참조하는 모델은 사용할 수 없습니다. 텍스처를 포함한 단일 GLB로 내보내세요."));
        }
      } else if (value && typeof value === 'object') pending.push(value);
    }
  }
}
export async function loadLocalGLB(file: File): Promise<THREE.Group> {
  if (!file.name.toLowerCase().endsWith('.glb')) throw new Error(msg("Select a .glb file with embedded textures.", "텍스처가 포함된 .glb 파일을 선택해 주세요."));
  if (file.size > MAX_MODEL_BYTES) throw new Error(msg("3D models must be no larger than 25 MiB.", "3D 모델은 25 MiB 이하여야 합니다."));
  const bytes = await file.arrayBuffer();
  validateSelfContainedGLB(bytes);
  const manager = new THREE.LoadingManager();
  manager.setURLModifier(url => {
    // GLTFLoader creates blob URLs only for embedded image bufferViews.
    if (url.startsWith('blob:') || url.startsWith('data:')) return url;
    throw new Error(msg("An external model resource request was blocked.", "모델의 외부 리소스 요청을 차단했습니다."));
  });
  const loaded = await new GLTFLoader(manager).parseAsync(bytes, '');
  const root = loaded.scene;
  const remove: THREE.Object3D[] = [];
  let meshes = 0, skinned = false;
  root.traverse(node => {
    if (node instanceof THREE.Camera || node instanceof THREE.Light) remove.push(node);
    if (node instanceof THREE.Mesh) meshes++;
    if (node instanceof THREE.SkinnedMesh) skinned = true;
  });
  if (skinned) {
    disposeModelSource(root);
    throw new Error(msg("Skinned or skeletal models are not supported here. Export a static-mesh GLB.", "스킨/뼈대가 있는 모델은 아직 지원하지 않습니다. 정적 메시 GLB로 내보내세요."));
  }
  remove.forEach(node => node.removeFromParent());
  root.updateMatrixWorld(true);
  const bounds = new THREE.Box3().setFromObject(root);
  if (!meshes || bounds.isEmpty() || ![...bounds.min.toArray(), ...bounds.max.toArray()].every(Number.isFinite)) {
    disposeModelSource(root);
    throw new Error(msg("The model contains no valid visible meshes.", "모델에 표시할 수 있는 유효한 메시가 없습니다."));
  }
  return root;
}
