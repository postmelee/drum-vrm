/** Additional review samples derived from score semantics, not fixture names.
 * Keep first-hit UI checkpoints and add settled groove and between-head paths.
 * These finite observations do not certify a continuous collision-free motion. */
export function motionReviewLandmarks(plan, checkpoints = []) {
  const result = checkpoints.map(checkpoint => ({ ...checkpoint, source: 'ui-checkpoint' }));
  const hits = [...plan.hits].filter(hit => Number.isFinite(hit.time)).sort((a, b) => a.time - b.time || String(a.id).localeCompare(String(b.id)));
  const first = hits[0]?.time ?? 0;
  result.push({ label: 'Before first strike', time: first - .25, source: 'source-boundary' });
  const hats = hits.filter(hit => ['hihat', 'openHat'].includes(hit.instrument));
  const runs = [];
  for (const hit of hats) {
    const last = runs.at(-1);
    if (!last || hit.time - last.at(-1).time > 1) runs.push([hit]);
    else last.push(hit);
  }
  const sustained = runs.filter(run => run.length >= 3).sort((a, b) => b.length - a.length || a[0].time - b[0].time)[0];
  if (sustained) {
    const hit = sustained[Math.floor(sustained.length / 2)];
    result.push({ label: 'Sustained groove', time: hit.time, source: 'hat-run', sourceEventIds: [hit.sourceEventId] });
    const snares = hits.filter(note => note.instrument === 'snare' && note.time >= sustained[0].time && note.time <= sustained.at(-1).time);
    const snare = snares.sort((a, b) => Math.abs(a.time - hit.time) - Math.abs(b.time - hit.time) || a.time - b.time)[0];
    if (snare) result.push({ label: 'Groove backbeat', time: snare.time, source: 'hat-run-snare', sourceEventIds: [snare.sourceEventId] });
  }
  const groups = [];
  for (const hit of hits.filter(hit => !['kick', 'pedalHat'].includes(hit.instrument))) {
    const last = groups.at(-1);
    if (!last || Math.abs(hit.time - last[0].time) > 1e-6) groups.push([hit]);
    else last.push(hit);
  }
  // Source groups keep unassigned transfers visible. They do not assume a hand.
  let count = 0;
  for (let index = 1; index < groups.length && count < 12; index++) {
    const previous = groups[index - 1], next = groups[index];
    if (next[0].time - previous[0].time > .6) continue;
    const pair = previous.flatMap(a => next.map(b => [a, b])).find(([a, b]) => a.pieceId !== b.pieceId && (a.instrument.startsWith('tom') || b.instrument.startsWith('tom')));
    if (!pair) continue;
    result.push({ label: `Tom path ${++count}`, time: (previous[0].time + next[0].time) / 2, source: 'between-source-groups', sourceEventIds: pair.map(hit => hit.sourceEventId) });
  }
  result.push({ label: 'Final recovery', time: Math.max(plan.duration, hits.at(-1)?.time ?? 0) + .5, source: 'source-boundary' });
  return result;
}
