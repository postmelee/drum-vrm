import { imageKey, validateFBXConfig, type FBXConfig } from './fbx-config';

const KEY = 'drum-vrm:fbx-setups:v1';
/** Settings only, keyed by source bytes; never persist model/image contents. */
export function savedFBXSetup(hash: string): FBXConfig | undefined {
  try { return validateFBXConfig(JSON.parse(localStorage.getItem(KEY) ?? '{}')[hash]); } catch { return undefined; }
}
export function rememberFBXSetup(hash: string, config: FBXConfig): void {
  try {
    const old = JSON.parse(localStorage.getItem(KEY) ?? '{}');
    const entries = Object.entries(old).filter(([key]) => /^[a-f0-9]{64}$/.test(key) && key !== hash).slice(-9);
    localStorage.setItem(KEY, JSON.stringify(Object.fromEntries([...entries, [hash, validateFBXConfig(config)]])));
  } catch { /* Private mode/quota must not prevent importing a character. */ }
}

/** Conservative name matches. Ties and non-base-color maps remain unassigned. */
export function suggestFBXTexture(material: string, filenames: string[]): string | undefined {
  const tokens = (value: string) => value.normalize('NFKC').toLowerCase()
    .replace(/\.(png|jpe?g)$/i, '').replace(/\.\d+$/, '').split(/[^\p{L}\p{N}]+/u).filter(Boolean);
  const ignored = new Set(['mtl', 'mat', 'material', 'tex', 'texture', 'diff', 'diffuse', 'albedo', 'basecolor', 'color']);
  const meaningful = (value: string) => tokens(value).filter(t => !ignored.has(t));
  const source = meaningful(material);
  if (!source.length) return;
  const scored = filenames.flatMap(filename => {
    const words = tokens(filename);
    if (words.some(t => ['normal', 'nrm', 'roughness', 'metallic', 'metalness', 'ao', 'occlusion', 'shadow', 'shad', 'specular', 'bump', 'mask'].includes(t))) return [];
    const target = meaningful(filename), shared = source.filter(t => target.includes(t));
    // Require at least a semantic/name token, never a numeric suffix alone.
    if (!shared.some(t => /\p{L}/u.test(t))) return [];
    const score = shared.length / source.length + shared.length / Math.max(target.length, 1);
    return [{ name: imageKey(filename), score }];
  }).sort((a, b) => b.score - a.score);
  return scored.length && (scored.length === 1 || scored[0].score > scored[1].score) ? scored[0].name : undefined;
}
