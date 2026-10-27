import { characterFiles, chooseCharacterFile } from './character-files';
import { rememberFBXSetup } from './fbx-preferences';
import { setupFBX } from './fbx-setup';
import { FBXHumanoidRigAdapter } from './fbx-rig';
import { loadLocalFBX } from './fbx-import';
import { imageKey, type FBXConfig } from './fbx-config';
import type { HumanoidRigAdapter } from './rig-humanoid';
import {msg, raw, bindText, boundText, bindAttribute, bindStaticTranslations, joinMessages, getLocale, setLocale, onLocaleChange} from './i18n';
import {engineMessage} from './engine-messages';
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
import { CHARACTER_PRESETS, characterAssetUrl, loadCharacterPreset, type CharacterPreset } from './character-presets';
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
import { DEMO_CATALOG, loadDemo, type DemoId } from './demo-catalog';
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo, type PieceId, type KitPiece, type KitFitProposal, type PoseInspection, type RigProfile, type Vec3 } from './types';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = studioTemplate;
bindStaticTranslations(app);
const description = document.querySelector<HTMLMetaElement>('meta[name="description"]');
if (description) bindAttribute(description, 'content', msg('Turn drum MIDI into 3D performances and videos. Files stay in your browser.', '드럼 MIDI를 3D 연주와 영상으로. 파일은 브라우저에서만 처리됩니다.'));
function uiOption(text: string, value: string) { const option = new Option('', value); bindText(option, text); return option; }

function uiError(error: unknown) { return engineMessage(error instanceof Error ? error.message : String(error)); }
const get = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
setButtonIcon(get('reset'), 'reset');
setButtonIcon(get('play'), 'play');
setButtonIcon(get('close-panel'), 'close');
for (const id of ['upload', 'start-upload']) setButtonIcon(get(id), 'upload', msg('Import MIDI…', 'MIDI 불러오기…'));
setButtonIcon(get('export'), 'download', msg("Export video", "영상 추출"));
const stage = get<HTMLCanvasElement>('stage');
const engine = new EngineClient();
const synthesis = new SynthesisClient();
let scene: DrumScene;
try { scene = new DrumScene(stage); } catch { bindText(get('message'), msg("Unable to start the 3D view. Use a recent browser with WebGL support.", "3D 화면을 시작할 수 없습니다. WebGL을 지원하는 최신 브라우저를 사용해 주세요.")); throw new Error('WebGL unavailable'); }
const defaultKit = scene.getKitLayout();
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
let fitState = msg("Default layout · not fitted to this character", "기본 배치 · 체형별 배치 보정 전");
let poseReview: PoseReview | null = null;
let reviewRunning = false;
let reviewGeneration = 0;
let projectBusy = false;
const projectFiles = new Map<string,File>();
let sourceReferences: ProjectAsset[] = [];
let currentFBX: FBXConfig | undefined;
const characterAssetKinds = new Set<ProjectAsset['kind']>(['vrm','fbx','fbxTexture']);
let pendingProject: DrumProject | null = null;
let preparedSelection: DrumProject['selection'] | null = null;
let arrangement:Arrangement={omittedSourceEventIds:[],handOverrides:{}};
let busy = false;
let exporting = false;
let loadGeneration = 0;
let demoController: AbortController | null = null;
let exportController: AbortController | null = null;
let downloadUrl: string | null = null;
let fileName = 'drum-performance';
async function rememberProjectFile(kind:ProjectAsset['kind'],file:File,pieceId?:PieceId) {
  const reference=await fileReference(kind,file,pieceId);commitProjectFile(reference,file);return reference;
}
function commitProjectFile(reference:ProjectAsset,file:File) {
  const {kind,pieceId}=reference;projectFiles.set(reference.sha256,file);
  sourceReferences=sourceReferences.filter(old=>old.kind!==kind||(kind==='drumPart'&&old.pieceId!==pieceId)||(kind==='fbxTexture'&&imageKey(old.name)!==imageKey(reference.name)));sourceReferences.push(reference);return reference;
}
type Panel = 'character' | 'drums' | 'source' | 'analysis' | 'export';
let activePanel: Panel | null = null;
let modelBusy = false;
let characterDownload: AbortController | null = null;
let committedCharacter = 'native';
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
  if(!id){get<HTMLSelectElement>('arrangement-event').replaceChildren(uiOption(msg("None selected", "선택 없음"),''));bindText(get('arrangement-detail'), '');get<HTMLInputElement>('arrangement-omit').checked=false;get<HTMLSelectElement>('arrangement-hand').value='auto';}
  else if(plan&&!busy&&!exporting&&!modelBusy){selectArrangementEvent(id);openPanel('analysis');get<HTMLDetailsElement>('arrangement-editor').open=true;}
}});
const panelTitles: Record<Panel, string> = {character:msg("Character settings", "캐릭터 설정"),drums:msg("Drum settings", "드럼 설정"),source:msg("Files / Audio", "파일 · 오디오"),analysis:msg("Analysis", "분석"),export:msg("Export", "내보내기")};
function openPanel(next: Panel | null) {
  if (exporting || modelBusy) return;
  activePanel = next;
  get('inspector').hidden = !next;
  bindText(get('panel-title'), next ? panelTitles[next] : '');
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
  bindAttribute(get('theme'), 'aria-label', next==='light'?msg("Switch to dark mode", "다크 모드로 전환"):msg("Switch to light mode", "라이트 모드로 전환"));
  try { localStorage.setItem('drum-theme',next); } catch { /* Storage is optional. */ }
}
function selectedPiece() { return get<HTMLSelectElement>('piece').value as PieceId; }
function refreshPiece() {
  const visual = scene.getPieceVisual(selectedPiece());
  get<HTMLSelectElement>('piece-model').value = visual.source==='library'||visual.source==='imported'?'asset':visual.model;
  get<HTMLInputElement>('piece-finish').value = visual.finish;
  get<HTMLInputElement>('piece-scale').value = String(visual.scale);
  bindText(get('piece-scale-value'), `${Math.round(visual.scale*100)}%`);
  get<HTMLInputElement>('piece-rotation').value=String(visual.rotation);bindText(get('piece-rotation-value'), `${visual.rotation}°`);
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
    if(generation===idlePoseGeneration&&!plan)bindText(get('calibration-message'), msg("Ready pose needs attention · {0}", "기본 자세 확인 필요 · {0}", uiError(error)));
  }
}
function invalidateVideo() {
  if(downloadUrl)URL.revokeObjectURL(downloadUrl); downloadUrl=null; get('download').hidden=true;
  bindText(get('export-note'), msg("1280 × 720 · 30 fps · Export again with the current settings. Includes 0.5 s preparation and 0.25 s tail.", "1280 × 720 · 30 fps · 현재 설정으로 다시 추출하세요. 시작 준비 0.5초 · 끝 여운 0.25초 포함."));
}
async function applyCalibration(seat: HumanoidCalibration, kit: KitPiece[], feedbackId: string, automatic=false) {
  if(busy||exporting||(modelBusy&&!automatic))return false;
  const oldSeat=scene.getCalibration(), oldKit=scene.getKitLayout();
  let applied=false,sceneChanged=false;
  let staged:Awaited<ReturnType<EngineClient['prepareSetup']>>|null=null;
  const time=audio?.pause()??0;
  if(!automatic){modelBusy=true;controls();}
  try {
    kit=scene.measureKitLayout(kit);
    const profile=scene.measureCalibration(seat,kit);
    staged=await prepareSetupInBackground(profile,kit);
    sceneChanged=true;scene.setKitLayout(kit);scene.setCalibration(seat);await scene.whenEquipmentReady();
    const next=staged.commit();
    if(next&&plan){plan=next;scene.setPlan(plan);timeline.setPlan(plan);renderDiagnostics();updateFrame(time);}
    if(!automatic){fitBaseline=structuredClone(kit);preFitSetup=null;}
    fitProposal=null;poseReview=null;fitState=msg("Manual calibration · current layout", "수동 보정 · 현재 배치");refreshFitState();invalidateVideo();
    const unavailable=plan?.hits.filter(h=>h.limb===null).length??0;
    bindText(get(feedbackId), msg("Calibration applied{0}", "보정 적용 완료{0}", unavailable?msg(" · {0} unassigned hits (see Analysis)", " · 표현하지 못한 타격 {0}개 (분석 확인)", unavailable):''));
    applied=true;
  } catch(error) {
    if(sceneChanged){scene.setKitLayout(oldKit);scene.setCalibration(oldSeat);updateFrame(time);}
    bindText(get(feedbackId), uiError(error));
  }finally{
    staged?.dispose();if(!automatic){modelBusy=false;controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));}
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
  bindText(get('fit-state'), fitState);
  bindText(get('setup-summary'), msg("Layout: {0}", "배치: {0}", fitState));
  const capability=fitProposal?.capabilities;
  get('fit-capabilities').hidden=!capability;
  const list=get('fit-capability-list');list.replaceChildren();
  if(capability){
    bindText(get('fit-capability-summary'), msg("{0} · {1} · {2} single-hand checks · {4} of {3} two-hand pairs found. This does not test skin or clothing collisions.", "{0} · {1} · 한 손 {2}회 · 양손 {3}조합 중 {4}조합 확인. 피부·의상 충돌 검사는 아닙니다.", fitProposal?.complete?msg("Applied layout", "적용 배치"):msg("Unapplied fit candidate", "미적용 맞춤 후보"), capability.geometrySource==='mesh-estimate'?msg("Mesh-based approximation", "메시 기반 근사"):msg("Skeleton-based estimate", "뼈대 기반 추정"), capability.handTargetChecks, capability.pairChecks, capability.availablePairs));
    for(const pair of capability.pairs){const row=document.createElement('li');bindText(row, msg("L {0} / R {1} · {2}", "L {0} / R {1} · {2}", pair.leftPieceId, pair.rightPieceId, {available:msg("Feasible pose found", "가능한 자세 확인"),'not-found':msg("Not found within search bounds", "탐색 범위에서 찾지 못함"),unmeasured:msg("Insufficient measurements", "측정 정보 부족")}[pair.status]));list.append(row);}
  }else bindText(get('fit-details'), '');

  controls();
}
let toastTimer: ReturnType<typeof setTimeout> | undefined;
function notify(text: string) {
  clearTimeout(toastTimer);bindText(get('toast-message'), text);get('toast').hidden=false;
  toastTimer=setTimeout(()=>{if(!get('toast').contains(document.activeElement))get('toast').hidden=true;},6000);
}
get('toast-dismiss').onclick=()=>{clearTimeout(toastTimer);get('toast').hidden=true;};
for(const button of document.querySelectorAll<HTMLButtonElement>('[data-open-analysis]'))button.onclick=()=>openPanel('analysis');
get('analysis-source').onclick=()=>{openPanel('source');get<HTMLDetailsElement>('midi-settings').open=true;get<HTMLSelectElement>('mapping-profile').focus();};
function showFitResult(result: Awaited<ReturnType<typeof prepareAutomaticFit>>) {
  fitProposal=result.proposal;fitState=result.applied?msg("Automatic body fit applied", "체형 자동 맞춤 적용"):msg("Automatic fit needs attention", "자동 맞춤 확인 필요");
  bindText(get('calibration-message'), result.applied?'':result.message);
  bindText(get('fit-details'), result.message);
  refreshFitState();
  if(result.applied)notify(msg("The drum layout has been fitted to the character.", "캐릭터에 맞게 드럼 배치를 조정했습니다."));
  else message(msg("Automatic fitting could not meet the constraints. See Character → Manual adjustment or Analysis.", "자동 맞춤 조건을 만족하지 못했습니다. 캐릭터의 수동 보정 또는 분석을 확인하세요."),true);
}
async function fitInBackground(profile: RigProfile, kit: readonly KitPiece[]) {
  fitRunning=true;controls();status(msg("Fitting to character…", "체형에 맞추는 중…"),'working');
  try { return await engine.fitKit(profile,kit); }
  finally { fitRunning=false;controls(); }
}
async function prepareSetupInBackground(profile: RigProfile, kit: readonly KitPiece[]) {
  setupRunning=true;controls();status(msg("Preparing performance poses…", "연주 자세를 준비하는 중…"),'working');
  try{return await engine.prepareSetup(profile,kit);}
  finally{setupRunning=false;controls();}
}
get('cancel-fit').onclick=()=>{if(fitRunning)engine.cancelFit();if(setupRunning)engine.cancel();};
async function autoFitCurrent() {
  if(busy||exporting||modelBusy)return;
  audio?.pause();modelBusy=true;controls();bindText(get('calibration-message'), msg("Fitting to character…", "체형에 맞추는 중…"));
  const previous={seat:scene.getCalibration(),kit:scene.getKitLayout()};
  try {
    const seat={...scene.getAutomaticCalibration(),palmOrientation:previous.seat.palmOrientation};
    const result=await prepareAutomaticFit(scene.measureCalibration(seat,fitBaseline),fitBaseline,previous.kit,fitInBackground);
    if(result.applied){
      if(!await applyCalibration(seat,result.kit,'calibration-message',true))return;
      preFitSetup=previous;
    }
    showFitResult(result);await refreshIdlePose();
  }catch(error){bindText(get('calibration-message'), uiError(error));}
  finally{modelBusy=false;controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));}
}
get('kit-fit').onclick=()=>void autoFitCurrent();
get('kit-fit-restore').onclick=async()=>{
  if(!preFitSetup||busy||exporting||modelBusy)return;
  const previous=preFitSetup;
  if(await applyCalibration(previous.seat,previous.kit,'calibration-message')){preFitSetup=null;fitState=msg("Restored layout before fitting", "맞춤 전 배치 복원");refreshFitState();notify(msg("Restored the layout from before automatic fitting.", "자동 맞춤 전 배치로 되돌렸습니다."));}
};
get('kit-layout-reset').onclick=()=>void applyCalibration(scene.getCalibration(),defaultKit,'calibration-message');
get('seat-auto').onclick=()=>void applyCalibration(scene.getAutomaticCalibration(),scene.getKitLayout(),'calibration-message');
get<HTMLSelectElement>('target-piece').onchange=refreshTarget;
get('target-apply').onclick=()=>{
  const fields=['x','y','z'].map(axis=>get<HTMLInputElement>(`target-${axis}`));
  if(fields.some(field=>!field.reportValidity()))return;
  try{applyCalibration(scene.getCalibration(),moveKitTarget(scene.getKitLayout(),get<HTMLSelectElement>('target-piece').value as PieceId,fields.map(field=>field.valueAsNumber/100) as Vec3),'target-message');}
  catch(error){bindText(get('target-message'), uiError(error));refreshTarget();}
};
get('target-reset').onclick=()=>{
  const id=get<HTMLSelectElement>('target-piece').value as PieceId;
  applyCalibration(scene.getCalibration(),moveKitTarget(scene.getKitLayout(),id,defaultKit.find(p=>p.id===id)!.position),'target-message');
};
for (const button of document.querySelectorAll<HTMLButtonElement>('[data-panel]')) button.onclick=()=>openPanel(activePanel===button.dataset.panel?null:button.dataset.panel as Panel);
get('close-panel').onclick=()=>{const previous=activePanel;openPanel(null);document.querySelector<HTMLButtonElement>(`[data-panel="${previous}"]`)?.focus();};
function refreshLanguageControl() {
  get('language').textContent = getLocale() === 'en' ? '한국어' : 'EN';
  get('language').setAttribute('lang', getLocale() === 'en' ? 'ko' : 'en');
  bindAttribute(get('language'), 'aria-label', msg('Switch to Korean', '영어로 전환'));
  bindAttribute(get('language'), 'title', msg('Switch to Korean', '영어로 전환'));
}
onLocaleChange(refreshLanguageControl);
get('language').onclick = () => setLocale(getLocale() === 'en' ? 'ko' : 'en');
refreshLanguageControl();
get('theme').onclick=()=>setTheme(theme==='light'?'dark':'light');
get('start-upload').onclick=()=>{openPanel('source');get<HTMLInputElement>('file').click();};
get('start-demo').onclick=()=>{openPanel('source');void loadSelectedDemo('studio');};
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
function showModelInfo(info: VRMInspection | null, preset?: CharacterPreset) {
  const host = get('character-info'); host.replaceChildren(); if (!info) return;
  const name=document.createElement('strong');bindText(name, raw(info.name));host.append(name);
  if(preset){const credit=document.createElement('p');credit.className='hint';bindText(credit,msg('By {0}', '제작: {0}',raw(preset.creator)));host.append(credit);}
  const metadata=document.createElement('details'),label=document.createElement('summary');bindText(label, msg("Model details / terms", "모델 정보 · 이용 조건"));metadata.append(label);host.append(metadata);
  for (const text of [msg("VRM {0} · model {1} · required bones {2}/15", "VRM {0} · 모델 {1} · 필수 본 {2}/15", raw(info.version), raw(info.modelVersion), info.required), msg("Finger bones L {0}/15 · R {1}/15", "손가락 본 L {0}/15 · R {1}/15", info.fingers.left, info.fingers.right), msg("Author: {0}", "제작자: {0}", info.authors.length ? raw(info.authors.join(', ')) : msg("Not specified", "미기재"))]) {
    const p = document.createElement('p'); p.className='hint'; bindText(p, text); metadata.append(p);
  }
  if(info.fingers.left<15 || info.fingers.right<15) { const p=document.createElement('p'); p.className='hint'; bindText(p, msg("Some finger bones are missing, which limits the grip pose. Stick attachment and performance remain available.", "일부 손가락 본이 없어 그립 표현이 제한됩니다. 스틱 부착과 연주는 유지됩니다.")); host.append(p); }
  for (const warning of info.metadataWarnings) { const p=document.createElement('p'); p.className='hint'; bindText(p, warning); metadata.append(p); }
  const details = document.createElement('details'), summary = document.createElement('summary'); bindText(summary, msg("Model terms", "모델 이용 조건")); details.append(summary);
  for (const [label, value] of info.terms) { const p=document.createElement('p'); p.className='hint'; bindText(p, msg('{0}: {1}', '{0}: {1}', label, raw(value))); details.append(p); }
  for(const reference of info.references) { const p=document.createElement('p'); p.className='hint'; bindText(p, msg("Reference: {0}", "참고: {0}", raw(reference))); details.append(p); }
  metadata.append(details);
  if(preset){
    const sources=document.createElement('p');sources.className='hint';
    for(const [label,url] of [[msg('Source', '출처'),preset.source],[msg('License', '라이선스'),preset.licenseUrl],[msg('Distribution record', '배포 기록'),characterAssetUrl(preset.provenanceFile)]] as const){
      if(sources.childNodes.length)sources.append(' · ');
      const link=document.createElement('a');bindText(link,label);link.href=url;link.target='_blank';link.rel='noreferrer';sources.append(link);
    }
    metadata.append(sources);
  }
}
async function adoptRig(adapter: HumanoidRigAdapter) {
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
function refreshCharacterPreset() {
  const select=get<HTMLSelectElement>('character-preset');
  select.replaceChildren(uiOption(msg('Default skeleton', '기본 스켈레톤'),'native'),...CHARACTER_PRESETS.map(preset=>uiOption(preset.label,preset.id)));
  if(committedCharacter==='local')select.append(uiOption(msg('Local character', '로컬 캐릭터'),'local'));
  select.value=committedCharacter;
}
refreshCharacterPreset();
async function importCharacter(fileOrPreset: File | CharacterPreset) {
  if(busy||exporting||modelBusy){refreshCharacterPreset();return;}
  audio?.pause(); modelBusy=true;
  const preset=fileOrPreset instanceof File?undefined:fileOrPreset;
  characterDownload=preset?new AbortController():null;
  controls();bindText(get('character-message'), preset?msg('Downloading character…', '캐릭터를 내려받고 있습니다…'):msg("Inspecting VRM…", "VRM을 검사하고 있습니다…"));
  let candidate: Awaited<ReturnType<typeof loadLocalVRM>> | null = null;
  let adapter: VRMHumanoidRigAdapter | null = null;
  try {
    const file=preset?await loadCharacterPreset(preset.id,characterDownload!.signal):fileOrPreset as File;
    characterDownload=null;controls();bindText(get('character-message'), msg("Inspecting VRM…", "VRM을 검사하고 있습니다…"));
    candidate=await loadLocalVRM(file);
    const reference=await fileReference('vrm',file),info=candidate.info;
    adapter=new VRMHumanoidRigAdapter(candidate.vrm,`vrm1:${reference.sha256}`); await adoptRig(adapter);
    candidate=null; adapter=null;
    const matchedPreset=CHARACTER_PRESETS.find(entry=>entry.sha256===reference.sha256);
    committedCharacter=matchedPreset?.id??'local';
    currentFBX=undefined;sourceReferences=sourceReferences.filter(ref=>!characterAssetKinds.has(ref.kind));
    commitProjectFile(reference,file);showModelInfo(info,matchedPreset);
    bindText(get('character-message'), raw(file.name));setCharacterDisplay('model');
  } catch(error) { if(adapter)adapter.dispose();else if(candidate)VRMUtils.deepDispose(candidate.vrm.scene); bindText(get('character-message'), characterDownload?.signal.aborted?msg('Character download canceled.', '캐릭터 다운로드를 취소했습니다.'):uiError(error)); }
  finally{characterDownload=null;modelBusy=false;refreshCharacterPreset();controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));}
}
function showFBXInfo(name:string) {
  const host=get('character-info');host.replaceChildren();
  const p=document.createElement('p');bindText(p,msg('FBX · {0} · mapped humanoid. Original model terms apply; toon shaders, physics and embedded animations are not reproduced.', 'FBX · {0} · 연결된 휴머노이드. 모델의 원본 이용 조건을 확인하세요. 툰 셰이더·물리·내장 애니메이션은 재현하지 않습니다.',raw(name)));host.append(p);
}
async function importFBXCharacter(file:File,images:File[]) {
  if(busy||exporting||modelBusy)return;
  audio?.pause();modelBusy=true;controls();
  let candidate:Awaited<ReturnType<typeof setupFBX>>=null,adapter:FBXHumanoidRigAdapter|null=null;
  try {
    candidate=await setupFBX(file,images);
    if(!candidate){bindText(get('character-message'),msg('Character import canceled.', '캐릭터 가져오기를 취소했습니다.'));return;}
    const reference=await fileReference('fbx',file),textureReferences=await Promise.all(candidate.loaded.images.map(image=>fileReference('fbxTexture',image)));
    await candidate.loaded.applyTextures(candidate.config);
    adapter=new FBXHumanoidRigAdapter(candidate.loaded.scene,candidate.config,`fbx:${reference.sha256}`,candidate.loaded.release);
    await adoptRig(adapter);
    rememberFBXSetup(reference.sha256,candidate.config);
    currentFBX=structuredClone(candidate.config);sourceReferences=sourceReferences.filter(ref=>!characterAssetKinds.has(ref.kind));commitProjectFile(reference,file);
    textureReferences.forEach((ref,index)=>commitProjectFile(ref,candidate!.loaded.images[index]));
    candidate=null;adapter=null;committedCharacter='local';showFBXInfo(file.name);bindText(get('character-message'),raw(file.name));setCharacterDisplay('model');
  }catch(error){if(adapter)adapter.dispose();else candidate?.loaded.dispose();bindText(get('character-message'),uiError(error));}
  finally{modelBusy=false;refreshCharacterPreset();controls();status(plan?msg('Paused','일시정지'):msg('Idle','대기'));}
}
get('character-import').onclick=()=>get<HTMLInputElement>('character-file').click();
get('character-import-folder').onclick=()=>get<HTMLInputElement>('character-folder').click();
const selectCharacterFiles = async (event: Event) => {
  const input=event.target as HTMLInputElement,files=[...input.files??[]];input.value='';
  if(!files.length||busy||exporting||modelBusy)return;
  const {models,images}=characterFiles(files);
  if(!models.length){bindText(get('character-message'),msg('No VRM or FBX found. Select an extracted character folder.', 'VRM 또는 FBX가 없습니다. 압축을 푼 캐릭터 폴더를 선택하세요.'));return;}
  const file=await chooseCharacterFile(models);if(!file)return;
  if(/\.fbx$/i.test(file.name))void importFBXCharacter(file,images);else void importCharacter(file);
};
get<HTMLInputElement>('character-file').onchange=selectCharacterFiles;
get<HTMLInputElement>('character-folder').onchange=selectCharacterFiles;
get('character-cancel').onclick=()=>characterDownload?.abort();
async function resetCharacter(){
  if(busy||exporting||modelBusy)return;audio?.pause();modelBusy=true;controls();
  const adapter=new NativeHumanoidRigAdapter();let adopted=false;
  try{await adoptRig(adapter);adopted=true;committedCharacter='native';currentFBX=undefined;sourceReferences=sourceReferences.filter(ref=>!characterAssetKinds.has(ref.kind));showModelInfo(null);bindText(get('character-message'), msg("Default skeleton", "기본 스켈레톤"));setCharacterDisplay('skeleton');}
  catch(error){if(!adopted)adapter.dispose();bindText(get('character-message'), uiError(error));}
  finally{modelBusy=false;refreshCharacterPreset();controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));}
}
get<HTMLSelectElement>('character-preset').onchange=e=>{
  const id=(e.target as HTMLSelectElement).value;
  if(id===committedCharacter)return;
  if(id==='native')void resetCharacter();
  else {const preset=CHARACTER_PRESETS.find(entry=>entry.id===id);if(preset)void importCharacter(preset);else refreshCharacterPreset();}
};
async function synchronizeEquipmentAppearance() {
  await scene.whenEquipmentReady();
  const measured=scene.getKitLayout();
  const baseline=scene.measureKitLayout(fitBaseline);
  const seat={...scene.rig.getAutomaticCalibration(baseline),palmOrientation:scene.getCalibration().palmOrientation};
  const result=await prepareAutomaticFit(scene.measureCalibration(seat,baseline),baseline,measured,fitInBackground);
  const chosenSeat=result.applied?seat:scene.getCalibration();
  if(!await applyCalibration(chosenSeat,result.kit,'model-message',true))throw new Error(msg("Could not prepare the performance with the new instrument dimensions.", "새 악기 치수로 연주를 다시 준비하지 못했습니다."));
  await scene.whenEquipmentReady();
  fitBaseline=structuredClone(baseline);preFitSetup=null;
  showFitResult(result);await refreshIdlePose();
  const unresolved=scene.getEquipmentDiagnostics();
  if(unresolved.length)message(msg("Instrument/support layout needs attention: {0}", "악기·받침대 배치 확인 필요: {0}", unresolved.join(', ')),true);
}
async function changeEquipment(change:()=>void|Promise<unknown>, commit:()=>void=()=>{}) {
  if(busy||exporting||modelBusy)return false;
  audio?.pause();modelBusy=true;controls();
  const previous={kit:scene.getKitLayout(),seat:scene.getCalibration(),baseline:fitBaseline,undo:preFitSetup,proposal:fitProposal,state:fitState};
  let rollback:Awaited<ReturnType<DrumScene['prepareProjectAppearance']>>|null=null,changed=false;
  try {
    const parts=new Map<PieceId,File>();for(const ref of sourceReferences)if(ref.kind==='drumPart'){const file=projectFiles.get(ref.sha256);if(file)parts.set(ref.pieceId!,file);}
    rollback=await scene.prepareProjectAppearance(committedPreset as KitPresetId,previous.kit.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).map(piece=>({id:piece.id,settings:scene.getPieceVisual(piece.id)})),parts);
    changed=true;await change();await synchronizeEquipmentAppearance();commit();refreshPiece();return true;
  }
  catch(error){
    if(changed&&rollback){rollback.apply(previous.kit);scene.setCalibration(previous.seat);await scene.whenEquipmentReady();fitBaseline=previous.baseline;preFitSetup=previous.undo;fitProposal=previous.proposal;fitState=previous.state;refreshFitState();refreshCalibration();refreshTarget();refreshPiece();if(plan)updateFrame(audio?.currentTime??0);else await refreshIdlePose();}
    bindText(get('model-message'), uiError(error));return false;
  }
  finally{rollback?.dispose();modelBusy=false;controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));}
}
get<HTMLSelectElement>('kit-preset').onchange=async e=>{
  const input=e.target as HTMLSelectElement,preset=input.value as KitPresetId;
  bindText(get('model-message'), msg("Preparing drum models…", "드럼 모델을 준비하고 있습니다…"));
  if(await changeEquipment(()=>scene.setKitPreset(preset),()=>{committedPreset=preset;sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart');}))bindText(get('model-message'), ({library:'Innerscene · CC0',batoski:'Batoski · CC BY 3.0',smoj:'smoj · CC BY 3.0'} as Record<string,string>)[preset] ?? msg("Procedural geometry · appearance can be changed per piece.", "자체 생성 도형 · 부품별로 외형을 변경할 수 있습니다."));
  input.value=committedPreset;
};
get<HTMLSelectElement>('piece').onchange=()=>{scene.selectPiece(selectedPiece());get<HTMLSelectElement>('target-piece').value=selectedPiece();refreshPiece();refreshTarget();};
scene.onPieceSelect=id=>{if(!id)return;get<HTMLSelectElement>('piece').value=id==='openHat'||id==='pedalHat'?'hihat':id;get<HTMLSelectElement>('target-piece').value=id==='openHat'?'hihat':id;refreshPiece();refreshTarget();};
get<HTMLSelectElement>('piece-model').onchange=e=>{const id=selectedPiece(),model=(e.target as HTMLSelectElement).value as 'standard'|'deep'|'compact';void changeEquipment(()=>scene.setPieceVisual(id,{model}),()=>{sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart'||ref.pieceId!==id);});};
get<HTMLInputElement>('piece-finish').oninput=e=>scene.setPieceVisual(selectedPiece(),{finish:(e.target as HTMLInputElement).value});
get<HTMLInputElement>('piece-scale').oninput=e=>{bindText(get('piece-scale-value'), `${Math.round(Number((e.target as HTMLInputElement).value)*100)}%`);};
get<HTMLInputElement>('piece-scale').onchange=e=>void changeEquipment(()=>scene.setPieceVisual(selectedPiece(),{scale:Number((e.target as HTMLInputElement).value)}));
get('piece-reset').onclick=()=>{const id=selectedPiece();void changeEquipment(()=>scene.resetPieceVisual(id),()=>{sourceReferences=sourceReferences.filter(ref=>ref.kind!=='drumPart'||ref.pieceId!==id);});};
get<HTMLInputElement>('piece-rotation').oninput=e=>{bindText(get('piece-rotation-value'), `${(e.target as HTMLInputElement).value}°`);};
get<HTMLInputElement>('piece-rotation').onchange=e=>void changeEquipment(()=>scene.setPieceVisual(selectedPiece(),{rotation:Number((e.target as HTMLInputElement).value)}));
get('piece-import').onclick=()=>get<HTMLInputElement>('model-file').click();
get<HTMLInputElement>('model-file').onchange=async e=>{
  const input=e.target as HTMLInputElement;const file=input.files?.[0];input.value='';if(!file)return;
  bindText(get('model-message'), msg("Loading GLB…", "GLB를 불러오는 중…"));const id=selectedPiece();let reference:ProjectAsset;
  if(await changeEquipment(async()=>{reference=await fileReference('drumPart',file,id);await scene.loadPieceModel(id,file);},()=>{commitProjectFile(reference,file);}))bindText(get('model-message'), msg("{0} · automatically scaled (check the strike target)", "{0} · 자동 크기 맞춤 (타점 확인 필요)", raw(file.name)));
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
function status(text: string, kind = '') { get('status').className = `status ${kind}`; bindText(get('status'), text); }
function message(text: string, error = false) { const node = get('message'); bindText(node, text); node.classList.toggle('error', error); }
function controls() {
  for (const id of ['play','reset','seek','export']) (get(id) as HTMLButtonElement).disabled = !plan || busy || exporting || modelBusy;
  for (const id of ['upload','demo-select','demo-load','replan','channel','mapping-profile','wav-import','wav-remove','wav-offset','wav-apply','mute-export','background']) (get(id) as HTMLInputElement).disabled = busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting || activePanel==='drums';
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-panel], #close-panel, #theme, #character-visible')) el.disabled=exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-drums input, #panel-drums select, #panel-drums button')) el.disabled=busy || exporting || modelBusy;
  for (const id of ['start-upload','start-demo']) get<HTMLButtonElement>(id).disabled=busy || exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-character input, #panel-character select, #panel-character button')) el.disabled=busy || exporting || modelBusy;
  get<HTMLButtonElement>('character-cancel').disabled=!characterDownload;get('character-cancel').hidden=!characterDownload;
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
  bindText(get('cancel-fit'), setupRunning?msg("Cancel preparation", "준비 취소"):msg("Cancel fitting", "맞춤 취소"));
  get('export-progress').hidden = !exporting;
}
function updateFrame(time: number) {
  if (!plan) return;
  const pose = engine.evaluate(time - PRE_ROLL);
  if (pose) scene.applyPose(pose, time - PRE_ROLL);
  if (activePanel==='analysis') {
    const samples=scene.getRigContactSamples();
    bindText(get('contact-info'), joinMessages(samples.map(s=>msg("{0} {1} mm{2}", "{0} {1} mm{2}", [msg("L stick", "L 스틱"),msg("R stick", "R 스틱"),msg("L foot", "L 발"),msg("R foot", "R 발")][s.limb], (s.errorMeters*1000).toFixed(2), s.wristSwingDegrees===undefined?'':msg(" · wrist {0}°", " · 손목 {0}°", s.wristSwingDegrees.toFixed(1))))));
  }
}
function fillSelection(info: MidiInfo) {
  const list = get('tracks'); list.replaceChildren();
  for (const track of info.tracks.filter(t=>t.noteCount>0)) {
    const label = document.createElement('label'); label.className = 'track-option';
    const input = document.createElement('input'); input.type = 'checkbox'; input.value = String(track.index); input.checked = info.suggestedTracks.includes(track.index);
    const name = document.createElement('span'); bindText(name, track.name ? raw(track.name) : msg("Track {0}", "트랙 {0}", track.index+1));
    const count = document.createElement('small'); bindText(count, msg("{0} notes", "음표 {0}개", track.noteCount));
    label.append(input,name,count); list.append(label);
  }
  const select = get<HTMLSelectElement>('channel'); select.replaceChildren(uiOption(msg("All channels", "전체 채널"),'all'));
  const channels = [...new Set(info.tracks.flatMap(t=>t.channels))].sort((a,b)=>a-b);
  channels.forEach(c=>select.add(uiOption(msg("{0}{1}", "{0}{1}", c+1, c===9?msg(" · GM drums", " · GM 드럼"):''),String(c))));
  if (channels.includes(9)) select.value = '9';
  get('selection').hidden = false;
}
async function prepare(generation: number,restoreTime=0) {
  const tracks = [...document.querySelectorAll<HTMLInputElement>('#tracks input:checked')].map(el=>Number(el.value));
  if (!tracks.length) throw new Error(msg("Select at least one track to perform.", "연주할 트랙을 하나 이상 선택해 주세요."));
  const selected = get<HTMLSelectElement>('channel').value;
  if(!get<HTMLInputElement>('hihat-openness').reportValidity())throw new Error(msg("Hi-hat openness must be between 5 and 100%.", "하이햇 열림은 5–100% 범위여야 합니다."));
  const selection:DrumProject['selection']={tracks,channel:selected==='all'?null:Number(selected),mappingProfile:get<HTMLSelectElement>('mapping-profile').value as 'gm'|'logicDkdGm',countInNote75:get<HTMLInputElement>('count-in-75').checked,hihatPolicy:get<HTMLInputElement>('hihat-policy').checked?'shortOpenClose':'literal',hihatOpenness:get<HTMLInputElement>('hihat-openness').valueAsNumber/100,...structuredClone(arrangement)};
  return prepareSelection(bytes!,selection,generation,restoreTime);
}
async function prepareSelection(data: Uint8Array, selection: DrumProject['selection'], generation: number, restoreTime=0) {
  message(msg("Preparing strikes and limb motion…", "타격과 손발 동작을 준비하고 있습니다…"));
  const prepared=await preparePerformance({candidate:()=>engine.plan(data,selection,false),
    audio:async candidate=>{message(wav?msg("Aligning WAV to the performance…", "WAV를 연주 시간에 맞추고 있습니다…"):msg("Preparing synthesized drum audio…", "합성 드럼 오디오를 준비하고 있습니다…"));
      const aligned=wav?alignWavToPlan(wav.buffer,candidate,wavOffset):null;
      return {buffer:aligned?.buffer??await synthesis.run(candidate),alignment:aligned?.info};},
    commit:candidate=>engine.restorePlan(candidate,scene.getRigProfile(),scene.getKitLayout()),current:()=>generation===loadGeneration});
  if(!prepared)return false;
  plan = prepared.plan;preparedSelection=selection;arrangement=arrangementOf(plan);
  if(prepared.audio.alignment)showWavInfo(prepared.audio.alignment);
  buffer = prepared.audio.buffer;
  audio?.dispose(); audio = new DrumAudio(buffer); audio.setVolume(Number(get<HTMLInputElement>('volume').value));
  scene.setPlan(plan);audio.seek(restoreTime);updateFrame(audio.currentTime);invalidateVideo();
  const total = outputDuration(plan);
  get<HTMLInputElement>('seek').max = String(total); get<HTMLInputElement>('seek').value = String(audio.currentTime);
  bindText(get('duration'), clockText(total)); timeline.setPlan(plan);
  bindText(get('metric-duration'), clockText(plan.duration)); bindText(get('metric-hits'), plan.hits.length.toLocaleString()); bindText(get('metric-tempos'), String(plan.tempoCount));
  bindText(get('plan-label'), 'READY'); get('stage-empty').hidden = true;
  poseReview=null;bindText(get('review-status'), msg("Review compares playback and arbitrary-seek poses at 30 fps and around strikes.", "곡 검수는 30fps와 타격 전후 표본의 재생·임의 탐색 자세를 비교합니다."));bindText(get('review-measurement'), '');renderDiagnostics();
  return true;
}
function renderDiagnostics() {
  if(!plan)return;
  const guidance=importGuidance(plan);
  bindText(get('mapping-evidence'), joinMessages(guidance.evidence, ' '));
  get('count-in-choice').hidden=!guidance.countInCandidates;
  bindText(get('count-in-label'), msg("Interpret as count-in · {0} candidates", "카운트인으로 해석 · 후보 {0}개", guidance.countInCandidates));
  get('hihat-choice').hidden=!guidance.hasHihatChoice;
  bindText(get('hihat-candidates'), msg("Prepared tracks: {0} open hits · {1} foot strikes · {2} linked closures", "준비한 트랙: 열기 {0}개 · 발 타격 {1}개 · 연결된 닫힘 {2}개", guidance.open, guidance.foot, guidance.paired));
  get('interpretation-empty').hidden=!!guidance.countInCandidates||guidance.hasHihatChoice;
  get<HTMLInputElement>('review-time').max=String(plan.duration);
  refreshFitState();
  const diagnosticList = get('diagnostic-list'); diagnosticList.replaceChildren();
  const unresolved = plan.hits.filter(h=>h.limb === null).length;
  const interpretations = plan.interpretations ?? [];
  const countIn = interpretations.filter(event=>event.kind==='countIn').length;
  const unmapped = interpretations.filter(event=>event.kind==='unmapped').length;
  const omitted=interpretations.filter(event=>event.kind==='omitted').length;
  const pathWarnings=plan.diagnostics.filter(d=>['BODY_PATH_CLEARANCE','EQUIPMENT_PATH_CLEARANCE','EQUIPMENT_LAYOUT_CLEARANCE','COUNT_IN_GESTURE_UNAVAILABLE'].includes(d.code)).length;
  const warnings=unmapped+unresolved+pathWarnings;bindText(get('analysis-badge'), warnings?` ${warnings}`:'');get('analysis-badge').hidden=!warnings;
  bindText(get('analysis-summary'), warnings?msg("{0} unmapped · {1} unassigned motions · {2} path diagnostics", "미매핑 {0}개 · 동작 미배정 {1}개 · 경로 진단 {2}개", unmapped, unresolved, pathWarnings):msg("All strikes have assigned motion. Run a separate path review.", "모든 타격에 동작이 배정되었습니다. 경로 검수는 별도로 실행하세요."));
  bindText(get('source-summary'), warnings?msg("{0} items need attention · see Analysis.", "확인할 항목 {0}개 · 분석에서 확인하세요.", warnings):'');get('source-analysis').hidden=!warnings;
  bindText(get('interpretation-summary'), msg("{0} · {1} count-in · {2} controls · {3} unmapped · {4} omitted · {5} hand overrides · {6} unassigned motions", "{0} · 카운트인 {1}개 · 제어 {2}개 · 미매핑 {3}개 · 제외 {4}개 · 손 지정 {5}개 · 동작 미배정 {6}개", plan.mappingProfile==='logicDkdGm'?'Logic DKD · GM':'General MIDI', countIn, interpretations.filter(event=>event.kind==='control').length, unmapped, omitted, Object.keys(plan.handOverrides??{}).length, unresolved));
  get('diagnostics').hidden = !plan.diagnostics.length;
  bindText(get('diagnostic-summary'), msg("{0} diagnostics{1}", "분석 알림 {0}개{1}", plan.diagnostics.length, unresolved?msg(" · {0} unassigned hits", " · 표현하지 못한 타격 {0}개", unresolved):''));
  for (const item of plan.diagnostics.slice(0,100)) { const li=document.createElement('li'); const text=msg('{0}{1}', '{0}{1}', raw(item.time !== undefined ? preciseClockText(item.time)+' · ':''), engineMessage(item.message));
    if(item.time!==undefined){const jump=document.createElement('button');bindText(jump, text);jump.onclick=()=>{if(!plan||busy||exporting||modelBusy)return;const time=item.time!+PRE_ROLL;audio?.seek(time);updateFrame(time);};li.append(jump);}else bindText(li, text);
    if(item.hitId!==undefined){const hit=plan.hits[item.hitId];if(hit){const related=plan.hits.filter(other=>other.timeUs===hit.timeUs&&other.time===hit.time);const detail=document.createElement('p');detail.className='hint';bindText(detail, msg("MIDI tick {0} · simultaneous hits: {1} · Select the button to jump here.", "MIDI tick {0} · 동시 타격: {1} · 버튼을 누르면 해당 위치로 이동합니다.", hit.tick, related.map(other=>`${other.pieceId??other.instrument} (${other.note})`).join(', ')));li.append(detail);}}
    diagnosticList.append(li); }
  if(plan.diagnostics.length>100){const li=document.createElement('li');bindText(li, msg("{0} more diagnostics", "외 {0}개 알림", plan.diagnostics.length-100));diagnosticList.append(li);}
  status(msg("Ready", "준비 완료"),'ready'); message(unmapped||unresolved?msg("Ready · {0} unmapped · {1} unassigned motions. See Analysis.", "준비 완료 · 미매핑 {0}개 · 동작 미배정 {1}개. 분석에서 확인하세요.", unmapped, unresolved):msg("Ready. Press Play to start the performance.", "준비가 끝났습니다. 재생 버튼을 눌러 연주를 시작하세요.")); timeline.resize();
}
function clearPerformance() {
  reviewGeneration++;poseReview=null;preparedSelection=null;audio?.dispose(); audio=null; buffer=null; plan=null;
  engine.clearPerformance(); scene.clearPlan();void refreshIdlePose(); wavAlignmentInfo=undefined;showWavInfo();
  for (const id of ['metric-duration','metric-hits','metric-tempos','plan-label']) bindText(get(id), '—');
  for (const id of ['duration','current-time']) bindText(get(id), '00:00');
  get('stage-empty').hidden=false; get('diagnostics').hidden=true; bindText(get('interpretation-summary'), '');
  bindText(get('analysis-summary'), msg("Load MIDI to see analysis results.", "MIDI를 불러오면 분석 결과를 확인할 수 있습니다."));get('analysis-badge').hidden=true;bindText(get('source-summary'), '');get('source-analysis').hidden=true;
  get<HTMLInputElement>('seek').value='0'; timeline.setPlan(null);
  setButtonIcon(get('play'), 'play'); bindAttribute(get('play'), 'aria-label', msg("Play", "재생"));
  invalidateVideo();
  timeline.resize();
}
async function loadFile(data: Uint8Array, name: string) {
  if (busy || exporting || modelBusy) return;
  arrangement={omittedSourceEventIds:[],handOverrides:{}};
  const generation = ++loadGeneration;
  clearPerformance(); engine.cancel(); synthesis.cancel();
  bytes = data; fileName=name; bindText(get('file-name'), raw(name));
  bindText(get('file-detail'), msg("{0} KiB · analyzed in your browser", "{0} KiB · 브라우저에서 분석", (data.byteLength/1024).toFixed(1)));
  get('selection').hidden=true; get('diagnostics').hidden=true; busy=true; controls(); status(msg("Analyzing", "분석 중"),'working');
  try {
    if(data.byteLength>MAX_FILE_BYTES)throw new Error(msg("The file exceeds 10 MiB. Export a shorter section.", "파일이 10 MiB를 초과합니다. 더 짧은 구간으로 내보내 주세요."));
    await rememberProjectFile('midi',new File([Uint8Array.from(data)],name,{type:'audio/midi'}));
    message(msg("Reading MIDI tracks and tempo…", "MIDI의 트랙과 템포를 읽고 있습니다…"));
    const info = await engine.inspect(data);
    if(generation!==loadGeneration)return;
    fillSelection(info); controls(); await prepare(generation);
  } catch(error) {
    if(generation!==loadGeneration)return;
    message(uiError(error),true); status(msg("Needs attention", "확인 필요"),'error');
  } finally { if(generation===loadGeneration){busy=false;controls();} }
}

async function loadSelectedDemo(id: DemoId) {
  if(busy||exporting||modelBusy)return;
  const generation=++loadGeneration,controller=new AbortController();
  demoController=controller;audio?.pause();busy=true;controls();status(msg("Preparing demo", "데모 준비 중"),'working');
  message(msg("Reading demo file…", "데모 파일을 읽고 있습니다…"));
  try {
    const demo=await loadDemo(id,undefined,(input,init)=>fetch(input,{...init,signal:controller.signal}));
    if(generation!==loadGeneration)return;
    if(demo.bytes.byteLength>MAX_FILE_BYTES)throw new Error(msg("The demo file exceeds 10 MiB.", "데모 파일이 10 MiB를 초과합니다."));
    const file=new File([Uint8Array.from(demo.bytes)],demo.filename,{type:'audio/midi'});
    const reference=await fileReference('midi',file);
    if(generation!==loadGeneration)return;
    const info=await engine.inspect(demo.bytes);
    if(generation!==loadGeneration)return;
    if(!info.suggestedTracks.length)throw new Error(msg("No playable track was found in the demo.", "데모에서 연주할 트랙을 찾지 못했습니다."));
    const selection:DrumProject['selection']={
      channel:null,mappingProfile:'gm',countInNote75:false,hihatPolicy:'literal',hihatOpenness:.2,
      omittedSourceEventIds:[],handOverrides:{},...demo.selection,tracks:info.suggestedTracks,
    };
    if(!await prepareSelection(demo.bytes,selection,generation))return;
    // Source bytes, directives and file references change only after plan/audio preflight.
    bytes=demo.bytes;fileName=demo.filename;commitProjectFile(reference,file);
    bindText(get('file-name'), raw(demo.filename));
    bindText(get('file-detail'), msg("{0} KiB · demo", "{0} KiB · 데모", (demo.bytes.byteLength/1024).toFixed(1)));
    fillSelection(info);
    get<HTMLSelectElement>('channel').value=selection.channel===null?'all':String(selection.channel);
    get<HTMLSelectElement>('mapping-profile').value=selection.mappingProfile!;
    get<HTMLInputElement>('count-in-75').checked=selection.countInNote75!;
    get<HTMLInputElement>('hihat-policy').checked=selection.hihatPolicy==='shortOpenClose';
    get<HTMLInputElement>('hihat-openness').value=String(selection.hihatOpenness!*100);
    get<HTMLSelectElement>('demo-select').value=id;
  } catch(error) {
    if(generation!==loadGeneration)return;
    message(msg("{0}{1}", "{0}{1}", uiError(error), plan?msg(" The previous performance and settings are preserved.", " 기존 연주와 설정은 유지됩니다."):''),true);
    status(msg("Demo needs attention", "데모 확인 필요"),'error');
  } finally {
    if(demoController===controller)demoController=null;
    if(generation===loadGeneration){busy=false;controls();}
  }
}

function showWavInfo(info?: WavAlignmentInfo) {
  if(info)wavAlignmentInfo=info;
  if(!plan)wavAlignmentInfo=undefined;
  info=info??wavAlignmentInfo;
  if(!wav){wavAlignmentInfo=undefined;bindText(get('wav-info'), msg("Synthesized drum audio · loading a WAV replaces this sound.", "합성 드럼 오디오 · WAV를 불러오면 이 소리를 대체합니다."));return;}
  const parts=[raw(wav.name), msg("{0} s", "{0}초", wav.buffer.duration.toFixed(2)), msg("Start {0} s", "시작 {0}초", wavOffset.toFixed(3))];
  if(info){
    if(info.copiedSeconds===0)parts.push(msg("Warning: no overlap with the current song; the result is silent", "주의: 현재 곡 범위와 겹치지 않아 전체 무음"));
    if(info.trimmedStartSeconds>0)parts.push(msg("{0} s trimmed from start", "앞 {0}초 잘림", info.trimmedStartSeconds.toFixed(2)));
    if(info.trimmedEndSeconds>0)parts.push(msg("{0} s trimmed from end", "뒤 {0}초 잘림", info.trimmedEndSeconds.toFixed(2)));
    if(info.trailingSilenceSeconds>.001)parts.push(msg("{0} s trailing silence", "끝 무음 {0}초", info.trailingSilenceSeconds.toFixed(2)));
  }else parts.push(msg("Applied once MIDI is prepared", "MIDI를 준비하면 적용됩니다"));
  bindText(get('wav-info'), joinMessages(parts));
}
function replaceAudio(next:AudioBuffer,time=audio?.pause()??0) {
  audio?.dispose();buffer=next;audio=new DrumAudio(next);audio.setVolume(Number(get<HTMLInputElement>('volume').value));audio.seek(time);invalidateVideo();updateFrame(time);status(msg("Paused", "일시정지"));
}
get('wav-import').onclick=()=>get<HTMLInputElement>('wav-file').click();
get<HTMLInputElement>('wav-file').onchange=async event=>{
  const input=event.target as HTMLInputElement,file=input.files?.[0];input.value='';
  if(!file||busy||exporting||modelBusy)return;
  const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));bindText(get('wav-info'), msg("Reading WAV…", "WAV를 읽고 있습니다…"));
  try{
    const candidate=await loadLocalWav(file);if(generation!==loadGeneration)return;
    const aligned=plan?alignWavToPlan(candidate.buffer,plan,wavOffset):null;
    const reference=await fileReference('wav',file);if(generation!==loadGeneration)return;
    wav=candidate;commitProjectFile(reference,file);if(aligned)replaceAudio(aligned.buffer,time);showWavInfo(aligned?.info);
  }catch(error){if(generation===loadGeneration){showWavInfo();message(uiError(error),true);}}
  finally{if(generation===loadGeneration){busy=false;controls();}}
};
get('wav-apply').onclick=()=>{
  if(!wav||busy||exporting||modelBusy)return;
  const field=get<HTMLInputElement>('wav-offset');if(!field.reportValidity())return;
  try{const next=field.valueAsNumber;const aligned=plan?alignWavToPlan(wav.buffer,plan,next):null;
    if(!Number.isFinite(next)||Math.abs(next)>300)throw new Error(msg("WAV start offset must be between −300 and 300 seconds.", "WAV 시작 위치는 −300–300초 범위여야 합니다."));
    wavOffset=next;if(aligned)replaceAudio(aligned.buffer);showWavInfo(aligned?.info);
  }catch(error){message(uiError(error),true);}
};
get('wav-remove').onclick=async()=>{
  if(!wav||busy||exporting||modelBusy)return;
  const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));
  try{const next=plan?await synthesis.run(plan):null;if(generation!==loadGeneration)return;
    wav=null;sourceReferences=sourceReferences.filter(ref=>ref.kind!=='wav');wavOffset=0;get<HTMLInputElement>('wav-offset').value='0';if(next)replaceAudio(next,time);showWavInfo();
  }catch(error){if(generation===loadGeneration)message(uiError(error),true);}
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
  select.replaceChildren(...group.map(event=>uiOption(msg('{0} · {1} · track {2}', '{0} · {1} · 트랙 {2}', names.get(event.id)?.name??msg('Note', '음표'), event.message.kind==='noteOn'?event.message.key:'', event.track+1),event.id)));
  select.value=id;
  const meaning=plan.interpretations?.find(event=>event.sourceEventId===id),hit=plan.hits.find(event=>event.sourceEventId===id);
  bindText(get('arrangement-detail'), msg("{0} s · tick {1} · source {2} · velocity {3} · {4}", "{0}초 · tick {1} · 원본 {2} · Velocity {3} · {4}", (selected.timeUs/1e6).toFixed(6), selected.tick, id, selected.message.kind==='noteOn'?selected.message.velocity:'', meaning?.kind==='omitted'?msg("Omitted from performance", "연주에서 제외"):hit?.limb==null?msg("No assigned motion", "동작 없음"):[msg("Left hand", "왼손"),msg("Right hand", "오른손"),msg("Left foot", "왼발"),msg("Right foot", "오른발")][hit.limb]));
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
  busy=true;projectBusy=true;controls();status(msg("Preparing note edits", "음표 편집 준비 중"),'working');
  try {
    const selection={...structuredClone(oldSelection),...next};
    const candidate=await engine.plan(bytes,selection,false);
    const aligned=wav?alignWavToPlan(wav.buffer,candidate,wavOffset):null;
    const nextBuffer=aligned?.buffer??await synthesis.run(candidate);
    const restored=await engine.restorePlan(candidate,scene.getRigProfile(),scene.getKitLayout());
    plan=restored;preparedSelection=selection;arrangement=arrangementOf(restored);poseReview=null;
    scene.setPlan(plan);timeline.setPlan(plan);replaceAudio(nextBuffer,time);showWavInfo(aligned?.info);invalidateVideo();
    bindText(get('metric-hits'), plan.hits.length.toLocaleString());renderDiagnostics();
    selectArrangementEvent(selectedId);
    bindText(get('arrangement-status'), msg("Note edits applied. Compare playback and seeking at the same time. Original MIDI and WAV are preserved.", "음표 편집을 적용했습니다. 같은 시각에서 재생·탐색으로 확인하세요. 원본 MIDI와 WAV는 보존됩니다."));
  } catch(error){
    plan=oldPlan;preparedSelection=oldSelection;
    bindText(get('arrangement-status'), uiError(error));
    status(msg("Edits need attention", "편집 확인 필요"),'error');
  } finally {busy=false;projectBusy=false;controls();updateFrame(time);}
}
get('arrangement-apply').onclick=()=>{
  if(!plan)return;
  try{void applyArrangement(withArrangementEdit(plan,get<HTMLSelectElement>('arrangement-event').value,get<HTMLInputElement>('arrangement-omit').checked,get<HTMLSelectElement>('arrangement-hand').value as 'auto'|'0'|'1'));}
  catch(error){bindText(get('arrangement-status'), uiError(error));}
};
get('arrangement-reset').onclick=()=>void applyArrangement({omittedSourceEventIds:[],handOverrides:{}});

function jumpToSongTime(time: number) {
  if(!plan||busy||exporting||modelBusy||!Number.isFinite(time))return;
  const songTime=Math.max(0,Math.min(plan.duration,time));
  audio?.pause();audio?.seek(songTime+PRE_ROLL);updateFrame(songTime+PRE_ROLL);
  get<HTMLInputElement>('review-time').value=songTime.toFixed(3);status(msg("Paused", "일시정지"));
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
  bindText(get('review-measurement'), msg("Song {0} s · {1}{2}", "곡 {0}초 · {1}{2}", songTime.toFixed(3), joinMessages(contacts.map(sample=>msg("{0} {1} mm", "{0} {1} mm", [msg("L stick", "L 스틱"),msg("R stick", "R 스틱"),msg("L foot", "L 발"),msg("R foot", "R 발")][sample.limb], (sample.errorMeters*1000).toFixed(2)))), proxy?msg(" · proxy clearance: torso {0} cm / arms {1} cm / sticks {2} cm{3}{4} (negative values indicate possible intersections)", " · 근사 간격: 몸통 {0} cm / 양팔 {1} cm / 스틱 {2} cm{3}{4} (음수는 침범 후보)", (proxy.torsoClearanceMeters*100).toFixed(1), (proxy.armClearanceMeters*100).toFixed(1), (proxy.stickClearanceMeters*100).toFixed(1), proxy.equipmentClearanceMeters!==undefined?msg(" / equipment {0} cm", " / 장비 {0} cm", (proxy.equipmentClearanceMeters*100).toFixed(1)):'', proxy.intentionalStickContact?msg(" · count-in tip contact", " · 카운트인 끝 접촉"):'') : ''));
  return { songTime, outputTime, contacts, proxy, pose:pose?Array.from(pose):null, sampledReview:poseReview };
}
get('review-current').onclick=()=>{if(plan&&!busy&&!exporting&&!modelBusy){currentReviewMeasurement();status(msg("Paused", "일시정지"));}};
get('review-trace-hide').onclick=()=>scene.setStickTrace(null);
get('review-trace').onclick=()=>{
  if(!plan||busy||exporting||modelBusy)return;
  const outputTime=audio?.pause()??0,center=outputTime-PRE_ROLL;
  const start=Math.max(-PRE_ROLL,center-.3),end=Math.min(plan.duration,center+.3),paths:[Vec3[],Vec3[]]=[[],[]];
  try {
    for(let index=0;index<=36;index++){
      const time=start+(end-start)*index/36,pose=engine.evaluate(time);if(!pose)throw new Error(msg("No stick pose is available.", "스틱 자세가 없습니다."));
      scene.applyPose(pose,time);
      for(const sample of scene.getRigContactSamples())if(sample.limb<2)paths[sample.limb].push([...sample.actual]);
    }
    scene.setStickTrace(paths);
    bindText(get('review-measurement'), msg("Song {0}–{1} s · 37 actual stick-tip samples · left orange / right blue. Fixed traces around the current time; not a collision test.", "곡 {0}–{1}초 · 실제 스틱 끝 37개 표본 · 왼손 주황 / 오른손 파랑. 현재 시각 주변의 고정 궤적이며 충돌 판정은 아닙니다.", start.toFixed(3), end.toFixed(3)));
  } catch(error){bindText(get('review-measurement'), uiError(error));}
  finally{updateFrame(outputTime);status(msg("Paused", "일시정지"));}
};
get('review-scan').onclick=async()=>{
  if(!plan||busy||exporting||modelBusy)return;
  const current=plan,restoreTime=audio?.pause()??0,generation=++reviewGeneration;
  const times=reviewTimes(current),ordered:Float32Array[]=[],shuffled:Float32Array[]=[],inspections:PoseInspection[]=[];
  busy=true;reviewRunning=true;controls();status(msg("Reviewing song", "곡 검수 중"),'working');
  let sinceYield=performance.now();
  const check=async(index:number)=>{
    if(generation!==reviewGeneration)throw new DOMException(msg("Review cancelled.", "검수를 취소했습니다."),'AbortError');
    if(performance.now()-sinceYield>12){bindText(get('review-status'), msg("Comparing poses {0} / {1} · 30 fps and samples around strikes", "자세 비교 {0} / {1} · 30fps 및 타격 전후 표본", index.toLocaleString(), (times.length*2).toLocaleString()));await new Promise<void>(resolve=>requestAnimationFrame(()=>resolve()));sinceYield=performance.now();}
  };
  try{
    for(let index=0;index<times.length;index++){await check(index);ordered[index]=Float32Array.from(engine.evaluate(times[index])??[]);const proxy=engine.inspectPose(times[index]);if(proxy)inspections.push(proxy);}
    let count=times.length;
    for(const index of reviewOrder(times.length)){await check(count++);shuffled[index]=Float32Array.from(engine.evaluate(times[index])??[]);}
    poseReview=summarizePoseReview(times,ordered,shuffled);
    if(inspections.length)poseReview.proxy=summarizeProxyReview(times,inspections);
    bindText(get('review-status'), msg("{0} times reviewed · seek-order difference {1} · {2} invalid values{3}. Checks pose reproducibility and proxy geometry, not skin or clothing collisions.", "{0}개 시각 검수 · 탐색 순서 차이 {1} · 유효하지 않은 값 {2}개{3}. 자세 재현성·근사 영역 검사이며 피부·옷 충돌 검사는 아닙니다.", poseReview.samples.toLocaleString(), poseReview.maxOrderDifference.toExponential(2), poseReview.nonfiniteValues, poseReview.proxy?msg(" · {0} samples with possible proxy intersections{1}", " · 근사 영역 침범 후보 {0}개 표본{1}", poseReview.proxy.intersectingSamples, poseReview.proxy.firstIntersectionSeconds===null?'':msg(" (first at {0} s)", " (처음 {0}초)", poseReview.proxy.firstIntersectionSeconds.toFixed(3))):''));
  }catch(error){bindText(get('review-status'), uiError(error));}
  finally{reviewRunning=false;if(generation===reviewGeneration){busy=false;status(msg("Paused", "일시정지"));controls();}updateFrame(restoreTime);}
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
  setTimeout(()=>URL.revokeObjectURL(url),1000);status(msg("Paused", "일시정지"));
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
      calibration:scene.getCalibration(),kit:scene.getKitLayout(),rig:currentFBX?'fbx':sourceReferences.some(ref=>ref.kind==='vrm')?'vrm':'native',...(currentFBX?{fbx:structuredClone(currentFBX)}:{}),assets:structuredClone(sourceReferences),
      appearance:{preset:committedPreset as KitPresetId,pieces:current.kit.filter(piece=>!['openHat','pedalHat'].includes(piece.id)).map(piece=>({id:piece.id,settings:scene.getPieceVisual(piece.id)}))},
      camera:scene.getViewState(),hitResponse:scene.getHitResponse(),audio:{source:wav?'wav':'synthesized',offsetSeconds:wavOffset,volume:Number(get<HTMLInputElement>('volume').value)},
      view:{theme,background:get<HTMLInputElement>('background').value,skeleton:get<HTMLInputElement>('skeleton-visible').checked,model:get<HTMLInputElement>('model-visible').checked,character:get<HTMLInputElement>('character-visible').checked,timelineZoom:Number(get<HTMLInputElement>('timeline-zoom').value)},outputTime};
    parseProject(JSON.stringify(project));
    downloadJson(project,`${fileName.replace(/\.(mid|midi)$/i,'')}.drumvrm.json`);
    bindText(get('project-message'), msg("Project saved. This private file includes MIDI notes and the plan. Keep the matching app version and original WAV, VRM, FBX, texture and GLB files.", "프로젝트를 저장했습니다. MIDI 노트·계획이 포함된 개인 파일입니다. 같은 버전의 앱과 원본 WAV·VRM·FBX·텍스처·GLB를 함께 보관하세요."));
  }catch(error){bindText(get('project-message'), uiError(error));}
  finally{busy=false;projectBusy=false;controls();status(msg("Paused", "일시정지"));}
};
function showMissingProjectAssets() {
  if(!pendingProject)return;
  const missing=missingProjectAssets(pendingProject,projectFiles);
  get('project-relink').hidden=!missing.length;
  if(missing.length)bindText(get('project-message'), msg("The current performance is preserved. Relink originals: {0}. A matching name is not enough; the SHA-256 content hash must match.", "현재 연주는 유지됩니다. 다시 연결할 원본: {0}. 이름이 같아도 내용의 SHA-256이 다르면 사용할 수 없습니다.", raw(missing.map(asset=>`${asset.kind} · ${asset.name}`).join(', '))));
}
async function restorePendingProject() {
  const project=pendingProject;if(!project)return;
  if(missingProjectAssets(project,projectFiles).length){showMissingProjectAssets();return;}
  const file=(kind:ProjectAsset['kind'])=>{const reference=project.assets.find(asset=>asset.kind===kind);return reference?projectFiles.get(reference.sha256)!:null;};
  let adapter:HumanoidRigAdapter|null=null;
  let preparedAppearance:Awaited<ReturnType<DrumScene['prepareProjectAppearance']>>|null=null;
  let modelInfo:VRMInspection|null=null;
  try {
    if(await projectEngineHash()!==project.engineModuleSha256)throw new Error(msg("The saved project uses a different engine version. Open it in the matching app version or prepare a new performance from the original MIDI.", "저장한 프로젝트와 현재 엔진 버전이 다릅니다. 동일 버전에서 열거나 원본 MIDI로 새 연주를 준비해 주세요."));
    const midiFile=file('midi')!,midiBytes=new Uint8Array(await midiFile.arrayBuffer()),info=await engine.inspect(midiBytes);
    await engine.validateProjectSource(midiBytes,project.selection,project.plan);
    if(project.rig==='vrm'){
      const loaded=await loadLocalVRM(file('vrm')!);modelInfo=loaded.info;
      try{adapter=new VRMHumanoidRigAdapter(loaded.vrm,`vrm1:${project.assets.find(asset=>asset.kind==='vrm')!.sha256}`);}
      catch(error){VRMUtils.deepDispose(loaded.vrm.scene);throw error;}
    }else if(project.rig==='fbx'){
      const ref=project.assets.find(a=>a.kind==='fbx')!;
      const original=(asset:ProjectAsset)=>new File([projectFiles.get(asset.sha256)!],asset.name);
      const loaded=await loadLocalFBX(original(ref),project.assets.filter(a=>a.kind==='fbxTexture').map(original));
      try{await loaded.applyTextures(project.fbx!);adapter=new FBXHumanoidRigAdapter(loaded.scene,project.fbx!,`fbx:${ref.sha256}`,loaded.release);}
      catch(error){loaded.dispose();throw error;}
    }else adapter=new NativeHumanoidRigAdapter();
    const profile=adapter.setCalibration(project.calibration,project.kit);
    if(JSON.stringify(profile)!==JSON.stringify(project.profile))throw new Error(msg("The saved body measurements differ from the current rig. Prepare a new performance from the original MIDI.", "저장한 체형 측정값과 현재 리그가 다릅니다. 원본 MIDI로 새 연주를 준비해 주세요."));
    const nextPlan=await engine.restorePlan(project.plan,profile,project.kit,false);
    const nextWav=project.audio.source==='wav'?await loadLocalWav(file('wav')!):null;
    const aligned=nextWav?alignWavToPlan(nextWav.buffer,nextPlan,project.audio.offsetSeconds):null;
    const nextBuffer=aligned?.buffer??await synthesis.run(nextPlan);
    const parts=new Map<PieceId,File>();for(const ref of project.assets)if(ref.kind==='drumPart')parts.set(ref.pieceId!,projectFiles.get(ref.sha256)!);
    preparedAppearance=await scene.prepareProjectAppearance(project.appearance.preset,project.appearance.pieces,parts);
    preparedAppearance.validateLayout(project.kit);
    // Every file, setting, measured body and Rust plan has passed before replacing the session.
    const restored=await engine.restorePlan(project.plan,profile,project.kit);
    audio?.pause();openPanel('source');preparedAppearance.apply(project.kit);preparedAppearance=null;scene.setRig(adapter);adapter=null;
    scene.setCalibration(project.calibration);await scene.whenEquipmentReady();
    plan=restored;bytes=midiBytes;fileName=midiFile.name;preparedSelection=structuredClone(project.selection);arrangement=arrangementOf(restored);sourceReferences=structuredClone(project.assets);currentFBX=project.fbx?structuredClone(project.fbx):undefined;
    wav=nextWav;wavOffset=project.audio.offsetSeconds;wavAlignmentInfo=aligned?.info;
    committedPreset=project.appearance.preset;get<HTMLSelectElement>('kit-preset').value=committedPreset;
    bindText(get('file-name'), raw(fileName));bindText(get('file-detail'), msg("{0} KiB · restored from project", "{0} KiB · 프로젝트에서 복원", (midiBytes.byteLength/1024).toFixed(1)));
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
    const characterPreset=CHARACTER_PRESETS.find(entry=>entry.sha256===project.assets.find(asset=>asset.kind==='vrm')?.sha256);
    committedCharacter=project.rig==='native'?'native':characterPreset?.id??'local';refreshCharacterPreset();
    scene.setPlan(plan);timeline.setPlan(plan);replaceAudio(nextBuffer,project.outputTime);showWavInfo(aligned?.info);showModelInfo(modelInfo,characterPreset);
    if(project.rig==='fbx')showFBXInfo(project.assets.find(a=>a.kind==='fbx')!.name);
    bindText(get('character-message'), project.rig==='fbx'?msg('Project FBX · bone, texture and body settings restored', '프로젝트 FBX · 본·텍스처·체형 설정 복원'):project.rig==='vrm'?msg("Project VRM · original file and body settings restored", "프로젝트 VRM · 같은 원본 파일과 체형 설정 복원"):msg("Default rig · project body settings restored", "기본 리그 · 프로젝트 체형 설정 복원"));
    get<HTMLInputElement>('seek').max=String(outputDuration(plan));bindText(get('duration'), clockText(outputDuration(plan)));
    bindText(get('metric-duration'), clockText(plan.duration));bindText(get('metric-hits'), plan.hits.length.toLocaleString());bindText(get('metric-tempos'), String(plan.tempoCount));bindText(get('plan-label'), 'READY');get('stage-empty').hidden=true;
    fitProposal=null;preFitSetup=null;fitBaseline=structuredClone(project.kit);fitState=msg("Saved layout restored", "저장한 배치 복원");poseReview=null;refreshCalibration();refreshTarget();refreshPiece();renderDiagnostics();
    pendingProject=null;get('project-relink').hidden=true;bindText(get('project-message'), msg("Original files verified and project restored. Compare WAV and poses during playback and seeking.", "같은 원본 파일을 확인하고 프로젝트를 복원했습니다. WAV와 자세를 재생·탐색으로 확인하세요."));
  } finally {adapter?.dispose();preparedAppearance?.dispose();}
}
get('project-open').onclick=()=>get<HTMLInputElement>('project-file').click();
get<HTMLInputElement>('project-file').onchange=async event=>{
  const input=event.target as HTMLInputElement,file=input.files?.[0];input.value='';if(!file||busy||exporting||modelBusy)return;
  busy=true;projectBusy=true;controls();audio?.pause();
  try{if(file.size>MAX_PROJECT_BYTES)throw new Error(msg("The project must be no larger than 32 MiB.", "프로젝트는 32 MiB 이하여야 합니다."));const candidate=parseProject(await file.text());if(await projectEngineHash()!==candidate.engineModuleSha256)throw new Error(msg("The saved project uses a different engine version. Prepare the original MIDI again.", "저장한 프로젝트와 현재 엔진 버전이 다릅니다. 원본 MIDI를 다시 준비해 주세요."));pendingProject=candidate;await restorePendingProject();}
  catch(error){bindText(get('project-message'), uiError(error));}
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
    if(rejected.length)bindText(get('project-message'), joinMessages([boundText(get('project-message')), msg(" Files excluded because they did not match: {0}", " 일치하지 않아 제외한 파일: {0}", raw(rejected.join(', ')))], ''));
  }catch(error){bindText(get('project-message'), uiError(error));}
  finally{busy=false;projectBusy=false;controls();}
};

get<HTMLInputElement>('hihat-policy').onchange=()=>controls();
get<HTMLSelectElement>('mapping-profile').onchange=()=>{bindText(get('mapping-evidence'), msg("Mapping changed. Apply settings to prepare results for the selected drum sound source.", "매핑을 변경했습니다. 다시 준비하면 선택한 음원 기준의 결과를 확인할 수 있습니다."));controls();};
get('upload').onclick=()=>get<HTMLInputElement>('file').click();
get<HTMLInputElement>('file').onchange=async(event)=>{const input=event.target as HTMLInputElement;const file=input.files?.[0]; if(file){if(file.size>MAX_FILE_BYTES){message(msg("Choose a MIDI file no larger than 10 MiB.", "10 MiB 이하의 MIDI 파일을 선택해 주세요."),true);}else await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}input.value='';};
for(const demo of DEMO_CATALOG)get<HTMLSelectElement>('demo-select').add(uiOption(demo.label,demo.id));
get<HTMLAnchorElement>('demo-notice').href=`${import.meta.env.BASE_URL}demos/NOTICE.txt`;
get('demo-load').onclick=()=>void loadSelectedDemo(get<HTMLSelectElement>('demo-select').value as DemoId);
const dropzone=get('dropzone');
dropzone.ondragover=e=>{e.preventDefault();if(!busy&&!exporting)dropzone.classList.add('dragging');};
dropzone.ondragleave=()=>dropzone.classList.remove('dragging');
dropzone.ondrop=async(e)=>{e.preventDefault();dropzone.classList.remove('dragging');if(busy||exporting||modelBusy)return;const file=e.dataTransfer?.files[0];if(file){if(file.size>MAX_FILE_BYTES){message(msg("Choose a MIDI file no larger than 10 MiB.", "10 MiB 이하의 MIDI 파일을 선택해 주세요."),true);return;}await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}};
get('cancel-analysis').onclick=()=>{++loadGeneration;++reviewGeneration;demoController?.abort();demoController=null;engine.cancel();synthesis.cancel();busy=false;showWavInfo();message(reviewRunning?msg("Song review cancelled.", "곡 검수를 취소했습니다."):msg("Preparation cancelled. Any previous performance is still available.", "준비를 취소했습니다. 기존 연주가 있다면 다시 재생할 수 있습니다."));status(plan?msg("Paused", "일시정지"):msg("Idle", "대기"));controls();};
get('replan').onclick=async()=>{if(!bytes||busy||exporting||modelBusy)return;const generation=++loadGeneration,time=audio?.pause()??0;busy=true;controls();status(msg("Preparing", "준비 중"),'working');try{await prepare(generation,time);}catch(e){if(generation===loadGeneration){message(msg("{0} The previous performance is preserved. Check that note edits remain compatible with the changed mapping and track selection.", "{0} 기존 연주는 유지됩니다. 음표 편집과 변경한 매핑·트랙 선택의 호환성을 확인하세요.", uiError(e)),true);status(msg("Needs attention", "확인 필요"),'error');}}finally{if(generation===loadGeneration){busy=false;controls();}}};
get('play').onclick=async()=>{
  if(!audio||!plan)return;
  try{if(audio.playing){audio.pause();status(msg("Paused", "일시정지"));}else{if(audio.currentTime>=outputDuration(plan)-.01)audio.seek(0);await audio.play();status(msg("Playing", "연주 중"),'playing');}}
  catch(e){message(msg("Unable to start audio. {0}", "소리를 시작하지 못했습니다. {0}", uiError(e)),true);}
};
get('reset').onclick=()=>{audio?.pause();audio?.seek(0);updateFrame(0);status(msg("Ready", "준비 완료"),'ready');};
get<HTMLInputElement>('seek').oninput=()=>{audio?.seek(Number(get<HTMLInputElement>('seek').value));updateFrame(Number(get<HTMLInputElement>('seek').value));};
get<HTMLInputElement>('volume').oninput=()=>audio?.setVolume(Number(get<HTMLInputElement>('volume').value));
for(const button of document.querySelectorAll<HTMLButtonElement>('[data-camera]'))button.onclick=()=>{scene.setCamera(button.dataset.camera as 'stage'|'front'|'top'|'player');for(const other of document.querySelectorAll('[data-camera]'))other.setAttribute('aria-pressed',String(other===button));};
get<HTMLInputElement>('background').oninput=()=>scene.setBackground(get<HTMLInputElement>('background').value);
get('cancel-export').onclick=()=>exportController?.abort();
get('export').onclick=async()=>{
  if(!plan||!buffer||exporting||modelBusy)return;
  const currentPlan=plan; const currentBuffer=buffer; const restoreTime=audio?.pause()??0;
  exporting=true;exportController=new AbortController();controls();status(msg("Creating video", "영상 만드는 중"),'working');
  get<HTMLProgressElement>('progress').value=0;bindText(get('progress-text'), msg("Checking encoders…", "인코더 확인 중…"));
  get('download').hidden=true;
  try {
    const profiles=await probeExport(get<HTMLInputElement>('mute-export').checked, exportController.signal);
    if(exportController.signal.aborted)throw new DOMException(msg("Export cancelled.", "내보내기를 취소했습니다."),'AbortError');
    if(!profiles.length)throw new Error(msg("No supported video/audio encoder was found in this browser. Use a recent version of Chrome or Edge.", "이 브라우저에서 지원하는 영상·오디오 인코더를 찾지 못했습니다. 최신 Chrome 또는 Edge를 사용해 주세요."));
    const profile=profiles[0];
    scene.setExportMode(true);scene.resize(1280,720);
    const result=await exportVideo({canvas:stage,renderFrame:(songTime)=>{const pose=engine.evaluate(songTime);if(pose)scene.applyPose(pose,songTime);scene.render();},plan:currentPlan,audio:currentBuffer,profile,signal:exportController.signal,onProgress:p=>{get<HTMLProgressElement>('progress').value=p;bindText(get('progress-text'), `${Math.round(p*100)}% · ${profile.extension.toUpperCase()}`);}});
    if(downloadUrl)URL.revokeObjectURL(downloadUrl);downloadUrl=URL.createObjectURL(result.blob);
    const download=get<HTMLAnchorElement>('download');download.href=downloadUrl;download.download=`${fileName.replace(/\.(mid|midi)$/i,'').replace(/[^\p{L}\p{N} _-]/gu,'_')}-drum.${result.extension}`;download.hidden=false;
    bindText(get('export-note'), msg("{0} · {1} frames · {2} s · {3} · {4} MiB", "{0} · {1} 프레임 · {2}초 · {3} · {4} MiB", result.extension.toUpperCase(), result.frames, result.duration.toFixed(2), profile.audioCodec?(wav?msg("Includes user WAV", "사용자 WAV 포함"):msg("Includes synthesized drums", "합성 드럼 오디오 포함")):msg("Silent export selected", "선택한 무음 출력"), (result.blob.size/1048576).toFixed(1)));
    message(msg("Video ready. Download it from the Export panel.", "영상이 완성됐습니다. 내보내기 패널에서 다운로드하세요."));status(msg("Video ready", "영상 준비 완료"),'ready');
  } catch(e){const cancelled=e instanceof DOMException&&e.name==='AbortError';message(cancelled?msg("Video export cancelled. You can play or export again.", "영상 추출을 취소했습니다. 다시 재생하거나 내보낼 수 있습니다."):msg("Video export failed: {0}", "영상 추출 실패: {0}", uiError(e)),!cancelled);status(cancelled?msg("Ready", "준비 완료"):msg("Export needs attention", "출력 확인 필요"),cancelled?'ready':'error');}
  finally {exporting=false;exportController=null;scene.setExportMode(false);scene.resize();updateFrame(restoreTime);controls();}
};
document.addEventListener('keydown',e=>{if(e.code==='Space'&&!/INPUT|SELECT|BUTTON|TEXTAREA/.test((e.target as HTMLElement).tagName)){e.preventDefault();get<HTMLButtonElement>('play').click();}});
const resizeObserver=new ResizeObserver(()=>{if(!exporting){scene.resize();timeline.resize();}});resizeObserver.observe(get('stage-view'));
let disposed=false;
function tick(){if(disposed)return;if(!exporting){const time=audio?.currentTime??0;if(plan){updateFrame(time);get<HTMLInputElement>('seek').value=String(time);bindText(get('current-time'), clockText(time));timeline.setTime(time);const playing=audio?.playing??false;if(get('play').dataset.icon!==(playing?'pause':'play')){setButtonIcon(get('play'),playing?'pause':'play');bindAttribute(get('play'), 'aria-label', playing?msg("Pause", "일시정지"):msg("Play", "재생"));}if(!playing&&get('status').classList.contains('playing'))status(time>=outputDuration(plan)-.02?msg("Performance complete", "연주 완료"):msg("Paused", "일시정지"),time>=outputDuration(plan)-.02?'ready':'');}scene.render();}requestAnimationFrame(tick);}tick();
controls();timeline.resize();setCharacterDisplay('skeleton');
async function initializeStudio() {
  modelBusy=true;controls();
  try{await scene.whenEquipmentReady();await engine.setSetup(scene.getRigProfile(),scene.getKitLayout());}
  catch(error){message(uiError(error),true);return;}
  finally{modelBusy=false;controls();}
  await autoFitCurrent();
}
void initializeStudio();
window.addEventListener('beforeunload',()=>{disposed=true;demoController?.abort();engine.dispose();synthesis.cancel();audio?.dispose();scene.dispose();timeline.dispose();resizeObserver.disconnect();clearTimeout(toastTimer);if(downloadUrl)URL.revokeObjectURL(downloadUrl);});
