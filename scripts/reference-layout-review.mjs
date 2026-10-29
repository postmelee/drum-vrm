// Numeric acceptance of one measured Studio layout, independent of MIDI.
// node scripts/reference-layout-review.mjs --mode inputs --output .local/reference-layout/inputs-<id>
// node scripts/reference-layout-review.mjs --mode fit --input <input.json> --output <new-directory>
// node scripts/reference-layout-review.mjs --mode review --output <new-directory> [--models native,twist]
// Review repeats original fitting, checks the adopted setup's fit idempotence,
// repeats preparation, then evaluates ordered/reversed poses.
// Inputs are geometry-only in-memory copies of unchanged licensed public VRMs.
// This does not establish skin/clothing clearance or full-song playability.
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { Quaternion, Vector3 } from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import * as wasm from '../src/wasm/drum_engine.js';
import { motionReviewLandmarks } from './lib/review-landmarks.mjs';
import { KINEMATIC_HALF_WINDOW_SECONDS, motionValidity, summarizeMotionSamples } from './lib/review-motion-metrics.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
process.chdir(root);
const args = process.argv.slice(2), options = {};
for (let index = 0; index < args.length; index += 2) {
  if (!['--mode', '--output', '--input', '--models', '--hardware'].includes(args[index]) || !args[index + 1]) throw Error('Expected --mode, --output, --input, --models or --hardware followed by a value');
  options[args[index].slice(2)] = args[index + 1];
}
const mode = options.mode ?? 'review';
const hardware = options.hardware ?? 'floor-legs';
if (!['floor-legs','suspended-floor'].includes(hardware)) throw Error('Unknown hardware preset');
if (mode === 'fit' && options.hardware) throw Error('Quick fit preserves the hardware encoded in its input');
if (!['inputs', 'fit', 'review'].includes(mode)) throw Error('Mode must be inputs, fit or review');
if (mode === 'fit' && !options.input) throw Error('Quick fit needs --input <input.json>');
if (options.input && mode !== 'fit') throw Error('--input is only accepted for quick fit');
const allIds = ['native', 'avatar-sample-b', 'seed-san', 'twist'];
const ids = options.models?.split(',') ?? allIds;
if (!ids.length || new Set(ids).size !== ids.length || ids.some(id => !allIds.includes(id))) throw Error('Unknown or repeated model ID');
const output = resolve(options.output ?? `.local/reference-layout/${new Date().toISOString().replaceAll(':', '-')}`);
if (!output.startsWith(resolve('.local/reference-layout') + '/')) throw Error('Evidence must stay under .local/reference-layout/');
if (fs.existsSync(output)) throw Error(`Refusing to overwrite an existing evidence directory: ${output}`);
fs.mkdirSync(output, { recursive: true });
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const jsonHash = object => hash(JSON.stringify(object));
const write = (name, object) => fs.writeFileSync(resolve(output, name), JSON.stringify(object, null, 2) + '\n', { flag: 'wx' });
const binary = fs.readFileSync('src/wasm/drum_engine_bg.wasm');
wasm.initSync({ module: binary });
const sourceFiles = execFileSync('git', ['ls-files', '-co', '--exclude-standard', 'crates/drum-engine', 'src', 'shared'], { encoding: 'utf8' })
  .split('\n').filter(Boolean).sort().map(path => ({ path, sha256: hash(fs.readFileSync(path)) }));
const environment = {
  createdAt: new Date().toISOString(), hardware: mode === 'fit' ? 'input-defined' : hardware, node: process.version, platform: `${process.platform}/${process.arch}`,
  commit: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  worktree: execFileSync('git', ['status', '--short'], { encoding: 'utf8' }).trim(),
  sourceTreeHash: jsonHash(sourceFiles), sourceFiles,
  wasmHash: hash(binary), scriptHash: hash(fs.readFileSync(fileURLToPath(import.meta.url))),
  landmarkHelperHash: hash(fs.readFileSync('scripts/lib/review-landmarks.mjs')),
  motionMetricsHelperHash: hash(fs.readFileSync('scripts/lib/review-motion-metrics.mjs')),
};
write('environment.json', environment);

async function geometryModel(bytes) {
  if (bytes.readUInt32LE(0) !== 0x46546c67 || bytes.readUInt32LE(4) !== 2) throw Error('Expected GLB v2');
  const size = bytes.readUInt32LE(12), json = JSON.parse(bytes.subarray(20, 20 + size).toString());
  // Retain bones, skin weights, morphs, constraints and vertex buffers. Only
  // image/material references are removed; no modified model file is written.
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

// This reports measurements, not a second implementation of fitter policy.
// Acceptance consumes Rust's layoutViolations and equipment unresolved IDs.
function layoutMetrics(kit, nominal) {
  const surface = piece => {
    const g = piece.geometry, scale = g ? piece.radius / g.referenceRadius : 1;
    return { center: piece.position.map((v, axis) => v + (g?.surface.center[axis] ?? 0) * scale), radius: (g?.surface.radius ?? piece.radius) * scale };
  };
  const snare = surface(kit.find(piece => piece.id === 'snare'));
  const pieces = kit.filter(piece => !['openHat', 'pedalHat'].includes(piece.id)).map(piece => {
    const baseline = nominal.find(other => other.id === piece.id), measured = surface(piece);
    const scale = piece.geometry ? piece.radius / piece.geometry.referenceRadius : 1;
    return { id: piece.id, mount: piece.mount ?? null, anchor: piece.position, ...measured,
      relativeToSnare: measured.center.map((v, axis) => v - snare.center[axis]),
      radiusRatioToNominal: piece.radius / baseline.radius,
      anchorDeltaFromNominal: piece.position.map((v, axis) => v - baseline.position[axis]),
      bodyHalfExtents: piece.geometry?.body.halfExtents.map(v => v * scale),
      surfaceNormal: piece.geometry?.surface.normal,
    };
  });
  const pairs = [];
  for (let i = 0; i < pieces.length; i++) for (let j = i + 1; j < pieces.length; j++) {
    const a = pieces[i], b = pieces[j], delta = b.center.map((v, axis) => v - a.center[axis]);
    pairs.push({ pieces: [a.id, b.id], delta, horizontalDistance: Math.hypot(delta[0], delta[2]), radiusSum: a.radius + b.radius });
  }
  return { pieces, pairs };
}

function maxNumericDifference(a, b) {
  if (typeof a === 'number' && typeof b === 'number') return Number.isFinite(a) && Number.isFinite(b) ? Math.abs(a - b) : Infinity;
  if (a === b) return 0;
  if (!a || !b || typeof a !== 'object' || typeof b !== 'object' || Array.isArray(a) !== Array.isArray(b)) return Infinity;
  const keys = Object.keys(a).sort(), other = Object.keys(b).sort();
  if (JSON.stringify(keys) !== JSON.stringify(other)) return Infinity;
  return Math.max(0, ...keys.map(key => maxNumericDifference(a[key], b[key])));
}

function equipmentMetrics(equipment) {
  const roles = {}, bounds = { min: [Infinity, Infinity, Infinity], max: [-Infinity, -Infinity, -Infinity] };
  const include = (point, halfExtents) => point.forEach((value, axis) => {
    bounds.min[axis] = Math.min(bounds.min[axis], value - halfExtents[axis]);
    bounds.max[axis] = Math.max(bounds.max[axis], value + halfExtents[axis]);
  });
  for (const box of [...equipment.bodies, ...equipment.pedals]) {
    if (!box.rotation) { include(box.center,box.halfExtents); continue; }
    const rotation=new Quaternion(...box.rotation);
    for(const x of [-1,1]) for(const y of [-1,1]) for(const z of [-1,1]) {
      const point=new Vector3(x*box.halfExtents[0],y*box.halfExtents[1],z*box.halfExtents[2]).applyQuaternion(rotation).add(new Vector3(...box.center));
      include(point.toArray(),[0,0,0]);
    }
  }
  for (const support of equipment.supports) {
    const length = Math.hypot(...support.end.map((v, axis) => v - support.start[axis]));
    const role = roles[support.role] ??= { count: 0, minLength: Infinity, maxLength: 0, minRadius: Infinity, maxRadius: 0 };
    role.count++; role.minLength = Math.min(role.minLength, length); role.maxLength = Math.max(role.maxLength, length);
    role.minRadius = Math.min(role.minRadius, support.radius); role.maxRadius = Math.max(role.maxRadius, support.radius);
    include(support.start, [support.radius, support.radius, support.radius]);
    include(support.end, [support.radius, support.radius, support.radius]);
  }
  return { bodyCount: equipment.bodies.length, supportCount: equipment.supports.length, bounds, roles };
}

function fitInput(input, repeat) {
  const profileJson = JSON.stringify(input.profile), kitJson = JSON.stringify(input.kit);
  const start = performance.now(), fit = JSON.parse(wasm.fit_kit(profileJson, kitJson)), fitMs = performance.now() - start;
  let fitDeterministic = null, repeatFitMs = null;
  if (repeat) {
    const secondStart = performance.now(), second = JSON.parse(wasm.fit_kit(profileJson, kitJson));
    repeatFitMs = performance.now() - secondStart;
    fitDeterministic = JSON.stringify(second) === JSON.stringify(fit);
  }
  return { id: input.id, inputHash: jsonHash(input), wasmHash: environment.wasmHash, fitMs, repeatFitMs, fitDeterministic,
    fit, nominalMetrics: layoutMetrics(input.kit, input.kit), fittedMetrics: layoutMetrics(fit.kit, input.kit) };
}

if (mode === 'fit') {
  const input = JSON.parse(fs.readFileSync(resolve(options.input), 'utf8'));
  write('input.json', input);
  const result = fitInput(input, false);
  result.equipment = JSON.parse(wasm.kit_geometry(JSON.stringify(result.fit.kit)));
  result.equipmentMetrics = equipmentMetrics(result.equipment);
  write('fit.json', result);
  console.log(JSON.stringify({ id: input.id, complete: result.fit.complete, layoutViolations: result.fit.layoutViolations,
    unresolved: result.equipment.unresolvedPieceIds, scale: result.fit.scale, fitMs: result.fitMs, output }, null, 2));
  if (!result.fit.complete || result.fit.layoutViolations?.length || result.equipment.unresolvedPieceIds.length) process.exitCode = 1;
} else {
  const vite = await createServer({ root, configFile: false, server: { middlewareMode: true, hmr: false, ws: false, watch: null }, appType: 'custom' });
  try {
    const [native, vrm, demos, scene, bodies, models, calibration, checkpoints, layouts, automaticFit, review, contacts] = await Promise.all([
      'rig-humanoid', 'rig-vrm', 'demo', 'scene', 'instrument-body', 'model-procedural', 'kit-calibration', 'review-checkpoints', 'kit-layout-presets', 'automatic-fit', 'performance-review', 'contact-events',
    ].map(name => vite.ssrLoadModule(`/src/${name}.ts`)));
    const nominal = layouts.applyKitHardwarePreset(scene.DEFAULT_KIT,hardware);
    for (const piece of nominal) {
      if (['openHat', 'pedalHat'].includes(piece.id)) continue;
      const built = bodies.createMeasuredInstrumentBody(piece, models.resolvePieceVisual('studio', piece.id), 'studio', nominal);
      try { piece.geometry = built.geometry; } finally { models.disposeModel(built.root); }
    }
    nominal.find(piece => piece.id === 'openHat').geometry = structuredClone(nominal.find(piece => piece.id === 'hihat').geometry);
    const studies = mode === 'review' ? [
      { id: 'posture', bytes: demos.makePostureDemo(), selection: { tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true } },
      { id: 'interpretation', bytes: demos.makeInterpretationDemo(), selection: { tracks: [1], channel: 0, mappingProfile: 'logicDkdGm', countInNote75: true, hihatPolicy: 'shortOpenClose', hihatOpenness: .2 } },
    ].map(study => ({ ...study, source: JSON.parse(wasm.plan_midi(study.bytes, JSON.stringify(study.selection))) })) : [];
    const summary = [];
    for (const id of ids) {
      let rig, modelHash = null, provenanceFile = null;
      if (id === 'native') rig = new native.NativeHumanoidRigAdapter();
      else {
        provenanceFile = `public/characters/${id}-provenance.json`;
        const provenance = JSON.parse(fs.readFileSync(provenanceFile)), bytes = fs.readFileSync(`public/characters/${provenance.file}`);
        modelHash = hash(bytes);
        if (bytes.length !== provenance.bytes || modelHash !== provenance.sha256) throw Error(`Source identity mismatch: ${id}`);
        rig = new vrm.VRMHumanoidRigAdapter(await geometryModel(bytes), `review-${id}`);
      }
      try {
        // Match startup: the native rig has an authored reference seat; imported
        // models derive theirs from semantic measurements before fitting.
        const kit = structuredClone(nominal), seat = id === 'native' ? rig.getCalibration() : rig.getAutomaticCalibration(kit), profile = rig.setCalibration(seat, kit);
        const input = { id, modelHash, provenanceFile, seat, profile, kit };
        write(`${id}-input.json`, input);
        if (mode === 'inputs') { summary.push({ id, inputHash: jsonHash(input), modelHash, seat }); continue; }
        const result = fitInput(input, true), applied = result.fit.complete ? result.fit.kit : kit;
        const appliedBefore = JSON.stringify(applied), inputBefore = JSON.stringify(input);
        const fittedSeat = automaticFit.fittedCalibration(seat, { kit: applied, proposal: result.fit, applied: result.fit.complete, message: '' });
        const candidateProfile = rig.measureCalibration(fittedSeat, applied);
        result.candidateMeasurementRestored = maxNumericDifference(rig.getProfile(), profile) === 0
          && maxNumericDifference(rig.getCalibration(), seat) === 0;
        const fittedProfile = rig.setCalibration(fittedSeat, applied);
        result.fittedSeat = fittedSeat;
        result.fittedProfile = fittedProfile;
        result.candidateMeasurementMatchesApplied = maxNumericDifference(candidateProfile, fittedProfile) === 0;
        result.boneLengthsPreserved = JSON.stringify(profile.lengths) === JSON.stringify(fittedProfile.lengths);
        // Equipment and subsequent solver calls consume the same accepted kit
        // and calibrated body that the browser uses for motion preparation.
        result.equipment = JSON.parse(wasm.kit_geometry(JSON.stringify(applied)));
        result.equipmentMetrics = equipmentMetrics(result.equipment);
        result.appliedFit = null;
        result.appliedFitIdempotent = null;
        if (result.fit.complete) {
          const start = performance.now();
          result.appliedFit = JSON.parse(wasm.fit_kit(JSON.stringify(fittedProfile), JSON.stringify(applied)));
          result.appliedFitMs = performance.now() - start;
          result.appliedFitMaxDifference = maxNumericDifference(result.appliedFit.kit, applied);
          result.appliedFitIdempotent = result.appliedFit.complete && result.appliedFit.scale === 1
            && result.appliedFit.translation.every(value => value === 0)
            && (result.appliedFit.seatOffset ?? [0, 0, 0]).every(value => value === 0)
            && result.appliedFit.pieceAdjustments.length === 0
            && Array.isArray(result.appliedFit.layoutViolations) && result.appliedFit.layoutViolations.length === 0
            && result.appliedFitMaxDifference < 1e-12;
        }
        result.studies = [];
        for (const study of studies) {
          const sourceBefore = JSON.stringify(study.source), prepared = calibration.applyKitLayout(study.source, applied);
          const start = performance.now(), run = wasm.Performance.from_json_with_profile(JSON.stringify(prepared), JSON.stringify(fittedProfile));
          const prepareMs = performance.now() - start;
          try {
            const plan = JSON.parse(run.plan_json()), beforeSeek = JSON.stringify(plan.kit);
            const landmarks = motionReviewLandmarks(plan, checkpoints.reviewCheckpoints(plan));
            const times = [...new Set(landmarks.flatMap(checkpoint => [checkpoint.time, checkpoint.time - .08, checkpoint.time + .08]))].sort((a, b) => a - b);
            const samples = times.map(time => {
              const pose = run.evaluate_rig_at(time); rig.applyPose(pose);
              return { time, pose: Array.from(pose), markers: rig.getContactSamples(), inspection: JSON.parse(run.inspect_pose_at(time)) };
            });
            let poseDeterministic = true, markerDifference = 0;
            for (const sample of [...samples].reverse()) {
              const pose = run.evaluate_rig_at(sample.time); rig.applyPose(pose);
              poseDeterministic &&= JSON.stringify(Array.from(pose)) === JSON.stringify(sample.pose);
              markerDifference = Math.max(markerDifference, maxNumericDifference(rig.getContactSamples(), sample.markers));
            }
            const sequenceTimes = [...new Set([...review.reviewTimes(plan), ...times])].sort((a, b) => a - b);
            const ordered = sequenceTimes.map(time => run.evaluate_rig_at(time));
            const shuffled = new Array(sequenceTimes.length);
            for (const index of review.reviewOrder(sequenceTimes.length)) shuffled[index] = run.evaluate_rig_at(sequenceTimes[index]);
            const sequence = review.summarizePoseReview(sequenceTimes, ordered, shuffled);
            sequence.proxy = review.summarizeProxyReview(sequenceTimes, sequenceTimes.map(time => JSON.parse(run.inspect_pose_at(time))));
            // Reuse the existing observation grid; derivatives add only t±h
            // evaluations, not another complete skin/shader render. Keep the
            // independent reverse/shuffled evaluations above uncached.
            const poses = new Map(sequenceTimes.map((time, index) => [time, ordered[index]]));
            const poseAt = time => {
              if (!poses.has(time)) poses.set(time, run.evaluate_rig_at(time));
              return poses.get(time);
            };
            const motionSamples = sequenceTimes.map(time => {
              const pose = poseAt(time); rig.applyPose(pose);
              return { time, pose, markers: rig.getContactSamples(),
                before: poseAt(time - KINEMATIC_HALF_WINDOW_SECONDS), after: poseAt(time + KINEMATIC_HALF_WINDOW_SECONDS) };
            });
            const { kinematics, posture } = summarizeMotionSamples(motionSamples);
            const validity = motionValidity(plan);
            const phaseReview = landmarks.map(landmark => ({ ...landmark, halfWindowSeconds: .08,
              ...summarizeMotionSamples(motionSamples.filter(sample => Math.abs(sample.time - landmark.time) <= .08 + 1e-9)),
              diagnostics: (plan.diagnostics ?? []).filter(diagnostic => Number.isFinite(diagnostic.time) && Math.abs(diagnostic.time - landmark.time) <= .08 + 1e-9),
            }));
            const planned = (plan.contactEvents ?? []).filter(event => event.status === 'planned' && event.toolPart !== 'beater');
            const impactMeasurements = [], missingImpactMarkers = [];
            for (const event of planned) {
              rig.applyPose(run.evaluate_rig_at(event.time));
              const actual = rig.getContactSamples().find(sample => sample.limb === event.limb)?.actual;
              const measured = actual && contacts.measurePlannedContact(event, plan.kit, [-actual[0], actual[1], actual[2]], plan.hihatMotion);
              if (measured) impactMeasurements.push(measured);
              else missingImpactMarkers.push(event.id);
            }
            const markerContactMaximumErrorMeters = Math.max(0, ...impactMeasurements.map(sample => sample.errorMeters));
            const actualContactsPassed = missingImpactMarkers.length === 0 && impactMeasurements.length === planned.length
              && impactMeasurements.every(sample => Number.isFinite(sample.errorMeters) && sample.errorMeters <= 1e-5);
            const repeated = wasm.Performance.from_json_with_profile(JSON.stringify(prepared), JSON.stringify(fittedProfile));
            let planDeterministic;
            try { planDeterministic = JSON.stringify(JSON.parse(repeated.plan_json())) === JSON.stringify(plan); }
            finally { repeated.free(); }
            // Exercise the actual worker-transfer boundary as well as direct
            // construction. Exact impact locks must survive serialization.
            let preparedRoundTrip = false, preparedRoundTripError = null;
            try {
              const restored = wasm.Performance.from_prepared_json(run.prepared_json(), plan.midiHash ?? '', JSON.stringify(fittedProfile));
              try {
                preparedRoundTrip = restored.plan_json() === run.plan_json()
                  && samples.every(sample => JSON.stringify(Array.from(restored.evaluate_rig_at(sample.time))) === JSON.stringify(sample.pose));
              } finally { restored.free(); }
            } catch (error) { preparedRoundTripError = String(error); }
            const record = { id: study.id, midiHash: hash(study.bytes), selection: study.selection, plannerVersion: plan.plannerVersion,
              prepareMs, hitCount: plan.hits.length, unassigned: plan.hits.filter(hit => hit.limb === null).map(hit => ({ id: hit.id, sourceEventId: hit.sourceEventId, time: hit.time, pieceId: hit.pieceId })),
              layoutRoundTripMaxDifference: maxNumericDifference(applied, plan.kit), layoutAfterSeekUnchanged: JSON.stringify(JSON.parse(run.plan_json()).kit) === beforeSeek,
              sourcePreserved: JSON.stringify(plan.sourceEvents) === JSON.stringify(study.source.sourceEvents) && JSON.stringify(study.source) === sourceBefore,
              poseDeterministic, markerDifference, planDeterministic, preparedRoundTrip, preparedRoundTripError, posesFinite: samples.every(sample => sample.pose.every(Number.isFinite)),
              diagnostics: plan.diagnostics, landmarks, samples, sequence, motionValidity: validity, kinematics, posture, phaseReview,
              impactMeasurements, missingImpactMarkers,
              markerContactMaximumErrorMeters, actualContactsPassed,
              observationScope: 'Sampled engine proxies and actual adapter markers; skin/clothing requires separate rendered review.' };
            result.studies.push(record);
          } finally { run.free(); }
        }
        result.inputsUnchanged = JSON.stringify(applied) === appliedBefore && JSON.stringify(input) === inputBefore;
        result.layoutPassed = result.fit.complete && Array.isArray(result.fit.layoutViolations) && result.fit.layoutViolations.length === 0
          && result.equipment.unresolvedPieceIds.length === 0 && result.fitDeterministic && result.boneLengthsPreserved && result.inputsUnchanged
          && result.candidateMeasurementRestored && result.candidateMeasurementMatchesApplied && result.appliedFitIdempotent
          && result.studies.every(study => study.layoutRoundTripMaxDifference < 1e-12
            && study.layoutAfterSeekUnchanged && study.sourcePreserved && study.poseDeterministic && study.markerDifference === 0 && study.planDeterministic && study.posesFinite
            && study.sequence.maxOrderDifference === 0 && study.sequence.nonfiniteValues === 0 && study.actualContactsPassed && study.preparedRoundTrip);
        // Note assignment is reported separately: stable layout is not proof
        // that a given motion/score is fully playable, or collision free.
        result.motionCoveragePassed = result.studies.every(study => study.unassigned.length === 0);
        result.motionValidityPassed = result.studies.every(study => study.motionValidity.passed
          && study.kinematics.observations > 0 && study.kinematics.invalidObservations === 0);
        result.sampledToolPathsPassed = result.studies.every(study => study.sequence.proxy.toolSamples === study.sequence.samples
          && study.sequence.proxy.toolIntersectingSamples === 0
          && study.sequence.proxy.toolEquipmentMinimumMeters >= 0 && study.sequence.proxy.toolPairMinimumMeters >= 0);
        result.passed = result.layoutPassed && result.motionCoveragePassed && result.motionValidityPassed && result.sampledToolPathsPassed;
        result.verificationScope = 'numeric-layout-motion-and-adapter-markers';
        result.renderedReview = { status: 'not-performed-by-this-script', requiredViews: ['front', 'top', 'left', 'right'],
          scope: 'Actual textured skin/clothing and instrument visibility at the recorded semantic landmarks.' };
        write(`${id}-review.json`, result);
        summary.push({ id, passed: result.passed, layoutPassed: result.layoutPassed, motionCoveragePassed: result.motionCoveragePassed,
          motionValidityPassed: result.motionValidityPassed, sampledToolPathsPassed:result.sampledToolPathsPassed,
          fitComplete: result.fit.complete, layoutViolations: result.fit.layoutViolations,
          fittedSeat, seatOffset: result.fit.seatOffset, appliedFitIdempotent: result.appliedFitIdempotent,
          equipmentUnresolved: result.equipment.unresolvedPieceIds, scale: result.fit.scale, fitMs: Math.round(result.fitMs), repeatFitMs: Math.round(result.repeatFitMs),
          studies: result.studies.map(({ id, prepareMs, unassigned, layoutRoundTripMaxDifference, sequence, motionValidity, kinematics, posture, phaseReview, markerContactMaximumErrorMeters, actualContactsPassed, preparedRoundTrip, preparedRoundTripError }) => ({ id, prepareMs: Math.round(prepareMs), unassigned: unassigned.length, layoutRoundTripMaxDifference, sequence, motionValidity, kinematics, posture, phaseReview, markerContactMaximumErrorMeters, actualContactsPassed, preparedRoundTrip, preparedRoundTripError })) });
        if (!result.passed) process.exitCode = 1;
      } finally { rig.dispose(); }
    }
    write('summary.json', summary);
    console.log(JSON.stringify({ mode, output, summary }, null, 2));
  } finally { await vite.close(); }
}
