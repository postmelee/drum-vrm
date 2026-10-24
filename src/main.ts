import './style.css';
import { DrumTimeline } from './timeline';
import { DrumScene } from './scene';
import { setButtonIcon } from './icons';
import type { KitPresetId } from './model-procedural';
import { EngineClient } from './engine';
import { DrumAudio } from './audio';
import { SynthesisClient } from './synthesis';
import { probeExport, exportVideo } from './export';
import { makeDemo } from './demo';
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo, type Instrument } from './types';

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
        <label class="check"><input id="character-visible" type="checkbox" checked/> 스켈레톤 표시</label>
        <p class="hint">관절과 뼈대, 스틱으로 연주를 표시합니다.</p>
        <h3>캐릭터 에셋</h3><p>캐릭터 파일 불러오기는 아직 지원하지 않습니다.</p><p class="hint">VRM 1.0과 휴머노이드 GLB 연결은 후속 작업입니다. 드럼용 GLB를 캐릭터에 적용할 수는 없습니다.</p>
        <a href="https://github.com/postmelee/drum-vrm/issues/17" target="_blank" rel="noreferrer">캐릭터 지원 계획</a>
      </section>
      <section id="panel-drums" class="panel" hidden>
        <label class="field">드럼 세트<select id="kit-preset"><option value="studio">기본 · Studio</option><option value="vintage">기본 · Vintage</option><option value="electronic">기본 · Electronic</option><option value="library">어쿠스틱 · Innerscene</option><option value="batoski">어쿠스틱 · Batoski</option><option value="smoj">로우 폴리 · smoj</option></select></label>
        <label class="check"><input id="hit-response" type="checkbox"/> 타격 반응</label><label class="field">반응 강도<input id="hit-strength" aria-label="타격 반응 강도" type="range" min="0" max="1" step="0.05" value="0.65"/></label>
        <p class="hint">드럼을 클릭하면 해당 부품을 설정합니다.</p>
        <label class="field">부품<select id="piece"><option value="kick">Kick</option><option value="snare">Snare</option><option value="hihat">Hi-hat</option><option value="tomHigh">High tom</option><option value="tomMid">Mid tom</option><option value="tomLow">Low tom</option><option value="crash">Crash</option><option value="ride">Ride</option></select></label>
        <label class="field">기본 모델<select id="piece-model"><option value="asset" disabled>프리셋 / 가져온 모델</option><option value="standard">표준</option><option value="deep">깊은 셸</option><option value="compact">컴팩트</option></select></label>
        <label class="field horizontal">색상<input id="piece-finish" type="color" value="#6b7280"/></label>
        <label class="field">크기 <output id="piece-scale-value">100%</output><input id="piece-scale" type="range" min="0.75" max="1.25" step="0.01" value="1"/></label>
        <label class="field">회전 <output id="piece-rotation-value">0°</output><input id="piece-rotation" aria-label="부품 회전" type="range" min="-180" max="180" step="1" value="0"/></label>
        <button id="piece-reset">부품을 프리셋으로 복원</button><button id="piece-import">이 부품의 GLB 불러오기…</button><input id="model-file" type="file" accept=".glb" hidden/>
        <p id="model-message" class="hint" role="status">텍스처가 포함된 정적 .glb · 최대 25 MiB</p>
        <p class="hint">외형만 변경합니다. 자동 크기 맞춤 후 모델의 표면과 타점이 다를 수 있습니다.</p>
        <details><summary>모델 출처와 지원 형식</summary><p>기본 3종은 자체 생성 도형입니다. Innerscene은 CC0, Batoski와 smoj는 CC BY 3.0 모델입니다. 부품별로 크기와 타점을 맞춥니다.</p><p>스킨·뼈대가 있는 GLB와 GLTF·FBX·OBJ·BLEND는 직접 지원하지 않습니다. 외부 파일 참조 없는 GLB로 변환하세요.</p><a href="https://github.com/postmelee/drum-vrm/blob/main/docs/research/drum-assets.md" target="_blank" rel="noreferrer">모델 조사 · 라이선스</a></details>
      </section>
      <section id="panel-source" class="panel" hidden>
        <div id="dropzone"><strong id="file-name">MIDI 파일 선택</strong><p id="file-detail" class="hint">여기에 드롭 · 최대 10 MiB / 5분</p><div class="button-row"><button id="upload">MIDI 불러오기…</button><button id="demo">30초 데모</button></div><input id="file" type="file" accept=".mid,.midi,audio/midi,audio/x-midi" hidden/></div>
        <div id="selection" hidden><h3>트랙</h3><div id="tracks" class="track-list"></div><label class="field">MIDI 채널<select id="channel"><option value="all">전체 채널</option></select></label><button id="replan">선택한 트랙으로 다시 준비</button></div>
        <button id="cancel-analysis" hidden>준비 취소</button>
        <h3>연주 정보 <span id="plan-label">—</span></h3><dl class="metrics"><div><dt>곡 길이</dt><dd id="metric-duration">—</dd></div><div><dt>타격</dt><dd id="metric-hits">—</dd></div><div><dt>템포 구간</dt><dd id="metric-tempos">—</dd></div></dl>
        <details id="diagnostics" hidden><summary id="diagnostic-summary"></summary><ul id="diagnostic-list"></ul></details>
        <p class="hint">GM 드럼 · 합성 음향<br>파일은 브라우저 안에서 처리됩니다.</p>
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
let plan: PerformancePlan | null = null;
let bytes: Uint8Array | null = null;
let audio: DrumAudio | null = null;
let buffer: AudioBuffer | null = null;
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
  if (!plan || busy || exporting) return;
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
function selectedPiece() { return get<HTMLSelectElement>('piece').value as Instrument; }
function refreshPiece() {
  const visual = scene.getPieceVisual(selectedPiece());
  get<HTMLSelectElement>('piece-model').value = visual.source==='library'||visual.source==='imported'?'asset':visual.model;
  get<HTMLInputElement>('piece-finish').value = visual.finish;
  get<HTMLInputElement>('piece-scale').value = String(visual.scale);
  get('piece-scale-value').textContent = `${Math.round(visual.scale*100)}%`;
  get<HTMLInputElement>('piece-rotation').value=String(visual.rotation);get('piece-rotation-value').textContent=`${visual.rotation}°`;
}
for (const button of document.querySelectorAll<HTMLButtonElement>('[data-panel]')) button.onclick=()=>openPanel(activePanel===button.dataset.panel?null:button.dataset.panel as Panel);
get('close-panel').onclick=()=>{const previous=activePanel;openPanel(null);document.querySelector<HTMLButtonElement>(`[data-panel="${previous}"]`)?.focus();};
get('theme').onclick=()=>setTheme(theme==='light'?'dark':'light');
get('start-upload').onclick=()=>{openPanel('source');get<HTMLInputElement>('file').click();};
get('start-demo').onclick=()=>{openPanel('source');void loadFile(makeDemo(),'Studio Groove.mid');};
get<HTMLInputElement>('character-visible').onchange=e=>scene.setCharacterVisible((e.target as HTMLInputElement).checked);
get<HTMLSelectElement>('kit-preset').onchange=async e=>{
  const input=e.target as HTMLSelectElement;
  modelBusy=true;controls();get('model-message').textContent='드럼 모델을 준비하고 있습니다…';
  try {await scene.setKitPreset(input.value as KitPresetId);committedPreset=input.value;refreshPiece();get('model-message').textContent=({library:'Innerscene · CC0',batoski:'Batoski · CC BY 3.0',smoj:'smoj · CC BY 3.0'} as Record<string,string>)[input.value] ?? '자체 생성 도형 · 부품별로 외형을 변경할 수 있습니다.';}
  catch(error){input.value=committedPreset;get('model-message').textContent=`모델 로딩 실패: ${error instanceof Error?error.message:String(error)}`;}
  finally{modelBusy=false;controls();}
};
get<HTMLSelectElement>('piece').onchange=()=>{scene.selectPiece(selectedPiece());refreshPiece();};
scene.onPieceSelect=id=>{if(!id)return;get<HTMLSelectElement>('piece').value=id==='openHat'||id==='pedalHat'?'hihat':id;refreshPiece();};
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
function status(text: string, kind = '') { get('status').className = `status ${kind}`; get('status').textContent = text; }
function message(text: string, error = false) { const node = get('message'); node.textContent = text; node.classList.toggle('error', error); }
function controls() {
  for (const id of ['play','reset','seek','export']) (get(id) as HTMLButtonElement).disabled = !plan || busy || exporting;
  for (const id of ['upload','demo','replan','channel','mute-export','background']) (get(id) as HTMLInputElement).disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting || activePanel==='drums';
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-panel], #close-panel, #theme, #character-visible')) el.disabled=exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-drums input, #panel-drums select, #panel-drums button')) el.disabled=exporting || modelBusy;
  for (const id of ['start-upload','start-demo']) get<HTMLButtonElement>(id).disabled=busy || exporting;
  get<HTMLInputElement>('hit-strength').disabled = exporting || modelBusy || !get<HTMLInputElement>('hit-response').checked;
  get<HTMLButtonElement>('export').disabled = !plan || busy || exporting || modelBusy;
  get('cancel-analysis').hidden = !busy;
  get('export-progress').hidden = !exporting;
}
function updateFrame(time: number) {
  if (!plan) return;
  const pose = engine.evaluate(time - PRE_ROLL);
  if (pose) scene.applyPose(pose, time - PRE_ROLL);
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
  message('타격과 손발 동작을 준비하고 있습니다…');
  const result = await engine.plan(bytes!,{tracks,channel:selected==='all'?null:Number(selected)});
  if (generation !== loadGeneration) return;
  message('합성 드럼 오디오를 준비하고 있습니다…');
  const nextBuffer = await synthesis.run(result);
  if (generation !== loadGeneration) return;
  plan = result;
  buffer = nextBuffer;
  audio?.dispose(); audio = new DrumAudio(buffer); audio.setVolume(Number(get<HTMLInputElement>('volume').value));
  scene.setPlan(plan); updateFrame(0);
  const total = outputDuration(plan);
  get<HTMLInputElement>('seek').max = String(total); get<HTMLInputElement>('seek').value = '0';
  get('duration').textContent = clockText(total); timeline.setPlan(plan);
  get('metric-duration').textContent = clockText(plan.duration); get('metric-hits').textContent = plan.hits.length.toLocaleString(); get('metric-tempos').textContent = String(plan.tempoCount);
  get('plan-label').textContent = 'READY'; get('stage-empty').hidden = true;
  const diagnosticList = get('diagnostic-list'); diagnosticList.replaceChildren();
  const unresolved = plan.hits.filter(h=>h.limb === null).length;
  get('diagnostics').hidden = !plan.diagnostics.length;
  get('diagnostic-summary').textContent = `분석 알림 ${plan.diagnostics.length}개${unresolved?` · 표현하지 못한 타격 ${unresolved}개`:''}`;
  for (const item of plan.diagnostics.slice(0,100)) { const li=document.createElement('li'); li.textContent=`${item.time !== undefined ? clockText(item.time)+' · ':''}${item.message}`; diagnosticList.append(li); }
  if(plan.diagnostics.length>100){const li=document.createElement('li');li.textContent=`외 ${plan.diagnostics.length-100}개 알림`;diagnosticList.append(li);}
  status('준비 완료','ready'); message(unresolved?`준비 완료. 표현하지 못한 ${unresolved}개 타격은 알림을 확인해 주세요.`:'준비가 끝났습니다. 재생 버튼을 눌러 연주를 시작하세요.'); timeline.resize();
}
function clearPerformance() {
  if(plan){const resting=engine.evaluate(-PRE_ROLL);if(resting)scene.applyPose(resting,-PRE_ROLL);}
  audio?.dispose(); audio=null; buffer=null; plan=null;
  for (const id of ['metric-duration','metric-hits','metric-tempos','plan-label']) get(id).textContent='—';
  for (const id of ['duration','current-time']) get(id).textContent='00:00';
  get('stage-empty').hidden=false; get('diagnostics').hidden=true;
  get<HTMLInputElement>('seek').value='0'; timeline.setPlan(null);
  get('play').textContent='▶'; get('play').setAttribute('aria-label','재생');
  if(downloadUrl)URL.revokeObjectURL(downloadUrl); downloadUrl=null; get('download').hidden=true;
  timeline.resize();
}
async function loadFile(data: Uint8Array, name: string) {
  if (exporting) return;
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
get('upload').onclick=()=>get<HTMLInputElement>('file').click();
get<HTMLInputElement>('file').onchange=async(event)=>{const input=event.target as HTMLInputElement;const file=input.files?.[0]; if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);}else await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}input.value='';};
get('demo').onclick=()=>void loadFile(makeDemo(),'Studio Groove.mid');
const dropzone=get('dropzone');
dropzone.ondragover=e=>{e.preventDefault();if(!busy&&!exporting)dropzone.classList.add('dragging');};
dropzone.ondragleave=()=>dropzone.classList.remove('dragging');
dropzone.ondrop=async(e)=>{e.preventDefault();dropzone.classList.remove('dragging');if(busy||exporting)return;const file=e.dataTransfer?.files[0];if(file){if(file.size>MAX_FILE_BYTES){message('10 MiB 이하의 MIDI 파일을 선택해 주세요.',true);return;}await loadFile(new Uint8Array(await file.arrayBuffer()),file.name);}};
get('cancel-analysis').onclick=()=>{++loadGeneration;engine.cancel();synthesis.cancel();busy=false;message('분석을 취소했습니다. 파일을 다시 선택할 수 있습니다.');status('대기');controls();};
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
    get('export-note').textContent=`${result.extension.toUpperCase()} · ${result.frames} 프레임 · ${result.duration.toFixed(2)}초 · ${profile.audioCodec?'합성 드럼 오디오 포함':'선택한 무음 출력'} · ${(result.blob.size/1048576).toFixed(1)} MiB`;
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
