import { describe, expect, it, vi } from 'vitest';
import { prepareAutomaticFit } from '../src/automatic-fit';
import type { KitFitProposal, KitPiece, PieceId, RigProfile } from '../src/types';

const ids: PieceId[]=['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
const layout=(radius=.2):KitPiece[]=>ids.map(id=>({id,label:id,position:[0,id==='kick'||id==='pedalHat'?.08:1,.5],radius}));
const profile={id:'body',lengths:[[.2,.3],[.2,.3],[.4,.4],[.4,.4]]} as RigProfile;
const proposal=(kit:KitPiece[],complete=true):KitFitProposal=>({kit,complete,scale:.8,translation:[0,0,0],pieceAdjustments:[],unreachablePieceIds:complete?[]:['ride'],clearancePieceIds:[]});

describe('automatic import fitting',()=>{
  it('uses the explicit baseline for every import rather than compounding automatic scale',async()=>{
    const baseline=layout(),old=structuredClone(baseline),fit=vi.fn(async(_profile:RigProfile,kit:readonly KitPiece[])=>proposal(kit.map(piece=>({...piece,radius:piece.radius*.8}))));
    const first=await prepareAutomaticFit(profile,baseline,baseline,fit);
    const repeated=await prepareAutomaticFit(profile,baseline,first.kit,fit);
    expect(repeated.kit).toEqual(first.kit);expect(baseline).toEqual(old);
    expect(repeated.kit[0].radius).toBeCloseTo(.16);
  });
  it('retains the active layout when a candidate cannot satisfy fit constraints',async()=>{
    const current=layout(.17),baseline=layout();
    const result=await prepareAutomaticFit(profile,baseline,current,async()=>proposal(layout(.12),false));
    expect(result.applied).toBe(false);expect(result.kit).toEqual(current);expect(result.message).toContain('ride');
    result.kit[0].radius=.3;expect(current[0].radius).toBe(.17);
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
