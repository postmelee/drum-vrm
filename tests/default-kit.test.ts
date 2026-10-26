import { readFileSync } from 'node:fs';
import { beforeAll, expect, it } from 'vitest';
import { initSync, plan_midi } from '../src/wasm/drum_engine';
import { DEFAULT_KIT } from '../src/kit-default';
import { createMeasuredInstrumentBody } from '../src/instrument-body';
import { disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { solveKitGeometry } from '../src/kit-hardware';
import { makePostureDemo } from '../src/demo';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));

it('uses the same default physical kit in Rust plans and the browser', () => {
  const plan = JSON.parse(plan_midi(makePostureDemo(), JSON.stringify({tracks:[1],channel:0,mappingProfile:'logicDkdGm',countInNote75:true})));
  expect(plan.kit).toEqual(DEFAULT_KIT);
});

it('starts the measured studio kit without body, approach or generated support collisions', () => {
  const kit = structuredClone(DEFAULT_KIT);
  for (const piece of kit) {
    if (piece.id === 'openHat' || piece.id === 'pedalHat') continue;
    const built = createMeasuredInstrumentBody(piece, resolvePieceVisual('studio', piece.id), 'studio', kit);
    piece.geometry = built.geometry; disposeModel(built.root);
  }
  kit.find(piece => piece.id === 'openHat')!.geometry = structuredClone(kit.find(piece => piece.id === 'hihat')!.geometry);
  expect(solveKitGeometry(kit).unresolvedPieceIds).toEqual([]);
});
