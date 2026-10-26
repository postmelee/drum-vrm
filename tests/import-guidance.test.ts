import { describe, expect, it } from 'vitest';
import { importGuidance } from '../src/import-guidance';
import type { PerformancePlan, SourceEvent } from '../src/types';

const event = (key: number, velocity = 80): SourceEvent => ({ id: `1:${key}`, track: 1, eventIndex: key, tick: key, timeUs: key * 1000, channel: 9, message: { kind: 'noteOn', key, velocity } });
const plan = (sourceEvents: SourceEvent[]): PerformancePlan => ({ schemaVersion: 2, sourceEvents, duration: 1, hits: [], kit: [], tempoCount: 1, diagnostics: [] });

describe('import evidence', () => {
  it('exposes count-in candidates without claiming their meaning or selecting a profile', () => {
    const input = plan([event(75), event(75, 0), event(36)]), before = JSON.stringify(input);
    const guidance = importGuidance(input);
    expect(guidance.countInCandidates).toBe(1);
    expect(guidance.hasHihatChoice).toBe(false);
    expect(JSON.stringify(input)).toBe(before);
    expect(input.mappingProfile).toBeUndefined();
  });
  it('uses interpreted roles and retains the choice after a foot close becomes a control', () => {
    const input = plan([]);
    input.interpretations = [{ sourceEventId: '1:1', kind: 'strike', instrument: 'openHat' }, { sourceEventId: '1:2', kind: 'control' }];
    expect(importGuidance(input)).toMatchObject({ open: 1, foot: 0, paired: 1, hasHihatChoice: true });
  });
  it('warns about ambiguous profile semantics even when both profiles support a note', () => {
    expect(importGuidance(plan([event(52)])).evidence.join(' ')).toContain('china');
    expect(importGuidance(plan([event(52)])).evidence.join(' ')).toContain('ride');
  });
});
