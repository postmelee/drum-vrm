import { bindText, msg } from './i18n';
import { engineMessage } from './engine-messages';
import { FBX_BONES, FBX_REQUIRED, imageKey, validateFBXConfig, type FBXConfig } from './fbx-config';
import { fbxBones, resolveFBXBones, suggestFBXBones } from './fbx-rig';
import { loadLocalFBX, type LoadedFBX } from './fbx-import';

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

/** A staged import; nothing touches the active character until confirmation. */
export function setupFBX(file: File, companions: File[] = []): Promise<{ loaded: LoadedFBX; config: FBXConfig } | null> {
  return new Promise(resolve => {
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const dialog = document.createElement('dialog'); dialog.className = 'fbx-setup'; dialog.setAttribute('aria-labelledby', 'fbx-title');
    const title = document.createElement('h2'); title.id = 'fbx-title'; bindText(title, msg('Import FBX character', 'FBX 캐릭터 불러오기'));
    const intro = document.createElement('p'); bindText(intro, msg('Select companion PNG/JPEG images, then check the bones, units and materials. The current character stays active until import succeeds.', '함께 사용할 PNG/JPEG 이미지를 선택한 뒤 본·단위·재질을 확인하세요. 가져오기가 완료될 때까지 현재 캐릭터를 유지합니다.'));
    const filename = document.createElement('p'); filename.textContent = file.name;
    const picker = document.createElement('input'); picker.type = 'file'; picker.multiple = true; picker.accept = '.png,.jpg,.jpeg'; picker.id = 'fbx-images';
    const label = document.createElement('label'); label.htmlFor = picker.id; bindText(label, msg('Companion images (optional)', '함께 사용할 이미지 (선택)'));
    const selected = document.createElement('p'); selected.className = 'hint';
    const showFiles = () => { selected.textContent = companions.map(f => f.name).join(', '); }; showFiles();
    picker.onchange = () => { companions = [...picker.files ?? []]; showFiles(); };
    const fields = document.createElement('div'), error = document.createElement('p'); error.setAttribute('role', 'status');
    const actions = document.createElement('div'); actions.className = 'button-row';
    const cancel = document.createElement('button'); bindText(cancel, msg('Cancel', '취소'));
    const next = document.createElement('button'); bindText(next, msg('Inspect file', '파일 검사'));
    actions.append(cancel, next); dialog.append(title, intro, filename, label, picker, selected, fields, error, actions); document.body.append(dialog);
    let loaded: LoadedFBX | null = null, finished = false, busy = false;
    let boneInputs: { name: typeof FBX_BONES[number]; input: HTMLInputElement }[] = [], unit: HTMLInputElement;
    const textureInputs: HTMLSelectElement[] = [];
    const finish = (value: { loaded: LoadedFBX; config: FBXConfig } | null) => {
      if (finished) return; finished = true;
      if (!value) loaded?.dispose(); dialog.close(); dialog.remove(); previousFocus?.focus(); resolve(value);
    };
    cancel.onclick = () => finish(null); dialog.oncancel = event => { event.preventDefault(); finish(null); };
    const render = () => {
      const source = loaded!, candidates = fbxBones(source.scene), suggested = suggestFBXBones(candidates);
      const info = document.createElement('p'); bindText(info, msg('{0} bones · {1} materials · {2} embedded clips (not played). Missing textures must be assigned below.', '본 {0}개 · 재질 {1}개 · 내장 클립 {2}개 (재생하지 않음). 연결되지 않은 텍스처는 아래에서 지정하세요.', candidates.length, source.materials.length, source.animationCount)); fields.append(info);
      const units = document.createElement('label'); bindText(units, msg('Metres per file unit', '파일 단위당 미터'));
      unit = document.createElement('input'); unit.id = 'fbx-unit'; unit.type = 'number'; unit.min = '.000001'; unit.max = '10'; unit.step = 'any'; unit.value = String(source.metersPerUnit); units.append(unit); fields.append(units);
      const hint = document.createElement('p'); hint.className = 'hint'; bindText(hint, msg('Centimetres: 0.01 · metres: 1. Use a standing rest pose. * marks required bones. Clear optional fields when uncertain.', '센티미터: 0.01 · 미터: 1. 서 있는 기본 자세를 사용하세요. *는 필수 본입니다. 확실하지 않은 선택 본은 비워 두세요.')); fields.append(hint);
      const list = document.createElement('datalist'); list.id = 'fbx-bone-options';
      const optionLabel = (bone: typeof candidates[number]) => `${bone.name} [${bone.path}]`;
      for (const bone of candidates) { const option = document.createElement('option'); option.value = optionLabel(bone); list.append(option); } fields.append(list);
      const optional = document.createElement('details'), summary = document.createElement('summary'); bindText(summary, msg('Optional bones / fingers', '선택 본 · 손가락')); optional.append(summary);
      boneInputs = FBX_BONES.map(name => {
        const row = document.createElement('label'), caption = document.createElement('span');
        bindText(caption, msg('{0}{1}', '{0}{1}', boneLabel(name), FBX_REQUIRED.includes(name) ? ' *' : '')); row.append(caption);
        const input = document.createElement('input'); input.setAttribute('list', list.id); input.id = `fbx-bone-${name}`; input.autocomplete = 'off'; input.dataset.bone = name;
        const match = candidates.find(bone => bone.path === suggested[name]); input.value = match ? optionLabel(match) : '';
        row.append(input); (FBX_REQUIRED.includes(name) ? fields : optional).append(row); return { name, input };
      }); fields.append(optional);
      const heading = document.createElement('h3'); bindText(heading, msg('Material base-color images', '재질 기본색 이미지')); fields.append(heading);
      source.materials.forEach((material, index) => {
        const row = document.createElement('label'); row.textContent = material.name || `Material ${index + 1}`;
        const select = document.createElement('select'); select.id = `fbx-material-${index}`;
        const keep = document.createElement('option'); keep.value = ''; bindText(keep, material.map ? msg('Keep original texture', '원래 텍스처 유지') : msg('No texture assigned', '연결된 텍스처 없음')); select.append(keep);
        for (const image of source.images) { const option = document.createElement('option'); option.value = imageKey(image.name); option.textContent = image.name; select.append(option); }
        row.append(select); fields.append(row); textureInputs.push(select);
      });
      const limitations = document.createElement('p'); limitations.className = 'hint'; bindText(limitations, msg('Phong/Lambert materials are preserved; this does not recreate Blender/MMD toon shaders, physics or constraints. Check the original model terms.', 'Phong/Lambert 재질을 유지합니다. Blender/MMD 툰 셰이더·물리·제약 설정을 재현하지는 않습니다. 모델의 원본 이용 조건을 확인하세요.')); fields.append(limitations);
      bindText(next, msg('Use character', '캐릭터 적용')); picker.disabled = true; next.focus();
    };
    next.onclick = async () => {
      if (busy) return;
      try {
        if (!loaded) {
          busy = true; next.disabled = true; picker.disabled = true; bindText(error, msg('Inspecting FBX…', 'FBX를 검사하고 있습니다…'));
          const candidate = await loadLocalFBX(file, companions);
          if (finished) { candidate.dispose(); return; }
          loaded = candidate; error.textContent = ''; render();
        } else {
          const candidates = fbxBones(loaded.scene), bones: FBXConfig['bones'] = {};
          for (const { name, input } of boneInputs) if (input.value.trim()) {
            const bone = candidates.find(b => `${b.name} [${b.path}]` === input.value);
            if (!bone) throw new Error(msg('Select a listed bone for {0}.', '{0}에 연결할 본을 목록에서 선택하세요.', name)); bones[name] = bone.path;
          }
          const config = validateFBXConfig({ bones, metersPerUnit: Number(unit.value), textures: Object.fromEntries(textureInputs.flatMap((select, index) => select.value ? [[String(index), select.value]] : [])) });
          resolveFBXBones(loaded.scene, config); finish({ loaded, config });
        }
      } catch (cause) { bindText(error, engineMessage(cause instanceof Error ? cause.message : String(cause))); }
      finally { busy = false; next.disabled = false; if (!loaded) picker.disabled = false; }
    };
    dialog.showModal(); next.focus();
  });
}
