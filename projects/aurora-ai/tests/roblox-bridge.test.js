'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('Roblox Studio bridge applies only a single explicitly selected script after a preview and second confirmation', () => {
  const lua = read('integrations/roblox-studio/AuroraBridge.plugin.lua');
  assert.match(lua, /#selected ~= 1 or selected\[1\] ~= loadedTarget/);
  assert.match(lua, /Připravit náhled změny/);
  assert.match(lua, /pendingApply\.proposed ~= proposed/);
  assert.match(lua, /Potvrdit a aplikovat změnu/);
  assert.match(lua, /current ~= loadedOriginal/);
  assert.match(lua, /ScriptEditorService:UpdateSourceAsync/);
  assert.match(lua, /ChangeHistoryService:TryBeginRecording/);
  assert.match(lua, /Enum\.FinishRecordingOperation\.Commit/);
  assert.match(lua, /Enum\.FinishRecordingOperation\.Cancel/);
  assert.match(lua, /MAX_SOURCE_BYTES = 300000/);
  assert.match(lua, /local applyBusy = false/);
  assert.match(lua, /if applyBusy then return end/);
  assert.match(lua, /local expectedOriginal = pendingApply\.original/);
  assert.match(lua, /if freshSource ~= expectedOriginal then/);
  assert.match(lua, /setApplyBusy\(true\)/);
  assert.match(lua, /setApplyBusy\(false\)/);
  assert.match(lua, /Zdroj byl aktualizován, ale Studio nedokončilo Undo záznam/);
  assert.match(lua, /Undo záznam se nepodařilo dokončit, ale ověřila jsem, že původní zdroj zůstal zachovaný/);
  assert.match(lua, /výsledek zápisu nelze bezpečně ověřit/);
  assert.match(lua, /local inspectOk, actualSource = pcall/);
  assert.doesNotMatch(lua, /freshSource ~= pendingApply\.original/);
});

test('Roblox bridge keeps multi-script selections read-only and does not make network calls', () => {
  const lua = read('integrations/roblox-studio/AuroraBridge.plugin.lua');
  assert.match(lua, /Hromadné zapisování je záměrně zakázané/);
  assert.match(lua, /No HTTP, telemetry/);
  assert.doesNotMatch(lua, /HttpService|RequestAsync|PostAsync|GetAsync/);
});

test('Roblox integration guide documents approval, backups and unverified Studio testing', () => {
  const doc = read('integrations/roblox-studio/README-CZ.md');
  assert.match(doc, /Klikni podruhé/);
  assert.match(doc, /Undo/);
  assert.match(doc, /testovací kopii place/);
});
