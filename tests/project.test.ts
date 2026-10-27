import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { makeInterpretationDemo } from '../src/demo';
import { NativeHumanoidRigAdapter } from '../src/rig-humanoid';
import { cameraPresetState } from '../src/scene';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import { measureInstrumentGeometry } from '../src/instrument-body';
import { fileReference, missingProjectAssets, parseProject, type DrumProject } from '../src/project';
import { initSync, Performance } from '../src/wasm/drum_engine';
import type { PerformancePlan } from '../src/types';

let fixture:DrumProject,midiFile:File;
// Independently authored sparse four-minute study: 120 BPM, then 80 BPM.
function makeTempoStudy():Uint8Array {
  const vlq=(value:number)=>{const out=[value&127];while((value>>=7))out.unshift((value&127)|128);return out;};
  const chunk=(name:string,data:number[])=>[...name].map(c=>c.charCodeAt(0)).concat([data.length>>>24,data.length>>>16&255,data.length>>>8&255,data.length&255],data);
  const change=192*480,end=384*480;
  const conductor=[0,255,81,3,7,161,32,...vlq(change),255,81,3,11,113,176,...vlq(end-change),255,47,0];
  const notes:number[]=[];let previous=0;
  for(const beat of [0,191.5,192,192.5,383.5]) {
    const tick=beat*480;notes.push(...vlq(tick-previous),0x99,38,90,24,0x89,38,0);previous=tick+24;
  }
  notes.push(...vlq(end-previous),255,47,0);
  return new Uint8Array([...chunk('MThd',[0,1,0,2,1,224]),...chunk('MTrk',conductor),...chunk('MTrk',notes)]);
}
beforeAll(async()=>{
  initSync({module:readFileSync(new URL('../src/wasm/drum_engine_bg.wasm',import.meta.url))});
  const selection={tracks:[1],channel:0,mappingProfile:'logicDkdGm' as const,countInNote75:true,hihatPolicy:'shortOpenClose' as const,hihatOpenness:.2,omittedSourceEventIds:[],handOverrides:{}};
  const bytes=makeInterpretationDemo(),raw=new Performance(bytes,JSON.stringify(selection));
  const rig=new NativeHumanoidRigAdapter(),profile=rig.getProfile();
  const candidate=Performance.from_json_with_profile(raw.plan_json(),JSON.stringify(profile));
  const plan=JSON.parse(candidate.plan_json()) as PerformancePlan;
  midiFile=new File([Uint8Array.from(bytes)],'Authored study.mid');
  const camera=cameraPresetState('stage');
  fixture={format:'drum-vrm-project',schemaVersion:1,engineModuleSha256:'a'.repeat(64),plan,profile,selection,calibration:rig.getCalibration(),kit:structuredClone(plan.kit),rig:'native',assets:[await fileReference('midi',midiFile)],
    appearance:{preset:'studio',pieces:plan.kit.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).map(piece=>({id:piece.id,settings:{...resolvePieceVisual('studio',piece.id),finishOverride:false,source:'procedural'}}))},
    camera:{view:'stage',position:camera.position.toArray(),target:camera.target.toArray(),fov:camera.fov,near:camera.near},hitResponse:{enabled:false,strength:.4},
    audio:{source:'synthesized',offsetSeconds:.125,volume:.3},view:{theme:'dark',background:'#000000',skeleton:true,model:true,character:true,timelineZoom:4},outputTime:2.25};
  raw.free();candidate.free();rig.dispose();
});

describe('versioned local project',()=>{
  it('reconstructs the saved native measured profile from calibration and layout',()=>{
    const rig=new NativeHumanoidRigAdapter();
    try{expect(JSON.stringify(rig.setCalibration(fixture.calibration,fixture.kit))).toBe(JSON.stringify(fixture.profile));}
    finally{rig.dispose();}
  });
  it('round-trips settings, full source evidence and a validated plan with identical arbitrary-time poses',()=>{
    const restored=parseProject(JSON.stringify(fixture));
    expect(restored).toEqual(JSON.parse(JSON.stringify(fixture)));
    const before=Performance.from_json_with_profile(JSON.stringify(fixture.plan),JSON.stringify(fixture.profile));
    const after=Performance.from_json_with_profile(JSON.stringify(restored.plan),JSON.stringify(restored.profile));
    try {
      const times=[-.5,0,1,2.3456,4,6.1234,7.999];
      const poses=times.map(time=>before.evaluate_rig_at(time));
      for(const index of [6,2,0,5,1,4,3])expect(after.evaluate_rig_at(times[index])).toEqual(poses[index]);
      expect(JSON.parse(after.plan_json()).sourceEvents).toEqual(fixture.plan.sourceEvents);
    }finally{before.free();after.free();}
  });
  it('preserves measured collision descriptors exactly and still accepts projects without them',()=>{
    const project=structuredClone(fixture),geometry=project.profile.bodyGeometry!;
    geometry.source='mesh-estimate';geometry.sampleCount=320;
    geometry.torso.halfExtents[2]=.12;
    geometry.handCapsules=[{start:[0,0,.02],end:[0,0,.09],radius:.025},{start:[0,0,.03],end:[0,0,.08],radius:.03}];
    expect(parseProject(JSON.stringify(project)).profile.bodyGeometry).toEqual(geometry);
    delete project.profile.bodyGeometry;
    expect(parseProject(JSON.stringify(project)).profile.bodyGeometry).toBeUndefined();
  });
  it('preserves curved contact geometry and explicit tilt while old appearances remain flat',()=>{
    const project=structuredClone(fixture),hat=project.kit.find(piece=>piece.id==='hihat')!;
    const settings=resolvePieceVisual('studio',hat.id,{tilt:6});
    const body=createPieceModel(hat,settings,'studio',project.kit,false);
    const geometry=measureInstrumentGeometry(body.root,hat,'procedural');disposeModel(body.root);
    for(const kit of [project.kit,project.plan.kit]) for(const piece of kit) if(piece.id==='hihat'||piece.id==='openHat') piece.geometry=structuredClone(geometry);
    project.appearance.pieces.find(piece=>piece.id==='hihat')!.settings.tilt=6;
    const restored=parseProject(JSON.stringify(project));
    expect(restored.kit.find(piece=>piece.id==='hihat')!.geometry!.surface.radialProfile).toEqual(geometry.surface.radialProfile);
    expect(restored.appearance.pieces.find(piece=>piece.id==='hihat')!.settings.tilt).toBe(6);
    expect(restored.kit.find(piece=>piece.id==='hihat')!.geometry!.surface.motion).toEqual({kind:'translation',axis:[0,1,0],travelMeters:.025});
    const legacy=JSON.parse(JSON.stringify(project));delete legacy.appearance.pieces[0].settings.tilt;
    expect(parseProject(JSON.stringify(legacy)).appearance.pieces[0].settings.tilt).toBe(0);
    for(const kit of [project.kit,project.plan.kit]) kit.find(piece=>piece.id==='hihat')!.geometry!.surface.radialProfile!.points[1][0]=-1;
    expect(()=>parseProject(JSON.stringify(project))).toThrow();
  });
  it('requires same-content relinking; renamed same files match and same-name edits do not',async()=>{
    const renamed=new File([await midiFile.arrayBuffer()],'renamed.mid'),edited=new File(['changed'],midiFile.name);
    const good=await fileReference('midi',renamed),bad=await fileReference('midi',edited);
    expect(good.sha256).toBe(fixture.assets[0].sha256);
    expect(missingProjectAssets(fixture,new Map([[bad.sha256,edited]]))).toHaveLength(1);
    expect(missingProjectAssets(fixture,new Map([[good.sha256,renamed]]))).toEqual([]);
  });
  it('preserves a long-song tempo change, offset and random-seek pose after reopening',async()=>{
    const project=structuredClone(fixture),bytes=makeTempoStudy();
    project.selection={tracks:[1],channel:9,mappingProfile:'gm',countInNote75:false,hihatPolicy:'literal',hihatOpenness:.2};
    const raw=new Performance(bytes,JSON.stringify(project.selection));
    const before=Performance.from_json_with_profile(raw.plan_json(),JSON.stringify(project.profile));
    project.plan=JSON.parse(before.plan_json());project.kit=structuredClone(project.plan.kit);
    project.assets=[await fileReference('midi',new File([Uint8Array.from(bytes)],'Tempo Study.mid'))];
    project.outputTime=180.125;project.audio.offsetSeconds=-.25;
    const restored=parseProject(JSON.stringify(project));
    const after=Performance.from_json_with_profile(JSON.stringify(restored.plan),JSON.stringify(restored.profile));
    try {
      expect(restored.plan.duration).toBeCloseTo(240,5);
      expect(restored.plan.tempoMap).toHaveLength(2);
      expect(restored.audio.offsetSeconds).toBe(-.25);
      expect(restored.outputTime).toBe(180.125);
      for(const time of [239.625,96,0,95.75,96.375,180.125])expect(after.evaluate_rig_at(time)).toEqual(before.evaluate_rig_at(time));
      expect(JSON.parse(after.plan_json()).tempoMap).toEqual(project.plan.tempoMap);
    }finally{raw.free();before.free();after.free();}
  },150_000); // Two independent full-score plans: measured 72s on CI; browser work runs in a Worker.
  it('reopens explicit omission and hand directives without editing source events',()=>{
    const project=structuredClone(fixture),omitted=project.plan.hits.find(hit=>hit.instrument==='snare')!.sourceEventId!;
    const forced=project.plan.hits.find(hit=>hit.instrument==='crash')!.sourceEventId!;
    project.selection={...project.selection,omittedSourceEventIds:[omitted],handOverrides:{[forced]:1}};
    const raw=new Performance(makeInterpretationDemo(),JSON.stringify(project.selection));
    const candidate=Performance.from_json_with_profile(raw.plan_json(),JSON.stringify(project.profile));
    project.plan=JSON.parse(candidate.plan_json());project.kit=structuredClone(project.plan.kit);
    const reopened=parseProject(JSON.stringify(project));
    try {
      expect(reopened.plan.sourceEvents).toEqual(fixture.plan.sourceEvents);
      expect(reopened.plan.interpretations?.find(event=>event.sourceEventId===omitted)?.kind).toBe('omitted');
      expect(reopened.plan.hits.some(hit=>hit.sourceEventId===omitted)).toBe(false);
      expect(reopened.selection.handOverrides).toEqual({[forced]:1});
      expect(reopened.plan.handOverrides).toEqual({[forced]:1});
    }finally{raw.free();candidate.free();}
  });
  it('accepts the engine-valid zero-duration score with a tick-zero strike',async()=>{
    const bytes=new Uint8Array([77,84,104,100,0,0,0,6,0,0,0,1,1,224,77,84,114,107,0,0,0,8,0,0x99,38,100,0,255,47,0]);
    const project=structuredClone(fixture);project.selection={tracks:[0],channel:9,mappingProfile:'gm',countInNote75:false,hihatPolicy:'literal',hihatOpenness:.2,omittedSourceEventIds:[],handOverrides:{}};
    const raw=new Performance(bytes,JSON.stringify(project.selection)),candidate=Performance.from_json_with_profile(raw.plan_json(),JSON.stringify(project.profile));
    try {
      project.plan=JSON.parse(candidate.plan_json());project.kit=structuredClone(project.plan.kit);
      project.assets=[await fileReference('midi',new File([bytes],'Tick Zero.mid'))];project.outputTime=0;
      expect(parseProject(JSON.stringify(project)).plan.duration).toBe(0);
    }finally{raw.free();candidate.free();}
  });
  it('rejects inconsistent intent, layout, asset links and unknown versions before UI changes',()=>{
    const rejected=(change:(project:DrumProject)=>void)=>{const project=structuredClone(fixture);change(project);expect(()=>parseProject(JSON.stringify(project))).toThrow();};
    rejected(project=>{project.schemaVersion=99 as 1;});
    rejected(project=>{project.selection.hihatPolicy='literal';});
    rejected(project=>{project.kit[1].position[0]+=.1;});
    rejected(project=>{project.assets[0].sha256='not-a-hash';});
    rejected(project=>{project.audio.source='wav';});
    rejected(project=>{project.rig='vrm';});
    rejected(project=>{project.appearance.pieces[0].settings.source='imported';});
    rejected(project=>{project.selection.omittedSourceEventIds=['1:1'];});
    rejected(project=>{project.selection.handOverrides={'1:1':1};});
    rejected(project=>{project.profile.bodyGeometry!.armRadii[0][0]=-1;});
    rejected(project=>{project.profile.bodyGeometry!.torso.center[0]=NaN;});
  });
  it('rejects unsafe presentation values and does not fetch asset references',()=>{
    const rejected=(change:(project:DrumProject)=>void)=>{const project=structuredClone(fixture);change(project);expect(()=>parseProject(JSON.stringify(project))).toThrow();};
    rejected(project=>{project.camera.position=[NaN,0,0];});
    rejected(project=>{project.camera.position=[...project.camera.target];});
    rejected(project=>{project.audio.offsetSeconds=301;});
    rejected(project=>{project.view.background='url(https://example.invalid)';});
    rejected(project=>{project.selection.tracks=[.5];});
    rejected(project=>{project.appearance.pieces.push(project.appearance.pieces[0]);});
  });
});
