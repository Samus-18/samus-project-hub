
#[derive(Clone,Default)]struct MapPreview{key:String,texture:Option<egui::TextureHandle>,label:String,error:String,pinned:bool,opacity:f32,epoch:u64,corner:u8,size:f32,pid:Option<u32>}
fn map_notes_panel(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();let id=egui::Id::new("hb-map-notes");let mut st=ui.data(|d|d.get_temp::<MapPreview>(id)).unwrap_or_default();
 if st.key!=key{st=MapPreview{key:key.clone(),..Default::default()};}
 let mut map=a.settings.map_notes.get(&key).cloned().unwrap_or_default();let mut changed=false;
 frame(PANEL).show(ui,|ui|{
 ui.set_min_width(ui.available_width());heading(ui,"MAP / ROUTE NOTES");caption(ui,"Import a local screenshot and click positions to add manual pins. No map data read from games.");
 let mut load=None;
 ui.horizontal_wrapped(|ui|{
 if ui.button("Choose local image").clicked()&&!a.is_smoke{load=crate::filedialog::select_image();}
 if ui.add_enabled(map.image.is_some(),egui::Button::new("Load saved image")).clicked()&&!a.is_smoke{load=map.image.clone();}
 });
 if let Some(p)=load{match crate::suite_finish::load_reference(&p){
 Ok(im)=>{if map.image.as_ref()!=Some(&p){map.pins.clear();}map.image=Some(p);st.texture=Some(ui.ctx().load_texture("map-reference",egui::ColorImage::from_rgba_unmultiplied([im.width() as usize,im.height() as usize],im.as_raw()),egui::TextureOptions::LINEAR));changed=true;st.error.clear();},Err(e)=>st.error=e}}
 if let Some(p)=&map.image{caption(ui,&p.display().to_string());}
 if !st.error.is_empty(){caption(ui,&st.error);}
 ui.add(egui::TextEdit::singleline(&mut st.label).hint_text("Pin label").char_limit(100));
  if st.texture.is_some(){ui.horizontal_wrapped(|ui|{
 if ui.button(if st.pinned{"Close passive reference"}else{"Pin passive reference"}).clicked()&&!a.is_smoke{st.pinned=!st.pinned;st.epoch=a.reference_epoch;st.pid=a.telem.active_game.as_ref().map(|g|g.pid);if st.opacity==0.{st.opacity=0.9;}}
 ui.add(egui::Slider::new(&mut st.opacity,0.4..=1.).text("Image opacity"));
 if st.size<200.{st.size=320.;}
 ui.add(egui::Slider::new(&mut st.size,200.0..=600.).text("Reference width"));
 for(i,n)in ["Top left","Top right","Bottom left","Bottom right"].iter().enumerate(){ui.selectable_value(&mut st.corner,i as u8,*n);}
 });caption(ui,"Reference is a click-through always-on-top window. Close from here or emergency stop. Exclusive fullscreen compatibility is not guaranteed.");}
 if let Some(t)=&st.texture{
 let scale=(ui.available_width()/t.size()[0] as f32).min(480./t.size()[1] as f32).min(1.);
 let res=ui.add(egui::Image::new(t).fit_to_exact_size(t.size_vec2()*scale).sense(Sense::click()));let r=res.rect;
 if res.clicked()&&let Some(q)=res.interact_pointer_pos(){changed|=map.add((q.x-r.left())/r.width(),(q.y-r.top())/r.height(),if st.label.is_empty(){"Pin"}else{&st.label});}
 for(i,p)in map.pins.iter().enumerate(){let q=pos2(r.left()+r.width()*p.x,r.top()+r.height()*p.y);let painter=ui.painter().with_clip_rect(r);painter.circle_filled(q,8.,if p.done{V25_MINT}else{RED});painter.text(q,egui::Align2::CENTER_CENTER,format!("{}",i+1),FontId::proportional(10.),WHITE);}
 }
 let mut remove=None;for(i,p)in map.pins.iter_mut().enumerate(){ui.push_id(i,|ui|ui.horizontal_wrapped(|ui|{ui.label(format!("{}",i+1));changed|=ui.checkbox(&mut p.done,"").changed();changed|=ui.add(egui::TextEdit::singleline(&mut p.label).char_limit(100)).changed();if ui.small_button("Remove").clicked(){remove=Some(i);}}));}
 if let Some(i)=remove{map.pins.remove(i);changed=true;}
 caption(ui,"Image remains at original location; pin coordinates and labels are saved per game. Replacing image clears its pins.");
 });
 if changed&&!a.is_smoke{map.sanitize();a.settings.map_notes.insert(key,map);a.save();}ui.data_mut(|d|d.insert_temp(id,st));
}
fn session_goal_panel(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();let mut goal=a.settings.session_goals.get(&key).copied().unwrap_or(60);
 frame(PANEL).show(ui,|ui|{
 heading(ui,"SESSION GOAL");caption(ui,"Target applies to the tracked game session, not a Windows restriction.");
 if ui.add(egui::Slider::new(&mut goal,5..=720).suffix(" min").text("Target session length")).changed(){a.settings.session_goals.insert(key.clone(),goal);a.save();}
 let elapsed=a.sessions.current.as_ref().filter(|x|x.key==key).map(|x|x.began.elapsed().as_secs()).unwrap_or(0);
 ui.add(egui::ProgressBar::new((elapsed as f32/(goal*60)as f32).min(1.)).text(format!("{} / {goal} min",elapsed/60)));
 if elapsed>=u64::from(goal)*60{ui.colored_label(V25_MINT,"Session target reached");}
 ui.ctx().request_repaint_after(std::time::Duration::from_secs(10));
 });
 let mut book=a.settings.journal_profiles.get(&key).cloned().unwrap_or_default();
 if journal_entries(ui,"Personal session goals",&mut book.routes,true)&&!a.is_smoke{book.sanitize();a.settings.journal_profiles.insert(key,book);a.save();}
 break_and_alert_panel(a,ui);
}
fn shortcut_reference(a:&mut BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new("Shortcut reference / active bindings").show(ui,|ui|{
 for (id,label) in crate::shortcut_registry::ACTIONS{ui.horizontal_wrapped(|ui|{ui.label(label);ui.label(RichText::new(a.settings.global_shortcuts.label(id)).color(MUTED));});}
 caption(ui,"Ctrl+Shift+F12: emergency stop. F12 cancels macros. Duplicate configured chords are rejected; external application conflicts cannot be detected reliably.");
 if ui.button("Edit shortcuts").clicked(){a.page=Page::Settings;a.settings.ui_advanced=true;}
 if ui.button("Magnifier / accessible view").clicked(){a.page=Page::Extras;a.extras_view=6;}
 if ui.button("Cursor / crosshair visibility").clicked(){a.page=Page::Extras;a.extras_view=0;}
 });
}
fn suite_safety_panel(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
 heading(ui,"PRIVACY / LOCAL PERMISSIONS");
 caption(ui,"Capture and microphone access are user-initiated. No continuous voice listener, automatic screenshot reading or upload.");
 ui.label(format!("Next recording: system audio {} / microphone {}",opt_on(a.audio_next.system),opt_on(a.audio_next.microphone)));
 if ui.button("Clear next recording audio consent").clicked(){a.audio_next=Default::default();}
 ui.horizontal_wrapped(|ui|{if ui.button("Capture controls").clicked(){a.page=Page::Capture;ui.data_mut(|d|d.insert_temp(egui::Id::new("capture-workspace-v114"),4usize));}if ui.button("Emergency stop").clicked()&&!a.is_smoke{a.emergency_stop();}});
 if ui.checkbox(&mut a.settings.quiet_clip_toasts,"Quiet successful clip notices (status bar stays visible)").changed(){a.save();}
 if ui.checkbox(&mut a.settings.pressure_quiet,"Pause decorative animation on measured own VRAM pressure >=90%").changed(){a.save();}
 caption(ui,"Unknown GPU memory pressure does not trigger automatic changes. No Windows settings changed.");
 });
 assist_panel(a,ui);
 shortcut_reference(a,ui);
 egui::CollapsingHeader::new("Capabilities / internal registry").show(ui,|ui|{
 caption(ui,&format!("Registry schema {}. Native built-in components only; no external executable plugins.",crate::suite_finish::SCHEMA));
 let mut changed=false;
 changed|=ui.checkbox(&mut a.settings.optional_modules.ocr,"Allow manual OCR module").changed();
 changed|=ui.checkbox(&mut a.settings.optional_modules.voice,"Allow one-shot voice module").changed();
 changed|=ui.checkbox(&mut a.settings.optional_modules.reference,"Allow passive reference module").changed();
 changed|=ui.checkbox(&mut a.settings.optional_modules.highlights,"Allow manual highlight module").changed();
 if changed{a.save();}
 caption(ui,"Disabling prevents new actions and cancels pending highlight/reference; existing bounded OCR/voice workers end at timeout.");
 if let Some(g)=a.telem.active_game.as_ref(){let v=crate::anticheat_guard::inspect(&g.path,&g.title);ui.label(format!("Selected game: {}",g.title));caption(ui,v.reason);}
 else{caption(ui,"No selected running game; per-game restriction unknown.");}
 for (name,state,detail) in crate::suite_finish::CAPABILITIES{ui.horizontal_wrapped(|ui|{ui.label(RichText::new(name).strong());ui.label(state);});caption(ui,detail);ui.separator();}
 });
}

fn hud_profile_panel(a:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
 heading(ui,"PER-GAME HUD PROFILE");let key=a.profile_key();
 if ui.checkbox(&mut a.settings.hud_layout.quest_list,"Show up to three recurring quests in passive HUD").changed(){a.save();}
 ui.horizontal_wrapped(|ui|{
 if ui.button("Save layout for this game").clicked(){let mut h=a.settings.hud_layout.clone();h.audio=false;h.sanitize();a.settings.hud_profiles.insert(key.clone(),h);a.save();}
 if ui.add_enabled(a.settings.hud_profiles.contains_key(&key),egui::Button::new("Load saved layout")).clicked(){if let Some(h)=a.settings.hud_profiles.get(&key).cloned(){a.settings.hud_layout=h;a.save();}}
 });
 caption(ui,"Explicit save/load preserves custom positions and widgets. Does not automatically enable audio or open a HUD.");
 if ui.button("Edit widget grid").clicked(){a.settings.ui_advanced=true;}
 if a.report.active()&&ui.button("Mark session moment").clicked()&&!a.is_smoke{a.report.mark("Manual HUD moment",std::time::Instant::now());a.toast("Session marker added");}
 });
}

#[derive(Clone,Default)]struct HighlightState{pending:Option<(String,Option<u32>,std::time::Instant,u64)>,post:u32,hotkey:bool,held:bool,clip_seconds:u32}
fn highlight_panel(a:&mut BoostApp,ui:&mut Ui){
 if !a.settings.optional_modules.highlights{caption(ui,"Highlight module disabled in Settings");return;}
 let id=egui::Id::new("manual-highlight");let mut s=ui.data(|d|d.get_temp::<HighlightState>(id)).unwrap_or_default();
 egui::CollapsingHeader::new("Manual highlight / post-roll").show(ui,|ui|{
 if s.clip_seconds<5{s.clip_seconds=a.replay.seconds();}
 s.clip_seconds=s.clip_seconds.clamp(5,a.replay.seconds());
 ui.add(egui::Slider::new(&mut s.clip_seconds,5..=a.replay.seconds()).step_by(5.).suffix(" s").text("Clip length"));
 ui.add(egui::Slider::new(&mut s.post,0..=15.min(s.clip_seconds.saturating_sub(5))).suffix(" s").text("Post-roll delay"));
 let ready=a.replay.is_buffering()&&a.replay.ready()&&s.post<s.clip_seconds&&s.clip_seconds<=a.replay.seconds()&&!a.replay.saving();
 if ui.add_enabled(ready&&s.pending.is_none(),egui::Button::new("Mark highlight and save after post-roll")).clicked()&&!a.is_smoke{s.pending=Some((a.profile_key(),a.telem.active_game.as_ref().map(|g|g.pid),std::time::Instant::now()+std::time::Duration::from_secs(s.post as u64),a.reference_epoch));}
 if ui.checkbox(&mut s.hotkey,"Use MARK shortcut for delayed highlights in this session").changed(){s.held=true;}
 if s.pending.is_some()&&ui.button("Cancel pending highlight").clicked(){s.pending=None;}
 caption(ui,"Manual marker: selected completed-segment clip length after delay. Pre/post timing is approximate (5s segment boundaries). No automatic game-event detector.");
 });
 ui.data_mut(|d|d.insert_temp(id,s));
}
fn highlight_tick(a:&mut BoostApp,ctx:&egui::Context){
 let id=egui::Id::new("manual-highlight");let mut s=ctx.data(|d|d.get_temp::<HighlightState>(id)).unwrap_or_default();
 if !a.settings.optional_modules.highlights{s.pending=None;s.hotkey=false;ctx.data_mut(|d|d.insert_temp(id,s));return;}
 let held=s.hotkey&&a.settings.global_shortcuts.down("MARK");
 if held&&!s.held&&s.pending.is_none()&&!a.is_smoke&&a.replay.is_buffering()&&a.replay.ready()&&!a.replay.saving()&&s.post<s.clip_seconds&&s.clip_seconds<=a.replay.seconds(){s.pending=Some((a.profile_key(),a.telem.active_game.as_ref().map(|g|g.pid),std::time::Instant::now()+std::time::Duration::from_secs(s.post as u64),a.reference_epoch));}
 let edge=held!=s.held;s.held=held;
 if let Some((key,pid,at,epoch))=&s.pending{
 if a.is_smoke||!a.replay.is_buffering()||*epoch!=a.reference_epoch||*key!=a.profile_key()||*pid!=a.telem.active_game.as_ref().map(|g|g.pid){s.pending=None;}
 else if std::time::Instant::now()>=*at{s.pending=None;if !a.replay.saving(){a.save_replay_window(s.clip_seconds);}}
 else{ctx.request_repaint_after(std::time::Duration::from_millis(200));}
 ctx.data_mut(|d|d.insert_temp(id,s));
 }else if edge{ctx.data_mut(|d|d.insert_temp(id,s));}
}

fn reference_viewport(a:&BoostApp,ctx:&egui::Context){
 let id=egui::Id::new("hb-map-notes");let mut s=ctx.data(|d|d.get_temp::<MapPreview>(id)).unwrap_or_default();
 if s.pinned&&(!a.settings.optional_modules.reference||s.key!=a.profile_key()||s.pid!=a.telem.active_game.as_ref().map(|g|g.pid)||a.is_smoke||s.epoch!=a.reference_epoch){s.pinned=false;ctx.data_mut(|d|d.insert_temp(id,s));return;}
 if !s.pinned{return;}
 if let Some(t)=&s.texture{
 let mut close=false;
 let size=egui::vec2(s.size.clamp(200.,600.),(s.size*0.70).clamp(140.,420.));
 let screen=ctx.input(|i|i.viewport().monitor_size).unwrap_or(egui::vec2(1280.,720.));
 let anchor=egui::pos2(if s.corner%2==1{(screen.x-size.x-12.).max(0.)}else{12.},if s.corner>=2{(screen.y-size.y-48.).max(0.)}else{12.});
 ctx.show_viewport_immediate(egui::ViewportId::from_hash_of("hb-passive-reference"),egui::ViewportBuilder::default().with_title("Horizon reference").with_position(anchor).with_inner_size(size).with_always_on_top().with_active(false).with_taskbar(false).with_mouse_passthrough(true).with_decorations(false).with_resizable(false),|child,_|{
 close=child.input(|i|i.viewport().close_requested());
 egui::CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(egui::Margin::same(5))).show(child,|ui|{
 let scale=(ui.available_width()/t.size()[0]as f32).min(ui.available_height()/t.size()[1]as f32).min(1.);
 let res=ui.add(egui::Image::new(t).fit_to_exact_size(t.size_vec2()*scale).tint(Color32::WHITE.gamma_multiply(s.opacity.clamp(0.4,1.))));
 if let Some(map)=a.settings.map_notes.get(&s.key){let r=res.rect;let p=ui.painter().with_clip_rect(r);for(i,v)in map.pins.iter().enumerate(){let at=pos2(r.left()+v.x*r.width(),r.top()+v.y*r.height());p.circle_filled(at,6.,if v.done{V25_MINT}else{RED});p.text(at,egui::Align2::CENTER_CENTER,(i+1).to_string(),FontId::proportional(9.),WHITE);}}
 });
 });
 if close{s.pinned=false;ctx.data_mut(|d|d.insert_temp(id,s));}
 }
}

#[derive(Clone,Default)]struct ReactionState{at:Option<std::time::Instant>,last:Option<f32>,best:Option<f32>,early:bool}
fn reaction_drill(ui:&mut Ui){
 let id=egui::Id::new("reaction-drill");let mut s=ui.data(|d|d.get_temp::<ReactionState>(id)).unwrap_or_default();let now=std::time::Instant::now();
 egui::CollapsingHeader::new("Reaction timing drill").show(ui,|ui|{
 if ui.button("Start random delay").clicked(){let n=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos();s.at=Some(now+std::time::Duration::from_millis(1200+u64::from(n%2300)));s.early=false;}
 let ready=s.at.is_some_and(|t|now>=t);
 let res=ui.add_sized([ui.available_width(),65.],egui::Button::new(if ready{"CLICK NOW"}else if s.at.is_some(){"Wait..."}else{"Start above"}).fill(if ready{V25_MINT}else{CARD}));
 if res.clicked(){if let Some(t)=s.at.take(){if now>=t{let ms=now.duration_since(t).as_secs_f32()*1000.;s.last=Some(ms);s.best=Some(s.best.map_or(ms,|v|v.min(ms)));}else{s.early=true;}}}
 if s.early{caption(ui,"Too early. Try again.");}
 if let Some(v)=s.last{ui.label(format!("Last {:.0} ms / best {:.0} ms",v,s.best.unwrap_or(v)));}
 if s.at.is_some(){ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));}
 caption(ui,"Local timing includes app refresh and input latency. No game interaction.");
 });ui.data_mut(|d|d.insert_temp(id,s));
}

#[derive(Clone,Default)]struct OverheadMatrix{base:Option<OptMeasurement>,rows:[Option<OptMeasurement>;3],config:Option<(u8,u8,u32)>,error:String}
fn overhead_matrix(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();let id=egui::Id::new(("overhead-matrix",&key));let mut s=ui.data(|d|d.get_temp::<OverheadMatrix>(id)).unwrap_or_default();
 let capture=a.recorder.elapsed().is_some()||a.replay.is_buffering();let output=a.native_mirror.as_ref().is_some_and(|w|w.running());let hud=a.overlay_open;
 let state=[capture,output,hud];let cfg=(a.settings.native_scale_mode,a.settings.processing_percent,a.settings.preview_fps_limit);
 egui::CollapsingHeader::new("Independent feature overhead samples").show(ui,|ui|{
 caption(ui,"Same game, same scene, same preset. Switch one feature on at a time and wait 10 seconds. No automatic feature toggles.");
 if ui.add_enabled(!state.iter().any(|v|*v),egui::Button::new("Sample all features OFF")).clicked(){s.base=opt_sample(a);s.rows=Default::default();s.config=Some(cfg);s.error=if s.base.is_some(){String::new()}else{"No valid game sample".into()};}
 for(i,n)in ["Capture","Own GPU output","HUD"].iter().enumerate(){
 ui.horizontal_wrapped(|ui|{ui.label(*n);
 if ui.add_enabled(s.base.is_some()&&state[i]&&state.iter().filter(|v|**v).count()==1,egui::Button::new("Sample ON")).clicked(){
 if let (Some(b),Some(v))=(&s.base,opt_sample(a)){if s.config==Some(cfg)&&opt_measurement_pair(b,&v){let row=crate::suite_finish::MeasuredOverhead{unix:crate::utility_book::now(),feature:i as u8,baseline:b.fps,active:v.fps,baseline_low:b.low,active_low:v.low,fg:a.settings.external_fg_factor.max(1)};if row.valid()&&!a.is_smoke{let h=a.settings.overhead_history.entry(key.clone()).or_default();h.push(row);if h.len()>24{h.remove(0);}a.save();}s.rows[i]=Some(v);s.error.clear();}else{s.error="Same PID/preset and minimum sample interval required".into();}}
 }
 if let (Some(b),Some(v))=(&s.base,&s.rows[i]){if let Some(p)=opt_delta(b.fps,v.fps){ui.label(format!("{p:+.1}% FPS / {:+.2} ms",v.ms-b.ms));}}
 });
 }
 if !s.error.is_empty(){caption(ui,&s.error);}
 if let Some(h)=a.settings.overhead_history.get(&key){egui::CollapsingHeader::new("Saved samples for this game").show(ui,|ui|{for v in h.iter().rev().take(12){ui.label(format!("{} / {:.1} -> {:.1} FPS / 1% low {:?} -> {:?}",["Capture","Own output","HUD","Capture + FG","Capture ON/OFF"][v.feature as usize],v.baseline,v.active,v.baseline_low,v.active_low));}});}
 let mut limit=a.settings.overhead_limits.get(&key).copied().unwrap_or(10);
 if ui.add(egui::Slider::new(&mut limit,1..=50).suffix("%").text("Allowed measured FPS loss")).changed(){a.settings.overhead_limits.insert(key.clone(),limit);a.save();}
 for(i,n)in ["Capture","GPU output","HUD","Capture + FG"].iter().enumerate(){
  let v=a.settings.overhead_history.get(&key).and_then(|h|crate::suite_finish::measured_cost(h,i as u8));
  match v{Some(p)=>{ui.colored_label(if p>f32::from(limit){RED}else{MUTED},format!("{n}: measured {p:+.1}% loss / {}",if p>f32::from(limit){"over budget"}else{"within budget"}));},None=>{caption(ui,&format!("{n}: not measured"));}}
 }
 caption(ui,"Budget advisor uses latest independent samples, never adds their percentages or changes settings without your action.");
 caption(ui,"Manual rolling samples, not controlled benchmarks or causal estimates.");
 });ui.data_mut(|d|d.insert_temp(id,s));
}

#[derive(Clone,Default)]struct LocalAssistState{ocr:Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<String,String>>>>>,text:String,voice:Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<bool,String>>>>>,target:Option<(String,Option<u32>,u64)>,status:String}
fn assist_panel(a:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("local-assist-v117");let mut s=ui.data(|d|d.get_temp::<LocalAssistState>(id)).unwrap_or_default();
 frame(PANEL).show(ui,|ui|{
 heading(ui,"LOCAL OCR / VOICE CLIP");
 caption(ui,"Installed Windows language support required. OCR only reads your selected local image. Voice is one-shot (up to 12 s), OFF by default; say save clip.");
 if ui.add_enabled(a.settings.optional_modules.ocr&&s.ocr.is_none(),egui::Button::new("Read text from local image")).clicked()&&!a.is_smoke{if let Some(p)=crate::filedialog::select_image(){let(tx,rx)=std::sync::mpsc::channel();s.ocr=Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));s.text.clear();std::thread::spawn(move||{let _=tx.send(crate::local_assist::ocr(&p));});}}
 if !s.text.is_empty(){ui.add(egui::TextEdit::multiline(&mut s.text).desired_rows(4).desired_width(ui.available_width()));if ui.button("Copy recognized text").clicked(){ui.ctx().copy_text(s.text.clone());}}
 let ready=a.replay.is_buffering()&&a.replay.ready()&&!a.replay.saving();
 if ui.add_enabled(a.settings.optional_modules.voice&&ready&&s.voice.is_none(),egui::Button::new("Allow microphone once / listen for save clip")).clicked()&&!a.is_smoke{let(tx,rx)=std::sync::mpsc::channel();s.voice=Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));s.target=Some((a.profile_key(),a.telem.active_game.as_ref().map(|g|g.pid),a.reference_epoch));s.status="Listening once; no audio file saved.".into();std::thread::spawn(move||{let _=tx.send(crate::local_assist::listen_once());});}
 if s.voice.is_some(){caption(ui,"Microphone session ends within 12 seconds.");if ui.button("Discard voice result").clicked(){s.target=None;s.status="Result discarded; microphone session ends at timeout.".into();}}
 if !s.status.is_empty(){caption(ui,&s.status);}
 });ui.data_mut(|d|d.insert_temp(id,s));
}
fn assist_tick(a:&mut BoostApp,ctx:&egui::Context){
 let id=egui::Id::new("local-assist-v117");let Some(mut s)=ctx.data(|d|d.get_temp::<LocalAssistState>(id))else{return;};
 let result=s.ocr.as_ref().and_then(|r|r.lock().ok().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("OCR worker stopped".into())),_=>None}));
 if let Some(v)=result{s.ocr=None;match v{Ok(t)=>{s.text=t;s.status="OCR completed".into();},Err(e)=>s.status=e}}
 let result=s.voice.as_ref().and_then(|r|r.lock().ok().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Speech worker stopped".into())),_=>None}));
 if let Some(v)=result{s.voice=None;let valid=a.settings.optional_modules.voice&&s.target.take()==Some((a.profile_key(),a.telem.active_game.as_ref().map(|g|g.pid),a.reference_epoch));match v{Ok(true) if valid&&a.replay.is_buffering()&&a.replay.ready()&&!a.replay.saving()=>{a.save_replay();s.status="Voice command matched; replay save requested.".into();},Ok(_)=>s.status="No usable voice command / target changed.".into(),Err(e)=>s.status=e}}
 if s.ocr.is_some()||s.voice.is_some(){ctx.request_repaint_after(std::time::Duration::from_millis(200));}ctx.data_mut(|d|d.insert_temp(id,s));
}

fn named_macro_binding(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();
 egui::CollapsingHeader::new("Named macro / per-game shortcut").show(ui,|ui|{
 let names=a.settings.window_macro_profiles.get(&key).map(|b|b.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
 let mut name=a.settings.named_macro_hotkeys.get(&key).cloned().unwrap_or_default();
 let old=name.clone();
 egui::ComboBox::from_id_salt("named-macro-hotkey").selected_text(if name.is_empty(){"Unbound"}else{&name}).show_ui(ui,|ui|{ui.selectable_value(&mut name,String::new(),"Unbound");for n in names{ui.selectable_value(&mut name,n.clone(),n);}});
 if name!=old&&!a.is_smoke{a.named_macro_arm=None;if name.is_empty(){a.settings.named_macro_hotkeys.remove(&key);}else{a.settings.named_macro_hotkeys.insert(key.clone(),name.clone());}a.save();}
 ui.label(format!("Existing conflict-checked MACRO chord: {}",a.settings.global_shortcuts.label("MACRO")));
 caption(ui,"Select permitted offline game window and confirm consent below, then arm here. One playback only, within 60 seconds, original game and window. No persistent consent.");
 let plan=a.settings.window_macro_profiles.get(&key).and_then(|b|b.get(&name)).cloned();
 let target=a.window_macro.selected.clone().filter(|t|a.telem.active_game.as_ref().is_some_and(|g|g.pid==t.pid&&!crate::anticheat_guard::inspect(&g.path,&g.title).blocked));
 let ready=plan.as_ref().is_some_and(|p|p.validate().is_ok())&&target.is_some()&&a.window_macro.consent&&!a.window_macro.busy()&&!a.macro_runner.running()&&a.macro_armed_pid.is_none()&&a.settings.global_shortcuts.get("MACRO").is_some();
 if ui.add_enabled(ready&&!a.is_smoke,egui::Button::new("Arm once for 60 seconds")).clicked(){if let(Some(t),Some(p))=(target,plan){a.named_macro_arm=Some((key.clone(),t,p,std::time::Instant::now()+std::time::Duration::from_secs(60)));a.window_macro.consent=false;}}
 if a.named_macro_arm.is_some(){ui.colored_label(RED,"ARMED / use MACRO chord in selected offline window");if ui.button("Disarm").clicked(){a.named_macro_arm=None;}}
 });
}
