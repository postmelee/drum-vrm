import { inspect_midi, Performance } from './wasm/drum_engine';
import { applyKitLayout } from './kit-calibration';
import type { EngineRequest, PreparedResponse } from './engine-protocol';
import type { PerformancePlan } from './types';

/** Called only after WASM initialization, in the planning worker. The transfer
 * remains opaque until Rust validates it in the rendering thread. */
export function executeEngineRequest(request:EngineRequest):string {
  if(request.kind==='inspect')return inspect_midi(request.bytes);
  if(request.kind==='source'){
    const performance=new Performance(request.bytes,JSON.stringify(request.selection));
    try{return performance.plan_json();}finally{performance.free();}
  }
  let plan:PerformancePlan;
  if(request.kind==='plan'){
    const source=new Performance(request.bytes,JSON.stringify(request.selection));
    try{plan=JSON.parse(source.plan_json());}finally{source.free();}
    if(request.layout)plan=applyKitLayout(plan,request.layout);
  }else plan=request.plan;
  const performance=request.profile?Performance.from_json_with_profile(JSON.stringify(plan),JSON.stringify(request.profile)):Performance.from_json(JSON.stringify(plan));
  try{
    const response:PreparedResponse={plan:JSON.parse(performance.plan_json()),prepared:performance.prepared_json()};
    return JSON.stringify(response);
  }finally{performance.free();}
}
