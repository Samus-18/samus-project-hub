'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Bounded = require('../src/bounded-response');

test('bounded JSON parser accepts an in-limit response', async () => {
  const response = new Response('{"ok":true,"count":3}', { headers: { 'content-type': 'application/json' } });
  assert.deepEqual(await Bounded.readJsonBounded(response, 64, 'test'), { ok: true, count: 3 });
});

test('bounded JSON parser rejects a declared oversized body before parsing', async () => {
  const response = new Response('{"long":"payload"}', { headers: { 'content-length': '9000' } });
  await assert.rejects(Bounded.readJsonBounded(response, 128, 'Odpověď'), /překročila limit/i);
});

test('bounded JSON parser rejects a chunked oversized body even without content-length', async () => {
  const response = new Response(JSON.stringify({ text: 'x'.repeat(2048) }));
  await assert.rejects(Bounded.readJsonBounded(response, 128, 'Odpověď'), /překročila limit/i);
});

test('bounded JSON parser clearly rejects malformed JSON', async () => {
  await assert.rejects(Bounded.readJsonBounded(new Response('{bad json'), 128, 'Lokální API'), /neobsahuje platný JSON/i);
});

test('bounded text reader caps byte count and returns a safe partial string', async () => {
  const text = await Bounded.readTextBounded(new Response('a'.repeat(1000)), 32);
  assert.equal(Buffer.byteLength(text), 32);
});
