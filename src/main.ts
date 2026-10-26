import './style.css';
import { studioTemplate } from './studio-template';
import { prepareAutomaticFit } from './automatic-fit';
import { DrumTimeline, createTimelineNotes } from './timeline';
import { arrangementOf, nearestNoteGroup, noteSources, withArrangementEdit, type Arrangement } from './arrangement';
import { DrumScene } from './scene';
import { setButtonIcon } from './icons';
import { NativeHumanoidRigAdapter, type HumanoidCalibration } from './rig-humanoid';
import { moveKitTarget } from './kit-calibration';
import { VRMHumanoidRigAdapter } from './rig-vrm';
import { loadLocalVRM, type VRMInspection } from './vrm-import';
import { VRMUtils } from '@pixiv/three-vrm';
import type { KitPresetId } from './model-procedural';
import { EngineClient } from './engine';
import { preparePerformance } from './preparation';
import { DrumAudio } from './audio';
import { importGuidance } from './import-guidance';
import { fileReference, missingProjectAssets, parseProject, projectEngineHash, MAX_PROJECT_BYTES, type DrumProject, type ProjectAsset } from './project';
import { createReviewSnapshot, reviewOrder, reviewTimes, summarizePoseReview, summarizeProxyReview, type PoseReview } from './performance-review';
import { loadLocalWav, alignWavToPlan, type LocalWav, type WavAlignmentInfo } from './wav';
import { SynthesisClient } from './synthesis';
import { probeExport, exportVideo } from './export';
import { makeDemo, makeInterpretationDemo } from './demo';
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo, type PieceId, type KitPiece, type KitFitProposal, type PoseInspection, type RigProfile, type Vec3 } from './types';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = studioTemplate;

const get = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
setButtonIcon(get('reset'), 'reset');
setButtonIcon(get('play'), 'play');
setButtonIcon(get('close-panel'), 'close');
for (const id of ['upload', 'start-upload']) setButtonIcon(get(id), 'upload', get(id).textContent ?? 'MIDI 불러오기');
setButtonIcon(get('export'), 'download', '영상 추출');
const stage = get<HTMLCanvasElement>('stage');
const engine = new EngineClient();
const synthesis = new SynthesisClient();
let scene: DrumScene;
try { scene = new DrumScene(stage); } catch { get('message').textContent = '3D 화면을 시작할 수 없습니다. WebGL을 지원하는 최신 브라우저를 사용해 주세요.'; throw new Error('WebGL unavailable'); }
const defaultKit = scene.getKitLayout();
void engine.setSetup(scene.getRigProfile(), defaultKit).catch(error=>message(String(error),true));
let plan: PerformancePlan | null = null;
let bytes: Uint8Array | null = null;
let audio: DrumAudio | null = null;
let buffer: AudioBuffer | null = null;
let wav: LocalWav | null = null;
let wavOffset = 0;
let wavAlignmentInfo: WavAlignmentInfo | undefined;
let fitProposal: KitFitProposal | null = null;
// Explicit manual/restored layout; automatic fits never feed their output back here.
let fitBaseline = structuredClone(defaultKit);
let preFitSetup: { seat: HumanoidCalibration; kit: KitPiece[] } | null = null;
let fitState = '기본 배치 · 체형별 배치 보정 전';
let poseReview: PoseReview | null = null;
let reviewRunning = false;
let reviewGeneration = 0;
let projectBusy = false;
const projectFiles = new Map<string,File>();
let sourceReferences: ProjectAsset[] = [];
let pendingProject: DrumProject | null = null;
let preparedSelection: DrumProject['selection'] | null = null;
let arrangement:Arrangement={omittedSourceEventIds:[],handOverrides:{}};
let busy = false;
let exporting = false;
let loadGeneration = 0;
let exportController: AbortController | null = null;
let downloadUrl: string | null = null;
let fileName = 'drum-performance';
async function rememberProjectFile(kind:ProjectAsset['kind'],file:File,pieceId?:PieceId) {
  const reference=await fileReference(kind,file,pieceId);commitProjectFile(reference,file);return reference;
}
function commitProjectFile(reference:ProjectAsset,file:File) {
  const {kind,pieceId}=reference;projectFiles.set(reference.sha256,file);
  sourceReferences=sourceReferences.filter(old=>old.kind!==kind||(kind==='drumPart'&&old.pieceId!==pieceId));sourceReferences.push(reference);return reference;
}
type Panel = 'character' | 'drums' | 'source' | 'analysis' | 'export';
let activePanel: Panel | null = null;
let modelBusy = false;
let fitRunning = false;
let setupRunning = false;
let idlePoseGeneration = 0;
let committedPreset = 'studio';
let theme: 'light' | 'dark' = 'light';
try { theme = localStorage.getItem('drum-theme') === 'dark' ? 'dark' : 'light'; } catch { /* Storage is optional. */ }
const timeline = new DrumTimeline(get('timeline'), { onSeek: time => {
  if (!plan || busy || exporting || modelBusy) return;
  audio?.seek(time); updateFrame(time);
},onSourceInspect:id=>{
  if(!id){get<HTMLSelectElement>('arrangement-event').replaceChildren(new Option('선택 없음',''));get('arrangement-detail').textContent='';get<HTMLInputElement>('arrangement-omit').checked=false;get<HTMLSelectElement>('arrangement-hand').value='auto';}
  else if(plan&&!busy&&!exporting&&!modelBusy){selectArrangementEvent(id);openPanel('analysis');get<HTMLDetailsElement>('arrangement-editor').open=true;}
}});
const panelTitles: Record<Panel, string> = {character:'캐릭터 설정',drums:'드럼 설정',source:'파일 · 오디오',analysis:'분석',export:'내보내기'};
function openPanel(next: Panel | null) {
  if (exporting || modelBusy) return;
  activePanel = next;
  get('inspector').hidden = !next;
  get('panel-title').textContent = next ? panelTitles[next] : '';
  for (const button of document.querySelectorAll<HTMLButtonElement>('[data-panel]')) button.setAttribute('aria-expanded',String(button.dataset.panel===next));
  for (const key of Object.keys(panelTitles)) get(`panel-${key}`).hidden = key !== next;
  scene.setEditorMode(next === 'drums');
  if(next==='drums'){scene.selectPiece(selectedPiece());refreshPiece();}
  controls();
}
function setTheme(next: 'light' | 'dark') {
  theme = next; document.documentElement.dataset.theme = next;
  scene.setTheme(next); timeline.setTheme(next);
  get<HTMLInputElement>('background').value = next==='light'?'#ffffff':'#000000';
  setButtonIcon(get('theme'),next==='light'?'dark':'light');
  get('theme').setAttribute('aria-label',next==='light'?'다크 모드로 전환':'라이트 모드로 전환');
  try { localStorage.setItem('drum-theme',next); } catch { /* Storage is optional. */ }
}
function selectedPiece() { return get<HTMLSelectElement>('piece').value as PieceId; }
function refreshPiece() {
  const visual = scene.getPieceVisual(selectedPiece());
  get<HTMLSelectElement>('piece-model').value = visual.source==='library'||visual.source==='imported'?'asset':visual.model;
  get<HTMLInputElement>('piece-finish').value = visual.finish;
  get<HTMLInputElement>('piece-scale').value = String(visual.scale);
  get('piece-scale-value').textContent = `${Math.round(visual.scale*100)}%`;
  get<HTMLInputElement>('piece-rotation').value=String(visual.rotation);get('piece-rotation-value').textContent=`${visual.rotation}°`;
}
function refreshCalibration() {
  const seat = scene.getCalibration();
  get<HTMLInputElement>('seat-height').value = String(Math.round(seat.seatHeight*1000)/10);
  get<HTMLInputElement>('seat-forward').value = String(Math.round(seat.seatForward*1000)/10);
  get<HTMLSelectElement>('palm-orientation').value = seat.palmOrientation ?? 'up';
}
function refreshTarget() {
  const piece = scene.getKitLayout().find(p=>p.id===get<HTMLSelectElement>('target-piece').value)!;
  const pedal=piece.id==='kick'||piece.id==='pedalHat', height=get<HTMLInputElement>('target-y');
  height.min=pedal?'4':'2';height.max=pedal?'20':'250';
  for (const [i,axis] of ['x','y','z'].entries()) get<HTMLInputElement>(`target-${axis}`).value=String(Math.round(piece.position[i]*1000)/10);
}
async function refreshIdlePose() {
  const generation=++idlePoseGeneration;if(plan)return;
  try{
    const pose=await engine.readyPose(scene.getRigProfile(),scene.getKitLayout());
    if(generation===idlePoseGeneration&&!plan)scene.applyReadyPose(pose);
  }catch(error){
    if(generation===idlePoseGeneration&&!plan)get('calibration-message').textContent=`기본 자세 확인 필요 · ${error instanceof Error?error.message:String(error)}`;
  }
}
function invalidateVideo() {
  if(downloadUrl)URL.revokeObjectURL(downloadUrl); downloadUrl=null; get('download').hidden=true;
  get('export-note').textContent='1280 × 720 · 30 fps · 현재 설정으로 다시 추출하세요. 시작 준비 0.5초 · 끝 여운 0.25초 포함.';
}
async function applyCalibration(seat: HumanoidCalibration, kit: KitPiece[], feedbackId: string, automatic=false) {
  if(busy||exporting||(modelBusy&&!automatic))return false;
  const oldSeat=scene.getCalibration(), oldKit=scene.getKitLayout();
  let applied=false,sceneChanged=false;
  let staged:Awaited<ReturnType<EngineClient['prepareSetup']>>|null=null;
  const time=audio?.pause()??0;
  if(!automatic){modelBusy=true;controls();}
  try {
    const profile=scene.measureCalibration(seat,kit);
    staged=await prepareSetupInBackground(profile,kit);
    sceneChanged=true;scene.setKitLayout(kit);scene.setCalibration(seat);
    const next=staged.commit();
    if(next&&plan){plan=next;scene.setPlan(plan);timeline.setPlan(plan);renderDiagnostics();updateFrame(time);}
    if(!automatic){fitBaseline=structuredClone(kit);preFitSetup=null;}
    fitProposal=null;poseReview=null;fitState='수동 보정 · 현재 배치';refreshFitState();invalidateVideo();
    const unavailable=plan?.hits.filter(h=>h.limb===null).length??0;
    get(feedbackId).textContent=`보정 적용 완료${unavailable?` · 표현하지 못한 타격 ${unavailable}개 (분석 확인)`:''}`;
    applied=true;
  } catch(error) {
    if(sceneChanged){scene.setKitLayout(oldKit);scene.setCalibration(oldSeat);updateFrame(time);}
    get(feedbackId).textContent=error instanceof Error?error.message:String(error);
  }finally{
    staged?.dispose();if(!automatic){modelBusy=false;controls();status(plan?'일시정지':'대기');}
  }
  refreshCalibration();refreshTarget();if(applied&&!automatic)void refreshIdlePose();return applied;
}
refreshCalibration();refreshTarget();
get('seat-apply').onclick=()=>{
  const height=get<HTMLInputElement>('seat-height'), forward=get<HTMLInputElement>('seat-forward');
  if(!height.reportValidity()||!forward.reportValidity())return;
  applyCalibration({...scene.getCalibration(),seatHeight:height.valueAsNumber/100,seatForward:forward.valueAsNumber/100},scene.getKitLayout(),'calibration-message');
};
get<HTMLSelectElement>('palm-orientation').onchange=()=>void applyCalibration({...scene.getCalibration(),palmOrientation:get<HTMLSelectElement>('palm-orientation').value as HumanoidCalibration['palmOrientation']},scene.getKitLayout(),'calibration-message');
function refreshFitState() {
  get('fit-state').textContent=fitState;
  get('setup-summary').textContent=`배치: ${fitState}`;
  const capability=fitProposal?.capabilities;
  get('fit-capabilities').hidden=!capability;
  const list=get('fit-capability-list');list.replaceChildren();
  if(capability){
    get('fit-capability-summary').textContent=`${fitProposal?.complete?'적용 배치':'미적용 맞춤 후보'} · ${capability.geometrySource==='mesh-estimate'?'메시 기반 근사':'뼈대 기반 추정'} · 한 손 ${capability.handTargetChecks}회 · 양손 ${capability.pairChecks}조합 중 ${capability.availablePairs}조합 확인. 피부·의상 충돌 검사는 아닙니다.`;
    for(const pair of capability.pairs){const row=document.createElement('li');row.textContent=`L ${pair.leftPieceId} / R ${pair.rightPieceId} · ${{available:'가능한 자세 확인','not-found':'탐색 범위에서 찾지 못함',unmeasured:'측정 정보 부족'}[pair.status]}`;list.append(row);}
  }else get('fit-details').textContent='';

  controls();
}
let toastTimer: ReturnType<typeof setTimeout> | undefined;
function notify(text: string) {
  clearTimeout(toastTimer);get('toast-message').textContent=text;get('toast').hidden=false;
  toastTimer=setTimeout(()=>{if(!get('toast').contains(document.activeElement))get('toast').hidden=true;},6000);
}
get('toast-dismiss').onclick=()=>{clearTimeout(toastTimer);get('toast').hidden=true;};
for(const button of document.querySelectorAll<HTMLButtonElement>('[data-open-analysis]'))button.onclick=()=>openPanel('analysis');
get('analysis-source').onclick=()=>{openPanel('source');get<HTMLDetailsElement>('midi-settings').open=true;get<HTMLSelectElement>('mapping-profile').focus();};
function showFitResult(result: Awaited<ReturnType<typeof prepareAutomaticFit>>) {
  fitProposal=result.proposal;fitState=result.applied?'체형 자동 맞춤 적용':'자동 맞춤 확인 필요';
  get('calibration-message').textContent=result.applied?'':result.message;
  get('fit-details').textContent=result.message;
  refreshFitState();
  if(result.applied)notify('캐릭터에 맞게 드럼 배치를 조정했습니다.');
  else message('자동 맞춤 조건을 만족하지 못했습니다. 캐릭터의 수동 보정 또는 분석을 확인하세요.',true);
}
async function fitInBackground(profile: RigProfile, kit: readonly KitPiece[]) {
  fitRunning=true;controls();status('체형에 맞추는 중…','working');
  try { return await engine.fitKit(profile,kit); }
  finally { fitRunning=false;controls(); }
}
async function prepareSetupInBackground(profile: RigProfile, kit: readonly KitPiece[]) {
  setupRunning=true;controls();status('연주 자세를 준비하는 중…','working');
  try{return await engine.prepareSetup(profile,kit);}
  finally{setupRunning=false;controls();}
}
get('cancel-fit').onclick=()=>{if(fitRunning)engine.cancelFit();if(setupRunning)engine.cancel();};
async function autoFitCurrent() {
  if(busy||exporting||modelBusy)return;
  audio?.pause();modelBusy=true;controls();get('calibration-message').textContent='체형에 맞추는 중…';
  const previous={seat:scene.getCalibration(),kit:scene.getKitLayout()};
  try {
    const seat={...scene.getAutomaticCalibration(),palmOrientation:previous.seat.palmOrientation};
    const result=await prepareAutomaticFit(scene.measureCalibration(seat,fitBaseline),fitBaseline,previous.kit,fitInBackground);
    if(result.applied){
      if(!await applyCalibration(seat,result.kit,'calibration-message',true))return;
      preFitSetup=previous;
    }
    showFitResult(result);await refreshIdlePose();
  }catch(error){get('calibration-message').textContent=error instanceof Error?error.message:String(error);}
  finally{modelBusy=false;controls();status(plan?'일시정지':'대기');}
}
get('kit-fit').onclick=()=>void autoFitCurrent();
get('kit-fit-restore').onclick=async()=>{
  if(!preFitSetup||busy||exporting||modelBusy)return;
  const previous=preFitSetup;
  if(await applyCalibration(previous.seat,previous.kit,'calibration-message')){preFitSetup=null;fitState='맞춤 전 배치 복원';refreshFitState();notify('자동 맞춤 전 배치로 되돌렸습니다.');}
};
get('kit-layout-reset').onclick=()=>void applyCalibration(scene.getCalibration(),defaultKit,'calibration-message');
get('seat-auto').onclick=()=>void applyCalibration(scene.getAutomaticCalibration(),scene.getKitLayout(),'calibration-message');
get<HTMLSelectElement>('target-piece').onchange=refreshTarget;
get('target-apply').onclick=()=>{
  const fields=['x','y','z'].map(axis=>get<HTMLInputElement>(`target-${axis}`));
  if(fields.some(field=>!field.reportValidity()))return;
  try{applyCalibration(scene.getCalibration(),moveKitTarget(scene.getKitLayout(),get<HTMLSelectElement>('target-piece').value as PieceId,fields.map(field=>field.valueAsNumber/100) as Vec3),'target-message');}
  catch(error){get('target-message').textContent=String(error);refreshTarget();}
};
get('target-reset').onclick=()=>{
  const id=get<HTMLSelectElement>('target-piece').value as PieceId;
  applyCalibration(scene.getCalibration(),moveKitTarget(scene.getKitLayout(),id,defaultKit.find(p=>p.id===id)!.position),'target-message');
};
for (const button of document.querySelectorAll<HTMLButtonElement>('[data-panel]')) button.onclick=()=>openPanel(activePanel===button.dataset.panel?null:button.dataset.panel as Panel);
get('close-panel').onclick=()=>{const previous=activePanel;openPanel(null);document.querySelector<HTMLButtonElement>(`[data-panel="${previous}"]`)?.focus();};
get('theme').onclick=()=>setTheme(theme==='light'?'dark':'light');
get('start-upload').onclick=()=>{openPanel('source');get<HTMLInputElement>('file').click();};
get('start-demo').onclick=()=>{openPanel('source');void loadFile(makeDemo(),'Studio Groove.mid');};
function refreshCharacterDisplay() {
  const visible=get<HTMLInputElement>('character-visible').checked, model=get<HTMLInputElement>('model-visible').checked, skeleton=get<HTMLInputElement>('skeleton-visible').checked;
  get<HTMLSelectElement>('character-display').value=!visible||(!model&&!skeleton)?'hidden':model?(skeleton?'both':'model'):'skeleton';
}
function setCharacterDisplay(value:string) {
  const visible=value!=='hidden',skeleton=value==='skeleton'||value==='both',model=value==='model'||value==='both';
  get<HTMLInputElement>('character-visible').checked=visible;get<HTMLInputElement>('skeleton-visible').checked=skeleton;get<HTMLInputElement>('model-visible').checked=model;
  scene.setCharacterVisible(visible);scene.setSkeletonVisible(skeleton);scene.setModelVisible(model);refreshCharacterDisplay();
}
get<HTMLSelectElement>('character-display').onchange=e=>setCharacterDisplay((e.target as HTMLSelectElement).value);
function showModelInfo(info: VRMInspection | null) {
  const host = get('character-info'); host.replaceChildren(); if (!info) return;
  const name=document.createElement('strong');name.textContent=info.name;host.append(name);
  const metadata=document.createElement('details'),label=document.createElement('summary');label.textContent='모델 정보 · 이용 조건';metadata.append(label);host.append(metadata);
  for (const text of [`VRM ${info.version} · 모델 ${info.modelVersion} · 필수 본 ${info.required}/15`, `손가락 본 L ${info.fingers.left}/15 · R ${info.fingers.right}/15`, `제작자: ${info.authors.join(', ') || '미기재'}`]) {
    const p = document.createElement('p'); p.className='hint'; p.textContent=text; metadata.append(p);
  }
  if(info.fingers.left<15 || info.fingers.right<15) { const p=document.createElement('p'); p.className='hint'; p.textContent='일부 손가락 본이 없어 그립 표현이 제한됩니다. 스틱 부착과 연주는 유지됩니다.'; host.append(p); }
  for (const warning of info.metadataWarnings) { const p=document.createElement('p'); p.className='hint'; p.textContent=warning; metadata.append(p); }
  const details = document.createElement('details'), summary = document.createElement('summary'); summary.textContent='모델 이용 조건'; details.append(summary);
  for (const [label, value] of info.terms) { const p=document.createElement('p'); p.className='hint'; p.textContent=`${label}: ${value}`; details.append(p); }
  for(const reference of info.references) { const p=document.createElement('p'); p.className='hint'; p.textContent=`참고: ${reference}`; details.append(p); }
  metadata.append(details);
}
async function adoptRig(adapter: NativeHumanoidRigAdapter | VRMHumanoidRigAdapter) {
  const oldKit=scene.getKitLayout(),palmOrientation=scene.getCalibration().palmOrientation;
  const seat={...adapter.getAutomaticCalibration(fitBaseline),palmOrientation};
  const profile=adapter.setCalibration(seat,fitBaseline);
  const result=await prepareAutomaticFit(profile,fitBaseline,oldKit,fitInBackground);
  const finalSeat=result.applied?seat:{...adapter.getAutomaticCalibration(oldKit),palmOrientation};
  const finalProfile=adapter.setCalibration(finalSeat,result.kit);
  const staged=await prepareSetupInBackground(finalProfile,result.kit);
  try {
    scene.setKitLayout(result.kit);
    scene.setRig(adapter);
  }catch(error){scene.setKitLayout(oldKit);staged.dispose();throw error;}
  const next=staged.commit();
  if(next&&plan){plan=next;scene.setPlan(plan);timeline.setPlan(plan);renderDiagnostics();updateFrame(audio?.currentTime??0);}
  preFitSetup=result.applied?{seat:{...adapter.getAutomaticCalibration(oldKit),palmOrientation},kit:oldKit}:null;
  poseReview=null;showFitResult(result);invalidateVideo();refreshCalibration();refreshTarget();await refreshIdlePose();
}
get('character-import').onclick=()=>get<HTMLInputElement>('character-file').click();
get<HTMLInputElement>('character-file').onchange=async e=>{
  const input=e.target as HTMLInputElement, file=input.files?.[0]; input.value=''; if(!file||busy||exporting||modelBusy)return;
  audio?.pause(); modelBusy=true;controls();get('character-message').textContent='VRM을 검사하고 있습니다…';
  let candidate: Awaited<ReturnType<typeof loadLocalVRM>> | null = null;
  let adapter: VRMHumanoidRigAdapter | null = null;
  try {
    candidate=await loadLocalVRM(file);
    const reference=await fileReference('vrm',file),info=candidate.info;
    adapter=new VRMHumanoidRigAdapter(candidate.vrm,`vrm1:${reference.sha256}`); await adoptRig(adapter);
    candidate=null; adapter=null;
    commitProjectFile(reference,file);showModelInfo(info);
    get('character-message').textContent=file.name;setCharacterDisplay('model');
  } catch(error) { if(adapter)adapter.dispose();else if(candidate)VRMUtils.deepDispose(candidate.vrm.scene); get('character-message').textContent=error instanceof Error?error.message:String(error); }
  finally{modelBusy=false;controls();status(plan?'일시정지':'대기');}
};
get('character-reset').onclick=async()=>{
  if(busy||exporting||modelBusy)return;audio?.pause();modelBusy=true;controls();
  const adapter=new NativeHumanoidRigAdapter();let adopted=false;
  try{await adoptRig(adapter);adopted=true;sourceReferences=sourceReferences.filter(ref=>ref.kind!=='vrm');showModelInfo(null);get('character-message').textContent='기본 스켈레톤';setCharacterDisplay('skeleton');}
  catch(error){if(!adopted)adapter.dispose();get('character-message').textContent=String(error);}
  finally{modelBusy=false;controls();status(plan?'일시정지':'대기');}
};
get<HTMLSelectElement>('kit-preset').onchange=async e=>{
  const input=e.target as HTMLSelectElement;
  modelBusy=true;controls();get('model-message').textContent='드럼 모델을 준비하고 있습니다…';
  try {await scene.setKitPreset(input.value as KitPresetId);committedPreset=input.value;sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart');refreshPiece();get('model-message').textContent=({library:'Innerscene · CC0',batoski:'Batoski · CC BY 3.0',smoj:'smoj · CC BY 3.0'} as Record<string,string>)[input.value] ?? '자체 생성 도형 · 부품별로 외형을 변경할 수 있습니다.';}
  catch(error){input.value=committedPreset;get('model-message').textContent=`모델 로딩 실패: ${error instanceof Error?error.message:String(error)}`;}
  finally{modelBusy=false;controls();}
};
get<HTMLSelectElement>('piece').onchange=()=>{scene.selectPiece(selectedPiece());get<HTMLSelectElement>('target-piece').value=selectedPiece();refreshPiece();refreshTarget();};
scene.onPieceSelect=id=>{if(!id)return;get<HTMLSelectElement>('piece').value=id==='openHat'||id==='pedalHat'?'hihat':id;get<HTMLSelectElement>('target-piece').value=id==='openHat'?'hihat':id;refreshPiece();refreshTarget();};
get<HTMLSelectElement>('piece-model').onchange=e=>{scene.setPieceVisual(selectedPiece(),{model:(e.target as HTMLSelectElement).value as 'standard'|'deep'|'compact'});sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart'||ref.pieceId!==selectedPiece());refreshPiece();};
get<HTMLInputElement>('piece-finish').oninput=e=>scene.setPieceVisual(selectedPiece(),{finish:(e.target as HTMLInputElement).value});
get<HTMLInputElement>('piece-scale').oninput=e=>{const scale=Number((e.target as HTMLInputElement).value);scene.setPieceVisual(selectedPiece(),{scale});get('piece-scale-value').textContent=`${Math.round(scale*100)}%`;};
get('piece-reset').onclick=()=>{scene.resetPieceVisual(selectedPiece());sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart'||ref.pieceId!==selectedPiece());refreshPiece();get('model-message').textContent='선택한 부품을 프리셋 상태로 복원했습니다.';};
get<HTMLInputElement>('piece-rotation').oninput=e=>{const rotation=Number((e.target as HTMLInputElement).value);scene.setPieceVisual(selectedPiece(),{rotation});get('piece-rotation-value').textContent=`${rotation}°`;};
get('piece-import').onclick=()=>get<HTMLInputElement>('model-file').click();
get<HTMLInputElement>('model-file').onchange=async e=>{
  const input=e.target as HTMLInputElement;const file=input.files?.[0];input.value='';if(!file)return;
  modelBusy=true;controls();get('model-message').textContent='GLB를 불러오는 중…';
  try{const id=selectedPiece(),reference=await fileReference('drumPart',file,id),result=await scene.loadPieceModel(id,file);commitProjectFile(reference,file);get('model-message').textContent=`${result.fileName} · 자동 크기 맞춤 (타점 확인 필요)`;refreshPiece();}
  catch(error){get('model-message').textContent=error instanceof Error?error.message:String(error);}
  finally{modelBusy=false;controls();}
};
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
let responseChosen = false;
get<HTMLInputElement>('hit-response').checked = !reducedMotion.matches;
function updateHitResponse() {
  const enabled = get<HTMLInputElement>('hit-response').checked;
  scene.setHitResponse({enabled, strength:Number(get<HTMLInputElement>('hit-strength').value)});
  controls();
}
get<HTMLInputElement>('hit-response').onchange = () => { responseChosen = true; updateHitResponse(); };
get<HTMLInputElement>('hit-strength').oninput = updateHitResponse;
reducedMotion.addEventListener('change', () => { if (!responseChosen) { get<HTMLInputElement>('hit-response').checked = !reducedMotion.matches; updateHitResponse(); } });
updateHitResponse();
get<HTMLInputElement>('timeline-zoom').oninput=e=>timeline.setZoom(Number((e.target as HTMLInputElement).value));
document.addEventListener('keydown',e=>{if(e.key==='Escape'&&activePanel)get<HTMLButtonElement>('close-panel').click();});
setTheme(theme);

const clockText = (seconds: number) => `${Math.floor(Math.max(0, seconds) / 60).toString().padStart(2,'0')}:${Math.floor(Math.max(0,seconds) % 60).toString().padStart(2,'0')}`;
const preciseClockText = (seconds: number) => `${clockText(seconds)}.${Math.floor((Math.max(0,seconds)%1)*1000).toString().padStart(3,'0')}`;
function status(text: string, kind = '') { get('status').className = `status ${kind}`; get('status').textContent = text; }
function message(text: string, error = false) { const node = get('message'); node.textContent = text; node.classList.toggle('error', error); }
function controls() {
  for (const id of ['play','reset','seek','export']) (get(id) as HTMLButtonElement).disabled = !plan || busy || exporting || modelBusy;
  for (const id of ['upload','demo','verification-demo','replan','channel','mapping-profile','wav-import','wav-remove','wav-offset','wav-apply','mute-export','background']) (get(id) as HTMLInputElement).disabled = busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting || activePanel==='drums';
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-panel], #close-panel, #theme, #character-visible')) el.disabled=exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-drums input, #panel-drums select, #panel-drums button')) el.disabled=busy || exporting || modelBusy;
  for (const id of ['start-upload','start-demo']) get<HTMLButtonElement>(id).disabled=busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-character input, #panel-character select, #panel-character button')) el.disabled=busy || exporting || modelBusy;
  get<HTMLInputElement>('hit-strength').disabled = exporting || modelBusy || !get<HTMLInputElement>('hit-response').checked;
  get<HTMLButtonElement>('export').disabled = !plan || busy || exporting || modelBusy;
  get<HTMLInputElement>('count-in-75').disabled = busy || exporting || modelBusy;
  get<HTMLButtonElement>('wav-remove').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLButtonElement>('wav-apply').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLInputElement>('hihat-policy').disabled=busy||exporting||modelBusy;
  get<HTMLInputElement>('hihat-openness').disabled=busy||exporting||modelBusy||!get<HTMLInputElement>('hihat-policy').checked;
  get('hihat-openness-field').hidden = !get<HTMLInputElement>('hihat-policy').checked;
  get<HTMLButtonElement>('kit-fit-restore').disabled=busy||exporting||modelBusy||!preFitSetup;
  for(const el of document.querySelectorAll<HTMLInputElement>('#performance-review input, #performance-review button'))el.disabled=!plan||busy||exporting||modelBusy;
  for(const el of document.querySelectorAll<HTMLInputElement>('#arrangement-editor input, #arrangement-editor select, #arrangement-editor button'))el.disabled=!plan||busy||exporting||modelBusy;
  updateArrangementControls();
  for(const id of ['project-open','project-relink'])get<HTMLButtonElement>(id).disabled=busy||exporting||modelBusy;
  get<HTMLButtonElement>('project-save').disabled=!plan||!preparedSelection||busy||exporting||modelBusy;
  get('cancel-analysis').hidden = !busy||projectBusy;
  get('cancel-fit').hidden = !fitRunning&&!setupRunning;
  get('cancel-fit').textContent=setupRunning?'준비 취소':'맞춤 취소';
  get('export-progress').hidden = !exporting;
}
function updateFrame(time: number) {
  if (!plan) return;
  const pose = engine.evaluate(time - PRE_ROLL);
  if (pose) scene.applyPose(pose, time - PRE_ROLL);
  if (activePanel==='analysis') {
    const samples=scene.getRigContactSamples();
    get('contact-info').textContent=samples.map(s=>`${['L 스틱','R 스틱','L 발','R 발'][s.limb]} ${(s.errorMeters*1000).toFixed(2)} mm${s.wristSwingDegrees===undefined?'':` · 손목 ${s.wristSwingDegrees.toFixed(1)}°`}`).join(' · ');
  }
}
function fillSelection(info: MidiInfo) {
  const list = get('tracks'); list.replaceChildren();
  for (const track of info.tracks.filter(t=>t.noteCount>0)) {
    const label = document.createElement('label'); label.className = 'track-option';
    const input = document.createElement('input'); input.type = 'checkbox'; input.value = String(track.index); input.checked = info.suggestedTracks.includes(track.index);
    const name = document.createElement('span'); name.textContent = track.name || `트랙 ${track.index+1}`;
    const count = document.createElement('small'); count.textContent = `${track.noteCount} notes`;
    label.append(input,name,count); list.append(label);
  }
  const select = get<HTMLSelectElement>('channel'); select.replaceChildren(new Option('전체 채널','all'));
  const channels = [...new Set(info.tracks.flatMap(t=>t.channels))].sort((a,b)=>a-b);
  channels.forEach(c=>select.add(new Option(`${c+1}${c===9?' · GM 드럼':''}`,String(c))));
  if (channels.includes(9)) select.value = '9';
  get('selection').hidden = false;
}
async function prepare(generation: number,restoreTime=0) {
  const tracks = [...document.querySelectorAll<HTMLInputElement>('#tracks input:checked')].map(el=>Number(el.value));
  if (!tracks.length) throw new Error('연주할 트랙을 하나 이상 선택해 주세요.');
  const selected = get<HTMLSelectElement>('channel').value;
  if(!get<HTMLInputElement>('hihat-openness').reportValidity())throw new Error('하이햇 열림은 5–100% 범위여야 합니다.');
  message('타격과 손발 동작을 준비하고 있습니다…');
  const selection:DrumProject['selection']={tracks,channel:selected==='all'?null:Number(selected),mappingProfile:get<HTMLSelectElement>('mapping-profile').value as 'gm'|'logicDkdGm',countInNote75:get<HTMLInputElement>('count-in-75').checked,hihatPolicy:get<HTMLInputElement>('hihat-policy').checked?'shortOpenClose':'literal',hihatOpenness:get<HTMLInputElement>('hihat-openness').valueAsNumber/100,...structuredClone(arrangement)};
  const prepared=await preparePerformance({candidate:()=>engine.plan(bytes!,selection,false),
    audio:async candidate=>{message(wav?'WAV를 연주 시간에 맞추고 있습니다…':'합성 드럼 오디오를 준비하고 있습니다…');
      const aligned=wav?alignWavToPlan(wav.buffer,candidate,wavOffset):null;
      return {buffer:aligned?.buffer??await synthesis.run(candidate),alignment:aligned?.info};},
    commit:candidate=>engine.restorePlan(candidate,scene.getRigProfile(),scene.getKitLayout()),current:()=>generation===loadGeneration});
  if(!prepared)return;
  plan = prepared.plan;preparedSelection=selection;arrangement=arrangementOf(plan);
  if(prepared.audio.alignment)showWavInfo(prepared.audio.alignment);
  buffer = prepared.audio.buffer;
  audio?.dispose(); audio = new DrumAudio(buffer); audio.setVolume(Number(get<HTMLInputElement>('volume').value));
  scene.setPlan(plan);audio.seek(restoreTime);updateFrame(audio.currentTime);invalidateVideo();
  const total = outputDuration(plan);
  get<HTMLInputElement>('seek').max = String(total); get<HTMLInputElement>('seek').value = String(audio.currentTime);
  get('duration').textContent = clockText(total); timeline.setPlan(plan);
  get('metric-duration').textContent = clockText(plan.duration); get('metric-hits').textContent = plan.hits.length.toLocaleString(); get('metric-tempos').textContent = String(plan.tempoCount);
  get('plan-label').textContent = 'READY'; get('stage-empty').hidden = true;
  poseReview=null;get('review-status').textContent='곡 검수는 30fps와 타격 전후 표본의 재생·임의 탐색 자세를 비교합니다.';get('review-measurement').textContent='';renderDiagnostics();
}
function renderDiagnostics() {
  if(!plan)return;
  const guidance=importGuidance(plan);
  get('mapping-evidence').textContent=guidance.evidence.join(' ');
  get('count-in-choice').hidden=!guidance.countInCandidates;
  get('count-in-label').textContent=`카운트인으로 해석 · 후보 ${guidance.countInCandidates}개`;
  get('hihat-choice').hidden=!guidance.hasHihatChoice;
  get('hihat-candidates').textContent=`준비한 트랙: 열기 ${guidance.open}개 · 발 타격 ${guidance.foot}개 · 연결된 닫힘 ${guidance.paired}개`;
  get('interpretation-empty').hidden=!!guidance.countInCandidates||guidance.hasHihatChoice;
  get<HTMLInputElement>('review-time').max=String(plan.duration);
  refreshFitState();
  const diagnosticList = get('diagnostic-list'); diagnosticList.replaceChildren();
  const unresolved = plan.hits.filter(h=>h.limb === null).length;
  const interpretations = plan.interpretations ?? [];
  const countIn = interpretations.filter(event=>event.kind==='countIn').length;
  const unmapped = interpretations.filter(event=>event.kind==='unmapped').length;
  const omitted=interpretations.filter(event=>event.kind==='omitted').length;
  const warnings=unmapped+unresolved;get('analysis-badge').textContent=warnings?` ${warnings}`:'';get('analysis-badge').hidden=!warnings;
  get('analysis-summary').textContent=warnings?`미매핑 ${unmapped}개 · 동작 미배정 ${unresolved}개`:'모든 타격에 동작이 배정되었습니다. 경로 검수는 별도로 실행하세요.';
  get('source-summary').textContent=warnings?`확인할 타격 ${warnings}개 · 분석에서 확인하세요.`:'';get('source-analysis').hidden=!warnings;
  get('interpretation-summary').textContent = `${plan.mappingProfile==='logicDkdGm'?'Logic DKD · GM':'General MIDI'} · 카운트인 ${countIn}개 · 제어 ${interpretations.filter(event=>event.kind==='control').length}개 · 미매핑 ${unmapped}개 · 제외 ${omitted}개 · 손 지정 ${Object.keys(plan.handOverrides??{}).length}개 · 동작 미배정 ${unresolved}개`;
  get('diagnostics').hidden = !plan.diagnostics.length;
  get('diagnostic-summary').textContent = `분석 알림 ${plan.diagnostics.length}개${unresolved?` · 표현하지 못한 타격 ${unresolved}개`:''}`;
  for (const item of plan.diagnostics.slice(0,100)) { const li=document.createElement('li'); const text=`${item.time !== undefined ? preciseClockText(item.time)+' · ':''}${item.message}`;
    if(item.time!==undefined){const jump=document.createElement('button');jump.textContent=text;jump.onclick=()=>{if(!plan||busy||exporting||modelBusy)return;const time=item.time!+PRE_ROLL;audio?.seek(time);updateFrame(time);};li.append(jump);}else li.textContent=text;
    if(item.hitId!==undefined){const hit=plan.hits[item.hitId];if(hit){const related=plan.hits.filter(other=>other.timeUs===hit.timeUs&&other.time===hit.time);const detail=document.createElement('p');detail.className='hint';detail.textContent=`MIDI tick ${hit.tick} · 동시 타격: ${related.map(other=>`${other.pieceId??other.instrument} (${other.note})`).join(', ')} · 버튼을 누르면 해당 위치로 이동합니다.`;li.append(detail);}}
    diagnosticList.append(li); }
  if(plan.diagnostics.length>100){const li=document.createElement('li');li.textContent=`외 ${plan.diagnostics.length-100}개 알림`;diagnosticList.append(li);}
  status('준비 완료','ready'); message(unmapped||unresolved?`준비 완료 · 미매핑 ${unmapped}개 · 동작 미배정 ${unresolved}개. 분석에서 확인하세요.`:'준비가 끝났습니다. 재생 버튼을 눌러 연주를 시작하세요.'); timeline.resize();
}
function clearPerformance() {
  reviewGeneration++;poseReview=null;preparedSelection=null;audio?.dispose(); audio=null; buffer=null; plan=null;
  engine.clearPerformance(); scene.clearPlan();void refreshIdlePose(); wavAlignmentInfo=undefined;showWavInfo();
  for (const id of ['metric-duration','metric-hits','metric-tempos','plan-label']) get(id).textContent='—';
  for (const id of ['duration','current-time']) get(id).textContent='00:00';
  get('stage-empty').hidden=false; get('diagnostics').hidden=true; get('interpretation-summary').textContent='';
  get('analysis-summary').textContent='MIDI를 불러오면 분석 결과를 확인할 수 있습니다.';get('analysis-badge').hidden=true;get('source-summary').textContent='';get('source-analysis').hidden=true;
  get<HTMLInputElement>('seek').value='0'; timeline.setPlan(null);
  setButtonIcon(get('play'), 'play'); get('play').setAttribute('aria-label','재생');
  invalidateVideo();
  timeline.resize();
}
async function loadFile(data: Uint8Array, name: string) {
  if (busy || exporting || modelBusy) return;
  arrangement={omittedSourceEventIds:[],handOverrides:{}};
  const generation = ++loadGeneration;
  clearPerformance(); engine.cancel(); synthesis.cancel();
  bytes = data; fileName=name; get('file-name').textContent=name;
  get('file-detail').textContent=`${(data.byteLength/1024).toFixed(1)} KiB · 브라우저에서 분석`;
  get('selection').hidden=true; get('diagnostics').hidden=true; busy=true; controls(); status('분석 중','working');
  try {
    if(data.byteLength>MAX_FILE_BYTES)throw new Error('파일이 10 MiB를 초과합니다. 더 짧은 구간으로 내보내 주세요.');
    await rememberProjectFile('midi',new File([Uint8Array.from(data)],name,{type:'audio/midi'}));
    message('MIDI의 트랙과 템포를 읽고 있습니다…');
    const info = await engine.inspect(data);
    if(generation!==loadGeneration)return;
    fillSelection(info); controls(); await prepare(generation);
  } catch(error) {
    if(generation!==loadGeneration)return;
    message(error instanceof Error ? error.message : String(error),true); status('확인 필요','error');
  } finally { if(generation===loadGeneration){busy=false;controls();} }
}

function showWavInfo(info?: WavAlignmentInfo) {
  if(info)wavAlignmentInfo=info;
  if(!plan)wavAlignmentInfo=undefined;
  info=info??wavAlignmentInfo;
  if(!wav){wavAlignmentInfo=undefined;get('wav-info').textContent='합성 드럼 오디오 · WAV를 불러오면 이 소리를 대체합니다.';return;}
  const parts=[wav.name, `${wav.buffer.duration.toFixed(2)}초`, `시작 ${wavOffset.toFixed(3)}초`];
  if(info){
    if(info.copiedSeconds===0)parts.push('주의: 현재 곡 범위와 겹치지 않아 전체 무음');
    if(info.trimmedStartSeconds>0)parts.push(`앞 ${info.trimmedStartSeconds.toFixed(2)}초 잘림`);
    if(info.trimmedEndSeconds>0)parts.push(`뒤 ${info.trimmedEndSeconds.toFixed(2)}초 잘림`);
    if(info.trailingSilenceSeconds>.001)parts.push(`끝 무음 ${info.trailingSilenceSeconds.toFixed(2)}초`);
  }else parts.push('MIDI를 준비하면 적용됩니다');
  get('wav-info').textContent=parts.join(' · ');
}
function replaceAudio(next:AudioBuffer,time=audio?.pause()??0) {
  audio?.dispose();buffer=next;audio=new DrumAudio(next);audio.setVolume(Number(get<HTMLInputElement>('volume').value));audio.seek(time);invalidateVideo();updateFrame(time);status('일시정지');
}
get('wav-import').onclick=()=>get<HTMLInputElement>('wav-file').click();
get<HTMLInputElement>('wav-file').onchange=async event=>{
  const input=event.target as HTMLInputElement,file=input.files?.[0];input.value='';
  if(!file||busy||exporting||modelBusy)return;
  const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status(plan?'일시정지':'대기');get('wav-info').textContent='WAV를 읽고 있습니다…';
  try{
    const candidate=await loadLocalWav(file);if(generation!==loadGeneration)return;
    const aligned=plan?alignWavToPlan(candidate.buffer,plan,wavOffset):null;
    const reference=await fileReference('wav',file);if(generation!==loadGeneration)return;
    wav=candidate;commitProjectFile(reference,file);if(aligned)replaceAudio(aligned.buffer,time);showWavInfo(aligned?.info);
  }catch(error){if(generation===loadGeneration){showWavInfo();message(error instanceof Error?error.message:String(error),true);}}
  finally{if(generation===loadGeneration){busy=false;controls();}}
};
get('wav-apply').onclick=()=>{
  if(!wav||busy||exporting||modelBusy)return;
  const field=get<HTMLInputElement>('wav-offset');if(!field.reportValidity())return;
  try{const next=field.valueAsNumber;const aligned=plan?alignWavToPlan(wav.buffer,plan,next):null;
    if(!Number.isFinite(next)||Math.abs(next)>300)throw new Error('WAV 시작 위치는 −300–300초 범위여야 합니다.');
    wavOffset=next;if(aligned)replaceAudio(aligned.buffer);showWavInfo(aligned?.info);
  }catch(error){message(error instanceof Error?error.message:String(error),true);}
};
get('wav-remove').onclick=async()=>{
  if(!wav||busy||exporting||modelBusy)return;
  const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status(plan?'일시정지':'대기');
  try{const next=plan?await synthesis.run(plan):null;if(generation!==loadGeneration)return;
    wav=null;sourceReferences=sourceReferences.filter(ref=>ref.kind!=='wav');wavOffset=0;get<HTMLInputElement>('wav-offset').value='0';if(next)replaceAudio(next,time);showWavInfo();
  }catch(error){if(generation===loadGeneration)message(String(error),true);}
  finally{if(generation===loadGeneration){busy=false;controls();}}
};

function updateArrangementControls() {
  const id=get<HTMLSelectElement>('arrangement-event').value;
  const meaning=plan?.interpretations?.find(event=>event.sourceEventId===id);
  const locked=!plan||busy||exporting||modelBusy;
  get<HTMLButtonElement>('arrangement-apply').disabled=locked||!meaning;
  get<HTMLInputElement>('arrangement-omit').disabled=locked||!meaning;
  get<HTMLSelectElement>('arrangement-hand').disabled=locked||get<HTMLInputElement>('arrangement-omit').checked||meaning?.kind!=='strike'||!meaning.instrument||['kick','pedalHat'].includes(meaning.instrument);
  get<HTMLButtonElement>('arrangement-reset').disabled=locked||(!arrangement.omittedSourceEventIds.length&&!Object.keys(arrangement.handOverrides).length);
}
function selectArrangementEvent(id:string) {
  if(!plan)return;
  const selected=noteSources(plan).find(event=>event.id===id);if(!selected)return;
  const names=new Map(createTimelineNotes(plan).map(note=>[note.id,note]));
  const group=nearestNoteGroup(plan,selected.timeUs/1e6),select=get<HTMLSelectElement>('arrangement-event');
  select.replaceChildren(...group.map(event=>new Option(`${names.get(event.id)?.name??'Note'} · ${event.message.kind==='noteOn'?event.message.key:''} · track ${event.track+1}`,event.id)));
  select.value=id;
  const meaning=plan.interpretations?.find(event=>event.sourceEventId===id),hit=plan.hits.find(event=>event.sourceEventId===id);
  get('arrangement-detail').textContent=`${(selected.timeUs/1e6).toFixed(6)}초 · tick ${selected.tick} · 원본 ${id} · Velocity ${selected.message.kind==='noteOn'?selected.message.velocity:''} · ${meaning?.kind==='omitted'?'연주에서 제외':hit?.limb==null?'동작 없음':['왼손','오른손','왼발','오른발'][hit.limb]}`;
  get<HTMLInputElement>('arrangement-omit').checked=meaning?.kind==='omitted';
  get<HTMLSelectElement>('arrangement-hand').value=plan.handOverrides?.[id]===undefined?'auto':String(plan.handOverrides[id]);
  updateArrangementControls();
}
get<HTMLSelectElement>('arrangement-event').onchange=event=>selectArrangementEvent((event.target as HTMLSelectElement).value);
get<HTMLInputElement>('arrangement-omit').onchange=updateArrangementControls;
get('arrangement-nearest').onclick=()=>{
  if(!plan)return;const group=nearestNoteGroup(plan,(audio?.currentTime??PRE_ROLL)-PRE_ROLL);
  if(group.length){selectArrangementEvent(group[0].id);jumpToSongTime(group[0].timeUs/1e6);}
};
for(const [id,direction] of [['arrangement-previous',-1],['arrangement-next',1]] as const)get(id).onclick=()=>{
  if(!plan)return;
  const notes=noteSources(plan),selected=notes.find(event=>event.id===get<HTMLSelectElement>('arrangement-event').value);
  const time=selected?.timeUs??((audio?.currentTime??PRE_ROLL)-PRE_ROLL)*1e6;
  const event=(direction<0?notes.reverse():notes).find(event=>direction*(event.timeUs-time)>0);
  if(event){selectArrangementEvent(event.id);jumpToSongTime(event.timeUs/1e6);}
};
async function applyArrangement(next:Arrangement) {
  if(!plan||!preparedSelection||!bytes||busy||exporting||modelBusy)return;
  const oldPlan=plan,oldSelection=preparedSelection,time=audio?.pause()??0,selectedId=get<HTMLSelectElement>('arrangement-event').value;
  busy=true;projectBusy=true;controls();status('음표 편집 준비 중','working');
  try {
    const selection={...structuredClone(oldSelection),...next};
    const candidate=await engine.plan(bytes,selection,false);
    const aligned=wav?alignWavToPlan(wav.buffer,candidate,wavOffset):null;
    const nextBuffer=aligned?.buffer??await synthesis.run(candidate);
    const restored=await engine.restorePlan(candidate,scene.getRigProfile(),scene.getKitLayout());
    plan=restored;preparedSelection=selection;arrangement=arrangementOf(restored);poseReview=null;
    scene.setPlan(plan);timeline.setPlan(plan);replaceAudio(nextBuffer,time);showWavInfo(aligned?.info);invalidateVideo();
    get('metric-hits').textContent=plan.hits.length.toLocaleString();renderDiagnostics();
    selectArrangementEvent(selectedId);
    get('arrangement-status').textContent='음표 편집을 적용했습니다. 같은 시각에서 재생·탐색으로 확인하세요. 원본 MIDI와 WAV는 보존됩니다.';
  } catch(error){
    plan=oldPlan;preparedSelection=oldSelection;
    get('arrangement-status').textContent=error instanceof Error?error.message:String(error);
    status('편집 확인 필요','error');
  } finally {busy=false;projectBusy=false;controls();updateFrame(time);}
}
get('arrangement-apply').onclick=()=>{
  if(!plan)return;
  try{void applyArrangement(withArrangementEdit(plan,get<HTMLSelectElement>('arrangement-event').value,get<HTMLInputElement>('arrangement-omit').checked,get<HTMLSelectElement>('arrangement-hand').value as 'auto'|'0'|'1'));}
  catch(error){get('arrangement-status').textContent=error instanceof Error?error.message:String(error);}
};
get('arrangement-reset').onclick=()=>void applyArrangement({omittedSourceEventIds:[],handOverrides:{}});

function jumpToSongTime(time: number) {
  if(!plan||busy||exporting||modelBusy||!Number.isFinite(time))return;
  const songTime=Math.max(0,Math.min(plan.duration,time));
  audio?.pause();audio?.seek(songTime+PRE_ROLL);updateFrame(songTime+PRE_ROLL);
  get<HTMLInputElement>('review-time').value=songTime.toFixed(3);status('일시정지');
}
get('review-jump').onclick=()=>{
  const field=get<HTMLInputElement>('review-time');if(field.reportValidity())jumpToSongTime(field.valueAsNumber);
};
for(const [id,direction] of [['review-previous',-1],['review-next',1]] as const)get(id).onclick=()=>{
  if(!plan)return;
  const songTime=(audio?.currentTime??PRE_ROLL)-PRE_ROLL;
  const hits=direction<0?[...plan.hits].reverse():plan.hits;
  const hit=hits.find(candidate=>direction*(candidate.time-songTime)>.000001);
  if(hit)jumpToSongTime(hit.time);
};
function currentReviewMeasurement() {
  const outputTime=audio?.pause()??0,songTime=outputTime-PRE_ROLL;
  updateFrame(outputTime);
  const contacts=scene.getRigContactSamples(),pose=engine.evaluate(songTime),proxy=engine.inspectPose(songTime);
  get('review-measurement').textContent=`곡 ${songTime.toFixed(3)}초 · ${contacts.map(sample=>`${['L 스틱','R 스틱','L 발','R 발'][sample.limb]} ${(sample.errorMeters*1000).toFixed(2)} mm`).join(' · ')}${proxy?` · 근사 간격: 몸통 ${(proxy.torsoClearanceMeters*100).toFixed(1)} cm / 양팔 ${(proxy.armClearanceMeters*100).toFixed(1)} cm / 스틱 ${(proxy.stickClearanceMeters*100).toFixed(1)} cm (음수는 침범 후보)` : ''}`;
  return { songTime, outputTime, contacts, proxy, pose:pose?Array.from(pose):null, sampledReview:poseReview };
}
get('review-current').onclick=()=>{if(plan&&!busy&&!exporting&&!modelBusy){currentReviewMeasurement();status('일시정지');}};
get('review-trace-hide').onclick=()=>scene.setStickTrace(null);
get('review-trace').onclick=()=>{
  if(!plan||busy||exporting||modelBusy)return;
  const outputTime=audio?.pause()??0,center=outputTime-PRE_ROLL;
  const start=Math.max(-PRE_ROLL,center-.3),end=Math.min(plan.duration,center+.3),paths:[Vec3[],Vec3[]]=[[],[]];
  try {
    for(let index=0;index<=36;index++){
      const time=start+(end-start)*index/36,pose=engine.evaluate(time);if(!pose)throw new Error('스틱 자세가 없습니다.');
      scene.applyPose(pose,time);
      for(const sample of scene.getRigContactSamples())if(sample.limb<2)paths[sample.limb].push([...sample.actual]);
    }
    scene.setStickTrace(paths);
    get('review-measurement').textContent=`곡 ${start.toFixed(3)}–${end.toFixed(3)}초 · 실제 스틱 끝 37개 표본 · 왼손 주황 / 오른손 파랑. 현재 시각 주변의 고정 궤적이며 충돌 판정은 아닙니다.`;
  } catch(error){get('review-measurement').textContent=error instanceof Error?error.message:String(error);}
  finally{updateFrame(outputTime);status('일시정지');}
};
get('review-scan').onclick=async()=>{
  if(!plan||busy||exporting||modelBusy)return;
  const current=plan,restoreTime=audio?.pause()??0,generation=++reviewGeneration;
  const times=reviewTimes(current),ordered:Float32Array[]=[],shuffled:Float32Array[]=[],inspections:PoseInspection[]=[];
  busy=true;reviewRunning=true;controls();status('곡 검수 중','working');
  let sinceYield=performance.now();
  const check=async(index:number)=>{
    if(generation!==reviewGeneration)throw new DOMException('검수를 취소했습니다.','AbortError');
    if(performance.now()-sinceYield>12){get('review-status').textContent=`자세 비교 ${index.toLocaleString()} / ${(times.length*2).toLocaleString()} · 30fps 및 타격 전후 표본`;await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()));sinceYield=performance.now();}
  };
  try{
    for(let index=0;index<times.length;index++){await check(index);ordered[index]=Float32Array.from(engine.evaluate(times[index])??[]);const proxy=engine.inspectPose(times[index]);if(proxy)inspections.push(proxy);}
    let count=times.length;
    for(const index of reviewOrder(times.length)){await check(count++);shuffled[index]=Float32Array.from(engine.evaluate(times[index])??[]);}
    poseReview=summarizePoseReview(times,ordered,shuffled);
    if(inspections.length)poseReview.proxy=summarizeProxyReview(times,inspections);
    get('review-status').textContent=`${poseReview.samples.toLocaleString()}개 시각 검수 · 탐색 순서 차이 ${poseReview.maxOrderDifference.toExponential(2)} · 유효하지 않은 값 ${poseReview.nonfiniteValues}개${poseReview.proxy?` · 근사 영역 침범 후보 ${poseReview.proxy.intersectingSamples}개 표본${poseReview.proxy.firstIntersectionSeconds===null?'':` (처음 ${poseReview.proxy.firstIntersectionSeconds.toFixed(3)}초)`}`:''}. 자세 재현성·근사 영역 검사이며 피부·옷 충돌 검사는 아닙니다.`;
  }catch(error){get('review-status').textContent=error instanceof Error?error.message:String(error);}
  finally{reviewRunning=false;if(generation===reviewGeneration){busy=false;status('일시정지');controls();}updateFrame(restoreTime);}
};
get('review-download').onclick=()=>{
  if(!plan||busy||exporting||modelBusy)return;
  const report=createReviewSnapshot(plan,scene.getRigProfile(),{
    calibration:scene.getCalibration(),fitState,hasUnappliedFitProposal:false,
    audio:{source:wav?'localWav':'synthesized',wavInfo:wav?.info,alignment:wavAlignmentInfo,offsetSeconds:wavOffset},
    selection:{tracks:[...document.querySelectorAll<HTMLInputElement>('#tracks input:checked')].map(input=>Number(input.value)),channel:get<HTMLSelectElement>('channel').value},
  },currentReviewMeasurement());
  const url=URL.createObjectURL(new Blob([JSON.stringify(report,null,2)],{type:'application/json'}));
  const link=document.createElement('a');link.href=url;link.download='drum-vrm-review.json';link.click();
  setTimeout(()=>URL.revokeObjectURL(url),1000);status('일시정지');
};

function downloadJson(value:unknown,name:string) {
  const url=URL.createObjectURL(new Blob([JSON.stringify(value,null,2)],{type:'application/json'}));
  const link=document.createElement('a');link.href=url;link.download=name;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
}
get('project-save').onclick=async()=>{
  if(!plan||!preparedSelection||busy||exporting||modelBusy)return;
  const current=plan,selection=structuredClone(preparedSelection),outputTime=audio?.pause()??0;
  busy=true;projectBusy=true;controls();
  try {
    const project:DrumProject={format:'drum-vrm-project',schemaVersion:1,engineModuleSha256:await projectEngineHash(),plan:current,profile:scene.getRigProfile(),selection,
      calibration:scene.getCalibration(),kit:scene.getKitLayout(),rig:sourceReferences.some(ref=>ref.kind==='vrm')?'vrm':'native',assets:structuredClone(sourceReferences),
      appearance:{preset:committedPreset as KitPresetId,pieces:current.kit.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).map(piece=>({id:piece.id,settings:scene.getPieceVisual(piece.id)}))},
      camera:scene.getViewState(),hitResponse:scene.getHitResponse(),audio:{source:wav?'wav':'synthesized',offsetSeconds:wavOffset,volume:Number(get<HTMLInputElement>('volume').value)},
      view:{theme,background:get<HTMLInputElement>('background').value,skeleton:get<HTMLInputElement>('skeleton-visible').checked,model:get<HTMLInputElement>('model-visible').checked,character:get<HTMLInputElement>('character-visible').checked,timelineZoom:Number(get<HTMLInputElement>('timeline-zoom').value)},outputTime};
    parseProject(JSON.stringify(project));
    downloadJson(project,`${fileName.replace(/\.(mid|midi)$/i,'')}.drumvrm.json`);
    get('project-message').textContent='프로젝트를 저장했습니다. MIDI 노트·계획이 포함된 개인 파일입니다. 같은 버전의 앱과 원본 WAV·VRM·GLB를 함께 보관하세요.';
  }catch(error){get('project-message').textContent=error instanceof Error?error.message:String(error);}
  finally{busy=false;projectBusy=false;controls();status('일시정지');}
};
function showMissingProjectAssets() {
  if(!pendingProject)return;
  const missing=missingProjectAssets(pendingProject,projectFiles);
  get('project-relink').hidden=!missing.length;
  if(missing.length)get('project-message').textContent=`현재 연주는 유지됩니다. 다시 연결할 원본: ${missing.map(asset=>`${asset.kind} · ${asset.name}`).join(', ')}. 이름이 같아도 내용의 SHA-256이 다르면 사용할 수 없습니다.`;
}
async function restorePendingProject() {
  const project=pendingProject;if(!project)return;
  if(missingProjectAssets(project,projectFiles).length){showMissingProjectAssets();return;}
  const file=(kind:ProjectAsset['kind'])=>{const reference=project.assets.find(asset=>asset.kind===kind);return reference?projectFiles.get(reference.sha256)!:null;};
  let adapter:NativeHumanoidRigAdapter|VRMHumanoidRigAdapter|null=null;
  let preparedAppearance:Awaited<ReturnType<DrumScene['prepareProjectAppearance']>>|null=null;
  let modelInfo:VRMInspection|null=null;
  try {
    if(await projectEngineHash()!==project.engineModuleSha256)throw new Error('저장한 프로젝트와 현재 엔진 버전이 다릅니다. 동일 버전에서 열거나 원본 MIDI로 새 연주를 준비해 주세요.');
    const midiFile=file('midi')!,midiBytes=new Uint8Array(await midiFile.arrayBuffer()),info=await engine.inspect(midiBytes);
    await engine.validateProjectSource(midiBytes,project.selection,project.plan);
    if(project.rig==='vrm'){
      const loaded=await loadLocalVRM(file('vrm')!);modelInfo=loaded.info;
      try{adapter=new VRMHumanoidRigAdapter(loaded.vrm,`vrm1:${project.assets.find(asset=>asset.kind==='vrm')!.sha256}`);}
      catch(error){VRMUtils.deepDispose(loaded.vrm.scene);throw error;}
    }else adapter=new NativeHumanoidRigAdapter();
    const profile=adapter.setCalibration(project.calibration,project.kit);
    if(JSON.stringify(profile)!==JSON.stringify(project.profile))throw new Error('저장한 체형 측정값과 현재 리그가 다릅니다. 원본 MIDI로 새 연주를 준비해 주세요.');
    const nextPlan=await engine.restorePlan(project.plan,profile,project.kit,false);
    const nextWav=project.audio.source==='wav'?await loadLocalWav(file('wav')!):null;
    const aligned=nextWav?alignWavToPlan(nextWav.buffer,nextPlan,project.audio.offsetSeconds):null;
    const nextBuffer=aligned?.buffer??await synthesis.run(nextPlan);
    const parts=new Map<PieceId,File>();for(const ref of project.assets)if(ref.kind==='drumPart')parts.set(ref.pieceId!,projectFiles.get(ref.sha256)!);
    preparedAppearance=await scene.prepareProjectAppearance(project.appearance.preset,project.appearance.pieces,parts);
    // Every file, setting and Rust plan has passed before replacing the session.
    const restored=await engine.restorePlan(project.plan,profile,project.kit);
    audio?.pause();openPanel('source');scene.setKitLayout(project.kit);scene.setRig(adapter);adapter=null;
    scene.setCalibration(project.calibration);preparedAppearance.apply();preparedAppearance=null;
    plan=restored;bytes=midiBytes;fileName=midiFile.name;preparedSelection=structuredClone(project.selection);arrangement=arrangementOf(restored);sourceReferences=structuredClone(project.assets);
    wav=nextWav;wavOffset=project.audio.offsetSeconds;wavAlignmentInfo=aligned?.info;
    committedPreset=project.appearance.preset;get<HTMLSelectElement>('kit-preset').value=committedPreset;
    get('file-name').textContent=fileName;get('file-detail').textContent=`${(midiBytes.byteLength/1024).toFixed(1)} KiB · 프로젝트에서 복원`;
    fillSelection(info);
    for(const input of document.querySelectorAll<HTMLInputElement>('#tracks input'))input.checked=project.selection.tracks.includes(Number(input.value));
    get<HTMLSelectElement>('channel').value=project.selection.channel===null?'all':String(project.selection.channel);
    get<HTMLSelectElement>('mapping-profile').value=project.selection.mappingProfile!;get<HTMLInputElement>('count-in-75').checked=project.selection.countInNote75!;
    get<HTMLInputElement>('hihat-policy').checked=project.selection.hihatPolicy==='shortOpenClose';get<HTMLInputElement>('hihat-openness').value=String(project.selection.hihatOpenness!*100);
    get<HTMLInputElement>('wav-offset').value=String(wavOffset);get<HTMLInputElement>('volume').value=String(project.audio.volume);
    setTheme(project.view.theme);scene.setBackground(project.view.background);get<HTMLInputElement>('background').value=project.view.background;
    for(const [id,value] of [['character-visible',project.view.character],['skeleton-visible',project.view.skeleton],['model-visible',project.view.model]] as const)get<HTMLInputElement>(id).checked=value;
    scene.setCharacterVisible(project.view.character);scene.setSkeletonVisible(project.view.skeleton);scene.setModelVisible(project.view.model);refreshCharacterDisplay();
    get<HTMLInputElement>('hit-response').checked=project.hitResponse.enabled;get<HTMLInputElement>('hit-strength').value=String(project.hitResponse.strength);responseChosen=true;scene.setHitResponse(project.hitResponse);
    get<HTMLInputElement>('timeline-zoom').value=String(project.view.timelineZoom);timeline.setZoom(project.view.timelineZoom);
    scene.setViewState(project.camera);for(const button of document.querySelectorAll<HTMLButtonElement>('[data-camera]'))button.setAttribute('aria-pressed',String(button.dataset.camera===project.camera.view));
    scene.setPlan(plan);timeline.setPlan(plan);replaceAudio(nextBuffer,project.outputTime);showWavInfo(aligned?.info);showModelInfo(modelInfo);
    get('character-message').textContent=project.rig==='vrm'?'프로젝트 VRM · 같은 원본 파일과 체형 설정 복원':'기본 리그 · 프로젝트 체형 설정 복원';
    get<HTMLInputElement>('seek').max=String(outputDuration(plan));get('duration').textContent=clockText(outputDuration(plan));
    get('metric-duration').textContent=clockText(plan.duration);get('metric-hits').textContent=plan.hits.length.toLocaleString();get('metric-tempos').textContent=String(plan.tempoCount);get('plan-label').textContent='READY';get('stage-empty').hidden=true;
    fitProposal=null;preFitSetup=null;fitBaseline=structuredClone(project.kit);fitState='저장한 배치 복원';poseReview=null;refreshCalibration();refreshTarget();refreshPiece();renderDiagnostics();
    pendingProject=null;get('project-relink').hidden=true;get('project-message').textContent='같은 원본 파일을 확인하고 프로젝트를 복원했습니다. WAV와 자세를 재생·탐색으로 확인하세요.';
  } finally {adapter?.dispose();preparedAppearance?.dispose();}
}
get('project-open').onclick=()=>get<HTMLInputElement>('project-file').click();
get<HTMLInputElement>('project-file').onchange=async event=>{
  const input=event.target as HTMLInputElement,file=input.files?.[0];input.value='';if(!file||busy||exporting||modelBusy)return;
  busy=true;projectBusy=true;controls();audio?.pause();
  try{if(file.size>MAX_PROJECT_BYTES)throw new Error('프로젝트는 32 MiB 이하여야 합니다.');const candidate=parseProject(await file.text());if(await projectEngineHash()!==candidate.engineModuleSha256)throw new Error('저장한 프로젝트와 현재 엔진 버전이 다릅니다. 원본 MIDI를 다시 준비해 주세요.');pendingProject=candidate;await restorePendingProject();}
  catch(error){get('project-message').textContent=error instanceof Error?error.message:String(error);}
  finally{busy=false;projectBusy=false;controls();}
};
get('project-relink').onclick=()=>get<HTMLInputElement>('project-assets').click();
get<HTMLInputElement>('project-assets').onchange=async event=>{
  const input=event.target as HTMLInputElement,files=[...input.files??[]];input.value='';if(!pendingProject||!files.length||busy||exporting||modelBusy)return;
  busy=true;projectBusy=true;controls();
  try {
    const rejected:string[]=[];
    for(const file of files){if(file.size>100*1024*1024){rejected.push(file.name);continue;}const ref=await fileReference('midi',file);if(pendingProject.assets.some(asset=>asset.sha256===ref.sha256&&asset.bytes===ref.bytes))projectFiles.set(ref.sha256,file);else rejected.push(file.name);}
    await restorePendingProject();
    if(rejected.length)get('project-message').textContent+=` 일치하지 않아 제외한 파일: ${rejected.join(', ')}`;
  }catch(error){get('project-message').textContent=error instanceof Error?error.message:String(error);}
  finally{busy=false;projectBusy=false;controls();}
};

get<HTMLInputElement>('hihat-policy').onchange=()=>controls();
get<HTMLSelectElement>('mapping-profile').onchange=()=>{get('mapping-evidence').textContent='매핑을 변경했습니다. 다시 준비하면 선택한 음원 기준의 결과를 확인할 수 있습니다.';controls();};
get('upload').onclick=()=>get<HTMLInputElement>('file').click();
get<HTMLInputElement>('file').onchange=async(event)=>{const input=event.target as HTMLInputElement;const file=input.files?.[0]; if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);}else await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}input.value='';};
get('verification-demo').onclick=()=>{get<HTMLSelectElement>('mapping-profile').value='logicDkdGm';get<HTMLInputElement>('count-in-75').checked=true;get<HTMLInputElement>('hihat-policy').checked=true;get<HTMLInputElement>('hihat-openness').value='20';void loadFile(makeInterpretationDemo(),'Interpretation Check.mid');};
get('demo').onclick=()=>void loadFile(makeDemo(),'Studio Groove.mid');
const dropzone=get('dropzone');
dropzone.ondragover=e=>{e.preventDefault();if(!busy&&!exporting)dropzone.classList.add('dragging');};
dropzone.ondragleave=()=>dropzone.classList.remove('dragging');
dropzone.ondrop=async(e)=>{e.preventDefault();dropzone.classList.remove('dragging');if(busy||exporting||modelBusy)return;const file=e.dataTransfer?.files[0];if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);return;}await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}};
get('cancel-analysis').onclick=()=>{++loadGeneration;++reviewGeneration;engine.cancel();synthesis.cancel();busy=false;showWavInfo();message(reviewRunning?'곡 검수를 취소했습니다.':'준비를 취소했습니다. 기존 연주가 있다면 다시 재생할 수 있습니다.');status(plan?'일시정지':'대기');controls();};
get('replan').onclick=async()=>{if(!bytes||busy||exporting||modelBusy)return;const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status('준비 중','working');try{await prepare(generation,time);}catch(e){if(generation===loadGeneration){message(`${String(e)} 기존 연주는 유지됩니다. 음표 편집과 변경한 매핑·트랙 선택의 호환성을 확인하세요.`,true);status('확인 필요','error');}}finally{if(generation===loadGeneration){busy=false;controls();}}};
get('play').onclick=async()=>{
  if(!audio||!plan)return;
  try{if(audio.playing){audio.pause();status('일시정지');}else{if(audio.currentTime>=outputDuration(plan)-.01)audio.seek(0);await audio.play();status('연주 중','playing');}}
  catch(e){message(`소리를 시작하지 못했습니다. ${String(e)}`,true);}
};
get('reset').onclick=()=>{audio?.pause();audio?.seek(0);updateFrame(0);status('준비 완료','ready');};
get<HTMLInputElement>('seek').oninput=()=>{audio?.seek(Number(get<HTMLInputElement>('seek').value));updateFrame(Number(get<HTMLInputElement>('seek').value));};
get<HTMLInputElement>('volume').oninput=()=>audio?.setVolume(Number(get<HTMLInputElement>('volume').value));
for(const button of document.querySelectorAll<HTMLButtonElement>('[data-camera]'))button.onclick=()=>{scene.setCamera(button.dataset.camera as 'stage'|'front'|'top'|'player');for(const other of document.querySelectorAll('[data-camera]'))other.setAttribute('aria-pressed',String(other===button));};
get<HTMLInputElement>('background').oninput=()=>scene.setBackground(get<HTMLInputElement>('background').value);
get('cancel-export').onclick=()=>exportController?.abort();
get('export').onclick=async()=>{
  if(!plan||!buffer||exporting||modelBusy)return;
  const currentPlan=plan; const currentBuffer=buffer; const restoreTime=audio?.pause()??0;
  exporting=true;exportController=new AbortController();controls();status('영상 만드는 중','working');
  get<HTMLProgressElement>('progress').value=0;get('progress-text').textContent='인코더 확인 중…';
  get('download').hidden=true;
  try {
    const profiles=await probeExport(get<HTMLInputElement>('mute-export').checked, exportController.signal);
    if(exportController.signal.aborted)throw new DOMException('내보내기를 취소했습니다.','AbortError');
    if(!profiles.length)throw new Error('이 브라우저에서 지원하는 영상·오디오 인코더를 찾지 못했습니다. 최신 Chrome 또는 Edge를 사용해 주세요.');
    const profile=profiles[0];
    scene.setExportMode(true);scene.resize(1280,720);
    const result=await exportVideo({canvas:stage,renderFrame:(songTime)=>{const pose=engine.evaluate(songTime);if(pose)scene.applyPose(pose,songTime);scene.render();},plan:currentPlan,audio:currentBuffer,profile,signal:exportController.signal,onProgress:p=>{get<HTMLProgressElement>('progress').value=p;get('progress-text').textContent=`${Math.round(p*100)}% · ${profile.extension.toUpperCase()}`;}});
    if(downloadUrl)URL.revokeObjectURL(downloadUrl);downloadUrl=URL.createObjectURL(result.blob);
    const download=get<HTMLAnchorElement>('download');download.href=downloadUrl;download.download=`${fileName.replace(/\.(mid|midi)$/i,'').replace(/[^\p{L}\p{N} _-]/gu,'_')}-drum.${result.extension}`;download.hidden=false;
    get('export-note').textContent=`${result.extension.toUpperCase()} · ${result.frames} 프레임 · ${result.duration.toFixed(2)}초 · ${profile.audioCodec?(wav?'사용자 WAV 포함':'합성 드럼 오디오 포함'):'선택한 무음 출력'} · ${(result.blob.size/1048576).toFixed(1)} MiB`;
    message('영상이 완성됐습니다. 내보내기 패널에서 다운로드하세요.');status('영상 준비 완료','ready');
  } catch(e){const cancelled=e instanceof DOMException&&e.name==='AbortError';message(cancelled?'영상 추출을 취소했습니다. 다시 재생하거나 내보낼 수 있습니다.':`영상 추출 실패: ${e instanceof Error?e.message:String(e)}`,!cancelled);status(cancelled?'준비 완료':'출력 확인 필요',cancelled?'ready':'error');}
  finally {exporting=false;exportController=null;scene.setExportMode(false);scene.resize();updateFrame(restoreTime);controls();}
};
document.addEventListener('keydown',e=>{if(e.code==='Space'&&!/INPUT|SELECT|BUTTON|TEXTAREA/.test((e.target as HTMLElement).tagName)){e.preventDefault();get<HTMLButtonElement>('play').click();}});
const resizeObserver=new ResizeObserver(()=>{if(!exporting){scene.resize();timeline.resize();}});resizeObserver.observe(get('stage-view'));
let disposed=false;
function tick(){if(disposed)return;if(!exporting){const time=audio?.currentTime??0;if(plan){updateFrame(time);get<HTMLInputElement>('seek').value=String(time);get('current-time').textContent=clockText(time);timeline.setTime(time);const playing=audio?.playing??false;if(get('play').dataset.icon!==(playing?'pause':'play')){setButtonIcon(get('play'),playing?'pause':'play');get('play').setAttribute('aria-label',playing?'일시정지':'재생');}if(!playing&&get('status').classList.contains('playing'))status(time>=outputDuration(plan)-.02?'연주 완료':'일시정지',time>=outputDuration(plan)-.02?'ready':'');}scene.render();}requestAnimationFrame(tick);}tick();
controls();timeline.resize();setCharacterDisplay('skeleton');void autoFitCurrent();
window.addEventListener('beforeunload',()=>{disposed=true;engine.dispose();synthesis.cancel();audio?.dispose();scene.dispose();timeline.dispose();resizeObserver.disconnect();clearTimeout(toastTimer);if(downloadUrl)URL.revokeObjectURL(downloadUrl);});
