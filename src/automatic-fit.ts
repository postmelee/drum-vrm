import type { HumanoidCalibration } from './rig-humanoid';
import { msg, joinMessages } from './i18n';
import { validateKitLayout } from './kit-calibration';
import type { KitFitProposal, KitPiece, RigProfile } from './types';

export interface PreparedFit {
  kit: KitPiece[];
  proposal: KitFitProposal;
  applied: boolean;
  message: string;
}

/** The baseline is the last explicit layout, never the previous automatic result.
 * Callers keep the active rig/evaluator intact while this candidate is prepared.
 * Project restoration deliberately bypasses this operation. */
export async function prepareAutomaticFit(
  profile: RigProfile,
  baseline: readonly KitPiece[],
  fallback: readonly KitPiece[],
  fit: (profile: RigProfile, kit: readonly KitPiece[]) => Promise<KitFitProposal>,
): Promise<PreparedFit> {
  const proposal = await fit(structuredClone(profile), validateKitLayout(baseline));
  const offset = proposal.seatOffset ?? [0,0,0];
  if (offset.length !== 3 || offset.some(v => !Number.isFinite(v)) || offset[0] !== 0
    || Math.abs(offset[1]) > .12 || offset[2] < -.08 || offset[2] > .24) {
    throw new Error('Invalid automatic seat adjustment.');
  }
  if (proposal.complete) {
    return {
      kit: validateKitLayout(proposal.kit), proposal, applied: true,
      message: msg('Layout adjusted to body measurements · scale {0}%', '체형에 맞게 배치를 조정했습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0)),
    };
  }
  const stages: Record<string,string> = {
    'hihat-witness': msg('Hi-hat contact/preparation', '하이햇 타격·준비 자세'),
    'placement': msg('Instrument and paired-posture placement', '악기·양손 자세 배치'),
    'feet': msg('Seat and pedal reach', '좌석·페달 도달'),
    'coupled-anchor': msg('Coupled seat/pedal/hi-hat candidates', '좌석·페달·하이햇 연동 후보'),
  };
  const trace = proposal.searchFailures?.map(f => msg('{0}: {1} ({2} branches)', '{0}: {1} ({2}개 경로)', stages[f.stage] ?? f.stage, f.pieceId, f.branches));
  const reasons = joinMessages([
    ...(trace ?? []),
    proposal.searchLimitReached ? msg("Layout search limit reached", "배치 탐색 한도 도달") : '',
    !trace?.length && proposal.unreachablePieceIds.length ? msg('Reach/posture: {0}', '도달·자세: {0}', proposal.unreachablePieceIds.join(', ')) : '',
    !trace?.length && proposal.clearancePieceIds?.length ? msg('Clearance: {0}', '간격: {0}', proposal.clearancePieceIds.join(', ')) : '',
  ].filter(Boolean));
  return {
    kit: validateKitLayout(fallback), proposal, applied: false,
    message: reasons ? msg('Automatic fit needs review · {0}. Keeping the existing layout.', '자동 맞춤 확인 필요 · {0}. 기존 배치를 유지합니다.', reasons) : msg('Automatic fit needs review. Keeping the existing layout.', '자동 맞춤 확인 필요. 기존 배치를 유지합니다.'),
  };
}

/** Apply the winning seat only with its complete kit, in the existing transaction. */
export function fittedCalibration(base: HumanoidCalibration, result: PreparedFit): HumanoidCalibration {
  if (!result.applied) return { ...base };
  const offset = result.proposal.seatOffset ?? [0, 0, 0];
  const seat = { ...base, seatHeight: base.seatHeight + offset[1], seatForward: base.seatForward + offset[2] };
  if (!Number.isFinite(seat.seatHeight) || seat.seatHeight < .3 || seat.seatHeight > 1.2
    || !Number.isFinite(seat.seatForward) || Math.abs(seat.seatForward) > .5) {
    throw new Error('Invalid fitted seating.');
  }
  return seat;
}
