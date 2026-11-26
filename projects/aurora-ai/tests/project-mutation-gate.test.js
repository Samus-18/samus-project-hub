'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Gate = require('../src/project-mutation-gate');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

function deferred() {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return { promise, resolve };
}

test('project mutations are serialized and preserve request order', async () => {
  const gate = Gate.createSerialMutationGate();
  const trace = [];
  const hold = deferred();
  const started = deferred();
  const first = gate.run('proposal-1', async () => {
    trace.push('first:start');
    started.resolve();
    await hold.promise;
    trace.push('first:end');
    return 1;
  });
  await started.promise;
  const second = gate.run('proposal-2', async () => {
    trace.push('second:start');
    return 2;
  });
  await Promise.resolve();
  assert.deepEqual(trace, ['first:start']);
  hold.resolve();
  assert.deepEqual(await Promise.all([first, second]), [1, 2]);
  assert.deepEqual(trace, ['first:start', 'first:end', 'second:start']);
  assert.equal(gate.isLocked('proposal-1'), false);
});

test('the same proposal cannot be approved twice while its first operation is active', async () => {
  const gate = Gate.createSerialMutationGate();
  const hold = deferred();
  const started = deferred();
  const first = gate.run('proposal-dup', async () => {
    started.resolve();
    await hold.promise;
    return 'written';
  });
  await started.promise;
  assert.equal(gate.isLocked('proposal-dup'), true);
  await assert.rejects(gate.run('proposal-dup', async () => 'must not run'), /už zpracovává/i);
  hold.resolve();
  assert.equal(await first, 'written');
  assert.equal(gate.isLocked('proposal-dup'), false);
});

test('a failed mutation releases the lock and does not block subsequent work', async () => {
  const gate = Gate.createSerialMutationGate();
  await assert.rejects(gate.run('failure', async () => { throw new Error('simulated write failure'); }), /simulated write failure/);
  assert.equal(gate.isLocked('failure'), false);
  assert.equal(await gate.run('recovery', async () => 'recovered'), 'recovered');
  assert.equal(await gate.run(null, async () => 'rollback complete'), 'rollback complete');
});

test('main process gates project approvals and cancellation reports an in-flight proposal honestly', () => {
  const main = read('src/main.js');
  const app = read('src/app.js');
  assert.match(main, /ProjectMutationGate\.createSerialMutationGate\(\)/);
  assert.match(main, /project:approve-edit',[\s\S]{0,100}withProjectMutationLock/);
  assert.match(main, /project:approve-batch',[\s\S]{0,100}withProjectMutationLock/);
  assert.match(main, /project:rollback',[\s\S]{0,100}withProjectMutationLock/);
  assert.match(main, /project:choose',[\s\S]{0,100}withProjectMutationLock/);
  assert.match(main, /payload\?\.id==='projectWrites'\?withProjectMutationLock/);
  assert.match(main, /projectMutationGate\.isLocked\(key\)/);
  assert.match(app, /outcome\?\.busy[\s\S]{0,100}návrh nelze zrušit/i);
  assert.match(app, /outcome\?\.busy[\s\S]{0,100}zrušení není možné/i);
});
