import { Object3D, Quaternion, Vector3 } from 'three';

/** A thumb pad and its distal axis constrain the same shaft. Joint rotations
 * are bounded about the model's rest pose; no fingers are stretched. */
export function fitThumbToShaft(joints: readonly Object3D[], pad: Object3D, target: Vector3, shaft: Vector3): void {
  const rest = joints.map(joint => joint.quaternion.clone());
  const direction = shaft.clone().normalize();
  const axis = new Vector3(), tip = new Vector3(), pivot = new Vector3();
  const score = () => {
    joints[0].updateWorldMatrix(true, true);
    pad.getWorldPosition(tip); joints.at(-1)!.getWorldPosition(pivot);
    axis.copy(tip).sub(pivot).normalize();
    // Soft alignment: opposition remains possible without locking the thumb flat.
    return tip.distanceToSquared(target) + .018 ** 2 * (1 - axis.dot(direction)) ** 2;
  };
  let best = score();
  const axes = [new Vector3(1, 0, 0), new Vector3(0, 1, 0), new Vector3(0, 0, 1)];
  for (const step of [.24, .12, .06, .025]) for (let pass = 0; pass < 5; pass++) {
    let improved = false;
    for (let i = 0; i < joints.length; i++) for (const axis of axes) {
      const joint = joints[i], before = joint.quaternion.clone();
      let selected = before;
      for (const sign of [-1, 1]) {
        joint.quaternion.copy(before).multiply(new Quaternion().setFromAxisAngle(axis, step * sign));
        if (rest[i].angleTo(joint.quaternion) > [1.2, 1.3, 1.3][i]) continue;
        const cost = score();
        if (cost < best - 1e-12) { best = cost; selected = joint.quaternion.clone(); improved = true; }
      }
      joint.quaternion.copy(selected);
    }
    if (!improved) break;
  }
  joints[0].updateWorldMatrix(true, true);
}
