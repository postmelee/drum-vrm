import { msg } from './i18n';
import avatar from '../public/characters/avatar-sample-b-provenance.json';
import seed from '../public/characters/seed-san-provenance.json';
import twist from '../public/characters/twist-provenance.json';
import { MAX_VRM_BYTES } from './vrm-import';

export interface CharacterPreset {
  id: string;
  title: string;
  creator: string;
  file: string;
  bytes: number;
  sha256: string;
  source: string;
  license: string;
  licenseUrl: string;
  label: string;
  provenanceFile: string;
}

/** Distribution metadata is owned by the reviewed asset records, not UI labels. */
export const CHARACTER_PRESETS: readonly CharacterPreset[] = [
  { ...avatar, label: avatar.title, provenanceFile: 'avatar-sample-b-provenance.json' },
  { ...seed, label: seed.title, provenanceFile: 'seed-san-provenance.json' },
  { ...twist, label: msg('Twist sample (fit limited)', 'Twist 샘플 (맞춤 제한)'), provenanceFile: 'twist-provenance.json' },
];

export function characterAssetUrl(file: string, baseUrl = import.meta.env.BASE_URL): string {
  return `${baseUrl.replace(/\/?$/, '/')}characters/${file}`;
}

/** Downloads never bypass the same importer and body-fit transaction used for local files. */
export async function loadCharacterPreset(
  id: string,
  signal?: AbortSignal,
  baseUrl = import.meta.env.BASE_URL,
  fetcher: typeof fetch = fetch,
): Promise<File> {
  const preset = CHARACTER_PRESETS.find(entry => entry.id === id);
  if (!preset) throw new Error(msg('Unknown character preset.', '알 수 없는 캐릭터 프리셋입니다.'));
  signal?.throwIfAborted();
  const response = await fetcher(characterAssetUrl(preset.file, baseUrl), { signal });
  if (!response.ok) throw new Error(msg('Could not download the character ({0}). Please try again.', '캐릭터를 내려받지 못했습니다 ({0}). 다시 시도해 주세요.', response.status));
  const bytes = await response.arrayBuffer();
  signal?.throwIfAborted();
  if (bytes.byteLength !== preset.bytes || bytes.byteLength > MAX_VRM_BYTES) throw new Error(msg('The character file size does not match. Reload and try again.', '캐릭터 파일 크기가 일치하지 않습니다. 새로고침 후 다시 시도해 주세요.'));
  const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), value => value.toString(16).padStart(2, '0')).join('');
  signal?.throwIfAborted();
  if (hash !== preset.sha256) throw new Error(msg('The character differs from the verified original. Reload and try again.', '캐릭터가 검증된 원본과 다릅니다. 새로고침 후 다시 시도해 주세요.'));
  return new File([bytes], preset.file, { type: 'model/gltf-binary' });
}
