# Aurora AI v0.6.6 — Agent control and safer model routing

## Změny
- Agent Studio má viditelné tlačítko „Zastavit analýzu“. Zastavení předá požadavek aktivnímu běhu modelu přes existující bezpečný IPC handler.
- V době běhu agentní analýzy je odesílání chatu pozastaveno; Agent Studio se také nespustí při aktivním chatu. Tím se předchází přepsání sdíleného `activeRequestId` a chybnému stavu zrušení.
- Dokončené agentní fáze zůstávají viditelné po zrušení a výsledek je označen jako zastavený. Tato analýza je stále pouze pro čtení: nespouští příkazy ani nepíše soubory.
- Model router a Model Hub filtrují embeddingové, rerankingové a speech modely z doporučených kandidátů pro konverzaci. Pomocný model Nomic Embed Text už nelze omylem nastavit jako chatovací model. Pokud jsou k dispozici pouze pomocné modely, Aurora zobrazí srozumitelný důvod místo nesprávného doporučení.
- Test vzdáleného API pro OpenAI, Anthropic i Gemini nyní odešle krátký ne-streamovací požadavek na generování a ověří, že tělo odpovědi obsahuje text. OpenAI už nekončí pouhou kontrolou endpointu `/models`.
- Chybové odpovědi Ollama se čtou s limitem dat; lokální benchmark má 120s timeout, limit 2 MB a zpracování posledního streamového záznamu.
- Přidány regresní testy pro zastavení Agent Studio a směrování pouze na chatovací modely.

## Ověření
- `npm test`
- `npm run check`

Nebyla provedena živá zkouška dlouhého agentního běhu na uživatelově Windows ani test s platnými cloudovými API klíči.
