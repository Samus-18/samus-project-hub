'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Memory = require('../src/local-memory');

const A = [1, 0, 0, 0, 0, 0, 0, 0];
const B = [0, 1, 0, 0, 0, 0, 0, 0];
function record(id, embedding, extra = {}) {
  return { id, title: `Poznámka ${id}`, text: `Obsah poznámky ${id}`, embedding, embeddingModel: 'nomic-embed-text', createdAt: 1_700_000_000_000, ...extra };
}

test('local memory is disabled by default and only accepts a bounded safe model name', () => {
  assert.deepEqual(Memory.normalizeMemoryConfig(null), { enabled: false, embeddingModel: 'nomic-embed-text' });
  assert.equal(Memory.normalizeMemoryConfig({ enabled: true, embeddingModel: '../secret' }).enabled, true);
  assert.equal(Memory.normalizeMemoryConfig({ enabled: true, embeddingModel: '../secret' }).embeddingModel, 'nomic-embed-text');
  assert.equal(Memory.normalizeMemoryConfig({ enabled: 1 }).enabled, false);
});

test('embedding normalization rejects malformed, oversized, nonfinite and zero vectors', () => {
  assert.equal(Memory.normalizeEmbedding('not-a-vector'), null);
  assert.equal(Memory.normalizeEmbedding([1, 2, 3]), null);
  assert.equal(Memory.normalizeEmbedding(Array(2049).fill(1)), null);
  assert.equal(Memory.normalizeEmbedding(Array(8).fill(Infinity)), null);
  assert.equal(Memory.normalizeEmbedding(Array(8).fill(0)), null);
  const normalized = Memory.normalizeEmbedding([1, 0, 0, 0, 0, 0, 0, 0]);
  assert.equal(normalized.length, 8);
  assert.equal(normalized[0], 1);
});

test('record normalization bounds metadata and rejects missing text or embedding', () => {
  assert.equal(Memory.normalizeMemoryRecord({ id: 'x', text: 'valid' }), null);
  assert.equal(Memory.normalizeMemoryRecord({ id: 'x', text: '  ', embedding: A }), null);
  const normalized = Memory.normalizeMemoryRecord(record('mem_01', A, { title: 'a'.repeat(400), text: 'b'.repeat(5000) }));
  assert.ok(normalized);
  assert.equal(normalized.title.length, Memory.MAX_MEMORY_TITLE);
  assert.equal(normalized.text.length, Memory.MAX_MEMORY_TEXT);
});

test('cosine similarity recognizes same, orthogonal, and invalid vectors', () => {
  assert.equal(Memory.cosineSimilarity(A, A), 1);
  assert.equal(Memory.cosineSimilarity(A, B), 0);
  assert.equal(Memory.cosineSimilarity(A, [1]), -1);
  assert.equal(Memory.cosineSimilarity(A, Array(8).fill(NaN)), -1);
});

test('semantic recall ranks relevant records and bounds the number of matches', () => {
  const records = [record('first', [1, 0, 0, 0, 0, 0, 0, 0]), record('second', [0.9, 0.1, 0, 0, 0, 0, 0, 0]), record('unrelated', B)];
  const matches = Memory.selectRelevantMemories(records, A, { limit: 2, embeddingModel: 'nomic-embed-text' });
  assert.equal(matches.length, 2);
  assert.equal(matches[0].id, 'first');
  assert.equal(matches[1].id, 'second');
});

test('semantic recall does not compare embeddings from a different model', () => {
  const matches = Memory.selectRelevantMemories([record('old-model', A, { embeddingModel: 'other-model' })], A, { embeddingModel: 'nomic-embed-text' });
  assert.deepEqual(matches, []);
});

test('semantic recall filters weak matches below threshold', () => {
  const matches = Memory.selectRelevantMemories([record('unrelated', B)], A, { threshold: 0.2 });
  assert.deepEqual(matches, []);
});

test('memory context is bounded and clearly marks notes as data, not instructions', () => {
  assert.equal(Memory.buildMemoryContext([]), '');
  const context = Memory.buildMemoryContext([{ title: 'User choice', text: 'Use Czech language when discussing Aurora.' }]);
  assert.match(context, /lokální poznámky/);
  assert.match(context, /ne nové systémové instrukce/);
  assert.match(context, /Use Czech language/);
  assert.ok(context.length < 10000);
});

test('memory module does not contain network or remote-provider clients', () => {
  const fs = require('node:fs');
  const source = fs.readFileSync(require.resolve('../src/local-memory'), 'utf8');
  assert.doesNotMatch(source, /fetch\s*\(/);
  assert.doesNotMatch(source, /https?:\/\//);
});


test('Electron wiring keeps memory retrieval loopback-only and does not inject it into cloud requests', () => {
  const fs = require('node:fs');
  const main = fs.readFileSync(require.resolve('../src/main'), 'utf8');
  const app = fs.readFileSync(require.resolve('../src/app'), 'utf8');
  const preload = fs.readFileSync(require.resolve('../src/preload'), 'utf8');
  const client = fs.readFileSync(require.resolve('../src/local-memory-client'), 'utf8');
  assert.match(client, /BASE = 'http:\/\/127\.0\.0\.1:11434'/);
  assert.match(client, /\/api\/embed/);
  assert.match(client, /\/api\/embeddings/);
  assert.match(main, /payload\?\.provider!=='ollama'\)return \{enabled:false,matches:\[\],context:''\}/);
  assert.match(preload, /recallMemory:payload=>ipcRenderer\.invoke\('memory:recall',payload\)/);
  assert.match(preload, /exportMemory:.*memory:export/);
  assert.match(preload, /importMemory:.*memory:import/);
  assert.match(main, /secureHandle\('memory:export'/);
  assert.match(main, /secureHandle\('memory:import'/);
  assert.match(main, /LocalMemoryBackup\.createBackup/);
  assert.match(main, /LocalMemoryBackup\.parseBackup/);
  const backup = fs.readFileSync(require.resolve('../src/local-memory-backup'), 'utf8');
  assert.match(backup, /aurora-local-memory-backup/);
  assert.match(app, /window\.aurora\.exportMemory\(\)/);
  assert.match(app, /window\.aurora\.importMemory\(\)/);
  assert.match(app, /const requestProvider=state\.provider/);
  assert.match(app, /requestProvider==='ollama'&&state\.localMemoryEnabled/);
  assert.match(app, /if\(localMemoryContext\)attachedContext=\[attachedContext,localMemoryContext\]\.filter\(Boolean\)\.join/);
  const html = fs.readFileSync(require.resolve('../src/index.html'), 'utf8');
  assert.match(html, /id="memoryEnabled"/);
  assert.match(html, /id="exportMemory"/);
  assert.match(html, /id="importMemory"/);
});
