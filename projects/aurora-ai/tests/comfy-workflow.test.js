'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const Comfy = require('../src/comfy-workflow.js');

function workflowWithNegativeNodeFirst() {
  return {
    '7': { class_type: 'CLIPTextEncode', inputs: { text: 'saved negative', clip: ['1', 0] } },
    '6': { class_type: 'CLIPTextEncode', inputs: { text: 'saved positive', clip: ['1', 0] } },
    '3': { class_type: 'KSampler', inputs: { positive: ['6', 0], negative: ['7', 0], seed: 1 } },
    '1': { class_type: 'CLIPTextEncode', inputs: { text: 'unused conditioning', clip: ['2', 0] } }
  };
}

test('positive and negative prompts follow sampler graph edges, not JSON node order', () => {
  const original = workflowWithNegativeNodeFirst();
  const result = Comfy.applyPrompts(original, 'a blue dragon', 'blurry, low quality');
  assert.equal(result.positiveId, '6');
  assert.equal(result.negativeId, '7');
  assert.equal(result.workflow['6'].inputs.text, 'a blue dragon');
  assert.equal(result.workflow['7'].inputs.text, 'blurry, low quality');
  assert.equal(result.workflow['1'].inputs.text, 'unused conditioning');
  assert.equal(original['6'].inputs.text, 'saved positive', 'saved workflow must not be mutated');
  assert.equal(original['7'].inputs.text, 'saved negative', 'saved workflow must not be mutated');
});

test('empty negative prompt retains the saved negative prompt', () => {
  const result = Comfy.applyPrompts(workflowWithNegativeNodeFirst(), 'fresh prompt', '  ');
  assert.equal(result.workflow['6'].inputs.text, 'fresh prompt');
  assert.equal(result.workflow['7'].inputs.text, 'saved negative');
  assert.equal(result.hasNegativeTarget, true);
});

test('single text-node workflow applies only the positive prompt without fabricating a negative node', () => {
  const result = Comfy.applyPrompts({ '5': { class_type: 'CLIPTextEncode', inputs: { text: 'saved', clip: ['1', 0] } } }, 'one prompt', 'avoid noise');
  assert.equal(result.workflow['5'].inputs.text, 'one prompt');
  assert.equal(result.hasNegativeTarget, false);
  assert.equal(result.negativeId, null);
});

test('malformed workflow objects and missing text encoders are rejected', () => {
  assert.throws(() => Comfy.validate(null), /platný ComfyUI API workflow/);
  assert.throws(() => Comfy.validate([]), /platný ComfyUI API workflow/);
  assert.throws(() => Comfy.validate({ '1': { class_type: 'KSampler', inputs: {} } }), /CLIPTextEncode/);
});

test('prompt length is bounded and input text is normalized', () => {
  const result = Comfy.applyPrompts(workflowWithNegativeNodeFirst(), '  ' + 'x'.repeat(9000) + '  ', 'n'.repeat(3000));
  assert.equal(result.workflow['6'].inputs.text.length, 5000);
  assert.equal(result.workflow['7'].inputs.text.length, 2000);
  assert.equal(result.workflow['6'].inputs.text.trim(), result.workflow['6'].inputs.text);
});


test('positive and negative prompts are found through intermediate conditioning nodes', () => {
  const graph = {
    '21': { class_type: 'CLIPTextEncode', inputs: { text: 'unused text node', clip: ['1', 0] } },
    '7': { class_type: 'CLIPTextEncode', inputs: { text: 'saved negative', clip: ['1', 0] } },
    '6': { class_type: 'CLIPTextEncode', inputs: { text: 'saved positive', clip: ['1', 0] } },
    '10': { class_type: 'ConditioningCombine', inputs: { conditioning_1: ['6', 0], conditioning_2: ['21', 0] } },
    '11': { class_type: 'ConditioningCombine', inputs: { conditioning_1: ['7', 0] } },
    '3': { class_type: 'KSampler', inputs: { positive: ['10', 0], negative: ['11', 0] } }
  };
  const result = Comfy.applyPrompts(graph, 'new positive', 'new negative');
  assert.equal(result.positiveId, '6');
  assert.equal(result.negativeId, '7');
  assert.equal(result.workflow['6'].inputs.text, 'new positive');
  assert.equal(result.workflow['7'].inputs.text, 'new negative');
  assert.equal(result.workflow['21'].inputs.text, 'unused text node');
});

test('conditioning graph traversal safely stops on cycles and unknown node links', () => {
  const graph = {
    '1': { class_type: 'ConditioningCombine', inputs: { conditioning_1: ['2', 0], conditioning_2: ['999', 0] } },
    '2': { class_type: 'ConditioningCombine', inputs: { conditioning_1: ['1', 0] } },
    '9': { class_type: 'CLIPTextEncode', inputs: { text: 'valid', clip: ['1', 0] } }
  };
  assert.equal(Comfy.findTextEncodeAncestor(graph, '1'), null);
  assert.equal(Comfy.findTextEncodeAncestor(graph, '9'), '9');
});


test('SDXL encoder receives prompt in both text fields and is routed by sampler edge', () => {
  const graph = {
    '8': { class_type: 'CLIPTextEncodeSDXL', inputs: { text_g: 'old negative global', text_l: 'old negative local', clip: ['1', 0], width: 1024, height: 1024 } },
    '7': { class_type: 'CLIPTextEncodeSDXL', inputs: { text_g: 'old positive global', text_l: 'old positive local', clip: ['1', 0], width: 1024, height: 1024 } },
    '3': { class_type: 'KSampler', inputs: { positive: ['7', 0], negative: ['8', 0] } }
  };
  const result = Comfy.applyPrompts(graph, 'a glass castle', 'text, watermark');
  assert.equal(result.positiveId, '7');
  assert.equal(result.negativeId, '8');
  assert.equal(result.workflow['7'].inputs.text_g, 'a glass castle');
  assert.equal(result.workflow['7'].inputs.text_l, 'a glass castle');
  assert.equal(result.workflow['8'].inputs.text_g, 'text, watermark');
  assert.equal(result.workflow['8'].inputs.text_l, 'text, watermark');
  assert.equal(graph['7'].inputs.text_g, 'old positive global');
});

test('Flux encoder fields are both updated without assuming a generic text property', () => {
  const graph = {
    '6': { class_type: 'CLIPTextEncodeFlux', inputs: { clip_l: 'old positive short', t5xxl: 'old positive detailed', clip: ['1', 0], guidance: 3.5 } },
    '7': { class_type: 'CLIPTextEncodeFlux', inputs: { clip_l: 'old negative short', t5xxl: 'old negative detailed', clip: ['1', 0], guidance: 3.5 } },
    '3': { class_type: 'KSampler', inputs: { positive: ['6', 0], negative: ['7', 0] } }
  };
  const result = Comfy.applyPrompts(graph, 'moonlit abandoned station', 'blurry');
  assert.equal(result.workflow['6'].inputs.clip_l, 'moonlit abandoned station');
  assert.equal(result.workflow['6'].inputs.t5xxl, 'moonlit abandoned station');
  assert.equal(result.workflow['7'].inputs.clip_l, 'blurry');
  assert.equal(result.workflow['7'].inputs.t5xxl, 'blurry');
});

test('SD3 encoder fields are bounded and treated as a supported text node', () => {
  const result = Comfy.applyPrompts({
    '4': { class_type: 'CLIPTextEncodeSD3', inputs: { clip_g: 'old g', clip_l: 'old l', t5xxl: 'old t5', empty_padding: 'none', clip: ['1', 0] } }
  }, 'x'.repeat(6000));
  assert.equal(result.workflow['4'].inputs.clip_g.length, 5000);
  assert.equal(result.workflow['4'].inputs.clip_l.length, 5000);
  assert.equal(result.workflow['4'].inputs.t5xxl.length, 5000);
});


test('CFGGuider-style graphs resolve prompts through semantic positive/negative ports', () => {
  const graph = {
    '90': { class_type: 'CLIPTextEncode', inputs: { text: 'unused node first', clip: ['1', 0] } },
    '4': { class_type: 'CLIPTextEncode', inputs: { text: 'real positive', clip: ['1', 0] } },
    '3': { class_type: 'CLIPTextEncode', inputs: { text: 'real negative', clip: ['1', 0] } },
    '10': { class_type: 'CFGGuider', inputs: { positive: ['4', 0], negative: ['3', 0], model: ['2', 0], cfg: 3.5 } },
    '11': { class_type: 'SamplerCustom', inputs: { guider: ['10', 0] } }
  };
  const result = Comfy.applyPrompts(graph, 'new positive', 'new negative');
  assert.equal(result.positiveId, '4');
  assert.equal(result.negativeId, '3');
  assert.equal(result.workflow['90'].inputs.text, 'unused node first');
});

test('a sampler with no negative graph link does not overwrite an unrelated text node', () => {
  const graph = {
    '90': { class_type: 'CLIPTextEncode', inputs: { text: 'do not overwrite', clip: ['1', 0] } },
    '4': { class_type: 'CLIPTextEncode', inputs: { text: 'real positive', clip: ['1', 0] } },
    '10': { class_type: 'KSampler', inputs: { positive: ['4', 0], negative: null } }
  };
  const result = Comfy.applyPrompts(graph, 'new positive', 'new negative');
  assert.equal(result.positiveId, '4');
  assert.equal(result.negativeId, null);
  assert.equal(result.workflow['90'].inputs.text, 'do not overwrite');
  assert.equal(result.hasNegativeTarget, false);
});
