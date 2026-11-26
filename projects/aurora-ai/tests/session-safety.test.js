'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');
const Safety = require('../src/session-safety.js');

test('conversation switching is allowed while idle and blocked during active chat or agent work', () => {
  assert.deepEqual(Safety.canSwitchConversation({ sending: false, agentRunning: false }), { allowed: true, reason: '' });
  assert.equal(Safety.canSwitchConversation({ sending: true, agentRunning: false }).allowed, false);
  assert.match(Safety.canSwitchConversation({ sending: true }).reason, /odpověď/i);
  assert.equal(Safety.canSwitchConversation({ sending: false, agentRunning: true }).allowed, false);
  assert.equal(Safety.canSwitchConversation({ sending: false, agentPreparing: true }).allowed, false);
  assert.match(Safety.canSwitchConversation({ agentPreparing: true }).reason, /ověřuje nastavení/i);
  assert.match(Safety.canSwitchConversation({ agentRunning: true }).reason, /Agentní úloha/i);
  assert.equal(Safety.canSwitchConversation({ workspaceBusy: true }).allowed, false);
  assert.match(Safety.canSwitchConversation({ workspaceBusy: true }).reason, /import zálohy/i);
});

test('the app protects history, new-chat, import, model and provider changes during generation', () => {
  const app = read('src/app.js');
  const html = read('src/index.html');
  assert.match(html, /src="session-safety\.js"[\s\S]*?src="workspace-persistence\.js"[\s\S]*?src="app\.js"/);
  assert.match(app, /function guardConversationSwitch\(\)/);
  assert.match(app, /async function sendMessage\(\)\{if\(state\.workspaceBusy\)/);
  assert.match(app, /const workIntent=window\.AuroraIntent\?\.detectWorkRequest\?\.\(text\);if\(workIntent\)\{\$\('prompt'\)\.value='';autoGrow\(\);await routeIntoWorkMode\(workIntent\);return\}state\.sending=true;/);
  assert.match(app, /newChat'\)\.onclick=\(\)=>\{if\(guardConversationSwitch\(\)\)return/);
  assert.match(app, /clearHistory'\)\.onclick=\(\)=>\{if\(guardConversationSwitch\(\)\)return/);
  assert.match(app, /importWorkspace'\)\.onclick=async\(\)=>\{\s*if\(guardConversationSwitch\(\)\)return;\s*state\.workspaceBusy=true;/);
  assert.match(app, /Import zálohy selhal: '\+e\.message\)\}finally\{state\.workspaceBusy=false\}\s*\};/);
  assert.match(app, /async function activateConversation\(c\)\{if\(!c\|\|guardConversationSwitch\(\)\)return;saveCurrentConversation\(\);[\s\S]{0,650}recoverInterruptedMessages\(c\.messages\|\|\[\]\)/, 'all conversation rows share the guarded activation path and recovery');
  assert.match(app, /if\(guardConversationSwitch\(\)\)\{\$\('providerSelect'\)\.value=state\.provider;return\}/);
  assert.match(app, /function selectModel\(name\)\{if\(guardConversationSwitch\(\)\)return/);
});

test('Work mode reserves the shared workspace during async provider checks and project selection', () => {
  const app = read('src/app.js');
  assert.match(app, /state\.agentPreparing=true/);
  assert.match(app, /state\.agentPreparing=false/);
  assert.match(app, /state\.agentRunning=true/);
  assert.match(app, /state\.agentRunning\|\|state\.agentPreparing/);
  assert.match(app, /async function chooseFolder\(keepWorkspaceLock=false\)/);
  assert.match(app, /async function chooseFolder\(keepWorkspaceLock=false\)[\s\S]{0,300}state\.workspaceBusy=true/);
  assert.match(app, /attachBtn'\)\.onclick=async\(\)=>\{if\(guardConversationSwitch\(\)\)return;state\.workspaceBusy=true/);
  assert.match(app, /async function sendMessage\(\)\{if\(state\.workspaceBusy\)/);
  assert.match(app, /openAIStatus\(provider\)/, 'Work checks remote provider readiness before starting its agent stages');
  assert.match(app, /if\(inlineMessage\)\{inlineMessage\.content=String\(e\.message\|\|e\);inlineMessage\.workStatus='error'/);
});

test('returning to Chat preserves the selected conversation title', () => {
  const app = read('src/app.js');
  assert.match(app, /const activeConversation=state\.conversations\.find\(c=>c\.id===state\.activeConversationId\);\$\('viewTitle'\)\.textContent=view==='chat'\?\(activeConversation\?\.title\|\|'Nová konverzace'\)/);
});


test('interrupted partial and placeholder responses recover honestly whenever a chat is opened', () => {
  const original = [
    { role: 'user', content: 'Explain the error' },
    { role: 'assistant', content: 'Přemýšlím…', pending: true, followups: [{ label: 'bad', prompt: 'bad' }] },
    { role: 'assistant', content: 'Partial output', pending: true },
    { role: 'assistant', content: 'Completed response', followups: [{ label: 'next', prompt: 'next' }] },
    { role: 'function', content: 'must be dropped' }
  ];
  const recovered = Safety.recoverInterruptedMessages(original);
  assert.equal(recovered.recovered, true);
  assert.equal(recovered.messages.length, 4);
  assert.match(recovered.messages[1].content, /Generování bylo přerušeno/);
  assert.equal('pending' in recovered.messages[1], false);
  assert.equal('followups' in recovered.messages[1], false);
  assert.match(recovered.messages[2].content, /Partial output/);
  assert.match(recovered.messages[2].content, /může být neúplná/);
  assert.deepEqual(recovered.messages[3].followups, [{ label: 'next', prompt: 'next' }]);
  assert.equal('pending' in original[1], true, 'recovery works on copies, not the persisted source');
});

test('completed conversation messages are unchanged during recovery', () => {
  const messages = [{ role: 'assistant', content: 'Complete answer' }, { role: 'user', content: 'Question' }];
  const recovered = Safety.recoverInterruptedMessages(messages);
  assert.equal(recovered.recovered, false);
  assert.deepEqual(recovered.messages, messages);
});


test('recovery is invoked both on startup and when selecting a historical chat', () => {
  const app = read('src/app.js');
  assert.match(app, /const recovery=recoverInterruptedMessages\(c\.messages\)/);
  assert.match(app, /const recovery=recoverInterruptedMessages\(c\.messages\|\|\[\]\)/);
});


test('image generation starts before taking the chat lock, persists a pending marker, and clears it on completion', () => {
  const app = read('src/app.js');
  const send = app.slice(app.indexOf('async function sendMessage()'), app.indexOf('let activeBatchProposal'));
  assert.ok(send.indexOf('const imageIntent=detectImageRequest(text)') < send.indexOf('state.sending=true;'), 'image branch must run while state.sending is still false');
  const image = app.slice(app.indexOf('async function sendImageRequest'), app.indexOf('async function sendMessage()'));
  assert.match(image, /content:'Kontroluji lokální obrazový engine…',pending:true/);
  assert.match(image, /renderMessages\(\);persistConversationDraft\(true\)/);
  assert.match(image, /finally\{delete placeholder\.pending;state\.sending=false/);
});

test('recent history selection also recovers interrupted turns and clears the trim warning after showing it once', () => {
  const app = read('src/app.js');
  const activation = app.slice(app.indexOf('async function activateConversation'), app.indexOf('function renderPinnedChats'));
  assert.match(activation, /const recovery=recoverInterruptedMessages\(c\.messages\|\|\[\]\);state\.messages=recovery\.messages/);
  assert.match(activation, /if\(c\.storageTrimmed\)\{state\.messages\.unshift\(\{role:'system',content:'Upozornění:[\s\S]{0,220}c\.storageTrimmed=false\}/);
});
