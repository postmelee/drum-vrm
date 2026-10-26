import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';
import { initSync } from '../src/wasm/drum_engine';
import * as THREE from 'three';
import { applyHitResponse, bindHitResponse, createHitResponseTrack, evaluateHitResponse, validateHitResponse } from '../src/hit-response';
import { createPieceModel, disposeModel, resolvePieceVisual } from '../src/model-procedural';
import type { Hit, Instrument, KitPiece } from '../src/types';

beforeAll(() => initSync({ module: readFileSync(new URL('../src/wasm/drum_engine_bg.wasm', import.meta.url)) }));

const hit = (time: number, id = 0, instrument: Instrument = 'snare', velocity = 100): Hit => ({ id, time, tick: id, track: 0, channel: 9, note: 38, velocity, instrument, limb: 0, target: [0, 1, 0] });
const settings = { enabled: true, strength: .65 };
const rest = { scale: 1, tiltX: 0, tiltZ: 0 };

describe('absolute-time hit response', () => {
  it('starts after impact, follows velocity, and returns to rest without a physics clock', () => {
    const strong = createHitResponseTrack([hit(1, 0, 'snare', 127)], 'snare');
    const soft = createHitResponseTrack([hit(1, 0, 'snare', 32)], 'snare');
    expect(evaluateHitResponse(strong, .99, settings)).toEqual(rest);
    expect(evaluateHitResponse(strong, 1, settings)).toEqual(rest);
    expect(evaluateHitResponse(strong, 1.045, settings).scale).toBeGreaterThan(evaluateHitResponse(soft, 1.045, settings).scale);
    expect(evaluateHitResponse(strong, 3, settings)).toEqual(rest);
    expect(evaluateHitResponse(strong, 1.045, { ...settings, enabled: false })).toEqual(rest);
    expect(evaluateHitResponse(strong, 1.045, { ...settings, strength: 0 })).toEqual(rest);
    expect(evaluateHitResponse(strong, 1.045, settings, true)).toEqual(rest);
  });
  it('gives identical seek/export samples and bounded response during dense repeated hits', () => {
    const hits = Array.from({ length: 2000 }, (_, index) => hit(index / 1000, index));
    const input = JSON.stringify(hits), track = createHitResponseTrack(hits, 'snare');
    const times = [0, .01, .5, 1.033333, 1.9, 3.6];
    const first = times.map(time => evaluateHitResponse(track, time, settings));
    [...times].reverse().forEach(time => {
      expect(evaluateHitResponse(track, time, settings)).toEqual(first[times.indexOf(time)]);
    });
    for (let frame = 0; frame < 120; frame++) {
      const sample = evaluateHitResponse(track, frame / 30, settings);
      expect(sample.scale).toBeGreaterThanOrEqual(1); expect(sample.scale).toBeLessThanOrEqual(1.02);
    }
    expect(JSON.stringify(hits)).toBe(input);
  });
  it('does not reset the previous response when a second hit starts', () => {
    const one = createHitResponseTrack([hit(0)], 'ride');
    const two = createHitResponseTrack([hit(0), hit(.08, 1)], 'ride');
    expect(evaluateHitResponse(two, .08, settings)).toEqual(evaluateHitResponse(one, .08, settings));
    const before = evaluateHitResponse(two, .08 - 1e-7, settings), after = evaluateHitResponse(two, .08 + 1e-7, settings);
    expect(Math.abs(before.tiltX - after.tiltX)).toBeLessThan(1e-6);
  });
  it('uses a small pulse for an unknown imported assembly, and validates strength', () => {
    const track = createHitResponseTrack([hit(0)], 'ride');
    const fallback = evaluateHitResponse(track, .045, settings, false, true);
    expect(fallback.scale).toBeGreaterThan(1); expect(fallback.scale).toBeLessThan(1.007);
    expect(fallback.tiltX).toBe(0); expect(fallback.tiltZ).toBe(0);
    expect(() => validateHitResponse({ enabled: true, strength: NaN })).toThrow();
    expect(() => validateHitResponse({ enabled: true, strength: 1.1 })).toThrow();
  });
});

describe('response pivots preserve targets and hardware', () => {
  it('keeps every procedural playing contact and stationary hardware fixed', () => {
    for (const preset of ['studio', 'vintage', 'electronic'] as const) for (const instrument of ['kick', 'snare', 'tomHigh', 'hihat', 'crash', 'ride'] as Instrument[]) {
      const piece: KitPiece = { id: instrument, label: instrument, position: [.2, instrument === 'kick' ? .08 : 1, .6], radius: .23 };
      const model = createPieceModel(piece, resolvePieceVisual(preset, instrument), preset, [piece]);
      const binding = bindHitResponse(model.root, piece.position);
      expect(binding.fallback).toBe(false);
      model.root.updateMatrixWorld(true);
      const fixed = model.root.children.filter(node => node !== binding.pivot).map(node => ({ node, matrix: node.matrixWorld.clone() }));
      applyHitResponse(binding, evaluateHitResponse(createHitResponseTrack([hit(0)], instrument), .045, settings));
      model.root.updateMatrixWorld(true);
      expect(binding.pivot.localToWorld(new THREE.Vector3()).distanceTo(new THREE.Vector3(...piece.position))).toBeLessThan(1e-9);
      fixed.forEach(({ node, matrix }) => expect(node.matrixWorld.elements).toEqual(matrix.elements));
      expect(binding.pivot.scale.x).toBe(binding.pivot.scale.y);
      disposeModel(model.root);
    }
  });
  it('retains fitted imported geometry at rest and does not include the target marker', () => {
    const root = new THREE.Group(), fit = new THREE.Group(); fit.position.set(.2, .7, -.4); fit.rotation.y = .7; fit.scale.setScalar(.8);
    const model = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshBasicMaterial()); fit.add(model); root.add(fit);
    const marker = new THREE.Mesh(new THREE.RingGeometry(), new THREE.MeshBasicMaterial()); marker.userData.hitResponseRole = 'marker'; root.add(marker);
    root.updateMatrixWorld(true); const before = model.matrixWorld.clone();
    const binding = bindHitResponse(root, [.2, 1.1, -.4], true); root.updateMatrixWorld(true);
    before.elements.forEach((value, index) => expect(model.matrixWorld.elements[index]).toBeCloseTo(value, 12));
    expect(marker.parent).toBe(root); expect(binding.fallback).toBe(true);
    applyHitResponse(binding, { scale: 1.005, tiltX: 0, tiltZ: 0 }); root.updateMatrixWorld(true);
    expect(binding.pivot.localToWorld(new THREE.Vector3()).toArray()).toEqual([.2, 1.1, -.4]);
    disposeModel(root);
  });
});
