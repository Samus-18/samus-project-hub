'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Backup = require('../src/local-memory-backup');
const Memory = require('../src/local-memory');
const A = [1, 0, 0, 0, 0, 0, 0, 0];
const B = [0, 1, 0, 0, 0, 0, 0, 0];
function record(id, embedding = A, extra = {}) {
  return { id, title: `Title ${id}`, text: `Local note ${id}`, embedding, embeddingModel: 'nomic-embed-text', createdAt: 1_700_000_000_000, ...extra };
}

test('local memory backup round-trips a normalized valid backup', () => {
  const backup = Backup.createBackup({ records: [record('one')], embeddingModel: 'nomic-embed-text', exportedAt: '2026-10-30T00:00:00.000Z' });
  assert.equal(backup.format, Backup.FORMAT);
  assert.equal(Backup.parseBackup(JSON.parse(JSON.stringify(backup))).records[0].id, 'one');
  assert.equal(backup.exportedAt, '2026-10-30T00:00:00.000Z');
});

test('local memory backup rejects wrong format and version', () => {
  assert.throws(() => Backup.parseBackup({ format: 'other', version: 1, embeddingModel: 'nomic-embed-text', records: [] }), /platná záloha/);
  assert.throws(() => Backup.parseBackup({ format: Backup.FORMAT, version: 99, embeddingModel: 'nomic-embed-text', records: [] }), /platná záloha/);
});

test('local memory backup rejects unsafe model names and invalid records', () => {
  assert.throws(() => Backup.createBackup({ records: [], embeddingModel: '../secret' }), /neplatný název/);
  assert.throws(() => Backup.createBackup({ records: [{ id: 'bad' }], embeddingModel: 'nomic-embed-text' }), /neplatnou poznámku/);
});

test('local memory backup rejects mixed embedding models and dimensions', () => {
  assert.throws(() => Backup.createBackup({ records: [record('one'), record('two', A, { embeddingModel: 'other-model' })], embeddingModel: 'nomic-embed-text' }), /jiným embedding modelem/);
  assert.throws(() => Backup.createBackup({ records: [record('one', A), record('two', B.slice(0, 7))], embeddingModel: 'nomic-embed-text' }), /neplatnou poznámku/);
});

test('local memory backup rejects duplicate IDs', () => {
  assert.throws(() => Backup.createBackup({ records: [record('duplicate'), record('duplicate')], embeddingModel: 'nomic-embed-text' }), /duplicitní ID/);
});

test('local memory merge overwrites by ID, sorts newest first and caps at limit', () => {
  const base = [record('keep', A, { createdAt: 10 }), record('same', A, { text: 'old', createdAt: 20 })];
  const incoming = [record('same', A, { text: 'new', createdAt: 30 }), record('new', A, { createdAt: 40 })];
  const merged = Backup.mergeRecords(base, incoming);
  assert.deepEqual(merged.map(item => item.id), ['new', 'same', 'keep']);
  assert.equal(merged.find(item => item.id === 'same').text, 'new');
  assert.equal(Backup.mergeRecords([], Array.from({ length: Memory.MAX_MEMORIES + 3 }, (_, i) => record(`id_${i}`, A, { createdAt: i + 1 }))).length, Memory.MAX_MEMORIES);
});

test('local memory merge refuses mixed model families or dimensions', () => {
  assert.throws(() => Backup.mergeRecords([record('first')], [record('second', A, { embeddingModel: 'different' })]), /různé embedding modely/);
  assert.throws(() => Backup.mergeRecords([record('first')], [record('second', [0,1,0,0,0,0,0,0,0])]), /rozměry/);
});

test('main process imports the tested backup helper and handles config rollback', () => {
  const fs = require('node:fs');
  const main = fs.readFileSync(require.resolve('../src/main'), 'utf8');
  assert.match(main, /const LocalMemoryBackup = require\('\.\/local-memory-backup'\)/);
  assert.match(main, /LocalMemoryBackup\.createBackup/);
  assert.match(main, /LocalMemoryBackup\.parseBackup/);
  assert.match(main, /LocalMemoryBackup\.mergeRecords/);
  assert.match(main, /obnovení původního nastavení modelu také selhalo/);
  assert.match(main, /BoundedFile\.readFileBounded\(fs,filename,MAX_LOCAL_MEMORY_STORE_BYTES,'Index lokální paměti'\)/);
});
