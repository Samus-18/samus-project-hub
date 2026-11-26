'use strict';

function finiteCount(value) {
  const number = Number(value);
  return Number.isSafeInteger(number) && number >= 0 && number <= 10_000_000 ? number : null;
}

function finiteDuration(value) {
  const number = Number(value);
  return Number.isFinite(number) && number >= 0 && number <= 24 * 60 * 60 * 1e9 ? number : null;
}

function rate(tokens, durationNs) {
  if (!Number.isFinite(tokens) || !Number.isFinite(durationNs) || durationNs <= 0) return null;
  const result = tokens / (durationNs / 1e9);
  return Number.isFinite(result) && result >= 0 && result <= 1_000_000 ? Math.round(result * 10) / 10 : null;
}

function fromDoneMessage(message) {
  const source = message && typeof message === 'object' ? message : {};
  const promptTokens = finiteCount(source.prompt_eval_count);
  const evalTokens = finiteCount(source.eval_count);
  const promptDurationNs = finiteDuration(source.prompt_eval_duration);
  const evalDurationNs = finiteDuration(source.eval_duration);
  return Object.freeze({
    promptTokens,
    evalTokens,
    promptDurationNs,
    evalDurationNs,
    totalDurationNs: finiteDuration(source.total_duration),
    loadDurationNs: finiteDuration(source.load_duration),
    tokensPerSecond: rate(evalTokens, evalDurationNs),
    promptTokensPerSecond: rate(promptTokens, promptDurationNs)
  });
}

module.exports = Object.freeze({ finiteCount, finiteDuration, rate, fromDoneMessage });
