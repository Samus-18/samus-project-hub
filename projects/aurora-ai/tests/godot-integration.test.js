'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('Godot audit is wired through the secure main-process IPC and preload API', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  assert.match(main, /const GodotProjectAudit = require\('\.\/godot-project-audit'\)/);
  assert.match(main, /secureHandle\('project:godot-audit',async\(\)=>\{[\s\S]*?requirePlugin\('projectRead'\)[\s\S]*?approvedProjectRoot[\s\S]*?GodotProjectAudit\.auditGodotProject/);
  assert.match(preload, /godotAudit:\(\)=>ipcRenderer\.invoke\('project:godot-audit'\)/);
});

test('Godot integration is visible in Tools and presents bounded, escaped audit findings', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /class="panel integration godot-integration"/);
  for (const id of ['godotStatus','auditGodot','godotReport']) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(html, /GDScript/);
  assert.match(app, /window\.aurora\.godotAudit\(\)/);
  assert.match(app, /findings\.slice\(0,60\)\.map\(item=>`<div class="unity-finding"><b>\[\$\{esc\(/);
  assert.match(app, /Godot audit:/);
  assert.match(app, /document\.documentElement\.lang==='en'/);
  const prefs = require('../src/ui-preferences.js');
  assert.equal(prefs.translations['Godot diagnostika'], 'Godot diagnostics');
  assert.equal(prefs.translations['Provést Godot audit'], 'Run Godot audit');
  assert.equal(prefs.translations['AUDIT: VAROVÁNÍ'], 'AUDIT: WARNINGS');
});

test('Godot static audit is included in syntax checks and the current release manifest', () => {
  const pkg = JSON.parse(read('package.json'));
  assert.match(pkg.scripts.check, /node scripts\/check-syntax\.js/);
  const syntax = read('scripts/check-syntax.js');
  assert.match(syntax, /const roots = \[path\.join\(root, 'src'\), path\.join\(root, 'scripts'\)\]/);
  assert.match(syntax, /for \(const dir of roots\).*walk\(dir\)/s);
  assert.equal(pkg.version, '0.8.6');
  assert.ok(pkg.build.files.includes('CHANGELOG-v0.8.6.md'));
  assert.ok(pkg.build.files.includes('AUDIT-CZ-v0.8.6.md'));
});
