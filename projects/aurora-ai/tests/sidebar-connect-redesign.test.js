'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('sidebar can be collapsed and restored without removing its own toggle', () => {
  const html = read('src/index.html');
  const prefs = read('src/ui-preferences.js');
  const css = read('src/connect-sidebar-refinement.css');
  assert.match(html, /id="sidebarToggle"/);
  assert.match(prefs, /getElementById\('sidebarToggle'\)\?\.addEventListener\('click'/);
  assert.match(prefs, /update\(\{sidebarCollapsed:!prefs\.sidebarCollapsed\}\)/);
  assert.match(css, /\.app\.sidebar-collapsed\{grid-template-columns:0 minmax\(0,1fr\)!important;/);
  assert.match(css, /\.sidebar-toggle\{display:inline-grid!important/);
});

test('legacy navigation-hidden preference cannot hide the main navigation in the new sidebar', () => {
  const prefs = read('src/ui-preferences.js');
  const css = read('src/connect-sidebar-refinement.css');
  assert.match(prefs, /const sidebar=document\.querySelector\('\.sidebar'\);if\(sidebar\)sidebar\.classList\.remove\('navigation-hidden'\)/);
  assert.match(css, /\.sidebar #mainNavigation\{display:grid!important/);
  assert.match(css, /#mainNavigation \.nav i svg\{width:17px!important/);
});

test('top bar gives Chat and Work their own column and separates them from runtime actions', () => {
  const html = read('src/index.html');
  const css = read('src/connect-sidebar-refinement.css');
  assert.match(html, /id="chatModeToggle"[\s\S]*?id="workModeToggle"/);
  assert.match(html, /id="sidebarToggle"/);
  assert.match(css, /grid-template-columns:34px minmax\(110px,1fr\) 184px auto!important/);
  assert.match(css, /\.topbar>\.chat-work-switch\{grid-column:3/);
  assert.match(css, /\.topbar>\.top-actions\{grid-column:4/);
});

test('Work composer has one project selector and a distinct paperclip, with project creation only in the sidebar', () => {
  const html = read('src/index.html');
  const css = read('src/connect-sidebar-refinement.css');
  const start = html.indexOf('id="workProjectControl"');
  const end = html.indexOf('</div>', start);
  const projectControl = html.slice(start, end);
  assert.match(projectControl, /id="workProjectSelect"/);
  assert.doesNotMatch(projectControl, /<button|\+|addProject/i);
  assert.match(html, /id="addProjectBtnSidebar"/);
  assert.match(html, /id="attachBtn"/);
  assert.match(css, /#workProjectControl>button,.work-project-add,#workAddProjectBtn\{display:none!important/);
});

test('Quick Actions routes Connect to the real integration hub instead of project-context attachment', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const navStart = html.indexOf('id="mainNavigation"');
  const nav = html.slice(html.lastIndexOf('<nav', navStart), html.indexOf('</nav>', navStart));
  const paletteStart = html.indexOf('id="paletteItems"');
  const palette = html.slice(paletteStart, html.indexOf('id="paletteEmpty"', paletteStart));
  assert.match(nav, /data-view="connect"/);
  assert.match(palette, /data-command="connect"/);
  assert.doesNotMatch(palette, /data-command="context"|Přidat projektový kontext/);
  assert.match(app, /case'connect':setView\('connect'\)/);
  assert.match(html, /id="connectUnityCard"[\s\S]*?id="connectUnrealCard"[\s\S]*?id="connectGodotCard"[\s\S]*?id="connectRobloxCard"[\s\S]*?id="connectVSCodeCard"/);
});

test('Connect only advertises implemented actions as ready and marks Unreal as planned', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  assert.match(html, /id="connectUnityBtn"/);
  assert.match(html, /id="connectGodotBtn"/);
  assert.match(html, /id="connectRobloxBtn"/);
  assert.match(html, /id="connectVSCodeBtn"/);
  assert.match(html, /id="connectUnrealCard"[\s\S]*?PŘIPRAVUJE SE/);
  assert.match(app, /connectUnityBtn[^\n]*openConnectTool\('unityIntegration'\)/);
  assert.match(app, /connectGodotBtn[^\n]*openConnectTool\('godotIntegration'\)/);
  assert.match(app, /connectRobloxBtn[^\n]*openConnectTool\('robloxIntegration'\)/);
  assert.match(app, /connectVSCodeBtn[^\n]*openConnectTool\('vscodeIntegration'\)/);
});

test('conversation actions use an accessible settings glyph and a menu role', () => {
  const app = read('src/app.js');
  const css = read('src/connect-sidebar-refinement.css');
  assert.match(app, /menu\.className='history-more'/);
  assert.match(app, /menu\.setAttribute\('aria-haspopup','menu'\)/);
  assert.match(app, /actions\.setAttribute\('role','menu'\)/);
  assert.match(app, /M5 7\.5h14M5 12h14M5 16\.5h14/);
  assert.match(css, /\.history-more:focus-visible\{outline:2px solid/);
});


test('empty chat message list is truly empty so the centered welcome state can render', () => {
  const html = read('src/index.html');
  const layout = read('src/chat-centered-layout.css');
  assert.match(html, /<div class="messages" id="messages"><\/div>/);
  assert.doesNotMatch(html, /<div class="messages" id="messages">\s+<\/div>/);
  assert.match(layout, /:has\(#messages:empty\)/);
});
