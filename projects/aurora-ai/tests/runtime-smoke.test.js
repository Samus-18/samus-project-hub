'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const {
  normalizeLocalBase,
  extractOllamaModels,
  inspectOllama,
  inspectComfyUI,
  fetchBoundedJson
} = require('../scripts/runtime-smoke');

async function withServer(t, handler) {
  const server = http.createServer(handler);
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  t.after(() => new Promise(resolve => server.close(resolve)));
  return `http://127.0.0.1:${server.address().port}`;
}

test('runtime smoke test only accepts loopback service URLs', () => {
  assert.equal(normalizeLocalBase('http://127.0.0.1:11434'), 'http://127.0.0.1:11434');
  assert.throws(() => normalizeLocalBase('https://example.com'), /only localhost \/ loopback/);
  assert.throws(() => normalizeLocalBase('http://user:secret@127.0.0.1:11434'), /credentials/);
  assert.throws(() => normalizeLocalBase('http://127.0.0.1:11434/api/tags'), /base URL without a path/);
});

test('Ollama runtime checker reports installed model names from bounded JSON', async t => {
  const base = await withServer(t, (req, res) => {
    res.setHeader('content-type', 'application/json');
    if (req.url === '/api/version') return res.end(JSON.stringify({ version: '0.40.11' }));
    if (req.url === '/api/tags') return res.end(JSON.stringify({ models: [{ name: 'qwen3:4b' }, { name: 'qwen2.5-coder:7b' }, { other: true }] }));
    res.writeHead(404); res.end('{}');
  });
  const result = await inspectOllama(base, { timeoutMs: 1000 });
  assert.equal(result.version, '0.40.11');
  assert.equal(result.modelCount, 2);
  assert.deepEqual(result.models, ['qwen3:4b', 'qwen2.5-coder:7b']);
});

test('ComfyUI runtime checker requires expected system statistics shape', async t => {
  const base = await withServer(t, (req, res) => {
    res.setHeader('content-type', 'application/json');
    res.end(JSON.stringify({ system: { os: 'win32' }, devices: [{ name: 'GPU 0', type: 'cuda' }] }));
  });
  const result = await inspectComfyUI(base, { timeoutMs: 1000 });
  assert.equal(result.os, 'win32');
  assert.equal(result.deviceCount, 1);
  assert.equal(result.devices[0].name, 'GPU 0');
});

test('runtime smoke test aborts oversized JSON even without a Content-Length header', async t => {
  const base = await withServer(t, (req, res) => {
    res.setHeader('content-type', 'application/json');
    res.write('{"payload":"');
    res.write('x'.repeat(300));
    res.end('"}');
  });
  await assert.rejects(fetchBoundedJson(base, '/large', { maxBytes: 64, timeoutMs: 1000 }), /exceeded the 64-byte limit/);
});

test('Ollama model response must have a valid model list', () => {
  assert.throws(() => extractOllamaModels({}), /no models array/);
  assert.deepEqual(extractOllamaModels({ models: [null, {}, { name: '  model-x  ' }] }), ['model-x']);
});
