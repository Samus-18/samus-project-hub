const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { EventEmitter } = require('node:events');
const { PassThrough } = require('node:stream');
const Runner = require('../src/project-check-runner');

async function tempProject() { return fs.mkdtemp(path.join(os.tmpdir(), 'aurora-check-')); }
function fakeSpawn({ code = 0, stdout = 'ok\n', stderr = '', delay = 0 } = {}) {
  return (executable, args, options) => {
    const child = new EventEmitter();
    child.pid = 12345;
    child.stdout = new PassThrough();
    child.stderr = new PassThrough();
    child.kill = () => {};
    child.unref = () => {};
    process.nextTick(() => setTimeout(() => {
      child.stdout.end(stdout);
      child.stderr.end(stderr);
      child.emit('close', code, null);
    }, delay));
    fakeSpawn.last = { executable, args, options };
    return child;
  };
}

test('requires selected project and exact supported action', async () => {
  await assert.rejects(Runner.inspectProject(os.tmpdir(), 'free-shell'), /Neznámá/);
  await assert.rejects(Runner.inspectProject(path.join(os.tmpdir(), 'missing-aurora-project'), 'git-status'));
});

test('npm checks require the corresponding declared package script', async () => {
  const root = await tempProject();
  await fs.writeFile(path.join(root, 'package.json'), JSON.stringify({ scripts: { test: 'node --test' } }));
  await assert.rejects(Runner.inspectProject(root, 'npm-check'), /neobsahuje skript/);
  const inspected = await Runner.inspectProject(root, 'npm-test');
  assert.equal(inspected.action.label, 'Node.js · npm test');
});

test('maps commands to a fixed argument list and never enables a shell', () => {
  const action = Runner.ACTIONS['git-diff-check'];
  const invocation = Runner.makeInvocation(action, 'linux');
  assert.deepEqual(invocation.args, ['diff', '--check']);
  assert.equal(invocation.shell, false);
  const windows = Runner.makeInvocation(Runner.ACTIONS['npm-test'], 'win32');
  assert.equal(path.win32.basename(windows.executable).toLowerCase(), 'cmd.exe');
  assert.equal(windows.args.at(-1), 'npm test');
  assert.equal(windows.shell, false);
});

test('keeps only safe environment variables, not API tokens', () => {
  const env = Runner.makeChildEnv({ PATH: '/bin', HOME: '/home/test', OPENAI_API_KEY: 'secret', AWS_SECRET_ACCESS_KEY: 'secret' });
  assert.equal(env.PATH, '/bin');
  assert.equal(env.HOME, '/home/test');
  assert.equal(env.OPENAI_API_KEY, undefined);
  assert.equal(env.AWS_SECRET_ACCESS_KEY, undefined);
});

test('runs an approved fixed check in the approved root and returns bounded output', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, '.git'));
  const spawnImpl = fakeSpawn({ stdout: ' M src/app.js\n' });
  const result = await Runner.runProjectCheck({ root, action: 'git-status', spawnImpl, platform: 'linux' });
  assert.equal(result.ok, true);
  assert.equal(result.stdout, ' M src/app.js\n');
  assert.equal(fakeSpawn.last.options.cwd, await fs.realpath(root));
  assert.equal(fakeSpawn.last.options.shell, false);
});

test('reports nonzero exit, timeout and output truncation explicitly', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, '.git'));
  const failed = await Runner.runProjectCheck({ root, action: 'git-status', spawnImpl: fakeSpawn({ code: 2, stderr: 'bad' }), platform: 'linux' });
  assert.equal(failed.ok, false);
  assert.equal(failed.exitCode, 2);
  const long = await Runner.runProjectCheck({ root, action: 'git-status', spawnImpl: fakeSpawn({ stdout: 'x'.repeat(100) }), platform: 'linux', maxOutputBytes: 12 });
  assert.equal(long.stdout.length, 12);
  assert.equal(long.outputTruncated, true);
});

test('Godot and Unity import checks require recognizable projects and fixed non-shell arguments', async () => {
  const godot = await tempProject();
  await fs.writeFile(path.join(godot, 'project.godot'), '[application]\n');
  const godotInspection = await Runner.inspectProject(godot, 'godot-import-check');
  const godotInvocation = Runner.makeInvocation(godotInspection.action, 'win32', await fs.realpath(godot));
  assert.equal(godotInvocation.executable, 'godot.exe');
  assert.deepEqual(godotInvocation.args.slice(0, 2), ['--headless', '--path']);
  assert.equal(godotInvocation.args[2], await fs.realpath(godot));
  assert.equal(godotInvocation.shell, false);

  const unity = await tempProject();
  await fs.mkdir(path.join(unity, 'Assets'));
  await fs.mkdir(path.join(unity, 'ProjectSettings'));
  await fs.writeFile(path.join(unity, 'ProjectSettings', 'ProjectVersion.txt'), 'm_EditorVersion: 6000.0.0f1\n');
  const unityInspection = await Runner.inspectProject(unity, 'unity-import-check');
  const unityInvocation = Runner.makeInvocation(unityInspection.action, 'win32', await fs.realpath(unity));
  assert.equal(unityInvocation.executable, 'Unity.exe');
  assert.ok(unityInvocation.args.includes('-batchmode'));
  assert.ok(unityInvocation.args.includes(await fs.realpath(unity)));
  assert.equal(unityInvocation.shell, false);
});

test('supports bounded Rust, Go and Python checks only in matching project roots', async () => {
  const cargo = await tempProject();
  await fs.writeFile(path.join(cargo, 'Cargo.toml'), '[package]\nname="demo"\nversion="0.1.0"\n');
  assert.equal((await Runner.inspectProject(cargo, 'cargo-check')).action.executable, 'cargo');
  assert.deepEqual(Runner.makeInvocation(Runner.ACTIONS['cargo-test'], 'linux').args, ['test']);

  const go = await tempProject();
  await fs.writeFile(path.join(go, 'go.mod'), 'module example.test/demo\n');
  assert.deepEqual((await Runner.inspectProject(go, 'go-test')).action.args, ['test', './...']);

  const python = await tempProject();
  await fs.writeFile(path.join(python, 'pyproject.toml'), '[project]\nname="demo"\n');
  assert.deepEqual((await Runner.inspectProject(python, 'python-pytest')).action.args, ['-m', 'pytest', '-q']);
  assert.deepEqual(Runner.makeInvocation(Runner.ACTIONS['python-compile'], 'linux').args, ['-m', 'compileall', '-q', '.']);
});

test('uses a selected real editor executable without allowing shell arguments', async () => {
  const root = await tempProject();
  await fs.writeFile(path.join(root, 'project.godot'), '[application]\n');
  const selected = path.join(root, 'godot.exe');
  await fs.writeFile(selected, 'fake executable');
  const spawnImpl = fakeSpawn({ stdout: 'Godot import ok\n' });
  const result = await Runner.runProjectCheck({ root, action: 'godot-import-check', toolPath: selected, spawnImpl, platform: 'win32' });
  assert.equal(result.ok, true);
  assert.equal(fakeSpawn.last.executable, selected);
  assert.equal(fakeSpawn.last.options.shell, false);
  assert.ok(fakeSpawn.last.args.includes('--headless'));
  await fs.unlink(selected);
  await fs.symlink(path.join(root, 'missing.exe'), selected);
  await assert.rejects(Runner.runProjectCheck({ root, action: 'godot-import-check', toolPath: selected, spawnImpl, platform: 'win32' }), /běžný spustitelný soubor/);
});

test('cancels a running project check and reports cancellation explicitly', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, '.git'));
  const controller = new AbortController();
  let killed = false;
  let markSpawned;
  const spawned = new Promise(resolve => { markSpawned = resolve; });
  const spawnImpl = () => {
    const child = new EventEmitter();
    child.stdout = new PassThrough();
    child.stderr = new PassThrough();
    child.kill = () => { killed = true; process.nextTick(() => child.emit('close', null, 'SIGTERM')); };
    markSpawned(child);
    return child;
  };
  const pending = Runner.runProjectCheck({ root, action: 'git-status', spawnImpl, platform: 'win32', signal: controller.signal });
  await spawned;
  controller.abort();
  const result = await pending;
  assert.equal(killed, true);
  assert.equal(result.cancelled, true);
  assert.equal(result.ok, false);
  assert.match(result.note, /ukončena/);
});

test('cancellation during project preflight returns a normal cancelled result without spawning', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, '.git'));
  const controller = new AbortController();
  controller.abort();
  let spawnCalls = 0;
  const result = await Runner.runProjectCheck({
    root,
    action: 'git-status',
    platform: 'win32',
    signal: controller.signal,
    spawnImpl: () => { spawnCalls += 1; throw new Error('must not spawn'); }
  });
  assert.equal(result.cancelled, true);
  assert.equal(result.ok, false);
  assert.equal(result.exitCode, null);
  assert.equal(spawnCalls, 0);
  assert.match(result.note, /před spuštěním/);
});

test('cancellation while asynchronous project preflight is pending does not become a build error', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, '.git'));
  const controller = new AbortController();
  let markStarted, releasePreflight;
  const started = new Promise(resolve => { markStarted = resolve; });
  const gate = new Promise(resolve => { releasePreflight = resolve; });
  let spawnCalls = 0;
  const pending = Runner.runProjectCheck({
    root,
    action: 'git-status',
    platform: 'win32',
    signal: controller.signal,
    inspectImpl: async (...args) => { markStarted(); await gate; return Runner.inspectProject(...args); },
    spawnImpl: () => { spawnCalls += 1; throw new Error('must not spawn'); }
  });
  await started;
  controller.abort();
  releasePreflight();
  const result = await pending;
  assert.equal(result.cancelled, true);
  assert.equal(result.ok, false);
  assert.equal(result.exitCode, null);
  assert.equal(spawnCalls, 0);
  assert.match(result.note, /před spuštěním/);
});

test('Unity Editor Bridge actions require the approved helper file and use fixed executeMethod names', async () => {
  const root = await tempProject();
  await fs.mkdir(path.join(root, 'Assets'), { recursive: true });
  await fs.mkdir(path.join(root, 'ProjectSettings'), { recursive: true });
  await fs.writeFile(path.join(root, 'ProjectSettings', 'ProjectVersion.txt'), 'm_EditorVersion: 6000.0.0f1\n');
  await assert.rejects(Runner.inspectProject(root, 'unity-scene-audit'), /Editor Bridge/);
  const bridge = path.join(root, 'Assets', 'Editor', 'AuroraAI', 'AuroraEditorBridge.cs');
  await fs.mkdir(path.dirname(bridge), { recursive: true });
  await fs.writeFile(bridge, 'namespace Aurora { public static class EditorBridge {} }');
  const inspected = await Runner.inspectProject(root, 'unity-scene-audit');
  const invocation = Runner.makeInvocation(inspected.action, 'win32', await fs.realpath(root));
  assert.ok(invocation.args.includes('-executeMethod'));
  assert.ok(invocation.args.includes('Aurora.EditorBridge.AuditBuildScenes'));
  assert.equal(invocation.shell, false);
  assert.deepEqual(Runner.makeInvocation(Runner.ACTIONS['unity-build-windows'], 'win32', root).args.slice(-2), ['-executeMethod', 'Aurora.EditorBridge.BuildWindowsPlayer']);
});
