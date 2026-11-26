'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');
const { readFileBounded } = require('./bounded-file');

const EXCLUDED_DIRS = new Set(['.git', '.godot', '.import', 'node_modules', 'bin', 'obj', 'build', 'dist', 'target', '.vscode']);
const TEXT_EXTENSIONS = new Set(['.gd', '.tscn', '.tres', '.gdshader', '.gdextension', '.cfg', '.json', '.cs']);
const MAX_ENTRIES = 8000;
const MAX_DEPTH = 12;
const MAX_FILES = 180;
const MAX_FILE_BYTES = 400_000;
const MAX_FINDINGS = 250;

function addFinding(findings, severity, code, file, line, message) {
  if (findings.length >= MAX_FINDINGS) return;
  findings.push({ severity, code, file: String(file || 'project.godot').slice(0, 300), line: Number.isInteger(line) && line > 0 ? line : null, message: String(message).slice(0, 500) });
}

function extractSetting(text, key) {
  const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const match = text.match(new RegExp(`^\\s*${escaped}\\s*=\\s*(.+?)\\s*$`, 'm'));
  return match ? match[1].trim() : null;
}

function cleanResourcePath(value) {
  if (typeof value !== 'string' || !value.startsWith('res://')) return null;
  const withoutSubresource = value.split('::')[0].slice('res://'.length).replace(/\\/g, '/');
  if (!withoutSubresource || withoutSubresource.startsWith('/') || withoutSubresource.split('/').some(part => !part || part === '.' || part === '..')) return null;
  return withoutSubresource;
}

async function inspectProjectResource(root, relativePath) {
  const rel = cleanResourcePath(`res://${String(relativePath || '').replace(/\\/g, '/')}`);
  if (!rel) return { status: 'unsafe' };
  const target = path.resolve(root, rel);
  if (target !== root && !target.startsWith(root + path.sep)) return { status: 'unsafe' };
  const parts = rel.split('/');
  let current = root;
  try {
    // Check every path component: lstat only on the final file would miss a
    // symlinked parent directory which points back inside the selected root.
    for (let index = 0; index < parts.length; index++) {
      current = path.join(current, parts[index]);
      const stat = await fs.lstat(current);
      const isLast = index === parts.length - 1;
      if (stat.isSymbolicLink()) return { status: 'unsafe', relative: rel };
      if (isLast ? !stat.isFile() : !stat.isDirectory()) return { status: 'unsafe', relative: rel };
      const real = await fs.realpath(current);
      if (real !== root && !real.startsWith(root + path.sep)) return { status: 'unsafe', relative: rel };
      if (isLast && stat.size > MAX_FILE_BYTES) return { status: 'too-large', relative: rel, size: stat.size };
      if (isLast) return { status: 'ok', absolute: real, relative: rel, size: stat.size };
    }
    return { status: 'unsafe', relative: rel };
  } catch (error) {
    return error?.code === 'ENOENT' || error?.code === 'ENOTDIR' ? { status: 'missing', relative: rel } : { status: 'unsafe', relative: rel };
  }
}


function parseVersion(projectText) {
  const features = extractSetting(projectText, 'config/features');
  const match = features?.match(/(?:^|["',\s])([34])\.(\d+)/);
  return match ? { major: Number(match[1]), version: `${match[1]}.${match[2]}` } : { major: null, version: 'Nezjištěno' };
}

async function auditGodotProject(rootPath) {
  if (typeof rootPath !== 'string' || !rootPath.trim()) throw new Error('Nejdřív vyber projektovou složku.');
  const root = await fs.realpath(rootPath);
  const rootStat = await fs.stat(root);
  if (!rootStat.isDirectory()) throw new Error('Vybraná cesta není složka.');

  const findings = [];
  const configPath = path.join(root, 'project.godot');
  let projectText = '';
  try {
    const st = await fs.lstat(configPath);
    if (!st.isFile() || st.isSymbolicLink() || st.size > MAX_FILE_BYTES) throw new Error('invalid');
    projectText = (await readFileBounded(fs, configPath, MAX_FILE_BYTES, 'project.godot')).toString('utf8');
  } catch {
    addFinding(findings, 'error', 'NO_PROJECT_FILE', 'project.godot', null, 'Soubor project.godot nebyl nalezen nebo není bezpečně čitelný. Vyber kořen Godot projektu.');
    return { isGodot: false, projectName: path.basename(root), engineVersion: 'Nezjištěno', mainScene: null, scannedScripts: 0, scannedScenes: 0, scannedResources: 0, visited: 0, limitReached: false, counts: { error: 1, warning: 0, info: 0 }, findings, note: 'Read-only statická kontrola. Projekt ani skripty se nespouštějí.' };
  }

  const nameValue = extractSetting(projectText, 'config/name');
  const projectName = nameValue?.match(/^"(.*)"$/)?.[1]?.replace(/\\"/g, '"').slice(0, 160) || path.basename(root);
  const versionInfo = parseVersion(projectText);
  const mainSceneValue = extractSetting(projectText, 'run/main_scene');
  const mainScene = mainSceneValue?.match(/^"(res:\/\/[^"\r\n]+)"$/)?.[1] || null;
  let scannedScripts = 0, scannedScenes = 0, scannedResources = 0, visited = 0, checkedFiles = 0;
  let resourceReferences = 0;
  const referencedPaths = new Map();
  const textFiles = [];

  if (!mainScene) {
    addFinding(findings, 'warning', 'MAIN_SCENE_NOT_SET', 'project.godot', null, 'Není nastavena run/main_scene. Projekt se nemusí spustit přímo přes hlavní scénu.');
  } else {
    const relMain = cleanResourcePath(mainScene);
    const mainResult = relMain ? await inspectProjectResource(root, relMain) : { status: 'unsafe' };
    if (mainResult.status === 'missing') addFinding(findings, 'error', 'MISSING_MAIN_SCENE', 'project.godot', null, `Nastavená hlavní scéna ${mainScene} nebyla nalezena.`);
    else if (mainResult.status === 'unsafe') addFinding(findings, 'error', 'UNSAFE_MAIN_SCENE', 'project.godot', null, `Cestu k hlavní scéně ${mainScene} nelze bezpečně ověřit.`);
    else if (mainResult.status === 'too-large') addFinding(findings, 'info', 'MAIN_SCENE_SIZE_LIMIT', 'project.godot', null, `Hlavní scéna existuje (${Math.round(mainResult.size / 1024)} kB), ale přesahuje limit čtení; její obsah byl vynechán.`);
  }

  async function walk(dir, depth = 0) {
    if (depth > MAX_DEPTH || visited >= MAX_ENTRIES || checkedFiles >= MAX_FILES) return;
    let entries;
    try { entries = await fs.readdir(dir, { withFileTypes: true }); } catch { return; }
    entries.sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
      if (visited >= MAX_ENTRIES || checkedFiles >= MAX_FILES) break;
      visited++;
      if (entry.name.startsWith('.') && !['.gitignore', '.gitattributes'].includes(entry.name)) continue;
      const absolute = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (!EXCLUDED_DIRS.has(entry.name)) await walk(absolute, depth + 1);
        continue;
      }
      if (!entry.isFile()) continue; // Symlinks and special files are not followed.
      const ext = path.extname(entry.name).toLowerCase();
      if (!TEXT_EXTENSIONS.has(ext)) continue;
      let st;
      try { st = await fs.stat(absolute); } catch { continue; }
      if (!st.isFile() || st.size > MAX_FILE_BYTES) {
        if (['.gd', '.tscn', '.tres'].includes(ext)) {
          addFinding(findings, 'info', 'FILE_SKIPPED_SIZE', path.relative(root, absolute), null, `Soubor přesahuje limit ${Math.round(MAX_FILE_BYTES / 1024)} kB a byl vynechán.`);
        }
        continue;
      }
      const relative = path.relative(root, absolute).split(path.sep).join('/');
      try {
        // Stat is only an early filter. A file may grow after stat, so all
        // actual reads use a bounded reader as well.
        const content = (await readFileBounded(fs, absolute, MAX_FILE_BYTES, relative)).toString('utf8');
        checkedFiles++;
        if (ext === '.gd') scannedScripts++;
        else if (ext === '.tscn') scannedScenes++;
        else if (ext === '.tres' || ext === '.gdshader' || ext === '.gdextension') scannedResources++;
        textFiles.push({ relative, ext, content });
      } catch (error) {
        if (/překročil limit/i.test(String(error?.message || ''))) {
          addFinding(findings, 'info', 'FILE_SKIPPED_SIZE', relative, null, `Soubor během čtení přesáhl limit ${Math.round(MAX_FILE_BYTES / 1024)} kB a byl vynechán.`);
        }
        // Files disappearing or becoming unreadable while scanning are not
        // treated as missing resource findings; the report remains read-only.
      }
    }
  }
  await walk(root);

  for (const file of textFiles) {
    if (file.ext === '.gd') {
      const lines = file.content.split(/\r?\n/);
      for (let index = 0; index < lines.length; index++) {
        const line = lines[index];
        if (/\b(TODO|FIXME|HACK)\b/i.test(line)) addFinding(findings, 'info', 'TODO_MARKER', file.relative, index + 1, line.trim().slice(0, 240));
      }
      if (!file.content.trim()) addFinding(findings, 'warning', 'EMPTY_SCRIPT', file.relative, 1, 'GDScript soubor je prázdný.');
      if (versionInfo.major === 4 && /^\s*(?:onready|tool)\s+(?:var|func)\b/m.test(file.content)) {
        addFinding(findings, 'warning', 'GODOT3_SYNTAX', file.relative, null, 'Skript může obsahovat syntaxi Godot 3 (např. onready/tool bez @). Ověř kompatibilitu s uvedenou verzí; jde o heuristiku.');
      }
      if (versionInfo.major === 3 && /^\s*@(?:onready|export|tool)\b/m.test(file.content)) {
        addFinding(findings, 'warning', 'GODOT4_SYNTAX', file.relative, null, 'Skript může obsahovat syntaxi Godot 4 (@onready/@export/@tool), která nemusí být kompatibilní s Godot 3.');
      }
    }

    if (['.tscn', '.tres', '.gdextension'].includes(file.ext)) {
      const regex = /(?:path|script|source|config_file)\s*=\s*"(res:\/\/[^"\r\n]+)"/g;
      for (const match of file.content.matchAll(regex)) {
        const resPath = cleanResourcePath(match[1]);
        if (!resPath) {
          addFinding(findings, 'warning', 'UNSAFE_RESOURCE_PATH', file.relative, null, `Odkaz na zdroj ${match[1]} nelze bezpečně vyhodnotit.`);
          continue;
        }
        resourceReferences++;
        if (!referencedPaths.has(resPath)) referencedPaths.set(resPath, file.relative);
      }
    }
  }

  for (const [ref, source] of referencedPaths) {
    const result = await inspectProjectResource(root, ref);
    if (result.status === 'missing') addFinding(findings, 'warning', 'MISSING_RESOURCE', source, null, `Odkaz na zdroj res://${ref} nebyl nalezen.`);
    else if (result.status === 'unsafe') addFinding(findings, 'warning', 'UNSAFE_RESOURCE', source, null, `Odkaz na zdroj res://${ref} míří na symbolický odkaz, speciální soubor nebo cestu, kterou nelze bezpečně ověřit.`);
    else if (result.status === 'too-large') addFinding(findings, 'info', 'RESOURCE_SIZE_LIMIT', source, null, `Zdroj res://${ref} existuje (${Math.round(result.size / 1024)} kB), ale jeho obsah přesahuje limit čtení; ověřena byla pouze existence souboru.`);
  }

  if (visited >= MAX_ENTRIES || checkedFiles >= MAX_FILES) {
    addFinding(findings, 'info', 'SCAN_LIMIT_REACHED', 'project.godot', null, `Audit dosáhl limitu ${MAX_ENTRIES} položek nebo ${MAX_FILES} textových souborů; výsledek může být neúplný.`);
  }
  const counts = {
    error: findings.filter(item => item.severity === 'error').length,
    warning: findings.filter(item => item.severity === 'warning').length,
    info: findings.filter(item => item.severity === 'info').length
  };
  return {
    isGodot: true,
    projectName,
    engineVersion: versionInfo.version,
    mainScene,
    scannedScripts,
    scannedScenes,
    scannedResources,
    referencedPaths: resourceReferences,
    visited,
    limitReached: visited >= MAX_ENTRIES || checkedFiles >= MAX_FILES,
    counts,
    findings: findings.slice(0, MAX_FINDINGS),
    note: 'Read-only statická heuristická kontrola. Godot se nespouští, GDScript se neparsuje skutečným enginem a výsledek nepotvrzuje, že projekt lze importovat nebo spustit.'
  };
}

module.exports = { auditGodotProject, cleanResourcePath, parseVersion, inspectProjectResource };
