
// Vanta FX / V138: clear direct-control desktop. Intentionally no home tiles,
// pop-up feature menu, giant gradient hero, or nested accordion navigation.
// The old expert widgets remain in Rust for backwards compatibility with profiles.
const VF_BACKGROUND:Color32=Color32::from_rgb(11,12,16);
const VF_PANEL:Color32=Color32::from_rgb(20,22,29);
const VF_SIDEBAR:Color32=Color32::from_rgb(15,16,21);
const VF_EDGE:Color32=Color32::from_rgb(43,46,56);
const VF_ACCENT:Color32=Color32::from_rgb(235,95,106);
const VF_MUTED:Color32=Color32::from_rgb(156,161,174);
const VF_FOREGROUND:Color32=Color32::from_rgb(237,240,245);
const VF_SUCCESS:Color32=Color32::from_rgb(112,209,168);
const VF_NAV_AREAS:[SuiteArea;10]=[
 SuiteArea::Home,SuiteArea::Visual,SuiteArea::Tools,
 SuiteArea::Macros,SuiteArea::Overlay,SuiteArea::Capture,SuiteArea::Library,
 SuiteArea::System,SuiteArea::Settings,SuiteArea::Lab,
];
fn vf_local(a:&BoostApp,en:&'static str,cs:&'static str)->&'static str{
 if a.settings.ui_language==1{cs}else{en}
}
fn vf_frame(ui:&mut Ui,body:impl FnOnce(&mut Ui)){
 Frame::new().fill(VF_PANEL).stroke(Stroke::new(1.,VF_EDGE))
  .corner_radius(egui::CornerRadius::same(12))
  .inner_margin(egui::Margin::symmetric(18,15))
  .show(ui,|ui|{ui.set_min_width(ui.available_width());body(ui);});
}
fn vf_section(ui:&mut Ui,name:&str){
 ui.label(RichText::new(name).size(16.).strong().color(VF_FOREGROUND));
 ui.add_space(9.);
}
fn vf_hint(ui:&mut Ui,msg:&str){
 ui.label(RichText::new(msg).size(12.).color(VF_MUTED));
}
fn vf_button(ui:&mut Ui,label:&str,primary:bool,enabled:bool)->bool{
 let button=egui::Button::new(RichText::new(label).size(13.).strong())
  .fill(if primary{VF_ACCENT}else{Color32::from_rgb(31,34,43)})
  .stroke(Stroke::new(1.,if primary{VF_ACCENT}else{VF_EDGE}))
  .corner_radius(8.);
 ui.add_enabled(enabled,button).clicked()
}
fn vf_title(app:&BoostApp)->(&'static str,&'static str){
 match suite_current(app){
  SuiteArea::Home=>(vf_local(app,"Overview","Přehled"),vf_local(app,"Your game and the controls you actually need.","Hra a ovládání, které opravdu potřebuješ.")),
  SuiteArea::Visual=>(vf_local(app,"Upscaling & Filters","Upscaling a filtry"),vf_local(app,"Scale your output and pick real GPU effects on one page.","Zvětšení obrazu a GPU efekty na jedné stránce.")),
  SuiteArea::Boost=>(vf_local(app,"Upscaling","Upscaling"),vf_local(app,"Capture, scale and present on your GPU.","Snímání, zvětšení a vykreslení na GPU.")),
  SuiteArea::Tools=>(vf_local(app,"Game Tools","Herní nástroje"),vf_local(app,"Crosshair, timers and practical gaming utilities.","Zaměřovač, časovače a herní pomůcky.")),
  SuiteArea::Macros=>(vf_local(app,"Macro Recorder","Záznam maker"),vf_local(app,"Record. Play. Stop. Offline/local apps only.","Nahraj. Přehraj. Zastav. Jen offline aplikace.")),
  SuiteArea::Overlay=>(vf_local(app,"In-game HUD","Herní HUD"),vf_local(app,"One click-through overlay with live measurements.","Jeden průchozí overlay se skutečnými hodnotami.")),
  SuiteArea::Capture=>(vf_local(app,"Record & Replay","Záznam a replay"),vf_local(app,"Capture real screenshots and video to local files.","Skutečné snímky a videa do místních souborů.")),
  SuiteArea::Library=>(vf_local(app,"Games & Profiles","Hry a profily"),vf_local(app,"Running games and saved preferences.","Spuštěné hry a uložená nastavení.")),
  SuiteArea::System=>(vf_local(app,"Hardware","Hardware"),vf_local(app,"Diagnostics and reversible system settings.","Diagnostika a vratné nastavení systému.")),
  SuiteArea::Settings=>(vf_local(app,"Preferences","Nastavení"),vf_local(app,"Appearance, language, capture permissions.","Vzhled, jazyk a oprávnění snímání.")),
  _=>(vf_local(app,"Developer lab","Vývojářská laboratoř"),"")
 }
}
fn vf_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
 ui.horizontal(|ui|{
  let (logo,drag)=ui.allocate_exact_size(egui::vec2(250.,40.),egui::Sense::click_and_drag());
  let p=ui.painter();
  let c=logo.left_center()+egui::vec2(19.,0.);
  p.rect_stroke(egui::Rect::from_center_size(c,egui::vec2(30.,30.)),
   8.,Stroke::new(1.1,VF_ACCENT),StrokeKind::Inside);
  let v=[c+egui::vec2(-8.,-7.),c+egui::vec2(0.,8.),c+egui::vec2(8.,-7.)];
  p.add(egui::Shape::line(v.to_vec(),Stroke::new(2.7,VF_ACCENT)));
  p.text(logo.left_center()+egui::vec2(44.,-7.),
   egui::Align2::LEFT_CENTER,"VANTA FX",egui::FontId::proportional(17.),VF_FOREGROUND);
  p.text(logo.left_center()+egui::vec2(45.,11.),
   egui::Align2::LEFT_CENTER,"NATIVE GAME TOOLS",egui::FontId::proportional(9.),VF_MUTED);
  if drag.dragged(){ctx.send_viewport_cmd(ViewportCommand::StartDrag);}
  ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
   if ui.add_sized([31.,27.],egui::Button::new("×").fill(VF_PANEL)).clicked(){
    ctx.send_viewport_cmd(ViewportCommand::Close);
   }
   if ui.add_sized([31.,27.],egui::Button::new("□").fill(VF_PANEL)).clicked(){
    let maximized=ctx.input(|i|i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
   }
   if ui.add_sized([31.,27.],egui::Button::new("−").fill(VF_PANEL)).clicked(){
    ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
   }
   ui.add_space(11.);
   let running=app.native_mirror.as_ref().is_some_and(|w|w.running());
   ui.colored_label(if running{VF_SUCCESS}else{VF_MUTED},
      if running{vf_local(app,"OUTPUT ACTIVE","VÝSTUP AKTIVNÍ")}
      else{vf_local(app,"READY","PŘIPRAVENO")});
  });
 });
}
fn vf_nav_label(app:&BoostApp,area:SuiteArea)->&'static str{
 match area {
  SuiteArea::Home=>vf_local(app,"Overview","Přehled"),
  SuiteArea::Visual=>vf_local(app,"Upscale + filters","Upscale + filtry"),
  SuiteArea::Boost=>vf_local(app,"Upscaling","Upscaling"),
  SuiteArea::Tools=>vf_local(app,"Game tools","Herní nástroje"),
  SuiteArea::Macros=>vf_local(app,"Macros","Makra"),
  SuiteArea::Overlay=>vf_local(app,"HUD","HUD"),
  SuiteArea::Capture=>vf_local(app,"Recording","Záznam"),
  SuiteArea::Library=>vf_local(app,"Games","Hry"),
  SuiteArea::System=>vf_local(app,"Hardware","Hardware"),
  SuiteArea::Settings=>vf_local(app,"Settings","Nastavení"),
  SuiteArea::Lab=>vf_local(app,"Developer lab","Vývojové nástroje"),
 }
}
fn vf_nav_item(app:&mut BoostApp,ui:&mut Ui,area:SuiteArea){
 let current=suite_current(app)==area;
 let text=vf_nav_label(app,area);
 let button=egui::Button::new(RichText::new(text).size(13.)
    .color(if current{VF_FOREGROUND}else{VF_MUTED}))
   .fill(if current{Color32::from_rgb(47,30,37)}else{VF_SIDEBAR})
   .stroke(Stroke::new(1.,if current{Color32::from_rgb(107,47,59)}else{VF_SIDEBAR}))
   .corner_radius(8.);
 if ui.add_sized([ui.available_width(),35.],button).clicked(){area.go(app);}
}
fn vf_navigation(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(8.);
 ui.label(RichText::new("WORKSPACE").size(10.).strong().color(VF_MUTED));
 ui.add_space(8.);
 for area in VF_NAV_AREAS.iter().copied(){
  if area==SuiteArea::Tools || area==SuiteArea::Library || area==SuiteArea::Lab{
   ui.add_space(13.);
   ui.label(RichText::new(if area==SuiteArea::Tools{"GAME TOOLS"}else if area==SuiteArea::Lab{"EXPERIMENTAL"}else{"MANAGE"})
     .size(10.).strong().color(VF_MUTED));ui.add_space(6.);
  }
  vf_nav_item(app,ui,area);
  ui.add_space(3.);
 }
 ui.add_space(9.);
 ui.separator();
 ui.horizontal(|ui|{
  let new_language=if app.settings.ui_language==1{0}else{1};
  if ui.add_sized([56.,28.],egui::Button::new(
     if new_language==1{"CZ"}else{"EN"})).clicked(){
   app.settings.ui_language=new_language;app.save();
  }
  ui.label(RichText::new("V138 / RUST").size(10.).color(VF_MUTED));
 });
}
fn vf_page_head(app:&BoostApp,ui:&mut Ui){
 let (title,desc)=vf_title(app);
 ui.label(RichText::new(title).size(26.).strong().color(VF_FOREGROUND));
 vf_hint(ui,desc);
 ui.add_space(18.);
}
fn vf_game_selector(app:&mut BoostApp,ui:&mut Ui){
 vf_section(ui,vf_local(app,"Selected game","Vybraná hra"));
 rm_game(app,ui);
 if let Some(g)=app.telem.active_game.as_ref(){
  ui.label(RichText::new(format!("{}  ·  PID {}",trim(&g.title,55),g.pid))
       .size(12.).color(VF_MUTED));
 }else{
  vf_hint(ui,vf_local(app,"Start a game, refresh and select its window. No game settings are changed.",
   "Spusť hru, obnov seznam a vyber její okno. Nic v nastavení hry automaticky neměním."));
 }
}
fn vf_status(app:&mut BoostApp,ui:&mut Ui){
 let Some(worker)=app.native_mirror.as_ref() else{
  vf_hint(ui,vf_local(app,"GPU output is off.","GPU výstup je vypnutý."));return;
 };
 let frames=worker.submitted_frames();
 let d=worker.diagnostics();
 let active=worker.running()&&frames>0;
 ui.colored_label(if active{VF_SUCCESS}else{VF_MUTED},
   if active{vf_local(app,"GPU frames submitted","GPU snímky odesílány")}
   else{vf_local(app,"Waiting for game frames","Čekám na snímky hry")});
 let diagnosed=crate::filter_doctor::diagnose(Some(&d),frames);
 if diagnosed!=crate::filter_doctor::Status::OutputFrames{
  ui.colored_label(VF_MUTED,crate::filter_doctor::message(diagnosed,app.settings.ui_language==1));
 }
 if frames>0{ui.label(format!("{}: {frames}",vf_local(app,"Own submitted frames","Odeslané snímky výstupu")));}
}
fn vf_output_action(app:&mut BoostApp,ui:&mut Ui,mode:u8){
 let enabled=!app.is_smoke&&(app.native_mirror.is_some()||app.telem.active_game.is_some());
 let is_running=app.native_mirror.is_some();
 let start=vf_local(app,"START GPU OUTPUT","SPUSTIT GPU VÝSTUP");
 let stop=vf_local(app,"STOP GPU OUTPUT","VYPNOUT GPU VÝSTUP");
 if vf_button(ui,if is_running{stop}else{start},true,enabled){
  if is_running{app.stop_native_mirror();}
  else{rm_start(app,rm_mode(mode),1);}
 }
}

fn vf_quick_preset(app:&mut BoostApp,ui:&mut Ui){
 vf_section(ui,vf_local(app,"Quick look","Rychlý vzhled"));
 ui.horizontal_wrapped(|ui|{
  for i in [0usize,2,4,7]{
   let (preset,en,cs,_,_,_)=RM_LOOKS[i];
   let selected=app.settings.native_preset==preset;
   let label=vf_local(app,en,cs);
   if ui.selectable_label(selected,label).clicked()&&!app.is_smoke{
    app.select_native_preset(preset);
   }
  }
 });
}
fn vf_home(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{vf_game_selector(app,ui);});
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"GPU output","GPU výstup"));
  ui.horizontal_wrapped(|ui|{
   let key=app.profile_key();
   let mut mode=app.settings.native_output_modes.get(&key).copied().unwrap_or(0).min(2);
   let old=mode;
   for(i,en,cs)in [(0u8,"Game view","Zobrazení hry"),(1,"Preview window","Náhled v okně"),(2,"DXGI fallback","Záložní DXGI")]{
    ui.selectable_value(&mut mode,i,vf_local(app,en,cs));
   }
   if old!=mode&&!app.is_smoke{app.settings.native_output_modes.insert(key,mode);app.save();}
  });
  ui.add_space(10.);
  let key=app.profile_key();
  let mode=app.settings.native_output_modes.get(&key).copied().unwrap_or(0).min(2);
  ui.horizontal_wrapped(|ui|{
   vf_output_action(app,ui,mode);
   vf_status(app,ui);
  });
  ui.add_space(8.);
  vf_hint(ui,vf_local(app,
   "Game view is an external WGC/DXGI output, not an injected in-game filter. Use windowed/borderless mode.",
   "Herní zobrazení je samostatný WGC/DXGI výstup, nikoli filtr vložený do hry. Použij okno / bez okrajů."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_quick_preset(app,ui);
  ui.separator();
  vf_section(ui,vf_local(app,"Live performance","Aktuální výkon"));
  ui.horizontal_wrapped(|ui|{
   ui.label(format!("{}: {}",vf_local(app,"Game FPS","FPS hry"),fps_text(app,"fps")));
   ui.add_space(18.);
   ui.label(format!("{}: {}",vf_local(app,"Frame time","Čas snímku"),fps_text(app,"frametime")));
   ui.add_space(18.);
   let own=app.output_rates.snapshot(std::time::Instant::now());
   let result=own.map_or_else(||"—".to_string(),|s|format!("{:.0}",s.real));
   ui.label(format!("{}: {result}",vf_local(app,"Own real output FPS","FPS skutečného výstupu")));
  });
  vf_hint(ui,vf_local(app,
   "No fabricated FPS gain: game FPS and independent GPU output FPS are different measurements.",
   "Žádný vymyšlený nárůst FPS: FPS hry a nezávislého GPU výstupu jsou jiné hodnoty."));
 });
}
fn vf_quality(app:&mut BoostApp,ui:&mut Ui){
 vf_section(ui,vf_local(app,"GPU workload","Zátěž GPU"));
 let mut q=app.settings.native_quality;
 ui.horizontal_wrapped(|ui|{
  for(i,en,cs)in [(3u8,"Adaptive","Adaptivní"),(2,"Eco","Úsporný"),(1,"Balanced","Vyvážený"),(0,"Full","Plný")]{
   ui.selectable_value(&mut q,i,vf_local(app,en,cs));
  }
 });
 if q!=app.settings.native_quality&&!app.is_smoke{app.settings.native_quality=q;app.save();app.refresh_native_mirror();}
 vf_hint(ui,vf_local(app,"Eco is best for weak GPUs. More passes and higher scaling can lower FPS.",
 "Úsporný režim je nejlepší pro slabší GPU. Více efektů a větší rozlišení může snížit FPS."));
}
fn vf_scaling(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_game_selector(app,ui);
  ui.separator();
  vf_section(ui,vf_local(app,"Output mode","Režim výstupu"));
  let key=app.profile_key();
  let mut mode=app.settings.native_output_modes.get(&key).copied().unwrap_or(0).min(2);
  let prev=mode;
  ui.horizontal_wrapped(|ui|{
   for(i,en,cs)in [(0u8,"Game view","Přes hru"),(1,"Separate preview","Samostatný náhled"),(2,"DXGI fallback","Záložní DXGI")]{
    ui.selectable_value(&mut mode,i,vf_local(app,en,cs));
   }
  });
  if mode!=prev&&!app.is_smoke{app.settings.native_output_modes.insert(key,mode);app.save();}
  ui.add_space(12.);
  vf_output_action(app,ui,mode);
  vf_status(app,ui);
  vf_hint(ui,vf_local(app,
   "In-game mode requires a visible foreground game in windowed/borderless mode. No DLL injection.",
   "Režim přes hru vyžaduje viditelnou hru v popředí a režim okno / bez okrajů. Bez injekce DLL."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_quality(app,ui);
  ui.separator();
  vf_section(ui,vf_local(app,"Scaling filter","Filtr zvětšení"));
  let mut algo=app.settings.native_scale_mode;
  ui.horizontal_wrapped(|ui|{
   for(id,en,cs)in [(1u8,"Fast","Rychlý"),(2,"Smooth","Jemný"),(8,"FSR1","FSR1"),(9,"NIS","NIS"),(10,"Lanczos","Lanczos")]{
    ui.selectable_value(&mut algo,id,vf_local(app,en,cs));
   }
  });
  if algo!=app.settings.native_scale_mode&&!app.is_smoke{app.apply_scale_algorithm(algo);}
  ui.add_space(9.);
  let old=app.settings.preview_fps_limit;
  let mut limit=old;
  ui.horizontal_wrapped(|ui|{
   ui.label(vf_local(app,"Output limit","Limit výstupu"));
   for fps in [0u32,45,60,75,90,120]{
    let label=if fps==0{"Off".to_owned()}else{format!("{fps}")};
    ui.selectable_value(&mut limit,fps,label);
   }
  });
  if limit!=old&&!app.is_smoke{
   app.settings.preview_fps_limit=limit;
   app.preview_guard.reset();
   if let Some(worker)=app.native_mirror.as_ref(){worker.set_preview_limit(limit);}
   app.save();
  }
  vf_hint(ui,vf_local(app,"This caps only our output — it does not set the game's FPS limit.",
   "Omezuje jen náš výstup, nikoliv FPS samotné hry."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Experimental frame generation","Experimentální generování snímků"));
  let label=vf_local(app,"Allow FG on this PC (experimental)","Povolit FG na tomto PC (experimentální)");
  if ui.checkbox(&mut app.settings.experimental_fg_allowed,label).changed(){app.save();}
  ui.horizontal_wrapped(|ui|{
   for f in [2u8,3]{
    if vf_button(ui,&format!("FG {f}×"),false,
      app.settings.experimental_fg_allowed&&!app.is_smoke&&app.telem.active_game.is_some()){
     rm_start(app,crate::native_mirror::MirrorMode::BorderlessScreen,f);
    }
   }
  });
  vf_hint(ui,vf_local(app,
   "Interpolation increases the number of output frames, not the game's rendered FPS. May worsen latency or smoothness.",
   "Interpolace přidává snímky výstupu, ne FPS herního enginu. Může zhoršit latenci i plynulost."));
 });
}
fn vf_filters(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"GPU effect presets","Předvolby GPU efektů"));
  vf_hint(ui,vf_local(app,"Choose directly. The effect applies to the independent GPU output.",
   "Vyber přímo. Efekt se použije na nezávislý GPU výstup."));
  ui.add_space(10.);
  for row in RM_LOOKS.chunks(4){
   ui.horizontal_wrapped(|ui|{
    for (preset,en,cs,_,_,_) in row{
     let selected=app.settings.native_preset==*preset;
     let label=vf_local(app,en,cs);
     if ui.add_sized([146.,33.],egui::Button::new(label)
       .fill(if selected{Color32::from_rgb(92,39,48)}else{Color32::from_rgb(32,34,43)})
       .stroke(Stroke::new(1.,if selected{VF_ACCENT}else{VF_EDGE}))).clicked()&&!app.is_smoke{
      app.select_native_preset(*preset);
     }
    }
   });
   ui.add_space(5.);
  }
  let key=app.profile_key();
  let ids=app.settings.native_stack_profiles.get(&key).cloned()
   .unwrap_or_else(||crate::native_presets::PRESETS.get(app.settings.native_preset)
      .map_or_else(Vec::new,|p|p.stages.to_vec()));
  if !ids.is_empty(){
   let mut intensity=ids.iter().enumerate()
      .map(|(i,_)|app.settings.lighting.layers.strength_at(&ids,i))
      .sum::<f32>()/ids.len() as f32;
   ui.add_space(9.);
   ui.horizontal_wrapped(|ui|{
    ui.label(vf_local(app,"Strength","Síla"));
    if ui.add(egui::Slider::new(&mut intensity,0.0..=1.0).fixed_decimals(2)).changed()&&!app.is_smoke{
     let mut light=app.settings.lighting.clone();
     for i in 0..ids.len(){light.layers.set_at(&ids,i,intensity);}
     app.apply_native_lighting(light);
    }
    if ui.small_button(vf_local(app,"Reset","Reset")).clicked()&&!app.is_smoke{
     let mut light=app.settings.lighting.clone();
     for i in 0..ids.len(){light.layers.set_at(&ids,i,1.);}
     app.apply_native_lighting(light);
    }
   });
  }
  ui.add_space(10.);
  let library=vf_local(app,"MORE FILTERS / full library","VÍCE FILTRŮ / celá knihovna");
  egui::CollapsingHeader::new(format!("{library} ({})",crate::native_presets::PRESETS.len()))
   .id_salt("vf-v137-filter-library").default_open(false).show(ui,|ui|{
    let hint=vf_local(app,"Search effects...","Hledat efekty...");
    ui.add(egui::TextEdit::singleline(&mut app.filter_search).hint_text(hint));
    let query=app.filter_search.to_lowercase();
    let mut selected=None;
    let current=app.settings.native_preset;
    egui::ScrollArea::vertical().max_height(245.).id_salt("vf-effect-list")
     .show(ui,|ui|{
       for (idx,preset) in crate::native_presets::PRESETS.iter().enumerate(){
        if !query.is_empty()&&!format!("{} {}",preset.label,preset.desc).to_lowercase().contains(&query){continue;}
        ui.horizontal(|ui|{
         if ui.selectable_label(current==idx,preset.label).on_hover_text(preset.desc).clicked(){selected=Some(idx);}
         ui.label(RichText::new(preset.desc).size(10.).color(VF_MUTED));
        });
       }
     });
    if let Some(index)=selected{if !app.is_smoke{app.select_native_preset(index);}}
   });
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Run GPU filter","Spustit GPU filtr"));
  vf_game_selector(app,ui);
  ui.add_space(6.);
  let key=app.profile_key();
  let mode=app.settings.native_output_modes.get(&key).copied().unwrap_or(0).min(2);
  ui.horizontal_wrapped(|ui|{
   vf_output_action(app,ui,mode);
   vf_status(app,ui);
  });
  vf_hint(ui,vf_local(app,"Changing a preset alone will not alter the original game window.",
   "Samotná změna předvolby neupravuje původní okno hry."));
 });
}

fn vf_combined_visual(app:&mut BoostApp,ui:&mut Ui){
 vf_filters(app,ui);
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Upscaling","Zvětšení obrazu"));
  vf_hint(ui,vf_local(app,"Simple settings first. Advanced algorithms are in More upscalers.",
   "Nejdřív jednoduché volby. Další metody najdeš pod Více upscalerů."));
  let mut mode=app.settings.native_scale_mode;
  ui.horizontal_wrapped(|ui|{
   ui.label(vf_local(app,"Method","Metoda"));
   for(id,en,cz)in [(0u8,"Native","Nativní"),(2,"Smooth","Jemný"),(8,"FSR1","FSR1"),(9,"NIS","NIS")]{
    ui.selectable_value(&mut mode,id,vf_local(app,en,cz));
   }
  });
  if mode!=app.settings.native_scale_mode&&!app.is_smoke{app.apply_scale_algorithm(mode);}
  vf_quality(app,ui);
  let mut cap=app.settings.preview_fps_limit;
  ui.horizontal(|ui|{
   ui.label(vf_local(app,"GPU output limit","Limit GPU výstupu"));
   egui::ComboBox::from_id_salt("vf-v137-output-fps")
    .selected_text(if cap==0{"Off".into()}else{format!("{cap} FPS")}).show_ui(ui,|ui|{
     for value in [0u32,30,45,60,75,90,120,144,165]{
      ui.selectable_value(&mut cap,value,if value==0{"Off".into()}else{format!("{value} FPS")});
     }
    });
  });
  if cap!=app.settings.preview_fps_limit&&!app.is_smoke{
   app.settings.preview_fps_limit=cap;app.preview_guard.reset();
   if let Some(w)=app.native_mirror.as_ref(){w.set_preview_limit(cap);}
   app.save();
  }
  egui::CollapsingHeader::new(vf_local(app,"MORE UPSCALERS / 17 real modes",
      "VÍCE UPSCALERŮ / 17 metod")).default_open(false).show(ui,|ui|{
   let mut selected=app.settings.native_scale_mode;
   egui::ScrollArea::vertical().max_height(220.).id_salt("vf-v137-upscaler-list").show(ui,|ui|{
    for (idx,label) in crate::spatial::MODES.iter().enumerate(){
     ui.selectable_value(&mut selected,idx as u8,*label);
    }
   });
   if selected!=app.settings.native_scale_mode&&!app.is_smoke{app.apply_scale_algorithm(selected);}
   vf_hint(ui,vf_local(app,"CNN 2x methods need enough GPU memory and exact 2x output. Unsupported dimensions fall back safely.",
    "CNN režimy potřebují paměť GPU a přesné 2x zvětšení. Při nepodporované velikosti se použije bezpečná náhrada."));
  });
  vf_hint(ui,vf_local(app,"Output scaling does not lower the game's internal render resolution or guarantee more game FPS.",
    "Zvětšení výstupu nesnižuje interní rozlišení hry ani nezaručuje vyšší FPS hry."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Frame Generation / experimental","Generování snímků / experimentální"));
  let enabled=vf_local(app,"Allow experimental interpolation","Povolit experimentální interpolaci");
  if ui.checkbox(&mut app.settings.experimental_fg_allowed,enabled).changed(){app.save();}
  ui.horizontal_wrapped(|ui|{
   for factor in [2u8,3]{
    if vf_button(ui,&format!("FG {factor}x"),false,
      app.settings.experimental_fg_allowed&&!app.is_smoke&&app.telem.active_game.is_some()){
     rm_start(app,crate::native_mirror::MirrorMode::BorderlessScreen,factor);
    }
   }
  });
  vf_hint(ui,vf_local(app,"More output frames are not more game-rendered frames. Latency can increase.",
    "Více výstupních snímků není více FPS herního enginu. Latence může vzrůst."));
 });
}
fn vf_macros(app:&mut BoostApp,ui:&mut Ui){
 let blocked=app.macro_runner.running()||app.macro_armed_pid.is_some()||crate::window_macro::hotkey_active()||app.is_smoke;
 let key=app.profile_key();
 let names=app.settings.window_macro_profiles.get(&key)
  .map_or_else(Vec::new,|s|s.keys().cloned().collect::<Vec<_>>());
 let mut save=false;
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Macro controls","Ovládání maker"));
  let s=&mut app.window_macro;
  let previous_window=s.selected.as_ref().map(|w|(w.hwnd,w.pid,w.tid));
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,local(ui,"Refresh windows","Obnovit okna"),false,!blocked){s.refresh();}
   egui::ComboBox::from_id_salt("vf-macro-window")
     .selected_text(s.selected.as_ref().map_or(local(ui,"Select target","Vyber okno"),|v|v.title.as_str()))
     .width(ui.available_width().min(370.)).show_ui(ui,|ui|{
      for win in &s.windows{
       ui.selectable_value(&mut s.selected,Some(win.clone()),&win.title);
      }
     });
  });
  if previous_window!=s.selected.as_ref().map(|w|(w.hwnd,w.pid,w.tid)){s.consent=false;}
  let consent=local(ui,"I chose an offline/local application","Vybral jsem offline/místní aplikaci");
  ui.checkbox(&mut s.consent,consent);
  let camera=local(ui,"Record raw mouse motion (Shift Lock camera)",
    "Zaznamenat relativní pohyb myši (Shift Lock kamera)");
  ui.add_enabled(!s.busy(),egui::Checkbox::new(&mut s.relative_mouse,camera));
  if s.relative_mouse{
   vf_hint(ui,local(ui,"Raw Windows mouse deltas, not cursor positions. Playback uses ordinary SendInput and may be ignored by some games.",
    "Snímá skutečné relativní pohyby myši, ne polohu kurzoru. Přehrávání používá SendInput; některé hry ho mohou ignorovat."));
  }
  ui.add_space(8.);
  let busy=s.busy();
  let enabled=!blocked&&s.selected.is_some()&&s.consent&&!busy;
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,local(ui,"RECORD","NAHRÁT"),true,enabled){
    if let Err(e)=s.start(true){s.status=e;}
   }
   if vf_button(ui,local(ui,"PLAY","PŘEHRÁT"),false,enabled&&s.plan.validate().is_ok()){
    if let Err(e)=s.start(false){s.status=e;}
   }
   if vf_button(ui,"STOP / F12",false,busy){s.cancel();}
   if busy{ui.spinner();}
  });
  ui.add_space(8.);
  ui.horizontal_wrapped(|ui|{
   ui.label(local(ui,"Speed","Rychlost"));
   ui.add_enabled(!busy,egui::Slider::new(&mut s.plan.speed,50..=200).suffix("%"));
   ui.label(local(ui,"Repeat","Opakování"));
   ui.add_enabled(!busy,egui::Slider::new(&mut s.plan.repeat,1..=10));
  });
  ui.horizontal_wrapped(|ui|{
   ui.label(local(ui,"Macro name","Název makra"));
   ui.add(egui::TextEdit::singleline(&mut s.name).desired_width(220.).char_limit(48));
   if vf_button(ui,local(ui,"Save macro","Uložit makro"),false,!busy&&s.plan.validate().is_ok()){
    save=true;
   }
  });
  ui.horizontal_wrapped(|ui|{
   ui.label(local(ui,"Saved macros","Uložená makra"));
   egui::ComboBox::from_id_salt("vf-saved-macro")
     .selected_text(local(ui,"Choose saved macro","Načíst uložené makro"))
     .width(ui.available_width().min(350.)).show_ui(ui,|ui|{
      for name in &names{
       if ui.selectable_label(s.name==*name,name).clicked()&&!busy{
        if let Some(saved)=app.settings.window_macro_profiles.get(&key)
           .and_then(|all|all.get(name)){
         s.plan=saved.clone();s.name=name.clone();
         s.status=local(ui,"Ready to play","Připraveno k přehrání").into();
        }
       }
      }
     });
  });
  ui.label(format!("{}: {}  ·  {:.1} s",
   local(ui,"Actions","Akce"),s.plan.events.len(),s.plan.duration() as f64/1000.));
  if !s.status.is_empty(){vf_hint(ui,&s.status);}
  vf_hint(ui,local(ui,"F12 or focus loss stops direct playback. Background hotkey is a separate, opt-in worker below.",
   "F12 nebo ztráta fokusu zastaví přímé přehrávání. Herní zkratka má vlastní volitelný režim níže."));
 });
 if save{
  let s=&mut app.window_macro;
  if let Err(e)=s.plan.save(&s.name,app.settings.window_macro_profiles.entry(key).or_default()){
   s.status=e;
  }else{app.save();}
 }
 ui.add_space(12.);
 vf_macro_background(app,ui);
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Timeline & mouse steps","Časová osa a kroky myši"));
  vf_macro_timeline(app,ui);
 });
}
fn vf_macro_background(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Background hotkey / select and arm",
    "Zkratka ve hře / nastav a aktivuj"));
  vf_hint(ui,vf_local(app,"Keep Vanta FX running (it may be minimized). Alt+Fx works only in the selected permitted game.",
    "Vanta FX musí běžet (může být minimalizovaná). Alt+Fx funguje jen ve vybrané povolené hře."));
  let mut picked=None;
  let current=app.settings.global_shortcuts.get("MACRO");
  ui.horizontal_wrapped(|ui|{
   ui.label(vf_local(app,"Play shortcut","Spouštěcí zkratka"));
   egui::ComboBox::from_id_salt("vf-v137-macro-chord")
    .selected_text(app.settings.global_shortcuts.label("MACRO")).show_ui(ui,|ui|{
     for f in [1u16,2,3,5,6,7,8,9,10,11]{
      let vk=0x6f+f;
      if ui.selectable_label(current.is_some_and(|c|c.mods==1&&c.key==vk),format!("Alt+F{f}")).clicked(){
       picked=Some(vk);
      }
     }
    });
  });
  if let Some(vk)=picked{
   match app.settings.global_shortcuts.assign("MACRO",Some(crate::shortcut_registry::Chord{key:vk,mods:1})){
    Ok(())=>{crate::window_macro::hotkey_disarm();app.save();},
    Err(e)=>app.window_macro.status=format!("Shortcut conflict: {e}"),
   }
  }
  let target=app.window_macro.selected.clone();
  let plan=app.window_macro.plan.clone();
  let allowed=target.as_ref().is_some_and(|t|{
   app.telem.active_game.as_ref().is_some_and(|g|
    g.pid==t.pid&&!crate::anticheat_guard::inspect(&g.path,&g.title).blocked)
  });
  let ready=target.is_some()&&allowed&&plan.validate().is_ok()
    &&app.window_macro.consent&&!app.window_macro.busy()
    &&!app.macro_runner.running()&&!app.is_smoke
    &&app.settings.global_shortcuts.get("MACRO").is_some();
  let armed=crate::window_macro::hotkey_active();
  ui.horizontal_wrapped(|ui|{
   if armed{
    if vf_button(ui,vf_local(app,"DISARM / F12","DEAKTIVOVAT / F12"),true,true){
     crate::window_macro::hotkey_disarm();
    }
   }else if vf_button(ui,vf_local(app,"ARM FOR BACKGROUND","AKTIVOVAT PRO HRU"),true,ready){
    if let (Some(t),Some(chord))=(target,app.settings.global_shortcuts.get("MACRO")){
     match crate::window_macro::hotkey_arm(t,plan,chord){
      Ok(())=>{app.window_macro.consent=false;app.named_macro_arm=None;},
      Err(e)=>app.window_macro.status=e,
     }
    }
   }
   if vf_button(ui,vf_local(app,"Minimize Vanta FX","Minimalizovat Vanta FX"),false,true){
    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
   }
   if vf_button(ui,vf_local(app,"Show HUD","Zobrazit HUD"),false,!app.overlay_open&&!app.is_smoke){
    app.toggle_passive_hud();
   }
  });
  ui.colored_label(if crate::window_macro::hotkey_active(){VF_SUCCESS}else{VF_MUTED},
   crate::window_macro::hotkey_status());
  vf_hint(ui,vf_local(app,"F12 stops macro and disarms shortcut. Alt+F4 and Alt+F12 are reserved. Focus loss stops playback. No anti-cheat bypass.",
   "F12 zastaví a deaktivuje makro. Alt+F4 a Alt+F12 jsou vyhrazené. Ztráta fokusu zastaví přehrávání. Bez obcházení anti-cheatu."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Screen reference / camera movement",
   "Video z obrazovky / pohyb kamery"));
  let recording=app.recorder.active();
  let button=if recording{vf_local(app,"Stop screen video","Zastavit video")}
     else{vf_local(app,"Record screen video","Nahrávat video obrazovky")};
  if vf_button(ui,button,false,!app.is_smoke&&!app.recorder.finishing()){
   app.recording();
  }
  vf_hint(ui,vf_local(app,"Optional raw relative mouse deltas are stored in the macro timeline. Screen video is separate. Replay uses SendInput, so shift-lock camera behavior remains game-dependent.",
   "Volitelný surový relativní pohyb myši se ukládá do časové osy makra. Video je zvlášť. Přehrávání přes SendInput závisí na konkrétní hře."));
 });
}
fn vf_hud(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"On-screen overlay","Overlay ve hře"));
  ui.horizontal_wrapped(|ui|{
   let label=if app.overlay_open{vf_local(app,"Hide HUD","Skrýt HUD")}
    else{vf_local(app,"Show HUD","Zobrazit HUD")};
   if vf_button(ui,label,true,!app.is_smoke){app.toggle_passive_hud();}
   if vf_button(ui,vf_local(app,"Edit position","Upravit polohu"),false,!app.is_smoke){
    app.overlay_open=true;app.overlay_edit_mode=true;
   }
  });
  ui.add_space(8.);
  let zoom=vf_local(app,"Widget size","Velikost widgetu");
  if ui.add(egui::Slider::new(&mut app.settings.overlay_size_scale,0.78..=1.35).text(zoom)).changed(){app.save();}
  let opacity=vf_local(app,"Opacity","Průhlednost");
  if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.0).text(opacity)).changed(){app.save();}
  vf_hint(ui,vf_local(app,
   "HUD is click-through except during position editing. It cannot boost the source game's FPS.",
   "HUD propouští kliknutí mimo režim úprav. Nezvyšuje FPS samotné hry."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Visible widgets and saved layouts","Viditelné widgety a uložená rozložení"));
  hud_layout_panel(app,ui);
  hud_profile_panel(app,ui);
 });
}
fn vf_capture(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Quick recording","Rychlý záznam"));
  let recording=app.recorder.active();
  let replay=app.replay.active();
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,vf_local(app,"Screenshot","Snímek"),false,!app.is_smoke){app.screenshot();}
   let rec=if recording{vf_local(app,"Stop recording","Ukončit záznam")}
    else{vf_local(app,"Start recording","Spustit záznam")};
   if vf_button(ui,rec,true,!app.is_smoke&&!app.recorder.finishing()&&(!replay||recording)){app.recording();}
   let rep=if replay{vf_local(app,"Stop replay","Vypnout replay")}
    else{vf_local(app,"Start replay","Zapnout replay")};
   if vf_button(ui,rep,false,!app.is_smoke&&!app.recorder.busy()){app.toggle_replay();}
   if vf_button(ui,vf_local(app,"Save last seconds","Uložit poslední sekundy"),false,
      replay&&app.replay.ready()&&!app.replay.saving()){app.save_replay();}
  });
  ui.add_space(8.);
  let status=if recording{vf_local(app,"RECORDING","● NAHRÁVÁM")}
   else if app.recorder.finishing(){vf_local(app,"● FINALIZING","● DOKONČUJI")}
   else if replay{vf_local(app,"● REPLAY ACTIVE","● REPLAY AKTIVNÍ")}
   else{vf_local(app,"● IDLE","PŘIPRAVENO")};
  ui.colored_label(if recording{VF_ACCENT}else{VF_MUTED},status);
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,vf_local(app,"Videos folder","Složka videí"),false,true){crate::record::open_folder("videos");}
   if vf_button(ui,vf_local(app,"Screenshots","Snímky"),false,true){crate::record::open_folder("pictures");}
  });
  vf_hint(ui,vf_local(app,"Video uses CPU/GPU and storage. Files stay local.",
   "Video zatěžuje CPU/GPU a disk. Soubory zůstávají místní."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Recording settings","Nastavení nahrávání"));
  capture_record_controls(app,ui);
  capture_encoder_panel(app,ui);
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Media library","Knihovna záznamů"));
  capture_library_panel(app,ui);
 });
}
fn vf_tools(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Choose a tool","Vyber nástroj"));
  let items=[(0usize,"Crosshair","Zaměřovač"),(6,"Zoom","Zoom"),(1,"Timers","Časovače"),
    (2,"Quest notes","Úkoly"),(3,"Practice","Trénink"),(18,"Map notes","Mapa")];
  ui.horizontal_wrapped(|ui|{
   for (view,en,cs) in items{
    let label=vf_local(app,en,cs);
    let chosen=app.extras_view==view;
    let button=egui::Button::new(label)
      .fill(if chosen{Color32::from_rgb(91,40,50)}else{Color32::from_rgb(32,34,43)})
      .stroke(Stroke::new(1.,if chosen{VF_ACCENT}else{VF_EDGE}));
    if ui.add_sized([118.,33.],button).clicked(){app.extras_view=view;}
   }
  });
  vf_hint(ui,vf_local(app,"One tool at a time. The selected tool is immediately below — no extra menus.",
   "Jeden nástroj najednou. Ovládání je rovnou níže — bez dalších nabídek."));
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  match app.extras_view{
   18=>map_notes_panel(app,ui),
   19=>session_goal_panel(app,ui),
   14..=17=>utility_panel(app,ui),
   10..=13=>game_journal_panel(app,ui),
   _=>game_extras_body(app,ui)
  }
 });
}
fn vf_library(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{vf_game_selector(app,ui);});
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Games and shortcuts","Hry a zkratky"));
  oled_library(app,ui);
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Saved profiles","Uložené profily"));
  library_hub_panel(app,ui);
 });
}
fn vf_system(app:&mut BoostApp,ui:&mut Ui){
 optimizer_panel(app,ui);
 ui.add_space(10.);
 egui::CollapsingHeader::new(vf_local(app,"MORE / Windows tools and diagnostics",
   "VICE / Windows nástroje a diagnostika")).default_open(false).show(ui,|ui|{
  vf_frame(ui,|ui|{performance_game_tools(app,ui);suite_moved_system_tune(app,ui);});
 });
}
fn vf_settings(app:&mut BoostApp,ui:&mut Ui){
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Language and appearance","Jazyk a vzhled"));
  ui.horizontal_wrapped(|ui|{
   for(i,label)in [(0u8,"English"),(1,"Čeština")]{
    if ui.selectable_value(&mut app.settings.ui_language,i,label).changed(){app.save();}
   }
  });
  ui.add_space(8.);
  let theme_label=vf_local(app,"Theme","Motiv");
  ui.horizontal_wrapped(|ui|{
   ui.label(theme_label);
   for(i,name)in [(0u8,"Crimson"),(1,"OLED"),(2,"Slate")]{
    if ui.selectable_value(&mut app.settings.ui_theme,i,name).changed(){app.save();}
   }
  });
  let ui_size=vf_local(app,"Interface scale","Velikost rozhraní");
  if ui.add(egui::Slider::new(&mut app.settings.ui_zoom,0.75..=1.35).text(ui_size)).changed(){app.save();}
  let anim=vf_local(app,"Subtle animations (paused during gaming)","Jemné animace (vypnuté při hraní)");
  if ui.checkbox(&mut app.settings.animations,anim).changed(){app.save();}
 });
 ui.add_space(12.);
 vf_frame(ui,|ui|{
  vf_section(ui,vf_local(app,"Privacy and game detection","Soukromí a detekce her"));
  let detect=vf_local(app,"Detect running games","Detekovat spuštěné hry");
  if ui.checkbox(&mut app.settings.detect_games,detect).changed(){app.save();}
  let measured=vf_local(app,"Measure actual game FPS where possible (ETW)","Měřit skutečné FPS hry tam, kde to lze (ETW)");
  if ui.checkbox(&mut app.settings.fps_monitoring,measured).changed(){app.save();}
  suite_safety_panel(app,ui);
 });
}

fn vf_content(app:&mut BoostApp,ui:&mut Ui){
 vf_page_head(app,ui);
 match suite_current(app){
  SuiteArea::Home=>vf_home(app,ui),
  SuiteArea::Boost|SuiteArea::Visual=>vf_combined_visual(app,ui),
  SuiteArea::Tools=>vf_tools(app,ui),
  SuiteArea::Macros=>vf_macros(app,ui),
  SuiteArea::Overlay=>vf_hud(app,ui),
  SuiteArea::Capture=>vf_capture(app,ui),
  SuiteArea::Library=>vf_library(app,ui),
  SuiteArea::System=>vf_system(app,ui),
  SuiteArea::Settings=>vf_settings(app,ui),
  SuiteArea::Lab=>{
   vf_frame(ui,|ui|{
    vf_section(ui,vf_local(app,"Technical diagnostics","Technická diagnostika"));
    rm_catalogue(app,ui);
    suite_moved_lab(app,ui);
   });
  }
 }
}
fn vf_visuals()->egui::Visuals{
 let mut v=egui::Visuals::dark();
 v.panel_fill=VF_BACKGROUND;
 v.window_fill=VF_PANEL;
 v.faint_bg_color=VF_SIDEBAR;
 v.extreme_bg_color=VF_BACKGROUND;
 v.override_text_color=Some(VF_FOREGROUND);
 v.selection.bg_fill=Color32::from_rgb(95,40,54);
 v.selection.stroke=Stroke::new(1.,VF_FOREGROUND);
 v.widgets.noninteractive.bg_fill=VF_PANEL;
 v.widgets.inactive.bg_fill=Color32::from_rgb(34,37,46);
 v.widgets.inactive.weak_bg_fill=v.widgets.inactive.bg_fill;
 v.widgets.inactive.bg_stroke=Stroke::new(1.,VF_EDGE);
 v.widgets.hovered.bg_fill=Color32::from_rgb(47,40,50);
 v.widgets.hovered.weak_bg_fill=v.widgets.hovered.bg_fill;
 v.widgets.active.bg_fill=Color32::from_rgb(86,40,53);
 v.widgets.inactive.corner_radius=CornerRadius::same(8);
 v.widgets.hovered.corner_radius=CornerRadius::same(8);
 v.widgets.active.corner_radius=CornerRadius::same(8);
 v
}
#[cfg(test)]
mod vanta_v136_layout_tests{
 use super::*;
 #[test]fn each_primary_destination_is_unique(){
  let mut seen=std::collections::HashSet::new();
  for area in VF_NAV_AREAS{assert!(seen.insert(area.name()));}
  assert_eq!(VF_NAV_AREAS.len(),10);
 }
 #[test]fn status_palette_is_distinct_and_readable(){
  assert_ne!(VF_BACKGROUND,VF_PANEL);
  assert_ne!(VF_MUTED,VF_FOREGROUND);
  assert!(VF_ACCENT.r()>VF_ACCENT.g());
 }
 #[test]fn primary_route_keeps_original_app_page_mapping(){
  assert_eq!(SuiteArea::Macros.name(),"Macro Studio");
  assert_eq!(SuiteArea::Visual.name(),"Visual");
  assert_eq!(SuiteArea::Settings.name(),"Settings");
 }
}

fn vf_compact(app:&mut BoostApp,ui:&mut Ui){
 egui::ScrollArea::horizontal().id_salt("vf-narrow-nav").show(ui,|ui|{
  ui.horizontal(|ui|{
   for area in VF_NAV_AREAS{
    let is_active=suite_current(app)==area;
    if ui.selectable_label(is_active,vf_nav_label(app,area)).clicked(){area.go(app);}
   }
  });
 });
}

fn vf_macro_timeline(app:&mut BoostApp,ui:&mut Ui){
 let s=&mut app.window_macro;
 let before=s.plan.clone();
 let mut history_action=false;
 vf_hint(ui,local(ui,
  "Edit your recorded steps here. Nothing is sent until you press PLAY.",
  "Tady můžeš upravit nahrané kroky. Nic se nespouští, dokud nestiskneš PŘEHRÁT."));
 ui.add_space(7.);
 ui.add_enabled_ui(!s.busy(),|ui|{
  ui.horizontal_wrapped(|ui|{
   for(i,en,cs)in [(0usize,"One key","Jedna klávesa"),
     (1,"Two keys","Dvě klávesy"),(2,"One click","Jedno kliknutí")]{
    if vf_button(ui,local(ui,en,cs),false,true){
     match crate::window_macro::starter(i){
      Ok(plan)=>{s.plan=plan;s.status=local(ui,"Editable sample loaded","Načtena upravitelná ukázka").into();}
      Err(error)=>s.status=error,
     }
    }
   }
  });
  ui.add_space(5.);
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,local(ui,"+ Add key","Přidat klávesu"),false,true){
    if let Err(error)=s.plan.append_key("E",180,90){s.status=error;}
   }
   if vf_button(ui,local(ui,"+ Add click","Přidat kliknutí"),false,true){
    if let Err(error)=s.plan.append_click(5000,5000,0,180,90){s.status=error;}
   }
   if vf_button(ui,local(ui,"Clear","Smazat"),false,true){
    s.plan=Default::default();
   }
   if vf_button(ui,local(ui,"Undo","Zpět"),false,s.history.can_undo()){
    history_action=s.history.undo(&mut s.plan);
   }
   if vf_button(ui,local(ui,"Redo","Znovu"),false,s.history.can_redo()){
    history_action=s.history.redo(&mut s.plan);
   }
   if vf_button(ui,local(ui,"Copy JSON","Kopírovat JSON"),false,s.plan.validate().is_ok()){
    if let Ok(data)=s.plan.export(){ui.ctx().copy_text(data);}
   }
  });
  ui.separator();
  let mut remove=None;
  let mut reorder=None;
  let mut wait_edit=None;
  let count=s.plan.events.len();
  let max=ui.available_width().min(960.);
  egui::ScrollArea::vertical().max_height(320.).id_salt("vf-direct-steps").show(ui,|ui|{
   ui.set_max_width(max);
   if s.plan.events.is_empty(){
    vf_hint(ui,local(ui,"No actions recorded. Press Record or add a step.",
     "Zatím žádné kroky. Stiskni Nahrát nebo přidej krok."));
   }
   for (i,event) in s.plan.events.iter_mut().enumerate(){
    ui.push_id(i,|ui|{
     ui.horizontal_wrapped(|ui|{
      ui.label(RichText::new(format!("{:02}",i+1)).color(VF_MUTED));
      ui.label(event.action.label());
      ui.add_space(12.);
      ui.label(local(ui,"Delay","Prodleva"));
      let mut ms=event.wait_ms;
      if ui.add(egui::DragValue::new(&mut ms).range(0..=60000).suffix("ms")).changed(){
       wait_edit=Some((i,ms));
      }
      if vf_button(ui,local(ui,"Up","Nahoru"),false,i>0){reorder=Some((i,i-1));}
      if vf_button(ui,local(ui,"Down","Dolů"),false,i+1<count){reorder=Some((i,i+1));}
      if vf_button(ui,local(ui,"Remove","Odebrat"),false,true){remove=Some(i);}
     });
    });
    if i+1<count{ui.separator();}
   }
  });
  if let Some((i,wait))=wait_edit{
   if let Err(error)=s.plan.set_wait(i,wait){s.status=error;}
  }
  if let Some(i)=remove{
   if let Err(error)=s.plan.remove_balanced(i){s.status=error;}
  }
  if let Some((a,b))=reorder{
   if let Err(error)=s.plan.swap_balanced(a,b){s.status=error;}
  }
  if let Err(error)=s.plan.validate(){
   vf_hint(ui,&error);
  }
 });
 if !history_action && before!=s.plan{
  s.history.record(before);
  s.preview_at=None;
 }
}
