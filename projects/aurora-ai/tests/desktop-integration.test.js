'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');
const has = (text, pattern, message) => assert.match(text, pattern, message);

test('v0.8.6 remains one integrated desktop release with a small runtime package whitelist', () => {
  const pkg = JSON.parse(read('package.json'));
  assert.equal(pkg.version, '0.8.6');
  assert.equal(pkg.main, 'src/main.js');
  assert.ok(pkg.build.files.includes('CHANGELOG-v0.8.6.md'));
  assert.ok(pkg.build.files.includes('AUDIT-CZ-v0.8.6.md'));
  assert.ok(pkg.build.files.includes('RELEASE-CHECKLIST-CZ.md'));
  assert.ok(pkg.build.files.includes('src/**/*'));
  assert.ok(pkg.build.files.includes('integrations/**/*'));
  assert.ok(!pkg.build.files.includes('CHANGELOG-v0.7.5.md'), 'historical changelogs should stay out of installed app');
  assert.ok(!pkg.build.files.includes('AUDIT-CZ-v0.7.5.md'), 'historical audits should stay out of installed app');
  const html = read('src/index.html');
  has(html, /href="style\.css"/);
  has(html, /href="desktop-refinement\.css"/);
  has(html, /id="auroraBackgroundVideo"[\s\S]{0,200}src="aurora-background\.mp4"|id="auroraBackgroundVideo"[\s\S]{0,200}<source src="aurora-background\.mp4"/);
  assert.ok(fs.statSync(path.join(root, 'src/aurora-background.mp4')).size > 1_000_000);
});

test('desktop refinement uses the provided video scene and suppresses obsolete abstract ribbons', () => {
  const css = read('src/desktop-refinement.css');
  has(css, /animation: aurora-scene-drift 84s/);
  has(css, /prefers-reduced-motion/);
  has(css, /body\.reduce-motion \.aurora-backdrop video/);
  has(css, /\.ambient, \.ambient::before, \.ambient::after[\s\S]{0,200}display: none!important/);
  has(css, /-apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI Variable"/);
  has(css, /\.main\.dashboard-hidden \.dashboard-rail \{ display:none!important/);
});

test('settings are categorized and only the selected category is visible at first paint', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  for (const category of ['appearance', 'personalization', 'generation', 'sound', 'providers', 'data', 'safety']) {
    has(html, new RegExp(`data-settings-category="${category}"`));
  }
  for (const category of ['appearance', 'personalization', 'generation', 'sound', 'providers', 'data', 'safety']) {
    has(html, new RegExp(`data-settings-filter="${category}"`));
  }
  assert.doesNotMatch(html, /data-settings-filter="all"/, 'settings should not expose an unstructured all-categories view');
  has(html, /data-settings-category="generation"[^>]*\shidden/);
  has(html, /data-settings-category="providers"[^>]*\shidden/);
  has(app, /function setSettingsCategory\(category\)/);
  has(app, /card\.hidden=selected!=='all'&&card\.dataset\.settingsCategory!==selected/);
  has(html, /data-settings-category="personalization"/);
  has(html, /data-settings-category="sound"/);
});

test('advanced setting controls are validated, persisted, mirrored to chat, and put inference in manual mode', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  for (const id of ['generationLanguage','settingsTemperature','settingsTopP','settingsTopK','settingsRepeatPenalty','settingsSeed','settingsContextSize','settingsMaxTokens','settingsThreadCount','systemPrompt','stopSequences','keepAlive','streamResponses']) {
    has(html, new RegExp(`id="${id}"`));
    has(app, new RegExp(`['"]${id}['"]`));
  }
  has(app, /response_language:\['auto','cs','en'\]\.includes\(savedPrefs\.response_language\)/);
  has(app, /p\.response_language=\['auto','cs','en'\]\.includes\(\$\('generationLanguage'\)\?\.value\)/);
  has(app, /state\.performanceProfile='manual'/);
  has(app, /set\('settingsContextSize',p\.num_ctx\)/);
});

test('response language reaches normal chat and each Agent Studio role', () => {
  const app = read('src/app.js');
  has(app, /response_language==='cs'\?\[\{role:'system',content:'ODPOVÍDEJ ČESKY/);
  has(app, /response_language==='en'\?\[\{role:'system',content:'RESPOND IN ENGLISH/);
  has(app, /const languageInstruction=state\.preferences\.response_language==='cs'/);
  has(app, /Respond in English unless the user explicitly requests another language/);
});

test('localized settings categories and generation options have English labels', () => {
  const ui = read('src/ui-preferences.js');
  for (const key of ['Vzhled', 'Model a generování', 'Poskytovatelé', 'Data a paměť', 'Bezpečnost', 'Všechno', 'Pokročilé parametry modelu', 'Jazyk odpovědí']) {
    has(ui, new RegExp(`'${key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}'`));
  }
  has(ui, /'Pokročilé parametry modelu':'Advanced model parameters'/);
  has(ui, /'Jazyk odpovědí':'Response language'/);
});

test('desktop dependencies target supported Electron and stable electron-builder majors', () => {
  const pkg = JSON.parse(read('package.json'));
  assert.equal(pkg.devDependencies.electron, '45.1.0');
  assert.equal(pkg.devDependencies['electron-builder'], '26.18.1');
  assert.doesNotMatch(pkg.devDependencies.electron, /^\^34\./);
  assert.doesNotMatch(pkg.devDependencies['electron-builder'], /^\^25\./);
});

test('Windows builder and docs agree on the integrated release version', () => {
  const bat = read('BUILD-AURORA.bat');
  const changelog = read('CHANGELOG-v0.8.6.md');
  const audit = read('AUDIT-CZ-v0.8.6.md');
  has(bat, /AURORA AI v0\.8\.6 - FINAL BUILD/);
  has(bat, /Aurora-AI-Setup-0\.8\.6\.exe/);
  has(changelog, /0\.8\.6/);
  has(audit, /Windows/);
});

test('navigation, installed-model picker, conversation archive and simple agent mode are wired', () => {
  const html = read('src/index.html');
  const app = read('src/app.js');
  const css = read('src/desktop-refinement.css');
  const main = read('src/main.js');
  has(html, /Stažené lokální modely[\s\S]*?id="quickModelMenu"|id="quickModelMenu"[\s\S]*?Stažené lokální modely/);
  has(html, /data-history-filter="recent"[\s\S]*?data-history-filter="archived"/);
  has(html, /id="pinnedChats"[\s\S]*?id="sidebarProjectList"[\s\S]*?id="viewAllProjects"/);
  has(html, /id="agentComplexity"[\s\S]*?value="simple"[\s\S]*?value="advanced"/);
  has(app, /function renderQuickModels\(\)/);
  has(app, /state\.agentComplexity==='advanced'\|\|\['planner','implementer'\]/);
  has(app, /function renderArchiveAndProjects\(\)/);
  has(css, /\.sidebar\.collapsed\{[\s\S]*?transition/);
  has(main, /function isValidOllamaModelName/);
  has(main, /!name\.includes\('\.\.'\)/);
});


test('portable workspace backup remembers folder re-selection without exporting the raw path', () => {
  const app = fs.readFileSync(path.join(root, 'src/app.js'), 'utf8');
  const main = fs.readFileSync(path.join(root, 'src/main.js'), 'utf8');
  const persistence = fs.readFileSync(path.join(root, 'src/workspace-persistence.js'), 'utf8');
  assert.match(app, /folder:state\.folder,projectFolder:state\.projectFolder/);
  assert.match(main, /folderSelected:!!parsed\.folder,projectFolderSelected:!!parsed\.projectFolder/);
  assert.match(app, /if\(incoming\.folderSelected\).*?Vyber složku znovu pro nové oprávnění/s);
  assert.match(app, /if\(incoming\.projectFolderSelected\).*?Vyber složku znovu pro nové oprávnění/s);
  assert.match(persistence, /folder: source\.folder \? 'selected' : null/);
  assert.match(persistence, /projectFolder: source\.projectFolder \? 'selected' : null/);
});


test('workspace import applies the same 100-chat storage cap while preserving the active conversation', () => {
  const app = read('src/app.js');
  assert.match(app, /const boundedWorkspace=window\.AuroraWorkspacePersistence\?\.prepareWorkspace\?\.\(\{conversations:state\.conversations,activeConversationId:state\.activeConversationId\}\)/);
  assert.match(app, /state\.conversations=boundedWorkspace\.payload\.conversations;state\.activeConversationId=boundedWorkspace\.payload\.activeConversationId/);
  const persistence = read('src/workspace-persistence.js');
  assert.match(persistence, /maxConversations: 100/);
  assert.match(persistence, /Never orphan the active conversation when a large export hits its last-resort size ceiling/);
});

test('web search HTML is byte-bounded before parsing third-party response content', () => {
  const main = read('src/main.js');
  assert.match(main, /const html=await readTextBounded\(response,1000000\);\s*parseResults\(html,endpoint\.source\);/);
});

test('bounded HTTP reader cancels the response when the byte ceiling is reached exactly', () => {
  const bounded = read('src/bounded-response.js');
  assert.match(bounded, /chunk\.byteLength < value\.byteLength \\|\| total >= limit/);
  assert.match(bounded, /await reader\.cancel\(\)/);
});

test('external research identifies the current app version instead of sending stale user-agent metadata', () => {
  const main = read('src/main.js');
  assert.ok(main.includes("'User-Agent':`Mozilla/5.0 AuroraAI/${app.getVersion()}`"));
  assert.doesNotMatch(main, /AuroraAI\/0\.7\.3/);
});

test('legacy localStorage restore normalizes the whole snapshot and never trusts arbitrary image URLs', () => {
  const app = read('src/app.js');
  assert.ok(app.includes("const raw=JSON.parse(localStorage.getItem(STORAGE_KEY)||'{}');const d=window.AuroraWorkspacePersistence?.prepareWorkspace?.(raw)?.payload||raw;"), 'legacy state uses the shared workspace normalizer before reading the active chat');
  has(app, /messages:c\.messages\.slice\(-200\)\.map\(m=>window\.AuroraWorkspacePersistence\?\.compactMessage\?\.\(m,50000\)/);
  assert.ok(app.includes('.filter(Boolean)})).sort((a,b)=>(b.updatedAt||0)-(a.updatedAt||0))'));
  const persistence = read('src/workspace-persistence.js');
  assert.ok(persistence.includes('127\\.0\\.0\\.1:8188\\/view\\?'));
  has(persistence, /if \(message\.pending === true\) out\.pending = true/);
});

test('sidebar collapse keeps recent/completed/project filters and chat list visible at narrow widths', () => {
  const html = read('src/index.html');
  const prefs = read('src/ui-preferences.js');
  const css = read('src/connect-sidebar-refinement.css');
  has(html, /<button[^>]*aria-expanded="true"[^>]*id="sidebarToggle"|<button[^>]*id="sidebarToggle"[^>]*aria-expanded="true"/);
  has(html, /data-history-filter="recent"[\s\S]*?data-history-filter="archived"/);
  has(html, /id="pinnedChats"[\s\S]*?id="sidebarProjectList"[\s\S]*?id="viewAllProjects"/);
  has(prefs, /classList\.remove\('navigation-hidden'\)/);
  has(css, /\.app\.sidebar-collapsed\{grid-template-columns:0 minmax\(0,1fr\)!important;?\}/);
  has(css, /@media\(max-width:980px\)[\s\S]*?\.topbar\.top-actions|@media\(max-width:980px\)/);
  has(css, /\.sidebar-bottom \.status-line/);
});
