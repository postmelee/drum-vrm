import { afterEach, describe, expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { ENGINE_MESSAGE_TEMPLATES, engineMessage } from '../src/engine-messages';
import { setLocale, translate } from '../src/i18n';
import { importGuidance } from '../src/import-guidance';
import { inspectWav } from '../src/wav';
import type { PerformancePlan } from '../src/types';

afterEach(() => setLocale('en', { persist: false, updateUrl: false }));

describe('engine presentation boundary', () => {
  it('translates nested diagnostic reasons without changing their original data', () => {
    const source = '1.250초 MIDI 음표 38: 현재 체형·좌석·그립에서 타점에 도달하지 못했습니다. 원본 이벤트와 소리의 시각은 유지하며 미배정 타격의 접촉 효과는 표시하지 않습니다.';
    const diagnostic = Object.freeze({ code: 'UNREACHABLE_TARGET', message: source, time: 1.25 });
    const english = engineMessage(diagnostic.message);
    expect(english).toContain('1.250s, MIDI note 38');
    expect(english).toContain('current body, seat and grip');
    expect(english).not.toMatch(/[가-힣]/);
    setLocale('ko', { persist: false, updateUrl: false });
    expect(translate(english)).toBe(source);
    expect(diagnostic.message).toBe(source);
  });
  it('preserves numerical evidence and source IDs in complex diagnostics', () => {
    const messages = [
      '준비·타격·회수·종료 경로 표본 119곳에서 팔·스틱과 악기/받침대의 대리 형상 겹침이 남았습니다(최대 31.2 mm). 원본 시각과 타점은 유지합니다.',
      '191.804초 카운트인 track-1-event-9: 양손 준비·복귀 구간에 실제 손 타격이 있습니다. 원본 cue와 실제 타격은 유지합니다.',
      'tick 480의 박자표 충돌: 트랙 2의 3/4를 선택했습니다 (트랙 1 우선, 동일 트랙 마지막 이벤트).',
      '트랙 2 / 채널 10 / MIDI 음표 52: 91개의 원본 이벤트를 보존했지만 선택한 매핑으로 연주할 수 없습니다.',
    ];
    for (const source of messages) {
      const english = engineMessage(source);
      expect(english).not.toMatch(/[가-힣]/);
      setLocale('ko', { persist: false, updateUrl: false });
      expect(translate(english)).toBe(source);
      setLocale('en', { persist: false, updateUrl: false });
    }
  });
  it('registers canonical Worker errors in the UI realm before any error is thrown', async () => {
    await import('../src/synthesis');
    setLocale('ko', { persist: false, updateUrl: false });
    expect(translate(engineMessage('Error: Unsupported audio sample rate.'))).toBe('지원하지 않는 오디오 샘플 레이트입니다.');
    expect(translate(engineMessage('Error: Invalid instrument dimensions.'))).toBe('악기 물리 치수가 유효하지 않습니다.');
  });
  it('preserves unknown vendor details instead of inventing translations', () => {
    const unknown = 'Decoder XYZ: unsupported extension';
    expect(engineMessage(unknown)).toBe(unknown);
  });
  it('localizes canonical English contact and posture diagnostics without rewriting them', () => {
    const source = 'The selected hi-hat surface has no movable upper cymbal. Openness changes the pedal state; the displayed strike surface remains fixed.';
    const canonical = engineMessage(source);
    expect(canonical).toBe(source);
    setLocale('ko', { persist: false, updateUrl: false });
    expect(translate(canonical)).toContain('움직일 수 있는 상단 심벌이 없습니다');
    const posture = engineMessage('3 hand-impact poses exceed the common posture limits. Contact timing is preserved; adjust the layout and inspect the arms.');
    expect(translate(posture)).toContain('손 타격 자세 3개');
    const sampledPosture = engineMessage('3 sampled hand poses exceed the common posture limits during preparation, contact, recovery or rest. Original contact times are preserved; inspect the marked interval.');
    expect(translate(sampledPosture)).toContain('준비·타격·회수·대기 중 손 자세 표본 3개');
    const tool=engineMessage('3 sampled whole-stick paths intersect drum equipment (minimum signed proxy clearance -4.2 mm). Original contact times are preserved; inspect the marked interval.');
    expect(translate(tool)).toContain('스틱 전체 경로 표본 3개');
    expect(translate(tool)).toContain('4.2 mm');
    expect(translate(engineMessage('1 sampled whole-stick paths intersect the other stick (minimum signed proxy clearance -2.1 mm). Original contact times are preserved; inspect the marked interval.'))).toContain('반대쪽 스틱');
  });
  it('covers every existing Korean Rust message template', () => {
    const known = new Set(ENGINE_MESSAGE_TEMPLATES.map(([, ko]) => ko));
    const directory = new URL('../crates/drum-engine/src/', import.meta.url);
    const missing: string[] = [];
    for (const name of readdirSync(directory).filter(name => name.endsWith('.rs') && name !== 'tests.rs')) {
      const source = readFileSync(new URL(name, directory), 'utf8');
      for (const match of source.matchAll(/"((?:[^"\\]|\\.)*)"/g)) {
        if (!/[가-힣]/.test(match[1])) continue;
        let index = 0;
        const template = match[1].replace(/\{[^}]*\}/g, () => `{${index++}}`);
        if (!known.has(template)) missing.push(`${name}: ${template}`);
      }
    }
    expect(missing).toEqual([]);
  });
  it('keeps helper errors canonical and allows Korean presentation', () => {
    let message = '';
    try { inspectWav(new ArrayBuffer(12)); } catch (error) { message = (error as Error).message; }
    expect(message).toBe('Not a valid RIFF/WAVE file.');
    setLocale('ko', { persist: false, updateUrl: false });
    expect(translate(message)).toBe('유효한 RIFF/WAVE 파일이 아닙니다.');
    const evidence = importGuidance({ sourceEvents: [], interpretations: [] } as unknown as PerformancePlan).evidence[0];
    expect(evidence).not.toMatch(/[가-힣]/);
    expect(translate(evidence)).toContain('노트만으로');
  });
});
