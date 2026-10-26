import { VRMHumanBoneName } from '@pixiv/three-vrm';
import { msg } from './i18n';

export const FBX_BONES = Object.values(VRMHumanBoneName);
export const FBX_REQUIRED = ['hips', 'spine', 'head', ...['left', 'right'].flatMap(side =>
  ['UpperArm', 'LowerArm', 'Hand', 'UpperLeg', 'LowerLeg', 'Foot'].map(bone => side + bone))];
export interface FBXConfig {
  bones: Partial<Record<VRMHumanBoneName, string>>;
  metersPerUnit: number;
  /** Stable material traversal index -> explicitly selected companion basename. */
  textures: Record<string, string>;
}
export const imageKey = (name: string) => name.replaceAll('\\', '/').split('/').pop()!.normalize('NFC').toLowerCase();
export function validateFBXConfig(value: unknown): FBXConfig {
  const fail = (): never => { throw new Error(msg('Invalid FBX bone, unit or texture settings.', 'FBX 본·단위·텍스처 설정이 올바르지 않습니다.')); };
  const obj = (v: unknown): Record<string, unknown> => v && typeof v === 'object' && !Array.isArray(v) ? v as Record<string, unknown> : fail();
  const input = obj(value), bones = obj(input.bones), textures = obj(input.textures);
  if (typeof input.metersPerUnit !== 'number' || !Number.isFinite(input.metersPerUnit) || input.metersPerUnit < .000001 || input.metersPerUnit > 10) fail();
  const used = new Set<string>();
  for (const [name, path] of Object.entries(bones)) {
    if (!FBX_BONES.includes(name as VRMHumanBoneName) || typeof path !== 'string' || path.length > 512 || !/^\d+(?:\/\d+)*$/.test(path) || used.has(path)) fail();
    used.add(path as string);
  }
  const missing = FBX_REQUIRED.filter(name => !bones[name]);
  if (missing.length) throw new Error(msg('Missing required FBX bones: {0}', 'FBX 필수 본이 연결되지 않았습니다: {0}', missing.join(', ')));
  if (Object.keys(textures).length > 128) fail();
  for (const [index, name] of Object.entries(textures)) {
    if (!/^(?:0|[1-9]\d{0,2})$/.test(index) || Number(index) >= 128 || typeof name !== 'string' || name.length > 512 || name !== imageKey(name) || !/\.(png|jpe?g)$/i.test(name) || name.includes('\0')) fail();
  }
  return { bones: { ...bones } as FBXConfig['bones'], metersPerUnit: input.metersPerUnit as number, textures: { ...textures } as Record<string, string> };
}
