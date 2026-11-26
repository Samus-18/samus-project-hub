'use strict';
(function(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  if (root) root.AuroraSessionSafety = api;
})(typeof window !== 'undefined' ? window : globalThis, function() {
  function canSwitchConversation(session = {}) {
    if (session.sending) return { allowed: false, reason: 'Nejdřív dokonči nebo zastav běžící odpověď. Konverzaci teď nepřepínám, aby se odpověď nepřipojila ke špatnému chatu.' };
    if (session.agentPreparing) return { allowed: false, reason: 'Agent Studio právě ověřuje nastavení. Počkej, až kontrolu dokončí, než přepneš konverzaci nebo model.' };
    if (session.agentRunning) return { allowed: false, reason: 'Agentní úloha právě běží. Počkej na její dokončení nebo ji zastav před přepnutím konverzace.' };
    if (session.workspaceBusy) return { allowed: false, reason: 'Právě probíhá změna projektu nebo import zálohy. Počkej, než se operace bezpečně dokončí.' };
    return { allowed: true, reason: '' };
  }
  function recoverInterruptedMessages(messages) {
    let recovered = false;
    const next = (Array.isArray(messages) ? messages : [])
      .filter(message => message && ['user', 'assistant', 'system'].includes(message.role) && typeof message.content === 'string')
      .map(message => ({ ...message }));
    const placeholder = /^(Přemýšlím…|Načítám omezený kontext|Prohledávám internet|Přepínám na )/;
    for (const message of next) {
      if (message.role !== 'assistant' || !(message.pending === true || placeholder.test(message.content))) continue;
      const content = message.content.trim();
      if (!content || placeholder.test(content)) {
        message.content = 'Generování bylo přerušeno zavřením aplikace nebo restartem. Odešli dotaz znovu a pokračuj.';
      } else {
        message.content = content + '\n\n*Generování bylo přerušeno zavřením aplikace nebo restartem. Výše uvedená odpověď může být neúplná.*';
      }
      delete message.pending;
      delete message.followups;
      recovered = true;
    }
    return { messages: next, recovered };
  }
  return Object.freeze({ canSwitchConversation, recoverInterruptedMessages });
});
