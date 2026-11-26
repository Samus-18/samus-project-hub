'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const read = name => fs.readFileSync(path.join(root, name), 'utf8');

test('host compatibility scan is connected through guarded main-process IPC and preload', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  assert.match(main, /require\(['"]\.\/compatibility-audit['"]\)/);
  assert.match(main, /secureHandle\(['"]system:compatibility['"],\s*async\s*\(\)\s*=>\s*inspectHostCompatibility\(\)\)/);
  assert.match(main, /spawn\(executable,\s*args,\s*\{[^}]*shell:\s*false/s);
  assert.match(preload, /compatibility:\(\)\s*=>\s*ipcRenderer\.invoke\(['"]system:compatibility['"]\)/);
});

test('compatibility view exposes refresh, runtime and tool status regions', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /id="refreshCompatibility"/);
  assert.match(html, /id="compatibilityRuntime"/);
  assert.match(html, /id="compatibilitySummary"/);
  assert.match(html, /id="compatibilityGrid"/);
  assert.match(app, /window\.aurora\.compatibility\(\)/);
  assert.match(app, /refreshCompatibility\(\{quiet:true\}\)/);
});

test('compatibility results are rendered as text rather than trusted HTML', () => {
  const app = read('src/app.js');
  const start = app.indexOf('async function refreshCompatibility');
  const end = app.indexOf('function toolCard', start);
  assert.ok(start >= 0 && end > start);
  const source = app.slice(start, end);
  assert.match(source, /purpose\.textContent/);
  assert.match(source, /detail\.textContent/);
  assert.match(source, /note\.textContent/);
  assert.doesNotMatch(source, /card\.innerHTML\s*=/);
});

test('host compatibility registry includes Java, Go and Rust toolchains', () => {
  const main = read('src/main.js');
  const audit = read('src/compatibility-audit.js');
  for (const id of ['java', 'go', 'rust']) {
    assert.match(main, new RegExp(`${id}: \{ names:`));
    assert.match(audit, new RegExp(`${id}: Object\.freeze`));
  }
});
