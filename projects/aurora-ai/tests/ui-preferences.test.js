const test = require('node:test');
const assert = require('node:assert/strict');
const UI = require('../src/ui-preferences.js');

function memoryStorage(initial = {}) {
  const map = new Map(Object.entries(initial));
  return {
    getItem(key) { return map.has(key) ? map.get(key) : null; },
    setItem(key, value) { map.set(key, String(value)); },
    removeItem(key) { map.delete(key); }
  };
}

test('UI preferences clamp values and accept only supported locale and theme', () => {
  const prefs = UI.normalizePreferences({locale:'fr',theme:'neon',backgroundIntensity:5,reduceMotion:1,fontScale:4,showDashboard:'yes'});
  assert.equal(prefs.locale, 'cs');
  assert.equal(prefs.theme, 'aurora');
  assert.equal(prefs.backgroundIntensity, 1);
  assert.equal(prefs.reduceMotion, false);
  assert.equal(prefs.fontScale, 1);
  assert.equal(prefs.showDashboard, false);
});

test('UI preferences support English, accessible graphite theme and reduced motion', () => {
  const prefs = UI.normalizePreferences({locale:'en',theme:'graphite',backgroundMotion:false,reduceMotion:true,fontScale:1.2,showDashboard:true});
  assert.equal(prefs.locale, 'en');
  assert.equal(prefs.theme, 'graphite');
  assert.equal(prefs.backgroundMotion, false);
  assert.equal(prefs.reduceMotion, true);
  assert.equal(prefs.fontScale, 1.2);
  assert.equal(prefs.showDashboard, true);
});

test('UI preferences persist and restore independently from chat history', () => {
  const storage = memoryStorage();
  const saved = UI.savePreferences(storage, {locale:'en',theme:'twilight',backgroundIntensity:0.4,showDashboard:true});
  const loaded = UI.loadPreferences(storage);
  assert.deepEqual(loaded, saved);
  assert.equal(JSON.parse(storage.getItem('aurora.ui.preferences.v1')).locale, 'en');
});

test('malformed persisted UI preferences fall back safely', () => {
  const storage = memoryStorage({'aurora.ui.preferences.v1':'{broken'});
  assert.deepEqual(UI.loadPreferences(storage), UI.DEFAULTS);
});


test('common select options and longer safety descriptions have English translations', () => {
  assert.equal(UI.translateLabel('Čeština', 'en'), 'Czech');
  assert.equal(UI.translateLabel('20 · soustředěné', 'en'), '20 · focused');
  assert.match(UI.translateLabel('Webové vyhledávání předává dotaz vyhledávači. ComfyUI běží lokálně. Projektové soubory se čtou jen z vybrané složky; změny vyžadují výslovné schválení a systémové příkazy se automaticky nespouštějí.', 'en'), /Web search sends your query/);
  assert.equal(UI.translateLabel('Czech', 'cs'), 'Čeština');
});


test('dynamic localization observes only the changed text node or attribute element', () => {
  const fs = require('node:fs');
  const source = fs.readFileSync(require.resolve('../src/ui-preferences.js'), 'utf8');
  assert.match(source, /record\.type==='characterData'\) translatedNodeText\(record\.target\)/);
  assert.match(source, /record\.type==='attributes'\) translateAttributes\(record\.target\)/);
  assert.doesNotMatch(source, /record\.type==='characterData'\)translateTree\(record\.target\.parentElement\)/);
});

test('background intensity supports the full 0–100 percent range', () => {
  assert.equal(UI.normalizePreferences({backgroundIntensity: 1}).backgroundIntensity, 1);
  assert.equal(UI.normalizePreferences({backgroundIntensity: 0}).backgroundIntensity, 0);
  assert.equal(UI.normalizePreferences({backgroundIntensity: 1.5}).backgroundIntensity, 1);
  assert.equal(UI.normalizePreferences({backgroundIntensity: -0.5}).backgroundIntensity, 0);
});

