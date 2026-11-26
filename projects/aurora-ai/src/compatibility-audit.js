'use strict';

const TOOL_REQUIREMENTS = Object.freeze({
  node: Object.freeze({ label: 'Node.js', purpose: 'Spouštění a kontrola JavaScript / TypeScript projektů a vývojářské sestavení.', minimum: '22.12.0', requiredFor: 'Vývojářské sestavení a Node.js projektové kontroly; není potřeba pro běžný chat v nainstalované Auroře.' }),
  python: Object.freeze({ label: 'Python', purpose: 'Python projektové kontroly a některé lokální vývojářské nástroje.', minimum: '3.10.0', requiredFor: 'Python projektové kontroly. Obrazový engine používá vlastní izolované prostředí.' }),
  git: Object.freeze({ label: 'Git', purpose: 'Kontrola změn, historie a práce s repozitáři.', minimum: null, requiredFor: 'Volitelné pro běžný chat; potřebné pro gitové kontroly a práci s historií repozitáře.' }),
  dotnet: Object.freeze({ label: '.NET SDK', purpose: 'Build a testování .NET / C# projektů mimo Unity Editor.', minimum: '8.0.0', requiredFor: 'Volitelné. Potřebné jen pro odpovídající .NET projektové kontroly.' }),
  java: Object.freeze({ label: 'Java / JDK', purpose: 'Java projekty, Gradle a některé Minecraft nástroje.', minimum: '17.0.0', requiredFor: 'Volitelné. Potřebné jen pro Java/Gradle projekty a příslušné modovací nástroje.' }),
  go: Object.freeze({ label: 'Go', purpose: 'Kontrola a sestavení Go projektů.', minimum: null, requiredFor: 'Volitelné. Potřebné pouze pro Go projekty.' }),
  rust: Object.freeze({ label: 'Rust', purpose: 'Kontrola a sestavení Rust projektů pomocí rustc a Cargo.', minimum: null, requiredFor: 'Volitelné. Potřebné pouze pro Rust projekty.' }),
  vscode: Object.freeze({ label: 'Visual Studio Code', purpose: 'Otevření vybraného projektu v externím editoru.', minimum: null, requiredFor: 'Volitelný externí editor; Aurora umí analyzovat projekty i bez něj.' }),
  godot: Object.freeze({ label: 'Godot', purpose: 'Otevření Godot projektů a následné editorové ověření.', minimum: '4.0.0', requiredFor: 'Volitelné. Statický audit Godot projektu může fungovat i bez nainstalovaného editoru.' }),
  blender: Object.freeze({ label: 'Blender', purpose: 'Otevření a kontrola 3D / asset workflow.', minimum: null, requiredFor: 'Volitelné rozšíření pro 3D workflow.' }),
  unity: Object.freeze({ label: 'Unity Editor', purpose: 'Editorové ověření Unity projektů po statické analýze.', minimum: null, requiredFor: 'Volitelné; statický audit funguje i bez editoru.' }),
  roblox: Object.freeze({ label: 'Roblox Studio', purpose: 'Instalace a komunikace s Aurora Bridge pluginem.', minimum: null, requiredFor: 'Volitelné pro Roblox projekty; samotný Bridge vyžaduje nainstalované Studio.' })
});

function parseVersion(value) {
  const text = String(value ?? '').trim();
  const match = text.match(/(?:^|[^\d])v?(\d+)\.(\d+)(?:\.(\d+))?(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?/);
  if (!match) return null;
  return { major: Number(match[1]), minor: Number(match[2]), patch: Number(match[3] || 0), prerelease: match[4] || '', raw: text.slice(0, 160) };
}

function compareVersions(actual, minimum) {
  const a = typeof actual === 'string' ? parseVersion(actual) : actual;
  const b = typeof minimum === 'string' ? parseVersion(minimum) : minimum;
  if (!a || !b) return null;
  for (const key of ['major', 'minor', 'patch']) {
    if (a[key] > b[key]) return 1;
    if (a[key] < b[key]) return -1;
  }
  if (a.prerelease && !b.prerelease) return -1;
  if (!a.prerelease && b.prerelease) return 1;
  return 0;
}

function assessTool(id, detected = {}) {
  const spec = TOOL_REQUIREMENTS[id];
  if (!spec) throw new Error(`Unknown compatibility tool: ${String(id).slice(0, 80)}`);
  if (detected.status === 'missing' || !detected.path) {
    return { id, ...spec, status: 'missing', version: '', path: '', compatible: spec.minimum ? false : null, note: spec.minimum ? `Není nalezeno. Minimální verze: ${spec.minimum}.` : 'Není nalezeno; tato integrace je volitelná.' };
  }
  const version = String(detected.version || '').trim().slice(0, 160);
  let compatible = null;
  let note = detected.note || 'Nástroj byl nalezen.';
  if (spec.minimum && version) {
    const comparison = compareVersions(version, spec.minimum);
    compatible = comparison === null ? null : comparison >= 0;
    note = comparison === null ? `Verzi se nepodařilo ověřit; doporučeno ${spec.minimum} nebo novější.` : compatible ? `Verze splňuje požadavek ${spec.minimum}+.` : `Verze je příliš stará. Aktualizuj alespoň na ${spec.minimum}.`;
  } else if (spec.minimum && !version) {
    compatible = null;
    note = `Cesta existuje, ale verzi se nepodařilo ověřit. Doporučeno ${spec.minimum} nebo novější.`;
  }
  return { id, ...spec, status: compatible === false ? 'outdated' : version ? 'available' : 'unverified', version, path: String(detected.path).slice(0, 700), compatible, note };
}

function summarizeCompatibility(items) {
  const list = Array.isArray(items) ? items : [];
  const counts = { available: 0, outdated: 0, missing: 0, unverified: 0 };
  for (const item of list) if (Object.prototype.hasOwnProperty.call(counts, item?.status)) counts[item.status] += 1;
  return { total: list.length, ...counts, checked: list.length - counts.missing };
}

module.exports = Object.freeze({ TOOL_REQUIREMENTS, parseVersion, compareVersions, assessTool, summarizeCompatibility });
