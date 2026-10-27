// Independently authored quarter-note study; no downloaded score content.
export function makeSongsterrStudy(format: 0 | 1 = 1): Uint8Array {
  const chunk = (tag: string, data: number[]) => [...tag].map(c => c.charCodeAt(0)).concat(
    [data.length >>> 24, data.length >>> 16 & 255, data.length >>> 8 & 255, data.length & 255], data);
  const tempo = [0, 255, 81, 3, 7, 161, 32];
  const notes = [0, 255, 3, 5, ...[...'Drums'].map(c => c.charCodeAt(0))];
  for (const [index, key] of [0, 3, 5, 8, 12, 22, 87].entries()) {
    notes.push(...(index ? [0x83, 0x48] : [0]), 0x99, key, 80 + index, 24, 0x89, key, 0);
  }
  const end = [0, 255, 47, 0];
  return new Uint8Array([
    ...chunk('MThd', [0, format, 0, format === 0 ? 1 : 2, 1, 224]),
    ...(format === 1 ? chunk('MTrk', [...tempo, ...end]) : []),
    ...chunk('MTrk', [...(format === 0 ? tempo : []), ...notes, ...end]),
  ]);
}
