type Json = Record<string, unknown>;
const object = (value: unknown): Json => value && typeof value === 'object' && !Array.isArray(value) ? value as Json : {};

/** UniVRM's documented v0 -> MToon 1 migration. Mutates only the parser's JSON,
 * before three-vrm converts materials; source bytes and explicit shade maps stay intact.
 * https://vrm.dev/en/univrm1/migrate_vrm0/feature/#shade-texture-の指定がない場合
 */
export function applyLegacyShadeCompatibility(json: Json): number {
  const legacy = object(object(json.extensions).VRM);
  if (legacy.specVersion !== '0.0' || !Array.isArray(legacy.materialProperties) || !Array.isArray(json.textures)) return 0;
  let count = 0;
  for (const value of legacy.materialProperties) {
    const material = object(value), textures = object(material.textureProperties);
    const main = textures._MainTex;
    if (material.shader !== 'VRM/MToon' || textures._ShadeTexture !== undefined ||
        !Number.isInteger(main) || (main as number) < 0 || (main as number) >= json.textures.length) continue;
    textures._ShadeTexture = main;
    count++;
  }
  return count;
}
