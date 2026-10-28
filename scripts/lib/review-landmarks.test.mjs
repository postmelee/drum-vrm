import assert from 'node:assert/strict';
import test from 'node:test';
import { motionReviewLandmarks } from './review-landmarks.mjs';
const hit = (id, time, instrument, pieceId = instrument) => ({ id, sourceEventId: id, time, instrument, pieceId, limb: null });
test('finds sustained groove, backbeat and tom paths even when assignment failed', () => {
  const plan = { duration: 8, hits: [hit('h0', 2, 'hihat'), hit('h1', 2.25, 'hihat'), hit('h2', 2.5, 'hihat'), hit(4, 2.5, 'snare'), hit('h3', 2.75, 'hihat'), hit('t0', 6, 'tomHigh'), hit('t1', 6.125, 'tomMid')] };
  const landmarks = motionReviewLandmarks(plan, [{ label: 'Ready', time: -.25 }]);
  assert.equal(landmarks.find(item => item.label === 'Sustained groove').time, 2.5);
  assert.equal(landmarks.find(item => item.label === 'Groove backbeat').time, 2.5);
  assert.equal(landmarks.find(item => item.label === 'Tom path 1').time, 6.0625);
  assert.equal(landmarks.find(item => item.label === 'Before first strike').time, 1.75);
  assert.equal(landmarks.find(item => item.label === 'Final recovery').time, 8.5);
  assert.deepEqual(motionReviewLandmarks({ ...plan, hits: [...plan.hits].reverse() }, [{ label: 'Ready', time: -.25 }]), landmarks);
});
test('does not call separated hat hits sustained or infer simultaneous tom transfer', () => {
  const plan = { duration: 5, hits: [hit('h0', 0, 'hihat'), hit('h1', 2, 'openHat'), hit('h2', 4, 'hihat'), hit('t0', 4, 'tomHigh'), hit('t1', 4, 'tomMid')] };
  const landmarks = motionReviewLandmarks(plan);
  assert.equal(landmarks.some(item => item.label === 'Sustained groove'), false);
  assert.equal(landmarks.some(item => item.label.startsWith('Tom path')), false);
  assert.deepEqual(motionReviewLandmarks({ duration: 0, hits: [] }).map(item => item.time), [-.25, .5]);
});
