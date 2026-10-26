import { afterEach, describe, expect, it } from 'vitest';
import { bindAttribute, bindStaticTranslations, bindText, boundText, getLocale, joinMessages, localizedUrl, msg, onLocaleChange, raw, resolveLocale, setLocale, translate } from '../src/i18n';
import { studioTemplate } from '../src/studio-template';
import { DEMO_CATALOG } from '../src/demo-catalog';

const change = (locale: 'en' | 'ko') => setLocale(locale, { persist: false, updateUrl: false });
afterEach(() => change('en'));

// The binding contract uses Node text and Element attributes only. Browser
// verification additionally covers actual controls, canvas and loaded assets.
function textNode(text = ''): Node { return { nodeType: 3, textContent: text, childNodes: [] } as unknown as Node; }
function element(children: Node[] = [], values: Record<string, string> = {}): Element {
  const attributes = new Map(Object.entries(values));
  return { nodeType: 1, childNodes: children, hasAttribute: (name: string) => attributes.has(name), getAttribute: (name: string) => attributes.get(name) ?? null, setAttribute: (name: string, value: string) => attributes.set(name, value) } as unknown as Element;
}

describe('UI locale selection and messages', () => {
  it('defaults to English, remembers a valid choice, and lets explicit URLs override it', () => {
    expect(resolveLocale('https://example.test/drum-vrm/')).toBe('en');
    expect(resolveLocale('https://example.test/drum-vrm/', 'ko')).toBe('ko');
    expect(resolveLocale('https://example.test/drum-vrm/?lang=en', 'ko')).toBe('en');
    expect(resolveLocale('https://example.test/drum-vrm/?lang=ko', 'en')).toBe('ko');
    expect(resolveLocale('https://example.test/drum-vrm/?lang=fr', 'unknown')).toBe('en');
    const url = new URL(localizedUrl('https://example.test/drum-vrm/?view=player#notes', 'ko'));
    expect(url.searchParams.get('lang')).toBe('ko');
    expect(url.searchParams.get('view')).toBe('player');
    expect(url.hash).toBe('#notes');
  });

  it('keeps canonical state in English and translates nested messages but not raw user text', () => {
    const ready = msg('Ready for playback', '재생 준비 완료');
    const result = msg('{0}: {1}', '{0}: {1}', ready, raw('Ready for playback'));
    change('ko');
    expect(getLocale()).toBe('ko');
    expect(result).toBe('Ready for playback: Ready for playback');
    expect(translate(result)).toBe('재생 준비 완료: Ready for playback');
    expect(translate('사용자가 만든 파일.mid')).toBe('사용자가 만든 파일.mid');
    expect(translate(joinMessages([ready, raw('Ready for playback')], ' / '))).toBe('재생 준비 완료 / Ready for playback');
  });

  it('does not build same-key translation wrappers when one-message lists rerender', () => {
    const target = textNode();
    for (let index = 0; index < 30; index++) {
      const source = msg('One diagnostic', '진단 한 개');
      bindText(target, joinMessages([source], ' '));
      bindText(target, msg('{0}{1}', '{0}{1}', '', source));
    }
    change('ko');
    expect(target.textContent).toBe('진단 한 개');
  });

  it('retains bound translations after dynamic-message cache eviction and supports literal replacement', () => {
    const nested = msg('Sample {0}', '표본 {0}', 17);
    const target = textNode();
    bindText(target, msg('Result: {0}', '결과: {0}', nested));
    for (let index = 0; index < 4200; index++) msg('Tick {0}', '틱 {0}', index);
    change('ko');
    expect(target.textContent).toBe('결과: 표본 17');
    expect(boundText(target)).toBe('Result: Sample 17');
    bindText(target, raw('Result: Sample 17'));
    change('en'); change('ko');
    expect(target.textContent).toBe('Result: Sample 17');
  });

  it('updates explicitly bound text and attributes without replacing control nodes or their values', () => {
    const name = textNode(msg('Choose test file', '테스트 파일 선택'));
    const input = Object.assign(element(), { value: 'kept input', checked: true });
    const userLabel = textNode('캐릭터 파일.vrm');
    const root = element([name, input, userLabel], { 'aria-label': msg('Test controls', '테스트 제어') });
    bindStaticTranslations(root);
    bindAttribute(input, 'title', msg('A control', '입력 항목'));
    let notifications = 0;
    const stop = onLocaleChange(() => { notifications++; expect(name.textContent).toBe('테스트 파일 선택'); });
    change('ko'); stop();
    expect(name.textContent).toBe('테스트 파일 선택');
    expect(root.getAttribute('aria-label')).toBe('테스트 제어');
    expect(input.getAttribute('title')).toBe('입력 항목');
    expect(root.childNodes[1]).toBe(input);
    expect(input.value).toBe('kept input');
    expect(input.checked).toBe(true);
    expect(userLabel.textContent).toBe('캐릭터 파일.vrm');
    expect(notifications).toBe(1);
  });

  it('ships English markup and public demo labels with a Korean language control', () => {
    expect(studioTemplate).toContain('Files / Audio');
    expect(studioTemplate).toContain('id="language"');
    expect(studioTemplate).toContain('Switch to Korean');
    expect(studioTemplate).toContain('>한국어</button>');
    expect(studioTemplate).not.toContain('MIDI 불러오기');
    expect(DEMO_CATALOG.map(demo => demo.label)).toEqual(['Studio Groove · 30 s', 'Interpretation check · 8 s', 'Transparent Girl 1 · 3 min 13 s']);
    change('ko');
    expect(translate(DEMO_CATALOG[2].label)).toBe('투명소녀1 · 3분 13초');
    expect(DEMO_CATALOG[2].filename.normalize('NFC')).toBe('투명소녀1.mid');
  });
});
