# Aurora AI v0.8.6 — Integrated workspace completion

## Hlavní změny

- **Agent Studio – bezpečnější změnové návrhy:** strukturované `aurora-changeset` návrhy sdílejí příponový allowlist s hlavním procesem. Jednotlivé i dávkové změny používají stejný panel revize.
- **Úplný obsah před schválením:** zkrácený náhled slouží jen jako rychlý přehled. Uživatel musí výslovně načíst úplný původní i nový obsah každého souboru; potvrzovací tlačítko zůstává vypnuté, dokud nejsou načteny všechny soubory a zkontrolován jejich původní hash.
- **Visual Studio Code:** nový zabezpečený konektor umožňuje vybrat spustitelný soubor a otevřít pouze aktuálně schválený kořen projektu. Proces se spouští s `shell:false`, omezeným prostředím a bez automatického spouštění příkazů v projektu.
- **Unity, Godot, CapCut a Roblox:** doplněna a sjednocena cesta přes hlavní proces, preload IPC a uživatelské rozhraní. CapCut zůstává pouze read-only audit; Unity a Godot používají statické diagnostiky a přednastavené kontroly s potvrzením; Roblox Bridge vyžaduje výslovný souhlas k instalaci a zápisu.
- **Projektové kontroly:** pevný seznam bezpečných akcí pro Git, Node/npm, Python, .NET, Rust, Go, Godot a Unity. Neexistuje volný shell v agentním chatu. Akce schopné spustit projektový kód vyžadují samostatné potvrzení.
- **Lokální runtime smoke:** `npm run smoke:local` ověřuje dostupnost Ollamy a ComfyUI pouze na loopback adresách, s limity velikosti, časovým limitem a bez přesměrování. Neodesílá prompt ani negeneruje obsah.
- **Zachování předchozích funkcí:** lokální sémantická paměť je volitelná a mimo Ollamu se nepřipojuje; rychlé chat UI, historie, osobní profil a guide zůstávají zachované.
- **Release metadata:** sjednoceny verze aplikace, HTML titulku, sestavovacího skriptu, changelogu a technického auditu.

## Bezpečnostní hranice

- Všechny projektové zápisy vyžadují náhled a výslovné schválení.
- Agentní návrhy změn nespouštějí shell a nesmějí obsahovat binární/spustitelné soubory.
- VS Code otevírá jen explicitně vybraný projekt a uživatelem vybranou aplikaci.
- CapCut audit nemění draft ani média.
- Unity/Godot kontroly, které spouštějí editor nebo import, jsou označené a vyžadují potvrzení.
- Roblox plugin se instaluje do místního adresáře pluginů až po potvrzení; cizí soubor se nepřepisuje.

## Zbývající cílové úkony

- Na Windows nainstalovat npm závislosti a vytvořit NSIS instalátor.
- Na cílovém PC ověřit Ollamu, embeddingy, ComfyUI, vzdálené providery a editory s vlastní konfigurací.
- Živě ověřit Roblox Studio plugin v testovací place a porovnat Unity/Godot diagnostiku se skutečnými editory.

Tyto cílové úkony nejsou vydávány za dokončené pouze na základě automatických testů.


### UI refinement — Home / Code and model selection
- Sidebar keeps one Home / Code switch; the duplicate header Code button is hidden.
- Local model picker reads the deduplicated chat-capable model list from Ollama `/api/tags`, includes an Auto-by-task option, and allows pinning an exact model.
- Manual model selection updates the active top status at the same time; pulling another model preserves the current Auto/manual selection.
- Effort picker exposes Auto, Low, Medium, High and Extra without placing low-level sampling controls in the chat composer.
- The supplied looping background video remains active for every theme when background motion is enabled.


## Oprava vzhledu Home / Code
- Opraveno obnovení původního video pozadí. Staré uložené preference grafitového motivu už nezpůsobí vypnutí MP4 nebo téměř neviditelné pozadí.
- Při jednorázové migraci se nastaví viditelnost pozadí alespoň na 58 %; poster slouží jako fallback při načítání.
- Chat startovní plocha je jednodušší a bez nadbytečných návrhových dlaždic.
- Seznam modelů, automatický výběr a výkon zůstávají v jednom kompaktním ovládání.


## Post-audit opravy UI a konzistence

- Model menu nyní skutečně zobrazuje volbu poskytovatele (Ollama/OpenAI/Anthropic/Gemini) jen uvnitř nabídky; v composeru se nezobrazuje duplicitní přepínač.
- Přidána položka Galerie do hlavní navigace pro historii lokálních obrázků.
- Pozadí respektuje uživatelskou intenzitu; migrační minimum 58 % je jednorázové, nikoliv trvalé CSS omezení.
- Opraveno měření scroll pozice před aktualizací streamované zprávy.
- Výkon Extra se zachovává při normalizaci, ukládání i importu workspace.
- Při původním UI auditu: 355 testů, 40/40 syntax souborů, 42/42 release preflight kontrol; novější kompatibilitní aktualizace je popsána níže.
- Dokumentace explicitně vyjmenovává omezení: bez živého Windows NSIS/ComfyUI/Ollama testu, bez binárního exportu DOCX/XLSX/PPTX/PDF a bez lockfile.


### Host compatibility and developer-experience polish

- Added a host compatibility dashboard to Tools & Integrations with OS/architecture and embedded Electron/Chromium/Node runtime information.
- Added safe fixed-argument version probes for Node.js, Python, Git, .NET SDK, Java/JDK, Go, Rust, VS Code, Godot, Blender, Unity Editor, and Roblox Studio. Probes use `shell:false`, a 2.5-second timeout and bounded output; they never run project scripts.
- Added explicit statuses for available, outdated, missing, and unverified tools. Optional tool absence does not block installed-app chat.
- Improved the compact compatibility cards by hiding full executable paths in the detail line while retaining them as a tooltip.
- Added regression coverage for Java/Go/Rust compatibility and secure host scanning.
- Historical source checks at that point: 366 tests, 41 syntax-checked JS files, and 42 release preflight checks.


### Windows test runner — cancellation race fix

- Fixed a Windows timing race in project-check cancellation: cancellation during asynchronous project preflight now returns a structured `cancelled` result instead of being reported as an error.
- Reworked the running-process cancellation test to wait for process spawn before aborting; added separate tests for cancellation before spawn and during asynchronous preflight.
- Updated the Tools UI message so a running process cancellation is not incorrectly described as cancellation before process launch.
- Historical regression result at that point: 366/366 tests, 41/41 JavaScript syntax files, 42/42 release preflight checks.


### Final chat and multi-project usability pass

- Restructured the sidebar into Saved chats, Projects, and Recent/Archive collections; project spaces no longer masquerade as a Recent-history filter.
- Added per-chat actions: pin/unpin, assign to project, rename, archive, and delete.
- Added independent project spaces, each bound to its own selected folder and chat IDs. Removing a project removes only Aurora metadata and links; files on disk are not deleted. Absolute folder paths remain private to the main process and are not exported in workspace backups.
- Fixed asynchronous workspace locking before folder selection, serialized project metadata changes, and kept history search consistent with the active filter.
- Added real Chromium screenshots rendered from the current application HTML/CSS under `previews/final/`. The screenshots use seeded sample content, a static background poster, and do not claim to prove Electron runtime behaviour.
- Final source checks: **377/377 tests**, **42/42 JavaScript syntax files**, **42/42 release preflight checks**.
- Windows NSIS packaging, live Ollama/ComfyUI generation and full Electron interaction remain target-PC checks, not claims made by these source tests.


### Unified Chat / Work — společné rozhraní podle referenčních screenshotů

- Chat a Work nyní sdílejí jedinou chatovou obrazovku: stejný sidebar, historii, seznam zpráv i composer. Work není samostatná stránka ani dashboard.
- Přepínač Chat / Work je uprostřed horní lišty. Přepnutí zachová aktivní konverzaci; režim se ukládá samostatně ke každé konverzaci a po jejím otevření se obnoví.
- Ve Work se v composeru zobrazí pouze výběr pracovního projektu. Průběh úkolu, kroky a návrhy změn se zobrazují uvnitř běžné zprávy asistenta. Změny souborů nadále vyžadují náhled a schválení.
- Opraven CSS konflikt, který skrýval tlačítko Work, a přidána kontrola všech stylesheetů a jejich lokálních assetů v release preflight.
- Přidány regresní testy společného DOM/composeru, per-conversation režimu, návrhů změn a kompletních CSS referencí.
- Reálné Chromium náhledy HTML/CSS jsou v `previews/current/Chat.png`, `previews/current/Work.png` a `previews/current/Settings.png`. Používají demonstrační konverzaci a poster video pozadí; nejsou tvrzením o živém Electron/Windows běhu.

Aktuální zdrojové kontroly po sjednocení Chat / Work: **387/387 testů**, **42/42 JavaScript souborů**, **43/43 release preflight kontrol**. Windows NSIS build a připojení Ollama/ComfyUI se musí ověřit na cílovém počítači.

## Oprava Windows testu projektových cest (30. 10. 2026)

- Opraven test `project rename sanitizes names and remove only removes metadata`, který natvrdo očekával POSIX cestu `/work/game`.
- Test nyní porovnává cestu vrácenou po odebrání metadat s původní kanonickou cestou vytvořenou stejným normalizátorem.
- Tím ověřuje skutečné chování na Windows (`C:\work\game`) i na POSIX (`/work/game`) bez závislosti na operačním systému vývojového stroje.
- Produkční logika projektových cest nebyla měněna; šlo o chybný předpoklad v testu.


## Poslední opravy podle screenshotů (30. 10. 2026)

- Opraveno CSS chování skrytého chatového view, které mohlo nechávat úvodní chat viditelný současně s Nastavením.
- Odstraněna duplicitní položka Work mode z Rychlých akcí; režim se mění jediným přepínačem v horní liště.
- Sjednocen štítek Uložené bez anglického `PINNED` v českém rozhraní a doplněn dynamický počet připnutých konverzací.
- Ikona sponky nahrazuje matoucí druhé plus v composeru Work; tlačítko plus zůstává pro vytvoření/přidání projektu.
- Zjemněn focus ring vyhledávání.
- Přidáno 5 screenshot-regresních testů. Aktuální výsledky: **387/387 testů**, **42/42 syntaxe**, **43/43 release preflight**.
- Aktualizované statické Chromium náhledy jsou v `previews/current/`; screenshoty používají demonstrační obsah a poster videa, nikoli živé Electron/AI služby.


### Chat layout refinement
- Empty conversations now center the welcome content and composer together, slightly below the visual midpoint, matching familiar ChatGPT/Claude landing-page rhythm.
- Short first turns settle near the composer; longer histories retain the normal scrollable message flow.


## UI refinement – 31. 10. 2026
- Přidáno jasné sbalení/obnovení postranní navigace, grid se při sbalení zmenší na nulu.
- Přepracován ovladač akcí konverzace a sjednocen výběr pracovního projektu bez duplicitního tlačítka plus.
- Nahrazeno zastaralé připojení projektového kontextu centrem Connect pro Unity, Godot, Roblox Studio, VS Code a plánovaný Unreal Engine konektor.
- Rozšířena mezera mezi Chat/Work přepínačem a runtime ovládáním.
- Opravena prázdná zprávová oblast, aby první obrazovka zobrazila uvítání uprostřed.
- Aktuální kontroly: **391/391 testů**, **42/42 syntaxe**, **43/43 release preflight**.
