'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('navigating to Settings hides the shared chat instead of stacking both views', () => {
  const app = read('src/app.js');
  const css = read('src/work-chat-unified.css');
  assert.match(app, /el\.classList\.toggle\('hidden',!visible\);el\.setAttribute\('aria-hidden',String\(!visible\)\);el\.inert=!visible/);
  assert.match(css, /#chatView:not\(\.hidden\)\{display:flex!important\}/);
  assert.match(css, /#chatView\.hidden\{display:none!important\}/);
  assert.doesNotMatch(css, /#chatView\{display:flex!important\}/);
  assert.match(css, /\.main>#settingsView[\s\S]*?height:auto!important/);
});

test('the Quick Actions palette does not duplicate the Chat / Work mode switch', () => {
  const html = read('src/index.html');
  assert.match(html, /id="chatModeToggle"[\s\S]*?id="workModeToggle"/);
  assert.doesNotMatch(html, /data-command="agent"/);
  assert.doesNotMatch(html, /<b>Work mode<\/b>/);
});

test('Saved section uses one localized heading and a live pinned-chat count', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const section = html.slice(html.indexOf('class="sidebar-collection-section"'), html.indexOf('class="sidebar-collection-section projects-sidebar-section"'));
  assert.match(section, /<span class="sidebar-heading-label">[\s\S]*?ULOŽENÉ/);
  assert.match(section, /id="pinnedChatsCount"/);
  assert.doesNotMatch(section, /PINNED/);
  assert.match(app, /const count=\$\('pinnedChatsCount'\)/);
  assert.match(app, /count\.textContent=pinned\.length\?String\(pinned\.length\):''/);
  assert.match(app, /Pinned chats stay here for quick access/);
});

test('Work composer distinguishes project creation from file attachment', () => {
  const html = read('src/index.html');
  const css = read('src/work-chat-unified.css');
  assert.match(html, /id="workProjectControl"[\s\S]*?id="workProjectSelect"/);
  assert.doesNotMatch(html, /workAddProjectBtn|work-project-add/);
  assert.match(html, /<button[^>]*aria-label="Připojit soubory nebo kontext"[^>]*id="attachBtn"|<button[^>]*id="attachBtn"[^>]*aria-label="Připojit soubory nebo kontext"/);
  const attach = html.slice(html.indexOf('id="attachBtn"'), html.indexOf('id="attachBtn"') + 650);
  assert.match(attach, /<svg[\s\S]*?<path/);
  assert.doesNotMatch(attach, />＋</);
  assert.match(css, /#attachBtn svg\{display:block!important/);
});

test('sidebar search has a single calm focus ring without a nested browser outline', () => {
  const css = read('src/work-chat-unified.css');
  assert.match(css, /\.sidebar-search input:focus-visible\{outline:none!important/);
  assert.match(css, /\.sidebar-search:focus-within\{border-color:rgba\(170,145,255/);
});


test('empty chat centers the welcome and composer as one lower-centered group', () => {
  const html = read('src/index.html');
  const css = read('src/chat-centered-layout.css');
  assert.match(html, /href="chat-centered-layout\.css"/);
  assert.match(css, /\.chatview:has\(#messages:empty\)\s*\{[^}]*justify-content:center!important/s);
  assert.match(css, /\.chatview:has\(#messages:empty\) \.welcome\s*\{[^}]*margin:0 auto 18px!important/s);
  assert.match(css, /\.chatview:has\(#messages:empty\) \.composer-wrap\s*\{[^}]*margin:0 auto!important/s);
  assert.match(css, /padding-top:clamp\(32px,6vh,60px\)!important/);
});

test('short conversations settle near the composer while longer histories stay scrollable', () => {
  const css = read('src/chat-centered-layout.css');
  const base = read('src/aurora-reference-refit.css');
  assert.match(css, /\.messages:has\(\.message\):not\(:has\(\.message:nth-child\(3\)\)\) \.message:first-child\s*\{[^}]*margin-top:auto!important/s);
  assert.match(base, /\.messages\{[^}]*overflow-y:auto!important/s);
});
