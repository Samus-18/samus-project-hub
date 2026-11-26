'use strict';

const ROLE_IDS = Object.freeze(['planner', 'researcher', 'implementer', 'reviewer']);
const PROVIDERS = Object.freeze(['ollama', 'openai', 'anthropic', 'gemini']);
const DEFAULT_REMOTE_MODELS = Object.freeze({
  openai: 'gpt-6-luna',
  anthropic: 'claude-haiku-4-5-20251001',
  gemini: 'gemini-3.5-flash-lite'
});
const MODEL_PATTERN = /^[a-zA-Z0-9._-]+(?:\/[a-zA-Z0-9._-]+)*(?::[a-zA-Z0-9._-]+)?$/;
const REMOTE_MODEL_PATTERN = /^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,119}$/;

function validModel(value) {
  const model = typeof value === 'string' ? value.trim() : '';
  return MODEL_PATTERN.test(model) && model.length <= 120 && !model.includes('..') && !model.includes('//') && !model.endsWith('/') && !model.split('/').some(part => part === '.' || part === '..') ? model : '';
}
function validRemoteModel(value) {
  const model = typeof value === 'string' ? value.trim() : '';
  return REMOTE_MODEL_PATTERN.test(model) && !model.includes('..') && !model.includes('//') && !model.endsWith('/') ? model : '';
}
function isChatCapableModel(name) {
  const lower = String(name || '').toLowerCase();
  return !!validModel(name) && !/(embed|embedding|rerank|whisper|speech|tts|text-to-speech|clip|vision-encoder|bge-m3)/.test(lower);
}

function createDefaultAgentConfig() {
  return Object.fromEntries(ROLE_IDS.map(role => [role, {
    provider: 'ollama',
    localModel: '',
    remoteModels: { ...DEFAULT_REMOTE_MODELS }
  }]));
}

function normalizeAgentConfig(raw) {
  const defaults = createDefaultAgentConfig();
  const input = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : {};
  for (const role of ROLE_IDS) {
    const source = input[role] && typeof input[role] === 'object' ? input[role] : {};
    const provider = PROVIDERS.includes(source.provider) ? source.provider : 'ollama';
    const remoteInput = source.remoteModels && typeof source.remoteModels === 'object' ? source.remoteModels : {};
    const remoteModels = {};
    for (const remoteProvider of Object.keys(DEFAULT_REMOTE_MODELS)) {
      const remoteModel = validRemoteModel(remoteInput[remoteProvider]);
      const retiredDefault = remoteProvider === 'anthropic' && /^(claude-3-5-haiku-latest|claude-3-5-haiku-20241022)$/.test(remoteModel) || remoteProvider === 'gemini' && remoteModel === 'gemini-2.0-flash';
      remoteModels[remoteProvider] = !remoteModel || retiredDefault ? DEFAULT_REMOTE_MODELS[remoteProvider] : remoteModel;
    }
    defaults[role] = {
      provider,
      localModel: validModel(source.localModel),
      remoteModels
    };
  }
  return defaults;
}

function getAgentRequestConfig(config, role, localModels = [], fallbackModel = '') {
  if (!ROLE_IDS.includes(role)) throw new TypeError(`Neznámá role agenta: ${role}`);
  const normalized = normalizeAgentConfig(config);
  const selected = normalized[role];
  if (selected.provider === 'ollama') {
    const available = Array.isArray(localModels)
      ? localModels.map(item => typeof item === 'string' ? item : item?.name).filter(isChatCapableModel)
      : [];
    if (!available.length) throw new Error('Ollama nemá načtený žádný dostupný chatovací model. Spusť Ollama a obnov Model Hub.');
    let model = selected.localModel || validModel(fallbackModel) || available[0] || '';
    if (available.length && !available.includes(model)) model = available.includes(fallbackModel) ? fallbackModel : available[0];
    if (!validModel(model)) throw new Error(`Pro roli ${role} vyber chatovací model Ollama.`);
    if (available.length && !available.includes(model)) throw new Error(`Model ${model} není dostupný v Ollama.`);
    return { role, provider: 'ollama', model };
  }
  const model = selected.remoteModels[selected.provider];
  if (!validRemoteModel(model)) throw new Error(`Pro roli ${role} není platný vzdálený model.`);
  return { role, provider: selected.provider, model };
}

const AuroraAgentConfig = {
  ROLE_IDS,
  PROVIDERS,
  DEFAULT_REMOTE_MODELS,
  createDefaultAgentConfig,
  normalizeAgentConfig,
  getAgentRequestConfig,
  validModel,
  validRemoteModel,
  isChatCapableModel
};
if (typeof module !== 'undefined' && module.exports) module.exports = AuroraAgentConfig;
if (typeof window !== 'undefined') window.AuroraAgentConfig = AuroraAgentConfig;
