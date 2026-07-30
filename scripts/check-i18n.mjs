import { readFileSync, readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');

function parseTsKeys(filePath) {
  const content = readFileSync(filePath, 'utf-8');
  const keys = [];
  const lines = content.split('\n');
  for (const line of lines) {
    const m = line.match(/^\s+(\w+):\s*['"]/);
    if (m) keys.push(m[1]);
  }
  return new Set(keys);
}

const enKeys = parseTsKeys(join(root, 'src', 'i18n', 'en.ts'));
const zhKeys = parseTsKeys(join(root, 'src', 'i18n', 'zh.ts'));

let missingEn = 0, missingZh = 0;
for (const k of enKeys) { if (!zhKeys.has(k)) { console.log(`Missing in zh: ${k}`); missingZh++; } }
for (const k of zhKeys) { if (!enKeys.has(k)) { console.log(`Missing in en: ${k}`); missingEn++; } }

const src = join(root, 'src');
const vueFiles = [];
function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name);
    if (entry.isDirectory() && entry.name !== 'node_modules' && !entry.name.startsWith('.')) walk(p);
    else if (entry.name.endsWith('.vue') || entry.name.endsWith('.ts')) vueFiles.push(p);
  }
}
walk(src);

const usedKeys = new Set();
for (const file of vueFiles) {
  const content = readFileSync(file, 'utf-8');
  const matches = content.matchAll(/['"]t\(['"]([^'"]+)['"]/g);
  for (const m of matches) usedKeys.add(m[1]);
}

let unused = 0;
for (const k of enKeys) {
  if (!usedKeys.has(k)) {
    console.log(`Possible unused key: ${k}`);
    unused++;
  }
}

console.log(`\nSummary: ${missingZh} missing zh, ${missingEn} missing en, ${unused} possibly unused`);
