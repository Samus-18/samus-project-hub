# Technický audit Aurora AI v0.6.7

## Dodatečné opravy ve v0.6.7
6. **Zastaralý Windows builder:** `BUILD-AURORA.bat` stále uváděl v0.5.6, kontroloval pouze šest starších souborů a vypisoval špatné jméno instalátoru. Nyní používá `npm run check` pro všechny moduly projektu, spouští plnou sadu testů a uvádí `Aurora-AI-Setup-0.6.7.exe`. Přidán test, který ověřuje shodu verze, kontrolních příkazů a názvu instalátoru.
7. **Zastaralá identifikace webového klienta:** User-Agent pro webový výzkum nyní ukazuje AuroraAI/0.6.7.

## Opravy z v0.6.6 zahrnuté v této verzi
1. **Agent Studio bez zrušení:** čtyřfázový workflow mohl běžet dlouho, aniž by měl vlastní tlačítko pro zastavení. Přidána viditelná akce, která zruší aktivní modelový požadavek a zabrání spuštění dalších fází.
2. **Možný souběh chatu a agentů:** oba workflow používaly jedno `state.activeRequestId`; souběžný chat mohl toto ID přepsat a způsobit, že stop zasáhne špatný požadavek. Chat a Agent Studio jsou nyní vzájemně vyloučeny.
3. **Nedokončený výstup při zrušení:** dokončené fáze zůstávají zachovány a rozhraní výslovně označí analýzu jako zastavenou. Zrušení během čtení projektového kontextu se uplatní před začátkem další fáze.
4. **Pomocné modely v automatickém směrování i Model Hubu:** embedding, reranking a speech modely se nemají vybírat jako chatovací modely. Router je filtruje; doporučený filtr je nezapočítá a pomocný Nomic Embed Text nelze omylem aktivovat pro chat.
5. **Test vzdáleného API:** dříve OpenAI větev pouze načetla `/models`, což mohlo potvrdit HTTP spojení bez funkčního generování. Všechny tři protokoly nyní posílají krátký ne-streamovací požadavek a vyžadují neprázdný textový výstup. Chybové odpovědi i benchmark jsou omezeny velikostí; lokální benchmark má 120sekundový timeout a zpracuje finální záznam streamu.

## Testy
- Výchozí sada v0.6.5: 100/100 automatických testů a `npm run check` bez syntaktických chyb.
- Finální sada v0.6.7: 110 automatických testů; `npm run check` bez syntaktických chyb. Automatické testy nenahrazují ruční Windows kontrolu ani živé cloudové API.

## Zbývající chyby a priority
- **Agent Studio stále používá jeden model pro plánovače/výzkumníka a druhý pro implementátora/recenzenta.** Změna poskytovatele za běhu může mít jiné chování; oddělená konfigurace role/provider/model chybí.
- **Zrušení musí potvrdit praktický test.** UI předá abort na aktivní IPC požadavek, ale nebyl ověřen dlouhý živý request Ollama nebo cloudového API v zabalené Windows aplikaci.
- **Model-fit je heuristika.** Router teď filtruje ne-chatovací modely, ale odhad nároků stále nezohledňuje přesnou kvantizaci, offload do GPU a kontext modelu.
- **Vzdálené poskytovatele nelze označit za produkčně ověřené.** Testy ověřují lokální tvorbu požadavků a parsování, ne živé volání s platným klíčem.
- **Instalátor a UI na Windows nejsou prakticky ověřeny.** Je nutné sestavit NSIS balík, spustit jej na Windows a zkontrolovat chování po restartu.
- **Integrace editorů zůstávají neúplné.** Unity část je převážně read-only diagnostika a Roblox Bridge MVP vyžaduje ruční přenos; nejde o hotové automatické konektory pro Unity, Roblox a Godot.
- **Sémantická paměť zatím chybí.** Konverzační historie a export/import jsou k dispozici, ale volitelná lokální vektorová paměť s řízením soukromí není implementována.

## Opatrný odhad dokončenosti
Celkový odhad zůstává přibližně 63 %; opravy v0.6.6 zlepšují běh agentů a směrování modelů, zatímco v0.6.7 se opravuje spolehlivost vývojového vydání. Změna build skriptu sama o sobě nepřináší velký nový uživatelský subsystém. Velké položky — Windows runtime, živé provider testy, plnohodnotné integrace a dlouhodobá paměť — zůstávají otevřené. Toto procento je expertní odhad, nikoli měřená metrika.


## Rozsah této dodatečné kontroly
- Plná Windows sestava nebyla spuštěna, protože v pracovním prostředí není instalovaná Electron závislost/builder. Nový `.bat` teď při spuštění na Windows zkontroluje všechny zdrojové moduly a testy před sestavením NSIS instalátoru.
