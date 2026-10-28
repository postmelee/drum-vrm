import { validateKitLayout } from './kit-calibration';
import type { KitPiece } from './types';

/** Physical support choice, independent of shell appearance or accepted fit. */
export type KitHardwarePresetId = 'floor-legs' | 'suspended-floor';

export function kitHardwarePreset(kit: readonly KitPiece[]): KitHardwarePresetId | 'custom' {
  const mount = kit.find(piece => piece.id === 'tomLow')?.mount;
  return !mount ? 'floor-legs' : mount.kind === 'standClamp' && mount.supportPieceId === 'ride' ? 'suspended-floor' : 'custom';
}

export function applyKitHardwarePreset(kit: readonly KitPiece[], preset: KitHardwarePresetId): KitPiece[] {
  if (preset !== 'floor-legs' && preset !== 'suspended-floor') throw new Error('Unknown hardware layout');
  const next = validateKitLayout(kit), floor = next.find(piece => piece.id === 'tomLow')!;
  if (preset === 'suspended-floor') floor.mount = {kind: 'standClamp', supportPieceId: 'ride'};
  else delete floor.mount;
  return validateKitLayout(next);
}
