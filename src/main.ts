import './style.css';
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
import { MAX_FILE_BYTES, PRE_ROLL, outputDuration, type PerformancePlan, type MidiInfo, type PieceId, type KitPiece, type KitFitProposal, type PoseInspection, type Vec3 } from './types';

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
        <div class="button-row"><button id="kit-fit">체형에 맞는 배치 제안</button><button id="kit-layout-reset">기본 배치</button></div><div class="button-row"><button id="kit-fit-apply" disabled>제안 적용</button><button id="kit-fit-restore" disabled>적용 전 배치로 복원</button></div><p id="fit-state" class="hint" role="status">기본 배치 · 체형별 배치 보정 전</p><p class="hint">손으로 치는 악기의 배치·크기를 함께 조정합니다. 킥과 페달은 유지됩니다. 피부·악기 간 충돌까지 보장하지 않으므로 화면에서 확인하세요.</p><p id="calibration-message" class="hint" role="status">팔·다리 길이를 유지하며 좌석과 타점을 조절합니다.</p>
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
        <details><summary>프로젝트 저장 · 열기</summary><div class="button-row"><button id="project-save" disabled>프로젝트 저장</button><button id="project-open">프로젝트 열기…</button><button id="project-relink" hidden>원본 파일 연결…</button></div><input id="project-file" type="file" accept=".json" hidden/><input id="project-assets" type="file" accept=".mid,.midi,.wav,.vrm,.glb" multiple hidden/><p id="project-message" class="hint" role="status">프로젝트 JSON에는 MIDI 노트·연주 계획과 설정이 포함됩니다. WAV·VRM·GLB 원본은 포함하지 않으며 다시 열 때 같은 파일을 연결합니다.</p></details><div id="dropzone"><strong id="file-name">MIDI 파일 선택</strong><p id="file-detail" class="hint">여기에 드롭 · 최대 10 MiB / 5분</p><div class="button-row"><button id="upload">MIDI 불러오기…</button><button id="demo">30초 데모</button><button id="verification-demo">연주 검증 패턴</button></div><input id="file" type="file" accept=".mid,.midi,audio/midi,audio/x-midi" hidden/></div>
        <div id="selection" hidden><h3>트랙</h3><div id="tracks" class="track-list"></div><label class="field">MIDI 채널<select id="channel"><option value="all">전체 채널</option></select></label><label class="field">드럼 음원 매핑<select id="mapping-profile"><option value="gm">General MIDI</option><option value="logicDkdGm">Logic · Drum Kit Designer (GM)</option></select></label><p id="mapping-evidence" class="hint"></p><details id="interpretation-options"><summary>연주 의도 해석</summary><label id="count-in-choice" class="check" hidden><input id="count-in-75" type="checkbox"/><span id="count-in-label">카운트인으로 해석</span></label><div id="hihat-choice" hidden><label class="check"><input id="hihat-policy" type="checkbox"/> 짧은 열기·닫기를 작은 개방 동작으로 연결</label><label id="hihat-openness-field" class="field" hidden>하이햇 열림 (%)<input id="hihat-openness" type="number" min="5" max="100" step="1" value="20" disabled/></label><p id="hihat-candidates" class="hint"></p><p class="hint">실제 발 타격일 수도 있으므로 의도한 경우에만 선택하세요. 원본 노트는 보존되며, 열림 값은 실제 심벌 간격이나 음색의 측정값이 아닙니다.</p></div><p id="interpretation-empty" class="hint">준비한 트랙에 추가 해석 후보가 없습니다.</p><p class="hint">75번은 다른 음원에서 클라베일 수 있습니다. 원래 음원의 매핑과 연주 의도를 확인하세요. 선택은 다시 준비한 뒤 적용됩니다.</p></details><details><summary>지원하는 매핑</summary><p class="hint">General MIDI 및 Drum Kit Designer의 GM 모드를 지원합니다. GM + ModWheel·V-Drum·장치별 연속 컨트롤러는 지원하지 않습니다. 파일명이나 트랙명으로 제작 프로그램을 확정하지 않습니다.</p></details><button id="replan">선택한 설정으로 다시 준비</button></div>
        <h3>연주 오디오</h3><div class="button-row"><button id="wav-import">WAV 불러오기…</button><button id="wav-remove" disabled>합성음으로 복원</button></div><input id="wav-file" type="file" accept=".wav,audio/wav" hidden/>
        <label class="field">WAV 시작 위치 (초 · MIDI 기준)<input id="wav-offset" type="number" min="-300" max="300" step="0.001" value="0"/></label><button id="wav-apply">오프셋 적용</button>
        <p id="wav-info" class="hint" role="status">합성 드럼 오디오 · WAV를 불러오면 이 소리를 대체합니다.</p><p class="hint">양수는 WAV를 늦게 시작합니다. 준비 0.5초는 자동으로 더해집니다. 곡 범위 밖은 잘리고, 부족한 부분은 무음입니다. PCM / float WAV · 모노·스테레오 · 최대 100 MiB / 5분.</p>
        <button id="cancel-analysis" hidden>준비 취소</button>
        <h3>연주 정보 <span id="plan-label">—</span></h3><dl class="metrics"><div><dt>곡 길이</dt><dd id="metric-duration">—</dd></div><div><dt>타격</dt><dd id="metric-hits">—</dd></div><div><dt>템포 구간</dt><dd id="metric-tempos">—</dd></div></dl>
        <p id="interpretation-summary" class="hint"></p><p id="setup-summary" class="hint"></p><details id="performance-review"><summary>자세 · 동기화 검수</summary><label class="field">곡 시각 (초)<input id="review-time" type="number" min="0" max="300" step="0.001" value="0"/></label><div class="button-row"><button id="review-jump">이동</button><button id="review-previous">이전 타격</button><button id="review-next">다음 타격</button></div><div class="button-row"><button id="review-current">현재 자세 검사</button><button id="review-trace">현재 스틱 궤적</button><button id="review-trace-hide">궤적 숨기기</button><button id="review-scan">곡 검수</button><button id="review-download">검수 JSON 저장</button></div><p id="review-status" class="hint" role="status">곡 검수는 30fps와 타격 전후 표본의 재생·임의 탐색 자세를 비교합니다.</p><p id="review-measurement" class="hint"></p><p class="hint">검수 파일에는 배치·체형·수치 결과가 저장됩니다. MIDI 노트 원문과 업로드 파일은 포함되지 않으며 서버로 전송하지 않습니다.</p></details><details id="diagnostics" hidden><summary id="diagnostic-summary"></summary><ul id="diagnostic-list"></ul></details>
        <details id="arrangement-editor"><summary>선택 음표의 연주 편집</summary><p class="hint">피아노 롤의 음표를 클릭하거나 현재 시각 주변 음표를 찾으세요. 원본 MIDI와 WAV는 바꾸지 않습니다.</p><div class="button-row"><button id="arrangement-nearest">현재 시각 음표</button><button id="arrangement-previous">이전 음표</button><button id="arrangement-next">다음 음표</button></div><label class="field">원본 음표<select id="arrangement-event"><option value="">선택 없음</option></select></label><p id="arrangement-detail" class="hint"></p><label class="check"><input id="arrangement-omit" type="checkbox"/> 이 음표를 연주에서 제외</label><label class="field">연주 손<select id="arrangement-hand"><option value="auto">자동</option><option value="0">왼손</option><option value="1">오른손</option></select></label><div class="button-row"><button id="arrangement-apply">음표 편집 적용</button><button id="arrangement-reset">모든 음표 편집 복원</button></div><p id="arrangement-status" class="hint" role="status">손 지정이 불가능한 경우 원본 타격을 보존하고 동작 미배정으로 알립니다. 편집은 프로젝트에 저장할 수 있습니다.</p></details>
        <p class="hint">파일은 브라우저 안에서 처리됩니다.</p>
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
let fitProposal: KitFitProposal | null = null;
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
type Panel = 'character' | 'drums' | 'source' | 'export';
let activePanel: Panel | null = null;
let modelBusy = false;
let committedPreset = 'studio';
let theme: 'light' | 'dark' = 'light';
try { theme = localStorage.getItem('drum-theme') === 'dark' ? 'dark' : 'light'; } catch { /* Storage is optional. */ }
const timeline = new DrumTimeline(get('timeline'), { onSeek: time => {
  if (!plan || busy || exporting || modelBusy) return;
  audio?.seek(time); updateFrame(time);
},onSourceInspect:id=>{
  if(!id){get<HTMLSelectElement>('arrangement-event').replaceChildren(new Option('선택 없음',''));get('arrangement-detail').textContent='';get<HTMLInputElement>('arrangement-omit').checked=false;get<HTMLSelectElement>('arrangement-hand').value='auto';}
  else if(plan&&!busy&&!exporting&&!modelBusy){selectArrangementEvent(id);openPanel('source');get<HTMLDetailsElement>('arrangement-editor').open=true;}
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
    fitProposal=null;poseReview=null;fitState='수동 보정 · 현재 배치';refreshFitState();invalidateVideo();
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
function refreshFitState() {
  get('fit-state').textContent=fitState;
  get('setup-summary').textContent=`배치: ${fitState}. 타격 배정과 몸·악기 간 여유는 별도로 검수하세요.`;
  controls();
}
async function proposeFit() {
  audio?.pause();modelBusy=true;fitProposal=null;controls();get('calibration-message').textContent='체형에 맞는 배치를 계산하고 있습니다…';
  try {
    const proposal=await engine.fitKit(scene.getRigProfile(),scene.getKitLayout());
    if(!proposal.complete){
      const reasons=[proposal.unreachablePieceIds.length?`도달·팔 자세: ${proposal.unreachablePieceIds.join(', ')}`:'',proposal.clearancePieceIds?.length?`몸·악기 간 여유: ${proposal.clearancePieceIds.join(', ')}`:''].filter(Boolean).join(' · ');
      get('calibration-message').textContent=`조건을 만족하는 배치를 찾지 못했습니다${reasons?` (${reasons})`:''}. 현재 배치를 유지합니다. 좌석과 타점을 조정해 주세요.`;return;
    }
    fitProposal=proposal;
    get('calibration-message').textContent=`배치 제안 준비 · 크기 ${(proposal.scale*100).toFixed(0)}% · 개별 타점 ${proposal.pieceAdjustments.length}개 조정. 제안 적용을 누르면 현재 배치를 바꿉니다.`;
  } catch(error){get('calibration-message').textContent=String(error);}
  finally{modelBusy=false;controls();}
}
get('kit-fit').onclick=()=>{if(!busy&&!exporting&&!modelBusy)void proposeFit();};
get('kit-fit-apply').onclick=()=>{
  if(!fitProposal||busy||exporting||modelBusy)return;
  const proposal=fitProposal, previous={seat:scene.getCalibration(),kit:scene.getKitLayout()};
  if(applyCalibration(previous.seat,proposal.kit,'calibration-message')){
    preFitSetup=previous;fitState='체형 배치 제안 적용';refreshFitState();
    get('calibration-message').textContent+=` · 크기 ${(proposal.scale*100).toFixed(0)}% · 개별 타점 ${proposal.pieceAdjustments.length}개 조정`;
  }
};
get('kit-fit-restore').onclick=()=>{
  if(!preFitSetup||busy||exporting||modelBusy)return;
  const previous=preFitSetup;
  if(applyCalibration(previous.seat,previous.kit,'calibration-message')){preFitSetup=null;fitState='이전 배치 복원';refreshFitState();}
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
  fitProposal=null;preFitSetup=null;poseReview=null;fitState='좌석만 자동 보정 · 체형 배치 적용 전';refreshFitState();invalidateVideo();refreshCalibration();
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
    adapter=new VRMHumanoidRigAdapter(candidate.vrm,`vrm1:${reference.sha256}`); adoptRig(adapter);
    candidate=null; adapter=null;
    commitProjectFile(reference,file);showModelInfo(info);
    get('character-message').textContent=`${file.name} · 본 매핑 완료. 아래 배치 제안을 적용한 뒤 연주를 확인하세요.`;
    await proposeFit();
  } catch(error) { if(adapter)adapter.dispose();else if(candidate)VRMUtils.deepDispose(candidate.vrm.scene); get('character-message').textContent=error instanceof Error?error.message:String(error); }
  finally{modelBusy=false;controls();}
};
get('character-reset').onclick=()=>{
  if(busy||exporting||modelBusy)return; audio?.pause();
  const adapter=new NativeHumanoidRigAdapter();
  try{adoptRig(adapter);sourceReferences=sourceReferences.filter(ref=>ref.kind!=='vrm');showModelInfo(null);get('character-message').textContent='VRM 1.0 표준 본 · 기본 스켈레톤';}
  catch(error){adapter.dispose();get('character-message').textContent=String(error);}
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
  for (const el of document.querySelectorAll<HTMLInputElement>('#tracks input')) el.disabled = busy || exporting;
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-camera]')) el.disabled = exporting || activePanel==='drums';
  for (const el of document.querySelectorAll<HTMLButtonElement>('[data-panel], #close-panel, #theme, #character-visible')) el.disabled=exporting || modelBusy;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-drums input, #panel-drums select, #panel-drums button')) el.disabled=busy || exporting || modelBusy;
  for (const id of ['start-upload','start-demo']) get<HTMLButtonElement>(id).disabled=busy || exporting;
  for (const el of document.querySelectorAll<HTMLInputElement>('#panel-character input, #panel-character select, #panel-character button')) el.disabled=busy || exporting || modelBusy;
  get<HTMLInputElement>('hit-strength').disabled = exporting || modelBusy || !get<HTMLInputElement>('hit-response').checked;
  get<HTMLButtonElement>('export').disabled = !plan || busy || exporting || modelBusy;
  get<HTMLInputElement>('count-in-75').disabled = busy || exporting || modelBusy;
  get<HTMLButtonElement>('wav-remove').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLButtonElement>('wav-apply').disabled = busy || exporting || modelBusy || !wav;
  get<HTMLInputElement>('hihat-policy').disabled=busy||exporting||modelBusy;
  get<HTMLInputElement>('hihat-openness').disabled=busy||exporting||modelBusy||!get<HTMLInputElement>('hihat-policy').checked;
  get('hihat-openness-field').hidden = !get<HTMLInputElement>('hihat-policy').checked;
  get<HTMLButtonElement>('kit-fit-apply').disabled=busy||exporting||modelBusy||!fitProposal;
  get<HTMLButtonElement>('kit-fit-restore').disabled=busy||exporting||modelBusy||!preFitSetup;
  for(const el of document.querySelectorAll<HTMLInputElement>('#performance-review input, #performance-review button'))el.disabled=!plan||busy||exporting||modelBusy;
  for(const el of document.querySelectorAll<HTMLInputElement>('#arrangement-editor input, #arrangement-editor select, #arrangement-editor button'))el.disabled=!plan||busy||exporting||modelBusy;
  updateArrangementControls();
  for(const id of ['project-open','project-relink'])get<HTMLButtonElement>(id).disabled=busy||exporting||modelBusy;
  get<HTMLButtonElement>('project-save').disabled=!plan||!preparedSelection||busy||exporting||modelBusy;
  get('cancel-analysis').hidden = !busy||projectBusy;
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
  get('interpretation-summary').textContent = `${plan.mappingProfile==='logicDkdGm'?'Logic DKD · GM':'General MIDI'} · 카운트인 ${countIn}개 · 제어 ${interpretations.filter(event=>event.kind==='control').length}개 · 미매핑 ${unmapped}개 · 제외 ${omitted}개 · 손 지정 ${Object.keys(plan.handOverrides??{}).length}개 · 동작 미배정 ${unresolved}개`;
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
  reviewGeneration++;poseReview=null;preparedSelection=null;audio?.dispose(); audio=null; buffer=null; plan=null;
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
    calibration:scene.getCalibration(),fitState,hasUnappliedFitProposal:!!fitProposal,
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
    scene.setCharacterVisible(project.view.character);scene.setSkeletonVisible(project.view.skeleton);scene.setModelVisible(project.view.model);
    get<HTMLInputElement>('hit-response').checked=project.hitResponse.enabled;get<HTMLInputElement>('hit-strength').value=String(project.hitResponse.strength);responseChosen=true;scene.setHitResponse(project.hitResponse);
    get<HTMLInputElement>('timeline-zoom').value=String(project.view.timelineZoom);timeline.setZoom(project.view.timelineZoom);
    scene.setViewState(project.camera);for(const button of document.querySelectorAll<HTMLButtonElement>('[data-camera]'))button.setAttribute('aria-pressed',String(button.dataset.camera===project.camera.view));
    scene.setPlan(plan);timeline.setPlan(plan);replaceAudio(nextBuffer,project.outputTime);showWavInfo(aligned?.info);showModelInfo(modelInfo);
    get('character-message').textContent=project.rig==='vrm'?'프로젝트 VRM · 같은 원본 파일과 체형 설정 복원':'기본 리그 · 프로젝트 체형 설정 복원';
    get<HTMLInputElement>('seek').max=String(outputDuration(plan));get('duration').textContent=clockText(outputDuration(plan));
    get('metric-duration').textContent=clockText(plan.duration);get('metric-hits').textContent=plan.hits.length.toLocaleString();get('metric-tempos').textContent=String(plan.tempoCount);get('plan-label').textContent='READY';get('stage-empty').hidden=true;
    fitProposal=null;preFitSetup=null;fitState='저장한 배치 복원';poseReview=null;refreshCalibration();refreshTarget();refreshPiece();renderDiagnostics();
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
controls();timeline.resize();
window.addEventListener('beforeunload',()=>{disposed=true;engine.dispose();synthesis.cancel();audio?.dispose();scene.dispose();timeline.dispose();resizeObserver.disconnect();if(downloadUrl)URL.revokeObjectURL(downloadUrl);});
