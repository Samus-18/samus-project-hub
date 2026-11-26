'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { auditGodotProject, cleanResourcePath, parseVersion } = require('../src/godot-project-audit');

async function fixture(fn) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-godot-audit-'));
  try { await fn(root); } finally { await fs.rm(root, {recursive:true, force:true}); }
}
async function write(root, rel, content) {
  const target = path.join(root, rel);
  await fs.mkdir(path.dirname(target), {recursive:true});
  await fs.writeFile(target, content, 'utf8');
}

const project = (main='res://scenes/main.tscn', features='PackedStringArray("4.3", "GL Compatibility")') =>
  `[application]\nconfig/name="Starfall Test"\nrun/main_scene="${main}"\n\n[config]\nconfig_version=5\nconfig/features=${features}\n`;

test('Godot project audit recognizes project metadata and resolves a healthy main scene', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene load_steps=2 format=3]\n[ext_resource type="Script" path="res://scripts/player.gd" id="1"]\n[node name="Main" type="Node"]\n');
    await write(root, 'scripts/player.gd', 'extends Node\nfunc _ready():\n    print("ready")\n');
    const report = await auditGodotProject(root);
    assert.equal(report.isGodot, true);
    assert.equal(report.projectName, 'Starfall Test');
    assert.equal(report.engineVersion, '4.3');
    assert.equal(report.mainScene, 'res://scenes/main.tscn');
    assert.equal(report.scannedScripts, 1);
    assert.equal(report.scannedScenes, 1);
    assert.equal(report.counts.error, 0);
    assert.equal(report.findings.some(item => item.code === 'MISSING_RESOURCE'), false);
  });
});

test('Godot audit reports an unset or missing main scene without executing the project', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project('res://scenes/nope.tscn'));
    const report = await auditGodotProject(root);
    assert.equal(report.isGodot, true);
    assert.ok(report.findings.some(item => item.code === 'MISSING_MAIN_SCENE' && item.severity === 'error'));
    assert.match(report.note, /Godot se nespouští/);
  });
});

test('Godot audit finds missing resources referenced by scene and resource files', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene format=3]\n[ext_resource type="Script" path="res://scripts/removed.gd" id="1"]\n');
    const report = await auditGodotProject(root);
    assert.ok(report.findings.some(item => item.code === 'MISSING_RESOURCE' && item.message.includes('scripts/removed.gd')));
  });
});

test('Godot audit detects common version-syntax mismatches as heuristic warnings', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene format=3]\n');
    await write(root, 'scripts/legacy.gd', 'extends Node\nonready var player = null\n# TODO revisit\n');
    const report = await auditGodotProject(root);
    assert.ok(report.findings.some(item => item.code === 'GODOT3_SYNTAX'));
    assert.ok(report.findings.some(item => item.code === 'TODO_MARKER'));
  });
});

test('Godot audit does not follow symlinks and rejects traversal-like resource paths', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene format=3]\n[ext_resource type="Script" path="res://../outside.gd" id="1"]\n[ext_resource type="Script" path="res://scripts/linked.gd" id="2"]\n');
    await write(root, 'scripts/real.gd', 'extends Node\n');
    try { await fs.symlink(path.join(root, 'scripts/real.gd'), path.join(root, 'scripts/linked.gd')); } catch { return; }
    const report = await auditGodotProject(root);
    assert.ok(report.findings.some(item => item.code === 'UNSAFE_RESOURCE_PATH'));
    assert.ok(report.findings.some(item => item.code === 'UNSAFE_RESOURCE' && item.message.includes('res://scripts/linked.gd')));
  });
});

test('Godot resource normalization rejects absolute, traversal and empty components', () => {
  assert.equal(cleanResourcePath('res://scene/main.tscn'), 'scene/main.tscn');
  assert.equal(cleanResourcePath('res://../secret.txt'), null);
  assert.equal(cleanResourcePath('res:///secret.txt'), null);
  assert.equal(cleanResourcePath('res://folder//file.gd'), null);
  assert.deepEqual(parseVersion('config/features=PackedStringArray("4.4", "Forward Plus")'), {major:4, version:'4.4'});
});


test('large existing Godot scenes and resources are not falsely reported as missing', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene format=3]\n' + ' '.repeat(420_000));
    await write(root, 'scenes/secondary.tscn', '[gd_scene format=3]\n[ext_resource type=\"Resource\" path=\"res://assets/large.tres\" id=\"1\"]\n');
    await write(root, 'assets/large.tres', '[gd_resource type=\"Resource\" format=3]\n' + ' '.repeat(420_000));
    const report = await auditGodotProject(root);
    assert.equal(report.findings.some(item => item.code === 'MISSING_MAIN_SCENE'), false);
    assert.equal(report.findings.some(item => item.code === 'MISSING_RESOURCE'), false);
    assert.ok(report.findings.some(item => item.code === 'MAIN_SCENE_SIZE_LIMIT'));
    assert.ok(report.findings.some(item => item.code === 'RESOURCE_SIZE_LIMIT'));
  });
});

test('Godot audit rejects references that pass through a symlinked parent directory', async () => {
  await fixture(async root => {
    await write(root, 'project.godot', project());
    await write(root, 'scenes/main.tscn', '[gd_scene format=3]\n[ext_resource type="Script" path="res://linked-scripts/player.gd" id="1"]\n');
    await write(root, 'real-scripts/player.gd', 'extends Node\n');
    try { await fs.symlink(path.join(root, 'real-scripts'), path.join(root, 'linked-scripts'), 'dir'); } catch { return; }
    const report = await auditGodotProject(root);
    assert.ok(report.findings.some(item => item.code === 'UNSAFE_RESOURCE' && item.message.includes('res://linked-scripts/player.gd')));
    assert.equal(report.findings.some(item => item.code === 'MISSING_RESOURCE' && item.message.includes('linked-scripts/player.gd')), false);
  });
});
