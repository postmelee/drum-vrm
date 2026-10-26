import source from '../shared/default-kit.json?raw';
import type { KitPiece, Vec3 } from './types';

/** One numeric seed for Rust planning, the renderer and default foot contacts.
 * Actual appearance bounds and avatar fitting are measured independently. */
export const DEFAULT_KIT = JSON.parse(source) as KitPiece[];
export const DEFAULT_PEDALS = ['pedalHat', 'kick'].map(id => [...DEFAULT_KIT.find(piece => piece.id === id)!.position]) as [Vec3, Vec3];
