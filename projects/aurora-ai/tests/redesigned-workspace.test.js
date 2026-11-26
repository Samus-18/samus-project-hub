const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('streaming chat preserves the reader position when not at the bottom', () => {
  const app = read('src/app.js');
  assert.match(app, /const wasNearBottom=\(box\.scrollHeight-box\.scrollTop-box\.clientHeight\)<110/);
  assert.match(app, /if\(wasNearBottom\)box\.scrollTop=box\.scrollHeight;else box\.scrollTop=previousScrollTop/);
});

test('the primary sidebar is simplified and the default workspace is clickable', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const navStart = html.indexOf('id="mainNavigation"');
  const nav = html.slice(html.lastIndexOf('<nav', navStart), html.indexOf('</nav>', navStart));
  assert.match(nav, /data-view="models"[\s\S]*data-view="tools"[\s\S]*data-view="images"[\s\S]*data-view="settings"/);
  assert.match(html, /id="chatModeToggle"[\s\S]*id="workModeToggle"/);
  assert.match(html, /id="workProjectControl"[\s\S]*id="workProjectSelect"/);
  assert.doesNotMatch(html, /id="sidebarHomeMode"|id="sidebarCodeMode"/);
  assert.match(html, /id="defaultWorkspace"/);
  assert.match(html, /id="pinnedChats"[\s\S]*id="sidebarProjectList"/);
  assert.match(app, /\$\('defaultWorkspace'\)\?\.addEventListener\('click',\(\)=>setView\('projects'\)\)/);
  assert.match(app, /\$\('chatModeToggle'\)\?\.addEventListener\('click',\(\)=>\{if\(setMode\('chat'\)\)setView\('chat'\)\}/);
  assert.match(app, /\$\('workModeToggle'\)\?\.addEventListener\('click',\(\)=>\{if\(setMode\('work'\)\)setView\('chat'\)\}/);
});

test('Ollama has safe start/stop controls with a narrow preload bridge', () => {
  const html=read('src/index.html'),app=read('src/app.js'),main=read('src/main.js'),preload=read('src/preload.js');
  assert.match(html,/id="ollamaStartBtn"/);assert.match(html,/id="ollamaStopBtn"/);
  assert.match(preload,/startOllama:.*ollama:start-service/);assert.match(preload,/installOllama:.*ollama:install-service/);assert.match(preload,/stopOllama:.*ollama:stop-service/);
  assert.match(main,/secureHandle\('ollama:start-service'/);assert.match(main,/secureHandle\('ollama:install-service'/);assert.match(main,/secureHandle\('ollama:stop-service'/);assert.match(main,/spawn\('winget\.exe',\['install','--id','Ollama\.Ollama','--exact','--silent'/);assert.match(main,/shell:false/);
  assert.match(main,/if\(!child\)return \{ok:true,stopped:false/);
  assert.match(app,/ollamaStartBtn/);assert.match(app,/ollamaStopBtn/);assert.match(app,/confirm\('Ollama není nainstalovaná/);assert.match(app,/window\.aurora\.installOllama\(\)/);
});

test('downloaded model does not silently replace the explicitly selected model', () => {
  const app=read('src/app.js');
  assert.match(app,/aktivní výběr zůstává beze změny/ );
  assert.doesNotMatch(app,/cfg\.localModel!==local\.value/,'Auto fallback must not be saved as a manual role model');
  assert.match(app,/agentFallbackModel=state\.autoRoute\?routeModelForPrompt\(goal\)/);
  assert.match(app,/PREFERRED_MODEL_KEY='aurora\.preferredModel\.v1'/);
});

test('task router is linked before normal text chat and image routing stays prior', () => {
  const app=read('src/app.js');
  assert.ok(app.indexOf('const imageIntent=detectImageRequest(text)') < app.indexOf('const workIntent=window.AuroraIntent?.detectWorkRequest?.(text)'));
  assert.match(app,/async function routeIntoWorkMode\(goal\)/);
});


test('streaming updates only the active assistant body and preserves manual scroll position', () => {
  const app=read('src/app.js');
  assert.match(app,/function renderStreamingAssistantMessage\(message\)/);
  assert.match(app,/if\(wasNearBottom\)box\.scrollTop=box\.scrollHeight;else box\.scrollTop=previousScrollTop/);
  assert.match(app,/if\(state\.messages\.includes\(placeholder\)\)renderStreamingAssistantMessage\(placeholder\)/);
});

test('Code only advertises output formats that its safe changeset writer can actually create', () => {
  const html=read('src/index.html');
  const proposals=require('../src/agent-change-proposals');
  assert.ok(proposals.ALLOWED_EXTENSIONS.has('.csv'));
  assert.ok(proposals.ALLOWED_EXTENSIONS.has('.tsv'));
  assert.ok(proposals.ALLOWED_EXTENSIONS.has('.md'));
  assert.match(html,/CSV a TSV/);
  assert.match(html,/Office\/PDF binární export není/);
});


test('compact chat hides the welcome banner after the first message and keeps only Claude-like model and effort controls', () => {
  const css=read('src/aurora-reference-refit.css');
  const html=read('src/index.html');
  const app=read('src/app.js');
  assert.match(css,/\.welcome\.compact\{display:none!important/);
  assert.match(css,/#autoRoute,.auto-route-toggle,#responseSettingsBtn,#responseSettings\{display:none!important/);
  assert.match(html,/id=\"modelQuick\"[\s\S]*id=\"effortToggle\"/);
  assert.match(app,/function syncEffortPicker\(\)/);
});

test('streaming scroll position is measured before the growing message is re-rendered', () => {
  const app=read('src/app.js');
  const fn=app.slice(app.indexOf('function renderStreamingAssistantMessage'),app.indexOf('function routeModelForPrompt'));
  assert.ok(fn.indexOf('const wasNearBottom=') < fn.indexOf('content.innerHTML=renderSafeMarkdown'));
  assert.match(fn,/if\(wasNearBottom\)box\.scrollTop=box\.scrollHeight;else box\.scrollTop=previousScrollTop/);
});

test('Extra effort survives settings save, workspace import, and persisted workspace normalization', () => {
  const app=read('src/app.js'),main=read('src/main.js');
  assert.match(app,/includes\(profile\)\)state\.performanceProfile=profile/);
  assert.match(app,/includes\(incoming\.performanceProfile\)\)state\.performanceProfile=incoming\.performanceProfile/);
  assert.match(main,/performanceProfile:\['adaptive','quick','balanced','deep','extra','manual'\]/);
});


test('existing installations get a fresh one-time migration that restores the cinematic video background', () => {
  const prefs=read('src/ui-preferences.js');
  assert.match(prefs,/aurora\.ui\.video-background\.migrated\.v4/);
  assert.match(prefs,/backgroundMotion:true,backgroundIntensity:Math\.max\(prefs\.backgroundIntensity,0\.58\)/);
  assert.match(prefs,/video\.play\(\)/);
});
