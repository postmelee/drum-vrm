import { describe, expect, it } from 'vitest';
import { createReviewSnapshot, reviewOrder, reviewTimes, summarizePoseReview, summarizeProxyReview } from '../src/performance-review';
import type { PerformancePlan, PoseInspection, RigProfile } from '../src/types';

const plan = { schemaVersion: 2, duration: .2, hits: [{ id: 0, time: .123456, limb: null, tick: 10, track: 0, channel: 9, note: 38, velocity: 80, instrument: 'snare', target: [0, 1, 0] }], sourceEvents: [{ id: 'private-note', track: 0, eventIndex: 0, tick: 0, timeUs: 0, channel: 9, message: { kind: 'noteOn', key: 75, velocity: 90 } }], interpretations: [{ sourceEventId: 'private-note', kind: 'unmapped' }], kit: [], tempoCount: 1, diagnostics: [{ code: 'unmapped', message: 'Private source detail' }] } as PerformancePlan;

describe('local performance review', () => {
  it('samples exact impacts/preparation as well as regular times', () => {
    const times = reviewTimes(plan);
    expect(times).toContain(.123456);
    expect(times).toContain(.123456 - .06);
    expect(times[0]).toBe(-.5);
    expect(times.at(-1)).toBe(.45);
    expect(new Set(times).size).toBe(times.length);
  });
  it('includes explicit count-in contact, approach and recovery without adding unmapped cues', () => {
    const cuePlan={...plan,duration:2,interpretations:[{sourceEventId:'private-note',kind:'countIn'}],sourceEvents:[{...plan.sourceEvents![0],timeUs:712345}]} as PerformancePlan;
    const times=reviewTimes(cuePlan);
    for(const time of [.712345,.712345-.22,.712345+.22,2.25])expect(times).toContain(time);
    expect(reviewTimes({...cuePlan,interpretations:[]})).not.toContain(.712345);
  });
  it('compares shuffled absolute-time evaluation and detects history-dependent or invalid poses', () => {
    const times = reviewTimes(plan), order = reviewOrder(times.length);
    expect(order).not.toEqual(times.map((_, index) => index));
    expect([...order].sort((a,b) => a-b)).toEqual(times.map((_, index) => index));
    const ordered = times.map(time => Float32Array.of(time, time * 2));
    const shuffled = [...ordered].map(frame => frame.slice());
    expect(summarizePoseReview(times, ordered, shuffled)).toMatchObject({ maxOrderDifference: 0, nonfiniteValues: 0 });
    shuffled[2][0] += .25; shuffled[3][1] = NaN;
    const result = summarizePoseReview(times, ordered, shuffled);
    expect(result.maxOrderDifference).toBeCloseTo(.25, 7);
    expect(result.nonfiniteValues).toBe(1);
  });
  it('keeps private note streams and diagnostic text out of a downloaded aggregate report', () => {
    const report = createReviewSnapshot(plan, { id: 'local-rig' } as RigProfile, { fitState: 'manual' }, { pose: [1,2,3] });
    expect(report.interpretation).toMatchObject({ unassignedHits: 1, diagnosticCounts: { unmapped: 1 } });
    const json = JSON.stringify(report);
    expect(json).not.toContain('private-note');
    expect(json).not.toContain('Private source detail');
    expect(json).not.toContain('"key":75');
  });
  it('records the first proxy intersection separately from pose determinism', () => {
    const sample: PoseInspection={scope:'skeletal-proxies',torsoClearanceMeters:.02,armClearanceMeters:.03,stickClearanceMeters:.01,boneLengthErrorMeters:1e-7,poseFloats:77};
    const result=summarizeProxyReview([0,.1,.2],[sample,{...sample,armClearanceMeters:-.01},{...sample,stickClearanceMeters:-.02}]);
    expect(result).toMatchObject({ firstIntersectionSeconds:.1,intersectingSamples:2,armsMinimumMeters:-.01,sticksMinimumMeters:-.02 });
    expect(()=>summarizeProxyReview([0],[{...sample,torsoClearanceMeters:NaN}])).toThrow(/Invalid/);
    expect(summarizeProxyReview([0],[{...sample,equipmentClearanceMeters:-.008,crossClearanceMeters:.02}])).toMatchObject({intersectingSamples:1,equipmentMinimumMeters:-.008,crossMinimumMeters:.02});
    expect(()=>summarizeProxyReview([0],[{...sample,crossClearanceMeters:NaN}])).toThrow(/Invalid/);
  });
});
