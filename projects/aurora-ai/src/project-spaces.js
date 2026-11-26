'use strict';
const path = require('node:path');

function normalizeName(value, fallback = 'Nový projekt') {
  const name = String(value ?? '').replace(/[\u0000-\u001f\u007f]/g, ' ').replace(/[\\/:*?"<>|]/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 64);
  return name || fallback;
}
function normalizeProjectSpace(item) {
  if (!item || typeof item !== 'object' || Array.isArray(item)) return null;
  const id = typeof item.id === 'string' ? item.id.slice(0, 100) : '';
  const folderPath = typeof item.folderPath === 'string' ? item.folderPath.trim() : '';
  if (!/^[a-zA-Z0-9_-]{8,100}$/.test(id) || !folderPath || folderPath.length > 2048 || !path.isAbsolute(folderPath)) return null;
  const resolved = path.resolve(folderPath);
  const updatedAt = Number(item.updatedAt);
  return {
    id,
    name: normalizeName(item.name, normalizeName(item.folderName, 'Projekt')),
    folderPath: resolved,
    folderName: normalizeName(item.folderName || path.basename(resolved), 'Projekt').slice(0, 120),
    updatedAt: Number.isFinite(updatedAt) && updatedAt >= 0 ? Math.min(updatedAt, Date.now() + 60_000) : 0
  };
}
function normalizeProjectSpaces(input) {
  const rows = Array.isArray(input) ? input : [];
  const seenIds = new Set();
  const seenPaths = new Set();
  const result = [];
  for (const raw of rows.slice(0, 200)) {
    const item = normalizeProjectSpace(raw);
    if (!item) continue;
    const pathKey = process.platform === 'win32' ? item.folderPath.toLocaleLowerCase('en-US') : item.folderPath;
    if (seenIds.has(item.id) || seenPaths.has(pathKey)) continue;
    seenIds.add(item.id);
    seenPaths.add(pathKey);
    result.push(item);
    if (result.length >= 80) break;
  }
  return result.sort((a, b) => b.updatedAt - a.updatedAt);
}
function addProjectSpace(input, item) {
  const list = normalizeProjectSpaces(input);
  const candidate = normalizeProjectSpace(item);
  if (!candidate) throw new Error('Projekt nemá platné ID nebo absolutní cestu ke složce.');
  const pathKey = process.platform === 'win32' ? candidate.folderPath.toLocaleLowerCase('en-US') : candidate.folderPath;
  const existing = list.find(x => (process.platform === 'win32' ? x.folderPath.toLocaleLowerCase('en-US') : x.folderPath) === pathKey);
  if (existing) return { list, project: existing, created: false };
  if (list.length >= 80) throw new Error('Můžeš mít nejvýše 80 projektových prostorů.');
  const next = [candidate, ...list];
  return { list: next, project: candidate, created: true };
}
function renameProjectSpace(input, id, name) {
  const list = normalizeProjectSpaces(input);
  const project = list.find(item => item.id === id);
  if (!project) throw new Error('Projekt už neexistuje.');
  project.name = normalizeName(name, project.name);
  project.updatedAt = Date.now();
  return { list, project };
}
function removeProjectSpace(input, id) {
  const list = normalizeProjectSpaces(input);
  const project = list.find(item => item.id === id) || null;
  return { list: list.filter(item => item.id !== id), project };
}
function toPublicProjectSpace(item) {
  const project = normalizeProjectSpace(item);
  if (!project) return null;
  return { id: project.id, name: project.name, folderName: project.folderName, updatedAt: project.updatedAt };
}
module.exports = { normalizeName, normalizeProjectSpace, normalizeProjectSpaces, addProjectSpace, renameProjectSpace, removeProjectSpace, toPublicProjectSpace };
