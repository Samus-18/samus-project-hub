'use strict';

async function readTextBounded(response, maxBytes = 1_000_000) {
  const limit = Math.max(1, Math.min(50_000_000, Math.trunc(Number(maxBytes) || 1_000_000)));
  const reader = response?.body?.getReader?.();
  if (!reader) return '';
  const decoder = new TextDecoder();
  let result = '';
  let total = 0;
  try {
    while (total < limit) {
      const { value, done } = await reader.read();
      if (done) break;
      const remaining = limit - total;
      const chunk = value.subarray(0, remaining);
      result += decoder.decode(chunk, { stream: true });
      total += chunk.byteLength;
      if (chunk.byteLength < value.byteLength || total >= limit) {
        try { await reader.cancel(); } catch {}
        break;
      }
    }
    result += decoder.decode();
    return result;
  } finally {
    try { reader.releaseLock(); } catch {}
  }
}

async function readJsonBounded(response, maxBytes = 1_000_000, label = 'JSON odpověď') {
  const limit = Math.max(1, Math.min(50_000_000, Math.trunc(Number(maxBytes) || 1_000_000)));
  const declared = Number(response?.headers?.get?.('content-length') || 0);
  if (Number.isFinite(declared) && declared > limit) {
    try { await response?.body?.cancel?.(); } catch {}
    throw new Error(`${label} překročila limit ${Math.round(limit / 1024)} kB.`);
  }
  const text = await readTextBounded(response, limit + 1);
  if (Buffer.byteLength(text, 'utf8') > limit) throw new Error(`${label} překročila limit ${Math.round(limit / 1024)} kB.`);
  try { return JSON.parse(text); }
  catch { throw new Error(`${label} neobsahuje platný JSON.`); }
}

module.exports = Object.freeze({ readTextBounded, readJsonBounded });
