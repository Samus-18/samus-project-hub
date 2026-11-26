'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const NavigationSafety = require('../src/navigation-safety');
const fs = require('node:fs');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

test('navigation allows only the exact local application entry page', () => {
  const entry = path.join(root, 'src', 'index.html');
  const allowed = pathToFileURL(entry).href;
  assert.equal(NavigationSafety.isAllowedAppNavigation(allowed, entry), true);
  assert.equal(NavigationSafety.isAllowedAppNavigation(pathToFileURL(path.join(root, 'README.md')).href, entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation('https://example.com/', entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation('file:///tmp/other.html', entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation(`${allowed}?redirect=1`, entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation(`${allowed}#fragment`, entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation('file://evil-host/etc/passwd', entry), false);
  assert.equal(NavigationSafety.isAllowedAppNavigation(null, entry), false);
});

test('Electron main process blocks renderer navigation outside the exact app entry point', () => {
  const main = read('src/main.js');
  assert.match(main, /const entryPath=path\.join\(__dirname,'index\.html'\)/);
  assert.match(main, /will-navigate',[\s\S]{0,160}NavigationSafety\.isAllowedAppNavigation\(url,entryPath\)/);
  assert.match(main, /setWindowOpenHandler\(\(\)=>\(\{action:'deny'\}\)\)/);
});

test('renderer CSP permits local APIs but blocks direct external fetch and remote images', () => {
  const html = read('src/index.html');
  const csp = html.match(/<meta\b(?=[^>]*http-equiv="Content-Security-Policy")(?=[^>]*content="([^"]+)")[^>]*>/i)?.[1] || html.match(/content="([^"]+)"[^>]*http-equiv="Content-Security-Policy"/i)?.[1] || '';
  assert.match(csp, /img-src 'self' data: blob: http:\/\/127\.0\.0\.1:8188/);
  assert.match(csp, /connect-src 'self' http:\/\/127\.0\.0\.1:11434 http:\/\/127\.0\.0\.1:8188/);
  assert.doesNotMatch(csp, /img-src[^;]*https:/i);
  assert.doesNotMatch(csp, /connect-src[^;]*https:/i);
  assert.match(mainScriptSource(), /fetch\(/, 'network requests stay in the main process, behind the IPC bridge');
});
function mainScriptSource() { return read('src/main.js'); }
