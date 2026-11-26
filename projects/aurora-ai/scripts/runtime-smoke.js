'use strict';

const DEFAULT_OLLAMA_URL = 'http://127.0.0.1:11434';
const DEFAULT_COMFYUI_URL = 'http://127.0.0.1:8188';
const ALLOWED_LOCAL_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1']);
const MAX_JSON_BYTES = 2 * 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 3500;

function normalizeLocalBase(raw, label = 'service') {
  let parsed;
  try { parsed = new URL(String(raw || '')); }
  catch { throw new Error(`${label}: URL is invalid.`); }
  if (!['http:', 'https:'].includes(parsed.protocol)) throw new Error(`${label}: only HTTP(S) is supported.`);
  if (!ALLOWED_LOCAL_HOSTS.has(parsed.hostname.toLowerCase())) {
    throw new Error(`${label}: only localhost / loopback addresses are allowed by this privacy-focused smoke test.`);
  }
  if (parsed.username || parsed.password || parsed.search || parsed.hash) throw new Error(`${label}: credentials, query strings and fragments are not allowed.`);
  if (parsed.pathname !== '/' && parsed.pathname !== '') throw new Error(`${label}: provide the service base URL without a path.`);
  return parsed.origin;
}

async function fetchBoundedJson(baseUrl, route, options = {}) {
  const base = normalizeLocalBase(baseUrl, options.label || 'service');
  const maxBytes = Number.isSafeInteger(options.maxBytes) && options.maxBytes > 0 ? Math.min(options.maxBytes, MAX_JSON_BYTES) : MAX_JSON_BYTES;
  const timeoutMs = Number.isSafeInteger(options.timeoutMs) && options.timeoutMs > 0 ? Math.min(options.timeoutMs, 15000) : DEFAULT_TIMEOUT_MS;
  const url = new URL(route.replace(/^\/+/, ''), `${base}/`);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await (options.fetchImpl || fetch)(url, {
      method: 'GET',
      headers: { accept: 'application/json' },
      redirect: 'error',
      signal: controller.signal
    });
    if (!response.ok) {
      try { await response.body?.cancel(); } catch {}
      throw new Error(`HTTP ${response.status}`);
    }
    const declaredLength = Number(response.headers.get('content-length'));
    if (Number.isFinite(declaredLength) && declaredLength > maxBytes) {
      try { await response.body?.cancel(); } catch {}
      throw new Error(`Response exceeded the ${maxBytes}-byte limit.`);
    }
    if (!response.body) throw new Error('Response body is empty.');
    const reader = response.body.getReader();
    const chunks = [];
    let total = 0;
    try {
      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        total += value.byteLength;
        if (total > maxBytes) {
          await reader.cancel('Response limit exceeded.').catch(() => {});
          throw new Error(`Response exceeded the ${maxBytes}-byte limit.`);
        }
        chunks.push(Buffer.from(value));
      }
    } finally {
      reader.releaseLock();
    }
    let parsed;
    try { parsed = JSON.parse(Buffer.concat(chunks, total).toString('utf8')); }
    catch { throw new Error('Response was not valid JSON.'); }
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('JSON response must be an object.');
    return parsed;
  } catch (error) {
    if (error?.name === 'AbortError') throw new Error(`Timed out after ${timeoutMs} ms.`);
    throw error;
  } finally {
    clearTimeout(timer);
  }
}

function extractOllamaModels(payload) {
  if (!Array.isArray(payload?.models)) throw new Error('Ollama /api/tags returned no models array.');
  return payload.models.slice(0, 300).map((model) => {
    if (!model || typeof model.name !== 'string' || !model.name.trim()) return null;
    return model.name.trim().slice(0, 180);
  }).filter(Boolean);
}

async function inspectOllama(baseUrl = DEFAULT_OLLAMA_URL, options = {}) {
  const version = await fetchBoundedJson(baseUrl, '/api/version', { ...options, label: 'Ollama' });
  if (typeof version.version !== 'string' || !version.version.trim()) throw new Error('Ollama returned no version field.');
  const tags = await fetchBoundedJson(baseUrl, '/api/tags', { ...options, label: 'Ollama' });
  const models = extractOllamaModels(tags);
  return { service: 'Ollama', url: normalizeLocalBase(baseUrl, 'Ollama'), version: version.version.slice(0, 80), modelCount: models.length, models };
}

async function inspectComfyUI(baseUrl = DEFAULT_COMFYUI_URL, options = {}) {
  const stats = await fetchBoundedJson(baseUrl, '/system_stats', { ...options, label: 'ComfyUI' });
  if (!stats.system || typeof stats.system !== 'object' || !Array.isArray(stats.devices)) {
    throw new Error('ComfyUI /system_stats response did not include the expected system and devices fields.');
  }
  return {
    service: 'ComfyUI',
    url: normalizeLocalBase(baseUrl, 'ComfyUI'),
    os: typeof stats.system.os === 'string' ? stats.system.os.slice(0, 120) : 'unknown',
    deviceCount: stats.devices.length,
    devices: stats.devices.slice(0, 12).map((device) => ({
      name: typeof device?.name === 'string' ? device.name.slice(0, 120) : 'unnamed device',
      type: typeof device?.type === 'string' ? device.type.slice(0, 80) : 'unknown'
    }))
  };
}

async function runLocalSmoke(options = {}) {
  const ollamaUrl = options.ollamaUrl || process.env.AURORA_OLLAMA_URL || DEFAULT_OLLAMA_URL;
  const comfyUrl = options.comfyUrl || process.env.AURORA_COMFYUI_URL || DEFAULT_COMFYUI_URL;
  const settled = await Promise.allSettled([
    inspectOllama(ollamaUrl, options),
    inspectComfyUI(comfyUrl, options)
  ]);
  const services = settled.map((result, index) => ({
    service: index === 0 ? 'Ollama' : 'ComfyUI',
    ok: result.status === 'fulfilled',
    details: result.status === 'fulfilled' ? result.value : { error: String(result.reason?.message || result.reason).slice(0, 240) }
  }));
  return { ok: services.every(item => item.ok), services };
}

if (require.main === module) {
  runLocalSmoke().then((report) => {
    console.log('\nAurora local runtime smoke test (loopback only)');
    for (const item of report.services) {
      console.log(`${item.ok ? 'PASS' : 'FAIL'} ${item.service}`);
      if (item.ok && item.service === 'Ollama') {
        console.log(`  version: ${item.details.version}`);
        console.log(`  installed models: ${item.details.modelCount}${item.details.models.length ? ` (${item.details.models.join(', ')})` : ''}`);
      } else if (item.ok && item.service === 'ComfyUI') {
        console.log(`  host OS: ${item.details.os}; devices: ${item.details.deviceCount}`);
        for (const device of item.details.devices) console.log(`  - ${device.name} (${device.type})`);
      } else {
        console.log(`  ${item.details.error}`);
      }
    }
    if (!report.ok) {
      console.error('\nOne or more local services are unavailable or did not return the expected response. Start the service and retry.');
      process.exitCode = 1;
    } else {
      console.log('\nBoth local services responded. This checks service availability only; it does not execute a model generation.');
    }
  }).catch((error) => {
    console.error(`Runtime smoke test failed: ${String(error?.message || error)}`);
    process.exitCode = 1;
  });
}

module.exports = Object.freeze({
  DEFAULT_OLLAMA_URL,
  DEFAULT_COMFYUI_URL,
  MAX_JSON_BYTES,
  normalizeLocalBase,
  fetchBoundedJson,
  extractOllamaModels,
  inspectOllama,
  inspectComfyUI,
  runLocalSmoke
});
