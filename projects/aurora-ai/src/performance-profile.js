(function (root) {
  'use strict';
  const PRESETS = Object.freeze({
    quick: Object.freeze({ num_ctx: 2048, num_predict: 384, threadFactor: 0.5, label: 'Rychlý', description: 'Krátká odpověď, menší kontext a nižší zátěž.' }),
    balanced: Object.freeze({ num_ctx: 4096, num_predict: 1024, threadFactor: 0.65, label: 'Vyvážený', description: 'Rozumný poměr rychlosti a kvality.' }),
    deep: Object.freeze({ num_ctx: 8192, num_predict: 2048, threadFactor: 1, label: 'Důkladný', description: 'Více prostoru pro složitější úlohy.' }),
    extra: Object.freeze({ num_ctx: 8192, num_predict: 3072, threadFactor: 1, label: 'Extra', description: 'Delší výstup pro rozsáhlejší úkoly při zachování rozumné zátěže paměti.' })
  });
  function classifyPrompt(prompt) {
    const text = String(prompt || '').trim();
    if (!text) return 'balanced';
    const complex = /\b(analyzuj|analyzovat|porovnej|porovnání|navrhni architekturu|refactor|debug|debuguj|oprav|crash|stack trace|výzkum|research|podrobn|krok za krokem|projekt|implementuj|naprogramuj|vytvoř aplikaci|rozbor|důkladně|komplex)\b/i.test(text);
    const code = /```|\b(python|javascript|typescript|java|c\+\+|c#|lua|sql|api|regex|kód|skript|funkce|třída|kompil|build|testy)\b/i.test(text);
    if ((complex || code) || text.length > 700) return 'deep';
    if (text.length <= 180 && /\?$/.test(text) && !/\b(proč|jak navrhnout|analyzuj|vysvětli do hloubky)\b/i.test(text)) return 'quick';
    if (text.length <= 100 && /^(ahoj|čau|díky|děkuju|co je|kdo je|kolik je|přelož)\b/i.test(text)) return 'quick';
    return 'balanced';
  }
  function collectSamplingOptions(saved) {
    const options = {
      temperature: clamp(saved.temperature, 0, 2, 0.7),
      top_p: clamp(saved.top_p, 0.05, 1, 0.9),
      top_k: Math.round(clamp(saved.top_k, 1, 200, 40)),
      repeat_penalty: clamp(saved.repeat_penalty, 0.5, 2, 1.1)
    };
    const seed = saved.seed;
    if (seed !== '' && seed !== null && seed !== undefined && Number.isFinite(Number(seed))) {
      options.seed = Math.round(clamp(seed, 0, 2147483647, 0));
    }
    const stop = Array.isArray(saved.stop) ? saved.stop : String(saved.stop_sequences || '').split('|');
    const safeStop = stop.map(value => String(value).trim().slice(0, 100)).filter(Boolean).slice(0, 8);
    if (safeStop.length) options.stop = safeStop;
    return options;
  }
  function resolvePerformance(profile, prompt, preferences, hardware = {}) {
    const saved = preferences && typeof preferences === 'object' ? preferences : {};
    const mode = ['adaptive', 'quick', 'balanced', 'deep', 'extra', 'manual'].includes(profile) ? profile : 'adaptive';
    const selected = mode === 'adaptive' ? classifyPrompt(prompt) : mode === 'manual' ? 'manual' : mode;
    const cores = Math.max(1, Math.min(64, Number(hardware.cores) || Number(saved.num_thread) || 4));
    if (selected === 'manual') {
      return {
        mode, selected, label: 'Ruční nastavení', reason: 'Používám přesně tvoje nastavené hodnoty.',
        options: {
          ...collectSamplingOptions(saved),
          num_ctx: clamp(saved.num_ctx, 1024, 131072, 4096),
          num_predict: clamp(saved.num_predict, 64, 8192, 1024),
          num_thread: clamp(saved.num_thread, 1, cores, Math.min(4, cores))
        }
      };
    }
    const preset = PRESETS[selected];
    const threads = Math.max(1, Math.min(cores, Math.ceil(cores * preset.threadFactor)));
    return {
      mode, selected, label: preset.label,
      reason: mode === 'adaptive' ? `Automaticky: ${preset.description}` : preset.description,
      options: { ...collectSamplingOptions(saved), num_ctx: preset.num_ctx, num_predict: preset.num_predict, num_thread: threads }
    };
  }
  function clamp(value, min, max, fallback) { const n = Number(value); return Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : fallback; }
  const api = { classifyPrompt, resolvePerformance, collectSamplingOptions, PRESETS };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraPerformance = api;
})(typeof window !== 'undefined' ? window : globalThis);
