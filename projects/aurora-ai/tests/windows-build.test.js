'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const read = name => fs.readFileSync(path.join(root, name), 'utf8');

test('Windows build explicitly downloads the Electron 45 runtime before packaging', () => {
  const pkg = JSON.parse(read('package.json'));
  const ps = read('BUILD-AURORA.ps1');
  const bat = read('BUILD-AURORA.bat');
  const start = read('START-BUILD-AURORA.bat');

  assert.equal(pkg.devDependencies?.electron, '45.1.0');
  assert.match(ps, /'--ignore-scripts'/);
  assert.match(ps, /22\.12\.0/);
  assert.match(ps, /'rebuild', '--foreground-scripts'/);
  assert.ok(ps.includes('node_modules\\.bin\\install-electron.cmd'));
  assert.match(ps, /Download Electron 45\.1\.0 Windows runtime \(install-electron\)/);
  assert.match(ps, /@\('--no'\)/);
  assert.ok(ps.includes('node_modules\\electron\\dist\\electron.exe'));
  assert.match(ps, /\$Attempt -le 2/);
  assert.match(ps, /--prefer-online/);
  assert.match(ps, /Add-LatestNpmDebugLog/);
  assert.match(ps, /Aurora-AI-Setup-\$AppVersion\.exe/);
  assert.match(bat, /BUILD-AURORA\.ps1/);
  assert.match(bat, /pause/);
  assert.match(start, /start "Aurora AI build"/);
});
