'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Spaces = require('../src/project-spaces');

test('project spaces keep multiple canonical project folders with unique IDs', () => {
  const result = Spaces.normalizeProjectSpaces([
    { id: 'project_00000001', name: 'Game', folderPath: '/work/game', folderName: 'game', updatedAt: 2 },
    { id: 'project_00000002', name: 'Aurora', folderPath: '/work/aurora', folderName: 'aurora', updatedAt: 1 }
  ]);
  assert.equal(result.length, 2);
  assert.equal(result[0].id, 'project_00000001');
});

test('project spaces reject relative paths, malformed IDs, duplicates and unsafe metadata', () => {
  const result = Spaces.normalizeProjectSpaces([
    { id: 'bad', name: 'Bad', folderPath: '/work/bad' },
    { id: 'project_00000003', name: 'Relative', folderPath: 'relative/path' },
    { id: 'project_00000004', name: 'One', folderPath: '/work/one' },
    { id: 'project_00000005', name: 'Duplicate', folderPath: '/work/one' }
  ]);
  assert.equal(result.length, 1);
  assert.equal(result[0].name, 'One');
});

test('adding a project returns an existing record for a duplicate folder rather than duplicating it', () => {
  const first = Spaces.addProjectSpace([], { id: 'project_00000006', name: 'Game', folderPath: '/work/game', folderName: 'game', updatedAt: 1 });
  const second = Spaces.addProjectSpace(first.list, { id: 'project_00000007', name: 'Renamed', folderPath: '/work/game', folderName: 'game', updatedAt: 2 });
  assert.equal(first.created, true);
  assert.equal(second.created, false);
  assert.equal(second.list.length, 1);
  assert.equal(second.project.id, 'project_00000006');
});

test('project rename sanitizes names and remove only removes metadata', () => {
  const base = [{ id: 'project_00000008', name: 'Game', folderPath: '/work/game', folderName: 'game', updatedAt: 1 }];
  const originalFolderPath = Spaces.normalizeProjectSpace(base[0]).folderPath;
  const renamed = Spaces.renameProjectSpace(base, 'project_00000008', ' My / New : Project ');
  assert.equal(renamed.project.name, 'My New Project');
  const removed = Spaces.removeProjectSpace(renamed.list, 'project_00000008');
  assert.equal(removed.project.folderPath, originalFolderPath); // preserve the native canonical path on Windows and POSIX
  assert.equal(removed.list.length, 0);
});

test('public project summaries never expose local folder paths', () => {
  const summary = Spaces.toPublicProjectSpace({ id: 'project_00000009', name: 'My Game', folderPath: '/home/sam/private/game', folderName: 'game' });
  assert.deepEqual(Object.keys(summary).sort(), ['folderName', 'id', 'name', 'updatedAt']);
  assert.equal(JSON.stringify(summary).includes('/home/sam'), false);
});
