import { msg } from './i18n';
import { NotificationStore, type NoticeAction, type NoticeContent, type NoticeKind } from './notification-store';

interface Operation { id: number; scope: string; result: NoticeContent; kind: Exclude<NoticeKind, 'progress'> }
/** Nested preparation steps report into their owning operation instead of emitting extra cards. */
export class NotificationFeedback {
  private active?: Operation;
  constructor(readonly store: NotificationStore, private details: (text: string) => NoticeAction) {}
  async run<T>(scope: string, title: string, success: string, action: NoticeAction, work: () => T | Promise<T>): Promise<T | undefined> {
    if (this.active) return work();
    const operation: Operation = { id: this.store.begin(scope, { title }), scope, kind: 'success', result: { title: success, action } };
    this.active = operation;
    try { return await work(); }
    catch (error) { if (this.active === operation) this.fail(error instanceof Error ? error.message : String(error)); return undefined; }
    finally {
      this.store.finish(operation.id, operation.kind, operation.result);
      if (this.active === operation) this.active = undefined;
    }
  }
  progress(title: string) { if (this.active) this.store.update(this.active.id, { title }); }
  success(title: string) { if (this.active?.kind === 'success') this.active.result.title = title; }
  warn(title: string, action: NoticeAction, required = false) {
    if (this.active && this.active.kind !== 'error' && (!this.active.result.required || required)) {
      this.active.kind = 'warning'; this.active.result = { title, action, required };
    }
  }
  fail(detail: string) {
    const action = this.details(detail);
    const titles: Record<string, string> = {
      performance: msg('Could not prepare the performance', '연주를 준비하지 못했습니다'),
      character: msg('Could not load the character', '캐릭터를 불러오지 못했습니다'),
      equipment: msg('Could not update the drums', '드럼을 변경하지 못했습니다'),
      layout: msg('Could not apply the layout', '배치를 적용하지 못했습니다'),
      audio: msg('Could not apply the audio', '오디오를 적용하지 못했습니다'),
      project: msg('Could not complete the project task', '프로젝트 작업을 완료하지 못했습니다'),
      review: msg('Could not complete the review', '검수를 완료하지 못했습니다'),
      export: msg('Could not create the video', '영상을 만들지 못했습니다'),
    };
    const result = { title: titles[this.active?.scope ?? ''] ?? msg('Could not complete the task', '작업을 완료하지 못했습니다'), detail: msg('Check the details and try again.', '상세 내용을 확인하고 다시 시도하세요.'), action };
    if (this.active) { this.active.kind = 'error'; this.active.result = result; }
    else this.store.post('application', 'error', result);
  }
  cancel() {
    const operation = this.active; if (!operation) return;
    this.store.finish(operation.id, 'info', { title: msg('Task cancelled', '작업을 취소했습니다') });
    this.active = undefined;
  }
}
