// Check this repository's Markdown links and published language pairs.
// Fenced/inline code is ignored. This is not a complete Markdown renderer.
import { existsSync, readFileSync, readdirSync, realpathSync, statSync } from 'node:fs';
import { dirname, extname, relative, resolve } from 'node:path';

const root = process.cwd();
const errors = [];
const files = [];
const name = path => relative(root, path).replaceAll('\\', '/');
const outside = path => { const rel = relative(root, path); return rel === '..' || rel.startsWith('../') || rel.startsWith('..\\'); };

function walk(directory) {
  if (!existsSync(directory)) return;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) walk(path);
    else if (entry.isFile() && entry.name.endsWith('.md')) files.push(path);
  }
}
for (const directory of ['docs', 'specs']) walk(resolve(directory));
for (const path of ['README.md', 'README.ko.md', 'AGENTS.md', 'public/models/README.md', 'public/models/README.ko.md']) if (existsSync(path)) files.push(resolve(path));
const texts = new Map(files.map(path => [path, readFileSync(path, 'utf8')]));
const generated = new Set(['docs/research/dependency-lock.md']);

function prose(body, path) {
  let fence = null;
  const lines = [];
  for (const line of body.split('\n')) {
    const match = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    if (fence) {
      if (match && match[1][0] === fence[0] && match[1].length >= fence.length && !match[2].trim()) fence = null;
    } else if (match) fence = match[1];
    else lines.push(line);
  }
  if (fence && path) errors.push(`${name(path)}: unclosed or mismatched code fence`);
  return lines.join('\n').replace(/<!--[\s\S]*?-->/g, '');
}

function links(body) {
  const content = body.replace(/(`+)[^\n]*?\1/g, '');
  const targets = [...content.matchAll(/\[[^\]\n]*\]\((<[^>]+>|[^\s)]+)(?:\s+["'][^\n]*?["'])?\)/g)].map(match => match[1]);
  const labels = new Set();
  for (const match of content.matchAll(/^ {0,3}\[([^\]\n]+)\]:\s*(<[^>]+>|\S+)/gm)) {
    labels.add(match[1].trim().toLowerCase());
    targets.push(match[2]);
  }
  const undefinedLabels = [...content.matchAll(/\[([^\]\n]+)\]\[([^\]\n]*)\]/g)]
    .map(match => (match[2] || match[1]).trim().toLowerCase()).filter(label => !labels.has(label));
  return { targets: targets.map(target => target.replace(/^<|>$/g, '')), undefinedLabels };
}

function anchors(body) {
  const ids = new Set();
  for (const match of prose(body).matchAll(/^ {0,3}#{1,6} +(.+?)(?: +#+)?$/gm)) {
    const heading = match[1].replace(/\[([^\]]+)\]\([^)]*\)/g, '$1').replace(/<[^>]+>/g, '')
      .replace(/[\[\]`*_]/g, '').toLowerCase().replace(/[^\p{L}\p{N}\p{M}_\- ]/gu, '').replace(/ /g, '-');
    let id = heading, suffix = 0;
    while (ids.has(id)) id = `${heading}-${++suffix}`;
    ids.add(id);
  }
  for (const match of body.matchAll(/<(?:a|h[1-6])[^>]+(?:id|name)=["']([^"']+)["']/g)) ids.add(match[1]);
  return ids;
}

let relativeLinks = 0;
const targetsByFile = new Map();
for (const [path, body] of texts) {
  if (!generated.has(name(path)) && body.trimEnd().split('\n').length >= 500) errors.push(`${name(path)}: authored Markdown must stay under 500 lines`);
  const { targets, undefinedLabels } = links(prose(body, path));
  targetsByFile.set(path, targets);
  for (const label of undefinedLabels) errors.push(`${name(path)}: undefined reference [${label}]`);
  for (const url of targets) {
    if (/^(?:[a-z][a-z0-9+.-]*:|\/\/)/i.test(url)) continue;
    relativeLinks++;
    const [rawPath, rawAnchor] = url.split('#');
    let target, anchor;
    try {
      target = rawPath ? resolve(dirname(path), decodeURIComponent(rawPath.split('?')[0])) : path;
      anchor = rawAnchor ? decodeURIComponent(rawAnchor) : null;
    } catch { errors.push(`${name(path)}: invalid URL ${url}`); continue; }
    if (outside(target) || !existsSync(target)) { errors.push(`${name(path)}: missing local target ${url}`); continue; }
    if (outside(realpathSync(target))) { errors.push(`${name(path)}: link escapes repository ${url}`); continue; }
    if (anchor && statSync(target).isFile() && extname(target) === '.md') {
      const body = texts.get(target) ?? readFileSync(target, 'utf8');
      if (!anchors(body).has(anchor)) errors.push(`${name(path)}: missing heading ${url}`);
    }
  }
}

// These are published guides. Historical research, ADRs and test evidence keep
// their original language; folder names do not impose a documentation taxonomy.
const paired = new Set(['README.md', 'public/models/README.md', ...['README', 'models', 'calibration', 'performance-foundations', 'export'].map(base => `docs/${base}.md`)]);
for (const path of files) {
  const rel = name(path);
  if (rel.startsWith('specs/') && !rel.endsWith('/AGENTS.md')) paired.add(rel.replace(/\.ko\.md$/, '.md'));
}
let bilingualPairs = 0;
for (const english of paired) {
  const korean = english.replace(/\.md$/, '.ko.md');
  if (english !== 'README.md' && !existsSync(english) && !existsSync(korean)) continue;
  if (!existsSync(english) || !existsSync(korean)) { errors.push(`Missing language counterpart: ${english} / ${korean}`); continue; }
  bilingualPairs++;
  for (const [from, to] of [[english, korean], [korean, english]]) {
    const link = relative(dirname(resolve(from)), resolve(to)).replaceAll('\\', '/');
    if (!(targetsByFile.get(resolve(from)) ?? []).includes(link)) errors.push(`${from}: missing reciprocal language link ${link}`);
  }
}

console.log(JSON.stringify({ files: files.length, relativeLinks, bilingualPairs, errors }, null, 2));
if (errors.length) process.exitCode = 1;
