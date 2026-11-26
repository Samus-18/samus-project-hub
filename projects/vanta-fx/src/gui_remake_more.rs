// Consolidated simple workspaces. Existing detailed tools remain accessible on demand.
fn rm_capture(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  heading(ui,rm(a,"Your recordings","Tvoje záznamy"));
  let rec=a.recorder.active();let replay=a.replay.active();
  ui.horizontal_wrapped(|ui|{
   if ui.add_sized([147.,39.],egui::Button::new(rm(a,"Take screenshot","Pořídit snímek")).fill(Color32::from_rgb(80,50,88))).clicked()&&!a.is_smoke{a.screenshot();}
   let rec_text=if rec{rm(a,"Stop recording","Ukončit záznam")}else{rm(a,"Start video","Spustit video")};
   if ui.add_enabled(!a.is_smoke&&!a.recorder.finishing()&&(!replay||rec),egui::Button::new(rec_text).fill(if rec{RED}else{Color32::from_rgb(86,42,67)})).clicked(){a.recording();}
   if ui.add_enabled(!a.is_smoke&&!a.recorder.busy(),egui::Button::new(if replay{rm(a,"Stop replay","Vypnout replay")}else{rm(a,"Start replay","Zapnout replay")})).clicked(){a.toggle_replay();}
  });
  ui.horizontal_wrapped(|ui|{
   if ui.add_enabled(replay&&a.replay.ready()&&!a.replay.saving(),egui::Button::new(rm(a,"Save last seconds","Uložit poslední sekundy"))).clicked(){a.save_replay();}
   if ui.small_button(rm(a,"Open videos folder","Otevřít složku videí")).clicked(){crate::record::open_folder("videos");}
   if ui.small_button(rm(a,"Open screenshots","Otevřít snímky")).clicked(){crate::record::open_folder("pictures");}
  });
  let status=if rec{rm(a,"RECORDING","NAHRÁVÁM")}else if a.recorder.finishing(){rm(a,"FINALIZING","DOKONČUJI")}else if replay{if a.replay.ready(){rm(a,"REPLAY READY","REPLAY PŘIPRAVEN")}else{rm(a,"BUFFERING","UKLÁDÁM DO PAMĚTI")}}else{rm(a,"IDLE","VYPNUTO")};
  ui.colored_label(if rec{RED}else if replay{V25_MINT}else{MUTED},status);
  caption(ui,rm(a,"Real desktop video, local files only. Capture usually uses CPU/GPU and storage; it cannot increase game FPS.","Skutečné video plochy, pouze místní soubory. Záznam zatěžuje CPU/GPU a disk; nemůže zvýšit FPS hry."));
 });
 ui.add_space(7.);
 egui::CollapsingHeader::new(rm(a,"My clips & screenshots","Moje klipy a snímky")).default_open(false).show(ui,|ui|capture_library_panel(a,ui));
 egui::CollapsingHeader::new(rm(a,"Replay and recording settings","Nastavení replay a nahrávání")).default_open(false).show(ui,|ui|capture_record_controls(a,ui));
 egui::CollapsingHeader::new(rm(a,"Montage editor","Editor sestřihu")).default_open(false).show(ui,|ui|montage_panel(a,ui));
 egui::CollapsingHeader::new(rm(a,"Audio & microphone consent","Zvuk a souhlas s mikrofonem")).default_open(false).show(ui,|ui|capture_audio_panel(a,ui));
 egui::CollapsingHeader::new(rm(a,"Encoder & resource budget","Enkodér a zátěž")).default_open(false).show(ui,|ui|{capture_encoder_panel(a,ui);capture_metrics_panel(a,ui);});
 egui::CollapsingHeader::new(rm(a,"Manual highlights and local OCR / voice","Ruční highlighty a místní OCR / hlas")).default_open(false).show(ui,|ui|{highlight_panel(a,ui);assist_panel(a,ui);});
}
fn rm_settings(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  heading(ui,rm(a,"Language & appearance","Jazyk a vzhled"));
  ui.horizontal_wrapped(|ui|{for(i,name)in [(0,"English"),(1,"Čeština")]{if ui.selectable_value(&mut a.settings.ui_language,i,name).changed(){a.save();}}});
  ui.add_space(5.);
  ui.horizontal_wrapped(|ui|{
   for(i,n)in [(0,rm(a,"Crimson","Červená")),(1,rm(a,"Midnight violet","Noční fialová")),(2,rm(a,"Arctic teal","Ledová tyrkysová"))]{if ui.selectable_value(&mut a.settings.ui_theme,i,n).changed(){a.save();}}
  });
  let anim=rm(a,"Animated interface (suspends while gaming/recording)","Animované rozhraní (pozastaví se při hraní či záznamu)");
  if ui.checkbox(&mut a.settings.animations,anim).changed(){a.save();}
  let zoom=rm(a,"Interface scale","Velikost rozhraní");
  if ui.add(egui::Slider::new(&mut a.settings.ui_zoom,0.75..=1.35).step_by(0.05).text(zoom)).changed(){a.save();}
  ui.horizontal_wrapped(|ui|{
   for(n,sz)in [(rm(a,"Compact","Malé"),[840.,560.]),(rm(a,"Standard","Střední"),[940.,620.]),(rm(a,"Spacious","Velké"),[1180.,760.])]{if ui.small_button(n).clicked()&&!a.is_smoke{a.set_main_window_size(ui.ctx(),sz);}}
  });
  let pressure=rm(a,"Reduce effects when VRAM pressure is high","Omezit efekty při vysokém zatížení VRAM");if ui.checkbox(&mut a.settings.pressure_quiet,pressure).changed(){a.save();}
  caption(ui,rm(a,"Appearance only changes Horizon, not the game. Actual performance is measured separately.","Vzhled mění pouze Horizon, nikoli hru. Výkon se měří samostatně."));
 });
 frame(PANEL).show(ui,|ui|{
  heading(ui,rm(a,"Game detection & monitoring","Detekce her a sledování"));
  let mut changed=false;let title=rm(a,"Detect running games","Vyhledávat spuštěné hry");changed|=ui.checkbox(&mut a.settings.detect_games,title).changed();
  let title=rm(a,"Measure game FPS (ETW)","Měřit FPS hry (ETW)");changed|=ui.checkbox(&mut a.settings.fps_monitoring,title).changed();
  if changed{a.save();}
  caption(ui,rm(a,"Detecting a game does not modify it. Measured values may be unavailable in certain games.","Detekce hru nemění. V některých hrách nemusí být měřená data dostupná."));
 });
 egui::CollapsingHeader::new(rm(a,"HUD appearance & advanced settings","Vzhled HUD a pokročilé nastavení")).show(ui,|ui|oled_settings(a,ui));
 egui::CollapsingHeader::new(rm(a,"Local privacy, permissions & shortcuts","Místní soukromí, oprávnění a zkratky")).show(ui,|ui|suite_safety_panel(a,ui));
 egui::CollapsingHeader::new(rm(a,"Technical settings and diagnostic tools","Technická nastavení a diagnostika")).show(ui,|ui|{settings(a,ui);suite_moved_settings(a,ui);});
}
fn rm_system(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  heading(ui,rm(a,"Your hardware","Tvůj hardware"));
  ui.label(RichText::new(&a.gpu).size(18.).strong());
  ui.horizontal_wrapped(|ui|{
   ui.label(format!("CPU: {} {}",std::thread::available_parallelism().map(|n|n.get()).unwrap_or(0),rm(a,"logical threads","logických vláken")));
   if a.telem.total_ram_gb>0.{ui.label(format!("RAM {:.1}/{:.1} GiB",a.telem.used_ram_gb,a.telem.total_ram_gb));}
  });
  caption(ui,rm(a,"System controls are opt-in. Changing settings does not guarantee FPS gains; restore is available for reversible changes.","Systémové funkce se zapínají ručně. Nastavení nezaručuje zlepšení FPS; vratné změny lze obnovit."));
  if ui.button(rm(a,"Open performance advisor","Otevřít poradce výkonu")).clicked(){SuiteArea::Boost.go(a);}
 });
 egui::CollapsingHeader::new(rm(a,"Hardware diagnostics & reversible controls","Diagnostika hardwaru a vratné změny")).show(ui,|ui|{performance_game_tools(a,ui);suite_moved_system_tune(a,ui);});
 egui::CollapsingHeader::new(rm(a,"CPU affinity (advanced)","Přiřazení CPU (pokročilé)")).show(ui,|ui|affinity_panel(a,ui));
}
fn rm_library(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,rm(a,"Select running game","Vyber spuštěnou hru"));rm_game(a,ui);
  if let Some(g)=a.telem.active_game.as_ref(){
   ui.label(RichText::new(&g.title).strong());
   caption(ui,&format!("PID {}",g.pid));
   caption(ui,if g.fullscreen_like{rm(a,"Fullscreen-size detected: own borderless output is not confirmed.","Zjištěno zobrazení přes celý monitor: výstup bez okrajů není zaručen.")}else{rm(a,"Windowed game detected; native capture may work.","Hra v okně: nativní snímání může fungovat.")});
  }else{caption(ui,rm(a,"Start Minecraft or another game, then refresh. No automatic control of game settings.","Spusť Minecraft či jinou hru a obnov seznam. Nastavení hry se automaticky nemění."));}
  if ui.button(rm(a,"Open game filters","Otevřít herní filtry")).clicked(){SuiteArea::Visual.go(a);}
 });
 egui::CollapsingHeader::new(rm(a,"Saved game library and launch shortcuts","Uložené hry a zkratky spouštění")).show(ui,|ui|oled_library(a,ui));
 egui::CollapsingHeader::new(rm(a,"Profiles, collections and backups","Profily, kolekce a zálohy")).show(ui,|ui|library_hub_panel(a,ui));
}
fn rm_macros(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();
 let blocked=a.macro_runner.running()||a.macro_armed_pid.is_some()||a.is_smoke;
 let names=a.settings.window_macro_profiles.get(&key)
  .map(|saved|saved.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
 let mut save=false;
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Simple macros / Record, Play, Stop","Jednoduchá makra / Nahrát, Přehrát, Stop"));
  caption(ui,local(ui,"Offline games and desktop apps only. Choose a window, record or load actions, then replay. F12 or focus loss stops immediately.","Jen offline hry a běžné aplikace. Vyber okno, nahraj nebo načti kroky a přehraj je. F12 nebo ztráta fokusu vše zastaví."));
  let s=&mut a.window_macro;
  let old_target=s.selected.as_ref().map(|t|(t.hwnd,t.pid,t.tid));
  ui.horizontal_wrapped(|ui|{
   if ui.button(local(ui,"Find windows","Najít okna")).clicked()&&!blocked{s.refresh();}
   egui::ComboBox::from_id_salt("rm-quick-macro-window")
    .selected_text(s.selected.as_ref().map(|v|v.title.as_str()).unwrap_or(local(ui,"Select window","Vyber okno")))
    .width(ui.available_width().min(390.)).show_ui(ui,|ui|{
     for t in &s.windows{ui.selectable_value(&mut s.selected,Some(t.clone()),&t.title);}
    });
  });
  if s.selected.as_ref().map(|t|(t.hwnd,t.pid,t.tid))!=old_target{s.consent=false;}
  ui.checkbox(&mut s.consent,local(ui,"I chose an offline/local app with no sensitive data","Vybral jsem offline/místní aplikaci bez citlivých údajů"));
  let busy=s.busy();
  ui.horizontal_wrapped(|ui|{
   let enabled=!blocked&&s.selected.is_some()&&s.consent&&!busy;
   if ui.add_enabled(enabled,egui::Button::new(local(ui,"● Record","● Nahrát"))).clicked(){
    if let Err(e)=s.start(true){s.status=e;}
   }
   if ui.add_enabled(enabled&&s.plan.validate().is_ok(),
    egui::Button::new(local(ui,"▶ Play","▶ Přehrát"))).clicked(){
    if let Err(e)=s.start(false){s.status=e;}
   }
   if ui.add_enabled(busy,egui::Button::new(local(ui,"■ STOP / F12","■ STOP / F12"))).clicked(){s.cancel();}
   if busy{ui.spinner();}
  });
  if busy{
   let state=if s.mode()=="Recording"{local(ui,"RECORDING","NAHRÁVÁNÍ")}
     else{local(ui,"PLAYING","PŘEHRÁVÁNÍ")};
   ui.colored_label(V25_MINT,format!("{state}  •  {} {}",s.progress(),
    local(ui,"steps","kroků")));
  }
  if !s.status.is_empty(){caption(ui,&s.status);}
  ui.separator();
  ui.horizontal_wrapped(|ui|{
   ui.label(local(ui,"Start from:","Začni od:"));
   for (i,en,cs) in [(0,"Single key","Jedna klávesa"),(1,"Two keys","Dvě klávesy"),(2,"One click","Jedno kliknutí")]{
    if ui.add_enabled(!busy,egui::Button::new(local(ui,en,cs))).clicked(){
     match crate::window_macro::starter(i){Ok(v)=>{s.plan=v;s.status=local(ui,"Example ready — not running","Ukázka připravena — neběží").into();},Err(e)=>s.status=e}
    }
   }
  });
  ui.horizontal_wrapped(|ui|{
   ui.add_enabled(!busy,egui::Slider::new(&mut s.plan.speed,50..=200).suffix("%").text(local(ui,"Speed","Rychlost")));
   ui.add_enabled(!busy,egui::Slider::new(&mut s.plan.repeat,1..=10).text(local(ui,"Repeat","Opakování")));
  });
  caption(ui,local(ui,"Only a selected foreground offline window. F12 aborts, focus loss aborts, no infinite loops.","Jen vybrané offline okno v popředí. F12 nebo ztráta fokusu makro zastaví. Bez nekonečných smyček."));
  ui.separator();
  ui.horizontal_wrapped(|ui|{
   ui.add(egui::TextEdit::singleline(&mut s.name).hint_text(local(ui,"Name this macro","Název makra")).char_limit(48));
   if ui.add_enabled(!busy&&s.plan.validate().is_ok(),
     egui::Button::new(local(ui,"Save","Uložit"))).clicked(){save=true;}
  });
  egui::ComboBox::from_id_salt("rm-quick-saved-macros")
   .selected_text(local(ui,"Load a saved macro","Načíst uložené makro"))
   .width(ui.available_width().min(350.)).show_ui(ui,|ui|{
    for name in &names{
     if ui.selectable_label(s.name==*name,name).clicked()&&!busy{
      if let Some(saved)=a.settings.window_macro_profiles.get(&key).and_then(|all|all.get(name)){
       s.plan=saved.clone();s.name=name.clone();s.status=local(ui,"Macro loaded — not playing","Makro načteno — nepřehrává se").into();
      }
     }
    }
   });
  caption(ui,&format!("{} {} / {:.1} {}",s.plan.events.len(),
    local(ui,"actions","akcí"),s.plan.duration() as f64/1000.,
    local(ui,"seconds total","sekund celkem")));
 });
 if save{
  let s=&mut a.window_macro;
  if let Err(e)=s.plan.save(&s.name,a.settings.window_macro_profiles.entry(key).or_default()){s.status=e;}
  else{s.status=local(ui,"Saved to this game's profile","Uloženo do profilu hry").into();a.save();}
 }
 egui::CollapsingHeader::new(local(ui,"Advanced macro timeline, mouse positions & import/export","Pokročilá časová osa, myš a import/export"))
  .default_open(false).show(ui,|ui|window_macro_panel(a,ui));
 egui::CollapsingHeader::new(local(ui,"Safety & keyboard shortcuts","Bezpečnost a klávesové zkratky"))
  .default_open(false).show(ui,|ui|shortcut_reference(a,ui));
}
fn rm_overlay(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,rm(a,"On-screen game HUD","Herní HUD na obrazovce"));
  ui.horizontal_wrapped(|ui|{
   let label=if a.overlay_open{rm(a,"Hide HUD","Skrýt HUD")}else{rm(a,"Show HUD","Zobrazit HUD")};
   if ui.button(label).clicked()&&!a.is_smoke{a.toggle_passive_hud();}
   if ui.button(rm(a,"Edit HUD placement","Upravit umístění HUD")).clicked()&&!a.is_smoke{a.overlay_open=true;a.overlay_edit_mode=true;}
  });
  let scale=rm(a,"Widget size","Velikost widgetů");
  if ui.add(egui::Slider::new(&mut a.settings.overlay_size_scale,0.78..=1.35).text(scale)).changed(){a.save();}
  let opacity=rm(a,"Transparency","Průhlednost");
  if ui.add(egui::Slider::new(&mut a.settings.overlay_opacity,0.72..=1.).text(opacity)).changed(){a.save();}
  caption(ui,rm(a,"This HUD is click-through. No automatic boost; actual measured values are displayed when available.","HUD propouští kliknutí. Nezvyšuje automaticky FPS; zobrazuje dostupné naměřené hodnoty."));
 });
 egui::CollapsingHeader::new(rm(a,"HUD widgets & saved game layouts","Widgety HUD a uložená rozložení")).show(ui,|ui|{hud_layout_panel(a,ui);hud_profile_panel(a,ui);});
 egui::CollapsingHeader::new(rm(a,"Overlay compatibility and shortcuts","Kompatibilita HUD a zkratky")).show(ui,|ui|{oled_overlay(a,ui);shortcut_reference(a,ui);});
}
fn rm_lighting(a:&mut BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new(rm(a,"GPU lighting intensity / live controls","Síla GPU osvětlení / živé ovládání")).show(ui,|ui|{
 let mut t=a.settings.lighting.clone();let mut changed=false;
 let enable=rm(a,"Enable extra lighting effects","Povolit dodatečné světelné efekty");
 changed|=ui.checkbox(&mut t.enabled,enable).changed();
 if t.enabled{
 for(index,en,cs)in [(0,"Shadow detail","Detaily ve stínech"),(1,"Glow / bloom","Záře / bloom"),(2,"Color bounce","Odrazy barev"),(3,"Fine details","Jemné detaily"),(4,"Cinematic contrast","Filmový kontrast"),(5,"Warmth","Teplota barev")]{let v=match index{0=>&mut t.shadow,1=>&mut t.glow,2=>&mut t.bounce,3=>&mut t.detail,4=>&mut t.film,_=>&mut t.warmth};let label=if a.settings.ui_language==1{cs}else{en};changed|=ui.add(egui::Slider::new(v,0.0..=1.).text(label)).changed();}
 }
 let label=rm(a,"GPU quality","Kvalita GPU");
 ui.label(label);
 ui.horizontal_wrapped(|ui|{
 for(level,en,cs)in [(3,"Adaptive","Adaptivní"),(0,"Full","Plná"),(1,"Balanced","Vyvážená"),(2,"Eco","Úsporná")]{let label=if a.settings.ui_language==1{cs}else{en};
 if ui.selectable_value(&mut a.settings.native_quality,level,label).changed(){a.save();a.refresh_native_mirror();}
 }
 });
 if changed{a.apply_native_lighting(t);}
 caption(ui,rm(a,"Changes Horizon's real native D3D11 effect layers only. Extra stages may reduce FPS; not true ray tracing.","Mění pouze skutečné D3D11 efekty Horizonu. Další vrstvy mohou snížit FPS; nejde o ray tracing."));
 });
}
fn rm_filter_status(a:&mut BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new(rm(a,"Output troubleshooting / active frame health","Diagnostika výstupu / stav snímků")).default_open(false).show(ui,|ui|{
 if let Some(w)=a.native_mirror.as_ref(){
  let d=w.diagnostics();let frames=w.submitted_frames();
  if d.live_look_rejected{ui.colored_label(RED,rm(a,"GPU shader rejected; last valid effect retained.","GPU shader byl odmítnut; zůstal poslední platný efekt."));}
  if d.paused{ui.colored_label(RED,rm(a,"Paused: return to selected game window.","Pozastaveno: vrať se do vybrané hry."));}
  else if frames==0{ui.colored_label(RED,rm(a,"No submitted frames. Check capture mode and foreground game.","Žádné vykreslené snímky. Zkontroluj snímání a hru v popředí."));}
  else{ui.colored_label(V25_MINT,format!("{}: {}",rm(a,"Frames submitted to Horizon output","Snímky odeslané do výstupu Horizonu"),frames));}
  caption(ui,rm(a,"Submitted frames indicate Horizon output activity only. They do not prove the original Minecraft window was filtered.","Odeslané snímky potvrzují činnost výstupu Horizonu, ne úpravu původního okna Minecraftu."));
  ui.label(format!("{}: {} / {} ms",rm(a,"Duplicate frames / capture age","Duplicitní snímky / stáří snímání"),d.duplicate_frames,d.worker_age_ms.map(|x|format!("{x:.0}")).unwrap_or_else(||"N/A".into())));
  let diagnosis=crate::filter_doctor::diagnose(Some(&d),frames);
  ui.label(crate::filter_doctor::message(diagnosis,a.settings.ui_language==1));
  if ui.small_button(rm(a,"Copy output diagnostics","Kopírovat stav výstupu")).clicked(){
   let fps=w.capture_average_fps().map(|v|format!("{v:.1}")).unwrap_or_else(||"N/A".into());
   let own=w.output_average_fps().map(|v|format!("{v:.1}")).unwrap_or_else(||"N/A".into());
   ui.ctx().copy_text(format!("Horizon output diagnosis: {:?}; frames={}; duplicate_frames={}; paused={}; rejected={}; capture_avg_fps={}; own_output_avg_fps={}; generated={}; mode={:?}; worker_age_ms={:?}. These are Horizon output metrics, not verified Minecraft engine FPS or game-image filter status.",
    diagnosis,frames,d.duplicate_frames,d.paused,d.live_look_rejected,fps,own,w.generated_frames(),w.mode(),d.worker_age_ms));
  }
 }else{caption(ui,rm(a,"Start native GPU output to see real frame health.","Spusť nativní GPU výstup pro skutečný stav snímků."));}
 if ui.add_enabled(a.telem.active_game.is_some(),egui::Button::new(rm(a,"Try separate output window (safe test)","Zkusit samostatné okno (bezpečný test)"))).clicked()&&!a.is_smoke{rm_start(a,crate::native_mirror::MirrorMode::SeparateWindow,1);}
 if ui.add_enabled(a.telem.active_game.is_some(),egui::Button::new(rm(a,"Try DXGI fallback (monitor-filling only)","Zkusit záložní DXGI (jen přes monitor)"))).clicked()&&!a.is_smoke{rm_start(a,crate::native_mirror::MirrorMode::DesktopDuplication,1);}
 caption(ui,rm(a,"Exclusive fullscreen, protected windows and some Vulkan/DX12 games are not supported universally; no Magpie or game injection.","Exkluzivní fullscreen, chráněná okna a některé Vulkan/DX12 hry nejsou univerzálně podporované; bez Magpie i injekce."));
 });
}

fn rm_visual_advanced_controls(a:&mut BoostApp,ui:&mut Ui){
 rm_intensity_and_color(a,ui);
 egui::CollapsingHeader::new(rm(a,"A/B looks, GPU budget, undo & favorites","A/B efekty, výkon GPU, návrat a oblíbené")).id_salt("rm-live-a-b").default_open(false).show(ui,|ui|{
  let key=a.profile_key();let pair=a.settings.look_pairs.get(&key);
  let stored=[pair.is_some_and(|p|p.stored(0)),pair.is_some_and(|p|p.stored(1))];
  ui.horizontal_wrapped(|ui|{
   for(i,en,cs) in [(0,"Save A","Uložit A"),(1,"Save B","Uložit B")]{
    if ui.button(rm(a,en,cs)).clicked()&&!a.is_smoke{a.save_look_slot(i);}
   }
   for(i,en,cs) in [(0,"Use A","Použít A"),(1,"Use B","Použít B")]{
    if ui.add_enabled(stored[i as usize],egui::Button::new(rm(a,en,cs))).clicked()&&!a.is_smoke{a.load_look_slot(i);}
   }
  });
  caption(ui,rm(a,"A/B stores actual shader presets, lighting and stages per game. Switching updates the running GPU output.","A/B ukládá skutečné GPU presety, osvětlení a vrstvy pro jednotlivé hry. Přepnutí mění běžící GPU výstup."));
  let mut quality=a.settings.native_quality;
  let old_quality=quality;
  ui.horizontal_wrapped(|ui|{
   for(i,en,cs) in [(3,"Adaptive","Adaptivní"),(0,"Full","Plná"),(1,"Balanced","Vyvážená"),(2,"Eco","Úsporná")]{
    let label=rm(a,en,cs);ui.selectable_value(&mut quality,i,label);
   }
  });
  if quality!=old_quality&&!a.is_smoke{a.settings.native_quality=quality;a.save();a.refresh_native_mirror();}
  let mut budget=a.settings.processing_percent;
  if ui.add(egui::Slider::new(&mut budget,50..=100).text(rm(a,"GPU stage budget","Limit GPU efektů")).suffix("%")).changed()&&!a.is_smoke{
   a.settings.processing_percent=crate::render_budget::valid(budget);
   a.settings.processing_profiles.insert(key,a.settings.processing_percent);
   a.save();a.refresh_native_mirror();
  }
  caption(ui,rm(a,"Lower values reduce Horizon's effect cost, not the game's render resolution. Applied to the actual GPU stages.","Nižší hodnota omezuje zátěž efektů Horizonu, ne rozlišení hry. Použije se při skutečném GPU zpracování."));
 });
}

// These are actual D3D11 look-stage weights and shader color-grade parameters.
// No simulated sliders or OS/GPU driver setting changes.
fn rm_intensity_and_color(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();
 let ids=a.settings.native_stack_profiles.get(&key)
  .cloned().unwrap_or_else(||crate::native_presets::PRESETS
   .get(a.settings.native_preset).map(|p|p.stages.to_vec()).unwrap_or_default());
 let mut tuning=a.settings.lighting.clone();
 let mut changed=false;
 frame(PANEL).show(ui,|ui|{
  heading(ui,rm(a,"Live filter strength","Síla filtru za běhu"));
  let mut amount=if ids.is_empty(){1.}else{
   ids.iter().enumerate().map(|(i,_)|tuning.layers.strength_at(&ids,i)).sum::<f32>()/ids.len() as f32
  };
  if ui.add(egui::Slider::new(&mut amount,0.0..=1.).text(rm(a,"Intensity","Intenzita")).show_value(true)).changed(){
   for i in 0..ids.len(){tuning.layers.set_at(&ids,i,amount);}
   changed=true;
  }
  ui.horizontal_wrapped(|ui|{
   for(v,en,cs)in [(0.0,"Off","Vypnout"),(0.50,"Half","Polovina"),(1.0,"Full","Plný")]{
    if ui.small_button(rm(a,en,cs)).clicked(){
     for i in 0..ids.len(){tuning.layers.set_at(&ids,i,v);}
     changed=true;
    }
   }
  });
  caption(ui,rm(a,"Controls the real GPU shader stages of this look without changing Minecraft settings.","Ovládá skutečné GPU shaderové vrstvy vybraného efektu, nemění nastavení Minecraftu."));
  egui::CollapsingHeader::new(rm(a,"Color, contrast, exposure and temperature","Barvy, kontrast, expozice a teplota")).id_salt("rm-color-grade").show(ui,|ui|{
   let mut enable=tuning.grade.enabled;
   if ui.checkbox(&mut enable,rm(a,"Enable live color grade","Zapnout úpravu barev")).changed(){tuning.grade.enabled=enable;changed=true;}
   if enable{
    for(i,en,cs)in [(0,"Saturation","Sytost"),(1,"Contrast","Kontrast"),(2,"Exposure","Expozice"),(3,"Temperature","Teplota")]{
     let value=match i{0=>&mut tuning.grade.saturation,1=>&mut tuning.grade.contrast,2=>&mut tuning.grade.exposure,_=>&mut tuning.grade.temperature};
     let range=match i{0=>0.0..=2.0,1=>0.5..=1.5,2=>-2.0..=2.0,_=>-1.0..=1.0};
     let text=if a.settings.ui_language==1{cs}else{en};
     changed|=ui.add(egui::Slider::new(value,range).text(text)).changed();
    }
   }
  });
 });
 if changed&&!a.is_smoke{a.apply_native_lighting(tuning);}
}

#[derive(Clone)]
struct McReadinessCache{pid:u32,at:std::time::Instant,value:Result<crate::win::CapturePreflight,String>}
fn rm_mc_detected(a:&BoostApp)->bool{
 a.telem.active_game.as_ref().is_some_and(|g|crate::win::is_minecraft_java_window(&g.path,&g.title,&g.class_name)||
  g.title.to_ascii_lowercase().contains("minecraft")||g.path.file_name().is_some_and(|p|p.to_string_lossy().to_ascii_lowercase().contains("minecraft")))
}
fn rm_update_mc_profile(a:&mut BoostApp,kind:Option<u8>)->Result<(),String>{
 let key=a.profile_key();
 if a.telem.active_game.is_none(){
  return Err("Select the running Minecraft window before applying a preset".into());
 }
 // The pending profile is never saved until the real running GPU pipeline
 // accepts the effect. A bad shader cannot leave a misleading active badge.
 let mut candidate=a.settings.clone();
 match kind{
  Some(which)=>crate::minecraft_profile::apply(&mut candidate,&key,which)?,
  None=>crate::minecraft_profile::restore(&mut candidate,&key)?,
 }
 if let Some(w)=a.native_mirror.as_ref().filter(|w|w.running()){
  let ids=candidate.native_stack_profiles.get(&key).map(Vec::as_slice);
  match ids{
   Some(stages)=>w.set_stack(stages,candidate.native_quality,&candidate.lighting)?,
   None=>w.set_look(candidate.native_preset,candidate.native_quality,&candidate.lighting)?,
  }
 }
 a.settings=candidate;
 a.save();
 a.native_mirror_status=if a.native_mirror.as_ref().is_some_and(|w|w.running()){
   rm(a,"Minecraft GPU look accepted by the running native output","GPU efekt pro Minecraft přijat běžícím nativním výstupem").into()
 }else{
   rm(a,"Minecraft look saved; start GPU output to preview it","Efekt pro Minecraft uložen; spusť GPU výstup").into()
 };
 Ok(())
}

fn rm_mc_presets(a:&mut BoostApp,ui:&mut Ui){
 if !rm_mc_detected(a){return;}
 frame(PANEL).show(ui,|ui|{
 heading(ui,rm(a,"Minecraft Java: reversible GPU looks","Minecraft Java: vratné GPU efekty"));
 let key=a.profile_key();
 let selected=a.settings.minecraft_active.get(&key).copied();
 ui.horizontal_wrapped(|ui|{
  for(i,en,cs)in [(0,"Vanilla+","Vanilla+"),(1,"Cave visibility","Viditelnost jeskyní"),(2,"Cinematic","Filmový"),(3,"Low GPU cost","Nízká zátěž GPU")]{
   if ui.selectable_label(selected==Some(i),rm(a,en,cs)).clicked()&&!a.is_smoke{
    if let Err(e)=rm_update_mc_profile(a,Some(i)){a.toast(e);}
   }
  }
 });
 let can_undo=a.settings.minecraft_undo.contains_key(&key);
 if ui.add_enabled(can_undo,egui::Button::new(rm(a,"Restore original game look","Obnovit původní vzhled hry"))).clicked()&&!a.is_smoke{
  if let Err(e)=rm_update_mc_profile(a,None){a.toast(e);}
 }
 let (en,cs)=match selected{
  Some(0)=>("Subtle vanilla color and contrast. Low-to-moderate GPU overhead.","Jemné vanilla barvy a kontrast. Nízká až střední zátěž GPU."),
  Some(1)=>("Dark-region visibility and mild gamma tuning; NOT fullbright or night vision.","Viditelnost tmavších oblastí a mírná gamma; NENÍ fullbright ani noční vidění."),
  Some(2)=>("Warmer cinema tone with glow; may cost more GPU time.","Teplejší filmové barvy se září; mohou zvýšit zátěž GPU."),
  Some(3)=>("Lower processing budget and a simple detail pass; still no engine FPS boost.","Nižší rozpočet zpracování a jednoduchý detailní efekt; FPS hry se nezvyšují."),
  _=>("Choose a reversible look. Applied to Horizon GPU output only.","Vyber vratný efekt. Mění pouze GPU výstup Horizonu."),
 };
 caption(ui,rm(a,en,cs));
 caption(ui,rm(a,"Choose a profile, then start Horizon native output below. Choosing a look alone does not change the Minecraft window.","Vyber profil a potom spusť nativní výstup Horizonu níže. Samotný výběr vzhled okna Minecraftu nemění."));
 caption(ui,rm(a,"These are not mods; Minecraft files, process memory and engine FPS remain unchanged.","Nejsou to mody; soubory, paměť a FPS enginu Minecraftu zůstávají beze změny."));
 });
}
fn rm_game_preflight(a:&mut BoostApp,ui:&mut Ui){
 let Some(game)=a.telem.active_game.as_ref()else{return;};
 let pid=game.pid;
 let id=egui::Id::new("rm-preflight-window");
 let mut refresh=false;
 egui::CollapsingHeader::new(rm(a,"Read-only window diagnostics","Pasivní diagnostika okna"))
 .default_open(false).show(ui,|ui|{
  let now=std::time::Instant::now();
  let mut cache=ui.data(|d|d.get_temp::<McReadinessCache>(id));
  if cache.as_ref().is_none_or(|v|v.pid!=pid||now.saturating_duration_since(v.at)>std::time::Duration::from_secs(10)){
   cache=Some(McReadinessCache{pid,at:now,value:crate::win::capture_preflight(pid)});
  }
  if let Some(snapshot)=cache.as_ref(){
   match &snapshot.value{
    Ok(p)=>{
     ui.colored_label(V25_MINT,format!("PID {}  |  {} x {}  |  {} DPI  |  {}",p.pid,p.width,p.height,p.dpi,p.class_name));
     ui.label(&p.title);
     ui.label(format!("{}: {} x {}",rm(a,"Monitor resolution","Rozlišení monitoru"),p.monitor_width,p.monitor_height));
     ui.label(if p.foreground{rm(a,"Game is foreground","Hra je v popředí")}else{rm(a,"Game is background; output may be paused","Hra je na pozadí; výstup může být pozastaven")});
     if p.monitor_filling{caption(ui,rm(a,"The window fills its monitor, but true exclusive fullscreen is unverified.","Okno vyplňuje monitor, ale exkluzivní fullscreen není ověřen."));}
     if p.minecraft_java{ui.colored_label(V25_MINT,rm(a,"Minecraft Java / GLFW window recognized","Rozpoznané okno Minecraft Java / GLFW"));}
     caption(ui,rm(a,"Read-only metadata; no screenshot, capture, focus or process-memory access.","Pouze metadata; bez snímku, snímání, fokusu i přístupu do paměti hry."));
     if ui.small_button(rm(a,"Copy diagnostics","Kopírovat diagnostiku")).clicked(){
      ui.ctx().copy_text(format!("Horizon game window: PID={} class={} resolution={}x{} monitor={}x{} DPI={} foreground={} monitor_filling={} minecraft_java={}",
       p.pid,p.class_name,p.width,p.height,p.monitor_width,p.monitor_height,p.dpi,p.foreground,p.monitor_filling,p.minecraft_java));
     }
    },
    Err(e)=>{ui.colored_label(RED,e);}
   }
  }
  refresh=ui.small_button(rm(a,"Refresh diagnostics","Obnovit diagnostiku")).clicked();
  if let Some(p)=cache.as_ref().and_then(|v|v.value.as_ref().ok()){
    let advice=crate::capture_start_guard::classify(crate::resource_guard::read(),p);
    match advice{
     Err(e)=>{ui.colored_label(RED,e);},
     Ok(crate::capture_start_guard::LaunchAdvice::LowRamWarning)=>{
       ui.colored_label(MUTED,rm(a,"Low free memory: postpone native capture for better game smoothness.","Málo volné paměti: odlož snímání, aby hra nezadrhávala."));
     },
     Ok(crate::capture_start_guard::LaunchAdvice::BackgroundGame)=>{
       caption(ui,rm(a,"Target window is valid but is not foreground.","Okno je platné, ale není v popředí."));
     },
     Ok(crate::capture_start_guard::LaunchAdvice::MonitorFillingUnverified)=>{
       caption(ui,rm(a,"Monitor-filling target: test independent WGC preview first.","Okno přes monitor: nejdřív otestuj samostatný WGC náhled."));
     },
     Ok(crate::capture_start_guard::LaunchAdvice::Ready)=>{
       ui.colored_label(V25_MINT,rm(a,"Window metadata is valid (GPU capture still untested).","Metadata okna jsou v pořádku (GPU snímání není ověřeno)."));
     },
    }
  }
  if refresh{cache=Some(McReadinessCache{pid,at:now,value:crate::win::capture_preflight(pid)});}
  if let Some(v)=cache{ui.data_mut(|d|d.insert_temp(id,v));}
 });
}


#[derive(Clone)]
struct RmMemoryCache{at:std::time::Instant,mem:Option<crate::resource_guard::PhysicalMemory>}
fn rm_cached_memory_warning(a:&BoostApp,ui:&mut Ui){
 let id=egui::Id::new("rm-physical-memory-2s");
 let now=std::time::Instant::now();
 let cached=ui.data(|d|d.get_temp::<RmMemoryCache>(id));
 let state=match cached{
  Some(s)if now.saturating_duration_since(s.at)<std::time::Duration::from_secs(2)=>s,
  _=>RmMemoryCache{at:now,mem:crate::resource_guard::read()}
 };
 if let Some(mem)=state.mem{
  if mem.free_mib<1536{
   ui.colored_label(if mem.free_mib<512{RED}else{MUTED},
    format!("{}: {} MiB • {}",
      rm(a,"Free RAM","Volná RAM"),mem.free_mib,
      if mem.free_mib<512{rm(a,"GPU output blocked to avoid game stutter","GPU výstup zablokován kvůli ochraně hry")}
      else{rm(a,"Low-memory warning","Upozornění na málo paměti")}
    ));
  }
 }
 ui.data_mut(|d|d.insert_temp(id,state));
}
