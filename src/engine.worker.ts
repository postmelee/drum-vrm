import init, { inspect_midi, Performance } from './wasm/drum_engine';
import type { Selection } from './types';
self.onmessage = async (event: MessageEvent<{kind: 'inspect'|'plan'; bytes: Uint8Array; selection?: Selection}>) => {
  try {
    await init();
    const {kind, bytes, selection} = event.data;
    if (kind === 'inspect') self.postMessage({ok: true, json: inspect_midi(bytes)});
    else {
      const performance = new Performance(bytes, JSON.stringify(selection));
      try { self.postMessage({ok: true, json: performance.plan_json()}); }
      finally { performance.free(); }
    }
  } catch (error) { self.postMessage({ok: false, error: String(error)}); }
};
