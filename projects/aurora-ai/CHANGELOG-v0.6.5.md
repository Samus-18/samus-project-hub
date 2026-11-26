# Aurora AI v0.6.5 — Remote streaming reliability

## Změny
- Vzdálený stream používá samostatný SSE dekodér, který správně zpracuje rozdělené síťové bloky, CRLF, vícerádková `data:` pole a poslední neukončený rámec.
- Vzdálené generování má časový limit 180 sekund a bezpečnostní limity 12 MB přijatých streamových dat a 8 milionů znaků výstupu.
- Chyba nebo příliš velký stream se zobrazí jako chyba požadavku místo neomezeného čekání či růstu paměti.
- Test spojení OpenAI-kompatibilního API nyní provede krátký ne-streamovací požadavek na chat endpoint; samotný seznam modelů už nestačí k potvrzení, že model umí generovat odpovědi.
- Přidán test fragmentovaného SSE streamu a ověření limitů.

## Ověření a omezení
- `npm test` a `npm run check` jsou součástí kontroly vydání.
- Neprovádí se živé volání cloudových API bez uživatelova klíče. Limity chrání před nekonečným čekáním a neomezenou odpovědí, nejsou benchmarkem rychlosti.
- Electron aplikace nebyla v tomto prostředí vizuálně spuštěna na Windows.
