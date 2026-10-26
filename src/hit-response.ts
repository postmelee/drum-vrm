import { msg } from './i18n';
import * as THREE from 'three';
import type { ContactEvent, Hit, Instrument, Vec3 } from './types';

export interface HitResponseSettings { enabled: boolean; strength: number; }
export interface ResponseSample { scale: number; tiltX: number; tiltZ: number; }
interface ResponseAnchor { time: number; impulse: number; pulse: number; sine: number; cosine: number; }
export interface HitResponseTrack { readonly anchors: readonly ResponseAnchor[]; readonly cymbal: boolean; }
const TAU = .045, DAMPING = 9, FREQUENCY = 35;
const REST: ResponseSample = Object.freeze({ scale: 1, tiltX: 0, tiltZ: 0 });

export function validateHitResponse(settings: HitResponseSettings): HitResponseSettings {
  if (typeof settings.enabled !== 'boolean' || !Number.isFinite(settings.strength) || settings.strength < 0 || settings.strength > 1) {
    throw new Error(msg("Hit response strength must be between 0% and 100%.", "타격 반응 강도는 0–100% 범위여야 합니다."));
  }
  return { ...settings };
}

/** Bake damped impulse sums once. Evaluation is O(log N), including dense rolls. */
export function createHitResponseTrack(hits: readonly Pick<Hit, 'id' | 'time' | 'velocity'>[], instrument: Instrument): HitResponseTrack {
  const grouped: { time: number; velocity: number }[] = [];
  for (const hit of [...hits].sort((a, b) => a.time - b.time || a.id - b.id)) {
    const velocity = Math.max(0, Math.min(1, hit.velocity / 127));
    const previous = grouped.at(-1);
    if (previous?.time === hit.time) previous.velocity = Math.max(previous.velocity, velocity);
    else grouped.push({ time: hit.time, velocity });
  }
  const anchors: ResponseAnchor[] = [];
  let impulse = 0, pulse = 0, sine = 0, cosine = 0, previousTime = grouped[0]?.time ?? 0;
  for (const hit of grouped) {
    const dt = hit.time - previousTime, decay = Math.exp(-dt / TAU), damping = Math.exp(-dt * DAMPING);
    pulse = (pulse + impulse * dt) * decay; impulse = impulse * decay + hit.velocity;
    const sin = Math.sin(dt * FREQUENCY), cos = Math.cos(dt * FREQUENCY);
    const nextSine = (sine * cos + cosine * sin) * damping;
    cosine = (cosine * cos - sine * sin) * damping + hit.velocity; sine = nextSine;
    anchors.push({ time: hit.time, impulse, pulse, sine, cosine }); previousTime = hit.time;
  }
  return { anchors, cymbal: ['hihat', 'openHat', 'crash', 'ride'].includes(instrument) };
}

/** Semantic events have already separated stick/beater contacts from pedal
 * controls. Unassigned events cannot make an instrument appear to be struck. */
export function createContactResponseTrack(events: readonly ContactEvent[], instrument: Instrument): HitResponseTrack {
  const hits = events.filter(event => event.status === 'planned' && event.limb !== null && event.toolPart !== 'sole' && event.role !== 'pairedTool')
    .map((event, id) => ({ id, time: event.time, velocity: event.velocity }));
  return createHitResponseTrack(hits, instrument);
}

/** Marker is parented in the same planning frame as the resolved contact. It
 * must not inherit an appearance scale/pivot left over from model construction. */
export function placeContactMarker(marker: THREE.Mesh<THREE.RingGeometry>, parent: THREE.Object3D, point: Vec3, normal: Vec3): void {
  if (marker.parent !== parent) parent.add(marker);
  if (marker.geometry.parameters.outerRadius !== .026) {
    marker.geometry.dispose(); marker.geometry = new THREE.RingGeometry(.018, .026, 40);
  }
  marker.scale.setScalar(1);
  const axis = new THREE.Vector3(...normal).normalize();
  marker.position.set(...point).addScaledVector(axis, .003);
  marker.quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), axis);
}

/** Absolute-time display feedback, never a physics accumulator or a new hit target. */
export function evaluateHitResponse(track: HitResponseTrack, songTime: number, settings: HitResponseSettings, reducedMotion = false, fallback = false): ResponseSample {
  if (!settings.enabled || !settings.strength || reducedMotion || !Number.isFinite(songTime)) return REST;
  let low = 0, high = track.anchors.length;
  while (low < high) { const mid = (low + high) >>> 1; if (track.anchors[mid].time <= songTime) low = mid + 1; else high = mid; }
  const anchor = track.anchors[low - 1]; if (!anchor) return REST;
  const age = songTime - anchor.time;
  if (age > 1.5) return REST;
  const pulse = Math.min(1, (anchor.pulse + anchor.impulse * age) * Math.exp(1 - age / TAU) / TAU);
  const wave = Math.max(-1, Math.min(1, (anchor.sine * Math.cos(age * FREQUENCY) + anchor.cosine * Math.sin(age * FREQUENCY)) * Math.exp(-age * DAMPING)));
  const amount = settings.strength * (fallback ? .35 : 1);
  // Unknown imported parts may include their stand; use only a tiny uniform pulse.
  if (track.cymbal && !fallback) return { scale: 1, tiltX: -.03 * wave * amount, tiltZ: -.012 * wave * amount };
  return { scale: 1 + .02 * pulse * amount, tiltX: 0, tiltZ: 0 };
}

export interface HitResponseBinding {
  pivot: THREE.Group; fallback: boolean; origin: THREE.Vector3;
  restPositions: { node: THREE.Object3D; position: THREE.Vector3 }[];
}
/** Reparent known moving parts while retaining their authored transform and static hardware. */
export function bindHitResponse(root: THREE.Group, contact: Vec3, imported = false): HitResponseBinding {
  const moving: THREE.Object3D[] = [];
  const collect = (node: THREE.Object3D): void => {
    const role = node.userData.hitResponseRole;
    if (role === 'marker' || role === 'hardware' || role === 'pedal' || node.userData.fixedContact) return;
    if (role === 'surface' || role === 'shell') { moving.push(node); return; }
    node.children.forEach(collect);
  };
  if (!imported) root.children.forEach(collect);
  const fallback = imported || moving.length === 0;
  if (fallback) moving.push(...root.children.filter(node => node.userData.hitResponseRole !== 'marker' && !node.userData.fixedContact));
  const pivot = new THREE.Group(); pivot.name = 'ContactResponsePivot'; pivot.position.set(...contact);
  root.add(pivot); root.updateMatrixWorld(true);
  moving.forEach(node => pivot.attach(node));
  return { pivot, fallback, origin: pivot.position.clone(), restPositions: moving.map(node => ({ node, position: node.position.clone() })) };
}
export function applyHitResponse(binding: HitResponseBinding, sample: ResponseSample, contact?: Vec3): void {
  const origin = contact ? new THREE.Vector3(...contact) : binding.origin;
  binding.pivot.position.copy(origin);
  const delta = binding.origin.clone().sub(origin);
  for (const rest of binding.restPositions) rest.node.position.copy(rest.position).add(delta);
  binding.pivot.scale.setScalar(sample.scale);
  binding.pivot.rotation.set(sample.tiltX, 0, sample.tiltZ);
}
