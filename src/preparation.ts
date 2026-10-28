import type { KitPiece, PerformancePlan, RigProfile } from './types';

/** Build audio against a noncommitted plan. An invalid mapping, incompatible
 * source override, failed synthesis or canceled request leaves the active
 * evaluator and its audio together until the entire candidate is ready. */
export async function preparePerformance<T>(options:{
  candidate:()=>Promise<PerformancePlan>;
  audio:(plan:PerformancePlan)=>Promise<T>|T;
  commit:(plan:PerformancePlan)=>Promise<PerformancePlan>;
  current:()=>boolean;
}):Promise<{plan:PerformancePlan;audio:T}|null> {
  const candidate=await options.candidate();
  if(!options.current())return null;
  const audio=await options.audio(candidate);
  if(!options.current())return null;
  const plan=await options.commit(candidate);
  return {plan,audio};
}

/** Snapshot the accepted body/equipment before asynchronous MIDI/audio work.
 * Song preparation consumes this setup; it never fits or measures a new one. */
export async function preparePerformanceInSetup<T>(options: {
  setup: { profile: RigProfile; kit: readonly KitPiece[] };
  candidate: () => Promise<PerformancePlan>;
  audio: (plan: PerformancePlan) => Promise<T> | T;
  commit: (plan: PerformancePlan, profile: RigProfile, kit: KitPiece[]) => Promise<PerformancePlan>;
  current: () => boolean;
}): Promise<{ plan: PerformancePlan; audio: T } | null> {
  const setup = structuredClone(options.setup);
  return preparePerformance({ ...options,
    commit: candidate => options.commit(candidate, setup.profile, setup.kit as KitPiece[]),
  });
}
