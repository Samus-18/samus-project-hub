'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');
const { readFileBounded } = require('./bounded-file');

const MAX_ENTRIES = 12000;
const MAX_DEPTH = 14;
const MAX_TEXT_FILES = 500;
const MAX_FILE_BYTES = 500_000;
const MAX_FINDINGS = 300;
const EXCLUDED_DIRS = new Set(['.git', 'Library', 'Temp', 'Obj', 'Build', 'Builds', 'Logs', '.vs', '.idea', 'PackagesCache']);
const TEXT_EXTENSIONS = new Set(['.cs', '.asmdef', '.asmref', '.json', '.unity', '.prefab', '.asset', '.mat', '.controller', '.anim', '.shader', '.compute', '.uxml', '.uss', '.txt']);

function addFinding(findings, severity, code, file, line, message) {
  if (findings.length >= MAX_FINDINGS) return;
  findings.push({
    severity: ['error', 'warning', 'info'].includes(severity) ? severity : 'info',
    code: String(code || 'UNKNOWN').slice(0, 80),
    file: String(file || 'ProjectSettings').replace(/[\u0000-\u001f]/g, '').slice(0, 400),
    line: Number.isInteger(line) && line > 0 ? line : null,
    message: String(message || '').replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g, '').slice(0, 500)
  });
}

async function safeRegularFile(root, relative, maxBytes = MAX_FILE_BYTES) {
  const parts = String(relative).split(/[\\/]/).filter(Boolean);
  let current = root;
  try {
    for (let i = 0; i < parts.length; i++) {
      current = path.join(current, parts[i]);
      const stat = await fs.lstat(current);
      if (stat.isSymbolicLink()) return { status: 'unsafe' };
      if (i === parts.length - 1 ? !stat.isFile() : !stat.isDirectory()) return { status: 'unsafe' };
      const real = await fs.realpath(current);
      if (real !== root && !real.startsWith(root + path.sep)) return { status: 'unsafe' };
      if (i === parts.length - 1) return { status: stat.size > maxBytes ? 'too-large' : 'ok', absolute: real, size: stat.size };
    }
  } catch (error) {
    return error?.code === 'ENOENT' || error?.code === 'ENOTDIR' ? { status: 'missing' } : { status: 'unsafe' };
  }
  return { status: 'unsafe' };
}

async function getUnityProjectInfo(rootPath) {
  if (typeof rootPath !== 'string' || !rootPath.trim()) throw new Error('Nejdřív vyber Unity projekt.');
  const root = await fs.realpath(rootPath);
  const rootStat = await fs.stat(root);
  if (!rootStat.isDirectory()) throw new Error('Vybraná cesta není složka.');
  const [versionFile, manifestFile, assetsEntry] = await Promise.all([
    safeRegularFile(root, 'ProjectSettings/ProjectVersion.txt', 100_000),
    safeRegularFile(root, 'Packages/manifest.json', 250_000),
    (async () => { try { return await fs.lstat(path.join(root, 'Assets')); } catch { return null; } })()
  ]);
  const hasAssets = !!assetsEntry?.isDirectory() && !assetsEntry.isSymbolicLink();
  let version = 'Nezjištěno';
  if (versionFile.status === 'ok') {
    try {
      const text = (await readFileBounded(fs, versionFile.absolute, 100_000, 'Unity ProjectVersion.txt')).toString('utf8');
      version = (text.match(/m_EditorVersion:\s*([^\r\n]+)/) || [])[1]?.trim().slice(0, 80) || version;
    } catch {}
  }
  let packages = [];
  if (manifestFile.status === 'ok') {
    try {
      const data = JSON.parse((await readFileBounded(fs, manifestFile.absolute, 250_000, 'Unity manifest.json')).toString('utf8'));
      if (data && data.dependencies && typeof data.dependencies === 'object' && !Array.isArray(data.dependencies)) packages = Object.keys(data.dependencies).slice(0, 100);
    } catch {}
  }
  const logs = [];
  for (const rel of ['Logs/AssetImportWorker0.log', 'Editor.log', 'Library/LastSceneManagerSetup.txt']) {
    const item = await safeRegularFile(root, rel);
    if (item.status === 'ok') logs.push({ path: rel, size: item.size });
  }
  return {
    isUnity: versionFile.status === 'ok' && manifestFile.status === 'ok' && hasAssets,
    version,
    hasAssets,
    assetsPathUnsafe: !!assetsEntry?.isSymbolicLink(),
    hasProjectSettings: versionFile.status === 'ok',
    projectVersionStatus: versionFile.status,
    hasManifest: manifestFile.status === 'ok',
    manifestStatus: manifestFile.status,
    packages,
    logs,
    root
  };
}

async function auditUnityProject(rootPath) {
  if (typeof rootPath !== 'string' || !rootPath.trim()) throw new Error('Nejdřív vyber Unity projekt.');
  const root = await fs.realpath(rootPath);
  const rootStat = await fs.stat(root);
  if (!rootStat.isDirectory()) throw new Error('Vybraná cesta není složka.');
  const findings = [];
  let scannedScripts = 0, scannedAssets = 0, visited = 0, checkedFiles = 0, packageCount = 0;
  let limitReached = false;
  const projectInfo = await getUnityProjectInfo(root);

  if (projectInfo.projectVersionStatus === 'missing') addFinding(findings, 'warning', 'NO_PROJECT_VERSION', 'ProjectSettings/ProjectVersion.txt', null, 'Soubor verze Unity nebyl nalezen. Vyber kořen Unity projektu.');
  else if (projectInfo.projectVersionStatus === 'too-large') addFinding(findings, 'info', 'PROJECT_VERSION_SIZE_LIMIT', 'ProjectSettings/ProjectVersion.txt', null, 'Soubor verze Unity překračuje limit 100 kB a nebyl načten.');
  else if (projectInfo.projectVersionStatus !== 'ok') addFinding(findings, 'warning', 'UNSAFE_PROJECT_VERSION', 'ProjectSettings/ProjectVersion.txt', null, 'Soubor verze Unity není běžný bezpečně čitelný soubor.');
  if (projectInfo.manifestStatus === 'missing') addFinding(findings, 'warning', 'NO_MANIFEST', 'Packages/manifest.json', null, 'Manifest balíčků nebyl nalezen.');
  else if (projectInfo.manifestStatus === 'too-large') addFinding(findings, 'warning', 'MANIFEST_SIZE_LIMIT', 'Packages/manifest.json', null, 'Manifest balíčků překračuje limit 250 kB a byl vynechán, protože by mohl obsahovat příliš velký nebo poškozený obsah.');
  else if (projectInfo.manifestStatus !== 'ok') addFinding(findings, 'warning', 'UNSAFE_MANIFEST', 'Packages/manifest.json', null, 'Manifest balíčků není běžný bezpečně čitelný soubor.');
  if (projectInfo.assetsPathUnsafe) addFinding(findings, 'error', 'UNSAFE_ASSETS_PATH', 'Assets', null, 'Složka Assets je symbolický odkaz; audit ji z bezpečnostních důvodů neprojde.');
  if (!projectInfo.hasAssets && !projectInfo.assetsPathUnsafe) addFinding(findings, 'warning', 'NO_ASSETS', 'Assets', null, 'Složka Assets nebyla nalezena.');

  let manifest = null;
  if (projectInfo.manifestStatus === 'ok') {
    const manifestPath = path.join(root, 'Packages', 'manifest.json');
    try {
      manifest = JSON.parse((await readFileBounded(fs, manifestPath, 250_000, 'Unity manifest.json')).toString('utf8'));
      if (!manifest || typeof manifest !== 'object' || Array.isArray(manifest) || !manifest.dependencies || typeof manifest.dependencies !== 'object' || Array.isArray(manifest.dependencies)) throw new Error('invalid dependencies');
      packageCount = Object.keys(manifest.dependencies).length;
      for (const [name, version] of Object.entries(manifest.dependencies)) {
        if (typeof version !== 'string' || !version.trim()) addFinding(findings, 'warning', 'PACKAGE_VERSION', 'Packages/manifest.json', null, `Balíček ${name} nemá platnou verzi nebo zdroj.`);
      }
    } catch {
      addFinding(findings, 'error', 'INVALID_MANIFEST', 'Packages/manifest.json', null, 'manifest.json není platný JSON s objektem dependencies.');
    }
  }

  const assetsPath = path.join(root, 'Assets');
  const assetsMeta = new Map();
  const contentFiles = [];
  if (projectInfo.hasAssets && !projectInfo.assetsPathUnsafe) {
    const rootAssetReal = await fs.realpath(assetsPath);
    async function walk(dir, depth = 0) {
      if (depth > MAX_DEPTH || visited >= MAX_ENTRIES || checkedFiles >= MAX_TEXT_FILES) { limitReached = true; return; }
      let entries;
      try { entries = await fs.readdir(dir, { withFileTypes: true }); } catch (error) {
        addFinding(findings, 'info', 'DIRECTORY_UNREADABLE', path.relative(root, dir), null, `Složku nelze přečíst (${String(error?.code || 'chyba').slice(0, 40)}).`);
        return;
      }
      entries.sort((a, b) => a.name.localeCompare(b.name));
      for (const entry of entries) {
        if (visited >= MAX_ENTRIES || checkedFiles >= MAX_TEXT_FILES) { limitReached = true; break; }
        visited++;
        if (entry.name.startsWith('.') || EXCLUDED_DIRS.has(entry.name)) continue;
        const full = path.join(dir, entry.name);
        const rel = path.relative(root, full).split(path.sep).join('/');
        // Do not follow symlinks or special files, even when a symlink points back inside the project.
        let lstat;
        try { lstat = await fs.lstat(full); } catch { continue; }
        if (lstat.isSymbolicLink()) { addFinding(findings, 'warning', 'SYMLINK_SKIPPED', rel, null, 'Symbolický odkaz byl přeskočen; audit jej nerozbalí.'); continue; }
        if (lstat.isDirectory()) {
          const real = await fs.realpath(full).catch(() => null);
          if (!real || (real !== rootAssetReal && !real.startsWith(rootAssetReal + path.sep))) { addFinding(findings, 'warning', 'UNSAFE_DIRECTORY', rel, null, 'Složku nelze bezpečně zařadit do Assets.'); continue; }
          const metaCheck = await fs.lstat(full + '.meta').catch(() => null);
          if (!metaCheck || !metaCheck.isFile() || metaCheck.isSymbolicLink()) addFinding(findings, 'info', 'MISSING_META', rel, null, 'Složka v Assets nemá běžný odpovídající .meta soubor. Unity jej může vytvořit, zkontroluj však případné změny GUID.');
          await walk(full, depth + 1);
          continue;
        }
        if (!lstat.isFile()) continue;
        scannedAssets++;
        if (entry.name.endsWith('.meta')) {
          const assetRel = rel.slice(0, -5);
          let metaContent = '';
          if (lstat.size <= 100_000) {
            try { metaContent = (await readFileBounded(fs, full, 100_000, rel)).toString('utf8'); } catch {}
          }
          const guid = (metaContent.match(/^guid:\s*([a-f0-9]{32})\s*$/m) || [])[1];
          if (guid) {
            const lower = guid.toLowerCase();
            if (assetsMeta.has(lower) && assetsMeta.get(lower) !== assetRel) addFinding(findings, 'error', 'DUPLICATE_GUID', rel, null, `GUID ${lower} je duplicitní; první výskyt je ${assetsMeta.get(lower)}. To může rozbít reference na assety.`);
            else assetsMeta.set(lower, assetRel);
          } else if (metaContent) addFinding(findings, 'info', 'META_GUID_UNREADABLE', rel, null, 'V .meta souboru nebyl nalezen standardní GUID.');
          continue;
        }
        if (entry.name.startsWith('.')) continue;
        const meta = await fs.lstat(full + '.meta').catch(() => null);
        if (!meta || !meta.isFile() || meta.isSymbolicLink()) addFinding(findings, 'warning', 'MISSING_META', rel, null, 'Soubor v Assets nemá běžný odpovídající .meta soubor; Unity jej může znovu vytvořit a změnit GUID.');
        const ext = path.extname(entry.name).toLowerCase();
        if (ext === '.cs') scannedScripts++;
        if (!TEXT_EXTENSIONS.has(ext)) continue;
        if (lstat.size > MAX_FILE_BYTES) {
          if (ext === '.cs') addFinding(findings, 'info', 'LARGE_SCRIPT', rel, null, 'C# skript přesahuje 500 kB a obsah byl vynechán.');
          continue;
        }
        try {
          const content = (await readFileBounded(fs, full, MAX_FILE_BYTES, rel)).toString('utf8');
          checkedFiles++;
          contentFiles.push({ relative: rel, ext, content });
        } catch (error) {
          if (/překročil limit/i.test(String(error?.message || ''))) addFinding(findings, 'info', 'FILE_GREW_DURING_SCAN', rel, null, 'Soubor během čtení překročil limit a byl přeskočen.');
        }
      }
    }
    await walk(assetsPath);
  }

  for (const file of contentFiles) {
    if (file.ext !== '.cs') continue;
    const lines = file.content.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) if (/\b(TODO|FIXME|HACK)\b/i.test(lines[i])) addFinding(findings, 'info', 'TODO', file.relative, i + 1, lines[i].trim().slice(0, 240));
    if (!file.content.trim()) addFinding(findings, 'warning', 'EMPTY_SCRIPT', file.relative, 1, 'C# skript je prázdný.');
    if (/\basync\s+void\s+(?!Start\b|Update\b|On[A-Z]\w*\b)\w+\s*\(/.test(file.content)) addFinding(findings, 'warning', 'ASYNC_VOID', file.relative, null, 'Skript obsahuje async void mimo běžných Unity callbacků; výjimky se mohou hůře zachytávat.');
    if (/\bGameObject\.Find\s*\(/.test(file.content) && !/\b(?:Awake|Start)\s*\(/.test(file.content)) addFinding(findings, 'info', 'SCENE_LOOKUP', file.relative, null, 'Skript používá GameObject.Find bez zjevné inicializační metody; ověř výkon a dostupnost objektů.');
  }
  if (limitReached) addFinding(findings, 'info', 'SCAN_LIMIT_REACHED', 'Assets', null, `Audit dosáhl limitu ${MAX_ENTRIES} položek nebo ${MAX_TEXT_FILES} čitelných textových souborů; výsledek může být neúplný.`);

  const counts = {
    error: findings.filter(x => x.severity === 'error').length,
    warning: findings.filter(x => x.severity === 'warning').length,
    info: findings.filter(x => x.severity === 'info').length
  };
  return {
    isUnity: projectInfo.isUnity,
    root,
    generatedAt: new Date().toISOString(),
    version: projectInfo.version,
    hasAssets: projectInfo.hasAssets,
    hasManifest: projectInfo.hasManifest,
    scannedScripts,
    scannedAssets,
    visited,
    checkedFiles,
    packageCount,
    limitReached,
    counts,
    findings: findings.slice(0, MAX_FINDINGS),
    note: 'Read-only statická heuristická kontrola Unity projektu. Neprovádí kompilaci, neimportuje assety ani nespouští Unity; varování je nutné ověřit v editoru.'
  };
}

module.exports = Object.freeze({ auditUnityProject, getUnityProjectInfo, MAX_ENTRIES, MAX_DEPTH, MAX_TEXT_FILES, MAX_FILE_BYTES });
