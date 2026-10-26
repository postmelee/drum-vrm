import init from './wasm/drum_engine';

// Engine and equipment renderer share one main-thread instance. Workers own
// their isolated module and do not import this initializer.
let ready: Promise<unknown> | null = null;
export function initializeWasm(): Promise<unknown> {
  ready ??= init().catch(error => { ready = null; throw error; });
  return ready;
}
