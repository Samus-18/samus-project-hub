# Aurora AI v0.6.7 — Windows build pipeline reliability

## Změny
- Opraven `BUILD-AURORA.bat`, který stále zobrazoval verzi 0.5.6 a oznamoval neaktuální název instalátoru.
- Windows build shortcut nyní před `electron-builder` spustí úplný `npm run check`, který pokrývá i nové runtime moduly, a kompletní `npm test`.
- Chybové hlášky ve skriptu nyní jasně rozlišují selhání instalace závislostí, syntaxe, testů a sestavení.
- User-Agent pro webový výzkum odpovídá aktuální verzi aplikace.
- Přidán regresní test, aby se stará verze nebo neúplná kontrolní pipeline do build skriptu nevrátila.

## Ověření
- `npm test`
- `npm run check`
- Kontrola ZIP archivu

Windows instalátor nebyl sestaven v tomto prostředí, protože Electron builder závislosti nejsou nainstalované; skript je však nyní sladěný s aktuálním `package.json` a kontrolami projektu.
