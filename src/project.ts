import { validateFBXConfig, imageKey, type FBXConfig } from './fbx-config';
import { msg } from './i18n';
import { validateKitLayout } from './kit-calibration';
import { validateHitResponse, type HitResponseSettings } from './hit-response';
import { validateBodyGeometry } from './body-geometry';
import { resolvePieceVisual, type KitPresetId, type PieceVisualSettings } from './model-procedural';
import type { HumanoidCalibration } from './rig-humanoid';
import type { SceneViewState } from './scene';
import type { KitPiece, PerformancePlan, PieceId, RigProfile, Selection } from './types';

export const PROJECT_SCHEMA_VERSION = 1;
export const MAX_PROJECT_BYTES = 32 * 1024 * 1024;
export interface ProjectAsset { kind:'midi'|'wav'|'vrm'|'fbx'|'fbxTexture'|'drumPart'; name:string; sha256:string; bytes:number; pieceId?:PieceId; }
export interface DrumProject {
  format:'drum-vrm-project';schemaVersion:1;
  engineModuleSha256:string;
  plan:PerformancePlan;profile:RigProfile;selection:Selection;
  calibration:HumanoidCalibration;kit:KitPiece[];
  rig:'native'|'vrm'|'fbx';fbx?:FBXConfig;assets:ProjectAsset[];
  appearance:{preset:KitPresetId;pieces:{id:PieceId;settings:PieceVisualSettings}[]};
  camera:SceneViewState;hitResponse:HitResponseSettings;
  audio:{source:'synthesized'|'wav';offsetSeconds:number;volume:number};
  view:{theme:'light'|'dark';background:string;skeleton:boolean;model:boolean;character:boolean;timelineZoom:number};
  outputTime:number;
}
const record=(value:unknown):Record<string,unknown>=>{
  if(!value||typeof value!=='object'||Array.isArray(value))throw new Error(msg("Invalid project object format.", "프로젝트 객체 형식이 잘못되었습니다."));
  return value as Record<string,unknown>;
};
const number=(value:unknown,min:number,max:number)=>{
  if(typeof value!=='number'||!Number.isFinite(value)||value<min||value>max)throw new Error(msg("A project value is outside the supported range.", "프로젝트 수치가 지원 범위를 벗어납니다."));
  return value;
};
const choice=<const T extends string>(value:unknown,values:readonly T[]):T=>{
  if(typeof value!=='string'||!values.includes(value as T))throw new Error(msg("Unsupported project setting.", "프로젝트 설정 값이 지원되지 않습니다."));
  return value as T;
};
const bool=(value:unknown)=>{if(typeof value!=='boolean')throw new Error(msg("Invalid project selection value.", "프로젝트 선택 값이 잘못되었습니다."));return value;};
const text=(value:unknown)=>{if(typeof value!=='string'||value.length>512||value.includes('\0'))throw new Error(msg("Invalid project string.", "프로젝트 문자열이 잘못되었습니다."));return value;};
const color=(value:unknown)=>{const result=text(value);if(!/^#[0-9a-f]{6}$/i.test(result))throw new Error(msg("Invalid project color.", "프로젝트 색상이 잘못되었습니다."));return result;};

export function validateProjectView(value:unknown):SceneViewState {
  const camera=record(value),vector=(v:unknown):[number,number,number]=>{
    if(!Array.isArray(v)||v.length!==3)throw new Error(msg("Invalid camera vector.", "카메라 벡터가 잘못되었습니다."));
    return v.map(n=>number(n,-100,100)) as [number,number,number];
  };
  const position=vector(camera.position),target=vector(camera.target);
  if(Math.hypot(...position.map((v,i)=>v-target[i]))<.01)throw new Error(msg("The camera position is too close to its target.", "카메라 위치와 목표가 너무 가깝습니다."));
  return {view:choice(camera.view,['stage','front','top','player']),position,target,
    fov:number(camera.fov,15,100),near:number(camera.near,.005,.5)};
}

/** The browser validates bounded presentation fields; Rust validates the full
 * source/interpretation/plan contract before an evaluator is committed. */
export function parseProject(json:string):DrumProject {
  if(new TextEncoder().encode(json).length>MAX_PROJECT_BYTES)throw new Error(msg("The project must be no larger than 32 MiB.", "프로젝트는 32 MiB 이하여야 합니다."));
  const input=record(JSON.parse(json));
  if(input.format!=='drum-vrm-project'||input.schemaVersion!==PROJECT_SCHEMA_VERSION)throw new Error(msg("Unsupported project format or version.", "지원하지 않는 프로젝트 형식 또는 버전입니다."));
  const profile=record(input.profile);
  if(profile.bodyGeometry!==undefined)validateBodyGeometry(profile.bodyGeometry);
  const plan=record(input.plan) as unknown as PerformancePlan;
  if(plan.schemaVersion!==2||!Array.isArray(plan.hits)||!Array.isArray(plan.sourceEvents)||plan.hits.length>50000||plan.sourceEvents.length>200000)throw new Error(msg("The project performance plan is invalid or too large.", "프로젝트 연주 계획이 잘못되었거나 너무 큽니다."));
  number(plan.duration,0,300);
  const kit=validateKitLayout(input.kit as KitPiece[]);
  if(JSON.stringify(kit)!==JSON.stringify(validateKitLayout(plan.kit)))throw new Error(msg("The saved layout differs from the performance plan.", "저장한 배치와 계획의 배치가 다릅니다."));
  const calibration=record(input.calibration),selection=record(input.selection),appearance=record(input.appearance),audio=record(input.audio),view=record(input.view);
  if(!Array.isArray(selection.tracks)||!selection.tracks.length||selection.tracks.length>512)throw new Error(msg("Invalid project track selection.", "프로젝트 트랙 선택이 잘못되었습니다."));
  const tracks=selection.tracks.map(track=>{const n=number(track,0,511);if(!Number.isInteger(n))throw new Error(msg("Track numbers must be integers.", "트랙 번호가 정수가 아닙니다."));return n;});
  const mappingProfile=choice(selection.mappingProfile,['gm','logicDkdGm']),hihatPolicy=choice(selection.hihatPolicy,['literal','shortOpenClose']);
  const countInNote75=bool(selection.countInNote75),hihatOpenness=number(selection.hihatOpenness,.05,1);
  const omittedInput=selection.omittedSourceEventIds??[],handInput=record(selection.handOverrides??{});
  if(!Array.isArray(omittedInput)||omittedInput.length>50000||Object.keys(handInput).length>50000)throw new Error(msg("The project contains too many note edits.", "프로젝트 음표 편집이 너무 큽니다."));
  const omittedSourceEventIds=omittedInput.map(text);
  if(new Set(omittedSourceEventIds).size!==omittedSourceEventIds.length)throw new Error(msg("The project contains duplicate note omissions.", "프로젝트 음표 제외가 중복됩니다."));
  const handOverrides:Record<string,0|1>={};
  for(const [id,value] of Object.entries(handInput)){text(id);if(value!==0&&value!==1)throw new Error(msg("Invalid playing-hand assignment.", "연주 손 지정이 잘못되었습니다."));handOverrides[id]=value;}
  const sortedHands=(value:Record<string,0|1>)=>Object.entries(value).sort(([a],[b])=>a.localeCompare(b));
  if(JSON.stringify([...omittedSourceEventIds].sort())!==JSON.stringify([...(plan.omittedSourceEventIds??[])].sort())||JSON.stringify(sortedHands(handOverrides))!==JSON.stringify(sortedHands(plan.handOverrides??{})))throw new Error(msg("The saved note edits differ from the performance plan.", "저장한 음표 편집과 계획이 다릅니다."));
  if(mappingProfile!==plan.mappingProfile||hihatPolicy!==plan.hihatPolicy||countInNote75!==plan.countInNote75||hihatOpenness!==plan.hihatOpenness)throw new Error(msg("The saved interpretation settings differ from the performance plan.", "저장한 해석 설정과 계획이 다릅니다."));
  const channel=selection.channel===null?null:number(selection.channel,0,15);
  if(channel!==null&&!Number.isInteger(channel))throw new Error(msg("Channel numbers must be integers.", "채널 번호가 정수가 아닙니다."));
  const rig=choice(input.rig,['native','vrm','fbx']);
  if(!Array.isArray(input.assets)||input.assets.length>77)throw new Error(msg("Invalid project asset references.", "프로젝트 파일 참조가 잘못되었습니다."));
  const assetIds=new Set<string>();
  const assets=input.assets.map(value=>{
    const a=record(value),kind=choice(a.kind,['midi','wav','vrm','fbx','fbxTexture','drumPart']),name=text(a.name),sha256=text(a.sha256),bytes=number(a.bytes,1,100*1024*1024);
    if(!/^[0-9a-f]{64}$/.test(sha256)||!Number.isInteger(bytes))throw new Error(msg("Invalid project asset hash or size.", "프로젝트 입력 해시·크기가 잘못되었습니다."));
    const pieceId=kind==='drumPart'?choice(a.pieceId,kit.map(piece=>piece.id)):undefined;
    const id=`${kind}:${kind==='fbxTexture'?imageKey(name):pieceId??''}`;if(assetIds.has(id))throw new Error(msg("The project contains duplicate asset references.", "프로젝트 파일 참조가 중복됩니다."));assetIds.add(id);
    return {kind,name,sha256,bytes,...(pieceId?{pieceId}:{})};
  });
  if(!assetIds.has('midi:')||(rig==='vrm')!==assetIds.has('vrm:'))throw new Error(msg("A MIDI or VRM asset reference is missing or inconsistent.", "MIDI 또는 VRM 파일 참조가 누락되었거나 일치하지 않습니다."));
  const fbx=rig==='fbx'?validateFBXConfig(input.fbx):undefined;
  if((rig==='fbx')!==assetIds.has('fbx:')||(rig!=='fbx'&&(input.fbx!==undefined||assets.some(a=>a.kind==='fbxTexture'))))throw new Error(msg('FBX references or settings are inconsistent.', 'FBX 참조 또는 설정이 일치하지 않습니다.'));
  const images=assets.filter(a=>a.kind==='fbxTexture');
  if(images.length>64||images.reduce((n,a)=>n+a.bytes,0)>50*1024*1024||images.some(a=>! /\.(png|jpe?g)$/i.test(a.name)))throw new Error(msg('Invalid FBX texture references.', 'FBX 텍스처 참조가 올바르지 않습니다.'));
  if(fbx&&Object.values(fbx.textures).some(name=>!images.some(a=>imageKey(a.name)===name)))throw new Error(msg('An assigned FBX texture reference is missing.', '지정한 FBX 텍스처 참조가 없습니다.'));
  const audioSource=choice(audio.source,['synthesized','wav']);
  if((audioSource==='wav')!==assetIds.has('wav:'))throw new Error(msg("A WAV asset reference is missing or inconsistent.", "WAV 파일 참조가 누락되었거나 일치하지 않습니다."));
  const preset=choice(appearance.preset,['studio','vintage','electronic','library','batoski','smoj']);
  if(!Array.isArray(appearance.pieces)||appearance.pieces.length>9)throw new Error(msg("Invalid drum appearance list.", "드럼 외형 목록이 잘못되었습니다."));
  const ids=new Set<PieceId>();
  const pieces=appearance.pieces.map(value=>{
    const p=record(value),id=choice(p.id,kit.map(piece=>piece.id));
    if(id==='openHat'||id==='pedalHat'||ids.has(id))throw new Error(msg("The drum appearance list contains duplicates.", "드럼 외형이 중복됩니다."));ids.add(id);
    const settings=record(p.settings),source=choice(settings.source,['procedural','library','imported']);
    if((source==='imported')!==assetIds.has(`drumPart:${id}`))throw new Error(msg("A drum model asset reference is missing or inconsistent.", "드럼 모델 파일 참조가 누락되었거나 일치하지 않습니다."));
    return {id,settings:{...resolvePieceVisual(preset,id,{finish:color(settings.finish),model:choice(settings.model,['standard','deep','compact']),scale:number(settings.scale,.65,1.4),rotation:number(settings.rotation,-360,360)}),finishOverride:bool(settings.finishOverride??false),source}};
  });
  if(pieces.length!==kit.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).length)throw new Error(msg("Drum appearance settings are missing.", "드럼 외형 설정이 누락되었습니다."));
  const engineModuleSha256=text(input.engineModuleSha256);if(!/^[0-9a-f]{64}$/.test(engineModuleSha256))throw new Error(msg("Invalid engine identifier.", "엔진 식별자가 잘못되었습니다."));
  return {format:'drum-vrm-project',schemaVersion:1,engineModuleSha256,plan,profile:record(input.profile) as unknown as RigProfile,kit,rig,assets,...(fbx?{fbx}:{}),
    calibration:{seatHeight:number(calibration.seatHeight,.3,1.2),seatForward:number(calibration.seatForward,-.5,.5),palmOrientation:choice(calibration.palmOrientation,['up','inward','down'])},
    selection:{tracks,channel,mappingProfile,countInNote75,hihatPolicy,hihatOpenness,omittedSourceEventIds,handOverrides},appearance:{preset,pieces},camera:validateProjectView(input.camera),
    hitResponse:validateHitResponse(record(input.hitResponse) as unknown as HitResponseSettings),
    audio:{source:audioSource,offsetSeconds:number(audio.offsetSeconds,-300,300),volume:number(audio.volume,0,1)},
    view:{theme:choice(view.theme,['light','dark']),background:color(view.background),skeleton:bool(view.skeleton),model:bool(view.model),character:bool(view.character),timelineZoom:number(view.timelineZoom,1,16)},
    outputTime:number(input.outputTime,0,301)};
}

export async function fileReference(kind:ProjectAsset['kind'],file:File,pieceId?:PieceId):Promise<ProjectAsset> {
  const hash=await crypto.subtle.digest('SHA-256',await file.arrayBuffer());
  return {kind,name:file.name,bytes:file.size,sha256:[...new Uint8Array(hash)].map(value=>value.toString(16).padStart(2,'0')).join(''),...(pieceId?{pieceId}:{})};
}
export function missingProjectAssets(project:DrumProject,files:ReadonlyMap<string,File>):ProjectAsset[] {
  return project.assets.filter(asset=>!files.has(asset.sha256)||files.get(asset.sha256)!.size!==asset.bytes);
}
let moduleHash:Promise<string>|undefined;
export function projectEngineHash():Promise<string> {
  return moduleHash??=(async()=>{
    const response=await fetch(new URL('./wasm/drum_engine_bg.wasm',import.meta.url));
    if(!response.ok)throw new Error(msg("The project engine identifier could not be read.", "프로젝트 엔진 식별자를 읽지 못했습니다."));
    const hash=await crypto.subtle.digest('SHA-256',await response.arrayBuffer());
    return [...new Uint8Array(hash)].map(value=>value.toString(16).padStart(2,'0')).join('');
  })().catch(error=>{moduleHash=undefined;throw error;});
}
