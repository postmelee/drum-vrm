import { beforeAll, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { automaticImportSelection, automaticMappingMessage } from '../src/automatic-mapping';
import { initSync, inspect_midi, plan_midi } from '../src/wasm/drum_engine';
import { makeSongsterrStudy } from './songsterr-fixture';
import { makeInterpretationDemo } from '../src/demo';
import type { MidiInfo } from '../src/types';

beforeAll(() => initSync({module:readFileSync(new URL('../src/wasm/drum_engine_bg.wasm',import.meta.url))}));

it.each([0,1] as const)('automatically imports format %i without a failed GM planning pass', format => {
  const bytes=makeSongsterrStudy(format),info:MidiInfo=JSON.parse(inspect_midi(bytes));
  const selection=automaticImportSelection(info);
  expect(selection).toEqual({
    tracks:[format],channel:9,mappingProfile:'songsterrDownloader',
    countInNote75:false,hihatPolicy:'literal',hihatOpenness:.2,omittedSourceEventIds:[],handOverrides:{},
  });
  const plan=JSON.parse(plan_midi(bytes,JSON.stringify(selection)));
  expect(plan.hits).toHaveLength(7);
  expect(plan.mappingProfile).toBe('songsterrDownloader');
  expect(info.mappingAnalysis.candidates.map(c=>c.mapped)).toEqual([0,0,7]);
  expect(automaticMappingMessage(info.mappingAnalysis)).toContain('Automatically selected Songsterr Downloader');
  selection.tracks.length=0;
  expect(info.suggestedTracks).toEqual([format]);
});

it('chooses Logic for its distinct source notes without enabling intent policies',()=>{
  const info:MidiInfo=JSON.parse(inspect_midi(makeInterpretationDemo()));
  const selection=automaticImportSelection(info);
  expect(selection.mappingProfile).toBe('logicDkdGm');
  expect(selection.countInNote75).toBe(false);
  expect(selection.hihatPolicy).toBe('literal');
});

it('labels a conflicting tie as provisional and reports every candidate coverage',()=>{
  const analysis={selected:'gm' as const,ambiguous:true,totalNotes:3,candidates:[
    {profile:'gm' as const,mapped:3,unmapped:0},{profile:'logicDkdGm' as const,mapped:3,unmapped:0},
    {profile:'songsterrDownloader' as const,mapped:0,unmapped:3},
  ]};
  const text=automaticMappingMessage(analysis);
  expect(text).toContain('Provisional choice: General MIDI');
  expect(text).toContain('disagree');
  expect(text).toContain('General MIDI: 3/3');
  expect(text).toContain('Songsterr Downloader: 0/3');
  expect(automaticMappingMessage({...analysis,selected:null,ambiguous:false})).toContain('No mapping candidate');
});
