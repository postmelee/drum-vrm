import { msg } from './i18n';
import type { FBXMaterialMode } from './fbx-config';

// Keep the persisted `original` value compatible with existing projects.
export function materialAppearanceLabel(mode: FBXMaterialMode = 'original'): string {
  return { original: msg('File settings (automatic)', '파일 설정 자동 적용'),
    matte: msg('Matte lighting', '무광 조명'), unlit: msg('Texture only (Unlit)', '텍스처 그대로 (Unlit)') }[mode];
}
export function materialAppearanceDescription(mode: FBXMaterialMode): string {
  return { original: msg('Uses the material settings stored in the file.', '파일에 저장된 재질 설정을 사용합니다.'),
    matte: msg('Keeps shading without highlights.', '명암을 유지하고 반짝임을 없앱니다.'),
    unlit: msg('Shows colors and textures without lighting.', '조명 없이 색상과 텍스처를 표시합니다.') }[mode];
}
