import { msg, joinMessages } from './i18n';
import { validateKitLayout } from './kit-calibration';
import type { KitFitProposal, KitPiece, RigProfile } from './types';

export function layoutViolationMessage(code: string): string {
  const messages: Record<string, string> = {
    'snare-body-center': msg('Snare needs to stay centered in front of the body', '스네어가 몸 앞쪽 중앙에 있어야 합니다'),
    'rack-row': msg('Rack toms need a shallow row', '랙 탐을 얕은 한 줄로 배치해야 합니다'),
    'floor-flank': msg('Floor tom needs the right-side sector', '플로어 탐을 오른쪽 영역에 배치해야 합니다'),
    'hihat-left-above-snare': msg('Hi-hat needs clearance above and left of the snare', '하이햇은 스네어 왼쪽 위에 공간이 필요합니다'),
    'ride-right-sector': msg('Ride needs the right-side sector', '라이드를 오른쪽 영역에 배치해야 합니다'),
  };
  if (messages[code]) return messages[code];
  if (code.endsWith('-ahead-of-snare')) return msg('Rack toms must stay ahead of the snare', '랙 탐이 스네어 앞쪽에 있어야 합니다');
  if (code.endsWith('-outer-sector')) return msg('Crashes need their outer left/right sectors', '크래시는 좌우 바깥 영역에 있어야 합니다');
  return code;
}

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
  options: { requireComplete?: boolean } = {},
): Promise<PreparedFit> {
  const proposal = await fit(structuredClone(profile), validateKitLayout(baseline));
  if (proposal.complete) {
    return {
      kit: validateKitLayout(proposal.kit), proposal, applied: true,
      message: proposal.scale < 1
        ? msg('Compact drum shells fitted · size {0}%', '소형 드럼 셸로 체형을 맞췄습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0))
        : msg('Layout adjusted to body measurements · scale {0}%', '체형에 맞게 배치를 조정했습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0)),
    };
  }
  const reasons = joinMessages([
    proposal.searchLimitReached ? msg("Layout search limit reached", "배치 탐색 한도 도달") : '',
    proposal.unreachablePieceIds.length ? msg('Reach/posture: {0}', '도달·자세: {0}', proposal.unreachablePieceIds.join(', ')) : '',
    proposal.clearancePieceIds?.length ? msg('Clearance: {0}', '간격: {0}', proposal.clearancePieceIds.join(', ')) : '',
    proposal.layoutViolations?.length ? joinMessages([...new Set(proposal.layoutViolations.map(layoutViolationMessage))]) : '',
  ].filter(Boolean));
  // A physical mount change cannot retain the newly measured but rejected
  // candidate as fallback. Throw before adoption so its owner restores the
  // accepted equipment, calibration and appearance in one transaction.
  if (options.requireComplete) throw new Error(reasons
    ? msg('Hardware layout was not applied · {0}. The previous setup is restored.', '하드웨어 배치를 적용하지 못했습니다 · {0}. 이전 설정으로 복원합니다.', reasons)
    : msg('Hardware layout was not applied. The previous setup is restored.', '하드웨어 배치를 적용하지 못했습니다. 이전 설정으로 복원합니다.'));
  return {
    kit: validateKitLayout(fallback), proposal, applied: false,
    message: reasons ? msg('Automatic fit needs review · {0}. Keeping the existing layout.', '자동 맞춤 확인 필요 · {0}. 기존 배치를 유지합니다.', reasons) : msg('Automatic fit needs review. Keeping the existing layout.', '자동 맞춤 확인 필요. 기존 배치를 유지합니다.'),
  };
}
