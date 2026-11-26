(function (root) {
  'use strict';
  const MAX_SAMPLES = 40;
  function createSample(input = {}) {
    const totalMs = clamp(input.totalMs, 0, 24 * 60 * 60 * 1000, null);
    const firstTokenMs = clamp(input.firstTokenMs, 0, totalMs ?? 24 * 60 * 60 * 1000, null);
    const outputChars = Math.round(clamp(input.outputChars, 0, 10_000_000, 0));
    return {
      at: typeof input.at === 'string' ? input.at : new Date().toISOString(),
      provider: String(input.provider || 'unknown').slice(0, 40),
      model: String(input.model || 'unknown').slice(0, 120),
      profile: String(input.profile || 'balanced').slice(0, 24),
      totalMs,
      firstTokenMs,
      outputChars,
      charsPerSecond: totalMs > 0 ? Math.round(outputChars / (totalMs / 1000) * 10) / 10 : null,
      contextChars: Math.round(clamp(input.contextChars, 0, 10_000_000, 0)),
      omittedMessages: Math.round(clamp(input.omittedMessages, 0, 100_000, 0)),
      completed: input.completed !== false
    };
  }
  function appendSample(samples, input) {
    const valid = Array.isArray(samples) ? samples.filter(s => s && typeof s === 'object') : [];
    return [...valid, createSample(input)].slice(-MAX_SAMPLES);
  }
  function summarize(samples) {
    const valid = (Array.isArray(samples) ? samples : []).filter(s => s && s.completed && Number.isFinite(s.totalMs) && s.totalMs > 0);
    if (!valid.length) return { count: 0, averageMs: null, medianMs: null, averageFirstTokenMs: null, averageCharsPerSecond: null };
    const sorted = valid.map(s => s.totalMs).sort((a,b) => a-b);
    const average = key => { const values = valid.map(s => s[key]).filter(Number.isFinite); return values.length ? Math.round(values.reduce((a,b)=>a+b,0)/values.length) : null; };
    const mid = Math.floor(sorted.length / 2);
    return { count: valid.length, averageMs: average('totalMs'), medianMs: Math.round(sorted.length % 2 ? sorted[mid] : (sorted[mid-1]+sorted[mid])/2), averageFirstTokenMs: average('firstTokenMs'), averageCharsPerSecond: average('charsPerSecond') };
  }
  function clamp(value, min, max, fallback) { const n = Number(value); return Number.isFinite(n) ? Math.max(min, Math.min(max, n)) : fallback; }
  const api = { MAX_SAMPLES, createSample, appendSample, summarize };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraPerformanceTelemetry = api;
})(typeof window !== 'undefined' ? window : globalThis);
