'use strict';

const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const failures = [];
const checks = [];
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = relative => fs.existsSync(path.join(root, relative));
const check = (name, pass, detail = '') => {
  checks.push({ name, pass: !!pass, detail });
  if (!pass) failures.push(`${name}${detail ? `: ${detail}` : ''}`);
};

let pkg;
try { pkg = JSON.parse(read('package.json')); }
catch (error) {
  console.error(`Release preflight: package.json is unreadable: ${error.message}`);
  process.exit(1);
}
const version = String(pkg.version || '');
check('Semantic version', /^\d+\.\d+\.\d+$/.test(version), version || 'missing');
check('Main entry exists', typeof pkg.main === 'string' && exists(pkg.main), pkg.main || 'missing');
check('Current changelog included', pkg.build?.files?.includes(`CHANGELOG-v${version}.md`) && exists(`CHANGELOG-v${version}.md`), `CHANGELOG-v${version}.md`);
check('Current audit included', pkg.build?.files?.includes(`AUDIT-CZ-v${version}.md`) && exists(`AUDIT-CZ-v${version}.md`), `AUDIT-CZ-v${version}.md`);
check('Source and integration trees included', pkg.build?.files?.includes('src/**/*') && pkg.build?.files?.includes('integrations/**/*'), 'src/**/*, integrations/**/*');
check('NSIS Windows target configured', pkg.build?.win?.target === 'nsis' && pkg.build?.nsis?.oneClick === false, 'Windows NSIS target must allow install-directory choice');
check('Electron stable dependency present', /^45\.1\.0$/.test(pkg.devDependencies?.electron || ''), pkg.devDependencies?.electron || 'missing');
check('Electron Builder dependency present', /^26\.18\.1$/.test(pkg.devDependencies?.['electron-builder'] || ''), pkg.devDependencies?.['electron-builder'] || 'missing');

const html = read('src/index.html');
const main = read('src/main.js');
const preload = read('src/preload.js');
const requiredFiles = [
  'src/index.html', 'src/style.css', 'src/desktop-refinement.css', 'src/app.js', 'src/main.js', 'src/preload.js',
  'src/icon.png', 'src/icon.ico', 'src/icon.svg', 'src/aurora-background.mp4',
  'src/unity-project-audit.js', 'src/godot-project-audit.js', 'src/capcut-project-audit.js', 'src/project-check-runner.js', 'src/roblox-plugin-manager.js', 'src/local-memory.js', 'src/local-memory-client.js', 'src/local-memory-backup.js', 'src/remote-providers.js', 'src/workspace-persistence.js',
  'src/agent-change-proposals.js', 'src/project-safety.js', 'src/project-mutation-gate.js', 'src/bounded-response.js', 'src/bounded-file.js', 'src/retry-policy.js', 'src/ollama-metrics.js', 'src/user-profile.js', 'scripts/check-syntax.js', 'scripts/runtime-smoke.js',
  'integrations/roblox-studio/AuroraBridge.plugin.lua', 'integrations/roblox-studio/README-CZ.md', 'integrations/unity/Editor/AuroraEditorBridge.cs'
];
check('Required runtime files exist', requiredFiles.every(exists), requiredFiles.filter(file => !exists(file)).join(', ') || `${requiredFiles.length} files`);
check('Local runtime smoke test exists', exists('scripts/runtime-smoke.js') && pkg.scripts?.['smoke:local'] === 'node scripts/runtime-smoke.js', 'npm run smoke:local');
const localMemoryModule=read('src/local-memory.js');
check('Optional local semantic memory is bounded and isolated', /MAX_MEMORIES\s*=\s*200/.test(localMemoryModule) && /selectRelevantMemories/.test(localMemoryModule) && /normalizeEmbedding/.test(localMemoryModule), 'bounded local embedding index');
check('Memory recall is restricted to local Ollama', /secureHandle\('memory:recall'/.test(main) && /payload\?\.provider!==['\"]ollama['\"]/.test(main) && /BASE = 'http:\/\/127\.0\.0\.1:11434'/.test(read('src/local-memory-client.js')) && /\/api\/embed/.test(read('src/local-memory-client.js')) && /\/api\/embeddings/.test(read('src/local-memory-client.js')), 'fixed loopback embedding endpoints including older Ollama compatibility');
check('Memory operations are exposed through allowlisted preload', /memoryStatus:/.test(preload) && /addMemory:/.test(preload) && /recallMemory:/.test(preload) && /exportMemory:/.test(preload) && /importMemory:/.test(preload), 'no direct filesystem access in renderer; explicit local backup import/export');
check('Local memory backup uses validated helper and atomic model rollback', /const LocalMemoryBackup = require\('.\/local-memory-backup'\)/.test(main) && /LocalMemoryBackup\.createBackup/.test(main) && /LocalMemoryBackup\.parseBackup/.test(main) && /LocalMemoryBackup\.mergeRecords/.test(main) && /obnovení původního nastavení modelu také selhalo/.test(main), 'validated JSON backup helper is wired into IPC');
check('Memory UI is explicit opt-in', /id=\"memoryEnabled\"/.test(html) && /checked/.test(html.match(/id=\"memoryEnabled\"[^>]*>/)?.[0] || '')===false && /Neposílají se do cloudu|neposílají do cloudu/i.test(html), 'off by default and local-only disclosure');
check('Local memory exposes explicit backup controls', /id="exportMemory"/.test(html) && /id="importMemory"/.test(html) && /memory:export/.test(preload) && /memory:import/.test(preload), 'notes have separate explicit JSON backup and restore controls');
check('Electron security defaults', /contextIsolation:\s*true/.test(main) && /nodeIntegration:\s*false/.test(main) && /sandbox:\s*true/.test(main) && /setWindowOpenHandler\(\(\)=>\(\{action:'deny'\}\)\)/.test(main), 'context isolation, sandbox, no node integration and denied popups');
check('IPC validates main frame', /function secureHandle\(/.test(main) && /event\.senderFrame!==event\.sender\.mainFrame/.test(main), 'trusted IPC origin guard');
check('Secret storage uses safeStorage', /safeStorage\.encryptString/.test(main) && /safeStorage\.decryptString/.test(main), 'encrypted provider API key storage');
check('No shell command execution is enabled by default', /Command execution is intentionally not enabled/.test(main), 'command execution remains disabled pending allowlist and approval UI');
check('CapCut audit is wired end-to-end', /CapCutProjectAudit\.auditCapCutProject\(approvedProjectRoot\)/.test(main) && /capcutAudit:\(\)=>ipcRenderer\.invoke\('project:capcut-audit'\)/.test(preload) && /id=\"auditCapCut\"/.test(html), 'read-only CapCut draft audit through secure IPC and preload');
check('Project check runner is wired with explicit approval', /ProjectCheckRunner\.runProjectCheck/.test(main) && /action\.executesProjectScripts/.test(main) && /runProjectCheck:/.test(preload) && /id=\"runProjectCheck\"/.test(html), 'fixed allowlist actions; script-running checks show native confirmation');
check('Roblox Bridge manager is wired with confirmation', /RobloxPluginManager\.install/.test(main) && /RobloxPluginManager\.uninstall/.test(main) && /installRobloxBridge:/.test(preload) && /id=\"installRobloxBridge\"/.test(html), 'plugin install/disable through explicit main-process confirmation');
check('Unity Editor Bridge uses approved file proposal', /project:unity-bridge-source/.test(main) && /unityBridgeSource:/.test(preload) && /id=\"installUnityBridge\"/.test(html) && /previewEdit/.test(read('src/app.js')), 'bundled bridge enters the existing review-and-approve edit flow');
check('Godot audit is wired through secure IPC', /GodotProjectAudit\.auditGodotProject/.test(main) && /godotAudit:\(\)=>ipcRenderer\.invoke\('project:godot-audit'\)/.test(preload) && /id="auditGodot"/.test(html), 'read-only Godot project audit is connected to UI');
check('VS Code opens only the approved project without shell', /secureHandle\('project:open-vscode'/.test(main) && /spawn\(executable, \['--new-window', root\]/.test(main) && /shell: false/.test(main) && /openVSCode:.*project:open-vscode/.test(preload) && /id="openVSCode"/.test(html), 'explicit editor path and selected project root');
const proposals = read('src/agent-change-proposals.js');
const mainSafeExts = new Set([...(main.match(/const SAFE_TEXT_EXTS = new Set\(\[([^\]]+)\]\)/)?.[1] || '').matchAll(/'([^']+)'/g)].map(match => match[1]));
const proposalExts = [...(proposals.match(/ALLOWED_EXTENSIONS = new Set\(\[([^\]]+)\]\)/)?.[1] || '').matchAll(/'([^']+)'/g)].map(match => match[1]);
check('Agent changesets share the main-process file allowlist', proposalExts.length > 0 && proposalExts.every(ext => mainSafeExts.has(ext)), 'all proposal extensions are also approved for secure writes');
check('Full proposal content is loaded before approval', /project:proposal-preview-file/.test(main) && /readProposalPreviewFile:payload=>ipcRenderer\.invoke\('project:proposal-preview-file',payload\)/.test(preload) && /approve\.disabled=loaded\.size!==files\.length/.test(read('src/app.js')), 'every file must be inspected before approve');
check('VS Code settings and entry point exist', exists('src/app.js') && /chooseVSCode/.test(html) && /id="vscodeStatus"/.test(html), 'editor selection and open-project controls');
check('Runtime smoke command is loopback-only', /ALLOWED_LOCAL_HOSTS/.test(read('scripts/runtime-smoke.js')) && /redirect: 'error'/.test(read('scripts/runtime-smoke.js')) && /npm run smoke:local/.test(read('README-CZ-START.md')), 'service check rejects non-loopback targets');

check('Preload uses allowlisted IPC methods', /contextBridge\.exposeInMainWorld\('aurora'/.test(preload) && !/exposeInMainWorld\(['"]require['"]/.test(preload), 'contextBridge only');

function localReference(raw) {
  const value = String(raw || '').trim();
  if (!value || value.startsWith('#') || /^(?:data:|https?:|mailto:|tel:|\/\/|javascript:)/i.test(value)) return null;
  return decodeURIComponent(value.split(/[?#]/)[0]);
}
const htmlRefs = [];
for (const match of html.matchAll(/\b(?:src|href)\s*=\s*["']([^"']+)["']/gi)) {
  const ref = localReference(match[1]);
  if (ref) htmlRefs.push(path.posix.normalize(ref.replace(/\\/g, '/')));
}
const missingHtmlRefs = [...new Set(htmlRefs)].filter(ref => !exists(path.join('src', ref)));
check('HTML local resources exist', missingHtmlRefs.length === 0, missingHtmlRefs.join(', ') || `${new Set(htmlRefs).size} local references`);

const stylesheetFiles = [...new Set(htmlRefs.filter(ref => ref.endsWith('.css')).map(ref => path.posix.join('src', ref)))];
const missingStylesheets = stylesheetFiles.filter(file => !exists(file));
check('All linked stylesheets exist', missingStylesheets.length === 0, missingStylesheets.join(', ') || `${stylesheetFiles.length} stylesheets`);
const cssRefs = [];
for (const cssFile of stylesheetFiles.filter(exists)) {
  const css = read(cssFile);
  for (const match of css.matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/gi)) {
    const ref = localReference(match[1]);
    if (ref) cssRefs.push({ cssFile, ref });
  }
}
const missingCssRefs = cssRefs.filter(({ cssFile, ref }) => !exists(path.join(path.dirname(cssFile), ref)));
check('CSS local resources exist', missingCssRefs.length === 0, missingCssRefs.map(item => `${item.cssFile} -> ${item.ref}`).join(', ') || `${cssRefs.length} local references across all linked stylesheets`);

const requiredModules = [...main.matchAll(/require\(['"](\.\/[^'"]+)['"]\)/g)].map(match => match[1]);
const missingModules = [...new Set(requiredModules)].filter(item => !exists(path.join('src', item.slice(2))) && !exists(path.join('src', `${item.slice(2)}.js`)) && !exists(path.join('src', item.slice(2), 'index.js')));
check('Main-process required modules exist', missingModules.length === 0, missingModules.join(', ') || `${new Set(requiredModules).size} modules`);

const appVersionReferences = [
  `Aurora-AI v${version}`,
  `Aurora-AI-Setup-${version}.exe`
];
const buildScript = read('BUILD-AURORA.bat');
check('Build script matches package version', new RegExp(`AURORA AI v${version.replace(/\./g, '\\.')} - FINAL BUILD`).test(buildScript) && buildScript.includes(appVersionReferences[1]), appVersionReferences.join(' / '));

for (const item of pkg.build?.files || []) {
  if (item.includes('*')) continue;
  check(`Packaged path exists (${item})`, exists(item), item);
}

const summary = {
  product: pkg.build?.productName || pkg.name,
  version,
  checks: checks.length,
  passed: checks.filter(item => item.pass).length,
  failed: failures.length,
  failures
};
console.log(`\n${summary.product} release preflight v${version}`);
console.log(`${summary.passed}/${summary.checks} checks passed`);
for (const item of checks) console.log(`${item.pass ? 'PASS' : 'FAIL'}  ${item.name}${item.detail ? ` — ${item.detail}` : ''}`);
if (failures.length) process.exitCode = 1;
else console.log('\nPreflight passed. This is static release validation, not a Windows installer or live-provider test.');
