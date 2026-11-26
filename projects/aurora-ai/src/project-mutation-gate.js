'use strict';

function createSerialMutationGate() {
  let tail = Promise.resolve();
  const activeIds = new Set();

  async function run(id, work) {
    if (typeof work !== 'function') throw new TypeError('Mutation work must be a function.');
    const key = typeof id === 'string' && id.trim() ? id : null;
    if (key && activeIds.has(key)) {
      throw new Error('Tento návrh se už zpracovává. Počkej na výsledek první operace.');
    }
    if (key) activeIds.add(key);

    const previous = tail;
    let release;
    tail = new Promise(resolve => { release = resolve; });
    try {
      await previous;
      return await work();
    } finally {
      release();
      if (key) activeIds.delete(key);
    }
  }

  function isLocked(id) {
    return typeof id === 'string' && activeIds.has(id);
  }

  return Object.freeze({ run, isLocked });
}

module.exports = Object.freeze({ createSerialMutationGate });
