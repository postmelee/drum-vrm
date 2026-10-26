/** UI messages keep canonical English values in application state. Only explicit
 * DOM bindings and canvas rendering localize them; user files are not rewritten. */
export type Locale = 'en' | 'ko';
interface RawValue { readonly raw: true; readonly value: string; }
type Value = string | number | boolean | null | undefined | RawValue;
interface NestedValue { readonly message: Message; }
type StoredValue = Value | NestedValue;
interface Message { en: string; ko: string; values: readonly StoredValue[]; }
const fixed = new Map<string, Message>();
const recent = new Map<string, Message>();
const RECENT_LIMIT = 4096;

export function raw(value: unknown): RawValue { return { raw: true, value: String(value ?? '') }; }
function isRaw(value: StoredValue): value is RawValue { return typeof value === 'object' && value !== null && 'raw' in value && value.raw === true; }
function isNested(value: StoredValue): value is NestedValue { return typeof value === 'object' && value !== null && 'message' in value; }
function canonical(value: StoredValue): string { return isRaw(value) ? value.value : isNested(value) ? format(value.message.en, value.message.values, false) : String(value ?? ''); }
function format(template: string, values: readonly StoredValue[], localize: boolean, depth = 0): string {
  return template.replace(/\{(\d+)\}/g, (placeholder, key: string) => {
    const index = Number(key);
    if (index >= values.length) return placeholder;
    const value = values[index];
    return localize && isNested(value) && depth < 8 ? render(value.message, depth + 1) : canonical(value);
  });
}
/** Use numbered placeholders and raw(file.name) for values originating outside
 * the app. Nested app messages remain translatable without changing state. */
export function msg(en: string, ko: string, ...values: Value[]): string {
  const source = format(en, values, false);
  // Joining a single app message must not register a same-key wrapper around
  // itself. Repeated rerenders would otherwise accumulate nesting indefinitely.
  const nonempty = values.filter(value => canonical(value) !== '');
  if (en === ko && en.replace(/\{\d+\}/g, '') === '' && nonempty.length === 1 &&
      typeof nonempty[0] === 'string' && nonempty[0] === source && lookup(source)) return source;
  const stored = values.map(value => typeof value === 'string' && lookup(value) ? { message: lookup(value)! } : value);
  const message: Message = { en, ko, values: stored };
  if (!values.length) fixed.set(source, message);
  else {
    recent.delete(source); recent.set(source, message);
    if (recent.size > RECENT_LIMIT) recent.delete(recent.keys().next().value!);
  }
  return source;
}
function lookup(source: string): Message | undefined { return recent.get(source) ?? fixed.get(source); }
function render(message: Message, depth = 0): string { return format(locale === 'ko' ? message.ko : message.en, message.values, locale === 'ko', depth); }
function resolve(source: string, depth = 0): string {
  if (depth > 8 || locale === 'en') return source;
  const message = lookup(source);
  return message ? render(message, depth) : source;
}
export function translate(source: string): string { return resolve(source); }
export function joinMessages(parts: readonly Value[], separator = ' · '): string {
  const template = parts.map((_, index) => `{${index}}`).join(separator);
  return msg(template, template, ...parts);
}

export function resolveLocale(url: string, stored?: string | null): Locale {
  try { const query = new URL(url, 'https://drum-vrm.invalid/').searchParams.get('lang'); if (query === 'en' || query === 'ko') return query; } catch { /* An invalid URL uses the preference/default. */ }
  return stored === 'ko' ? 'ko' : 'en';
}
function initialLocale(): Locale {
  if (typeof window === 'undefined') return 'en';
  let stored: string | null = null;
  try { stored = localStorage.getItem('drum-language'); } catch { /* Storage is optional. */ }
  return resolveLocale(window.location.href, stored);
}
let locale: Locale = initialLocale();
const listeners = new Set<() => void>();
interface Binding { source: string; message?: Message; literal?: boolean; }
interface TargetBindings { target: WeakRef<Node>; fields: Map<string, Binding>; }
const targetBindings = new WeakMap<Node, TargetBindings>();
const bindings = new Set<TargetBindings>();
function bindingFor(target: Node): TargetBindings {
  let entry = targetBindings.get(target);
  if (!entry) { entry = { target: new WeakRef(target), fields: new Map() }; targetBindings.set(target, entry); bindings.add(entry); }
  return entry;
}
function display(binding: Binding): string { return binding.literal ? binding.source : binding.message ? render(binding.message) : translate(binding.source); }
function apply(target: Node, field: string, binding: Binding): void {
  if (field === '#text') target.textContent = display(binding);
  else (target as Element).setAttribute(field, display(binding));
}
export function bindText(target: Node, source: string | RawValue | null): void {
  const text = canonical(source), binding = { source: text, message: isRaw(source) ? undefined : lookup(text), literal: isRaw(source) };
  bindingFor(target).fields.set('#text', binding); apply(target, '#text', binding);
}
export function bindAttribute(target: Element, attribute: string, source: string): void {
  const binding = { source, message: lookup(source) };
  bindingFor(target).fields.set(attribute, binding); apply(target, attribute, binding);
}
/** The source of an explicitly bound message, useful for intentional appends. */
export function boundText(target: Node): string { return targetBindings.get(target)?.fields.get('#text')?.source ?? target.textContent ?? ''; }
/** Called once on owned static markup, before any uploaded content is inserted.
 * Text nodes are bound individually so controls, selections and listeners survive. */
export function bindStaticTranslations(root: Element): void {
  const visit = (node: Node) => {
    if (node.nodeType === 3) {
      const source = node.textContent ?? '', trimmed = source.trim();
      if (lookup(trimmed)) {
        const leading = source.slice(0, source.indexOf(trimmed)), trailing = source.slice(source.indexOf(trimmed) + trimmed.length);
        bindText(node, leading || trailing ? msg('{0}{1}{2}', '{0}{1}{2}', raw(leading), trimmed, raw(trailing)) : source);
      }
    } else if (node.nodeType === 1) {
      const element = node as Element;
      if (element.hasAttribute('data-no-localize')) return;
      for (const name of ['aria-label', 'title', 'placeholder', 'href']) {
        const source = element.getAttribute(name);
        if (source && lookup(source)) bindAttribute(element, name, source);
      }
      for (const child of [...node.childNodes]) visit(child);
    }
  };
  visit(root);
}
export function getLocale(): Locale { return locale; }
export function onLocaleChange(listener: () => void): () => void { listeners.add(listener); return () => listeners.delete(listener); }
export function localizedUrl(url: string, language: Locale): string {
  const result = new URL(url, typeof window === 'undefined' ? 'https://drum-vrm.invalid/' : window.location.href);
  result.searchParams.set('lang', language); return result.href;
}
export function setLocale(next: Locale, options: { persist?: boolean; updateUrl?: boolean } = {}): void {
  locale = next;
  if (typeof document !== 'undefined') document.documentElement.lang = next;
  if (typeof window !== 'undefined') {
    if (options.persist !== false) try { localStorage.setItem('drum-language', next); } catch { /* Storage is optional. */ }
    if (options.updateUrl !== false) try { window.history.replaceState(window.history.state, '', localizedUrl(window.location.href, next)); } catch { /* Embedded contexts may restrict history. */ }
  }
  for (const entry of bindings) {
    const target = entry.target.deref();
    if (!target) { bindings.delete(entry); continue; }
    for (const [field, binding] of entry.fields) apply(target, field, binding);
  }
  for (const listener of listeners) listener();
}
if (typeof document !== 'undefined') document.documentElement.lang = locale;
