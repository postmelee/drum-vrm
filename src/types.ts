import type { EquipmentMechanism } from './equipment-types';
export type { EquipmentMechanism } from './equipment-types';
export type Vec3 = [number, number, number];
export type Quat = [number, number, number, number];
/** Numeric Rust contract. Geometry is reflected only at the native VRM adapter boundary. */
export interface HandControlFrame { forearm: Vec3; palm: Vec3; desiredPalm: Vec3; }
/** Bounded sampled envelopes, never a skin/clothing collision certificate. */
export interface BodyBox { center: Vec3; halfExtents: Vec3; }
export interface BodyGeometry {
  version: 1;
  source: 'mesh-estimate' | 'skeleton-fallback';
  torso: { center: Vec3; halfExtents: Vec3 };
  /** LH/RH, each upper arm and forearm radius, in metres. */
  armRadii: [[number, number], [number, number]];
  handRadii: [number, number];
  /** Optional LF/RF, thigh and shin radii; older profiles use segment proportions. */
  legRadii?: [[number, number], [number, number]];
  footRadii?: [number, number];
  /** Offset mesh envelopes in segment frames, paired LF/RF and thigh/shin. */
  legBoxes?: [[BodyBox, BodyBox], [BodyBox, BodyBox]];
  /** Measured local-box → rest-world frames; transport with the leg's minimum swing. */
  legRestFrames?: [[Quat, Quat], [Quat, Quat]];
  /** Offset footwear from ankle in the neutral tool frame. */
  footBoxes?: [BodyBox, BodyBox];
  /** Optional offsets from wrist to the sampled hand sphere, in tool frame. */
  handCenters?: [Vec3, Vec3];
  handCapsules?: [{ start: Vec3; end: Vec3; radius: number }, { start: Vec3; end: Vec3; radius: number }];
  sampleCount: number;
}
export interface RigProfile {
  schemaVersion: 1; id: string;
  coordinateSystem: 'legacy-y-up-z-forward-left-negative-x';
  roots: [Vec3, Vec3, Vec3, Vec3];
  lengths: [[number, number], [number, number], [number, number], [number, number]];
  restTips: [Vec3, Vec3, Vec3, Vec3]; bendPoles: [Vec3, Vec3, Vec3, Vec3];
  gripOffsets: [Vec3, Vec3, Vec3, Vec3]; tipOffsets: [Vec3, Vec3, Vec3, Vec3];
  contactRotations: [Quat, Quat, Quat, Quat]; handFrames?: [HandControlFrame, HandControlFrame]; torsoPivot?: Vec3; bodyGeometry?: BodyGeometry; pelvis: Vec3; chest: Vec3; head: Vec3;
}
export const RIG_POSE_FLOATS = 77;
export type Instrument = 'kick'|'snare'|'hihat'|'openHat'|'pedalHat'|'tomHigh'|'tomMid'|'tomLow'|'crash'|'ride';
/** Physical identity is separate from its voice; both crashes use the crash voice. */
export type PieceId = Instrument | 'crashRight';
export interface KitCapabilities {
  scope: 'sampled-body-proxies'; geometrySource: string;
  handTargetChecks: number; pairChecks: number; availablePairs: number;
  requiredPairsAvailable: boolean; unresolvedRequiredPairs: string[];
  pairs: { leftPieceId: string; rightPieceId: string; status: 'available' | 'not-found' | 'unmeasured' }[];
}
export interface KitFitProposal {
  kit: KitPiece[]; scale: number; translation: Vec3;
  pieceAdjustments: {pieceId: string; translation: Vec3}[];
  unreachablePieceIds: string[]; clearancePieceIds?: string[]; layoutViolations?: string[];
  complete: boolean; capabilities?: KitCapabilities;
  searchCandidates?: number; searchLimitReached?: boolean;
  seatOffset?: Vec3; searchFailures?: {stage: string; pieceId: string; branches: number}[];
}
/** Measured body bounds and playing surface, relative to the piece's contact.
 * Radius edits scale every offset/extent by radius / referenceRadius. */
export interface PieceGeometry {
  version: 1;
  referenceRadius: number;
  source: 'mesh-bounds' | 'procedural';
  body: { center: Vec3; halfExtents: Vec3 };
  surface: { center: Vec3; normal: Vec3; radius: number;
    radialProfile?: { origin: Vec3; rotation: Quat; points: [number, number][]; bellRadius?: number };
    motion?: { kind: 'translation'; axis: Vec3; travelMeters: number } };
}
/** Shared Rust equipment geometry in the legacy Y-up planning coordinates. */
export interface KitGeometry {
  version: 1;
  mechanisms?: EquipmentMechanism[];
  bodies: { pieceId: string; center: Vec3; halfExtents: Vec3 }[];
  supports: { pieceId: string; start: Vec3; end: Vec3; radius: number; role?: string }[];
  pedals: { pieceId: string; kind: 'board' | 'base'; center: Vec3; halfExtents: Vec3; rotation?: Quat }[];
  unresolvedPieceIds: string[];
}
export interface KitPiece { id: PieceId; instrument?: Instrument; label: string; position: Vec3; radius: number; geometry?: PieceGeometry; pedalForward?: Vec3; standRotation?: number; mount?: { kind: 'standClamp'; supportPieceId: PieceId }; }
export const pieceInstrument = (piece: KitPiece): Instrument => piece.instrument ?? (piece.id === 'crashRight' ? 'crash' : piece.id);
export const hitPieceId = (hit: Hit): PieceId => hit.pieceId ?? hit.instrument;
export interface Hit { pieceId?: PieceId; articulation?: string; sourceEventId?: string; eventIndex?: number; timeUs?: number; id: number; time: number; tick: number; track: number; channel: number; note: number; velocity: number; instrument: Instrument; limb: number | null; target: Vec3; }
export interface Diagnostic { code: string; message: string; hitId?: number; time?: number; }
export interface TrackInfo { index: number; name: string; noteCount: number; drumNotes: number; channels: number[]; }
export interface MappingAnalysis { selected: MappingProfile | null; ambiguous: boolean; totalNotes: number; candidates: {profile: MappingProfile; mapped: number; unmapped: number}[]; }
export interface MidiInfo { tracks: TrackInfo[]; suggestedTracks: number[]; format: number; ppq: number; suggestedChannel: number | null; mappingAnalysis: MappingAnalysis; }
export type MappingProfile = 'gm' | 'logicDkdGm' | 'songsterrDownloader';
export type HihatPolicy = 'literal'|'shortOpenClose';
export interface Selection { tracks: number[]; channel: number | null; mappingProfile?: MappingProfile; countInNote75?: boolean; hihatPolicy?: HihatPolicy; hihatOpenness?: number; omittedSourceEventIds?: string[]; handOverrides?: Record<string,0|1>; }
export type SourceMessage = { kind: 'noteOn'|'noteOff'; key: number; velocity: number }
  | { kind: 'controller'; controller: number; value: number } | { kind: 'programChange'; program: number }
  | { kind: 'pitchBend'; value: number } | { kind: 'polyAftertouch'; key: number; pressure: number }
  | { kind: 'channelAftertouch'; pressure: number };
export interface SourceEvent { id: string; track: number; eventIndex: number; tick: number; timeUs: number; channel: number; message: SourceMessage; }
export interface EventInterpretation { sourceEventId: string; kind: 'strike'|'countIn'|'unmapped'|'preserved'|'control'|'omitted'; relatedSourceEventId?: string; instrument?: Instrument; pieceId?: PieceId; articulation?: string; hitId?: number; }
export interface TempoAnchor { tick: number; time: number; microsecondsPerQuarter: number; }
export interface TimeSignature { tick: number; numerator: number; denominator: number; }
export interface HihatControl { kind: 'hihatClose'; sourceEventId: string; openSourceEventId: string; releaseStartTimeUs: number; openTimeUs: number; timeUs: number; openness: number; }
/** Derived contact intent. Planned never means a rendered mesh was verified touching. */
export interface ContactEvent {
  version: 1; id: string; sourceEventId: string; hitId: number | null;
  time: number; timeUs: number; velocity: number; limb: number | null;
  toolPart: 'stickTip' | 'sole' | 'beater'; role: 'primary' | 'mechanism' | 'pairedTool';
  pieceId: string | null; surfaceId: string; frame: 'pieceReference' | 'world';
  localPoint: Vec3; localNormal: Vec3 | null; mechanismOffset: Vec3;
  status: 'planned' | 'unassigned';
}
export interface HihatMotionTrack { travelMeters: number; anchors: { time: number; opening: number }[]; }
export interface PerformancePlan { contactEvents?: ContactEvent[]; hihatMotion?: HihatMotionTrack; omittedSourceEventIds?: string[]; handOverrides?: Record<string,0|1>; countInNote75?: boolean; hihatPolicy?: HihatPolicy; hihatOpenness?: number; controls?: HihatControl[]; mappingProfile?: MappingProfile; mappingVersion?: string; sourceEvents?: SourceEvent[]; interpretations?: EventInterpretation[]; engineVersion?: string; plannerVersion?: string; midiHash?: string; calibrationId?: string; preRoll?: number; tail?: number; schemaVersion: number; duration: number; hits: Hit[]; diagnostics: Diagnostic[]; kit: KitPiece[]; tempoCount: number; ppq?: number; endTick?: number; tempoMap?: TempoAnchor[]; timeSignatures?: TimeSignature[]; }
// Packed world-space XYZ: left arm, right arm, left leg, right leg, each
// root, middle joint, end joint, stick tip/foot toe (12 floats); pelvis,
// chest and head at 48, 51, 54. Total 57 floats. Metres, +Y up, +Z forward.
export const POSE_FLOATS = 57;
export const PRE_ROLL = 0.5;
export const TAIL = 0.25;
export const FPS = 30;
export const SAMPLE_RATE = 48000;
export const MAX_FILE_BYTES = 10 * 1024 * 1024;
export const outputFrameCount = (plan: PerformancePlan) => Math.ceil((PRE_ROLL + plan.duration + TAIL) * FPS);
export const outputDuration = (plan: PerformancePlan) => outputFrameCount(plan) / FPS;

export interface PoseInspection { equipmentClearanceMeters?:number; intentionalStickContact?:boolean; countIn?:{sourceEventId:string;time:number;contactPoint:Vec3}; scope: "skeletal-proxies"; torsoClearanceMeters: number; armClearanceMeters: number; stickClearanceMeters: number; crossClearanceMeters?: number; geometrySource?: string; boneLengthErrorMeters: number; poseFloats: number; }
