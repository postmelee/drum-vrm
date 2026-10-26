import { bindText } from './i18n';
import { createElement, Play, Pause, SkipBack, X, Moon, Sun, Upload, Download, type IconNode } from 'lucide';

const icons: Record<string, IconNode> = { play: Play, pause: Pause, reset: SkipBack, close: X, dark: Moon, light: Sun, upload: Upload, download: Download };

/** Keep the native button and its accessible name; only update SVG on state changes. */
export function setButtonIcon(button: HTMLElement, name: keyof typeof icons, text?: string): void {
  if (button.dataset.icon === name) return;
  button.dataset.icon = name;
  const svg = createElement(icons[name], { width: 16, height: 16, 'stroke-width': 1.75, 'aria-hidden': 'true', focusable: 'false' });
  button.replaceChildren(svg);
  button.classList.add('icon-button');
  if (text) { const label = document.createTextNode(''); bindText(label, text); button.append(label); }
}
