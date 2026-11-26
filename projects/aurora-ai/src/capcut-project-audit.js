'use strict';

const fs = require('node:fs/promises');
const path = require('node:path');
const { readFileBounded } = require('./bounded-file');

const MAX_DRAFT_BYTES = 24 * 1024 * 1024;
const MAX_META_BYTES = 2 * 1024 * 1024;
const MAX_TRACKS = 500;
const MAX_SEGMENTS = 30_000;
const MAX_FINDINGS = 120;
const TIMELINE_FILES = Object.freeze(['draft_content.json', 'draft_info.json']);
const MATERIAL_GROUPS = Object.freeze(['videos', 'audios', 'texts', 'stickers', 'video_effects', 'material_animations', 'transitions', 'masks', 'canvases', 'speeds', 'placeholder_infos']);

function finding(findings, severity, code, message, detail = '') {
  if (findings.length >= MAX_FINDINGS) return;
  findings.push({ severity, code, message: String(message).slice(0, 420), detail: String(detail).slice(0, 240) });
}

function cleanText(value, max = 160) {
  return typeof value === 'string' ? value.replace(/[\u0000-\u001f\u007f]/g, '').trim().slice(0, max) : '';
}

function finiteNonNegative(value) {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0;
}

async function safeRegularFile(root, fileName, maxBytes) {
  const target = path.join(root, fileName);
  try {
    const stat = await fs.lstat(target);
    if (stat.isSymbolicLink() || !stat.isFile()) return { status: 'unsafe' };
    const real = await fs.realpath(target);
    if (real !== path.join(root, fileName)) return { status: 'unsafe' };
    if (stat.size > maxBytes) return { status: 'too-large', size: stat.size };
    return { status: 'ok', absolute: real, size: stat.size };
  } catch (error) {
    return error?.code === 'ENOENT' || error?.code === 'ENOTDIR' ? { status: 'missing' } : { status: 'unsafe' };
  }
}

function summarizeMaterials(materials, findings) {
  const counts = {};
  const ids = new Set();
  let duplicateIds = 0;
  let textItems = 0;
  for (const group of MATERIAL_GROUPS) {
    const items = Array.isArray(materials?.[group]) ? materials[group] : [];
    counts[group] = Math.min(items.length, MAX_SEGMENTS);
    for (const item of items.slice(0, MAX_SEGMENTS)) {
      if (!item || typeof item !== 'object' || Array.isArray(item)) continue;
      const id = cleanText(item.id, 180);
      if (id) {
        if (ids.has(id)) duplicateIds++;
        ids.add(id);
      }
      if (group === 'texts') {
        textItems++;
        const text = cleanText(item.content ?? item.text ?? item.text_content, 5000);
        if (!text) finding(findings, 'warning', 'EMPTY_TEXT_MATERIAL', 'Textový materiál neobsahuje čitelný text.');
      }
    }
    if (items.length > MAX_SEGMENTS) finding(findings, 'warning', 'MATERIAL_LIMIT', `Skupina ${group} překročila bezpečný limit položek; další položky byly vynechány.`);
  }
  if (duplicateIds) finding(findings, 'warning', 'DUPLICATE_MATERIAL_IDS', `Nalezeno ${duplicateIds} duplicitních ID materiálů.`);
  return { counts, ids, textItems, duplicateIds };
}

async function auditCapCutProject(rootPath) {
  if (typeof rootPath !== 'string' || !rootPath.trim()) throw new Error('Nejdřív vyber složku CapCut projektu.');
  const root = await fs.realpath(rootPath);
  const rootStat = await fs.lstat(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) throw new Error('Vybraný CapCut projekt není bezpečná běžná složka.');

  const findings = [];
  let timeline = null;
  let timelineName = '';
  let fileStatus = 'missing';
  for (const name of TIMELINE_FILES) {
    const candidate = await safeRegularFile(root, name, MAX_DRAFT_BYTES);
    if (candidate.status === 'ok') { timeline = candidate; timelineName = name; fileStatus = 'ok'; break; }
    if (candidate.status === 'too-large') { fileStatus = 'too-large'; timelineName = name; }
    else if (candidate.status === 'unsafe') { fileStatus = 'unsafe'; timelineName = name; }
  }
  if (!timeline) {
    if (fileStatus === 'too-large') throw new Error(`${timelineName} překračuje limit ${Math.round(MAX_DRAFT_BYTES / 1024 / 1024)} MB. Vyber menší nebo zkopírovaný projektový draft.`);
    if (fileStatus === 'unsafe') throw new Error('Timeline soubor je symbolický odkaz nebo není běžný soubor; audit jej z bezpečnostních důvodů neotevře.');
    throw new Error('V této složce nebyl nalezen draft_content.json ani draft_info.json. Vyber přímo složku jednoho CapCut projektu.');
  }

  let data;
  try { data = JSON.parse((await readFileBounded(fs, timeline.absolute, MAX_DRAFT_BYTES, timelineName)).toString('utf8')); }
  catch (error) { throw new Error(`CapCut timeline nelze bezpečně načíst: ${String(error.message || error).slice(0, 240)}`); }
  if (!data || typeof data !== 'object' || Array.isArray(data)) throw new Error('CapCut timeline nemá očekávaný JSON objekt.');

  const tracks = Array.isArray(data.tracks) ? data.tracks : [];
  const materials = data.materials && typeof data.materials === 'object' && !Array.isArray(data.materials) ? data.materials : {};
  if (!Array.isArray(data.tracks)) finding(findings, 'error', 'TRACKS_MISSING', 'Timeline neobsahuje pole tracks.');
  if (!data.materials || typeof data.materials !== 'object' || Array.isArray(data.materials)) finding(findings, 'warning', 'MATERIALS_MISSING', 'Timeline neobsahuje očekávaný objekt materials.');
  if (tracks.length > MAX_TRACKS) finding(findings, 'warning', 'TRACK_LIMIT', `Projekt obsahuje ${tracks.length} stop; analyzováno bude prvních ${MAX_TRACKS}.`);

  const materialSummary = summarizeMaterials(materials, findings);
  const availableMaterialIds = materialSummary.ids;
  const trackTypes = {};
  let segmentCount = 0;
  let missingMaterialRefs = 0;
  let malformedSegments = 0;
  let invalidDurations = 0;
  let firstStart = null;
  let lastEnd = 0;
  const seenSegmentIds = new Set();
  let duplicateSegments = 0;

  for (const track of tracks.slice(0, MAX_TRACKS)) {
    if (!track || typeof track !== 'object' || Array.isArray(track)) { finding(findings, 'warning', 'INVALID_TRACK', 'Jedna stopa má neplatný datový tvar.'); continue; }
    const type = cleanText(track.type, 40).toLowerCase() || 'unknown';
    trackTypes[type] = (trackTypes[type] || 0) + 1;
    const segments = Array.isArray(track.segments) ? track.segments : [];
    for (const segment of segments) {
      segmentCount++;
      if (segmentCount > MAX_SEGMENTS) break;
      if (!segment || typeof segment !== 'object' || Array.isArray(segment)) { malformedSegments++; continue; }
      const segmentId = cleanText(segment.id, 180);
      if (segmentId) {
        if (seenSegmentIds.has(segmentId)) duplicateSegments++;
        seenSegmentIds.add(segmentId);
      }
      const materialId = cleanText(segment.material_id ?? segment.materialId, 180);
      if (materialId && availableMaterialIds.size && !availableMaterialIds.has(materialId)) missingMaterialRefs++;
      const start = Number(segment.target_timerange?.start ?? segment.start ?? NaN);
      const duration = Number(segment.target_timerange?.duration ?? segment.duration ?? NaN);
      if (Number.isFinite(start) && start >= 0 && Number.isFinite(duration) && duration >= 0) {
        firstStart = firstStart === null ? start : Math.min(firstStart, start);
        lastEnd = Math.max(lastEnd, start + duration);
        if (duration === 0) invalidDurations++;
      } else if (segment.target_timerange || segment.start !== undefined || segment.duration !== undefined) invalidDurations++;
    }
  }
  if (segmentCount > MAX_SEGMENTS) finding(findings, 'warning', 'SEGMENT_LIMIT', `Projekt překročil limit ${MAX_SEGMENTS} segmentů; další segmenty nebyly analyzovány.`);
  if (malformedSegments) finding(findings, 'warning', 'INVALID_SEGMENTS', `${malformedSegments} segmentů má neplatný datový tvar.`);
  if (missingMaterialRefs) finding(findings, 'warning', 'MISSING_MATERIAL_REFS', `${missingMaterialRefs} segmentů odkazuje na materiál, který nebyl nalezen v indexovaných skupinách.`);
  if (duplicateSegments) finding(findings, 'warning', 'DUPLICATE_SEGMENT_IDS', `Nalezeno ${duplicateSegments} duplicitních ID segmentů.`);
  if (invalidDurations) finding(findings, 'warning', 'INVALID_DURATIONS', `${invalidDurations} segmentů má nulovou, zápornou nebo neplatnou délku/časování.`);
  if (!tracks.length) finding(findings, 'warning', 'EMPTY_TIMELINE', 'Projekt neobsahuje žádné stopy.');
  if (!materialSummary.textItems && (trackTypes.text || trackTypes.subtitle)) finding(findings, 'info', 'TEXT_TRACK_WITHOUT_MATERIAL', 'Projekt obsahuje textovou stopu, ale nebyly nalezeny běžné textové materiály.');

  const canvas = data.canvas_config && typeof data.canvas_config === 'object' ? data.canvas_config : {};
  const width = finiteNonNegative(Number(canvas.width)) ? Math.min(16384, Number(canvas.width)) : null;
  const height = finiteNonNegative(Number(canvas.height)) ? Math.min(16384, Number(canvas.height)) : null;
  const fps = finiteNonNegative(Number(data.fps)) ? Math.min(1000, Number(data.fps)) : null;
  const durationUs = finiteNonNegative(Number(data.duration)) ? Math.min(Number(data.duration), 7 * 24 * 60 * 60 * 1_000_000) : Math.max(0, lastEnd);
  const platform = data.platform && typeof data.platform === 'object' ? data.platform : {};
  const projectName = cleanText(data.name, 160) || cleanText(data.id, 100) || path.basename(root);
  const metadata = await safeRegularFile(root, 'draft_meta_info.json', MAX_META_BYTES);
  if (metadata.status === 'unsafe') finding(findings, 'warning', 'UNSAFE_META', 'Metadata soubor je nebezpečný odkaz nebo není běžný soubor; byl vynechán.');
  if (metadata.status === 'too-large') finding(findings, 'info', 'META_TOO_LARGE', 'Metadata překračují limit čtení a byla vynechána.');

  const counts = {
    tracks: tracks.length,
    segments: Math.min(segmentCount, MAX_SEGMENTS),
    trackTypes,
    materials: Object.fromEntries(Object.entries(materialSummary.counts).filter(([, count]) => count > 0)),
    missingMaterialRefs,
    textMaterials: materialSummary.textItems
  };
  const bySeverity = { error: 0, warning: 0, info: 0 };
  for (const item of findings) bySeverity[item.severity]++;
  return {
    isCapCut: true,
    projectName,
    timelineFile: timelineName,
    sizeBytes: timeline.size,
    appVersion: cleanText(platform.app_version, 60) || 'Nezjištěno',
    appSource: cleanText(platform.app_source, 20) || 'Nezjištěno',
    fps,
    resolution: width && height ? `${width} × ${height}` : 'Nezjištěno',
    ratio: cleanText(canvas.ratio, 24) || 'Nezjištěno',
    durationSeconds: Math.round((durationUs / 1_000_000) * 100) / 100,
    timelineRangeSeconds: Math.round((Math.max(0, lastEnd - (firstStart || 0)) / 1_000_000) * 100) / 100,
    counts,
    metadataAvailable: metadata.status === 'ok',
    findings,
    summary: bySeverity,
    generatedAt: new Date().toISOString(),
    readOnly: true,
    note: 'Read-only diagnostika CapCut draft JSON. Projektové soubory a média nebyly změněny, média nebyla otevírána ani kopírována. Formát draftu se může mezi verzemi CapCut lišit; audit není náhradou editoru.'
  };
}

module.exports = Object.freeze({ auditCapCutProject, MAX_DRAFT_BYTES, MAX_TRACKS, MAX_SEGMENTS });
