fn oled_advanced(app:&mut BoostApp,ui:&mut Ui){
 if outline_button(ui,local(ui,"More controls","Další ovládání")){app.settings.ui_advanced=true;app.save();}
}
fn oled_title(app:&mut BoostApp,ui:&mut Ui,name:&str){
 ui.horizontal(|ui|{title(ui,name);ui.with_layout(Layout::right_to_left(Align::Center),|ui|oled_advanced(app,ui));});ui.add_space(8.);
}
fn oled_looks(app:&mut BoostApp,ui:&mut Ui){
 oled_action_bar(app,ui);ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  let w=ui.available_width();ui.set_min_width(w);
  let i=app.settings.native_preset.min(crate::native_presets::PRESETS.len()-1);
  let p=&crate::native_presets::PRESETS[i];
  ui.horizontal_wrapped(|ui|{
   if app.settings.native_stack_profiles.contains_key(&app.profile_key()){heading(ui,"Custom effect stack");}else{heading(ui,p.label);}
   let favorite=app.settings.native_favorites.contains(&i);
   if ui.button(if favorite{"★"}else{"☆"}).on_hover_text(local(ui,"Favorite","Oblíbené")).clicked(){if favorite{app.settings.native_favorites.retain(|v|*v!=i);}else{app.settings.native_favorites.push(i);}app.save();}
   if app.settings.native_stack_profiles.contains_key(&app.profile_key()){ui.colored_label(RED,local(ui,"Custom stack active","Vlastní vrstvy aktivní"));}
  });
  ui.horizontal_wrapped(|ui|{
   let hint=local(ui,"Search native looks","Hledat 83 vzhledů");ui.add(egui::TextEdit::singleline(&mut app.filter_search).hint_text(hint).desired_width(180.));
   if ui.small_button(local(ui,"Clear","Vymazat")).clicked(){app.filter_search.clear();}
   if outline_button(ui,local(ui,"Lighting","Osvětlení")){app.filter_view=1;}
   if outline_button(ui,local(ui,"Layers","Vrstvy")){app.filter_view=3;}
  });
  let id=egui::Id::new("oled-gallery-favorites");let mut only=ui.data(|d|d.get_temp::<bool>(id)).unwrap_or(false);
  ui.checkbox(&mut only,local(ui,"Favorites only","Pouze oblíbené"));ui.data_mut(|d|d.insert_temp(id,only));
  ui.horizontal_wrapped(|ui|{
   for(scene,label)in [(0,"Photo reference"),(1,"Gameplay reference")]{ui.selectable_value(&mut app.native_gallery_scene,scene,label);}
   if let Some(ids)=app.settings.native_stack_profiles.get(&app.profile_key()){caption(ui,&ids.iter().filter_map(|&id|crate::native_filter::PROGRAMS.get(id).map(|v|v.0)).collect::<Vec<_>>().join(" / "));}else{caption(ui,p.desc);}
  });
  if let Some(worker)=app.native_mirror.as_ref(){if worker.diagnostics().live_look_rejected{ui.colored_label(RED,"Requested look failed; last valid stack retained.");}}
  let source=native_look_original_handle(app,ui.ctx());let filtered=if app.settings.native_stack_profiles.contains_key(&app.profile_key()){None}else{native_look_preview_handle(app,ui.ctx(),i)};
  ui.horizontal(|ui|{
   for(tex,label)in [(source,local(ui,"Original","Originál")),(filtered,local(ui,"Preset preview","Náhled vzhledu"))]{
    ui.vertical(|ui|{caption(ui,label);if let Some(t)=tex{let iw=((w-12.)*0.5).min(260.);let(r,_)=ui.allocate_exact_size(vec2(iw,iw*0.5625),Sense::hover());ui.painter().image(t.id(),r,Rect::from_min_max(pos2(0.,0.),pos2(1.,1.)),WHITE);}});
   }
  });
  caption(ui,local(ui,"Preset reference only; custom effects use the GPU photo lab.","Skutečný GPU náhled; vlastní osvětlení není v těchto snímcích."));
  ui.horizontal_wrapped(|ui|{let mut t=app.settings.lighting.clone();if ui.checkbox(&mut t.reconstruction.animated_grain,"Animate grain").on_hover_text("Only the film grain stage, 1x output without temporal filtering.").changed(){app.apply_native_lighting(t);}});
  egui::CollapsingHeader::new("Quick effect stacks").show(ui,|ui|{
   ui.horizontal_wrapped(|ui|{for (label,ids) in [("Shadow mask",&[62usize][..]),("Curved arcade",&[63usize][..]),("CRT phosphor",&[19usize][..]),("Lens",&[23usize,48][..]),("Film",&[49usize,59][..]),("Eye comfort",&[44usize][..]),("Deband",&[55usize][..]),("CAS",&[61usize][..])]{if outline_button(ui,label){app.apply_custom_native_stack(ids.to_vec());}}});
   caption(ui,"Selects a native GPU stack; use Apply effects or Preview to start output.");
  });
  egui::CollapsingHeader::new("All individual GPU effects").show(ui,|ui|native_individual_effects(app,ui));
  egui::CollapsingHeader::new("Saved custom looks / A-B").show(ui,|ui|{named_native_looks_panel(app,ui);ui.separator();look_compare_controls(app,ui,true);});
  let query=app.filter_search.trim().to_lowercase();
  let entries=(0..crate::native_presets::PRESETS.len()).filter(|&j|{let p=&crate::native_presets::PRESETS[j];(!only||app.settings.native_favorites.contains(&j))&&(query.is_empty()||p.label.to_lowercase().contains(&query)||p.desc.to_lowercase().contains(&query))}).collect::<Vec<_>>();
  let cols=if w>=700.{4}else if w>=480.{3}else{2};let cw=(w-8.*(cols-1)as f32)/cols as f32;
  let mut keep=vec![false;crate::native_presets::PRESETS.len()];keep[i]=true;
  egui::ScrollArea::vertical().id_salt("oled-look-grid").max_height(325.).show(ui,|ui|{
   if entries.is_empty(){caption(ui,local(ui,"No matches","Žádné výsledky"));}
   for chunk in entries.chunks(cols){ui.horizontal(|ui|{for &j in chunk{
    let(r,res)=click_surface(ui,cw,97.,i==j&&!app.settings.native_stack_profiles.contains_key(&app.profile_key()));let tw=(cw-10.).max(1.).min(61.*16./9.);let thumb=Rect::from_center_size(pos2(r.center().x,r.top()+35.),vec2(tw,tw*9./16.));
    if ui.is_rect_visible(thumb){keep[j]=true;}
    if keep[j]&&let Some(t)=native_look_preview_handle(app,ui.ctx(),j){ui.painter().image(t.id(),thumb,Rect::from_min_max(pos2(0.,0.),pos2(1.,1.)),WHITE);}
    ui.painter().text(r.left_bottom()+vec2(8.,-20.),egui::Align2::LEFT_TOP,trim(crate::native_presets::PRESETS[j].label,21),FontId::proportional(11.),WHITE);
    if app.settings.native_favorites.contains(&j){ui.painter().text(r.right_top()+vec2(-8.,7.),egui::Align2::RIGHT_TOP,"★",FontId::proportional(13.),RED);}
    if res.on_hover_text(crate::native_presets::PRESETS[j].desc).clicked(){app.select_native_preset(j);}
   }});ui.add_space(3.);}
  });
  trim_native_gallery_cache(app,&keep);
 });
}
fn oled_scaling(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Upscaling");oled_action_bar(app,ui);ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  let w=ui.available_width();ui.set_min_width(w);
  ui.horizontal_wrapped(|ui|{
    for (i,label) in ["Low overhead","Balanced","Full quality"].into_iter().enumerate(){
      if outline_button(ui,label){app.apply_render_profile(crate::render_profiles::PROFILES[i]);}
    }
  });
  caption(ui,"Processing size controls Horizon's GPU cost; it does not lower game rendering resolution.");
  let cap=app.settings.preview_fps_limit;
  ui.horizontal_wrapped(|ui|{ui.label("1x output cap");for n in crate::preview_pacer::LIMITS{if ui.selectable_label(cap==n,if n==0{"Off".into()}else{n.to_string()}).clicked(){app.settings.preview_fps_limit=n;app.preview_guard.reset();if let Some(w)=app.native_mirror.as_ref(){w.set_preview_limit(n);}app.save();}}});
  if ui.checkbox(&mut app.settings.preview_composition,"DirectComposition output (separate 1x, experimental)").changed(){app.save();app.refresh_native_mirror();}
  heading(ui,local(ui,"Reconstruction","Rekonstrukce obrazu"));
  let mut mode=app.settings.native_scale_mode;let mut ratio=app.settings.native_scale_ratio;let mut processing=app.settings.processing_percent;let mut quality=app.settings.native_quality;
  egui::ComboBox::from_id_salt("oled-scale-algorithm").width(w.min(350.)).selected_text(crate::spatial::MODES[mode.min((crate::spatial::MODES.len()-1)as u8)as usize]).show_ui(ui,|ui|{for(i,n)in crate::spatial::MODES.iter().enumerate(){ui.selectable_value(&mut mode,i as u8,*n);}});
  ui.add_space(5.);
  ui.horizontal_wrapped(|ui|{ui.label(local(ui,"Output size","Velikost výstupu"));for r in crate::spatial::RATIOS{ui.selectable_value(&mut ratio,r,format!("{r:.2}x"));}});
  ui.horizontal_wrapped(|ui|{ui.label(local(ui,"Processing size","Velikost zpracování"));for r in crate::render_budget::PERCENTAGES{ui.selectable_value(&mut processing,r,format!("{r}%"));}});
  ui.horizontal_wrapped(|ui|{ui.label(local(ui,"GPU budget","Zátěž GPU"));for(i,n)in [(3,"Adaptive"),(2,"Eco"),(1,"Balanced"),(0,"Full")]{ui.selectable_value(&mut quality,i,tr(ui,n));}});
  if outline_button(ui,local(ui,"Reset scaling","Obnovit zvětšení")){mode=0;ratio=1.;processing=100;quality=3;}
  if(mode,ratio,processing,quality)!=(app.settings.native_scale_mode,app.settings.native_scale_ratio,app.settings.processing_percent,app.settings.native_quality){
   if (ratio,processing)==(app.settings.native_scale_ratio,app.settings.processing_percent){
    if quality!=app.settings.native_quality{app.apply_gpu_quality(quality);}
    if mode!=app.settings.native_scale_mode{app.apply_scale_algorithm(mode);}
   }else{
    app.settings.native_scale_mode=mode;app.settings.native_scale_ratio=ratio;app.settings.processing_percent=processing;app.settings.native_quality=quality;
    app.settings.processing_profiles.insert(app.profile_key(),processing);app.save();app.refresh_native_mirror();
   }
  }
  if mode==14{caption(ui,"Anime4K CNN S: four learned convolution layers + depth-to-space. Requires exact 2x dimensions; other ratios use bilinear. Select separate output or 50% processing. Higher GPU cost than SGSR1.");}
  if mode==13{caption(ui,"Qualcomm SGSR1: spatial edge reconstruction, up to 2x. Lower-cost option.");}
  if mode==12{caption(ui,local(ui,"Integer fit: equal-sized source pixels, centered black borders if needed. Smaller output uses nearest.","Celé násobky pixelů, případně černé okraje. Menší výstup používá nejbližší pixel."));}
  if let Some(w)=app.native_mirror.as_ref(){let h=w.diagnostics();
   if let (Some(src),Some(dst),Some(m))=(h.scale_input,h.scale_output,h.effective_scale_mode){ui.label(format!("{} x {}  ->  {} x {}",src.0,src.1,dst.0,dst.1));caption(ui,&format!("Actual: {}",crate::spatial::MODES.get(m as usize).unwrap_or(&"Unknown")));if h.scale_runtime_fallback{ui.colored_label(RED,"Algorithm fallback active. Change algorithm to retry.");}}
   caption(ui,&format!("Active stages: {} | WGC interval: {}",h.effective_passes,if h.capture_interval_active{"native"}else{"software"}));}
  caption(ui,local(ui,"Changes Horizon output only, never the game resolution.","Mění pouze výstup Horizon, nikoli rozlišení hry."));
 });
 egui::CollapsingHeader::new("Monitor-specific FPS profiles").show(ui,|ui|monitor_fps_panel(app,ui));
 egui::CollapsingHeader::new("Output, VSync and GPU diagnostics").show(ui,|ui|capture_health_panel(app,ui));
}
fn oled_motion(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Motion Lab");
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Frame generation","Generování snímků"));
  let mut allow=app.settings.experimental_fg_allowed;
  if ui.checkbox(&mut allow,local(ui,"Experimental interpolation","Experimentální interpolace")).changed(){
   app.settings.experimental_fg_allowed=allow;
   if !allow{if app.native_mirror.as_ref().is_some_and(|w|w.multiplier()>1){app.stop_native_mirror();}app.settings.external_fg_factor=1;}
   else{app.settings.external_fg_factor=2;}app.save();
  }
  ui.add_enabled_ui(allow,|ui|{
   ui.horizontal(|ui|{for f in [2u8,3]{if ui.selectable_label(app.settings.external_fg_factor==f,format!("{f}x")).clicked(){app.change_fg_multiplier(f);}}});
   let active=v22_gpu_active(app);
   let stopping=app.native_mirror.is_some()&&!active;
   if active&&app.native_mirror.as_ref().is_some_and(|w|w.multiplier()==1){caption(ui,"1x effects are active. Stop them before starting interpolation.");}
   let mut tuning=app.settings.lighting.clone();
   if ui.checkbox(&mut tuning.motion.adaptive_fg,local(ui,"Adaptive 3x → 2x → 1x","Adaptivní 3x → 2x → 1x")).changed(){app.apply_native_lighting(tuning);}
   if stopping{caption(ui,local(ui,"Stopping previous preview…","Ukončuji předchozí náhled…"));}
   if let Some(w)=app.native_mirror.as_ref().filter(|w|w.multiplier()>1){let h=w.diagnostics();ui.label(if h.paused{"Paused".into()}else if h.fg_effective==0{"Starting".into()}else{format!("Effective: {}x | {}",h.fg_effective,crate::render_pressure::fg_reason(h.fg_reason))});}
   if ui.add_enabled(!app.is_smoke&&!stopping&&(active||app.telem.active_game.is_some()),egui::Button::new(if active{local(ui,"Stop preview","Ukončit náhled")}else{local(ui,"Start preview","Spustit náhled")})).clicked(){
    if active{app.stop_native_mirror();}else{app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,app.settings.external_fg_factor);}
   }
  });
  caption(ui,local(ui,"Separate preview only. May add latency or artifacts; game FPS stays unchanged.","Pouze samostatný náhled. Může přidat zpoždění nebo artefakty; FPS hry se nemění."));
 });
 ui.add_space(8.);
 fg_readout_panel(app,ui);
 let w=(ui.available_width()-16.)/3.;
 let rates=app.output_rates.snapshot(std::time::Instant::now());
 let num=|v:Option<f64>|v.map(|v|format!("{v:.1}")).unwrap_or("—".into());
 ui.horizontal(|ui|{
  v25_metric(ui,w,local(ui,"Game FPS","FPS hry"),&fps_text(app,"fps"),WHITE);
  v25_metric(ui,w,local(ui,"Real output","Skutečný výstup"),&num(rates.map(|v|v.real)),V25_MINT);
  v25_metric(ui,w,local(ui,"Generated","Generované"),&num(rates.map(|v|v.generated)),RED);
 });
 egui::CollapsingHeader::new("Motion quality, HUD mask and GPU test").show(ui,|ui|motion_quality_panel(app,ui));
}
fn oled_performance(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"FPS & Stability");
 egui::CollapsingHeader::new("Optimize / presets & PC analysis").default_open(true).id_salt("hb-optimizer-simple").show(ui,|ui|optimizer_panel(app,ui));
 ui.add_space(10.);
 let w=(ui.available_width()-16.)/3.;
 ui.horizontal(|ui|{v25_metric(ui,w,"FPS",&fps_text(app,"fps"),V25_MINT);v25_metric(ui,w,local(ui,"Frame time","Čas snímku"),&fps_text(app,"frametime"),RED);v25_metric(ui,w,"1% low",&fps_text(app,"low"),V25_LAVENDER);});
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Frame stability","Stabilita snímků"));
  let points=app.fps.frames.iter().rev().take(120).rev().filter_map(|f|(f.ms.is_finite()&&f.ms>0.).then_some(f.ms)).collect::<Vec<_>>();
  let(r,_)=ui.allocate_exact_size(vec2(ui.available_width(),100.),Sense::hover());ui.painter().rect_filled(r,7.,BG);
  if points.len()>1&&app.fps.snapshot().is_some(){let max=points.iter().copied().fold(33.3f32,f32::max);
   let line=points.iter().enumerate().map(|(i,v)|pos2(r.left()+r.width()*i as f32/(points.len()-1)as f32,r.bottom()-r.height()*(*v/max).clamp(0.,1.))).collect();
   let budget=1000./app.target_fps.clamp(20,360) as f32;let y=r.bottom()-r.height()*(budget/max).clamp(0.,1.);ui.painter().line_segment([pos2(r.left(),y),pos2(r.right(),y)],Stroke::new(1.0_f32,V25_MINT));
   ui.painter().add(egui::Shape::line(line,Stroke::new(1.5_f32,RED)));
   if let Some(h)=crate::perf_insights::FrameHealth::from_frames(&app.fps.frames){ui.label(format!("P95 {:.1} ms  /  P99 {:.1} ms  /  {} {}",h.p95_ms,h.p99_ms,h.spike_count,local(ui,"spikes","výkyvů")));}
  }else{ui.painter().text(r.center(),egui::Align2::CENTER_CENTER,local(ui,"Waiting for real game frames","Čekám na skutečné snímky hry"),FontId::proportional(12.),MUTED);}
  if app.fps.snapshot().is_some(){if let Some((miss,n))=crate::perf_insights::FrameHealth::target_misses(&app.fps.frames,app.target_fps){caption(ui,&format!("{} FPS target: {miss}/{n} frames over budget",app.target_fps));}}
  ui.horizontal_wrapped(|ui|{for target in [30u32,45,60,75,90,100,120,144,165]{if ui.selectable_label(app.target_fps==target,format!("{target} FPS")).clicked(){app.target_fps=target;app.save();}}});
  if app.fps.snapshot().is_some(){if let Some(g)=crate::perf_insights::gpu_coverage(&app.fps.frames){caption(ui,&format!("GPU time coverage: {}/{} frames | median GPU/frame {:.0}% | near frame budget {}/{}",g.paired,g.valid,g.median_ratio*100.,g.near_budget,g.paired));caption(ui,"Timing association only; not proof of a CPU/GPU bottleneck.");}else{caption(ui,"Insufficient paired GPU timing samples.");}}
  egui::CollapsingHeader::new("Measured display latency").show(ui,|ui|{
   let samples=crate::perf_insights::latency_points(&app.fps.frames);let(r,_)=ui.allocate_exact_size(vec2(ui.available_width(),75.),Sense::hover());ui.painter().rect_filled(r,5.,BG);
   if app.fps.snapshot().is_some()&&samples.iter().filter(|v|v.is_some()).count()>=2{let max=samples.iter().flatten().copied().fold(10f32,f32::max);for(i,pair)in samples.windows(2).enumerate(){if let [Some(a),Some(b)]=pair{let x=|j:usize|r.left()+r.width()*j as f32/(samples.len()-1)as f32;let y=|v:f32|r.bottom()-r.height()*(v/max).clamp(0.,1.);ui.painter().line_segment([pos2(x(i),y(*a)),pos2(x(i+1),y(*b))],Stroke::new(1.3,V25_LAVENDER));}}caption(ui,&format!("Display latency, 0–{max:.1} ms; missing samples remain gaps."));}else{ui.painter().text(r.center(),egui::Align2::CENTER_CENTER,"ETW display latency unavailable",FontId::proportional(11.),MUTED);}
  });
  let label=local(ui,"Warn about stutters","Upozornit na záseky");if ui.checkbox(&mut app.settings.stutter_scout_enabled,label).changed(){app.save();}
  if app.settings.stutter_scout_enabled{if ui.add(egui::Slider::new(&mut app.settings.stutter_threshold_ms,20.0..=200.).suffix(" ms")).changed(){app.save();}}
 });
 ui.add_space(8.);
 ui.horizontal(|ui|{
  let cpu=if app.telem.active_game.is_some(){format!("{:.1}",app.telem.cpu_percent/100.)}else{"—".into()};
  let ram=if app.telem.active_game.is_some(){format!("{} MB",app.telem.game_ram_mb)}else{"—".into()};
  let sys=if app.telem.total_ram_gb>0.{format!("{:.1}/{:.1} GB",app.telem.used_ram_gb,app.telem.total_ram_gb)}else{"—".into()};
  v25_metric(ui,w,local(ui,"CPU cores","Jádra CPU"),&cpu,RED);v25_metric(ui,w,local(ui,"Game RAM","RAM hry"),&ram,V25_MINT);v25_metric(ui,w,"RAM",&sys,V25_LAVENDER);
 });
 ui.add_space(8.);
 ui.horizontal_wrapped(|ui|{
  for(i,en,cz)in [(1,"Tuning","Ladění"),(2,"Analysis","Analýza"),(3,"Stability","Stabilita"),(4,"CPU affinity","Jádra CPU")]{if outline_button(ui,local(ui,en,cz)){app.performance_view=i;app.settings.ui_advanced=true;app.save();}}
 });
}
fn oled_tools(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Game Tools");
 let tools=[("Crosshair","Zaměřovač"),("Timers","Časovače"),("Quests","Úkoly"),("Training","Trénink"),("Quick actions","Rychlé akce"),("Macros","Makra"),("View tools","Obraz"),("Reminders","Připomínky"),("Controller","Ovladač"),("Session lab","Relace")];
 let w=ui.available_width();let cols=if w>=700.{3}else if w>=420.{2}else{1};let cw=(w-8.*(cols-1)as f32)/cols as f32;
 for(row,chunk)in tools.chunks(cols).enumerate(){ui.horizontal(|ui|{for(col,(en,cz))in chunk.iter().enumerate(){let i=row*cols+col;if oled_choice(ui,local(ui,en,cz),local(ui,"Open tool","Otevřít nástroj"),[Page::Overlay,Page::FrameGen,Page::Library,Page::Extras,Page::Capture,Page::Settings,Page::Filters,Page::Home,Page::Extras,Page::Performance][i],app.extras_view==i,cw){app.extras_view=i;}}});}
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Quick controls","Rychlé ovládání"));
  ui.horizontal_wrapped(|ui|{
   if ui.add_enabled(app.settings.crosshair.enabled||app.telem.active_game.is_some(),egui::Button::new(if app.settings.crosshair.enabled{"Stop crosshair"}else{"Apply crosshair"})).clicked()&&!app.is_smoke{app.toggle_crosshair();}
   if outline_button(ui,if app.overlay_open{tr(ui,"Hide HUD")}else{tr(ui,"Open HUD")})&&!app.is_smoke{app.toggle_passive_hud();}
   if outline_button(ui,if app.game_timer.running(){local(ui,"Pause timer","Pozastavit časovač")}else{local(ui,"Start timer","Spustit časovač")})&&!app.is_smoke{app.toggle_game_timer();}
   ui.label(RichText::new(crate::gametimer::RunTimer::fmt_elapsed(app.game_timer.elapsed())).monospace());
   if app.window_macro.busy(){if red_button(ui,"Stop macro / F12"){app.window_macro.cancel();}}
  });
 });
 ui.add_space(10.);
 heading(ui,local(ui,tools[app.extras_view.min(9)].0,tools[app.extras_view.min(9)].1));
 game_extras_body(app,ui);
}
fn oled_overlay(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Game Overlay");
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Game HUD","Herní HUD"));
  hud_monitor_options(app,ui);
  ui.horizontal_wrapped(|ui|{
   if red_button(ui,if app.overlay_open{tr(ui,"Hide HUD")}else{tr(ui,"Open HUD")})&&!app.is_smoke{app.toggle_passive_hud();}
   if outline_button(ui,local(ui,"Edit HUD","Upravit HUD"))&&!app.is_smoke{app.overlay_open=true;app.overlay_edit_mode=true;}
   ui.label(format!("{}{}",app.settings.global_shortcuts.label("HUD"),if app.settings.hud_f12{" / F12"}else{""}));
  });
  let s=local(ui,"Size","Velikost");if ui.add(egui::Slider::new(&mut app.settings.overlay_size_scale,0.78..=1.35).text(s)).changed(){app.save();}
  let s=local(ui,"Opacity","Průhlednost");if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.).text(s)).changed(){app.save();}
  if app.overlay_open{ui.colored_label(V25_MINT,if app.overlay_edit_mode{"HUD editor"}else if app.last_overlay_passive==Some(true){"Passive HUD configured"}else{"HUD initializing"});}
  caption(ui,local(ui,"Passive HUD is click-through. Exclusive fullscreen is unverified.","Pasivní HUD propouští kliknutí. Exkluzivní fullscreen není ověřen."));
 });
 ui.add_space(8.);
 egui::CollapsingHeader::new("HUD layout / drag position").show(ui,|ui|hud_layout_panel(app,ui));
 egui::CollapsingHeader::new("Global shortcuts").show(ui,|ui|shortcut_panel(app,ui));
 if outline_button(ui,local(ui,"HUD layout and compatibility","Rozložení HUD a kompatibilita")){app.page=Page::Settings;app.settings.ui_advanced=true;app.save();}
}
fn oled_capture(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Capture");
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Screenshots and video","Snímky a video"));
  ui.horizontal_wrapped(|ui|{
   if red_button(ui,local(ui,"Screenshot","Snímek obrazovky"))&&!app.is_smoke{app.screenshot();}
   if outline_button(ui,if app.recorder.active(){local(ui,"Stop recording","Ukončit záznam")}else{local(ui,"Record desktop","Nahrát plochu")})&&!app.is_smoke{app.recording();}
  });
  caption(ui,local(ui,"Recording captures the desktop. Audio is off unless explicitly selected in Advanced.","Záznam snímá plochu. Zvuk je vypnutý, pokud ho výslovně nezapneš v pokročilém režimu."));
  if app.recorder.active(){ui.colored_label(RED,local(ui,"Recording","Nahrávání"));}
  if app.recorder.finishing(){ui.spinner();}
  ui.add_enabled_ui(!app.recorder.busy()&&!app.replay.busy(),|ui|{
   let label=local(ui,"Bitrate","Datový tok");if ui.add(egui::Slider::new(&mut app.settings.capture_mbps,2..=16).suffix(" Mbps").text(label)).changed(){app.save();}
   let label=local(ui,"Hardware encoder","Hardwarový enkodér");if ui.checkbox(&mut app.settings.hardware_encoder,label).changed(){app.save();}
  });
 });
 ui.add_space(8.);
 if outline_button(ui,local(ui,"Replay / audio / media library","Replay / zvuk / knihovna médií")){app.settings.ui_advanced=true;app.save();}
}
fn oled_library(app:&mut BoostApp,ui:&mut Ui){
 oled_title(app,ui,"Game Library");
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Running games","Spuštěné hry"));
  if outline_button(ui,local(ui,"Refresh","Obnovit"))&&!app.is_smoke{app.telem.refresh(true);}
  let games=app.telem.visible_games.clone();
  if games.is_empty(){caption(ui,local(ui,"No detected game. Start a game and refresh.","Žádná rozpoznaná hra. Spusť hru a obnov seznam."));}
  for g in games{let selected=app.telem.selected_pid==Some(g.pid);if ui.selectable_label(selected,trim(&g.title,60)).clicked()&&!app.is_smoke{app.choose_game(g.pid);}}
  if outline_button(ui,local(ui,"Automatic selection","Automatický výběr"))&&!app.is_smoke{app.telem.unlock_game();app.telem.refresh(true);}
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Saved library","Uložené hry"));
  let entries=app.settings.curated_games.clone();
  if entries.is_empty(){caption(ui,local(ui,"Add games in advanced library.","Přidej hry v pokročilé knihovně."));}
  let hint=local(ui,"Find saved game","Hledat uloženou hru");
   ui.add(egui::TextEdit::singleline(&mut app.library_filter).hint_text(hint).char_limit(100));
   if !app.library_filter.is_empty()&&ui.small_button(local(ui,"Clear","Vymazat")).clicked(){app.library_filter.clear();app.library_confirm=None;}
   let mut count=0;
   egui::ScrollArea::vertical().id_salt("simple-saved-games").max_height(330.).show(ui,|ui|{
    for (i,e) in entries.iter().enumerate(){
     if !crate::command_search::matches(&app.library_filter,&e.title){continue;}
     count+=1;ui.horizontal_wrapped(|ui|{
      ui.label(RichText::new(&e.title).strong());caption(ui,&e.source);
      if e.executable.is_some(){
       if app.library_confirm==Some(i){
        if red_button(ui,local(ui,"Confirm launch","Potvrdit spuštění"))&&!app.is_smoke{
         app.library_confirm=None;match crate::game_library::launch(e){Ok(())=>app.toast("Game launched"),Err(err)=>app.toast(&err)}
        }
        if outline_button(ui,local(ui,"Cancel","Zrušit")){app.library_confirm=None;}
       }else if outline_button(ui,local(ui,"Launch","Spustit")){app.library_confirm=Some(i);}
      }else if outline_button(ui,local(ui,"Choose executable","Vybrat program")){app.settings.ui_advanced=true;app.save();}
     });ui.separator();
    }
   });
   if !entries.is_empty()&&count==0{caption(ui,local(ui,"No matching saved games","Žádné odpovídající hry"));}
  if outline_button(ui,local(ui,"Manage library and profiles","Správa knihovny a profilů")){app.settings.ui_advanced=true;app.save();}
 });
}

fn monitor_fps_panel(app:&mut BoostApp,ui:&mut Ui){
 if ui.checkbox(&mut app.settings.monitor_fps_auto_apply,"Auto-apply saved FPS profile when game or output monitor changes").changed(){app.monitor_profile_applied=None;app.save();}
 let id=ui.id().with("monitor-fps-selection");let mut selected=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_default();
 if outline_button(ui,"Refresh monitor list")&&!app.is_smoke{app.hud_monitor_cache=Some((std::time::Instant::now(),crate::display_probe::read()));}
 let rows=app.hud_monitor_cache.as_ref().map(|(_,v)|v.iter().filter_map(|v|v.target()).collect::<Vec<_>>()).unwrap_or_default();
 if selected.is_empty(){selected=app.settings.preview_display.clone().or_else(||rows.first().map(|v|v.name.clone())).unwrap_or_default();}
 egui::ComboBox::from_id_salt(id.with("display")).selected_text(if selected.is_empty(){"Choose monitor"}else{&selected}).show_ui(ui,|ui|{for d in &rows{ui.selectable_value(&mut selected,d.name.clone(),format!("{} {}x{} / {}",d.name,d.width,d.height,d.refresh.map(|v|format!("{v} Hz")).unwrap_or("unknown Hz".into())));}});
 let key=app.profile_key();let current=crate::monitor_fps_profiles::chosen(&app.settings.monitor_fps_profiles,&key,Some(&selected));
 if selected!=app.settings.preview_display.as_deref().unwrap_or("")&&rows.iter().any(|d|d.name==selected)&&outline_button(ui,"Use selected monitor for own output") {app.settings.preview_display=Some(selected.clone());app.monitor_profile_applied=None;app.save();app.refresh_native_mirror();}
 let rec=rows.iter().find(|v|v.name==selected).and_then(|v|crate::monitor_fps_profiles::recommended(v.refresh));
 let mut apply=None;
 ui.horizontal_wrapped(|ui|{
  if ui.add_enabled(rec.is_some(),egui::Button::new("Use nominal refresh recommendation")).clicked(){apply=rec;}
  if ui.add_enabled(current.is_some(),egui::Button::new("Load saved monitor profile")).clicked(){apply=current;}
  if ui.add_enabled(!selected.is_empty(),egui::Button::new("Save current target / cap")).clicked(){let p=crate::monitor_fps_profiles::Profile{target:app.target_fps,cap:app.settings.preview_fps_limit};match crate::monitor_fps_profiles::put(&mut app.settings.monitor_fps_profiles,&key,&selected,p){Ok(())=>{app.save();app.toast("Monitor FPS profile saved");},Err(e)=>app.toast(e)}}
 });
 if let Some(p)=apply{app.target_fps=p.target;app.settings.preview_fps_limit=p.cap;app.preview_guard.reset();app.render_guard.reset();if let Some(w)=app.native_mirror.as_ref(){w.set_preview_limit(p.cap);}app.save();}
 if let Some(p)=current{caption(ui,&format!("Saved target {} FPS / own cap {}",p.target,p.cap));}
 caption(ui,"Nominal monitor refresh only; not a VRR measurement. Profiles never change game FPS limits or Windows display settings.");
 ui.data_mut(|d|d.insert_temp(id,selected));
}
