/** Numeric classification of geometry reconstructed from packed f32 poses.
 * Keep raw signed measurements; this is not a planner/contact safety margin. */
export const POSE_CLEARANCE_TOLERANCE_METERS = 1e-6;

export function inspectedClearancePassed(clearance: unknown): clearance is number {
  return typeof clearance === 'number' && Number.isFinite(clearance)
    && clearance >= -POSE_CLEARANCE_TOLERANCE_METERS;
}
