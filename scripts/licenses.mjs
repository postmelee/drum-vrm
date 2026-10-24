import { readFile, readdir, writeFile, mkdir, access } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
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
const intro = `Drum VRM — third-party notices\n\nApplication source: MIT; https://github.com/postmelee/drum-vrm\n\nGenerated from package-lock.json, installed npm package license files, Cargo.lock, and locked offline cargo metadata.\nThe npm inventory is the production dependency closure (declaration-only dependencies are labeled).\nThe Rust inventory conservatively includes build/proc-macro dependencies; inclusion does not mean every crate is linked into the final WASM.\nBuild/test-only npm dependencies are separately pinned in package-lock.json and are not shipped as application runtime code.\n\nMediabunny ${mediabunnyPackage.version} is used without modifying its covered source files. Its MPL-2.0 license remains in force.\nExact corresponding source, including src/ and the original LICENSE, is available without charge from:\n${mediabunnyPackage.source}\nThe package integrity below identifies the published source archive used for this build.\nThis application's separate source files are MIT licensed; no relicensing of third-party components is implied.\n\nExternal drum models and embedded textures retain their individual terms; see the generated model notices below. No external drum samples, soundfonts, character models, motion datasets, or ML weights are included. System fonts are selected from the device; no font files are redistributed.\n`;
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
  for (const field of ['title', 'creator', 'license', 'source', 'download', 'licenseUrl', 'licenseEvidence', 'retrieved', 'runtimeAdaptation']) {
    if (typeof asset[field] !== 'string' || !asset[field].trim()) throw new Error(`Missing asset attribution field ${field}: ${filename}`);
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
const notices = intro + modelNotice + assetLicenseTexts.join('\n') + group('NPM PRODUCTION DEPENDENCY CLOSURE', npmPackages) + group('RUST LOCKED DEPENDENCY CLOSURE', cargoPackages) + rustSection;

// Framework operating documents belong to the repository, not to the browser
// distribution. Their pinned source is recorded by framework adoption.
let frameworkNotice = '';
const frameworkStatePath = join(root, '.hyper-waterfall/version.json');
if (await exists(frameworkStatePath)) {
  const framework = await json(frameworkStatePath);
  if (!/^[0-9a-f]{40}$/.test(framework.sourceCommit ?? '') || framework.license?.expression !== 'MIT') {
    throw new Error('Review the adopted framework source commit and license before generating repository notices.');
  }
  const noticePath = resolve(root, framework.license.noticePreservedAt ?? '');
  const noticeRelative = relative(root, noticePath).replaceAll('\\', '/');
  if (!noticeRelative || noticeRelative.startsWith('../')) throw new Error('Framework license notice must be preserved inside this repository.');
  const licenseText = await readFile(noticePath);
  if (createHash('sha256').update(licenseText).digest('hex') !== framework.license.sha256) {
    throw new Error('Adopted framework license notice no longer matches its recorded SHA-256.');
  }
  frameworkNotice = `
## Repository tooling

Hyper-Waterfall ${framework.frameworkVersion} operating documents and templates are adopted from [${framework.releaseTag}](${framework.releaseUrl}), pinned to [commit ${framework.sourceCommit}](${framework.sourceRepository}/tree/${framework.sourceCommit}). Repository placeholders and the designated project-specific rules were adapted for Drum VRM. The adopted file inventory is recorded in [.hyper-waterfall/version.json](.hyper-waterfall/version.json).

License: ${framework.license.expression}. The [original license](${framework.license.source}) copyright and permission notice is preserved in [${noticeRelative}](${noticeRelative}); its SHA-256 is \`${framework.license.sha256}\`.

These operating files are repository tooling and are not shipped in the browser application. This section is therefore included only in the repository notice; the application's runtime and asset notices remain in [public/licenses.txt](public/licenses.txt).
`;
}

const table = packages => packages.map(pkg => `| ${pkg.name} | ${pkg.version} | ${pkg.license} | [archive](${pkg.source}) | \`${pkg.checksum}\` |`).join('\n');
const dependencyDoc = `# Locked dependencies

[English guide](README.md) · [한국어 안내](README.ko.md)

Generated by \`npm run licenses\` from the installed packages and locked dependency graph. The table is shared by both language guides; package names, versions, SPDX expressions, URLs, and checksums have one source.

## npm production dependency closure

Packages under \`@types/\` provide declarations, not browser runtime code. Build/test-only npm dependencies remain pinned separately in \`package-lock.json\`.

| Package | Version | License | Source archive | Lock integrity |
|---|---|---|---|---|
${table(npmPackages)}

## Rust locked dependency closure

Registry packages resolved for \`wasm32-unknown-unknown\`, conservatively including build and procedural-macro dependencies. This is not a claim that every listed crate is linked into the final WASM.

| Crate | Version | License | Source archive | Cargo.lock checksum |
|---|---|---|---|---|
${table(cargoPackages)}

## Source evidence exceptions

| Component | Recorded source | Handling |
|---|---|---|
| midly 0.5.3 | [License at commit ${midlyCommit}](${midlyLicenseUrl}) | Published crate omits LICENSE; the generator verifies the crate's recorded source commit and license expression before using its preserved original text. |
| Rust ${rustRelease} standard library | [Source commit ${rustCommit}](https://github.com/rust-lang/rust/tree/${rustCommit}/library) | Original toolchain copyright inventory; minimal installations may reuse only an existing notice matching both release and commit. |

Full original license texts and attribution are in [repository notices](../../../THIRD_PARTY_NOTICES.md) and [distribution notices](../../../public/licenses.txt). Bundled model metadata is maintained in [model provenance records](../../../public/models/README.md).
`;

await mkdir(join(root, 'public'), { recursive: true });
await mkdir(join(root, 'docs/reference/dependencies'), { recursive: true });
await writeFile(join(modelDirectory, 'NOTICE.txt'), modelNotice);
await writeFile(join(root, 'public/licenses.txt'), notices);
await writeFile(join(root, 'THIRD_PARTY_NOTICES.md'), `# Third-party notices\n\nGenerated by \`npm run licenses\`. Full runtime and asset notices are also distributed as [licenses.txt](public/licenses.txt).\n${frameworkNotice}\n## Application distribution\n\n\`\`\`text\n${notices}\`\`\`\n`);
await writeFile(join(root, 'docs/reference/dependencies/locked.md'), dependencyDoc);
console.log(`License notices generated for ${npmPackages.length} npm and ${cargoPackages.length} Rust packages, plus ${assets.length} bundled 3D assets.`);
