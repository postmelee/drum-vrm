import './style.css';
import { DrumScene } from './scene';
import { EngineClient } from './engine';
import { DrumAudio } from './audio';
import { SynthesisClient } from './synthesis';
import { probeExport, exportVideo } from './export';
import { makeDemo } from './demo';
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo } from './types';

const app = document.querySelector<HTMLDivElement>('#app')!;
app.innerHTML = `
  <header class="app-header">
    <a class="brand" href="./" aria-label="Drum VRM 홈"><span class="brand-icon">◒</span><span>DRUM<span class="brand-light"> / VRM</span></span></a>
    <span class="header-note">MIDI PERFORMANCE STUDIO</span>
    <a class="github-link" href="https://github.com/postmelee/drum-vrm" target="_blank" rel="noreferrer">GitHub <span>↗</span></a>
  </header>
  <main>
    <div class="workspace-heading"><div><span class="eyebrow">YOUR RHYTHM, IN MOTION</span><h1>MIDI에서 무대로.</h1></div><div class="privacy-note"><span class="privacy-dot"></span>파일은 이 브라우저 안에만 머뭅니다.</div></div>
    <div class="workspace">
      <section class="studio" aria-label="3D 드럼 연주">
        <div class="stage-top"><span class="stage-title">PERFORMANCE STAGE</span><span id="status" class="status"><i></i><span>대기</span></span></div>
        <div class="stage-view" id="stage-view"><canvas id="stage" aria-label="기본 드러머와 드럼 세트"></canvas>
          <div class="stage-empty" id="stage-empty"><span class="small-label">READY WHEN YOU ARE</span><p>당신의 리듬을 기다리고 있어요.</p></div>
          <div class="view-controls" aria-label="카메라 시점"><button data-camera="stage" class="selected" title="무대 시점">무대</button><button data-camera="front" title="정면 시점">정면</button><button data-camera="top" title="위에서 보기">탑뷰</button><span class="control-divider"></span><label title="배경색" class="color-control"><input id="background" type="color" value="#182321" aria-label="영상 배경색" /></label></div>
          <div class="stage-bottom"><span>PRIMITIVE RIG <b>01</b></span><span>드래그로 회전 · 스크롤로 확대</span></div>
        </div>
        <div class="transport">
          <div class="transport-row"><div class="play-controls"><button id="reset" class="icon-button" aria-label="처음으로" disabled>↤</button><button id="play" class="play-button" aria-label="재생" disabled>▶</button><div class="time-display"><strong id="current-time">00:00</strong><span>/</span><span id="duration">00:00</span></div></div><label class="volume-control"><span aria-hidden="true">♪</span><input id="volume" aria-label="음량" type="range" min="0" max="1" step="0.01" value="0.6"/></label></div>
          <div class="timeline-wrap"><canvas id="timeline" aria-hidden="true"></canvas><input id="seek" aria-label="재생 위치" type="range" min="0" max="1" step="0.001" value="0" disabled/><span id="playhead"></span></div>
          <div class="timeline-labels"><span>00:00</span><span id="timeline-caption">MIDI를 불러오면 연주 타임라인이 표시됩니다.</span><span id="timeline-end">00:00</span></div>
        </div>
      </section>
      <aside class="inspector">
        <section class="panel source-panel"><div class="panel-heading"><h2><span>01</span> 연주 준비</h2><span class="tag">MIDI</span></div>
          <div id="dropzone" class="dropzone"><span class="upload-symbol">↥</span><strong id="file-name">드럼 MIDI를 불러오세요</strong><p id="file-detail">파일을 여기에 놓거나 선택하세요<br>최대 10 MiB · 5분</p><button id="upload" class="primary-button">MIDI 불러오기 <span>＋</span></button><input id="file" type="file" accept=".mid,.midi,audio/midi,audio/x-midi" hidden/></div>
          <button id="demo" class="demo-button"><span>▷</span> 30초 데모로 시작 <span>↗</span></button>
          <div id="selection" hidden><div class="field-label">연주할 트랙</div><div id="tracks" class="track-list"></div><label class="channel-row">MIDI 채널<select id="channel"><option value="all">전체 채널</option></select></label><button id="replan" class="secondary-button">선택한 트랙으로 다시 준비</button></div>
          <button id="cancel-analysis" class="text-button" hidden>분석 취소</button>
          <p id="message" class="message" role="status" aria-live="polite">데모로 업로드부터 영상 출력까지 확인해 보세요.</p>
        </section>
        <section class="panel details-panel"><div class="panel-heading"><h2><span>02</span> 연주 정보</h2><span id="plan-label" class="tag">—</span></div><dl class="metrics"><div><dt>곡 길이</dt><dd id="metric-duration">—</dd></div><div><dt>타격</dt><dd id="metric-hits">—</dd></div><div><dt>템포 구간</dt><dd id="metric-tempos">—</dd></div></dl><details id="diagnostics" hidden><summary id="diagnostic-summary"></summary><ul id="diagnostic-list"></ul></details><p class="footnote">기본 리그 · 오른손 주도 · 합성 드럼 음향</p></section>
        <section class="panel export-panel"><div class="panel-heading"><h2><span>03</span> 영상 저장</h2><span class="tag">720p</span></div><div class="export-spec"><strong>1280 × 720</strong><span>30 FPS</span></div><label class="mute-option"><input type="checkbox" id="mute-export"/> 오디오 없이 내보내기</label><button id="export" class="export-button" disabled>영상 추출 <span>↓</span></button><div id="export-progress" hidden><progress id="progress" max="1" value="0"></progress><div class="progress-label"><span id="progress-text">인코더 확인 중…</span><button id="cancel-export" class="text-button">취소</button></div></div><a id="download" class="download-link" hidden>완성된 영상 다운로드 ↓</a><p id="export-note" class="footnote">지원 환경에 따라 MP4 또는 WebM으로 저장합니다.<br>시작 준비 0.5초 · 끝 여운 0.25초 포함</p></section>
      </aside>
    </div>
  </main>
  <footer><span>DRUM / VRM <span class="version">MVP 0.1</span></span><span>Rust · WASM · Three.js</span><a href="./licenses.txt" target="_blank">오픈소스 라이선스 ↗</a></footer>`;

const get = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
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
const clockText = (seconds: number) => `${Math.floor(Math.max(0, seconds) / 60).toString().padStart(2,'0')}:${Math.floor(Math.max(0,seconds) % 60).toString().padStart(2,'0')}`;
function status(text: string, kind = '') { get('status').className = `status ${kind}`; get('status').lastElementChild!.textContent = text; }
function message(text: string, error = false) { const node = get('message'); node.textContent = text; node.classList.toggle('error', error); }
function controls() {
  for (const id of ['play','reset','seek','export']) (get(id) as HTMLButtonElement).disabled = !plan || busy || exporting;
  for (const id of ['upload','demo','replan','channel','mute-export','background']) (get(id) as HTMLInputElement).disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting;
  get('cancel-analysis').hidden = !busy;
  get('export-progress').hidden = !exporting;
}
function updateFrame(time: number) {
  if (!plan) return;
  const pose = engine.evaluate(time - PRE_ROLL);
  if (pose) scene.applyPose(pose, time - PRE_ROLL);
}
function drawTimeline() {
  const canvas = get<HTMLCanvasElement>('timeline');
  const width = Math.max(1,canvas.clientWidth); canvas.width = width * devicePixelRatio; canvas.height = 58 * devicePixelRatio;
  const context = canvas.getContext('2d')!; context.scale(devicePixelRatio,devicePixelRatio);
  const colors = ['#d8b77a','#a7cbb1','#81aca6','#b5a2c7'];
  for (let i = 0; i < 4; i++) { context.strokeStyle = '#ffffff0b'; context.beginPath(); context.moveTo(0,10+i*12); context.lineTo(width,10+i*12); context.stroke(); }
  if (!plan) return;
  const total = outputDuration(plan);
  for (const hit of plan.hits) { const lane = hit.limb ?? 0; context.fillStyle = hit.limb === null ? '#d67c69' : colors[lane]; const x = (hit.time+PRE_ROLL)/total*width; context.globalAlpha = .4+hit.velocity/127*.6; context.fillRect(x,5+lane*12,Math.max(2,width/500),9); }
  context.globalAlpha = 1;
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
  get('duration').textContent = clockText(total); get('timeline-end').textContent = clockText(total);
  get('metric-duration').textContent = clockText(plan.duration); get('metric-hits').textContent = plan.hits.length.toLocaleString(); get('metric-tempos').textContent = String(plan.tempoCount);
  get('plan-label').textContent = 'READY'; get('stage-empty').hidden = true;
  get('timeline-caption').textContent = '양손 · 양발 타격 타임라인';
  const diagnosticList = get('diagnostic-list'); diagnosticList.replaceChildren();
  const unresolved = plan.hits.filter(h=>h.limb === null).length;
  get('diagnostics').hidden = !plan.diagnostics.length;
  get('diagnostic-summary').textContent = `분석 알림 ${plan.diagnostics.length}개${unresolved?` · 표현하지 못한 타격 ${unresolved}개`:''}`;
  for (const item of plan.diagnostics.slice(0,100)) { const li=document.createElement('li'); li.textContent=`${item.time !== undefined ? clockText(item.time)+' · ':''}${item.message}`; diagnosticList.append(li); }
  if(plan.diagnostics.length>100){const li=document.createElement('li');li.textContent=`외 ${plan.diagnostics.length-100}개 알림`;diagnosticList.append(li);}
  status('준비 완료','ready'); message(unresolved?`준비 완료. 표현하지 못한 ${unresolved}개 타격은 알림을 확인해 주세요.`:'준비가 끝났습니다. 재생 버튼을 눌러 연주를 시작하세요.'); drawTimeline();
}
function clearPerformance() {
  audio?.dispose(); audio=null; buffer=null; plan=null;
  for (const id of ['metric-duration','metric-hits','metric-tempos','plan-label']) get(id).textContent='—';
  for (const id of ['duration','current-time','timeline-end']) get(id).textContent='00:00';
  get('stage-empty').hidden=false; get('diagnostics').hidden=true;
  get('timeline-caption').textContent='MIDI를 불러오면 연주 타임라인이 표시됩니다.';
  get<HTMLInputElement>('seek').value='0'; get('playhead').style.left='0%';
  get('play').textContent='▶'; get('play').setAttribute('aria-label','재생');
  if(downloadUrl)URL.revokeObjectURL(downloadUrl); downloadUrl=null; get('download').hidden=true;
  drawTimeline();
}
async function loadFile(data: Uint8Array, name: string) {
  if (exporting) return;
  const generation = ++loadGeneration;
  engine.cancel(); synthesis.cancel(); clearPerformance();
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
for(const button of document.querySelectorAll<HTMLButtonElement>('[data-camera]'))button.onclick=()=>{scene.setCamera(button.dataset.camera as 'stage'|'front'|'top');document.querySelector('[data-camera].selected')?.classList.remove('selected');button.classList.add('selected');};
get<HTMLInputElement>('background').oninput=()=>scene.setBackground(get<HTMLInputElement>('background').value);
get('cancel-export').onclick=()=>exportController?.abort();
get('export').onclick=async()=>{
  if(!plan||!buffer||exporting)return;
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
    message('영상이 완성됐습니다. 아래 다운로드 링크에서 저장하세요.');status('영상 준비 완료','ready');
  } catch(e){const cancelled=e instanceof DOMException&&e.name==='AbortError';message(cancelled?'영상 추출을 취소했습니다. 다시 재생하거나 내보낼 수 있습니다.':`영상 추출 실패: ${e instanceof Error?e.message:String(e)}`,!cancelled);status(cancelled?'준비 완료':'출력 확인 필요',cancelled?'ready':'error');}
  finally {exporting=false;exportController=null;scene.setExportMode(false);scene.resize();updateFrame(restoreTime);controls();}
};
document.addEventListener('keydown',e=>{if(e.code==='Space'&&!/INPUT|SELECT|BUTTON|TEXTAREA/.test((e.target as HTMLElement).tagName)){e.preventDefault();get<HTMLButtonElement>('play').click();}});
const resizeObserver=new ResizeObserver(()=>{if(!exporting){scene.resize();drawTimeline();}});resizeObserver.observe(get('stage-view'));
let disposed=false;
function tick(){if(disposed)return;if(!exporting){const time=audio?.currentTime??0;if(plan){updateFrame(time);get<HTMLInputElement>('seek').value=String(time);get('current-time').textContent=clockText(time);get('playhead').style.left=`${time/outputDuration(plan)*100}%`;const playing=audio?.playing??false;get('play').textContent=playing?'Ⅱ':'▶';get('play').setAttribute('aria-label',playing?'일시정지':'재생');if(!playing&&get('status').classList.contains('playing'))status(time>=outputDuration(plan)-.02?'연주 완료':'일시정지',time>=outputDuration(plan)-.02?'ready':'');}scene.render();}requestAnimationFrame(tick);}tick();
controls();drawTimeline();
window.addEventListener('beforeunload',()=>{disposed=true;engine.dispose();synthesis.cancel();audio?.dispose();scene.dispose();if(downloadUrl)URL.revokeObjectURL(downloadUrl);});
