'use strict';

const RETRYABLE_HTTP_STATUSES = new Set([408, 425, 429, 500, 502, 503, 504]);
const DEFAULT_DELAYS_MS = Object.freeze([450, 1100]);
const MAX_RETRY_AFTER_MS = 8000;

function parseRetryAfter(value, now = Date.now()) {
  if (typeof value !== 'string' || !value.trim()) return null;
  const text = value.trim();
  if (/^\d+(?:\.\d+)?$/.test(text)) return clamp(Number(text) * 1000, 0, MAX_RETRY_AFTER_MS);
  const date = Date.parse(text);
  if (!Number.isFinite(date)) return null;
  return clamp(date - now, 0, MAX_RETRY_AFTER_MS);
}

function clamp(value, min, max) {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.round(value)));
}

function wait(ms, signal) {
  if (signal?.aborted) return Promise.reject(abortError());
  return new Promise((resolve, reject) => {
    const timer = setTimeout(done, ms);
    const onAbort = () => {
      clearTimeout(timer);
      signal?.removeEventListener('abort', onAbort);
      reject(abortError());
    };
    function done() {
      signal?.removeEventListener('abort', onAbort);
      resolve();
    }
    signal?.addEventListener('abort', onAbort, { once: true });
  });
}

function abortError() {
  const error = new Error('Požadavek byl zrušen.');
  error.name = 'AbortError';
  return error;
}

async function fetchWithRetry(url, init = {}, options = {}) {
  const fetchImpl = options.fetchImpl || globalThis.fetch;
  if (typeof fetchImpl !== 'function') throw new TypeError('Fetch adaptér není dostupný.');
  const maxAttempts = clamp(options.maxAttempts ?? 3, 1, 3);
  const signal = options.signal || init.signal;
  const sleepImpl = options.sleepImpl || wait;
  const now = options.now || Date.now;
  let lastError;

  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    if (signal?.aborted) throw abortError();
    try {
      const response = await fetchImpl(url, { ...init, signal });
      if (!RETRYABLE_HTTP_STATUSES.has(Number(response?.status)) || attempt + 1 >= maxAttempts) return response;
      try { await response.body?.cancel?.(); } catch { /* discard only the retry response body */ }
      const retryAfter = parseRetryAfter(response.headers?.get?.('retry-after') || '', now());
      const delay = retryAfter ?? DEFAULT_DELAYS_MS[Math.min(attempt, DEFAULT_DELAYS_MS.length - 1)];
      await sleepImpl(delay, signal);
    } catch (error) {
      if (signal?.aborted || error?.name === 'AbortError') throw abortError();
      lastError = error;
      if (!(error instanceof TypeError) || attempt + 1 >= maxAttempts) throw error;
      await sleepImpl(DEFAULT_DELAYS_MS[Math.min(attempt, DEFAULT_DELAYS_MS.length - 1)], signal);
    }
  }
  throw lastError || new Error('Vzdálený požadavek se nepodařilo dokončit.');
}

module.exports = Object.freeze({
  RETRYABLE_HTTP_STATUSES,
  DEFAULT_DELAYS_MS,
  MAX_RETRY_AFTER_MS,
  parseRetryAfter,
  fetchWithRetry
});
