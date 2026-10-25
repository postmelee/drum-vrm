import type { PerformancePlan } from './types';

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
