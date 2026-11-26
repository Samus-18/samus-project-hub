'use strict';
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  if (root) root.AuroraComfyWorkflow = api;
})(typeof window !== 'undefined' ? window : globalThis, function () {
  const TEXT_INPUTS = Object.freeze({
    CLIPTextEncode: ['text'],
    CLIPTextEncodeSDXL: ['text_g', 'text_l'],
    CLIPTextEncodeSDXLRefiner: ['text'],
    CLIPTextEncodeSD3: ['clip_g', 'clip_l', 't5xxl'],
    CLIPTextEncodeFlux: ['clip_l', 't5xxl']
  });
  const textFields = node => {
    if (!node || !node.inputs) return [];
    return (TEXT_INPUTS[node.class_type] || []).filter(key => typeof node.inputs[key] === 'string');
  };
  const isTextEncode = node => textFields(node).length > 0;
  function applyTextPrompt(node, prompt, maxChars) {
    const bounded = String(prompt || '').trim().slice(0, maxChars);
    for (const field of textFields(node)) node.inputs[field] = bounded;
  }
  function validate(workflow) {
    if (!workflow || typeof workflow !== 'object' || Array.isArray(workflow) || !Object.keys(workflow).length) {
      throw new Error('Soubor neobsahuje platný ComfyUI API workflow.');
    }
    if (!Object.values(workflow).some(isTextEncode)) {
      throw new Error('Workflow neobsahuje podporovaný textový uzel (CLIPTextEncode, SDXL, SD3 nebo Flux). V ComfyUI použij Save (API Format).');
    }
    return Object.keys(workflow).length;
  }
  function findTextEncodeAncestor(workflow, nodeId, visited = new Set()) {
    const id = String(nodeId ?? '');
    if (!id || visited.has(id) || !Object.prototype.hasOwnProperty.call(workflow, id)) return null;
    const node = workflow[id];
    if (isTextEncode(node)) return id;
    visited.add(id);
    for (const value of Object.values(node?.inputs || {})) {
      // ComfyUI API links are [upstream node id, output index]. Ignore ordinary
      // arrays unless their first item refers to an actual node in this graph.
      if (!Array.isArray(value) || value.length < 2 || value[0] === null || value[0] === undefined) continue;
      const upstreamId = String(value[0]);
      if (!Object.prototype.hasOwnProperty.call(workflow, upstreamId)) continue;
      const found = findTextEncodeAncestor(workflow, upstreamId, visited);
      if (found) return found;
    }
    return null;
  }
  function resolvePromptNodes(workflow) {
    const textIds = Object.entries(workflow || {}).filter(([, node]) => isTextEncode(node)).map(([id]) => String(id));
    let positiveId = null, negativeId = null;
    // Prefer graph semantics over node order: KSampler, KSamplerAdvanced and
    // guider-style workflows can all expose positive/negative conditioning ports.
    const owners = Object.values(workflow || {}).filter(node => node?.inputs &&
      ['positive', 'negative'].some(key => Array.isArray(node.inputs[key]) && node.inputs[key].length >= 1));
    for (const node of owners) {
      const positiveRef = node.inputs?.positive;
      const negativeRef = node.inputs?.negative;
      if (!positiveId && Array.isArray(positiveRef) && positiveRef.length) positiveId = findTextEncodeAncestor(workflow, positiveRef[0]);
      if (!negativeId && Array.isArray(negativeRef) && negativeRef.length) negativeId = findTextEncodeAncestor(workflow, negativeRef[0]);
    }
    // Only use node order for intentionally minimal workflows with no graph
    // owner at all. Never promote an unconnected text node to a negative prompt.
    if (!owners.length) {
      positiveId = textIds[0] || null;
      negativeId = textIds.find(id => id !== positiveId) || null;
    }
    return { positiveId, negativeId, textNodeIds: textIds, hasConditioningOwner: owners.length > 0 };
  }
  function applyPrompts(inputWorkflow, prompt, negative = '') {
    validate(inputWorkflow);
    const workflow = JSON.parse(JSON.stringify(inputWorkflow));
    const selected = resolvePromptNodes(workflow);
    if (!selected.positiveId || !workflow[selected.positiveId]?.inputs) throw new Error('Nepodařilo se určit pozitivní prompt ve workflow.');
    applyTextPrompt(workflow[selected.positiveId], prompt, 5000);
    if (String(negative || '').trim() && selected.negativeId && selected.negativeId !== selected.positiveId) {
      applyTextPrompt(workflow[selected.negativeId], negative, 2000);
    }
    return { workflow, positiveId: selected.positiveId, negativeId: selected.negativeId, hasNegativeTarget: !!selected.negativeId && selected.negativeId !== selected.positiveId, textNodeIds: selected.textNodeIds };
  }
  return Object.freeze({ validate, findTextEncodeAncestor, resolvePromptNodes, applyPrompts, isTextEncode, textFields });
});
