'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('agent file changeset extensions are a subset of the main-process write allowlist', () => {
  const proposals = require('../src/agent-change-proposals');
  const main = read('src/main.js');
  const block = main.match(/const SAFE_TEXT_EXTS = new Set\(\[([^\]]+)\]\)/);
  assert.ok(block, 'main process has an explicit file-write allowlist');
  const allowed = new Set([...block[1].matchAll(/'([^']+)'/g)].map(match => match[1]));
  for (const extension of proposals.ALLOWED_EXTENSIONS) assert.ok(allowed.has(extension), `main process must allow proposal extension ${extension}`);
});

test('proposal preview loads the complete old and new file only on explicit request and before approval', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  const app = read('src/app.js');
  assert.match(main, /secureHandle\('project:proposal-preview-file'/);
  assert.match(main, /hash!==edit\.oldHash/);
  assert.match(main, /return \{path:relative,exists:edit\.oldContent!==null,oldContent:edit\.oldContent,newContent:edit\.content/);
  assert.match(preload, /readProposalPreviewFile:payload=>ipcRenderer\.invoke\('project:proposal-preview-file',payload\)/);
  assert.match(app, /function renderProposalReview\(/);
  assert.match(app, /window\.aurora\.readProposalPreviewFile\(payload\)/);
  assert.match(app, /approve\.disabled=loaded\.size!==files\.length/);
  assert.match(app, /Nejdřív načti a projdi celý obsah všech souborů/);
});

test('main process and changeset parser share cross-platform duplicate-safe project file types', () => {
  const main = read('src/main.js');
  const proposals = require('../src/agent-change-proposals');
  assert.match(main, /relative\.toLocaleLowerCase\('en'\)/);
  assert.match(main, /project:proposal-preview-file/);
  for (const extension of ['.csv','.tsv','.gd','.tscn','.tres','.godot','.lua','.rs','.go','.java','.csproj','.sln','.props','.targets']) {
    assert.ok(proposals.ALLOWED_EXTENSIONS.has(extension), `changesets support ${extension}`);
  }
});

test('simple Agent Studio mode clearly starts two roles and advanced mode starts four', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /Spustit hlavního a vedlejšího agenta/);
  assert.match(app, /mode==='advanced'\?'＋ Spustit čtyři role':'＋ Spustit hlavního a vedlejšího agenta'/);
  assert.match(app, /state\.agentComplexity==='advanced'\|\|\['planner','implementer'\]\.includes\(stage\.id\)/);
});

test('VS Code is selected explicitly and opened on the approved project with shell disabled', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  const app = read('src/app.js');
  const html = read('src/index.html');
  assert.match(main, /'vscode'/);
  assert.match(main, /secureHandle\('project:open-vscode'/);
  assert.match(main, /spawn\(executable, \['--new-window', root\]/);
  assert.match(main, /shell: false, detached: true/);
  assert.match(preload, /openVSCode:.*project:open-vscode/);
  assert.match(app, /window\.aurora\.openVSCode\(\)/);
  assert.match(html, /id="chooseVSCode"/);
  assert.match(html, /id="openVSCode"/);
});
