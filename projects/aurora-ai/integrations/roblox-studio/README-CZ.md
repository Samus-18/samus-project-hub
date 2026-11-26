# Aurora Bridge pro Roblox Studio — bezpečný MVP

Bridge je lokální Studio plugin pro ručně vybraný zdrojový kód. Nepřipojuje se k internetu ani k náhodnému síťovému serveru.

## Instalace
1. Otevři Roblox Studio a vytvoř lokální plugin podle aktuálního postupu Studio.
2. Vlož obsah souboru `AuroraBridge.plugin.lua` a ulož plugin.
3. Otevři tlačítko **Aurora AI** v horní liště.
4. Vyber jeden nebo více `Script`, `LocalScript` nebo `ModuleScript` objektů v Exploreru a klikni na **Načíst vybrané skripty**.
5. Zkopíruj zdroj do Aurora chatu a zadej úkol. Více vybraných skriptů se načte pouze jako kontext; nelze je tímto pluginem hromadně přepsat.

## Bezpečné použití navržené změny
1. Vyber a načti přesně jeden skript.
2. V textovém poli zkontroluj a uprav celý navržený zdroj.
3. Klikni na **Připravit náhled změny…**. Panel vypíše cíl a souhrn počtu řádků/znaků před a po; obsah návrhu musíš zkontrolovat v poli.
4. Klikni podruhé na **Potvrdit a aplikovat změnu**. Plugin znovu porovná původní zdroj, ověří aktuální výběr a použije `ScriptEditorService:UpdateSourceAsync`.
5. Změna se zaznamená do `ChangeHistoryService`, takže ji lze vrátit přes Undo. Po aplikaci zkontroluj script a otestuj kopii place.

## Bezpečnost a omezení
- Zápis se provádí pouze do jednoho skriptu, který byl předem načten a zůstává vybraný.
- Pokud se zdroj mezitím změnil, výběr se změnil, obsah přesahuje 300 kB nebo Studio nemůže zahájit Undo recording, plugin změnu odmítne.
- Kód se automaticky neposílá Auroře ani na internet. Kopírování do vzdáleného poskytovatele z Aurory podléhá nastavení daného API a jeho soukromí.
- Změna zdrojového kódu je citlivá akce; vždy prohlédni navržený celý obsah a měj zálohu projektu.
- Plugin zde nebyl spuštěn uvnitř Roblox Studio. Použij testovací kopii place a před běžným použitím ověř zápis, editor a Undo/Redo.
