/** Session-only notification lifecycle. No UI, file contents, or persistent storage. */
export type NoticeKind = 'progress' | 'success' | 'info' | 'warning' | 'error';
export interface NoticeAction { label: string; run: () => void }
export interface NoticeContent {
  title: string;
  detail?: string;
  action?: NoticeAction;
  required?: boolean;
}
export interface Notice extends NoticeContent {
  id: number;
  scope: string;
  kind: NoticeKind;
  dismissed: boolean;
  resolved: boolean;
  remaining: number | null;
  ready: boolean;
}
type Listener = (announcement?: Notice) => void;
const lifetime = (kind: NoticeKind, required = false): number | null =>
  required || kind === 'error' || kind === 'progress' ? null : kind === 'success' ? 4000 : kind === 'warning' ? 8000 : 5000;
const priority = (n: Notice) => n.kind === 'error' || n.required ? 4 : n.kind === 'warning' ? 3 : n.kind === 'progress' ? 2 : 1;

export class NotificationStore {
  private sequence = 0;
  private records = new Map<number, Notice>();
  private listeners = new Set<Listener>();
  private timers = new Map<number, { timer: ReturnType<typeof setTimeout>; started: number }>();
  private delays = new Map<number, ReturnType<typeof setTimeout>>();
  private pauses = new Map<number, Set<string>>();
  private hidden = false;
  private capacity = 3;
  private visibleIds: number[] = [];
  private historyIds: number[] = [];

  get visible() { return this.visibleIds.map(id => this.records.get(id)!); }
  get history() { return this.historyIds.map(id => this.records.get(id)!).filter(Boolean); }
  get unresolved() { return [...this.records.values()].filter(n => !n.resolved && (n.kind === 'error' || n.required)); }
  get current() { return [...this.records.values()].filter(n => n.kind === 'progress'); }
  subscribe(listener: Listener) { this.listeners.add(listener); return () => this.listeners.delete(listener); }

  /** A replacement source makes previous callbacks/actions in that scope stale. */
  invalidate(scope: string) {
    for (const n of this.records.values()) if (n.scope === scope) {
      this.stop(n.id); this.clearDelay(n.id);
      n.action = undefined; n.resolved = true; n.dismissed = true;
      if (n.kind === 'progress') this.records.delete(n.id);
    }
    this.refresh();
  }
  begin(scope: string, content: NoticeContent) {
    this.invalidate(scope);
    const n = this.create(scope, 'progress', content, false);
    this.delays.set(n.id, setTimeout(() => {
      this.delays.delete(n.id);
      if (!this.records.has(n.id)) return;
      n.ready = true; this.refresh(n);
    }, 400));
    this.refresh();
    return n.id;
  }
  update(id: number, content: Partial<NoticeContent>) {
    const n = this.records.get(id);
    if (!n || n.kind !== 'progress') return false;
    Object.assign(n, content); this.refresh(); return true;
  }
  finish(id: number, kind: Exclude<NoticeKind, 'progress'>, content: NoticeContent) {
    const n = this.records.get(id);
    if (!n || n.kind !== 'progress') return false;
    this.stop(id); this.clearDelay(id);
    Object.assign(n, { detail: undefined, action: undefined, required: false }, content,
      { kind, ready: true, dismissed: false, remaining: lifetime(kind, content.required) });
    this.remember(n); this.refresh(n); return true;
  }
  post(scope: string, kind: Exclude<NoticeKind, 'progress'>, content: NoticeContent) {
    const n = this.create(scope, kind, content, true);
    this.remember(n); this.refresh(n); return n.id;
  }
  dismiss(id: number) {
    const n = this.records.get(id); if (!n) return;
    this.stop(id); n.dismissed = true; this.refresh();
  }
  pause(id: number, reason: string, paused: boolean) {
    const reasons = this.pauses.get(id) ?? new Set<string>();
    if (paused) reasons.add(reason); else reasons.delete(reason);
    this.pauses.set(id, reasons); this.syncTimers();
  }
  setHidden(hidden: boolean) { this.hidden = hidden; this.syncTimers(); }
  setCapacity(capacity: 1 | 3) { this.capacity = capacity; this.refresh(); }
  dispose() {
    for (const id of this.timers.keys()) this.stop(id);
    for (const id of this.delays.keys()) this.clearDelay(id);
    this.listeners.clear();
  }
  private create(scope: string, kind: NoticeKind, content: NoticeContent, ready: boolean) {
    const n: Notice = { ...content, id: ++this.sequence, scope, kind, ready, dismissed: false,
      resolved: false, remaining: lifetime(kind, content.required) };
    this.records.set(n.id, n); return n;
  }
  private remember(n: Notice) {
    this.historyIds.unshift(n.id); this.historyIds = this.historyIds.slice(0, 20);
  }
  private clearDelay(id: number) { clearTimeout(this.delays.get(id)); this.delays.delete(id); }
  private stop(id: number) {
    const running = this.timers.get(id); if (!running) return;
    clearTimeout(running.timer); this.timers.delete(id);
    const n = this.records.get(id);
    if (n?.remaining != null) n.remaining = Math.max(0, n.remaining - (Date.now() - running.started));
  }
  private syncTimers() {
    for (const n of this.records.values()) {
      const runnable = this.visibleIds.includes(n.id) && !this.hidden && !this.pauses.get(n.id)?.size && n.remaining != null;
      if (!runnable) this.stop(n.id);
      else if (!this.timers.has(n.id)) this.timers.set(n.id, {
        started: Date.now(), timer: setTimeout(() => this.dismiss(n.id), n.remaining!),
      });
    }
  }
  private refresh(announcement?: Notice) {
    const candidates = [...this.records.values()].filter(n => n.ready && !n.dismissed && !n.resolved);
    const sorted = candidates.sort((a, b) => priority(b) - priority(a) || a.id - b.id);
    // Keep existing card order. Higher-priority arrivals can displace lower-priority cards.
    const chosen = new Set(sorted.slice(0, this.capacity).map(n => n.id));
    this.visibleIds = this.visibleIds.filter(id => chosen.has(id));
    for (const n of sorted) if (chosen.has(n.id) && !this.visibleIds.includes(n.id)) this.visibleIds.push(n.id);
    for (const n of sorted.slice(this.capacity + 8)) n.dismissed = true;
    for (const [id, n] of this.records) {
      if (n.dismissed && n.kind !== 'progress' && (n.resolved || (!n.required && n.kind !== 'error')) && !this.historyIds.includes(id)) {
        this.stop(id); this.records.delete(id); this.pauses.delete(id);
      }
    }
    this.syncTimers();
    for (const listener of this.listeners) listener(announcement);
  }
}
