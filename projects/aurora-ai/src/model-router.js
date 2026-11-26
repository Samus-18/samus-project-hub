(function (root) {
  'use strict';
  const BENCHMARK_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;
  function modelSizeB(name) {
    const match = String(name || '').match(/(?:^|:)(\d+(?:\.\d+)?)b\b/i) || String(name || '').match(/(\d+(?:\.\d+)?)b\b/i);
    return match ? Number(match[1]) : null;
  }
  function isChatModel(name) {
    const value = String(name || '').trim().toLowerCase();
    if (!value) return false;
    // Ollama also lists embedding/reranking/speech models; they cannot answer chat prompts.
    return !/(?:embed(?:ding)?|rerank(?:er)?|whisper|text-to-speech|speech-to-text|\btts\b|\basr\b|tokenizer)/i.test(value);
  }
  function benchmarkIsFresh(result, now = Date.now()) {
    if (!result || !Number.isFinite(Number(result.tokensPerSecond)) || Number(result.tokensPerSecond) <= 0) return false;
    const measured = Date.parse(result.measuredAt || '');
    return Number.isFinite(measured) && measured <= now && now - measured <= BENCHMARK_MAX_AGE_MS;
  }
  function classifyTask(prompt) {
    const text = String(prompt || '').toLowerCase();
    if (/\b(code|kód\w*|program|script|unity|debug|lad[ěe]n[ií]|crash|exception|stack trace|logy|soubor|refactor|typescript|javascript|python|c#|\.cs\b|\.js\b|\.py\b)/i.test(text)) return 'coding';
    if (/(analyzuj|navrhni architekt|komplex|podrobn|strateg|porovnej|pl[aá]n|prozkoumej|research|why does|proč\s+to|krok za krokem)/i.test(text) || text.length > 700) return 'complex';
    return 'simple';
  }
  function chooseModel(models, prompt, benchmarks = {}, hardware = {}, now = Date.now()) {
    const rawNames = (Array.isArray(models) ? models : []).map(item => typeof item === 'string' ? item : item?.name).filter(name => typeof name === 'string' && name.trim());
    if (!rawNames.length) return { model: '', task: classifyTask(prompt), reason: 'no-models', usedBenchmark: false };
    const names = rawNames.filter(isChatModel);
    if (!names.length) return { model: '', task: classifyTask(prompt), reason: 'no-chat-models', usedBenchmark: false };
    const task = classifyTask(prompt);
    const freeRamGB = Number(hardware.freeRamGB);
    let candidates = names.map(name => {
      const benchmark = benchmarks[name];
      const fresh = benchmarkIsFresh(benchmark, now);
      return { name, size: modelSizeB(name), benchmark: fresh ? benchmark : null, fresh, lower: name.toLowerCase() };
    });
    // Avoid recommending very large models when memory pressure is already high.
    if (Number.isFinite(freeRamGB) && freeRamGB < 3) {
      const safer = candidates.filter(m => !m.size || m.size <= 4);
      if (safer.length) candidates = safer;
    } else if (Number.isFinite(freeRamGB) && freeRamGB < 5) {
      const safer = candidates.filter(m => !m.size || m.size <= 8);
      if (safer.length) candidates = safer;
    }
    if (task === 'coding') {
      const codeModels = candidates.filter(m => /coder|codegemma|starcoder|codestral|deepseek-coder/.test(m.lower));
      if (codeModels.length) candidates = codeModels;
    }
    if (task === 'simple') {
      const small = candidates.filter(m => m.size && m.size <= 4);
      if (small.length) candidates = small;
    } else if (task === 'complex') {
      const capable = candidates.filter(m => !m.size || m.size >= 3);
      if (capable.length) candidates = capable;
    }
    candidates.sort((a, b) => {
      const aSpeed = Number(a.benchmark?.tokensPerSecond) || 0;
      const bSpeed = Number(b.benchmark?.tokensPerSecond) || 0;
      if (aSpeed !== bSpeed) return bSpeed - aSpeed;
      if (task === 'complex') return (b.size || 0) - (a.size || 0);
      return (a.size || 99) - (b.size || 99);
    });
    const chosen = candidates[0];
    return { model: chosen.name, task, reason: chosen.fresh ? 'fresh-benchmark' : 'size-heuristic', usedBenchmark: !!chosen.fresh };
  }
  const api = { BENCHMARK_MAX_AGE_MS, modelSizeB, isChatModel, benchmarkIsFresh, classifyTask, chooseModel };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraModelRouter = api;
})(typeof window !== 'undefined' ? window : globalThis);
