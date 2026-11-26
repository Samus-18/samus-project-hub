'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('reference-style UI keeps the original animated background available in every theme', () => {
  const html = read('src/index.html');
  const prefs = read('src/ui-preferences.js');
  const css = read('src/aurora-reference-refit.css');
  assert.match(html, /aurora-reference-refit\.css/);
  assert.match(html, /<video[^>]*id="auroraBackgroundVideo"[^>]*autoplay[^>]*loop[^>]*muted|<video[^>]*autoplay[^>]*id="auroraBackgroundVideo"[^>]*loop[^>]*muted/);
  assert.match(prefs, /const active=prefs\.backgroundMotion\s*&&\s*!document\.hidden/);
  assert.match(prefs, /VIDEO_MIGRATION_KEY/);
  assert.match(prefs, /aurora\.ui\.video-background\.migrated\.v4/);
  assert.match(prefs, /backgroundMotion:true,backgroundIntensity:Math\.max\(prefs\.backgroundIntensity,0\.58\)/);
  assert.match(html, /poster=\"aurora-background-poster\.jpg\"[^>]*preload=\"auto\"|preload=\"auto\"[^>]*poster=\"aurora-background-poster\.jpg\"/);
  assert.match(css, /background:#07101e url\(\"aurora-background-poster\.jpg\"\) center center \/ cover no-repeat!important/);
  assert.match(prefs, /backgroundMotion:true/);
  assert.doesNotMatch(prefs, /prefs\.theme\s*===\s*['"]aurora['"]/);
  assert.match(css, /body\[data-theme="graphite"\] \.aurora-backdrop video/);
});

test('chat history suppresses the welcome hero while keeping the messages area scrollable', () => {
  const css = read('src/aurora-reference-refit.css');
  const app = read('src/app.js');
  assert.match(css, /\.chatview:has\(#messages:not\(:empty\)\) \.welcome\s*\{\s*display:none!important/);
  assert.match(css, /\.messages\s*\{[^}]*overflow-y:auto/s);
  assert.match(app, /if\(wasNearBottom\)box\.scrollTop=box\.scrollHeight;else box\.scrollTop=previousScrollTop/);
});

test('Chat and Work share the exact same chat shell and Work only adds project context', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const css = read('src/work-chat-unified.css');
  assert.match(html, /id="chatModeToggle"[\s\S]*id="workModeToggle"/);
  assert.match(html, /id="chatView"[\s\S]*id="messages"[\s\S]*id="prompt"/);
  assert.match(html, /id="workProjectControl"[\s\S]*id="workProjectSelect"/);
  assert.doesNotMatch(html, /id="sidebarHomeMode"|id="sidebarCodeMode"/);
  assert.match(app, /if\(view==='agent'\)\{setMode\('work'\);view='chat'\}/);
  assert.match(app, /async function routeIntoWorkMode\(goal\)[\s\S]*state\.messages\.push\(user,progress\)/);
  assert.match(css, /#agentView\{display:none!important\}/);
  assert.match(css, /\.main\[data-mode=\"work\"\] \.work-project-control\{display:flex\}/);
});

test('local model menu is built from the deduplicated installed Ollama model list only', () => {
  const app = read('src/app.js');
  assert.match(app, /state\.models=\[\.\.\.new Map\(/);
  assert.match(app, /function renderQuickModels\(\)[\s\S]*?const installed=\[\.\.\.new Map\(\(Array\.isArray\(state\.models\)/);
  assert.match(app, /function selectAutomaticModel\(\)[\s\S]*?state\.autoRoute=true/);
  assert.match(app, /function selectModel\(name\)[\s\S]*?state\.autoRoute=false/);
  assert.match(app, /automatic:!!state\.autoRoute/);
  assert.match(app, /async function pullModel\([\s\S]*?aktivní výběr zůstává beze změny/);
  assert.match(app, /function renderQuickModels\([\s\S]*?provider-menu-row[\s\S]*?Nainstalované modely \(\$\{installed\.length\}\)/);
  assert.match(app, /label\.textContent=state\.autoRoute\?'Auto'/);
  assert.match(app, /names=\{openai:'OpenAI · API',anthropic:'Claude · API',gemini:'Gemini · API'\}/);
  assert.match(app, /setCoreState\('ready',`PŘIPRAVENA · \${name}`\)/);
  assert.match(app, /state\.models=\[\.\.\.new Map\(/);
});

test('composer provides the Claude/ChatGPT-style effort choices without exposing tuning controls', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const css = read('src/aurora-reference-refit.css');
  for (const choice of ['adaptive','quick','balanced','deep','extra']) assert.match(html, new RegExp(`data-effort="${choice}"`));
  assert.match(app, /function setEffort\(mode\)/);
  assert.match(css, /\.composer-tools>\.provider-select-label,\.auto-route-toggle,#responseSettingsBtn,#responseSettings\{display:none!important\}/);
  assert.match(css, /#quickModelMenu \.provider-select-label\{display:flex!important/);
  assert.match(html, /data-view="images"[^>]*>[\s\S]*?<span>\s*Galerie\s*<\/span>/);
  assert.match(html, /<section[^>]*id="imagesView"[^>]*class="view subview hidden"|<section[^>]*class="view subview hidden"[^>]*id="imagesView"/);
  assert.match(app, /document\.querySelectorAll\('\[data-view\]'\)\.forEach\(button=>[\s\S]*?setView\(button\.dataset\.view\)/);
  assert.match(read('src/ui-preferences.js'), /'Galerie':'Gallery'/);
  assert.match(css, /#mainNavigation \.nav\[data-view="images"\]\{order:4!important;display:flex!important\}/);
  assert.match(css, /opacity:var\(--aurora-video-opacity,\.62\)!important;visibility:visible!important;display:block!important/);
  assert.doesNotMatch(css, /opacity:max\(var\(--aurora-video-opacity/);
});
