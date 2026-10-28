/** Review evidence, not another planner or a continuous motion certificate. */
export const KINEMATIC_HALF_WINDOW_SECONDS = 1 / 240;
const blockingCodes = new Set([
  'POSTURE_LIMIT', 'CONTACT_PITCH_LIMIT', 'COUNT_IN_GESTURE_UNAVAILABLE',
  // The planner also inspects refined times absent from the review grid.
  // A clear outer sample cannot erase a measured physical failure there.
  'TOOL_EQUIPMENT_PATH_CLEARANCE', 'TOOL_PAIR_PATH_CLEARANCE',
]);

/** Count-in has no ordinary Hit: compare source intent to both planned contacts. */
export function motionValidity(plan) {
  const source = new Map((plan.sourceEvents ?? []).map(event => [event.id, event]));
  const interpreted = [...new Set((plan.interpretations ?? []).filter(event => event.kind === 'countIn').map(event => event.sourceEventId))];
  const expected = [], invalidSourceEventIds = [];
  for (const id of interpreted) {
    const event = source.get(id);
    if (!event || event.message?.kind !== 'noteOn' || !(event.message.velocity > 0) || !Number.isFinite(event.timeUs)) invalidSourceEventIds.push(id);
    else expected.push({ id, time: event.timeUs / 1e6 });
  }
  const contacts = (plan.contactEvents ?? []).filter(event => event.role === 'pairedTool' && event.status === 'planned');
  const contactsBySource = new Map();
  for (const contact of contacts) {
    if (!contactsBySource.has(contact.sourceEventId)) contactsBySource.set(contact.sourceEventId, []);
    contactsBySource.get(contact.sourceEventId).push(contact);
  }
  const expectedIds = new Set(expected.map(event => event.id));
  const unexpectedContactIds = contacts.filter(event => !expectedIds.has(event.sourceEventId)).map(event => event.id);
  const incompleteCues = expected.flatMap(cue => {
    const pair = contactsBySource.get(cue.id) ?? [];
    const valid = pair.length === 2 && [0, 1].every(limb => pair.filter(event => event.limb === limb && Number.isFinite(event.time) && Math.abs(event.time - cue.time) <= 1e-9).length === 1);
    return valid ? [] : [{ sourceEventId: cue.id, time: cue.time, contactIds: pair.map(event => event.id), limbs: pair.map(event => event.limb) }];
  });
  const blockingDiagnostics = (plan.diagnostics ?? []).filter(diagnostic => blockingCodes.has(diagnostic.code));
  const countIn = {
    expectedCueCount: expected.length, plannedCueCount: expected.length - incompleteCues.length,
    expectedContactCount: expected.length * 2, plannedContactCount: contacts.length,
    invalidSourceEventIds, incompleteCues, unexpectedContactIds,
  };
  countIn.passed = invalidSourceEventIds.length === 0 && incompleteCues.length === 0 && unexpectedContactIds.length === 0 && contacts.length === expected.length * 2;
  return { passed: blockingDiagnostics.length === 0 && countIn.passed, blockingDiagnostics, countIn };
}

const peak = () => ({ value: null, time: null, limb: null });
function updatePeak(result, value, time, limb) {
  if (result.value === null || value > result.value) Object.assign(result, { value, time, limb });
}
const vectorDifference = (a, b, offset, divisor) => Math.hypot(...[0, 1, 2].map(axis => (a[offset + axis] - b[offset + axis]) / divisor));
function angularSpeed(a, b, offset, interval) {
  const qa = Array.from(a.slice(offset, offset + 4)), qb = Array.from(b.slice(offset, offset + 4));
  const denominator = Math.hypot(...qa) * Math.hypot(...qb);
  if (!(denominator > 0)) return NaN;
  const dot = Math.abs(qa.reduce((sum, value, index) => sum + value * qb[index], 0) / denominator);
  return 2 * Math.acos(Math.min(1, dot)) / interval;
}

/** Each sample supplies f32 primary poses at t-h, t, t+h and adapter markers at t.
 * The derivative window is fixed, but the observations are NOT a dense h-grid.
 * 12 m/s limits assignment target chords; it is not a final-tip velocity gate. */
export function summarizeMotionSamples(samples, halfWindow = KINEMATIC_HALF_WINDOW_SECONDS) {
  if (!(halfWindow > 0) || !Number.isFinite(halfWindow)) throw Error('Expected a positive finite derivative half-window');
  const kinematics = {
    scope: 'sampled-primary-pose-hands', method: 'central-finite-difference',
    halfWindowSeconds: halfWindow, observations: samples.length, invalidObservations: 0,
    continuousBound: false, speedAcceptanceThresholdMetersPerSecond: null,
    tipSpeedMetersPerSecond: peak(), wristSpeedMetersPerSecond: peak(),
    tipAccelerationMetersPerSecondSquared: peak(), toolAngularSpeedRadiansPerSecond: peak(),
  };
  const posture = { scope: 'sampled-adapter-bone-angles', observations: samples.length, hands: [0, 1].map(limb => ({
    limb, wristSwingDegrees: { samples: 0, minimum: null, maximum: null },
    upperArmElevationDegrees: { samples: 0, minimum: null, maximum: null },
    elbowInteriorDegrees: { samples: 0, minimum: null, maximum: null },
  })) };
  for (const sample of samples) {
    const { time, before, pose, after } = sample;
    const finite = Number.isFinite(time) && [before, pose, after].every(values => values?.length >= 65 && Array.from(values).every(Number.isFinite));
    let invalid = !finite;
    if (finite) for (const limb of [0, 1]) {
      const tip = limb * 12 + 9, wrist = limb * 12 + 6;
      const angular = angularSpeed(before, after, 57 + limb * 4, halfWindow * 2);
      if (!Number.isFinite(angular)) invalid = true;
      else updatePeak(kinematics.toolAngularSpeedRadiansPerSecond, angular, time, limb);
      updatePeak(kinematics.tipSpeedMetersPerSecond, vectorDifference(after, before, tip, halfWindow * 2), time, limb);
      updatePeak(kinematics.wristSpeedMetersPerSecond, vectorDifference(after, before, wrist, halfWindow * 2), time, limb);
      const acceleration = Math.hypot(...[0, 1, 2].map(axis => (after[tip + axis] - 2 * pose[tip + axis] + before[tip + axis]) / (halfWindow * halfWindow)));
      updatePeak(kinematics.tipAccelerationMetersPerSecondSquared, acceleration, time, limb);
    }
    if (invalid) kinematics.invalidObservations++;
    for (const hand of posture.hands) {
      const marker = (sample.markers ?? []).find(marker => marker.limb === hand.limb);
      for (const field of ['wristSwingDegrees', 'upperArmElevationDegrees', 'elbowInteriorDegrees']) {
        const value = marker?.[field], metric = hand[field];
        if (!Number.isFinite(value)) continue;
        metric.samples++;
        metric.minimum = metric.minimum === null ? value : Math.min(metric.minimum, value);
        metric.maximum = metric.maximum === null ? value : Math.max(metric.maximum, value);
      }
    }
  }
  return { kinematics, posture };
}
