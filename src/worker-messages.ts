import { msg } from './i18n';

/** Registered in both Worker and UI realms so canonical errors crossing
 * postMessage remain translatable without serializing UI catalogs. */
export const WORKER_MESSAGES = {
  kitTargets: msg("Each drum target must be present exactly once.", "모든 드럼 타점이 한 번씩 필요합니다."),
  pieceRole: msg("The drum piece does not match its instrument role.", "드럼 부품과 악기 역할이 일치하지 않습니다."),
  dimensions: msg("Invalid instrument dimensions.", "악기 물리 치수가 유효하지 않습니다."),
  targetRange: msg("The drum target is outside the adjustment range.", "드럼 타점이 조절 범위를 벗어납니다."),
  pedalHeight: msg("Pedal height must be between 4 and 20 cm.", "페달 높이는 4–20cm 범위에서 조절해 주세요."),
  hiHatTarget: msg("Open and closed hi-hat must share the same target.", "열린 하이햇과 닫힌 하이햇은 같은 타점을 사용합니다."),
  missingPiece: msg("No drum piece is connected to this strike.", "타격에 연결된 드럼 부품이 없습니다."),
  audioSampleRate: msg("Unsupported audio sample rate.", "지원하지 않는 오디오 샘플 레이트입니다."),
} as const;
