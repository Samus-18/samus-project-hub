// Compact bilingual front door, backed by the actual native pipeline.
fn rm(a:&BoostApp,en:&'static str,cs:&'static str)->&'static str{if a.settings.ui_language==1{cs}else{en}}
const RM_BG_QUADS:(usize,usize)=(8,6); // 48 quads, down from 140 in v129.
fn rm_bg(ui:&Ui,a:&BoostApp){
 let r=ui.max_rect();let p=ui.painter();
 let animated=e_motion(ui)&&!a.native_mirror.as_ref().is_some_and(|w|w.running())&&!a.recorder.elapsed().is_some()&&!a.replay.is_buffering()&&!a.overlay_open;
 // Quantize ambient motion so repaints triggered by normal UI input don't
 // allocate a fresh long animation state every few milliseconds.
 let now=if animated{ui.input(|i|i.time)as f32}else{0.};
 let t=(now*5.).floor()/5.;
 let mut mesh=egui::Mesh::default();
 let (nx,ny)=RM_BG_QUADS;
 for y in 0..=ny{for x in 0..=nx{
 let u=x as f32/nx as f32;let v=y as f32/ny as f32;
 let a1=(-((u-0.85-0.07*(t*0.23).sin()).powi(2)*6.+(v-0.12).powi(2)*9.)).exp();
 let a2=(-((u-0.08).powi(2)*9.+(v-0.86-0.05*(t*0.17).cos()).powi(2)*6.)).exp();
 let (base,hot,soft)=match a.settings.ui_theme{1=>([23.,17.,37.],[36.,17.,47.],[27.,13.,42.]),2=>([15.,27.,29.],[11.,47.,35.],[16.,34.,37.]),_=>([24.,15.,34.],[54.,17.,34.],[37.,14.,33.])};
 mesh.colored_vertex(pos2(r.left()+u*r.width(),r.top()+v*r.height()),Color32::from_rgb((base[0]+hot[0]*a1+soft[0]*a2)as u8,(base[1]+hot[1]*a1+soft[1]*a2)as u8,(base[2]+hot[2]*a1+soft[2]*a2)as u8));
 }}
 for y in 0..ny{for x in 0..nx{let i=(y*(nx+1)+x)as u32;let j=i+nx as u32+1;mesh.add_triangle(i,i+1,j);mesh.add_triangle(j,i+1,j+1);}}
 p.add(egui::Shape::mesh(mesh));
 for i in 0..8{let y=r.top()+((i as f32*71.+t*4.).rem_euclid(r.height().max(1.)));let x=r.left()+((i as f32*137.).rem_euclid(r.width().max(1.)));p.circle_filled(pos2(x,y),1.5,Color32::from_rgba_unmultiplied(255,110,150,65));}
 if animated{ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));}
}
fn rm_name(a:&BoostApp,x:SuiteArea)->&'static str{match x{SuiteArea::Home=>rm(a,"Overview","Přehled"),SuiteArea::Boost=>rm(a,"Performance","Výkon"),SuiteArea::Visual=>rm(a,"Upscale & filters","Zvětšení a filtry"),SuiteArea::Tools=>rm(a,"Game tools","Herní nástroje"),SuiteArea::Macros=>rm(a,"Macros","Makra"),SuiteArea::Overlay=>rm(a,"HUD","HUD"),SuiteArea::Capture=>rm(a,"Clips & recording","Klipy a záznam"),SuiteArea::Library=>rm(a,"Games","Hry"),SuiteArea::System=>rm(a,"System","Systém"),SuiteArea::Settings=>rm(a,"Settings","Nastavení"),SuiteArea::Lab=>rm(a,"Experiments","Experimenty")}}
fn rm_detail(a:&BoostApp,x:SuiteArea)->&'static str{match x{
 SuiteArea::Home=>rm(a,"Select a game and start.","Vyber hru a začni."),
 SuiteArea::Boost=>rm(a,"Scaling, pacing and frame generation.","Upscaling, časování a generování snímků."),
 SuiteArea::Visual=>rm(a,"One-click window scaling, color and shader looks.","Zvětšení okna, barvy a GPU efekty jedním kliknutím."),
 SuiteArea::Tools=>rm(a,"Timers, crosshair, maps and builds.","Časovače, zaměřovač, mapy a buildy."),
 SuiteArea::Macros=>rm(a,"Record and edit offline sequences.","Záznam a úprava offline sekvencí."),
 SuiteArea::Overlay=>rm(a,"Live game statistics and widgets.","Herní statistiky a widgety."),
 SuiteArea::Capture=>rm(a,"Record, save and edit your clips.","Nahrávej, ukládej a upravuj klipy."),
 SuiteArea::Library=>rm(a,"Select running games and saved profiles.","Výběr spuštěných her a profilů."),
 SuiteArea::System=>rm(a,"Inspect hardware and reversible controls.","Kontrola hardwaru a vratná nastavení."),
 SuiteArea::Settings=>rm(a,"Language, appearance and permissions.","Jazyk, vzhled a oprávnění."),
 SuiteArea::Lab=>rm(a,"Unfinished or experimental integrations.","Nedokončené nebo experimentální integrace.")}}
fn rm_game(a:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("remake-game-filter");
 let mut query=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_default();
 let active=a.telem.active_game.as_ref().map(|g|trim(&g.title,42)).unwrap_or_else(||rm(a,"Select running game","Vyber spuštěnou hru").into());
 let candidates=a.telem.visible_games.iter().filter(|g|{let q=query.trim().to_lowercase();q.is_empty()||g.title.to_lowercase().contains(&q)||g.path.file_name().is_some_and(|n|n.to_string_lossy().to_lowercase().contains(&q))}).take(75).map(|g|(g.pid,g.title.clone(),g.path.file_name().map(|n|n.to_string_lossy().to_string()).unwrap_or_default())).collect::<Vec<_>>();
 let minecraft=a.telem.visible_games.iter().find(|g|{let title=g.title.to_lowercase();title.contains("minecraft")||crate::win::is_minecraft_java_window(&g.path,&g.title,&g.class_name)}).map(|g|(g.pid,g.title.clone()));
 let mut next_pid=None;let mut reveal=None;
 ui.horizontal_wrapped(|ui|{
  egui::ComboBox::from_id_salt("remake-running-game").width(ui.available_width().min(300.)).selected_text(active).show_ui(ui,|ui|{
   ui.add(egui::TextEdit::singleline(&mut query).hint_text(rm(a,"Search window / exe","Hledat okno / exe")).char_limit(64));
   if candidates.is_empty(){caption(ui,rm(a,"No matching window. Try Show all windows.","Žádné odpovídající okno. Zkus zobrazit všechna okna."));}
   for(pid,title,exe) in &candidates{
    let selected=a.telem.selected_pid==Some(*pid);
    if ui.selectable_label(selected,format!("{} ({})",trim(title,42),trim(exe,24))).clicked(){next_pid=Some(*pid);}
   }
  });
  if ui.small_button(rm(a,"Refresh","Obnovit")).clicked(){reveal=Some(a.telem.show_capture_windows);}
  if let Some((pid,_))=&minecraft{
   if ui.small_button(rm(a,"Minecraft","Minecraft")).clicked(){next_pid=Some(*pid);}
  }
 });
 ui.horizontal_wrapped(|ui|{
  let label=if a.telem.show_capture_windows{rm(a,"Only detected games","Pouze rozpoznané hry")}else{rm(a,"Show other visible windows","Zobrazit další viditelná okna")};
  if ui.small_button(label).clicked(){reveal=Some(!a.telem.show_capture_windows);}
  ui.label(RichText::new(format!("{} {}",a.telem.visible_games.len(),rm(a,"windows available","dostupných oken"))).size(10.).color(MUTED));
 });
 if let Some(value)=reveal{if !a.is_smoke{a.telem.show_capture_windows=value;a.telem.refresh(true);}}
 if let Some(pid)=next_pid{if !a.is_smoke{a.choose_game(pid);}}
 ui.data_mut(|d|d.insert_temp(id,query));
}
const RM_LOOKS:[(usize,&str,&str,&str,&str,[u8;3]);8]=[
 (0,"Natural","Přirozený","Natural color and fine texture.","Přirozené barvy a jemné detaily.",[82,152,102]),
 (4,"Caves","Jeskyně","Brighten darker areas, not all pixels.","Prosvětlí tmavé oblasti, ne celý obraz.",[88,112,186]),
 (2,"Clarity","Ostrost","Lightweight local detail.","Lehké zvýraznění detailů.",[122,174,195]),
 (5,"Warm sunset","Teplý západ","Warmer golden color grade.","Teplejší zlaté barvy.",[231,140,75]),
 (64,"Daylight film","Denní film","Outdoor tone and restrained highlights.","Denní kontrast a mírnější světla.",[222,190,99]),
 (69,"Natural glow","Přirozená záře","Bloom and balanced highlights.","Bloom a vyvážená světla.",[195,105,156]),
 (7,"Monochrome","Černobílý","High contrast grayscale.","Kontrastní černobílý obraz.",[148,148,158]),
 (77,"Balanced","Vyvážený","Tone, vibrance and detail.","Tón, živost barev a detaily.",[102,165,162]),
];
fn rm_look_card(a:&mut BoostApp,ui:&mut Ui,index:usize,w:f32){
 let (preset,en,cs,desc_en,desc_cs,c)=RM_LOOKS[index];
 let active=a.settings.native_preset==preset&&!a.settings.native_stack_profiles.contains_key(&a.profile_key());
 let thumb_h=(w-10.).max(1.)*9./16.;let(r,res)=ui.allocate_exact_size(vec2(w,thumb_h+82.),Sense::click());let t=motion_value(ui,res.id.with("look-v120"),res.hovered(),0.16);
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,10.,Color32::from_rgb(24,25,36));
 let sw=Rect::from_min_max(r.left_top()+vec2(5.,5.),pos2(r.right()-5.,r.top()+5.+thumb_h));
 // Genuine native GPU reference if present; fallback color swatch is labeled as illustrative.
 e_gradient(&p,sw,Color32::from_rgb(c[0]/3,c[1]/3,c[2]/3),Color32::from_rgb(c[0],c[1],c[2]));
 if let Some(tex)=native_look_preview_handle(a,ui.ctx(),preset){p.image(tex.id(),sw,Rect::from_min_max(pos2(0.,0.),pos2(1.,1.)),WHITE);}else{p.circle_filled(sw.right_center()-vec2(22.,0.),10.+3.*t,Color32::from_white_alpha(110));}
 p.rect_stroke(r,10.,Stroke::new(if active{2.}else{1.},if active{RED}else{Color32::from_rgb(61,55,73)}),StrokeKind::Inside);
 p.text(r.left_top()+vec2(10.,thumb_h+12.),egui::Align2::LEFT_TOP,rm(a,en,cs),FontId::proportional(14.),WHITE);
 let g=p.layout(rm(a,desc_en,desc_cs).into(),FontId::proportional(11.),MUTED,(w-20.).max(30.));p.galley(r.left_top()+vec2(10.,thumb_h+35.),g,MUTED);
 if res.on_hover_text(rm(a,"Reference image; not live gameplay. GPU effects may cost FPS.","Referenční snímek, nikoli živá hra. GPU efekty mohou snížit FPS.")).clicked(){a.select_native_preset(preset);}
}
/// Compact visual cards instead of a long row of identical text buttons.
fn rm_quick_look_tile(a:&mut BoostApp,ui:&mut Ui,index:usize,width:f32){
 let(preset,en,cs,desc_en,desc_cs,c)=RM_LOOKS[index];
 let selected=a.settings.native_preset==preset&&!a.settings.native_stack_profiles.contains_key(&a.profile_key());
 let(r,res)=ui.allocate_exact_size(vec2(width,118.),Sense::click());
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,9.,Color32::from_rgb(25,24,36));
 let picture=Rect::from_min_max(r.left_top()+vec2(4.,4.),pos2(r.right()-4.,r.top()+73.));
 e_gradient(&p,picture,Color32::from_rgb(c[0]/3,c[1]/3,c[2]/3),Color32::from_rgb(c[0],c[1],c[2]));
 if let Some(tex)=native_look_preview_handle(a,ui.ctx(),preset){
  p.image(tex.id(),picture,Rect::from_min_max(pos2(0.,0.),pos2(1.,1.)),WHITE);
 }
 p.text(r.left_top()+vec2(10.,81.),egui::Align2::LEFT_TOP,rm(a,en,cs),
  FontId::proportional(13.),WHITE);
 let label=trim(rm(a,desc_en,desc_cs),40);
 p.text(r.left_top()+vec2(10.,100.),egui::Align2::LEFT_TOP,label,
  FontId::proportional(10.),MUTED);
 p.rect_stroke(r,9.,Stroke::new(if selected{2.}else{1.},
  if selected{RED}else{Color32::from_rgb(64,52,74)}),StrokeKind::Inside);
 if res.on_hover_text(rm(a,desc_en,desc_cs)).clicked(){a.select_native_preset(preset);}
}
/// Compact actions to recover existing Windows capture without guessing FPS.
/// Only user clicks can restart or change the renderer.
fn rm_capture_recovery(a:&mut BoostApp,ui:&mut Ui,mode:crate::native_mirror::MirrorMode){
 let d=a.native_mirror.as_ref().map(|w|{
  (crate::filter_doctor::diagnose(Some(&w.diagnostics()),w.submitted_frames()),
   w.capture_average_fps(),w.output_average_fps(),w.submitted_frames(),
   w.diagnostics().effective_passes)
 });
 let Some((kind,source_fps,output_fps,count,passes))=d else{return;};
 if kind==crate::filter_doctor::Status::OutputFrames{
  ui.horizontal_wrapped(|ui|{
   let val=|n:Option<f32>|n.filter(|x|x.is_finite()&&*x>0.).map_or_else(||"—".into(),|x|format!("{x:.0}"));
   ui.label(format!("{}: {}    {}: {}    {}: {}",
     rm(a,"Capture","Snímání"),val(source_fps),
     rm(a,"Output","Výstup"),val(output_fps),
     rm(a,"Active effects","Aktivní efekty"),passes));
  });
  caption(ui,rm(a,"Horizon capture/output FPS are NOT game engine FPS or measured performance gain.","FPS snímání a výstupu nejsou FPS herního enginu ani změřené zrychlení."));
 }else if kind==crate::filter_doctor::Status::NoFrames && count==0{
  ui.horizontal_wrapped(|ui|{
   if ui.button(rm(a,"Try separate preview","Zkusit samostatný náhled")).clicked()&&!a.is_smoke{
    rm_start(a,crate::native_mirror::MirrorMode::SeparateWindow,1);
   }
   if ui.button(rm(a,"Refresh window list","Obnovit seznam oken")).clicked()&&!a.is_smoke{
    a.telem.refresh(true);
   }
  });
  caption(ui,rm(a,"A separate preview is a capture test, not a filter injected into the game.","Samostatný náhled je test snímání, ne filtr vložený do hry."));
 }else if kind==crate::filter_doctor::Status::LookRejected{
  if ui.button(rm(a,"Restore light Natural look","Obnovit lehký přirozený efekt")).clicked()&&!a.is_smoke{
    a.select_native_preset(0);
  }
 }else if kind==crate::filter_doctor::Status::Paused{
  caption(ui,rm(a,"Bring the selected game to the foreground. The GPU output intentionally pauses when you Alt+Tab away.","Přepni vybranou hru do popředí. Po Alt+Tab pryč se GPU výstup záměrně pozastaví."));
 }
 let _=mode;
}
fn rm_visual(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
 ui.set_min_width(ui.available_width());
 heading(ui,rm(a,"1 / Choose game","1 / Vyber hru"));rm_game(a,ui);
 rm_mc_presets(a,ui);rm_game_preflight(a,ui);
 ui.add_space(8.);heading(ui,rm(a,"2 / Choose look","2 / Vyber efekt"));
 let key=a.profile_key();let current=crate::native_presets::PRESETS.get(a.settings.native_preset).map(|p|p.label).unwrap_or("Custom");
 ui.label(format!("{}: {}",rm(a,"Selected","Vybráno"),if a.settings.native_stack_profiles.contains_key(&key){rm(a,"Custom stack","Vlastní sada")}else{current}));
 if let Some((_,_,_,desc_en,desc_cs,_))=RM_LOOKS.iter().find(|x|x.0==a.settings.native_preset){
  caption(ui,rm(a,desc_en,desc_cs));
 }
 caption(ui,rm(a,"FPS boost: not guaranteed; GPU effects can reduce FPS. Use the measured before/after test below.","FPS navíc: bez záruky; GPU efekty mohou FPS snížit. Použij měření před/po níže."));
 let w=ui.available_width();let cols=if w>=650.{3}else{2};
 let tilew=((w-8.*(cols-1)as f32)/cols as f32).max(115.);
 for chunk in [0usize,1,3].chunks(cols){
  ui.horizontal(|ui|{for &idx in chunk{rm_quick_look_tile(a,ui,idx,tilew);}});
 }
 caption(ui,rm(a,"Cards show GPU-rendered reference images, not the live game.","Karty ukazují GPU referenční obrázky, nikoli aktuální hru."));
 // One clear intensity slider for current native HLSL look. Stage weights
 // are passed to the existing live GPU filter pipeline; duplicates retain slots.
 let ids=a.settings.native_stack_profiles.get(&key).cloned()
   .unwrap_or_else(||crate::native_presets::PRESETS.get(a.settings.native_preset)
     .map_or_else(Vec::new,|p|p.stages.to_vec()));
 if !ids.is_empty(){
  let mut strength=ids.iter().enumerate()
    .map(|(i,_)|a.settings.lighting.layers.strength_at(&ids,i)).sum::<f32>()/ids.len() as f32;
  ui.horizontal_wrapped(|ui|{
   ui.label(rm(a,"Effect strength","Síla efektu"));
   if ui.add(egui::Slider::new(&mut strength,0.0..=1.0).show_value(true)
       .fixed_decimals(2)).changed()&&!a.is_smoke{
     let mut light=a.settings.lighting.clone();
     for i in 0..ids.len(){light.layers.set_at(&ids,i,strength);}
     a.apply_native_lighting(light);
   }
   if ui.small_button(rm(a,"Reset","Obnovit")).clicked()&&!a.is_smoke{
     let mut light=a.settings.lighting.clone();
     for i in 0..ids.len(){light.layers.set_at(&ids,i,1.);}
     a.apply_native_lighting(light);
   }
  });
 }

 ui.add_space(6.);heading(ui,rm(a,"3 / Scale to your monitor","3 / Zvětši na monitor"));
 let key=a.profile_key();
 let default_mode=if a.telem.active_game.is_some(){0}else{1};
 let mut mode=a.settings.native_output_modes.get(&key).copied().unwrap_or(default_mode).min(2);
 let prior_mode=mode;
 ui.horizontal_wrapped(|ui|{for(i,(en,cs))in [("Fit to screen","Přizpůsobit monitoru"),("Window preview","Náhled v okně"),("DXGI fallback","Záložní DXGI")].iter().enumerate(){ui.selectable_value(&mut mode,i as u8,rm(a,en,cs));}});
 caption(ui,rm(a,"Fullscreen scaling uses WGC, a GPU shader and a click-through DXGI window on the same monitor. No game DLL injection.","Zvětšení používá WGC, GPU shader a průchozí DXGI okno na stejném monitoru. Bez injekce DLL do hry."));
 if mode!=prior_mode&&!a.is_smoke{
  a.settings.native_output_modes.insert(key.clone(),mode);a.save();
 }
 if mode==0{
  ui.horizontal_wrapped(|ui|{
   ui.label(rm(a,"GPU workload","Zátěž GPU"));
   let mut q=a.settings.native_quality;
   for(level,en,cs)in [(3,"Auto","Automaticky"),(2,"Eco","Úsporný"),(1,"Balanced","Vyvážený"),(0,"Full","Plný")]{
    ui.selectable_value(&mut q,level,rm(a,en,cs));
   }
   if q!=a.settings.native_quality&&!a.is_smoke{
    a.settings.native_quality=q;a.save();a.refresh_native_mirror();
   }
  });
  ui.horizontal_wrapped(|ui|{
   ui.label(rm(a,"Scaling filter","Filtr zvětšení"));
   let mut algo=a.settings.native_scale_mode;
   for(id,en,cs)in [(1,"Fast","Rychlý"),(2,"Smooth","Jemný"),(8,"FSR1","FSR1"),(9,"NIS","NIS")]{
    ui.selectable_value(&mut algo,id,rm(a,en,cs));
   }
   if algo!=a.settings.native_scale_mode&&!a.is_smoke{a.apply_scale_algorithm(algo);}
  });
  // Limit Horizon's own output only, never the game's Present queue.
  ui.horizontal_wrapped(|ui|{
   ui.label(rm(a,"Output limit","Limit GPU výstupu"));
   let mut fps=a.settings.preview_fps_limit;
   for(v,en,cs)in [(0,"Off","Bez limitu"),(45,"45 FPS","45 FPS"),
      (60,"60 FPS","60 FPS"),(75,"75 FPS","75 FPS"),
      (90,"90 FPS","90 FPS"),(120,"120 FPS","120 FPS")]{
     ui.selectable_value(&mut fps,v,rm(a,en,cs));
   }
   if fps!=a.settings.preview_fps_limit&&!a.is_smoke{
      a.settings.preview_fps_limit=fps;
      a.preview_guard.reset();
      if let Some(w)=a.native_mirror.as_ref(){w.set_preview_limit(fps);}
      a.save();
   }
  });
  caption(ui,rm(a,"Limits Horizon's presented/captured output work, NOT the source game's FPS. Start with 60 FPS to reduce RX 580 load.",
    "Omezuje práci GPU výstupu Horizonu, NIKOLI FPS hry. Na RX 580 začni s 60 FPS."));
  caption(ui,match a.settings.native_quality{
  3=>rm(a,"Auto: keeps a 1.5× scale budget and adjusts active HLSL passes using sampled GPU filter time. Never changes your game's FPS cap.","Automaticky: limit zvětšení 1,5× a počet aktivních HLSL efektů podle naměřeného času GPU. Nemění FPS limit hry."),
  2=>rm(a,"Eco: up to 1.25× scaling per axis; lowest output GPU pixel load.","Úsporný: zvětšení nejvýše 1,25× v každé ose; nejmenší zátěž výstupních GPU pixelů."),
  1=>rm(a,"Balanced: up to 1.5×, with all selected effect passes.","Vyvážený: zvětšení nejvýše 1,5×, všechny zvolené efekty."),
  _=>rm(a,"Full: up to 2× scaling; can consume significantly more GPU time.","Plný: zvětšení až 2×; může výrazně zvýšit dobu GPU vykreslení."),
 });
 }
 // The main start button below is the single authoritative action. Picking
 // a mode never starts or silently stops the renderer.

 if a.telem.active_game.is_none(){ui.colored_label(RED,rm(a,"First choose Minecraft or another visible game above.","Nejdřív vyber Minecraft nebo jinou viditelnou hru nahoře."));}
 else if mode==0{ui.colored_label(V25_MINT,rm(a,"Press START, then Alt+Tab back to the game. The GPU output appears only when your game has focus.","Stiskni SPUSTIT a pak Alt+Tab zpátky do hry. GPU výstup se zobrazí jen tehdy, když má hra fokus."));}
 else if a.telem.active_game.as_ref().is_some_and(|g|crate::win::is_minecraft_java_window(&g.path,&g.title,&g.class_name)||g.title.to_ascii_lowercase().contains("minecraft")){
  caption(ui,rm(a,"Minecraft: use Windowed/Borderless. First test in a separate GPU window, then switch to Game view.","Minecraft: použij okno / bez okrajů. Nejdřív vyzkoušej samostatné GPU okno, potom herní zobrazení."));
 }

 caption(ui,match mode{
 0=>rm(a,"Aspect-preserving GPU window scaling to the game's monitor. Source must be windowed/borderless and foreground; no exclusive-fullscreen guarantee.","GPU zvětšení bez deformace na monitor hry. Hra musí být v okně a v popředí; exkluzivní fullscreen není zaručen."),
 1=>rm(a,"A separate preview for checking capture and effect quality; this does NOT filter the original Minecraft window.","Samostatné okno pro kontrolu snímání a efektu; původní okno Minecraftu se NEMĚNÍ."),
 _=>rm(a,"DXGI monitor capture for a foreground, monitor-filling game; fallback can fail with protected output.","Záložní snímání DXGI pro hru přes celý monitor v popředí; může selhat při chráněném výstupu.")
 });
 rm_cached_memory_warning(a,ui);
 let running=a.native_mirror.as_ref().is_some_and(|w|w.running());let pending=a.native_mirror.is_some()&&!running;
 let diagnosis=a.native_mirror.as_ref().map(|w|{
  let health=w.diagnostics();crate::filter_doctor::diagnose(Some(&health),w.submitted_frames())
 }).unwrap_or(crate::filter_doctor::Status::Idle);
 let diagnosis_message=crate::filter_doctor::message(diagnosis,a.settings.ui_language==1);
 if diagnosis!=crate::filter_doctor::Status::OutputFrames{
  ui.colored_label(if diagnosis==crate::filter_doctor::Status::Idle{MUTED}else{RED},diagnosis_message);
 }
 ui.horizontal_wrapped(|ui|{
 let btn=egui::Button::new(if a.native_mirror.is_some(){rm(a,"STOP OUTPUT","VYPNOUT VÝSTUP")}else{rm(a,"START GPU OUTPUT","SPUSTIT GPU VÝSTUP")}).fill(Color32::from_rgb(149,36,65)).min_size(vec2(155.,36.));
 if ui.add_enabled(!a.is_smoke&&(a.native_mirror.is_some()||a.telem.active_game.is_some()),btn).clicked(){if a.native_mirror.is_some(){a.stop_native_mirror();}else{rm_start(a,rm_mode(mode),1);}}
 ui.colored_label(if running&&a.native_mirror.as_ref().is_some_and(|w|w.submitted_frames()>0){V25_MINT}else{MUTED},
  if running&&a.native_mirror.as_ref().is_some_and(|w|w.submitted_frames()>0){rm(a,"GPU frames streaming","GPU snímky běží")}
  else if running{rm(a,"Awaiting game frames","Čekám na snímky hry")}
  else if pending{rm(a,"Starting / stopping","Spouštění / vypínání")}
  else{rm(a,"Horizon output off","Výstup Horizon vypnutý")});
 });
 rm_capture_recovery(a,ui,rm_mode(mode));
 if let Some(w)=a.native_mirror.as_ref(){let frames=w.submitted_frames();let diag=w.diagnostics();ui.label(format!("{}: {}",rm(a,"Submitted frames","Vykreslené snímky"),frames));
 if diag.live_look_rejected{ui.colored_label(RED,rm(a,"Effect rejected; previous effect retained.","Efekt odmítnut; zůstal předchozí efekt."));}
 if diag.paused||frames==0{caption(ui,rm(a,"No game frames yet; return to the game and allow WGC to initialize. This is not a successful live-filter test.","Zatím žádné herní snímky; vrať se do hry a nech WGC inicializovat. Není to potvrzený test filtru."));}
 if w.mode()!=rm_mode(mode)&&ui.button(rm(a,"Switch output method","Změnit způsob výstupu")).clicked(){rm_start(a,rm_mode(mode),1);}
 }
 if !a.native_mirror_status.is_empty(){
  let low=a.native_mirror_status.to_ascii_lowercase();
  let error=low.contains("error")||low.contains("failed")||low.contains("rejected")||low.contains("unavailable")||low.contains("lost")||low.contains("denied");
  if error{ui.colored_label(RED,&a.native_mirror_status);}else{
   egui::CollapsingHeader::new(rm(a,"Technical output status","Technický stav výstupu")).show(ui,|ui|{ui.label(&a.native_mirror_status);});
  }
 }
 // References below are explicitly optional and not a live-game preview.
 ui.horizontal_wrapped(|ui|{let old=a.native_gallery_scene;for (i,en,cs) in [(1,"Gameplay reference","Herní reference"),(0,"Photo reference","Fotografie")]{let label=if a.settings.ui_language==1{cs}else{en};ui.selectable_value(&mut a.native_gallery_scene,i,label);}if old!=a.native_gallery_scene{let keep=vec![false;crate::native_presets::PRESETS.len()];trim_native_gallery_cache(a,&keep);}});
 let w=ui.available_width();let cols=if w>700.{4}else if w>450.{3}else{2};let cw=(w-(cols-1)as f32*8.)/cols as f32;
 let mut gallery_visible=false;
 egui::CollapsingHeader::new(rm(a,"Browse 8 GPU-rendered reference looks","Procházet 8 GPU referenčních efektů"))
 .default_open(false).id_salt("remake-gpu-reference-cards").show(ui,|ui|{
 gallery_visible=true;
 for chunk in (0..RM_LOOKS.len()).collect::<Vec<_>>().chunks(cols){ui.horizontal(|ui|{for &i in chunk{rm_look_card(a,ui,i,cw);}});ui.add_space(5.);}
 });

 egui::CollapsingHeader::new(rm(a,"Advanced effects & shader studio","Pokročilé efekty a studio"))
  .id_salt("v133-combined-advanced").default_open(false).show(ui,|ui|{
   rm_all_looks(a,ui);
   rm_studio(a,ui);
   rm_lighting(a,ui);
   rm_visual_advanced_controls(a,ui);
  });
 ui.add_space(4.);
 let selected=a.settings.native_preset;
  egui::CollapsingHeader::new(rm(a,"Selected effect / GPU-rendered reference","Vybraný efekt / GPU náhled"))
 .default_open(false).show(ui,|ui|{
   if let Some(tex)=native_look_preview_handle(a,ui.ctx(),selected){
     let w=ui.available_width().min(460.);
     ui.add(egui::Image::new(&tex).fit_to_exact_size(vec2(w,w*9./16.)));
   }else{caption(ui,rm(a,"Preview image unavailable; GPU output may still work.","Referenční obrázek není dostupný; GPU výstup může stále fungovat."));}
 });
 let mut keep=vec![false;crate::native_presets::PRESETS.len()];
 if gallery_visible{for (j,_,_,_,_,_) in RM_LOOKS{keep[j]=true;}}
 for idx in [0usize,1,3]{keep[RM_LOOKS[idx].0]=true;}
 keep[selected]=true; // Reuse the selected look if expanded in either preview pane.
 trim_native_gallery_cache(a,&keep);
 let mut light=a.settings.lighting.clone();let mut split=light.layers.split.is_some();let mut changed=false;
 if ui.checkbox(&mut split,rm(a,"Live before / after split","Živé porovnání před / po")).changed(){light.layers.split=split.then_some(0.5);changed=true;}
 if let Some(mut at)=light.layers.split{
  if ui.add(egui::Slider::new(&mut at,0.05..=0.95).text(rm(a,"Comparison divider position","Poloha hranice porovnání"))).changed(){light.layers.split=Some(at);changed=true;}
 }
 if changed{a.apply_native_lighting(light);}
 caption(ui,rm(a,"Color effects run on Horizon's native GPU output. Minecraft's own rendered image and engine FPS remain untouched.","Barevné efekty běží ve vlastním GPU výstupu Horizon. Vykreslování Minecraftu ani FPS enginu se nemění."));
 });
 egui::CollapsingHeader::new(rm(a,"Measure real FPS before / after this filter","Změřit skutečné FPS před / po filtru")).default_open(false).show(ui,|ui|rm_measure(a,ui));
 rm_filter_status(a,ui);
 egui::CollapsingHeader::new(rm(a,"All effects, lighting & custom stacks","Všechny efekty, světla a vlastní sady")).id_salt("v120-advanced-visual").default_open(false).show(ui,|ui|{v22_filters(a,ui);});
 egui::CollapsingHeader::new(rm(a,"HDR, shaders and diagnostics","HDR, shadery a diagnostika")).default_open(false).show(ui,|ui|suite_moved_visual(a,ui));
}
fn rm_home(a:&mut BoostApp,ui:&mut Ui){
 let w=ui.available_width();let(r,_)=ui.allocate_exact_size(vec2(w,140.),Sense::hover());let p=ui.painter().with_clip_rect(r);
 e_gradient(&p,r,Color32::from_rgb(38,22,42),Color32::from_rgb(107,28,55));
 p.text(r.left_top()+vec2(18.,14.),egui::Align2::LEFT_TOP,"HORIZON  /  PLAY",FontId::proportional(11.),Color32::from_rgb(255,164,184));
 p.text(r.left_top()+vec2(18.,40.),egui::Align2::LEFT_TOP,rm(a,"Your game, enhanced.","Tvoje hra. Lepší zážitek."),FontId::proportional(if w<470.{22.}else{29.}),WHITE);
 p.text(r.left_top()+vec2(18.,84.),egui::Align2::LEFT_TOP,rm(a,"One game. One GPU workflow. No injection.","Jedna hra. Jeden přehledný GPU postup. Bez injekce."),FontId::proportional(12.),WHITE);
 for(i,(en,cs))in [("1  Game","1  Hra"),("2  Look","2  Efekt"),("3  Scale","3  Zvětšení")].iter().enumerate(){
  let x=r.left()+18.+i as f32*(w.min(460.)/3.);
  let pill=Rect::from_min_size(pos2(x,r.bottom()-34.),vec2((w.min(460.)/3.-8.).max(70.),23.));
  p.rect_filled(pill,7.,Color32::from_rgba_unmultiplied(20,16,28,180));
  p.text(pill.center(),egui::Align2::CENTER_CENTER,rm(a,en,cs),FontId::proportional(11.),Color32::from_rgb(255,201,213));
 }
 ui.add_space(10.);
 frame(PANEL).show(ui,|ui|{
 ui.set_min_width(ui.available_width());rm_game(a,ui);
 ui.horizontal_wrapped(|ui|{for(label,value)in [(rm(a,"GAME FPS","FPS HRY"),fps_text(a,"fps")),(rm(a,"FRAME TIME","ČAS SNÍMKU"),fps_text(a,"frametime")),(rm(a,"1% LOW","1% LOW"),fps_text(a,"low"))]{ui.vertical(|ui|{ui.label(RichText::new(label).color(MUTED).size(10.));ui.label(RichText::new(value).size(24.).strong());});ui.add_space(18.);}});
 let active=a.native_mirror.is_some();
 ui.horizontal_wrapped(|ui|{
  let label=if active{rm(a,"STOP GPU","VYPNOUT GPU")}else{rm(a,"START GPU SCALING","SPUSTIT GPU ZVĚTŠENÍ")};
  if ui.add_enabled(active||a.telem.active_game.is_some(),egui::Button::new(label)
    .fill(Color32::from_rgb(160,37,74)).min_size(vec2(200.,38.))).clicked()&&!a.is_smoke{
     if active{a.stop_native_mirror();}
     else{
      let key=a.profile_key();
      let selected_mode=a.settings.native_output_modes.get(&key).copied().unwrap_or(0).min(2);
      rm_start(a,rm_mode(selected_mode),1);
     }
  }
  if ui.button(rm(a,"Choose effect","Vybrat efekt")).clicked(){SuiteArea::Visual.go(a);}
  if ui.button(rm(a,"Game tools","Herní nástroje")).clicked(){SuiteArea::Tools.go(a);}
 });
 caption(ui,rm(a,"Scaling shows a separate GPU output. It does not change the game's internal renderer or guarantee extra FPS.",
     "Zvětšení vytváří samostatný GPU výstup. Nemění renderer hry a nezaručuje vyšší FPS."));
 });
 ui.add_space(9.);
 let cards=[SuiteArea::Visual,SuiteArea::Boost,SuiteArea::Tools];
 let cols=if w>720.{3}else if w>440.{2}else{1};
 let cw=(w-8.*(cols-1)as f32)/cols as f32;
 for row in cards.chunks(cols){ui.horizontal(|ui|{
    for &x in row{suite_card(a,ui,x,cw);}
 });ui.add_space(6.);}
 frame(PANEL).show(ui,|ui|{
  ui.horizontal_wrapped(|ui|{
   for x in [SuiteArea::Capture,SuiteArea::Macros,SuiteArea::Overlay,SuiteArea::Library]{
    if ui.small_button(rm_name(a,x)).clicked(){x.go(a);}
   }
  });
 });
}
fn rm_boost(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
 heading(ui,rm(a,"Quick performance presets","Rychlé profily výkonu"));
 caption(ui,rm(a,"Apply a profile, then measure actual game FPS. Presets change Horizon's own rendering budget, not game settings.","Použij profil a změř skutečné FPS hry. Profily mění rozpočet vykreslování Horizon, ne nastavení hry."));
 for(i,en,cs,desc_en,desc_cs)in [(0,"Older GPU / Minecraft","Starší GPU / Minecraft","Lower processing cost, lightweight reconstruction.","Nižší režie, lehká rekonstrukce."),(1,"Balanced","Vyvážený","Moderate processing budget.","Střední rozpočet vykreslování."),(2,"Quality","Kvalita","Full processing, higher GPU cost.","Plné zpracování, vyšší nároky."),(3,"Low latency","Nízká latence","Shorter output queue; no guarantee of lower game latency.","Kratší fronta výstupu; bez záruky nižší latence hry."),(4,"Baseline","Základ","Minimal reconstruction for comparison.","Minimální rekonstrukce pro porovnání.")]{
 ui.horizontal_wrapped(|ui|{if ui.button(rm(a,en,cs)).clicked()&&!a.is_smoke{let id=egui::Id::new("rm-measured-fps");let mut s=ui.data(|d|d.get_temp::<RmMeasure>(id)).unwrap_or_default();s.undo=Some((a.profile_key(),OptConfig::current(a)));s.after=None;ui.data_mut(|d|d.insert_temp(id,s));OPT_PRESETS[i].apply(a);}ui.label(rm(a,desc_en,desc_cs));});
 }
 });
 rm_measure(a,ui);ui.add_space(8.);rm_motion(a,ui);
 egui::CollapsingHeader::new(rm(a,"All performance controls & actual FPS comparison","Všechna nastavení výkonu a měření FPS")).default_open(false).show(ui,|ui|suite_boost(a,ui));
}

fn rm_all_looks(a:&mut BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new(rm(a,"All GPU looks / favorites","Všechny GPU efekty / oblíbené")).show(ui,|ui|{
 let i=a.settings.native_preset;
 let favorite=a.settings.native_favorites.contains(&i);
 if ui.button(if favorite{rm(a,"Remove favorite","Odebrat oblíbený")}else{rm(a,"Add favorite","Přidat oblíbený")}).clicked(){if favorite{a.settings.native_favorites.retain(|n|*n!=i);}else{a.settings.native_favorites.push(i);}a.save();}
 let id=egui::Id::new("rm-all-favorites");let mut only=ui.data(|d|d.get_temp::<bool>(id)).unwrap_or(false);
 ui.checkbox(&mut only,rm(a,"Favorites only","Pouze oblíbené"));ui.data_mut(|d|d.insert_temp(id,only));
 let hint=rm(a,"Search by effect name","Hledat podle názvu efektu");ui.add(egui::TextEdit::singleline(&mut a.filter_search).hint_text(hint).char_limit(64));
 let q=a.filter_search.to_lowercase();
 let entries=crate::native_presets::PRESETS.iter().enumerate().filter(|(j,p)|(!only||a.settings.native_favorites.contains(j))&&(q.is_empty()||p.label.to_lowercase().contains(&q)||p.desc.to_lowercase().contains(&q))).map(|(j,_)|j).collect::<Vec<_>>();
 let mut next=None;
 egui::ScrollArea::vertical().id_salt("rm-all-list").max_height(210.).show(ui,|ui|{for j in entries{let p=&crate::native_presets::PRESETS[j];if ui.selectable_label(i==j,p.label).on_hover_text(p.desc).clicked(){next=Some(j);}}});
 if let Some(j)=next{a.select_native_preset(j);}
 if let Some(t)=native_look_preview_handle(a,ui.ctx(),a.settings.native_preset){let w=ui.available_width().min(320.);ui.add(egui::Image::new(&t).fit_to_exact_size(vec2(w,w*9./16.)));}
 });
}
fn rm_mode(i:u8)->crate::native_mirror::MirrorMode{match i{0=>crate::native_mirror::MirrorMode::BorderlessScreen,1=>crate::native_mirror::MirrorMode::SeparateWindow,_=>crate::native_mirror::MirrorMode::DesktopDuplication}}
fn rm_start(a:&mut BoostApp,mode:crate::native_mirror::MirrorMode,factor:u8){
 if a.is_smoke{return;}
 if a.native_mirror.is_none(){
  if let Err(e)=crate::resource_guard::allowed_to_start(crate::resource_guard::read()){
   a.native_mirror_status=e.clone();a.toast(e);return;
  }
 }
 let key=a.profile_key();
 let mode_id=match mode{crate::native_mirror::MirrorMode::BorderlessScreen=>0,crate::native_mirror::MirrorMode::SeparateWindow=>1,_=>2};
 if a.settings.native_output_modes.get(&key).copied()!=Some(mode_id){
  a.settings.native_output_modes.insert(key,mode_id);a.save();
 }
 if let(Some(w),Some(pid))=(a.native_mirror.as_ref(),a.native_mirror_pid){
 if a.telem.active_game.as_ref().is_some_and(|g|g.pid==pid){
  if w.running()&&w.mode()==mode&&w.multiplier()==factor{a.native_mirror_status="Native output already using selected mode".into();return;}
  a.pending_mirror_restart=Some((pid,mode,factor));w.request_stop();a.output_rates.clear();a.native_mirror_status="Changing native output mode...".into();
 }else{a.native_mirror_status="Output still shutting down after changing game. Retry once it stops.".into();}
 }else{a.toggle_native_mirror_factor(mode,factor);}
}
#[derive(Clone,Default)]struct RmMeasure{before:Option<OptMeasurement>,after:Option<OptMeasurement>,undo:Option<(String,OptConfig)>}
fn rm_measure(a:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("rm-measured-fps");let mut s=ui.data(|d|d.get_temp::<RmMeasure>(id)).unwrap_or_default();
 if s.before.as_ref().is_some_and(|b|b.key!=a.profile_key()||a.telem.active_game.as_ref().is_none_or(|g|g.pid!=b.pid)){s.before=None;s.after=None;}
 frame(PANEL).show(ui,|ui|{
 heading(ui,rm(a,"Measured performance","Naměřený výkon"));
 ui.horizontal_wrapped(|ui|{
 if ui.add_enabled(a.telem.active_game.is_some(),egui::Button::new(rm(a,"Save baseline","Uložit výchozí FPS"))).clicked(){s.before=opt_sample(a);s.after=None;}
 if ui.add_enabled(s.before.is_some(),egui::Button::new(rm(a,"Measure after change","Změřit po změně"))).clicked(){if let(Some(b),Some(v))=(&s.before,opt_sample(a)){if opt_measurement_pair(b,&v){s.after=Some(v);}}}
 });
 if let(Some(b),Some(v))=(&s.before,&s.after){if let Some(p)=opt_delta(b.fps,v.fps){ui.colored_label(if p>=0.{V25_MINT}else{RED},RichText::new(format!("{p:+.1}%")).size(29.).strong());ui.label(format!("{:.1} → {:.1} FPS   |   {:.2} → {:.2} ms",b.fps,v.fps,b.ms,v.ms));}}
 else if let Some(b)=&s.before{ui.label(format!("{} {:.1} FPS",rm(a,"Baseline:","Výchozí:"),b.fps));caption(ui,rm(a,"Change one setting, wait at least 10 seconds in the same scene, then measure.","Změň jedno nastavení, počkej nejméně 10 sekund ve stejné scéně a změř."));}
 else{caption(ui,rm(a,"No valid comparison yet. Run a game and collect frame telemetry first.","Zatím bez měření. Spusť hru a nech nasbírat data snímků."));}
 if let Some(v)=a.output_rates.snapshot(std::time::Instant::now()){ui.label(format!("{} {:.0} + {:.0} FG = {:.0} FPS",rm(a,"Own output:","Vlastní výstup:"),v.real,v.generated,v.total));}
 caption(ui,rm(a,"Measured game FPS and generated display FPS are separate. Scene differences affect results.","Naměřené FPS hry a generované FPS výstupu jsou odlišné. Výsledek ovlivňuje scéna."));
 if ui.add_enabled(s.undo.as_ref().is_some_and(|(k,_)|*k==a.profile_key()),egui::Button::new(rm(a,"Undo quick profile","Vrátit rychlý profil"))).clicked(){if let Some((_,v))=s.undo.take(){v.apply(a);s.after=None;}}
 });
 ui.data_mut(|d|d.insert_temp(id,s));
}
fn rm_motion(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
 heading(ui,rm(a,"Upscaling & motion","Upscaling a plynulost"));rm_game(a,ui);
 caption(ui,rm(a,"Lower the game's resolution yourself to save render work. Horizon reconstructs its output; it does not change the game's resolution.","Sniž rozlišení přímo ve hře, aby méně zatěžovala GPU. Horizon rekonstruuje výstup; nemění rozlišení hry."));
 let mut mode=a.settings.native_scale_mode;
 egui::ComboBox::from_id_salt("rm-scaler").selected_text(crate::spatial::MODES[mode as usize]).show_ui(ui,|ui|{for(i,n)in crate::spatial::MODES.iter().enumerate(){ui.selectable_value(&mut mode,i as u8,*n);}});
 if mode!=a.settings.native_scale_mode{a.apply_scale_algorithm(mode);}
 let mut ratio=a.settings.native_scale_ratio;ui.horizontal_wrapped(|ui|{for v in crate::spatial::RATIOS{ui.selectable_value(&mut ratio,v,format!("{v:.2}x"));}});
 if ratio!=a.settings.native_scale_ratio{a.settings.native_scale_ratio=ratio;a.save();a.refresh_native_mirror();}
 let allow=rm(a,"Enable experimental frame generation","Povolit experimentální generování snímků");if ui.checkbox(&mut a.settings.experimental_fg_allowed,allow).changed(){a.save();}
 ui.horizontal_wrapped(|ui|{
 for f in [1u8,2,3]{let label=if f==1{rm(a,"Start 1x output","Spustit výstup 1x").into()}else{format!("FG {f}x")};
 if ui.add_enabled(!a.is_smoke&&a.telem.active_game.is_some()&&(f==1||a.settings.experimental_fg_allowed),egui::Button::new(label)).clicked(){rm_start(a,crate::native_mirror::MirrorMode::BorderlessScreen,f);}}
 if a.native_mirror.is_some()&&ui.button(rm(a,"Stop output","Vypnout výstup")).clicked(){a.stop_native_mirror();}
 });
 caption(ui,rm(a,"Return focus to the game. FG can add motion artifacts, latency and GPU cost; it does not multiply engine FPS.","Vrať se do hry. FG může přidat artefakty, latenci a zátěž GPU; nenásobí FPS enginu."));
 });
}
