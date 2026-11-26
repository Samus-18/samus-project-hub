'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { auditCapCutProject, MAX_DRAFT_BYTES } = require('../src/capcut-project-audit');

async function makeDraft() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-capcut-'));
  const data = {
    id: 'draft-1', name: 'Test timeline', duration: 5_000_000, fps: 30,
    canvas_config: { width: 1080, height: 1920, ratio: '9:16' },
    platform: { app_source: 'cc', app_version: '9.0.0' },
    tracks: [
      { id: 'track-v', type: 'video', segments: [{ id: 'seg-v', material_id: 'video-1', target_timerange: { start: 0, duration: 4_000_000 } }] },
      { id: 'track-t', type: 'text', segments: [{ id: 'seg-t', material_id: 'text-1', target_timerange: { start: 500_000, duration: 2_000_000 } }] }
    ],
    materials: { videos: [{ id: 'video-1', path: 'assets/video/source.mp4' }], audios: [], texts: [{ id: 'text-1', content: '{"text":"Hello"}' }] }
  };
  await fs.writeFile(path.join(root, 'draft_content.json'), JSON.stringify(data));
  return { root, data };
}

test('CapCut audit reads a draft summary without changing source files', async t => {
  const { root } = await makeDraft(); t.after(() => fs.rm(root, { recursive: true, force: true }));
  const before = await fs.readFile(path.join(root, 'draft_content.json'));
  const report = await auditCapCutProject(root);
  assert.equal(report.isCapCut, true);
  assert.equal(report.projectName, 'Test timeline');
  assert.equal(report.timelineFile, 'draft_content.json');
  assert.equal(report.resolution, '1080 × 1920');
  assert.equal(report.fps, 30);
  assert.equal(report.counts.tracks, 2);
  assert.equal(report.counts.segments, 2);
  assert.equal(report.counts.textMaterials, 1);
  assert.equal(report.counts.missingMaterialRefs, 0);
  assert.equal(report.readOnly, true);
  assert.deepEqual(await fs.readFile(path.join(root, 'draft_content.json')), before);
});

test('CapCut audit supports draft_info.json when Windows draft_content.json is absent', async t => {
  const { root } = await makeDraft(); t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.rename(path.join(root, 'draft_content.json'), path.join(root, 'draft_info.json'));
  const report = await auditCapCutProject(root);
  assert.equal(report.timelineFile, 'draft_info.json');
});

test('CapCut audit reports orphan material references and malformed timeline data', async t => {
  const { root, data } = await makeDraft(); t.after(() => fs.rm(root, { recursive: true, force: true }));
  data.tracks[0].segments.push({ id: 'seg-orphan', material_id: 'missing-video', target_timerange: { start: 4_000_000, duration: 0 } });
  await fs.writeFile(path.join(root, 'draft_content.json'), JSON.stringify(data));
  const report = await auditCapCutProject(root);
  assert.equal(report.counts.missingMaterialRefs, 1);
  assert.ok(report.findings.some(item => item.code === 'MISSING_MATERIAL_REFS'));
  assert.ok(report.findings.some(item => item.code === 'INVALID_DURATIONS'));
});

test('CapCut audit rejects missing, symlinked, oversized and malformed draft files', async t => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'aurora-capcut-invalid-')); t.after(() => fs.rm(root, { recursive: true, force: true }));
  await assert.rejects(auditCapCutProject(root), /draft_content\.json/);
  await fs.writeFile(path.join(root, 'draft_content.json'), '{invalid');
  await assert.rejects(auditCapCutProject(root), /neplatný|JSON/i);
  await fs.rm(path.join(root, 'draft_content.json'));
  await fs.writeFile(path.join(root, 'draft_content.json'), Buffer.alloc(MAX_DRAFT_BYTES + 1, 0x20));
  await assert.rejects(auditCapCutProject(root), /překračuje limit/i);
  await fs.rm(path.join(root, 'draft_content.json'));
  const elsewhere = path.join(root, 'outside.json');
  await fs.writeFile(elsewhere, '{}');
  await fs.symlink(elsewhere, path.join(root, 'draft_content.json'));
  await assert.rejects(auditCapCutProject(root), /symbolický odkaz/i);
});
