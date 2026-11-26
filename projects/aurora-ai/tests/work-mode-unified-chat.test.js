'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('Chat and Work use the same conversation, message list and composer DOM', () => {
  const html = read('src/index.html');
  const main = html.slice(html.indexOf('<main class="main'), html.indexOf('<aside id="dashboardRail"'));
  assert.match(main, /id="chatModeToggle"/);
  assert.match(main, /id="workModeToggle"/);
  assert.match(main, /id="chatView"/);
  assert.match(main, /id="messages"/);
  assert.match(main, /id="prompt"/);
  assert.match(main, /id="sendBtn"/);
  assert.match(main, /id="workProjectControl"/);
  assert.ok(main.indexOf('id="workProjectControl"') < main.indexOf('id="prompt"'));
  assert.doesNotMatch(html, /id="sidebarHomeMode"|id="sidebarCodeMode"/);
});

test('Work mode is a state of the shared chat and never opens the retired dashboard', () => {
  const app = read('src/app.js');
  const css = read('src/work-chat-unified.css');
  assert.match(app, /if\(view==='agent'\)\{setMode\('work'\);view='chat'\}/);
  assert.match(app, /chatModeToggle'[\s\S]*?setView\('chat'\)/);
  assert.match(app, /workModeToggle'[\s\S]*?setView\('chat'\)/);
  assert.match(css, /#agentView\{display:none!important\}/);
  assert.match(css, /#chatModeToggle,#workModeToggle\{display:inline-flex!important;align-items:center;justify-content:center;visibility:visible!important;opacity:1!important\}/);
  assert.match(css, /#chatView:not\(\.hidden\)\{display:flex!important\}/);
  assert.match(css, /#chatView\.hidden\{display:none!important\}/);
  assert.doesNotMatch(css, /#chatView\{display:flex!important\}/, 'the shared chat must not override the hidden state');
  assert.match(css, /\.main\[data-mode="work"\] \.work-project-control\{display:flex\}/);
});

test('Work task progress and approved proposals render inside normal assistant messages', () => {
  const app = read('src/app.js');
  const persistence = read('src/workspace-persistence.js');
  assert.match(app, /function renderMessages\(\)/);
  assert.match(app, /work-progress-card/);
  assert.match(app, /work-proposal-card/);
  assert.match(app, /state\.messages\.push\(user,progress\)/);
  assert.match(app, /inlineMessage\.workStages=stages\.map/);
  assert.match(app, /inlineMessage\.workProposal=changeSetResult\.proposal/);
  assert.match(persistence, /out\.workStatus = message\.workStatus/);
  assert.match(persistence, /out\.workStages =/);
  assert.match(persistence, /out\.workProposal =/);
});

test('active conversation mode is persisted and restored with the conversation', () => {
  const app = read('src/app.js');
  const persistence = read('src/workspace-persistence.js');
  assert.match(app, /state\.mode=c\.mode==='work'\?'work':state\.mode/);
  assert.match(app, /if\(activeConversation\)activeConversation\.mode=next/);
  assert.match(app, /c\.mode=state\.mode==='work'\?'work':'chat'/);
  assert.match(app, /mode:c\.mode==='work'\?'work':'chat'/);
  assert.match(persistence, /mode: raw\.mode === 'work' \? 'work' : 'chat'/);
});

test('every locally linked stylesheet and every local CSS url target exists', () => {
  const html = read('src/index.html');
  const stylesheetRefs = [...html.matchAll(/<link\b[^>]*>/gi)].filter(match => /rel=["']stylesheet["']/i.test(match[0])).map(match => match[0].match(/href=["']([^"']+\.css)["']/i)?.[1]).filter(Boolean);
  assert.ok(stylesheetRefs.includes('work-chat-unified.css'));
  for (const ref of stylesheetRefs) assert.ok(fs.existsSync(path.join(root, 'src', ref)), `Missing stylesheet ${ref}`);
  for (const ref of stylesheetRefs) {
    const css = fs.readFileSync(path.join(root, 'src', ref), 'utf8');
    for (const match of css.matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/gi)) {
      const value = match[1].trim();
      if (!value || /^(?:data:|https?:|\/\/|#)/i.test(value)) continue;
      const relative = decodeURIComponent(value.split(/[?#]/)[0]);
      assert.ok(fs.existsSync(path.resolve(root, 'src', relative)), `${ref} references missing ${relative}`);
    }
  }
});
