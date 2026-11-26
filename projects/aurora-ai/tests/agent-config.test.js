'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const A = require('../src/agent-config');

test('agent config has four independently configurable roles', () => {
  const defaults = A.createDefaultAgentConfig();
  assert.deepEqual(Object.keys(defaults), ['planner', 'researcher', 'implementer', 'reviewer']);
  assert.notEqual(defaults.planner, defaults.reviewer);
  assert.equal(defaults.planner.provider, 'ollama');
});

test('each agent role can route to a different provider and model', () => {
  const config = A.createDefaultAgentConfig();
  config.planner = {provider:'ollama',localModel:'qwen3:4b'};
  config.researcher = {provider:'openai',remoteModels:{openai:'gpt-4.1-mini'}};
  config.implementer = {provider:'anthropic',remoteModels:{anthropic:'claude-sonnet-4-5'}};
  config.reviewer = {provider:'gemini',remoteModels:{gemini:'gemini-2.5-flash'}};
  const normalized = A.normalizeAgentConfig(config);
  assert.deepEqual(A.getAgentRequestConfig(normalized,'planner',['qwen3:4b','nomic-embed-text']), {role:'planner',provider:'ollama',model:'qwen3:4b'});
  assert.deepEqual(A.getAgentRequestConfig(normalized,'researcher',['qwen3:4b']), {role:'researcher',provider:'openai',model:'gpt-4.1-mini'});
  assert.deepEqual(A.getAgentRequestConfig(normalized,'implementer',['qwen3:4b']), {role:'implementer',provider:'anthropic',model:'claude-sonnet-4-5'});
  assert.deepEqual(A.getAgentRequestConfig(normalized,'reviewer',['qwen3:4b']), {role:'reviewer',provider:'gemini',model:'gemini-2.5-flash'});
});

test('remote model selections persist independently per provider', () => {
  const config=A.createDefaultAgentConfig();
  config.planner.provider='openai';config.planner.remoteModels.openai='gpt-4.1';
  config.planner.provider='anthropic';config.planner.remoteModels.anthropic='claude-sonnet-4-5';
  const restored=A.normalizeAgentConfig(config);
  assert.equal(restored.planner.remoteModels.openai,'gpt-4.1');
  assert.equal(restored.planner.remoteModels.anthropic,'claude-sonnet-4-5');
  assert.equal(restored.researcher.remoteModels.openai,'gpt-6-luna');
});

test('invalid provider and model strings normalize to safe defaults', () => {
  const normalized=A.normalizeAgentConfig({planner:{provider:'file',localModel:'../../secret',remoteModels:{openai:'../secret'}}});
  assert.equal(normalized.planner.provider,'ollama');
  assert.equal(normalized.planner.localModel,'');
  assert.equal(normalized.planner.remoteModels.openai,'gpt-6-luna');
});

test('local role requires a known available model and rejects embedding models', () => {
  assert.throws(()=>A.getAgentRequestConfig(undefined,'planner',[],'qwen3:4b'),/Ollama nemá načtený/);
  const config=A.createDefaultAgentConfig();config.planner.localModel='nomic-embed-text';
  assert.deepEqual(A.getAgentRequestConfig(config,'planner',['qwen3:4b','nomic-embed-text'],'qwen3:4b'), {role:'planner',provider:'ollama',model:'qwen3:4b'});
});

test('unknown agent role and malformed model names are rejected', () => {
  assert.throws(()=>A.getAgentRequestConfig(undefined,'administrator',['qwen3:4b']),/Neznámá role/);
  const config=A.createDefaultAgentConfig();config.reviewer.provider='openai';config.reviewer.remoteModels.openai='../secret';
  assert.equal(A.getAgentRequestConfig(config,'reviewer',['qwen3:4b']).model,'gpt-6-luna');
});


test('remote providers can use namespaced model IDs while rejecting traversal-like names', () => {
  const config = A.createDefaultAgentConfig();
  config.reviewer.provider = 'openai';
  config.reviewer.remoteModels.openai = 'openrouter/deepseek-r1';
  assert.deepEqual(A.getAgentRequestConfig(config, 'reviewer', ['qwen3:4b'], 'qwen3:4b'), {
    role: 'reviewer', provider: 'openai', model: 'openrouter/deepseek-r1'
  });
  assert.equal(A.validRemoteModel('../secret'), '');
  assert.equal(A.validRemoteModel('openrouter//deepseek'), '');
});


test('retired remote defaults are migrated without overwriting custom model IDs', () => {
 const migrated = A.normalizeAgentConfig({
  planner:{provider:'anthropic',remoteModels:{anthropic:'claude-3-5-haiku-latest'}},
  reviewer:{provider:'gemini',remoteModels:{gemini:'gemini-2.0-flash'}},
  implementer:{provider:'openai',remoteModels:{openai:'gpt-4.1-mini'}}
 });
 assert.equal(migrated.planner.remoteModels.anthropic,'claude-haiku-4-5-20251001');
 assert.equal(migrated.reviewer.remoteModels.gemini,'gemini-3.5-flash-lite');
 assert.equal(migrated.implementer.remoteModels.openai,'gpt-4.1-mini');
});


test('Agent configuration module exposes a browser global as well as the Node export', () => {
  const fs = require('node:fs');
  const source = fs.readFileSync(require.resolve('../src/agent-config.js'), 'utf8');
  assert.match(source, /typeof module !== 'undefined' && module\.exports/);
  assert.match(source, /window\.AuroraAgentConfig = AuroraAgentConfig/);
});

test('local Ollama model IDs support safe namespaces such as Nanbeige packages', () => {
  const config = A.createDefaultAgentConfig();
  config.planner.localModel = 'tomng/nanbeige4.1:3b';
  const normalized = A.normalizeAgentConfig(config);
  assert.equal(normalized.planner.localModel, 'tomng/nanbeige4.1:3b');
  assert.equal(A.validModel('hf.co/bartowski/Nanbeige_Nanbeige4.3-33B-GGUF:Q4_K_M'), 'hf.co/bartowski/Nanbeige_Nanbeige4.3-33B-GGUF:Q4_K_M');
  assert.equal(A.validModel('../secret'), '');
  assert.equal(A.validModel('owner//model'), '');
  assert.equal(A.validModel('owner/../model'), '');
});


test('blank local role model stays automatic and uses the current task-routed fallback', () => {
  const config=A.createDefaultAgentConfig();
  assert.equal(config.implementer.localModel,'');
  assert.equal(A.getAgentRequestConfig(config,'implementer',['qwen3:4b','qwen2.5-coder:7b'],'qwen2.5-coder:7b').model,'qwen2.5-coder:7b');
  assert.equal(config.implementer.localModel,'','resolving a fallback must not persist a manual selection');
});

test('manual local role model overrides the automatic task fallback only when installed', () => {
  const config=A.createDefaultAgentConfig();config.implementer.localModel='qwen3:4b';
  assert.equal(A.getAgentRequestConfig(config,'implementer',['qwen3:4b','qwen2.5-coder:7b'],'qwen2.5-coder:7b').model,'qwen3:4b');
});
