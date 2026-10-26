import { describe, expect, it, vi } from 'vitest';
import { FitClient, type FitRequest, type FitResponse } from '../src/fit-client';
import type { KitFitProposal, KitPiece, RigProfile } from '../src/types';

class TestWorker {
  onmessage: ((event: MessageEvent<FitResponse>) => void) | null = null;
  onerror: (() => void) | null = null;
  onmessageerror: (() => void) | null = null;
  postMessage = vi.fn<(request: FitRequest) => void>();
  terminate = vi.fn();
  respond(requestId: number, proposal: object) { this.onmessage?.({ data: { requestId, ok: true, json: JSON.stringify(proposal) } } as MessageEvent<FitResponse>); }
  request() { return this.postMessage.mock.calls.at(-1)![0]; }
}
const profile = { id: 'test-body', lengths: [[.3,.25],[.3,.25],[.4,.4],[.4,.4]] } as RigProfile;
const kit = [{ id: 'snare', label: 'Snare', position: [0,.8,.5], radius: .15 }] as KitPiece[];
const result = { complete: true, kit, scale: 1, translation: [0,0,0], pieceAdjustments: [], unreachablePieceIds: [] } as unknown as KitFitProposal;
function fixture() {
  const workers: TestWorker[] = [];
  const factory = vi.fn(() => { const worker = new TestWorker(); workers.push(worker); return worker as unknown as Worker; });
  return { client: new FitClient(factory), workers, factory };
}

describe('background fitting lifecycle', () => {
  it('captures inputs and reuses the initialized worker for subsequent fits', async () => {
    const { client, workers, factory } = fixture(), body = structuredClone(profile), layout = structuredClone(kit);
    const first = client.fit(body,layout), worker = workers[0], request = worker.request();
    body.id = 'edited'; layout[0].position[0] = 12;
    expect(JSON.parse(request.profileJson).id).toBe('test-body');
    expect(JSON.parse(request.kitJson)[0].position[0]).toBe(0);
    worker.respond(request.requestId,result); expect(await first).toEqual(result);
    const next = client.fit(profile,kit), nextId = worker.request().requestId;
    expect(nextId).toBeGreaterThan(request.requestId);
    worker.respond(nextId,result); await next;
    expect(factory).toHaveBeenCalledTimes(1); expect(worker.terminate).not.toHaveBeenCalled(); client.dispose();
  });

  it('terminates a superseded search and ignores its late success and failure', async () => {
    const { client, workers } = fixture();
    const first = client.fit(profile,kit), rejected = expect(first).rejects.toMatchObject({ name:'AbortError' });
    const oldWorker = workers[0], oldId = oldWorker.request().requestId;
    const next = client.fit(profile,kit), current = workers[1];
    await rejected; expect(oldWorker.terminate).toHaveBeenCalledOnce();
    const completed = vi.fn(); next.then(completed);
    oldWorker.respond(oldId,result); oldWorker.onerror?.(); await Promise.resolve();
    expect(completed).not.toHaveBeenCalled(); expect(current.terminate).not.toHaveBeenCalled();
    current.respond(current.request().requestId,result); await expect(next).resolves.toEqual(result); client.dispose();
  });

  it('cancels the actual search and starts a fresh worker on retry', async () => {
    const { client, workers } = fixture();
    const pending = client.fit(profile,kit), rejected = expect(pending).rejects.toMatchObject({ name:'AbortError' });
    client.cancel(); await rejected; expect(workers[0].terminate).toHaveBeenCalledOnce();
    const retry = client.fit(profile,kit), worker = workers[1]; worker.respond(worker.request().requestId,result);
    await expect(retry).resolves.toEqual(result); client.dispose();
    await expect(client.fit(profile,kit)).rejects.toThrow('disposed');
  });

  it.each(['onerror','onmessageerror'] as const)('recovers after worker %s without retaining a pending request', async event => {
    const { client, workers } = fixture();
    const pending = client.fit(profile,kit), rejected = expect(pending).rejects.toThrow();
    workers[0][event]?.(); await rejected; expect(workers[0].terminate).toHaveBeenCalledOnce();
    const retry = client.fit(profile,kit), worker = workers[1]; worker.respond(worker.request().requestId,result);
    await expect(retry).resolves.toEqual(result); client.dispose();
  });

  it('does not apply another request response on a reused worker', async () => {
    const { client, workers } = fixture(), first = client.fit(profile,kit), worker = workers[0];
    const firstId = worker.request().requestId; worker.respond(firstId,result); await first;
    const next = client.fit(profile,kit), completed = vi.fn(); next.then(completed);
    worker.respond(firstId,{ complete:false }); await Promise.resolve(); expect(completed).not.toHaveBeenCalled();
    worker.respond(worker.request().requestId,result); await expect(next).resolves.toEqual(result); client.dispose();
  });

  it('settles fit errors and malformed results without leaving a hanging search', async () => {
    const { client, workers } = fixture(), first = client.fit(profile,kit), worker = workers[0];
    worker.onmessage?.({ data: { requestId:worker.request().requestId,ok:false,error:'Invalid body' } } as MessageEvent<FitResponse>);
    await expect(first).rejects.toThrow('Invalid body');
    const next = client.fit(profile,kit);
    worker.onmessage?.({ data: { requestId:worker.request().requestId,ok:true,json:'broken' } } as MessageEvent<FitResponse>);
    await expect(next).rejects.toThrow(); client.dispose();
  });

  it('recovers from a failed worker startup and a synchronous posting failure', async () => {
    const worker = new TestWorker(), factory = vi.fn<() => Worker>().mockImplementationOnce(() => { throw new Error('startup'); }).mockReturnValue(worker as unknown as Worker);
    const client = new FitClient(factory);
    await expect(client.fit(profile,kit)).rejects.toThrow('startup');
    worker.postMessage.mockImplementationOnce(() => { throw new Error('posting'); });
    await expect(client.fit(profile,kit)).rejects.toThrow('posting'); expect(worker.terminate).toHaveBeenCalledOnce();
    const retry = client.fit(profile,kit); worker.respond(worker.request().requestId,result); await retry; client.dispose();
  });
});
