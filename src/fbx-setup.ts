import { createHelp } from './ui-help';
import { materialAppearanceLabel, materialAppearanceDescription } from './material-appearance';
import { bindAttribute, bindText, joinMessages, msg, raw } from './i18n';
import { engineMessage } from './engine-messages';
import { FBX_BONES, FBX_REQUIRED, imageKey, type FBXMaterialMode, validateFBXConfig, type FBXConfig } from './fbx-config';
import { availableFBXBones, matchingFBXBones, fbxBones, resolveFBXBones, suggestFBXBones } from './fbx-rig';
import { loadLocalFBX, type LoadedFBX } from './fbx-import';
import { createFBXPreview } from './fbx-preview';
import { savedFBXSetup, suggestFBXTexture } from './fbx-preferences';

function boneLabel(name: string): string {
  const central: Record<string, [string, string]> = { hips:['Hips','골반'], spine:['Spine','척추'], chest:['Chest','가슴'], upperChest:['Upper chest','위 가슴'], neck:['Neck','목'], head:['Head','머리'], jaw:['Jaw','턱'] };
  if (central[name]) return msg(...central[name]);
  const side = name.startsWith('left') ? msg('Left','왼쪽') : msg('Right','오른쪽');
  const part = name.replace(/^(left|right)/, '');
  const parts: Record<string, [string,string]> = { UpperArm:['upper arm','위팔'], LowerArm:['forearm','아래팔'], Hand:['hand','손'], UpperLeg:['thigh','허벅지'], LowerLeg:['shin','종아리'], Foot:['foot','발'], Toes:['toes','발가락'], Shoulder:['shoulder','어깨'], Eye:['eye','눈'] };
  if (parts[part]) return msg('{0} {1}', '{0} {1}', side, msg(...parts[part]));
  const finger = part.match(/^(Thumb|Index|Middle|Ring|Little)(Metacarpal|Proximal|Intermediate|Distal)$/);
  const fingers: Record<string,[string,string]> = { Thumb:['thumb','엄지'], Index:['index','검지'], Middle:['middle','중지'], Ring:['ring','약지'], Little:['little finger','소지'] };
  const joints: Record<string,[string,string]> = { Metacarpal:['metacarpal','손허리뼈'], Proximal:['proximal','첫 마디'], Intermediate:['intermediate','중간 마디'], Distal:['distal','끝 마디'] };
  return finger ? msg('{0} {1} {2}', '{0} {1} {2}', side, msg(...fingers[finger[1]]), msg(...joints[finger[2]])) : name;
}

const text = <K extends keyof HTMLElementTagNameMap>(tag: K, en: string, ko: string) => {
  const element = document.createElement(tag);
  const target = tag === 'label' ? document.createElement('span') : element;
  bindText(target, msg(en, ko)); if (target !== element) element.append(target); return element;
};
const option = (label: string | ReturnType<typeof raw>, value: string) => { const element = document.createElement('option'); bindText(element, label); element.value = value; return element; };

/** A staged import; nothing touches the active character until confirmation. */
export function setupFBX(file: File, companions: File[] = []): Promise<{ loaded: LoadedFBX; config: FBXConfig } | null> {
  return new Promise(resolve => {
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const dialog = document.createElement('dialog'); dialog.className = 'fbx-setup'; dialog.setAttribute('aria-labelledby', 'fbx-title');
    const title = text('h2', 'Import FBX character', 'FBX 캐릭터 불러오기'); title.id = 'fbx-title';

    const filename = document.createElement('p'); filename.textContent = file.name;
    const picker = document.createElement('input'); picker.type = 'file'; picker.multiple = true; picker.accept = '.png,.jpg,.jpeg'; picker.id = 'fbx-images';
    const label = text('label', 'Companion images (optional)', '함께 사용할 이미지 (선택)'); label.htmlFor = picker.id;
    const selected = document.createElement('p'); selected.className = 'hint';
    const showFiles = () => { selected.textContent = companions.map(f => f.name).join(', '); }; showFiles();
    picker.onchange = () => { companions = [...picker.files ?? []]; showFiles(); };
    const files = document.createElement('div'); files.append(filename, label, picker, selected);
    const fields = document.createElement('div'), error = document.createElement('p'); error.setAttribute('role', 'status'); error.className = 'field-status';
    const actions = document.createElement('div'); actions.className = 'button-row fbx-actions';
    const cancel = text('button', 'Cancel', '취소'), next = text('button', 'Inspect file', '파일 검사');
    actions.append(cancel, next); dialog.append(title, files, fields, error, actions); document.body.append(dialog);
    let loaded: LoadedFBX | null = null, finished = false, busy = false;
    let preview: ReturnType<typeof createFBXPreview> | undefined;
    let textureWork: Promise<void> = Promise.resolve();
    let boneInputs: { name: typeof FBX_BONES[number]; input: HTMLSelectElement }[] = [], unit: HTMLInputElement, materialMode: HTMLSelectElement;
    const textureInputs: HTMLSelectElement[] = [];
    const mapping = (): FBXConfig['bones'] => Object.fromEntries(boneInputs.flatMap(({ name, input }) => input.value ? [[name, input.value]] : []));
    const config = (): FBXConfig => ({ materialMode: materialMode.value as FBXMaterialMode, bones: mapping(), metersPerUnit: Number(unit.value), textures: Object.fromEntries(textureInputs.flatMap((select, index) => select.value ? [[String(index), select.value]] : [])) });
    const finish = async (value: { loaded: LoadedFBX; config: FBXConfig } | null) => {
      if (finished) return; finished = true; next.disabled = true; cancel.disabled = true;
      await textureWork; preview?.dispose();
      if (!value) loaded?.dispose(); dialog.close(); dialog.remove(); previousFocus?.focus(); resolve(value);
    };
    cancel.onclick = () => { void finish(null); }; dialog.oncancel = event => { event.preventDefault(); void finish(null); };
    const render = async () => {
      const source = loaded!, candidates = fbxBones(source.scene), suggested = suggestFBXBones(candidates);
      const digest = await crypto.subtle.digest('SHA-256', await file.arrayBuffer());
      const hash = [...new Uint8Array(digest)].map(b => b.toString(16).padStart(2, '0')).join('');
      if (finished) return;
      let saved = savedFBXSetup(hash);
      try { if (saved) resolveFBXBones(source.scene, saved); } catch { saved = undefined; }
      const initial = saved?.bones ?? suggested;
      files.hidden = true;
      const info = document.createElement('p'); info.textContent = file.name; fields.append(info);
      if (saved) fields.append(text('p', 'Saved settings restored', '저장한 설정 복원'));
      const guidance: string[] = [];
      const layout = document.createElement('div'); layout.className = 'fbx-layout';
      const visual = document.createElement('div'); visual.className = 'fbx-visual';
      const viewport = document.createElement('div'); viewport.className = 'fbx-preview';
      const highlighted = text('p', 'Bone preview', '본 미리보기'); highlighted.setAttribute('role', 'status');
      const resetView = text('button', 'Reset view', '시점 초기화');
      visual.append(viewport, highlighted, text('p', 'Drag to rotate · scroll to zoom', '드래그로 회전 · 스크롤로 확대'), resetView);
      const settings = document.createElement('div'); layout.append(visual, settings); fields.append(layout);
      const progress = document.createElement('p'); progress.setAttribute('role', 'status'); progress.className = 'field-status'; settings.append(progress);
      const pending = document.createElement('div'); settings.append(pending);
      const moreLabel = text('label', 'Show candidates with other names', '이름이 다른 후보도 표시'); moreLabel.className = 'fbx-more';
      const more = document.createElement('input'); more.type = 'checkbox'; moreLabel.append(more); settings.append(moreLabel);
      guidance.push(msg('Bone lists exclude used bones and incompatible ancestry. Review assigned and optional bones to change an earlier connection.', '본 목록은 사용 중인 본과 상하 관계가 맞지 않는 본을 제외합니다. 기존 연결은 연결된 본·선택 본에서 수정하세요.'));
      const assigned = document.createElement('details'); assigned.append(text('summary', 'Review assigned bones', '연결된 본 수정')); settings.append(assigned);
      const optional = document.createElement('details'); optional.append(text('summary', 'Optional bones / fingers', '선택 본 · 손가락')); settings.append(optional);
      const refresh = () => {
        const current = mapping();
        for (const { name, input } of boneInputs) {
          const value = input.value, available = availableFBXBones(candidates, current, name);
          const matches = new Set(matchingFBXBones(available, name).map(b => b.path));
          const shown = !more.checked && matches.size ? available.filter(b => matches.has(b.path) || b.path === value) : available;
          input.replaceChildren(option(msg('Choose a bone…', '본 선택…'), ''));
          for (const recommended of [true, false]) {
            const group = document.createElement('optgroup'); bindAttribute(group, 'label', recommended ? msg('Matching names', '이름이 일치하는 후보') : msg('Other available bones', '그 외 연결 가능한 본'));
            for (const bone of shown.filter(b => matches.has(b.path) === recommended)) {
              let label = `${bone.name || msg('Unnamed', '이름 없음')} · ${bone.node.parent?.name || msg('Root', '루트')}`;
              if (available.some(b => b !== bone && b.name === bone.name && b.node.parent?.name === bone.node.parent?.name)) label += ` [${bone.path}]`;
              group.append(option(raw(label), bone.path));
            }
            if (group.children.length) input.append(group);
          }
          input.value = value;
        }
        const missing = FBX_REQUIRED.filter(name => !current[name as keyof typeof current]);
        bindText(progress, missing.length ? msg('{0} required bones left: {1}', '필수 본 {0}개를 선택하세요: {1}', missing.length, joinMessages(missing.map(boneLabel), ', ')) : msg('Required bones: 15/15', '필수 본: 15/15'));
        next.disabled = busy || missing.length > 0;
      };
      more.onchange = refresh;
      boneInputs = FBX_BONES.map(name => {
        const row = document.createElement('label'), caption = document.createElement('span'); bindText(caption, boneLabel(name)); row.append(caption);
        const input = document.createElement('select'); input.id = `fbx-bone-${name}`; input.dataset.bone = name;
        if (initial[name]) input.append(option('', initial[name]!));
        row.append(input); (FBX_REQUIRED.includes(name) ? initial[name] ? assigned : pending : optional).append(row);
        const highlight = () => {
          const bone = candidates.find(b => b.path === input.value); preview?.highlight(bone?.node);
          bindText(highlighted, bone ? msg('{0}: {1}', '{0}: {1}', boneLabel(name), raw(bone.name)) : msg('Choose a candidate to see its position.', '후보를 선택하면 위치를 확인할 수 있습니다.'));
        };
        input.onchange = () => { error.textContent = ''; refresh(); highlight(); }; input.onfocus = highlight;
        return { name, input };
      });
      const advanced = document.createElement('details'); advanced.append(text('summary', 'Scale and file details', '크기 · 파일 정보'));
      const units = text('label', 'Metres per file unit', '파일 단위당 미터');
      unit = document.createElement('input'); unit.id = 'fbx-unit'; unit.type = 'number'; unit.min = '.000001'; unit.max = '10'; unit.step = 'any'; unit.value = String(saved?.metersPerUnit ?? source.metersPerUnit); units.append(unit); advanced.append(units);
      advanced.append(text('p', 'Centimetres: 0.01 · metres: 1. Use a standing rest pose.', '센티미터: 0.01 · 미터: 1. 서 있는 기본 자세를 사용하세요.'));
      const details = document.createElement('p'); bindText(details, msg('{0} bones · {1} materials · {2} embedded clips (not played).', '본 {0}개 · 재질 {1}개 · 내장 클립 {2}개 (재생하지 않음).', candidates.length, source.materials.length, source.animationCount)); advanced.append(details); settings.append(advanced);
      settings.append(text('h3', 'Colors and textures', '색상 · 텍스처'));
      guidance.push(msg('Texture connections are suggested from image names. Use the preview to check and change them.', '이미지 이름으로 텍스처 연결을 제안합니다. 미리보기에서 확인하고 수정하세요.'));
      const applyTextures = async () => {
        busy = true; next.disabled = true; materialMode.disabled = true; textureInputs.forEach(input => { input.disabled = true; });
        try { await source.applyTextures(config()); if (!finished) preview?.draw(); }
        catch (cause) { if (!finished) bindText(error, engineMessage(cause instanceof Error ? cause.message : String(cause))); }
        finally { busy = false; materialMode.disabled = false; textureInputs.forEach(input => { input.disabled = false; }); if (!finished) refresh(); }
      };
      const modeLabel = text('label', 'Material appearance', '재질 표현');
      materialMode = document.createElement('select'); materialMode.id = 'fbx-material-mode';
      for (const mode of ['original', 'matte', 'unlit'] as const) materialMode.append(option(materialAppearanceLabel(mode), mode));
      materialMode.value = saved?.materialMode ?? 'original';
      const modeDescription = document.createElement('p'); modeDescription.className = 'control-help'; modeDescription.id = 'fbx-mode-description';
      materialMode.setAttribute('aria-describedby', modeDescription.id);
      const describeMode = () => bindText(modeDescription, materialAppearanceDescription(materialMode.value as FBXMaterialMode));
      describeMode();
      materialMode.onchange = () => { error.textContent = ''; describeMode(); textureWork = applyTextures(); };
      modeLabel.append(materialMode); settings.append(modeLabel, modeDescription);
      guidance.push(msg('Automatic follows supported file material properties; it cannot recover missing authoring-tool shaders or website lighting. Preview lighting differs from the performance scene. Unlit also disables tone mapping.', '자동 적용은 지원되는 파일 재질 속성을 따릅니다. 파일에 없는 제작 도구의 셰이더나 웹사이트 조명은 복원하지 않습니다. 미리보기와 연주 장면의 조명은 다르며, Unlit은 톤 매핑도 끕니다.'));
      source.materials.forEach((material, index) => {
        const row = document.createElement('label'); row.textContent = material.name || `Material ${index + 1}`;
        const select = document.createElement('select'); select.id = `fbx-material-${index}`;
        select.append(option(material.map ? msg('Keep original texture', '원래 텍스처 유지') : msg('No texture assigned', '연결된 텍스처 없음'), ''));
        for (const image of source.images) select.append(option(raw(image.name), imageKey(image.name)));
        const remembered = saved?.textures[String(index)];
        select.value = (saved ? remembered : material.map ? undefined : suggestFBXTexture(material.name, source.images.map(image => image.name))) ?? '';
        if (!select.value) select.value = '';
        select.onchange = () => { error.textContent = ''; textureWork = applyTextures(); };
        row.append(select); settings.append(row); textureInputs.push(select);
      });
      guidance.push(msg( 'Successful settings are remembered for this file in this browser (up to 10 files). Model and image contents are not stored. Keep a project and the source files for later use.', '성공한 설정은 이 브라우저에서 같은 파일에 재사용합니다(최근 10개). 모델·이미지 내용은 저장하지 않습니다. 나중에 사용하려면 프로젝트와 원본 파일을 보관하세요.'));
      guidance.push(msg( 'Original model terms apply. Toon shaders, physics and constraints are not reproduced.', '모델의 원본 이용 조건을 확인하세요. 툰 셰이더·물리·제약 설정은 재현하지 않습니다.'));
      settings.append(createHelp(msg('Import help', '불러오기 도움말'), guidance));
      try { preview = createFBXPreview(viewport, source.scene); resetView.onclick = () => preview?.reset(); }
      catch { bindText(viewport, msg('3D preview unavailable. Bone and texture selection is still available.', '3D 미리보기를 사용할 수 없습니다. 본과 텍스처는 계속 선택할 수 있습니다.')); }
      bindText(next, msg('Use character', '캐릭터 적용')); refresh();
      textureWork = applyTextures(); await textureWork;
      boneInputs.find(({ input }) => !input.value && FBX_REQUIRED.includes(input.dataset.bone!))?.input.focus();
    };
    next.onclick = async () => {
      if (busy || finished) return;
      try {
        if (!loaded) {
          busy = true; next.disabled = true; picker.disabled = true; bindText(error, msg('Inspecting FBX…', 'FBX를 검사하고 있습니다…'));
          const candidate = await loadLocalFBX(file, companions);
          if (finished) { candidate.dispose(); return; }
          loaded = candidate; error.textContent = ''; await render();
        } else {
          const chosen = validateFBXConfig(config()); resolveFBXBones(loaded.scene, chosen); void finish({ loaded, config: chosen });
        }
      } catch (cause) { if (!finished) bindText(error, engineMessage(cause instanceof Error ? cause.message : String(cause))); }
      finally {
        busy = false;
        if (!finished) { next.disabled = loaded ? FBX_REQUIRED.some(name => !mapping()[name as keyof FBXConfig['bones']]) : false; if (!loaded) picker.disabled = false; }
      }
    };
    dialog.showModal(); next.focus();
    // Folder/multi-file selection already supplies the images: proceed directly.
    if (companions.length) next.click();
  });
}
