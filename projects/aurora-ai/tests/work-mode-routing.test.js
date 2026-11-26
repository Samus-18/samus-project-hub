const test = require('node:test');
const assert = require('node:assert/strict');
const { detectWorkRequest } = require('../src/intent-router');

test('explicit work commands always route to Work mode', () => {
  assert.equal(detectWorkRequest('/work vytvoř desktopovou aplikaci'), 'vytvoř desktopovou aplikaci');
  assert.equal(detectWorkRequest('/codex refaktoruj src/app.js'), 'refaktoruj src/app.js');
});
test('Czech and English creation/edit tasks route to Work mode', () => {
  assert.ok(detectWorkRequest('Předělej rozhraní aplikace a přidej tlačítko Start Ollama'));
  assert.ok(detectWorkRequest('Create a website with a settings panel'));
  assert.ok(detectWorkRequest('Fix the Python script in my project'));
  assert.ok(detectWorkRequest('Napiš mi profesionální životopis jako DOCX'));
  assert.ok(detectWorkRequest('Vytvoř prezentaci o astronomii'));
  assert.ok(detectWorkRequest('Create an Excel spreadsheet for tracking expenses'));
  assert.ok(detectWorkRequest('Napiš technickou dokumentaci k API')); 
});
test('image and pure web research intents stay outside Work mode', () => {
  assert.equal(detectWorkRequest('Vytvoř obrázek fantasy hradu'), null);
  assert.equal(detectWorkRequest('Najdi na internetu nejlepší lokální modely'), null);
  assert.equal(detectWorkRequest('Jak vytvořit webovou stránku?'), null);
});
test('ordinary text questions remain in normal chat', () => {
  assert.equal(detectWorkRequest('Vysvětli mi rozdíl mezi RAM a VRAM.'), null);
  assert.equal(detectWorkRequest('Napiš mi krátké blahopřání.'), null);
});
