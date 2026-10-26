import { Quaternion, SkinnedMesh, Vector3, type Object3D } from 'three';
import type { VRMHumanoid, VRMHumanBoneName } from '@pixiv/three-vrm';
import type { BodyGeometry, RigProfile, Quat, Vec3 } from './types';

/** Sampling is deterministic and bounded across the whole model, not per mesh.
 * Bounds are estimates of weighted surface vertices, not triangle containment.
 * Garments weighted to torso/arms contribute; hair/head/legs are excluded by
 * semantic weights and the torso height band. No material or avatar names. */
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
    handRadii: [.025, .025], handCenters: [[0, 0, 0], [0, 0, 0]] };
}

type Region = 'torso' | 'leftUpperArm' | 'leftLowerArm' | 'leftHand' | 'rightUpperArm' | 'rightLowerArm' | 'rightHand';
const REGIONS: Region[] = ['torso', 'leftUpperArm', 'leftLowerArm', 'leftHand', 'rightUpperArm', 'rightLowerArm', 'rightHand'];

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
        : /^(left|right)(Thumb|Index|Middle|Ring|Little)/.test(name) ? `${name.startsWith('left') ? 'left' : 'right'}Hand` : null;
    groups.set(bone.node, region);
  }
  // Extra twist bones inherit the nearest semantic parent. Explicit head/leg
  // mappings stop traversal, preventing hair and skirts from becoming torso.
  const classify = (bone: Object3D): Region | null => {
    for (let node: Object3D | null = bone; node; node = node.parent) if (groups.has(node)) return groups.get(node)!;
    return null;
  };
  const meshes: SkinnedMesh[] = [];
  source.updateMatrixWorld(true);
  source.traverse(node => { if (node instanceof SkinnedMesh && node.geometry.getAttribute('position') && node.geometry.getAttribute('skinIndex') && node.geometry.getAttribute('skinWeight')) meshes.push(node); });
  const total = meshes.reduce((sum, mesh) => sum + mesh.geometry.getAttribute('position').count, 0);
  const stride = Math.max(1, Math.ceil(total / MAX_BODY_SAMPLES));
  const points = new Map<Region, Vector3[]>(REGIONS.map(region => [region, []]));
  let offset = 0, sampled = 0;
  for (const mesh of meshes) {
    const position = mesh.geometry.getAttribute('position'), indices = mesh.geometry.getAttribute('skinIndex'), weights = mesh.geometry.getAttribute('skinWeight');
    if (position.count !== indices.count || position.count !== weights.count || indices.itemSize !== 4 || weights.itemSize !== 4) { offset += position.count; continue; }
    mesh.skeleton.update();
    const boneRegions = mesh.skeleton.bones.map(classify);
    for (let i = (stride - offset % stride) % stride; i < position.count; i += stride) {
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
    offset += position.count;
  }
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
  const rawPoint = (name: VRMHumanBoneName) => { const node = humanoid.getRawBoneNode(name); return node ? toProfile(node.getWorldPosition(new Vector3())) : null; };
  for (const [limb, side] of (['left', 'right'] as const).entries()) {
    for (const [segment, name, next] of [[0, `${side}UpperArm`, `${side}LowerArm`], [1, `${side}LowerArm`, `${side}Hand`]] as const) {
      const a = rawPoint(name), b = rawPoint(next); if (!a || !b) continue;
      const axis = b.clone().sub(a), length = axis.length(); if (length < .03) continue; axis.divideScalar(length);
      const radii = points.get(name)!.flatMap(p => { const d = p.clone().sub(a), along = d.dot(axis); return along >= length * .05 && along <= length * .95 ? [d.addScaledVector(axis, -along).length()] : []; });
      if (radii.length >= MIN_REGION_SAMPLES) result.armRadii[limb][segment] = clamp(percentile(radii, .98) + .003, .005, .25);
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
