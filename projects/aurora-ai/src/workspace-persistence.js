'use strict';
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  if (root) root.AuroraWorkspacePersistence = api;
})(typeof window !== 'undefined' ? window : globalThis, function () {
  const ROLES = new Set(['user', 'assistant', 'system']);
  const safeString = (value, max) => String(value ?? '').slice(0, max);
  function normalizeTimestamp(value, fallback = Date.now()) {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) return fallback;
    return Math.max(0, Math.min(Date.now() + 60_000, parsed));
  }
  const defaultOptions = Object.freeze({
    maxSerializedChars: 1_650_000,
    contentBudget: 1_250_000,
    maxMessages: 1_500,
    maxConversations: 100,
    perConversationMessages: 200,
    perMessageChars: 30_000
  });

  function clampContent(value, max) {
    const text = String(value ?? '');
    if (text.length <= max) return { text, truncated: false };
    if (max < 120) return { text: text.slice(-Math.max(0, max)), truncated: true };
    const marker = '\n\n[… část zprávy byla zkrácena kvůli limitu úložiště …]\n\n';
    const remaining = Math.max(0, max - marker.length);
    const head = Math.ceil(remaining * 0.58);
    const tail = Math.max(0, remaining - head);
    return { text: text.slice(0, head) + marker + (tail ? text.slice(-tail) : ''), truncated: true };
  }

  function compactPreferences(value) {
    const p = value && typeof value === 'object' && !Array.isArray(value) ? value : {};
    const number = (key, min, max, fallback) => {
      const n = Number(p[key]);
      return Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : fallback;
    };
    return {
      temperature: number('temperature', 0, 2, 0.7),
      top_p: number('top_p', 0.05, 1, 0.9),
      top_k: Math.round(number('top_k', 1, 200, 40)),
      repeat_penalty: number('repeat_penalty', 0.5, 2, 1.1),
      seed: p.seed === null || p.seed === undefined || p.seed === '' ? null : Math.round(number('seed', 0, 2147483647, 0)),
      stop_sequences: safeString(p.stop_sequences, 300),
      system_prompt: safeString(p.system_prompt, 6000),
      display_name: safeString(p.display_name, 64).replace(/[\u0000-\u001f\u007f]/g, ' ').trim(),
      personal_instructions: safeString(p.personal_instructions, 1600).replace(/[\u0000-\u001f\u007f]/g, ' ').trim(),
      response_language: ['auto', 'cs', 'en'].includes(p.response_language) ? p.response_language : 'auto',
      keep_alive: ['0', '1m', '5m', '10m', '30m', '1h'].includes(String(p.keep_alive)) ? String(p.keep_alive) : '10m',
      stream_responses: p.stream_responses !== false,
      num_ctx: Math.round(number('num_ctx', 512, 131072, 4096)),
      num_predict: Math.round(number('num_predict', 1, 32768, 1024)),
      num_thread: Math.round(number('num_thread', 1, 128, 4))
    };
  }

  function compactAgentConfig(value) {
    const src = value && typeof value === 'object' && !Array.isArray(value) ? value : {};
    const roles = ['planner', 'researcher', 'implementer', 'reviewer'];
    const providers = ['ollama', 'openai', 'anthropic', 'gemini'];
    const defaults = {
      openai: 'gpt-6-luna',
      anthropic: 'claude-haiku-4-5-20251001',
      gemini: 'gemini-3.5-flash-lite'
    };
    const modelOk = value => typeof value === 'string' && /^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,119}$/.test(value) && !value.includes('..') && !value.includes('//') && !value.endsWith('/');
    return Object.fromEntries(roles.map(role => {
      const item = src[role] && typeof src[role] === 'object' && !Array.isArray(src[role]) ? src[role] : {};
      const remote = item.remoteModels && typeof item.remoteModels === 'object' && !Array.isArray(item.remoteModels) ? item.remoteModels : {};
      return [role, {
        provider: providers.includes(item.provider) ? item.provider : 'ollama',
        localModel: modelOk(item.localModel) ? item.localModel : '',
        remoteModels: Object.fromEntries(Object.keys(defaults).map(p => [p, modelOk(remote[p]) ? remote[p] : defaults[p]]))
      }];
    }));
  }

  function compactMessage(message, maxChars) {
    if (!message || !ROLES.has(message.role)) return null;
    const bounded = clampContent(message.content, maxChars);
    const out = { role: message.role, content: bounded.text };
    if (typeof message.model === 'string') out.model = safeString(message.model, 120);
    if (message.pending === true) out.pending = true;
    if (message.storageTruncated === true || bounded.truncated) out.storageTruncated = true;
    if (Array.isArray(message.followups)) {
      out.followups = message.followups.slice(0, 4).filter(x => x && typeof x === 'object')
        .map(x => ({ label: safeString(x.label, 120), prompt: safeString(x.prompt, 1200) }))
        .filter(x => x.label && x.prompt);
    }
    if (message.action && typeof message.action === 'object') {
      const action = message.action;
      const view = ['images', 'projects'].includes(action.view) ? action.view : null;
      if (view) out.action = { label: safeString(action.label, 140), view,
        ...(action.chooseProject === true ? { chooseProject: true } : {}),
        ...(typeof action.resumePrompt === 'string' ? { resumePrompt: safeString(action.resumePrompt, 2000) } : {}) };
    }
    if (Array.isArray(message.images)) {
      out.images = message.images.slice(0, 8).filter(x => x && typeof x.url === 'string' && /^http:\/\/127\.0\.0\.1:8188\/view\?/.test(x.url))
        .map(x => ({ url: safeString(x.url, 2000) }));
      if (!out.images.length) delete out.images;
    }
    if (typeof message.imagePrompt === 'string') out.imagePrompt = safeString(message.imagePrompt, 2000);
    if (['preparing','waiting','working','done','stopped','error'].includes(message.workStatus)) out.workStatus = message.workStatus;
    if (Array.isArray(message.workStages)) {
      out.workStages = message.workStages.slice(0, 4).filter(stage => stage && typeof stage === 'object').map(stage => ({
        role: safeString(stage.role, 48),
        provider: ['ollama','openai','anthropic','gemini'].includes(stage.provider) ? stage.provider : 'ollama',
        model: safeString(stage.model, 120),
        status: ['pending','working','done','stopped','error'].includes(stage.status) ? stage.status : 'pending',
        content: safeString(stage.content, 7000)
      }));
    }
    if (message.workProposal && typeof message.workProposal === 'object' && Array.isArray(message.workProposal.edits)) {
      const edits = message.workProposal.edits.slice(0, 12).filter(edit => edit && typeof edit.path === 'string' && typeof edit.content === 'string' && edit.path.length <= 220 && !edit.path.startsWith('/') && !edit.path.includes('\\') && !edit.path.split('/').some(part => !part || part === '.' || part === '..') && !edit.path.includes(':') && !/[\u0000-\u001f]/.test(edit.path)).map(edit => ({path:safeString(edit.path,220),content:safeString(edit.content,20000)}));
      if (edits.length) out.workProposal = {summary:safeString(message.workProposal.summary,1000),tests:Array.isArray(message.workProposal.tests)?message.workProposal.tests.slice(0,20).map(test=>safeString(test,300)):[],edits};
    }
    return out;
  }

  function prepareWorkspace(input, customOptions = {}) {
    const opts = { ...defaultOptions, ...customOptions };
    const maxSerializedChars = Math.max(100_000, Number(opts.maxSerializedChars) || defaultOptions.maxSerializedChars);
    let remainingContent = Math.max(20_000, Number(opts.contentBudget) || defaultOptions.contentBudget);
    let remainingMessages = Math.max(100, Number(opts.maxMessages) || defaultOptions.maxMessages);
    const source = input && typeof input === 'object' ? input : {};
    const rawConversations = Array.isArray(source.conversations) ? source.conversations : [];
    const maxConversations = Math.max(1, Math.min(500, Math.round(Number(opts.maxConversations) || defaultOptions.maxConversations)));
    const sortedRecent = rawConversations.slice().sort((a, b) => (Number(b?.updatedAt) || 0) - (Number(a?.updatedAt) || 0));
    const requestedActiveId = typeof source.activeConversationId === 'string' ? safeString(source.activeConversationId, 100) : null;
    const activeConversation = requestedActiveId ? sortedRecent.find(c => c && safeString(c.id, 100) === requestedActiveId) : null;
    // A valid active chat must survive caps even if it came from an older imported backup.
    const ordered = activeConversation
      ? [activeConversation, ...sortedRecent.filter(c => c !== activeConversation).slice(0, maxConversations - 1)]
      : sortedRecent.slice(0, maxConversations);
    let omittedMessages = Math.max(0, rawConversations.length - ordered.length);
    let omittedCharacters = 0;
    let trimmedConversations = 0;
    const conversations = [];

    for (const raw of ordered) {
      if (!raw || typeof raw !== 'object' || typeof raw.id !== 'string') continue;
      const sourceMessages = Array.isArray(raw.messages) ? raw.messages.slice(-opts.perConversationMessages) : [];
      const picked = [];
      let wasTrimmed = !!raw.storageTrimmed || sourceMessages.length !== (raw.messages || []).length;
      for (let i = sourceMessages.length - 1; i >= 0; i--) {
        if (remainingMessages <= 0 || remainingContent <= 0) { omittedMessages += i + 1; wasTrimmed = true; break; }
        const original = sourceMessages[i];
        if (!original || !ROLES.has(original.role)) { wasTrimmed = true; continue; }
        const content = String(original.content ?? '');
        const maxForMessage = Math.min(opts.perMessageChars, remainingContent);
        if (maxForMessage < 80 && content.length > maxForMessage) { omittedMessages++; wasTrimmed = true; omittedCharacters += content.length; continue; }
        const message = compactMessage(original, maxForMessage);
        if (!message) { wasTrimmed = true; continue; }
        if (message.storageTruncated) { wasTrimmed = true; omittedCharacters += Math.max(0, content.length - message.content.length); }
        remainingContent -= Math.min(content.length, message.content.length) + 60;
        remainingMessages--;
        picked.push(message);
      }
      picked.reverse();
      if (wasTrimmed) trimmedConversations++;
      conversations.push({
        id: safeString(raw.id, 100),
        title: safeString(raw.title || 'Konverzace', 160),
        updatedAt: normalizeTimestamp(raw.updatedAt, 0),
        archived: raw.archived === true,
        project: (raw.project === true || (typeof raw.projectSpaceId === 'string' && /^[a-zA-Z0-9_-]{8,100}$/.test(raw.projectSpaceId))) && raw.archived !== true,
        pinned: raw.pinned === true && raw.archived !== true,
        projectSpaceId: typeof raw.projectSpaceId === 'string' && /^[a-zA-Z0-9_-]{8,100}$/.test(raw.projectSpaceId) ? safeString(raw.projectSpaceId, 100) : null,
        mode: raw.mode === 'work' ? 'work' : 'chat',
        storageTrimmed: wasTrimmed,
        messages: picked
      });
    }

    const payload = {
      model: safeString(source.model, 120),
      mode: source.mode === 'work' ? 'work' : 'chat',
      conversations,
      folder: source.folder ? 'selected' : null,
      projectFolder: source.projectFolder ? 'selected' : null,
      activeConversationId: requestedActiveId && conversations.some(c => c.id === requestedActiveId) ? safeString(requestedActiveId, 100) : null,
      activeProjectId: typeof source.activeProjectId === 'string' && /^[a-zA-Z0-9_-]{8,100}$/.test(source.activeProjectId) ? safeString(source.activeProjectId, 100) : null,
      preferences: compactPreferences(source.preferences),
      performanceProfile: safeString(source.performanceProfile, 30),
      autoRoute: source.autoRoute !== false,
      provider: ['ollama', 'openai', 'anthropic', 'gemini'].includes(source.provider) ? source.provider : 'ollama',
      agentConfig: compactAgentConfig(source.agentConfig),
      agentComplexity: source.agentComplexity === 'advanced' ? 'advanced' : 'simple'
    };

    conversations.sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0));
    // Never orphan the active conversation when a large export hits its last-resort size ceiling.
    payload.activeConversationId = requestedActiveId && conversations.some(c => c.id === requestedActiveId) ? requestedActiveId : null;
    let json = JSON.stringify(payload);
    // A final guard for large non-message metadata or unusual imported values. Trim non-active chats first.
    while (json.length > maxSerializedChars) {
      let removed = false;
      const activeId = payload.activeConversationId;
      const order = conversations.map((_, index) => index).reverse().filter(index => conversations[index].id !== activeId);
      const activeIndex = conversations.findIndex(c => c.id === activeId);
      if (activeIndex >= 0) order.push(activeIndex);
      for (const ci of order) {
        const conversation = conversations[ci];
        if (conversation.messages.length) {
          const dropped = conversation.messages.shift();
          omittedMessages++;
          omittedCharacters += String(dropped.content || '').length;
          conversation.storageTrimmed = true;
          removed = true;
          break;
        }
        if (conversation.id !== activeId) {
          conversations.splice(ci, 1);
          omittedMessages++;
          removed = true;
          break;
        }
      }
      if (!removed) break;
      json = JSON.stringify(payload);
    }
    return { payload, json, trimmed: trimmedConversations > 0 || omittedMessages > 0 || omittedCharacters > 0,
      stats: { conversations: conversations.length, omittedMessages, omittedCharacters, trimmedConversations, serializedChars: json.length } };
  }

  return Object.freeze({ prepareWorkspace, compactMessage, compactPreferences, compactAgentConfig, normalizeTimestamp, defaultOptions });
});
