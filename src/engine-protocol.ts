import type { KitPiece, PerformancePlan, RigProfile, Selection } from './types';

export type EngineRequest =
  | { kind: 'inspect'; bytes: Uint8Array }
  | { kind: 'source'; bytes: Uint8Array; selection: Selection }
  | { kind: 'plan'; bytes: Uint8Array; selection: Selection; profile: RigProfile | null; layout: KitPiece[] | null }
  | { kind: 'prepare'; plan: PerformancePlan; profile: RigProfile | null };
export interface PreparedResponse { plan: PerformancePlan; prepared: string }
