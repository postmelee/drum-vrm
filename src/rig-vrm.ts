import { msg } from './i18n';
import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import { NormalizedHumanoidRigAdapter } from './rig-humanoid';
import { disposeModelSource } from './model-procedural';
import type { Vec3 } from './types';

/** VRM 1.0 normalized retargeting with a stateless secondary-motion policy.
 * Spring state is reset before each pose; update(0) applies humanoid, node
 * constraints, expressions and material updates without spring integration. */
export class VRMHumanoidRigAdapter extends NormalizedHumanoidRigAdapter {
  readonly vrm: VRM;
  private readonly eyeOffset: THREE.Vector3;
  constructor(vrm: VRM, id: string) {
    if (vrm.meta?.metaVersion !== '1') throw new Error(msg("Only VRM 1.0 humanoids are supported.", "VRM 1.0 휴머노이드만 지원합니다."));
    const restHead = vrm.humanoid.getRawBoneNode('head');
    const eyeOffset = vrm.lookAt?.offsetFromHeadBone.clone() ?? new THREE.Vector3(0, .06, 0);
    if (restHead) eyeOffset.multiply(restHead.getWorldScale(new THREE.Vector3())).applyQuaternion(restHead.getWorldQuaternion(new THREE.Quaternion()));
    super(vrm.humanoid, vrm.scene, id, {
      beforePose: () => {
        vrm.springBoneManager?.reset();
        if (vrm.lookAt) { vrm.lookAt.autoUpdate = false; vrm.lookAt.target = null; vrm.lookAt.yaw = 0; vrm.lookAt.pitch = 0; }
        vrm.expressionManager?.resetValues();
      },
      afterPose: () => vrm.update(0),
      disposeSource: () => {
        const skeletons = new Set<THREE.Skeleton>(); vrm.scene.traverse(node => { if (node instanceof THREE.SkinnedMesh) skeletons.add(node.skeleton); });
        skeletons.forEach(skeleton => skeleton.dispose()); disposeModelSource(vrm.scene);
      },
    });
    this.vrm = vrm; this.eyeOffset = eyeOffset; vrm.firstPerson?.setup(); this.refreshModelVisibility();
  }
  override getPlayerPosition(): Vec3 { return [-this.profile.head[0] + this.eyeOffset.x, this.profile.head[1] + this.eyeOffset.y, this.profile.head[2] + this.eyeOffset.z + .035]; }
}
