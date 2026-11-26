const test = require('node:test');
const assert = require('node:assert/strict');
const { validateAgents, makeAgentPrompt, runAgentWorkflow } = require('../src/agent-workflow');

test('agent roster validates bounded size, roles, model and unique IDs', () => {
  assert.throws(() => validateAgents([]), /1 až 4/);
  assert.throws(() => validateAgents([{id:'x',role:'hacker',model:'m'}]), /Neznámá role/);
  assert.throws(() => validateAgents([{id:'x',model:'m'},{id:'x',model:'n'}]), /Duplicitní/);
  assert.throws(() => validateAgents([{id:'x'}]), /nemá vybraný model/);
  assert.equal(validateAgents([{id:'a',role:'planner',model:'qwen'}])[0].model,'qwen');
});

test('prompt keeps task bounded and gives role-specific instructions', () => {
  assert.throws(() => makeAgentPrompt({task:'  ',role:'planner'}), /prázdný/);
  assert.match(makeAgentPrompt({task:'Oprav chybu',role:'reviewer'}), /Nezávisle zkontroluj/);
  assert.ok(makeAgentPrompt({task:'x'.repeat(20000),role:'implementer'}).length < 13000);
});

test('workflow executes agents in order, passes prior results and emits visible events', async () => {
  const calls=[]; const events=[];
  const out=await runAgentWorkflow({task:'Zkontroluj projekt',agents:[
    {id:'plan',role:'planner',model:'local-a'}, {id:'review',role:'reviewer',model:'local-b'}
  ],callModel:async request=>{calls.push(request);return `výsledek-${calls.length}`},onEvent:e=>events.push(e)});
  assert.equal(out.completed,2); assert.equal(out.failed,0); assert.equal(calls.length,2);
  assert.match(calls[1].messages[0].content,/výsledek-1/);
  assert.equal(events.filter(e=>e.type==='agent-complete').length,2);
});

test('workflow reports a failed agent without hiding failure', async () => {
  const out=await runAgentWorkflow({task:'Test',agents:[{id:'a',role:'planner',model:'m'}],callModel:async()=>{throw new Error('offline')}});
  assert.equal(out.failed,1); assert.equal(out.results[0].status,'failed'); assert.match(out.results[0].error,/offline/);
});

test('workflow honors cancellation before starting next agent', async () => {
  const controller=new AbortController(); let calls=0;
  await assert.rejects(runAgentWorkflow({task:'Test',signal:controller.signal,agents:[{id:'a',role:'planner',model:'m'},{id:'b',role:'reviewer',model:'m'}],callModel:async()=>{calls++;controller.abort();return 'done'}}),/zrušen/);
  assert.equal(calls,1);
});
