const test = require('node:test');
const assert = require('node:assert/strict');
const { classifyPrompt, resolvePerformance } = require('../src/performance-profile.js');

test('adaptive profile sends simple questions to the quick preset', () => {
  assert.equal(classifyPrompt('Co je RAM?'), 'quick');
  const result = resolvePerformance('adaptive', 'Co je RAM?', { temperature: 0.4, num_ctx: 4096, num_predict: 1024, num_thread: 4 }, { cores: 8 });
  assert.equal(result.selected, 'quick');
  assert.equal(result.options.num_ctx, 2048);
  assert.equal(result.options.num_predict, 384);
  assert.equal(result.options.temperature, 0.4);
});

test('adaptive profile allocates more context and output to complex coding work', () => {
  const prompt = 'Analyzuj projekt, oprav crash a navrhni refactor této funkce.';
  assert.equal(classifyPrompt(prompt), 'deep');
  const result = resolvePerformance('adaptive', prompt, { temperature: 0.7, num_ctx: 4096, num_predict: 1024, num_thread: 4 }, { cores: 4 });
  assert.equal(result.options.num_ctx, 8192);
  assert.equal(result.options.num_predict, 2048);
  assert.equal(result.options.num_thread, 4);
});

test('manual mode respects saved values but clamps them to safe bounds', () => {
  const result = resolvePerformance('manual', 'hello', { temperature: 5, num_ctx: 32, num_predict: 99999, num_thread: 99 }, { cores: 6 });
  assert.equal(result.options.temperature, 2);
  assert.equal(result.options.num_ctx, 1024);
  assert.equal(result.options.num_predict, 8192);
  assert.equal(result.options.num_thread, 6);
});

test('fixed profiles are deterministic and thread counts never exceed reported cores', () => {
  const result = resolvePerformance('quick', 'write a book', { temperature: 0.7 }, { cores: 2 });
  assert.equal(result.selected, 'quick');
  assert.equal(result.options.num_thread, 1);
  assert.equal(result.options.num_predict, 384);
});

test('long prompts default to deep and ordinary prompts remain balanced', () => {
  assert.equal(classifyPrompt('x'.repeat(701)), 'deep');
  assert.equal(classifyPrompt('Můžeš mi pomoct s tímto nápadem?'), 'quick');
  assert.equal(classifyPrompt('Napiš krátké shrnutí toho, jak funguje databáze.'), 'balanced');
});

test('generation profiles preserve bounded sampling values and optional deterministic seed', () => {
  const saved = {temperature:0.6,top_p:0.83,top_k:77,repeat_penalty:1.25,seed:42,stop_sequences:'END | ### | '+('X'.repeat(160)),num_ctx:4096,num_predict:1024,num_thread:4};
  const result = resolvePerformance('balanced', 'Co je paměť?', saved, {cores:8});
  assert.equal(result.options.temperature,0.6);
  assert.equal(result.options.top_p,0.83);
  assert.equal(result.options.top_k,77);
  assert.equal(result.options.repeat_penalty,1.25);
  assert.equal(result.options.seed,42);
  assert.deepEqual(result.options.stop,['END','###', 'X'.repeat(100)]);
});

test('generation profile drops invalid seed and clamps sampling parameters', () => {
  const result = resolvePerformance('manual', 'hello', {temperature:9,top_p:0,top_k:999,repeat_penalty:12,seed:'oops',num_ctx:4096,num_predict:256,num_thread:4}, {cores:4});
  assert.equal(result.options.temperature,2);
  assert.equal(result.options.top_p,0.05);
  assert.equal(result.options.top_k,200);
  assert.equal(result.options.repeat_penalty,2);
  assert.equal('seed' in result.options, false);
});
