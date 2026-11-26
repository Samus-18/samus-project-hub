'use strict';
const DEFAULTS={openai:{baseUrl:'https://api.openai.com/v1',model:'gpt-6-luna'},anthropic:{baseUrl:'https://api.anthropic.com/v1',model:'claude-haiku-4-5-20251001'},gemini:{baseUrl:'https://generativelanguage.googleapis.com/v1beta',model:'gemini-3.5-flash-lite'}};
function normalizeProtocol(value){return ['openai','anthropic','gemini'].includes(value)?value:'openai'}
function isValidModelName(value){const model=String(value||'').trim();return /^[a-zA-Z0-9][a-zA-Z0-9._:/-]{0,119}$/.test(model)&&!model.includes('..')&&!model.includes('//')&&!model.endsWith('/')} 
function toPositiveInt(value,fallback,max){const n=Number(value);return Number.isFinite(n)?Math.max(1,Math.min(max,Math.floor(n))):fallback}
function usesOpenAICompletionTokenField(model){return /^gpt-(?:5|6)(?:[.-]|$)/i.test(String(model||''))}
function supportsOpenAITemperature(model){return !usesOpenAICompletionTokenField(model)}
function supportsAnthropicTemperature(model){const id=String(model||'').toLowerCase();return !/^claude-(?:(?:opus|sonnet|fable)-5(?:-|$)|(?:opus|sonnet|fable)-4-(?:7|8)(?:-|$)|mythos(?:-|$))/.test(id)}
function supportsGeminiTemperature(model){return !/^gemini-(?:3|4)(?:[.-]|$)/i.test(String(model||''))}
const OPENAI_REASONING_MODELS=new Set(['gpt-6-astra','gpt-6.1-sol','gpt-6.1-terra','gpt-6-luna','gpt-6-sol']);
const OPENAI_NONE_EFFORT_MODELS=new Set(['gpt-6.1-terra','gpt-6-luna','gpt-6-sol']);
function normalizePerformanceProfile(value){return ['quick','balanced','deep','manual'].includes(value)?value:'balanced'}
function resolveOpenAIReasoningEffort(model,profile='balanced'){
 const id=String(model||'').trim().toLowerCase();if(!OPENAI_REASONING_MODELS.has(id))return null;
 const mode=normalizePerformanceProfile(profile);
 if(mode==='quick')return OPENAI_NONE_EFFORT_MODELS.has(id)?'none':'low';
 if(mode==='deep')return 'high';
 return 'medium';
}
function isOfficialOpenAIEndpoint(baseUrl){try{return new URL(String(baseUrl||DEFAULTS.openai.baseUrl)).hostname.toLowerCase()==='api.openai.com'}catch{return false}}
function resolveGeminiThinkingLevel(model,profile='balanced'){
 const id=String(model||'').trim().toLowerCase();if(!/^gemini-(?:3|4)(?:[.-]|$)/.test(id))return null;
 const mode=normalizePerformanceProfile(profile);
 if(mode==='quick')return id==='gemini-3.5-flash-lite'?'minimal':'low';
 if(mode==='deep')return 'high';
 return 'medium';
}
function normalizeSamplingOptions(options={}){
 const finite=(value,fallback)=>Number.isFinite(Number(value))?Number(value):fallback;
 const result={topP:Math.max(0.05,Math.min(1,finite(options.top_p,0.9))),topK:Math.round(Math.max(1,Math.min(200,finite(options.top_k,40)))),repeatPenalty:Math.max(0.5,Math.min(2,finite(options.repeat_penalty,1.1)))};
 result.stop=Array.isArray(options.stop)?options.stop.map(x=>String(x).slice(0,100)).filter(Boolean).slice(0,8):[];
 const seed=options.seed;if(seed!==undefined&&seed!==null&&seed!==''&&Number.isFinite(Number(seed)))result.seed=Math.round(Math.max(0,Math.min(2147483647,Number(seed))));
 return result;
}
function buildRemoteRequest({protocol='openai',baseUrl,model,messages,options={},apiKey}){
 protocol=normalizeProtocol(protocol);const defaults=DEFAULTS[protocol];const base=String(baseUrl||defaults.baseUrl).replace(/\/+$/,'');const selectedModel=String(model||defaults.model);
 if(!isValidModelName(selectedModel))throw new Error('Neplatný název vzdáleného modelu.');
 const temperature=Math.max(0,Math.min(2,Number(options.temperature??0.7)));const maxTokens=toPositiveInt(options.num_predict,1024,8192);const stream=options.stream!==false;
 const sampling=normalizeSamplingOptions(options);
 const system=messages.filter(m=>m.role==='system').map(m=>m.content).join('\n\n');const conversation=messages.filter(m=>m.role!=='system');
 if(protocol==='openai'){
  const tokenLimit=usesOpenAICompletionTokenField(selectedModel)?{max_completion_tokens:maxTokens}:{max_tokens:maxTokens};
  const official=isOfficialOpenAIEndpoint(base);const effort=official?resolveOpenAIReasoningEffort(selectedModel,options.performance_profile):null;
  const standardSampling=supportsOpenAITemperature(selectedModel)?{top_p:sampling.topP,...(sampling.stop.length?{stop:sampling.stop}:{}),...(sampling.seed!==undefined?{seed:sampling.seed}:{})}:{};
  return {url:`${base}/chat/completions`,headers:{Authorization:`Bearer ${apiKey}`,'Content-Type':'application/json'},body:{model:selectedModel,messages,stream,...(supportsOpenAITemperature(selectedModel)?{temperature}:{}),...standardSampling,...(effort?{reasoning_effort:effort}:{}),...tokenLimit}};
 }
 if(protocol==='anthropic'){
  return {url:`${base}/messages`,headers:{'x-api-key':apiKey,'anthropic-version':'2023-06-01','Content-Type':'application/json'},body:{model:selectedModel,system:system||undefined,messages:conversation.map(m=>({role:m.role==='assistant'?'assistant':'user',content:m.content})),stream,...(supportsAnthropicTemperature(selectedModel)?{temperature}:{}),top_p:sampling.topP,top_k:sampling.topK,...(sampling.stop.length?{stop_sequences:sampling.stop}:{}),max_tokens:maxTokens}};
 }
 const thinkingLevel=resolveGeminiThinkingLevel(selectedModel,options.performance_profile);
 return {url:`${base}/models/${encodeURIComponent(selectedModel)}:${stream?'streamGenerateContent?alt=sse':'generateContent'}`,headers:{'x-goog-api-key':apiKey,'Content-Type':'application/json'},body:{systemInstruction:system?{parts:[{text:system}]}:undefined,contents:conversation.map(m=>({role:m.role==='assistant'?'model':'user',parts:[{text:m.content}]})),generationConfig:{...(supportsGeminiTemperature(selectedModel)?{temperature}:{}),topP:sampling.topP,topK:sampling.topK,...(sampling.stop.length?{stopSequences:sampling.stop}:{}),...(sampling.seed!==undefined?{seed:sampling.seed}:{}),...(thinkingLevel?{thinkingConfig:{thinkingLevel}}:{}),maxOutputTokens:maxTokens}}};
}

function buildRemoteTestRequest({protocol='openai',baseUrl,model,apiKey}){
 protocol=normalizeProtocol(protocol);const defaults=DEFAULTS[protocol];const base=String(baseUrl||defaults.baseUrl).replace(/\/+$/,'');const selectedModel=String(model||defaults.model);
 if(!isValidModelName(selectedModel))throw new Error('Neplatný název vzdáleného modelu.');
 if(protocol==='openai'){
  const tokenLimit=usesOpenAICompletionTokenField(selectedModel)?{max_completion_tokens:8}:{max_tokens:8};
  const effort=isOfficialOpenAIEndpoint(base)?resolveOpenAIReasoningEffort(selectedModel,'quick'):null;
  return {method:'POST',url:`${base}/chat/completions`,headers:{Authorization:`Bearer ${apiKey}`,'Content-Type':'application/json'},body:{model:selectedModel,messages:[{role:'user',content:'Odpověz pouze OK.'}],stream:false,...(supportsOpenAITemperature(selectedModel)?{temperature:0}:{}),...(effort?{reasoning_effort:effort}:{}),...tokenLimit}}
 }
 if(protocol==='anthropic')return {method:'POST',url:`${base}/messages`,headers:{'x-api-key':apiKey,'anthropic-version':'2023-06-01','Content-Type':'application/json'},body:{model:selectedModel,max_tokens:8,stream:false,messages:[{role:'user',content:'Odpověz pouze OK.'}],...(supportsAnthropicTemperature(selectedModel)?{temperature:0}:{})}};
 const thinkingLevel=resolveGeminiThinkingLevel(selectedModel,'quick');
 return {method:'POST',url:`${base}/models/${encodeURIComponent(selectedModel)}:generateContent`,headers:{'x-goog-api-key':apiKey,'Content-Type':'application/json'},body:{contents:[{role:'user',parts:[{text:'Odpověz pouze OK.'}]}],generationConfig:{...(supportsGeminiTemperature(selectedModel)?{temperature:0}:{}),...(thinkingLevel?{thinkingConfig:{thinkingLevel}}:{}),maxOutputTokens:8}}};
}

function extractRemoteTestText(protocol, payload){
 protocol=normalizeProtocol(protocol);if(!payload||typeof payload!=='object')return '';
 if(protocol==='openai'){
  const content=payload.choices?.[0]?.message?.content;
  if(typeof content==='string')return content;
  if(Array.isArray(content))return content.map(part=>typeof part?.text==='string'?part.text:'').join('');
  return '';
 }
 if(protocol==='anthropic')return Array.isArray(payload.content)?payload.content.filter(part=>part?.type==='text'&&typeof part.text==='string').map(part=>part.text).join(''):'';
 return (payload.candidates?.[0]?.content?.parts||[]).map(part=>typeof part?.text==='string'?part.text:'').join('');
}

function createSSEDecoder(onEvent){
 let buffer='';
 function dispatch(frame){
  if(!frame)return;
  let event='message';const data=[];
  for(const line of frame.split(/\r?\n/)){
   if(!line||line.startsWith(':'))continue;
   const colon=line.indexOf(':');const field=colon<0?line:line.slice(0,colon);let value=colon<0?'':line.slice(colon+1);if(value.startsWith(' '))value=value.slice(1);
   if(field==='event')event=value;else if(field==='data')data.push(value);
  }
  if(data.length)onEvent({event,data:data.join('\n')});
 }
 return {
  push(chunk){buffer+=String(chunk||'');const frames=buffer.split(/\r?\n\r?\n/);buffer=frames.pop()||'';for(const frame of frames)dispatch(frame)},
  finish(){if(buffer.trim())dispatch(buffer);buffer=''}
 };
}

function parseRemoteEvent(protocol,raw){protocol=normalizeProtocol(protocol);if(!raw||!raw.trim())return '';let obj;try{obj=JSON.parse(raw)}catch{return ''}
 if(protocol==='openai'){if(obj.error)throw new Error(obj.error.message||'Vzdálené API vrátilo chybu.');const text=obj.choices?.[0]?.delta?.content;return typeof text==='string'?text:''}
 if(protocol==='anthropic'){if(obj.type==='error')throw new Error(obj.error?.message||'Anthropic API stream selhal.');return obj.type==='content_block_delta'&&obj.delta?.type==='text_delta'&&typeof obj.delta.text==='string'?obj.delta.text:''}
 if(obj.error)throw new Error(obj.error.message||'Gemini API stream selhal.');return (obj.candidates?.[0]?.content?.parts||[]).map(p=>typeof p.text==='string'?p.text:'').join('');
}
module.exports={DEFAULTS,normalizeProtocol,isValidModelName,usesOpenAICompletionTokenField,supportsOpenAITemperature,supportsAnthropicTemperature,supportsGeminiTemperature,normalizePerformanceProfile,resolveOpenAIReasoningEffort,resolveGeminiThinkingLevel,isOfficialOpenAIEndpoint,normalizeSamplingOptions,buildRemoteRequest,buildRemoteTestRequest,extractRemoteTestText,createSSEDecoder,parseRemoteEvent};
