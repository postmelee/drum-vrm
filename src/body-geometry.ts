import { Matrix4, Quaternion, SkinnedMesh, Vector3, type Object3D } from 'three';
import type { VRMHumanoid, VRMHumanBoneName } from '@pixiv/three-vrm';
import type { BodyBox, BodyGeometry, RigProfile, Quat, Vec3 } from './types';

/** Sampling is deterministic and bounded across the whole model, not per mesh.
 * Bounds are estimates of weighted surface vertices, not triangle containment.
 * Garments contribute to their semantic torso/limb regions; head/hair are
 * excluded. No material or avatar names. */
export const MAX_BODY_SAMPLES = 12000;
const MIN_REGION_SAMPLES = 24;
const clamp = (n: number, min: number, max: number) => Math.min(max, Math.max(min, n));
const tuple = (v: Vector3): Vec3 => [v.x, v.y, v.z];
const percentile = (values: number[], p: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor((sorted.length - 1) * p))];
};

/** Principal-axis capsule avoids the empty space in a wrist/hand sphere. Bounds
 * remain sampled percentiles, with a fixed physical clearance allowance. */
function handCapsule(vertices: Vector3[]) {
  const center = vertices.reduce((sum, p) => sum.add(p), new Vector3()).divideScalar(vertices.length);
  const deltas = vertices.map(p => p.clone().sub(center));
  const covariance = Array.from({ length: 3 }, (_, i) => Array.from({ length: 3 }, (_, j) => deltas.reduce((sum, d) => sum + d.getComponent(i) * d.getComponent(j), 0)));
  const largest = [0, 1, 2].reduce((a, b) => covariance[a][a] >= covariance[b][b] ? a : b);
  let axis = new Vector3().setComponent(largest, 1);
  for (let i = 0; i < 16; i++) {
    const values = axis.toArray();
    const next = new Vector3(...covariance.map(row => row.reduce((sum, n, j) => sum + n * values[j], 0)));
    if (next.lengthSq() < 1e-16) break; axis = next.normalize();
  }
  // Fix eigenvector sign for exact serialized reproducibility.
  if (axis.getComponent(largest) < 0) axis.negate();
  const projected = deltas.map(d => d.dot(axis)), low = percentile(projected, .01), high = percentile(projected, .99);
  const distances = deltas.map((d, i) => d.clone().addScaledVector(axis, -clamp(projected[i], low, high)).length());
  return { start: center.clone().addScaledVector(axis, low), end: center.clone().addScaledVector(axis, high), radius: percentile(distances, .98) + .003 };
}

export function skeletonBodyGeometry(profile: RigProfile): BodyGeometry {
  const width = new Vector3(...profile.roots[0]).distanceTo(new Vector3(...profile.roots[1]));
  const low = profile.pelvis[1] + .04, high = (profile.roots[0][1] + profile.roots[1][1]) / 2 - .025;
  return { version: 1, source: 'skeleton-fallback', sampleCount: 0,
    torso: { center: [(profile.roots[0][0] + profile.roots[1][0]) / 2, (low + high) / 2, (profile.roots[0][2] + profile.roots[1][2]) / 2],
      halfExtents: [clamp(width * .44, .055, .30), Math.max(.01, (high - low) / 2), clamp(width * .29, .045, .20)] },
    armRadii: [0, 1].map(i => { const upper = clamp(profile.lengths[i][0] * .085, .015, .035); return [upper, upper * .8]; }) as BodyGeometry['armRadii'],
    legRadii: [2, 3].map(i => [clamp(profile.lengths[i][0] * .14, .025, .09), clamp(profile.lengths[i][1] * .10, .02, .07)]) as NonNullable<BodyGeometry['legRadii']>,
    footRadii: [.04, .04],
    handRadii: [.025, .025], handCenters: [[0, 0, 0], [0, 0, 0]] };
}

type Region = 'torso' | `${'left' | 'right'}${'UpperArm' | 'LowerArm' | 'Hand' | 'UpperLeg' | 'LowerLeg' | 'Foot'}`;
const REGIONS: Region[] = ['torso', ...(['left', 'right'] as const).flatMap(side => (['UpperArm', 'LowerArm', 'Hand', 'UpperLeg', 'LowerLeg', 'Foot'] as const).map(part => `${side}${part}` as Region))];

/** Input vertices are world positions before the seated translation. The adapter
 * owns the native→engine reflection and supplies that single conversion here. */
export function measureBodyGeometry(
  source: Object3D, humanoid: VRMHumanoid, profile: RigProfile,
  toProfile: (world: Vector3) => Vector3,
  handToolRotations?: [Quat, Quat],
): BodyGeometry {
  const fallback = skeletonBodyGeometry(profile), groups = new Map<Object3D, Region | null>();
  for (const [name, bone] of Object.entries(humanoid.rawHumanBones)) {
    const region: Region | null = ['hips', 'spine', 'chest', 'upperChest'].includes(name) ? 'torso'
      : REGIONS.includes(name as Region) ? name as Region
        : /^(left|right)(Thumb|Index|Middle|Ring|Little)/.test(name) ? `${name.startsWith('left') ? 'left' : 'right'}Hand`
          : name === 'leftToes' ? 'leftFoot' : name === 'rightToes' ? 'rightFoot' : null;
    groups.set(bone.node, region);
  }
  // Extra twist bones inherit the nearest semantic parent. Explicit semantic
  // boundaries prevent head decoration or leg garments from becoming torso.
  const classify = (bone: Object3D): Region | null => {
    for (let node: Object3D | null = bone; node; node = node.parent) if (groups.has(node)) return groups.get(node)!;
    return null;
  };
  const meshes: SkinnedMesh[] = [];
  source.updateMatrixWorld(true);
  source.traverse(node => { if (node instanceof SkinnedMesh && node.geometry.getAttribute('position') && node.geometry.getAttribute('skinIndex') && node.geometry.getAttribute('skinWeight')) meshes.push(node); });
  // glTF primitives can share one position buffer while drawing disjoint index
  // ranges. Sampling the whole buffer measures unused outfits/accessories and
  // repeats unrelated surfaces for every material primitive.
  const drawnVertices = meshes.map(mesh => {
    const geometry = mesh.geometry, index = geometry.index;
    const available = index?.count ?? geometry.getAttribute('position').count;
    const begin = Math.max(0, geometry.drawRange.start), end = Math.min(available, begin + geometry.drawRange.count);
    const drawn = new Set<number>();
    const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
    const ranges = Array.isArray(mesh.material) ? geometry.groups : [{ start: begin, count: end - begin, materialIndex: 0 }];
    for (const group of ranges) {
      if (materials[group.materialIndex ?? 0]?.visible === false) continue;
      for (let i = Math.max(begin, group.start); i < Math.min(end, group.start + group.count); i++) drawn.add(index ? index.getX(i) : i);
    }
    return [...drawn].sort((a, b) => a - b);
  });
  const total = drawnVertices.reduce((sum, vertices) => sum + vertices.length, 0);
  const stride = Math.max(1, Math.ceil(total / MAX_BODY_SAMPLES));
  const points = new Map<Region, Vector3[]>(REGIONS.map(region => [region, []]));
  let offset = 0, sampled = 0;
  for (const [meshIndex, mesh] of meshes.entries()) {
    const vertices = drawnVertices[meshIndex];
    const position = mesh.geometry.getAttribute('position'), indices = mesh.geometry.getAttribute('skinIndex'), weights = mesh.geometry.getAttribute('skinWeight');
    if (position.count !== indices.count || position.count !== weights.count || indices.itemSize !== 4 || weights.itemSize !== 4) { offset += vertices.length; continue; }
    mesh.skeleton.update();
    const boneRegions = mesh.skeleton.bones.map(classify);
    for (let sample = (stride - offset % stride) % stride; sample < vertices.length; sample += stride) {
      const i = vertices[sample];
      const votes = new Map<Region, number>();
      let totalWeight = 0;
      for (let j = 0; j < 4; j++) {
        const weight = weights.getComponent(i, j), index = indices.getComponent(i, j);
        if (!Number.isFinite(weight) || weight < 0 || !Number.isInteger(index) || index < 0 || index >= boneRegions.length) continue;
        totalWeight += weight;
        const region = boneRegions[index]; if (region) votes.set(region, (votes.get(region) ?? 0) + weight);
      }
      if (totalWeight < .9 || totalWeight > 1.1) continue;
      const selected = [...votes].sort((a, b) => b[1] - a[1])[0];
      if (!selected || selected[1] / totalWeight < .65) continue;
      const v = new Vector3(); mesh.getVertexPosition(i, v); v.applyMatrix4(mesh.matrixWorld);
      const p = toProfile(v);
      if (![p.x, p.y, p.z].every(Number.isFinite)) continue;
      points.get(selected[0])!.push(p); sampled++;
    }
    offset += vertices.length;
  }
  const rawPoint = (name: VRMHumanBoneName) => { const node = humanoid.getRawBoneNode(name); return node ? toProfile(node.getWorldPosition(new Vector3())) : null; };
  const segmentRadius = (region: Region, start: VRMHumanBoneName, end: VRMHumanBoneName) => {
    const a = rawPoint(start), b = rawPoint(end); if (!a || !b) return null;
    const axis = b.clone().sub(a), length = axis.length(); if (length < .03) return null; axis.divideScalar(length);
    const radii = points.get(region)!.flatMap(p => {
      const d = p.clone().sub(a), along = d.dot(axis);
      return along >= length * .05 && along <= length * .95 ? [d.addScaledVector(axis, -along).length()] : [];
    });
    return radii.length >= MIN_REGION_SAMPLES ? clamp(percentile(radii, .98) + .003, .005, .25) : null;
  };
  const bodyRight = new Vector3(...profile.roots[1]).sub(new Vector3(...profile.roots[0])).normalize();
  const legEnvelopes: (BodyBox | null)[] = [], footEnvelopes: (BodyBox | null)[] = [];
  const legFrames: (Quat | null)[] = [];
  const localBox = (vertices: Vector3[], transform: (p: Vector3) => Vector3): BodyBox | null => {
    if (vertices.length < MIN_REGION_SAMPLES) return null;
    const local = vertices.map(p => transform(p.clone()));
    const limits = [0, 1, 2].map(i => { const values = local.map(p => p.getComponent(i)); return [percentile(values, .01) - .003, percentile(values, .99) + .003]; });
    const center = limits.map(([low, high]) => (low + high) / 2) as Vec3;
    const halfExtents = limits.map(([low, high]) => (high - low) / 2) as Vec3;
    if (Math.hypot(...center) > 3 || halfExtents.some(n => n < .001 || n > 1)) return null;
    return { center, halfExtents };
  };
  // Leg regions are independent of torso sample coverage. A mesh with sparse
  // torso weights must still retain measured trousers/shoes rather than silently
  // replacing every lower-limb envelope with skeleton proportions.
  for (const [limb, side] of (['left', 'right'] as const).entries()) {
    for (const [segment, name, next] of [[0, `${side}UpperLeg`, `${side}LowerLeg`], [1, `${side}LowerLeg`, `${side}Foot`]] as const) {
      const radius = segmentRadius(name, name, next);
      if (radius !== null) { fallback.legRadii![limb][segment] = radius; fallback.source = 'mesh-estimate'; }
      const a = rawPoint(name), b = rawPoint(next);
      let envelope: BodyBox | null = null;
      let frame: Quat | null = null;
      if (a && b) {
        const y = b.clone().sub(a).normalize(), x = bodyRight.clone().addScaledVector(y, -bodyRight.dot(y)).normalize();
        const z = x.clone().cross(y);
        if (x.lengthSq() > .9) {
          const length = a.distanceTo(b);
          const vertices = points.get(name)!.filter(p => { const along = p.clone().sub(a).dot(y); return along >= length * .05 && along <= length * .95; });
          envelope = localBox(vertices, p => { p.sub(a); return new Vector3(p.dot(x), p.dot(y), p.dot(z)); });
          if (envelope) frame = new Quaternion().setFromRotationMatrix(new Matrix4().makeBasis(x, y, z)).normalize().toArray();
        }
      }
      legEnvelopes.push(envelope);
      legFrames.push(frame);
    }
    const vertices = points.get(`${side}Foot`)!, ankle = rawPoint(`${side}Foot`);
    const inverse = new Quaternion(...profile.contactRotations[limb + 2]).invert();
    footEnvelopes.push(ankle ? localBox(vertices, p => p.sub(ankle).applyQuaternion(inverse)) : null);
    if (vertices.length >= MIN_REGION_SAMPLES) {
      const radius = segmentRadius(`${side}Foot`, `${side}Foot`, `${side}Toes`) ?? handCapsule(vertices).radius;
      fallback.footRadii![limb] = clamp(radius, .005, .25); fallback.source = 'mesh-estimate';
    }
  }
  if (legEnvelopes.every(Boolean)) {
    fallback.legBoxes = [[legEnvelopes[0]!, legEnvelopes[1]!], [legEnvelopes[2]!, legEnvelopes[3]!]];
    fallback.legRestFrames = [[legFrames[0]!, legFrames[1]!], [legFrames[2]!, legFrames[3]!]];
  }
  if (footEnvelopes.every(Boolean)) fallback.footBoxes = [footEnvelopes[0]!, footEnvelopes[1]!];
  if (fallback.source === 'mesh-estimate') fallback.sampleCount = sampled;
  const right = new Vector3(...profile.roots[1]).sub(new Vector3(...profile.roots[0])); right.y = 0; right.normalize();
  if (right.lengthSq() < .9) return fallback;
  const forward = new Vector3(-right.z, 0, right.x), origin = new Vector3(...fallback.torso.center);
  const low = origin.y - fallback.torso.halfExtents[1], high = origin.y + fallback.torso.halfExtents[1];
  const torso = points.get('torso')!.filter(p => p.y >= low && p.y <= high);
  if (torso.length < MIN_REGION_SAMPLES) return fallback;
  const local = torso.map(p => { const d = p.clone().sub(origin); return [d.dot(right), d.dot(forward)]; });
  const edges = [0, 1].map(axis => { const values = local.map(p => p[axis]); return [percentile(values, .01), percentile(values, .99)]; });
  const center = origin.clone().addScaledVector(right, (edges[0][0] + edges[0][1]) / 2).addScaledVector(forward, (edges[1][0] + edges[1][1]) / 2);
  const rx = (edges[0][1] - edges[0][0]) / 2 + .004, rz = (edges[1][1] - edges[1][0]) / 2 + .004;
  // Degenerate surfaces or unsupported scale do not establish usable volume.
  if (rx < .01 || rz < .01 || rx > 1 || rz > 1 || tuple(center).some(v => Math.abs(v) > 10)) return fallback;
  const result = structuredClone(fallback); result.source = 'mesh-estimate'; result.sampleCount = sampled;
  result.torso = { center: tuple(center), halfExtents: [rx, fallback.torso.halfExtents[1], rz] };
  for (const [limb, side] of (['left', 'right'] as const).entries()) {
    for (const [segment, name, next] of [[0, `${side}UpperArm`, `${side}LowerArm`], [1, `${side}LowerArm`, `${side}Hand`]] as const) {
      const radius = segmentRadius(name, name, next);
      if (radius !== null) result.armRadii[limb][segment] = radius;
    }
    const hand = rawPoint(`${side}Hand`), vertices = points.get(`${side}Hand`)!;
    if (!hand || !handToolRotations || vertices.length < MIN_REGION_SAMPLES) continue;
    // A hand is not a large sphere centered on its wrist. Fit an offset sphere
    // to the prepared palm/fingers, then express its center in the tool frame
    // used by Rust so the same measured volume follows every solved pose.
    const center = new Vector3(...[0, 1, 2].map(axis => percentile(vertices.map(p => p.getComponent(axis)), .5)));
    const radius = percentile(vertices.map(p => p.distanceTo(center)), .98) + .003;
    const local = center.sub(hand).applyQuaternion(new Quaternion(...handToolRotations[limb]).invert());
    if (local.length() <= .5 && radius >= .005 && radius <= .25) {
      result.handCenters![limb] = tuple(local); result.handRadii[limb] = radius;
    }
    const capsule = handCapsule(vertices), inverse = new Quaternion(...handToolRotations[limb]).invert();
    const start = capsule.start.sub(hand).applyQuaternion(inverse), end = capsule.end.sub(hand).applyQuaternion(inverse);
    if (start.length() <= .5 && end.length() <= .5 && capsule.radius >= .005 && capsule.radius <= .25) {
      result.handCapsules ??= [0, 1].map(i => ({ start: [...result.handCenters![i]], end: [...result.handCenters![i]], radius: result.handRadii[i] })) as NonNullable<BodyGeometry['handCapsules']>;
      result.handCapsules[limb] = { start: tuple(start), end: tuple(end), radius: capsule.radius };
    }
  }
  return result;
}

/** Validate optional saved numeric geometry before it reaches scene state.
 * Rust independently validates the same contract at the computation boundary. */
export function validateBodyGeometry(value: unknown): asserts value is BodyGeometry {
  const object = (v: unknown): Record<string, unknown> => { if (!v || typeof v !== 'object' || Array.isArray(v)) throw new Error('Invalid body geometry object.'); return v as Record<string, unknown>; };
  const vector = (v: unknown, length: number, min: number, max: number) => { if (!Array.isArray(v) || v.length !== length || v.some(n => typeof n !== 'number' || !Number.isFinite(n) || n < min || n > max)) throw new Error('Invalid body geometry bounds.'); };
  const geometry = object(value), torso = object(geometry.torso);
  if (geometry.version !== 1 || !['mesh-estimate', 'skeleton-fallback'].includes(String(geometry.source))) throw new Error('Unsupported body geometry version/source.');
  vector(torso.center, 3, -10, 10); vector(torso.halfExtents, 3, .01, 1);
  if (!Array.isArray(geometry.armRadii) || geometry.armRadii.length !== 2) throw new Error('Invalid arm geometry.');
  geometry.armRadii.forEach(v => vector(v, 2, .005, .25)); vector(geometry.handRadii, 2, .005, .25);
  if (geometry.legRadii !== undefined) {
    if (!Array.isArray(geometry.legRadii) || geometry.legRadii.length !== 2) throw new Error('Invalid leg geometry.');
    geometry.legRadii.forEach(v => vector(v, 2, .005, .25));
  }
  if (geometry.footRadii !== undefined) vector(geometry.footRadii, 2, .005, .25);
  const limbBox = (value: unknown) => {
    const shape = object(value);
    vector(shape.center, 3, -3, 3);
    if (Math.hypot(...shape.center as number[]) > 3) throw new Error('Invalid limb box offset.');
    vector(shape.halfExtents, 3, .001, 1);
  };
  if (geometry.legBoxes !== undefined) {
    if (!Array.isArray(geometry.legBoxes) || geometry.legBoxes.length !== 2) throw new Error('Invalid leg boxes.');
    geometry.legBoxes.forEach(pair => { if (!Array.isArray(pair) || pair.length !== 2) throw new Error('Invalid leg boxes.'); pair.forEach(limbBox); });
  }
  if (geometry.legRestFrames !== undefined) {
    if (geometry.legBoxes === undefined || !Array.isArray(geometry.legRestFrames) || geometry.legRestFrames.length !== 2) throw new Error('Invalid leg rest frames.');
    geometry.legRestFrames.forEach(pair => {
      if (!Array.isArray(pair) || pair.length !== 2) throw new Error('Invalid leg rest frames.');
      pair.forEach(frame => {
        vector(frame, 4, -1, 1);
        if (Math.abs((frame as number[]).reduce((sum, n) => sum + n * n, 0) - 1) > .001) throw new Error('Invalid leg rest frame rotation.');
      });
    });
  }
  if (geometry.footBoxes !== undefined) {
    if (!Array.isArray(geometry.footBoxes) || geometry.footBoxes.length !== 2) throw new Error('Invalid foot boxes.');
    geometry.footBoxes.forEach(limbBox);
  }
  if (geometry.handCenters !== undefined) {
    if (!Array.isArray(geometry.handCenters) || geometry.handCenters.length !== 2) throw new Error('Invalid hand geometry centers.');
    geometry.handCenters.forEach(v => { vector(v, 3, -.5, .5); if (Math.hypot(...v as number[]) > .5) throw new Error('Invalid hand geometry offset.'); });
  }
  if (geometry.handCapsules !== undefined) {
    if (!Array.isArray(geometry.handCapsules) || geometry.handCapsules.length !== 2) throw new Error('Invalid hand capsules.');
    geometry.handCapsules.forEach(value => {
      const capsule = object(value);
      for (const point of [capsule.start, capsule.end]) { vector(point, 3, -.5, .5); if (Math.hypot(...point as number[]) > .5) throw new Error('Invalid hand capsule offset.'); }
      vector([capsule.radius], 1, .005, .25);
    });
  }
  if (!Number.isInteger(geometry.sampleCount) || (geometry.sampleCount as number) < 0 || (geometry.sampleCount as number) > 1000000) throw new Error('Invalid body geometry sample count.');
}
