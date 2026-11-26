const test=require('node:test');const assert=require('node:assert/strict');const P=require('../src/remote-providers');
const messages=[{role:'system',content:'Buď stručný.'},{role:'user',content:'Ahoj'},{role:'assistant',content:'Zdravím'},{role:'user',content:'Jak se máš?'}];
test('OpenAI adapter preserves chat messages and compatible auth',()=>{const r=P.buildRemoteRequest({protocol:'openai',model:'gpt-4o-mini',messages,apiKey:'secret',options:{temperature:0.2,num_predict:120}});assert.equal(r.url,'https://api.openai.com/v1/chat/completions');assert.equal(r.headers.Authorization,'Bearer secret');assert.equal(r.body.messages.length,4);assert.equal(r.body.max_tokens,120)});
test('Anthropic adapter moves system prompt and maps roles',()=>{const r=P.buildRemoteRequest({protocol:'anthropic',model:'claude-haiku-4-5-20251001',messages,apiKey:'secret'});assert.equal(r.url,'https://api.anthropic.com/v1/messages');assert.equal(r.headers['anthropic-version'],'2023-06-01');assert.equal(r.body.system,'Buď stručný.');assert.deepEqual(r.body.messages.map(m=>m.role),['user','assistant','user'])});
test('Gemini adapter maps assistant to model role and keeps key out of URL',()=>{const r=P.buildRemoteRequest({protocol:'gemini',model:'gemini-3.5-flash-lite',messages,apiKey:'secret'});assert.match(r.url,/models\/gemini-3\.5-flash-lite:streamGenerateContent\?alt=sse$/);assert.equal(r.headers['x-goog-api-key'],'secret');assert.equal(r.url.includes('secret'),false);assert.deepEqual(r.body.contents.map(m=>m.role),['user','model','user'])});
test('provider stream parsers extract text for all protocols',()=>{assert.equal(P.parseRemoteEvent('openai',JSON.stringify({choices:[{delta:{content:'A'}}]})),'A');assert.equal(P.parseRemoteEvent('anthropic',JSON.stringify({type:'content_block_delta',delta:{type:'text_delta',text:'B'}})),'B');assert.equal(P.parseRemoteEvent('gemini',JSON.stringify({candidates:[{content:{parts:[{text:'C'},{text:'D'}]}}]})),'CD')});
test('provider stream errors surface readable messages',()=>{assert.throws(()=>P.parseRemoteEvent('anthropic',JSON.stringify({type:'error',error:{message:'bad key'}})),/bad key/);assert.throws(()=>P.parseRemoteEvent('gemini',JSON.stringify({error:{message:'quota'}})),/quota/)});
test('unknown protocol safely falls back to OpenAI-compatible mode',()=>{assert.equal(P.normalizeProtocol('surprise'),'openai')});
test('invalid model names are rejected',()=>{assert.throws(()=>P.buildRemoteRequest({protocol:'gemini',model:'../leak',messages,apiKey:'secret'}),/Neplatný název/)});

test('connection tests use protocol-correct non-streaming endpoints',()=>{
 const gemini=P.buildRemoteTestRequest({protocol:'gemini',apiKey:'secret',model:'gemini-3.5-flash-lite'});
 assert.equal(gemini.method,'POST');assert.match(gemini.url,/models\/gemini-3\.5-flash-lite:generateContent$/);assert.doesNotMatch(gemini.url,/streamGenerateContent|alt=sse/);assert.equal(gemini.headers['x-goog-api-key'],'secret');
 const anthropic=P.buildRemoteTestRequest({protocol:'anthropic',apiKey:'secret',model:'claude-haiku-4-5-20251001'});
 assert.equal(anthropic.url,'https://api.anthropic.com/v1/messages');assert.equal(anthropic.body.stream,false);assert.equal(anthropic.headers['anthropic-version'],'2023-06-01');
 const openai=P.buildRemoteTestRequest({protocol:'openai',apiKey:'secret'});assert.equal(openai.method,'POST');assert.equal(openai.url,'https://api.openai.com/v1/chat/completions');assert.equal(openai.headers.Authorization,'Bearer secret');assert.equal(openai.body.stream,false);assert.equal(openai.body.model,'gpt-6-luna');assert.equal(openai.body.max_completion_tokens,8);assert.equal(openai.body.max_tokens,undefined);
});
test('connection-test builder rejects unsafe model identifiers',()=>{
 assert.throws(()=>P.buildRemoteTestRequest({protocol:'gemini',model:'../secret'}),/Neplatný název/);
});

test('SSE decoder handles fragmented chunks, CRLF, multi-line data and final frame',()=>{
 const events=[];const parser=P.createSSEDecoder(e=>events.push(e));
 parser.push('event: message\r\ndata: {\"choices\":\r\n');parser.push('data: [{\"delta\":{\"content\":\"OK\"}}]}\r\n\r\ndata: [DONE]');parser.finish();
 assert.deepEqual(events,[{event:'message',data:'{\"choices\":\n[{\"delta\":{\"content\":\"OK\"}}]}'},{event:'message',data:'[DONE]'}]);
});


test('remote connection-test responses must contain generated text for all three protocols',()=>{
 assert.equal(P.extractRemoteTestText('openai',{choices:[{message:{content:'OK'}}]}),'OK');
 assert.equal(P.extractRemoteTestText('openai',{choices:[{message:{content:[{type:'text',text:'O'}, {type:'text',text:'K'}]}}]}),'OK');
 assert.equal(P.extractRemoteTestText('anthropic',{content:[{type:'text',text:'OK'}]}),'OK');
 assert.equal(P.extractRemoteTestText('gemini',{candidates:[{content:{parts:[{text:'O'}, {text:'K'}]}}]}),'OK');
 assert.equal(P.extractRemoteTestText('gemini',{candidates:[]}), '');
});


test('OpenAI-compatible model IDs may contain provider namespaces but not path traversal',()=>{
 const r=P.buildRemoteRequest({protocol:'openai',baseUrl:'https://openrouter.ai/api/v1',model:'openrouter/deepseek-r1',messages,apiKey:'secret'});
 assert.equal(r.body.model,'openrouter/deepseek-r1');
 assert.throws(()=>P.buildRemoteRequest({protocol:'openai',model:'../secret',messages,apiKey:'secret'}),/Neplatný název/);
});


test('current provider defaults avoid retired models and use protocol-compatible generation fields',()=>{
 assert.equal(P.DEFAULTS.openai.model,'gpt-6-luna');
 assert.equal(P.DEFAULTS.anthropic.model,'claude-haiku-4-5-20251001');
 assert.equal(P.DEFAULTS.gemini.model,'gemini-3.5-flash-lite');
 const openai=P.buildRemoteRequest({protocol:'openai',model:'gpt-6-luna',messages,apiKey:'secret',options:{num_predict:256,temperature:0.2}});
 assert.equal(openai.body.max_completion_tokens,256);assert.equal(openai.body.max_tokens,undefined);assert.equal(openai.body.temperature,undefined);
 const claude=P.buildRemoteRequest({protocol:'anthropic',model:'claude-opus-4-8',messages,apiKey:'secret',options:{temperature:0.2}});
 assert.equal(claude.body.temperature,undefined);
 const claudeTest=P.buildRemoteTestRequest({protocol:'anthropic',model:'claude-opus-4-8',apiKey:'secret'});
 assert.equal(claudeTest.body.temperature,undefined);
 const gemini=P.buildRemoteRequest({protocol:'gemini',model:'gemini-3.5-flash-lite',messages,apiKey:'secret',options:{temperature:0.2}});
 assert.equal(gemini.body.generationConfig.temperature,undefined);
});

test('OpenAI official GPT-6 reasoning effort follows the selected performance profile',()=>{
 const quick=P.buildRemoteRequest({protocol:'openai',model:'gpt-6-luna',messages,apiKey:'secret',options:{performance_profile:'quick',num_predict:384,temperature:0.7}});
 assert.equal(quick.body.reasoning_effort,'none');assert.equal(quick.body.max_completion_tokens,384);assert.equal(quick.body.temperature,undefined);
 const deep=P.buildRemoteRequest({protocol:'openai',model:'gpt-6-luna',messages,apiKey:'secret',options:{performance_profile:'deep',num_predict:2048}});
 assert.equal(deep.body.reasoning_effort,'high');assert.equal(deep.body.max_completion_tokens,2048);
 const noNone=P.buildRemoteRequest({protocol:'openai',model:'gpt-6-astra',messages,apiKey:'secret',options:{performance_profile:'quick'}});
 assert.equal(noNone.body.reasoning_effort,'low');
 const proxy=P.buildRemoteRequest({protocol:'openai',baseUrl:'https://openrouter.ai/api/v1',model:'openrouter/gpt-6-luna',messages,apiKey:'secret',options:{performance_profile:'quick'}});
 assert.equal(proxy.body.reasoning_effort,undefined);assert.equal(proxy.body.max_tokens,1024);
});

test('Gemini 3.5 Flash-Lite maps fast, balanced and deep profiles to thinking levels',()=>{
 const build=profile=>P.buildRemoteRequest({protocol:'gemini',model:'gemini-3.5-flash-lite',messages,apiKey:'secret',options:{performance_profile:profile}}).body.generationConfig;
 assert.equal(build('quick').thinkingConfig.thinkingLevel,'minimal');
 assert.equal(build('balanced').thinkingConfig.thinkingLevel,'medium');
 assert.equal(build('deep').thinkingConfig.thinkingLevel,'high');
 const older=P.buildRemoteRequest({protocol:'gemini',model:'gemini-2.5-flash',messages,apiKey:'secret',options:{performance_profile:'quick'}});
 assert.equal(older.body.generationConfig.thinkingConfig,undefined);assert.equal(older.body.generationConfig.temperature,0.7);
});

test('Anthropic 4.7+ omits deprecated temperature for all current families',()=>{
 for(const model of ['claude-opus-4-7','claude-opus-4-8','claude-sonnet-4-7','claude-fable-5']){
  assert.equal(P.supportsAnthropicTemperature(model),false,model);
  const request=P.buildRemoteRequest({protocol:'anthropic',model,messages,apiKey:'secret',options:{temperature:0.2}});
  assert.equal(request.body.temperature,undefined,model);
 }
 assert.equal(P.supportsAnthropicTemperature('claude-haiku-4-5-20251001'),true);
});

test('remote generation forwards only protocol-appropriate sampling options', () => {
  const base = {messages:[{role:'system',content:'Be clear.'},{role:'user',content:'Explain briefly.'}],options:{temperature:0.4,top_p:0.82,top_k:33,repeat_penalty:1.25,seed:21,stop:['END','###'],num_predict:512}};
  const openai = P.buildRemoteRequest({...base,protocol:'openai',baseUrl:'https://api.openai.com/v1',model:'gpt-4.1',apiKey:'test-key'});
  assert.equal(openai.body.top_p,0.82);
  assert.equal(openai.body.seed,21);
  assert.deepEqual(openai.body.stop,['END','###']);
  assert.equal('top_k' in openai.body, false);
  assert.equal('repeat_penalty' in openai.body, false);
  const anthropic = P.buildRemoteRequest({...base,protocol:'anthropic',baseUrl:'https://api.anthropic.com/v1',model:'claude-haiku-4-5-20251001',apiKey:'test-key'});
  assert.equal(anthropic.body.top_p,0.82);
  assert.equal(anthropic.body.top_k,33);
  assert.deepEqual(anthropic.body.stop_sequences,['END','###']);
  const gemini = P.buildRemoteRequest({...base,protocol:'gemini',baseUrl:'https://generativelanguage.googleapis.com/v1beta',model:'gemini-2.5-flash',apiKey:'test-key'});
  assert.equal(gemini.body.generationConfig.topP,0.82);
  assert.equal(gemini.body.generationConfig.topK,33);
  assert.equal(gemini.body.generationConfig.seed,21);
  assert.deepEqual(gemini.body.generationConfig.stopSequences,['END','###']);
});

test('remote provider can request a non-streaming response without using an SSE endpoint', () => {
  const request = P.buildRemoteRequest({protocol:'gemini',baseUrl:'https://generativelanguage.googleapis.com/v1beta',model:'gemini-2.5-flash',messages:[{role:'user',content:'Hi'}],options:{stream:false},apiKey:'test-key'});
  assert.match(request.url, /:generateContent$/);
  assert.equal('stream' in request.body, false);
});
