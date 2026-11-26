(function (root) {
  'use strict';
  const STORAGE_KEY = 'aurora.ui.preferences.v1';
  const VIDEO_MIGRATION_KEY = 'aurora.ui.video-background.migrated.v4';
  const DEFAULTS = Object.freeze({
    locale: 'cs', theme: 'aurora', backgroundMotion: true, backgroundIntensity: 0.5,
    reduceMotion: false, fontScale: 1, showDashboard: false, navigationHidden: false, sidebarCollapsed: false, interfaceEffects: true, soundEffects: false, soundVolume: 0.25
  });
  const EN = Object.freeze({
    'Aurora AI':'Aurora AI','AURORA':'AURORA','OVĚŘUJI LOKÁLNÍ SLUŽBY':'CHECKING LOCAL SERVICES','PŘIPRAVENA · LOKÁLNÍ REŽIM':'READY · LOCAL MODE','OLLAMA OFFLINE · KLIKNUTÍM OTEVŘI KONZOLI':'OLLAMA OFFLINE · CLICK TO OPEN QUICK COMMANDS','OLLAMA OFFLINE':'OLLAMA OFFLINE','PŘEMÝŠLÍM':'THINKING','PROHLEDÁVÁM WEB':'SEARCHING THE WEB','VIZUÁLNÍ SYNTÉZA':'IMAGE GENERATION','POTŘEBUJE NASTAVIT NÁSTROJ':'TOOL SETUP REQUIRED','STAHUJI MODEL':'DOWNLOADING MODEL','ÚKOL SE NEPOVEDL':'TASK FAILED','PŘIPRAVENA · VYBER MODEL':'READY · CHOOSE A MODEL','PŘIPRAVENA · VZDÁLENÉ API':'READY · REMOTE API','Lokální AI asistentka':'Local AI assistant','Spusť Ollama a zkontroluj její lokální službu na 127.0.0.1:11434.':'Start Ollama and check its local service at 127.0.0.1:11434.','Ollama není dostupná. Zkontroluj, že služba běží.':'Ollama is unavailable. Check that the service is running.','Lokální asistentka':'Local assistant',
    'Nová konverzace':'New conversation','Chat':'Chat','Agentní režim':'Agent Studio','Agentní':'Agent','pracoviště':'workspace',
    'Projekty':'Projects','Paměť':'Memory','Nástroje':'Tools','Modely':'Models','Galerie':'Gallery','Nastavení':'Settings','Connect':'Connect','Připojení nástrojů':'Tool connections','Vývojové nástroje':'Development tools','PODPOROVÁNO':'SUPPORTED','PŘIPRAVUJE SE':'IN DEVELOPMENT','AUDIT DOSTUPNÝ':'AUDIT AVAILABLE','PLUGIN BRIDGE':'PLUGIN BRIDGE','Nastavit Unity':'Set up Unity','Zobrazit informace':'Show details','Nastavit Godot':'Set up Godot','Spravovat Roblox Bridge':'Manage Roblox Bridge','Nastavit VS Code':'Set up VS Code','Otevřít pokročilé nástroje':'Open advanced tools','Připojit Unity, Unreal, Godot, Roblox a další nástroje':'Connect Unity, Unreal, Godot, Roblox, and more tools','Propoj Auroru s editory a herními nástroji. Vyber projekt, zobraz dostupné funkce a připojení nastav bezpečně.':'Connect Aurora to editors and game tools. Choose a project, review available capabilities, and configure connections safely.','Stavy rozlišují hotové integrace od konektorů, které ještě nejsou implementované.':'Statuses distinguish implemented integrations from connectors that are not available yet.','Samostatný Unreal Editor Bridge zatím není součástí této verze. Nezobrazuji falešný stav připojení ani neinstaluji neexistující plugin.':'A dedicated Unreal Editor Bridge is not included in this version. Aurora will not show a fake connected status or install a non-existent plugin.','Unreal konektor bude vyžadovat vlastní plugin do Unreal Editoru a lokální bezpečný most do Aurory. V této verzi se nepovažuje za připojený.':'The Unreal connector will require its own Unreal Editor plugin and a secure local bridge to Aurora. It is not considered connected in this version.','Bezpečnost připojení':'Connection security','Každý konektor má vlastní oprávnění. Výběr projektu pouze připojí vybranou složku; zápis nebo spuštění kontrol vyžaduje samostatné schválení.':'Each connector has its own permissions. Selecting a project only attaches the selected folder; writing files or running checks requires separate approval.','Skrýt postranní panel':'Hide sidebar','Zobrazit postranní panel':'Show sidebar','ULOŽENÉ':'SAVED','NEDÁVNÉ':'RECENTS','Nová konverzace':'New chat','Možnosti chatu':'Chat options','Projekt':'Project','Vybrat projekt…':'Choose project…','Importováno':'Imported','Exportováno':'Exported','Ověřit embedding model':'Check embedding model','Importovat poznámky':'Import memory notes','Exportovat poznámky':'Export memory notes',
    'PRACOVNÍ PROSTORY':'WORKSPACES','Výchozí workspace':'Default workspace','NEDÁVNÉ':'RECENTS','CHATY':'CHATS',
    'Kontroluji Ollama…':'Checking Ollama…','Lokální profil':'Local profile','Zjišťuji hardware…':'Detecting hardware…',
    'WORKSPACE':'WORKSPACE','Nová konverzace':'New conversation','TVŮJ LOKÁLNÍ AI ASISTENT':'YOUR LOCAL AI ASSISTANT',
    'Ahoj, jsem':'Hello, I’m','Aurora':'Aurora','Tvá lokální AI asistentka. S čím ti dnes pomohu?':'Your local AI assistant. What can I help with today?',
    'Nový chat':'New chat','Analýza projektu':'Project analysis','Vytvořit obrázek':'Create image','Hledat na internetu':'Search the web','Více nástrojů':'More tools',
    'Přidat kontext':'Add context','Vyber model':'Choose model','Poskytovatel':'Provider','Lokální Ollama':'Local Ollama',
    'OpenAI / kompatibilní API':'OpenAI / compatible API','Anthropic Claude':'Anthropic Claude','Google Gemini':'Google Gemini','Auto model':'Auto model',
    '⚙ Nastavení odpovědi':'⚙ Response settings','● SOUKROMÉ':'● PRIVATE','Výkon modelu':'Model performance',
    'Automaticky podle úkolu':'Adaptive by task','Rychlý · jednoduché dotazy':'Fast · simple questions','Vyvážený · běžná práce':'Balanced · everyday work',
    'Důkladný · složité úkoly':'Deep · complex tasks','Ruční · moje hodnoty':'Manual · custom values','Aurora zvolí úroveň podle složitosti zadání.':'Aurora adjusts the level to task complexity.',
    'Kreativita':'Creativity','Kontext':'Context','2 048 tokenů · rychlejší':'2,048 tokens · faster','4 096 tokenů · vyvážené':'4,096 tokens · balanced',
    '8 192 tokenů · delší úkoly':'8,192 tokens · longer tasks','16 384 tokenů · náročnější RAM':'16,384 tokens · more RAM',
    'Délka odpovědi':'Response length','Krátká · 512':'Short · 512','Normální · 1 024':'Normal · 1,024','Dlouhá · 2 048':'Long · 2,048','Velmi dlouhá · 4 096':'Very long · 4,096',
    'CPU vlákna':'CPU threads','2 · nižší zátěž':'2 · lower load','4 · vyvážené':'4 · balanced','6 · vyšší výkon':'6 · faster','8 · maximální zátěž':'8 · maximum load',
    'Vyšší kontext a delší odpovědi mohou spotřebovat více RAM a zpomalit model.':'Larger context and longer answers use more RAM and may slow the model.',
    'Vyber lokální model v horní liště.':'Choose a local model in the toolbar.','Aurora může chybovat. Důležité změny souborů a příkazy budou vyžadovat tvoje schválení.':'Aurora can make mistakes. Important file changes and commands require your approval.',
    'Stav systému':'System status','Celá Aurora AI':'Aurora AI overall','Základ aplikace a chat':'App foundation and chat','Ověřuji Ollama…':'Checking Ollama…','Lokální model se načítá':'Loading local model',
    'Kontrolní seznam':'Checklist','Chat a Ollama integrace':'Chat and Ollama integration','Průběžné odpovědi':'Streaming responses','Automatické testy':'Automated tests',
    'Ruční test chatu ve Windows':'Manual Windows chat test','Ověření chybových stavů':'Error-state verification','Zobrazit diagnostiku':'Open diagnostics',
    'Rychlé akce':'Quick actions','Otevřít projekt':'Open project','Diagnostika':'Diagnostics','AUTONOMNÍ PRACOVNÍ TOK':'AGENT WORKFLOW',
    'Čtyři navazující role: plánovač, výzkumník, implementátor a recenzent. Výsledky se kontrolují; soubory zůstávají beze změn.':'Four sequential roles: planner, researcher, implementer, and reviewer. Results are checked; files remain unchanged.',
    'ROLE 01':'ROLE 01','PLÁNOVAČ':'PLANNER','Planner':'Planner','Rozdělí zadání na ověřitelné kroky, priority a rizika.':'Breaks the task into verifiable steps, priorities, and risks.',
    'Lokální model':'Local model','Nejdřív načti modely':'Load models first','Vzdálený model':'Remote model','Lokální zpracování přes Ollama.':'Local processing through Ollama.',
    'ROLE 02':'ROLE 02','VÝZKUMNÍK':'RESEARCHER','Researcher':'Researcher','Prověří kontext, hledá rozpory a označí neznámé informace.':'Checks context, looks for contradictions, and flags unknowns.',
    'ROLE 03':'ROLE 03','IMPLEMENTÁTOR':'IMPLEMENTER','Implementer':'Implementer','Navrhne konkrétní změny, dotčené soubory a potřebné testy; nic sám nezapisuje.':'Proposes concrete changes, affected files, and tests; never writes changes itself.',
    'ROLE 04':'ROLE 04','RECENZENT':'REVIEWER','Reviewer':'Reviewer','Nezávisle zkontroluje návrh, bezpečnostní rizika a kvalitu testů.':'Independently reviews the proposal, safety risks, and test quality.',
    'Bezpečnostní hranice':'Safety boundaries','STRICT':'STRICT','Přístup pouze k vybrané složce':'Access only to the selected folder','Zatím není vybrána žádná složka':'No folder selected yet','Vybrat složku':'Choose folder',
    'Spouštět příkazy automaticky':'Run commands automatically','ZAMČENO':'LOCKED','Agent může číst jen zvolený projekt. Návrhy nemění soubory a systémové příkazy se nespouštějí.':'The agent can only read the selected project. Proposals do not change files, and system commands are not executed.',
    '＋ Spustit čtyři role':'＋ Run four roles','■ Zastavit analýzu':'■ Stop analysis','Pracovní fronta':'Work queue','PŘIPRAVENO':'READY','Zatím žádné úkoly':'No tasks yet','Vyber složku, napiš cíl a přiřaď modely jednotlivým rolím.':'Choose a folder, describe the goal, and assign models to each role.',
    'LOKÁLNÍ MODEL MANAGEMENT':'LOCAL MODEL MANAGEMENT','Model':'Model','Hub':'Hub','Najdi, stáhni a spravuj modely přímo přes Ollama.':'Find, download, and manage models through Ollama.',
    'Analyzuji hardware…':'Analyzing hardware…','Modely budeme doporučovat podle dostupné paměti a výkonu.':'Model recommendations use available memory and performance.',
    'Znovu zjistit':'Refresh detection','Všechny modely':'All models','Nainstalované':'Installed','Doporučené pro můj PC':'Recommended for my PC','↻ Obnovit':'↻ Refresh',
    'Veřejná knihovna ↗':'Public library ↗','＋ Stáhnout podle názvu':'＋ Download by name','Modely v knihovně':'Library models','Nainstalované lokálně':'Installed locally',
    'Odhad vhodných pro PC':'Estimated fit for this PC','Lokální knihovna':'Local library','Modely zůstávají na tvém zařízení v Ollama.':'Models stay on your device in Ollama.',
    'Stahuji model…':'Downloading model…','Připojuji se k Ollama.':'Connecting to Ollama.','LOCAL RUNTIME TELEMETRY':'LOCAL RUNTIME TELEMETRY',
    'Výkon':'Performance','& runtime':'& runtime','Výkon & runtime':'Performance & runtime','Aktuální stav Ollama, modely načtené v paměti a dostupné systémové prostředky. Data se čtou lokálně z Ollama a Windows.':'Current Ollama status, loaded models, and available system resources. Data is read locally from Ollama and Windows.',
    'Čekám na první kontrolu…':'Waiting for the first check…','Runtime zatím nebyl načten.':'Runtime data has not been loaded yet.','↻ Obnovit runtime':'↻ Refresh runtime',
    'Volná systémová RAM':'Free system RAM','Nainstalované modely':'Installed models','V místní knihovně Ollama':'In the local Ollama library','Načtené modely':'Loaded models','Aktuálně drží paměť':'Currently occupying memory',
    'Součet velikostí načtených modelů':'Total size of loaded models','Odhad Ollama, ne přesné využití RAM':'Ollama estimate, not exact RAM usage','Modely v paměti':'Models in memory','OLLAMA /API/PS':'OLLAMA /API/PS',
    'Klikni na Obnovit runtime.':'Click Refresh runtime.','Naměřená odezva Aurory':'Aurora response measurements','LOKÁLNÍ STATISTIKY':'LOCAL STATISTICS',
    'Měří se čas do prvního výstupu a celková doba odpovědi. Rychlost znaků za sekundu je orientační, nikoli skutečný počet tokenů.':'Time to first output and total response time are measured. Characters per second is an estimate, not a true token rate.',
    'Dokončené odpovědi':'Completed responses','Posledních 40 měření':'Last 40 measurements','Průměrná doba':'Average duration','Od požadavku po konec':'From request to completion',
    'První výstup':'First output','Průměrná odezva':'Average latency','Výstupní rychlost':'Output rate','Znaky za sekundu':'Characters per second','Po dokončení chatu se zde zobrazí první měření.':'The first measurement appears here after a chat completes.',
    'Vymazat měření':'Clear measurements','Jak Aurora vybírá model':'How Aurora selects a model','INTEGRATION REGISTRY':'INTEGRATION REGISTRY','& integrace':'& integrations',
    'Godot diagnostika':'Godot diagnostics','Read-only kontrola projektu: hlavní scéna, odkazy v .tscn/.tres a základní upozornění ve skriptech GDScript. Godot ani kód se nespouštějí.':'Read-only project check: main scene, references in .tscn/.tres, and basic GDScript warnings. Neither Godot nor project code is executed.','ČEKÁ NA PROJEKT':'WAITING FOR PROJECT','Provést Godot audit':'Run Godot audit','Vyber kořen projektu obsahující project.godot a spusť kontrolu.':'Choose the project root containing project.godot, then run the audit.','Statická kontrola nenahrazuje import projektu ani editorové chyby.':'Static checks do not replace project import or editor diagnostics.','Kontroluji Godot projekt…':'Checking Godot project…','NENÍ GODOT PROJEKT':'NOT A GODOT PROJECT','AUDIT: CHYBY':'AUDIT: ERRORS','AUDIT: VAROVÁNÍ':'AUDIT: WARNINGS','AUDIT DOKONČEN':'AUDIT COMPLETE',
    'Centrální přehled funkcí Aurory. Stav se zjišťuje z lokálních služeb; položky vyžadující nastavení jsou tak výslovně označené.':'Central overview of Aurora features. Status is checked against local services; items that require setup are clearly marked.',
    'Stav ještě nebyl ověřen':'Status has not been verified yet','Lokální funkce nevyžadují placený API klíč.':'Local features do not require a paid API key.','↻ Zkontrolovat vše':'↻ Check all',
    'KONTROLUJI':'CHECKING','Ollama · lokální AI':'Ollama · local AI','Chat, streaming, zastavení generování, správa a stahování modelů.':'Chat, streaming, generation cancellation, model management, and downloads.',
    'LIVE WEB SEARCH':'LIVE WEB SEARCH','Internet':'Web','Search':'Search','Vyhledej dokumentaci, řešení chyb a aktuální informace. Vyhledávání používá internet; dotaz se odešle vyhledávači.':'Find documentation, troubleshooting, and current information. Web search sends your query to a search provider.',
    'Hledat ↗':'Search ↗','Výsledky se otevřou v zabezpečeném externím prohlížeči.':'Results open in the secure external browser.','LOCAL IMAGE GENERATION':'LOCAL IMAGE GENERATION',
    'Vizuální':'Visual','engine':'engine','Tuto stránku otevři jen při prvním nastavení nebo změně ComfyUI workflow. Potom bude Aurora generovat obrázky přímo v chatu a nastavení si zapamatuje.':'Open this page only to set up or change the ComfyUI workflow. Afterwards, Aurora can generate images directly in chat and remember the setup.',
    'Kontroluji ComfyUI…':'Checking ComfyUI…','Lokální server 127.0.0.1:8188':'Local server 127.0.0.1:8188','Zkontrolovat':'Check','Popis obrázku':'Image prompt','Negativní prompt (volitelné)':'Negative prompt (optional)',
    'Nastavit / změnit workflow (.json)':'Set / change workflow (.json)','Workflow není vybraný':'No workflow selected','✧ Generovat obrázek':'✧ Generate image',
    'OUTPUT CANVAS':'OUTPUT CANVAS','Výsledek generování':'Generation result','Připraveno':'Ready','Tvůj další obrázek začíná tady':'Your next image starts here','Vyber workflow, napiš prompt a spusť generování.':'Choose a workflow, enter a prompt, and start generation.',
    'Poslední generování':'Recent generations','Vymazat historii':'Clear history','Zatím žádná historie.':'No history yet.','TVŮJ VÝVOJOVÝ PROSTOR':'YOUR DEVELOPMENT WORKSPACE',
    'Bezpečný základ pro Unity, repozitáře a lokální pracovní složky.':'A safe foundation for Unity, repositories, and local project folders.', 'Unity diagnostika':'Unity diagnostics',
    'ČEKÁ NA PROJEKT':'WAITING FOR PROJECT','Analyzovat Unity projekt':'Analyze Unity project','Spustit statický audit':'Run static audit','Exportovat poslední audit (.md / .json)':'Export last audit (.md / .json)',
    'Vyber složku projektu a spusť analýzu.':'Choose a project folder and start an analysis.','Otevřít Unity dokumentaci ↗':'Open Unity documentation ↗','Lokální složka':'Local folder',
    'Návrhy změn & zálohy':'Change proposals & backups','SCHVÁLENÍ POVINNÉ':'APPROVAL REQUIRED','Relativní cesta':'Relative path','Obsah souboru (návrh)':'File content (proposal)',
    'Zkontrolovat návrh':'Review proposal','Zatím žádný návrh.':'No proposal yet.','Balík více-souborových změn':'Multi-file change set','Vlož JSON pole objektů':'Paste a JSON array of objects',
    '. Nejvýše 12 souborů. Všechny změny se nejprve zobrazí k revizi a zapíší se až po jednom výslovném schválení.':'. Up to 12 files. All changes are reviewed first and written only after explicit approval.',
    'Připravit a zkontrolovat celý balík':'Prepare and review the full set','Žádný balík změn není připraven.':'No change set prepared.',
    'KONTROLA DAT':'DATA CONTROLS','Moje':'My','paměť':'memory','Historie chatu je ukládána lokálně. Sémantická dlouhodobá paměť napříč konverzacemi zatím není implementována.':'Chat history is stored locally. Cross-conversation semantic memory is not implemented yet.',
    'Paměť zatím není aktivní':'Memory is not active yet','Historie konverzací se ukládá lokálně v tomto profilu aplikace. Můžeš ji exportovat; dlouhodobá sémantická paměť napříč konverzacemi zatím není aktivní.':'Conversation history is stored locally in this app profile. You can export it; long-term semantic memory across conversations is not active yet.',
    'Exportovat aktuální chat':'Export current chat','PREFERENCE A PROPOJENÍ':'PREFERENCES & CONNECTIONS','Lokální spojení, výkon a bezpečnostní výchozí hodnoty.':'Local connections, performance, and security defaults.',
    'API poskytovatelé':'API providers','VOLITELNÉ':'OPTIONAL','Protokol API':'API protocol','OpenAI kompatibilní':'OpenAI-compatible',
    'Název poskytovatele':'Provider name','HTTPS API endpoint':'HTTPS API endpoint','API klíč':'API key','Bezpečně uložit':'Save securely','Otestovat spojení':'Test connection','Odebrat klíč':'Remove key',
    'Kontroluji zabezpečené úložiště…':'Checking secure storage…','Ollama':'Ollama','Výchozí adresa lokálního serveru':'Default local server address','Stáhnout Ollama ↗':'Download Ollama ↗',
    'Záloha a přenos':'Backup & transfer','Exportuj konverzace a preference do přenosného JSON souboru. Import sloučí zálohu s aktuální historií; zdrojové projektové soubory se nikdy nepřibalují.':'Export conversations and preferences to a portable JSON file. Import merges the backup with the current history; source project files are never included.',
    'Exportovat zálohu':'Export backup','Importovat zálohu':'Import backup','Záloha se vytváří pouze na tvůj pokyn.':'A backup is created only when you ask for it.',
    'Bezpečnost':'Security','Potvrdit úpravy souborů':'Confirm file edits','Potvrdit systémové příkazy':'Confirm system commands','Výchozí přístup pouze ke složce':'Default access limited to selected folder',
    'Bezpečnostní pravidla jsou v této verzi pevně nastavena. Nelze je vypnout omylem.':'Security rules are fixed in this version and cannot be disabled accidentally.',
    'Historie a paměť':'History & memory','Konverzace zůstávají lokálně v profilu aplikace. Sémantická dlouhodobá paměť napříč konverzacemi zatím není aktivní.':'Conversations stay local to this app profile. Cross-conversation semantic memory is not active yet.',
    'Spravovat historii':'Manage history','HOLOGRAFICKÁ KONZOLE':'COMMAND PALETTE','Vyvolej schopnost. Aurora zůstává v jednom chatu.':'Invoke a capability. Aurora stays in one chat.','SCHOPNOSTI A PRACOVNÍ PROSTOR':'CAPABILITIES & WORKSPACE',
    'Pokračovat v chatu':'Continue chat','Vrátit se k hlavní konverzaci':'Return to the main conversation','Vytvořit obrázek':'Create image','Zadat vizuální nápad přímo do chatu':'Describe an image in chat',
    'Prozkoumat web':'Explore the web','Vyhledat zdroje a předat je lokálnímu modelu':'Search sources and pass them to the local model','Přidat projektový kontext':'Add project context',
    'Načíst vybrané textové soubory pro příští zprávu':'Load selected text files for the next message','Analyzovat projekt':'Analyze project','Vybereš složku jednou; analýza potom běží z chatu':'Choose a folder once; continue analysis from chat',
    'Model Hub':'Model Hub','Vybrat, stáhnout nebo spravovat lokální modely':'Select, download, or manage local models','Výkon a runtime':'Performance & runtime',
    'Stav Ollama, běžící modely a paměť':'Ollama status, running models, and memory','Agent Studio':'Agent Studio','Plán a návrh práce s explicitním schválením':'Plan and proposal workflow with explicit approval',
    'Projektové nástroje':'Project tools','Unity audit, skenování a návrhy změn':'Unity audits, scanning, and change proposals','Nastavení a záloha':'Settings & backup',
    'Lokální preference, import a export workspace':'Local preferences, workspace import and export','Žádná odpovídající akce. Zkus jiné slovo nebo pokračuj v chatu.':'No matching action. Try another term or continue in chat.',
    'výběr':'select','spustit':'run','konzole':'palette','LOKÁLNÍ JÁDRO':'LOCAL CORE',
    'Vzhled a chování':'Appearance & behavior','Rozhraní, jazyk a animace pozadí. Změny se ukládají lokálně.':'Interface, language, and background motion. Changes are stored locally.',
    'Jazyk rozhraní':'Interface language','Motiv':'Theme','Polární záře · video':'Aurora · video','Čistý grafit':'Clean graphite','Noční soumrak':'Night twilight',
    'Animované pozadí':'Animated background','Pohyb zapnutý':'Motion enabled','Síla pozadí':'Background intensity','Omezit animace rozhraní':'Reduce interface motion','Velikost písma':'Font size',
    'Menší':'Smaller','Výchozí':'Default','Větší':'Larger','Velké':'Large','Přehled systému při startu':'Show system overview at startup','Průběžné zobrazování odpovědi':'Stream responses',
    'Video je uloženo přímo v aplikaci. Při přepnutí na jinou obrazovku nebo minimalizaci se pozadí pozastaví.':'The video is bundled with the app. It pauses when the window is hidden or minimized.',
    'Generování a odpovědi':'Generation & responses','Běžné parametry, které podporují lokální modely; vzdálené poskytovatele Aurora nastaví podle jejich API.':'Common parameters for local models; remote providers receive only settings supported by their APIs.',
    'Výchozí profil výkonu':'Default performance profile','Vlastní systémové instrukce':'Custom system instructions','Jak má Aurora odpovídat? (volitelné)':'How should Aurora respond? (optional)',
    'Stop sekvence':'Stop sequences','Volitelné, odděluj | znakem':'Optional; separate with |','Udržet model v paměti':'Keep model loaded','Vypnout po odpovědi':'Unload after response',
    '1 minuta':'1 minute','5 minut':'5 minutes','10 minut':'10 minutes','30 minut':'30 minutes','1 hodina':'1 hour','Uložit nastavení generování':'Save generation settings','Nastavení je uloženo lokálně.':'Settings are stored locally.',
    'Top‑P':'Top-P','Top‑K':'Top-K','Opakování':'Repetition penalty','Seed':'Seed','Náhodný':'Random','Jak ti má Aurora říkat?':'What should Aurora call you?','Např. preferuji češtinu, stručné vysvětlení a nejdřív bezpečné řešení.':'For example: use Czech, explain briefly, and prefer safe solutions.','Přidat kontext':'Add context','Prázdné = náhodný výběr.':'Blank = random seed.',
    'Tento projekt je pouze pro čtení.':'This project is read-only.','Rychlost':'Speed','Kreativita':'Creativity','Důkladný':'Deep','Rychlý':'Fast','Vyvážený':'Balanced','Ruční':'Manual',
    'Model a výkon':'Model & performance','Rychlé akce':'Quick actions','Stav systému':'System status','Rychlé nástroje':'Quick tools','Výchozí workspace':'Default workspace','Nastavení odpovědi':'Response settings','Zobrazit přehled systému':'Show system overview','Skrýt přehled systému':'Hide system overview','Lokální režim':'Local mode','Vyber složku znovu pro nové oprávnění':'Choose the folder again to grant access','Zastavit generování':'Stop generation','Odeslat zprávu':'Send message','Zobrazit náhled':'Show preview','Otevřít':'Open','Zavřít':'Close','Zrušit':'Cancel','Uložit':'Save','Uložit změny':'Save changes','Potvrdit':'Confirm','Přehled':'Overview','Rychlé':'Fast','Odpověď':'Response','Výkonové profily':'Performance profiles','Systémové instrukce':'System instructions','Pokročilé generování':'Advanced generation','Automatický výběr modelu':'Automatic model selection','Použije se ručně vybraný model.':'Use the manually selected model.','Automatický výběr modelu zapnutý.':'Automatic model selection enabled.','Záloha byla importována.':'Backup imported.','Nový chat':'New chat','Analyzovat projekt':'Analyze project','Vyhledávání':'Search','Projekty & integrace':'Projects & integrations','Projekt nebyl nalezen':'Project not found','Přepnout rozhraní do češtiny':'Switch interface to Czech',
    'OVĚŘUJI LOKÁLNÍ SLUŽBY':'CHECKING LOCAL SERVICES',
    'Pro vzdálené role musí být příslušný protokol uložený v Nastavení → API a poskytovatelé.':'Remote roles require the matching protocol to be saved in Settings → API providers.',
    'je přepínač v chatu. Kód, Unity, logy a ladění preferují nainstalovaný model s označením coder/code. Běžné otázky preferují menší model; složité plánování může':'is the chat toggle. Code, Unity, logs, and debugging prefer installed models tagged coder/code. Everyday questions prefer a smaller model; complex planning may',
    'Webové vyhledávání předává dotaz vyhledávači. ComfyUI běží lokálně. Projektové soubory se čtou jen z vybrané složky; změny vyžadují výslovné schválení a systémové příkazy jsou ve výchozím stavu vypnuté.':'Web search sends the query to a search provider. ComfyUI runs locally. Project files are read only from the selected folder; changes require explicit approval and system commands are disabled by default.',
    'V ComfyUI použij Save (API Format). Aurora nahradí text v CLIP Text Encode uzlech. Před generováním zkontroluj model a rozlišení ve workflow.':'In ComfyUI, use Save (API Format). Aurora replaces text in CLIP Text Encode nodes. Check the model and resolution in the workflow before generating.',
    'Aktuálně dostupné: statický audit projektu, kontrola vybraných logů, export nálezů a schvalované návrhy úprav. Kompilaci ani spuštění Unity Editoru tato verze sama neprovádí.':'Available now: static project audit, selected log checks, findings export, and approved change proposals. This version does not compile or launch the Unity Editor.',
    'Vyber složku projektu. Aurora bude nejdříve pouze číst; úpravy budou připravené k potvrzení.':'Choose a project folder. Aurora starts in read-only mode; edits are prepared for approval.',
    'Žádná složka není vybrána':'No folder selected',
    'Agent může připravit návrh úpravy textového souboru. Před zápisem se zobrazí potvrzení a existující soubor dostane zálohu.':'An agent can prepare a text-file change proposal. Approval is required before writing, and existing files are backed up first.',
    'Podporuje OpenAI-kompatibilní API, nativní Anthropic Claude a Google Gemini. Vzdálené zprávy opouštějí počítač a účtování se řídí poskytovatelem.':'Supports OpenAI-compatible APIs, native Anthropic Claude, and Google Gemini. Remote messages leave your computer and billing depends on the provider.',
    'Endpoint se musí shodovat s vybraným protokolem. Klíč zůstává v šifrovaném úložišti hlavního procesu.':'The endpoint must match the selected protocol. The key stays encrypted in the main process.',
    'Klíč se ukládá zašifrovaný systémovým úložištěm. Nezobrazuje se zpět.':'The key is encrypted using system storage and is never displayed again.',
    'Čeština':'Czech', '40 · vyvážené':'40 · balanced', '20 · soustředěné':'20 · focused',
    '80 · rozmanitější':'80 · more varied', '100 · nejširší výběr':'100 · widest range',
    'OpenAI / kompatibilní':'OpenAI / compatible',
    'Webové vyhledávání předává dotaz vyhledávači. ComfyUI běží lokálně. Projektové soubory se čtou jen z vybrané složky; změny vyžadují výslovné schválení a systémové příkazy se automaticky nespouštějí.':'Web search sends your query to a search provider. ComfyUI runs locally. Project files are read only from the selected folder; changes require explicit approval and system commands are not run automatically.',
    'je přepínač v chatu. Kód, Unity, logy a ladění preferují nainstalovaný model s označením coder/code. Běžné otázky preferují menší model; složité plánování může použít větší dostupný model. Pokud není vhodný kandidát, použije se ručně vybraný model. Automatické směrování nemění výchozí model.':'is a switch in chat. Code, Unity, logs, and debugging prefer an installed model marked coder/code. Everyday questions prefer a smaller model; complex planning can use a larger available model. If no suitable candidate exists, the manually selected model is used. Automatic routing does not change the default model.',
    'Vzhled':'Appearance','Model a generování':'Model & generation','Poskytovatelé':'Providers','Data a paměť':'Data & memory','Bezpečnost':'Safety','Všechno':'All settings',
    'Vzhled a chování':'Appearance & behavior','Generování a odpovědi':'Generation & responses','Jazyk odpovědí':'Response language','Automaticky podle dotazu':'Automatically follow the prompt','Pokročilé parametry modelu':'Advanced model parameters',
    'Omezení opakování':'Repetition penalty','CPU vlákna':'CPU threads','Udržet model v paměti':'Keep model loaded','Uložit nastavení generování':'Save generation settings','Nastavení je uloženo lokálně.':'Settings are saved locally.',
    'Síla pozadí':'Background intensity','Animované pozadí':'Animated background','Omezit animace rozhraní':'Reduce interface motion','Velikost písma':'Font size','Přehled systému při startu':'Show system overview at startup',
    '4 096 tokenů · vyvážené':'4,096 tokens · balanced','8 192 tokenů · delší úkoly':'8,192 tokens · longer tasks','16 384 tokenů · náročnější RAM':'16,384 tokens · more RAM',
    '2 · nižší zátěž':'2 · lower load','4 · vyvážené':'4 · balanced','6 · vyšší výkon':'6 · higher throughput','8 · vyšší zátěž':'8 · higher load',
    'Krátká · 512':'Short · 512','Normální · 1 024':'Normal · 1,024','Dlouhá · 2 048':'Long · 2,048','Velmi dlouhá · 4 096':'Very long · 4,096',
    'Pohyb zapnutý':'Motion enabled','Polární záře · video':'Aurora borealis · video','Čistý grafit':'Clean graphite','Noční soumrak':'Night twilight',
    'Název poskytovatele':'Provider name','HTTPS API endpoint':'HTTPS API endpoint','Bezpečně uložit':'Save securely','Otestovat spojení':'Test connection','Odebrat klíč':'Remove key',
    'Generování a odpovědi':'Generation & responses','Pokročilé parametry modelu':'Advanced model parameters','Stop sekvence':'Stop sequences','Vlastní systémové instrukce':'Custom system instructions',
    'Výchozí profil výkonu':'Default performance profile','Výchozí adresa lokálního serveru':'Local server address','Záloha a přenos':'Backup & transfer','Exportovat zálohu':'Export backup','Importovat zálohu':'Import backup','Historie a paměť':'History & memory',
    'Potvrdit úpravy souborů':'Require approval for file edits','Potvrdit systémové příkazy':'Require approval for system commands','Výchozí přístup pouze ke složce':'Folder-scoped access by default',
    'Agent Studio':'Agent Studio','Pracovní režim':'Workspace mode','Hlavní navigace':'Main navigation','Přehled projektů':'Project overview','Projektové nástroje':'Project tools','Přehled nástrojů':'Tools overview','Internetový výzkum':'Web research','Generování obrázků':'Image generation','Knihovna modelů':'Model library','Výkon & runtime':'Performance & runtime','Osobní profil':'Personal profile','Model a odpovědi':'Model & responses','Zvuk a animace':'Sound & animation','API a poskytovatelé':'APIs & providers','Data a soukromí':'Data & privacy','Všechno důležité, bez složitostí':'Everything important, without the complexity','Krátký vizuální průvodce pracovním prostorem a soukromím.':'A short visual guide to your workspace and privacy.','ZAČNI JEDNODUŠE':'START SIMPLY','Jedno místo pro chat, modely i projekty.':'One place for chat, models, and projects.','Otevřít hlavní chat':'Open main chat','Tvůj profil a osobní pokyny':'Your profile and personal instructions','Jméno nebo přezdívka':'Name or nickname','Krátké osobní instrukce (volitelné)':'Short personal instructions (optional)','Uložit osobní profil':'Save personal profile','Plynulé přechody rozhraní':'Smooth interface transitions','Jemné zvuky':'Subtle sounds','Hlasitost zvuků':'Sound volume','Přehrát ukázku':'Play preview','Výchozí nastavení':'Reset to defaults','Skrýt postranní panel':'Hide sidebar','Zobrazit postranní panel':'Show sidebar','Hledat v konverzacích':'Search conversations','Možnosti chatu':'Chat options','Jak ti má Aurora říkat?':'What should Aurora call you?','Nezadávej hesla ani tajné údaje.':'Do not enter passwords or secrets.','Pokročilé parametry modelu':'Advanced model parameters','Rozmanitost slov (Top‑P)':'Word diversity (Top‑P)','Šířka výběru (Top‑K)':'Candidate range (Top-K)','Pevný náhodný základ (Seed)':'Fixed random seed','Skrýt hlavní kategorie; historie chatů zůstane vidět':'Hide main categories; keep chat history visible','Zobrazit hlavní kategorie':'Show main categories','Průvodce Aurorou':'Aurora guide','Otevřít průvodce Aurorou':'Open Aurora guide','Chceš osobní nastavení?':'Want personal settings?','Nastavit osobní profil':'Set up your personal profile','Soukromí':'Privacy','Agentní režim':'Agent Studio','Dokončené chaty':'Completed chats',
    'Volitelná lokální dlouhodobá paměť':'Optional local long-term memory',
    'Ukládej jen poznámky, které chceš, aby si Aurora pamatovala. Vyhledávání používá embedding model přes lokální Ollamu; poznámky se neposílají do cloudu a automaticky se neukládá celý chat. Paměť je oddělená od běžné zálohy workspace.':'Save only notes that you want Aurora to remember. Retrieval uses an embedding model through local Ollama; notes are not sent to the cloud and entire chats are not saved automatically. Memory is separate from the regular workspace backup.',
    'Používat lokální paměť':'Use local memory','Vypnuto ve výchozím stavu':'Off by default','Lokální embedding model':'Local embedding model',
    'Ověřit embedding model':'Test embedding model','Obnovit poznámky':'Refresh notes','Název poznámky':'Note title',
    'Co si má Aurora pamatovat?':'What should Aurora remember?','Uložit lokální poznámku':'Save local note','Smazat všechny poznámky':'Delete all notes',
    'Spravovat historii chatů':'Manage chat history','Lokální poznámka':'Local note',
    'Např. Můj styl práce':'E.g. My work style','Např. Pracuji na projektu Aurora, preferuji češtinu a změny chci před zápisem vždy zkontrolovat.':'E.g. I work on Aurora, prefer Czech, and want changes reviewed before writing.',
    'ULOŽENÉ':'SAVED','Uložené':'Saved','PINNED':'PINNED','PROJEKTY':'PROJECTS','NEDÁVNÉ':'RECENTS','Archiv':'Archive','Připni chat přes nabídku ⋯.':'Pin a chat from its ⋯ menu.','Zatím žádné projekty. Přidej první přes +.':'No projects yet. Add one with +.','Zobrazit všechny projekty':'View all projects','Přidat projekt':'Add project','Pracovní prostory':'Project workspaces','Každý projekt má vlastní složku a samostatné chaty. Odebrání projektu nikdy nesmaže soubory na disku.':'Each project has its own folder and chats. Removing a project never deletes files from disk.','Načítám projekty…':'Loading projects…','Vyber projekt…':'Choose a project…','Pracovní projekt':'Project workspace','＋ Přidat projekt':'＋ Add project','Klikni pro připojení':'Click to connect','Připojeno':'Connected','Otevřít Code':'Open Code','Přejmenovat':'Rename','Odebrat':'Remove','Odebrat z Aurory':'Remove from Aurora','Přiřadit k projektu':'Assign to project','Projekt chatu':'Chat project','Bez projektu':'No project','Připnout do Uložených':'Pin to Saved','Odepnout z Uložených':'Unpin from Saved','Přesunout do archivu':'Move to archive','Smazat konverzaci':'Delete conversation','Přejmenovat':'Rename','Přejmenovat chat':'Rename chat','Smazat chat':'Delete chat','Nový název konverzace':'New conversation title','Možnosti chatu':'Chat options','Možnosti konverzace':'Conversation options','Otevřít pracovní prostory':'Open workspaces','Zobrazit pracovní prostory':'View workspaces','Nový název projektu':'New project name','Odebrat projekt':'Remove project','Vytvoření projektu selhalo:':'Could not create project:','Projekt byl odebrán z Aurory; soubory zůstaly na disku.':'The project was removed from Aurora; files remain on disk.','Vyber projekt (volitelné; jinak se použije název složky)':'Project name (optional; the folder name will be used otherwise)','Síla pozadí':'Background intensity','%':'%','Omezit animace rozhraní':'Reduce interface motion','Velikost písma':'Font size','Přehled systému při startu':'Show system overview on startup','Pohyb zapnutý':'Motion enabled','Lokální složka':'Local folder','Přidat pracovní projekt':'Add workspace project','Otevřít projekt':'Open project','Pokračovat v chatu':'Continue in chat','Žádné archivované chaty.':'No archived chats.','Zatím žádné konverzace.':'No conversations yet.','Přiřadit chat k projektu':'Assign chat to a project','Nový název konverzace':'New conversation name','Připnout chat':'Pin chat','Odepnout chat':'Unpin chat','Přidat do projektových chatů':'Add to project chats','Odebrat z projektových chatů':'Remove from project chats','Odebrat chat z projektu':'Remove chat from project','Přiřadit chat k projektu':'Assign chat to a project','Projekt chatu':'Chat project','Bez projektu':'No project','Možnosti chatu':'Chat options','Možnosti konverzace':'Conversation options','Přejmenovat chat':'Rename chat','Smazat chat':'Delete chat','Obnovit mezi nedávné':'Restore to recents','Přesunout do archivu':'Move to archive','Žádné archivované chaty.':'No archived chats.','Zatím žádné konverzace.':'No conversations yet.','Chaty označené jako projekty':'Chats marked as projects','Odebrat projekt':'Remove project','Připojeno':'Connected','Klikni pro připojení':'Click to connect','Zobrazit všechny projekty':'View all projects','Přejmenovat projekt':'Rename project','Odebrat z Aurory':'Remove from Aurora','Projekt byl odebrán z Aurory; soubory zůstaly na disku.':'Project removed from Aurora; files remain on disk.'
  });
  const ATTRS = Object.freeze({
    'Všechno důležité, bez složitostí':'Everything important, without the complexity','Krátký vizuální průvodce pracovním prostorem a soukromím.':'A short visual guide to your workspace and privacy.','ZAČNI JEDNODUŠE':'START SIMPLY','Jedno místo pro chat, modely i projekty.':'One place for chat, models, and projects.','Otevřít hlavní chat':'Open main chat','Zvuk a animace rozhraní':'Interface sound and animation','Tvůj profil a osobní pokyny':'Your profile and personal instructions','Jméno nebo přezdívka':'Name or nickname','Uložit osobní profil':'Save personal profile','Plynulé přechody rozhraní':'Smooth interface transitions','Jemné zvuky':'Subtle sounds','Přehrát ukázku':'Play preview','Výchozí nastavení':'Reset to defaults','Skrýt postranní panel':'Hide sidebar','Zobrazit postranní panel':'Show sidebar','Hledat v konverzacích':'Search conversations','Možnosti chatu':'Chat options','Napiš zprávu…':'Type a message…','Napiš zprávu… (/image · /web · /project)':'Type a message… (/image · /web · /project)',
    'Např. analyzuj můj Unity projekt, najdi chyby a navrhni testy bez úprav souborů…':'For example: analyze my Unity project, find bugs, and propose tests without editing files…',
    'Hledej schopnost nebo akci…':'Search for a capability or action…','Filtrovat rychlé akce':'Filter quick actions','API klíč od vybraného poskytovatele':'API key from the selected provider',
    'Napiš podrobnější popis':'Enter a more detailed description','Vlož nový obsah pro kontrolu…':'Paste proposed content for review…',
    'Assets/Scripts/Example.cs':'Assets/Scripts/Example.cs',
    'Jak má Aurora odpovídat? (volitelné)':'How should Aurora respond? (optional)',
    'Volitelné, odděluj | znakem':'Optional, separate with |',
    'Náhodný':'Random','Jak ti má Aurora říkat?':'What should Aurora call you?','Např. preferuji češtinu, stručné vysvětlení a nejdřív bezpečné řešení.':'For example: use Czech, explain briefly, and prefer safe solutions.','Přidat kontext':'Add context',
    'Pracovní režim':'Work mode','Hlavní a vedlejší agenti':'Primary and supporting agents','Výkon počítače':'Computer performance','RAM, modely a odezva':'RAM, models, and latency',
    'Přehled projektů':'Project overview','Chaty označené jako projekty':'Chats marked as projects','Projektové nástroje':'Project tools','Skenování a bezpečné návrhy':'Scanning and safe proposals',
    'Dokončené chaty':'Completed chats','Přehled nástrojů':'Tools overview','Co je dostupné a připravené':'Availability and setup status','Internetový výzkum':'Web research','Vyhledávání a zdroje':'Search and sources',
    'Generování obrázků':'Image generation','Lokální ComfyUI':'Local ComfyUI','Knihovna modelů':'Model library','Stažení a správa':'Download and manage',
    'Osobní profil':'Personal profile','Jméno a vlastní pokyny':'Name and personal instructions','Model a odpovědi':'Model and responses','Jazyk a pokročilé parametry':'Language and advanced settings',
    'Zvuk a animace':'Sound and animation','Jemné efekty rozhraní':'Subtle interface effects','API a poskytovatelé':'APIs and providers','Lokální nebo vzdálené AI':'Local or remote AI',
    'Data a soukromí':'Data and privacy','Zálohy a lokální historie':'Backups and local history','Ochrany a schvalování':'Safeguards and approvals',
    'Všechno důležité, bez složitostí':'Everything important, without the complexity','Krátký vizuální průvodce pracovním prostorem a soukromím.':'A short visual guide to your workspace and privacy.',
    'ZAČNI JEDNODUŠE':'START SIMPLE','Jedno místo pro chat, modely i projekty.':'One place for chat, models, and projects.','Nemusíš znát technické parametry. Začni chatem a pokročilé nástroje otevři až tehdy, když je potřebuješ.':'You do not need to know technical parameters. Start with chat and open advanced tools only when needed.',
    'Otevřít hlavní chat':'Open the main chat','Přejít do chatu':'Go to chat','Otevřít knihovnu modelů':'Open model library','Přejít do Agent Studio':'Open Agent Studio',
    'Soukromí':'Privacy','Lokální Ollama zůstává na tvém počítači. Vzdálené API posílá zprávy vybranému poskytovateli. Klíče jsou uloženy odděleně.':'Local Ollama stays on your computer. Remote APIs send messages to the selected provider. Keys are stored separately.',
    'Nastavení připojení':'Connection settings','Chceš osobní nastavení?':'Want a more personal setup?','Jméno, přezdívku a krátké vlastní pokyny můžeš přidat kdykoliv.':'You can add a name, nickname, and short personal instructions at any time.','Nastavit osobní profil':'Set up personal profile',
    'Napiš normálně. Aurora využije vybraný model a průběžně ukazuje odpověď. Technické volby nech na automatickém profilu.':'Just write naturally. Aurora uses the selected model and streams the answer. Leave technical choices to the adaptive profile.',
    'V chatu se vybírají stažené modely. V knihovně můžeš jiné stáhnout, zobrazit jejich nároky nebo je odebrat.':'Choose installed models in chat. In the library, download others, check their requirements, or remove them.',
    'Pro složitější práci zapoj hlavního a vedlejšího agenta. Více rolí zapínej jen tehdy, když to výkon počítače dovolí.':'Use a primary and supporting agent for complex work. Enable more roles only if your hardware can handle them.',
    'Pokročilé parametry modelu':'Advanced model parameters','Tón rozhraní':'Interface sounds','Přehrát ukázku':'Preview sound','Plynulé přechody rozhraní':'Smooth interface transitions',
    'Např. Můj styl práce':'E.g. My work style','Např. Pracuji na projektu Aurora, preferuji češtinu a změny chci před zápisem vždy zkontrolovat.':'E.g. I work on Aurora, prefer Czech, and want changes reviewed before writing.'
  });
  function normalizePreferences(raw) {
    const p = raw && typeof raw === 'object' ? raw : {};
    return {
      locale: p.locale === 'en' ? 'en' : 'cs', theme: ['aurora','graphite','twilight'].includes(p.theme) ? p.theme : 'aurora',
      backgroundMotion: p.backgroundMotion !== false,
      backgroundIntensity: Math.max(0, Math.min(1, Number.isFinite(Number(p.backgroundIntensity)) ? Number(p.backgroundIntensity) : DEFAULTS.backgroundIntensity)),
      reduceMotion: p.reduceMotion === true,
      fontScale: [0.9,1,1.1,1.2].includes(Number(p.fontScale)) ? Number(p.fontScale) : 1,
      showDashboard: p.showDashboard === true, navigationHidden: p.navigationHidden === true, sidebarCollapsed: p.sidebarCollapsed === true,
      interfaceEffects: p.interfaceEffects !== false, soundEffects: p.soundEffects === true,
      soundVolume: Math.max(0,Math.min(0.6,Number.isFinite(Number(p.soundVolume))?Number(p.soundVolume):DEFAULTS.soundVolume))
    };
  }
  function loadPreferences(storage) { try { return normalizePreferences(JSON.parse(storage?.getItem(STORAGE_KEY) || '{}')); } catch { return {...DEFAULTS}; } }
  function savePreferences(storage, prefs) { const safe=normalizePreferences(prefs); try { storage?.setItem(STORAGE_KEY,JSON.stringify(safe)); } catch {} return safe; }
  function keyOf(s) { return String(s || '').replace(/\s+/g,' ').trim(); }
  let prefs = null;
  function shouldSkipText(node) {
    const el = node?.parentElement;
    if (!el) return true;
    // Option labels must be translated, even though most form controls are skipped.
    if (el.tagName === 'OPTION') return false;
    return !!el.closest('script,style,textarea,input,select,code,pre,.message-body,.message-content,.code-block,[data-no-translate]');
  }
  function translateLabel(value, locale = prefs?.locale || 'cs') {
    const normalized = keyOf(value);
    if (!normalized) return value;
    if (locale === 'en') return EN[normalized] || normalized;
    return reverseEN[normalized] || normalized;
  }
  function translatedNodeText(node) {
    if (!node || node.nodeType !== 3 || shouldSkipText(node)) return;
    const raw = node.nodeValue || '';
    const leading = raw.match(/^\s*/)?.[0] || '';
    const trailing = raw.match(/\s*$/)?.[0] || '';
    const inner = raw.slice(leading.length, raw.length - trailing.length);
    const normalized = keyOf(inner);
    if (!normalized) return;
    const translated = translateLabel(normalized, prefs?.locale || 'cs');
    if (translated && translated !== normalized) node.nodeValue = `${leading}${translated}${trailing}`;
  }
  function translateTree(node) {
    if (!node) return;
    if (node.nodeType === 3) { translatedNodeText(node); return; }
    const rootNode = node.nodeType === 9 ? node.body : node;
    if (!rootNode) return;
    const visit = document.createTreeWalker(rootNode, NodeFilter.SHOW_TEXT);
    let current;
    while ((current = visit.nextNode())) translatedNodeText(current);
  }
  const reverseEN = Object.freeze(Object.fromEntries(Object.entries(EN).map(([cs,en])=>[en,cs])));
  function translateAttributes(target = document) {
    const selector='[placeholder],[title],[aria-label]';
    const nodes=target.nodeType===1
      ? [...(target.matches(selector)?[target]:[]), ...target.querySelectorAll(selector)]
      : [...target.querySelectorAll(selector)];
    nodes.forEach(el=>{
      for (const attr of ['placeholder','title','aria-label']) {
        const val=el.getAttribute(attr); if(!val)continue; const k=keyOf(val);
        if(prefs.locale==='en' && ATTRS[k] && ATTRS[k]!==val)el.setAttribute(attr,ATTRS[k]);
        if(prefs.locale==='cs') { const reverse=reverseAttr[k]; if(reverse && reverse!==val)el.setAttribute(attr,reverse); }
      }
    });
  }
  const reverseAttr=Object.freeze(Object.fromEntries(Object.entries(ATTRS).map(([cs,en])=>[en,cs])));
  function applyVideo() {
    const video=document.getElementById('auroraBackgroundVideo'); if(!video)return;
    video.playbackRate=0.82;
    const active=prefs.backgroundMotion && !document.hidden;
    if(active) { const promise=video.play(); if(promise?.catch)promise.catch(()=>{}); }
    else video.pause();
  }
  function applyPreferences() {
    if(!prefs) return;
    document.documentElement.lang=prefs.locale; document.title=prefs.locale==='en'?'Aurora AI — Local assistant':'Aurora AI';
    document.body.dataset.theme=prefs.theme;
    document.documentElement.style.setProperty('--aurora-video-opacity', String(prefs.backgroundIntensity));
    document.documentElement.style.setProperty('--ui-font-scale', String(prefs.fontScale));
    document.body.classList.toggle('reduce-motion',prefs.reduceMotion);
    const main=document.querySelector('.main'); if(main){main.classList.toggle('dashboard-hidden',!prefs.showDashboard);main.dataset.view=main.dataset.view||'chat';}
    const sidebar=document.querySelector('.sidebar');if(sidebar)sidebar.classList.remove('navigation-hidden');const appShell=document.querySelector('.app');if(appShell)appShell.classList.toggle('sidebar-collapsed',prefs.sidebarCollapsed);const sidebarToggle=document.getElementById('sidebarToggle');if(sidebarToggle){sidebarToggle.setAttribute('aria-expanded',String(!prefs.sidebarCollapsed));sidebarToggle.setAttribute('aria-label',prefs.sidebarCollapsed?'Zobrazit postranní panel':'Skrýt postranní panel');sidebarToggle.title=prefs.sidebarCollapsed?'Zobrazit postranní panel':'Skrýt postranní panel';sidebarToggle.innerHTML=prefs.sidebarCollapsed?'<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16m-5-10-2 2 2 2"/></svg>':'<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16m5-10 2 2-2 2"/></svg>'; }
    document.body.classList.toggle('quiet-ui-effects',!prefs.interfaceEffects||prefs.reduceMotion);
    const assign=(id,value)=>{const el=document.getElementById(id);if(el&&document.activeElement!==el)el.value=String(value)};
    assign('uiLanguage',prefs.locale);assign('uiTheme',prefs.theme);assign('backgroundIntensity',Math.round(prefs.backgroundIntensity*100));assign('uiFontScale',prefs.fontScale);assign('soundVolume',Math.round(prefs.soundVolume*100));
    const check=(id,value)=>{const el=document.getElementById(id);if(el)el.checked=!!value};
    check('backgroundMotion',prefs.backgroundMotion);check('reduceMotion',prefs.reduceMotion);check('showDashboardDefault',prefs.showDashboard);check('interfaceEffects',prefs.interfaceEffects);check('soundEffects',prefs.soundEffects);
    const intensity=document.getElementById('backgroundIntensityValue');if(intensity)intensity.textContent=`${Math.round(prefs.backgroundIntensity*100)} %`;const soundVolume=document.getElementById('soundVolumeValue');if(soundVolume)soundVolume.textContent=`${Math.round(prefs.soundVolume*100)} %`;
    const toggle=document.getElementById('toggleDashboard');if(toggle){toggle.setAttribute('aria-expanded',String(prefs.showDashboard));toggle.title=prefs.showDashboard?'Skrýt přehled systému':'Zobrazit přehled systému';}
    const langButton=document.getElementById('toggleLanguage');if(langButton){langButton.textContent=prefs.locale==='cs'?'EN':'CS';langButton.title=prefs.locale==='cs'?'Switch interface to English':'Přepnout rozhraní do češtiny';}
    if(prefs.locale==='en') translateTree(document); else translateTree(document);
    translateAttributes();
    applyVideo();
  }
  function update(patch) { prefs=normalizePreferences({...prefs,...patch}); savePreferences(localStorage,prefs); applyPreferences(); return prefs; }
  let audioContext=null;
  function playSound(kind='navigate',force=false){
    if(!force&&!prefs?.soundEffects)return false;
    try{const Ctx=root.AudioContext||root.webkitAudioContext;if(!Ctx)return false;audioContext=audioContext||new Ctx();if(audioContext.state==='suspended')audioContext.resume().catch(()=>{});const now=audioContext.currentTime;const tones=kind==='preview'?[[523,.08],[659,.09],[784,.12]]:kind==='complete'?[[660,.08],[880,.12]]:kind==='error'?[[330,.12],[247,.16]]:kind==='open'?[[493,.06],[659,.08]]:[[440,.035]];tones.forEach(([frequency,duration],index)=>{const osc=audioContext.createOscillator(),gain=audioContext.createGain();osc.type='sine';osc.frequency.value=frequency;const start=now+index*.075,volume=Math.max(.001,Math.min(.08,(prefs?.soundVolume??DEFAULTS.soundVolume)*.22));gain.gain.setValueAtTime(.0001,start);gain.gain.exponentialRampToValueAtTime(volume,start+.012);gain.gain.exponentialRampToValueAtTime(.0001,start+duration);osc.connect(gain);gain.connect(audioContext.destination);osc.start(start);osc.stop(start+duration+.02)});return true}catch{return false}
  }
  function init() {
    prefs=loadPreferences(localStorage);
    try {
      if (localStorage.getItem(VIDEO_MIGRATION_KEY) !== 'done') {
        prefs=normalizePreferences({...prefs,backgroundMotion:true,backgroundIntensity:Math.max(prefs.backgroundIntensity,0.58)});
        savePreferences(localStorage,prefs);
        localStorage.setItem(VIDEO_MIGRATION_KEY,'done');
      }
    } catch {}
    applyPreferences();
    const video=document.getElementById('auroraBackgroundVideo'); if(video){video.addEventListener('loadedmetadata',applyVideo);video.addEventListener('loadeddata',applyVideo);video.addEventListener('canplay',applyVideo);video.addEventListener('error',()=>{document.body.classList.add('background-video-failed')});}
    document.addEventListener('visibilitychange',applyVideo);
    const bind=(id, fn)=>document.getElementById(id)?.addEventListener('change',fn);
    bind('uiLanguage',e=>update({locale:e.target.value}));bind('uiTheme',e=>update({theme:e.target.value}));
    bind('backgroundMotion',e=>update({backgroundMotion:e.target.checked}));bind('backgroundIntensity',e=>update({backgroundIntensity:Number(e.target.value)/100}));
    bind('reduceMotion',e=>update({reduceMotion:e.target.checked}));bind('uiFontScale',e=>update({fontScale:Number(e.target.value)}));bind('showDashboardDefault',e=>update({showDashboard:e.target.checked}));bind('interfaceEffects',e=>update({interfaceEffects:e.target.checked}));bind('soundEffects',e=>update({soundEffects:e.target.checked}));bind('soundVolume',e=>update({soundVolume:Number(e.target.value)/100}));document.getElementById('resetUiEffects')?.addEventListener('click',()=>update({interfaceEffects:true,soundEffects:false,soundVolume:DEFAULTS.soundVolume,reduceMotion:false}));document.getElementById('previewSound')?.addEventListener('click',()=>playSound('preview',true));
    document.getElementById('toggleDashboard')?.addEventListener('click',()=>update({showDashboard:!prefs.showDashboard}));
    document.getElementById('toggleLanguage')?.addEventListener('click',()=>update({locale:prefs.locale==='cs'?'en':'cs'}));
    document.getElementById('sidebarToggle')?.addEventListener('click',()=>update({sidebarCollapsed:!prefs.sidebarCollapsed}));document.addEventListener('keydown',e=>{if((e.ctrlKey||e.metaKey)&&e.shiftKey&&e.key.toLowerCase()==='b'){e.preventDefault();update({sidebarCollapsed:!prefs.sidebarCollapsed});}});
    const observer=new MutationObserver(records=>{
      for(const record of records){
        if(record.type==='childList') record.addedNodes.forEach(n=>{if(n.nodeType===1||n.nodeType===3)translateTree(n)});
        else if(record.type==='characterData') translatedNodeText(record.target);
        else if(record.type==='attributes') translateAttributes(record.target);
      }
    });
    observer.observe(document.body,{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:['placeholder','title','aria-label']});
    translateTree(document);translateAttributes();
    return {get:()=>({...prefs}),update};
  }
  const API={DEFAULTS,normalizePreferences,loadPreferences,savePreferences,translateLabel,init,playSound,translations:EN};
  if(typeof module!=='undefined'&&module.exports)module.exports=API;
  if(root&&root.document){root.AuroraUI=API; if(root.document.readyState==='loading')root.document.addEventListener('DOMContentLoaded',init,{once:true});else init();}
})(typeof window!=='undefined'?window:globalThis);
