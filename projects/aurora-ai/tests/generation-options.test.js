const test = require('node:test');
const assert = require('node:assert/strict');
const Generation = require('../src/generation-options.js');

test('Ollama generation options clamp numeric values and reject non-finite input', () => {
  assert.deepEqual(Generation.normalizeOllamaOptions({
    temperature: 5, top_p: -1, top_k: 500, repeat_penalty: 0.1,
    num_ctx: 1, num_predict: 999999, num_thread: 2.6, seed: -1,
    not_a_real_option: 'should be dropped',
  }), {
    temperature: 2, top_p: 0.05, top_k: 200, repeat_penalty: 0.5,
    num_ctx: 512, num_predict: 32768, num_thread: 3, seed: 0,
  });
  assert.deepEqual(Generation.normalizeOllamaOptions({ temperature: 'NaN', top_k: Infinity }), {});
});

test('stop sequences have count and length bounds and discard non-text values', () => {
  const opts = Generation.normalizeOllamaOptions({ stop: [42, 'x'.repeat(150), '', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'] });
  assert.equal(opts.stop.length, 8);
  assert.equal(opts.stop[0].length, 100);
  assert.deepEqual(opts.stop.slice(1), ['a','b','c','d','e','f','g']);
});

test('keep-alive permits only supported presets and defaults safely', () => {
  assert.equal(Generation.normalizeKeepAlive('0'), 0);
  assert.equal(Generation.normalizeKeepAlive('1h'), '1h');
  assert.equal(Generation.normalizeKeepAlive('4 days'), '10m');
  assert.equal(Generation.normalizeKeepAlive(null), '10m');
});
