'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Integrity = require('../src/workflow-integrity');
const main = fs.readFileSync(path.join(__dirname, '..', 'src', 'main.js'), 'utf8');

test('workflow digest is stable SHA-256 for the exact selected bytes', () => {
  const bytes = Buffer.from('{"1":{"class_type":"CheckpointLoaderSimple"}}', 'utf8');
  assert.equal(Integrity.sha256(bytes), Integrity.sha256(Buffer.from(bytes)));
  assert.match(Integrity.sha256(bytes), /^[a-f0-9]{64}$/);
});

test('saved workflow accepts unchanged content and rejects altered content', () => {
  const original = Buffer.from('{"workflow":"approved"}');
  const digest = Integrity.sha256(original);
  assert.equal(Integrity.assertSavedDigest(digest, original), digest);
  assert.throws(() => Integrity.assertSavedDigest(digest, Buffer.from('{"workflow":"changed"}')), /změnil/i);
  assert.equal(Integrity.assertSavedDigest(undefined, original), digest, 'legacy settings can be migrated after validation');
  assert.throws(() => Integrity.assertSavedDigest('', original), /otisk.*neplatný/i);
  assert.throws(() => Integrity.assertSavedDigest(null, original), /otisk.*neplatný/i);
  assert.throws(() => Integrity.assertSavedDigest('not-a-hash', original), /otisk.*neplatný/i);
});

test('main process verifies persisted workflow digest and bounds local JSON API responses', () => {
  assert.match(main, /WorkflowIntegrity\.assertSavedDigest\(raw\.sha256,bytes\)/);
  assert.match(main, /BoundedFile\.readFileBounded\(fs,filePath,5\*1024\*1024/);
  assert.match(main, /await saveComfyWorkflowSelection\(filePath,digest\)/);
  assert.match(main, /readJsonBounded\(response,500\*1024,'Stav ComfyUI'\)/);
  assert.match(main, /readJsonBounded\(historyResponse,4\*1024\*1024,'Historie generování ComfyUI'\)/);
  assert.match(main, /readJsonBounded\(r,2\*1024\*1024,'Seznam modelů Ollama'\)/);
});
