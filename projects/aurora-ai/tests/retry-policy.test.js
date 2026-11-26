'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { fetchWithRetry, parseRetryAfter, MAX_RETRY_AFTER_MS } = require('../src/retry-policy');

const response = (status, retryAfter = null) => ({
  status,
  headers: { get: name => name.toLowerCase() === 'retry-after' ? retryAfter : null },
  body: { cancel: async () => {} }
});

test('Retry-After accepts seconds and HTTP dates, capped at a safe maximum', () => {
  assert.equal(parseRetryAfter('2'), 2000);
  assert.equal(parseRetryAfter('99'), MAX_RETRY_AFTER_MS);
  assert.equal(parseRetryAfter('invalid'), null);
  assert.equal(parseRetryAfter('Thu, 01 Jan 1970 00:00:03 GMT', 1000), 2000);
});

test('temporary 429 and 5xx responses retry and return the successful response', async () => {
  const statuses = [429, 503, 200];
  const delays = [];
  const result = await fetchWithRetry('https://provider.invalid/chat', {}, {
    fetchImpl: async () => response(statuses.shift(), '1'),
    sleepImpl: async delay => delays.push(delay)
  });
  assert.equal(result.status, 200);
  assert.deepEqual(delays, [1000, 1000]);
});

test('permanent 400 responses are not retried', async () => {
  let calls = 0;
  const result = await fetchWithRetry('https://provider.invalid/chat', {}, {
    fetchImpl: async () => { calls += 1; return response(400); },
    sleepImpl: async () => assert.fail('permanent errors must not wait and retry')
  });
  assert.equal(result.status, 400);
  assert.equal(calls, 1);
});

test('network TypeErrors retry at most three total attempts', async () => {
  let calls = 0;
  const delays = [];
  await assert.rejects(fetchWithRetry('https://provider.invalid/chat', {}, {
    fetchImpl: async () => { calls += 1; throw new TypeError('network offline'); },
    sleepImpl: async delay => delays.push(delay)
  }), /network offline/);
  assert.equal(calls, 3);
  assert.equal(delays.length, 2);
});

test('an aborted request is never retried', async () => {
  const controller = new AbortController();
  controller.abort();
  let calls = 0;
  await assert.rejects(fetchWithRetry('https://provider.invalid/chat', {}, {
    signal: controller.signal,
    fetchImpl: async () => { calls += 1; return response(200); }
  }), error => error.name === 'AbortError');
  assert.equal(calls, 0);
});
