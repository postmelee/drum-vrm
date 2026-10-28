import * as THREE from 'three';
import type { FBXMaterialMode } from './fbx-config';

export type FBXSourceMaterial = THREE.MeshPhongMaterial | THREE.MeshLambertMaterial;
export interface FBXMaterialMetadata {
  factors: Map<number, number>;
  /** Material IDs in FBX connection order, keyed by mesh model ID. */
  bindings: Map<number, number[]>;
}

/** IDs, not names, connect bounded source metadata to the pinned FBXLoader output. */
export function createFBXMaterials(root: THREE.Group, metadata: FBXMaterialMetadata) {
  const bindings: { mesh: THREE.Mesh; materials: THREE.Material[]; array: boolean }[] = [];
  const originals = new Set<FBXSourceMaterial>();
  root.traverse(node => {
    if (!(node instanceof THREE.Mesh)) return;
    const materials = Array.isArray(node.material) ? node.material : [node.material];
    bindings.push({ mesh: node, materials, array: Array.isArray(node.material) });
    const ids = metadata.bindings.get((node as THREE.Mesh & { ID?: number }).ID ?? -1) ?? [];
    materials.forEach((material, index) => {
      if (!(material instanceof THREE.MeshPhongMaterial || material instanceof THREE.MeshLambertMaterial) || originals.has(material)) return;
      if (material instanceof THREE.MeshPhongMaterial) material.specular.multiplyScalar(metadata.factors.get(ids[index]) ?? 1);
      originals.add(material);
    });
  });
  const matte = new Map<FBXSourceMaterial, FBXSourceMaterial>(), unlit = new Map<FBXSourceMaterial, THREE.MeshBasicMaterial>();
  const appearance = (source: THREE.Material, mode: FBXMaterialMode): THREE.Material => {
    if (!(source instanceof THREE.MeshPhongMaterial || source instanceof THREE.MeshLambertMaterial) || mode === 'original') return source;
    if (mode === 'matte') {
      let material = matte.get(source); if (!material) { material = source.clone(); matte.set(source, material); }
      if (material instanceof THREE.MeshPhongMaterial && source instanceof THREE.MeshPhongMaterial) material.copy(source);
      else if (material instanceof THREE.MeshLambertMaterial && source instanceof THREE.MeshLambertMaterial) material.copy(source);
      if (material instanceof THREE.MeshPhongMaterial) material.specular.setRGB(0, 0, 0);
      material.envMap = null; material.reflectivity = 0; material.needsUpdate = true; return material;
    }
    let material = unlit.get(source); if (!material) { material = new THREE.MeshBasicMaterial(); unlit.set(source, material); }
    // Preserve opacity, cutout, sidedness, depth/blending and vertex colors. Unlit
    // intentionally uses only base color/alpha, without lighting, reflections or tone mapping.
    THREE.Material.prototype.copy.call(material, source);
    material.color.copy(source.color); material.map = source.map; material.alphaMap = source.alphaMap;
    material.wireframe = source.wireframe; material.wireframeLinewidth = source.wireframeLinewidth;
    material.toneMapped = false; material.fog = false; material.needsUpdate = true; return material;
  };
  let disposed = false;
  return {
    apply(mode: FBXMaterialMode) {
      for (const { mesh, materials, array } of bindings) {
        const next = materials.map(source => appearance(source, mode)); mesh.material = array ? next : next[0];
      }
    },
    dispose() {
      if (disposed) return; disposed = true;
      // Inactive variants and original materials are no longer reachable through the scene.
      for (const material of [...originals, ...matte.values(), ...unlit.values()]) material.dispose();
      matte.clear(); unlit.clear();
    },
  };
}
