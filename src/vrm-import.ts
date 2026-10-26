import { msg } from './i18n';
import { LoadingManager } from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRMUtils, type VRM } from '@pixiv/three-vrm';
import { validateSelfContainedGLB } from './model-import';

export const MAX_VRM_BYTES = 50 * 1024 * 1024;
export const MAX_VRM_IMAGES = 64, MAX_VRM_IMAGE_DIMENSION = 4096, MAX_VRM_IMAGE_PIXELS = 64_000_000;
export const REQUIRED_BONES = ['hips', 'spine', 'head', ...['left', 'right'].flatMap(side =>
  ['UpperArm', 'LowerArm', 'Hand', 'UpperLeg', 'LowerLeg', 'Foot'].map(bone => side + bone))];
const fingers = ['ThumbMetacarpal', 'ThumbProximal', 'ThumbDistal', ...['Index', 'Middle', 'Ring', 'Little'].flatMap(f => ['Proximal', 'Intermediate', 'Distal'].map(b => f + b))];
type Json = Record<string, unknown>;
const object = (value: unknown): Json => value && typeof value === 'object' && !Array.isArray(value) ? value as Json : {};
const text = (value: unknown): string => typeof value === 'string' ? value.slice(0, 16000) : msg("Not specified", "미기재");
export interface VRMInspection {
  version: string; name: string; authors: string[]; required: number; missing: string[];
  fingers: { left: number; right: number }; terms: [string, string][];
  modelVersion: string; references: string[]; rawMeta: Json; metadataWarnings: string[];
}

/** PNG/JPEG header budgets are checked before image decoding can allocate textures. */
function imageDimensions(bytes: Uint8Array): [number, number] {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (bytes.length >= 24 && view.getUint32(0) === 0x89504e47 && view.getUint32(4) === 0x0d0a1a0a && view.getUint32(12) === 0x49484452) return [view.getUint32(16), view.getUint32(20)];
  if (bytes.length >= 4 && bytes[0] === 0xff && bytes[1] === 0xd8) {
    let cursor = 2;
    while (cursor + 4 <= bytes.length) {
      if (bytes[cursor++] !== 0xff) break;
      while (bytes[cursor] === 0xff) cursor++;
      const marker = bytes[cursor++];
      if (marker === 0xd9 || marker === 0xda) break;
      if (marker === 0x01 || (marker >= 0xd0 && marker <= 0xd7)) continue;
      if (cursor + 2 > bytes.length) break;
      const length = view.getUint16(cursor);
      if (length < 2 || cursor + length > bytes.length) break;
      if ([0xc0, 0xc1, 0xc2, 0xc3, 0xc5, 0xc6, 0xc7, 0xc9, 0xca, 0xcb, 0xcd, 0xce, 0xcf].includes(marker) && length >= 8) return [view.getUint16(cursor + 5), view.getUint16(cursor + 3)];
      cursor += length;
    }
  }
  throw new Error(msg("VRM images must be PNG or JPEG with readable dimensions.", "VRM 이미지는 크기를 확인할 수 있는 PNG 또는 JPEG여야 합니다."));
}
function inspectImageBudget(json: Json, bytes: ArrayBuffer): void {
  const images = Array.isArray(json.images) ? json.images : [];
  if (images.length > MAX_VRM_IMAGES) throw new Error(msg("VRM models can contain at most 64 images.", "VRM 이미지는 최대 64개까지 지원합니다."));
  const bufferViews = Array.isArray(json.bufferViews) ? json.bufferViews : [], buffers = Array.isArray(json.buffers) ? json.buffers : [];
  const view = new DataView(bytes); let bin: Uint8Array | undefined;
  for (let offset = 12; offset < bytes.byteLength;) {
    const length = view.getUint32(offset, true), kind = view.getUint32(offset + 4, true);
    if (kind === 0x004e4942) bin = new Uint8Array(bytes, offset + 8, length);
    offset += 8 + length;
  }
  const decodeData = (uri: unknown): Uint8Array => {
    if (typeof uri !== 'string' || !/^data:(?:image\/(?:png|jpeg)|application\/(?:octet-stream|gltf-buffer));base64,/i.test(uri)) throw new Error(msg("Invalid VRM image resource.", "VRM 이미지 리소스가 올바르지 않습니다."));
    try { return Uint8Array.from(atob(uri.slice(uri.indexOf(',') + 1)), c => c.charCodeAt(0)); }
    catch { throw new Error(msg("The VRM image base64 data could not be read.", "VRM 이미지의 base64 데이터를 읽을 수 없습니다.")); }
  };
  const cache = new Map<number, Uint8Array>();
  let pixels = 0;
  for (const raw of images) {
    const image = object(raw); let data: Uint8Array;
    if (image.uri !== undefined) data = decodeData(image.uri);
    else {
      if (!Number.isInteger(image.bufferView) || (image.bufferView as number) < 0 || (image.bufferView as number) >= bufferViews.length) throw new Error(msg("Invalid VRM image bufferView.", "VRM 이미지 bufferView가 올바르지 않습니다."));
      const range = object(bufferViews[image.bufferView as number]), bufferIndex = range.buffer;
      if (!Number.isInteger(bufferIndex) || (bufferIndex as number) < 0 || (bufferIndex as number) >= buffers.length) throw new Error(msg("Invalid VRM image buffer.", "VRM 이미지 buffer가 올바르지 않습니다."));
      let buffer = cache.get(bufferIndex as number);
      if (!buffer) {
        const definition = object(buffers[bufferIndex as number]);
        buffer = definition.uri === undefined && bufferIndex === 0 ? bin : decodeData(definition.uri);
        if (!buffer) throw new Error(msg("VRM image binary data is missing.", "VRM 이미지 바이너리 데이터가 없습니다."));
        cache.set(bufferIndex as number, buffer);
      }
      const offset = range.byteOffset ?? 0, length = range.byteLength;
      if (!Number.isInteger(offset) || !Number.isInteger(length) || (offset as number) < 0 || (length as number) <= 0 || (offset as number) + (length as number) > buffer.length) throw new Error(msg("The VRM image range exceeds its binary buffer.", "VRM 이미지 범위가 바이너리를 벗어났습니다."));
      data = buffer.subarray(offset as number, (offset as number) + (length as number));
    }
    const [width, height] = imageDimensions(data); pixels += width * height;
    if (!width || !height || width > MAX_VRM_IMAGE_DIMENSION || height > MAX_VRM_IMAGE_DIMENSION || pixels > MAX_VRM_IMAGE_PIXELS) throw new Error(msg("VRM images must be no larger than 4096 pixels per side and 64M pixels in total.", "VRM 이미지 한 변은 4096픽셀, 전체는 64M 픽셀 이하여야 합니다."));
  }
}

/** Inspect before any loader runs. Metadata is displayed as text, never executed or fetched. */
export function inspectVRM(bytes: ArrayBuffer): VRMInspection {
  validateSelfContainedGLB(bytes, MAX_VRM_BYTES);
  const length = new DataView(bytes).getUint32(12, true);
  const json = JSON.parse(new TextDecoder().decode(new Uint8Array(bytes, 20, length))) as Json;
  const extensions = object(json.extensions);
  const vrm = object(extensions.VRMC_vrm);
  const version = vrm.specVersion === '1.0' ? '1.0' : extensions.VRM ? '0.x' : text(vrm.specVersion);
  if (version !== '1.0') throw new Error(msg('Detected VRM: {0}. Only VRM 1.0 is supported. Export VRM 1.0 from VRoid Studio or UniVRM.', '감지된 VRM: {0}. 현재 VRM 1.0을 지원합니다. VRoid Studio 또는 UniVRM에서 VRM 1.0으로 내보내세요.', version));
  inspectImageBudget(json, bytes);
  const nodes = Array.isArray(json.nodes) ? json.nodes : [];
  if (!nodes.length || nodes.length > 10000) throw new Error(msg("The VRM node count is outside the supported range.", "VRM 노드 수가 허용 범위를 벗어났습니다."));
  const parents = new Map<number, number>();
  nodes.forEach((raw, index) => {
    const node = object(raw);
    for (const [key, length] of [['translation', 3], ['rotation', 4], ['scale', 3], ['matrix', 16]] as const) {
      if (node[key] !== undefined && (!Array.isArray(node[key]) || (node[key] as unknown[]).length !== length || !(node[key] as unknown[]).every(v => typeof v === 'number' && Number.isFinite(v)))) throw new Error(msg("The VRM contains an invalid transform.", "VRM에 유효하지 않은 변환 값이 있습니다."));
    }
    if (node.children !== undefined && !Array.isArray(node.children)) throw new Error(msg("Invalid VRM bone children list.", "VRM 본의 자식 목록이 올바르지 않습니다."));
    for (const child of Array.isArray(node.children) ? node.children : []) {
      if (!Number.isInteger(child) || child < 0 || child >= nodes.length || parents.has(child)) throw new Error(msg("Invalid VRM bone hierarchy.", "VRM 본 계층이 올바르지 않습니다."));
      parents.set(child, index);
    }
  });
  const complete = new Set<number>();
  for (let i = 0; i < nodes.length; i++) {
    const visited = new Set<number>(); let current: number | undefined = i;
    while (current !== undefined && !complete.has(current)) {
      if (visited.has(current)) throw new Error(msg("The VRM bone hierarchy contains a cycle.", "VRM 본 계층에 순환이 있습니다."));
      visited.add(current); current = parents.get(current);
    }
    visited.forEach(index => complete.add(index));
  }
  const bones = object(object(vrm.humanoid).humanBones);
  const validBone = (name: string) => {
    const index = object(bones[name]).node;
    return typeof index === 'number' && Number.isInteger(index) && index >= 0 && index < nodes.length;
  };
  const missing = REQUIRED_BONES.filter(name => !validBone(name));
  if (missing.length) throw new Error(msg('Missing required VRM 1.0 bones: {0}', 'VRM 1.0 필수 본 누락: {0}', missing.join(', ')));
  const indices = Object.values(bones).map(bone => object(bone).node);
  if (new Set(indices).size !== indices.length || Object.keys(bones).some(name => !validBone(name))) throw new Error(msg("VRM humanoid bone assignments are duplicated or invalid.", "VRM 휴머노이드 본 연결이 중복되었거나 유효하지 않습니다."));
  const meta = object(vrm.meta);
  const termLabels: [string, string][] = [
    ['licenseUrl',msg("License document", "라이선스 문서")], ['avatarPermission',msg("Avatar users", "아바타 사용자")], ['commercialUsage',msg("Commercial use", "상업적 이용")],
    ['creditNotation',msg("Credit", "크레딧")], ['allowRedistribution',msg("Model redistribution", "모델 재배포")], ['modification',msg("Model modification", "모델 수정")],
    ['allowExcessivelyViolentUsage',msg("Excessive violence", "과도한 폭력 표현")], ['allowExcessivelySexualUsage',msg("Excessive sexual content", "과도한 성적 표현")],
    ['allowPoliticalOrReligiousUsage',msg("Political or religious content", "정치·종교 표현")], ['allowAntisocialOrHateUsage',msg("Antisocial or hateful content", "반사회·혐오 표현")],
    ['copyrightInformation',msg("Copyright", "저작권")], ['thirdPartyLicenses',msg("Third-party terms", "제3자 조건")], ['otherLicenseUrl',msg("Additional terms", "추가 조건")],
  ];
  const defaults: Record<string, string | boolean> = {
    avatarPermission: 'onlyAuthor', commercialUsage: 'personalNonProfit', creditNotation: 'required',
    allowRedistribution: false, modification: 'prohibited', allowExcessivelyViolentUsage: false,
    allowExcessivelySexualUsage: false, allowPoliticalOrReligiousUsage: false, allowAntisocialOrHateUsage: false,
  };
  const valueText = (value: unknown): string => typeof value === 'boolean' ? (value ? msg("Allowed", "허용") : msg("Not allowed", "불허")) : text(value);
  const metadataWarnings: string[] = [];
  if (typeof meta.name !== 'string' || !meta.name.trim()) metadataWarnings.push(msg("The required model name is missing.", "필수 모델 이름이 없습니다."));
  if (!Array.isArray(meta.authors) || !meta.authors.some(author => typeof author === 'string' && author.trim())) metadataWarnings.push(msg("The required author information is missing.", "필수 제작자 정보가 없습니다."));
  if (meta.licenseUrl !== 'https://vrm.dev/licenses/1.0/') metadataWarnings.push(msg("The VRM 1.0 standard license URL is missing or different. Check the model's original terms.", "VRM 1.0 표준 라이선스 URL이 없거나 다릅니다. 모델의 원본 조건을 확인하세요."));
  const enums: Record<string, string[]> = {
    avatarPermission: ['onlyAuthor', 'onlySeparatelyLicensedPerson', 'everyone'],
    commercialUsage: ['personalNonProfit', 'personalProfit', 'corporation'],
    creditNotation: ['required', 'unnecessary'], modification: ['prohibited', 'allowModification', 'allowModificationRedistribution'],
  };
  for (const [key, fallback] of Object.entries(defaults)) {
    if (meta[key] === undefined) continue;
    if (typeof fallback === 'boolean' ? typeof meta[key] !== 'boolean' : !enums[key].includes(meta[key] as string)) metadataWarnings.push(msg('The {0} usage term differs from the specification. Check the original terms.', '이용 조건 {0} 값이 명세와 다릅니다. 원본 조건을 확인하세요.', key));
  }
  return {version, name: text(meta.name), authors: Array.isArray(meta.authors) ? meta.authors.map(text) : [], required: REQUIRED_BONES.length, missing,
    modelVersion: text(meta.version), references: Array.isArray(meta.references) ? meta.references.map(text) : [], rawMeta: meta, metadataWarnings,
    fingers: {left: fingers.filter(f => validBone('left'+f)).length, right: fingers.filter(f => validBone('right'+f)).length},
    terms: termLabels.map(([key,label]) => [label, meta[key] === undefined && key in defaults ? msg('Not specified (VRM 1.0 default: {0})', '미기재 (VRM 1.0 기본값: {0})', valueText(defaults[key])) : valueText(meta[key])]),
  };
}

export async function loadLocalVRM(file: File): Promise<{vrm: VRM; info: VRMInspection}> {
  if (!file.name.toLowerCase().endsWith('.vrm')) throw new Error(msg("Select a runtime .vrm model. A .vroid file is a VRoid Studio editing project.", ".vrm 실행용 모델을 선택하세요. .vroid는 VRoid Studio 편집 파일입니다."));
  if (file.size > MAX_VRM_BYTES) throw new Error(msg("VRM files must be no larger than 50 MiB.", "VRM은 50 MiB 이하여야 합니다."));
  const bytes = await file.arrayBuffer();
  const info = inspectVRM(bytes);
  const manager = new LoadingManager();
  manager.setURLModifier(url => {
    if (url.startsWith('blob:') || url.startsWith('data:')) return url;
    throw new Error(msg("An external VRM resource request was blocked.", "VRM 외부 리소스 요청을 차단했습니다."));
  });
  const loader = new GLTFLoader(manager).register(parser => new VRMLoaderPlugin(parser));
  const gltf = await loader.parseAsync(bytes, '');
  const vrm = gltf.userData.vrm as VRM | undefined;
  if (!vrm || vrm.meta.metaVersion !== '1') { VRMUtils.deepDispose(gltf.scene); throw new Error(msg("The VRM 1.0 humanoid could not be loaded.", "VRM 1.0 휴머노이드를 읽지 못했습니다.")); }
  const remove: import('three').Object3D[] = [];
  vrm.scene.traverse(node => { if (node.type.includes('Camera') || node.type.includes('Light')) remove.push(node); });
  remove.forEach(node => node.removeFromParent());
  return {vrm, info};
}
