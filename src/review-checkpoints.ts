import type { PerformancePlan } from './types';
import { msg } from './i18n';

/** Semantic landmarks derived from any score, never filenames or model IDs. */
export function reviewCheckpoints(plan: PerformancePlan): { label: string; time: number }[] {
  const result = [{ label: msg('Ready', '준비'), time: -.25 }];
  // Keep failed assignments inspectable instead of hiding their review stage.
  const groove = plan.hits.find(hit => ['hihat', 'openHat'].includes(hit.instrument));
  const tom = plan.hits.find(hit => hit.instrument.startsWith('tom'));
  const ride = plan.hits.find(hit => hit.instrument === 'ride');
  const crash = plan.hits.find(hit => hit.instrument === 'crash' && plan.hits.some(other => other.id !== hit.id && other.instrument === 'crash' && other.pieceId !== hit.pieceId && Math.abs(other.time - hit.time) < .001));
  if (groove) result.push({ label: msg('Groove', '그루브'), time: groove.time });
  if (tom) result.push({ label: msg('Tom transfer', '탐 이동'), time: tom.time });
  if (ride) result.push({ label: msg('Ride', '라이드'), time: ride.time });
  if (crash) result.push({ label: msg('Both crashes', '양쪽 크래시'), time: crash.time });
  result.push({ label: msg('Ending', '종료'), time: plan.duration });
  return result;
}
