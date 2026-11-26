# Technický audit Aurora AI v0.6.6

## Opraveno v této iteraci
1. **Agent Studio bez zrušení:** čtyřfázový workflow mohl běžet dlouho, aniž by měl vlastní tlačítko pro zastavení. Přidána viditelná akce, která zruší aktivní modelový požadavek a zabrání spuštění dalších fází.
2. **Možný souběh chatu a agentů:** oba workflow používaly jedno `state.activeRequestId`; souběžný chat mohl toto ID přepsat a způsobit, že stop zasáhne špatný požadavek. Chat a Agent Studio jsou nyní vzájemně vyloučeny.
3. **Nedokončený výstup při zrušení:** dokončené fáze zůstávají zachovány a rozhraní výslovně označí analýzu jako zastavenou. Zrušení během čtení projektového kontextu se uplatní před začátkem další fáze.
4. **Pomocné modely v automatickém směrování i Model Hubu:** embedding, reranking a speech modely se nemají vybírat jako chatovací modely. Router je filtruje; doporučený filtr je nezapočítá a pomocný Nomic Embed Text nelze omylem aktivovat pro chat.
5. **Test vzdáleného API:** dříve OpenAI větev pouze načetla `/models`, což mohlo potvrdit HTTP spojení bez funkčního generování. Všechny tři protokoly nyní posílají krátký ne-streamovací požadavek a vyžadují neprázdný textový výstup. Chybové odpovědi i benchmark jsou omezeny velikostí; lokální benchmark má 120sekundový timeout a zpracuje finální záznam streamu.

## Testy
- Výchozí sada v0.6.5: 100/100 automatických testů a `npm run check` bez syntaktických chyb.
- Finální sada v0.6.6: 108 automatických testů; `npm run check` bez syntaktických chyb. Automatické testy nenahrazují ruční Windows kontrolu ani živé cloudové API.

## Zbývající chyby a priority
- **Agent Studio stále používá jeden model pro plánovače/výzkumníka a druhý pro implementátora/recenzenta.** Změna poskytovatele za běhu může mít jiné chování; oddělená konfigurace role/provider/model chybí.
- **Zrušení musí potvrdit praktický test.** UI předá abort na aktivní IPC požadavek, ale nebyl ověřen dlouhý živý request Ollama nebo cloudového API v zabalené Windows aplikaci.
- **Model-fit je heuristika.** Router teď filtruje ne-chatovací modely, ale odhad nároků stále nezohledňuje přesnou kvantizaci, offload do GPU a kontext modelu.
- **Vzdálené poskytovatele nelze označit za produkčně ověřené.** Testy ověřují lokální tvorbu požadavků a parsování, ne živé volání s platným klíčem.
- **Instalátor a UI na Windows nejsou prakticky ověřeny.** Je nutné sestavit NSIS balík, spustit jej na Windows a zkontrolovat chování po restartu.
- **Integrace editorů zůstávají neúplné.** Unity část je převážně read-only diagnostika a Roblox Bridge MVP vyžaduje ruční přenos; nejde o hotové automatické konektory pro Unity, Roblox a Godot.
- **Sémantická paměť zatím chybí.** Konverzační historie a export/import jsou k dispozici, ale volitelná lokální vektorová paměť s řízením soukromí není implementována.

## Opatrný odhad dokončenosti
Celkový odhad se posouvá jen mírně z 61 % na přibližně 63 %: opraveno je rušení agentního běhu a nesprávný výběr některých modelů. Velké položky — Windows runtime, živé provider testy, plnohodnotné integrace a dlouhodobá paměť — zůstávají otevřené. Toto procento je expertní odhad, nikoli měřená metrika.
