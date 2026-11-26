'use strict';

const LocalMemory = require('./local-memory');
const BASE = 'http://127.0.0.1:11434';
const MAX_RESPONSE_BYTES = 1024 * 1024;

function makeEmbeddingRequests(model, text) {
  const name = String(model || '').trim();
  const input = String(text || '').trim().slice(0, LocalMemory.MAX_MEMORY_TEXT);
  if (!/^[a-zA-Z0-9._-]+(?::[a-zA-Z0-9._-]+)?$/.test(name) || name.includes('..')) {
    throw new Error('Název embedding modelu není platný.');
  }
  if (!input) throw new Error('Text pro lokální vektor je prázdný.');
  return [
    { path: '/api/embed', body: { model: name, input }, protocol: 'modern' },
    { path: '/api/embeddings', body: { model: name, prompt: input }, protocol: 'legacy' },
  ];
}

function vectorFromResponse(payload) {
  const raw = Array.isArray(payload?.embeddings) ? payload.embeddings[0] : payload?.embedding;
  return LocalMemory.normalizeEmbedding(raw);
}

async function createEmbedding({ fetchImpl = globalThis.fetch, readJsonBounded, readTextBounded, model, text, timeoutMs = 30000 } = {}) {
  if (typeof fetchImpl !== 'function' || typeof readJsonBounded !== 'function' || typeof readTextBounded !== 'function') {
    throw new TypeError('Lokální embedding vyžaduje bezpečného HTTP klienta a omezené čtení odpovědi.');
  }
  const requests = makeEmbeddingRequests(model, text);
  const timeout = Math.max(1000, Math.min(30000, Number(timeoutMs) || 30000));
  let lastError = null;
  for (const request of requests) {
    const response = await fetchImpl(BASE + request.path, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', accept: 'application/json' },
      body: JSON.stringify(request.body),
      signal: AbortSignal.timeout(timeout),
      redirect: 'error',
    });
    if (request.protocol === 'modern' && [404, 405].includes(response.status)) {
      lastError = new Error(`Ollama HTTP ${response.status}: starší embedding endpoint se ověřuje jako záloha.`);
      try { await response.body?.cancel?.(); } catch {}
      continue;
    }
    if (!response.ok) {
      const detail = (await readTextBounded(response, 1000)).slice(0, 500);
      throw new Error(`Lokální embedding přes Ollamu selhal (HTTP ${response.status}). Zkontroluj, že je stažen embedding model „${model}“. ${detail}`.slice(0, 900));
    }
    const payload = await readJsonBounded(response, MAX_RESPONSE_BYTES, 'Embedding lokální paměti');
    const vector = vectorFromResponse(payload);
    if (!vector) throw new Error('Ollama nevrátila platný embedding vektor. Ověř, že model podporuje lokální embedding API.');
    return vector;
  }
  throw new Error(`Tato verze Ollamy nepodporuje ani /api/embed, ani /api/embeddings. ${lastError?.message || ''}`.trim());
}

module.exports = Object.freeze({ BASE, MAX_RESPONSE_BYTES, makeEmbeddingRequests, vectorFromResponse, createEmbedding });
