(function (root) {
  'use strict';
  const RANGES = Object.freeze({
    temperature: [0, 2], top_p: [0.05, 1], top_k: [1, 200], repeat_penalty: [0.5, 2],
    num_ctx: [512, 131072], num_predict: [1, 32768], num_thread: [1, 128], seed: [0, 2147483647]
  });
  const INTEGER_KEYS = new Set(['top_k', 'num_ctx', 'num_predict', 'num_thread', 'seed']);
  const KEEP_ALIVE = Object.freeze({
    '0': 0, '1m': '1m', '5m': '5m', '10m': '10m', '30m': '30m', '1h': '1h'
  });
  function clamp(value, min, max, integer = false) {
    const n = Number(value);
    if (!Number.isFinite(n)) return null;
    const bounded = Math.max(min, Math.min(max, n));
    return integer ? Math.round(bounded) : bounded;
  }
  function normalizeOllamaOptions(raw) {
    const input = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : {};
    const output = {};
    for (const [key, [min, max]] of Object.entries(RANGES)) {
      const value = input[key];
      if (value === undefined || value === null || value === '') continue;
      const bounded = clamp(value, min, max, INTEGER_KEYS.has(key));
      if (bounded !== null) output[key] = bounded;
    }
    if (Array.isArray(input.stop)) {
      output.stop = input.stop.filter(value => typeof value === 'string')
        .map(value => value.slice(0, 100)).filter(Boolean).slice(0, 8);
    }
    return output;
  }
  function normalizeKeepAlive(value) {
    const key = String(value ?? '10m');
    return Object.prototype.hasOwnProperty.call(KEEP_ALIVE, key) ? KEEP_ALIVE[key] : '10m';
  }
  const api = { RANGES, KEEP_ALIVE, normalizeOllamaOptions, normalizeKeepAlive };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  if (root) root.AuroraGenerationOptions = api;
})(typeof window !== 'undefined' ? window : globalThis);
