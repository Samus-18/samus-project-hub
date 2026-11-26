'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Client = require('../src/local-memory-client');

const VALID_VECTOR = [1, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125];
function response(status, payload = {}, text = '') {
  return {
    status,
    ok: status >= 200 && status < 300,
    payload,
    text,
    body: { cancelled: false, async cancel() { this.cancelled = true; } },
  };
}
const readJsonBounded = async res => res.payload;
const readTextBounded = async res => res.text;

test('local embedding client uses modern Ollama /api/embed first and normalizes its vectors', async () => {
  const calls = [];
  const vector = await Client.createEmbedding({
    fetchImpl: async (url, options) => {
      calls.push({ url, options });
      return response(200, { embeddings: [VALID_VECTOR] });
    }, readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: 'test', timeoutMs: 8000,
  });
  assert.equal(calls.length, 1);
  assert.equal(calls[0].url, 'http://127.0.0.1:11434/api/embed');
  assert.equal(calls[0].options.redirect, 'error');
  assert.equal(JSON.parse(calls[0].options.body).input, 'test');
  assert.equal(vector.length, VALID_VECTOR.length);
  assert.ok(Math.abs(Math.sqrt(vector.reduce((sum, v) => sum + v * v, 0)) - 1) < 1e-9);
});

test('older Ollama falls back to /api/embeddings with the legacy prompt field after 404', async () => {
  const calls = [];
  const responses = [response(404, {}, 'not found'), response(200, { embedding: VALID_VECTOR })];
  const vector = await Client.createEmbedding({
    fetchImpl: async (url, options) => { calls.push({ url, options }); return responses.shift(); },
    readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: 'legacy compatibility',
  });
  assert.equal(calls.length, 2);
  assert.equal(calls[0].url, 'http://127.0.0.1:11434/api/embed');
  assert.equal(calls[1].url, 'http://127.0.0.1:11434/api/embeddings');
  assert.equal(JSON.parse(calls[1].options.body).prompt, 'legacy compatibility');
  assert.equal(vector.length, VALID_VECTOR.length);
});

test('modern endpoint fallback is restricted to unsupported-route statuses', async () => {
  const calls = [];
  await assert.rejects(Client.createEmbedding({
    fetchImpl: async url => { calls.push(url); return response(500, {}, 'internal server error'); },
    readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: 'test',
  }), /HTTP 500/);
  assert.equal(calls.length, 1);
});

test('legacy endpoint errors remain clear and do not silently fake a memory vector', async () => {
  let call = 0;
  await assert.rejects(Client.createEmbedding({
    fetchImpl: async () => (++call === 1 ? response(404) : response(404, {}, 'legacy missing')),
    readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: 'test',
  }), /HTTP 404|HTTP 404/);
  assert.equal(call, 2);
});

test('embedding client validates model and text before issuing any request', async () => {
  let called = false;
  const fetchImpl = async () => { called = true; throw new Error('must not be called'); };
  await assert.rejects(Client.createEmbedding({ fetchImpl, readJsonBounded, readTextBounded, model: '../secret', text: 'test' }), /Název embedding modelu/);
  await assert.rejects(Client.createEmbedding({ fetchImpl, readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: '   ' }), /prázdný/);
  assert.equal(called, false);
});

test('embedding transport has a fixed loopback origin and bounds timeouts', async () => {
  const optionsSeen = [];
  await Client.createEmbedding({
    fetchImpl: async (url, options) => { optionsSeen.push({ url, options }); return response(200, { embedding: VALID_VECTOR }); },
    readJsonBounded, readTextBounded, model: 'nomic-embed-text', text: 'test', timeoutMs: 999999,
  });
  assert.equal(new URL(optionsSeen[0].url).origin, 'http://127.0.0.1:11434');
  assert.ok(optionsSeen[0].options.signal);
  assert.equal(optionsSeen[0].options.redirect, 'error');
});
