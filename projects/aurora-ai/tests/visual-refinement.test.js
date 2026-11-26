'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('v0.7.2 aligns welcome actions, message list and composer in one centered workspace', () => {
  const css = read('src/desktop-refinement.css');
  assert.match(css, /\.welcome,\.welcome\.compact,\.main\.dashboard-hidden \.welcome\{[\s\S]*?text-align:center!important/);
  assert.match(css, /\.suggestions\.dashboard-actions,\.main\.dashboard-hidden \.dashboard-actions\{[\s\S]*?grid-template-columns:repeat\(3,minmax\(0,1fr\)\)!important/);
  assert.match(css, /\.messages,\.composer-wrap,\.main\.dashboard-hidden \.messages,\.main\.dashboard-hidden \.composer-wrap\{[\s\S]*?max-width:850px!important/);
  assert.match(css, /\.main:not\(\.dashboard-hidden\) \.chatview\{padding-right:276px!important/);
  assert.match(css, /@media\(max-width:640px\)/);
});

test('sidebar toggle collapses the complete sidebar while keeping its restore control visible', () => {
  const html = read('src/index.html');
  const prefs = read('src/ui-preferences.js');
  const css = read('src/connect-sidebar-refinement.css');
  assert.match(html, /<button[^>]*aria-expanded="true"[^>]*id="sidebarToggle"|<button[^>]*id="sidebarToggle"[^>]*aria-expanded="true"/);
  assert.match(html, /data-history-filter="recent"[\s\S]*?data-history-filter="archived"/);
  assert.match(html, /id="pinnedChats"[\s\S]*?id="sidebarProjectList"[\s\S]*?id="viewAllProjects"/);
  assert.match(prefs, /getElementById\('sidebarToggle'\)[\s\S]*?sidebarCollapsed/);
  assert.match(css, /\.app\.sidebar-collapsed\{grid-template-columns:0 minmax\(0,1fr\)!important;?\}/);
  assert.match(css, /\.sidebar-toggle\{display:inline-grid!important/);
});


test('interface preference normalization keeps sound off by default and clamps volume', () => {
  const prefs = require('../src/ui-preferences.js');
  const defaults = prefs.normalizePreferences({});
  assert.equal(defaults.navigationHidden, false);
  assert.equal(defaults.interfaceEffects, true);
  assert.equal(defaults.soundEffects, false);
  assert.equal(defaults.soundVolume, 0.25);
  const custom = prefs.normalizePreferences({ navigationHidden: true, soundEffects: true, soundVolume: 9 });
  assert.equal(custom.navigationHidden, true);
  assert.equal(custom.soundEffects, true);
  assert.equal(custom.soundVolume, 0.6);
});

test('local image generation uses the same optional success/error interface tones', () => {
  const app = read('src/app.js');
  assert.match(app, /setCoreState\('ready', 'OBRÁZEK HOTOV · LOKÁLNÍ VÝSTUP'\);window\.AuroraUI\?\.playSound\?\.\('complete'\)/);
  assert.match(app, /catch\(e\)\{window\.AuroraUI\?\.playSound\?\.\('error'\);setCoreState\('error', 'GENEROVÁNÍ SELHALO/);
});


test('inline navigation children open the right view and selected Settings category', () => {
  const app = read('src/app.js');
  assert.match(app, /querySelectorAll\('#mainNavigation \.nav-child'\)\.forEach\(button=>button\.addEventListener\('click'/);
  assert.match(app, /const category=button\.dataset\.settingsCategoryTarget;setView\(button\.dataset\.view,\{navGroup:group,\.\.\.\(category\?\{settingsCategory:category\}:\{\}\)\}\)/);
  assert.match(app, /function setSettingsCategory\(category\)/);
  assert.match(app, /children\.setAttribute\('aria-hidden',String\(!active\)\);children\.inert=!active/);
});


test('settings categories are not duplicated as a second horizontal tab strip', () => {
  const css = read('src/desktop-refinement.css');
  const html = read('src/index.html');
  assert.match(css, /\.settings-view \.settings-tabs\{display:none!important;\}/);
  for (const category of ['appearance','personalization','generation','sound','providers','data','safety']) {
    assert.match(html, new RegExp(`data-settings-category-target="${category}"`));
    assert.match(html, new RegExp(`data-settings-category="${category}"`));
  }
});
