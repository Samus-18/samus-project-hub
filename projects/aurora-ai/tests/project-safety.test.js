'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const os = require('node:os');
const ProjectSafety = require('../src/project-safety');
const root = path.resolve(__dirname, '..');
const read = file => require('node:fs').readFileSync(path.join(root, file), 'utf8');

test('project target resolver rejects lexical traversal and accepts nested project files', () => {
  const project = path.join(os.tmpdir(), 'aurora-safe-project');
  const nested = ProjectSafety.resolveProjectTarget(project, path.join('src', 'main.js'));
  assert.equal(nested.target, path.join(project, 'src', 'main.js'));
  assert.equal(nested.relative, path.join('src', 'main.js'));
  assert.throws(() => ProjectSafety.resolveProjectTarget(project, '..' + path.sep + 'outside.txt'), /mimo schválenou složku/i);
  assert.throws(() => ProjectSafety.resolveProjectTarget(project, project), /mimo schválenou složku/i);
});

test('single-file approval must still target the currently approved project root', () => {
  const first = path.join(os.tmpdir(), 'aurora-project-a');
  const second = path.join(os.tmpdir(), 'aurora-project-b');
  assert.equal(ProjectSafety.assertSameApprovedRoot(first, path.resolve(first)), path.resolve(first));
  assert.throws(() => ProjectSafety.assertSameApprovedRoot(first, second), /změnila/i);
  assert.throws(() => ProjectSafety.assertSameApprovedRoot(first, null), /změnila/i);
});

test('backup names are unique, portable and still map back to the original file', () => {
  const target = path.join(os.tmpdir(), 'aurora-backup-project', 'src', 'main.js');
  const fixed = new Date('2026-10-29T20:15:12.123Z');
  const one = ProjectSafety.makeBackupPath(target, fixed, 101);
  const two = ProjectSafety.makeBackupPath(target, fixed, 202);
  assert.notEqual(one, two);
  const relative = path.relative(path.dirname(path.dirname(target)), one);
  // Resolve with the actual project root so both timestamp and numeric suffix are checked.
  const rootPath = path.join(os.tmpdir(), 'aurora-backup-project');
  const backupRelative = path.relative(rootPath, one);
  const parsed = ProjectSafety.originalFromBackup(rootPath, backupRelative, new Set(['.js']));
  assert.equal(parsed.originalTarget, target);
  assert.match(path.basename(one), /\.aurora-backup-2026-10-29T20-15-12-123Z-101$/);
  assert.throws(() => ProjectSafety.makeBackupPath(target, fixed, -1), /identifikátor/i);
});

test('backup validator accepts legitimate Aurora backup files and rejects forged suffixes/extensions', () => {
  const project = path.join(os.tmpdir(), 'aurora-backup-project');
  const valid = ProjectSafety.originalFromBackup(project, path.join('src', 'main.js.aurora-backup-2026-10-29T20-15-12-123Z'), new Set(['.js', '.md']));
  assert.equal(valid.originalTarget, path.join(project, 'src', 'main.js'));
  assert.equal(valid.originalRelative.split(path.sep).join('/'), 'src/main.js');
  assert.throws(() => ProjectSafety.originalFromBackup(project, 'src/main.js.aurora-backup-../../outside', new Set(['.js'])), /platné zálohy/i);
  assert.throws(() => ProjectSafety.originalFromBackup(project, 'src/image.png.aurora-backup-2026-10-29T20-15-12-123Z', new Set(['.js'])), /povolenému typu/i);
  assert.throws(() => ProjectSafety.originalFromBackup(project, '../outside.js.aurora-backup-2026-10-29T20-15-12-123Z', new Set(['.js'])), /mimo schválenou složku/i);
});

test('mutation path guard accepts a normal file and never creates paths as a side effect', async t => {
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-project-safety-'));
  t.after(() => fs.rm(temporary, { recursive: true, force: true }));
  const project = path.join(temporary, 'project');
  await fs.mkdir(path.join(project, 'src'), { recursive: true });
  const file = path.join(project, 'src', 'app.js');
  await fs.writeFile(file, 'const safe = true;', 'utf8');
  const result = await ProjectSafety.assertSafeMutationPath(fs, project, file);
  assert.equal(result.exists, true);
  const missing = path.join(project, 'new', 'folder', 'file.js');
  const missingResult = await ProjectSafety.assertSafeMutationPath(fs, project, missing);
  assert.equal(missingResult.exists, false);
  await assert.rejects(fs.access(path.dirname(missing)), { code: 'ENOENT' });
  await assert.rejects(ProjectSafety.assertSafeMutationPath(fs, project, path.join(temporary, 'outside.js')), /mimo schválenou složku/i);
});

test('mutation path guard refuses symlink ancestors and symlink target files where supported', async t => {
  const temporary = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-project-symlink-'));
  t.after(() => fs.rm(temporary, { recursive: true, force: true }));
  const project = path.join(temporary, 'project');
  const outside = path.join(temporary, 'outside');
  await fs.mkdir(project, { recursive: true });
  await fs.mkdir(outside, { recursive: true });
  await fs.writeFile(path.join(outside, 'existing.js'), 'outside', 'utf8');
  try {
    await fs.symlink(outside, path.join(project, 'linked-dir'), process.platform === 'win32' ? 'junction' : 'dir');
    await fs.symlink(path.join(outside, 'existing.js'), path.join(project, 'linked.js'), 'file');
  } catch (error) {
    if (['EPERM', 'EACCES', 'ENOTSUP', 'UNKNOWN'].includes(error.code)) return t.skip('Host OS does not permit creating test symlinks.');
    throw error;
  }
  await assert.rejects(ProjectSafety.assertSafeMutationPath(fs, project, path.join(project, 'linked-dir', 'new.js')), /symbolick/i);
  await assert.rejects(ProjectSafety.assertSafeMutationPath(fs, project, path.join(project, 'linked.js')), /symbolick/i);
  await assert.rejects(fs.access(path.join(outside, 'new.js')), { code: 'ENOENT' });
});

test('Electron project edit and rollback handlers enforce the tested safety helpers', () => {
  const main = read('src/main.js');
  assert.match(main, /secureHandle\('project:approve-edit',[\s\S]{0,300}requirePlugin\('projectWrites'\)[\s\S]{0,300}assertSameApprovedRoot/);
  assert.match(main, /secureHandle\('project:rollback',[\s\S]{0,180}requirePlugin\('projectWrites'\)/);
  assert.match(main, /assertSafeMutationPath\(fs,root,item\.target\)[\s\S]{0,100}fs\.mkdir\(parent,\{recursive:true\}\)/);
  assert.match(main, /originalFromBackup\(approvedProjectRoot,relativeBackup,SAFE_TEXT_EXTS\)/);
  assert.match(main, /fs\.rename\(temp,originalTarget\)/);
  assert.match(main, /ProjectMutationGate\.createSerialMutationGate\(\)/);
  assert.match(main, /withProjectMutationLock/);
  assert.match(main, /projectMutationGate\.isLocked\(key\)/);
  assert.match(main, /cancel-edit',[\s\S]{0,160}busy:true/);
  assert.match(main, /cancel-batch',[\s\S]{0,160}busy:true/);
  const app = read('src/app.js');
  assert.match(app, /outcome\?\.busy[\s\S]{0,100}návrh nelze zrušit/i);
  assert.match(app, /outcome\?\.busy[\s\S]{0,100}zrušení není možné/i);
  assert.match(main, /project:approve-batch',[\s\S]{0,120}withProjectMutationLock/);
  assert.match(main, /project:approve-edit',[\s\S]{0,120}withProjectMutationLock/);
  assert.match(main, /project:rollback',[\s\S]{0,120}withProjectMutationLock/);
  assert.match(main, /Hromadná změna selhala:[\s\S]{0,200}Zkontroluj tyto soubory ručně/);
});
