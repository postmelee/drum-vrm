// Repeatable geometry/pose audit of the three unchanged, licensed public samples.
// Run after npm run build:wasm. Output is private under .local/public-posture-review/.
// Texture-free Node inspection is not a substitute for browser skin/clothing review.
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import * as wasm from '../src/wasm/drum_engine.js';

const root = fileURLToPath(new URL('../', import.meta.url));
process.chdir(root);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const binary = fs.readFileSync('src/wasm/drum_engine_bg.wasm');
wasm.initSync({ module: binary });
const output = resolve('.local/public-posture-review');
fs.mkdirSync(output, { recursive: true });

async function geometryModel(bytes) {
  if (bytes.readUInt32LE(0) !== 0x46546c67 || bytes.readUInt32LE(4) !== 2) throw Error('Expected GLB v2');
  const size = bytes.readUInt32LE(12), json = JSON.parse(bytes.subarray(20, 20 + size).toString());
  // Preserve bones, skin, constraints, vertex buffers and morphs. Discard only
  // image/material references in this in-memory copy; never export modified VRM.
  json.images = []; json.textures = []; json.samplers = []; json.materials = [];
  for (const mesh of json.meshes ?? []) for (const primitive of mesh.primitives ?? []) delete primitive.material;
  if (json.extensions?.VRMC_vrm?.meta) delete json.extensions.VRMC_vrm.meta.thumbnailImage;
  const materialExtensions = new Set(['KHR_texture_basisu', 'KHR_texture_transform', 'VRMC_materials_mtoon', 'VRMC_materials_hdr_emissiveMultiplier']);
  for (const key of ['extensionsUsed', 'extensionsRequired']) json[key] = (json[key] ?? []).filter(extension => !materialExtensions.has(extension));
  const encoded = Buffer.from(JSON.stringify(json)), padded = Buffer.alloc(Math.ceil(encoded.length / 4) * 4, 32);
  encoded.copy(padded);
  const rest = bytes.subarray(20 + size), header = Buffer.alloc(20);
  header.writeUInt32LE(0x46546c67, 0); header.writeUInt32LE(2, 4); header.writeUInt32LE(20 + padded.length + rest.length, 8);
  header.writeUInt32LE(padded.length, 12); header.writeUInt32LE(0x4e4f534a, 16);
  const rebuilt = Buffer.concat([header, padded, rest]), loader = new GLTFLoader();
  loader.register(parser => new VRMLoaderPlugin(parser));
  const gltf = await loader.parseAsync(rebuilt.buffer.slice(rebuilt.byteOffset, rebuilt.byteOffset + rebuilt.byteLength), '');
  if (!gltf.userData.vrm) throw Error('VRM loader produced no humanoid');
  return gltf.userData.vrm;
}

const vite = await createServer({ root, configFile: false, server: { middlewareMode: true, hmr: false, ws: false, watch: null }, appType: 'custom' });
try {
  const [rigModule, demos, scene, bodies, models, calibration, checkpoints, review, contacts] = await Promise.all([
    'rig-vrm', 'demo', 'scene', 'instrument-body', 'model-procedural', 'kit-calibration', 'review-checkpoints', 'performance-review', 'contact-events',
  ].map(name => vite.ssrLoadModule(`/src/${name}.ts`)));
  const source = JSON.parse(wasm.plan_midi(demos.makePostureDemo(), JSON.stringify({ tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true })));
  const results = [];
  for (const id of ['avatar-sample-b', 'seed-san', 'twist']) {
    const provenance = JSON.parse(fs.readFileSync(`public/characters/${id}-provenance.json`));
    const bytes = fs.readFileSync(`public/characters/${provenance.file}`);
    if (bytes.length !== provenance.bytes || hash(bytes) !== provenance.sha256) throw Error(`Source identity mismatch: ${id}`);
    const rig = new rigModule.VRMHumanoidRigAdapter(await geometryModel(bytes), `review-${id}`);
    let run;
    try {
      const kit = structuredClone(scene.DEFAULT_KIT);
      for (const piece of kit) {
        if (['openHat', 'pedalHat'].includes(piece.id)) continue;
        piece.geometry = bodies.createMeasuredInstrumentBody(piece, models.resolvePieceVisual('studio', piece.id), 'studio', kit).geometry;
      }
      kit.find(piece => piece.id === 'openHat').geometry = structuredClone(kit.find(piece => piece.id === 'hihat').geometry);
      const seat = rig.getAutomaticCalibration(kit), initialProfile = rig.setCalibration(seat, kit);
      let start = performance.now();
      const fit = JSON.parse(wasm.fit_kit(JSON.stringify(initialProfile), JSON.stringify(kit))), fitMs = performance.now() - start;
      const layout = fit.complete ? fit.kit : kit, profile = rig.setCalibration(seat, layout);
      start = performance.now();
      run = wasm.Performance.from_json_with_profile(JSON.stringify(calibration.applyKitLayout(source, layout)), JSON.stringify(profile));
      const prepareMs = performance.now() - start, plan = JSON.parse(run.plan_json());
      const samples = checkpoints.reviewCheckpoints(plan).map(checkpoint => {
        const pose = run.evaluate_rig_at(checkpoint.time); rig.applyPose(pose);
        return { ...checkpoint, pose: [...pose], contacts: rig.getContactSamples(), inspection: JSON.parse(run.inspect_pose_at(checkpoint.time)) };
      });
      let deterministic = true;
      for (const sample of [...samples].reverse()) {
        const pose = run.evaluate_rig_at(sample.time); rig.applyPose(pose);
        deterministic &&= JSON.stringify([...pose]) === JSON.stringify(sample.pose) && JSON.stringify(rig.getContactSamples()) === JSON.stringify(sample.contacts);
      }
      const times = review.reviewTimes(plan), ordered = times.map(time => run.evaluate_rig_at(time));
      const shuffled = new Array(times.length);
      for (const index of review.reviewOrder(times.length)) shuffled[index] = run.evaluate_rig_at(times[index]);
      const sequence = review.summarizePoseReview(times, ordered, shuffled);
      sequence.proxy = review.summarizeProxyReview(times, times.map(time => JSON.parse(run.inspect_pose_at(time))));
      deterministic &&= sequence.maxOrderDifference === 0 && sequence.nonfiniteValues === 0;
      const impactMeasurements = [];
      for (const event of plan.contactEvents.filter(event => event.status === 'planned' && event.toolPart !== 'beater')) {
        rig.applyPose(run.evaluate_rig_at(event.time));
        const actual = rig.getContactSamples().find(sample => sample.limb === event.limb)?.actual;
        if (actual) impactMeasurements.push(contacts.measurePlannedContact(event, plan.kit, [-actual[0], actual[1], actual[2]], plan.hihatMotion));
      }
      const sourceFactsPreserved = JSON.stringify(plan.sourceEvents) === JSON.stringify(source.sourceEvents);
      const result = { id, modelHash: provenance.sha256, wasmHash: hash(binary), plannerVersion: plan.plannerVersion, fitMs, prepareMs,
        profile, kit: layout, plan, fitComplete: fit.complete, scale: fit.scale, searchCandidates: fit.searchCandidates, capabilities: fit.capabilities,
        sequence, impactMeasurements, unassigned: plan.hits.filter(hit => hit.limb === null).length, sourceFactsPreserved, deterministic, diagnostics: plan.diagnostics, samples };
      fs.writeFileSync(resolve(output, `${id}.json`), JSON.stringify(result, null, 2) + '\n');
      results.push({ id, fitComplete: result.fitComplete, scale: fit.scale, fitMs: Math.round(fitMs), prepareMs: Math.round(prepareMs), unassigned: result.unassigned, sourceFactsPreserved, deterministic });
      if (!sourceFactsPreserved || !deterministic) process.exitCode = 1;
    } finally { run?.free(); rig.dispose(); }
  }
  console.table(results);
  console.log(`Detailed markers, angles and explicit remaining diagnostics: ${output}`);
} finally { await vite.close(); }
