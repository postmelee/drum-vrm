import { msg } from './i18n';
import { makeDemo, makeInterpretationDemo, makePostureDemo } from './demo';
import song from '../public/demos/transparent-girl-1-provenance.json';
import type { Selection } from './types';

export type DemoId = 'studio' | 'interpretation' | 'posture' | 'transparent-girl';
interface DemoEntry {
  id: DemoId;
  label: string;
  filename: string;
  selection: Partial<Selection>;
  generate?: () => Uint8Array;
  asset?: { file: string; bytes: number; sha256: string };
}

/** Demo directives describe reviewed content; uploaded filenames never select rules. */
const entries: readonly DemoEntry[] = [
  {
    id: 'posture', label: msg('Posture study · 12 s', '자세 검증 패턴 · 12초'), filename: 'Posture Study.mid',
    generate: makePostureDemo,
    selection: { channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true,
      hihatPolicy: 'literal', hihatOpenness: .2, omittedSourceEventIds: [], handOverrides: {} },
  },
  {
    id: 'studio', label: msg('Studio Groove · 30 s', 'Studio Groove · 30초'), filename: 'Studio Groove.mid',
    generate: () => makeDemo(),
    selection: { channel: 9, mappingProfile: 'gm', countInNote75: false,
      hihatPolicy: 'literal', hihatOpenness: .2, omittedSourceEventIds: [], handOverrides: {} },
  },
  {
    id: 'interpretation', label: msg('Interpretation check · 8 s', '연주 검증 패턴 · 8초'), filename: 'Interpretation Check.mid',
    generate: makeInterpretationDemo,
    selection: { channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true,
      hihatPolicy: 'shortOpenClose', hihatOpenness: .2, omittedSourceEventIds: [], handOverrides: {} },
  },
  {
    id: 'transparent-girl', label: msg('Transparent Girl 1 · 3 min 13 s', '투명소녀1 · 3분 13초'), filename: song.filename,
    asset: { file: song.file, bytes: song.bytes, sha256: song.sha256 },
    selection: song.selection as Partial<Selection>,
  },
];

export const DEMO_CATALOG: readonly Readonly<Pick<DemoEntry, 'id' | 'label' | 'filename'>>[] =
  entries.map(({ id, label, filename }) => ({ id, label, filename }));

export async function loadDemo(
  id: DemoId,
  baseUrl = import.meta.env.BASE_URL,
  fetcher: typeof fetch = fetch,
): Promise<{ bytes: Uint8Array; filename: string; selection: Partial<Selection> }> {
  const entry = entries.find(demo => demo.id === id);
  if (!entry) throw new Error(msg('Unknown demo.', '알 수 없는 데모입니다.'));
  let bytes: Uint8Array;
  if (entry.generate) bytes = entry.generate();
  else {
    const asset = entry.asset!;
    const response = await fetcher(`${baseUrl.replace(/\/?$/, '/')}demos/${asset.file}`);
    if (!response.ok) throw new Error(msg('Could not download the demo ({0}). Please try again.', '데모를 내려받지 못했습니다 ({0}). 다시 시도해 주세요.', response.status));
    const buffer = await response.arrayBuffer();
    if (buffer.byteLength !== asset.bytes) throw new Error(msg('The demo file size does not match. Reload and try again.', '데모 파일 크기가 일치하지 않습니다. 새로고침 후 다시 시도해 주세요.'));
    const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', buffer)), value => value.toString(16).padStart(2, '0')).join('');
    if (hash !== asset.sha256) throw new Error(msg('The demo file differs from the verified original. Reload and try again.', '데모 파일이 검증된 원본과 다릅니다. 새로고침 후 다시 시도해 주세요.'));
    bytes = new Uint8Array(buffer);
  }
  return { bytes, filename: entry.filename, selection: structuredClone(entry.selection) };
}
