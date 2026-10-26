import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { contactFeedbackEvents, latestContactEvent, measurePlannedContact, resolveContactEvent, sampleHihatOpening } from '../src/contact-events';
import { applyHitResponse, bindHitResponse, createContactResponseTrack, evaluateHitResponse, placeContactMarker } from '../src/hit-response';
import type { ContactEvent, HihatMotionTrack, KitPiece, PerformancePlan } from '../src/types';

const event = (changes: Partial<ContactEvent> = {}): ContactEvent => ({
  version: 1, id: 'hit:0:edge', sourceEventId: '1:1', hitId: 0, time: 1, timeUs: 1_000_000,
  velocity: 100, limb: 0, toolPart: 'stickTip', role: 'primary', pieceId: 'snare', surfaceId: 'edge',
  frame: 'pieceReference', localPoint: [.1, 0, -.04], localNormal: [0, 1, 0], mechanismOffset: [0, 0, 0], status: 'planned', ...changes,
});
const piece: KitPiece = { id: 'snare', label: 'Snare', position: [.2, .7, .5], radius: .16,
  geometry: { version: 1, source: 'procedural', referenceRadius: .2,
    body: { center: [0, -.06, 0], halfExtents: [.2, .06, .2] }, surface: { center: [0, 0, 0], normal: [0, 1, 0], radius: .2 } } };
const plan = (events: ContactEvent[]): PerformancePlan => ({ schemaVersion: 2, duration: 2, hits: [], contactEvents: events, kit: [piece], diagnostics: [], tempoCount: 1 });

describe('semantic contact display boundary', () => {
  it('uses the resolved region and reference scaling instead of the piece center', () => {
    const result = resolveContactEvent(event(), [piece])!;
    expect(result.position[0]).toBeCloseTo(.28, 12); expect(result.position[1]).toBe(.7); expect(result.position[2]).toBeCloseTo(.468, 12);
    expect(result.position).not.toEqual(piece.position);
    expect(resolveContactEvent(event({ pieceId: 'absent' }), [piece])).toBeNull();
  });
  it('distinguishes pedal pressure from beater impact and does not fake unassigned strikes', () => {
    const events = [event({ id: 'sole', pieceId: 'kick', limb: 3, toolPart: 'sole', surfaceId: 'pedal' }),
      event({ id: 'beater', pieceId: 'kick', limb: 3, toolPart: 'beater', role: 'mechanism', surfaceId: 'batter' }),
      event({ id: 'missed', status: 'unassigned', limb: null }),
      event({ id: 'cue', pieceId: null, frame: 'world', role: 'pairedTool', localNormal: null }), event()];
    expect(contactFeedbackEvents(plan(events), 'kick').map(value => value.id)).toEqual(['beater']);
    expect(contactFeedbackEvents(plan(events), 'snare').map(value => value.id)).toEqual(['hit:0:edge']);
    const silent = createContactResponseTrack(events.filter(value => ['sole', 'missed', 'cue'].includes(value.id)), 'snare');
    expect(silent.anchors).toHaveLength(0);
  });
  it('retains a distinct measurement of intended contact and actual rendered marker', () => {
    const measured = measurePlannedContact(event(), [piece], [.28, .71, .468])!;
    expect(measured.errorMeters).toBeCloseTo(.01, 12);
    expect(measured.normalErrorMeters).toBeCloseTo(.01, 12);
    expect(measured.tangentErrorMeters).toBeCloseTo(0, 12);
    expect(measured.scope).toBe('actual-marker-to-planned-contact');
    expect(measured.status).toBe('planned');
  });
  it('anchors a small marker in the surface normal without inheriting model scale', () => {
    const root = new THREE.Group(), scaled = new THREE.Group(); scaled.scale.setScalar(2); root.add(scaled);
    const marker = new THREE.Mesh(new THREE.RingGeometry(.08, .1), new THREE.MeshBasicMaterial()); scaled.add(marker);
    placeContactMarker(marker, root, [.3, .8, .5], [0, .6, -.8]); root.updateMatrixWorld(true);
    expect(marker.parent).toBe(root); expect(marker.scale.toArray()).toEqual([1, 1, 1]);
    marker.position.toArray().forEach((value, axis) => expect(value).toBeCloseTo([.3, .8018, .4976][axis], 12));
    expect(new THREE.Vector3(0, 0, 1).applyQuaternion(marker.quaternion).distanceTo(new THREE.Vector3(0, .6, -.8))).toBeLessThan(1e-12);
    expect(marker.geometry.parameters.outerRadius).toBe(.026);
  });
  it('preserves the current impact point while responding around a changed articulation', () => {
    const root = new THREE.Group(), body = new THREE.Mesh(new THREE.SphereGeometry(.01), new THREE.MeshBasicMaterial());
    body.position.set(.3, .8, .5); body.userData.hitResponseRole = 'surface'; root.add(body);
    const binding = bindHitResponse(root, [.2, .8, .5]);
    applyHitResponse(binding, { scale: 1.02, tiltX: .03, tiltZ: -.01 }, [.3, .8, .5]); root.updateMatrixWorld(true);
    expect(body.getWorldPosition(new THREE.Vector3()).distanceTo(new THREE.Vector3(.3, .8, .5))).toBeLessThan(1e-12);
    applyHitResponse(binding, { scale: 1, tiltX: 0, tiltZ: 0 }); root.updateMatrixWorld(true);
    expect(body.getWorldPosition(new THREE.Vector3()).distanceTo(new THREE.Vector3(.3, .8, .5))).toBeLessThan(1e-12);
  });
});

describe('shared upper hi-hat time track', () => {
  const track: HihatMotionTrack = { travelMeters: .025, anchors: [{ time: 0, opening: 0 }, { time: .1, opening: .2 }, { time: .12, opening: 0 }] };
  it('matches the Rust endpoint/midpoint contract in any seek order', () => {
    const cases = [[.1, .2], [0, 0], [.12, 0], [.05, .1], [.11, .1], [2, 0]];
    for (const [time, expected] of [...cases, ...[...cases].reverse()]) expect(sampleHihatOpening(track, time)).toBeCloseTo(expected, 12);
  });
  it('includes openness once at impact and follows the closing surface afterward', () => {
    const hat: KitPiece = { id: 'hihat', label: 'Hat', position: [0, 1, 0], radius: .2 };
    const contact = event({ pieceId: 'hihat', time: .1, localPoint: [0, .005, 0], mechanismOffset: [0, .005, 0] });
    expect(resolveContactEvent(contact, [hat], track)!.position[1]).toBeCloseTo(1.005, 12);
    expect(resolveContactEvent(contact, [hat], track, .12)!.position[1]).toBeCloseTo(1, 12);
    const ordered = [contact, event({ id: 'later', time: 2 })];
    expect(latestContactEvent(ordered, .1)?.id).toBe(contact.id); expect(latestContactEvent(ordered, .5)).toBeNull();
    const response = createContactResponseTrack([contact], 'hihat');
    const settings = { enabled: true, strength: .65 };
    expect(evaluateHitResponse(response, .1, settings)).toEqual({ scale: 1, tiltX: -0, tiltZ: -0 });
  });
  it('honors an explicit surface translation and keeps unknown measured surfaces fixed', () => {
    const hat: KitPiece = { ...structuredClone(piece), id: 'hihat' };
    const fixed = event({ pieceId: 'hihat', localPoint: [0, 0, 0], time: .1 });
    expect(resolveContactEvent(fixed, [hat], track)!.position).toEqual(hat.position);
    hat.geometry!.surface.motion = { kind: 'translation', axis: [0, .6, .8], travelMeters: .02 };
    const moving = event({ ...fixed, localPoint: [0, .003, .004], mechanismOffset: [0, .0024, .0032] });
    expect(resolveContactEvent(moving, [hat], track, .12)!.position).toEqual(hat.position);
    resolveContactEvent(moving, [hat], track)!.position.forEach((value, axis) => expect(value).toBeCloseTo(hat.position[axis] + [0, .0024, .0032][axis], 12));
  });
});
