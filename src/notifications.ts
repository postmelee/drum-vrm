import { Bell, CircleCheck, CircleX, Info, LoaderCircle, TriangleAlert, X, createElement } from 'lucide';
import { bindAttribute, bindText, getLocale, msg, translate } from './i18n';
import { NotificationStore, type Notice } from './notification-store';
import './notifications.css';

const icon = (definition: Parameters<typeof createElement>[0]) => {
  const svg = createElement(definition, { width: 18, height: 18, 'stroke-width': 1.75, 'aria-hidden': 'true' });
  return svg;
};
const kinds = { success: CircleCheck, error: CircleX, warning: TriangleAlert, info: Info, progress: LoaderCircle };
const kindLabels = () => ({ success: msg('Completed', '완료'), error: msg('Failed', '실패'),
  warning: msg('Needs attention', '확인 필요'), info: msg('Information', '안내'), progress: msg('In progress', '진행 중') });

/** Keyed DOM preserves focus and i18n bindings across progress updates and locale changes. */
export function mountNotifications(store: NotificationStore, stage: HTMLElement, bell: HTMLButtonElement) {
  bell.append(icon(Bell));
  const badge = document.createElement('span'); badge.className = 'notification-badge'; badge.hidden = true; bell.append(badge);
  bindAttribute(bell, 'aria-label', msg('Notifications', '알림'));
  bell.setAttribute('aria-expanded', 'false'); bell.setAttribute('aria-controls', 'notification-history');
  const region = document.createElement('section'); region.className = 'notification-region';
  bindAttribute(region, 'aria-label', msg('Notifications', '알림'));
  const stack = document.createElement('div'); stack.className = 'notification-stack';
  const history = document.createElement('section'); history.id = 'notification-history'; history.className = 'notification-history'; history.hidden = true;
  bindAttribute(history, 'aria-label', msg('Notification history', '알림 기록'));
  const heading = document.createElement('header'), title = document.createElement('strong'), closeHistory = document.createElement('button');
  bindText(title, msg('Notifications', '알림')); closeHistory.append(icon(X));
  bindAttribute(closeHistory, 'aria-label', msg('Close notification history', '알림 기록 닫기'));
  heading.append(title, closeHistory);
  const historyList = document.createElement('div'); historyList.className = 'notification-history-list';
  const empty = document.createElement('p'); bindText(empty, msg('No notifications yet.', '아직 알림이 없습니다.'));
  history.append(heading, historyList, empty); region.append(stack, history); stage.append(region);
  const polite = document.createElement('div'), urgent = document.createElement('div');
  for (const [node, live] of [[polite, 'polite'], [urgent, 'assertive']] as const) {
    node.className = 'notification-sr'; node.setAttribute('aria-live', live); node.setAttribute('aria-atomic', 'true'); document.body.append(node);
  }
  type Card = { node: HTMLElement; symbol: HTMLElement; title: HTMLElement; detail: HTMLElement; action: HTMLButtonElement; label: HTMLElement; dismiss: HTMLButtonElement; kind?: string };
  const cards = new Map<number, Card>(), historyCards = new Map<number, Card>();
  const createCard = (n: Notice, historic: boolean): Card => {
    const node = document.createElement('article'); node.className = 'notification-card'; node.tabIndex = -1;
    const symbol = document.createElement('span'); symbol.className = 'notification-symbol';
    const body = document.createElement('div'); body.className = 'notification-body';
    const title = document.createElement('strong'), detail = document.createElement('p');
    const action = document.createElement('button'), label = document.createElement('span');
    action.className = 'notification-action'; label.className = 'notification-action-label'; action.append(label);
    const dismiss = document.createElement('button'); dismiss.className = 'notification-dismiss'; dismiss.append(icon(X));
    bindAttribute(dismiss, 'aria-label', msg('Dismiss notification', '알림 닫기'));
    body.append(title, detail, action); node.append(symbol, body, dismiss);
    dismiss.hidden = historic;
    const dismissFocused = () => {
      const hadFocus = node.contains(document.activeElement);
      store.dismiss(n.id);
      if (hadFocus) (stack.querySelector<HTMLButtonElement>('button:not([hidden])') ?? bell).focus();
    };
    dismiss.onclick = dismissFocused;
    action.onclick = () => { if (!n.resolved && n.action) { if(historic){history.hidden=true;bell.setAttribute('aria-expanded','false');} n.action.run(); if (!historic) store.dismiss(n.id); } };
    if (!historic) {
      node.addEventListener('pointerenter', e => { if (e.pointerType !== 'touch') store.pause(n.id, 'hover', true); });
      node.addEventListener('pointerleave', () => store.pause(n.id, 'hover', false));
      node.addEventListener('focusin', () => store.pause(n.id, 'focus', true));
      node.addEventListener('focusout', e => { if (!node.contains(e.relatedTarget as Node)) store.pause(n.id, 'focus', false); });
      node.addEventListener('keydown', e => { if (e.key === 'Escape') { e.stopPropagation(); e.preventDefault(); dismissFocused(); } });
    }
    return { node, symbol, title, detail, action, label, dismiss };
  };
  const update = (card: Card, n: Notice) => {
    if (card.kind !== n.kind) { card.symbol.replaceChildren(icon(kinds[n.kind])); card.kind = n.kind; }
    card.node.dataset.kind = n.kind;
    bindAttribute(card.node, 'aria-label', kindLabels()[n.kind]);
    bindText(card.title, n.title); bindText(card.detail, n.detail ?? ''); card.detail.hidden = !n.detail;
    card.action.hidden = !n.action || n.resolved;
    if (n.action) bindText(card.label, n.action.label);
  };
  const reduced = matchMedia('(prefers-reduced-motion: reduce)'), narrow = matchMedia('(max-width: 700px)');
  const leaving = new Map<number, ReturnType<typeof setTimeout>>();
  const renderCards = (items: Notice[], parent: HTMLElement, map: Map<number, Card>, historic: boolean) => {
    const ids = new Set(items.map(n => n.id));
    const positions = new Map([...map].map(([id, card]) => [id, card.node.getBoundingClientRect().top]));
    for (const [id, card] of map) if (!ids.has(id)) {
      map.delete(id);
      if (card.node.contains(document.activeElement)) bell.focus();
      if (historic) card.node.remove();
      else {
        card.node.classList.add('notification-leaving'); card.node.inert = true;
        leaving.set(id, setTimeout(() => { card.node.remove(); leaving.delete(id); }, reduced.matches ? 80 : 150));
      }
    }
    items.forEach((n, index) => {
      let card = map.get(n.id);
      if (!card) { card = createCard(n, historic); map.set(n.id, card); }
      update(card, n);
      // Avoid reparenting a focused card on every progress tick.
      const active = [...parent.children].filter(child => !child.classList.contains('notification-leaving'));
      if (active[index] !== card.node) parent.insertBefore(card.node, active[index] ?? null);
      const before = positions.get(n.id), after = card.node.getBoundingClientRect().top;
      if (!historic && !reduced.matches && before !== undefined && before !== after)
        card.node.animate([{ transform: `translateY(${before - after}px)` }, { transform: 'translateY(0)' }], { duration: 180, easing: 'cubic-bezier(.2,0,0,1)' });
    });
  };
  const render = (announcement?: Notice) => {
    renderCards(store.visible, stack, cards, false);
    const count = store.unresolved.length;
    badge.hidden = !count; badge.textContent = String(count);
    bindAttribute(bell, 'aria-label', count ? msg('Notifications · {0} need attention', '알림 · 확인 필요 {0}개', count) : msg('Notifications', '알림'));
    if (!history.hidden) {
      const all = [...store.unresolved, ...store.current, ...store.history];
      const items = [...new Map(all.map(n => [n.id, n])).values()];
      renderCards(items, historyList, historyCards, true); empty.hidden = items.length > 0;
    }
    if (announcement) {
      const live = announcement.kind === 'error' ? urgent : polite;
      live.lang = getLocale(); live.textContent = translate(announcement.title);
    }
  };
  const close = () => { history.hidden = true; bell.setAttribute('aria-expanded', 'false'); bell.focus(); };
  bell.onclick = () => {
    if (!history.hidden) { close(); return; }
    history.hidden = false; bell.setAttribute('aria-expanded', 'true'); render(); closeHistory.focus();
  };
  closeHistory.onclick = close;
  history.addEventListener('keydown', e => { if (e.key === 'Escape') { e.stopPropagation(); e.preventDefault(); close(); } });
  const capacity = () => store.setCapacity(narrow.matches ? 1 : 3);
  const visibility = () => store.setHidden(document.hidden);
  const unsubscribe = store.subscribe(render);
  narrow.addEventListener('change', capacity); document.addEventListener('visibilitychange', visibility);
  capacity(); visibility(); render();
  return () => {
    unsubscribe(); store.dispose(); narrow.removeEventListener('change', capacity); document.removeEventListener('visibilitychange', visibility);
    for (const timer of leaving.values()) clearTimeout(timer);
    region.remove(); polite.remove(); urgent.remove();
  };
}
