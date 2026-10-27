import { bindText, msg } from './i18n';

export function characterFiles(files: File[]) {
  return { models: files.filter(file => /\.(vrm|fbx)$/i.test(file.name)), images: files.filter(file => /\.(png|jpe?g)$/i.test(file.name)) };
}
/** Directory paths distinguish models; never silently choose the first one. */
export function chooseCharacterFile(models: File[]): Promise<File | null> {
  if (models.length === 1) return Promise.resolve(models[0]);
  if (!models.length) return Promise.resolve(null);
  return new Promise(resolve => {
    const focus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const dialog = document.createElement('dialog'); dialog.className = 'fbx-setup';
    const heading = document.createElement('h2'); heading.id = 'choose-character-title'; bindText(heading, msg('Choose a character from the folder', '폴더에서 캐릭터 선택')); dialog.setAttribute('aria-labelledby', heading.id);
    const label = document.createElement('label'); const caption = document.createElement('span'); bindText(caption, msg('Character file', '캐릭터 파일')); label.append(caption);
    const select = document.createElement('select');
    models.forEach((file, index) => { const option = document.createElement('option'); option.value = String(index); option.textContent = file.webkitRelativePath || file.name; select.append(option); }); label.append(select);
    const actions = document.createElement('div'); actions.className = 'button-row';
    const cancel = document.createElement('button'), next = document.createElement('button'); bindText(cancel, msg('Cancel', '취소')); bindText(next, msg('Continue', '계속'));
    const finish = (file: File | null) => { dialog.close(); dialog.remove(); focus?.focus(); resolve(file); };
    cancel.onclick = () => finish(null); next.onclick = () => finish(models[Number(select.value)]); dialog.oncancel = event => { event.preventDefault(); finish(null); };
    actions.append(cancel, next); dialog.append(heading, label, actions); document.body.append(dialog); dialog.showModal(); select.focus();
  });
}
