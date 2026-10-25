import './style.css';
import { DrumTimeline } from './timeline';
import { DrumScene } from './scene';
import { setButtonIcon } from './icons';
import { NativeHumanoidRigAdapter, type HumanoidCalibration } from './rig-humanoid';
import { moveKitTarget } from './kit-calibration';
import { VRMHumanoidRigAdapter } from './rig-vrm';
import { loadLocalVRM, type VRMInspection } from './vrm-import';
import { VRMUtils } from '@pixiv/three-vrm';
import type { KitPresetId } from './model-procedural';
import { EngineClient } from './engine';
import { DrumAudio } from './audio';
import { loadLocalWav, alignWavToPlan, type LocalWav, type WavAlignmentInfo } from './wav';
import { SynthesisClient } from './synthesis';
import { probeExport, exportVideo } from './export';
import { makeDemo, makeInterpretationDemo } from './demo';
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo, type PieceId, type KitPiece, type Vec3 } from './types';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = `  <header class="toolbar">
    <h1>Drum VRM</h1>
    <div class="transport" aria-label="재생 제어">
      <button id="reset" aria-label="처음으로" title="처음으로" disabled></button>
      <button id="play" aria-label="재생" title="재생 / 일시정지 (Space)" disabled></button>
      <output class="time-display"><span id="current-time">00:00</span> / <span id="duration">00:00</span></output>
      <input id="volume" aria-label="음량" title="음량" type="range" min="0" max="1" step="0.01" value="0.6"/>
    </div>
    <nav class="menus" aria-label="설정 메뉴">
      <button data-panel="character" aria-expanded="false" aria-controls="panel-character">캐릭터</button>
      <button data-panel="drums" aria-expanded="false" aria-controls="panel-drums">드럼</button>
      <button data-panel="source" aria-expanded="false" aria-controls="panel-source">업로드 · 연주정보</button>
      <button data-panel="export" aria-expanded="false" aria-controls="panel-export">내보내기</button>
      <button id="theme" aria-label="다크 모드로 전환" title="테마 전환">다크</button>
    </nav>
  </header>
  <main class="workspace">
    <section class="studio" aria-label="3D 드럼 연주">
      <div class="stage-bar"><span id="status" role="status">대기</span><div class="view-controls" aria-label="카메라 시점"><button data-camera="stage" aria-pressed="true">연주</button><button data-camera="front" aria-pressed="false">정면</button><button data-camera="top" aria-pressed="false">위</button><button data-camera="player" aria-pressed="false">연주자</button></div></div>
      <div class="stage-view" id="stage-view"><canvas id="stage" aria-label="드럼 세트와 연주 스켈레톤"></canvas><div id="stage-empty" class="stage-empty"><button id="start-upload">MIDI 불러오기</button><button id="start-demo">데모 재생 준비</button></div></div>
      <p id="message" class="message" role="status" aria-live="polite">MIDI를 선택하면 연주를 준비합니다.</p>
    </section>
    <aside id="inspector" class="inspector" hidden>
      <div class="inspector-heading"><h2 id="panel-title"></h2><button id="close-panel" aria-label="설정 닫기" title="닫기 (Esc)"></button></div>
      <section id="panel-character" class="panel" hidden>
        <label class="check"><input id="character-visible" type="checkbox" checked/> 캐릭터 표시</label>
        <label class="check"><input id="skeleton-visible" type="checkbox" checked/> 뼈대 표시</label>
        <label class="check"><input id="model-visible" type="checkbox" checked/> 모델 표시</label>
        <div class="button-row"><button id="character-import">VRM 불러오기…</button><button id="character-reset">기본 리그</button></div><input id="character-file" type="file" accept=".vrm" hidden/>
        <p id="character-message" class="hint" role="status">VRM 1.0 표준 본 · 기본 스켈레톤</p>
        <p class="hint">단일 .vrm · VRM 1.0 · 최대 50 MiB<br>.vroid는 편집 파일입니다. VRM 1.0으로 내보내세요.</p>
        <div id="character-info"></div>
        <label class="field">손바닥 방향<select id="palm-orientation"><option value="up">위</option><option value="inward">안쪽 · 엄지 위</option><option value="down">아래</option></select></label>
        <p class="hint">양손의 방향을 함께 조절합니다. 전통 그립의 좌우 다른 손가락 배치는 포함하지 않습니다.</p>
        <h3>좌석</h3>
        <label class="field">높이 (cm)<input id="seat-height" type="number" min="30" max="120" step="0.1" required/></label>
        <label class="field">앞뒤 위치 (cm · 앞쪽 +)<input id="seat-forward" type="number" min="-50" max="50" step="0.1" required/></label>
        <div class="button-row"><button id="seat-apply">좌석 적용</button><button id="seat-auto">자동 맞춤</button></div>
        <div class="button-row"><button id="kit-fit">키트를 체형에 맞춤</button><button id="kit-layout-reset">기본 배치</button></div><p class="hint">손으로 치는 악기의 배치·크기를 함께 조정합니다. 킥과 페달은 유지됩니다. 피부·악기 간 충돌까지 보장하지 않으므로 화면에서 확인하세요.</p><p id="calibration-message" class="hint" role="status">팔·다리 길이를 유지하며 좌석과 타점을 조절합니다.</p>
        <details><summary>접촉 계측</summary><p id="contact-info" class="hint">실제 손에 부착한 스틱 끝과 발의 기준점을 계획과 비교합니다.</p><p class="hint">발 기준점은 신발 표면의 충돌 검사가 아닙니다. 도달하지 못한 타격은 연주 정보에 표시됩니다.</p></details>
        <p class="hint">파일은 브라우저에서만 처리합니다. 영상 공개와 모델 재배포의 이용 조건은 별개입니다. 머리카락·옷 물리는 재생과 탐색의 일치를 위해 정지합니다.</p>
        <a href="https://github.com/postmelee/drum-vrm/blob/main/docs/models.ko.md" target="_blank" rel="noreferrer">모델 확보 · 업로드 안내 / English</a>
      </section>
      <section id="panel-drums" class="panel" hidden>
        <label class="field">드럼 세트<select id="kit-preset"><option value="studio">기본 · Studio</option><option value="vintage">기본 · Vintage</option><option value="electronic">기본 · Electronic</option><option value="library">어쿠스틱 · Innerscene</option><option value="batoski">어쿠스틱 · Batoski</option><option value="smoj">로우 폴리 · smoj</option></select></label>
        <label class="check"><input id="hit-response" type="checkbox"/> 타격 반응</label><label class="field">반응 강도<input id="hit-strength" aria-label="타격 반응 강도" type="range" min="0" max="1" step="0.05" value="0.65"/></label>
        <p class="hint">드럼을 클릭하면 해당 부품을 설정합니다.</p>
        <label class="field">부품<select id="piece"><option value="kick">Kick</option><option value="snare">Snare</option><option value="hihat">Hi-hat</option><option value="tomHigh">High tom</option><option value="tomMid">Mid tom</option><option value="tomLow">Low tom</option><option value="crash">Crash L</option><option value="crashRight">Crash R</option><option value="ride">Ride</option></select></label>
        <details><summary>타점 위치</summary>
          <label class="field">조절할 타점<select id="target-piece"><option value="kick">Kick 페달 · 킥 배치</option><option value="snare">Snare</option><option value="hihat">Hi-hat 심벌</option><option value="pedalHat">Hi-hat 페달</option><option value="tomHigh">High tom</option><option value="tomMid">Mid tom</option><option value="tomLow">Low tom</option><option value="crash">Crash L</option><option value="crashRight">Crash R</option><option value="ride">Ride</option></select></label>
          <label class="field">좌우 (cm · 연주자 오른쪽 +)<input id="target-x" type="number" min="-200" max="200" step="1"/></label>
          <label class="field">높이 (cm)<input id="target-y" type="number" min="2" max="250" step="1"/></label>
          <label class="field">앞뒤 (cm · 앞쪽 +)<input id="target-z" type="number" min="-50" max="250" step="1"/></label>
          <div class="button-row"><button id="target-apply">타점 적용</button><button id="target-reset">타점 초기화</button></div>
          <p id="target-message" class="hint" role="status">실제 배치와 손발 계획을 함께 변경합니다. 열린·닫힌 하이햇은 같은 타점을 사용합니다.</p>
        </details>
        <label class="field">기본 모델<select id="piece-model"><option value="asset" disabled>프리셋 / 가져온 모델</option><option value="standard">표준</option><option value="deep">깊은 셸</option><option value="compact">컴팩트</option></select></label>
        <label class="field horizontal">색상<input id="piece-finish" type="color" value="#6b7280"/></label>
        <label class="field">크기 <output id="piece-scale-value">100%</output><input id="piece-scale" type="range" min="0.75" max="1.25" step="0.01" value="1"/></label>
        <label class="field">회전 <output id="piece-rotation-value">0°</output><input id="piece-rotation" aria-label="부품 회전" type="range" min="-180" max="180" step="1" value="0"/></label>
        <button id="piece-reset">부품을 프리셋으로 복원</button><button id="piece-import">이 부품의 GLB 불러오기…</button><input id="model-file" type="file" accept=".glb" hidden/>
        <p id="model-message" class="hint" role="status">텍스처가 포함된 정적 .glb · 최대 25 MiB</p>
        <p class="hint">모델 교체는 드럼 본체만 바꿉니다. 받침대·페달과 배치는 유지됩니다. GLB는 받침대·페달을 제외한 부품 파일을 사용하세요. 자동 맞춤 뒤 표면과 타점을 확인하세요.</p>
        <details><summary>모델 출처와 지원 형식</summary><p>기본 3종은 자체 생성 도형입니다. Innerscene은 CC0, Batoski와 smoj는 CC BY 3.0 모델입니다. 부품별로 크기와 타점을 맞춥니다.</p><p>스킨·뼈대가 있는 GLB와 GLTF·FBX·OBJ·BLEND는 직접 지원하지 않습니다. 외부 파일 참조 없는 GLB로 변환하세요.</p><a href="https://github.com/postmelee/drum-vrm/blob/main/docs/research/drum-assets.md" target="_blank" rel="noreferrer">모델 조사 · 라이선스</a></details>
      </section>
      <section id="panel-source" class="panel" hidden>
        <div id="dropzone"><strong id="file-name">MIDI 파일 선택</strong><p id="file-detail" class="hint">여기에 드롭 · 최대 10 MiB / 5분</p><div class="button-row"><button id="upload">MIDI 불러오기…</button><button id="demo">30초 데모</button><button id="verification-demo">연주 검증 패턴</button></div><input id="file" type="file" accept=".mid,.midi,audio/midi,audio/x-midi" hidden/></div>
        <div id="selection" hidden><h3>트랙</h3><div id="tracks" class="track-list"></div><label class="field">MIDI 채널<select id="channel"><option value="all">전체 채널</option></select></label><label class="field">드럼 매핑<select id="mapping-profile"><option value="gm">General MIDI</option><option value="logicDkdGm">Logic · Drum Kit Designer (GM)</option></select></label><label class="check"><input id="count-in-75" type="checkbox" disabled/> 노트 75를 카운트인으로 처리</label><label class="check"><input id="hihat-policy" type="checkbox" disabled/> 짧은 Open → Foot Close를 작은 열림으로 연결</label><label class="field">하이햇 열림 (%)<input id="hihat-openness" type="number" min="5" max="100" step="1" value="20" disabled/></label><p class="hint">선택한 트랙의 짧은 닫힘을 발목의 연속 동작으로 해석합니다. 원본 노트는 보존됩니다. 심벌 간격과 실제 음색을 재현한 값은 아닙니다.</p><p class="hint">Logic의 GM + ModWheel과 V-Drum 모드는 아직 지원하지 않습니다. 매핑을 바꾼 후 다시 준비하세요.</p><button id="replan">선택한 설정으로 다시 준비</button></div>
        <h3>영상 오디오</h3><div class="button-row"><button id="wav-import">WAV 불러오기…</button><button id="wav-remove" disabled>합성음으로 복원</button></div><input id="wav-file" type="file" accept=".wav,audio/wav" hidden/>
        <label class="field">WAV 시작 위치 (초 · MIDI 기준)<input id="wav-offset" type="number" min="-300" max="300" step="0.001" value="0"/></label><button id="wav-apply">오프셋 적용</button>
        <p id="wav-info" class="hint" role="status">합성 드럼 오디오 · WAV를 불러오면 이 소리를 대체합니다.</p><p class="hint">양수는 WAV를 늦게 시작합니다. 준비 0.5초는 자동으로 더해집니다. 곡 범위 밖은 잘리고, 부족한 부분은 무음입니다. PCM / float WAV · 모노·스테레오 · 최대 100 MiB / 5분.</p>
        <button id="cancel-analysis" hidden>준비 취소</button>
        <h3>연주 정보 <span id="plan-label">—</span></h3><dl class="metrics"><div><dt>곡 길이</dt><dd id="metric-duration">—</dd></div><div><dt>타격</dt><dd id="metric-hits">—</dd></div><div><dt>템포 구간</dt><dd id="metric-tempos">—</dd></div></dl>
        <p id="interpretation-summary" class="hint"></p><details id="diagnostics" hidden><summary id="diagnostic-summary"></summary><ul id="diagnostic-list"></ul></details>
        <p class="hint">선택한 드럼 매핑 · 합성 음향<br>파일은 브라우저 안에서 처리됩니다.</p>
      </section>
      <section id="panel-export" class="panel" hidden>
        <p>1280 × 720 · 30 fps</p><label class="check"><input type="checkbox" id="mute-export"/> 오디오 없이 내보내기</label>
        <button id="export" disabled>영상 추출</button><div id="export-progress" hidden><progress id="progress" max="1" value="0"></progress><div class="progress-label"><span id="progress-text">인코더 확인 중…</span><button id="cancel-export">취소</button></div></div>
        <a id="download" class="download-link" hidden>완성된 영상 다운로드</a><p id="export-note" class="hint">지원 코덱에 따라 MP4 또는 WebM으로 저장합니다. 시작 준비 0.5초 · 끝 여운 0.25초 포함.</p>
        <label class="field horizontal">영상 배경<input id="background" type="color" value="#ffffff"/></label>
        <a class="license-link" href="./licenses.txt" target="_blank">오픈소스 · 에셋 라이선스</a>
      </section>
    </aside>
  </main>
  <section class="editor" aria-label="MIDI 미리보기"><div class="editor-toolbar"><strong>Piano Roll</strong><span class="hint">미리보기</span><input id="seek" aria-label="재생 위치" type="range" min="0" max="1" step="0.001" value="0" disabled/><label>확대 <input id="timeline-zoom" aria-label="MIDI 확대" type="range" min="1" max="16" step="0.25" value="1"/></label></div><div id="timeline"></div></section>
`;

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
engine.setSetup(scene.getRigProfile(), defaultKit);
let plan: PerformancePlan | null = null;
let bytes: Uint8Array | null = null;
let audio: DrumAudio | null = null;
let buffer: AudioBuffer | null = null;
let wav: LocalWav | null = null;
let wavOffset = 0;
let wavAlignmentInfo: WavAlignmentInfo | undefined;
let busy = false;
let exporting = false;
let loadGeneration = 0;
let exportController: AbortController | null = null;
let downloadUrl: string | null = null;
let fileName = 'drum-performance';
type Panel = 'character' | 'drums' | 'source' | 'export';
let activePanel: Panel | null = null;
let modelBusy = false;
let committedPreset = 'studio';
let theme: 'light' | 'dark' = 'light';
try { theme = localStorage.getItem('drum-theme') === 'dark' ? 'dark' : 'light'; } catch { /* Storage is optional. */ }
const timeline = new DrumTimeline(get('timeline'), { onSeek: time => {
  if (!plan || busy || exporting || modelBusy) return;
  audio?.seek(time); updateFrame(time);
}});
const panelTitles: Record<Panel, string> = {character:'캐릭터 설정',drums:'드럼 설정',source:'업로드 · 연주정보',export:'내보내기'};
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
function invalidateVideo() {
  if(downloadUrl)URL.revokeObjectURL(downloadUrl); downloadUrl=null; get('download').hidden=true;
  get('export-note').textContent='1280 × 720 · 30 fps · 현재 설정으로 다시 추출하세요. 시작 준비 0.5초 · 끝 여운 0.25초 포함.';
}
function applyCalibration(seat: HumanoidCalibration, kit: KitPiece[], feedbackId: string) {
  if(busy||exporting||modelBusy)return false;
  const oldSeat=scene.getCalibration(), oldKit=scene.getKitLayout();
  let applied=false;
  const time=audio?.pause()??0;
  try {
    scene.setKitLayout(kit);
    const profile=scene.setCalibration(seat);
    const next=engine.setSetup(profile,kit);
    if(next&&plan){plan=next;scene.setPlan(plan);timeline.setPlan(plan);renderDiagnostics();updateFrame(time);}
    invalidateVideo();
    const unavailable=plan?.hits.filter(h=>h.limb===null).length??0;
    get(feedbackId).textContent=`보정 적용 완료${unavailable?` · 표현하지 못한 타격 ${unavailable}개 (연주정보 확인)`:''}`;
    applied=true;
  } catch(error) {
    scene.setKitLayout(oldKit);scene.setCalibration(oldSeat);updateFrame(time);
    get(feedbackId).textContent=error instanceof Error?error.message:String(error);
  }
  refreshCalibration();refreshTarget();return applied;
}
refreshCalibration();refreshTarget();
get('seat-apply').onclick=()=>{
  const height=get<HTMLInputElement>('seat-height'), forward=get<HTMLInputElement>('seat-forward');
  if(!height.reportValidity()||!forward.reportValidity())return;
  applyCalibration({...scene.getCalibration(),seatHeight:height.valueAsNumber/100,seatForward:forward.valueAsNumber/100},scene.getKitLayout(),'calibration-message');
};
get<HTMLSelectElement>('palm-orientation').onchange=()=>applyCalibration({...scene.getCalibration(),palmOrientation:get<HTMLSelectElement>('palm-orientation').value as HumanoidCalibration['palmOrientation']},scene.getKitLayout(),'calibration-message');
get('kit-fit').onclick=async()=>{
  if(busy||exporting||modelBusy)return;
  audio?.pause();modelBusy=true;controls();get('calibration-message').textContent='체형에 맞는 배치를 계산하고 있습니다…';
  try{const proposal=await engine.fitKit(scene.getRigProfile(),scene.getKitLayout());modelBusy=false;
    if(!proposal.complete){get('calibration-message').textContent=`기본 팔 자세를 만족하는 배치를 찾지 못했습니다: ${proposal.unreachablePieceIds.join(', ')}. 현재 배치를 유지합니다. 좌석과 타점을 직접 조정해 주세요.`;return;}
    if(!applyCalibration(scene.getCalibration(),proposal.kit,'calibration-message'))return;
    get('calibration-message').textContent+=` · 키트 ${(proposal.scale*100).toFixed(0)}% · 개별 타점 ${proposal.pieceAdjustments.length}개 조정 · 타점과 기본 팔 자세 확인`;
  }catch(error){get('calibration-message').textContent=String(error);}
  finally{modelBusy=false;controls();}
};
get('kit-layout-reset').onclick=()=>applyCalibration(scene.getCalibration(),defaultKit,'calibration-message');
get('seat-auto').onclick=()=>applyCalibration(scene.getAutomaticCalibration(),scene.getKitLayout(),'calibration-message');
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
get<HTMLInputElement>('character-visible').onchange=e=>scene.setCharacterVisible((e.target as HTMLInputElement).checked);
get<HTMLInputElement>('skeleton-visible').onchange=e=>scene.setSkeletonVisible((e.target as HTMLInputElement).checked);
get<HTMLInputElement>('model-visible').onchange=e=>scene.setModelVisible((e.target as HTMLInputElement).checked);
function showModelInfo(info: VRMInspection | null) {
  const host = get('character-info'); host.replaceChildren(); if (!info) return;
  const heading = document.createElement('h3'); heading.textContent = info.name; host.append(heading);
  for (const text of [`VRM ${info.version} · 모델 ${info.modelVersion} · 필수 본 ${info.required}/15`, `손가락 본 L ${info.fingers.left}/15 · R ${info.fingers.right}/15`, `제작자: ${info.authors.join(', ') || '미기재'}`]) {
    const p = document.createElement('p'); p.className='hint'; p.textContent=text; host.append(p);
  }
  if(info.fingers.left<15 || info.fingers.right<15) { const p=document.createElement('p'); p.className='hint'; p.textContent='일부 손가락 본이 없어 그립 표현이 제한됩니다. 스틱 부착과 연주는 유지됩니다.'; host.append(p); }
  for (const warning of info.metadataWarnings) { const p=document.createElement('p'); p.className='hint'; p.textContent=warning; host.append(p); }
  const details = document.createElement('details'), summary = document.createElement('summary'); summary.textContent='모델 이용 조건'; details.append(summary);
  for (const [label, value] of info.terms) { const p=document.createElement('p'); p.className='hint'; p.textContent=`${label}: ${value}`; details.append(p); }
  for(const reference of info.references) { const p=document.createElement('p'); p.className='hint'; p.textContent=`참고: ${reference}`; details.append(p); }
  host.append(details);
}
function adoptRig(adapter: NativeHumanoidRigAdapter | VRMHumanoidRigAdapter) {
  const kit=scene.getKitLayout();
  const profile=adapter.setCalibration({...adapter.getAutomaticCalibration(kit),palmOrientation:scene.getCalibration().palmOrientation},kit);
  const next = engine.setSetup(profile,kit);
  scene.setRig(adapter);
  scene.setSkeletonVisible(get<HTMLInputElement>('skeleton-visible').checked);
  scene.setModelVisible(get<HTMLInputElement>('model-visible').checked);
  if (next && plan) { plan = next; scene.setPlan(plan); timeline.setPlan(plan); renderDiagnostics(); updateFrame(audio?.currentTime ?? 0); }
  invalidateVideo();refreshCalibration();
}
get('character-import').onclick=()=>get<HTMLInputElement>('character-file').click();
get<HTMLInputElement>('character-file').onchange=async e=>{
  const input=e.target as HTMLInputElement, file=input.files?.[0]; input.value=''; if(!file||busy||exporting||modelBusy)return;
  audio?.pause(); modelBusy=true;controls();get('character-message').textContent='VRM을 검사하고 있습니다…';
  let candidate: Awaited<ReturnType<typeof loadLocalVRM>> | null = null;
  let adapter: VRMHumanoidRigAdapter | null = null;
  try {
    candidate=await loadLocalVRM(file);
    const hash=await crypto.subtle.digest('SHA-256',await file.arrayBuffer());
    const id='vrm1:'+Array.from(new Uint8Array(hash)).map(v=>v.toString(16).padStart(2,'0')).join('');
    adapter=new VRMHumanoidRigAdapter(candidate.vrm,id); adoptRig(adapter);
    showModelInfo(candidate.info); candidate=null; adapter=null;
    get('character-message').textContent=`${file.name} · 불러오기 완료 · 타격/그립은 연주하며 확인하세요.`;
  } catch(error) { if(adapter)adapter.dispose();else if(candidate)VRMUtils.deepDispose(candidate.vrm.scene); get('character-message').textContent=error instanceof Error?error.message:String(error); }
  finally{modelBusy=false;controls();}
};
get('character-reset').onclick=()=>{
  if(busy||exporting||modelBusy)return; audio?.pause();
  const adapter=new NativeHumanoidRigAdapter();
  try{adoptRig(adapter);showModelInfo(null);get('character-message').textContent='VRM 1.0 표준 본 · 기본 스켈레톤';}
  catch(error){adapter.dispose();get('character-message').textContent=String(error);}
};
get<HTMLSelectElement>('kit-preset').onchange=async e=>{
  const input=e.target as HTMLSelectElement;
  modelBusy=true;controls();get('model-message').textContent='드럼 모델을 준비하고 있습니다…';
  try {await scene.setKitPreset(input.value as KitPresetId);committedPreset=input.value;refreshPiece();get('model-message').textContent=({library:'Innerscene · CC0',batoski:'Batoski · CC BY 3.0',smoj:'smoj · CC BY 3.0'} as Record<string,string>)[input.value] ?? '자체 생성 도형 · 부품별로 외형을 변경할 수 있습니다.';}
  catch(error){input.value=committedPreset;get('model-message').textContent=`모델 로딩 실패: ${error instanceof Error?error.message:String(error)}`;}
  finally{modelBusy=false;controls();}
};
get<HTMLSelectElement>('piece').onchange=()=>{scene.selectPiece(selectedPiece());get<HTMLSelectElement>('target-piece').value=selectedPiece();refreshPiece();refreshTarget();};
scene.onPieceSelect=id=>{if(!id)return;get<HTMLSelectElement>('piece').value=id==='openHat'||id==='pedalHat'?'hihat':id;get<HTMLSelectElement>('target-piece').value=id==='openHat'?'hihat':id;refreshPiece();refreshTarget();};
get<HTMLSelectElement>('piece-model').onchange=e=>{scene.setPieceVisual(selectedPiece(),{model:(e.target as HTMLSelectElement).value as 'standard'|'deep'|'compact'});refreshPiece();};
get<HTMLInputElement>('piece-finish').oninput=e=>scene.setPieceVisual(selectedPiece(),{finish:(e.target as HTMLInputElement).value});
get<HTMLInputElement>('piece-scale').oninput=e=>{const scale=Number((e.target as HTMLInputElement).value);scene.setPieceVisual(selectedPiece(),{scale});get('piece-scale-value').textContent=`${Math.round(scale*100)}%`;};
get('piece-reset').onclick=()=>{scene.resetPieceVisual(selectedPiece());refreshPiece();get('model-message').textContent='선택한 부품을 프리셋 상태로 복원했습니다.';};
get<HTMLInputElement>('piece-rotation').oninput=e=>{const rotation=Number((e.target as HTMLInputElement).value);scene.setPieceVisual(selectedPiece(),{rotation});get('piece-rotation-value').textContent=`${rotation}°`;};
get('piece-import').onclick=()=>get<HTMLInputElement>('model-file').click();
get<HTMLInputElement>('model-file').onchange=async e=>{
  const input=e.target as HTMLInputElement;const file=input.files?.[0];input.value='';if(!file)return;
  modelBusy=true;controls();get('model-message').textContent='GLB를 불러오는 중…';
  try{const result=await scene.loadPieceModel(selectedPiece(),file);get('model-message').textContent=`${result.fileName} · 자동 크기 맞춤 (타점 확인 필요)`;refreshPiece();}
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
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting || activePanel==='drums';
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-panel], #close-panel, #theme, #character-visible')) el.disabled=exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-drums input, #panel-drums select, #panel-drums button')) el.disabled=busy || exporting || modelBusy;
  for (const id of ['start-upload','start-demo']) get<HTMLButtonElement>(id).disabled=busy || exporting;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-character input, #panel-character select, #panel-character button')) el.disabled=busy || exporting || modelBusy;
  get<HTMLInputElement>('hit-strength').disabled = exporting || modelBusy || !get<HTMLInputElement>('hit-response').checked;
  get<HTMLButtonElement>('export').disabled = !plan || busy || exporting || modelBusy;
  get<HTMLInputElement>('count-in-75').disabled = busy || exporting || modelBusy || get<HTMLSelectElement>('mapping-profile').value !== 'logicDkdGm';
  get<HTMLButtonElement>('wav-remove').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLButtonElement>('wav-apply').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLInputElement>('hihat-policy').disabled=busy||exporting||modelBusy||get<HTMLSelectElement>('mapping-profile').value!=='logicDkdGm';
  get<HTMLInputElement>('hihat-openness').disabled=busy||exporting||modelBusy||!get<HTMLInputElement>('hihat-policy').checked;
  get('cancel-analysis').hidden = !busy;
  get('export-progress').hidden = !exporting;
}
function updateFrame(time: number) {
  if (!plan) return;
  const pose = engine.evaluate(time - PRE_ROLL);
  if (pose) scene.applyPose(pose, time - PRE_ROLL);
  if (activePanel==='character') {
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
async function prepare(generation: number) {
  const tracks = [...document.querySelectorAll<HTMLInputElement>('#tracks input:checked')].map(el=>Number(el.value));
  if (!tracks.length) throw new Error('연주할 트랙을 하나 이상 선택해 주세요.');
  const selected = get<HTMLSelectElement>('channel').value;
  if(!get<HTMLInputElement>('hihat-openness').reportValidity())throw new Error('하이햇 열림은 5–100% 범위여야 합니다.');
  message('타격과 손발 동작을 준비하고 있습니다…');
  const result = await engine.plan(bytes!,{tracks,channel:selected==='all'?null:Number(selected),mappingProfile:get<HTMLSelectElement>('mapping-profile').value as 'gm'|'logicDkdGm',countInNote75:get<HTMLInputElement>('count-in-75').checked,hihatPolicy:get<HTMLInputElement>('hihat-policy').checked?'shortOpenClose':'literal',hihatOpenness:get<HTMLInputElement>('hihat-openness').valueAsNumber/100});
  if (generation !== loadGeneration) return;
  message(wav?'WAV를 영상 시간에 맞추고 있습니다…':'합성 드럼 오디오를 준비하고 있습니다…');
  const aligned = wav ? alignWavToPlan(wav.buffer,result,wavOffset) : null;
  const nextBuffer = aligned?.buffer ?? await synthesis.run(result);
  if (generation !== loadGeneration) return;
  plan = result;
  if(aligned)showWavInfo(aligned.info);
  buffer = nextBuffer;
  audio?.dispose(); audio = new DrumAudio(buffer); audio.setVolume(Number(get<HTMLInputElement>('volume').value));
  scene.setPlan(plan); updateFrame(0);
  const total = outputDuration(plan);
  get<HTMLInputElement>('seek').max = String(total); get<HTMLInputElement>('seek').value = '0';
  get('duration').textContent = clockText(total); timeline.setPlan(plan);
  get('metric-duration').textContent = clockText(plan.duration); get('metric-hits').textContent = plan.hits.length.toLocaleString(); get('metric-tempos').textContent = String(plan.tempoCount);
  get('plan-label').textContent = 'READY'; get('stage-empty').hidden = true;
  renderDiagnostics();
}
function renderDiagnostics() {
  if(!plan)return;
  const diagnosticList = get('diagnostic-list'); diagnosticList.replaceChildren();
  const unresolved = plan.hits.filter(h=>h.limb === null).length;
  const interpretations = plan.interpretations ?? [];
  const countIn = interpretations.filter(event=>event.kind==='countIn').length;
  const unmapped = interpretations.filter(event=>event.kind==='unmapped').length;
  get('interpretation-summary').textContent = `${plan.mappingProfile==='logicDkdGm'?'Logic DKD · GM':'General MIDI'} · 카운트인 ${countIn}개 · 제어 ${interpretations.filter(event=>event.kind==='control').length}개 · 미매핑 ${unmapped}개 · 동작 미배정 ${unresolved}개`;
  get('diagnostics').hidden = !plan.diagnostics.length;
  get('diagnostic-summary').textContent = `분석 알림 ${plan.diagnostics.length}개${unresolved?` · 표현하지 못한 타격 ${unresolved}개`:''}`;
  for (const item of plan.diagnostics.slice(0,100)) { const li=document.createElement('li'); const text=`${item.time !== undefined ? preciseClockText(item.time)+' · ':''}${item.message}`;
    if(item.time!==undefined){const jump=document.createElement('button');jump.textContent=text;jump.onclick=()=>{if(!plan||busy||exporting||modelBusy)return;const time=item.time!+PRE_ROLL;audio?.seek(time);updateFrame(time);};li.append(jump);}else li.textContent=text;
    if(item.hitId!==undefined){const hit=plan.hits[item.hitId];if(hit){const related=plan.hits.filter(other=>other.timeUs===hit.timeUs&&other.time===hit.time);const detail=document.createElement('p');detail.className='hint';detail.textContent=`MIDI tick ${hit.tick} · 동시 타격: ${related.map(other=>`${other.pieceId??other.instrument} (${other.note})`).join(', ')} · 버튼을 누르면 해당 위치로 이동합니다.`;li.append(detail);}}
    diagnosticList.append(li); }
  if(plan.diagnostics.length>100){const li=document.createElement('li');li.textContent=`외 ${plan.diagnostics.length-100}개 알림`;diagnosticList.append(li);}
  status('준비 완료','ready'); message(unmapped||unresolved?`준비 완료 · 미매핑 ${unmapped}개 · 동작 미배정 ${unresolved}개. 연주정보의 매핑과 알림을 확인하세요.`:'준비가 끝났습니다. 재생 버튼을 눌러 연주를 시작하세요.'); timeline.resize();
}
function clearPerformance() {
  audio?.dispose(); audio=null; buffer=null; plan=null;
  engine.clearPerformance(); scene.clearPlan(); wavAlignmentInfo=undefined;showWavInfo();
  for (const id of ['metric-duration','metric-hits','metric-tempos','plan-label']) get(id).textContent='—';
  for (const id of ['duration','current-time']) get(id).textContent='00:00';
  get('stage-empty').hidden=false; get('diagnostics').hidden=true; get('interpretation-summary').textContent='';
  get<HTMLInputElement>('seek').value='0'; timeline.setPlan(null);
  setButtonIcon(get('play'), 'play'); get('play').setAttribute('aria-label','재생');
  invalidateVideo();
  timeline.resize();
}
async function loadFile(data: Uint8Array, name: string) {
  if (busy || exporting || modelBusy) return;
  const generation = ++loadGeneration;
  clearPerformance(); engine.cancel(); synthesis.cancel();
  bytes = data; fileName=name; get('file-name').textContent=name;
  get('file-detail').textContent=`${(data.byteLength/1024).toFixed(1)} KiB · 브라우저에서 분석`;
  get('selection').hidden=true; get('diagnostics').hidden=true; busy=true; controls(); status('분석 중','working');
  try {
    if(data.byteLength>MAX_FILE_BYTES)throw new Error('파일이 10 MiB를 초과합니다. 더 짧은 구간으로 내보내 주세요.');
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
    wav=candidate;if(aligned)replaceAudio(aligned.buffer,time);showWavInfo(aligned?.info);
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
    wav=null;wavOffset=0;get<HTMLInputElement>('wav-offset').value='0';if(next)replaceAudio(next,time);showWavInfo();
  }catch(error){if(generation===loadGeneration)message(String(error),true);}
  finally{if(generation===loadGeneration){busy=false;controls();}}
};

get<HTMLInputElement>('hihat-policy').onchange=()=>controls();
get<HTMLSelectElement>('mapping-profile').onchange=()=>{if(get<HTMLSelectElement>('mapping-profile').value!=='logicDkdGm'){get<HTMLInputElement>('count-in-75').checked=false;get<HTMLInputElement>('hihat-policy').checked=false;}controls();};
get('upload').onclick=()=>get<HTMLInputElement>('file').click();
get<HTMLInputElement>('file').onchange=async(event)=>{const input=event.target as HTMLInputElement;const file=input.files?.[0]; if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);}else await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}input.value='';};
get('verification-demo').onclick=()=>{get<HTMLSelectElement>('mapping-profile').value='logicDkdGm';get<HTMLInputElement>('count-in-75').checked=true;get<HTMLInputElement>('hihat-policy').checked=true;get<HTMLInputElement>('hihat-openness').value='20';void loadFile(makeInterpretationDemo(),'Interpretation Check.mid');};
get('demo').onclick=()=>void loadFile(makeDemo(),'Studio Groove.mid');
const dropzone=get('dropzone');
dropzone.ondragover=e=>{e.preventDefault();if(!busy&&!exporting)dropzone.classList.add('dragging');};
dropzone.ondragleave=()=>dropzone.classList.remove('dragging');
dropzone.ondrop=async(e)=>{e.preventDefault();dropzone.classList.remove('dragging');if(busy||exporting||modelBusy)return;const file=e.dataTransfer?.files[0];if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);return;}await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}};
get('cancel-analysis').onclick=()=>{++loadGeneration;engine.cancel();synthesis.cancel();busy=false;showWavInfo();message('준비를 취소했습니다. 기존 연주가 있다면 다시 재생할 수 있습니다.');status(plan?'일시정지':'대기');controls();};
get('replan').onclick=async()=>{if(!bytes)return;const generation=++loadGeneration;clearPerformance();busy=true;controls();status('준비 중','working');try{await prepare(generation);}catch(e){if(generation===loadGeneration){message(String(e),true);status('확인 필요','error');}}finally{if(generation===loadGeneration){busy=false;controls();}}};
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
controls();timeline.resize();
window.addEventListener('beforeunload',()=>{disposed=true;engine.dispose();synthesis.cancel();audio?.dispose();scene.dispose();timeline.dispose();resizeObserver.disconnect();if(downloadUrl)URL.revokeObjectURL(downloadUrl);});
