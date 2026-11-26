'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const BoundedFile = require('../src/bounded-file');

test('bounded file reader returns the exact content within limit', async t => {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-bounded-file-'));
  t.after(() => fs.rm(dir, { recursive: true, force: true }));
  const file = path.join(dir, 'workflow.json');
  const content = Buffer.from('{"ok":true,"name":"workflow"}');
  await fs.writeFile(file, content);
  assert.deepEqual(await BoundedFile.readFileBounded(fs, file, content.length, 'Test'), content);
});

test('bounded file reader probes one byte past the limit and rejects concurrent file growth', async t => {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-bounded-file-'));
  t.after(() => fs.rm(dir, { recursive: true, force: true }));
  const file = path.join(dir, 'large.json');
  await fs.writeFile(file, 'x'.repeat(8192));
  await assert.rejects(BoundedFile.readFileBounded(fs, file, 128, 'Workflow'), /překročil limit/i);
});
