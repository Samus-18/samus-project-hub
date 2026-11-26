const test = require('node:test');
const assert = require('node:assert/strict');
const { prepareConversationContext, LIMITS } = require('../src/context-budget.js');

test('quick context keeps only a small recent slice', () => {
  const messages = Array.from({ length: 20 }, (_, i) => ({ role: i % 2 ? 'assistant' : 'user', content: `message-${i}` }));
  const result = prepareConversationContext(messages, 'quick');
  assert.ok(result.messages.length <= LIMITS.quick.messages);
  assert.equal(result.messages.at(-1).content, 'message-19');
  assert.ok(result.omittedMessages > 0);
});

test('context is trimmed by character budget without changing recent message order', () => {
  const messages = Array.from({ length: 12 }, (_, i) => ({ role: 'user', content: `${i}:` + 'x'.repeat(1200) }));
  const result = prepareConversationContext(messages, 'quick');
  assert.ok(result.usedChars <= LIMITS.quick.chars);
  assert.match(result.messages.at(-1).content, /^11:/);
  assert.ok(result.omittedChars > 0);
});

test('a single oversized latest message is clipped from the start, preserving its end', () => {
  const result = prepareConversationContext([{ role: 'user', content: 'begin-' + 'x'.repeat(9000) + '-latest' }], 'quick');
  assert.equal(result.usedChars, LIMITS.quick.chars);
  assert.ok(result.messages[0].content.endsWith('-latest'));
});

test('invalid roles and malformed messages are ignored', () => {
  const result = prepareConversationContext([{ role: 'tool', content: 'skip' }, null, { role: 'user', content: 'keep' }], 'balanced');
  assert.deepEqual(result.messages, [{ role: 'user', content: 'keep' }]);
  assert.equal(result.omittedMessages, 0);
});
