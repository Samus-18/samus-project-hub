# Aurora AI v0.8.6 — konsolidovaný technický audit

## Shrnutí

Tento dokument popisuje skutečný stav zdrojového projektu v tomto ZIPu. Čísla testů jsou regresní výsledky, nikoli procento dokončenosti. Verze zůstává `0.8.6`, protože jde o opravný a auditní balíček stejného vydání.

## Co je zapojené ve zdrojovém projektu

1. **Chat / Work:** jeden společný chatový shell. Přepínač je uprostřed horní lišty; obě volby používají stejnou levou navigaci, historii, DOM zpráv i composer. Work přidá pouze výběr projektu a pracovní kroky / návrhy změn uvnitř zpráv asistenta.
2. **Ochrana změn:** Code neprovádí volné shell příkazy. Návrhy souborových změn se zobrazují k revizi; zápis vyžaduje explicitní schválení. Textový allowlist zahrnuje mimo jiné CSV a TSV.
3. **Modely:** seznam lokálních modelů se načítá z Ollama API, deduplikuje a filtruje na chat modely. Volba Auto a ručně připnutý model jsou oddělené; stažení dalšího modelu nemění aktivní volbu.
4. **Poskytovatelé:** Ollama, OpenAI/kompatibilní API, Anthropic a Gemini jsou dostupné uvnitř modelové nabídky. Tajné klíče zůstávají v hlavním procesu a používají Electron safeStorage.
5. **Výkon:** jednoduché volby Auto, Low, Medium, High a Extra jsou v composeru; pokročilé sampling parametry zůstávají v Nastavení. Profil Extra se zachovává při uložení a importu workspace.
6. **Ollama:** stav se kontroluje v aplikaci; Start může spustit lokální službu a nabídnout instalaci přes winget, pokud Ollama chybí. Stop zastaví pouze instanci spuštěnou Aurorou, nikoli externí službu.
7. **Streamování:** aktivní zpráva se aktualizuje samostatně. Scroll pozice se měří před změnou DOM; automatické posunutí je určeno jen pro čtenáře, který zůstává u konce.
8. **Pozadí:** původní video `src/aurora-background.mp4` i poster jsou v projektu. Jednorázová migrace starých preferencí obnoví video; CSS následně respektuje aktuální uživatelskou intenzitu.
9. **Galerie:** hlavní navigace zpřístupňuje historii lokálně vytvořených obrázků. Projektové soubory vytvářené přes Code se ukládají do zvolené složky a nejsou automaticky sjednoceny do Galerie.
10. **Obrázky:** první požadavek může nabídnout automatickou přípravu Pythonu, ComfyUI, CPU PyTorch, výchozího SD 1.5 checkpointu a workflow. Checkpoint má přibližně 4,27 GB; instalace potřebuje alespoň 14 GB volného místa.
11. **Integrace:** zdroj obsahuje omezené audity a konektory pro VS Code, Unity, Godot, CapCut a Roblox. Každá integrace má vlastní bezpečnostní omezení a cílové aplikace musí být nainstalované.
12. **Build:** Windows skript kontroluje Node.js (min. 22.12.0), zajišťuje závislosti, výslovně stahuje Electron runtime pro Electron 45 a ukládá logy jednotlivých kroků.
13. **Kompatibilita hostitele:** nová karta Nástroje na tomto počítači kontroluje OS/architekturu, Electron, Chromium, vložený Node.js a dostupnost/verze Node.js, Pythonu, Gitu, .NET SDK, Java/JDK, Go, Rustu, VS Code, Godotu, Blenderu, Unity Editoru a Roblox Studia. Verze se zjišťují pevnými argumenty `spawn(..., shell:false)`; kontrola nic neinstaluje ani nespouští projektové příkazy.

## Poslední opravy z auditu

- Poskytovatel byl skrytý i po přesunutí dovnitř modelové nabídky. Opraveno: v composeru není, v otevřené nabídce je viditelný a ovladatelný.
- Galerie obrázků nebyla dostupná v hlavní navigaci. Přidána položka Galerie.
- CSS vynucovalo 58% minimální opacity, čímž blokovalo uživatelský posuvník. Vynucení odstraněno; 58% se nastavuje jen při jednorázové migraci starých preferencí.
- Streamovací scroll byl vyhodnocován po změně velikosti obsahu. Opraveno pořadí měření.
- `Extra` profil se mohl ztratit při ukládání nebo importu workspace. Normalizace byla sjednocena.

## Oprava testu rušení projektové kontroly po běhu na Windows

- Diagnostický log z 30. 10. 2026 ukázal, že instalace npm, rebuild nástrojů, stažení Electronu 45.1.0 a syntax check prošly; zastavení nastalo výhradně v kroku 5/7 (`npm test`).
- Jediný selhávající test používal pevný timeout 5 ms. Na Windows mohl signál AbortSignal dorazit během asynchronní validace projektové složky, tedy ještě před spuštěním procesu. Runner tehdy vyhodil výjimku místo standardního výsledku `cancelled`.
- Runner nyní vrací jednotný strukturovaný výsledek při zrušení před spuštěním i během validace; žádný projektový proces se v takovém případě nespustí.
- Test běžícího procesu nyní čeká na skutečnou událost spuštění a teprve potom odešle abort. Další testy pokrývají zrušení před spuštěním i během pozastavené asynchronní validace.
- UI nyní rozlišuje zrušení před spuštěním od ukončení běžící kontroly a zobrazí správnou zprávu.


## Závěrečné dotažení chatu, projektů a navigace

- Levá navigace byla přeorganizována na běžný chatový model: Chat/Work (jeden společný chat), nová konverzace, hledání, hlavní nástroje, Uložené chaty, Projekty a Nedávné/Archiv.
- Konverzace mají akce připnout/odepnout, přiřadit k projektu, přejmenovat, archivovat a smazat. Akce probíhají přes kontrolované přechody; během aktivního generování se neprovádí nebezpečné přepnutí.
- Projektové prostory jsou oddělené entity s vlastním ID a složkou. Chaty si ukládají vazbu na ID projektu; přepnutí projektu nemění jiné projektové složky. Odebrání projektu odstraní pouze jeho metadata a vazby, nemaže soubory na disku.
- Projektové cesty zůstávají v hlavním procesu a nejsou součástí exportu workspace. Záloha zachovává ID vazeb, ne absolutní cesty.
- Vyhledávání historie respektuje zvolený filtr. Seznam Uložené je určen pro připnuté chaty; projekty mají samostatnou sekci a vlastní obrazovku.
- Work režim drží vybraný projekt, ale zůstává ve stejné konverzaci jako Chat. Průběh úkolu a návrhy změn se zobrazují v běžných zprávách asistenta. Zámek pracovního prostoru se aktivuje před asynchronním výběrem složky. Zápisy stále vyžadují náhled a schválení.
- Přidány regresní testy pro samostatné projektové prostory, pinned chaty, akční nabídku konverzace, bezpečné odebrání projektu a uchování ID vazeb v zálohách.

## Opravy posledního screenshot auditu UI

- `#chatView` už nemá natvrdo vynucené zobrazení; při přechodu do Nastavení, Projektů nebo Modelů se skryje podle `.hidden`, `aria-hidden` a `inert`. Tím se odstraní souběžné vykreslení úvodního chatu a Nastavení.
- Z Rychlých akcí byla odstraněna duplicitní položka Work mode; jediný přepínač Chat / Work zůstává v horní liště.
- Sekce Uložené používá pouze český/anglický název podle jazyka UI, bez osamoceného štítku `PINNED`; počet připnutých chatů se zobrazuje jen pokud je nenulový.
- Připojení souborů má ikonu sponky; tlačítko `+` v pracovním výběru zůstává vyhrazené pro přidání projektu.
- Vyhledávání v levém panelu používá jeden tlumený focus rámeček místo dvojitého obrysu.
- Přidány regresní testy pro viditelnost jednotlivých view, unikátní přepínač Work, jazyk sekce Uložené, oddělené ovládání projektu/sponky a jeden focus ring.

## Skutečné obrazové náhledy

Složka `previews/current/` obsahuje aktuální snímky Chat, Work, Connect a sbaleného sidebaru. Historické náhledy nejsou součástí zmenšeného release ZIPu. Jde o render skutečného `src/index.html` a připojených CSS v Chromium, nikoli o AI-generované návrhy. Pro screenshot jsou vypnuty JavaScriptové funkce aplikace a vložen demonstrační obsah (zejména ukázkové názvy projektů a chatů); náhled proto dokládá HTML/CSS vzhled, ne živé připojení Ollamy ani klikací Electron runtime. Animované video je v aplikaci zachováno; ve statickém screenshotu je použit jeho poster.

## Automatické kontroly v tomto pracovním běhu

- `node --test`: **391/391** testů prošlo.
- `npm run check`: **42/42** JavaScript souborů syntakticky platných.
- `npm run release:preflight`: **43/43** statických release kontrol prošlo.
- Nové kompatibilitní testy ověřují parsování verzí, zastaralé/chybějící nástroje, zabezpečené IPC a registraci Java/Go/Rust toolchainů.
- ZIP archiv byl zkontrolován po vytvoření (`unzip -t`).

## Známá omezení — nezamlčovat

- **Windows instalátor není v tomto pracovním prostředí sestaven ani spuštěn.** Release preflight nenahrazuje Windows NSIS build.
- **Electron UI nebylo zde živě spuštěno na Windows.** Zdrojové testy neověřují kompletní vizuální vykreslení ani všechny klikací cesty v nativním desktopovém běhu.
- **Ollama nebyla v tomto běhu připojena k uživatelově počítači.** Skutečné stažené modely, inferenci, start/stop služby a paměť je nutné ověřit lokálně.
- **ComfyUI instalace a generování nebyly živě dokončeny.** Automatizační skript je součástí balíčku, ale síťové stažení Pythonu, ComfyUI, PyTorch a checkpointu závisí na Windows, winget, přístupu k síti a volném místě. Výchozí CPU inference může být na starším PC velmi pomalá.
- **GPU akcelerace RX 580 není potvrzená.** Projekt proto používá konzervativní CPU backend místo slibu nepodporované akcelerace.
- **Code není neomezený autonomní agent s plným přístupem k systému.** Je omezený na vybranou složku, textové změny a přednastavené akce; změny souborů vyžadují schválení.
- **Přímé binární exporty DOCX, XLSX, PPTX a PDF nejsou v této verzi implementované.** Router umí požadavek na tvorbu předat do Code, ale bezpečný changeset přijímá pouze podporované textové přípony. Nesmí se tvrdit, že vytvořil binární Office/PDF soubor, pokud ho skutečně nevygeneroval specializovaný exportér.
- **Instalace vyžaduje síť a npm registry.** V repozitáři není ověřený `package-lock.json`; tento balíček proto neslibuje přesně reprodukovatelný transitive dependency tree.
- **Node.js chybějící v systému:** build se pokusí použít winget; pokud winget není dostupný nebo instalace vyžaduje zásah uživatele, zobrazí konkrétní návod.

## Doporučená cílová kontrola na Windows

1. Rozbalit ZIP do nové zapisovatelné složky a spustit `START-BUILD-AURORA.bat`.
2. Požadovat existenci `dist/Aurora-AI-Setup-0.8.6.exe`; samotný návratový kód bez souboru není úspěch.
3. Nainstalovat aplikaci a ověřit video, přepnutí Chat/Work ve stejné konverzaci, změnu poskytovatele, Auto i ruční model, výkon Extra, scroll během streamování, galerii a tlačítka Ollama.
4. Teprve po výslovném potvrzení instalace ověřit ComfyUI setup a skutečné vygenerování obrázku.
5. Testovat projektové změny nejprve na kopii a potvrdit, že bez schválení nedojde k zápisu.

**Závěr:** zdrojový projekt aktuálně prochází uvedenými automatickými testy. Za hotový Windows instalátor, živě ověřenou AI inference nebo kompletní export všech dokumentových formátů se toto vydání nevydává.


## Hostitelská kompatibilita — rozsah kontroly

Kontrola je diagnostická, ne instalátor. Výsledek `Dostupné` znamená, že byl nástroj nalezen a jeho pevný příkaz pro zjištění verze skončil úspěšně. `Zastaralé` znamená verzi pod zadaným minimem; `Nenalezeno` znamená, že nebyla nalezena očekávaná spustitelná cesta; `Neověřeno` označuje existující cestu bez spolehlivého výsledku verze. U volitelných nástrojů chybějící instalace neblokuje běžný chat.

Kontrola používá pevný allowlist argumentů a `shell:false`, má časový limit 2,5 sekundy na proces a omezuje délku zachyceného výstupu. Neposkílá data do sítě a neprochází soubory uživatelových projektů. Skenuje běžné instalační cesty Windows a aktuální PATH; vlastní umístění nástrojů lze vybrat tam, kde pro ně aplikace má nastavení editoru. Není to úplný inventář všech možných instalací.


## Sjednocení Chat / Work podle referenčních screenshotů

- Work již není samostatná stránka ani dashboard. `setView('agent')` se převádí na Work režim a `chat` view; `#agentView` je skrytý kompatibilní host pro starý agentní formulář.
- Oba režimy používají `#chatView`, `#messages`, `#prompt` a `#sendBtn`. Společná je také celá levá navigace. Horní přepínač po volbě Chat nebo Work vždy vrátí uživatele do stejné aktivní konverzace.
- Work zobrazuje malý `workProjectControl` v composeru a přidává průběh rolí / stav návrhů změn do běžné zprávy asistenta. Při reloadu se režim i pracovní status/proposal zachovávají v rámci limitované workspace persistence.
- Opraven starší CSS konflikt `#workModeToggle { display:none!important }`, který v reálném vykreslení skryl volbu Work navzdory existujícímu HTML. Oba přepínače mají explicitní viditelnost a regresní test.
- Release preflight nyní kontroluje existenci všech sedmi stylesheetů odkazovaných z `index.html` a jejich lokálních `url(...)` assetů, nejen prvních dvou hlavních CSS souborů.
- Nové Chromium screenshoty: `previews/current/Chat.png`, `previews/current/Work.png` a `previews/current/Settings.png`. Jde o skutečné vykreslení HTML/CSS s demonstračním obsahem; JS/Electron IPC je vypnuté, takže náhled neprokazuje živý runtime.

## Aktuální zdrojové kontroly po sjednocení Work

- `node --test`: **391/391**.
- `node scripts/check-syntax.js`: **42/42** JavaScript souborů.
- `node scripts/release-preflight.js`: **43/43** kontrol.
- Tyto výsledky nepotvrzují živý Windows NSIS build ani skutečné připojení k Ollama/ComfyUI na cílovém počítači.

## Windows portability fix — project-space test

- Build output from 30 Oct 2026 showed 381/382 passing tests and one failing assertion in `tests/project-spaces.test.js`.
- The test expected the literal POSIX path `/work/game`, while Windows correctly normalized the same fixture to `C:\work\game`.
- The test now compares the returned folder path with the original canonical path produced by `normalizeProjectSpace`, so the assertion checks path preservation rather than a platform-specific spelling.
- No production project-space behavior was weakened or bypassed.

## Poslední UI a připojení (31. 10. 2026)

- Sidebar má viditelné tlačítko pro sbalení/obnovení a po sbalení zabere nulový sloupec; hlavní navigace se obnoví i ze staršího profilu s původní preferencí skrytí.
- Chat/Work jsou oddělené od Start/Stop akcí; oba režimy sdílejí shell, historii, zprávy a composer.
- Composer Work obsahuje jediný výběr projektu; vytvoření projektu zůstává v postranním panelu a připojení souborů má samostatnou sponku.
- Rychlé akce vedou do Connect centra. Unity, Godot, Roblox Studio a VS Code otevírají existující integrace; Unreal Engine je poctivě označen jako plánovaný, bez falešného stavu připojení.
- Prázdný chat má skutečně prázdný `#messages` element, aby CSS `:empty` zobrazilo vycentrované uvítání.
- Ověřovací snímky Chat, Work a Connect byly pořízeny z reálného HTML/CSS v Chromiu; nejde o AI generované makety ani o důkaz živého Electron runtime.

Aktuální zdrojové kontroly po této úpravě: **391/391 testů**, **42/42 syntaktických souborů**, **43/43 release preflight kontrol**. Windows NSIS instalátor a živé externí integrace stále vyžadují ověření na cílovém Windows počítači.

