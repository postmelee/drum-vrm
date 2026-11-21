import type { HumanoidCalibration } from './rig-humanoid';
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
  const approach = /^head-approach:([^:]+):([^:]+)$/.exec(code);
  if (approach) return msg('{0} has limited space above its head near {1}', '{0} 위쪽 공간이 {1} 근처에서 좁습니다', approach[1], approach[2]);
  const leg = /^(left|right)-leg-(unreachable|clearance)(?::(.+))?$/.exec(code);
  if (leg) {
    const side = leg[1] === 'left' ? msg('Left leg', '왼쪽 다리') : msg('Right leg', '오른쪽 다리');
    return leg[2] === 'unreachable'
      ? msg('{0} cannot reach its pedal', '{0}가 페달에 닿지 않습니다', side)
      : msg('{0} needs clearance from {1}', '{0}와 {1} 사이에 공간이 필요합니다', side, leg[3] ?? '');
  }
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
  const offset = proposal.seatOffset ?? [0,0,0];
  if (offset.length !== 3 || offset.some(v => !Number.isFinite(v)) || offset[0] !== 0
    || Math.abs(offset[1]) > .12 || offset[2] < -.08 || offset[2] > .24) {
    throw new Error('Invalid automatic seat adjustment.');
  }
  if (proposal.complete) {
    return {
      kit: validateKitLayout(proposal.kit), proposal, applied: true,
      message: proposal.scale < 1
        ? msg('Compact drum shells fitted · size {0}%', '소형 드럼 셸로 체형을 맞췄습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0))
        : msg('Layout adjusted to body measurements · scale {0}%', '체형에 맞게 배치를 조정했습니다 · 크기 {0}%', (proposal.scale * 100).toFixed(0)),
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
    proposal.layoutViolations?.length ? msg('Reference layout: {0}', '기준 배치: {0}', joinMessages([...new Set(proposal.layoutViolations.map(layoutViolationMessage))])) : '',
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
