'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('CapCut audit is wired through protected IPC and the allowlisted preload', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  assert.match(main, /const CapCutProjectAudit = require\('\.\/capcut-project-audit'\)/);
  assert.match(main, /secureHandle\('project:capcut-audit'/);
  assert.match(main, /CapCutProjectAudit\.auditCapCutProject\(approvedProjectRoot\)/);
  assert.match(preload, /capcutAudit:\(\)=>ipcRenderer\.invoke\('project:capcut-audit'\)/);
});

test('CapCut audit card renders bounded escaped read-only report', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /class="panel integration capcut-integration"/);
  for (const id of ['capcutStatus','auditCapCut','capcutReport']) assert.match(html, new RegExp(`id="${id}"`));
  assert.match(app, /window\.aurora\.capcutAudit\(\)/);
  assert.match(app, /findings\.slice\(0,40\)\.map\(item=>`<div class="unity-finding"><b>\[\$\{esc\(/);
  assert.match(app, /CapCut audit:/);
});
