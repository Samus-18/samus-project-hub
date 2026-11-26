// Locally bundled catalogue. No claimed benchmark estimates; current source status is explicit.
#[derive(Clone,serde::Deserialize)]struct ExtensionHelp{id:String,request:String,status:String}
fn rm_area_explainer(a:&BoostApp,area:SuiteArea)->(&'static str,&'static str,&'static str,&'static str,&'static str,&'static str){
 match area{
 SuiteArea::Home=>("Select a running game and choose a task.","Vyber spuštěnou hru a zvol úkol.","One place to reach filters, capture and live statistics.","Jedno místo pro filtry, záznam a živé statistiky.","No change to the game until an action is started.","Dokud nic nespustíš, hra se nemění."),
 SuiteArea::Boost=>("Controls Horizon's own rendering, resolution scaling and generated frames.","Řídí vykreslování Horizonu, upscaling a generované snímky.","Potentially smoother output with enough GPU headroom; actual gain must be measured.","Při volné kapacitě GPU může být výstup plynulejší; přínos se musí změřit.","Extra GPU load, latency or artifacts; engine FPS is not multiplied.","Může zvýšit zátěž GPU, latenci či artefakty; FPS hry se nenásobí."),
 SuiteArea::Visual=>("Applies genuine D3D11 post-processing to captured game frames.","Aplikuje skutečné D3D11 efekty na snímané snímky hry.","Changes color, contrast, detail or bloom in Horizon's output.","Mění barvy, kontrast, detaily či bloom ve výstupu Horizonu.","Requires capture-compatible window; filters usually cost GPU time and do not boost game FPS.","Vyžaduje podporované okno hry; filtry většinou zatěžují GPU a nezvyšují FPS hry."),
 SuiteArea::Tools=>("Manual in-game companion tools: clocks, notes, maps, crosshair and practice.","Ruční herní pomůcky: časovače, poznámky, mapy, zaměřovač a trénink.","Better organization; no change to engine rendering or performance.","Lepší organizace; nemění vykreslování ani výkon hry.","Some overlays may be unavailable in true exclusive fullscreen.","Některé překryvné nástroje nemusí fungovat v exkluzivním fullscreenu."),
 SuiteArea::Macros=>("Record and edit time-bounded input for permitted offline use.","Nahrávání a úprava časově omezených vstupů pro povolené offline použití.","Repeats local repetitive actions; no FPS improvement.","Opakuje místní rutinní úkony; FPS nezlepšuje.","Focus loss, protected games and F12 stop playback; use responsibly.","Ztráta fokusu, chráněné hry a F12 přehrávání zastaví; používej zodpovědně."),
 SuiteArea::Overlay=>("Display measured FPS, timing, timers, and HUD widgets.","Zobrazuje měřená FPS, časování, časovače a widgety.","Access information while playing; no intrinsic FPS boost.","Informace během hraní; samo o sobě FPS nezvyšuje.","Overlay support depends on the Windows display mode.","Podpora HUD závisí na režimu zobrazení Windows."),
 SuiteArea::Capture=>("Create local screenshots, clips, video and edit projects.","Vytváří místní snímky, klipy, videa a projekty sestřihu.","Save game moments to your own PC.","Ukládá herní okamžiky do počítače.","Uses disk, encoding and CPU/GPU capacity; does not increase FPS.","Spotřebovává disk a výkon CPU/GPU pro kódování; FPS nezvyšuje."),
 SuiteArea::Library=>("Manage game entries, profiles, local notes and backups.","Spravuje hry, profily, místní poznámky a zálohy.","Faster access and reversible saved settings.","Rychlejší přístup a vratná uložená nastavení.","Some actions require selecting the correct game process.","Některé funkce vyžadují správně vybraný proces hry."),
 SuiteArea::System=>("Inspect hardware and propose reversible Windows/game-adjacent changes.","Kontroluje hardware a nabízí vratné změny nastavení systému.","May help stability where a real bottleneck exists.","Při skutečném úzkém hrdle může pomoci stabilitě.","System changes need explicit approval and may not help every game.","Systémové změny vyžadují souhlas a nemusí pomoci každé hře."),
 SuiteArea::Settings=>("Change interface, shortcuts, permissions and safety guards.","Mění rozhraní, zkratky, oprávnění a ochrany.","More comfortable and controlled operation.","Pohodlnější a kontrolovanější používání.","Changing cosmetic settings does not improve game FPS.","Změna vzhledu sama o sobě nezvyšuje FPS hry."),
 SuiteArea::Lab=>("Inspect experimental or unsupported hardware integrations.","Zobrazuje experimentální či nepodporované hardwarové integrace.","Shows what can be tested and what is not ready.","Ukazuje, co lze testovat a co ještě není připravené.","Not all features are integrated or game-verified.","Ne všechny funkce jsou integrované či ověřené ve hrách.")
 }
}
fn rm_help_status(a:&BoostApp,s:&str)->&'static str{
 let cs=a.settings.ui_language==1;
 match s{
 "INTEGRATED_NOT_GAME_VERIFIED"|"SOURCE_TESTED_NOT_LIVE_VERIFIED"=>if cs{"Kód testován, hra neověřena"}else{"Source-tested, game unverified"},
 "PARTIAL"=>if cs{"Částečně hotovo"}else{"Partial"},
 "PLANNED"|"PLANNED_NOT_IMPLEMENTED"=>if cs{"Chybí"}else{"Not implemented"},
 "LEGACY_ONLY_NOT_NATIVE"=>if cs{"Pouze starší cesta"}else{"Legacy only"},
 _=>if cs{"Neověřeno"}else{"Unverified"}
 }
}
fn rm_help_panel(a:&BoostApp,ui:&mut Ui,area:SuiteArea){
 let(title_en,title_cs,benefit_en,benefit_cs,risk_en,risk_cs)=rm_area_explainer(a,area);
 egui::CollapsingHeader::new(rm(a,"What this section does / FPS impact","Co tato sekce dělá / dopad na FPS")).id_salt(("remake-help",area.name())).default_open(false).show(ui,|ui|{
   frame(CARD).show(ui,|ui|{
   ui.horizontal_wrapped(|ui|{ui.label(RichText::new("●").color(V25_MINT));ui.label(rm(a,"Purpose","Účel"));});
   ui.label(rm(a,title_en,title_cs));
   ui.horizontal_wrapped(|ui|{ui.label(RichText::new("▲").color(V25_MINT));ui.label(rm(a,"Potential benefit","Možný přínos"));});
   ui.label(rm(a,benefit_en,benefit_cs));
   ui.horizontal_wrapped(|ui|{ui.label(RichText::new("!").color(RED));ui.label(rm(a,"Risk / limits","Rizika / omezení"));});
   ui.label(rm(a,risk_en,risk_cs));
   caption(ui,rm(a,"FPS gain: not estimated. Measured percentages appear only after a valid before/after sample.","Zlepšení FPS: bez odhadu. Procenta se ukážou až po platném měření před a po."));
   });
 });
}
fn rm_how_to_minecraft(a:&BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new(rm(a,"Minecraft filter troubleshooting","Oprava filtrů v Minecraftu")).default_open(false).show(ui,|ui|{
 frame(PANEL).show(ui,|ui|{
 heading(ui,rm(a,"If the filter is not visible","Když filtr není vidět"));
 let steps=[
 (rm(a,"1. Start Minecraft Java/Bedrock, select the correct process above.","1. Spusť Minecraft Java/Bedrock a vyber správný proces výše.")),
 (rm(a,"2. Set Minecraft to windowed/borderless; exclusive fullscreen is not supported reliably.","2. Nastav Minecraft do okna či bez okrajů; exkluzivní fullscreen není spolehlivě podporovaný.")),
 (rm(a,"3. Choose a look and start separate GPU output as the safest first test.","3. Vyber efekt a nejprve vyzkoušej samostatné GPU okno.")),
 (rm(a,"4. Return to Minecraft with Alt+Tab. Borderless output may pause until the game has focus.","4. Vrať se přes Alt+Tab do Minecraftu. Výstup se může pozastavit, dokud hra není v popředí.")),
 (rm(a,"5. If WGC capture fails, try DXGI fallback. Anti-cheat-protected games may be blocked.","5. Pokud WGC selže, zkus DXGI. Hry chráněné anti-cheatem mohou být zablokované."))
 ];
 for (i,v) in steps.iter().enumerate(){ui.horizontal_wrapped(|ui|{ui.label(RichText::new(format!("{}",i+1)).color(RED).strong());ui.label(*v);});}
 caption(ui,rm(a,"No app can guarantee effects in every exclusive Vulkan/DX12 game without integration. No Magpie and no game injection.","Bez integrace nelze slíbit efekty ve všech exkluzivních Vulkan/DX12 hrách. Bez Magpie i injekce do hry."));
 });
 });
}
fn rm_feature_impact(a:&BoostApp,area:SuiteArea)->&'static str{
 let(_,_,_,_,en,cs)=rm_area_explainer(a,area);
 rm(a,en,cs)
}
fn rm_catalogue(a:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("rm-global-help-search");
 let mut query=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_default();
 ui.add(egui::TextEdit::singleline(&mut query).hint_text(rm(a,"Search all 257 features by name or ID","Hledat všech 257 funkcí podle názvu nebo ID")).char_limit(100));
 ui.data_mut(|d|d.insert_temp(id,query.clone()));
 caption(ui,rm(a,"Original feature names are preserved; category-level EN/CZ guidance and each item's actual completion status follow.","Původní názvy funkcí jsou zachované; vysvětlení kategorií je CZ/EN a stav jednotlivých položek odpovídá skutečnosti."));
 let needle=crate::command_search::fold(&query);
 let mut matched=0usize;
 egui::ScrollArea::vertical().max_height(430.).id_salt("rm-global-help-scroll").show(ui,|ui|{
 for v in crate::unsorted_catalog::entries(){
  if !needle.is_empty()&&!crate::command_search::fold(&format!("{} {}",v.id,v.request)).contains(&needle){continue;}
  matched+=1;if matched>70{continue;}
  let area=suite_owner(&v.id);
  egui::CollapsingHeader::new(format!("#{}  {}",v.id,v.request)).id_salt(("explain",v.id.as_str())).show(ui,|ui|{
   ui.label(RichText::new(rm_help_status(a,&v.status)).color(if v.status=="INTEGRATED_NOT_GAME_VERIFIED"{MUTED}else{RED}));
   ui.label(rm_feature_impact(a,area));
   if ui.small_button(rm(a,"Open section","Otevřít sekci")).clicked(){area.go(a);}
  });
 }
 let ext:Vec<ExtensionHelp>=serde_json::from_str(include_str!("../assets/gaming_suite_expansion.json")).unwrap_or_default();
 for v in &ext{
  if !needle.is_empty()&&!crate::command_search::fold(&format!("{} {}",v.id,v.request)).contains(&needle){continue;}
  matched+=1;if matched>70{continue;}
  let idx=v.id.strip_prefix("GS-").unwrap_or(&v.id).parse::<usize>().unwrap_or(0);
  let area=match idx{201..=212=>SuiteArea::Capture,213..=230=>SuiteArea::Tools,231..=235=>SuiteArea::Macros,236..=240=>SuiteArea::Overlay,241..=246=>SuiteArea::Library,247..=251=>SuiteArea::Boost,_=>SuiteArea::Settings};
  egui::CollapsingHeader::new(format!("{}  {}",v.id,v.request)).id_salt(("explain",v.id.as_str())).show(ui,|ui|{
   ui.label(RichText::new(rm_help_status(a,&v.status)).color(if v.status=="SOURCE_TESTED_NOT_LIVE_VERIFIED"{MUTED}else{RED}));
   ui.label(rm_feature_impact(a,area));
   if ui.small_button(rm(a,"Open section","Otevřít sekci")).clicked(){area.go(a);}
  });
 }
 });
 if matched>70{caption(ui,rm(a,"Showing first 70 results. Narrow your search.","Zobrazeno prvních 70 výsledků. Upřesni hledání."));}
}
