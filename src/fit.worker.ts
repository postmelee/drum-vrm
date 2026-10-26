import init, { fit_kit } from './wasm/drum_engine';
import type { FitRequest, FitResponse } from './fit-client';

let ready: Promise<unknown> | null = null;
self.onmessage = async ({ data }: MessageEvent<FitRequest>) => {
  const { requestId, profileJson, kitJson } = data;
  let response: FitResponse;
  try {
    ready ??= init();
    try { await ready; } catch (error) { ready = null; throw error; }
    response = { requestId, ok: true, json: fit_kit(profileJson, kitJson) };
  } catch (error) { response = { requestId, ok: false, error: error instanceof Error ? error.message : String(error) }; }
  self.postMessage(response);
};
