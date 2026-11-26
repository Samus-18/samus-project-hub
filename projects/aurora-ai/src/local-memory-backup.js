'use strict';

const LocalMemory = require('./local-memory');
const FORMAT = 'aurora-local-memory-backup';
const VERSION = 1;

function normalizeEmbeddingModel(value) {
  if (typeof value !== 'string') throw new Error('Záloha musí uvádět textový název embedding modelu.');
  const name = value.trim();
  const normalized = LocalMemory.normalizeMemoryConfig({ enabled: false, embeddingModel: name });
  if (!name || normalized.embeddingModel !== name) throw new Error('Záloha obsahuje neplatný název embedding modelu.');
  return name;
}

function normalizeRecords(records, embeddingModel) {
  if (!Array.isArray(records) || records.length > LocalMemory.MAX_MEMORIES) {
    throw new Error(`Záloha může obsahovat nejvýše ${LocalMemory.MAX_MEMORIES} poznámek.`);
  }
  const clean = records.map(raw => LocalMemory.normalizeMemoryRecord(raw));
  if (clean.some(record => !record)) throw new Error('Záloha obsahuje neplatnou poznámku nebo embedding.');
  if (clean.some(record => record.embeddingModel !== embeddingModel)) {
    throw new Error('Záloha obsahuje poznámky vytvořené jiným embedding modelem.');
  }
  if (new Set(clean.map(record => record.embedding.length)).size > 1) {
    throw new Error('Záloha obsahuje nekompatibilní rozměry embeddingů.');
  }
  if (new Set(clean.map(record => record.id)).size !== clean.length) {
    throw new Error('Záloha obsahuje duplicitní ID poznámek.');
  }
  return clean;
}

function createBackup({ records, embeddingModel, exportedAt = new Date().toISOString() } = {}) {
  const model = normalizeEmbeddingModel(embeddingModel);
  const clean = normalizeRecords(records, model);
  const timestamp = new Date(exportedAt);
  if (!Number.isFinite(timestamp.getTime())) throw new Error('Záloha má neplatný čas exportu.');
  return Object.freeze({ format: FORMAT, version: VERSION, exportedAt: timestamp.toISOString(), embeddingModel: model, records: clean });
}

function parseBackup(payload) {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload) || payload.format !== FORMAT || payload.version !== VERSION) {
    throw new Error('Soubor není platná záloha lokální paměti Aurory.');
  }
  const embeddingModel = normalizeEmbeddingModel(payload.embeddingModel);
  const records = normalizeRecords(payload.records, embeddingModel);
  return Object.freeze({ format: FORMAT, version: VERSION, embeddingModel, records });
}

function mergeRecords(current, incoming) {
  const baseRaw = Array.isArray(current) ? current : [];
  const nextRaw = Array.isArray(incoming) ? incoming : [];
  const base = baseRaw.map(LocalMemory.normalizeMemoryRecord);
  const next = nextRaw.map(LocalMemory.normalizeMemoryRecord);
  if (base.some(record => !record) || next.some(record => !record)) {
    throw new Error('Import obsahuje neplatné záznamy lokální paměti.');
  }
  const all = [...base, ...next];
  const models = new Set(all.map(record => record.embeddingModel));
  if (models.size > 1) throw new Error('Poznámky používají různé embedding modely; před sloučením je nutný export a převod.');
  const dimensions = new Set(all.map(record => record.embedding.length));
  if (dimensions.size > 1) throw new Error('Poznámky mají nekompatibilní rozměry embeddingů.');
  const byId = new Map(base.map(record => [record.id, record]));
  for (const record of next) byId.set(record.id, record);
  return [...byId.values()].sort((a, b) => b.createdAt - a.createdAt).slice(0, LocalMemory.MAX_MEMORIES);
}

module.exports = Object.freeze({ FORMAT, VERSION, createBackup, parseBackup, mergeRecords });
