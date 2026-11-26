const test = require('node:test');
const assert = require('node:assert/strict');
const Proposals = require('../src/agent-change-proposals');

test('extracts a bounded Aurora changeset block', () => {
  const text = 'Návrh:\n```aurora-changeset\n{"summary":"Oprava","edits":[{"path":"src/main.js","content":"ok"}],"tests":["npm test"]}\n```';
  const result = Proposals.extractChangeSet(text);
  assert.equal(result.found, true);
  assert.equal(result.error, '');
  assert.equal(result.proposal.summary, 'Oprava');
  assert.deepEqual(result.proposal.edits, [{ path: 'src/main.js', content: 'ok' }]);
});

test('ignores ordinary JSON and requires the explicit Aurora changeset fence', () => {
  assert.deepEqual(Proposals.extractChangeSet('{"edits":[]}'), { found: false, proposal: null, error: '' });
});

test('rejects traversal, absolute paths, unsupported files and duplicates', () => {
  for (const path of ['../secret.txt', '/tmp/secret.txt', 'C:/secret.txt', 'src/../secret.js', 'image.png']) {
    assert.throws(() => Proposals.validateChangeSet({ edits: [{ path, content: 'x' }] }));
  }
  assert.throws(() => Proposals.validateChangeSet({ edits: [{ path: 'src/a.js', content: 'a' }, { path: 'src/a.js', content: 'b' }] }), /duplicitní/);
});

test('bounds proposal count and content size', () => {
  assert.throws(() => Proposals.validateChangeSet({ edits: Array.from({ length: 13 }, (_, i) => ({ path: `src/${i}.js`, content: '' })) }), /1 až 12/);
  assert.throws(() => Proposals.validateChangeSet({ edits: [{ path: 'src/a.js', content: 'x'.repeat(500001) }] }), /500 kB/);
});

test('reports invalid structured JSON without creating a proposal', () => {
  const result = Proposals.extractChangeSet('```aurora-changeset\n{not json}\n```');
  assert.equal(result.found, true);
  assert.equal(result.proposal, null);
  assert.match(result.error, /platný JSON/);
});


test('allows real UTF-8 tabular text deliverables but not fake Office binaries', () => {
  assert.equal(Proposals.normalizeRelativePath('exports/report.csv'),'exports/report.csv');
  assert.equal(Proposals.normalizeRelativePath('exports/report.tsv'),'exports/report.tsv');
  for (const file of ['report.xlsx','report.docx','report.pptx','report.pdf']) {
    assert.throws(()=>Proposals.normalizeRelativePath(file),/není povolen/);
  }
});
