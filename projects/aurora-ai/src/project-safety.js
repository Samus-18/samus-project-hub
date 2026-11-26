'use strict';
const path = require('node:path');
const crypto = require('node:crypto');

function isPathInside(rootPath, candidatePath, { allowRoot = true } = {}) {
  if (typeof rootPath !== 'string' || !rootPath.trim() || typeof candidatePath !== 'string' || !candidatePath.trim()) return false;
  const root = path.resolve(rootPath);
  const candidate = path.resolve(candidatePath);
  if (candidate === root) return allowRoot;
  return candidate.startsWith(root + path.sep);
}

function resolveProjectTarget(rootPath, relativePath) {
  if (typeof rootPath !== 'string' || !rootPath.trim() || typeof relativePath !== 'string' || !relativePath.trim()) {
    throw new Error('Nejdřív vyber schválenou projektovou složku a soubor.');
  }
  const root = path.resolve(rootPath);
  const target = path.resolve(root, relativePath);
  if (!isPathInside(root, target, { allowRoot: false })) throw new Error('Cesta je mimo schválenou složku.');
  return { root, target, relative: path.relative(root, target) };
}

function assertSameApprovedRoot(proposalRoot, approvedRoot) {
  if (typeof proposalRoot !== 'string' || typeof approvedRoot !== 'string' || !proposalRoot || !approvedRoot || path.resolve(proposalRoot) !== path.resolve(approvedRoot)) {
    throw new Error('Schválená složka projektu se změnila. Vyber správný projekt a připrav návrh znovu.');
  }
  return path.resolve(approvedRoot);
}

async function assertSafeMutationPath(fsPromises, rootPath, targetPath) {
  const root = await fsPromises.realpath(rootPath);
  if (path.resolve(root) !== path.resolve(rootPath)) throw new Error('Kořen projektu je dostupný přes symbolický odkaz; vyber skutečnou složku projektu.');
  const target = path.resolve(targetPath);
  if (!isPathInside(root, target, { allowRoot: false })) throw new Error('Cílová cesta je mimo schválenou složku.');
  const parent = path.dirname(target);
  if (!isPathInside(root, parent)) throw new Error('Cílová složka je mimo schválený projekt.');
  const parts = path.relative(root, parent).split(path.sep).filter(Boolean);
  let current = root;
  for (const part of parts) {
    current = path.join(current, part);
    let stat;
    try { stat = await fsPromises.lstat(current); }
    catch (error) { if (error.code === 'ENOENT') break; throw error; }
    if (stat.isSymbolicLink() || !stat.isDirectory()) throw new Error('Zápis přes symbolický odkaz nebo nedirectory cestu není povolen.');
    const real = await fsPromises.realpath(current);
    if (path.resolve(real) !== path.resolve(current) || !isPathInside(root, real)) throw new Error('Cílová složka míří mimo schválený projekt.');
  }
  try {
    const stat = await fsPromises.lstat(target);
    if (stat.isSymbolicLink() || !stat.isFile()) throw new Error('Cílový soubor musí být běžný soubor bez symbolického odkazu.');
    const real = await fsPromises.realpath(target);
    if (path.resolve(real) !== path.resolve(target) || !isPathInside(root, real, { allowRoot: false })) throw new Error('Cílový soubor míří mimo schválený projekt.');
    return { exists: true, root, target };
  } catch (error) {
    if (error.code === 'ENOENT') return { exists: false, root, target };
    throw error;
  }
}

function makeBackupPath(targetPath, date = new Date(), nonce = crypto.randomInt(1, 2147483647)) {
  if (typeof targetPath !== 'string' || !targetPath.trim()) throw new Error('Neplatná cesta pro zálohu.');
  const serial = Number(nonce);
  if (!Number.isSafeInteger(serial) || serial < 1 || serial > 2147483647) throw new Error('Neplatný identifikátor zálohy.');
  const stamp = date.toISOString().replace(/[:.]/g, '-');
  return `${targetPath}.aurora-backup-${stamp}-${serial}`;
}

function originalFromBackup(rootPath, relativeBackup, allowedExtensions) {
  const { root, target, relative } = resolveProjectTarget(rootPath, relativeBackup);
  const basename = path.basename(relative);
  if (!/^[^/\\]+\.aurora-backup-[0-9TZ-]+$/.test(basename)) throw new Error('Povoleny jsou pouze platné zálohy Aurora.');
  const originalRelative = relative.replace(/\.aurora-backup-[0-9TZ-]+$/, '');
  if (!originalRelative || originalRelative === basename || (allowedExtensions && !allowedExtensions.has(path.extname(originalRelative).toLowerCase()))) {
    throw new Error('Záloha neodpovídá povolenému typu původního souboru.');
  }
  const originalTarget = path.resolve(root, originalRelative);
  if (!isPathInside(root, originalTarget, { allowRoot: false })) throw new Error('Neplatná cesta původního souboru.');
  return { root, backupTarget: target, backupRelative: relative, originalRelative: originalRelative.split(path.sep).join(path.posix.sep), originalTarget };
}

async function resolveContainedRealPath(fsPromises, rootPath, candidatePath, { allowRoot = true, rejectSymlinkAlias = false } = {}) {
  const root = await fsPromises.realpath(rootPath);
  const realTarget = await fsPromises.realpath(candidatePath);
  if (!isPathInside(root, realTarget, { allowRoot })) throw new Error('Cesta míří mimo schválenou složku.');
  if (rejectSymlinkAlias && path.resolve(candidatePath) !== path.resolve(realTarget)) throw new Error('Zápis přes symbolické odkazy není povolen.');
  return { root, target: realTarget };
}

module.exports = Object.freeze({ isPathInside, resolveProjectTarget, assertSameApprovedRoot, makeBackupPath, originalFromBackup, resolveContainedRealPath, assertSafeMutationPath });
