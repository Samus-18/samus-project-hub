const test = require('node:test');
const assert = require('node:assert/strict');
const { boundExternalContext, LIMITS } = require('../src/external-context-budget.js');

test('external context stays intact under the profile limit', () => {
  const input = 'Výzkum a zdroje '.repeat(50);
  const result = boundExternalContext(input, 'balanced');
  assert.equal(result.text, input);
  assert.equal(result.truncated, false);
  assert.equal(result.omittedChars, 0);
});

test('oversized external context is bounded and preserves its beginning and ending', () => {
  const input = 'BEGIN-' + 'x'.repeat(30000) + '-END';
  const result = boundExternalContext(input, 'quick');
  assert.equal(result.truncated, true);
  assert.ok(result.text.length <= LIMITS.quick);
  assert.ok(result.text.startsWith('BEGIN-'));
  assert.ok(result.text.endsWith('-END'));
  assert.match(result.text, /zkrácena kvůli limitu výkonu/i);
  assert.equal(result.omittedChars, input.length - result.text.length);
});

test('unknown profile falls back to balanced budget and empty context stays empty', () => {
  assert.equal(boundExternalContext('', 'deep').usedChars, 0);
  assert.equal(boundExternalContext('a'.repeat(13000), 'mystery').limit, LIMITS.balanced);
});
