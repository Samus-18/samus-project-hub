'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const root = path.resolve(__dirname, '..');
const roots = [path.join(root, 'src'), path.join(root, 'scripts')];
const files = [];
function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && entry.name.endsWith('.js')) files.push(full);
  }
}
for (const dir of roots) if (fs.existsSync(dir)) walk(dir);
files.sort();
const failures = [];
for (const file of files) {
  const result = spawnSync(process.execPath, ['--check', file], { cwd: root, encoding: 'utf8', windowsHide: true });
  if (result.status !== 0) failures.push({ file: path.relative(root, file), output: (result.stderr || result.stdout || `exit ${result.status}`).trim() });
}
console.log(`JavaScript syntax: ${files.length - failures.length}/${files.length} files valid`);
for (const failure of failures) console.error(`FAIL ${failure.file}\n${failure.output}`);
if (failures.length) process.exitCode = 1;
