import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { bindAttribute, msg } from './i18n';

/** Owns only preview GPU resources. The caller retains the loaded character. */
export function createFBXPreview(host: HTMLElement, model: THREE.Group) {
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  bindAttribute(renderer.domElement, 'aria-label', msg('Character preview; drag to rotate, scroll to zoom', '캐릭터 미리보기: 드래그로 회전, 스크롤로 확대'));
  host.append(renderer.domElement);
  const scene = new THREE.Scene(); scene.add(model);
  scene.add(new THREE.HemisphereLight(0xffffff, 0x777777, 2));
  const light = new THREE.DirectionalLight(0xffffff, 2); light.position.set(2, 3, 4); scene.add(light);
  model.updateMatrixWorld(true);
  const box = new THREE.Box3().setFromObject(model);
  const center = box.isEmpty() ? new THREE.Vector3() : box.getCenter(new THREE.Vector3());
  const span = Math.max(box.isEmpty() ? 1 : box.getSize(new THREE.Vector3()).length(), .001);
  const camera = new THREE.PerspectiveCamera(35, 1, span / 10000, span * 100);
  const controls = new OrbitControls(camera, renderer.domElement); controls.target.copy(center);
  const marker = new THREE.Mesh(new THREE.SphereGeometry(span * .012, 16, 12), new THREE.MeshBasicMaterial({ color: 0xff3366, depthTest: false, depthWrite: false }));
  marker.renderOrder = 10000; marker.visible = false; scene.add(marker);
  const draw = () => renderer.render(scene, camera);
  const reset = () => { camera.position.copy(center).add(new THREE.Vector3(0, span * .05, span * 1.6)); controls.target.copy(center); controls.update(); draw(); };
  const resize = () => { const width = host.clientWidth, height = host.clientHeight; if (!width || !height) return; renderer.setSize(width, height); camera.aspect = width / height; camera.updateProjectionMatrix(); draw(); };
  controls.addEventListener('change', draw);
  const observer = new ResizeObserver(resize); observer.observe(host); resize(); reset();
  return {
    draw, reset,
    highlight(node?: THREE.Bone) { marker.visible = !!node; if (node) marker.position.copy(node.getWorldPosition(new THREE.Vector3())); draw(); },
    dispose() { observer.disconnect(); controls.dispose(); model.removeFromParent(); marker.geometry.dispose(); marker.material.dispose(); renderer.dispose(); renderer.forceContextLoss(); renderer.domElement.remove(); },
  };
}
