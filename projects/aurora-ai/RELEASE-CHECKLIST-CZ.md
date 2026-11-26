# Aurora AI v0.8.6 — kontrolní seznam pro Windows

Tento seznam slouží pro první sestavení a provozní ověření. Zdrojový balík není hotový instalátor; `.exe` musí být sestaveno na Windows.

## 1. Sestavení

1. Rozbal ZIP do vlastní složky (ne do `Program Files` a ne přes starou běžící instalaci).
2. Nainstaluj Node.js 22.12.0 nebo novější a ověř, že fungují příkazy `node --version` a `npm --version`.
3. Dvakrát klikni na `BUILD-AURORA.bat`. Skript stáhne závislosti, spustí kontrolu syntaxe, testy, release preflight a potom vytvoří NSIS instalátor.
4. Pokračuj jen pokud skript skončí stavem `SUCCESS` a soubor `dist/Aurora-AI-Setup-0.8.6.exe` skutečně existuje. Zaznamenej SHA-256 z přiloženého souboru vedle instalátoru.
5. Nainstaluj nejprve do samostatného testovacího profilu. Před migrací důležitých dat měj vlastní export workspace.

## 2. První spuštění aplikace

- Otevři/zavři aplikaci a ověř, že si zachová jazyk, vzhled, jméno profilu, vlastní instrukce a nastavení zvuků.
- Zmenši okno pod šířku 900 px. Ověř, že zůstávají dostupné přepínač Chat/Work (oba režimy se stejným chatem), Nová konverzace, vyhledávání, Uložené, Projekty, Nedávné/Archiv a profil.
- Přepni Chat → Work → Chat a ověř, že zůstane otevřená stejná konverzace, stejný seznam zpráv i composer. Ve Work se navíc objeví jen výběr projektu; po odeslání pracovního úkolu se průběh zobrazí uvnitř zprávy asistenta, nikoli na odděleném dashboardu.
- Otevři podsekce v levé navigaci; ověř, že kategorie Nastavení otevře právě vybranou oblast a nikde nevznikne druhá duplicitní lišta.
- Vytvoř dva chaty, jeden připni, druhý přiřaď k projektu; otestuj přejmenování, archivaci, smazání a odebrání z Uložených. Vytvoř dva samostatné projekty s odlišnými složkami, přepínej mezi nimi, přejmenuj jeden a odeber jej; ověř, že složka a soubory na disku zůstaly. Exportuj a importuj zálohu a ověř zachování chatů a ID vazeb bez přenosu absolutních cest.
- Otevři průvodce tlačítkem `?`; ověř zavření tlačítkem, kliknutím na pozadí a klávesou Escape.

## 3. Lokální modely a obrázky

1. Spusť Ollamu obvyklým způsobem a ověř, že má stažený alespoň jeden chatovací model.
2. Spusť ComfyUI a otevři Nastavení → API/poskytovatelé či Nástroje a integrace.
3. V adresáři projektu spusť `npm run smoke:local`. Skript zkontroluje pouze loopback adresy `localhost`/`127.0.0.1`, vypíše verzi Ollamy a seznam modelů a zkontroluje stav ComfyUI. Nespouští generování.
4. V samotné Auroře odešli krátký chat, přeruš ho tlačítkem stop, restartuj Ollamu a ověř viditelný offline/online stav.
5. Vygeneruj jeden obrázek známým workflow; ověř pozitivní i negativní prompt, průběh, chybu při odpojeném ComfyUI a opětovné spuštění po návratu služby.

## 4. Poskytovatelé a klíče

- Ověř pouze poskytovatele, kterého sám nakonfiguruješ, se svým vlastním klíčem.
- Otestuj neplatný klíč, HTTP 401/403, limit 429, timeout, stop a opětovné spojení.
- Ověř, že klíče nejsou součástí exportu workspace a že aplikace hlásí, zda je klíč uložen přes `safeStorage`.
- Pokud `safeStorage` vrátí dostupnost ochrany jako nevyhovující, nepředpokládej, že je tajemství stejně chráněné jako v běžném profilu Windows.

## 5. Agent Studio a projektové zápisy

- Začni v kopii testovacího projektu, ne v jediném pracovním originálu.
- Ověř dvě role a teprve potom pokročilý víceagentní režim; vyzkoušej zastavení, selhání jednoho agenta a obnovení dalšího kroku.
- Před hromadným zápisem ověř náhled diffu, souhrn změn a explicitní schválení. Dvakrát rychle neschvaluj stejný návrh; ověř, že zrušení během probíhajícího zápisu hlásí stav `busy`.
- Vyzkoušej Unity a Godot audit na kopii projektu a porovnej varování s příslušným editorem. Statické varování je heuristika, ne důkaz chyby při kompilaci.
- V Roblox Studio nainstaluj plugin podle `integrations/roblox-studio/README-CZ.md`; vyber jediný testovací skript, připrav změnu, prohlédni náhled a teprve druhým kliknutím schval zápis. Ověř Undo/Redo v testovací kopii place.

## 6. Lokální dlouhodobá paměť

- Paměť je standardně vypnutá. Před použitím stáhni embedding model (např. `ollama pull nomic-embed-text`) a použij tlačítko **Ověřit embedding model**.
- Ulož jen poznámky, které chceš vědomě uchovat. Otestuj sémantické vybavení při chatu s lokální Ollamou; po přepnutí na cloudového poskytovatele se paměť do požadavku nepřipojuje.
- Pro zálohu použij **Exportovat poznámky** a v testovacím profilu ověř **Importovat poznámky**. JSON záloha obsahuje texty i vektory; uchovávej ji jako soukromý soubor.
- Přepnutí embedding modelu s existujícími poznámkami vyžaduje export, smazání a následný import / přepočítání poznámek podle modelu.

## 7. Hlášení chyby

Ke každému problému poznamenej: verzi Aurory, krok, očekávaný výsledek, skutečný výsledek a log bez klíčů/tajemství. Nepřikládej `.env`, API klíče, celé složky osobních projektů ani syrové zálohy workspace.

## Co tato kontrola sama neprokazuje

Automatické testy a `npm run release:preflight` nekontrolují živé cloudové API, běžící Ollamu/ComfyUI, skutečné Unity/Godot projekty ani Roblox Studio. Za ověřené považuj jen kroky, které jsi skutečně spustil na cílovém počítači.


## 7. Integrační nástroje v0.8.6

- [ ] Vyber složku CapCut projektu a spusť read-only audit `draft_content.json` / `draft_info.json`. Ověř, že zdrojové soubory zůstaly beze změny.
- [ ] Ve Unity kartě připrav Editor Bridge; zkontroluj návrh `Assets/Editor/AuroraAI/AuroraEditorBridge.cs` a schval ho až po kontrole.
- [ ] Ve vybraném Unity projektu nastav editor executable a otestuj nejprve `unity-scene-audit`; build Windows hry spouštěj jen na kopii projektu.
- [ ] Ve vybraném Godot projektu nastav editor executable a otestuj import-check nejprve na kopii projektu.
- [ ] V Roblox Studio kartě ověř cestu pluginu, nainstaluj Aurora Bridge po potvrzení, restartuj Studio a v testovací place ověř plugin. Poté vyzkoušej jeho vypnutí a návrat.
- [ ] V Agent Studiu požádej implementátora o strukturovaný `aurora-changeset`, zkontroluj každý soubor a teprve pak použij schválení dávky.
- [ ] Spusť pouze kontroly odpovídající projektu; akce, které spouštějí projektové skripty/import, vyžadují nativní potvrzení.


## 8. Dokončení integrací v0.8.6

- [ ] Otevři vybranou projektovou složku ve zvoleném Visual Studio Code a ověř, že se neotevřel jiný kořen.
- [ ] Připrav Agent Studio změnový návrh s více soubory. U každého souboru klikni na načtení úplného obsahu, projdi původní i nový text a až potom schval.
- [ ] Ověř, že druhý pokus o schválení stejného návrhu je odmítnut a duplicitní cesty se stejným názvem s jinou velikostí písmen nejsou přijaty.
- [ ] V Godotu/Unity ověř nejprve statický audit; import/build spusť pouze na kopii projektu a jen po výslovném potvrzení.
- [ ] V Roblox Studiu ověř Bridge na testovací place, Undo/Redo a vypnutí pluginu.
- [ ] U CapCut ověř read-only audit na kopii draftu a zkontroluj, že žádný zdrojový soubor nebyl změněn.


## 9. Kompatibilita počítače

- [ ] Otevři Nástroje → Nástroje na tomto počítači a klikni na Zkontrolovat znovu.
- [ ] Ověř, že OS, architektura, Electron, Chromium a vložený Node.js mají hodnoty.
- [ ] Porovnej Node.js, Python, Git, .NET SDK, Java/JDK, Go, Rust, VS Code, Godot, Blender, Unity Editor a Roblox Studio s reálnými instalacemi.
- [ ] Ověř, že volitelné chybějící nástroje neblokují běžný chat.
- [ ] Ověř `Neověřeno` u instalace, jejíž spustitelný soubor existuje, ale příkaz pro verzi není podporovaný.
- [ ] Zkus znovu kontrolu po změně PATH nebo instalaci nástroje. Kontrola nic neinstaluje a nespouští obsah projektů.


## 10. Finální chat a projektové prostory

- [ ] Ověř, že Uložené obsahují pouze připnuté konverzace a po odebrání z Uložených chat zůstane v Nedávných.
- [ ] V nabídce `⋯` ověř připnutí/odepnutí, přejmenování, přesun do projektu, archivaci a smazání.
- [ ] Vytvoř více projektů s vlastními složkami; ověř, že přepnutí projektu nepřepíše aktivní složku jiného projektu.
- [ ] Odeber testovací projekt a potvrď, že se odstraní pouze položka v Auroře, nikoli skutečná složka nebo soubory.
- [ ] Otevři pět náhledů v `previews/final/`. Jsou to statické Chromium náhledy aktuálního HTML/CSS s demonstračními daty; skutečný běh ověř v Electronu.


## 8. Sjednocené Chat / Work rozhraní

- [ ] Ověř, že horní přepínač Chat / Work je viditelný v obou stavech a oba taby jsou ovladatelné.
- [ ] Přepnutí z Projekty/Nastavení do Chat nebo Work vrátí stejnou aktivní konverzaci.
- [ ] Ověř, že se mezi režimy nemění levý panel, zprávy ani composer; Work pouze zobrazí výběr projektu.
- [ ] Vyber projekt a spusť pracovní úkol; průběh a případný návrh změn se musí objevit jako součást zprávy asistenta.
- [ ] Obnov aplikaci a otevři starší konverzaci; má se obnovit její vlastní režim Chat/Work.
- [ ] Ověř, že starý Agent dashboard není viditelný ani dosažitelný jako samostatná stránka.


## 11. UI sidebar, Connect a první chat

- [ ] Klikni na tlačítko sbalení sidebaru; ověř, že postranní panel zmizí, hlavní chat roztáhne šířku a tlačítko zůstane dostupné. Klikni znovu a obnov sidebar.
- [ ] Přepínej Chat / Work; ověř, že přepínač je oddělen od Start/Stop a že se zachová stejná konverzace.
- [ ] V Work composeru zkontroluj, že je pouze jeden výběr projektu, sponka připojuje soubory a nový projekt přidáváš v levém panelu.
- [ ] V Rychlých akcích otevři Connect. Unity, Godot, Roblox Studio a VS Code mají otevřít existující integrační obrazovky; Unreal je pouze plánovaný a nesmí být označen jako připojený.
- [ ] Otevři čistou novou konverzaci; uvítání musí být vycentrované. Ověř i skrytí uvítání po první zprávě.
- [ ] Statické náhledy v `previews/current/` slouží jako referenční HTML/CSS render; skutečné klikací chování potvrď v Electronu na Windows.

Aktuální automatické kontroly: **391/391 testů**, **42/42 syntaxe**, **43/43 release preflight**.
