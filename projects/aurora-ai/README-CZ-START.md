# Aurora AI v0.8.6 — rychlý start

## 1. Sestavení Windows instalátoru

1. Rozbal celý ZIP do běžné zapisovatelné složky, například na Plochu. Nespouštěj build přímo z archivu.
2. Měj nainstalovaný Node.js 22.12.0 nebo novější a npm. Pro tento Electron 45 projekt doporučuji Node.js 24 LTS; Node.js 26.8.1 je také podporovaný build skriptem.
3. Dvojklikem spusť `START-BUILD-AURORA.bat`. Okno zůstane otevřené i po chybě.
4. Skript nainstaluje závislosti, stáhne runtime Electronu příkazem `install-electron`, ověří binárku, provede kontrolu syntaxe, všech 391 automatických testů, release preflight a Windows NSIS packaging. Každý krok vypíše vlastní návratový kód. Test zrušení projektové kontroly je synchronizován se skutečným spuštěním procesu; zrušení během předběžné validace se vrací jako normální stav „Zrušeno“.
5. Úspěch je potvrzen pouze tehdy, když existuje `dist/Aurora-AI-Setup-0.8.6.exe`. Vedle něj se vytvoří SHA-256 soubor.

Při chybě pošli oba soubory `BUILD-AURORA-output.log` a `BUILD-AURORA-last-run.log`. První obsahuje kompletní výstup příkazů včetně chyb, druhý stručně označí krok, který selhal. Už není potřeba dělat screenshoty terminálu.

`npm run dist` zůstává dostupný pro ruční sestavení. Startovací BAT používá stejné kontroly jako samostatné kroky, aby bylo možné jednoznačně určit chybu bez skrytého řetězení příkazů.

## 2. První otevření

- Nastav jméno a vlastní krátké instrukce v Nastavení → Osobní profil.
- V chatu je výchozí volba Auto; model se vybírá podle úkolu. V nabídce modelu můžeš kdykoliv připnout přesný nainstalovaný model Ollama. Model Hub slouží ke stahování a správě dalších modelů; stažení nemění aktivní volbu.
- Přepínač Chat / Work je uprostřed horní lišty. Oba režimy používají stejnou konverzaci, historii, levou navigaci, zprávy i composer. Work pouze přidá výběr projektu a pracovní kroky / návrhy změn přímo do zprávy asistenta. Výběr modelu a výkonu zůstává jednoduchý jako v ChatGPT nebo Claude.
- Pokročilé parametry jsou v Nastavení → Model a odpovědi; zvuky a přechody v Nastavení → Zvuk a animace.

## 3. Lokální AI a obrázky

- Spusť Ollamu na `127.0.0.1:11434`.
- Volitelně spusť ComfyUI na `127.0.0.1:8188` a vyber ověřený API workflow.
- Ze zdrojové složky lze spustit `npm run smoke:local`. Příkaz kontroluje jen dostupnost lokálních služeb; neposílá prompty a negeneruje obsah.
- Lokální sémantická paměť je standardně vypnutá. Pro použití stáhni kompatibilní embedding model a zapni ji v Nastavení → Data a soukromí.

## 4. Vývojové nástroje

- Vyber projektovou složku v Nástroje → Projektové kontroly.
- VS Code executable vyber jednou; poté otevře aktuálně schválený projekt.
- Unity a Godot mají read-only diagnostiku; import/build akce vyžadují potvrzení.
- CapCut audit nemění draft ani média.
- Roblox Bridge se instaluje až po potvrzení. Po instalaci restartuj Roblox Studio a testuj nejprve na kopii place.
- Agent může připravit návrh změn; před schválením musíš otevřít celý obsah každého upravovaného souboru.

## 5. Bezpečnost

Volné shell příkazy z agentního chatu nejsou povolené. Projektový runner nabízí jen přednastavené akce. Zápisy vyžadují náhled a schválení. API klíče se ukládají přes Electron `safeStorage` a nejsou součástí workspace zálohy.

## 6. Poznámka k Windows buildu v0.8.6

Test projektových prostorů porovnává původní a vrácenou kanonickou cestu, místo aby natvrdo předpokládal linuxový tvar `/work/game`. Na Windows proto správně zachová například `C:\work\game`; na POSIX systémech zachová POSIX cestu. Test ověřuje neměnnost umístění při přejmenování a odebrání metadat projektu.

Windows build nyní spouští syntax check, testy, release preflight a NSIS packaging samostatně. Výstup každého kroku ukládá do `BUILD-AURORA-output.log` a stav do `BUILD-AURORA-last-run.log`. Tím se rozliší chyba testu od chyby stahování Electron/NSIS nástrojů nebo samotného instalátoru.

Statické kontroly samy o sobě nejsou potvrzením vytvořeného instalátoru. Živé Ollama/ComfyUI generování, cloudové volání a skutečné otevření externích editorů je nutné ověřit na cílovém počítači.

## Když sestavení selže

Aurora používá Electron 45.1.0. Od Electronu 42 se runtime nestahuje automaticky při `npm install`, proto build spouští lokální `node_modules\.bin\install-electron.cmd --no` ještě před testy a balením. Následně ověří existenci i velikost `node_modules\electron\dist\electron.exe`. Instalace závislostí má jeden opakovaný pokus po ověření npm cache. Podrobnosti včetně npm debug logu se ukládají do `BUILD-AURORA-output.log`. Další postup je popsán v `BUILD-LOGS-HELP-CZ.md`.


## Nové ovládání a obrazový engine

- Hlavní navigace nabízí Uložené chaty, Projekty, Nedávné/Archiv, Modely, Nástroje a Galerii. Přepínač Chat / Work je uprostřed horní lišty a nemění stránku ani historii; pouze mění chování stejného chatu. Nastavení zůstává dostupné přes ozubené kolo.
- Běžný chat zůstává určený pro text. Zadání na vytvoření nebo úpravu projektu se automaticky přesměruje do Work mode ve stejné konverzaci; vyžádá si pouze výběr pracovní složky a zobrazí postup i návrhy změn přímo mezi zprávami.
- Při generování obrázku z chatu se při chybějícím enginu nabídne jednorázová automatická příprava: Python 3.13, ComfyUI, CPU PyTorch a výchozí Stable Diffusion 1.5 model. Model má přibližně 4,27 GB a instalace potřebuje alespoň 14 GB volného místa.
- Obrazový engine se spouští lokálně na 127.0.0.1:8188. Výchozí režim je CPU, aby fungoval i bez podporovaného GPU backendu; generování proto může být pomalé. Model se ověřuje SHA-256 a výchozí workflow se nastaví automaticky.
- Node.js je potřeba pro vývojářské sestavení instalátoru, ne pro samotné generování obrázků. Pokud Node.js při sestavení chybí, build skript se jej pokusí nainstalovat přes winget.
- Výběr již nainstalovaného chatovacího modelu se ukládá samostatně. Stažení dalšího modelu samo o sobě nemění aktivní výběr. Nabídka modelu obsahuje také poskytovatele (Ollama, OpenAI/kompatibilní API, Anthropic a Gemini); poskytovatel se mění uvnitř nabídky, nikoli druhým prvkem v composeru.
- Tlačítka Start/Stop Ollama ovládají pouze instanci, kterou spustila Aurora. Již běžící externí službu Aurora sama nevypíná.


## Obnovení animovaného pozadí při aktualizaci

Při první instalaci této aktualizace se jednorázově obnoví přehrávání původního videa polární záře a intenzita se nastaví nejméně na 58 %. Video je v `src/aurora-background.mp4`; poster zůstává jako statická záloha při načítání. Následně funguje uživatelský posuvník intenzity bez vynuceného minima a pozdější ruční změna zůstane uložená.


## Závěrečný audit UI a omezení

- V nabídce modelů je dostupný výběr poskytovatele; mimo nabídku není v composeru druhý samostatný přepínač poskytovatele.
- Galerie je dostupná přímo z hlavní navigace a otevírá historii lokálně vytvořených obrázků. Projektové soubory vytvořené v Code zůstávají v uživatelem vybrané pracovní složce; Galerie není univerzální správce všech souborů.
- Intenzita pozadí respektuje aktuální uživatelské nastavení. Jednorázová migrace napravuje staré vypnuté nebo příliš nízké preference, ale CSS dále nepřepisuje uživatelský posuvník.
- Automatické směrování do Code není neomezený shell ani plný autonomní Codex: čtení je omezené na vybraný projekt, změny se připravují jako návrhy a zápis vyžaduje explicitní kontrolu a schválení.
- Bezpečné změnové návrhy jsou textové. CSV/TSV jsou podporované; přímé binární exporty DOCX/XLSX/PPTX/PDF nejsou součástí této verze.
- CPU ComfyUI setup je automatizovaný, ale vyžaduje internet, přibližně 4,27 GB checkpointu a alespoň 14 GB volného místa. Na RX 580 je výchozí CPU režim pomalý; GPU backend se automaticky neslibuje ani netvrdí jako otestovaný.
- Testy a release preflight jsou regresní kontroly zdrojového kódu. Nepotvrzují skutečný NSIS build, vizuální vykreslení v Electronu na Windows, živý běh Ollamy ani první stažení a inference ComfyUI.


## 7. Kompatibilita počítače

- V nabídce Nástroje otevři kartu **Nástroje na tomto počítači** a klikni na **Zkontrolovat znovu**.
- Aurora ověřuje verze Electronu, Chromium a vloženého Node.js a hledá Node.js, Python, Git, .NET SDK, Java/JDK, Go, Rust, VS Code, Godot, Blender, Unity Editor a Roblox Studio.
- Kontrola používá pouze pevně dané příkazy pro zjištění verze (například `node --version`, `python --version`, `git --version`) a spouští je bez shellu. Neprochází obsah tvých projektů a nic neinstaluje.
- Chybějící volitelný nástroj neblokuje běžný chat v nainstalované Auroře. Node.js je vyžadován pro build ze zdrojů a odpovídající Node projektové akce; Python/Java/Go/Rust jsou potřeba jen pro příslušné typy projektů.
- Stav **Neověřeno** znamená, že spustitelný soubor byl nalezen, ale nepodařilo se bezpečně získat jeho verzi. Neznamená to automaticky nekompatibilitu.


## 8. Chaty, uložené a více projektů

- Chat připneš nebo odebereš z Uložených přes nabídku `⋯` u konverzace. Stejná nabídka nabízí přejmenování, archivaci, smazání a přiřazení k projektu.
- Projekt vytvoříš tlačítkem `+` v sekci Projekty. Každý projekt má vlastní vybranou složku a vlastní chaty; projekt můžeš přejmenovat nebo odebrat bez mazání zdrojových souborů.
- Přesunutí chatu do projektu mění vazbu v Auroře, nepřesouvá ani nekopíruje soubory projektu. Odebráním projektu se smažou jen metadata projektu a jeho vazby v Auroře.
- `previews/final/` obsahuje snímky pěti hlavních obrazovek z aktuálního HTML/CSS. Jde o statické Chromium náhledy se vzorovým obsahem; živé služby a Electron runtime neověřují.

## 9. Aktuální zdrojové kontroly tohoto balíčku

- JavaScript syntaxe: **42/42** souborů.
- Automatické testy: **391/391**.
- Release preflight: **43/43**.
- Tyto výsledky potvrzují zdrojové kontroly; neprokazují vytvoření Windows NSIS instalátoru ani živou inferenci Ollama/ComfyUI.


## Chat / Work používá jednu společnou konverzaci

- Horní přepínač Chat / Work vždy zobrazí stejnou chatovou stránku a zachová aktuální konverzaci. Přepnutí z Projekty nebo Nastavení se vrátí ke stejné konverzaci v hlavním chatu.
- Work není samostatný dashboard. Používá stejnou levou navigaci, stejný seznam zpráv a stejný composer. Jen zobrazí výběr pracovního projektu a pracovní průběh / náhledy změn přímo ve zprávách asistenta.
- Režim se ukládá ke konkrétní konverzaci; po otevření chatu se obnoví jeho režim. Starý interní Agent dashboard zůstává pouze jako skrytý kompatibilní host, nikoli jako samostatná stránka.
- Skutečné Chromium náhledy jsou v `previews/current/Chat.png`, `previews/current/Work.png` a `previews/current/Settings.png`. Jde o statické vykreslení HTML/CSS s demonstračním obsahem, nikoli o živé screenshoty Electron/Windows.


## Poslední opravy podle screenshotů

- Chat se při otevření jiné sekce skryje; Nastavení, Projekty a další stránky už nepřekrývá úvodní chat.
- V Rychlých akcích není duplicitní Work mode.
- Sekce Uložené je lokalizovaná a zobrazuje počet připnutých chatů.
- Sponka připojuje soubory; plus u pracovního projektu přidává projekt.
- Vyhledávání používá jeden jemný focus rámeček.
- Aktuální statické Chromium snímky jsou v `previews/current/`.

Aktuální kontroly: **391/391 testů**, **42/42 JavaScript souborů**, **43/43 release kontrol**.


### Umístění chatu
Prázdný chat je vycentrovaný lehce pod středem. Po zahájení konverzace se zprávy zobrazí v běžné posuvné oblasti a composer zůstane dole.


## Poslední UI úprava – 31. 10. 2026
- Sidebar lze sbalit a obnovit tlačítkem v horní liště.
- Chat a Work používají stejný chat a composer; Work pouze doplňuje projektový kontext a schvalované návrhy souborů.
- Connect centrum sjednocuje Unity, Godot, Roblox Studio, VS Code a plánovaný Unreal konektor.
- Úvodní chat se zobrazuje i při prázdné historii; prázdný kontejner zpráv je skutečně prázdný.
- Testy: **391/391**, syntaxe: **42/42**, release preflight: **43/43**.


### Aktuální obrazové náhledy
Ve složce `previews/current/` jsou skutečné statické Chromium rendery z aktuálního HTML/CSS: `Chat.png`, `Work.png`, `Connect.png` a `Sidebar-Collapsed.png`. Používají poster původního videa a demonstrační text; nejsou to AI ilustrace ani živý screenshot běžícího Electronu.
