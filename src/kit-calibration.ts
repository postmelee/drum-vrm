import { hitPieceId, pieceInstrument, type PieceId, type KitPiece, type PerformancePlan, type Vec3 } from './types';

const IDS: readonly PieceId[] = ['kick','snare','hihat','openHat','pedalHat','tomHigh','tomMid','tomLow','crash','ride'];
/** Layout is a physical contact contract, separate from appearance and MIDI. */
export function validateKitLayout(kit: readonly KitPiece[]): KitPiece[] {
  if (kit.length < IDS.length || kit.length > IDS.length + 1 || new Set(kit.map(p => p.id)).size !== kit.length || IDS.some(id => !kit.some(p => p.id === id)) || kit.some(p => !IDS.includes(p.id) && p.id !== 'crashRight')) throw new Error('모든 드럼 타점이 한 번씩 필요합니다.');
  for (const piece of kit) {
    const expected = piece.id === 'crashRight' ? 'crash' : piece.id;
    if (pieceInstrument(piece) !== expected) throw new Error('드럼 부품과 악기 역할이 일치하지 않습니다.');
    const g=piece.geometry;
    if(g && (g.version!==1 || !['procedural','mesh-bounds'].includes(g.source) || !Number.isFinite(g.referenceRadius) || g.referenceRadius<.01 || g.referenceRadius>1 || !g.body.center.every(Number.isFinite) || !g.body.halfExtents.every(v=>Number.isFinite(v)&&v>=.001&&v<=2) || !g.surface.center.every(Number.isFinite) || !g.surface.normal.every(Number.isFinite) || Math.abs(Math.hypot(...g.surface.normal)-1)>1e-5 || !Number.isFinite(g.surface.radius) || g.surface.radius<.01 || g.surface.radius>1)) throw new Error('악기 물리 치수가 유효하지 않습니다.');
    const [x,y,z] = piece.position;
    if (piece.position.length !== 3 || !piece.position.every(Number.isFinite) || Math.abs(x)>2 || y<.02 || y>2.5 || z<-.5 || z>2.5 || !Number.isFinite(piece.radius) || piece.radius<=0 || piece.radius>1) throw new Error('드럼 타점이 조절 범위를 벗어납니다.');
    if ((piece.id==='kick'||piece.id==='pedalHat') && (y<.04||y>.20)) throw new Error('페달 높이는 4–20cm 범위에서 조절해 주세요.');
  }
  const closed = kit.find(p=>p.id==='hihat')!, open = kit.find(p=>p.id==='openHat')!;
  if (closed.position.some((v,i)=>v!==open.position[i])) throw new Error('열린 하이햇과 닫힌 하이햇은 같은 타점을 사용합니다.');
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
    if (!target) throw new Error('타격에 연결된 드럼 부품이 없습니다.');
    return {...hit,target:[...target]};
  }),diagnostics:plan.diagnostics.map(d=>({...d}))};
}
