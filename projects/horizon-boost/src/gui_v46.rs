fn process_load_panel(app:&BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"GAME VS HORIZON / READ-ONLY CPU & RAM");
  for (name,v) in [("Horizon",app.process_load.own_sample),("Pinned game",app.process_load.game_sample)]{
   ui.horizontal_wrapped(|ui|{ui.label(name);caption(ui,&v.map(|s|format!("CPU {} / RAM {}",s.cpu.map(|x|format!("{x:.1}%")).unwrap_or("Collecting".into()),s.ram.map(|x|format!("{x:.1} MiB")).unwrap_or("Unavailable".into()))).unwrap_or("Unavailable".into()));});
  }
  caption(ui,"One-second CPU time deltas normalized by available logical processors; resident working set. No invented GPU load or temperature, no process modification.");
 });
}
fn pipeline_profile_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"PIPELINE PROFILER / GPU TIMESTAMPS");
  caption(ui,"Ten GPU measurements at each of 720p, 1080p and 1440p. Synthetic resource copy, shaders, area downsample, FSR1, NIS, SGSR1, Anime4K, motion estimate, warp and combined processing. Not WGC or game latency.");
  let idle=app.native_mirror.is_none()&&app.effect_bench_rx.is_none()&&app.photo_rx.is_none()&&app.motion_lab_rx.is_none()&&app.pipeline_bench_rx.is_none();
  if ui.add_enabled(idle,egui::Button::new("Profile synthetic pipeline")).clicked()&&!app.is_smoke{
   let stack=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());
   let(tx,rx)=std::sync::mpsc::channel();app.pipeline_bench_rx=Some(rx);app.pipeline_bench_result=None;
   std::thread::spawn(move||{let _=tx.send(crate::pipeline_bench::run(&stack));});
  }
  if app.pipeline_bench_rx.is_some(){ui.spinner();}
  match &app.pipeline_bench_result{
   Some(Ok(rows))=>{
    if outline_button(ui,"Copy pipeline JSON"){if let Ok(v)=serde_json::to_string_pretty(rows){ui.ctx().copy_text(v);}}
    for height in [720,1080,1440]{egui::CollapsingHeader::new(format!("{height}p")).default_open(height==720).show(ui,|ui|{
     for row in rows.iter().filter(|r|r.height==height){ui.horizontal_wrapped(|ui|{ui.label(&row.label);caption(ui,&row.gpu_ms.map(|x|format!("{x:.3} ms")).unwrap_or("Unavailable".into()));});}
    });}
   },Some(Err(e))=>caption(ui,e),None=>{}
  }
 });
}
fn hud_layout_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"CUSTOM HUD WORKSPACE");
  let mut h=app.settings.hud_layout.clone();let mut changed=false;
  changed|=ui.checkbox(&mut h.enabled,"Use custom widget grid").changed();
  changed|=ui.checkbox(&mut h.graph,"Frametime graph").changed();
  changed|=ui.checkbox(&mut h.audio,"Audio-reactive accent (read output peak)").changed();
  caption(ui,"Output amplitude only, no audio samples or microphone. Audio reads stop when the HUD closes. Passive HUD remains click-through.");
  let mut remove=None;let mut move_by=None;
  for(i,w)in h.widgets.iter_mut().enumerate(){ui.push_id(i,|ui|{
   ui.horizontal_wrapped(|ui|{
    egui::ComboBox::from_id_salt("hud-widget").selected_text(crate::hud_layout::NAMES[w.metric.min((crate::hud_layout::NAMES.len()-1) as u8) as usize]).show_ui(ui,|ui|{
     for(j,name)in crate::hud_layout::NAMES.iter().enumerate(){changed|=ui.selectable_value(&mut w.metric,j as u8,*name).changed();}
    });
    if ui.small_button("Up").clicked(){move_by=Some((i,-1));}
    if ui.small_button("Down").clicked(){move_by=Some((i,1));}
    if outline_button(ui,"Remove"){remove=Some(i);}
   });
   changed|=ui.add(egui::Slider::new(&mut w.scale,0.8..=1.25).text("Text scale")).changed();
   changed|=ui.add(egui::Slider::new(&mut w.opacity,0.4..=1.).text("Opacity")).changed();
   changed|=ui.checkbox(&mut w.mono,"Monospaced value").changed();
  });}
  if let Some(i)=remove{if h.widgets.len()>1{h.widgets.remove(i);changed=true;}}else if let Some((i,delta))=move_by{changed|=h.move_widget(i,delta);}
  if h.widgets.len()<6&&outline_button(ui,"Add widget"){h.widgets.push(Default::default());changed=true;}
  egui::CollapsingHeader::new("Passive HUD display / physical placement").show(ui,|ui|{
   if outline_button(ui,"Refresh displays")&&!app.is_smoke{app.hud_monitor_cache=Some((std::time::Instant::now(),crate::display_probe::read()));}
   let rows=app.hud_monitor_cache.as_ref().map(|(_,v)|v.as_slice()).unwrap_or(&[]);
   egui::ComboBox::from_id_salt("hud-display-choice").selected_text(h.display.as_deref().unwrap_or("Default / primary")).show_ui(ui,|ui|{
    changed|=ui.selectable_value(&mut h.display,None,"Default / primary").changed();
    for d in rows{if let Some(t)=d.target(){changed|=ui.selectable_value(&mut h.display,Some(t.name.clone()),format!("{} {}x{}{}",t.name,t.width,t.height,if d.primary{" primary"}else{""})).changed();}}
   });
   caption(ui,&format!("Actual own HUD DPI: {}",app.hud_observed_dpi.map(|v|v.to_string()).unwrap_or("Unavailable".into())));
   caption(ui,"Only this app's HUD moves; a disconnected display falls back to primary. Mixed-DPI hardware validation pending.");
  });
  let mut custom=h.custom_position.is_some();
  if ui.checkbox(&mut custom,"Custom position on selected display").changed(){h.custom_position=custom.then_some([0.8,0.1]);changed=true;}
  if let Some(p)=h.custom_position.as_mut(){
   let(r,response)=ui.allocate_exact_size(egui::vec2(ui.available_width().min(320.),135.),egui::Sense::click_and_drag());
   ui.painter().rect_filled(r,5.,BG);
   let marker=egui::vec2((r.width()*0.23).clamp(32.,72.),24.);
   if (response.dragged()||response.clicked())&&let Some(q)=response.interact_pointer_pos(){*p=crate::hud_layout::drag_anchor([q.x-r.left(),q.y-r.top()],[r.width(),r.height()],[marker.x,marker.y]);changed=true;}
   let pos=egui::pos2(r.left()+(r.width()-marker.x)*p[0],r.top()+(r.height()-marker.y)*p[1]);let rect=egui::Rect::from_min_size(pos,marker);ui.painter().rect_filled(rect,4.,CARD);ui.painter().rect_stroke(rect,4.,Stroke::new(1.0_f32,RED),StrokeKind::Inside);ui.painter().text(rect.center(),egui::Align2::CENTER_CENTER,"HUD",FontId::proportional(10.),WHITE);
   caption(ui,"Drag the HUD preview. All four edges remain inside the monitor; changes apply immediately. Mixed-DPI test pending.");
  }
  ui.horizontal_wrapped(|ui|{for(label,p)in [("Top left",[0.,0.]),("Top right",[1.,0.]),("Bottom left",[0.,1.]),("Bottom right",[1.,1.]),("Center",[0.5,0.5])]{if outline_button(ui,label){h.custom_position=Some(p);changed=true;}}});
  if outline_button(ui,"Reset HUD layout"){h=Default::default();changed=true;}
  if changed{h.sanitize();app.settings.hud_layout=h;app.save();}
 });
}
fn widget_hud(app:&BoostApp,ui:&mut Ui){
 if app.settings.hud_layout.quest_list{
  let key=app.profile_key();let now=crate::utility_book::now();
  if let Some(book)=app.settings.utility_profiles.get(&key){
   for q in book.quests.iter().take(3){ui.label(egui::RichText::new(format!("{} {}",if q.done(now){"[x]"}else{"[ ]"},q.name.chars().take(30).collect::<String>())).size(10.));}
  }
 }
 let h=&app.settings.hud_layout;let z=app.settings.overlay_size_scale.clamp(0.78,1.35);
 let alpha=|c:Color32,a:f32|c.gamma_multiply(a);
 let spectrum=if h.audio&&app.spectrum_enabled{app.spectrum.reading()}else{None};
 let peak=if h.audio{spectrum.map(|r|r.peak).or_else(||app.audio_meter.value.as_ref().and_then(|v|v.as_ref().ok()).copied()).unwrap_or(0.)}else{0.};
 let value=|metric:u8|->String{match metric{
 0=>hud_stats_text(app,"fps"),1=>hud_stats_text(app,"frametime"),2=>hud_stats_text(app,"low"),3=>hud_stats_text(app,"gpu"),
 4=>app.process_load.own_sample.and_then(|s|s.ram).map(|v|format!("{v:.0} MiB")).unwrap_or("--".into()),
 5=>app.network_probe.stats().and_then(|v|v.0).map(|v|format!("{v} ms")).unwrap_or("--".into()),
 6=>{let s=app.game_timer.elapsed().as_secs();format!("{:02}:{:02}",s/60,s%60)},
 7=>app.output_rates.snapshot(std::time::Instant::now()).filter(|_|app.hud_output_matches()).map(|v|format!("{:.0}/s",v.total)).unwrap_or("--".into()),
 8=>app.hud_snapshot().and_then(|_|app.hud_meter().low_01_fps()).map(|v|format!("{v:.0}")).unwrap_or("--".into()),
 9=>app.process_load.game_sample.filter(|_|app.hud_uses_main).and_then(|s|s.ram).map(|v|format!("{v:.0} MiB")).unwrap_or("--".into()),
 10=>app.process_load.game_sample.filter(|_|app.hud_uses_main).and_then(|s|s.cpu).map(|v|format!("{v:.1}%")).unwrap_or("--".into()),
 11=>app.process_load.own_sample.and_then(|s|s.cpu).map(|v|format!("{v:.1}%")).unwrap_or("--".into()),
 12=>app.network_probe.stats().map(|v|format!("{:.0}%",v.1)).unwrap_or("--".into()),
 13=>app.network_probe.distribution().and_then(|v|v.jitter).map(|v|format!("{v:.1} ms")).unwrap_or("--".into()),
 14=>app.native_mirror.as_ref().filter(|w|w.running()&&app.hud_output_matches()).and_then(|w|w.diagnostics().present_age_ms).map(|v|format!("{v:.1} ms")).unwrap_or("--".into()),
 15=>app.output_rates.snapshot(std::time::Instant::now()).filter(|_|app.hud_output_matches()).map(|v|format!("{:.0}/s",v.generated)).unwrap_or("--".into()),
 16=>app.settings.game_notes.get(&app.profile_key()).map(|s|s.chars().take(18).collect()).unwrap_or("--".into()),
 17=>app.settings.utility_profiles.get(&app.profile_key()).map(|b|format!("{}/{}",b.quests.iter().filter(|q|q.done(crate::utility_book::now())).count(),b.quests.len())).unwrap_or("--".into()),
 18=>app.settings.utility_profiles.get(&app.profile_key()).and_then(|b|b.timers.first()).map(|t|{let s=t.left(crate::utility_book::now());format!("{}:{:02}",s/60,s%60)}).unwrap_or("--".into()),
 19=>app.recorder.elapsed().map(|s|format!("REC {:.0}s",s)).unwrap_or_else(||if app.replay.is_buffering(){"BUFFER".into()}else{"OFF".into()}),
 20=>app.settings.utility_profiles.get(&app.profile_key()).and_then(|b|b.quests.iter().find(|q|!q.done(crate::utility_book::now()))).map(|q|q.name.chars().take(20).collect()).unwrap_or("--".into()),
 21=>app.settings.utility_profiles.get(&app.profile_key()).and_then(|b|b.timers.first()).map(|t|t.name.chars().take(20).collect()).unwrap_or("--".into()),
 _=>"--".into()
 }};
 let width=ui.available_width();let (r,_)=ui.allocate_exact_size(egui::vec2(width,20.*z),egui::Sense::hover());
 ui.painter().text(r.left_center(),egui::Align2::LEFT_CENTER,"HORIZON / LIVE",FontId::proportional(11.*z),WHITE);
 if app.settings.hud_show_fg{let v=current_fg_readout(app,true);ui.painter().text(r.right_center(),egui::Align2::RIGHT_CENTER,format!("{} > {} | {}",v.game,v.output,v.state),FontId::proportional(9.*z),V25_MINT);}
 if let Some(s)=spectrum{
  let dx=width/12.;for(i,b)in s.bands.iter().enumerate(){let v=b.map(|n|((n+72.)/72.).clamp(0.,1.)).unwrap_or(0.);ui.painter().rect_filled(egui::Rect::from_min_size(r.left_bottom()+egui::vec2(i as f32*dx,0.),egui::vec2((dx-1.).max(1.),2.*z)),0.,RED.gamma_multiply(0.15+0.85*v));}
 }else{ui.painter().rect_filled(egui::Rect::from_min_size(r.left_bottom(),egui::vec2(width*(0.15+peak*0.85),2.*z)),1.,RED);}
 let cell=(width-5.*z)/2.;
 for chunk in h.widgets.chunks(2){ui.horizontal(|ui|{
  ui.spacing_mut().item_spacing.x=5.*z;
  for w in chunk{
   let(rect,_)=ui.allocate_exact_size(egui::vec2(cell,31.*z),egui::Sense::hover());
   ui.painter().rect_filled(rect,4.,alpha(CARD,w.opacity));
   ui.painter().text(rect.left_top()+egui::vec2(5.*z,2.*z),egui::Align2::LEFT_TOP,crate::hud_layout::NAMES[w.metric.min((crate::hud_layout::NAMES.len()-1) as u8) as usize],FontId::proportional(9.*z*w.scale),alpha(MUTED,w.opacity));
   ui.painter().text(rect.left_bottom()+egui::vec2(5.*z,-2.*z),egui::Align2::LEFT_BOTTOM,value(w.metric),if w.mono{FontId::monospace(12.*z*w.scale)}else{FontId::proportional(12.*z*w.scale)},alpha(WHITE,w.opacity));
  }
 });}
 if h.graph{
  let(r,_)=ui.allocate_exact_size(egui::vec2(width,29.*z),egui::Sense::hover());ui.painter().rect_filled(r,4.,BG);
  let n=app.hud_meter().frames.len().min(60);
  if n>1&&app.hud_snapshot().is_some(){let points=app.hud_meter().frames.iter().rev().take(n).rev().enumerate().map(|(i,f)|egui::pos2(r.left()+r.width()*i as f32/(n-1) as f32,r.bottom()-r.height()*(f.ms/50.).clamp(0.,1.))).collect();ui.painter().add(egui::Shape::line(points,Stroke::new(1.0_f32,V25_MINT)));}
  else{ui.painter().text(r.center(),egui::Align2::CENTER_CENTER,"No fresh ETW frames",FontId::proportional(9.*z),MUTED);}
 }
}
fn accuracy_panel(app:&mut BoostApp,ui:&mut Ui){
 reaction_drill(ui);
 frame(PANEL).show(ui,|ui|{
  heading(ui,"ACCURACY PRACTICE / LOCAL TARGETS");
  caption(ui,"Manual clicks on local targets. No interaction with a game, no aim assist or automated clicks.");
  let now=std::time::Instant::now();let t=&mut app.accuracy_trainer;t.tick(now);
  if t.duration_secs==0{t.duration_secs=30;t.radius=17.;}
  ui.add_enabled_ui(!t.active,|ui|{ui.add(egui::Slider::new(&mut t.duration_secs,10..=60).text("Seconds"));ui.add(egui::Slider::new(&mut t.radius,8.0..=24.0).text("Target radius"));});
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Start practice"){t.start(now);}
   if outline_button(ui,"Stop"){t.stop();}
   ui.label(format!("{} s / {} hits / {} misses",t.remaining(now),t.hits,t.misses));
   if let Some(a)=t.accuracy(){ui.label(format!("{a:.1}% accuracy"));}
  });
  let(r,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),if app.accuracy_window{ui.available_height().clamp(160.,360.)-40.}else{190.}),egui::Sense::click());
  ui.painter().rect_filled(r,8.,BG);
  if t.active{
   let pos=crate::accuracy::target_position(t.target,[r.width(),r.height()],t.radius);let q=r.min+egui::vec2(pos[0],pos[1]);
   ui.painter().circle_filled(q,t.radius*0.75,RED);ui.painter().circle_stroke(q,t.radius,Stroke::new(1.0_f32,WHITE));
   if response.clicked()&&let Some(p)=response.interact_pointer_pos(){t.click(now,p.distance(q)<=t.radius);}
   ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));
  }else{ui.painter().text(r.center(),egui::Align2::CENTER_CENTER,"Start practice",FontId::proportional(14.),MUTED);}
  if let Some(m)=t.median_ms(){caption(ui,&format!("Median successful target interval: {m:.0} ms"));}
  if let Some(v)=t.p90_ms(){caption(ui,&format!("P90 successful interval: {v} ms"));}
  if let Some(v)=t.hits_per_minute(now){caption(ui,&format!("Hits/min: {v:.1}"));}
  if !t.times.is_empty(){caption(ui,&format!("Mean interval between successful targets: {:.0} ms",t.times.iter().map(|v|*v as f32).sum::<f32>()/t.times.len() as f32));}
 });
}
fn session_timeline(app:&BoostApp,ui:&mut Ui){
 let end=app.report.rows.back().map(|v|v.elapsed_ms).unwrap_or(0).max(app.report.markers.last().map(|v|v.elapsed_ms).unwrap_or(0)).max(app.report.spikes.back().map(|v|v.observed_ms).unwrap_or(0));
 if end==0{return;}
 let(r,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),52.),egui::Sense::hover());ui.painter().rect_filled(r,5.,BG);
 let x=|ms:u64|r.left()+r.width()*(ms as f64/end as f64).clamp(0.,1.) as f32;
 for s in &app.report.spikes{let q=x(s.observed_ms);let h=(s.frame_ms/200.).clamp(0.1,1.)*r.height();ui.painter().line_segment([egui::pos2(q,r.bottom()),egui::pos2(q,r.bottom()-h)],Stroke::new(1.0_f32,RED));}
 for s in &app.report.markers{let q=x(s.elapsed_ms);ui.painter().line_segment([egui::pos2(q,r.top()),egui::pos2(q,r.bottom())],Stroke::new(1.0_f32,V25_MINT));}
 caption(ui,&format!("0–{:.1}s / red: stutters / green: manual events",end as f64/1000.));
}
