'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Persistence = require('../src/workspace-persistence.js');

test('workspace snapshots sort recent chats first and preserve archive/project state', () => {
  const result = Persistence.prepareWorkspace({
    model: 'qwen3:4b', activeConversationId: 'older', provider: 'ollama', autoRoute: true,
    conversations: [
      { id: 'older', title: 'Dokončený', updatedAt: 10, archived: true, project: true, messages: [{ role: 'user', content: 'A' }] },
      { id: 'recent', title: 'Projekt', updatedAt: 20, archived: false, project: true, messages: [{ role: 'user', content: 'B' }] }
    ]
  });
  assert.deepEqual(result.payload.conversations.map(c => c.id), ['recent', 'older']);
  assert.equal(result.payload.conversations[0].project, true);
  assert.equal(result.payload.conversations[1].archived, true);
  assert.equal(result.payload.conversations[1].project, false, 'archived conversations are not simultaneously projects');
  assert.equal(result.payload.activeConversationId, 'older');
  assert.equal(result.payload.model, 'qwen3:4b');
});

test('workspace trims oversized history deterministically and keeps the newest message tail', () => {
  const conversations = Array.from({ length: 12 }, (_, i) => ({
    id: `c${i}`, title: `Chat ${i}`, updatedAt: i + 1,
    messages: Array.from({ length: 12 }, (_, j) => ({ role: j % 2 ? 'assistant' : 'user', content: `${i}/${j} ` + 'x'.repeat(9000) + ` END-${i}-${j}` }))
  }));
  const result = Persistence.prepareWorkspace({ conversations }, {
    maxSerializedChars: 100_000, contentBudget: 20_000, maxMessages: 100,
    maxConversations: 12, perConversationMessages: 12, perMessageChars: 12_000
  });
  assert.equal(result.trimmed, true);
  assert.ok(result.stats.omittedMessages > 0 || result.stats.omittedCharacters > 0);
  assert.ok(result.payload.conversations[0].updatedAt >= result.payload.conversations.at(-1).updatedAt);
  assert.ok(result.payload.conversations.some(c => c.storageTrimmed));
  assert.ok(result.json.length <= 100_000, `snapshot exceeded size ceiling: ${result.json.length}`);
  assert.match(result.json, /END-11-11|END-11-10|END-11-9/, 'recent messages should be retained preferentially');
});

test('metadata is whitelisted and bounded before it can inflate local storage', () => {
  const result = Persistence.prepareWorkspace({
    preferences: { system_prompt: 'x'.repeat(100_000), extra: 'do not persist'.repeat(20_000), temperature: 99, response_language: 'ru' },
    agentConfig: { planner: { provider: 'evil', localModel: '../bad', remoteModels: { openai: 'bad..model', anthropic: 'claude-haiku-4-5-20251001' }, ignored: 'x'.repeat(100_000) } },
    conversations: []
  });
  assert.equal(result.payload.preferences.system_prompt.length, 6000);
  assert.equal(result.payload.preferences.temperature, 2);
  assert.equal(result.payload.preferences.response_language, 'auto');
  assert.equal('extra' in result.payload.preferences, false);
  assert.equal(result.payload.agentConfig.planner.provider, 'ollama');
  assert.equal(result.payload.agentConfig.planner.localModel, '');
  assert.equal(result.payload.agentConfig.planner.remoteModels.openai, 'gpt-6-luna');
  assert.ok(result.json.length < 20_000);
});

test('message compaction preserves valid pending state and local generated-image references only', () => {
  const valid = Persistence.compactMessage({
    role: 'assistant', content: 'waiting', pending: true,
    images: [{ url: 'http://127.0.0.1:8188/view?filename=out.png' }, { url: 'https://example.com/track.png' }],
    followups: [{ label: 'Continue', prompt: 'Next step' }],
    action: { label: 'Open images', view: 'images' }
  }, 1000);
  assert.equal(valid.pending, true);
  assert.equal(valid.images.length, 1);
  assert.deepEqual(valid.followups, [{ label: 'Continue', prompt: 'Next step' }]);
  assert.equal(valid.action.view, 'images');
  assert.equal(Persistence.compactMessage({ role: 'function', content: 'not a chat role' }, 1000), null);
});

test('a single long response keeps its beginning and ending with a clear truncation marker', () => {
  const source = 'BEGIN-' + 'a'.repeat(90_000) + '-KEEP-ENDING';
  const result = Persistence.prepareWorkspace({ conversations: [{ id: 'c', updatedAt: 1, messages: [{ role: 'assistant', content: source }] }] }, {
    maxSerializedChars: 100_000, contentBudget: 20_000, maxMessages: 100, maxConversations: 10,
    perConversationMessages: 10, perMessageChars: 20_000
  });
  const content = result.payload.conversations[0].messages[0].content;
  assert.ok(content.startsWith('BEGIN-'));
  assert.match(content, /část zprávy byla zkrácena/);
  assert.ok(content.endsWith('-KEEP-ENDING'));
  assert.equal(result.payload.conversations[0].messages[0].storageTruncated, true);
});


test('workspace snapshots never persist the raw local project path', () => {
  const result = Persistence.prepareWorkspace({
    folder: { path: 'C:/Users/some-user/Projects/Aurora', name: 'Aurora' },
    projectFolder: { path: 'C:/Users/some-user/Projects/Aurora', name: 'Aurora' },
    conversations: []
  });
  assert.equal(result.payload.folder, 'selected');
  assert.equal(result.payload.projectFolder, 'selected');
  assert.doesNotMatch(result.json, /some-user|C:\//);
});


test('timestamps reject invalid values and clamp negative or far-future dates', () => {
  assert.equal(Persistence.normalizeTimestamp('not a date', 123), 123);
  assert.equal(Persistence.normalizeTimestamp(-50, 123), 0);
  const now = Date.now();
  assert.ok(Persistence.normalizeTimestamp(now + 365 * 24 * 60 * 60 * 1000) <= Date.now() + 60_000);
});


test('active conversation survives the conversation cap even when much older than other chats', () => {
  const conversations = Array.from({ length: 8 }, (_, index) => ({
    id: `recent-${index}`, title: `Recent ${index}`, updatedAt: 1000 + index,
    messages: [{ role: 'user', content: `message-${index}` }]
  }));
  conversations.push({ id: 'active-old', title: 'Open import chat', updatedAt: 1, messages: [{ role: 'user', content: 'do not lose me' }] });
  const result = Persistence.prepareWorkspace({ activeConversationId: 'active-old', conversations }, { maxConversations: 4 });
  assert.equal(result.payload.conversations.length, 4);
  assert.ok(result.payload.conversations.some(c => c.id === 'active-old'));
  assert.equal(result.payload.activeConversationId, 'active-old');
  assert.ok(result.payload.conversations.some(c => c.messages.some(m => m.content === 'do not lose me')));
});

test('last-resort size trimming preserves active chat and its identifying metadata', () => {
  const conversations = [{
    id: 'active', title: 'Active', updatedAt: 1,
    messages: Array.from({ length: 20 }, (_, i) => ({ role: 'user', content: `ACTIVE-${i}-` + 'a'.repeat(4000) }))
  }];
  for (let index = 0; index < 9; index++) {
    conversations.push({
      id: `recent-${index}`, title: `Recent ${index}`, updatedAt: 100 + index,
      messages: Array.from({ length: 20 }, (_, i) => ({ role: 'user', content: `RECENT-${index}-${i}-` + 'b'.repeat(4000) }))
    });
  }
  const result = Persistence.prepareWorkspace({ activeConversationId: 'active', conversations }, {
    maxSerializedChars: 100000, contentBudget: 1000000, maxMessages: 1000,
    maxConversations: 10, perConversationMessages: 20, perMessageChars: 6000
  });
  assert.equal(result.trimmed, true, 'test data must exercise the last-resort serialized-size guard');
  assert.ok(result.payload.conversations.some(c => c.id === 'active'));
  assert.equal(result.payload.activeConversationId, 'active');
  const active = result.payload.conversations.find(c => c.id === 'active');
  assert.ok(active.messages.length > 0);
  assert.match(active.messages.at(-1).content, /ACTIVE-19-/);
  assert.ok(result.json.length <= 100000);
});


test('active conversation IDs use the same bounded value as persisted conversation IDs', () => {
  const longId = 'active-' + 'x'.repeat(150);
  const result = Persistence.prepareWorkspace({
    activeConversationId: longId,
    conversations: [{ id: longId, title: 'Active with legacy ID', updatedAt: 5, messages: [{ role: 'user', content: 'keep active' }] }]
  });
  assert.equal(result.payload.conversations[0].id.length, 100);
  assert.equal(result.payload.activeConversationId, result.payload.conversations[0].id);
});


test('workspace backup preserves bounded user profile fields but drops unrelated metadata', () => {
  const result = Persistence.prepareWorkspace({
    preferences: {
      display_name: '  Alex  ',
      personal_instructions: 'Preferuji jasné odpovědi.\u0000 Používej češtinu.',
      privateToken: 'must-not-persist'
    },
    conversations: []
  });
  assert.equal(result.payload.preferences.display_name, 'Alex');
  assert.equal(result.payload.preferences.personal_instructions, 'Preferuji jasné odpovědi.  Používej češtinu.');
  assert.equal('privateToken' in result.payload.preferences, false);
  const bounded = Persistence.compactPreferences({ display_name: 'x'.repeat(200), personal_instructions: 'y'.repeat(5000) });
  assert.equal(bounded.display_name.length, 64);
  assert.equal(bounded.personal_instructions.length, 1600);
});


test('workspace keeps pinned chats and project-space links without saving project paths', () => {
  const result = Persistence.prepareWorkspace({
    activeProjectId: 'project_0123456789',
    conversations: [{ id: 'saved-chat', title: 'Pinned', pinned: true, project: true, projectSpaceId: 'project_0123456789', updatedAt: 10, messages: [{ role: 'user', content: 'Hello' }] }]
  });
  const chat = result.payload.conversations[0];
  assert.equal(chat.pinned, true);
  assert.equal(chat.project, true);
  assert.equal(chat.projectSpaceId, 'project_0123456789');
  assert.equal(result.payload.activeProjectId, 'project_0123456789');
  assert.doesNotMatch(result.json, /folderPath|C:\\\\Users/);
});

test('workspace drops malformed project IDs and never pins archived chats', () => {
  const result = Persistence.prepareWorkspace({
    conversations: [
      { id: 'archived-pin', archived: true, pinned: true, projectSpaceId: '../outside', updatedAt: 2, messages: [] },
      { id: 'regular', pinned: false, projectSpaceId: 'bad/id', updatedAt: 1, messages: [] }
    ]
  });
  assert.equal(result.payload.conversations.find(c => c.id === 'archived-pin').pinned, false);
  assert.equal(result.payload.conversations.find(c => c.id === 'archived-pin').projectSpaceId, null);
  assert.equal(result.payload.conversations.find(c => c.id === 'regular').projectSpaceId, null);
});
