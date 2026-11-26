const test = require('node:test');
const assert = require('node:assert/strict');
const router = require('../src/model-router');
const now = Date.parse('2026-10-28T00:00:00.000Z');

test('classifies simple, coding and complex tasks', () => {
  assert.equal(router.classifyTask('Kolik je 2 + 2?'), 'simple');
  assert.equal(router.classifyTask('Oprav Python skript a stack trace'), 'coding');
  assert.equal(router.classifyTask('Proveď podrobnou analýzu architektury projektu'), 'complex');
});

test('ignores stale or future-dated benchmark data', () => {
  assert.equal(router.benchmarkIsFresh({ tokensPerSecond: 20, measuredAt: new Date(now - 1000).toISOString() }, now), true);
  assert.equal(router.benchmarkIsFresh({ tokensPerSecond: 20, measuredAt: new Date(now - 31 * 86400000).toISOString() }, now), false);
  assert.equal(router.benchmarkIsFresh({ tokensPerSecond: 20, measuredAt: new Date(now + 10000).toISOString() }, now), false);
});

test('simple prompts prefer small models and fresh benchmark speed breaks ties', () => {
  const models = ['qwen3:4b', 'qwen3:8b', 'tiny:2b'];
  const benchmarks = {
    'qwen3:4b': { tokensPerSecond: 25, measuredAt: new Date(now - 1000).toISOString() },
    'tiny:2b': { tokensPerSecond: 18, measuredAt: new Date(now - 1000).toISOString() },
    'qwen3:8b': { tokensPerSecond: 80, measuredAt: new Date(now - 1000).toISOString() }
  };
  const result = router.chooseModel(models, 'Kolik je 2 + 2?', benchmarks, {}, now);
  assert.equal(result.model, 'qwen3:4b');
  assert.equal(result.usedBenchmark, true);
});

test('low available RAM avoids recommending large models when smaller ones exist', () => {
  const result = router.chooseModel(['qwen3:14b', 'qwen3:4b'], 'Vysvětli toto', {}, { freeRamGB: 2 }, now);
  assert.equal(result.model, 'qwen3:4b');
});

test('coding task prefers code-specialized models and falls back safely', () => {
  const result = router.chooseModel(['qwen3:4b', 'qwen2.5-coder:3b'], 'Oprav Python skript', {}, {}, now);
  assert.equal(result.model, 'qwen2.5-coder:3b');
  assert.equal(result.task, 'coding');
});

test('empty model list returns a safe empty result', () => {
  assert.equal(router.chooseModel([], 'Ahoj').model, '');
});

test('embedding, reranking, and speech models are never selected for chat', () => {
  assert.equal(router.isChatModel('nomic-embed-text'), false);
  assert.equal(router.isChatModel('bge-reranker-v2-m3'), false);
  assert.equal(router.isChatModel('whisper-large-v3'), false);
  assert.equal(router.isChatModel('qwen3:4b'), true);
  const mixed = router.chooseModel(['nomic-embed-text', 'qwen3:4b'], 'Kolik je 2 + 2?', {}, {}, now);
  assert.equal(mixed.model, 'qwen3:4b');
});

test('embedding-only installations return an explicit no-chat-models result', () => {
  const result = router.chooseModel(['nomic-embed-text', 'bge-reranker-v2-m3'], 'Oprav projekt', {}, {}, now);
  assert.equal(result.model, '');
  assert.equal(result.reason, 'no-chat-models');
});
