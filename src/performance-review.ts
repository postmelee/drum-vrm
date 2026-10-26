import { msg } from './i18n';
import type { PerformancePlan, PoseInspection, RigProfile } from './types';

export interface PoseReview {
  samples: number;
  maxOrderDifference: number;
  nonfiniteValues: number;
  durationSeconds: number;
  proxy?: { scope: 'skeletal-proxies'; samples: number; torsoMinimumMeters: number;
    equipmentMinimumMeters?:number; crossMinimumMeters?:number; armsMinimumMeters: number; sticksMinimumMeters: number; boneLengthMaximumErrorMeters: number;
    intersectingSamples: number; firstIntersectionSeconds: number | null };
}

/** Includes exact impacts and a bounded regular grid. Sampling is evidence, not
 * a continuous collision proof. Does not depend on the order of uploaded notes. */
export function reviewTimes(plan: PerformancePlan, sampleRate = 30): number[] {
  if (!Number.isFinite(sampleRate) || sampleRate < 1 || sampleRate > 60) throw new Error(msg("Inspection rate must be between 1 and 60 fps.", "검수 간격은 1–60 fps 범위여야 합니다."));
  const times = new Set<number>([-0.5, 0, plan.duration]);
  for (let index = 0; index <= Math.ceil(plan.duration * sampleRate); index++) times.add(Math.min(plan.duration, index / sampleRate));
  for (const hit of plan.hits) {
    times.add(hit.time);
    times.add(Math.max(-0.5, hit.time - 0.06));
    times.add(Math.min(plan.duration, hit.time + 0.06));
  }
  const cues=new Set((plan.interpretations??[]).filter(i=>i.kind==='countIn').map(i=>i.sourceEventId));
  for(const e of plan.sourceEvents??[])if(cues.has(e.id)){const t=e.timeUs/1e6;times.add(t);times.add(Math.max(-.5,t-.22));times.add(Math.min(plan.duration+.25,t+.22));}
  times.add(plan.duration+.25);
  return [...times].sort((a, b) => a - b);
}

/** A reproducible permutation, separate from any performer variation seed. */
export function reviewOrder(length: number): number[] {
  const order = Array.from({ length }, (_, index) => index);
  let seed = 0x4d56524d;
  for (let i = length - 1; i > 0; i--) {
    seed = Math.imul(seed ^ seed >>> 15, 0x2c1b3c6d) >>> 0;
    const j = seed % (i + 1);
    [order[i], order[j]] = [order[j], order[i]];
  }
  return order;
}

export function summarizePoseReview(times: number[], ordered: readonly Float32Array[], shuffled: readonly Float32Array[]): PoseReview {
  if (times.length !== ordered.length || times.length !== shuffled.length) throw new Error(msg("Inspection frame counts do not match.", "검수 프레임 수가 다릅니다."));
  let maxOrderDifference = 0, nonfiniteValues = 0;
  ordered.forEach((frame, index) => {
    const other = shuffled[index];
    if (!frame.length || frame.length !== other.length) throw new Error(msg("Inspection poses have different formats or are empty.", "검수 자세 형식이 다르거나 비어 있습니다."));
    frame.forEach((value, component) => {
      if (!Number.isFinite(value) || !Number.isFinite(other[component])) nonfiniteValues++;
      else maxOrderDifference = Math.max(maxOrderDifference, Math.abs(value - other[component]));
    });
  });
  return { samples: times.length, maxOrderDifference, nonfiniteValues, durationSeconds: Math.max(0, times.at(-1) ?? 0) };
}

export function summarizeProxyReview(times: number[], inspections: readonly PoseInspection[]): NonNullable<PoseReview['proxy']> {
  if (!times.length || times.length !== inspections.length) throw new Error(msg("Inspection sample counts do not match.", "검수 표본 수가 다릅니다."));
  const result: NonNullable<PoseReview['proxy']> = { scope:'skeletal-proxies', samples: times.length,
    torsoMinimumMeters: Infinity, armsMinimumMeters: Infinity, sticksMinimumMeters: Infinity,
    boneLengthMaximumErrorMeters: 0, intersectingSamples: 0, firstIntersectionSeconds: null };
  inspections.forEach((sample,index) => {
    const values=[sample.torsoClearanceMeters,sample.armClearanceMeters,sample.stickClearanceMeters,sample.boneLengthErrorMeters];
    if (!values.every(Number.isFinite)) throw new Error(msg("Invalid clearance measurement.", "유효하지 않은 간격 계측값입니다."));
    result.torsoMinimumMeters=Math.min(result.torsoMinimumMeters,sample.torsoClearanceMeters);
    result.armsMinimumMeters=Math.min(result.armsMinimumMeters,sample.armClearanceMeters);
    result.sticksMinimumMeters=Math.min(result.sticksMinimumMeters,sample.stickClearanceMeters);
    result.boneLengthMaximumErrorMeters=Math.max(result.boneLengthMaximumErrorMeters,sample.boneLengthErrorMeters);
    const extra:number[]=[];
    if(sample.equipmentClearanceMeters!==undefined){if(!Number.isFinite(sample.equipmentClearanceMeters))throw new Error(msg("Invalid equipment clearance.", "유효하지 않은 장비 간격입니다."));result.equipmentMinimumMeters=Math.min(result.equipmentMinimumMeters??Infinity,sample.equipmentClearanceMeters);extra.push(sample.equipmentClearanceMeters);}
    if(sample.crossClearanceMeters!==undefined){if(!Number.isFinite(sample.crossClearanceMeters))throw new Error(msg("Invalid crossing clearance.", "유효하지 않은 교차 간격입니다."));result.crossMinimumMeters=Math.min(result.crossMinimumMeters??Infinity,sample.crossClearanceMeters);extra.push(sample.crossClearanceMeters);}
    if (Math.min(...values.slice(0,3),...extra) < 0) { result.intersectingSamples++;result.firstIntersectionSeconds??=times[index]; }
  });
  return result;
}

/** Explicitly copy only setup and aggregate evidence. Never serialize a plan:
 * it contains the user's source note stream. The report is a local download. */
export function createReviewSnapshot(plan: PerformancePlan, profile: RigProfile, setup: Record<string, unknown>, measurement: Record<string, unknown>) {
  const interpretations = plan.interpretations ?? [];
  const counts = Object.fromEntries(['strike','control','countIn','unmapped','preserved','omitted'].map(kind => [kind, interpretations.filter(event => event.kind === kind).length]));
  const diagnosticCounts: Record<string, number> = {};
  for (const diagnostic of plan.diagnostics) diagnosticCounts[diagnostic.code] = (diagnosticCounts[diagnostic.code] ?? 0) + 1;
  return {
    schemaVersion: 1, application: 'Drum VRM', reportKind: 'local-performance-review',
    createdAt: new Date().toISOString(), engineVersion: plan.engineVersion, plannerVersion: plan.plannerVersion,
    source: { midiHash: plan.midiHash, duration: plan.duration, sourceEventCount: plan.sourceEvents?.length ?? 0 },
    interpretation: { mappingProfile: plan.mappingProfile ?? 'gm', mappingVersion: plan.mappingVersion,
      countInNote75: plan.countInNote75 ?? false, hihatPolicy: plan.hihatPolicy ?? 'literal', hihatOpenness: plan.hihatOpenness,
      counts, unassignedHits: plan.hits.filter(hit => hit.limb === null).length, diagnosticCounts },
    profile: structuredClone(profile), kit: structuredClone(plan.kit), setup, measurement,
    limits: ['No uploaded files or source note stream are included.', 'Sampled proxy/contact checks do not certify continuous mesh or skin clearance.'],
  };
}
