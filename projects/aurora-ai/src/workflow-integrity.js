'use strict';
const crypto = require('node:crypto');

function sha256(content) {
  return crypto.createHash('sha256').update(content).digest('hex');
}

function assertSavedDigest(expected, content) {
  const actual = sha256(content);
  if (expected === undefined) return actual; // Only truly missing fields are treated as legacy settings.
  if (typeof expected !== 'string' || !/^[a-f0-9]{64}$/.test(expected)) {
    throw new Error('Kontrolní otisk uloženého ComfyUI workflow je neplatný. Vyber workflow znovu.');
  }
  if (expected !== actual) {
    throw new Error('Soubor ComfyUI workflow se od posledního výběru změnil. Vyber ho znovu, aby se změny výslovně potvrdily.');
  }
  return actual;
}

module.exports = Object.freeze({ sha256, assertSavedDigest });
