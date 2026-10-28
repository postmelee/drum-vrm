import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { NotificationStore } from '../src/notification-store';
import { NotificationFeedback } from '../src/notification-feedback';

describe('notification lifecycle', () => {
  let store: NotificationStore;
  beforeEach(() => { vi.useFakeTimers(); store = new NotificationStore(); });
  afterEach(() => { store.dispose(); vi.useRealTimers(); });
  it('delays progress and replaces it once; a fast result never flashes progress', () => {
    const announcements: string[] = []; store.subscribe(n => { if(n) announcements.push(n.kind); });
    const id = store.begin('midi', { title: 'Reading' });
    vi.advanceTimersByTime(399); expect(store.visible).toHaveLength(0);
    store.finish(id, 'success', { title: 'Ready' });
    vi.advanceTimersByTime(1); expect(announcements).toEqual(['success']);
    expect(store.visible.map(n => n.id)).toEqual([id]);
    expect(store.finish(id, 'error', { title: 'Stale error' })).toBe(false);
    vi.advanceTimersByTime(4000); expect(store.visible).toHaveLength(0);
    expect(store.history.map(n => n.title)).toEqual(['Ready']);
  });
  it('dismissal of progress does not cancel work and allows exactly one result', () => {
    const id = store.begin('midi', { title: 'Reading' });
    vi.advanceTimersByTime(400); store.dismiss(id); expect(store.current).toHaveLength(1);
    store.update(id, { title: 'Preparing' }); expect(store.visible).toHaveLength(0);
    store.finish(id, 'success', { title: 'Ready' }); expect(store.visible).toHaveLength(1);
    expect(store.history).toHaveLength(1);
  });
  it('pauses the remaining duration for overlapping hover/focus and background tab', () => {
    const id = store.post('audio', 'success', { title: 'Ready' });
    vi.advanceTimersByTime(1500); store.pause(id, 'hover', true); store.pause(id, 'focus', true);
    vi.advanceTimersByTime(10000); store.pause(id, 'hover', false);
    vi.advanceTimersByTime(10000); expect(store.visible).toHaveLength(1);
    store.setHidden(true); store.pause(id, 'focus', false);
    vi.advanceTimersByTime(10000); store.setHidden(false);
    vi.advanceTimersByTime(2499); expect(store.visible).toHaveLength(1);
    vi.advanceTimersByTime(1); expect(store.visible).toHaveLength(0);
  });
  it('keeps dismissed errors and choices outside the 20-item terminal history limit', () => {
    const error = store.post('project', 'error', { title: 'Missing original', action: {label:'Relink',run:vi.fn()} });
    store.dismiss(error);
    for(let i=0;i<30;i++) store.post('other', 'success', { title: String(i) });
    expect(store.history).toHaveLength(20); expect(store.unresolved.map(n=>n.id)).toEqual([error]);
    store.invalidate('project'); expect(store.unresolved).toHaveLength(0);
  });
  it('invalidates source-scoped callbacks and historical actions', () => {
    const action = {label:'Settings',run:vi.fn()};
    const first = store.begin('midi', { title:'Reading' });
    const next = store.begin('midi', { title:'New file' });
    expect(store.finish(first,'success',{title:'Old result',action})).toBe(false);
    store.finish(next,'warning',{title:'Choose mapping',action,required:true});
    vi.advanceTimersByTime(20000); expect(store.visible).toHaveLength(1);
    store.invalidate('midi'); expect(store.history[0].action).toBeUndefined(); expect(store.visible).toHaveLength(0);
  });
  it('prioritizes failures, caps the pending queue, and does not expire a waiting card', () => {
    store.setCapacity(1);
    const first=store.post('midi','success',{title:'Done'});
    vi.advanceTimersByTime(1000);
    const error=store.post('project','error',{title:'Error'});
    expect(store.visible[0].id).toBe(error);
    vi.advanceTimersByTime(20000); store.dismiss(error);
    expect(store.visible[0].id).toBe(first);
    vi.advanceTimersByTime(2999); expect(store.visible[0].id).toBe(first);
    vi.advanceTimersByTime(1); expect(store.visible).toHaveLength(0);
    for(let i=0;i<15;i++)store.post('queue','info',{title:String(i)});
    expect(store.visible).toHaveLength(1);
    expect(store.history.filter(n=>!n.dismissed)).toHaveLength(9);
  });
  it('keeps stable visible order and adjusts capacity without resetting time', () => {
    const ids=[1,2,3].map(i=>store.post('x','success',{title:String(i)}));
    vi.advanceTimersByTime(1000); store.setCapacity(1); expect(store.visible.map(n=>n.id)).toEqual([ids[0]]);
    store.setCapacity(3); expect(store.visible.map(n=>n.id)).toEqual(ids);
    vi.advanceTimersByTime(3000); expect(store.visible).toHaveLength(0);
  });
  it('coalesces nested work, preserves warning severity and cancels stale completion', async () => {
    const feedback = new NotificationFeedback(store, () => ({label:'Details',run:vi.fn()}));
    const action={label:'Analysis',run:vi.fn()};
    await feedback.run('midi','Reading','Ready',action,async()=>{
      await feedback.run('fit','Fitting','Fitted',action,()=>feedback.warn('Needs review',action));
      feedback.success('Ready');
    });
    expect(store.history.map(n=>n.kind)).toEqual(['warning']);
    let complete!:()=>void;
    const pending=feedback.run('midi','Reading','Ready',action,()=>new Promise<void>(resolve=>{complete=resolve;}));
    feedback.cancel(); complete(); await pending;
    expect(store.history.map(n=>n.kind)).toEqual(['info','warning']);
  });
  it('does not let a cancelled operation reject into its replacement', async () => {
    const feedback = new NotificationFeedback(store, () => ({label:'Details',run:vi.fn()}));
    const action={label:'Settings',run:vi.fn()};
    let reject!: (error: Error) => void, finish!: () => void;
    const old=feedback.run('midi','Old file','Old ready',action,()=>new Promise<void>((_,fail)=>{reject=fail;}));
    feedback.cancel();
    const next=feedback.run('midi','New file','New ready',action,()=>new Promise<void>(resolve=>{finish=resolve;}));
    reject(new Error('Late old failure')); await old; finish(); await next;
    expect(store.history.map(n=>[n.kind,n.title])).toEqual([['success','New ready'],['info','Task cancelled']]);
  });
  it('retains a required choice when a later preparation stage has a recoverable warning', async () => {
    const feedback = new NotificationFeedback(store, () => ({label:'Details',run:vi.fn()}));
    const action={label:'Mapping',run:vi.fn()};
    await feedback.run('midi','Reading','Ready',action,()=>{
      feedback.warn('Choose mapping',action,true);
      feedback.warn('Some poses need review',action);
    });
    expect(store.history[0].title).toBe('Choose mapping');
    expect(store.unresolved).toHaveLength(1);
  });
});
