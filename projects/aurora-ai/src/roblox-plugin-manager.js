'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');

const PLUGIN_NAME = 'AuroraBridge.rbxmx';
const PLUGIN_MARKER = '-- Aurora AI Bridge for Roblox Studio';
const MAX_PLUGIN_BYTES = 256 * 1024;

function nonce() { return `${Date.now()}-${crypto.randomBytes(4).toString('hex')}`; }

function toPluginModel(source) {
  const safeSource = String(source).replace(/\]\]>/g, ']]]]><![CDATA[>');
  return `<?xml version="1.0" encoding="utf-8"?>
<roblox version="4">
  <External>null</External>
  <External>nil</External>
  <Item class="Script" referent="RBX0">
    <Properties>
      <bool name="Archivable">true</bool>
      <bool name="Disabled">false</bool>
      <string name="Name">AuroraBridge</string>
      <ProtectedString name="Source"><![CDATA[${safeSource}]]></ProtectedString>
    </Properties>
  </Item>
</roblox>
`;
}

function isAuroraPluginModel(content) {
  return typeof content === 'string' && content.startsWith('<?xml') && content.includes('<roblox') && content.includes('<Item class="Script"') && content.includes(PLUGIN_MARKER);
}

async function inspectRegularFile(file, maxBytes = MAX_PLUGIN_BYTES) {
  let stat;
  try { stat = await fs.lstat(file); }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
  if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('Soubor pluginu není běžný soubor nebo je symbolickým odkazem.');
  if (stat.size > maxBytes) throw new Error('Soubor pluginu překračuje bezpečný limit 256 kB.');
  return stat;
}

async function ensurePluginsDirectory(localAppData) {
  if (typeof localAppData !== 'string' || !path.isAbsolute(localAppData)) throw new Error('Cesta LOCALAPPDATA není platná.');
  const base = path.resolve(localAppData);
  const robloxRoot = path.join(base, 'Roblox');
  const plugins = path.join(robloxRoot, 'Plugins');
  for (const dir of [robloxRoot, plugins]) {
    try {
      const stat = await fs.lstat(dir);
      if (!stat.isDirectory() || stat.isSymbolicLink()) throw new Error('Složka Roblox Plugins obsahuje symbolický odkaz nebo není běžná složka.');
    } catch (error) {
      if (error.code !== 'ENOENT') throw error;
      await fs.mkdir(dir, { recursive: false });
    }
  }
  const realBase = await fs.realpath(base);
  const realPlugins = await fs.realpath(plugins);
  if (realPlugins !== path.join(realBase, 'Roblox', 'Plugins')) throw new Error('Cílová složka pluginů Roblox míří mimo očekávané umístění.');
  return { plugins, target: path.join(plugins, PLUGIN_NAME) };
}

async function readTrustedSource(sourcePath) {
  const stat = await inspectRegularFile(sourcePath);
  if (!stat) throw new Error('Instalační soubor Aurora Roblox Bridge nebyl nalezen.');
  const realPath = await fs.realpath(sourcePath);
  if (path.resolve(realPath) !== path.resolve(sourcePath)) throw new Error('Instalační zdroj pluginu nesmí být symbolický odkaz.');
  const content = await fs.readFile(realPath, 'utf8');
  if (!content.startsWith(PLUGIN_MARKER)) throw new Error('Instalační zdroj neobsahuje očekávanou značku Aurora Bridge.');
  return content;
}

async function status({ localAppData } = {}) {
  if (typeof localAppData !== 'string' || !path.isAbsolute(localAppData)) return { installed: false, conflict: false, available: false, reason: 'LOCALAPPDATA není nastaveno.' };
  const robloxRoot = path.join(path.resolve(localAppData), 'Roblox');
  const plugins = path.join(robloxRoot, 'Plugins');
  const target = path.join(plugins, PLUGIN_NAME);
  for (const dir of [robloxRoot, plugins]) {
    try {
      const dirStat = await fs.lstat(dir);
      if (!dirStat.isDirectory() || dirStat.isSymbolicLink()) throw new Error('Složka Roblox Plugins nebo její nadřazená složka není bezpečná běžná složka.');
    } catch (error) {
      if (error.code === 'ENOENT') return { installed: false, conflict: false, available: true, directory: plugins, target };
      throw error;
    }
  }
  const expected = path.join(await fs.realpath(localAppData), 'Roblox', 'Plugins');
  if (await fs.realpath(plugins) !== expected) throw new Error('Složka Roblox Plugins míří mimo očekávané umístění.');
  if (await inspectRegularFile(target) === null) return { installed: false, conflict: false, available: true, directory: plugins, target };
  const content = await fs.readFile(target, 'utf8');
  const owned = isAuroraPluginModel(content);
  return { installed: owned, conflict: !owned, available: true, directory: plugins, target, size: Buffer.byteLength(content) };
}

async function install({ localAppData, sourcePath } = {}) {
  const source = await readTrustedSource(sourcePath);
  const content = toPluginModel(source);
  const { plugins, target } = await ensurePluginsDirectory(localAppData);
  const existingStat = await inspectRegularFile(target);
  let backupPath = null;
  if (existingStat) {
    const existing = await fs.readFile(target, 'utf8');
    if (!isAuroraPluginModel(existing)) throw new Error('V cílovém umístění už je jiný AuroraBridge.rbxmx. Nebyl přepsán. Přejmenuj ho nebo ho ručně zazálohuj.');
    backupPath = `${target}.backup-${nonce()}`;
    await fs.copyFile(target, backupPath, fs.constants.COPYFILE_EXCL);
  }

  const tempPath = path.join(plugins, `.${PLUGIN_NAME}.tmp-${nonce()}`);
  let movedExisting = false;
  try {
    const handle = await fs.open(tempPath, 'wx', 0o600);
    try { await handle.writeFile(content, 'utf8'); await handle.sync(); }
    finally { await handle.close(); }
    if (existingStat) { await fs.unlink(target); movedExisting = true; }
    await fs.rename(tempPath, target);
  } catch (error) {
    await fs.rm(tempPath, { force: true }).catch(() => {});
    if (movedExisting && backupPath) {
      const restore = `${target}.restore-${nonce()}`;
      try { await fs.copyFile(backupPath, restore, fs.constants.COPYFILE_EXCL); await fs.rename(restore, target); }
      catch (restoreError) { throw new Error(`Instalace selhala a původní plugin se nepodařilo obnovit. Záloha: ${backupPath}. Původní chyba: ${error.message}; obnova: ${restoreError.message}`); }
    }
    throw new Error(`Plugin se nepodařilo nainstalovat: ${String(error.message || error).slice(0, 240)}`);
  }
  return { installed: true, target, backupPath, updated: !!existingStat, bytes: Buffer.byteLength(content) };
}

async function uninstall({ localAppData } = {}) {
  const { target } = await ensurePluginsDirectory(localAppData);
  const stat = await inspectRegularFile(target);
  if (!stat) return { removed: false, reason: 'Plugin není nainstalovaný.' };
  const content = await fs.readFile(target, 'utf8');
  if (!isAuroraPluginModel(content)) throw new Error('Cílový soubor není Aurora Bridge. Nebyl změněn.');
  const disabledPath = `${target}.disabled-${nonce()}`;
  await fs.rename(target, disabledPath);
  return { removed: true, disabledPath };
}

module.exports = Object.freeze({ PLUGIN_NAME, PLUGIN_MARKER, MAX_PLUGIN_BYTES, ensurePluginsDirectory, toPluginModel, isAuroraPluginModel, status, install, uninstall });
