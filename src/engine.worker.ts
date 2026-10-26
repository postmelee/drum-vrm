import init from './wasm/drum_engine';
import type { EngineRequest } from './engine-protocol';
import { executeEngineRequest } from './engine-worker-actions';
self.onmessage = async (event: MessageEvent<EngineRequest & {requestId:number}>) => {
  const request=event.data;
  try {
    await init();
    const json=executeEngineRequest(request);
    self.postMessage({requestId:request.requestId,ok:true,json});
  } catch (error) { self.postMessage({requestId:request.requestId,ok:false,error:String(error)}); }
};
