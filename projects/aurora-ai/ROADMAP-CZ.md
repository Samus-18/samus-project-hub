# Aurora AI — plán vývoje

Tento plán zachovává lokální-first směr aplikace a rozšiřuje ji postupně. Neoznačovat plánované funkce jako hotové, dokud nejsou implementované a ověřené.

## Fáze 1 — vzhled a použitelnost (aktuální práce)
- [x] Animovaná, plynule se pohybující polární záře, horská silueta a odraz na jezeře.
- [x] Vlastní značka Aurora „A“ a ikona aplikace.
- [x] Zmenšené informační karty, sbalitelné pravé panely a méně zbytečného scrollování.
- [x] Jasné rozdělení Chat a Agentní režim.
- [x] Respektovat `prefers-reduced-motion`.
- [ ] Ruční vizuální kontrola v zabalené Windows aplikaci na běžném 1080p displeji.

## Fáze 2 — Chat a agentní pracoviště
- Chat: běžná konverzace s lokálním modelem, historie, streaming, stop, opakování, kopírování.
- Agentní režim: jeden agent nebo tým více agentů/modelů; plán, role, průběh a stručné výsledky jednotlivých agentů.
- Přehledně oddělit návrh plánu, čekající schválení, prováděné kroky a výsledek.
- Zápisy souborů, spouštění příkazů a další citlivé akce musí mít náhled, jasný rozsah oprávnění a výslovné schválení uživatele.

## Fáze 3 — poskytovatelé modelů a API klíče
- Zachovat Ollama jako první a plnohodnotnou lokální cestu bez placeného API.
- [x] Obecná konfigurace HTTPS endpointu pro OpenAI-kompatibilní API (OpenAI, OpenRouter, Groq, Together a vlastní endpoint), model a test spojení.
- [ ] Nativní adaptéry pro Anthropic/Claude Messages API a Google/Gemini API včetně jejich odlišného streamování a autentizace.
- API klíče ukládat bezpečně přes Windows Credential Manager / systémový trezor, nikdy do zdrojového kódu, logů ani exportů workspace.
- Zřetelně vysvětlit, že předplatné ChatGPT samo o sobě nezahrnuje kredit pro OpenAI API; použití API se může účtovat zvlášť.
- U vzdálených poskytovatelů zobrazovat, že text opouští lokální počítač; uživatel musí mít možnost zůstat čistě lokální.

## Fáze 4 — pluginy a integrace
- Rozhraní pro vlastní pluginy: manifest, oprávnění, nastavení, zapnutí/vypnutí, diagnostika a protokol akcí.
- Nejdříve bezpečné integrace pro čtení projektu, vyhledávání, správu souborů s potvrzením a exporty.
- Herní a kreativní nástroje: Roblox Studio (plugin MVP v `integrations/roblox-studio/`), Godot, Unity, CapCut; dále VS Code/Visual Studio, GitHub a další běžné vývojářské aplikace.
- Pro každou integraci jasně uvést, co reálně umí: například čtení projektu, diagnostiku logů, tvorbu návrhů změn nebo ovládání přes oficiální API/CLI. Nepředstírat integraci, pokud aplikace neposkytuje podporovaný způsob připojení.
- Preferovat oficiální API, lokální bridge nebo uživatelem schválené skripty. Nepřidávat neomezené automatické ovládání aplikací a her bez kontroly oprávnění.

## Fáze 5 — stabilita a vydání
- Automatické testy, testy bezpečnostních hranic, testy chybových stavů a obnova přerušené práce.
- Ověřit balíček na Windows: spuštění, ikona v taskbaru, minimalizace/maximalizace, DPI, zmenšení okna, animace a snížení zátěže.
- Animace musí mít nízkou zátěž, plynulou smyčku a respektovat systémové omezení pohybu.
- Každé vydání obsahuje changelog, známá omezení a skutečné výsledky testů.

## Pravidla produktu
1. Jednoduchost a čitelnost mají přednost před počtem panelů.
2. Podrobnosti patří do sbalitelných panelů nebo samostatných obrazovek, ne do obřího dashboardu.
3. Žádné falešné statistiky, falešný stav online ani neexistující integrace.
4. Lokální režim zůstává užitečný bez API klíčů a placených služeb.
5. Rozlišovat ověřenou implementaci, automaticky testovaný zdroj a ručně neověřené chování.

## Pokrok v0.5.5
- [x] Volitelný OpenAI API provider v composeru.
- [x] Ukládání API klíče šifrovaně přes Electron `safeStorage`; klíč se nevrací do rendereru.
- [x] Test připojení přes OpenAI `/v1/models` a odstranění uloženého klíče.
- [x] Streamované odpovědi OpenAI přes stejný UI stream jako Ollama.
- [ ] Živě ověřit ukládání klíče a skutečný chat ve Windows; vyžaduje uživatelský API klíč a Windows.


## Pokrok v0.5.6
- [x] API nastavení už není omezené na klíč s prefixem OpenAI; podporuje libovolný klíč pro OpenAI-kompatibilní HTTPS endpoint.
- [x] Ukládá se popisek poskytovatele a bezpečně validovaná adresa endpointu.
- [x] Chat a test `/models` používají nakonfigurovaný endpoint.
- [x] Přidán Roblox Studio plugin MVP: vyčtení skriptů, které uživatel explicitně vybere, do panelu pro předání kontextu Auroře.
- [ ] Automatické bezpečné přenášení návrhů změn do Roblox Studio přes lokální bridge; zatím ruční kopírování.
- [ ] Nativní Claude a Gemini protokoly; současná vzdálená cesta je OpenAI-compatible.
- [ ] Živý test API a Roblox Studio na Windows.


## Pokrok v0.5.7 — agentní workflow primitives
- [x] Přidán samostatný, testovatelný orchestrátor pro 1–4 agentní role (planner, researcher, implementer, reviewer).
- [x] Agenti běží sekvenčně a dostávají omezený kontext předchozích výsledků; události start/completion/failure lze zobrazit v UI.
- [x] Validace rolí/modelů/ID, limity vstupu, zachycení chyb a zrušení přes AbortSignal.
- [ ] Napojit orchestrátor na skutečné provider adaptéry a samostatnou obrazovku Agentní režim; tato verze přidává ověřené jádro, ne hotové UI workflow.
- [ ] Testovat reálné multi-model běhy a obnovení po přerušení na Windows.

## Aktualizace v0.5.8 — čtyřrole Agent Studio
- Agent Studio nyní navazuje čtyři fáze: plánovač, výzkumník, implementátor a recenzent.
- Průběh i chyby jednotlivých rolí se zobrazují ve frontě; výstupy lze předat do běžného chatu.
- Projekt zůstává v tomto pracovním toku pouze pro čtení. Změny souborů ani příkazy se automaticky nespouštějí.
- Zbývá: streamovaný výstup přímo ve fázích, samostatné modely pro výzkumníka a recenzenta, nativní Anthropic/Gemini adaptéry a ruční test UI ve Windows.

## Fáze 6 — adaptivní výkon a optimalizace (nová priorita)
- [x] Profily výkonu: Automaticky, Rychlý, Vyvážený, Důkladný a Ruční.
- [x] Automaticky volit menší kontext a kratší výstup pro jednoduché dotazy, větší rozpočet pro složité programování, analýzu a dlouhá zadání.
- [x] Omezit počet CPU vláken podle zjištěného počtu jader; uložit zvolený profil mezi spuštěními.
- [x] Vysvětlit, jaký profil byl použit, a zobrazit efektivní rozpočet v nastavení odpovědi.
- [ ] Měřit reálnou latenci a tokeny/s pro konkrétní model a zařízení; neplést měření s odhadem.
- [ ] Na základě naměřených výsledků nabídnout doporučení modelu/kvantizace a upozornit na nedostatek RAM/VRAM.
- [x] Omezit zbytečně dlouhou historii konverzace podle profilu výkonu; při vynechání starších zpráv to uživateli oznámit.
- [ ] Omezit kontext webových stránek a připojených projektových dat stejným rozpočtem; přidat bezpečné shrnutí starších částí.
- [ ] Optimalizovat překreslování streamu a animace při generování, ověřit spotřebu CPU/RAM v zabalené Windows aplikaci.
- [ ] U vzdálených providerů zobrazit jen nastavení, která API skutečně podporuje; přidat nativní řízení rozpočtu pro jednotlivé adaptéry.

## Aktualizace v0.5.9 — Adaptive Performance
- Přidán výběr profilu výkonu do nastavení odpovědi.
- Automatický režim klasifikuje jednoduché otázky, běžné úkoly a složité programování/analýzu.
- Profily nastavují omezený kontext, výstupní tokeny a počet CPU vláken; ruční režim respektuje hodnoty uživatele v bezpečných mezích.
- Volba profilu se ukládá a každá odpověď uvádí použitý profil.
- Otevřené: skutečný benchmark na uživatelově hardwaru, optimalizace paměti/streamování a vizuální test Windows.


## Aktualizace v0.6.0 — Bounded Conversation Context
- Profily výkonu nyní omezují počet a objem historických zpráv posílaných modelu.
- Rychlý režim upřednostňuje krátkou nedávnou historii; Důkladný ponechává více kontextu.
- Pokud dojde k vynechání starších zpráv, odpověď to transparentně označí.
- Přidány testy limitů a pořadí historie.
- Otevřené: limit pro externí připojený kontext, měření zrychlení na skutečném zařízení a test zabalené Windows aplikace.


## Nově dokončeno ve v0.6.1
- Lokální telemetrie odezvy: čas do prvního výstupu, celková latence, orientační znaky/s, vybrané modely a profily; maximálně 40 vzorků, bez ukládání obsahu zpráv.
- Profilové limity připojeného kontextu webu a projektů; při zkrácení se uživateli zobrazí upozornění.

## Další výkonové kroky
- Porovnat modely opakovaným benchmarkem za stejných podmínek a zobrazit skutečné tokeny/s z Ollama.
- Omezit připojený kontext z webu/projektu stejnými rozpočty jako historii chatu.
- Profilovat vykreslování a animace a změřit spotřebu paměti při delším běhu.

## v0.6.2 — Směrování podle měření
- [x] Oddělený a testovaný výběr modelu podle typu úkolu.
- [x] Čerstvost benchmarku (30 dní) a ignorování budoucích/neplatných měření.
- [x] Konzervativní heuristika při nízké volné RAM.
- [ ] Přesnější odhad nároků podle velikosti souboru modelu, kvantizace a VRAM.
- [ ] Reálné porovnání více modelů a doporučení podle historie měření.
- [ ] Windows runtime a vizuální ověření.


## Pokrok v0.6.2
- [x] Samostatný testovaný modul pro automatický výběr modelu.
- [x] Jednoduché úkoly směrovat k menším modelům, programování ke specializovaným coder modelům.
- [x] Čerstvost benchmarků a konzervativní filtr podle volné RAM.
- [ ] Přesnější model-fit podle skutečné velikosti na disku, kvantizace a VRAM.
- [ ] Nativní Claude/Gemini adaptéry a plné propojení agentů s různými poskytovateli.

## Aktualizace v0.6.3
- [x] Nativní adaptéry požadavků a streamování pro OpenAI-kompatibilní API, Anthropic Claude a Google Gemini.
- [x] Testy mapování rolí, streamových událostí, chyb a názvů modelů.
- [ ] Automatická detekce protokolu/modelů podle uloženého poskytovatele.
- [ ] Integrovaný agentní workflow přes libovolný vybraný vzdálený/lokální model.
- [ ] Změřit rychlost a paměť na reálném Windows počítači a otestovat vizuální UI.

## Audit v0.6.4 — nálezy a priority
- [x] Opravit test spojení Gemini: nepoužívat streamový endpoint pro ne-streamový požadavek; test nyní používá `:generateContent`.
- [x] Opravit Agent Studio, které nepředávalo vybraného poskytovatele a při vzdáleném režimu tak chybně padalo do větve vzdáleného API bez modelu/konfigurace.
- [x] Přidat testovatelné sestavení testovacích požadavků pro všechny tři protokoly.
- [ ] Přidat živé integrační testy s opt-in API klíči; bez klíčů je ověřená jen konstrukce požadavků a simulované odpovědi.
- [ ] Propojit víceagentní výstup s bezpečným návrhem změn souborů (zatím agenti jen analyzují a navrhují text).
- [ ] Umožnit skutečný výběr modelů/rolí napříč poskytovateli v Agent Studio; nyní vzdálené role používají nakonfigurovaný vzdálený model.
- [ ] Změřit zabalenou aplikaci na Windows, otestovat rozložení, instalaci, bezpečné ukládání klíčů a dlouhé streamy.
- [ ] Přidat sémantickou dlouhodobou paměť pouze jako volitelnou funkci s jasným řízením soukromí.
- [ ] Rozšířit integrace Unity/Roblox/Godot z dílčích workflow na ověřené, verzované konektory; neprezentovat prototyp jako plnou integraci.


## v0.6.5 — další spolehlivost API

- [x] SSE parser pro fragmentované a vícerádkové rámce
- [x] Časový limit vzdáleného generování a limity přenesených dat/výstupu
- [x] Test spojení OpenAI-kompatibilního endpointu ověřuje chat generování
- [ ] Živé ověření všech cloudových protokolů s platnými klíči
- [ ] Windows build, instalace a ruční smoke test
- [ ] Oddělené modely pro jednotlivé agentní role a zrušení celého workflow
- [ ] Změřit RAM/VRAM a rychlost na reálném počítači


## v0.6.6 — koordinace běhů a směrování správných modelů
- [x] Agent Studio má viditelné tlačítko zastavení a předá zrušení aktivnímu požadavku modelu.
- [x] Zabránit souběhu chatu a Agent Studio, který mohl sdílet a přepisovat ID právě běžícího požadavku.
- [x] Při zrušení zachovat dokončené fáze a jasně označit neúplný výsledek.
- [x] Automatický router a Model Hub vyřadí embedding, reranking a speech modely z doporučeného běžného chatování.
- [x] Test OpenAI/Anthropic/Gemini ověřuje skutečný krátký textový výstup, ne jen dostupnost endpointu; chyby mají limit velikosti.
- [x] Lokální benchmark má časový limit, limit velikosti streamu a zpracuje poslední neukončený záznam.
- [ ] Ověřit zastavení na skutečném Ollama procesu v zabalené aplikaci Windows a otestovat dlouhý běh.
- [ ] Přidat individuální výběr poskytovatele/modelu pro každou roli Agent Studio.
- [ ] Dokončit projektovou paměť a bezpečné vytváření/schvalování návrhů změn přímo z agentních výstupů.
- [ ] Provést živé testy cloudových protokolů s dobrovolně dodanými API klíči a ruční smoke test Windows.


## v0.6.7 — spolehlivější vydání
- [x] Aktualizovat Windows build script na verzi 0.6.7.
- [x] Před balením spouštět úplnou kontrolu syntaxe všech modulů přes `npm run check`.
- [x] Před balením spouštět kompletní automatické testy.
- [x] Přidat regresní test, který hlídá aktuální verzi instalátoru a přítomnost obou kontrol.
- [ ] Sestavit NSIS instalátor ve Windows a ručně ho otestovat po čisté instalaci.


## Aktualizace v0.6.8 — vzdálené profily výkonu

- [x] Předat adaptivní profil oficiálnímu OpenAI API pro podporované GPT‑6 modely.
- [x] Mapovat rychlost/hloubku na Gemini `thinkingLevel` pro rozpoznané modely Gemini 3/4.
- [x] Vynechat nekompatibilní Claude `temperature` pro rodiny 4.7+ a 5.
- [x] Zachovat izolaci lokální Ollamy a neodesílat vzdálené parametry do jejího API.
- [ ] Živě ověřit OpenAI / Claude / Gemini s platnými klíči.
- [ ] Ověřit instalační balík na skutečných Windows.
- [ ] Dokončit hlubší paměť a rozšířené konektory pro Unity, Roblox Studio a Godot.


## v0.6.9 — oprava UI a preference
- [x] Opravit HTML strom: všech deset hlavních obrazovek je uvnitř `main.main`.
- [x] Zabalit uživatelem dodané video polární záře do aplikace, bez zvuku, s pomalým přehráváním a zastavením při skrytém okně / omezeném pohybu.
- [x] Přidat lokální preference jazyka čeština/English, motivu, síly pozadí, velikosti písma a přehledu systému.
- [x] Propojit běžné parametry generování a validovat je před předáním lokální Ollamě.
- [x] Rozšířit anglický slovník i o názvy položek v selectech; přidat regresní testy.
- [ ] Na Windows spustit Electron v čistém profilu, ověřit strukturu všech obrazovek, video, stream a stop generování.
- [ ] Sestavit a ručně projít instalátor NSIS.
- [ ] Dokončit plnou lokalizaci dynamických hlášení a živé ověření vzdálených poskytovatelů.


### v0.6.9 — struktura UI, pozadí a nastavení
- [x] Opravit vnoření všech 10 hlavních obrazovek do jednoho aplikačního kontejneru.
- [x] Přibalit dodané video pozadí H.264 1920×1080; 0,82× playback a pozastavení při skrytí/omezení pohybu.
- [x] Přidat lokální preference jazyka cs/en, motivu, intenzity a pohybu pozadí, font scale a běžných parametrů generování.
- [x] Opravit zachování pokročilých nastavení při obnovení aplikace a browser-safe export `agent-config.js`.
- [x] Odstranit chybnou prázdnou značku a rušivý toast při startu bez Ollama.
- [x] `npm run check` + `npm test` 148/148; izolovaný Chromium renderer smoke test s testovacím IPC stubem.
- [ ] Přímý smoke test Electron aplikace a instalátoru na Windows; živé testy Ollama, ComfyUI a vzdálených API.


## Aktualizace v0.7.0 — integrování desktopové aplikace

- [x] Jednotná desktopová vrstva stylů a čistší rozložení hlavní obrazovky.
- [x] Lokální video polární záře jako hlavní pozadí; playback 0,82×, bez zvuku, s respektem k omezení pohybu.
- [x] Kategorie nastavení místo jedné dlouhé stránky.
- [x] Pokročilé parametry propojené se stejným uloženým stavem chatu a modelu.
- [x] Jazyk odpovědí automaticky / čeština / angličtina pro chat a role Agent Studia.
- [ ] Build a instalátor ověřit na Windows; živé lokální a vzdálené modely zatím nejsou v této iteraci potvrzeny.
- [ ] Dokončit plnohodnotné konektory vývojových nástrojů a sémantickou dlouhodobou paměť.

## v0.7.7 — Godot statická diagnostika a integrační základ
- [x] Přidat read-only Godot audit napojený přes zabezpečené Electron IPC.
- [x] Ověřit metadata `project.godot`, hlavní scénu a vybrané `res://` reference v `.tscn` / `.tres`.
- [x] Přidat základní, výslovně heuristická varování pro GDScript a značky TODO.
- [x] Omezit hloubku/počet/velikost skenovaných souborů; nepřecházet symlinky ani nespouštět kód projektu.
- [x] Přidat UI kartu a regresní testy pro audit, IPC a preload.
- [ ] Ověřit audit na reálném projektu Godot 3 i Godot 4 a porovnat nálezy s editorem.
- [ ] Sestavit a otestovat Windows instalátor v čistém profilu.
- [ ] Rozšířit Godot/Unity/Roblox konektory o bezpečné předání kontextu a výslovně schválené změny.
- [ ] Dokončit živé testy Ollama, ComfyUI a jednotlivých vzdálených API providerů.



## v0.8.0 — konsolidace integrací a vydání

- [x] Extrahovat Unity audit do samostatného testovatelného modulu s omezeným čtením a odmítnutím symlinků.
- [x] Přidat kontroly Unity manifestu, chybějících `.meta`, duplicitních GUID a vybraných C# varování (heuristicky).
- [x] Rozšířit Roblox Studio Bridge o schválený zápis do jediného vybraného skriptu s kontrolou původního obsahu a Undo recordingem.
- [x] Zachovat víceskriptové načtení pouze jako kontext; hromadné zápisy a síťový přenos pluginem zůstávají vypnuté.
- [x] Přidat `scripts/release-preflight.js` a předvydávací kontrolu zdrojů, bezpečnostních nastavení Electronu a metadat vydání.
- [x] Zjednodušit seznam souborů přibalených do instalované aplikace na aktuální runtime a dokumentaci.
- [x] Aktualizovat `BUILD-AURORA.bat`, aby před instalátorem spustil kontrolu, testy a preflight a ověřil výsledné `.exe`.
- [ ] Sestavit NSIS instalátor v čistém Windows prostředí a otestovat instalaci, odinstalaci, DPI, minimalizaci a restart.
- [ ] Spustit živé Ollama chaty/stop/benchmark a ComfyUI generování s reálným zvoleným workflow.
- [ ] Ověřit Anthropic, Gemini a OpenAI-kompatibilní endpointy s platnými uživatelskými klíči (bez ukládání klíčů do logů/exportů).
- [ ] Porovnat nálezy Unity/Godot auditu s reálnými editory a ověřit zápis/Undo/Redo Roblox Bridge v testovací place.
- [ ] Otestovat dlouhý agentní běh, zrušení mezi rolemi a chování po chybě v reálné desktopové aplikaci.

- [x] Přidat `npm run smoke:local` pro lokální ověření dostupnosti Ollamy a ComfyUI s loopback-only URL, timeoutem a limitem 2 MB na JSON odpověď.
- [x] Pokrýt runtime smoke helper testovacími HTTP servery (verze/list modelů, ComfyUI statistiky, loopback validace a limity odpovědi).
- [x] Zpevnit Roblox Bridge proti duplicitnímu kliknutí a změně stavu během asynchronního `UpdateSourceAsync`; po selhání Undo záznamu ověřit skutečný stav zdroje.
- [x] Rozlišit příliš velký Unity manifest od poškozeného JSON; přidat cílenou regresní zkoušku.
- [x] `npm run check`, `npm test` (260/260) a `npm run release:preflight` (25/25) prošly.
- [ ] Vygenerovat `package-lock.json` v prostředí s přístupem k npm registry; v tomto pracovním prostředí požadavek skončil chybou DNS `EAI_AGAIN`.

### Co v0.8.0 neznačí jako hotové

Úspěšné zdrojové testy nenahrazují Windows runtime test. Projekt zůstává kandidátem na finální vydání do doby, než vznikne a bude prakticky ověřen instalační `.exe`; lokální servery a cloudová API je nutné otestovat na cílovém počítači.


## Závěrečný integrační průchod — v0.8.0

- [x] Rozšířit Unity audit do samostatného modulu s limity pro soubory a manifest.
- [x] Zpevnit Roblox Bridge proti duplicitnímu schválení během asynchronní aktualizace.
- [x] Přidat loopback-only smoke kontrolu dostupnosti Ollamy a ComfyUI.
- [x] Přidat statický release preflight pro Electron bezpečnost, IPC, balicí manifest a aktuální vydání.
- [x] Ponechat při šířce okna do 900 px historii chatů a filtry viditelné i při sbalené hlavní navigaci; navigace se posouvá samostatně.
- [x] Přiložit český praktický checklist pro sestavení a ruční integrační testy.
- [x] Staticky parsovat oba hlavní CSS soubory bez syntaktických chyb.
- [ ] V prostředí s npm přístupem vytvořit a ověřit lockfile pro reprodukovatelné transitive závislosti.
- [ ] Sestavit a otestovat NSIS instalátor v čistém Windows profilu.
- [ ] Živě ověřit Ollama, ComfyUI, skutečné vzdálené API a Roblox Studio na cílovém PC.


## v0.8.1 — lokální sémantická paměť a robustní zálohy

- [x] Přidat volitelnou lokální vektorovou paměť s ručními poznámkami; standardně vypnuto.
- [x] Připojovat paměťový kontext pouze k požadavkům lokální Ollamy, nikdy automaticky ke cloudovým providerům.
- [x] Podporovat moderní `/api/embed` a fallback na starší `/api/embeddings` pouze při 404/405.
- [x] Přidat validovaný JSON export/import paměti s deduplikací a kontrolou kompatibility modelu i rozměrů embeddingů.
- [x] Zobrazit chybu poškozené konfigurace místo tichého obnovení defaultu a číst index po blocích s limitem.
- [x] Při neúspěšném importu pokusit se vrátit předchozí konfiguraci a jasně hlásit selhání rollbacku.
- [x] `npm run check`, `npm test` (284/284), `npm run release:preflight` (31/31).
- [ ] Na Windows vytvořit a ověřit `package-lock.json` přes npm registry a sestavit NSIS instalátor.
- [ ] Živě ověřit oba Ollama embedding endpointy, chat + stop, ComfyUI generování a cloudové providery.
- [ ] Otestovat sémantickou paměť v reálné aplikaci včetně exportu/importu, změny modelu a vypnutí paměti.


## v0.8.5 — dokončení propojení integrací a Agent Studio

- [x] Zapojit CapCut read-only audit end-to-end přes Electron IPC, preload a Nástroje UI.
- [x] Zapojit pevně přednastavený projektový runner pro Git, Node, Python, .NET, Rust, Go, Godot a Unity včetně časových limitů, omezení výstupu a zrušení.
- [x] Vyžadovat nativní potvrzení před spuštěním kontrol, které mohou načíst/spustit projektové skripty nebo import.
- [x] Přidat lokální výběr a revalidaci Godot/Unity executable.
- [x] Zapojit Unity Editor Bridge přes bezpečný návrh souboru a existující schvalovací workflow.
- [x] Zapojit správu Roblox Bridge pluginu: status, potvrzená instalace/aktualizace a reverzibilní vypnutí.
- [x] Zapojit validované `aurora-changeset` bloky z implementátora do dávkového náhledu; žádný automatický zápis.
- [x] Použít omezenou retry politiku u krátkých testů vzdáleného API; neopakovat streamované generování.
- [x] Sjednotit parser metrik Ollama benchmarku.
- [x] Zahrnout všechny runtime JS moduly do jednotné syntax checker kontroly a synchronizovat release metadata.
- [ ] Uživatel sestaví Windows NSIS instalátor a provede živé testy na vlastním PC; tyto kontroly nejsou započítané do funkčního procenta.

### Záměrná hranice

Volný shell/terminál v agentním chatu není součástí bezpečného rozsahu. Provozní příkazy jsou dostupné pouze jako konkrétní přednastavené kontroly s potvrzením. CapCut zůstává read-only; Unity/Godot build a import vyžadují potvrzení a důvěryhodný projekt.

## v0.8.6 — konsolidované propojení a bezpečné předávání projektů

- [x] Sjednotit hlavní proces, preload a UI pro CapCut, Unity, Godot, Roblox Studio, projektový runner a VS Code.
- [x] Doplnit explicitní výběr VS Code executable a bezpečné otevření schváleného kořene projektu bez `shell`.
- [x] Sjednotit parser agentních změnových návrhů s hlavním write allowlistem.
- [x] Vyžadovat načtení úplného původního i nového obsahu všech souborů před schválením jednotlivé, dávkové nebo agentní změny.
- [x] Odmítnout duplicitní cíle v dávkovém návrhu nezávisle na velikosti písmen.
- [x] Rozšířit release preflight o kontrolu těchto integračních hranic, VS Code a úplného review gate.
- [x] Sjednotit aktuální release metadata na v0.8.6.
- [ ] Na Windows vytvořit NSIS instalátor a ověřit čistou instalaci.
- [ ] Živě ověřit Ollama chat/stop, staré i nové embedding API, ComfyUI generování, cloudové providery, Unity/Godot editory, Roblox Studio Bridge a CapCut draft.

**Procento dokončenosti funkčního rozsahu:** testy a ruční kontrola se do něj nezapočítávají. Zdrojové funkce a konektory jsou implementované v deklarovaném bezpečném rozsahu; zůstává hlavně vytvoření a zabalení cílového Windows artefaktu. Živou kontrolu na uživatelově PC vede uživatel samostatně.

### Odhad implementační dokončenosti v0.8.6

**Přibližně 98 % implementačního rozsahu.** Toto číslo vyjadřuje dokončenost plánovaných uživatelských funkcí a propojení, nikoli počet testů, automatických kontrol ani ručního ověřování. Zbývající přibližná 2 % jsou vytvoření reprodukovatelného Windows release balíku (npm lockfile + skutečný NSIS `.exe`) v prostředí s dostupným npm registry. Tvoje ruční testování na PC je samostatná fáze a do procent se nepočítá.
