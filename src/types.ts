export type Vec3 = [number, number, number];
export type Instrument = 'kick'|'snare'|'hihat'|'openHat'|'pedalHat'|'tomHigh'|'tomMid'|'tomLow'|'crash'|'ride';
export interface KitPiece { id: Instrument; label: string; position: Vec3; radius: number; }
export interface Hit { eventIndex?: number; timeUs?: number; id: number; time: number; tick: number; track: number; channel: number; note: number; velocity: number; instrument: Instrument; limb: number | null; target: Vec3; }
export interface Diagnostic { code: string; message: string; hitId?: number; time?: number; }
export interface TrackInfo { index: number; name: string; noteCount: number; drumNotes: number; channels: number[]; }
export interface MidiInfo { tracks: TrackInfo[]; suggestedTracks: number[]; format: number; ppq: number; }
export interface Selection { tracks: number[]; channel: number | null; }
export interface TempoAnchor { tick: number; time: number; microsecondsPerQuarter: number; }
export interface TimeSignature { tick: number; numerator: number; denominator: number; }
export interface PerformancePlan { engineVersion?: string; plannerVersion?: string; midiHash?: string; calibrationId?: string; preRoll?: number; tail?: number; schemaVersion: number; duration: number; hits: Hit[]; diagnostics: Diagnostic[]; kit: KitPiece[]; tempoCount: number; ppq?: number; endTick?: number; tempoMap?: TempoAnchor[]; timeSignatures?: TimeSignature[]; }
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
