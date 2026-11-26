(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.AuroraAgentChangeProposals = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  const MAX_EDITS = 12;
  const MAX_CONTENT_CHARS = 500_000;
  const MAX_BLOCK_CHARS = 6_500_000;
  const ALLOWED_EXTENSIONS = new Set([
    '.txt', '.md', '.json', '.csv', '.tsv', '.js', '.mjs', '.cjs', '.ts', '.tsx', '.jsx', '.css', '.scss', '.html', '.xml', '.yaml', '.yml', '.toml',
    '.cs', '.shader', '.compute', '.asmdef', '.py', '.gd', '.tscn', '.tres', '.godot', '.lua', '.rs', '.go', '.java', '.kt', '.swift',
    '.c', '.h', '.cpp', '.hpp', '.sql', '.ini', '.cfg', '.gitignore', '.csproj', '.sln', '.props', '.targets'
  ]);

  function normalizeRelativePath(value) {
    if (typeof value !== 'string') throw new TypeError('Cesta změny musí být text.');
    const raw = value.trim().replace(/\\/g, '/');
    if (!raw || raw.length > 220 || raw.startsWith('/') || /^[a-zA-Z]:/.test(raw) || raw.includes('\0')) throw new Error('Cesta změny musí být relativní a kratší než 220 znaků.');
    const parts = raw.split('/');
    if (parts.some(part => !part || part === '.' || part === '..' || part.startsWith('~'))) throw new Error('Cesta změny obsahuje nepovolený segment.');
    if (parts.some(part => /[<>:"|?*]/.test(part))) throw new Error('Cesta změny obsahuje nepovolené znaky.');
    const normalized = parts.join('/');
    const last = parts[parts.length - 1];
    const dot = last.lastIndexOf('.');
    const extension = dot >= 0 ? last.slice(dot).toLowerCase() : '';
    if (!ALLOWED_EXTENSIONS.has(extension) && last.toLowerCase() !== '.gitignore') throw new Error(`Typ souboru ${extension || '(bez přípony)'} není povolen pro návrhy změn.`);
    return normalized;
  }

  function validateChangeSet(raw) {
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) throw new TypeError('Návrh změn musí být JSON objekt.');
    if (!Array.isArray(raw.edits) || raw.edits.length < 1 || raw.edits.length > MAX_EDITS) throw new Error(`Návrh musí obsahovat 1 až ${MAX_EDITS} souborů.`);
    const seen = new Set();
    const edits = raw.edits.map((item, index) => {
      if (!item || typeof item !== 'object' || Array.isArray(item)) throw new TypeError(`Položka ${index + 1} není objekt.`);
      const path = normalizeRelativePath(item.path);
      const key = path.toLocaleLowerCase('en');
      if (seen.has(key)) throw new Error(`Návrh obsahuje duplicitní cestu: ${path}`);
      seen.add(key);
      if (typeof item.content !== 'string') throw new TypeError(`Obsah ${path} musí být text.`);
      if (item.content.length > MAX_CONTENT_CHARS) throw new Error(`Obsah ${path} překračuje limit 500 kB.`);
      return { path, content: item.content };
    });
    return {
      summary: typeof raw.summary === 'string' ? raw.summary.trim().slice(0, 500) : 'Návrh změn z Agent Studia',
      tests: Array.isArray(raw.tests) ? raw.tests.filter(x => typeof x === 'string').slice(0, 12).map(x => x.slice(0, 300)) : [],
      edits
    };
  }

  function extractChangeSet(text) {
    const source = String(text || '');
    const match = source.match(/```aurora-changeset\s*\n([\s\S]*?)\n```/i);
    if (!match) return { found: false, proposal: null, error: '' };
    if (match[1].length > MAX_BLOCK_CHARS) return { found: true, proposal: null, error: 'Blok návrhu změn překračuje bezpečný limit.' };
    let parsed;
    try { parsed = JSON.parse(match[1]); }
    catch { return { found: true, proposal: null, error: 'Blok aurora-changeset neobsahuje platný JSON.' }; }
    try { return { found: true, proposal: validateChangeSet(parsed), error: '' }; }
    catch (error) { return { found: true, proposal: null, error: String(error?.message || error).slice(0, 400) }; }
  }

  return Object.freeze({ MAX_EDITS, MAX_CONTENT_CHARS, ALLOWED_EXTENSIONS, normalizeRelativePath, validateChangeSet, extractChangeSet });
});
