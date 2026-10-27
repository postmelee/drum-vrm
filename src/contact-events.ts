import type { ContactEvent, HihatMotionTrack, KitPiece, PerformancePlan, Vec3 } from './types';

export interface WorldContact { position: Vec3; normal: Vec3 | null; }
/** Same absolute-time smoothstep interpolation as Rust hihat_motion::evaluate. */
export function sampleHihatOpening(track: HihatMotionTrack | undefined, time: number): number {
  if (!track || !Number.isFinite(time)) return 0;
  let low = 0, high = track.anchors.length;
  while (low < high) { const mid = (low + high) >>> 1; if (track.anchors[mid].time <= time) low = mid + 1; else high = mid; }
  const previous = track.anchors[low - 1], next = track.anchors[low];
  if (!previous) return 0;
  if (!next) return previous.opening;
  const t = Math.max(0, Math.min(1, (time - previous.time) / (next.time - previous.time)));
  return previous.opening + (next.opening - previous.opening) * t * t * (3 - 2 * t);
}

/** Planning coordinates, before the renderer's reflected kit/display basis.
 * The record contains the exact impact displacement. A later timestamp moves
 * its afterglow with the upper hi-hat instead of adding that displacement twice. */
export function resolveContactEvent(event: ContactEvent, kit: readonly KitPiece[], track?: HihatMotionTrack, time = event.time): WorldContact | null {
  if (event.frame === 'world') return { position: [...event.localPoint], normal: event.localNormal && [...event.localNormal] };
  const piece = kit.find(piece => piece.id === event.pieceId);
  if (!piece) return null;
  const ratio = piece.radius / (piece.geometry?.referenceRadius ?? piece.radius);
  const position = piece.position.map((value, axis) => value + event.localPoint[axis] * ratio) as Vec3;
  if (track && event.toolPart === 'stickTip' && (piece.id === 'hihat' || piece.id === 'openHat')) {
    const motion = piece.geometry ? piece.geometry.surface.motion : { axis: [0, 1, 0] as Vec3, travelMeters: track.travelMeters };
    const opening = sampleHihatOpening(track, time);
    for (let axis = 0; axis < 3; axis++) position[axis] += (motion ? motion.axis[axis] * motion.travelMeters * opening : 0) - event.mechanismOffset[axis];
  }
  return { position, normal: event.localNormal && [...event.localNormal] };
}

const displayPiece = (id: string | null): string | null => id === 'openHat' || id === 'pedalHat' ? 'hihat' : id;
/** Instrument feedback is a planned surface strike, never an unassigned MIDI
 * note or a foot pressing a pedal. Kick feedback uses its secondary beater event. */
export function contactFeedbackEvents(plan: PerformancePlan, pieceId: string): ContactEvent[] {
  return (plan.contactEvents ?? []).filter(event =>
    event.status === 'planned' && event.limb !== null && displayPiece(event.pieceId) === pieceId
    && event.toolPart !== 'sole' && event.role !== 'pairedTool' && event.localNormal !== null,
  ).sort((a, b) => a.time - b.time || a.id.localeCompare(b.id));
}

export function latestContactEvent(events: readonly ContactEvent[], time: number, lifetime = .35): ContactEvent | null {
  let low = 0, high = events.length;
  while (low < high) { const mid = (low + high) >>> 1; if (events[mid].time <= time) low = mid + 1; else high = mid; }
  const event = events[low - 1];
  return event && event.status === 'planned' && time - event.time < lifetime ? event : null;
}

/** Actual marker versus intended surface point. This does not certify triangle
 * contact or clear clothing; callers must not relabel it as realized contact. */
export function measurePlannedContact(event: ContactEvent, kit: readonly KitPiece[], actual: Vec3, track?: HihatMotionTrack) {
  const intended = resolveContactEvent(event, kit, track);
  if (!intended) return null;
  const delta = actual.map((value, axis) => value - intended.position[axis]) as Vec3;
  const errorMeters = Math.hypot(...delta);
  const normalErrorMeters = intended.normal ? delta.reduce((sum, value, axis) => sum + value * intended.normal![axis], 0) : null;
  return { eventId: event.id, status: event.status, planned: intended.position, actual: [...actual] as Vec3,
    errorMeters, normalErrorMeters,
    tangentErrorMeters: normalErrorMeters === null ? null : Math.sqrt(Math.max(0, errorMeters ** 2 - normalErrorMeters ** 2)),
    scope: 'actual-marker-to-planned-contact' as const };
}
