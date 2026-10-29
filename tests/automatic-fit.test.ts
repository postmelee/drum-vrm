import { describe, expect, it, vi } from 'vitest';
import { layoutViolationMessage, prepareAutomaticFit } from '../src/automatic-fit';
import { applyKitHardwarePreset } from '../src/kit-layout-presets';
import { setLocale, translate } from '../src/i18n';
import type { KitFitProposal, KitPiece, PieceId, RigProfile } from '../src/types';

const ids: PieceId[]=['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
const layout=(radius=.2):KitPiece[]=>ids.map(id=>({id,label:id,position:[0,id==='kick'||id==='pedalHat'?.08:1,.5],radius}));
const profile={id:'body',lengths:[[.2,.3],[.2,.3],[.4,.4],[.4,.4]]} as RigProfile;
const proposal=(kit:KitPiece[],complete=true):KitFitProposal=>({kit,complete,scale:.8,translation:[0,0,0],pieceAdjustments:[],unreachablePieceIds:complete?[]:['ride'],clearancePieceIds:[]});

describe('automatic import fitting',()=>{
  it('retains layout advisories on a complete fit and explains head access in both languages',async()=>{
    const code='head-approach:tomMid:crashRight';
    const result=await prepareAutomaticFit(profile,layout(),layout(),async()=>({...proposal(layout()),advisoryLayoutViolations:[code]}),{requireComplete:true});
    expect(result.applied).toBe(true);expect(result.proposal.advisoryLayoutViolations).toEqual([code]);
    expect(layoutViolationMessage(code)).toBe('tomMid has limited space above its head near crashRight');
    try {setLocale('ko');expect(translate(layoutViolationMessage(code))).toBe('tomMid 위쪽 공간이 crashRight 근처에서 좁습니다');} finally {setLocale('en');}
  });
  it('uses the explicit baseline for every import rather than compounding automatic scale',async()=>{
    const baseline=layout(),old=structuredClone(baseline),fit=vi.fn(async(_profile:RigProfile,kit:readonly KitPiece[])=>proposal(kit.map(piece=>({...piece,radius:piece.radius*.8}))));
    const first=await prepareAutomaticFit(profile,baseline,baseline,fit);
    const repeated=await prepareAutomaticFit(profile,baseline,first.kit,fit);
    expect(repeated.kit).toEqual(first.kit);expect(baseline).toEqual(old);
    expect(repeated.kit[0].radius).toBeCloseTo(.16);
    expect(repeated.message).toBe('Compact drum shells fitted · size 80%');
  });
  it('retains the active layout when a candidate cannot satisfy fit constraints',async()=>{
    const current=layout(.17),baseline=layout();
    const result=await prepareAutomaticFit(profile,baseline,current,async()=>proposal(layout(.12),false));
    expect(result.applied).toBe(false);expect(result.kit).toEqual(current);expect(result.message).toContain('ride');
    result.kit[0].radius=.3;expect(current[0].radius).toBe(.17);
  });
  it('rejects an incomplete physical mount change before any adoption instead of accepting its measured fallback',async()=>{
    const accepted=layout(.17), candidate=applyKitHardwarePreset(accepted,'suspended-floor');
    const before=structuredClone({accepted,candidate}), adopt=vi.fn();
    await expect(prepareAutomaticFit(profile,candidate,candidate,async()=>({...proposal(candidate,false),clearancePieceIds:['tomLow']}),{requireComplete:true}).then(adopt)).rejects.toThrow('Hardware layout was not applied');
    expect(adopt).not.toHaveBeenCalled();expect({accepted,candidate}).toEqual(before);
    const result=await prepareAutomaticFit(profile,candidate,accepted,async()=>proposal(candidate),{requireComplete:true});
    expect(result.applied).toBe(true);expect(result.kit.find(piece=>piece.id==='tomLow')!.mount).toEqual({kind:'standClamp',supportPieceId:'ride'});
  });
  it('distinguishes compact and full-size fits in both interface languages',async()=>{
    const compact=await prepareAutomaticFit(profile,layout(),layout(),async()=>proposal(layout()));
    const full=await prepareAutomaticFit(profile,layout(),layout(),async()=>({...proposal(layout()),scale:1}));
    expect(compact.message).toContain('Compact drum shells');expect(full.message).not.toContain('Compact');
    try {
      setLocale('ko');expect(translate(compact.message)).toBe('소형 드럼 셸로 체형을 맞췄습니다 · 크기 80%');
      expect(translate(full.message)).toBe('체형에 맞게 배치를 조정했습니다 · 크기 100%');
    } finally { setLocale('en'); }
  });
  it('rejects a malformed success before any caller can adopt it',async()=>{
    const malformed=layout();malformed[0].position[1]=NaN;
    await expect(prepareAutomaticFit(profile,layout(),layout(),async()=>proposal(malformed))).rejects.toThrow();
  });
  it('propagates calculation failure without mutating the supplied profile or either layout',async()=>{
    const baseline=layout(),current=layout(.18),before=structuredClone({profile,baseline,current});
    await expect(prepareAutomaticFit(profile,baseline,current,async(body,kit)=>{
      body.id='discarded';kit[0].position[0]=1;throw new Error('fit failed');
    })).rejects.toThrow('fit failed');
    expect({profile,baseline,current}).toEqual(before);
  });
});

it('applies a successful seat and kit together and leaves failed seating untouched',async()=>{
  const {fittedCalibration}=await import('../src/automatic-fit');
  const base={seatHeight:.7,seatForward:.06,palmOrientation:'natural' as const};
  const success=await prepareAutomaticFit(profile,layout(),layout(),async()=>({...proposal(layout()),seatOffset:[0,.06,.16]}));
  expect(fittedCalibration(base,success)).toEqual({...base,seatHeight:.76,seatForward:.22});
  const failure=await prepareAutomaticFit(profile,layout(),layout(),async()=>({...proposal(layout(),false),seatOffset:[0,.06,.16],searchFailures:[{stage:'coupled-anchor',pieceId:'hihat',branches:4}]}));
  expect(fittedCalibration(base,failure)).toEqual(base);
  expect(failure.message).toContain('Coupled seat/pedal/hi-hat candidates');
  expect(failure.message).not.toContain('Reach/posture: ride');
  await expect(prepareAutomaticFit(profile,layout(),layout(),async()=>({...proposal(layout()),seatOffset:[0,NaN,0]}))).rejects.toThrow();
  expect(()=>fittedCalibration({...base,seatHeight:1.19},success)).toThrow();
});

it('reports canonical layout failures without adopting the rejected candidate', async () => {
  const current = layout(.17), result = await prepareAutomaticFit(profile, layout(), current, async () => ({...proposal(layout(.12), false), layoutViolations: ['rack-row', 'tomHigh-ahead-of-snare']}));
  expect(result.applied).toBe(false); expect(result.kit).toEqual(current);
  expect(result.message).toContain('Rack toms need a shallow row');
  expect(result.message).toContain('Rack toms must stay ahead of the snare');
});

it('keeps search-stage and returned-layout evidence distinct for an incomplete hardware fit', async () => {
  const accepted = layout(.17), candidate = applyKitHardwarePreset(accepted, 'suspended-floor');
  const failed: KitFitProposal = {
    ...proposal(candidate, false),
    seatOffset: [0, .06, .08],
    searchFailures: [{stage: 'placement', pieceId: 'snare', branches: 3}],
    layoutViolations: ['left-leg-clearance:snare', 'right-leg-unreachable'],
  };
  const result = await prepareAutomaticFit(profile, candidate, accepted, async () => failed);
  expect(result.kit).toEqual(accepted);
  expect(result.message).toContain('Instrument and paired-posture placement: snare (3 branches)');
  expect(result.message).toContain('Reference layout: Left leg needs clearance from snare');
  expect(result.message).toContain('Right leg cannot reach its pedal');
  expect(result.message).not.toContain('Reach/posture: ride');
  await expect(prepareAutomaticFit(profile, candidate, candidate, async () => failed, {requireComplete: true}))
    .rejects.toThrow('Hardware layout was not applied');
  try {
    setLocale('ko');
    expect(translate(result.message)).toContain('왼쪽 다리와 snare 사이에 공간이 필요합니다');
    expect(translate(result.message)).toContain('오른쪽 다리가 페달에 닿지 않습니다');
  } finally { setLocale('en'); }
});
