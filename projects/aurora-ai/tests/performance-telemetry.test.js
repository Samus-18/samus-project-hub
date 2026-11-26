const test = require('node:test');
const assert = require('node:assert/strict');
const { createSample, appendSample, summarize, MAX_SAMPLES } = require('../src/performance-telemetry.js');

test('telemetry records latency and a clearly labelled character throughput estimate', () => {
  const sample = createSample({ totalMs: 2000, firstTokenMs: 350, outputChars: 600, model: 'qwen3:4b', provider: 'ollama' });
  assert.equal(sample.charsPerSecond, 300);
  assert.equal(sample.firstTokenMs, 350);
  assert.equal(sample.completed, true);
});

test('telemetry clamps malformed values and bounds stored history', () => {
  const sample = createSample({ totalMs: -20, firstTokenMs: 999999999, outputChars: 'bad', contextChars: 999999999 });
  assert.equal(sample.totalMs, 0);
  assert.equal(sample.firstTokenMs, 0);
  assert.equal(sample.outputChars, 0);
  assert.equal(sample.contextChars, 10000000);
  let history = [];
  for (let i = 0; i < MAX_SAMPLES + 5; i++) history = appendSample(history, { totalMs: 100 + i, outputChars: 10 });
  assert.equal(history.length, MAX_SAMPLES);
  assert.equal(history[0].totalMs, 105);
});

test('summary calculates mean, median, and first-output latency', () => {
  const summary = summarize([
    createSample({ totalMs: 100, firstTokenMs: 20, outputChars: 100 }),
    createSample({ totalMs: 300, firstTokenMs: 40, outputChars: 300 }),
    createSample({ totalMs: 200, firstTokenMs: 30, outputChars: 200, completed: false })
  ]);
  assert.equal(summary.count, 2);
  assert.equal(summary.averageMs, 200);
  assert.equal(summary.medianMs, 200);
  assert.equal(summary.averageFirstTokenMs, 30);
});

test('empty summary reports no fabricated measurements', () => {
  assert.deepEqual(summarize([]), { count: 0, averageMs: null, medianMs: null, averageFirstTokenMs: null, averageCharsPerSecond: null });
});
