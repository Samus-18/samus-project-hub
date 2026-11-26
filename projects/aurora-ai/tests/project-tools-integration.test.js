'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('Project tool backends are imported and exposed through guarded main-process IPC', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  for (const name of ['CapCutProjectAudit', 'ProjectCheckRunner', 'RobloxPluginManager']) {
    assert.match(main, new RegExp(`const ${name} = require\\('./${name === 'CapCutProjectAudit' ? 'capcut-project-audit' : name === 'ProjectCheckRunner' ? 'project-check-runner' : 'roblox-plugin-manager'}'\\)`));
  }
  for (const channel of ['project:capcut-audit', 'project:check-actions', 'project:run-check', 'project:cancel-check', 'project:unity-bridge-source', 'roblox:status', 'roblox:install', 'roblox:disable']) {
    assert.ok(main.includes(`secureHandle('${channel}'`), `missing guarded IPC ${channel}`);
  }
  for (const api of ['capcutAudit:', 'projectCheckActions:', 'runProjectCheck:', 'cancelProjectCheck:', 'unityBridgeSource:', 'robloxStatus:', 'installRobloxBridge:', 'disableRobloxBridge:']) assert.ok(preload.includes(api), `missing preload API ${api}`);
});

test('CapCut audit is reachable from tools UI and renders escaped bounded findings', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /class="panel integration capcut-integration"/);
  for (const id of ['capcutStatus', 'auditCapCut', 'capcutReport']) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(app, /window\.aurora\.capcutAudit\(\)/);
  assert.match(app, /findings\.slice\(0,40\)\.map\(item=>`<div class="unity-finding"><b>\[\$\{esc\(/);
  assert.match(app, /CapCut audit:/);
});

test('Project check runner has UI, cancellation and native confirmation for script-running actions', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const main = read('src/main.js');
  for (const id of ['projectCheckAction', 'chooseGodotEditor', 'chooseUnityEditor', 'runProjectCheck', 'cancelProjectCheck', 'projectCheckOutput']) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(app, /window\.aurora\.runProjectCheck\(action\)/);
  assert.match(app, /window\.aurora\.cancelProjectCheck\(\)/);
  assert.match(main, /action\.executesProjectScripts/);
  assert.match(main, /Spustit kontrolu/);
  assert.match(main, /ProjectCheckRunner\.runProjectCheck/);
});

test('Roblox Bridge and Unity Editor Bridge have explicit install/proposal actions', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const main = read('src/main.js');
  for (const id of ['robloxStatus', 'refreshRobloxBridge', 'installRobloxBridge', 'disableRobloxBridge', 'installUnityBridge']) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(app, /window\.aurora\.installRobloxBridge\(\)/);
  assert.match(app, /window\.aurora\.disableRobloxBridge\(\)/);
  assert.match(app, /window\.aurora\.unityBridgeSource\(\)/);
  assert.match(app, /\$\('previewEdit'\)\.click\(\)/);
  assert.match(main, /RobloxPluginManager\.install/);
  assert.match(main, /RobloxPluginManager\.uninstall/);
  assert.match(main, /Assets\/Editor\/AuroraAI\/AuroraEditorBridge\.cs/);
});
