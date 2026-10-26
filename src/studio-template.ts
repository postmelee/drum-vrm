/** Action-first workspace. Detailed diagnostics live in Analysis. */
export const studioTemplate = `  <header class="toolbar">
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
      <button data-panel="source" aria-expanded="false" aria-controls="panel-source">파일 · 오디오</button>
      <button data-panel="analysis" aria-expanded="false" aria-controls="panel-analysis">분석<span id="analysis-badge" hidden></span></button>
      <button data-panel="export" aria-expanded="false" aria-controls="panel-export">내보내기</button>
      <button id="theme" aria-label="다크 모드로 전환" title="테마 전환">다크</button>
    </nav>
  </header>
  <main class="workspace">
    <section class="studio" aria-label="3D 드럼 연주">
      <div class="stage-bar"><span id="status" role="status">대기</span><button id="cancel-fit" hidden>맞춤 취소</button><div class="view-controls" aria-label="카메라 시점"><button data-camera="stage" aria-pressed="true">연주</button><button data-camera="front" aria-pressed="false">정면</button><button data-camera="top" aria-pressed="false">위</button><button data-camera="player" aria-pressed="false">연주자</button></div></div>
      <div class="stage-view" id="stage-view"><canvas id="stage" aria-label="드럼 세트와 연주 스켈레톤"></canvas><div id="stage-empty" class="stage-empty"><button id="start-upload">MIDI 불러오기</button><button id="start-demo">데모 재생 준비</button></div></div>
      <p id="message" class="message" role="status" aria-live="polite">MIDI를 선택하면 연주를 준비합니다.</p>
    </section>
    <aside id="inspector" class="inspector" hidden>
      <div class="inspector-heading"><h2 id="panel-title"></h2><button id="close-panel" aria-label="설정 닫기" title="닫기 (Esc)"></button></div>
      <section id="panel-character" class="panel" hidden>
        <label class="field">표시<select id="character-display"><option value="model">모델</option><option value="skeleton" selected>뼈대</option><option value="both">모델 + 뼈대</option><option value="hidden">숨김</option></select></label>
        <div hidden><input id="character-visible" type="checkbox" checked/><input id="skeleton-visible" type="checkbox" checked/><input id="model-visible" type="checkbox"/></div>
        <div class="button-row"><button id="character-import">VRM 불러오기…</button><button id="character-reset">기본 리그</button></div><input id="character-file" type="file" accept=".vrm" hidden/>
        <p id="character-message" class="hint" role="status">VRM 1.0 표준 본 · 기본 스켈레톤</p>
        <p class="hint">VRM 1.0 · 최대 50 MiB</p>
        <div id="character-info"></div>
        <div class="button-row"><button id="kit-fit">자동 맞춤</button><button id="kit-fit-restore" disabled>맞춤 되돌리기</button></div>
        <p id="fit-state" class="hint" role="status">기본 배치</p>
        <p id="calibration-message" class="hint" role="status"></p>
        <button data-open-analysis>분석 보기</button>
        <details><summary>수동 보정</summary>
        <label class="field">손바닥 방향<select id="palm-orientation"><option value="up">위</option><option value="inward">안쪽 · 엄지 위</option><option value="down">아래</option></select></label>

        <h3>좌석</h3>
        <label class="field">높이 (cm)<input id="seat-height" type="number" min="30" max="120" step="0.1" required/></label>
        <label class="field">앞뒤 위치 (cm · 앞쪽 +)<input id="seat-forward" type="number" min="-50" max="50" step="0.1" required/></label>
        <div class="button-row"><button id="seat-apply">좌석 적용</button><button id="seat-auto">좌석만 자동 조정</button></div>
        <button id="kit-layout-reset">드럼을 기본 배치로</button>
        </details>
        <details><summary>파일 · 모델 안내</summary><p class="hint">.vroid 편집 파일은 VRM 1.0으로 내보내세요. 머리카락·옷 물리는 재생과 탐색의 일치를 위해 정지합니다.</p><a href="https://github.com/postmelee/drum-vrm/blob/main/docs/models.ko.md" target="_blank" rel="noreferrer">모델 확보 · 업로드 안내 / English</a></details>
      </section>
      <section id="panel-drums" class="panel" hidden>
        <label class="field">드럼 세트<select id="kit-preset"><option value="studio">기본 · Studio</option><option value="vintage">기본 · Vintage</option><option value="electronic">기본 · Electronic</option><option value="library">어쿠스틱 · Innerscene</option><option value="batoski">어쿠스틱 · Batoski</option><option value="smoj">로우 폴리 · smoj</option></select></label>
        <label class="field">부품<select id="piece"><option value="kick">Kick</option><option value="snare">Snare</option><option value="hihat">Hi-hat</option><option value="tomHigh">High tom</option><option value="tomMid">Mid tom</option><option value="tomLow">Low tom</option><option value="crash">Crash L</option><option value="crashRight">Crash R</option><option value="ride">Ride</option></select></label>
        <details><summary>배치 · 타점 수동 조정</summary>
          <label class="field">조절할 타점<select id="target-piece"><option value="kick">Kick 페달 · 킥 배치</option><option value="snare">Snare</option><option value="hihat">Hi-hat 심벌</option><option value="pedalHat">Hi-hat 페달</option><option value="tomHigh">High tom</option><option value="tomMid">Mid tom</option><option value="tomLow">Low tom</option><option value="crash">Crash L</option><option value="crashRight">Crash R</option><option value="ride">Ride</option></select></label>
          <label class="field">좌우 (cm · 연주자 오른쪽 +)<input id="target-x" type="number" min="-200" max="200" step="1"/></label>
          <label class="field">높이 (cm)<input id="target-y" type="number" min="2" max="250" step="1"/></label>
          <label class="field">앞뒤 (cm · 앞쪽 +)<input id="target-z" type="number" min="-50" max="250" step="1"/></label>
          <div class="button-row"><button id="target-apply">타점 적용</button><button id="target-reset">타점 초기화</button></div>
          <p id="target-message" class="hint" role="status">실제 배치와 손발 계획을 함께 변경합니다. 열린·닫힌 하이햇은 같은 타점을 사용합니다.</p>
        </details>
        <label class="field">기본 모델<select id="piece-model"><option value="asset" disabled>프리셋 / 가져온 모델</option><option value="standard">표준</option><option value="deep">깊은 셸</option><option value="compact">컴팩트</option></select></label>
        <button id="piece-reset">부품을 프리셋으로 복원</button><button id="piece-import">이 부품의 GLB 불러오기…</button><input id="model-file" type="file" accept=".glb" hidden/>
        <p id="model-message" class="hint" role="status">정적 GLB · 최대 25 MiB</p>
        <details><summary>외형 · 타격 반응</summary>
        <label class="field horizontal">색상<input id="piece-finish" type="color" value="#6b7280"/></label>
        <label class="field">외형 크기 <output id="piece-scale-value">100%</output><input id="piece-scale" type="range" min="0.75" max="1.25" step="0.01" value="1"/></label>
        <label class="field">회전 <output id="piece-rotation-value">0°</output><input id="piece-rotation" aria-label="부품 회전" type="range" min="-180" max="180" step="1" value="0"/></label>
        <label class="check"><input id="hit-response" type="checkbox"/> 타격 반응</label><label class="field">반응 강도<input id="hit-strength" aria-label="타격 반응 강도" type="range" min="0" max="1" step="0.05" value="0.65"/></label>
        <p class="hint">외형 설정은 타점과 연주 배치를 바꾸지 않습니다.</p></details>
        <details><summary>모델 출처 · 파일 안내</summary><p>받침대·페달이 없는 부품 GLB를 사용하세요. 모델 교체 후 표면과 타점을 확인하세요.</p><p>기본 3종은 자체 생성 도형입니다. Innerscene은 CC0, Batoski와 smoj는 CC BY 3.0 모델입니다. 부품별로 크기와 타점을 맞춥니다.</p><p>스킨·뼈대가 있는 GLB와 GLTF·FBX·OBJ·BLEND는 직접 지원하지 않습니다. 외부 파일 참조 없는 GLB로 변환하세요.</p><a href="https://github.com/postmelee/drum-vrm/blob/main/docs/research/drum-assets.md" target="_blank" rel="noreferrer">모델 조사 · 라이선스</a></details>
      </section>
      <section id="panel-source" class="panel" hidden>
        <h3 class="section-first">프로젝트</h3><div class="button-row"><button id="project-save" disabled>프로젝트 저장</button><button id="project-open">프로젝트 열기…</button><button id="project-relink" hidden>원본 파일 연결…</button></div><input id="project-file" type="file" accept=".json" hidden/><input id="project-assets" type="file" accept=".mid,.midi,.wav,.vrm,.glb" multiple hidden/><p id="project-message" class="hint" role="status">개인 프로젝트 파일 · 다시 열 때 원본 파일 연결</p><div id="dropzone"><strong id="file-name">MIDI 파일 선택</strong><p id="file-detail" class="hint">여기에 드롭 · 최대 10 MiB / 5분</p><div class="button-row"><button id="upload">MIDI 불러오기…</button></div><input id="file" type="file" accept=".mid,.midi,audio/midi,audio/x-midi" hidden/></div>
        <p id="source-summary" class="hint" role="status"></p><button id="source-analysis" data-open-analysis hidden>분석 보기</button>
        <details id="midi-settings"><summary>MIDI 설정</summary><div id="selection" hidden><h3>트랙</h3><div id="tracks" class="track-list"></div><label class="field">MIDI 채널<select id="channel"><option value="all">전체 채널</option></select></label><label class="field">드럼 음원 매핑<select id="mapping-profile"><option value="gm">General MIDI</option><option value="logicDkdGm">Logic · Drum Kit Designer (GM)</option></select></label><p id="mapping-evidence" class="hint"></p><details id="interpretation-options"><summary>연주 의도 해석</summary><label id="count-in-choice" class="check" hidden><input id="count-in-75" type="checkbox"/><span id="count-in-label">카운트인으로 해석</span></label><div id="hihat-choice" hidden><label class="check"><input id="hihat-policy" type="checkbox"/> 짧은 열기·닫기를 작은 개방 동작으로 연결</label><label id="hihat-openness-field" class="field" hidden>하이햇 열림 (%)<input id="hihat-openness" type="number" min="5" max="100" step="1" value="20" disabled/></label><p id="hihat-candidates" class="hint"></p><p class="hint">실제 발 타격일 수도 있으므로 의도한 경우에만 선택하세요. 원본 노트는 보존되며, 열림 값은 실제 심벌 간격이나 음색의 측정값이 아닙니다.</p></div><p id="interpretation-empty" class="hint">준비한 트랙에 추가 해석 후보가 없습니다.</p><p class="hint">75번은 다른 음원에서 클라베일 수 있습니다. 원래 음원의 매핑과 연주 의도를 확인하세요. 선택은 다시 준비한 뒤 적용됩니다.</p></details><details><summary>지원하는 매핑</summary><p class="hint">General MIDI 및 Drum Kit Designer의 GM 모드를 지원합니다. GM + ModWheel·V-Drum·장치별 연속 컨트롤러는 지원하지 않습니다. 파일명이나 트랙명으로 제작 프로그램을 확정하지 않습니다.</p></details><button id="replan">설정 적용</button></div></details>
        <h3>연주 오디오</h3><div class="button-row"><button id="wav-import">WAV 불러오기…</button><button id="wav-remove" disabled>합성음으로 복원</button></div><input id="wav-file" type="file" accept=".wav,audio/wav" hidden/>
        <label class="field">WAV 시작 위치 (초 · MIDI 기준)<input id="wav-offset" type="number" min="-300" max="300" step="0.001" value="0"/></label><button id="wav-apply">오프셋 적용</button>
        <p id="wav-info" class="hint" role="status">합성 드럼 오디오 · WAV를 불러오면 이 소리를 대체합니다.</p><details><summary>오디오 · 파일 안내</summary><p class="hint">양수는 WAV를 늦게 시작합니다. 준비 0.5초는 자동으로 더해집니다. 곡 범위 밖은 잘리고, 부족한 부분은 무음입니다. PCM / float WAV · 모노·스테레오 · 최대 100 MiB / 5분. 프로젝트에는 MIDI 노트·계획이 포함되며 WAV·VRM·GLB 원본은 별도로 보관하세요.</p></details>
        <button id="cancel-analysis" hidden>준비 취소</button>
        <details><summary>데모</summary><div class="button-row"><button id="demo">30초 데모</button><button id="verification-demo">연주 검증 패턴</button></div></details>
        <p class="hint">파일은 브라우저 안에서 처리됩니다.</p>
      </section>
      <section id="panel-analysis" class="panel" hidden>
        <p id="fit-details" class="hint"></p>
        <details id="fit-capabilities" hidden><summary>배치 · 양손 조합 검사</summary><p id="fit-capability-summary" class="hint"></p><ul id="fit-capability-list"></ul></details>
        <p id="analysis-summary" role="status">MIDI를 불러오면 분석 결과를 확인할 수 있습니다.</p>
        <button id="analysis-source">MIDI 설정 열기</button>
        <h3>연주 정보 <span id="plan-label">—</span></h3><dl class="metrics"><div><dt>곡 길이</dt><dd id="metric-duration">—</dd></div><div><dt>타격</dt><dd id="metric-hits">—</dd></div><div><dt>템포 구간</dt><dd id="metric-tempos">—</dd></div></dl>
        <p id="interpretation-summary" class="hint"></p><p id="setup-summary" class="hint"></p><details id="performance-review"><summary>자세 · 동기화 검수</summary><label class="field">곡 시각 (초)<input id="review-time" type="number" min="0" max="300" step="0.001" value="0"/></label><div class="button-row"><button id="review-jump">이동</button><button id="review-previous">이전 타격</button><button id="review-next">다음 타격</button></div><div class="button-row"><button id="review-current">현재 자세 검사</button><button id="review-trace">현재 스틱 궤적</button><button id="review-trace-hide">궤적 숨기기</button><button id="review-scan">곡 검수</button><button id="review-download">검수 JSON 저장</button></div><p id="review-status" class="hint" role="status">곡 검수는 30fps와 타격 전후 표본의 재생·임의 탐색 자세를 비교합니다.</p><p id="review-measurement" class="hint"></p><p class="hint">검수 파일에는 배치·체형·수치 결과가 저장됩니다. MIDI 노트 원문과 업로드 파일은 포함되지 않으며 서버로 전송하지 않습니다.</p></details><details id="diagnostics" hidden><summary id="diagnostic-summary"></summary><ul id="diagnostic-list"></ul></details>
        <details id="arrangement-editor"><summary>선택 음표의 연주 편집</summary><p class="hint">피아노 롤의 음표를 클릭하거나 현재 시각 주변 음표를 찾으세요. 원본 MIDI와 WAV는 바꾸지 않습니다.</p><div class="button-row"><button id="arrangement-nearest">현재 시각 음표</button><button id="arrangement-previous">이전 음표</button><button id="arrangement-next">다음 음표</button></div><label class="field">원본 음표<select id="arrangement-event"><option value="">선택 없음</option></select></label><p id="arrangement-detail" class="hint"></p><label class="check"><input id="arrangement-omit" type="checkbox"/> 이 음표를 연주에서 제외</label><label class="field">연주 손<select id="arrangement-hand"><option value="auto">자동</option><option value="0">왼손</option><option value="1">오른손</option></select></label><div class="button-row"><button id="arrangement-apply">음표 편집 적용</button><button id="arrangement-reset">모든 음표 편집 복원</button></div><p id="arrangement-status" class="hint" role="status">손 지정이 불가능한 경우 원본 타격을 보존하고 동작 미배정으로 알립니다. 편집은 프로젝트에 저장할 수 있습니다.</p></details>
        <details><summary>접촉 계측</summary><p id="contact-info" class="hint">실제 손에 부착한 스틱 끝과 발의 기준점을 계획과 비교합니다.</p><p class="hint">발 기준점은 신발 표면의 충돌 검사가 아닙니다. 도달하지 못한 타격은 분석 알림에 표시됩니다.</p></details>
        <p class="hint">근사 형상 검사는 피부·의상 충돌의 보장이 아닙니다.</p>
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
  <div id="toast" class="toast" hidden><span id="toast-message" role="status" aria-live="polite"></span><button id="toast-dismiss" aria-label="알림 닫기">닫기</button></div>
  <section class="editor" aria-label="MIDI 미리보기"><div class="editor-toolbar"><strong>Piano Roll</strong><span class="hint">미리보기</span><input id="seek" aria-label="재생 위치" type="range" min="0" max="1" step="0.001" value="0" disabled/><label>확대 <input id="timeline-zoom" aria-label="MIDI 확대" type="range" min="1" max="16" step="0.25" value="1"/></label></div><div id="timeline"></div></section>
`;
