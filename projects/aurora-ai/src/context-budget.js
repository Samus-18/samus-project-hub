(function (root) {
  'use strict';
  const LIMITS = Object.freeze({ quick: { messages: 10, chars: 6000 }, balanced: { messages: 18, chars: 14000 }, deep: { messages: 30, chars: 28000 }, manual: { messages: 24, chars: 18000 } });
  function prepareConversationContext(messages, profile = 'balanced') {
    const source = Array.isArray(messages) ? messages.filter(m => m && ['user', 'assistant', 'system'].includes(m.role) && typeof m.content === 'string') : [];
    const limit = LIMITS[profile] || LIMITS.balanced;
    const selected = [];
    let chars = 0;
    for (let i = source.length - 1; i >= 0; i--) {
      const item = source[i];
      const content = item.content;
      if (selected.length >= limit.messages) break;
      if (selected.length && chars + content.length > limit.chars) break;
      if (!selected.length && content.length > limit.chars) {
        selected.push({ role: item.role, content: content.slice(-limit.chars) });
        chars += limit.chars;
        break;
      }
      selected.push({ role: item.role, content });
      chars += content.length;
    }
    selected.reverse();
    return { messages: selected, omittedMessages: Math.max(0, source.length - selected.length), omittedChars: Math.max(0, source.reduce((sum, m) => sum + m.content.length, 0) - chars), limits: { ...limit }, usedChars: chars };
  }
  const api = { prepareConversationContext, LIMITS };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraContextBudget = api;
})(typeof window !== 'undefined' ? window : globalThis);
