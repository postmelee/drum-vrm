import assert from 'node:assert/strict';
import test from 'node:test';
import { KINEMATIC_HALF_WINDOW_SECONDS as h, motionValidity, summarizeMotionSamples } from './review-motion-metrics.mjs';

const cuePlan = () => ({
  sourceEvents: [{ id: 'count', timeUs: 1000000, message: { kind: 'noteOn', velocity: 80 } }],
  interpretations: [{ sourceEventId: 'count', kind: 'countIn' }], diagnostics: [],
  contactEvents: [0, 1].map(limb => ({ id: `cue:count:${limb}`, sourceEventId: 'count', time: 1, limb, role: 'pairedTool', status: 'planned' })),
});
test('validity includes count-in contacts even though ordinary hit coverage can be complete', () => {
  const plan = cuePlan();
  assert.equal(motionValidity(plan).passed, true);
  plan.contactEvents = [];
  const absent = motionValidity(plan);
  assert.equal(absent.passed, false);
  assert.equal(absent.countIn.expectedCueCount, 1);
  assert.equal(absent.countIn.plannedCueCount, 0);
  assert.deepEqual(absent.countIn.incompleteCues[0].sourceEventId, 'count');
});
test('same contact count cannot conceal duplicate limbs, shifted cues or unknown source intent', () => {
  for (const alter of [
    plan => { plan.contactEvents[1].limb = 0; },
    plan => { plan.contactEvents[1].time += .01; },
    plan => { plan.contactEvents[1].sourceEventId = 'other'; },
    plan => { plan.sourceEvents = []; },
  ]) {
    const plan = cuePlan(); alter(plan);
    assert.equal(motionValidity(plan).passed, false);
  }
  const plan = cuePlan();
  plan.sourceEvents.push({ id: 'simultaneous', timeUs: 1000000, message: { kind: 'noteOn', velocity: 80 } });
  plan.interpretations.push({ sourceEventId: 'simultaneous', kind: 'countIn' });
  plan.contactEvents.push(...[0, 1].map(limb => ({ ...plan.contactEvents[limb], id: `cue:simultaneous:${limb}`, sourceEventId: 'simultaneous' })));
  assert.equal(motionValidity(plan).countIn.expectedCueCount, 2);
  assert.equal(motionValidity(plan).passed, true);
});
test('posture, pitch and unavailable cues block validity while advisory body overlap does not', () => {
  for (const code of ['POSTURE_LIMIT', 'CONTACT_PITCH_LIMIT', 'COUNT_IN_GESTURE_UNAVAILABLE']) {
    const plan = cuePlan(); plan.diagnostics.push({ code, time: 1 });
    assert.equal(motionValidity(plan).passed, false);
  }
  const plan = cuePlan();
  plan.diagnostics.push({ code: 'BODY_PATH_CLEARANCE' }, { code: 'EQUIPMENT_PATH_CLEARANCE' });
  assert.equal(motionValidity(plan).passed, true);
});
test('planner-only physical failures remain blocking when ordinary review samples are clear', () => {
  for (const code of ['TOOL_EQUIPMENT_PATH_CLEARANCE', 'TOOL_PAIR_PATH_CLEARANCE']) {
    const plan = cuePlan();
    // This refined planner instant need not occur on the external 30 Hz grid.
    plan.diagnostics.push({ code, time: 1.0125, message: 'Residual physical intersection' });
    const result = motionValidity(plan);
    assert.equal(result.countIn.passed, true);
    assert.equal(result.passed, false);
    assert.deepEqual(result.blockingDiagnostics, plan.diagnostics);
  }
});

function poseAt(time) {
  const pose = new Float64Array(77);
  for (const limb of [0, 1]) {
    pose[limb * 12 + 9] = time * time; // speed 2t, acceleration 2
    pose[limb * 12 + 6] = time * 3;
    pose[57 + limb * 4 + 1] = Math.sin(time / 2);
    pose[57 + limb * 4 + 3] = Math.cos(time / 2);
  }
  return pose;
}
const sampleAt = time => ({ time, before: poseAt(time - h), pose: poseAt(time), after: poseAt(time + h), markers: [] });
test('finite differences report actual pose motion without claiming the assignment-speed bound', () => {
  const { kinematics } = summarizeMotionSamples([sampleAt(1), sampleAt(10)]);
  assert.ok(Math.abs(kinematics.tipSpeedMetersPerSecond.value - 20) < 1e-9);
  assert.ok(Math.abs(kinematics.wristSpeedMetersPerSecond.value - 3) < 1e-9);
  assert.ok(Math.abs(kinematics.tipAccelerationMetersPerSecondSquared.value - 2) < 1e-8);
  assert.ok(Math.abs(kinematics.toolAngularSpeedRadiansPerSecond.value - 1) < 1e-8);
  assert.equal(kinematics.tipSpeedMetersPerSecond.time, 10);
  assert.equal(kinematics.speedAcceptanceThresholdMetersPerSecond, null);
  assert.equal(kinematics.continuousBound, false);
  assert.equal(kinematics.invalidObservations, 0);
});
test('antipodal quaternions do not create a false angular jump; malformed observations stay visible', () => {
  const sample = sampleAt(0);
  for (let index = 57; index < 65; index++) sample.after[index] *= -1;
  assert.ok(Math.abs(summarizeMotionSamples([sample]).kinematics.toolAngularSpeedRadiansPerSecond.value - 1) < 1e-8);
  sample.before[9] = NaN;
  const result = summarizeMotionSamples([sample]);
  assert.equal(result.kinematics.invalidObservations, 1);
  assert.equal(result.kinematics.tipSpeedMetersPerSecond.value, null);
});
test('posture reports measured ranges and distinguishes absent measurements from zero angles', () => {
  const a = sampleAt(0), b = sampleAt(1);
  a.markers = [{ limb: 0, wristSwingDegrees: 0, upperArmElevationDegrees: 40, elbowInteriorDegrees: 90 }];
  b.markers = [{ limb: 0, wristSwingDegrees: 20, upperArmElevationDegrees: 30, elbowInteriorDegrees: 110 }];
  const result = summarizeMotionSamples([a, b]).posture;
  assert.deepEqual(result.hands[0].wristSwingDegrees, { samples: 2, minimum: 0, maximum: 20 });
  assert.deepEqual(result.hands[0].upperArmElevationDegrees, { samples: 2, minimum: 30, maximum: 40 });
  assert.deepEqual(result.hands[1].wristSwingDegrees, { samples: 0, minimum: null, maximum: null });
});
