'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Metrics = require('../src/ollama-metrics');

test('converts Ollama done metrics from nanoseconds to real token rates', () => {
  const metrics = Metrics.fromDoneMessage({
    prompt_eval_count: 60, prompt_eval_duration: 2_000_000_000,
    eval_count: 120, eval_duration: 3_000_000_000,
    total_duration: 4_000_000_000, load_duration: 500_000_000
  });
  assert.equal(metrics.promptTokens, 60);
  assert.equal(metrics.evalTokens, 120);
  assert.equal(metrics.tokensPerSecond, 40);
  assert.equal(metrics.promptTokensPerSecond, 30);
  assert.equal(metrics.totalDurationNs, 4_000_000_000);
});

test('invalid, missing or zero-duration metrics never create a fabricated rate', () => {
  const metrics = Metrics.fromDoneMessage({ eval_count: 'not-a-number', eval_duration: 0, prompt_eval_count: -1 });
  assert.equal(metrics.evalTokens, null);
  assert.equal(metrics.promptTokens, null);
  assert.equal(metrics.tokensPerSecond, null);
  assert.equal(metrics.promptTokensPerSecond, null);
});
