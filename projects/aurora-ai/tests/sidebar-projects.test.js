'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('ChatGPT-style sidebar orders Saved, Projects, and Recents with independent actions', () => {
  const html = read('src/index.html');
  const saved = html.indexOf('id="pinnedChats"');
  const projects = html.indexOf('id="sidebarProjectList"');
  const recent = html.indexOf('class="history-title"');
  assert.ok(saved > 0 && projects > saved && recent > projects, 'Saved must be above Projects, and Projects above Recents');
  for (const id of ['newChat','historySearch','defaultWorkspace','addProjectBtnSidebar','viewAllProjects','createProjectSpaceBtn']) assert.ok(html.includes(`id="${id}"`), `missing ${id}`);
  assert.match(html, /data-history-filter="recent"[\s\S]*data-history-filter="archived"/);
  assert.doesNotMatch(html, /data-history-filter="projects"/, 'projects are their own navigation collection, not a Recents filter');
});

test('chat action menu defines and wires rename, delete, pin, archive, and project assignment', () => {
  const app = read('src/app.js');
  const history = app.slice(app.indexOf('function renderHistory()'), app.indexOf('async function chooseFolder'));
  for (const action of ['const pin=document.createElement', 'const project=document.createElement', 'const move=document.createElement', 'const rename=document.createElement', 'const remove=document.createElement', 'const archive=document.createElement']) {
    assert.ok(history.includes(action), `missing action declaration: ${action}`);
  }
  assert.match(history, /actions\.append\(pin,project,moveLabel,rename,archive,remove\)/);
  assert.match(history, /remove\.onclick=\(\)=>\{if\(guardConversationSwitch\(\)\)return;if\(!confirm/);
  assert.match(history, /move\.onchange=\(\)=>\{if\(guardConversationSwitch\(\)\)\{move\.value=c\.projectSpaceId/);
  assert.match(app, /async function activateConversation\(c\)\{if\(!c\|\|guardConversationSwitch\(\)\)return/);
});

test('multiple project workspaces are safely persisted and removing one never deletes its disk folder', () => {
  const main = read('src/main.js');
  const app = read('src/app.js');
  const preload = read('src/preload.js');
  const persistence = read('src/workspace-persistence.js');
  for (const channel of ['project-spaces:list','project-spaces:create','project-spaces:select','project-spaces:rename','project-spaces:remove']) assert.ok(main.includes(`secureHandle('${channel}'`), `missing ${channel}`);
  assert.match(main, /fs\.realpath\(result\.filePaths\[0\]\)/);
  assert.match(main, /ProjectSpaces\.toPublicProjectSpace/);
  assert.match(main, /project-spaces:remove'[\s\S]{0,700}ProjectSpaces\.removeProjectSpace/);
  assert.doesNotMatch(main, /project-spaces:remove'[\s\S]{0,700}fs\.rm|fs\.rmdir|removeSync/);
  for (const method of ['listProjectSpaces','createProjectSpace','selectProjectSpace','renameProjectSpace','removeProjectSpace']) assert.ok(preload.includes(method), `missing preload method ${method}`);
  assert.match(app, /for\(const c of state\.conversations\)\{if\(c\.projectSpaceId===id\)\{c\.projectSpaceId=null;c\.project=false\}\}/);
  assert.match(app, /async function activateProjectSpace\(id,destination='chat'\)/);
  assert.match(persistence, /activeProjectId/);
  assert.match(persistence, /projectSpaceId/);
  assert.doesNotMatch(persistence, /folderPath|approvedProjectRoot/);
});
