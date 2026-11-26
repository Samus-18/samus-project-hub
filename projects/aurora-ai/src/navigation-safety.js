'use strict';
const path = require('node:path');
const { fileURLToPath } = require('node:url');

function isAllowedAppNavigation(rawUrl, entryPath) {
  if (typeof rawUrl !== 'string' || typeof entryPath !== 'string' || !entryPath) return false;
  try {
    const url = new URL(rawUrl);
    if (url.protocol !== 'file:' || url.search || url.hash || url.username || url.password || url.port) return false;
    const candidatePath = fileURLToPath(url);
    return path.resolve(candidatePath) === path.resolve(entryPath);
  } catch {
    return false;
  }
}

module.exports = Object.freeze({ isAllowedAppNavigation });
