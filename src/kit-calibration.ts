import { WORKER_MESSAGES } from './worker-messages';
import { hitPieceId, pieceInstrument, type PieceId, type KitPiece, type PerformancePlan, type Vec3 } from './types';

const IDS: readonly PieceId[] = ['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
/** Layout is a physical contact contract, separate from appearance and MIDI. */
export function validateKitLayout(kit: readonly KitPiece[]): KitPiece[] {
  if (kit.length < IDS.length || kit.length > IDS.length + 1 || new Set(kit.map(p => p.id)).size !== kit.length || IDS.some(id => !kit.some(p => p.id === id)) || kit.some(p => !IDS.includes(p.id) && p.id !== 'crashRight')) throw new Error(WORKER_MESSAGES.kitTargets);
  for (const piece of kit) {
    const expected = piece.id === 'crashRight' ? 'crash' : piece.id;
    if (pieceInstrument(piece) !== expected) throw new Error(WORKER_MESSAGES.pieceRole);
    if (piece.mount) {
      const host = kit.find(candidate => candidate.id === piece.mount!.supportPieceId);
      if (piece.id !== 'tomLow' || piece.mount.kind !== 'standClamp' || !host || host.id === piece.id || host.mount || !['ride','crash'].includes(pieceInstrument(host))) throw new Error(WORKER_MESSAGES.mount);
    }
    if (piece.pedalForward && (piece.id !== 'pedalHat' || piece.pedalForward.length !== 3 || !piece.pedalForward.every(Number.isFinite) || Math.abs(piece.pedalForward[1]) > 1e-8 || Math.abs(Math.hypot(...piece.pedalForward)-1) > 1e-6)) throw new Error(WORKER_MESSAGES.dimensions);
    const rotatable = ['snare','hihat','crash','ride'].includes(pieceInstrument(piece)) || (pieceInstrument(piece) === 'tomLow' && !piece.mount);
    if (piece.standRotation !== undefined && (!rotatable || !Number.isFinite(piece.standRotation) || Math.abs(piece.standRotation)>Math.PI)) throw new Error(WORKER_MESSAGES.dimensions);
    const g=piece.geometry;
    if(g && (g.version!==1 || !['procedural','mesh-bounds'].includes(g.source) || !Number.isFinite(g.referenceRadius) || g.referenceRadius<.01 || g.referenceRadius>1 || !g.body.center.every(Number.isFinite) || !g.body.halfExtents.every(v=>Number.isFinite(v)&&v>=.001&&v<=2) || !g.surface.center.every(Number.isFinite) || !g.surface.normal.every(Number.isFinite) || Math.abs(Math.hypot(...g.surface.normal)-1)>1e-5 || !Number.isFinite(g.surface.radius) || g.surface.radius<.01 || g.surface.radius>1)) throw new Error(WORKER_MESSAGES.dimensions);
    const bodyRotation = g?.body.rotation;
    if (bodyRotation !== undefined && (!Array.isArray(bodyRotation) || bodyRotation.length !== 4 || !bodyRotation.every(Number.isFinite) || Math.abs(bodyRotation.reduce((sum,value)=>sum+value*value,0)-1)>=1e-5)) throw new Error(WORKER_MESSAGES.dimensions);
    const motion = g?.surface.motion;
    if (motion && (motion.kind !== 'translation' || !Array.isArray(motion.axis) || motion.axis.length!==3 || !motion.axis.every(Number.isFinite)
      || Math.abs(Math.hypot(...motion.axis)-1)>1e-5 || !Number.isFinite(motion.travelMeters) || motion.travelMeters<0 || motion.travelMeters>.025)) throw new Error(WORKER_MESSAGES.dimensions);
    const radial = g?.surface.radialProfile;
    if (radial && (!Array.isArray(radial.origin) || radial.origin.length !== 3 || !radial.origin.every(v=>Number.isFinite(v)&&Math.abs(v)<=3)
      || !Array.isArray(radial.rotation) || radial.rotation.length !== 4 || !radial.rotation.every(Number.isFinite) || Math.abs(Math.hypot(...radial.rotation)-1)>1e-5
      || !Array.isArray(radial.points) || radial.points.length < 2 || radial.points.length > 32
      || radial.points.some((point,index)=>!Array.isArray(point)||point.length!==2||!Number.isFinite(point[0])||point[0]<0||point[0]>1||!Number.isFinite(point[1])||Math.abs(point[1])>1||index>0&&point[0]<=radial.points[index-1][0]))) throw new Error(WORKER_MESSAGES.dimensions);
    const [x,y,z] = piece.position;
    if (piece.position.length !== 3 || !piece.position.every(Number.isFinite) || Math.abs(x)>2 || y<.02 || y>2.5 || z<-.5 || z>2.5 || !Number.isFinite(piece.radius) || piece.radius<=0 || piece.radius>1) throw new Error(WORKER_MESSAGES.targetRange);
    if ((piece.id==='kick'||piece.id==='pedalHat') && (y<.04||y>.20)) throw new Error(WORKER_MESSAGES.pedalHeight);
  }
  const closed = kit.find(p=>p.id==='hihat')!, open = kit.find(p=>p.id==='openHat')!;
  if (closed.position.some((v,i)=>v!==open.position[i])) throw new Error(WORKER_MESSAGES.hiHatTarget);
  return structuredClone(kit) as KitPiece[];
}

export function moveKitTarget(kit: readonly KitPiece[], id: PieceId, position: Vec3): KitPiece[] {
  const coupled = id==='hihat'||id==='openHat';
  return validateKitLayout(kit.map(p => (p.id===id || coupled && (p.id==='hihat'||p.id==='openHat')) ? {...p,position:[...position]} : p));
}

export function applyKitLayout(plan: PerformancePlan, layout: readonly KitPiece[]): PerformancePlan {
  const kit = validateKitLayout(layout), targets = new Map(kit.map(p=>[p.id,p.position]));
  return {...plan,kit,hits:plan.hits.map(hit=>{
    const target = targets.get(hitPieceId(hit));
    if (!target) throw new Error(WORKER_MESSAGES.missingPiece);
    return {...hit,target:[...target]};
  }),diagnostics:plan.diagnostics.map(d=>({...d}))};
}

/** Ignore JSON object-key order and sub-nanometre round-trip noise, not actual
 * equipment edits. Keep the accepted descriptor rather than remeasure it. */
export function sameKitLayout(a: readonly KitPiece[], b: readonly KitPiece[]): boolean {
  const same = (left: unknown, right: unknown): boolean => {
    if (typeof left === 'number' && typeof right === 'number') return Number.isFinite(left) && Number.isFinite(right) && Math.abs(left - right) <= 1e-10;
    if (left === right) return true;
    if (Array.isArray(left) && Array.isArray(right)) return left.length === right.length && left.every((value, index) => same(value, right[index]));
    if (!left || !right || typeof left !== 'object' || typeof right !== 'object' || Array.isArray(left) || Array.isArray(right)) return false;
    const l = left as Record<string, unknown>, r = right as Record<string, unknown>;
    const keys = Object.keys(l).filter(key => l[key] !== undefined), other = Object.keys(r).filter(key => r[key] !== undefined);
    return keys.length === other.length && keys.every(key => Object.hasOwn(r, key) && same(l[key], r[key]));
  };
  return same(a.map(piece => ({...piece, instrument: pieceInstrument(piece)})), b.map(piece => ({...piece, instrument: pieceInstrument(piece)})));
}
