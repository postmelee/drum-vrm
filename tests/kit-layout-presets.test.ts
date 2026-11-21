import { describe, expect, it } from 'vitest';
import { applyKitHardwarePreset, kitHardwarePreset } from '../src/kit-layout-presets';
import { validateKitLayout, sameKitLayout } from '../src/kit-calibration';
import { DEFAULT_KIT } from '../src/kit-default';
import type { KitPiece } from '../src/types';

describe('semantic hardware presets', () => {
  it('drops floor-leg phase when replacing those legs with a stand clamp', () => {
    const input = structuredClone(DEFAULT_KIT);
    input.find(piece => piece.id === 'tomLow')!.standRotation = .8;
    const result = applyKitHardwarePreset(input, 'suspended-floor');
    expect(result.find(piece => piece.id === 'tomLow')!.standRotation).toBeUndefined();
    expect(input.find(piece => piece.id === 'tomLow')!.standRotation).toBe(.8);
    expect(applyKitHardwarePreset(result, 'floor-legs').find(piece => piece.id === 'tomLow')!.standRotation).toBeUndefined();
  });
  it('changes only floor support metadata and preserves accepted measurements', () => {
    const input = structuredClone(DEFAULT_KIT), original = structuredClone(input);
    const result = applyKitHardwarePreset(input, 'suspended-floor');
    expect(input).toEqual(original);
    expect(kitHardwarePreset(result)).toBe('suspended-floor');
    expect(result.find(piece => piece.id === 'tomLow')!.mount).toEqual({kind:'standClamp',supportPieceId:'ride'});
    const withoutLegPhase = structuredClone(original);
    delete withoutLegPhase.find(piece => piece.id === 'tomLow')!.standRotation;
    expect(result.map(({mount, ...piece})=>piece)).toEqual(withoutLegPhase);
    expect(sameKitLayout(input,result)).toBe(false);
    const restored = applyKitHardwarePreset(result,'floor-legs');
    expect(restored).toEqual(withoutLegPhase); expect(kitHardwarePreset(restored)).toBe('floor-legs');
  });

  it('retains explicit mount dependencies through layout validation and rejects invalid links', () => {
    const supported = applyKitHardwarePreset(DEFAULT_KIT,'suspended-floor');
    expect(validateKitLayout(JSON.parse(JSON.stringify(supported)))).toEqual(supported);
    const floor = (kit:KitPiece[])=>kit.find(piece=>piece.id==='tomLow')!;
    for (const host of ['snare','tomLow','missing']) {
      const bad = structuredClone(supported);
      floor(bad).mount = {kind:'standClamp',supportPieceId:host as 'ride'};
      expect(()=>validateKitLayout(bad)).toThrow(/support/);
    }
    const alternate=structuredClone(supported);floor(alternate).mount!.supportPieceId='crash';
    expect(kitHardwarePreset(validateKitLayout(alternate))).toBe('custom');
    const cyclic=structuredClone(supported);cyclic.find(piece=>piece.id==='ride')!.mount={kind:'standClamp',supportPieceId:'tomLow'};
    expect(()=>validateKitLayout(cyclic)).toThrow(/support/);
    const wrongKind=structuredClone(supported);(floor(wrongKind).mount as unknown as {kind:string}).kind='invisible';
    expect(()=>validateKitLayout(wrongKind)).toThrow(/support/);
  });
});
