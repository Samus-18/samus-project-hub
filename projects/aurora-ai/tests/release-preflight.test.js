'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const root = path.resolve(__dirname, '..');

test('release preflight passes for the current source package', () => {
  const output = execFileSync(process.execPath, [path.join(root, 'scripts', 'release-preflight.js')], { cwd: root, encoding: 'utf8' });
  assert.match(output, /Aurora AI release preflight v0\.8\.6/);
  assert.match(output, /Preflight passed/);
  assert.match(output, /Electron security defaults/);
  assert.match(output, /CSS local resources exist/);
  assert.match(output, /Local runtime smoke test exists/);
});

test('release package does not pack historical audit files and targets NSIS', () => {
  const fs = require('node:fs');
  const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
  assert.ok(pkg.build.files.includes('src/**/*'));
  assert.ok(pkg.build.files.includes('integrations/**/*'));
  assert.ok(pkg.build.files.includes('AUDIT-CZ-v0.8.6.md'));
  assert.ok(!pkg.build.files.includes('AUDIT-CZ-v0.7.7.md'));
  assert.equal(pkg.build.win.target, 'nsis');
  assert.equal(pkg.devDependencies.electron, '45.1.0');
  assert.equal(pkg.devDependencies['electron-builder'], '26.18.1');
  assert.equal(pkg.scripts['smoke:local'], 'node scripts/runtime-smoke.js');
});
