import { bindText } from './i18n';

// Shared disclosure for explanatory content. Controls and live status belong
// outside it; help is keyboard-accessible and never hidden behind hover alone.
const escape = (text: string) => text.replace(/[&<>"']/g, char => ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[char]!));
export function helpMarkup(title: string, paragraphs: string[]): string {
  return `<details class="help-section"><summary>${escape(title)}</summary>${paragraphs.map(text => `<p>${escape(text)}</p>`).join('')}</details>`;
}
export function createHelp(title: string, paragraphs: string[]): HTMLDetailsElement {
  const details = document.createElement('details'); details.className = 'help-section';
  const summary = document.createElement('summary'); bindText(summary, title); details.append(summary);
  for (const text of paragraphs) { const p = document.createElement('p'); bindText(p, text); details.append(p); }
  return details;
}
