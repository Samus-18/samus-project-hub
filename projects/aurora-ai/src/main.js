const { app, BrowserWindow, ipcMain, shell, dialog, safeStorage } = require('electron');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const dns = require('node:dns/promises');
const net = require('node:net');
const { spawn } = require('node:child_process');
const RemoteProviders = require('./remote-providers');
const GenerationOptions = require('./generation-options');
const WorkspacePersistence = require('./workspace-persistence');
const ComfyWorkflow = require('./comfy-workflow');
const ProjectSafety = require('./project-safety');
const ProjectSpaces = require('./project-spaces');
const NavigationSafety = require('./navigation-safety');
const ProjectMutationGate = require('./project-mutation-gate');
const BoundedResponse = require('./bounded-response');
const WorkflowIntegrity = require('./workflow-integrity');
const BoundedFile = require('./bounded-file');
const GodotProjectAudit = require('./godot-project-audit');
const UnityProjectAudit = require('./unity-project-audit');
const LocalMemory = require('./local-memory');
const LocalMemoryClient = require('./local-memory-client');
const LocalMemoryBackup = require('./local-memory-backup');
const CapCutProjectAudit = require('./capcut-project-audit');
const ProjectCheckRunner = require('./project-check-runner');
const RobloxPluginManager = require('./roblox-plugin-manager');
const RetryPolicy = require('./retry-policy');
const OllamaMetrics = require('./ollama-metrics');
const HostCompatibility = require('./compatibility-audit');
const REMOTE_PROTOCOLS = Object.freeze(['openai','anthropic','gemini']);
function isValidOllamaModelName(name){return typeof name==='string'&&name.length<=120&&/^[a-zA-Z0-9._-]+(?:\/[a-zA-Z0-9._-]+)*(?::[a-zA-Z0-9._-]+)?$/.test(name)&&!name.includes('..')&&!name.includes('//')&&!name.split('/').some(part=>part==='.'||part==='..')&&!name.endsWith('/');}
function requireRemoteProtocol(value, fallback='openai') {
 const protocol=value===undefined||value===null||value===''?fallback:value;
 if(!REMOTE_PROTOCOLS.includes(protocol))throw new Error(`Neznámý API protokol: ${String(protocol).slice(0,80)}`);
 return protocol;
}
function migrateRemoteModel(protocol, model, fallback) {
 const value=typeof model==='string'&&RemoteProviders.isValidModelName(model)?model:fallback;
 if(protocol==='anthropic'&&['claude-3-5-haiku-latest','claude-3-5-haiku-20241022'].includes(value))return RemoteProviders.DEFAULTS.anthropic.model;
 if(protocol==='gemini'&&value==='gemini-2.0-flash')return RemoteProviders.DEFAULTS.gemini.model;
 return value;
}

// Validate every privileged IPC request against the local app's main frame.
function secureHandle(channel, handler) {
 ipcMain.handle(channel, (event, ...args) => {
  const frame = event.senderFrame;
  if (!frame || frame !== event.sender.mainFrame) throw new Error('IPC odmítnuto: neplatný renderer.');
  let pathname = '';
  try { const url = new URL(frame.url); pathname = url.protocol === 'file:' ? require('node:url').fileURLToPath(url) : ''; } catch {}
  if (pathname !== path.join(__dirname, 'index.html')) throw new Error('IPC odmítnuto: neznámý původ.');
  return handler(event, ...args);
 });
}
let mainWindow=null;
let managedOllamaProcess=null;
let managedOllamaExitCode=null;
let imageSetupProcess=null;
function createWindow(){
 const win=new BrowserWindow({width:1440,height:900,minWidth:960,minHeight:620,backgroundColor:'#070B13',autoHideMenuBar:true,title:'Aurora AI',icon:path.join(__dirname,'icon.png'),webPreferences:{preload:path.join(__dirname,'preload.js'),contextIsolation:true,nodeIntegration:false,sandbox:true,webviewTag:false}});
 mainWindow=win;
 win.webContents.setWindowOpenHandler(()=>({action:'deny'}));
 const entryPath=path.join(__dirname,'index.html');
 win.webContents.on('will-navigate',(event,url)=>{if(!NavigationSafety.isAllowedAppNavigation(url,entryPath))event.preventDefault()});
 win.loadFile(entryPath);
}
app.whenReady().then(async()=>{await loadPluginSettings();await loadSavedComfyWorkflow();createWindow();app.on('activate',()=>{if(BrowserWindow.getAllWindows().length===0)createWindow()})});
// Defense-in-depth: every IPC request must originate from this app's main frame.
const registerIpcHandler=ipcMain.handle.bind(ipcMain);
ipcMain.handle=(channel,handler)=>registerIpcHandler(channel,(event,...args)=>{
 if(!mainWindow||mainWindow.isDestroyed()||event.sender!==mainWindow.webContents||event.senderFrame!==event.sender.mainFrame)throw new Error('IPC request rejected: untrusted renderer.');
 return handler(event,...args);
});
let approvedProjectRoot = null;
const SAFE_TEXT_EXTS = new Set(['.txt','.md','.json','.csv','.tsv','.js','.mjs','.cjs','.ts','.jsx','.tsx','.css','.scss','.html','.xml','.yaml','.yml','.toml','.cs','.shader','.compute','.asmdef','.py','.gd','.tscn','.tres','.godot','.lua','.rs','.go','.java','.kt','.swift','.c','.h','.cpp','.hpp','.sql','.ini','.cfg','.gitignore','.csproj','.sln','.props','.targets','.bat','.ps1','.log']);
const IGNORE_DIRS = new Set(['.git','node_modules','Library','Temp','Obj','Build','Builds','.vs','.idea','PackagesCache']);
const BUILTIN_PLUGINS=[
 {id:'webResearch',title:'Internetový výzkum',description:'DuckDuckGo vyhledávání a načítání vybraných HTTPS zdrojů.',defaultEnabled:true},
 {id:'imageGeneration',title:'Generování obrázků',description:'Generování přes lokální ComfyUI API.',defaultEnabled:true},
 {id:'projectRead',title:'Čtení projektů',description:'Skenování a čtení povolených textových souborů.',defaultEnabled:true},
 {id:'projectWrites',title:'Zápisy do projektů',description:'Příprava a zápis změn pouze po výslovném schválení.',defaultEnabled:true}
];
let pluginSettings={};
const pluginSettingsPath=()=>path.join(app.getPath('userData'),'plugins.json');
async function loadPluginSettings(){try{const parsed=JSON.parse(await fs.readFile(pluginSettingsPath(),'utf8'));pluginSettings=parsed&&typeof parsed==='object'?parsed:{}}catch{pluginSettings={}}}
function pluginEnabled(id){const item=BUILTIN_PLUGINS.find(p=>p.id===id);return !!item&&(pluginSettings[id]===undefined?item.defaultEnabled:pluginSettings[id]===true)}
function requirePlugin(id){if(!pluginEnabled(id))throw new Error(`Integrace „${BUILTIN_PLUGINS.find(p=>p.id===id)?.title||id}“ je vypnutá. Povol ji v registru nástrojů.`)}

// Remote providers are stored independently so Agent Studio can mix local and remote models per role.
// API keys remain encrypted by Electron safeStorage and are never returned to the renderer.
const providerConfigPath=()=>path.join(app.getPath('userData'),'provider-config.json'); // legacy v0.6.x config
const providerSecretPath=()=>path.join(app.getPath('userData'),'provider-api.secret'); // legacy v0.6.x secret
const providerConfigPathFor=protocol=>path.join(app.getPath('userData'),`provider-config.${RemoteProviders.normalizeProtocol(protocol)}.json`);
const providerSecretPathFor=protocol=>path.join(app.getPath('userData'),`provider-api.${RemoteProviders.normalizeProtocol(protocol)}.secret`);
async function readLegacyProviderConfig(){try{const c=JSON.parse(await fs.readFile(providerConfigPath(),'utf8'));const protocol=requireRemoteProtocol(c.protocol, '');const defaults=RemoteProviders.DEFAULTS[protocol];const baseUrl=String(c.baseUrl||defaults.baseUrl).replace(/\/+$/,'');const u=new URL(baseUrl);if(u.protocol!=='https:'||u.username||u.password||u.hash||u.search)throw new Error('invalid endpoint');return {protocol,model:migrateRemoteModel(protocol,c.model,defaults.model),baseUrl,label:String(c.label||protocol).slice(0,60)}}catch{return null}}
async function readProviderConfig(protocol){
 protocol=requireRemoteProtocol(protocol);const defaults=RemoteProviders.DEFAULTS[protocol];
 try{const c=JSON.parse(await fs.readFile(providerConfigPathFor(protocol),'utf8'));const baseUrl=String(c.baseUrl||defaults.baseUrl).replace(/\/+$/,'');const u=new URL(baseUrl);if(u.protocol!=='https:'||u.username||u.password||u.hash||u.search)throw new Error('invalid endpoint');return {protocol,model:migrateRemoteModel(protocol,c.model,defaults.model),baseUrl,label:String(c.label||protocol).slice(0,60),configuredFile:true}}catch{}
 const legacy=await readLegacyProviderConfig();if(legacy?.protocol===protocol)return {...legacy,configuredFile:false};
 return {protocol,model:defaults.model,baseUrl:defaults.baseUrl,label:({openai:'OpenAI',anthropic:'Anthropic Claude',gemini:'Google Gemini'})[protocol],configuredFile:false};
}
async function readProviderKey(protocol){
 protocol=RemoteProviders.normalizeProtocol(protocol);
 if(!safeStorage.isEncryptionAvailable())throw new Error('Systémové šifrování není dostupné; API klíč nebyl načten.');
 try{return safeStorage.decryptString(await fs.readFile(providerSecretPathFor(protocol)))}catch{}
 // Migrate older single-provider installs on read, but only for the protocol their legacy config names.
 const legacy=await readLegacyProviderConfig();if(legacy?.protocol!==protocol)return '';
 try{return safeStorage.decryptString(await fs.readFile(providerSecretPath()))}catch{return ''}
}
async function readOpenAIKey(){const legacy=await readLegacyProviderConfig();return readProviderKey(legacy?.protocol||'openai')}
async function readOpenAIConfig(){const legacy=await readLegacyProviderConfig();return readProviderConfig(legacy?.protocol||'openai')}
secureHandle('provider:openai-status',async(_event,requestedProtocol)=>{
 const legacy=await readLegacyProviderConfig();const protocol=requestedProtocol!==undefined&&requestedProtocol!==null&&requestedProtocol!==''?requireRemoteProtocol(requestedProtocol):(legacy?.protocol||'openai');
 const config=await readProviderConfig(protocol);return {configured:!!(await readProviderKey(protocol).catch(()=>'')),...config,encryptionAvailable:safeStorage.isEncryptionAvailable()};
});
secureHandle('provider:openai-save',async(_event,payload)=>{
 const protocol=requireRemoteProtocol(payload?.protocol);const defaults=RemoteProviders.DEFAULTS[protocol];const key=String(payload?.apiKey||'').trim();const model=String(payload?.model||defaults.model).trim();const label=String(payload?.label||protocol).trim().slice(0,60);const baseUrl=String(payload?.baseUrl||defaults.baseUrl).trim().replace(/\/+$/,'');
 if(!RemoteProviders.isValidModelName(model))throw new Error('Název modelu obsahuje nepovolené znaky.');
 let endpoint;try{endpoint=new URL(baseUrl)}catch{throw new Error('Adresa API není platná URL.')}if(endpoint.protocol!=='https:'||endpoint.username||endpoint.password||endpoint.hash||endpoint.search)throw new Error('Vzdálený endpoint musí být HTTPS URL bez přihlašovacích údajů, query parametrů a fragmentu.');if(!label)throw new Error('Zadej název poskytovatele.');
 if(!safeStorage.isEncryptionAvailable())throw new Error('Systémové šifrování není dostupné. Klíč nelze bezpečně uložit.');
 // Preserve an existing key when only endpoint/model settings are edited, including legacy migration.
 const oldKey=key?'':await readProviderKey(protocol).catch(()=>'' );
 if(key.length&& (key.length<8||key.length>500))throw new Error('API klíč musí mít 8 až 500 znaků.');
 await fs.mkdir(app.getPath('userData'),{recursive:true});
 if(key||oldKey){await fs.writeFile(providerSecretPathFor(protocol),safeStorage.encryptString(key||oldKey),{mode:0o600});}
 await fs.writeFile(providerConfigPathFor(protocol),JSON.stringify({protocol,model,label,baseUrl},null,2),{mode:0o600});
 return {configured:!!(key||oldKey),protocol,model,label,baseUrl};
});
secureHandle('provider:openai-clear',async(_event,requestedProtocol)=>{
 const legacy=await readLegacyProviderConfig();const protocol=requestedProtocol!==undefined&&requestedProtocol!==null&&requestedProtocol!==''?requireRemoteProtocol(requestedProtocol):(legacy?.protocol||'openai');
 await fs.rm(providerSecretPathFor(protocol),{force:true});
 // Only remove the legacy key if it belongs to the requested provider. Its config is retained for compatibility.
 if(legacy?.protocol===protocol)await fs.rm(providerSecretPath(),{force:true});
 return {configured:false,protocol};
});
secureHandle('provider:openai-test',async(_event,requestedProtocol)=>{
 const legacy=await readLegacyProviderConfig();const protocol=requestedProtocol!==undefined&&requestedProtocol!==null&&requestedProtocol!==''?requireRemoteProtocol(requestedProtocol):(legacy?.protocol||'openai');
 const key=await readProviderKey(protocol);if(!key)throw new Error(`API klíč pro ${protocol} není nastaven. Otevři Nastavení → API a poskytovatelé.`);const cfg=await readProviderConfig(protocol);
 const request=RemoteProviders.buildRemoteTestRequest({protocol:cfg.protocol,baseUrl:cfg.baseUrl,model:cfg.model,apiKey:key});
 const testSignal=AbortSignal.timeout(12000);const response=await RetryPolicy.fetchWithRetry(request.url,{method:request.method,headers:request.headers,body:request.body?JSON.stringify(request.body):undefined,signal:testSignal},{signal:testSignal,maxAttempts:3});
 if(!response.ok)throw new Error(`${cfg.label} API ${response.status}: ${(await readTextBounded(response,1200)).slice(0,1000)}`);
 const raw=await readTextBounded(response,65536);let data;try{data=JSON.parse(raw)}catch{throw new Error(`${cfg.label} vrátilo neplatnou JSON odpověď z testu generování.`)}
 const generated=RemoteProviders.extractRemoteTestText(cfg.protocol,data).trim();if(!generated)throw new Error(`${cfg.label} vrátilo úspěšný HTTP status, ale žádný textový výstup modelu.`);
 return {ok:true,protocol:cfg.protocol,model:cfg.model,models:1,responsePreview:generated.slice(0,80)};
});

secureHandle('plugins:list',async()=>({plugins:BUILTIN_PLUGINS.map(p=>({...p,enabled:pluginEnabled(p.id)}))}));
secureHandle('plugins:set-enabled',(_event,payload)=>{const update=async()=>{const id=String(payload?.id||''),item=BUILTIN_PLUGINS.find(p=>p.id===id);if(!item)throw new Error('Neznámá integrace.');if(typeof payload.enabled!=='boolean')throw new Error('Stav integrace musí být true/false.');pluginSettings[id]=payload.enabled;await fs.mkdir(path.dirname(pluginSettingsPath()),{recursive:true});await fs.writeFile(pluginSettingsPath(),JSON.stringify(pluginSettings,null,2),'utf8');return {id,title:item.title,enabled:pluginEnabled(id)}};return payload?.id==='projectWrites'?withProjectMutationLock(null,update):update()});

secureHandle('project:choose', ()=>withProjectMutationLock(null,async()=>{const result=await dialog.showOpenDialog({properties:['openDirectory']});if(result.canceled||!result.filePaths[0])return null;approvedProjectRoot=await fs.realpath(result.filePaths[0]);return {path:approvedProjectRoot,name:path.basename(approvedProjectRoot)}}));
secureHandle('project:scan',async(_e,options={})=>{requirePlugin('projectRead');if(!approvedProjectRoot)throw new Error('Nejdřív vyber složku projektu.');const maxFiles=Math.min(Math.max(Number(options.maxFiles)||80,1),150);const files=[];async function walk(dir,depth=0){if(depth>5||files.length>=maxFiles)return;let entries;try{entries=await fs.readdir(dir,{withFileTypes:true})}catch{return}for(const ent of entries){if(files.length>=maxFiles)break;if(ent.name.startsWith('.')&&ent.name!=='.github')continue;const full=path.join(dir,ent.name);if(ent.isDirectory()){if(!IGNORE_DIRS.has(ent.name))await walk(full,depth+1)}else if(ent.isFile()&&SAFE_TEXT_EXTS.has(path.extname(ent.name).toLowerCase())){try{const st=await fs.stat(full);if(st.size>200000)continue;const rel=path.relative(approvedProjectRoot,full);files.push({path:rel,size:st.size})}catch{}}}}await walk(approvedProjectRoot);return {root:approvedProjectRoot,files,readOnly:true}});
secureHandle('project:read',async(_e,relativePath)=>{requirePlugin('projectRead');if(!approvedProjectRoot||typeof relativePath!=='string')throw new Error('Projekt není vybrán.');const target=path.resolve(approvedProjectRoot,relativePath);if(target!==approvedProjectRoot&&!target.startsWith(approvedProjectRoot+path.sep))throw new Error('Cesta je mimo vybranou složku.');const ext=path.extname(target).toLowerCase();if(!SAFE_TEXT_EXTS.has(ext))throw new Error('Tento typ souboru není povolen pro čtení.');const realRoot=await fs.realpath(approvedProjectRoot);const realTarget=await fs.realpath(target);if(realTarget!==realRoot&&!realTarget.startsWith(realRoot+path.sep))throw new Error('Odkaz na soubor míří mimo schválenou složku.');const st=await fs.stat(realTarget);if(!st.isFile()||st.size>200000)throw new Error('Soubor je příliš velký nebo není běžný soubor.');return {path:relativePath,content:await fs.readFile(realTarget,'utf8')}});

const pendingEdits = new Map();
const projectMutationGate = ProjectMutationGate.createSerialMutationGate();
const withProjectMutationLock = (proposalId, work) => projectMutationGate.run(proposalId, work);

// Project-space metadata stays in app userData. Absolute folder paths never leave
// the main process and are deliberately excluded from workspace export/import.
const projectSpacesFile = () => path.join(app.getPath('userData'), 'project-spaces.json');
async function readProjectSpacesStore() {
 try {
  const parsed = JSON.parse(await fs.readFile(projectSpacesFile(), 'utf8'));
  return ProjectSpaces.normalizeProjectSpaces(parsed?.projects);
 } catch { return []; }
}
async function writeProjectSpacesStore(projects) {
  const normalized = ProjectSpaces.normalizeProjectSpaces(projects);
  const file = projectSpacesFile();
  await fs.mkdir(path.dirname(file), { recursive: true });
  const temp = `${file}.${require('node:crypto').randomUUID()}.tmp`;
  await fs.writeFile(temp, JSON.stringify({ version: 1, projects: normalized }, null, 2), { encoding: 'utf8', mode: 0o600 });
  await fs.rename(temp, file);
  return normalized;
}
async function activateStoredProject(project) {
 if (!project) throw new Error('Projekt už není uložené.');
 const real = await fs.realpath(project.folderPath);
 const stat = await fs.stat(real);
 if (!stat.isDirectory()) throw new Error('Uložená projektová cesta už není složka.');
 approvedProjectRoot = real;
 const refreshed = { ...project, folderPath: real, folderName: path.basename(real), updatedAt: Date.now() };
 const list = (await readProjectSpacesStore()).map(item => item.id === project.id ? refreshed : item);
 await writeProjectSpacesStore(list);
 return { project: ProjectSpaces.toPublicProjectSpace(refreshed), path: real, name: refreshed.name };
}
secureHandle('project-spaces:list', async () => {
 const projects = await readProjectSpacesStore();
 return projects.map(ProjectSpaces.toPublicProjectSpace).filter(Boolean);
});
secureHandle('project-spaces:create', (_event, requestedName) => withProjectMutationLock(null, async () => {
 const result = await dialog.showOpenDialog({ title: 'Vyber složku nového projektu', properties: ['openDirectory', 'createDirectory'] });
 if (result.canceled || !result.filePaths[0]) return null;
 const real = await fs.realpath(result.filePaths[0]);
 const stat = await fs.stat(real);
 if (!stat.isDirectory()) throw new Error('Vybraná cesta není složka.');
 const current = await readProjectSpacesStore();
 const item = {
  id: `project_${require('node:crypto').randomUUID()}`,
  name: ProjectSpaces.normalizeName(requestedName, path.basename(real)),
  folderPath: real,
  folderName: path.basename(real),
  updatedAt: Date.now()
 };
 const added = ProjectSpaces.addProjectSpace(current, item);
 await writeProjectSpacesStore(added.list);
 const active = await activateStoredProject(added.project);
 return { ...active, created: added.created };
}));
secureHandle('project-spaces:select', (_event, id) => withProjectMutationLock(null, async () => {
 if (typeof id !== 'string' || !/^[a-zA-Z0-9_-]{8,100}$/.test(id)) throw new Error('Neplatný identifikátor projektu.');
 const projects = await readProjectSpacesStore();
 return activateStoredProject(projects.find(item => item.id === id));
}));
secureHandle('project-spaces:rename', (_event, payload) => withProjectMutationLock(null, async () => {
 if (typeof payload?.id !== 'string' || typeof payload?.name !== 'string') throw new Error('Chybí název projektu.');
 const changed = ProjectSpaces.renameProjectSpace(await readProjectSpacesStore(), payload.id, payload.name);
 await writeProjectSpacesStore(changed.list);
 return ProjectSpaces.toPublicProjectSpace(changed.project);
}));
secureHandle('project-spaces:remove', (_event, id) => withProjectMutationLock(null, async () => {
 const current = await readProjectSpacesStore();
 const removed = ProjectSpaces.removeProjectSpace(current, id);
 await writeProjectSpacesStore(removed.list);
 if (removed.project && approvedProjectRoot === removed.project.folderPath) approvedProjectRoot = null;
 return { removed: !!removed.project, id: String(id || '') };
}));
function safeProjectTarget(relativePath){
 if(!approvedProjectRoot)throw new Error('Nejdřív vyber schválenou projektovou složku a soubor.');
 return ProjectSafety.resolveProjectTarget(approvedProjectRoot,relativePath);
}

// Optional desktop-tool paths are stored locally and revalidated every time they are used.
const editorPathsFile = () => path.join(app.getPath('userData'), 'editor-paths.json');
async function readEditorPaths() {
 const result = { godot: null, unity: null, vscode: null };
 let parsed;
 try {
  const stat = await fs.stat(editorPathsFile());
  if (!stat.isFile() || stat.size > 16 * 1024) throw new Error('Konfigurace cest editorů je příliš velká.');
  parsed = JSON.parse(await fs.readFile(editorPathsFile(), 'utf8'));
 } catch (error) {
  if (error.code === 'ENOENT') return result;
  throw new Error('Konfiguraci cest editorů nelze přečíst. Oprav nebo přejmenuj editor-paths.json v profilu Aurory.');
 }
 for (const kind of ['godot', 'unity', 'vscode']) {
  const value = parsed?.[kind];
  if (typeof value !== 'string' || !path.isAbsolute(value)) continue;
  try {
   const stat = await fs.lstat(value);
   if (!stat.isFile() || stat.isSymbolicLink()) continue;
   const real = await fs.realpath(value);
   const same = process.platform === 'win32' ? path.resolve(real).toLowerCase() === path.resolve(value).toLowerCase() : path.resolve(real) === path.resolve(value);
   if (same) result[kind] = real;
  } catch {}
 }
 return result;
}
async function writeEditorPaths(paths) {
 const file = editorPathsFile();
 await fs.mkdir(path.dirname(file), { recursive: true });
 const temp = `${file}.tmp-${require('node:crypto').randomUUID()}`;
 await fs.writeFile(temp, JSON.stringify({ version: 1, ...paths }, null, 2), { encoding: 'utf8', flag: 'wx' });
 try { await fs.rename(temp, file); } catch (error) { await fs.rm(temp, { force: true }).catch(() => {}); throw error; }
}
secureHandle('project:editor-paths', async () => readEditorPaths());
secureHandle('project:choose-editor', async (_event, kind) => {
 if (!['godot', 'unity', 'vscode'].includes(kind)) throw new Error('Neznámý typ editoru.');
 const result = await dialog.showOpenDialog(mainWindow, {
  title: kind === 'godot' ? 'Vybrat Godot executable' : kind === 'unity' ? 'Vybrat Unity Editor executable' : 'Vybrat Visual Studio Code executable',
  properties: ['openFile'],
  filters: process.platform === 'win32' ? [{ name: 'Aplikace Windows', extensions: ['exe'] }, { name: 'Všechny soubory', extensions: ['*'] }] : [{ name: 'Aplikace', extensions: ['*'] }]
 });
 if (result.canceled || !result.filePaths?.[0]) return { cancelled: true };
 const selected = path.resolve(result.filePaths[0]);
 const stat = await fs.lstat(selected);
 if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('Vybraný editor není běžný soubor.');
 const real = await fs.realpath(selected);
 const same = process.platform === 'win32' ? real.toLowerCase() === selected.toLowerCase() : real === selected;
 if (!same) throw new Error('Spustitelný soubor editoru nesmí být symbolický odkaz.');
 const paths = await readEditorPaths(); paths[kind] = real; await writeEditorPaths(paths);
 return { cancelled: false, kind, path: real, name: path.basename(real) };
});
secureHandle('project:open-vscode', async () => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber projektovou složku.');
 const root = await fs.realpath(approvedProjectRoot);
 const paths = await readEditorPaths();
 const executable = paths.vscode;
 if (!executable) throw new Error('Nejdřív vyber Visual Studio Code executable.');
 const stat = await fs.lstat(executable);
 if (!stat.isFile() || stat.isSymbolicLink() || await fs.realpath(executable) !== executable) throw new Error('Vybraný Visual Studio Code executable už není bezpečný běžný soubor. Vyber ho znovu.');
 return new Promise((resolve, reject) => {
  let child;
  try { child = spawn(executable, ['--new-window', root], { cwd: root, env: ProjectCheckRunner.makeChildEnv(), shell: false, detached: true, windowsHide: true, stdio: 'ignore' }); }
  catch (error) { reject(new Error(`VS Code se nepodařilo spustit: ${String(error.message || error).slice(0, 220)}`)); return; }
  child.once('error', error => reject(new Error(`VS Code se nepodařilo spustit: ${String(error.message || error).slice(0, 220)}`)));
  child.once('spawn', () => { child.unref(); resolve({ opened: true, folder: root, executable }); });
 });
});
secureHandle('project:capcut-audit', async () => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber složku projektu CapCut.');
 return CapCutProjectAudit.auditCapCutProject(approvedProjectRoot);
});
secureHandle('project:check-actions', async () => ({ actions: Object.entries(ProjectCheckRunner.ACTIONS).map(([id, action]) => ({ id, label: action.label, executesProjectScripts: !!action.executesProjectScripts, needs: action.needs })) }));
let activeProjectCheck = null;
secureHandle('project:run-check', async (_event, actionId) => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber projektovou složku.');
 if (typeof actionId !== 'string' || !Object.prototype.hasOwnProperty.call(ProjectCheckRunner.ACTIONS, actionId)) throw new Error('Tato kontrola není v bezpečném seznamu povolených akcí.');
 const action = ProjectCheckRunner.ACTIONS[actionId];
 const requestedRoot = await fs.realpath(approvedProjectRoot);
 if (action.executesProjectScripts) {
  const answer = await dialog.showMessageBox(mainWindow, {
   type: 'warning', title: 'Povolit spuštění projektové kontroly?',
   message: action.label,
   detail: 'Tato kontrola může načíst projektové skripty nebo spustit build/import kroky pod oprávněními tvého uživatelského účtu. Spouští se pouze pevně přednastavená akce, nikoli libovolný příkaz. Spusť ji jen v projektu, kterému důvěřuješ.',
   buttons: ['Spustit kontrolu', 'Zrušit'], defaultId: 1, cancelId: 1, noLink: true
  });
  if (answer.response !== 0) return { cancelled: true, action: actionId };
 }
 return withProjectMutationLock(null, async () => {
  if (activeProjectCheck) throw new Error('Jiná projektová kontrola už běží.');
  if (!approvedProjectRoot) throw new Error('Projekt už není vybrán.');
  const currentRoot = await fs.realpath(approvedProjectRoot);
  if (currentRoot !== requestedRoot) throw new Error('Projekt se během potvrzení změnil. Vyber ho znovu a spusť kontrolu znovu.');
  const paths = await readEditorPaths();
  const toolPath = action.needs === 'godot' ? paths.godot : action.needs === 'unity' ? paths.unity : undefined;
  const controller = new AbortController();
  activeProjectCheck = { controller, actionId, root: currentRoot };
  try {
   return await ProjectCheckRunner.runProjectCheck({ root: currentRoot, action: actionId, toolPath, signal: controller.signal });
  } finally { activeProjectCheck = null; }
 });
});
secureHandle('project:cancel-check', async () => {
 if (!activeProjectCheck) return { cancelled: false, reason: 'Žádná kontrola právě neběží.' };
 activeProjectCheck.controller.abort();
 return { cancelled: true, action: activeProjectCheck.actionId };
});
secureHandle('project:unity-bridge-source', async () => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber Unity projekt.');
 const sourcePath = path.resolve(__dirname, '..', 'integrations', 'unity', 'Editor', 'AuroraEditorBridge.cs');
 const stat = await fs.lstat(sourcePath);
 if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 500000) throw new Error('Zabalený Unity Bridge není bezpečný běžný soubor.');
 const real = await fs.realpath(sourcePath);
 if (path.resolve(real) !== path.resolve(sourcePath)) throw new Error('Unity Bridge nesmí být symbolický odkaz.');
 return { path: 'Assets/Editor/AuroraAI/AuroraEditorBridge.cs', content: await fs.readFile(real, 'utf8') };
});
secureHandle('roblox:status', async () => RobloxPluginManager.status({ localAppData: process.env.LOCALAPPDATA || (process.platform === 'win32' ? path.join(os.homedir(), 'AppData', 'Local') : '') }));
secureHandle('roblox:install', async () => {
 if (process.platform !== 'win32') throw new Error('Instalace Roblox Studio pluginu je podporována pouze ve Windows.');
 const answer = await dialog.showMessageBox(mainWindow, { type: 'warning', title: 'Instalovat Aurora Bridge do Roblox Studio?', message: 'Roblox Studio bude načítat skript AuroraBridge při spuštění.', detail: 'Plugin může komunikovat s Aurorou přes lokální Studio Bridge. Instaluj ho pouze pokud důvěřuješ tomuto balíku. Existující cizí plugin nebude přepsán; vlastní předchozí Aurora Bridge bude zazálohován.', buttons: ['Instalovat plugin', 'Zrušit'], defaultId: 1, cancelId: 1, noLink: true });
 if (answer.response !== 0) return { cancelled: true };
 const sourcePath = path.resolve(__dirname, '..', 'integrations', 'roblox-studio', 'AuroraBridge.plugin.lua');
 return RobloxPluginManager.install({ localAppData: process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local'), sourcePath });
});
secureHandle('roblox:disable', async () => {
 if (process.platform !== 'win32') throw new Error('Správa Roblox Studio pluginu je podporována pouze ve Windows.');
 const answer = await dialog.showMessageBox(mainWindow, { type: 'question', title: 'Vypnout Aurora Bridge?', message: 'Plugin bude bezpečně přejmenován na .disabled a Roblox Studio ho přestane načítat.', buttons: ['Vypnout plugin', 'Zrušit'], defaultId: 1, cancelId: 1, noLink: true });
 if (answer.response !== 0) return { cancelled: true };
 return RobloxPluginManager.uninstall({ localAppData: process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local') });
});
secureHandle('project:godot-audit',async()=>{
 requirePlugin('projectRead');
 if(!approvedProjectRoot)throw new Error('Nejdřív vyber složku projektu.');
 return GodotProjectAudit.auditGodotProject(approvedProjectRoot);
});

secureHandle('project:unity-info', async () => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber Unity projekt.');
 return UnityProjectAudit.getUnityProjectInfo(approvedProjectRoot);
});

secureHandle('project:unity-audit', async () => {
 requirePlugin('projectRead');
 if (!approvedProjectRoot) throw new Error('Nejdřív vyber Unity projekt.');
 return UnityProjectAudit.auditUnityProject(approvedProjectRoot);
});

secureHandle('project:unity-audit-export',async(_e,payload={})=>{
 requirePlugin('projectRead');
 if(!approvedProjectRoot)throw new Error('Nejdřív vyber Unity projekt.');
 const report=payload?.report;
 if(!report||typeof report!=='object'||!Array.isArray(report.findings)||!report.counts||typeof report.counts!=='object')throw new Error('Chybí platný výsledek auditu. Spusť audit znovu.');
 const safeText=(v,max=1000)=>String(v??'').replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g,'').slice(0,max);
 const safe={root:safeText(report.root,2000),generatedAt:safeText(report.generatedAt,80),scannedScripts:Math.max(0,Math.min(12000,Number(report.scannedScripts)||0)),scannedAssets:Math.max(0,Math.min(12000,Number(report.scannedAssets)||0)),visited:Math.max(0,Math.min(12000,Number(report.visited)||0)),limitReached:!!report.limitReached,packageCount:Math.max(0,Math.min(1000,Number(report.packageCount)||0)),counts:{error:Math.max(0,Number(report.counts.error)||0),warning:Math.max(0,Number(report.counts.warning)||0),info:Math.max(0,Number(report.counts.info)||0)},findings:report.findings.slice(0,250).map(f=>({severity:['error','warning','info'].includes(f.severity)?f.severity:'info',code:safeText(f.code,80),file:safeText(f.file,500),line:f.line==null?null:Math.max(1,Math.min(10000000,Number(f.line)||1)),message:safeText(f.message,1200)})),note:safeText(report.note,1000)};
 const markdown=`# Aurora AI — Unity static audit\n\n- Generated: ${safe.generatedAt}\n- Project: ${safe.root}\n- C# scripts scanned: ${safe.scannedScripts}\n- Assets files scanned: ${safe.scannedAssets}\n- Entries visited: ${safe.visited}${safe.limitReached?' (limit reached; results may be incomplete)':''}\n- Package count: ${safe.packageCount}\n- Errors: ${safe.counts.error} · Warnings: ${safe.counts.warning} · Info: ${safe.counts.info}\n\n## Findings\n\n${safe.findings.length?safe.findings.map(f=>`### [${f.severity.toUpperCase()}] ${f.code}\n- File: \`${f.file.replace(/`/g,'\\`')}\`${f.line?` (line ${f.line})`:''}\n\n${f.message}`).join('\n\n'):'No findings from the enabled heuristics.'}\n\n> ${safe.note}\n`;
 const result=await dialog.showSaveDialog({title:'Exportovat Unity audit',defaultPath:`Aurora-Unity-Audit-${new Date().toISOString().slice(0,10)}.md`,filters:[{name:'Markdown report',extensions:['md']},{name:'JSON report',extensions:['json']}]});
 if(result.canceled||!result.filePath)return {cancelled:true};
 const ext=path.extname(result.filePath).toLowerCase();if(!['.md','.json'].includes(ext))throw new Error('Povoleny jsou pouze soubory .md a .json.');
 await fs.writeFile(result.filePath,ext==='.json'?JSON.stringify({format:'aurora-unity-audit',version:1,...safe},null,2):markdown,'utf8');
 return {cancelled:false,path:result.filePath,format:ext.slice(1),findings:safe.findings.length};
});

secureHandle('project:unity-diagnostics',async()=>{
 requirePlugin('projectRead');
 if(!approvedProjectRoot)throw new Error('Nejdřív vyber projektovou složku.');
 const root=await fs.realpath(approvedProjectRoot), candidates=[];
 async function findLogs(dir,depth=0){if(depth>4||candidates.length>=8)return;let entries;try{entries=await fs.readdir(dir,{withFileTypes:true})}catch{return}for(const ent of entries){if(candidates.length>=8)break;if(ent.name.startsWith('.')||IGNORE_DIRS.has(ent.name))continue;const full=path.join(dir,ent.name);if(ent.isDirectory())await findLogs(full,depth+1);else if(ent.isFile()&&/\.log$/i.test(ent.name)){try{const st=await fs.stat(full);if(st.size>0&&st.size<=2*1024*1024)candidates.push(full)}catch{}}}}
 await findLogs(root);
 const patterns=[/\b(error|exception|fatal|failed|failure|crash|compiler error|CS\d{4})\b/i,/NullReferenceException|MissingReferenceException|ArgumentException|InvalidOperationException/i,/\berror CS\d{4}\b/i];
 const findings=[];
 for(const file of candidates){let content;try{content=await fs.readFile(file,'utf8')}catch{continue}const lines=content.split(/\r?\n/);for(let i=0;i<lines.length;i++){const line=lines[i].trim();if(line&&patterns.some(re=>re.test(line))){findings.push({file:path.relative(root,file),line:i+1,text:line.slice(0,500)});if(findings.length>=100)break}}if(findings.length>=100)break}
 return {scanned:candidates.map(f=>path.relative(root,f)),findings,root,generatedAt:new Date().toISOString(),note:'Automatické hledání výrazů v dostupných .log souborech; nejedná se o úplnou kompilaci ani potvrzení příčiny.'};
});

secureHandle('project:prepare-edit',async(_e,payload)=>{
 requirePlugin('projectWrites');
 const {root,target,relative}=safeProjectTarget(payload?.path); const content=payload?.content;
 if(typeof content!=='string'||content.length>500000)throw new Error('Obsah musí být text do 500 kB.');
 const ext=path.extname(target).toLowerCase();if(!SAFE_TEXT_EXTS.has(ext))throw new Error('Tento typ souboru není povolen pro úpravy.');
 await ProjectSafety.assertSafeMutationPath(fs,root,target);
 let oldContent=null;try{const real=await fs.realpath(target);if(!ProjectSafety.isPathInside(root,real,{allowRoot:false}))throw new Error('Soubor míří mimo projekt.');if(path.resolve(real)!==path.resolve(target))throw new Error('Zápis přes symbolický odkaz není povolen.');const st=await fs.stat(real);if(!st.isFile()||st.size>500000)throw new Error('Soubor je příliš velký.');oldContent=await fs.readFile(real,'utf8')}catch(e){if(e.code!=='ENOENT')throw e}
 for(const [key,value] of pendingEdits){if(Date.now()-value.createdAt>30*60*1000)pendingEdits.delete(key)}
 const crypto=require('node:crypto');const oldHash=crypto.createHash('sha256').update(oldContent??'__AURORA_NEW_FILE__').digest('hex');const id=crypto.randomUUID();const proposal={id,root,target,relative,content,oldContent,oldHash,createdAt:Date.now()};pendingEdits.set(id,proposal);
 return {id,path:relative,exists:oldContent!==null,oldContent:oldContent===null?null:oldContent.slice(0,20000),newContent:content.slice(0,20000),truncated:content.length>20000};
});
secureHandle('project:prepare-batch',async(_e,payload)=>{
 requirePlugin('projectWrites');
 if(!approvedProjectRoot)throw new Error('Nejdřív vyber schválenou projektovou složku.');
 const edits=Array.isArray(payload?.edits)?payload.edits:[];
 if(!edits.length||edits.length>12)throw new Error('Balík musí obsahovat 1 až 12 souborů.');
 const root=await fs.realpath(approvedProjectRoot), seen=new Set(), prepared=[];
 for(const edit of edits){
  const {target,relative}=safeProjectTarget(edit?.path);
  const duplicateKey=relative.toLocaleLowerCase('en');if(seen.has(duplicateKey))throw new Error(`Duplicitní cesta v balíku (bez rozlišení velikosti písmen): ${relative}`); seen.add(duplicateKey);
  if(!SAFE_TEXT_EXTS.has(path.extname(target).toLowerCase()))throw new Error(`Nepovolená přípona: ${relative}`);
  await ProjectSafety.assertSafeMutationPath(fs,root,target);
  if(typeof edit.content!=='string'||edit.content.length>500000)throw new Error(`Obsah ${relative} musí být text do 500 kB.`);
  let oldContent=null;
  try{
   const real=await fs.realpath(target); if(!ProjectSafety.isPathInside(root,real,{allowRoot:false}))throw new Error('Soubor míří mimo projekt.');if(path.resolve(real)!==path.resolve(target))throw new Error('Zápis přes symbolický odkaz není povolen.');
   const st=await fs.stat(real);if(!st.isFile()||st.size>500000)throw new Error(`Soubor ${relative} není běžný textový soubor nebo je příliš velký.`);
   oldContent=await fs.readFile(real,'utf8');
  }catch(e){if(e.code!=='ENOENT')throw e}
  const oldHash=require('node:crypto').createHash('sha256').update(oldContent??'__AURORA_NEW_FILE__').digest('hex');
  prepared.push({path:relative,target,content:edit.content,oldContent,oldHash,exists:oldContent!==null,addedLines:edit.content.split(/\r?\n/).length,removedLines:(oldContent??'').split(/\r?\n/).length});
 }
 for(const [key,value] of pendingEdits){if(Date.now()-value.createdAt>30*60*1000)pendingEdits.delete(key)}
 const id=require('node:crypto').randomUUID();pendingEdits.set(id,{id,root,createdAt:Date.now(),batch:true,edits:prepared});
 return {id,count:prepared.length,files:prepared.map(x=>({path:x.path,exists:x.exists,oldContent:(x.oldContent??'').slice(0,12000),newContent:x.content.slice(0,12000),truncated:x.content.length>12000,oldHash:x.oldHash,addedLines:x.addedLines,removedLines:x.removedLines})),note:'Před zápisem se znovu ověří původní obsah; změny vyžadují samostatné výslovné schválení.'};
});
secureHandle('project:proposal-preview-file',async(_e,payload={})=>{
 requirePlugin('projectWrites');
 const id=String(payload.id||'');const item=pendingEdits.get(id);
 if(!item)throw new Error('Návrh neexistuje nebo už vypršel. Připrav ho znovu.');
 const root=await fs.realpath(item.root);if(root!==item.root||root!==approvedProjectRoot)throw new Error('Vybraný projekt se změnil. Návrh připrav znovu.');
 let edit;
 if(item.batch){
  const index=Number(payload.index);if(!Number.isInteger(index)||index<0||index>=item.edits.length)throw new Error('Neplatný index souboru v návrhu.');edit=item.edits[index];
 }else{
  if(payload.index!==undefined&&payload.index!==null)throw new Error('Jednotlivý návrh neobsahuje index souboru.');edit=item;
 }
 const {target,relative}=safeProjectTarget(edit.relative||edit.path);
 if(target!==edit.target)throw new Error('Cílová cesta návrhu se změnila.');
 await ProjectSafety.assertSafeMutationPath(fs,root,target);
 let current=null;try{const real=await fs.realpath(target);if(!ProjectSafety.isPathInside(root,real,{allowRoot:false})||path.resolve(real)!==path.resolve(target))throw new Error('Cílový soubor už není bezpečný.');const stat=await fs.stat(real);if(!stat.isFile()||stat.size>500000)throw new Error('Cílový soubor překračuje limit 500 kB.');current=await fs.readFile(real,'utf8')}catch(error){if(error.code!=='ENOENT')throw error}
 const hash=require('node:crypto').createHash('sha256').update(current??'__AURORA_NEW_FILE__').digest('hex');if(hash!==edit.oldHash)throw new Error(`Soubor ${relative} se od vytvoření návrhu změnil. Zruš návrh a připrav ho znovu.`);
 return {path:relative,exists:edit.oldContent!==null,oldContent:edit.oldContent,newContent:edit.content,bytes:Buffer.byteLength(edit.content,'utf8')};
});
secureHandle('project:cancel-batch',async(_e,id)=>{const key=String(id||'');if(projectMutationGate.isLocked(key))return {cancelled:false,busy:true};const item=pendingEdits.get(key);if(!item?.batch)return {cancelled:false};pendingEdits.delete(key);return {cancelled:true}});
secureHandle('project:approve-batch',(_e,id)=>withProjectMutationLock(String(id||''),async()=>{
 requirePlugin('projectWrites');
 const item=pendingEdits.get(id);if(!item?.batch)throw new Error('Balík neexistuje nebo už byl zpracován.');
 const root=await fs.realpath(item.root);if(root!==item.root||root!==approvedProjectRoot)throw new Error('Kořen projektu se změnil. Vyber složku znovu.');
 const crypto=require('node:crypto'), staged=[];
 const hashText=value=>crypto.createHash('sha256').update(value??'__AURORA_NEW_FILE__').digest('hex');
 try{
  for(const edit of item.edits){
   const {target,relative}=safeProjectTarget(edit.path);if(target!==edit.target)throw new Error('Cílová cesta se změnila.');
   const parent=path.dirname(target);await ProjectSafety.assertSafeMutationPath(fs,root,target);await fs.mkdir(parent,{recursive:true});await ProjectSafety.assertSafeMutationPath(fs,root,target);const parentReal=await fs.realpath(parent);
   if(!ProjectSafety.isPathInside(root,parentReal)||path.resolve(parentReal)!==path.resolve(parent))throw new Error(`Cílová složka ${relative} míří mimo projekt nebo obsahuje symbolický odkaz.`);
   let current=null;try{const real=await fs.realpath(target);if(!ProjectSafety.isPathInside(root,real,{allowRoot:false}))throw new Error(`Soubor ${relative} míří mimo projekt.`);if(path.resolve(real)!==path.resolve(target))throw new Error(`Zápis přes symbolický odkaz není povolen: ${relative}`);const st=await fs.stat(real);if(!st.isFile()||st.size>500000)throw new Error(`Soubor ${relative} není povolen.`);current=await fs.readFile(real,'utf8')}catch(e){if(e.code!=='ENOENT')throw e}
   if(hashText(current)!==edit.oldHash)throw new Error(`Soubor ${relative} se od vytvoření návrhu změnil. Zruš návrh a připrav ho znovu.`);
   const backup=current===null?null:ProjectSafety.makeBackupPath(target);
   if(backup)await fs.writeFile(backup,current,{encoding:'utf8',flag:'wx'});
   const temp=`${target}.aurora-tmp-${crypto.randomUUID()}`;await fs.writeFile(temp,edit.content,{encoding:'utf8',flag:'wx'});staged.push({target,temp,relative,backup,created:current===null,oldContent:current,applied:false});
  }
  for(const file of staged){await fs.rename(file.temp,file.target);file.applied=true}
  pendingEdits.delete(id);
  return {count:staged.length,files:staged.map(x=>({path:x.relative,backup:x.backup?path.relative(root,x.backup):null,created:x.created})),note:'Zápis dokončen. Změny nebyly automaticky spuštěny ani kompilovány.'};
 }catch(error){
  for(const file of staged){try{await fs.rm(file.temp,{force:true})}catch{}}
  const restoreFailures=[];
  for(const file of [...staged].reverse()){
   if(!file.applied)continue;
   try{
    await ProjectSafety.assertSafeMutationPath(fs,root,file.target);
    if(file.created){await fs.rm(file.target,{force:true});continue;}
    const parent=path.dirname(file.target),parentReal=await fs.realpath(parent);
    if(!ProjectSafety.isPathInside(root,parentReal)||path.resolve(parentReal)!==path.resolve(parent))throw new Error('Cílová složka obnovy není bezpečná.');
    const restoreTemp=`${file.target}.aurora-restore-${crypto.randomUUID()}`;
    await fs.writeFile(restoreTemp,file.oldContent,{encoding:'utf8',flag:'wx'});
    try{await ProjectSafety.assertSafeMutationPath(fs,root,file.target);await fs.rename(restoreTemp,file.target)}catch(restoreError){try{await fs.rm(restoreTemp,{force:true})}catch{}throw restoreError}
   }catch(restoreError){restoreFailures.push(`${file.relative}${file.backup?` (záloha: ${path.relative(root,file.backup)})`:''}: ${String(restoreError?.message||restoreError).slice(0,180)}`)}
  }
  if(restoreFailures.length)throw new Error(`Hromadná změna selhala: ${String(error?.message||error).slice(0,240)}. Navíc se nepodařilo bezpečně obnovit: ${restoreFailures.join('; ')}. Zkontroluj tyto soubory ručně.`);
  throw error;
 }
}));
secureHandle('project:cancel-edit',async(_e,id)=>{const key=String(id||'');if(projectMutationGate.isLocked(key))return {cancelled:false,busy:true};const existed=pendingEdits.delete(key);return {cancelled:existed}});
secureHandle('project:approve-edit',(_e,id)=>withProjectMutationLock(String(id||''),async()=>{
 requirePlugin('projectWrites');
 const item=pendingEdits.get(id);if(!item)throw new Error('Návrh neexistuje nebo už byl zpracován.');
 const root=ProjectSafety.assertSameApprovedRoot(item.root,approvedProjectRoot);const realRoot=await fs.realpath(root);if(realRoot!==root)throw new Error('Kořen projektu se změnil. Vyber složku znovu.');
 const parent=path.dirname(item.target);
 await ProjectSafety.assertSafeMutationPath(fs,root,item.target);
 await fs.mkdir(parent,{recursive:true});
 await ProjectSafety.assertSafeMutationPath(fs,root,item.target);
 const parentReal=await fs.realpath(parent);
 if(!ProjectSafety.isPathInside(root,parentReal)||path.resolve(parentReal)!==path.resolve(parent))throw new Error('Cílová složka míří mimo projekt nebo prochází symbolickým odkazem.');
 let backup=null,current=null;
 try{
  const existingReal=await fs.realpath(item.target);
  if(!ProjectSafety.isPathInside(root,existingReal,{allowRoot:false}))throw new Error('Cílový soubor míří mimo projekt.');
  if(path.resolve(existingReal)!==path.resolve(item.target))throw new Error('Zápis do souboru přes symbolický odkaz není povolen.');
  const st=await fs.stat(existingReal);
  if(!st.isFile()||st.size>500000)throw new Error('Cílový soubor není běžný textový soubor nebo je příliš velký.');
  current=await fs.readFile(existingReal,'utf8');

 }catch(e){if(e.code!=='ENOENT')throw e}
 const crypto=require('node:crypto');const currentHash=crypto.createHash('sha256').update(current??'__AURORA_NEW_FILE__').digest('hex');
 if(currentHash!==item.oldHash)throw new Error('Soubor se od vytvoření návrhu změnil. Připrav nový návrh a zkontroluj ho znovu.');
 if(current!==null){backup=ProjectSafety.makeBackupPath(item.target);await fs.writeFile(backup,current,{encoding:'utf8',flag:'wx'})}
 const tempPath=item.target+'.aurora-tmp-'+require('node:crypto').randomUUID();
 try{await fs.writeFile(tempPath,item.content,{encoding:'utf8',flag:'wx'});await fs.rename(tempPath,item.target);}catch(error){try{await fs.rm(tempPath,{force:true})}catch{}throw error}
 pendingEdits.delete(id);
 return {path:item.relative,backup:backup?path.relative(root,backup):null,created:!backup};
}));
secureHandle('project:rollback',(_e,relativeBackup)=>withProjectMutationLock(null,async()=>{
 requirePlugin('projectWrites');
 const {root,backupTarget,backupRelative:relative,originalRelative:original,originalTarget}=ProjectSafety.originalFromBackup(approvedProjectRoot,relativeBackup,SAFE_TEXT_EXTS);
 ProjectSafety.assertSameApprovedRoot(root,approvedProjectRoot);
 const rootReal=await fs.realpath(root);if(rootReal!==root)throw new Error('Kořen projektu se změnil. Vyber složku znovu.');
 const {target:real}=await ProjectSafety.resolveContainedRealPath(fs,root,backupTarget,{allowRoot:false,rejectSymlinkAlias:true});
 const backupStat=await fs.stat(real);if(!backupStat.isFile()||backupStat.size>500000)throw new Error('Záloha není povolený běžný textový soubor.');
 const parent=path.dirname(originalTarget);const parentReal=await fs.realpath(parent);
 if(!ProjectSafety.isPathInside(root,parentReal)||path.resolve(parentReal)!==path.resolve(parent))throw new Error('Cílová složka obnovy chybí, míří mimo projekt nebo obsahuje symbolický odkaz.');
 try{const existingReal=await fs.realpath(originalTarget);if(!ProjectSafety.isPathInside(root,existingReal,{allowRoot:false})||path.resolve(existingReal)!==path.resolve(originalTarget))throw new Error('Cílový soubor obnovy je symbolický odkaz nebo míří mimo projekt.')}catch(error){if(error.code!=='ENOENT')throw error}
 const temp=originalTarget+'.aurora-rollback-'+require('node:crypto').randomUUID();
 try{await fs.copyFile(real,temp,require('node:fs').constants.COPYFILE_EXCL);await fs.rename(temp,originalTarget);}catch(error){try{await fs.rm(temp,{force:true})}catch{}throw error}
 return {restored:original,from:relative};
}));


// Portable workspace backup: only app-owned conversation/settings data, never project files or secrets.
secureHandle('workspace:export', async (_event, payload) => {
 const result = await dialog.showSaveDialog({title:'Exportovat zálohu Aurory',defaultPath:`Aurora-workspace-${new Date().toISOString().slice(0,10)}.json`,filters:[{name:'Aurora workspace JSON',extensions:['json']}]});
 if(result.canceled||!result.filePath)return {cancelled:true};
 const prepared=WorkspacePersistence.prepareWorkspace(payload&&typeof payload==='object'?payload:{},{maxSerializedChars:8_000_000,contentBudget:6_000_000,maxMessages:4000,maxConversations:100,perConversationMessages:200,perMessageChars:50000});
 const data={format:'aurora-workspace',version:1,exportedAt:new Date().toISOString(),...prepared.payload};
 const serialized=JSON.stringify(data,null,2);
 const size=Buffer.byteLength(serialized,'utf8');
 if(size>24*1024*1024)throw new Error('Záloha by překročila limit 24 MB. Zkrať historii nebo exportuj menší část pracovního prostoru.');
 await fs.writeFile(result.filePath,serialized,'utf8');
 return {cancelled:false,path:result.filePath,conversations:data.conversations.length,bytes:size,trimmed:prepared.trimmed,omittedMessages:prepared.stats.omittedMessages,omittedCharacters:prepared.stats.omittedCharacters};
});
secureHandle('workspace:import', async () => {
 const result=await dialog.showOpenDialog({title:'Importovat zálohu Aurory',properties:['openFile'],filters:[{name:'Aurora workspace JSON',extensions:['json']}]});
 if(result.canceled||!result.filePaths[0])return {cancelled:true};
 const stat=await fs.stat(result.filePaths[0]);if(!stat.isFile()||stat.size>25*1024*1024)throw new Error('Záloha musí být běžný soubor do 25 MB.');
 const parsed=JSON.parse(await fs.readFile(result.filePaths[0],'utf8'));if(parsed?.format!=='aurora-workspace'||parsed?.version!==1||!Array.isArray(parsed.conversations))throw new Error('Soubor není platná záloha Aurory.');
 const conversations=parsed.conversations.slice(0,500).filter(c=>c&&typeof c==='object').map(c=>({id:String(c.id||require('node:crypto').randomUUID()).slice(0,100),title:String(c.title||'Importovaná konverzace').slice(0,160),updatedAt:WorkspacePersistence.normalizeTimestamp(c.updatedAt,Date.now()),archived:c.archived===true,project:c.project===true&&c.archived!==true,storageTrimmed:c.storageTrimmed===true,messages:Array.isArray(c.messages)?c.messages.slice(-200).map(m=>WorkspacePersistence.compactMessage(m,50000)).filter(Boolean):[]}));
 return {cancelled:false,model:typeof parsed.model==='string'?parsed.model.slice(0,120):'',activeConversationId:typeof parsed.activeConversationId==='string'?parsed.activeConversationId.slice(0,100):null,folderSelected:!!parsed.folder,projectFolderSelected:!!parsed.projectFolder,preferences:WorkspacePersistence.compactPreferences(parsed.preferences),provider:['ollama','openai','anthropic','gemini'].includes(parsed.provider)?parsed.provider:'ollama',performanceProfile:['adaptive','quick','balanced','deep','extra','manual'].includes(parsed.performanceProfile)?parsed.performanceProfile:'adaptive',autoRoute:parsed.autoRoute!==false,agentConfig:WorkspacePersistence.compactAgentConfig(parsed.agentConfig),agentComplexity:parsed.agentComplexity==='advanced'?'advanced':'simple',conversations};
});

app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit()});
const BASE='http://127.0.0.1:11434';
const activeChats = new Map();
async function ollama(pathname, options={}) { const res=await fetch(BASE+pathname,{...options,headers:{'Content-Type':'application/json',...(options.headers||{})}}); if(!res.ok) throw new Error(`Ollama ${res.status}: ${(await readTextBounded(res,1200)).slice(0,1000)}`); return res; }

// Optional long-term memory is opt-in, persisted separately from chat backups,
// and embeddings are requested only from the fixed loopback Ollama endpoint.
const localMemoryConfigPath=()=>path.join(app.getPath('userData'),'local-memory-config.json');
const localMemoryStorePath=()=>path.join(app.getPath('userData'),'local-memory-index.json');
const MAX_LOCAL_MEMORY_STORE_BYTES=8*1024*1024;
let localMemoryMutationTail=Promise.resolve();
function withLocalMemoryMutation(work){const task=localMemoryMutationTail.then(work,work);localMemoryMutationTail=task.catch(()=>{});return task;}
async function readLocalMemoryConfig(){try{const raw=JSON.parse(await BoundedFile.readFileBounded(fs,localMemoryConfigPath(),64*1024,'Konfigurace lokální paměti').then(bytes=>bytes.toString('utf8')));if(!raw||typeof raw!=='object'||Array.isArray(raw))throw new Error('Konfigurace nemá platnou strukturu.');return LocalMemory.normalizeMemoryConfig(raw)}catch(error){if(error?.code==='ENOENT')return LocalMemory.normalizeMemoryConfig(null);throw new Error('Konfigurace lokální paměti je poškozená nebo ji nelze přečíst. Oprav nebo přejmenuj soubor local-memory-config.json v profilu Aurory.')}}
async function readLocalMemoryRecords(){
 try{const filename=localMemoryStorePath(),st=await fs.stat(filename);if(!st.isFile()||st.size>MAX_LOCAL_MEMORY_STORE_BYTES)throw new Error('Index lokální paměti překračuje bezpečný limit 8 MB.');const bytes=await BoundedFile.readFileBounded(fs,filename,MAX_LOCAL_MEMORY_STORE_BYTES,'Index lokální paměti');const raw=JSON.parse(bytes.toString('utf8'));if(raw?.format!=='aurora-local-memory'||!Array.isArray(raw.records))throw new Error('Index lokální paměti má neplatný formát.');return raw.records.slice(0,LocalMemory.MAX_MEMORIES).map(LocalMemory.normalizeMemoryRecord).filter(Boolean)}catch(error){if(error?.code==='ENOENT')return [];if(String(error?.message||'').includes('8 MB'))throw error;throw new Error('Index lokální paměti nejde přečíst nebo je poškozený. Nejdřív exportuj případný soubor mimo aplikaci a pak ho oprav nebo smaž.')}
}
async function writeLocalMemoryJson(filename,value){
 const serialized=JSON.stringify(value);if(Buffer.byteLength(serialized,'utf8')>MAX_LOCAL_MEMORY_STORE_BYTES)throw new Error('Lokální paměť je příliš velká (limit 8 MB).');
 await fs.mkdir(path.dirname(filename),{recursive:true});const temp=filename+'.tmp-'+require('node:crypto').randomUUID();
 try{await fs.writeFile(temp,serialized,{encoding:'utf8',flag:'wx',mode:0o600});await fs.rename(temp,filename)}catch(error){try{await fs.rm(temp,{force:true})}catch{}throw error}
}
async function saveLocalMemoryRecords(records){const clean=records.slice(0,LocalMemory.MAX_MEMORIES).map(LocalMemory.normalizeMemoryRecord).filter(Boolean);await writeLocalMemoryJson(localMemoryStorePath(),{format:'aurora-local-memory',version:1,updatedAt:new Date().toISOString(),records:clean});return clean}
async function getLocalEmbedding(text,model,timeoutMs=30000){
 const clean=String(text||'').trim().slice(0,LocalMemory.MAX_MEMORY_TEXT);if(!clean)throw new Error('Text pro lokální vektor je prázdný.');
 return LocalMemoryClient.createEmbedding({fetchImpl:fetch,readJsonBounded,readTextBounded,model,text:clean,timeoutMs});
}
secureHandle('memory:status',async()=>{const [config,records]=await Promise.all([readLocalMemoryConfig(),readLocalMemoryRecords()]);return {...config,count:records.length,localOnly:true,backupIncluded:false}});
secureHandle('memory:set-config',(_event,payload)=>withLocalMemoryMutation(async()=>{
 if(!payload||typeof payload.enabled!=='boolean')throw new Error('Stav lokální paměti musí být zapnutý/vypnutý.');const requestedModel=String(payload.embeddingModel||LocalMemory.DEFAULT_EMBEDDING_MODEL).trim();if(!/^[a-zA-Z0-9._-]+(?::[a-zA-Z0-9._-]+)?$/.test(requestedModel)||requestedModel.includes('..'))throw new Error('Název embedding modelu není platný.');const config=LocalMemory.normalizeMemoryConfig({enabled:payload.enabled,embeddingModel:requestedModel});const records=await readLocalMemoryRecords();const previous=await readLocalMemoryConfig();if(records.length&&previous.embeddingModel!==config.embeddingModel)throw new Error('Model pro paměť nelze změnit, dokud jsou uložené poznámky. Nejprve je exportuj ručně nebo smaž a pak změň model.');await writeLocalMemoryJson(localMemoryConfigPath(),{...config,updatedAt:new Date().toISOString()});return {...config,count:records.length,localOnly:true,backupIncluded:false};
}));
secureHandle('memory:check-model',async()=>{const config=await readLocalMemoryConfig();const vector=await getLocalEmbedding('Krátká kontrolní věta pro ověření místní paměti.',config.embeddingModel);return {ok:true,model:config.embeddingModel,dimensions:vector.length,localOnly:true}});
secureHandle('memory:list',async()=>{const records=await readLocalMemoryRecords();return records.map(({id,title,text,createdAt,embeddingModel})=>({id,title,text,createdAt,embeddingModel}))});
secureHandle('memory:export',()=>withLocalMemoryMutation(async()=>{
 const result=await dialog.showSaveDialog(mainWindow,{title:'Exportovat pouze lokální poznámky Aurory',defaultPath:`Aurora-Lokalni-Pamet-${new Date().toISOString().slice(0,10)}.json`,filters:[{name:'Záloha lokální paměti JSON',extensions:['json']}]});if(result.canceled||!result.filePath)return {cancelled:true};
 const config=await readLocalMemoryConfig();const records=await readLocalMemoryRecords();const data=LocalMemoryBackup.createBackup({records,embeddingModel:config.embeddingModel});const serialized=JSON.stringify(data,null,2);if(Buffer.byteLength(serialized,'utf8')>MAX_LOCAL_MEMORY_STORE_BYTES)throw new Error('Export lokální paměti překračuje limit 8 MB.');
 const target=path.resolve(result.filePath);const temp=target+'.tmp-'+require('node:crypto').randomUUID();try{await fs.writeFile(temp,serialized,{encoding:'utf8',flag:'wx',mode:0o600});await fs.rename(temp,target)}catch(error){try{await fs.rm(temp,{force:true})}catch{}throw error}return {cancelled:false,count:records.length,fileName:path.basename(target)};
}));
secureHandle('memory:import',()=>withLocalMemoryMutation(async()=>{
 const result=await dialog.showOpenDialog(mainWindow,{title:'Importovat zálohu lokální paměti Aurory',properties:['openFile'],filters:[{name:'Záloha lokální paměti JSON',extensions:['json']}]});if(result.canceled||!result.filePaths[0])return {cancelled:true};
 const filename=await fs.realpath(result.filePaths[0]);const stat=await fs.stat(filename);if(!stat.isFile()||stat.size>MAX_LOCAL_MEMORY_STORE_BYTES)throw new Error('Záložní soubor musí být běžný JSON soubor do 8 MB.');const bytes=await BoundedFile.readFileBounded(fs,filename,MAX_LOCAL_MEMORY_STORE_BYTES,'Záloha lokální paměti');const parsed=JSON.parse(bytes.toString('utf8'));
 const backup=LocalMemoryBackup.parseBackup(parsed);
 const imported=backup.records;
 const config=await readLocalMemoryConfig();const current=await readLocalMemoryRecords();
 if(current.length&&config.embeddingModel!==backup.embeddingModel)throw new Error('Aktuální paměť používá jiný embedding model. Exportuj aktuální poznámky, vymaž je a potom import opakuj.');
 const merged=LocalMemoryBackup.mergeRecords(current,imported);
 const configChanged=config.embeddingModel!==backup.embeddingModel;
 if(configChanged)await writeLocalMemoryJson(localMemoryConfigPath(),{...config,embeddingModel:backup.embeddingModel,updatedAt:new Date().toISOString()});
 try {
  await saveLocalMemoryRecords(merged);
 } catch(error) {
  if(configChanged) {
   try { await writeLocalMemoryJson(localMemoryConfigPath(),{...config,updatedAt:new Date().toISOString()}); }
   catch(rollbackError) { throw new Error(`Import lokální paměti selhal a obnovení původního nastavení modelu také selhalo: ${error.message}; ${rollbackError.message}`); }
  }
  throw error;
 }
 return {cancelled:false,imported:imported.length,count:merged.length,embeddingModel:backup.embeddingModel,enabled:config.enabled};
}));
secureHandle('memory:add',(_event,payload)=>withLocalMemoryMutation(async()=>{
 const config=await readLocalMemoryConfig();if(!config.enabled)throw new Error('Nejprve zapni volitelnou lokální paměť.');const text=String(payload?.text||'').replace(/\u0000/g,'').trim().slice(0,LocalMemory.MAX_MEMORY_TEXT);if(text.length<3)throw new Error('Poznámka musí mít alespoň 3 znaky.');const title=String(payload?.title||text.split(/\s+/).slice(0,8).join(' ')).replace(/[\r\n\t]+/g,' ').trim().slice(0,LocalMemory.MAX_MEMORY_TITLE)||'Lokální poznámka';const embedding=await getLocalEmbedding(`${title}\n${text}`,config.embeddingModel);const records=await readLocalMemoryRecords();const record=LocalMemory.normalizeMemoryRecord({id:require('node:crypto').randomUUID(),title,text,embedding,embeddingModel:config.embeddingModel,createdAt:Date.now()});if(!record)throw new Error('Poznámku se nepodařilo bezpečně normalizovat.');const saved=await saveLocalMemoryRecords([record,...records].slice(0,LocalMemory.MAX_MEMORIES));return {ok:true,count:saved.length,record:{id:record.id,title:record.title,text:record.text,createdAt:record.createdAt,embeddingModel:record.embeddingModel}};
}));
secureHandle('memory:delete',(_event,id)=>withLocalMemoryMutation(async()=>{const key=String(id||'');if(!/^[a-f0-9-]{36}$/i.test(key))throw new Error('Neplatné ID poznámky.');const records=await readLocalMemoryRecords();const filtered=records.filter(record=>record.id!==key);if(filtered.length===records.length)return {deleted:false,count:records.length};await saveLocalMemoryRecords(filtered);return {deleted:true,count:filtered.length}}));
secureHandle('memory:clear',()=>withLocalMemoryMutation(async()=>{await fs.rm(localMemoryStorePath(),{force:true});return {cleared:true,count:0}}));
secureHandle('memory:recall',async(_event,payload)=>{
 // Personal memory is deliberately not injected into cloud-provider requests.
 if(payload?.provider!=='ollama')return {enabled:false,matches:[],context:''};const config=await readLocalMemoryConfig();if(!config.enabled)return {enabled:false,matches:[],context:''};const records=await readLocalMemoryRecords();const compatible=records.filter(record=>record.embeddingModel===config.embeddingModel);if(!compatible.length)return {enabled:true,matches:[],context:''};const query=String(payload?.query||'').trim().slice(0,3000);if(!query)return {enabled:true,matches:[],context:''};const vector=await getLocalEmbedding(query,config.embeddingModel,8000);const matches=LocalMemory.selectRelevantMemories(compatible,vector,{limit:3,embeddingModel:config.embeddingModel});return {enabled:true,matches:matches.map(({id,title,text,createdAt,similarity})=>({id,title,text,createdAt,similarity})),context:LocalMemory.buildMemoryContext(matches)};
});
async function getOllamaHealth(){
 try{const response=await fetch('http://127.0.0.1:11434/api/tags',{signal:AbortSignal.timeout(1200)});return response.ok}catch{return false}
}
function locateOllamaExecutable(){
 const candidates=[process.env.OLLAMA_EXE,process.env.LOCALAPPDATA?path.join(process.env.LOCALAPPDATA,'Programs','Ollama','ollama.exe'):null,process.env.ProgramFiles?path.join(process.env.ProgramFiles,'Ollama','ollama.exe'):null,'ollama'];
 for(const candidate of candidates.filter(Boolean)){if(candidate==='ollama')return candidate;try{require('node:fs').accessSync(candidate);return candidate}catch{}}
 return 'ollama';
}
secureHandle('ollama:service-status',async()=>({online:await getOllamaHealth(),managed:!!managedOllamaProcess,exitCode:managedOllamaExitCode}));
secureHandle('ollama:install-service',async()=>{
 if(process.platform!=='win32')throw new Error('Automatická instalace přes winget je dostupná pouze ve Windows. Otevři https://ollama.com/download.');
 if(await getOllamaHealth())return {ok:true,alreadyInstalled:true,message:'Ollama už běží.'};
 return await new Promise((resolve,reject)=>{
  let settled=false,output='',timer=null;
  const finish=(error,result)=>{if(settled)return;settled=true;clearTimeout(timer);if(error)reject(error);else resolve(result)};
  let child;
  try{child=spawn('winget.exe',['install','--id','Ollama.Ollama','--exact','--silent','--accept-package-agreements','--accept-source-agreements'],{windowsHide:true,shell:false,stdio:['ignore','pipe','pipe']});}
  catch(error){finish(new Error(`[WINGET_NOT_AVAILABLE] Nelze spustit Windows Package Manager: ${error.message}. Nainstaluj Ollama z https://ollama.com/download.`));return}
  const capture=chunk=>{output=(output+String(chunk||'')).slice(-6000)};
  child.stdout?.on('data',capture);child.stderr?.on('data',capture);
  timer=setTimeout(()=>{try{child.kill()}catch{}finish(new Error('Instalace Ollama přes winget překročila časový limit 10 minut. Zkontroluj připojení a Windows Package Manager.'))},10*60*1000);
  child.once('error',error=>finish(new Error(`[WINGET_NOT_AVAILABLE] Instalátor Ollama nelze spustit (${error.message}). Nainstaluj Ollama z https://ollama.com/download.`)));
  child.once('close',code=>{
   if(code===0)finish(null,{ok:true,installed:true,message:'Instalace Ollama byla dokončena. Teď ji spouštím…'});
   else finish(new Error(`Instalace Ollama selhala (winget exit ${code??'unknown'}). ${output.slice(-1800)}`));
  });
 });
});
secureHandle('ollama:start-service',async()=>{
 if(await getOllamaHealth())return {ok:true,alreadyRunning:true,managed:!!managedOllamaProcess,message:'Ollama už běží. Aurora ji nepřebírá ani nemění.'};
 if(managedOllamaProcess)return {ok:true,alreadyRunning:false,managed:true,message:'Ollama se už spouští.'};
 const executable=locateOllamaExecutable();
 return await new Promise((resolve,reject)=>{
  let settled=false;const child=spawn(executable,['serve'],{windowsHide:true,stdio:'ignore',shell:false});managedOllamaProcess=child;managedOllamaExitCode=null;
  const finish=(error,result)=>{if(settled)return;settled=true;if(error)reject(error);else resolve(result)};
  child.once('error',error=>{if(managedOllamaProcess===child)managedOllamaProcess=null;const missing=error?.code==='ENOENT';finish(new Error(`${missing?'[OLLAMA_NOT_INSTALLED] ':''}Ollama nebyla nalezena nebo ji nelze spustit (${error.message}). ${missing?'Instalaci může Aurora nabídnout přes winget.':'Zkontroluj oprávnění a instalaci.'}`))});
  child.once('exit',(code)=>{managedOllamaExitCode=code;if(managedOllamaProcess===child)managedOllamaProcess=null;if(!settled)finish(new Error(`Ollama skončila před spuštěním služby (exit ${code??'unknown'}).`))});
  const started=Date.now();const check=async()=>{if(settled)return;if(await getOllamaHealth()){finish(null,{ok:true,alreadyRunning:false,managed:true,message:'Ollama byla spuštěna Aurorou.'});return}if(Date.now()-started>15000){finish(new Error('Ollama se do 15 sekund nepřipojila na 127.0.0.1:11434. Zkontroluj instalaci nebo jinou běžící instanci.'));return}setTimeout(check,500)};setTimeout(check,350);
 });
});
secureHandle('ollama:stop-service',async()=>{
 const child=managedOllamaProcess;if(!child)return {ok:true,stopped:false,message:'Tuto Ollama nespustila Aurora, proto ji bezpečně nevypínám. Zavři ji v její vlastní aplikaci nebo systémové liště.'};
 managedOllamaProcess=null;const killed=child.kill();return {ok:killed,stopped:killed,message:killed?'Aurora ukončila svoji instanci Ollama.':'Proces Ollama se nepodařilo ukončit.'};
});
secureHandle('ollama:status',async()=>{try{const r=await ollama('/api/tags');const data=await readJsonBounded(r,2*1024*1024,'Seznam modelů Ollama');return {ok:true,models:Array.isArray(data.models)?data.models.slice(0,2000):[]}}catch(e){return {ok:false,error:e.message,models:[]}}});
secureHandle('ollama:running',async()=>{try{const [running,available,system]=await Promise.all([ollama('/api/ps').then(r=>readJsonBounded(r,2*1024*1024,'Seznam běžících modelů')),ollama('/api/tags').then(r=>readJsonBounded(r,2*1024*1024,'Seznam modelů Ollama')),Promise.resolve({ramGB:Math.round(os.totalmem()/1024**3),freeRamGB:Math.round(os.freemem()/1024**3),cpu:os.cpus()[0]?.model||'Unknown CPU',cores:os.cpus().length})]);return {ok:true,running:Array.isArray(running.models)?running.models.slice(0,2000):[],available:Array.isArray(available.models)?available.models.slice(0,2000):[],system,checkedAt:new Date().toISOString()}}catch(e){return {ok:false,error:e.message,running:[],available:[],system:{ramGB:Math.round(os.totalmem()/1024**3),freeRamGB:Math.round(os.freemem()/1024**3),cpu:os.cpus()[0]?.model||'Unknown CPU',cores:os.cpus().length},checkedAt:new Date().toISOString()}}});
secureHandle('ollama:pull',async(event,name)=>{if(!isValidOllamaModelName(name))throw new Error('Neplatný název modelu.');const r=await ollama('/api/pull',{method:'POST',body:JSON.stringify({name,stream:true})});const reader=r.body.getReader();const decoder=new TextDecoder();let buffer='',last={status:'Začínám stahování',completed:0,total:0};while(true){const {value,done}=await reader.read();if(done)break;buffer+=decoder.decode(value,{stream:true});const lines=buffer.split('\n');buffer=lines.pop();for(const line of lines){if(!line.trim())continue;let obj;try{obj=JSON.parse(line)}catch{continue}if(obj.error)throw new Error(obj.error);last={status:obj.status||'Stahuji…',completed:Number(obj.completed)||0,total:Number(obj.total)||0};event.sender.send('ollama:pull-progress',{name,...last,percent:last.total?Math.min(100,Math.round(last.completed/last.total*100)):null});}}return last});
secureHandle('ollama:delete',async(_e,name)=>{if(!isValidOllamaModelName(name))throw new Error('Neplatný název modelu.');await ollama('/api/delete',{method:'DELETE',body:JSON.stringify({name})});return true});
secureHandle('ollama:chat',async(event,payload)=>{
 const requestId=String(payload?.requestId||'');
 if(!/^[a-zA-Z0-9_-]{8,100}$/.test(requestId))throw new Error('Neplatné ID požadavku.');
 if(!payload||!['ollama',...REMOTE_PROTOCOLS].includes(payload.provider))throw new Error('Neznámý poskytovatel modelu. Vyber Ollama, OpenAI-kompatibilní, Anthropic nebo Gemini.');
 if(activeChats.has(requestId))throw new Error('Požadavek s tímto ID už běží.');
 if(!payload||typeof payload.model!=='string'||(payload.provider!=='ollama'?payload.model!==''&&!RemoteProviders.isValidModelName(payload.model):!isValidOllamaModelName(payload.model)))throw new Error('Neplatný název modelu.');
 if(!Array.isArray(payload.messages)||payload.messages.length<1||payload.messages.length>500)throw new Error('Zpráva musí obsahovat 1 až 500 položek konverzace.');
 const messages=payload.messages.map((m)=>{if(!m||!['system','user','assistant'].includes(m.role)||typeof m.content!=='string'||m.content.length>200000)throw new Error('Konverzace obsahuje neplatnou nebo příliš dlouhou zprávu.');return {role:m.role,content:m.content}});
 const controller=new AbortController();activeChats.set(requestId,{controller,webContents:event.sender});
 try{
  if(payload.provider!=='ollama'){
   const protocol=RemoteProviders.normalizeProtocol(payload.provider);const key=await readProviderKey(protocol);if(!key)throw new Error(`API klíč pro ${protocol} není nastaven. Otevři Nastavení → API a poskytovatelé.`);
   const cfg=await readProviderConfig(protocol);const model=payload.model||cfg.model;
   const request=RemoteProviders.buildRemoteRequest({protocol,baseUrl:cfg.baseUrl,model,messages,options:{...(payload.options&&typeof payload.options==='object'?payload.options:{}),stream:payload.stream!==false},apiKey:key});
   const timeoutSignal=AbortSignal.timeout(180000);
   const response=await fetch(request.url,{method:'POST',headers:request.headers,body:JSON.stringify(request.body),signal:AbortSignal.any([controller.signal,timeoutSignal])});
   if(!response.ok)throw new Error(`${cfg.label} API ${response.status}: ${(await readTextBounded(response,1200)).slice(0,300)}`);
   if(payload.stream===false){
    const raw=await readTextBounded(response,12_000_000);let parsed;try{parsed=JSON.parse(raw)}catch{throw new Error(`${cfg.label} vrátilo neplatnou JSON odpověď.`)}
    const generated=RemoteProviders.extractRemoteTestText(protocol,parsed).trim();if(!generated)throw new Error(`${cfg.label} vrátilo prázdnou odpověď.`);
    if(generated.length>8_000_000)throw new Error('Odpověď překročila bezpečný limit 8 MB.');
    if(!event.sender.isDestroyed())event.sender.send('ollama:chunk',{requestId,text:generated});return generated;
   }
   if(!response.body)throw new Error(`${cfg.label} API nevrátilo stream odpovědi.`);
   const reader=response.body.getReader(),decoder=new TextDecoder();let result='',receivedBytes=0;
   const parser=RemoteProviders.createSSEDecoder(({data})=>{const raw=data.trim();if(!raw||raw==='[DONE]')return;const chunk=RemoteProviders.parseRemoteEvent(protocol,raw);if(chunk){result+=chunk;if(result.length>8_000_000)throw new Error('Odpověď překročila bezpečný limit 8 MB.');if(!event.sender.isDestroyed())event.sender.send('ollama:chunk',{requestId,text:chunk})}});
   try{while(true){const {value,done}=await reader.read();if(done)break;receivedBytes+=value.byteLength;if(receivedBytes>12_000_000){await reader.cancel().catch(()=>{});throw new Error('Stream vzdáleného modelu překročil bezpečný limit 12 MB.')}parser.push(decoder.decode(value,{stream:true}))}parser.push(decoder.decode());parser.finish();return result}
   catch(error){if(timeoutSignal.aborted&&!controller.signal.aborted)throw new Error(`${cfg.label} neodpovědělo do 180 sekund. Zkus rychlejší model nebo menší výstup.`);throw error}
   finally{reader.releaseLock()}
  }
  const request={model:payload.model,messages,keep_alive:GenerationOptions.normalizeKeepAlive(payload.keep_alive),stream:payload.stream!==false,options:GenerationOptions.normalizeOllamaOptions(payload.options)};
  if(typeof payload.format==='string'||(payload.format&&typeof payload.format==='object'))request.format=payload.format;
  const r=await ollama('/api/chat',{method:'POST',body:JSON.stringify(request),signal:controller.signal});
  const reader=r.body.getReader(),decoder=new TextDecoder();let buffer='',receivedBytes=0,receivedChars=0;const chunks=[];
  const consumeLine=(line)=>{if(!line.trim())return;let obj;try{obj=JSON.parse(line)}catch{return}if(obj.error)throw new Error(String(obj.error));if(typeof obj.message?.content==='string'&&obj.message.content){receivedChars+=obj.message.content.length;if(receivedChars>8_000_000)throw new Error('Odpověď lokálního modelu překročila bezpečný limit 8 MB.');chunks.push(obj.message.content);if(!event.sender.isDestroyed())event.sender.send('ollama:chunk',{requestId,text:obj.message.content})}};
  try{while(true){const {value,done}=await reader.read();if(done)break;receivedBytes+=value.byteLength;if(receivedBytes>12_000_000){await reader.cancel().catch(()=>{});throw new Error('Odpověď lokálního modelu překročila limit 12 MB.');}buffer+=decoder.decode(value,{stream:true});const lines=buffer.split('\n');buffer=lines.pop();for(const line of lines)consumeLine(line)}buffer+=decoder.decode();if(buffer.trim())consumeLine(buffer);return chunks.join('');}
  catch(error){await reader.cancel().catch(()=>{});throw error}
  finally{try{reader.releaseLock()}catch{}}
 }catch(e){if(e.name==='AbortError')return '';throw e}finally{const active=activeChats.get(requestId);if(active?.controller===controller)activeChats.delete(requestId)}
});
secureHandle('ollama:stop',async(_event,requestId)=>{const active=activeChats.get(String(requestId));if(!active)return false;active.controller.abort();return true});
secureHandle('ollama:benchmark',async(_event,name)=>{
 if(!isValidOllamaModelName(name))throw new Error('Neplatný název modelu.');
 const started=Date.now();const r=await ollama('/api/chat',{method:'POST',signal:AbortSignal.timeout(120000),body:JSON.stringify({model:name,stream:true,messages:[{role:'user',content:'Odpověz česky stručně: napiš číslovaný seznam pěti tipů, jak bezpečně a efektivně organizovat vývojový projekt.'}],options:{temperature:0.2,num_predict:160,num_ctx:2048}})});
 const reader=r.body.getReader(),decoder=new TextDecoder();let buffer='',receivedBytes=0,evalCount=0,evalDuration=0,promptCount=0,promptDuration=0,chars=0;
 const consumeBenchmarkLine=line=>{if(!line.trim())return;let obj;try{obj=JSON.parse(line)}catch{return}if(obj.error)throw new Error(String(obj.error));chars+=obj.message?.content?.length||0;if(obj.done){evalCount=Number(obj.eval_count)||0;evalDuration=Number(obj.eval_duration)||0;promptCount=Number(obj.prompt_eval_count)||0;promptDuration=Number(obj.prompt_eval_duration)||0}};
 try{while(true){const {value,done}=await reader.read();if(done)break;receivedBytes+=value.byteLength;if(receivedBytes>2_000_000){await reader.cancel().catch(()=>{});throw new Error('Benchmark překročil bezpečný limit 2 MB.')}buffer+=decoder.decode(value,{stream:true});const lines=buffer.split('\n');buffer=lines.pop()||'';for(const line of lines)consumeBenchmarkLine(line)}buffer+=decoder.decode();if(buffer.trim())consumeBenchmarkLine(buffer)}finally{reader.releaseLock()}
 const elapsedMs=Date.now()-started;const metrics=OllamaMetrics.fromDoneMessage({eval_count:evalCount,prompt_eval_count:promptCount,eval_duration:evalDuration,prompt_eval_duration:promptDuration});return {model:name,elapsedMs,evalCount:metrics.evalTokens??0,promptCount:metrics.promptTokens??0,tokensPerSecond:metrics.tokensPerSecond,promptTokensPerSecond:metrics.promptTokensPerSecond,metrics,outputCharacters:chars,measuredAt:new Date().toISOString()};
});
secureHandle('system:info',async()=>({platform:process.platform,arch:process.arch,ramGB:Math.round(os.totalmem()/1024**3),cpu:os.cpus()[0]?.model||'Unknown CPU',cores:os.cpus().length,freeRamGB:Math.round(os.freemem()/1024**3)}));

const HOST_TOOL_SPECS = Object.freeze({
 node: { names: ['node.exe', 'node'], args: ['--version'] },
 python: { names: ['python.exe', 'python3.exe', 'python', 'python3'], args: ['--version'] },
 git: { names: ['git.exe', 'git'], args: ['--version'] },
 dotnet: { names: ['dotnet.exe', 'dotnet'], args: ['--version'] },
 java: { names: ['java.exe', 'java'], args: ['-version'] },
 go: { names: ['go.exe', 'go'], args: ['version'] },
 rust: { names: ['rustc.exe', 'rustc'], args: ['--version'] },
 vscode: { names: ['Code.exe', 'code.exe', 'code'], args: ['--version'] },
 godot: { names: ['godot.exe', 'godot4.exe', 'Godot.exe', 'godot'], args: ['--version'] },
 blender: { names: ['blender.exe', 'blender'], args: ['--version'] }
});
function dedupePaths(values) {
 const seen = new Set(); const result = [];
 for (const value of values.filter(Boolean)) { const key = process.platform === 'win32' ? path.resolve(value).toLowerCase() : path.resolve(value); if (!seen.has(key)) { seen.add(key); result.push(path.resolve(value)); } }
 return result;
}
async function findHostExecutable(id, configuredPath = null) {
 const spec = HOST_TOOL_SPECS[id]; if (!spec) return null;
 const candidates = [];
 if (configuredPath) candidates.push(configuredPath);
 const pathDirs = String(process.env.PATH || '').split(path.delimiter).filter(Boolean);
 for (const dir of pathDirs) for (const name of spec.names) candidates.push(path.join(dir, name));
 const pf = process.env.ProgramFiles || 'C:\\Program Files';
 const pf86 = process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)';
 const local = process.env.LOCALAPPDATA || (process.env.USERPROFILE ? path.join(process.env.USERPROFILE, 'AppData', 'Local') : '');
 const common = {
  node: [path.join(pf,'nodejs','node.exe'),path.join(local,'Programs','nodejs','node.exe')],
  python: [path.join(local,'Programs','Python','Python313','python.exe'),path.join(local,'Programs','Python','Python312','python.exe'),path.join(pf,'Python313','python.exe'),path.join(pf,'Python312','python.exe')],
  git: [path.join(pf,'Git','cmd','git.exe'),path.join(pf,'Git','bin','git.exe')],
  dotnet: [path.join(pf,'dotnet','dotnet.exe'),path.join(pf86,'dotnet','dotnet.exe')],
  java: [path.join(pf,'Java','jdk-21','bin','java.exe'),path.join(pf,'Java','jdk-17','bin','java.exe'),path.join(pf86,'Java','jdk-21','bin','java.exe'),path.join(pf86,'Java','jdk-17','bin','java.exe')],
  go: [path.join(pf,'Go','bin','go.exe')],
  rust: [path.join(process.env.USERPROFILE || '', '.cargo','bin','rustc.exe')],
  vscode: [path.join(local,'Programs','Microsoft VS Code','Code.exe'),path.join(pf,'Microsoft VS Code','Code.exe'),path.join(pf86,'Microsoft VS Code','Code.exe')],
  godot: [path.join(pf,'Godot','godot.exe'),path.join(pf,'Godot Engine','godot.exe')],
  blender: [path.join(pf,'Blender Foundation','Blender','blender.exe')]
 };
 candidates.push(...(common[id] || []));
 if (id === 'python' || id === 'java') {
  const roots = id === 'python'
   ? [path.join(local,'Programs','Python'),path.join(pf,'Python'),path.join(pf86,'Python')]
   : [path.join(pf,'Java'),path.join(pf86,'Java')];
  const executableName = id === 'python' ? 'python.exe' : 'bin\\java.exe';
  for (const root of roots) {
   try { const dirs=await fs.readdir(root,{withFileTypes:true}); for (const dir of dirs.filter(item=>item.isDirectory()).sort((a,b)=>b.name.localeCompare(a.name))) candidates.push(path.join(root,dir.name,executableName)); } catch {}
  }
 }
 if (id === 'blender') {
  const root=path.join(pf,'Blender Foundation');
  try { const dirs=await fs.readdir(root,{withFileTypes:true}); for (const dir of dirs.filter(item=>item.isDirectory()).sort((a,b)=>b.name.localeCompare(a.name))) candidates.push(path.join(root,dir.name,'blender.exe')); } catch {}
 }
 for (const candidate of dedupePaths(candidates)) {
  try { const stat = await fs.stat(candidate); if (stat.isFile()) return candidate; } catch {}
 }
 return null;
}
function probeHostExecutable(executable, args) {
 return new Promise(resolve => {
  let settled = false, output = '', timer = null;
  const finish = result => { if (settled) return; settled = true; if (timer) clearTimeout(timer); resolve(result); };
  let child;
  try { child = spawn(executable, args, { cwd: app.getPath('home'), windowsHide: true, shell: false, stdio: ['ignore','pipe','pipe'] }); }
  catch (error) { finish({ status: 'unverified', path: executable, version: '', note: String(error.message || error).slice(0,180) }); return; }
  const collect = chunk => { output = (output + String(chunk || '')).slice(-1200); };
  child.stdout?.on('data', collect); child.stderr?.on('data', collect);
  timer = setTimeout(() => { try { child.kill(); } catch {} finish({ status: 'unverified', path: executable, version: '', note: 'Ověření verze překročilo limit 2,5 sekundy.' }); }, 2500);
  child.once('error', error => finish({ status: 'unverified', path: executable, version: '', note: String(error.message || error).slice(0,180) }));
  child.once('close', code => { const line = output.split(/\r?\n/).map(v => v.trim()).find(Boolean) || ''; finish(code === 0 ? { status: 'available', path: executable, version: line.slice(0,160) } : { status: 'unverified', path: executable, version: line.slice(0,160), note: `Program vrátil kód ${code ?? 'unknown'} při kontrole verze.` }); });
 });
}
async function inspectHostCompatibility() {
 const editors = await readEditorPaths().catch(() => ({ godot:null, unity:null, vscode:null }));
 const ids = Object.keys(HostCompatibility.TOOL_REQUIREMENTS);
 const results = await Promise.all(ids.map(async id => {
  if (id === 'unity') {
   let unityPath=editors.unity||null;
   if(!unityPath && process.env.ProgramFiles){const root=path.join(process.env.ProgramFiles,'Unity','Hub','Editor');try{const dirs=await fs.readdir(root,{withFileTypes:true});for(const dir of dirs.filter(item=>item.isDirectory()).sort((a,b)=>b.name.localeCompare(a.name))){const candidate=path.join(root,dir.name,'Editor','Unity.exe');try{if((await fs.stat(candidate)).isFile()){unityPath=candidate;break}}catch{}}}catch{}}
   return HostCompatibility.assessTool(id, unityPath ? { path: unityPath } : { status: 'missing' });
  }
  if (id === 'roblox') {
   const local = process.env.LOCALAPPDATA;
   let studioPath = null;
   if (local) { try { const root = path.join(local,'Roblox','Versions'); const dirs = await fs.readdir(root,{withFileTypes:true}); const candidates = []; for (const dir of dirs) if (dir.isDirectory()) candidates.push(path.join(root,dir.name,'RobloxStudioBeta.exe')); for (const candidate of candidates) { try { const stat=await fs.stat(candidate); if(stat.isFile() && (!studioPath || stat.mtimeMs>(await fs.stat(studioPath)).mtimeMs)) studioPath=candidate; } catch {} } } catch {} }
   return HostCompatibility.assessTool(id, studioPath ? {path:studioPath} : {status:'missing'});
  }
  const configured = id === 'vscode' ? editors.vscode : id === 'godot' ? editors.godot : null;
  const executable = await findHostExecutable(id, configured);
  if (!executable) return HostCompatibility.assessTool(id,{status:'missing'});
  const probe = await probeHostExecutable(executable, HOST_TOOL_SPECS[id].args);
  return HostCompatibility.assessTool(id,probe);
 }));
 return { platform:process.platform, arch:process.arch, desktop:{electron:process.versions.electron||'', chromium:process.versions.chrome||'', embeddedNode:process.versions.node||'', appVersion:app.getVersion()}, tools:results, summary:HostCompatibility.summarizeCompatibility(results), checkedAt:new Date().toISOString() };
}
secureHandle('system:compatibility', async () => inspectHostCompatibility());
secureHandle('shell:open',(_e,url)=>{if(typeof url==='string'&&['https://ollama.com/','https://docs.unity3d.com/'].some(prefix=>url.startsWith(prefix)))return shell.openExternal(url);return false});
secureHandle('agent:run-command',async(_e,cmd)=>{if(!cmd||typeof cmd!=='string'||cmd.length>500)throw new Error('Invalid command');return {stdout:'',stderr:'',note:'Command execution is intentionally not enabled in this first preview. Add a reviewed allowlist and approval dialog before enabling.'}});


// External search is isolated in the main process; only validated HTTPS result URLs reach the renderer.
const { decodeSearchText, safeSearchUrl, parseDuckDuckGoHtml } = require('./search-parser');
const webSearchCache=new Map();
const recentSearchUrls=new Set();
function publicHttpsUrl(raw){
 try{
  const u=new URL(String(raw));
  if(u.protocol!=='https:'||!u.hostname||u.username||u.password)return null;
  const host=u.hostname.toLowerCase().replace(/^\[|\]$/g,'');
  if(host==='localhost'||host.endsWith('.localhost')||host.endsWith('.local')||host.endsWith('.internal')||host==='::1'||host==='0.0.0.0'||host.startsWith('127.')||host.startsWith('10.')||host.startsWith('192.168.')||/^169\.254\./.test(host)||/^172\.(1[6-9]|2[0-9]|3[01])\./.test(host)||/^\d{1,3}(\.\d{1,3}){3}$/.test(host)||host.includes(':'))return null;
  return u;
 }catch{return null}
}
secureHandle('web:search',async(_event,query)=>{
 requirePlugin('webResearch');
 const q=String(query||'').trim().slice(0,240);
 if(q.length<2)throw new Error('Zadej alespoň dva znaky.');
 const cacheKey=q.toLowerCase();const cached=webSearchCache.get(cacheKey);if(cached&&Date.now()-cached.at<180000)return {...cached.value,cached:true};
 const endpoints=[
  {url:'https://html.duckduckgo.com/html/',source:'DuckDuckGo'},
  {url:'https://lite.duckduckgo.com/lite/',source:'DuckDuckGo Lite'}
 ];
 const results=[], errors=[];
 const parseResults=(html,source)=>{
  for(const item of parseDuckDuckGoHtml(html,source,10)){
   if(!results.some(existing=>existing.url===item.url))results.push(item);
  }
 };
 for(const endpoint of endpoints){
  try{
   const response=await fetch(endpoint.url,{method:'POST',headers:{'User-Agent':`Mozilla/5.0 AuroraAI/${app.getVersion()}`,'Accept':'text/html,application/xhtml+xml','Content-Type':'application/x-www-form-urlencoded;charset=UTF-8','Referer':'https://duckduckgo.com/'},body:new URLSearchParams({q}),signal:AbortSignal.timeout(12000)});
   if(!response.ok){errors.push(`${endpoint.source}: HTTP ${response.status}`);continue;}
   const html=await readTextBounded(response,1000000);
   parseResults(html,endpoint.source);
   if(results.length>=3)break;
  }catch(e){errors.push(`${endpoint.source}: ${String(e?.message||e).slice(0,160)}`)}
 }
 const value={query:q,results:results.slice(0,10),source:results.length?[...new Set(results.map(x=>x.source))].join(' + '):'DuckDuckGo',diagnostic:results.length?'':errors.length?`Vyhledávač nevrátil použitelné výsledky. ${errors.join(' · ')}`:'Stránka vyhledávače neobsahovala rozpoznatelné výsledky; může jít o změnu HTML nebo blokování požadavku.'};
 for(const item of value.results)recentSearchUrls.add(item.url);
 while(recentSearchUrls.size>100)recentSearchUrls.delete(recentSearchUrls.values().next().value);
 webSearchCache.set(cacheKey,{at:Date.now(),value});if(webSearchCache.size>30)webSearchCache.delete(webSearchCache.keys().next().value);
 return value;
});
secureHandle('web:open-result',async(_event,raw)=>{
 const url=safeSearchUrl(raw);
 if(!url)throw new Error('Odkaz nebyl z bezpečnostních důvodů povolen.');
 await shell.openExternal(url);
 return {opened:true};
});
async function assertPublicHost(host){
 const normalized=String(host||'').toLowerCase().replace(/^\[|\]$/g,'');
 if(!normalized||normalized==='localhost'||normalized.endsWith('.localhost')||normalized.endsWith('.local')||normalized.endsWith('.internal'))throw new Error('Soukromá adresa není povolená.');
 let addresses;
 try{addresses=net.isIP(normalized)?[{address:normalized,family:net.isIP(normalized)}]:await dns.lookup(normalized,{all:true,verbatim:true})}catch{throw new Error('Doménu zdroje se nepodařilo bezpečně ověřit.');}
 if(!addresses.length)throw new Error('Doména nemá veřejnou IP adresu.');
 for(const item of addresses){const ip=item.address.toLowerCase();if(item.family===4){const parts=ip.split('.').map(Number);if(parts[0]===0||parts[0]===10||parts[0]===127||parts[0]>=224||parts[0]===169&&parts[1]===254||parts[0]===172&&parts[1]>=16&&parts[1]<=31||parts[0]===192&&parts[1]===168||parts[0]===100&&parts[1]>=64&&parts[1]<=127)throw new Error('Zdroj se překládá na neveřejnou IP adresu.');}else if(!ip.startsWith('2')&&!ip.startsWith('3'))throw new Error('Neveřejná IPv6 adresa není povolená.');}
}
const { readTextBounded, readJsonBounded } = BoundedResponse;
secureHandle('web:fetch-pages',async(_event,rawUrls)=>{
 requirePlugin('webResearch');
 if(!Array.isArray(rawUrls)||rawUrls.length<1||rawUrls.length>5)throw new Error('Vyber jeden až pět zdrojů.');
 const urls=[...new Set(rawUrls.map(raw=>{const url=safeSearchUrl(raw);if(!url||!recentSearchUrls.has(url))throw new Error('Zdroj není z nedávných výsledků vyhledávání. Vyhledej ho znovu.');return url;}))];
 const decodeHtml=value=>String(value||'').replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi,' ').replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi,' ').replace(/<(?:nav|footer|header|aside|noscript|svg|form)\b[^>]*>[\s\S]*?<\/(?:nav|footer|header|aside|noscript|svg|form)>/gi,' ').replace(/<[^>]+>/g,' ').replace(/&nbsp;|&#160;/gi,' ').replace(/&amp;/gi,'&').replace(/&quot;|&#34;/gi,'"').replace(/&#39;|&apos;/gi,"'").replace(/&lt;/gi,'<').replace(/&gt;/gi,'>').replace(/&#(\d+);/g,(_,n)=>String.fromCodePoint(Math.min(0x10ffff,Number(n)||32))).replace(/&#x([0-9a-f]+);/gi,(_,n)=>String.fromCodePoint(Math.min(0x10ffff,parseInt(n,16)||32))).replace(/[\t\r ]+/g,' ').replace(/\n\s+/g,'\n').trim();
 async function fetchPublicPage(initial){
  let current=publicHttpsUrl(initial);if(!current)throw new Error('Neplatná veřejná HTTPS adresa.');
  for(let hop=0;hop<4;hop++){
   await assertPublicHost(current.hostname);
   const response=await fetch(current,{redirect:'manual',headers:{'User-Agent':`Mozilla/5.0 AuroraAI/${app.getVersion()}`,'Accept':'text/html,application/xhtml+xml'},signal:AbortSignal.timeout(9000)});
   if([301,302,303,307,308].includes(response.status)){const location=response.headers.get('location');if(!location)throw new Error('Zdroj má neplatné přesměrování.');const next=publicHttpsUrl(new URL(location,current).toString());if(!next)throw new Error('Zdroj přesměrovává na nepovolenou adresu.');current=next;continue;}
   if(!response.ok)throw new Error(`Zdroj vrátil HTTP ${response.status}.`);
   const type=response.headers.get('content-type')||'';if(!/text\/html|application\/xhtml\+xml/i.test(type))throw new Error('Zdroj není HTML stránka.');
   const html=await readTextBounded(response,1000000);const title=decodeHtml(html.match(/<title[^>]*>([\s\S]*?)<\/title>/i)?.[1]||current.hostname);
   const body=html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i)?.[1]||html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/i)?.[1]||html.match(/<body\b[^>]*>([\s\S]*?)<\/body>/i)?.[1]||html;
   const readable=body.replace(/<(script|style|template|noscript|svg|canvas|iframe|nav|footer|header|aside|form)\b[^>]*>[\s\S]*?<\/\1>/gi,' ')
    .replace(/<(?:br|\/p|\/div|\/li|\/h[1-6]|\/article|\/section|\/blockquote|\/tr)\b[^>]*>/gi,'\n')
    .replace(/<\/(?:td|th)\s*>/gi,' · ');
   const content=decodeHtml(readable).slice(0,12000);if(content.length<80)throw new Error('Na stránce se nepodařilo najít dostatek čitelného textu.');
   return {title,url:current.toString(),domain:current.hostname,content};
  }
  throw new Error('Příliš mnoho přesměrování.');
 }
 const results=await Promise.all(urls.map(async url=>{try{return await fetchPublicPage(url)}catch(e){return {url,domain:new URL(url).hostname,error:e.message,content:''}}}));
 return {results,checkedAt:new Date().toISOString()};
});

let comfyWorkflowPath=null;
let comfyWorkflow=null;
let comfyWorkflowError='';
const comfyWorkflowSettingsPath=()=>path.join(app.getPath('userData'),'comfy-workflow.json');
function validateComfyWorkflow(parsed){ return ComfyWorkflow.validate(parsed); }
function workflowDigest(data){return WorkflowIntegrity.sha256(data)}
async function saveComfyWorkflowSelection(filePath,sha256){
 const settingsPath=comfyWorkflowSettingsPath();await fs.mkdir(path.dirname(settingsPath),{recursive:true});
 const temporaryPath=settingsPath+'.tmp-'+require('node:crypto').randomUUID();
 try{await fs.writeFile(temporaryPath,JSON.stringify({path:filePath,sha256,savedAt:new Date().toISOString()}),{encoding:'utf8',flag:'wx'});await fs.rename(temporaryPath,settingsPath)}catch(error){try{await fs.rm(temporaryPath,{force:true})}catch{}throw error}
}
async function loadSavedComfyWorkflow(){
 let raw;
 try{raw=JSON.parse(await fs.readFile(comfyWorkflowSettingsPath(),'utf8'))}catch(error){if(error?.code==='ENOENT')return;comfyWorkflowError=String(error?.message||error).slice(0,500);return}
 try{
  const storedPath=typeof raw?.path==='string'?raw.path:'';if(!storedPath)return;
  const filePath=await fs.realpath(storedPath);const stat=await fs.stat(filePath);
  if(!stat.isFile()||stat.size>5*1024*1024)throw new Error('Uložený workflow už není dostupný nebo je větší než 5 MB.');
  const bytes=await BoundedFile.readFileBounded(fs,filePath,5*1024*1024,'Uložený workflow');
  const digest=WorkflowIntegrity.assertSavedDigest(raw.sha256,bytes);
  const parsed=JSON.parse(bytes.toString('utf8'));validateComfyWorkflow(parsed);comfyWorkflowPath=filePath;comfyWorkflow=parsed;comfyWorkflowError='';
  if(!Object.prototype.hasOwnProperty.call(raw,'sha256')||path.resolve(storedPath)!==path.resolve(filePath)){try{await saveComfyWorkflowSelection(filePath,digest)}catch(error){comfyWorkflowError='Workflow byl načten, ale jeho kontrolní otisk nešlo uložit: '+String(error?.message||error).slice(0,300)}}
 }catch(error){comfyWorkflowPath=null;comfyWorkflow=null;comfyWorkflowError=String(error?.message||error).slice(0,500)}
}
async function loadBundledComfyWorkflow(){
 if(comfyWorkflow)return {loaded:true,alreadyLoaded:true,name:comfyWorkflowPath?path.basename(comfyWorkflowPath):'workflow'};
 const source=path.join(app.getAppPath(),'integrations','workflows','aurora-sd15-api.json');
 const bytes=await BoundedFile.readFileBounded(fs,source,5*1024*1024,'Výchozí Aurora workflow');
 const parsed=JSON.parse(bytes.toString('utf8'));
 const nodes=validateComfyWorkflow(parsed);
 const folder=path.join(app.getPath('userData'),'image-engine');
 await fs.mkdir(folder,{recursive:true});
 const target=path.join(folder,'aurora-sd15-api.json');
 const temporary=target+'.tmp';
 await fs.writeFile(temporary,bytes,{mode:0o600});
 await fs.rename(temporary,target);
 const digest=workflowDigest(bytes);
 await saveComfyWorkflowSelection(target,digest);
 comfyWorkflowPath=target;comfyWorkflow=parsed;comfyWorkflowError='';
 return {loaded:true,alreadyLoaded:false,name:path.basename(target),nodes};
}
secureHandle('image:prepare-engine',async(event)=>{
 if(imageSetupProcess)return {ok:false,alreadyRunning:true,message:'Příprava obrazového enginu už běží.'};
 const installRoot=path.join(app.getPath('userData'),'AuroraImageEngine');
 const model=path.join(installRoot,'ComfyUI','models','checkpoints','v1-5-pruned-emaonly.safetensors');
 const python=path.join(installRoot,'venv','Scripts','python.exe');
 let modelReady=false,pythonReady=false,comfyReady=false;
 try{modelReady=(await fs.stat(model)).size>1000000000}catch{}
 try{pythonReady=(await fs.stat(python)).isFile()}catch{}
 try{comfyReady=(await fs.stat(path.join(installRoot,'ComfyUI','main.py'))).isFile()}catch{}
 const response=await dialog.showMessageBox(mainWindow,{type:'info',title:'Připravit lokální generování obrázků',message:'Aurora automaticky připraví obrazový engine.',detail:`ComfyUI: ${comfyReady?'již stažené':'bude staženo'}\nPython: ${pythonReady?'již připravený':'bude připraveno'}\nModel Stable Diffusion 1.5: ${modelReady?'již stažený':'stáhne se přibližně 4,27 GB'}\n\nNainstaluje se izolované CPU prostředí. Na starších grafických kartách to může generovat pomalu. Doporučené volné místo: alespoň 14 GB.`,buttons:['Připravit automaticky','Zrušit'],defaultId:0,cancelId:1,noLink:true});
 if(response.response!==0)return {ok:false,cancelled:true,message:'Příprava byla zrušena.'};
 const script=app.isPackaged?path.join(process.resourcesPath,'scripts','prepare-image-engine.ps1'):path.join(app.getAppPath(),'scripts','prepare-image-engine.ps1');
 try{await fs.access(script)}catch{throw new Error('Instalační skript obrazového enginu chybí. Zkontroluj úplnost instalace Aurory.');}
 const powershell=process.env.SystemRoot?path.join(process.env.SystemRoot,'System32','WindowsPowerShell','v1.0','powershell.exe'):'powershell.exe';
 return await new Promise((resolve,reject)=>{
  let outputBuffer='';let settled=false;
  const child=spawn(powershell,['-NoLogo','-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-File',script,'-InstallRoot',installRoot,'-StartServer'],{cwd:installRoot,windowsHide:true,shell:false,stdio:['ignore','pipe','pipe']});
  imageSetupProcess=child;
  const emit=(line)=>{const text=String(line||'').trim();if(!text)return;try{if(!event.sender.isDestroyed())event.sender.send('image:setup-progress',text.replace(/^AURORA_PROGRESS\|/,''))}catch{}};
  const finish=(error,result)=>{if(settled)return;settled=true;if(imageSetupProcess===child)imageSetupProcess=null;if(error)reject(error);else resolve(result)};
  child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');
  child.stdout.on('data',chunk=>{outputBuffer+=chunk;const lines=outputBuffer.split(/\r?\n/);outputBuffer=lines.pop()||'';for(const line of lines)emit(line)});
  child.stderr.on('data',chunk=>{outputBuffer+=chunk;const lines=outputBuffer.split(/\r?\n/);outputBuffer=lines.pop()||'';for(const line of lines)emit(line)});
  child.once('error',error=>finish(new Error('Nepodařilo se spustit PowerShell instalátor: '+error.message)));
  child.once('close',async code=>{
   emit(outputBuffer);outputBuffer='';
   if(code!==0){finish(new Error(`Příprava obrazového enginu skončila s kódem ${code}. Otevři lokální logy v ${installRoot}.`));return;}
   try{const workflow=await loadBundledComfyWorkflow();finish(null,{ok:true,workflow,installRoot,message:'ComfyUI, CPU backend a model jsou připravené.'})}
   catch(error){finish(new Error('Engine byl připraven, ale výchozí workflow se nepodařilo načíst: '+error.message))}
  });
 });
});
secureHandle('image:comfy-status',async()=>{
 let available=false,message='ComfyUI není dostupné na 127.0.0.1:8188',device='';
 try{
  const response=await fetch('http://127.0.0.1:8188/system_stats',{signal:AbortSignal.timeout(1800)});
  if(!response.ok)message=`ComfyUI odpovídá HTTP ${response.status}`;
  else{const data=await readJsonBounded(response,500*1024,'Stav ComfyUI');available=true;message='ComfyUI běží lokálně';device=(Array.isArray(data.devices)?data.devices:[]).map(d=>d.name).filter(Boolean).slice(0,16).join(', ')}
 }catch(error){if(error?.message&&!String(error.message).includes('fetch failed')&&!String(error.message).includes('ECONNREFUSED'))message=String(error.message).slice(0,180)}
 let pythonVersion='';let engineInstalled=false;const engineRoot=path.join(app.getPath('userData'),'AuroraImageEngine');const enginePython=path.join(engineRoot,'venv','Scripts','python.exe');const engineModel=path.join(engineRoot,'ComfyUI','models','checkpoints','v1-5-pruned-emaonly.safetensors');try{await fs.access(path.join(engineRoot,'ComfyUI','main.py'));await fs.access(enginePython);const modelStat=await fs.stat(engineModel);engineInstalled=modelStat.size>1000000000;if(engineInstalled){const python=spawn(enginePython,['--version'],{windowsHide:true,stdio:['ignore','pipe','pipe'],shell:false});python.stdout.setEncoding('utf8');python.stderr.setEncoding('utf8');pythonVersion=await new Promise(resolve=>{let text='';const timer=setTimeout(()=>{try{python.kill()}catch{}resolve(text.trim())},1500);python.stdout.on('data',chunk=>text+=chunk);python.stderr.on('data',chunk=>text+=chunk);python.on('close',()=>{clearTimeout(timer);resolve(text.trim())});python.on('error',()=>{clearTimeout(timer);resolve('')})})}}catch{}return {available,message,device,workflowLoaded:!!comfyWorkflow,workflowName:comfyWorkflowPath?path.basename(comfyWorkflowPath):'',workflowError:comfyWorkflowError,nodeVersion:process.versions.node,pythonVersion,engineInstalled,engineSetupRunning:!!imageSetupProcess,engineRoot};
});
secureHandle('image:choose-workflow',async()=>{
 const result=await dialog.showOpenDialog({title:'Vybrat ComfyUI workflow v API formátu',properties:['openFile'],filters:[{name:'Workflow JSON',extensions:['json']}]});
 if(result.canceled||!result.filePaths[0])return {cancelled:true};
 const filePath=await fs.realpath(result.filePaths[0]);
 const stat=await fs.stat(filePath);if(!stat.isFile()||stat.size>5*1024*1024)throw new Error('Workflow musí být běžný soubor do 5 MB.');
 const bytes=await BoundedFile.readFileBounded(fs,filePath,5*1024*1024,'Workflow');
 const parsed=JSON.parse(bytes.toString('utf8'));const nodes=validateComfyWorkflow(parsed);const digest=workflowDigest(bytes);
 await saveComfyWorkflowSelection(filePath,digest);comfyWorkflowPath=filePath;comfyWorkflow=parsed;comfyWorkflowError='';
 return {cancelled:false,name:path.basename(filePath),nodes};
});
secureHandle('image:generate',async(_event,payload)=>{
 requirePlugin('imageGeneration');
 if(!comfyWorkflow)throw new Error('Nejdřív načti ComfyUI workflow v API formátu.');
 const prompt=String(payload?.prompt||'').trim().slice(0,5000);
 const negative=String(payload?.negative||'').trim().slice(0,2000);
 if(prompt.length<3)throw new Error('Napiš podrobnější popis obrázku.');
 const applied=ComfyWorkflow.applyPrompts(comfyWorkflow,prompt,negative);
 const workflow=applied.workflow;
 const warnings=negative&&!applied.hasNegativeTarget?['Workflow nemá samostatný negativní textový uzel napojený do sampleru; negativní prompt nebyl použit.']:[];
 const response=await fetch('http://127.0.0.1:8188/prompt',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({prompt:workflow,client_id:'aurora-ai'}),signal:AbortSignal.timeout(10000)});
 let data;try{data=await readJsonBounded(response,1024*1024,'Odpověď ComfyUI')}catch(error){if(response.ok)throw error;data={error:error.message}}
 if(!response.ok||data.error||!data.prompt_id){const detail=data.error?.message||data.error||data.node_errors;throw new Error(typeof detail==='string'?detail:detail?JSON.stringify(detail):`ComfyUI request failed (${response.status})`)}
 const promptId=String(data.prompt_id);
 const started=Date.now();
 while(Date.now()-started<240000){
  await new Promise(resolve=>setTimeout(resolve,1300));
  const historyResponse=await fetch(`http://127.0.0.1:8188/history/${encodeURIComponent(promptId)}`,{signal:AbortSignal.timeout(5000)});
  if(!historyResponse.ok)continue;
  const history=await readJsonBounded(historyResponse,4*1024*1024,'Historie generování ComfyUI');
  const entry=history[promptId];
  if(!entry)continue;
  const outputs=[];
  for(const [nodeId,node] of Object.entries(entry.outputs||{})){
   for(const image of node.images||[]){
    const filename=String(image.filename||'');
    const subfolder=String(image.subfolder||'');
    if(!filename||filename.includes('..')||filename.includes('/')||filename.includes('\\\\'))continue;
    outputs.push({nodeId,filename,subfolder,type:image.type||'output',url:'http://127.0.0.1:8188/view?'+new URLSearchParams({filename,subfolder,type:image.type||'output'}).toString()});
   }
  }
  if(outputs.length)return {promptId,outputs:outputs.slice(0,8),elapsedMs:Date.now()-started,warnings};
  if(entry.status?.status_str==='error')throw new Error('ComfyUI nedokončilo generování. Zkontroluj konzoli ComfyUI.');
 }
 throw new Error('Generování stále běží, ale vypršel limit čekání. Zkontroluj frontu v ComfyUI.');
});
