import { PRE_ROLL, outputDuration } from './types';
import type { Hit, Instrument, PerformancePlan, TimeSignature, EventInterpretation } from './types';

const FONT_STACK = '-apple-system, system-ui, "Segoe UI", "Apple SD Gothic Neo", "Noto Sans KR", sans-serif';
let timelineInstance = 0;

export interface TimelineGridMark {
  outputTime: number;
  tick?: number;
  bar?: number;
  beat?: number;
  label: string;
  major: boolean;
  signature?: string;
}

function lastAtOrBefore<T>(items: T[], value: number, key: (item: T) => number): number {
  let low = 0, high = items.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (key(items[middle]) <= value) low = middle + 1;
    else high = middle;
  }
  return Math.max(0, low - 1);
}

function hasMetricTime(plan: PerformancePlan): boolean {
  return !!plan.ppq && !!plan.tempoMap?.length && plan.tempoMap[0].tick === 0;
}

/** Source ticks use the global MIDI tempo map. Pre-roll is added only by the UI. */
export function tickToSongTime(plan: PerformancePlan, tick: number): number {
  if (!hasMetricTime(plan)) throw new Error('이 계획에는 MIDI 템포맵이 없습니다.');
  const anchors = plan.tempoMap!;
  const anchor = anchors[lastAtOrBefore(anchors, tick, item => item.tick)];
  return anchor.time + (tick - anchor.tick) * anchor.microsecondsPerQuarter / plan.ppq! / 1_000_000;
}

export function songTimeToTick(plan: PerformancePlan, time: number): number {
  if (!hasMetricTime(plan)) throw new Error('이 계획에는 MIDI 템포맵이 없습니다.');
  const anchors = plan.tempoMap!;
  const anchor = anchors[lastAtOrBefore(anchors, time, item => item.time)];
  return anchor.tick + (time - anchor.time) * 1_000_000 * plan.ppq! / anchor.microsecondsPerQuarter;
}

/** Build only the visible grid. Dense files skip whole bars instead of allocating
 * one mark per beat. A meter change starts a new bar; an incomplete prior bar is
 * counted once. Missing legacy metadata uses an honest seconds ruler. */
export function createTimelineGrid(plan: PerformancePlan, fromOutput = 0, toOutput = outputDuration(plan), maxLines = 2000): TimelineGridMark[] {
  const marks: TimelineGridMark[] = [];
  const from = Math.max(0, fromOutput - PRE_ROLL);
  const to = Math.min(plan.duration, toOutput - PRE_ROLL);
  if (to < from) return marks;
  const budget = Math.max(8, Math.floor(maxLines));
  if (!hasMetricTime(plan)) {
    const step = Math.max(1, Math.ceil((to - from) / budget));
    for (let time = Math.ceil(from / step) * step; time <= to; time += step) {
      marks.push({ outputTime: time + PRE_ROLL, label: `${time}s`, major: true });
    }
    return marks;
  }
  const endTick = plan.endTick ?? songTimeToTick(plan, plan.duration);
  const fromTick = Math.max(0, songTimeToTick(plan, from));
  const toTick = Math.min(endTick, songTimeToTick(plan, to));
  const signatures: TimeSignature[] = plan.timeSignatures?.length ? plan.timeSignatures : [{ tick: 0, numerator: 4, denominator: 4 }];
  const segments = signatures.map((signature, i) => {
    const end = Math.min(endTick, signatures[i + 1]?.tick ?? endTick);
    const beatTicks = plan.ppq! * 4 / signature.denominator;
    const count = Math.max(0, Math.min(end, toTick) - Math.max(signature.tick, fromTick)) / beatTicks;
    const exclusiveEnd = signatures[i + 1]?.tick === end;
    return { signature, end, beatTicks, count, exclusiveEnd };
  });
  const totalBeats = segments.reduce((sum, segment) => sum + segment.count, 0);
  let firstBar = 1;
  for (const { signature, end, beatTicks, count, exclusiveEnd } of segments) {
    if (signature.tick > toTick || marks.length >= budget + 8) break;
    const segmentBudget = totalBeats ? budget * count / totalBeats : budget;
    const stride = count <= segmentBudget ? 1 : Math.max(1, Math.ceil(count / Math.max(1, segmentBudget) / signature.numerator)) * signature.numerator;
    const firstBeat = Math.max(0, Math.ceil(((fromTick - signature.tick) / beatTicks - 1e-8) / stride) * stride);
    for (let index = firstBeat; marks.length < budget + 8; index += stride) {
      const tick = signature.tick + index * beatTicks;
      if (tick > end + 1e-8 || tick > toTick + 1e-8 || (exclusiveEnd && tick >= end - 1e-8)) break;
      const beat = index % signature.numerator + 1;
      const bar = firstBar + Math.floor(index / signature.numerator);
      marks.push({ outputTime: tickToSongTime(plan, tick) + PRE_ROLL, tick, bar, beat, label: beat === 1 ? String(bar) : `${bar}.${beat}`, major: beat === 1, ...(index === 0 ? { signature: `${signature.numerator}/${signature.denominator}` } : {}) });
    }
    firstBar += Math.ceil((end - signature.tick) / beatTicks / signature.numerator - 1e-8);
  }
  return marks;
}

const instrumentNames: Record<Instrument, string> = {
  kick: 'Kick', snare: 'Snare', hihat: 'Hi-hat', openHat: 'Open hat', pedalHat: 'Hat pedal',
  tomHigh: 'High tom', tomMid: 'Mid tom', tomLow: 'Low tom', crash: 'Crash', ride: 'Ride',
};

export interface TimelineNote { id: string; time: number; note: number; velocity: number; name: string; kind: EventInterpretation['kind']; hit?: Hit; }
const articulationNames: Record<string,string> = {rimshot:'Snare rimshot',rimshotEdge:'Snare rim edge',sideStick:'Side stick',edge:'Snare edge'};
const hitName = (hit: Hit) => hit.pieceId==='crashRight'?'Crash R':hit.instrument==='crash'?'Crash L':hit.instrument==='snare'?(articulationNames[hit.articulation??'']??'Snare'):hit.instrument==='ride'&&hit.articulation==='edge'?'Ride edge':instrumentNames[hit.instrument];
/** Display source note-ons, including controls/count-in/unmapped notes. Derived
 * hit removal must not erase the original piano-roll evidence. */
export function createTimelineNotes(plan:PerformancePlan):TimelineNote[] {
  if(!plan.sourceEvents?.length)return plan.hits.map(hit=>({id:String(hit.id),time:hit.time,note:hit.note,velocity:hit.velocity,name:hitName(hit),kind:'strike',hit}));
  const meanings=new Map(plan.interpretations?.map(value=>[value.sourceEventId,value]));
  return plan.sourceEvents.flatMap(source=>{
    if(source.message.kind!=='noteOn'||source.message.velocity===0)return [];
    const meaning=meanings.get(source.id),hit=meaning?.hitId===undefined?undefined:plan.hits[meaning.hitId];
    const kind=meaning?.kind??'unmapped';
    return [{id:source.id,time:source.timeUs/1e6,note:source.message.key,velocity:source.message.velocity,
      name:hit?hitName(hit):kind==='countIn'?'Count-in':kind==='control'?'Hat close':`Note ${source.message.key}`,kind,hit}];
  });
}
interface NoteRow { note: number; name: string; hits: TimelineNote[]; }
interface TimelineOptions { onSeek: (outputTime: number) => void; onInspect?: (hit: Hit | null) => void; }
type Theme = 'light' | 'dark';

const colors = {
  light: { background: '#f5f5f5', stripe: '#eeeeee', ruler: '#e5e5e5', keys: '#fafafa', blackKey: '#e6e6e6', line: '#dedede', bar: '#c5c5c5', text: '#353535', muted: '#848484', prep: '#e8e8e8', playhead: '#1d1d1d', selected: '#202020' },
  dark: { background: '#242424', stripe: '#292929', ruler: '#343434', keys: '#303030', blackKey: '#272727', line: '#343434', bar: '#484848', text: '#dfdfdf', muted: '#939393', prep: '#202020', playhead: '#f5f5f5', selected: '#f7f7f7' },
};

/** A read-only drum piano roll. The canvas is viewport-sized even when zoomed,
 * keeping 5-minute files and high-DPI displays from allocating huge bitmaps. */
export class DrumTimeline {
  private readonly root = document.createElement('div');
  private readonly scroll = document.createElement('div');
  private readonly spacer = document.createElement('div');
  private readonly canvas = document.createElement('canvas');
  private readonly playhead = document.createElement('div');
  private readonly inspector = document.createElement('div');
  private readonly accessible = document.createElement('p');
  private readonly observer: ResizeObserver;
  private readonly options: TimelineOptions;
  private plan: PerformancePlan | null = null;
  private rows: NoteRow[] = [];
  private theme: Theme = 'light';
  private zoom = 1;
  private time = 0;
  private selected: TimelineNote | null = null;
  private pixelsPerSecond = 20;
  private width = 1;
  private height = 1;
  private labelWidth = 126;
  private readonly rulerHeight = 27;
  private readonly rowHeight = 20;
  private readonly onScroll = () => { this.draw(); this.drawPlayhead(); };
  private readonly onPointer = (event: PointerEvent) => this.pointer(event);
  private readonly onKey = (event: KeyboardEvent) => this.key(event);

  constructor(container: HTMLElement, options: TimelineOptions) {
    this.options = options;
    this.root.className = 'midi-roll';
    Object.assign(this.root.style, { position: 'relative', width: '100%', height: '100%', minWidth: '0', contain: 'inline-size', minHeight: '120px', overflow: 'hidden' });
    this.scroll.className = 'midi-roll-scroll';
    Object.assign(this.scroll.style, { width: '100%', height: '100%', overflow: 'auto', position: 'relative', scrollbarWidth: 'thin' });
    this.scroll.tabIndex = 0;
    this.scroll.setAttribute('role', 'slider');
    this.scroll.setAttribute('aria-label', 'MIDI 미리보기. 좌우 화살표로 탐색, 위아래로 악기 목록 스크롤');
    this.scroll.setAttribute('aria-valuemin', '0');
    this.scroll.setAttribute('aria-valuemax', '0');
    this.scroll.setAttribute('aria-valuenow', '0');
    this.canvas.className = 'midi-roll-canvas';
    this.canvas.setAttribute('aria-hidden', 'true');
    Object.assign(this.canvas.style, { position: 'sticky', top: '0', left: '0', display: 'block', touchAction: 'pan-x pan-y' });
    this.spacer.append(this.canvas);
    this.scroll.append(this.spacer);
    this.playhead.className = 'midi-roll-playhead';
    Object.assign(this.playhead.style, { position: 'absolute', top: '0', bottom: '12px', width: '1px', pointerEvents: 'none', zIndex: '2' });
    const head = document.createElement('span');
    Object.assign(head.style, { display: 'block', position: 'absolute', left: '-4px', width: '9px', height: '7px', background: 'inherit', clipPath: 'polygon(0 0, 100% 0, 50% 100%)' });
    this.playhead.append(head);
    this.inspector.className = 'midi-roll-inspector';
    this.inspector.setAttribute('role', 'status');
    Object.assign(this.inspector.style, { position: 'absolute', right: '16px', bottom: '15px', padding: '5px 8px', borderRadius: '3px', font: `11px ${FONT_STACK}`, pointerEvents: 'none', zIndex: '3', display: 'none' });
    this.accessible.className = 'midi-roll-sr';
    this.accessible.id = `midi-roll-description-${++timelineInstance}`;
    this.scroll.setAttribute('aria-describedby', this.accessible.id);
    Object.assign(this.accessible.style, { position: 'absolute', width: '1px', height: '1px', padding: '0', margin: '-1px', overflow: 'hidden', clipPath: 'inset(50%)', whiteSpace: 'nowrap' });
    this.root.append(this.scroll, this.playhead, this.inspector, this.accessible);
    container.append(this.root);
    this.scroll.addEventListener('scroll', this.onScroll, { passive: true });
    this.canvas.addEventListener('pointerdown', this.onPointer);
    this.scroll.addEventListener('keydown', this.onKey);
    this.observer = new ResizeObserver(() => this.resize());
    this.observer.observe(container);
    this.setPlan(null);
  }

  setPlan(plan: PerformancePlan | null): void {
    this.plan = plan; this.selected = null; this.time = 0;
    const byNote = new Map<number, NoteRow>();
    for (const hit of plan ? createTimelineNotes(plan) : []) {
      if (!byNote.has(hit.note)) byNote.set(hit.note, { note: hit.note, name: hit.name, hits: [] });
      byNote.get(hit.note)!.hits.push(hit);
    }
    this.rows = [...byNote.values()].sort((a, b) => b.note - a.note);
    this.scroll.scrollLeft = 0; this.scroll.scrollTop = 0;
    this.scroll.setAttribute('aria-valuemax', String(plan ? outputDuration(plan) : 0));
    this.scroll.setAttribute('aria-disabled', String(!plan));
    this.accessible.textContent = plan ? `${this.rows.reduce((count,row)=>count+row.hits.length,0)}개 원본 음표 · ${plan.hits.length}개 타격. ${this.rows.map(row => `MIDI ${row.note} ${row.name}, ${row.hits.length}개`).join('. ')}. 좌우 화살표 0.1초, Shift와 함께 1초, Home과 End로 처음과 끝을 탐색합니다.` : 'MIDI 파일을 불러오면 악기별 음표와 마디가 표시됩니다.';
    this.inspector.style.display = 'none';
    this.resize();
    this.options.onInspect?.(null);
  }

  setTime(outputTime: number): void {
    const next = Math.max(0, Math.min(this.plan ? outputDuration(this.plan) : 0, Number.isFinite(outputTime) ? outputTime : 0));
    const moved = Math.abs(next - this.time) > 0.00001;
    this.time = next;
    if (moved && this.plan) {
      const x = this.labelWidth + this.time * this.pixelsPerSecond - this.scroll.scrollLeft;
      if (x > this.width - 10 || x < this.labelWidth) {
        this.scroll.scrollLeft = Math.max(0, this.time * this.pixelsPerSecond - (this.width - this.labelWidth) * 0.25);
      }
    }
    this.scroll.setAttribute('aria-valuenow', this.time.toFixed(2));
    this.scroll.setAttribute('aria-valuetext', `${this.time.toFixed(2)}초`);
    this.drawPlayhead();
  }

  setTheme(theme: Theme): void { this.theme = theme; this.draw(); this.drawPlayhead(); }

  setZoom(scale: number): void {
    const centerTime = (this.scroll.scrollLeft + (this.width - this.labelWidth) / 2) / this.pixelsPerSecond;
    this.zoom = Math.min(16, Math.max(1, Number.isFinite(scale) ? scale : 1));
    this.resize();
    this.scroll.scrollLeft = Math.max(0, centerTime * this.pixelsPerSecond - (this.width - this.labelWidth) / 2);
    this.draw(); this.drawPlayhead();
  }

  resize(): void {
    const duration = this.plan ? outputDuration(this.plan) : 8;
    // A second measurement accounts for scrollbars introduced by the first pass.
    for (let pass = 0; pass < 2; pass++) {
      this.width = Math.max(1, this.scroll.clientWidth);
      this.height = Math.max(1, this.scroll.clientHeight);
      this.labelWidth = this.width < 600 ? 104 : 126;
      this.pixelsPerSecond = Math.max(1, this.width - this.labelWidth - 12) / duration * this.zoom;
      this.spacer.style.width = `${Math.max(this.width, this.labelWidth + duration * this.pixelsPerSecond + 12)}px`;
      this.spacer.style.height = `${Math.max(this.height, this.rulerHeight + Math.max(7, this.rows.length) * this.rowHeight)}px`;
    }
    const ratio = Math.min(3, window.devicePixelRatio || 1);
    this.canvas.width = Math.round(this.width * ratio);
    this.canvas.height = Math.round(this.height * ratio);
    this.canvas.style.width = `${this.width}px`; this.canvas.style.height = `${this.height}px`;
    this.draw(); this.drawPlayhead();
  }

  private draw(): void {
    const context = this.canvas.getContext('2d');
    if (!context) return;
    const c = colors[this.theme];
    const ratio = this.canvas.width / this.width;
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.clearRect(0, 0, this.width, this.height);
    context.fillStyle = c.background; context.fillRect(0, 0, this.width, this.height);
    context.font = `11px ${FONT_STACK}`;
    context.textBaseline = 'middle';
    const scrollX = this.scroll.scrollLeft, scrollY = this.scroll.scrollTop;
    const xAt = (output: number) => this.labelWidth + output * this.pixelsPerSecond - scrollX;
    const rowCount = Math.max(7, this.rows.length);
    for (let index = 0; index < rowCount; index++) {
      const y = this.rulerHeight + index * this.rowHeight - scrollY;
      if (y + this.rowHeight < this.rulerHeight || y > this.height) continue;
      if (index % 2) { context.fillStyle = c.stripe; context.fillRect(this.labelWidth, Math.max(this.rulerHeight, y), this.width, this.rowHeight); }
      context.strokeStyle = c.line; context.beginPath(); context.moveTo(this.labelWidth, y + this.rowHeight + .5); context.lineTo(this.width, y + this.rowHeight + .5); context.stroke();
    }
    const prepEnd = xAt(PRE_ROLL);
    if (prepEnd > this.labelWidth) { context.fillStyle = c.prep; context.fillRect(this.labelWidth, this.rulerHeight, prepEnd - this.labelWidth, this.height); }
    context.fillStyle = c.ruler; context.fillRect(0, 0, this.width, this.rulerHeight);
    if (this.plan) {
      const from = scrollX / this.pixelsPerSecond;
      const to = (scrollX + this.width - this.labelWidth) / this.pixelsPerSecond;
      const grid = createTimelineGrid(this.plan, from, to, Math.max(16, (this.width - this.labelWidth) / 12));
      let lastLabel = -Infinity;
      for (const mark of grid) {
        const x = Math.round(xAt(mark.outputTime)) + .5;
        if (x < this.labelWidth || x > this.width) continue;
        context.strokeStyle = mark.major ? c.bar : c.line;
        context.beginPath(); context.moveTo(x, mark.major ? 18 : this.rulerHeight); context.lineTo(x, this.height); context.stroke();
        if (mark.major && x - lastLabel > 36) {
          context.fillStyle = c.text;
          const label = mark.signature ? `${mark.label}   ${mark.signature}` : mark.label;
          context.fillText(label, x + 5, 12); lastLabel = x + (mark.signature ? 25 : 0);
        }
      }
      context.save(); context.beginPath(); context.rect(this.labelWidth, this.rulerHeight, this.width - this.labelWidth, this.height - this.rulerHeight); context.clip();
      const noteWidth = Math.max(3, Math.min(26, this.pixelsPerSecond * 0.085));
      for (let index = 0; index < this.rows.length; index++) {
        const row = this.rows[index];
        const y = this.rulerHeight + index * this.rowHeight - scrollY;
        if (y + this.rowHeight < this.rulerHeight || y > this.height) continue;
        const first = Math.max(0, lastAtOrBefore(row.hits, from - PRE_ROLL - noteWidth / this.pixelsPerSecond, hit => hit.time));
        for (let i = first; i < row.hits.length; i++) {
          const hit = row.hits[i];
          const x = xAt(hit.time + PRE_ROLL);
          if (x > this.width) break;
          if (x + noteWidth < this.labelWidth) continue;
          const lightness = (this.theme === 'dark' ? 30 : 35) + hit.velocity / 127 * 24;
          context.fillStyle = hit.kind==='countIn'?c.muted:hit.kind==='control'?`hsl(210 45% ${lightness}%)`:`hsl(125 43% ${lightness}%)`;
          context.fillRect(x, y + 4, noteWidth, this.rowHeight - 8);
          if (hit.kind==='unmapped' || hit.hit?.limb === null || this.selected?.id === hit.id) {
            context.strokeStyle = this.selected?.id === hit.id ? c.selected : '#c79443';
            context.strokeRect(x + .5, y + 4.5, noteWidth - 1, this.rowHeight - 9);
          }
        }
      }
      context.restore();
    }
    // Frozen labels are painted last, so horizontal/vertical scrolling cannot
    // place a note or a grid line over the instrument keyboard.
    context.fillStyle = c.keys; context.fillRect(0, this.rulerHeight, this.labelWidth, this.height);
    for (let index = 0; index < rowCount; index++) {
      const row = this.rows[index];
      const y = this.rulerHeight + index * this.rowHeight - scrollY;
      if (y + this.rowHeight < this.rulerHeight || y > this.height) continue;
      if (row && [1, 3, 6, 8, 10].includes(row.note % 12)) { context.fillStyle = c.blackKey; context.fillRect(0, Math.max(this.rulerHeight, y), this.labelWidth, this.rowHeight); }
      context.fillStyle = c.text;
      if (row && y >= this.rulerHeight - 1) { context.fillText(row.name, 10, y + this.rowHeight / 2); context.fillStyle = c.muted; context.textAlign = 'right'; context.fillText(String(row.note), this.labelWidth - 10, y + this.rowHeight / 2); context.textAlign = 'left'; }
      context.strokeStyle = c.line; context.beginPath(); context.moveTo(0, y + this.rowHeight + .5); context.lineTo(this.labelWidth, y + this.rowHeight + .5); context.stroke();
    }
    context.fillStyle = c.ruler; context.fillRect(0, 0, this.labelWidth, this.rulerHeight);
    context.fillStyle = c.muted; context.fillText('DRUM / NOTE', 10, 13);
    context.strokeStyle = c.bar; context.beginPath(); context.moveTo(this.labelWidth + .5, 0); context.lineTo(this.labelWidth + .5, this.height); context.stroke();
    if (!this.plan) { context.fillStyle = c.muted; context.fillText('MIDI를 불러오면 음표가 표시됩니다', this.labelWidth + 20, this.rulerHeight + 32); }
    else if (prepEnd - this.labelWidth > 42) { context.fillStyle = c.muted; context.fillText('준비', this.labelWidth + 7, 13); }
    this.root.style.background = c.background;
    this.inspector.style.background = this.theme === 'dark' ? '#171717ed' : '#ffffffed';
    this.inspector.style.color = c.text;
  }

  private drawPlayhead(): void {
    const x = this.labelWidth + this.time * this.pixelsPerSecond - this.scroll.scrollLeft;
    this.playhead.style.display = this.plan && x >= this.labelWidth && x <= this.width ? 'block' : 'none';
    this.playhead.style.left = `${x}px`;
    this.playhead.style.background = colors[this.theme].playhead;
  }

  private pointer(event: PointerEvent): void {
    if (!this.plan || event.button !== 0) return;
    const rect = this.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left, y = event.clientY - rect.top;
    if (x < this.labelWidth) return;
    const time = Math.max(0, Math.min(outputDuration(this.plan), (x - this.labelWidth + this.scroll.scrollLeft) / this.pixelsPerSecond));
    const rowIndex = Math.floor((y - this.rulerHeight + this.scroll.scrollTop) / this.rowHeight);
    const row = y >= this.rulerHeight ? this.rows[rowIndex] : undefined;
    let hit: TimelineNote | null = null;
    if (row) {
      const previous = lastAtOrBefore(row.hits, time - PRE_ROLL, item => item.time);
      let closest = Infinity;
      for (const candidate of row.hits.slice(Math.max(0, previous - 1), previous + 2)) {
        const distance = Math.abs((candidate.time + PRE_ROLL - time) * this.pixelsPerSecond);
        if (distance < Math.max(8, this.pixelsPerSecond * .085) && distance < closest) { hit = candidate; closest = distance; }
      }
    }
    this.selected = hit;
    this.inspector.textContent = hit ? `${hit.name} · ${hit.note} · Velocity ${hit.velocity} · ${hit.time.toFixed(3)}s · ${hit.kind==='control'?'하이햇 제어':hit.kind==='countIn'?'카운트인':hit.kind==='unmapped'?'미매핑':hit.hit?.limb==null?'동작 미배정':['왼손','오른손','왼발','오른발'][hit.hit.limb]}` : '';
    this.inspector.style.display = hit ? 'block' : 'none';
    this.options.onInspect?.(hit?.hit??null);
    this.options.onSeek(hit ? hit.time + PRE_ROLL : time);
    this.scroll.focus({ preventScroll: true });
    this.draw();
  }

  private key(event: KeyboardEvent): void {
    if (!this.plan || !['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const delta = event.shiftKey ? 1 : .1;
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? outputDuration(this.plan) : this.time + (event.key === 'ArrowLeft' ? -delta : delta);
    this.options.onSeek(Math.max(0, Math.min(outputDuration(this.plan), next)));
  }

  dispose(): void {
    this.observer.disconnect();
    this.scroll.removeEventListener('scroll', this.onScroll);
    this.canvas.removeEventListener('pointerdown', this.onPointer);
    this.scroll.removeEventListener('keydown', this.onKey);
    this.root.remove();
  }
}
