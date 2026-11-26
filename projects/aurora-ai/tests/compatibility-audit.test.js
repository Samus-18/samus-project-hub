'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Compatibility = require('../src/compatibility-audit');

test('parses common semantic version formats without guessing missing versions', () => {
  assert.deepEqual(Compatibility.parseVersion('v22.12.1'), { major: 22, minor: 12, patch: 1, prerelease: '', raw: 'v22.12.1' });
  assert.deepEqual(Compatibility.parseVersion('Python 3.12.4'), { major: 3, minor: 12, patch: 4, prerelease: '', raw: 'Python 3.12.4' });
  assert.equal(Compatibility.parseVersion('not installed'), null);
});

test('compares versions and treats prereleases below stable releases', () => {
  assert.equal(Compatibility.compareVersions('22.12.0', '22.12.0'), 0);
  assert.equal(Compatibility.compareVersions('24.1.0', '22.12.0'), 1);
  assert.equal(Compatibility.compareVersions('22.11.9', '22.12.0'), -1);
  assert.equal(Compatibility.compareVersions('4.0.0-beta.1', '4.0.0'), -1);
  assert.equal(Compatibility.compareVersions('4.0.0.stable.official', '4.0.0'), 0);
  assert.equal(Compatibility.compareVersions('v24.1.0+build.7', '24.1.0'), 0);
  assert.equal(Compatibility.compareVersions('unknown', '4.0.0'), null);
});

test('assesses an available, outdated, missing and optional tool honestly', () => {
  const node = Compatibility.assessTool('node', { path: 'C:\\Program Files\\nodejs\\node.exe', version: 'v26.8.1' });
  assert.equal(node.status, 'available');
  assert.equal(node.compatible, true);
  const oldGodot = Compatibility.assessTool('godot', { path: '/opt/godot', version: '3.5.2' });
  assert.equal(oldGodot.status, 'outdated');
  assert.equal(oldGodot.compatible, false);
  const missingPython = Compatibility.assessTool('python', { status: 'missing' });
  assert.equal(missingPython.status, 'missing');
  assert.equal(missingPython.compatible, false);
  const missingBlender = Compatibility.assessTool('blender', { status: 'missing' });
  assert.equal(missingBlender.status, 'missing');
  assert.equal(missingBlender.compatible, null);
  assert.match(missingBlender.note, /volitelná/);
});

test('summarizes compatibility status without counting missing tools as installed', () => {
  const results = [
    Compatibility.assessTool('node', { path: 'node.exe', version: '24.0.0' }),
    Compatibility.assessTool('godot', { path: 'godot.exe', version: '3.5.0' }),
    Compatibility.assessTool('python', { status: 'missing' }),
    Compatibility.assessTool('git', { path: 'git.exe' })
  ];
  assert.deepEqual(Compatibility.summarizeCompatibility(results), { total: 4, available: 1, outdated: 1, missing: 1, unverified: 1, checked: 3 });
});

test('covers Java, Go and Rust toolchains used by project workflows', () => {
  assert.equal(Compatibility.assessTool('java', { path: 'java.exe', version: '17.0.10' }).status, 'available');
  assert.equal(Compatibility.assessTool('java', { path: 'java.exe', version: '11.0.22' }).status, 'outdated');
  assert.equal(Compatibility.assessTool('go', { path: 'go.exe', version: 'go1.23.4' }).status, 'available');
  assert.equal(Compatibility.assessTool('rust', { path: 'rustc.exe', version: 'rustc 1.81.0' }).status, 'available');
});
