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
  if (proposal.complete) {
    return {
      kit: validateKitLayout(proposal.kit), proposal, applied: true,
      message: msg('Layout adjusted to body measurements · scale {0}%', '체형에 맞게 배치를 조정했습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0)),
    };
  }
  const reasons = joinMessages([
    proposal.searchLimitReached ? msg("Layout search limit reached", "배치 탐색 한도 도달") : '',
    proposal.unreachablePieceIds.length ? msg('Reach/posture: {0}', '도달·자세: {0}', proposal.unreachablePieceIds.join(', ')) : '',
    proposal.clearancePieceIds?.length ? msg('Clearance: {0}', '간격: {0}', proposal.clearancePieceIds.join(', ')) : '',
  ].filter(Boolean));
  return {
    kit: validateKitLayout(fallback), proposal, applied: false,
    message: reasons ? msg('Automatic fit needs review · {0}. Keeping the existing layout.', '자동 맞춤 확인 필요 · {0}. 기존 배치를 유지합니다.', reasons) : msg('Automatic fit needs review. Keeping the existing layout.', '자동 맞춤 확인 필요. 기존 배치를 유지합니다.'),
  };
}
