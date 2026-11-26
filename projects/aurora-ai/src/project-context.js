(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.AuroraProjectContext = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  const priority = [
    /(^|\/)readme(?:\.[^/]*)?$/i,
    /(^|\/)projectversion\.txt$/i,
    /(^|\/)manifest\.json$/i,
    /(^|\/)packages-lock\.json$/i,
    /(^|\/)package\.json$/i,
    /\.asmdef$/i,
    /\.cs$/i,
    /\.log$/i,
    /\.md$/i,
    /\.json$/i,
  ];

  function selectRelevantProjectFiles(files, limit = 8) {
    if (!Array.isArray(files)) return [];
    const ranked = files
      .filter((file) => file && typeof file.path === 'string' && file.path.trim())
      .map((file, index) => {
        const path = file.path.replace(/\\/g, '/').replace(/^\.\//, '');
        const rank = priority.findIndex((pattern) => pattern.test(path));
        return { file, path, rank: rank < 0 ? priority.length + 1 : rank, index };
      })
      .filter((item) => item.rank <= priority.length)
      .sort((a, b) => a.rank - b.rank || a.path.localeCompare(b.path, 'en') || a.index - b.index);
    const cap = Math.max(0, Math.min(20, Number.isFinite(Number(limit)) ? Math.floor(Number(limit)) : 8));
    if (cap === 0) return [];
    const seen = new Set();
    const output = [];
    for (const item of ranked) {
      const key = item.path.toLocaleLowerCase('en');
      if (seen.has(key)) continue;
      seen.add(key);
      output.push({ ...item.file, path: item.path });
      if (output.length >= cap) break;
    }
    return output;
  }

  function formatFileManifest(files, limit = 80) {
    if (!Array.isArray(files)) return '';
    const cap = Math.max(0, Math.min(200, Number.isFinite(Number(limit)) ? Math.floor(Number(limit)) : 80));
    return files.slice(0, cap).filter((file) => file && typeof file.path === 'string')
      .map((file) => `${file.path} (${Math.max(0, Number(file.size) || 0)} B)`).join('\n');
  }

  return { selectRelevantProjectFiles, formatFileManifest };
});
