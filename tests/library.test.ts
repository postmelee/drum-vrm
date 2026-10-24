import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { buildLibraryKit, type LibraryPreset } from '../src/model-library';
import { applyHitResponse, bindHitResponse } from '../src/hit-response';
import { fitLibraryStandHardware } from '../src/scene';
import type { Vec3 } from '../src/types';

const sources = [
  ['library', 'innerscene-drum-kit.glb', '3cf8d568b604e99ce63ee82829d7f25044d996ed6cd42c39bc7f226919d1a5a7'],
  ['batoski', 'batoski-drum-kit.glb', '1ced36a850608f146407ad9387208209c59dd4784377a41c53727632bf00f895'],
  ['smoj', 'smoj-drum-kit.glb', '4893bf0cd89542862298456be893bce24742947d31c8293230b817e91629d5de'],
] as const;
describe('bundled instrument libraries', () => {
  it.each(['innerscene', 'batoski', 'smoj'])('%s has intact source and license attribution evidence', id => {
    const base = new URL('../public/models/', import.meta.url);
    const provenance = JSON.parse(readFileSync(new URL(`${id}-provenance.json`, base), 'utf8'));
    const notice = readFileSync(new URL('NOTICE.txt', base), 'utf8');
    expect(provenance.unmodifiedBinary).toBe(true);
    expect(notice).toContain(`${provenance.title} — ${provenance.creator}`);
    expect(notice).toContain(provenance.source);
    expect(notice).toContain(provenance.licenseUrl);
    expect(notice).toContain(provenance.runtimeAdaptation);
    expect(provenance.files.some((file: { file: string }) => file.file === provenance.licenseFile)).toBe(true);
    for (const file of provenance.files) {
      const bytes = readFileSync(new URL(file.file, base));
      expect(bytes.length).toBe(file.bytes);
      expect(createHash('sha256').update(bytes).digest('hex')).toBe(file.sha256);
    }
  });
  it.each(sources)('%s retains its licensed binary and independent surface contacts', async (preset, file, hash) => {
    const buffer = readFileSync(new URL(`../public/models/${file}`, import.meta.url));
    expect(createHash('sha256').update(buffer).digest('hex')).toBe(hash);
    const jsonLength = buffer.readUInt32LE(12);
    const gltf = JSON.parse(buffer.subarray(20, 20 + jsonLength).toString());
    expect((gltf.images ?? []).every((image: { uri?: string }) => !image.uri)).toBe(true);
    expect(gltf.buffers.every((item: { uri?: string }) => !item.uri)).toBe(true);
    // Decode the real mesh hierarchy without browser-only image decoding.
    // Materials and geometry remain unchanged; only test texture references go.
    for (const material of gltf.materials) {
      delete material.pbrMetallicRoughness.baseColorTexture;
      delete material.pbrMetallicRoughness.metallicRoughnessTexture;
      delete material.normalTexture;
      delete material.occlusionTexture;
      delete material.emissiveTexture;
    }
    gltf.images = [];
    gltf.textures = [];
    const bytes = Buffer.from(JSON.stringify(gltf));
    const alignedLength = Math.ceil(bytes.length / 4) * 4;
    const binary = buffer.subarray(28 + jsonLength);
    const fixture = Buffer.alloc(28 + alignedLength + binary.length, 0x20);
    fixture.writeUInt32LE(0x46546c67, 0);
    fixture.writeUInt32LE(2, 4);
    fixture.writeUInt32LE(fixture.length, 8);
    fixture.writeUInt32LE(alignedLength, 12);
    fixture.writeUInt32LE(0x4e4f534a, 16);
    bytes.copy(fixture, 20);
    fixture.writeUInt32LE(binary.length, 20 + alignedLength);
    fixture.writeUInt32LE(0x004e4942, 24 + alignedLength);
    binary.copy(fixture, 28 + alignedLength);
    const parsed = await new GLTFLoader().parseAsync(fixture.buffer.slice(fixture.byteOffset, fixture.byteOffset + fixture.byteLength), '');
    const kit = buildLibraryKit(parsed.scene, preset as LibraryPreset);
    expect(Object.keys(kit).sort()).toEqual(['kick', 'snare', 'hihat', 'pedalHat', 'tomHigh', 'tomMid', 'tomLow', 'crash', 'ride'].sort());
    let draws = 0;
    for (const group of Object.values(kit)) {
      expect(group.position.toArray()).toEqual([0, 0, 0]);
      expect(group.userData.contact).toHaveLength(3);
      expect(group.userData.contact.every(Number.isFinite)).toBe(true);
      group.updateMatrixWorld(true);
      const [x, y, z] = group.userData.contact;
      const ray = new THREE.Raycaster(new THREE.Vector3(x, y + .005, z), new THREE.Vector3(0, -1, 0));
      const hit = ray.intersectObjects(group.children, true)[0];
      expect(hit, group.name).toBeDefined();
      expect(hit.point.y, group.name).toBeCloseTo(y, 5);
      draws += group.children.length;
    }
    expect(draws).toBeLessThan(100);
    expect(kit.hihat!.userData.contact[0]).toBeLessThan(0);
    expect(kit.ride!.userData.contact[0]).toBeGreaterThan(0);
    expect(kit.kick!.userData.contact[1]).toBeLessThan(.1);
    expect(kit.kick!.userData.contact[1]).toBeGreaterThan(.025);
    expect(kit.snare!.userData.contact[1]).toBeGreaterThan(.5);
    for (const group of Object.values(kit)) for (const child of group.children) expect(['surface', 'shell', 'hardware', 'pedal']).toContain(child.userData.hitResponseRole);
    const minimumUsedY = (mesh: THREE.Mesh, world = false): number => {
      const positions = mesh.geometry.getAttribute('position'), index = mesh.geometry.index;
      let minimum = Infinity; const point = new THREE.Vector3();
      for (let n = 0; n < (index?.count ?? positions.count); n++) {
        point.fromBufferAttribute(positions, index ? index.getX(n) : n);
        if (world) point.applyMatrix4(mesh.matrixWorld);
        minimum = Math.min(minimum, point.y);
      }
      return minimum;
    };
    for (const id of ['snare', 'hihat', 'tomHigh', 'tomMid', 'tomLow', 'crash', 'ride'] as const) {
      const group = kit[id]!, contact = group.userData.contact as Vec3;
      const hardware = group.children.filter(child => child.userData.hitResponseRole === 'hardware') as THREE.Mesh[];
      const footed = hardware.filter(mesh => Math.abs(minimumUsedY(mesh)) < .025);
      fitLibraryStandHardware(group, contact[1], 1.15, 1.2);
      hardware.filter(mesh => !footed.includes(mesh)).forEach(mesh => expect(mesh.scale.toArray()).toEqual([1, 1, 1]));
      const fitting = new THREE.Group(); fitting.scale.setScalar(1.2); fitting.position.set(.3, 1.15, .4);
      group.position.set(-contact[0], -contact[1], -contact[2]); fitting.add(group); fitting.updateMatrixWorld(true);
      expect(group.localToWorld(new THREE.Vector3(...contact)).distanceTo(new THREE.Vector3(.3, 1.15, .4))).toBeLessThan(1e-7);
      footed.forEach(mesh => expect(Math.abs(minimumUsedY(mesh, true)), `${preset}/${id} hardware reaches the floor`).toBeLessThan(1e-6));
      for (const child of group.children.filter(node => node.userData.hitResponseRole !== 'hardware')) expect(child.scale.toArray()).toEqual([1, 1, 1]);
      fitting.remove(group); group.position.set(0, 0, 0);
    }
    for (const id of ['snare', 'hihat', 'tomLow', 'crash', 'ride'] as const) {
      expect(kit[id]!.children.some(child => child.userData.hitResponseRole === 'hardware'), `${preset}/${id} has static hardware`).toBe(true);
      expect(kit[id]!.children.some(child => ['surface', 'shell'].includes(child.userData.hitResponseRole)), `${preset}/${id} has a response part`).toBe(true);
      const root = kit[id]!, contact = root.userData.contact as Vec3;
      root.position.set(.2, .1, -.3); root.rotation.y = .3; root.scale.set(1.1, .9, 1.1); root.updateMatrixWorld(true);
      const fixed = root.children.filter(child => ['hardware', 'pedal'].includes(child.userData.hitResponseRole));
      const matrices = fixed.map(child => child.matrixWorld.clone());
      const worldContact = root.localToWorld(new THREE.Vector3(...contact));
      const binding = bindHitResponse(root, contact);
      expect(binding.fallback, `${preset}/${id} uses explicit response parts`).toBe(false);
      applyHitResponse(binding, { scale: 1.02, tiltX: -.02, tiltZ: .01 }); root.updateMatrixWorld(true);
      expect(binding.pivot.getWorldPosition(new THREE.Vector3()).distanceTo(worldContact)).toBeLessThan(1e-7);
      fixed.forEach((child, index) => expect(child.matrixWorld.elements).toEqual(matrices[index].elements));
      expect(binding.pivot.children.length).toBeGreaterThan(0);
    }
    for (const group of Object.values(kit)) group.traverse(node => {
      if (node instanceof THREE.Mesh) {
        node.geometry.dispose();
        (node.material as THREE.Material).dispose();
      }
    });
  });
});
