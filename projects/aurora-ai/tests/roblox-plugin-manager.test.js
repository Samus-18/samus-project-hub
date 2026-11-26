const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const Manager = require('../src/roblox-plugin-manager');

async function fixture() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-roblox-plugin-'));
  const localAppData = path.join(root, 'LocalAppData');
  const sourcePath = path.join(root, 'AuroraBridge.plugin.lua');
  await fs.mkdir(localAppData, { recursive: true });
  await fs.writeFile(sourcePath, `${Manager.PLUGIN_MARKER}\n-- trusted test plugin\n`, 'utf8');
  return { root, localAppData, sourcePath };
}

test('installs bridge into Roblox Plugins and reports installed state', async () => {
  const f = await fixture();
  const installed = await Manager.install(f);
  assert.equal(installed.installed, true);
  assert.equal(installed.updated, false);
  assert.match(installed.target, /Roblox[\\/]Plugins[\\/]AuroraBridge\.rbxmx$/);
  const content = await fs.readFile(installed.target, 'utf8');
  assert.match(content, /<roblox version=\"4\">/);
  assert.match(content, /<ProtectedString name=\"Source\">/);
  assert.match(content, /trusted test plugin/);
  const status = await Manager.status({ localAppData: f.localAppData });
  assert.equal(status.installed, true);
  assert.equal(status.conflict, false);
});

test('updates only its own plugin and preserves a backup', async () => {
  const f = await fixture();
  const first = await Manager.install(f);
  await fs.writeFile(f.sourcePath, `${Manager.PLUGIN_MARKER}\n-- updated version\n`, 'utf8');
  const updated = await Manager.install(f);
  assert.equal(updated.updated, true);
  assert.ok(updated.backupPath);
  assert.match(await fs.readFile(updated.backupPath, 'utf8'), /trusted test plugin/);
  assert.match(await fs.readFile(first.target, 'utf8'), /updated version/);
});

test('refuses to overwrite another plugin and refuses symlink targets', async () => {
  const f = await fixture();
  const { target } = await Manager.ensurePluginsDirectory(f.localAppData);
  await fs.writeFile(target, '-- someone else\n', 'utf8');
  await assert.rejects(Manager.install(f), /jiný AuroraBridge/);
  await fs.unlink(target);
  const outside = path.join(f.root, 'outside.lua');
  await fs.writeFile(outside, Manager.PLUGIN_MARKER, 'utf8');
  await fs.symlink(outside, target);
  await assert.rejects(Manager.install(f), /symbolickým odkazem/);
});

test('uninstall disables only owned bridge and keeps recoverable file', async () => {
  const f = await fixture();
  await Manager.install(f);
  const result = await Manager.uninstall({ localAppData: f.localAppData });
  assert.equal(result.removed, true);
  assert.match(result.disabledPath, /\.disabled-/);
  await assert.rejects(fs.access(path.join(f.localAppData, 'Roblox', 'Plugins', Manager.PLUGIN_NAME)));
  assert.match(await fs.readFile(result.disabledPath, 'utf8'), /Aurora AI Bridge/);
});

test('rejects untrusted install source and non-absolute local app data', async () => {
  const f = await fixture();
  await fs.writeFile(f.sourcePath, '-- not Aurora\n', 'utf8');
  await assert.rejects(Manager.install(f), /značku Aurora Bridge/);
  await assert.rejects(Manager.ensurePluginsDirectory('relative/path'), /LOCALAPPDATA/);
});
