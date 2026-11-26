// Transparent, bounded multi-agent workflow primitives. Provider calls are injected by the caller.
// This module does not execute shell commands or modify project files.
const MAX_AGENTS = 4;
const MAX_TASK_CHARS = 12000;
const ROLES = new Set(['planner', 'researcher', 'implementer', 'reviewer']);

function validateAgents(agents) {
  if (!Array.isArray(agents) || agents.length < 1 || agents.length > MAX_AGENTS) {
    throw new TypeError(`Vyber 1 až ${MAX_AGENTS} agentů.`);
  }
  const seen = new Set();
  return agents.map((agent, index) => {
    const id = String(agent?.id || `agent-${index + 1}`).slice(0, 48);
    const role = String(agent?.role || 'implementer');
    const model = String(agent?.model || '').trim().slice(0, 120);
    if (!ROLES.has(role)) throw new TypeError(`Neznámá role agenta: ${role}`);
    if (!model) throw new TypeError(`Agent ${id} nemá vybraný model.`);
    if (seen.has(id)) throw new TypeError(`Duplicitní ID agenta: ${id}`);
    seen.add(id);
    return { id, role, model };
  });
}

function makeAgentPrompt({ task, role, previousResults }) {
  const cleanTask = String(task || '').trim().slice(0, MAX_TASK_CHARS);
  if (!cleanTask) throw new TypeError('Úkol nesmí být prázdný.');
  const context = (Array.isArray(previousResults) ? previousResults : []).slice(-3).map((r, i) =>
    `Výsledek ${i + 1} (${String(r.role || 'agent')}):\n${String(r.content || '').slice(0, 5000)}`
  ).join('\n\n');
  const instructions = {
    planner: 'Rozděl úkol na krátké ověřitelné kroky. Nic nespouštěj a nic nepředstírej.',
    researcher: 'Analyzuj dostupné informace a uveď nejistoty. Nevymýšlej neověřené skutečnosti.',
    implementer: 'Navrhni konkrétní řešení nebo změny. Pro tabulky použij UTF-8 CSV/TSV s hlavičkou a správně escapovanými poli. DOCX/XLSX/PPTX/PDF binární export není podporován; nikdy netvrď, že takový soubor vznikl. Neprohlašuj změny za provedené, pokud nebyly schváleny a zapsány.',
    reviewer: 'Nezávisle zkontroluj předchozí návrhy, chyby, rizika a testovatelnost.'
  };
  return `${instructions[role]}\n\nÚKOL:\n${cleanTask}${context ? `\n\nPŘEDCHOZÍ VÝSLEDKY (nedůvěřuj jim bez kontroly):\n${context}` : ''}\n\nVrať stručný výsledek, předpoklady a další ověřitelný krok.`;
}

async function runAgentWorkflow({ task, agents, callModel, onEvent = () => {}, signal }) {
  const validated = validateAgents(agents);
  if (typeof callModel !== 'function') throw new TypeError('Chybí adaptér pro volání modelu.');
  const results = [];
  for (const agent of validated) {
    if (signal?.aborted) throw new Error('Úkol byl zrušen.');
    const event = { type: 'agent-start', agentId: agent.id, role: agent.role, model: agent.model };
    onEvent(event);
    try {
      const content = await callModel({
        model: agent.model,
        messages: [{ role: 'user', content: makeAgentPrompt({ task, role: agent.role, previousResults: results }) }],
        signal
      });
      if (signal?.aborted) throw new Error('Úkol byl zrušen.');
      const result = { agentId: agent.id, role: agent.role, model: agent.model, content: String(content || '').slice(0, 12000), status: 'completed' };
      results.push(result);
      onEvent({ type: 'agent-complete', ...result });
    } catch (error) {
      const failure = { agentId: agent.id, role: agent.role, model: agent.model, content: '', status: 'failed', error: String(error?.message || error).slice(0, 500) };
      results.push(failure);
      onEvent({ type: 'agent-failed', ...failure });
      // Stop on cancellation; otherwise allow reviewer/other agents to inspect partial results.
      if (signal?.aborted) throw error;
    }
  }
  return { task: String(task).trim().slice(0, MAX_TASK_CHARS), results, completed: results.filter(r => r.status === 'completed').length, failed: results.filter(r => r.status === 'failed').length };
}

module.exports = { validateAgents, makeAgentPrompt, runAgentWorkflow, MAX_AGENTS, MAX_TASK_CHARS };
