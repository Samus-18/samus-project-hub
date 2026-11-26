'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { auditUnityProject, getUnityProjectInfo } = require('../src/unity-project-audit');

async function createProject(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-unity-audit-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, 'ProjectSettings'), { recursive: true });
  await fs.mkdir(path.join(root, 'Packages'), { recursive: true });
  await fs.mkdir(path.join(root, 'Assets', 'Scripts'), { recursive: true });
  await fs.writeFile(path.join(root, 'ProjectSettings', 'ProjectVersion.txt'), 'm_EditorVersion: 6000.0.33f1\n', 'utf8');
  await fs.writeFile(path.join(root, 'Packages', 'manifest.json'), JSON.stringify({ dependencies: { 'com.unity.test-framework': '1.4.5', 'com.unity.inputsystem': '1.11.2' } }), 'utf8');
  await fs.writeFile(path.join(root, 'Assets', 'Scripts', 'Player.cs'), 'using UnityEngine;\npublic class Player : MonoBehaviour { // TODO verify movement\n}\n', 'utf8');
  await fs.writeFile(path.join(root, 'Assets', 'Scripts', 'Player.cs.meta'), 'fileFormatVersion: 2\nguid: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n', 'utf8');
  return root;
}

test('Unity audit recognizes a project and reports source markers without executing it', async t => {
  const root = await createProject(t);
  const info = await getUnityProjectInfo(root);
  assert.equal(info.isUnity, true);
  assert.equal(info.version, '6000.0.33f1');
  assert.equal(info.packages.length, 2);
  const result = await auditUnityProject(root);
  assert.equal(result.isUnity, true);
  assert.equal(result.scannedScripts, 1);
  assert.ok(result.findings.some(f => f.code === 'TODO' && f.file.endsWith('Player.cs')));
  assert.match(result.note, /nespouští Unity/);
});

test('Unity audit reports missing project markers and malformed package manifest', async t => {
  const root = await createProject(t);
  await fs.writeFile(path.join(root, 'Packages', 'manifest.json'), '{not json', 'utf8');
  await fs.rm(path.join(root, 'ProjectSettings', 'ProjectVersion.txt'));
  const result = await auditUnityProject(root);
  assert.ok(result.findings.some(f => f.code === 'NO_PROJECT_VERSION'));
  assert.ok(result.findings.some(f => f.code === 'INVALID_MANIFEST'));
});

test('Unity audit flags missing meta files and duplicate GUIDs', async t => {
  const root = await createProject(t);
  await fs.writeFile(path.join(root, 'Assets', 'Scripts', 'Other.cs'), 'public class Other {}', 'utf8');
  await fs.writeFile(path.join(root, 'Assets', 'Scripts', 'Other.cs.meta'), 'fileFormatVersion: 2\nguid: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n', 'utf8');
  await fs.writeFile(path.join(root, 'Assets', 'Loose.txt'), 'hello', 'utf8');
  const result = await auditUnityProject(root);
  assert.ok(result.findings.some(f => f.code === 'DUPLICATE_GUID'));
  assert.ok(result.findings.some(f => f.code === 'MISSING_META' && f.file.endsWith('Loose.txt')));
});

test('Unity audit never follows an Assets directory symlink', async t => {
  const root = await createProject(t);
  const outside = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-unity-outside-'));
  t.after(() => fs.rm(outside, { recursive: true, force: true }));
  await fs.writeFile(path.join(outside, 'Secret.cs'), 'class Secret { }', 'utf8');
  await fs.rm(path.join(root, 'Assets'), { recursive: true, force: true });
  try { await fs.symlink(outside, path.join(root, 'Assets'), 'junction'); }
  catch (error) { if (['EPERM', 'EACCES', 'ENOTSUP', 'EINVAL'].includes(error.code)) return t.skip('Symlinks are not available in this environment.'); throw error; }
  const result = await auditUnityProject(root);
  assert.equal(result.scannedScripts, 0);
  assert.ok(result.findings.some(f => f.code === 'UNSAFE_ASSETS_PATH'));
});

test('Unity audit skips nested symlinks and does not read outside project files', async t => {
  const root = await createProject(t);
  const outside = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-unity-outside-'));
  t.after(() => fs.rm(outside, { recursive: true, force: true }));
  await fs.writeFile(path.join(outside, 'Escape.cs'), '// TODO SECRET_OUTSIDE_PROJECT', 'utf8');
  try { await fs.symlink(outside, path.join(root, 'Assets', 'Scripts', 'External'), 'junction'); }
  catch (error) { if (['EPERM', 'EACCES', 'ENOTSUP', 'EINVAL'].includes(error.code)) return t.skip('Symlinks are not available in this environment.'); throw error; }
  const result = await auditUnityProject(root);
  assert.ok(result.findings.some(f => f.code === 'SYMLINK_SKIPPED'));
  assert.ok(!result.findings.some(f => f.message.includes('SECRET_OUTSIDE_PROJECT')));
});


test('Unity project info reports oversized metadata distinctly without parsing it as valid', async t => {
  const root = await createProject(t);
  await fs.writeFile(path.join(root, 'Packages', 'manifest.json'), JSON.stringify({ dependencies: {} }) + ' '.repeat(260_000), 'utf8');
  const info = await getUnityProjectInfo(root);
  assert.equal(info.manifestStatus, 'too-large');
  assert.equal(info.hasManifest, false);
  const result = await auditUnityProject(root);
  assert.ok(result.findings.some(f => f.code === 'MANIFEST_SIZE_LIMIT'));
  assert.ok(!result.findings.some(f => f.code === 'INVALID_MANIFEST'));
  assert.ok(!result.findings.some(f => f.code === 'UNSAFE_MANIFEST'));
});
