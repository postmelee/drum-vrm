import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { describe, expect, it } from 'vitest';
import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { buildLibraryKit } from '../src/model-library';

describe('bundled CC0 instrument library', () => {
  it('keeps the exact licensed binary and adapts all independent contacts', async () => {
    const buffer = readFileSync(new URL('../public/models/innerscene-drum-kit.glb', import.meta.url));
    expect(createHash('sha256').update(buffer).digest('hex')).toBe('3cf8d568b604e99ce63ee82829d7f25044d996ed6cd42c39bc7f226919d1a5a7');
    const jsonLength = buffer.readUInt32LE(12);
    const gltf = JSON.parse(buffer.subarray(20, 20 + jsonLength).toString());
    expect(gltf.images.every((image: { uri?: string }) => !image.uri)).toBe(true);
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
    const kit = buildLibraryKit(parsed.scene);
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
    expect(draws).toBeLessThan(70);
    expect(kit.hihat!.userData.contact[0]).toBeLessThan(0);
    expect(kit.ride!.userData.contact[0]).toBeGreaterThan(0);
    expect(kit.kick!.userData.contact[1]).toBeLessThan(.1);
    expect(kit.snare!.userData.contact[1]).toBeGreaterThan(.7);
    for (const group of Object.values(kit)) group.traverse(node => {
      if (node instanceof THREE.Mesh) {
        node.geometry.dispose();
        (node.material as THREE.Material).dispose();
      }
    });
  });
});
