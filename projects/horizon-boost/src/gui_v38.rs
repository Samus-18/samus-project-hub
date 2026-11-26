fn portable_profile_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"PORTABLE GAME PROFILE");
  caption(ui,"Includes native stack, look, scale, preview settings, lighting, reticle design, offline macro, reminders, checklist and notes. No executable path, PID or running tools. Notes are included; share only if desired.");
  if outline_button(ui,"Export current profile JSON"){
   match crate::profile_portable::Portable::capture(&app.settings,&app.profile_key(),app.target_fps).encode(){Ok(v)=>app.portable_edit=v,Err(e)=>app.toast(e)}
  }
  ui.add(egui::TextEdit::multiline(&mut app.portable_edit).desired_rows(5).desired_width(ui.available_width()).char_limit(65536));
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Copy JSON"){ui.ctx().copy_text(app.portable_edit.clone());}
   if outline_button(ui,"Validate and apply to selected profile"){
    match crate::profile_portable::Portable::decode(&app.portable_edit).and_then(|p|app.import_portable_profile(p)){
     Ok(())=>app.toast("Profile imported. Live tools remain stopped."),Err(e)=>app.toast(e)
    }
   }
   let can_undo=app.portable_undo.as_ref().is_some_and(|(key,_)|*key==app.profile_key());
   if ui.add_enabled(can_undo,egui::Button::new("Swap with prior profile")).clicked(){
    if let Some((_,p))=app.portable_undo.clone(){
     match app.import_portable_profile(p){Ok(())=>app.toast("Previous profile restored"),Err(e)=>app.toast(e)}
    }
   }
  });
  caption(ui,"Import validates and verifies a local backup of the previous profile before state changes. Backups persist across restarts; one-step swap also works during this session.");
  egui::CollapsingHeader::new("Local profile backups").show(ui,|ui|{
   ui.horizontal_wrapped(|ui|{
    if outline_button(ui,"Save backup file")&&!app.is_smoke{
     let profile=crate::profile_portable::Portable::capture(&app.settings,&app.profile_key(),app.target_fps);
     match crate::profile_backup::save(&profile){
      Ok(name)=>{app.toast(format!("Saved {name}"));app.profile_backups=crate::profile_backup::list().unwrap_or_default();},
      Err(e)=>app.toast(e)
     }
    }
    if outline_button(ui,"Refresh backups"){match crate::profile_backup::list(){Ok(v)=>app.profile_backups=v,Err(e)=>app.toast(e)}}
   });
   egui::ScrollArea::vertical().id_salt("profile-backup-list").max_height(140.).show(ui,|ui|{
    for name in app.profile_backups.clone(){if ui.button(&name).clicked(){
     match crate::profile_backup::load(&name).and_then(|v|v.encode()){
      Ok(raw)=>{app.portable_edit=raw;app.toast("Backup loaded into draft; validate and apply when ready");},Err(e)=>app.toast(e)
     }
    }}
   });
   caption(ui,"Backups remain local in HorizonBoostRust/profile-backups; no automatic deletions or uploads.");
  });
  ui.add_space(8.);
  if ui.checkbox(&mut app.settings.quiet_ui,"Lower idle UI refresh rate").changed(){app.save();}
  caption(ui,"When the main window lacks focus, use the existing 90 ms global shortcut poll cadence rather than fast animation redraws.");
 });
}
fn stick_view(ui:&mut Ui,raw:[f32;2],zone:f32){
 let (r,_)=ui.allocate_exact_size(egui::vec2(98.,98.),egui::Sense::hover());
 let p=ui.painter();let radius=43.;let center=r.center();
 p.circle_stroke(center,radius,Stroke::new(1.0_f32,BORDER));
 p.circle_stroke(center,radius*zone,Stroke::new(1.0_f32,MUTED));
 p.line_segment([center-egui::vec2(radius,0.),center+egui::vec2(radius,0.)],Stroke::new(0.6_f32,BORDER));
 p.line_segment([center-egui::vec2(0.,radius),center+egui::vec2(0.,radius)],Stroke::new(0.6_f32,BORDER));
 let mag=crate::controller_check::magnitude(raw).max(1.);let raw_pos=center+egui::vec2(raw[0],-raw[1])*(radius/mag);
 let v=crate::controller_check::deadzone(raw,zone);
 let processed=center+egui::vec2(v[0],-v[1])*radius;
 p.circle_filled(raw_pos,4.,RED);p.circle_stroke(processed,6.,Stroke::new(1.5_f32,V25_MINT));
}
fn controller_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"CONTROLLER LAB / XINPUT");
  caption(ui,"Optional read-only polling of up to four XInput controllers. No game remapping, injected input or vibration. Non-XInput devices may not appear.");
  if ui.checkbox(&mut app.controller_enabled,"Read connected controllers").changed(){
   app.controller_last=None;if !app.controller_enabled{app.controller_drift.reset();app.controller_readings=None;}
  }
  if ui.add(egui::Slider::new(&mut app.settings.controller_deadzone,0.0..=0.95).text("Preview deadzone")).changed(){app.save();}
  caption(ui,"Red = raw / green = simulated radial deadzone. This setting does not alter a game's actual input.");
  let readings=app.controller_readings.clone();
  match readings{
   Some(Ok(rows)) if rows.is_empty()=>caption(ui,"No XInput controller currently reported by Windows."),
   Some(Ok(rows))=>for row in rows{
    frame(CARD).show(ui,|ui|{
     ui.label(RichText::new(format!("Controller {} / packet {}",row.slot+1,row.packet)).strong());
     caption(ui,row.battery.unwrap_or("Battery unavailable"));
     ui.horizontal_wrapped(|ui|{stick_view(ui,row.left,app.settings.controller_deadzone);stick_view(ui,row.right,app.settings.controller_deadzone);});
     caption(ui,&format!("Left {:.3}, {:.3} / Right {:.3}, {:.3}",row.left[0],row.left[1],row.right[0],row.right[1]));
     ui.add(egui::ProgressBar::new(row.triggers[0]).text("LT"));ui.add(egui::ProgressBar::new(row.triggers[1]).text("RT"));
     ui.horizontal_wrapped(|ui|{for (mask,label) in [(0x1000,"A"),(0x2000,"B"),(0x4000,"X"),(0x8000,"Y"),(0x0100,"LB"),(0x0200,"RB"),(0x0001,"Up"),(0x0002,"Down"),(0x0004,"Left"),(0x0008,"Right"),(0x0010,"Start"),(0x0020,"Back"),(0x0040,"LS"),(0x0080,"RS")]{
      ui.colored_label(if row.buttons&mask!=0{V25_MINT}else{MUTED},label);
     }});
     if outline_button(ui,"Measure released-stick drift (100 samples)"){app.controller_drift.start(row.slot);}
    });
   },
   Some(Err(e))=>caption(ui,&e),None=>caption(ui,"Polling disabled or waiting for first reading.")
  }
  if let Some(slot)=app.controller_drift.source_slot(){caption(ui,&format!("Drift source: controller {}",slot+1));}
  if app.controller_drift.count()>0{
   if let Some(avg)=app.controller_drift.mean(){
    caption(ui,&format!("Drift samples {}/100 / mean L {:.3} R {:.3} / max L {:.3} R {:.3}",app.controller_drift.count(),avg[0],avg[1],app.controller_drift.maximum[0],app.controller_drift.maximum[1]));
   }
  }
  if let Some(p)=app.controller_drift.p95(){caption(ui,&format!("P95 released-stick drift L {:.3} / R {:.3}",p[0],p[1]));}
  if let Some(v)=app.controller_drift.recommended(){if outline_button(ui,&format!("Use {:.2} preview deadzone",v)){app.settings.controller_deadzone=v;app.save();}}
  if app.controller_drift.running()&&outline_button(ui,"Stop drift measurement"){app.controller_drift.stop();}
  if app.controller_drift.source_slot().is_some()&&outline_button(ui,"Clear drift result"){app.controller_drift.reset();}
  caption(ui,"Leave sticks released during drift measurement; movement changes the result. Battery values are discrete Windows statuses, not percentages.");
 });
}
fn session_report_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"SESSION LAB / ACTUAL ETW");
  caption(ui,"Manual capture of measured 1-second telemetry snapshots. Up to 3600 samples and 256 manual markers. Stops when selected process changes. Missing readings are omitted.");
  let pid=app.telem.active_game.as_ref().filter(|g|Some(g.pid)==app.telem.selected_pid).map(|g|g.pid);
  ui.horizontal_wrapped(|ui|{
   if app.report.active(){if red_button(ui,"Stop session"){app.report.stop();}}
   else if ui.add_enabled(pid.is_some()&&app.fps.snapshot().is_some(),egui::Button::new("Start new measured session")).clicked(){
    if let Some(pid)=pid{app.report.start(pid,std::time::Instant::now());}
   }
   if outline_button(ui,"Pin game"){app.page=Page::Library;}
  });
  caption(ui,&format!("{} samples / {} markers / {} recorded stutters",app.report.rows.len(),app.report.markers.len(),app.report.spikes.len()));
  caption(ui,"Ctrl+Shift+N adds a manual marker while an explicit measured session is running.");
  session_timeline(app,ui);
  let now=std::time::Instant::now();
  caption(ui,&format!("Session elapsed {:.1} s",app.report.elapsed(now) as f64/1000.));
  let key=egui::Id::new("session-comparison-v54");
  let mut saved=ui.data_mut(|d|d.get_temp::<Vec<crate::session_report::Summary>>(key)).unwrap_or_default();
  if let Some(s)=app.report.summary(now){
   caption(ui,&format!("{} samples / mean {:.1} FPS / median {:.2} ms / P95 {:.2} ms of 1-second averages",s.samples,s.mean_fps,s.median_ms,s.p95_sample_ms));
   if !app.report.active()&&outline_button(ui,"Keep completed session for comparison"){
    if saved.last()!=Some(&s){if saved.len()==8{saved.remove(0);}saved.push(s);}
   }
  }
  if !saved.is_empty(){
   egui::CollapsingHeader::new("Session comparison (local memory)").show(ui,|ui|{
    caption(ui,"P95 of sampled 1-second averages, NOT individual frame P95. Compare similar game workloads manually.");
    for (i,s) in saved.iter().enumerate(){caption(ui,&format!("#{} | {:.1}s | {} samples | {:.1} mean FPS | {:.2} ms P95",i+1,s.duration_ms as f64/1000.,s.samples,s.mean_fps,s.p95_sample_ms));}
    if outline_button(ui,"Copy comparison JSON"){if let Ok(s)=serde_json::to_string_pretty(&saved){ui.ctx().copy_text(s);}}
    if outline_button(ui,"Clear comparison"){saved.clear();}
   });
  }
  ui.data_mut(|d|d.insert_temp(key,saved));
  egui::CollapsingHeader::new("Actual retained frame distribution").show(ui,|ui|{
   caption(ui,&format!("{} retained / {} total valid ETW intervals; up to 32768 latest intervals, not full-session if truncated.",app.report.raw_count(),app.report.raw_total()));
   let id=egui::Id::new("session-raw-summary");let mut summary=ui.data_mut(|d|d.get_temp::<crate::session_report::RawSummary>(id));
   if app.report.raw_total()==0{summary=None;ui.data_mut(|d|d.remove::<crate::session_report::RawSummary>(id));}
   if outline_button(ui,"Calculate raw-frame metrics"){
    summary=app.report.raw_summary();if let Some(s)=summary.clone(){ui.data_mut(|d|d.insert_temp(id,s));}
   }
   if let Some(s)=summary{
    caption(ui,&format!("{} retained at calculation / {:.1} FPS from mean interval / median {:.2} / P95 {:.2} / P99 {:.2} ms",s.retained,s.mean_fps,s.median_ms,s.p95_ms,s.p99_ms));
    caption(ui,&format!("1% low {:?} / 0.1% low {:?} FPS / adjacent jitter {:.3} ms",s.low_1.map(|x|x.round()),s.low_01.map(|x|x.round()),s.adjacent_jitter_ms));
   }
   caption(ui,"Computed only on demand. Missing 1%/0.1% lows stay unavailable until >=100/1000 intervals. JSON exports current retained raw summary, not raw audio or game data.");
  });
  if let Some(low)=app.fps.low_01_fps(){caption(ui,&format!("0.1% low: {low:.1} FPS (at least 1000 fresh frame intervals)"));}
  if let Some(rates)=app.output_rates.snapshot(std::time::Instant::now()){
   caption(ui,&format!("Own presents/s: real {:.1} / generated {:.1} / total {:.1}",rates.real,rates.generated,rates.total));
  }
  egui::CollapsingHeader::new("Stutter events").show(ui,|ui|{
   if ui.add(egui::Slider::new(&mut app.settings.stutter_threshold_ms,20.0..=200.0).suffix(" ms").text("Threshold")).changed(){app.save();}
   caption(ui,"Actual ETW intervals exceeding threshold; times are observed poll times, not precise frame timestamps.");
   for s in app.report.spikes.iter().rev().take(12){caption(ui,&format!("+{:.1}s / {:.1} ms",s.observed_ms as f64/1000.,s.frame_ms));}
  });
  ui.horizontal_wrapped(|ui|{
   ui.add(egui::TextEdit::singleline(&mut app.report_marker).desired_width(150.).char_limit(48).hint_text("Manual event"));
   if ui.add_enabled(app.report.active(),egui::Button::new("Add marker")).clicked(){
    let label=if app.report_marker.trim().is_empty(){"Manual marker"}else{app.report_marker.as_str()};
    app.report.mark(label,std::time::Instant::now());app.report_marker.clear();
   }
  });
  if let Some(last)=app.report.rows.back(){
   ui.label(format!("{:.1} FPS / {:.2} ms",last.fps,last.frame_ms));
   caption(ui,&format!("Own output counts: real {:?} / generated {:?}",last.output_real,last.output_generated));
  }
  let points=app.report.rows.iter().rev().take(120).rev().map(|s|s.frame_ms).collect::<Vec<_>>();
  if points.len()>1{
   let (r,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),72.),egui::Sense::hover());
   ui.painter().rect_filled(r,6.,BG);
   let max=points.iter().copied().fold(33.3f32,f32::max);
   let line=points.iter().enumerate().map(|(i,v)|egui::pos2(r.left()+r.width()*i as f32/(points.len()-1) as f32,r.bottom()-r.height()*(*v/max).clamp(0.,1.))).collect();
   ui.painter().add(egui::Shape::line(line,Stroke::new(1.5_f32,RED)));
   caption(ui,&format!("Latest {} sample averages / vertical max {:.1} ms",points.len(),max));
  }
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Copy CSV"){ui.ctx().copy_text(app.report.csv());}
   if outline_button(ui,"Copy JSON"){match app.report.json(){Ok(s)=>ui.ctx().copy_text(s),Err(e)=>app.toast(e)}}
   if outline_button(ui,"Save report files"){
    if app.is_smoke{return;}
    let result=(||->Result<String,String>{
     let dir=crate::settings::folder().join("reports");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
     let time=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis();
     let stem=format!("session-{time}-{}",std::process::id());
     crate::settings::atomic_json_replace(&dir.join(format!("{stem}.json")),app.report.json()?.as_bytes())?;
     crate::settings::atomic_json_replace(&dir.join(format!("{stem}.csv")),app.report.csv().as_bytes())?;
     Ok(format!("Saved {stem} CSV/JSON"))
    })();app.toast(result.unwrap_or_else(|e|e));
   }
  });
  caption(ui,"Exports omit game names, executable paths and PIDs; marker labels are included. GPU and latency columns are blank when unavailable. Own present counters are not engine FPS.");
 });
}
fn command_palette(app:&mut BoostApp,ctx:&egui::Context){
 use crate::command_search::{Hit,ROUTES};
 if app.is_smoke{return;}
 let active=egui::Id::new("command-search-active");let row=egui::Id::new("command-search-row");
 if ctx.input(|i|i.modifiers.ctrl&&i.key_pressed(egui::Key::K)){app.palette_open=!app.palette_open;}
 if !app.palette_open||ctx.input(|i|i.key_pressed(egui::Key::Escape)){
  app.palette_open=false;ctx.data_mut(|d|d.insert_temp(active,false));return;
 }
 let fresh=!ctx.data(|d|d.get_temp::<bool>(active).unwrap_or(false));
 let mut selected=ctx.data(|d|d.get_temp::<usize>(row).unwrap_or(0));
 let mut open=true;let mut choice=None;
 egui::Window::new(if app.settings.ui_language==1{"Najít nástroj / Ctrl+K"}else{"Find a tool / Ctrl+K"}).open(&mut open).collapsible(false).resizable(true).default_width(450.).show(ctx,|ui|{
  let hint=local(ui,"Search tools, looks and games","Hledat nástroje, filtry a hry");
  let input=ui.add(egui::TextEdit::singleline(&mut app.palette_query).hint_text(hint).desired_width(ui.available_width()).char_limit(64));
  if fresh{input.request_focus();selected=0;}
  if input.changed(){selected=0;}
  let hits=crate::command_search::search(&app.palette_query,&app.settings.curated_games);
  selected=selected.min(hits.len().saturating_sub(1));
  let down=ctx.input_mut(|i|i.consume_key(egui::Modifiers::NONE,egui::Key::ArrowDown));
  let up=ctx.input_mut(|i|i.consume_key(egui::Modifiers::NONE,egui::Key::ArrowUp));
  if down{selected=(selected+1).min(hits.len().saturating_sub(1));}if up{selected=selected.saturating_sub(1);}
  if ctx.input_mut(|i|i.consume_key(egui::Modifiers::NONE,egui::Key::Enter)){choice=hits.get(selected).copied();}
  if hits.is_empty(){caption(ui,local(ui,"No matching tools","Žádné výsledky"));}
  egui::ScrollArea::vertical().max_height(340.).show(ui,|ui|{
   for (i,hit) in hits.iter().enumerate(){
    let label=match *hit{
     Hit::Route(n)=>local(ui,ROUTES[n].en,ROUTES[n].cz).to_owned(),
     Hit::Look(n)=>format!("{} · {}",local(ui,"Look","Filtr"),crate::native_presets::PRESETS[n].label),
     Hit::Game(n)=>format!("{} · {}",local(ui,"Game","Hra"),app.settings.curated_games[n].title)};
    let res=ui.selectable_label(i==selected,label);
    if i==selected&&(up||down){res.scroll_to_me(Some(egui::Align::Center));}
    if res.clicked(){choice=Some(*hit);}
   }
  });
  caption(ui,local(ui,"↑ ↓ Select   Enter Open   Esc Close","↑ ↓ Vybrat   Enter Otevřít   Esc Zavřít"));
 });
 if let Some(hit)=choice{
  match hit{
   Hit::Route(n)=>{let r=&ROUTES[n];app.page=r.page;match r.page{
    Page::Extras=>{app.extras_view=r.sub;app.settings.ui_advanced=true;},
    Page::Performance=>{app.performance_view=r.sub;if r.sub>0{app.settings.ui_advanced=true;}},
    Page::Filters=>app.filter_view=r.sub,_=>{}}},
   Hit::Game(n)=>{app.library_filter=app.settings.curated_games[n].title.clone();app.page=Page::Library;app.settings.ui_advanced=false;},
   Hit::Look(n)=>{app.filter_search=crate::native_presets::PRESETS[n].label.to_owned();app.page=Page::Filters;app.filter_view=0;app.settings.ui_advanced=false;ctx.data_mut(|d|d.insert_temp(egui::Id::new("oled-gallery-favorites"),false));}
  }
  app.palette_open=false;app.palette_query.clear();app.save();
 }
 app.palette_open&=open;
 ctx.data_mut(|d|{d.insert_temp(active,app.palette_open);d.insert_temp(row,selected);});
}
fn native_favorite_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"FAVORITE NATIVE LOOKS");
  let active=app.settings.native_preset;
  let marked=app.settings.native_favorites.contains(&active);
  if outline_button(ui,if marked{"Remove active look from favorites"}else{"Add active look to favorites"}){
   if marked{app.settings.native_favorites.retain(|v|*v!=active);}else{app.settings.native_favorites.push(active);}app.save();
  }
  ui.horizontal_wrapped(|ui|{
   for id in app.settings.native_favorites.clone(){
    if let Some(p)=crate::native_presets::PRESETS.get(id){
     if outline_button(ui,p.label){app.select_native_preset(id);}
    }
   }
  });
 });ui.add_space(8.);
}
