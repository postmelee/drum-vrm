import { readFile, readdir, writeFile, mkdir, access } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { execFileSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const json = async path => JSON.parse(await readFile(path, 'utf8'));
const rootPackage = await json(join(root, 'package.json'));
const npmLock = await json(join(root, 'package-lock.json'));
const exists = async path => { try { await access(path); return true; } catch { return false; } };
const sourceRepo = value => (typeof value === 'string' ? value : value?.url ?? '').replace(/^git\+/, '').replace(/\.git$/, '');
// midly's published include list omits LICENSE. This exact upstream text was
// verified at the crate's recorded .cargo_vcs_info.json commit on 2026-10-24.
const midlyCommit = '816cb2a585b4d5d1fd7aeb0cb35668f21dc00869';
const midlyLicenseUrl = `https://raw.githubusercontent.com/negamartin/midly/${midlyCommit}/LICENSE`;
const midlyLicense = `This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org>`;

async function licenseFiles(directory, explicit) {
  const names = (await readdir(directory)).filter(name => /^(unlicen[sc]e|licen[sc]e|copying|notice|copyright)([._-].*)?$/i.test(name));
  if (explicit && !names.includes(explicit)) names.push(explicit);
  const contents = [];
  for (const name of names.sort()) {
    try { contents.push({ name, text: (await readFile(join(directory, name), 'utf8')).trimEnd() }); } catch { /* Non-file entries. */ }
  }
  if (!contents.length) throw new Error('Dependency has no readable license file; review its published source before release.');
  return contents;
}

async function resolveInstalled(name, from) {
  let directory = from;
  while (directory.startsWith(root)) {
    const candidate = join(directory, 'node_modules', name);
    if (await exists(join(candidate, 'package.json'))) return candidate;
    const parent = dirname(directory);
    if (parent === directory) break;
    directory = parent;
  }
  throw new Error(`Missing installed runtime dependency: ${name}. Run npm ci first.`);
}

const npmPackages = [];
const seen = new Set();
async function visitNpm(name, from) {
  const directory = await resolveInstalled(name, from);
  const key = relative(root, directory).replaceAll('\\', '/');
  if (seen.has(key)) return;
  seen.add(key);
  const pkg = await json(join(directory, 'package.json'));
  const locked = npmLock.packages[key];
  if (!locked || locked.version !== pkg.version || !locked.integrity || !locked.resolved) {
    throw new Error(`Missing or mismatched lock evidence for ${name}.`);
  }
  npmPackages.push({
    name: pkg.name, version: pkg.version, license: pkg.license, source: locked.resolved,
    checksum: locked.integrity, repository: sourceRepo(pkg.repository),
    scope: pkg.name.startsWith('@types/') ? 'Type declarations; no browser runtime code' : 'Browser runtime',
    licenses: await licenseFiles(directory),
  });
  for (const dependency of Object.keys(pkg.dependencies ?? {}).sort()) await visitNpm(dependency, directory);
  for (const dependency of Object.keys(pkg.optionalDependencies ?? {}).sort()) {
    try { await visitNpm(dependency, directory); } catch (error) {
      if (!String(error).includes('Missing installed runtime dependency')) throw error;
    }
  }
}
for (const name of Object.keys(rootPackage.dependencies ?? {}).sort()) await visitNpm(name, root);
npmPackages.sort((a, b) => a.name.localeCompare(b.name));

if (!await exists(join(root, 'Cargo.lock'))) throw new Error('Cargo.lock is required; resolve the locked Rust dependencies first.');
const cargoLock = await readFile(join(root, 'Cargo.lock'), 'utf8');
const cargoChecksums = new Map();
for (const section of cargoLock.split('[[package]]').slice(1)) {
  const value = name => section.match(new RegExp(`^${name} = "([^"]+)"`, 'm'))?.[1];
  cargoChecksums.set(`${value('name')}@${value('version')}`, value('checksum'));
}
let metadata;
try {
  metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked', '--offline', '--filter-platform', 'wasm32-unknown-unknown'], {
    cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'],
  }));
} catch {
  throw new Error('Locked offline cargo metadata failed. Install the pinned Rust toolchain and fetch/build locked dependencies first.');
}
const resolvedIds = new Set(metadata.resolve.nodes.map(node => node.id));
const cargoPackages = [];
for (const pkg of metadata.packages.filter(pkg => pkg.source?.startsWith('registry+') && resolvedIds.has(pkg.id))) {
  const checksum = cargoChecksums.get(`${pkg.name}@${pkg.version}`);
  if (!checksum) throw new Error(`Missing Cargo.lock checksum for ${pkg.name}@${pkg.version}`);
  let licenses;
  let licenseSource;
  if (pkg.name === 'midly' && pkg.version === '0.5.3') {
    const vcs = await json(join(dirname(pkg.manifest_path), '.cargo_vcs_info.json'));
    if (vcs.git.sha1 !== midlyCommit || pkg.license !== 'Unlicense') throw new Error('midly license exception no longer matches source evidence.');
    licenses = [{ name: 'LICENSE (upstream commit; omitted from crate archive)', text: midlyLicense }];
    licenseSource = midlyLicenseUrl;
  } else {
    licenses = await licenseFiles(dirname(pkg.manifest_path), pkg.license_file);
  }
  cargoPackages.push({
    name: pkg.name, version: pkg.version, license: pkg.license ?? 'See license file',
    source: `https://crates.io/api/v1/crates/${pkg.name}/${pkg.version}/download`,
    checksum: `sha256:${checksum}`, repository: sourceRepo(pkg.repository),
    scope: 'Rust WASM dependency closure, including build/proc-macro dependencies',
    licenses, licenseSource,
  });
}
cargoPackages.sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));

const rustVersion = execFileSync('rustc', ['--version', '--verbose'], { cwd: root, encoding: 'utf8' });
const rustCommit = rustVersion.match(/^commit-hash: (\w+)$/m)?.[1];
const rustRelease = rustVersion.match(/^release: ([\w.]+)$/m)?.[1];
if (!rustCommit || !rustRelease) throw new Error('Cannot identify Rust standard library source.');
const sysroot = execFileSync('rustc', ['--print', 'sysroot'], { cwd: root, encoding: 'utf8' }).trim();
const rustNoticePath = join(sysroot, 'share/doc/rust/COPYRIGHT-library.html');
const rustHeading = `\n${'='.repeat(78)}\nRUST STANDARD LIBRARY ${rustRelease}\n${'='.repeat(78)}\nSource commit: ${rustCommit}\n`;
let rustSection;
if (await exists(rustNoticePath)) {
  const rustHtml = await readFile(rustNoticePath, 'utf8');
  // The toolchain notice contains the complete license texts and copyright inventory.
  const rustNotices = rustHtml.replace(/<[^>]*>/g, '').replace(/&#(\d+);/g, (_, code) => String.fromCodePoint(Number(code)))
    .replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&quot;', '"').replaceAll('&amp;', '&')
    .replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').trim();
  rustSection = `${rustHeading}Source: https://github.com/rust-lang/rust/tree/${rustCommit}/library\nToolchain-provided COPYRIGHT-library.html rendered as text. The inventory includes platform-specific components and is conservative for wasm32.\n\n${rustNotices}\n`;
} else {
  // Minimal CI toolchains omit rust-docs. Reuse only the already-reviewed notice
  // for the exact compiler release AND commit, never another toolchain's text.
  const previous = await readFile(join(root, 'public/licenses.txt'), 'utf8').catch(() => '');
  const index = previous.indexOf(rustHeading);
  if (index < 0) throw new Error('Rust standard-library notices missing for this compiler. Install rust-docs and regenerate.');
  rustSection = previous.slice(index);
}

const mediabunnyPackage = npmPackages.find(pkg => pkg.name === "mediabunny");
if (!mediabunnyPackage) throw new Error("Mediabunny license inventory is missing.");
const intro = `Drum VRM — third-party notices\n\nApplication source: MIT; https://github.com/postmelee/drum-vrm\n\nGenerated from package-lock.json, installed npm package license files, Cargo.lock, and locked offline cargo metadata.\nThe npm inventory is the production dependency closure (declaration-only dependencies are labeled).\nThe Rust inventory conservatively includes build/proc-macro dependencies; inclusion does not mean every crate is linked into the final WASM.\nBuild/test-only npm dependencies are separately pinned in package-lock.json and are not shipped as application runtime code.\n\nMediabunny ${mediabunnyPackage.version} is used without modifying its covered source files. Its MPL-2.0 license remains in force.\nExact corresponding source, including src/ and the original LICENSE, is available without charge from:\n${mediabunnyPackage.source}\nThe package integrity below identifies the published source archive used for this build.\nThis application's separate source files are MIT licensed; no relicensing of third-party components is implied.\n\nExternal drum models and embedded textures: Innerscene Five-piece drum kit (CC0-1.0), Full Drum Kit by Batoski (CC-BY-3.0), and Drum Kit by smoj (CC-BY-3.0); see asset notices below. Character sample models retain their VRM Public License settings and any additional terms; see character notices below. No external drum samples, soundfonts, motion datasets, or ML weights are included. System fonts are selected from the device; no font files are redistributed.\n`;
const group = (title, packages) => `\n${'='.repeat(78)}\n${title}\n${'='.repeat(78)}\n` + packages.map(pkg =>
  `\n${pkg.name} ${pkg.version}\nLicense expression: ${pkg.license}\nScope: ${pkg.scope}\nSource archive: ${pkg.source}\nLock checksum: ${pkg.checksum}\nRepository: ${pkg.repository || '(not declared)'}\n${pkg.licenseSource ? `License text source: ${pkg.licenseSource}\n` : ''}` +
  pkg.licenses.map(license => `\n--- ${license.name} ---\n${license.text}\n`).join(''),
).join('\n');
const modelDirectory = join(root, 'public/models');
const assets = [];
for (const filename of (await readdir(modelDirectory)).filter(name => name.endsWith('-provenance.json')).sort()) {
  const asset = await json(join(modelDirectory, filename));
  if (!asset.licenseFile || !asset.files.some(file => file.file === asset.licenseFile)) throw new Error(`Asset license evidence missing: ${filename}`);
  for (const file of asset.files) {
    if (file.file.includes('/') || file.file.includes('\\')) throw new Error(`Invalid asset file name: ${filename}`);
    const bytes = await readFile(join(modelDirectory, file.file));
    if (bytes.length !== file.bytes || createHash('sha256').update(bytes).digest('hex') !== file.sha256) throw new Error(`Asset integrity mismatch: ${file.file}`);
  }
  assets.push({ ...asset, provenanceFile: filename });
}
if (assets.length !== 3) throw new Error('Expected three reviewed bundled model provenance records. Update the inventory deliberately when adding/removing an asset.');
const assetSections = assets.map(asset => `
EXTERNAL 3D ASSET
${asset.title} — ${asset.creator}
License: ${asset.license}
License URL: ${asset.licenseUrl}
Source: ${asset.source}
Original download: ${asset.download}
License evidence: ${asset.licenseEvidence}
Retrieved: ${asset.retrieved}
Binary SHA-256: ${asset.files[0].sha256}
Bundled binary unmodified: ${asset.unmodifiedBinary}
Application modifications/adaptation: ${asset.runtimeAdaptation}
Full license text: models/${asset.licenseFile}
Exact provenance: models/${asset.provenanceFile}
`);
const modelNotice = `Drum VRM — bundled model notices

Generated by npm run licenses from models/*-provenance.json.
Each model retains its own license. Full legal texts accompany the models and are included below in the distribution's licenses.txt.
${assetSections.join('\n')}`;
const assetLicenseTexts = [];
for (const filename of [...new Set(assets.map(asset => asset.licenseFile))].sort()) assetLicenseTexts.push(`\n--- models/${filename} ---\n${await readFile(join(modelDirectory, filename), 'utf8')}\n`);
const characterDirectory = join(root, 'public/characters');
const characters = [];
const characterLegalFiles = new Set();
for (const filename of (await readdir(characterDirectory)).filter(name => name.endsWith('-provenance.json')).sort()) {
  const asset = await json(join(characterDirectory, filename));
  if (!asset.id || !asset.sourceCommit || !asset.ownerPermission?.scope || !asset.credit || !asset.licenseFile
    || !asset.files?.some(file => file.file === asset.file) || !asset.files.some(file => file.file === asset.licenseFile)
    || !asset.unmodifiedBinary) throw new Error(`Character distribution evidence missing: ${filename}`);
  for (const file of asset.files) {
    if (file.file.includes('/') || file.file.includes('\\') || file.file.startsWith('.')) throw new Error(`Invalid character file: ${filename}`);
    const bytes = await readFile(join(characterDirectory, file.file));
    if (bytes.length !== file.bytes || createHash('sha256').update(bytes).digest('hex') !== file.sha256) throw new Error(`Character integrity mismatch: ${file.file}`);
    if (file.file !== asset.file) characterLegalFiles.add(file.file);
  }
  const bytes = await readFile(join(characterDirectory, asset.file));
  if (!asset.file.endsWith('.vrm') || bytes.length !== asset.bytes || createHash('sha256').update(bytes).digest('hex') !== asset.sha256) throw new Error(`Character binary mismatch: ${filename}`);
  if (bytes.readUInt32LE(0) !== 0x46546c67 || bytes.readUInt32LE(16) !== 0x4e4f534a) throw new Error(`Character is not GLB: ${filename}`);
  const embedded = JSON.parse(bytes.subarray(20, 20 + bytes.readUInt32LE(12)).toString('utf8')).extensions?.VRMC_vrm;
  if (embedded?.specVersion !== '1.0' || embedded.meta.allowRedistribution !== true
    || !isDeepStrictEqual(Object.fromEntries(Object.entries(embedded.meta).filter(([key]) => key !== 'thumbnailImage')), asset.embeddedMetadata)) throw new Error(`Character rights metadata mismatch: ${filename}`);
  if (embedded.meta.creditNotation === 'required' && !asset.creditRequired) throw new Error(`Character credit requirement missing: ${filename}`);
  if (embedded.meta.otherLicenseUrl && !asset.additionalTermsUrl) throw new Error(`Character additional terms missing: ${filename}`);
  characters.push({...asset, provenanceFile: filename});
}
const declaredCharacters = characters.map(asset => asset.file).sort();
const shippedCharacters = (await readdir(characterDirectory)).filter(name => name.endsWith('.vrm')).sort();
if (JSON.stringify(declaredCharacters) !== JSON.stringify(shippedCharacters)) throw new Error('Character binaries must each have one reviewed provenance record.');
const characterNotice = `Drum VRM — bundled character notices

Generated by npm run licenses from characters/*-provenance.json.
These unmodified sample avatars are not covered by the application's MIT license.
Model settings and additional terms remain binding. Rendered-video credit and model redistribution are separate obligations.
${characters.map(asset => `
CHARACTER SAMPLE
${asset.title} — ${asset.creator}
Credit: ${asset.credit}
Credit required: ${asset.creditRequired}
License: ${asset.license}
License URL: ${asset.licenseUrl}
Additional terms: ${asset.additionalTermsUrl || '(none declared)'}
Source: ${asset.source}
Pinned source commit: ${asset.sourceCommit}
File: characters/${asset.file}
Bytes: ${asset.bytes}
SHA-256: ${asset.sha256}
Bundled binary unmodified: ${asset.unmodifiedBinary}
Application adaptation: ${asset.runtimeAdaptation}
Conditions: ${asset.termsSummary}
Full license: characters/${asset.licenseFile}
Exact provenance and embedded rights: characters/${asset.provenanceFile}
`).join('\n')}`;
const characterLegalTexts = [];
for (const filename of [...characterLegalFiles].sort()) characterLegalTexts.push(`\n--- characters/${filename} ---\n${await readFile(join(characterDirectory, filename), 'utf8')}\n`);
const demoDirectory = join(root, 'public/demos');
const demoAssets = [];
for (const filename of (await readdir(demoDirectory)).filter(name => name.endsWith('-provenance.json')).sort()) {
  const asset = await json(join(demoDirectory, filename));
  if (!asset.permissionBasis || !asset.distributionScope || !asset.licenseStatement || !asset.permissionConfirmedOn) throw new Error(`Demo distribution evidence missing: ${filename}`);
  if (typeof asset.file !== 'string' || asset.file.includes('/') || asset.file.includes('\\') || !asset.file.endsWith('.mid')) throw new Error(`Invalid demo file name: ${filename}`);
  const bytes = await readFile(join(demoDirectory, asset.file));
  if (bytes.length !== asset.bytes || createHash('sha256').update(bytes).digest('hex') !== asset.sha256) throw new Error(`Demo integrity mismatch: ${asset.file}`);
  demoAssets.push({ ...asset, provenanceFile: filename });
}
const demoNotice = `Drum VRM — bundled MIDI demo notices

Generated by npm run licenses from demos/*-provenance.json.
Application source-code licensing does not relicense the following MIDI files.
${demoAssets.map(asset => `
MIDI DEMO
${asset.title}
Provided by: ${asset.providedBy}
Source: ${asset.source}
Permission confirmed: ${asset.permissionConfirmedOn}
Permission basis: ${asset.permissionBasis}
Distribution scope: ${asset.distributionScope}
License statement: ${asset.licenseStatement}
File: demos/${asset.file}
Bytes: ${asset.bytes}
SHA-256: ${asset.sha256}
Bundled binary unmodified: ${asset.unmodifiedBinary}
Arrangement: ${asset.arrangementNote}
Exact provenance and interpretation settings: demos/${asset.provenanceFile}
`).join('\n')}`;
const notices = intro + modelNotice + assetLicenseTexts.join('\n') + demoNotice + characterNotice + characterLegalTexts.join('\n') + group('NPM PRODUCTION DEPENDENCY CLOSURE', npmPackages) + group('RUST LOCKED DEPENDENCY CLOSURE', cargoPackages) + rustSection;

const table = packages => packages.map(pkg => `| ${pkg.name} | ${pkg.version} | ${pkg.license} | [archive](${pkg.source}) | \`${pkg.checksum}\` |`).join('\n');
const dependencyDoc = `# 실제 도입 의존성 잠금과 라이선스\n\n이 파일과 배포 고지는 \`npm run licenses\`로 생성한다. 네트워크 조회에 의존하지 않고 설치된 파일·lockfile·\`cargo metadata --locked --offline\`을 대조한다. 버전이나 체크섬이 맞지 않거나 원본 라이선스 파일이 없으면 실패한다. 절대 로컬 경로는 배포 문서에 포함하지 않는다.\n\n## 브라우저 패키지\n\nproduction 의존성의 전이 폐쇄를 기록한다. \`@types/\` 두 패키지는 타입 선언이며 브라우저 실행 코드가 아니다. 개발·검사 도구(Vite, TypeScript, Vitest, @types/three 등)는 \`package-lock.json\`의 별도 dev 의존성으로 고정하며 아래 런타임 목록과 구분한다.\n\n| 패키지 | 버전 | 라이선스 | 정확한 소스 | lock integrity |\n|---|---|---|---|---|\n${table(npmPackages)}\n\n## Rust 패키지\n\n\`wasm32-unknown-unknown\` 기준 Cargo 해석 그래프의 registry 패키지다. proc-macro/build 구성요소도 보수적으로 포함하며 모두가 WASM에 링크된다는 뜻은 아니다. crate 원본 archive와 Cargo.lock SHA-256을 제공한다. Rust 표준 라이브러리는 고정된 \`rust-toolchain.toml\`의 툴체인 구성요소로 사용한다.\n\n| crate | 버전 | 라이선스 | 정확한 소스 | Cargo.lock checksum |\n|---|---|---|---|---|\n${table(cargoPackages)}\n\n## 배포 조건과 범위\n\n- 원본 라이선스/저작권 문구는 [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md)와 배포물 \`licenses.txt\`에 그대로 포함한다. dual-license crate는 원본에 있는 양쪽 라이선스 모두 보존한다.\n- Mediabunny의 MPL-2.0 covered source는 수정하지 않았다. 정확한 npm archive에는 \`src/\`와 원본 LICENSE가 포함되며, 사용자가 별도 비용 없이 해당 버전의 대응 소스를 받을 수 있다. 자체 앱 파일은 MIT다.\n- Innerscene CC0 모델과 Batoski·smoj CC BY 3.0 모델을 도입했다. public/models/*-provenance.json의 원본 파일·라이선스 크기와 SHA-256을 검증하고 라이선스 원문을 고지에 합친다. 제목·저작자·원본 URL·라이선스·앱의 수정 사항을 함께 표기한다. 원본 GLB는 수정하지 않고 런타임에서 부품 분리·좌표 보정·역할별 병합을 수행한다. VRM 캐릭터 샘플은 public/characters/*-provenance.json의 원본 해시·내장 이용 조건·추가 약관을 검증하고 저작자와 고지를 보존한다. 앱의 MIT와 별개이며 Seed-san의 저작자 표기 의무와 AvatarSample_B의 별도 샘플 조건을 유지한다. 외부 음원 샘플·모션·데이터·ML 가중치는 도입하지 않았다. 글꼴 파일은 배포하지 않고 운영체제의 시스템 글꼴을 사용한다. 참고 조사 링크는 실행 코드 도입과 별개다.\n- 이 고지는 실제 설치 패키지와 잠금 체크섬을 근거로 한다. 확인하지 않은 Git tag/commit은 기재하지 않는다. 버전 변경 시 설치·잠금·고지를 함께 갱신한다.\n`;
await mkdir(join(root, 'public'), { recursive: true });
await writeFile(join(modelDirectory, 'NOTICE.txt'), modelNotice);
await writeFile(join(demoDirectory, 'NOTICE.txt'), demoNotice);
await writeFile(join(characterDirectory, 'NOTICE.txt'), characterNotice);
await writeFile(join(root, 'public/licenses.txt'), notices);
await writeFile(join(root, 'THIRD_PARTY_NOTICES.md'), `# Third-party notices\n\nThe same notices are distributed as [licenses.txt](public/licenses.txt). Generated by \`npm run licenses\`.\n\n\`\`\`text\n${notices}\`\`\`\n`);
const evidenceNotes = `\n## 원본 고지 예외와 툴체인\n\nmidly 0.5.3 crate의 include 목록은 LICENSE를 누락한다. crate 내부 \`.cargo_vcs_info.json\`의 commit \`${midlyCommit}\`를 확인한 뒤 [그 commit의 LICENSE](${midlyLicenseUrl}) 원문을 생성기에 보관했다. 버전·commit·license 표현식이 모두 같을 때만 이 예외를 적용한다.\n\nRust ${rustRelease}의 표준 라이브러리 고지도 설치 툴체인의 \`COPYRIGHT-library.html\`에서 추출하여 함께 배포한다. 소스 commit은 \`rustc --version --verbose\`에서 확인한 \`${rustCommit}\`다. rust-docs가 없는 최소 CI 환경에서는 이 릴리스와 commit이 정확히 같은 기존 배포 고지만 재사용한다. 새로운 툴체인은 rust-docs를 설치해 다시 생성해야 한다.\n`;
await writeFile(join(root, 'docs/research/dependency-lock.md'), dependencyDoc + evidenceNotes);
console.log(`License notices generated for ${npmPackages.length} npm and ${cargoPackages.length} Rust packages, plus ${assets.length} drum assets, ${characters.length} character samples and ${demoAssets.length} MIDI demos.`);
