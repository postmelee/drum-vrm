// Validate project-owned documentation without crawling private files or dependencies.
import { readFileSync, readdirSync, existsSync, statSync, realpathSync, readlinkSync } from 'node:fs';
import { resolve, relative as platformRelative, dirname, extname } from 'node:path';
import { createHash } from 'node:crypto';
const relative = (...args) => platformRelative(...args).replaceAll('\\', '/');
const root = process.cwd();
const errors = [];
const files = [];
for (const path of ['README.md','README.ko.md','docs/README.md','docs/README.ko.md','specs/README.md','mydocs/README.md']) if (!existsSync(path)) errors.push(`Missing required entry: ${path}`);
function walk(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = resolve(dir, entry.name);
    if (entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) walk(path);
    else if (entry.name.endsWith('.md')) files.push(path);
  }
}
for (const dir of ['docs', 'specs', 'mydocs', '.github', 'public/models']) walk(resolve(root, dir));
for (const name of ['README.md', 'README.ko.md', 'AGENTS.md', 'CLAUDE.md']) if (existsSync(name)) files.push(resolve(name));
const state = JSON.parse(readFileSync('.hyper-waterfall/version.json', 'utf8'));
const managed = new Set((state.managedFiles ?? []).map(item => item.target ?? item.path));
const generated = new Set(['docs/reference/dependencies/locked.md']);
const text = new Map(files.map(path => [path, readFileSync(path, 'utf8')]));
function outside(path) { const rel = relative(root, path); return rel.startsWith('..') || rel.startsWith('/'); }
function stripped(value, filename) {
  let fence = null;
  const lines = [];
  for (const line of value.split('\n')) {
    const match = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    if (fence) {
      if (match && match[1][0] === fence[0] && match[1].length >= fence.length && !match[2].trim()) fence = null;
    } else if (match) fence = match[1];
    else lines.push(line);
  }
  if (fence && filename) errors.push(`${filename}: unclosed or mismatched fence`);
  return lines.join('\n');
}
function linkTargets(value) {
  const content = value.replace(/(`+)[^\n]*?\1/g, '').replace(/<!--[\s\S]*?-->/g, '');
  const urls = [...content.matchAll(/\[[^\]\n]*\]\((<[^>]+>|[^\s)]+)(?:\s+["'][^\n]*?["'])?\)/g)].map(m=>m[1]);
  const definitions = new Map();
  for (const m of content.matchAll(/^ {0,3}\[([^\]\n]+)\]:\s*(<[^>]+>|\S+)/gm)) {
    definitions.set(m[1].toLowerCase(), m[2]); urls.push(m[2]);
  }
  for (const m of content.matchAll(/\[([^\]\n]+)\]\[([^\]\n]*)\]/g)) {
    if (!definitions.has((m[2] || m[1]).toLowerCase())) urls.push(`UNDEFINED_REFERENCE:${m[2] || m[1]}`);
  }
  return urls.map(url=>url.replace(/^<|>$/g, ''));
}
function anchors(value) {
  const seen = new Map();
  const ids = new Set();
  for (const match of stripped(value).matchAll(/^ {0,3}#{1,6} +(.+?)(?: +#+)?$/gm)) {
    const base = match[1].replace(/<[^>]+>/g, '').replace(/[\[\]`*_]/g, '').toLowerCase().replace(/[^\p{L}\p{N}\p{M}_\- ]/gu, '').replace(/ /g, '-');
    const n = seen.get(base) ?? 0;
    seen.set(base, n + 1); ids.add(n ? `${base}-${n}` : base);
  }
  for (const match of value.matchAll(/<(?:a|h[1-6])[^>]+(?:id|name)=["']([^"']+)["']/g)) ids.add(match[1]);
  return ids;
}
let links = 0, pairs = 0, diagrams = 0, checked = 0;
const graph = new Map();
const sha = value => createHash('sha256').update(value).digest('hex');
for (const record of state.managedFiles) {
  const path = resolve(root, record.target);
  if (!existsSync(path) || sha(readFileSync(path)) !== record.installedSha256) errors.push(`Framework drift: ${record.target}`);
}
for (const path of ['.agents/skills','.claude/skills']) {
  try { if (readlinkSync(path) !== '../mydocs/skills' || realpathSync(path) !== resolve(root,'mydocs/skills')) errors.push(`Unexpected skill symlink: ${path}`); } catch { errors.push(`Missing skill symlink: ${path}`); }
}
for (const [path, body] of text) {
  const rel = relative(root, path);
  if (managed.has(rel)) continue; // Upstream templates contain intentional example paths.
  checked++;
  if (!generated.has(rel) && body.split('\n').length > 499) errors.push(`${rel}: exceeds 499 lines`);

  diagrams += [...body.matchAll(/^(?:```|~~~)mermaid\s*$/gm)].length;
  const content = stripped(body, rel);
  graph.set(path, []);
  for (const url of linkTargets(content)) {
    if (url.startsWith('UNDEFINED_REFERENCE:')) { errors.push(`${rel}: ${url}`); continue; }
    if (/^(?:[a-z][a-z0-9+.-]*:|\/\/)/i.test(url)) continue;
    links++;
    const [rawPath, fragment] = url.split('#');
    let target;
    try { target = resolve(dirname(path), decodeURIComponent(rawPath.split('?')[0] || '.')); }
    catch { errors.push(`${rel}: invalid URL ${url}`); continue; }
    if (!rawPath) target = path;
    if (outside(target) || !existsSync(target)) { errors.push(`${rel}: missing target ${url}`); continue; }
    if (outside(realpathSync(target))) { errors.push(`${rel}: target escapes repository ${url}`); continue; }
    graph.get(path).push(target);
    if (fragment && statSync(target).isFile() && extname(target) === '.md') {
      const targetBody = text.get(target) ?? readFileSync(target, 'utf8');
      if (!anchors(targetBody).has(decodeURIComponent(fragment))) errors.push(`${rel}: missing anchor ${url}`);
    }
  }
  const reader = (rel.startsWith('docs/') || rel === 'README.md' || rel === 'README.ko.md' || rel.startsWith('public/models/README'));
  const redirect = body.includes('<!-- migration-redirect -->');
  if (reader && !generated.has(rel) && !rel.endsWith('AGENTS.md') && !redirect) {
    const other = path.endsWith('.ko.md') ? path.replace(/\.ko\.md$/, '.md') : path.replace(/\.md$/, '.ko.md');
    if (!existsSync(other)) errors.push(`${rel}: missing language counterpart ${relative(root, other)}`);
    else {
      const link = relative(dirname(path), other);
      if (!body.includes(`](${link})`)) errors.push(`${rel}: missing reciprocal language link ${link}`);
      if (!path.endsWith('.ko.md')) pairs++;
    }
  }
}
for (const area of ['docs', 'specs']) {
  const dirs = new Set(files.filter(p => relative(root, p).startsWith(`${area}/`)).map(dirname));
  for (const dir of dirs) if (!existsSync(resolve(dir, 'README.md'))) errors.push(`${relative(root, dir)}: missing directory README`);
}
const ledgerPath = 'mydocs/report/migrations/task_m3_36_inventory.json';
let preserved = 0;
if (!existsSync(ledgerPath)) errors.push('Missing migration inventory');
else {
  const ledger = JSON.parse(readFileSync(ledgerPath,'utf8'));
  if (ledger.records.length !== 48 || new Set(ledger.records.map(r=>r.source)).size !== 48) errors.push('Migration inventory must cover 48 unique baseline paths');
  for (const record of ledger.records) {
    for (const destination of record.destinations) if (!existsSync(destination)) errors.push(`Missing migrated destination: ${destination}`);
    if (record.preserved_copy) {
      const body = readFileSync(record.preserved_copy,'utf8').split('\n\n').slice(1).join('\n\n');
      const urls = [...new Set(body.match(/https?:\/\/[^\s)>]+/g) ?? [])].sort();
      const missingUrls = (record.preserved_external_urls ?? []).filter(url => !urls.includes(url));
      if (missingUrls.length) errors.push(`Historical source URLs removed: ${record.preserved_copy}: ${missingUrls.join(', ')}`);
      const normalized = body.replace(/(\[[^\]\n]*\]\()([^)]*)(\))/g,'$1TARGET$3').trim();
      if (sha(normalized) !== record.preserved_body_sha256) errors.push(`Historical body changed: ${record.preserved_copy}`);
      else preserved++;
    }
  }
}
const reached = new Set();
function visit(path) { if (reached.has(path)) return; reached.add(path); for (const next of graph.get(path) ?? []) visit(next); }
for (const path of ['README.md','README.ko.md','docs/README.md','specs/README.md','mydocs/README.md']) visit(resolve(root,path));
const orphans = [];
for (const [path, body] of text) {
  const rel = relative(root,path);
  if (!managed.has(rel) && /^(docs|specs|mydocs)\//.test(rel) && !rel.endsWith('AGENTS.md') && !body.includes('<!-- migration-redirect -->') && !reached.has(path)) orphans.push(rel);
}
// Work-record indexes supplied by the framework contain generic instructions;
// issue-specific records must be linked by their current task plan/report.
for (const rel of orphans) errors.push(`Unlinked project document: ${rel}`);
const summary = { preservedHistoricalBodies: preserved, files: files.length, projectFilesChecked: checked, relativeLinks: links, bilingualPairs: pairs, mermaidBlocks: diagrams, errors };
console.log(JSON.stringify(summary, null, 2));
if (errors.length) process.exitCode = 1;
