'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Comfy = require('../src/comfy-workflow.js');
const root = path.join(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('bundled SD 1.5 API workflow is valid and routes both positive and negative prompts', () => {
  const workflow = JSON.parse(read('integrations/workflows/aurora-sd15-api.json'));
  assert.equal(Comfy.validate(workflow), 7);
  const applied = Comfy.applyPrompts(workflow, 'floating islands at sunset', 'blurry');
  assert.equal(applied.workflow['2'].inputs.text, 'floating islands at sunset');
  assert.equal(applied.workflow['3'].inputs.text, 'blurry');
  assert.equal(applied.hasNegativeTarget, true);
  assert.equal(workflow['2'].inputs.text.includes('floating islands'), false, 'template is not mutated');
});

test('image setup is exposed through main/preload and bundled in the Windows installer', () => {
  const main = read('src/main.js');
  const preload = read('src/preload.js');
  const packageJson = JSON.parse(read('package.json'));
  assert.match(main, /secureHandle\('image:prepare-engine'/);
  assert.match(main, /loadBundledComfyWorkflow/);
  assert.match(preload, /prepareImageEngine:.*image:prepare-engine/);
  assert.match(preload, /onImageSetupProgress/);
  assert.ok(packageJson.build.extraResources.some(item => item.from === 'scripts/prepare-image-engine.ps1'));
});

test('image engine installer prepares Python, CPU PyTorch, ComfyUI and a hash-verified SD 1.5 checkpoint', () => {
  const script = read('scripts/prepare-image-engine.ps1');
  assert.match(script, /Python\.Python\.3\.13/);
  assert.match(script, /Comfy-Org\/ComfyUI/);
  assert.match(script, /download\.pytorch\.org\/whl\/cpu/);
  assert.match(script, /stable-diffusion-1\.5-archive/);
  assert.match(script, /6ce0161689b3853acaa03779ec93eafe75a02f4ced659bee03f50797806fa2fa/);
  assert.match(script, /--cpu/);
  assert.match(script, /127\.0\.0\.1:8188/);
});

test('chat image requests offer automatic setup instead of requiring a manually chosen API workflow', () => {
  const app = read('src/app.js');
  const image = app.slice(app.indexOf('async function sendImageRequest'), app.indexOf('async function routeIntoWorkMode'));
  assert.match(image, /window\.aurora\.prepareImageEngine\(\)/);
  assert.match(image, /onImageSetupProgress/);
  assert.match(image, /status=await window\.aurora\.comfyStatus\(\)/);
});
