import source from '../shared/default-kit.json?raw';
import type { KitPiece, Vec3 } from './types';

/** Reviewed reference arrangement shared by planning, rendering and foot contacts.
 * Imported bodies are measured against this arrangement before any layout repair. */
export const DEFAULT_KIT = JSON.parse(source) as KitPiece[];
export const DEFAULT_PEDALS = ['pedalHat', 'kick'].map(id => [...DEFAULT_KIT.find(piece => piece.id === id)!.position]) as [Vec3, Vec3];

/** Authored seating for the built-in skeleton, certified with DEFAULT_KIT.
 * Imported humanoids use their actual bone measurements, not this preset. */
export const NATIVE_REFERENCE_SEAT = { seatHeight: .79, seatForward: .14, palmOrientation: 'natural' as const };
