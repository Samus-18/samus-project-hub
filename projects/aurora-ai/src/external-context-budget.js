(function (root) {
  'use strict';
  const LIMITS = Object.freeze({ quick: 6000, balanced: 12000, deep: 22000, manual: 12000 });
  function boundExternalContext(value, profile = 'balanced') {
    const text = typeof value === 'string' ? value : '';
    const limit = LIMITS[profile] || LIMITS.balanced;
    if (text.length <= limit) return { text, originalChars: text.length, usedChars: text.length, truncated: false, omittedChars: 0, limit };
    const marker = '\n\n[ČÁST PŘIPOJENÉHO KONTEXTU BYLA ZKRÁCENA KVŮLI LIMITU VÝKONU. Některé podrobnosti mohou chybět; vyžádej si konkrétní soubor nebo zdroj znovu.]\n\n';
    const available = Math.max(0, limit - marker.length);
    const head = Math.ceil(available * 0.68);
    const tail = available - head;
    const bounded = text.slice(0, head) + marker + (tail ? text.slice(-tail) : '');
    return { text: bounded, originalChars: text.length, usedChars: bounded.length, truncated: true, omittedChars: text.length - bounded.length, limit };
  }
  const api = { LIMITS, boundExternalContext };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraExternalContextBudget = api;
})(typeof window !== 'undefined' ? window : globalThis);
