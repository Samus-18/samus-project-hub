'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');
const { spawn } = require('node:child_process');

const MAX_OUTPUT_BYTES = 512 * 1024;
const DEFAULT_TIMEOUT_MS = 180_000;

const ACTIONS = Object.freeze({
  'git-status': { label: 'Git · stav pracovního stromu', executable: 'git', args: ['status', '--short'], needs: 'git', timeoutMs: 30_000 },
  'git-diff-check': { label: 'Git · kontrola whitespace chyb', executable: 'git', args: ['diff', '--check'], needs: 'git', timeoutMs: 30_000 },
  'npm-test': { label: 'Node.js · npm test', executable: 'npm', args: ['test'], windowsCommand: 'npm test', needs: 'npm-test', timeoutMs: 300_000, executesProjectScripts: true },
  'npm-check': { label: 'Node.js · npm run check', executable: 'npm', args: ['run', 'check'], windowsCommand: 'npm run check', needs: 'npm-check', timeoutMs: 180_000, executesProjectScripts: true },
  'npm-build': { label: 'Node.js · npm run build', executable: 'npm', args: ['run', 'build'], windowsCommand: 'npm run build', needs: 'npm-build', timeoutMs: 300_000, executesProjectScripts: true },
  'npm-dist': { label: 'Node.js · npm run dist', executable: 'npm', args: ['run', 'dist'], windowsCommand: 'npm run dist', needs: 'npm-dist', timeoutMs: 900_000, executesProjectScripts: true },
  'dotnet-build': { label: '.NET · dotnet build', executable: 'dotnet', args: ['build', '--nologo'], needs: 'dotnet', timeoutMs: 300_000, executesProjectScripts: true },
  'dotnet-test': { label: '.NET · dotnet test', executable: 'dotnet', args: ['test', '--nologo'], needs: 'dotnet', timeoutMs: 300_000, executesProjectScripts: true },
  'godot-import-check': { label: 'Godot · načtení a import projektu', executable: 'godot', args: [], needs: 'godot', timeoutMs: 600_000, executesProjectScripts: true },
  'unity-import-check': { label: 'Unity · načtení projektu a kompilace skriptů', executable: process.platform === 'win32' ? 'Unity.exe' : 'Unity', args: [], needs: 'unity', timeoutMs: 900_000, executesProjectScripts: true },
  'unity-editor-refresh': { label: 'Unity · obnovení Asset Database', executable: process.platform === 'win32' ? 'Unity.exe' : 'Unity', args: [], needs: 'unity', method: 'Aurora.EditorBridge.RefreshAssets', timeoutMs: 900_000, executesProjectScripts: true },
  'unity-scene-audit': { label: 'Unity · kontrola Build Settings scén', executable: process.platform === 'win32' ? 'Unity.exe' : 'Unity', args: [], needs: 'unity', method: 'Aurora.EditorBridge.AuditBuildScenes', timeoutMs: 900_000, executesProjectScripts: true },
  'unity-build-windows': { label: 'Unity · sestavení Windows hry', executable: process.platform === 'win32' ? 'Unity.exe' : 'Unity', args: [], needs: 'unity', method: 'Aurora.EditorBridge.BuildWindowsPlayer', timeoutMs: 1_800_000, executesProjectScripts: true },
  'cargo-check': { label: 'Rust · cargo check', executable: 'cargo', args: ['check', '--all-targets'], needs: 'cargo', timeoutMs: 600_000, executesProjectScripts: true },
  'cargo-test': { label: 'Rust · cargo test', executable: 'cargo', args: ['test'], needs: 'cargo', timeoutMs: 900_000, executesProjectScripts: true },
  'go-test': { label: 'Go · go test ./...', executable: 'go', args: ['test', './...'], needs: 'go', timeoutMs: 900_000, executesProjectScripts: true },
  'go-vet': { label: 'Go · go vet ./...', executable: 'go', args: ['vet', './...'], needs: 'go', timeoutMs: 600_000, executesProjectScripts: true },
  'python-pytest': { label: 'Python · pytest', executable: 'python', args: ['-m', 'pytest', '-q'], needs: 'python-tests', timeoutMs: 900_000, executesProjectScripts: true },
  'python-compile': { label: 'Python · compileall', executable: 'python', args: ['-m', 'compileall', '-q', '.'], needs: 'python', timeoutMs: 300_000, executesProjectScripts: false }
});

async function isRegularFile(file) {
  try { const stat = await fs.lstat(file); return stat.isFile() && !stat.isSymbolicLink(); }
  catch { return false; }
}

async function isDirectory(file) {
  try { const stat = await fs.lstat(file); return stat.isDirectory() && !stat.isSymbolicLink(); }
  catch { return false; }
}

async function inspectProject(root, actionId) {
  const action = ACTIONS[actionId];
  if (!action) throw new Error('Neznámá přednastavená kontrola.');
  const realRoot = await fs.realpath(root);
  if (!(await isDirectory(realRoot))) throw new Error('Vybraný projekt není běžná složka.');
  if (action.needs === 'git') {
    if (!(await isDirectory(path.join(realRoot, '.git'))) && !(await isRegularFile(path.join(realRoot, '.git')))) throw new Error('V kořeni projektu není nalezen Git repozitář.');
  } else if (action.needs.startsWith('npm-')) {
    const packagePath = path.join(realRoot, 'package.json');
    if (!(await isRegularFile(packagePath))) throw new Error('V kořeni projektu není package.json.');
    const manifestStat = await fs.stat(packagePath);
    if (manifestStat.size > 1024 * 1024) throw new Error('package.json překračuje bezpečný limit 1 MB.');
    let manifest;
    try { manifest = JSON.parse(await fs.readFile(packagePath, 'utf8')); }
    catch { throw new Error('package.json není platný JSON.'); }
    const script = actionId === 'npm-test' ? 'test' : actionId === 'npm-check' ? 'check' : actionId === 'npm-dist' ? 'dist' : 'build';
    if (typeof manifest.scripts?.[script] !== 'string' || !manifest.scripts[script].trim()) throw new Error(`package.json neobsahuje skript "${script}".`);
  } else if (action.needs === 'dotnet') {
    const entries = await fs.readdir(realRoot, { withFileTypes: true });
    if (!entries.some(entry => entry.isFile() && /\.(?:sln|csproj)$/i.test(entry.name))) throw new Error('V kořeni projektu není .sln ani .csproj soubor.');
  } else if (action.needs === 'godot') {
    if (!(await isRegularFile(path.join(realRoot, 'project.godot')))) throw new Error('V kořeni projektu nebyl nalezen project.godot.');
  } else if (action.needs === 'unity') {
    if (!(await isDirectory(path.join(realRoot, 'Assets'))) || !(await isRegularFile(path.join(realRoot, 'ProjectSettings', 'ProjectVersion.txt')))) throw new Error('Nejde o rozpoznaný Unity projekt (chybí Assets nebo ProjectSettings/ProjectVersion.txt).');
    if (action.method && !(await isRegularFile(path.join(realRoot, 'Assets', 'Editor', 'AuroraAI', 'AuroraEditorBridge.cs')))) throw new Error('Nejdřív nainstaluj Unity Editor Bridge přes náhled a schválení v kartě Unity.');
  } else if (action.needs === 'cargo') {
    if (!(await isRegularFile(path.join(realRoot, 'Cargo.toml')))) throw new Error('V kořeni projektu nebyl nalezen Cargo.toml.');
  } else if (action.needs === 'go') {
    if (!(await isRegularFile(path.join(realRoot, 'go.mod')))) throw new Error('V kořeni projektu nebyl nalezen go.mod.');
  } else if (action.needs === 'python' || action.needs === 'python-tests') {
    const markerNames = action.needs === 'python-tests' ? ['pyproject.toml', 'pytest.ini', 'setup.cfg', 'tox.ini', 'pytest.yml'] : ['pyproject.toml', 'requirements.txt', 'setup.py', 'setup.cfg'];
    let found = false;
    for (const name of markerNames) if (await isRegularFile(path.join(realRoot, name))) { found = true; break; }
    if (action.needs === 'python-tests' && !found) found = await isDirectory(path.join(realRoot, 'tests'));
    if (!found) throw new Error(action.needs === 'python-tests' ? 'Nenašel jsem pytest konfiguraci ani složku tests.' : 'Nenašel jsem Python projekt (pyproject.toml, requirements.txt, setup.py nebo setup.cfg).');
  }
  return { root: realRoot, action };
}

function makeInvocation(action, platform = process.platform, root = '') {
  if (action.needs === 'godot') return { executable: platform === 'win32' ? 'godot.exe' : 'godot', args: ['--headless', '--path', root, '--editor', '--quit'], shell: false, windowsHide: true };
  if (action.needs === 'unity') {
    const args = ['-batchmode', '-quit', '-projectPath', root, '-logFile', '-'];
    if (action.method) args.push('-executeMethod', action.method);
    return { executable: platform === 'win32' ? 'Unity.exe' : 'Unity', args, shell: false, windowsHide: true };
  }
  if (platform === 'win32' && action.windowsCommand) {
    // The command is a fixed string from this allowlist; project paths are passed only as cwd.
    return { executable: process.env.ComSpec || 'cmd.exe', args: ['/d', '/s', '/c', action.windowsCommand], shell: false, windowsHide: true };
  }
  return { executable: action.executable, args: [...action.args], shell: false, windowsHide: true };
}

function makeChildEnv(source = process.env) {
  const allowed = ['PATH', 'Path', 'SYSTEMROOT', 'SystemRoot', 'WINDIR', 'WINDIR', 'TEMP', 'TMP', 'HOME', 'USERPROFILE', 'APPDATA', 'LOCALAPPDATA', 'COMSPEC', 'ComSpec', 'PATHEXT', 'ProgramFiles', 'ProgramFiles(x86)', 'DOTNET_CLI_HOME', 'NUGET_PACKAGES', 'npm_config_cache'];
  const result = { CI: '1', NO_COLOR: '1', FORCE_COLOR: '0' };
  for (const key of allowed) if (typeof source[key] === 'string' && source[key]) result[key] = source[key];
  return result;
}

function cancelledBeforeSpawn(actionId, startedAt, root = '') {
  const action = ACTIONS[actionId];
  return {
    action: actionId,
    label: action?.label || String(actionId || 'Projektová kontrola'),
    root,
    ok: false,
    cancelled: true,
    exitCode: null,
    signal: null,
    timedOut: false,
    outputTruncated: false,
    stdout: '',
    stderr: '',
    durationMs: Math.max(0, Date.now() - startedAt),
    note: 'Kontrola byla zrušena před spuštěním procesu. Nebyl spuštěn žádný projektový příkaz.'
  };
}

async function runProjectCheck({ root, action: actionId, platform = process.platform, spawnImpl = spawn, timeoutMs, maxOutputBytes = MAX_OUTPUT_BYTES, signal, toolPath, inspectImpl = inspectProject } = {}) {
  const startedAt = Date.now();
  const action = ACTIONS[actionId];
  if (!action) throw new Error('Neznámá přednastavená kontrola.');
  if (signal?.aborted) return cancelledBeforeSpawn(actionId, startedAt);
  let inspected;
  try {
    inspected = await inspectImpl(root, actionId);
  } catch (error) {
    // A cancellation during asynchronous filesystem preflight is a normal
    // cancellation outcome, not a failed project check.
    if (signal?.aborted) return cancelledBeforeSpawn(actionId, startedAt);
    throw error;
  }
  if (signal?.aborted) return cancelledBeforeSpawn(actionId, startedAt, inspected.root);
  const invocation = makeInvocation(action, platform, inspected.root);
  if (['godot', 'unity'].includes(action.needs) && toolPath) {
    if (typeof toolPath !== 'string' || !path.isAbsolute(toolPath)) throw new Error('Cesta ke zvolenému editoru není absolutní.');
    const stat = await fs.lstat(toolPath).catch(error => { throw new Error(`Zvolený editor nelze najít: ${error.message}`); });
    if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('Zvolený editor není běžný spustitelný soubor.');
    const realTool = await fs.realpath(toolPath);
    if (path.resolve(realTool) !== path.resolve(toolPath)) throw new Error('Zvolený editor nesmí být symbolickým odkazem.');
    invocation.executable = realTool;
  }
  const safeLimit = Number.isSafeInteger(maxOutputBytes) && maxOutputBytes > 0 ? Math.min(maxOutputBytes, MAX_OUTPUT_BYTES) : MAX_OUTPUT_BYTES;
  const timeout = Number.isSafeInteger(timeoutMs) && timeoutMs > 0 ? Math.min(timeoutMs, action.timeoutMs) : action.timeoutMs;
  if (signal?.aborted) return cancelledBeforeSpawn(actionId, startedAt, inspected.root);

  return new Promise((resolve, reject) => {
    let child;
    try {
      child = spawnImpl(invocation.executable, invocation.args, {
        cwd: inspected.root,
        env: makeChildEnv(),
        shell: false,
        windowsHide: true,
        detached: platform !== 'win32',
        stdio: ['ignore', 'pipe', 'pipe']
      });
    } catch (error) {
      reject(new Error(`Kontrolu nelze spustit: ${String(error?.message || error).slice(0, 300)}`));
      return;
    }

    let stdout = '', stderr = '', totalBytes = 0, outputTruncated = false, timedOut = false, cancelled = false, settled = false;
    const terminate = () => {
      if (platform === 'win32' && Number.isInteger(child.pid)) {
        try { const killer = spawnImpl('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { shell: false, windowsHide: true, stdio: 'ignore' }); killer.unref?.(); } catch { child.kill('SIGTERM'); }
      } else if (Number.isInteger(child.pid)) { try { process.kill(-child.pid, 'SIGTERM'); } catch { child.kill('SIGTERM'); } }
      else child.kill('SIGTERM');
    };
    const onAbort = () => { if (cancelled || settled) return; cancelled = true; terminate(); };
    const capture = (target, chunk) => {
      const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
      const remaining = Math.max(0, safeLimit - totalBytes);
      totalBytes += buffer.length;
      if (remaining > 0) {
        const text = buffer.subarray(0, remaining).toString('utf8');
        if (target === 'stdout') stdout += text; else stderr += text;
      }
      if (buffer.length > remaining) outputTruncated = true;
    };
    child.stdout?.on('data', chunk => capture('stdout', chunk));
    child.stderr?.on('data', chunk => capture('stderr', chunk));
    const timer = setTimeout(() => { timedOut = true; terminate(); }, timeout);
    signal?.addEventListener('abort', onAbort, { once: true });
    if (signal?.aborted) onAbort();
    const finish = (error, result) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      signal?.removeEventListener('abort', onAbort);
      if (error) reject(error); else resolve(result);
    };
    child.once('error', error => finish(new Error(`Nepodařilo se spustit ${action.executable}: ${String(error?.message || error).slice(0, 240)}`)));
    child.once('close', (code, signal) => finish(null, {
      action: actionId,
      label: action.label,
      root: inspected.root,
      ok: code === 0 && !timedOut && !cancelled,
      cancelled,
      exitCode: Number.isInteger(code) ? code : null,
      signal: signal || null,
      timedOut,
      outputTruncated,
      stdout,
      stderr,
      durationMs: Date.now() - startedAt,
      note: cancelled ? 'Běžící kontrola byla na požádání ukončena.' : timedOut ? 'Kontrola překročila časový limit a byla ukončena.' : action.executesProjectScripts ? 'Tato kontrola načetla projekt nebo spustila jeho build/import skripty pod oprávněními aktuálního uživatele. Neprovádí herní runtime.' : 'Byl spuštěn pouze pevně přednastavený Git příkaz; žádný volný příkaz nebyl přijat.'
    }));
  });
}

module.exports = Object.freeze({ ACTIONS, MAX_OUTPUT_BYTES, DEFAULT_TIMEOUT_MS, inspectProject, makeInvocation, makeChildEnv, runProjectCheck });
