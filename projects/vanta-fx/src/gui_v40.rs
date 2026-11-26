fn capture_encoder_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"CAPTURE SETTINGS / ENCODER LAB");
  let idle=!app.recorder.busy()&&!app.replay.busy()&&app.media_export.is_none()&&app.encoder_rx.is_none();
  ui.add_enabled_ui(idle,|ui|{
   ui.checkbox(&mut app.audio_next.system,"Include all default playback audio in NEXT recording");
   ui.checkbox(&mut app.audio_next.microphone,"Include default microphone as separate track in NEXT recording");
   if ui.add(egui::Slider::new(&mut app.settings.capture_gains[0],0.0..=2.).text("Playback gain")).changed(){app.save();}
   if ui.add(egui::Slider::new(&mut app.settings.capture_gains[1],0.0..=2.).text("Microphone gain")).changed(){app.save();}
   if ui.checkbox(&mut app.settings.capture_mute_mic,"Mute captured mic samples").changed(){app.save();}
   caption(ui,"Applied during next recording; not Windows volume. Muted mic still requires explicit endpoint consent.");
   caption(ui,"Both off by default; each selection is consumed once when starting recording. System audio includes other apps, microphone may include nearby voices. No capture until you start recording. Separate local WAVs and original MKV retained.");
   if ui.add(egui::Slider::new(&mut app.settings.capture_mbps,2..=16).suffix(" Mbps").text("Target bitrate")).changed(){app.save();}
   if ui.add(egui::Slider::new(&mut app.settings.replay_limit_mib,64..=1024).suffix(" MiB").text("Replay stop threshold")).changed(){app.save();}
   if ui.checkbox(&mut app.settings.capture_remux,"After recording, also create MP4").changed(){app.save();}
   if outline_button(ui,"Test AMD AMF and CPU encoder")&&!app.is_smoke{
    let(tx,rx)=std::sync::mpsc::channel();app.encoder_rx=Some(rx);
    std::thread::spawn(move||{let _=tx.send(crate::media_tools::encoder_probe());});
   }
  });
  if app.encoder_rx.is_some(){ui.spinner();caption(ui,"Encoding synthetic test frames, not the desktop.");}
  match &app.encoder_result{
   Some(Ok(v))=>{caption(ui,&format!("Actual 8-frame encoding: AMD AMF {} / CPU x264 {}",if v.amf{"PASS"}else{"Unavailable"},if v.cpu{"PASS"}else{"Unavailable"}));
    if !v.amf&&v.cpu&&idle&&outline_button(ui,"Select CPU fallback"){app.settings.hardware_encoder=false;app.save();}
   },Some(Err(e))=>caption(ui,e),None=>{}
  }
  if let Some(t)=app.recorder.elapsed(){caption(ui,&format!("Recording {:.1} s / {}",t,app.recorder.encoder().unwrap_or("Unknown")));}
  if app.recorder.finishing(){caption(ui,"Finalizing video / verifying stream. Please keep app open.");}
  caption(ui,&format!("Replay buffer last measured: {:.1} MiB. Stop threshold sampled once per second; temporary export copies use additional space.",app.replay.bytes() as f64/1048576.));
  if app.recorder.audio_requested()>0{ui.colored_label(RED,format!("{} audio endpoint(s) requested for this recording",app.recorder.audio_requested()));}
  caption(ui,"Video-only by default. Optional WASAPI system/mic tracks are separate, approximate initial A/V alignment; failure keeps original MKV and any WAV. Replay remains video-only. No automatic CPU fallback.");
 });
}
fn capture_audio_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"OUTPUT AUDIO PEAK");
  if ui.checkbox(&mut app.audio_enabled,"Read default playback endpoint peak").changed()&&!app.audio_enabled{app.audio_meter.reset();}
  match &app.audio_meter.value{
   Some(Ok(v))=>{ui.add(egui::ProgressBar::new(*v).text(format!("{:.0}% peak amplitude",v*100.)));},
   Some(Err(e))=>caption(ui,e),None=>caption(ui,"Not sampled")
  }
  caption(ui,"Windows endpoint peak meter does not access raw PCM.");
  ui.separator();heading(ui,"PLAYBACK SPECTRUM / OPTIONAL");
  ui.checkbox(&mut app.spectrum_enabled,"Enable transient system playback FFT for this session");
  caption(ui,"Opt-in: analyzes first channel of default playback, including other apps. No microphone, file recording, raw export or network. Stops on leaving Capture unless audio HUD is open; Emergency Stop clears it.");
  if let Some(r)=app.spectrum.reading(){
   caption(ui,&format!("{} Hz sample rate / peak {:.0}% / RMS {:.0}% / dominant {}",r.rate,r.peak*100.,r.rms*100.,r.dominant_hz.map(|v|format!("{v:.0} Hz")).unwrap_or("Unavailable".into())));
   let(rct,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),105.),egui::Sense::hover());ui.painter().rect_filled(rct,5.,BG);
   let width=rct.width()/12.;
   for(i,b)in r.bands.iter().enumerate(){let value=b.map(|v|((v+72.)/72.).clamp(0.,1.)).unwrap_or(0.);let left=rct.left()+i as f32*width+2.;let bottom=rct.bottom()-19.;
    ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(left,bottom-value*(rct.height()-23.)),egui::pos2(left+(width-4.).max(1.),bottom)),2.,if i<4{RED}else if i<8{V25_MINT}else{Color32::from_rgb(110,162,245)});
    let hz=crate::audio_spectrum::EDGES[i];let label=if hz>=1000.{format!("{:.0}k",hz/1000.)}else{format!("{hz:.0}")};
    ui.painter().text(egui::pos2(left,bottom+2.),egui::Align2::LEFT_TOP,label,FontId::proportional(9.),MUTED);
   }
   ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
  }else if let Some(e)=app.spectrum.error(){caption(ui,e);}else if app.spectrum_enabled{caption(ui,"Waiting for fresh playback samples");ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));}
  else{caption(ui,"Disabled / no raw audio access");}
  caption(ui,"2048-point Hann FFT, relative peak-bin dBFS per band; not calibrated sound pressure.");
 });
}
fn capture_library_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"LOCAL CAPTURE LIBRARY");
  caption(ui,"Only HBoost_ files in app screenshot/video directories. Read-only list, no automatic deletion, upload or playback.");
  if outline_button(ui,"Refresh library")&&!app.is_smoke{app.media_gallery=crate::media_tools::gallery();}
  let idle=app.media_rx.is_none()&&app.media_export.is_none()&&!app.recorder.busy()&&!app.replay.busy()&&app.encoder_rx.is_none();
  let filter_id=egui::Id::new("clip-book-filter");let mut filters=ui.data(|d|d.get_temp::<(String,bool,bool)>(filter_id)).unwrap_or_default();
  ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut filters.0).hint_text("Search title, tags, collection").char_limit(100));ui.checkbox(&mut filters.1,"Favorites");ui.checkbox(&mut filters.2,"This game");});
  ui.data_mut(|d|d.insert_temp(filter_id,filters.clone()));
  let game=app.profile_key();
  let mut thumbs=thumb_state(ui);
  ui.checkbox(&mut thumbs.enabled,"Generate local thumbnail previews (optional)");
  thumb_poll(ui,&mut thumbs);
  let mut visible=0usize;
  egui::ScrollArea::vertical().id_salt("capture-media-gallery").max_height(330.).show(ui,|ui|{
   for item in app.media_gallery.clone(){
    let label=item.path.file_name().and_then(|n|n.to_str()).unwrap_or("Capture");
    let meta=app.settings.clip_annotations.get(label).cloned().unwrap_or_default();
    if !meta.matches(label,&filters.0)||(filters.1&&!meta.favorite)||(filters.2&&meta.game!=game){continue;}
    let name=if meta.title.is_empty(){label}else{&meta.title};
    visible+=1;
    let mut clicked=false;
    ui.horizontal(|ui|{
     if thumbs.enabled&&visible<=12{thumb_row(app,ui,&mut thumbs,&item.path,idle);}
     clicked=ui.selectable_label(app.media_selected.as_ref()==Some(&item.path),format!("{}{} / {:.1} MiB",if meta.favorite{"* "}else{""},name,item.bytes as f64/1048576.)).clicked();
    });
    if clicked&&idle{
     app.media_selected=Some(item.path.clone());app.media_info=None;
     if item.video{
      let(tx,rx)=std::sync::mpsc::channel();app.media_rx=Some(rx);
      std::thread::spawn(move||{let _=tx.send(crate::media_tools::probe(&item.path));});
     }
    }
   }
  });
  if thumbs.pending.is_some(){ui.spinner();ui.ctx().request_repaint_after(std::time::Duration::from_millis(150));}
  if ui.small_button("Clear thumbnail cache").clicked(){thumbs.textures.clear();thumbs.failed.clear();}
  ui.data_mut(|d|d.insert_temp(egui::Id::new("hb-clip-thumbs"),thumbs));
  if let Some(path)=app.media_selected.clone(){
   clip_annotation_panel(app,ui,&path);
   media_still_panel(app,ui,&path,idle);
   if outline_button(ui,"Copy local file path"){ui.ctx().copy_text(path.to_string_lossy().into_owned());}
   if ui.add_enabled(idle,egui::Button::new("Export copy to chosen folder")).clicked()&&!app.is_smoke{
    if let Some(dir)=crate::filedialog::select_directory(){let source=path.clone();let(tx,rx)=std::sync::mpsc::channel();app.media_export=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::suite_finish::local_copy(&source,&dir));});}
   }
   caption(ui,"Local sharing: copy a capture to your chosen folder, then share it yourself. Never overwrites or uploads.");
   if let Some(info)=app.media_info.clone(){
    caption(ui,&format!("{:.2}s / {}x{} / {}",info.duration,info.width,info.height,info.codec));
    ui.add_enabled_ui(idle,|ui|{
     ui.add(egui::Slider::new(&mut app.trim_start,0.0..=info.duration).suffix(" s").text("Start"));
     ui.add(egui::Slider::new(&mut app.trim_end,0.0..=info.duration).suffix(" s").text("End"));
     if outline_button(ui,"Export selected range (keep audio tracks)")&&!app.is_smoke{
      let start=app.trim_start;let end=app.trim_end;let hardware=app.settings.hardware_encoder;let rate=app.settings.capture_mbps;
      let output=crate::record::videos().join(format!("HBoost_Trim_{}.mp4",crate::media_tools::stamp()));
      let(tx,rx)=std::sync::mpsc::channel();app.media_export=Some(rx);
      let p=path.clone();std::thread::spawn(move||{let _=tx.send(crate::media_tools::trim(&p,&output,start,end,hardware,rate).map(|_|output));});
     }
     clip_export_panel(app,ui,&path);
     clip_split_panel(app,ui,&path,info.duration);
     audio_gain_panel(app,ui,&path);
     montage_add_controls(app,ui,&path);
     if path.extension().is_some_and(|v|v=="mkv")&&outline_button(ui,"Export MKV to new MP4")&&!app.is_smoke{
      let output=crate::record::videos().join(format!("HBoost_Remux_{}.mp4",crate::media_tools::stamp()));
      let(tx,rx)=std::sync::mpsc::channel();app.media_export=Some(rx);
      let p=path.clone();std::thread::spawn(move||{let _=tx.send(crate::media_tools::remux(&p,&output).map(|_|output));});
     }
    });
   }
  }
  if app.media_export.is_some()||app.media_rx.is_some(){ui.spinner();}
  caption(ui,"Trim re-encodes up to 120 seconds for accurate cuts, without replacing the source. MKV remux preserves original streams.");
 });
}
fn capture_studio_panel(app:&mut BoostApp,ui:&mut Ui){capture_encoder_panel(app,ui);capture_audio_panel(app,ui);capture_library_panel(app,ui);montage_panel(app,ui);}

fn break_and_alert_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"BREAK TIMER / RESOURCE WARNINGS");
  if ui.add(egui::Slider::new(&mut app.settings.break_minutes,5..=180).suffix(" min").text("Break interval")).changed(){app.save();}
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Start one-shot timer"){app.break_timer.start(std::time::Instant::now(),app.settings.break_minutes);}
   if outline_button(ui,"Cancel"){app.break_timer.stop();}
   if let Some(s)=app.break_timer.remaining(std::time::Instant::now()){ui.label(format!("{}:{:02}",s/60,s%60));}
  });
  if ui.checkbox(&mut app.settings.resource_alerts,"Opt-in VRAM/frametime warnings").changed(){app.save();}
  caption(ui,"Measured AMD edge >=85 C (while sensor polling enabled), own-process DXGI budget >=90% or actual game ETW average frametime above configured threshold. At most one message per 30s. No CPU temperature guesses or automatic hardware changes.");
 });
}

#[derive(Clone,Default)]struct StillState{selected:Option<std::path::PathBuf>,seconds:f64,texture:Option<egui::TextureHandle>,pending:Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<image::RgbaImage,String>>>>>,message:Option<String>}
fn media_still_panel(app:&mut BoostApp,ui:&mut Ui,path:&std::path::Path,idle:bool){
 let id=egui::Id::new("media-still-v60");let mut state=ui.data_mut(|d|d.get_temp::<StillState>(id)).unwrap_or_default();
 if state.selected.as_deref()!=Some(path){state=StillState{selected:Some(path.to_path_buf()),..Default::default()};}
 if let Some(rx)=state.pending.as_ref(){let v=rx.lock().ok().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Preview worker stopped".into())),_=>None});
  if let Some(v)=v{state.pending=None;match v{Ok(img)=>{state.texture=Some(ui.ctx().load_texture("media-still",egui::ColorImage::from_rgba_unmultiplied([img.width() as usize,img.height() as usize],img.as_raw()),egui::TextureOptions::LINEAR));state.message=None;},Err(e)=>state.message=Some(e)}}}
 if let Some(info)=&app.media_info{ui.add(egui::Slider::new(&mut state.seconds,0.0..=(info.duration-0.05).max(0.)).text("Still preview time").suffix(" s"));}
 if ui.add_enabled(idle&&state.pending.is_none(),egui::Button::new("Load selected still preview")).clicked()&&!app.is_smoke{let(tx,rx)=std::sync::mpsc::channel();let p=path.to_path_buf();let seconds=state.seconds;state.pending=Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));state.texture=None;std::thread::spawn(move||{let _=tx.send(crate::media_tools::still(&p,seconds));});}
 if state.pending.is_some(){ui.spinner();ui.ctx().request_repaint_after(std::time::Duration::from_millis(80));}
 if let Some(t)=&state.texture{let z=(ui.available_width()/t.size()[0] as f32).min(1.);ui.add(egui::Image::new(t).fit_to_exact_size(t.size_vec2()*z));}
 if let Some(s)=&state.message{caption(ui,s);}caption(ui,"Local decoded still, not video playback. No auto-open or upload.");
 ui.data_mut(|d|d.insert_temp(id,state));
}

fn clip_annotation_panel(a:&mut BoostApp,ui:&mut Ui,path:&std::path::Path){
 let Some(key)=crate::clip_book::key(path)else{return;};
 let mut meta=a.settings.clip_annotations.get(&key).cloned().unwrap_or_default();
 let mut changed=false;
 ui.group(|ui|{
  heading(ui,"CLIP DETAILS / BOOKMARKS");
  ui.horizontal_wrapped(|ui|{changed|=ui.checkbox(&mut meta.favorite,"Favorite").changed();if ui.small_button("Assign selected game").clicked(){meta.game=a.profile_key();changed=true;}});
  changed|=ui.add(egui::TextEdit::singleline(&mut meta.title).hint_text("Clip title").char_limit(120)).changed();
  changed|=ui.add(egui::TextEdit::singleline(&mut meta.tags).hint_text("Tags").char_limit(240)).changed();
  changed|=ui.add(egui::TextEdit::singleline(&mut meta.collection).hint_text("Collection").char_limit(80)).changed();
  if !meta.game.is_empty(){caption(ui,"Assigned to a saved game profile");}
  if let Some(info)=&a.media_info{
   let id=egui::Id::new(("clip-marker-input",&key));let mut mark=ui.data(|d|d.get_temp::<(f64,String)>(id)).unwrap_or_default();
   mark.0=mark.0.clamp(0.,info.duration);
   ui.add(egui::Slider::new(&mut mark.0,0.0..=info.duration).suffix(" s").text("Bookmark time"));
   ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut mark.1).hint_text("Moment name").char_limit(100));if ui.button("Add bookmark").clicked()&&meta.add(mark.0,info.duration,&mark.1){changed=true;mark.1.clear();}});
   ui.data_mut(|d|d.insert_temp(id,mark));
   let (rect,_)=ui.allocate_exact_size(vec2(ui.available_width(),24.),Sense::hover());
   ui.painter().rect_filled(rect,5.,CARD);
   for v in &meta.markers{let x=rect.left()+rect.width()*(v.ms as f64/1000./info.duration).clamp(0.,1.) as f32;ui.painter().line_segment([pos2(x,rect.top()+3.),pos2(x,rect.bottom()-3.)],Stroke::new(2.,RED));}
   let mut remove=None;
   for (i,v) in meta.markers.iter().enumerate(){ui.horizontal_wrapped(|ui|{ui.label(format!("{:.2} s",v.ms as f64/1000.));ui.label(&v.label);if ui.small_button("Set trim start").clicked(){a.trim_start=(v.ms as f64/1000.).min(info.duration);}if ui.small_button("Remove").clicked(){remove=Some(i);}});}
   if let Some(i)=remove{meta.markers.remove(i);changed=true;}
   caption(ui,"Manual bookmarks use the verified duration of the selected recording.");
  }
 });
 if changed&&!a.is_smoke&&(a.settings.clip_annotations.contains_key(&key)||a.settings.clip_annotations.len()<2000){meta.sanitize();a.settings.clip_annotations.insert(key,meta);a.save();}
}
fn clip_export_panel(a:&mut BoostApp,ui:&mut Ui,path:&std::path::Path){
 let id=egui::Id::new("clip-export-format");let mut mode=ui.data(|d|d.get_temp::<usize>(id)).unwrap_or(0).min(2);
 ui.horizontal_wrapped(|ui|{for(i,name)in crate::media_tools::EXPORT_NAMES.iter().enumerate(){ui.selectable_value(&mut mode,i,*name);}});
 ui.data_mut(|d|d.insert_temp(id,mode));
 caption(ui,"CPU export / original preserved. Aspect ratio retained with black padding. Audio tracks retained. No upload.");
 if ui.add_enabled(a.media_export.is_none(),egui::Button::new("Export selected format")).clicked()&&!a.is_smoke{
  let start=a.trim_start;let end=a.trim_end;let rate=a.settings.capture_mbps;let p=path.to_owned();
  let output=crate::record::videos().join(format!("HBoost_Format_{}.mp4",crate::media_tools::stamp()));
  let(tx,rx)=std::sync::mpsc::channel();a.media_export=Some(rx);
  std::thread::spawn(move||{let _=tx.send(crate::media_tools::export_format(&p,&output,start,end,mode,rate).map(|_|output));});
 }
}

fn recording_session_panel(a:&mut BoostApp,ui:&mut Ui){
 let idle=!a.recorder.busy();
 ui.add_enabled_ui(idle,|ui|{
  if ui.add(egui::Slider::new(&mut a.settings.recording_limit_mib,256..=16384).step_by(256.).suffix(" MiB").text("Video stop threshold")).changed(){a.save();}
  if ui.checkbox(&mut a.settings.recording_stop_on_exit,"Stop when selected game exits").changed(){a.save();}
 });
 caption(ui,"Manually start desktop recording; stop threshold sampled once per second, video bytes only. Audio and finalization may use extra space. Stop-on-exit tracks the original process, not foreground focus.");
 if let Some(elapsed)=a.recorder.elapsed(){
  ui.label(format!("{:.1} s   /   {:.1} MiB video",elapsed,a.capture_session.bytes as f64/1048576.));
  let id=egui::Id::new("record-live-mark-label");let mut label=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_default();
  ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut label).hint_text("Moment").char_limit(100));if ui.button("Mark moment").clicked()&&!a.is_smoke&&a.capture_session.add_marker(elapsed,&label){label.clear();}});
  ui.data_mut(|d|d.insert_temp(id,label));
  for mark in a.capture_session.markers().iter().rev().take(5){caption(ui,&format!("{:.1} s  {}",mark.ms as f64/1000.,mark.label));}
  caption(ui,"Live marks use recording wall-clock time; encoder startup may cause a small offset. Saved with the finalized clip and original.");
  ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));
 }
 if let Some(s)=&a.capture_session.reason{caption(ui,s);}
}

fn clip_split_panel(a:&mut BoostApp,ui:&mut Ui,path:&std::path::Path,duration:f64){
 if duration<0.5||duration>240.{return;}
 let id=egui::Id::new(("clip-split",path));let mut at=ui.data(|d|d.get_temp::<f64>(id)).unwrap_or(duration/2.).clamp(0.25,duration-0.25);
 ui.add(egui::Slider::new(&mut at,0.25..=duration-0.25).suffix(" s").text("Split point"));ui.data_mut(|d|d.insert_temp(id,at));
 if ui.add_enabled(a.media_export.is_none()&&at<=120.&&duration-at<=120.,egui::Button::new("Split into two new clips")).clicked()&&!a.is_smoke{
  let src=path.to_owned();let dir=crate::record::videos();let stamp=crate::media_tools::stamp();let left=dir.join(format!("HBoost_Split_{stamp}_A.mp4"));let right=dir.join(format!("HBoost_Split_{stamp}_B.mp4"));let rate=a.settings.capture_mbps;
  let(tx,rx)=std::sync::mpsc::channel();a.media_export=Some(rx);
  std::thread::spawn(move||{let _=tx.send(crate::media_tools::split(&src,&left,&right,at,rate).map(|_|right));});
 }
 caption(ui,"CPU accurate split. Two new files, source unchanged; up to 120 s each. Existing audio streams mapped.");
}

#[derive(Clone,Default,serde::Serialize,serde::Deserialize)]
struct MontageState{pieces:Vec<crate::montage::Piece>,format:usize,audio:bool}
fn montage_state(ui:&Ui)->MontageState{ui.data(|d|d.get_temp::<MontageState>(egui::Id::new("hb-montage-v112"))).unwrap_or_default()}
fn montage_store(ui:&Ui,v:MontageState){ui.data_mut(|d|d.insert_temp(egui::Id::new("hb-montage-v112"),v));}
fn montage_add_controls(a:&mut BoostApp,ui:&mut Ui,path:&std::path::Path){
 let mut s=montage_state(ui);
 ui.horizontal_wrapped(|ui|{
  if ui.add_enabled(s.pieces.len()<8,egui::Button::new("Add range to montage")).clicked(){
   let v=crate::montage::Piece{path:path.to_owned(),start:a.trim_start,end:a.trim_end,caption:String::new()};
   let mut candidate=s.pieces.clone();candidate.push(v);
   match crate::montage::total(&candidate){Ok(_)=>s.pieces=candidate,Err(e)=>a.toast(e)}
  }
  if ui.add_enabled(a.media_export.is_none(),egui::Button::new("Center crop to selected format")).clicked()&&!a.is_smoke{
   let mode=ui.data(|d|d.get_temp::<usize>(egui::Id::new("clip-export-format"))).unwrap_or(0).min(2);
   let src=path.to_owned();let start=a.trim_start;let end=a.trim_end;let rate=a.settings.capture_mbps;
   let output=crate::record::videos().join(format!("HBoost_Crop_{}.mp4",crate::media_tools::stamp()));
   let(tx,rx)=std::sync::mpsc::channel();a.media_export=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::montage::crop(&src,&output,start,end,mode,rate).map(|_|output));});
  }
 });
 caption(ui,"Center crop fills the chosen frame and cuts off edges, unlike padded export.");
 montage_store(ui,s);
}
fn montage_panel(a:&mut BoostApp,ui:&mut Ui){
 let mut s=montage_state(ui);
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"MONTAGE STUDIO");
  caption(ui,"Add selected video ranges from Clip library. Up to 8 ranges, 120 seconds total. Save or reopen local edit project.");
  let project=crate::settings::folder().join("montage-project.json");
  let msg_id=egui::Id::new("montage-project-msg");let mut message=ui.data(|d|d.get_temp::<String>(msg_id)).unwrap_or_default();
  ui.horizontal_wrapped(|ui|{
   if ui.button("Save edit project").clicked()&&!a.is_smoke{message=(||->Result<String,String>{crate::montage::total(&s.pieces)?;let bytes=serde_json::to_vec_pretty(&s).map_err(|e|e.to_string())?;std::fs::create_dir_all(crate::settings::folder()).map_err(|e|e.to_string())?;crate::settings::atomic_json_replace(&project,&bytes)?;Ok("Project saved".into())})().unwrap_or_else(|e|e);}
   if ui.button("Load saved project").clicked()&&!a.is_smoke{let result=(||->Result<MontageState,String>{if project.metadata().map_err(|e|e.to_string())?.len()>65536{return Err("Project too large".into());}let mut v:MontageState=serde_json::from_slice(&std::fs::read(&project).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;crate::montage::total(&v.pieces)?;for p in &mut v.pieces{if crate::clip_book::key(&p.path).is_none()||p.path.parent()!=Some(crate::record::videos().as_path()){return Err("Project references a non-library file".into());}p.caption=p.caption.chars().take(160).collect();}v.format=v.format.min(2);Ok(v)})();match result{Ok(v)=>{s=v;message="Project loaded".into();},Err(e)=>message=e}}
  });if !message.is_empty(){caption(ui,&message);}ui.data_mut(|d|d.insert_temp(msg_id,message));
  if let Ok(total)=crate::montage::total(&s.pieces){let(r,_)=ui.allocate_exact_size(vec2(ui.available_width(),34.),Sense::hover());let mut x=r.left();for(i,p)in s.pieces.iter().enumerate(){let w=r.width()*((p.end-p.start)/total)as f32;let q=Rect::from_min_size(pos2(x,r.top()),vec2(w.max(1.),r.height()));ui.painter().rect_filled(q.shrink(1.),3.,if i%2==0{RED.gamma_multiply(0.65)}else{CARD});ui.painter().with_clip_rect(q).text(q.center(),egui::Align2::CENTER_CENTER,format!("{}",i+1),FontId::proportional(11.),WHITE);x+=w;}}
  montage_scrub(a,ui,&s.pieces);
  let mut action=None;let len=s.pieces.len();
  for(i,v)in s.pieces.iter_mut().enumerate(){ui.push_id(("montage",i),|ui|{
   ui.horizontal_wrapped(|ui|{ui.label(format!("{} / {}",i+1,v.path.file_name().and_then(|n|n.to_str()).unwrap_or("Clip")));ui.label(format!("{:.1}–{:.1} s",v.start,v.end));});
   ui.horizontal_wrapped(|ui|{
    if ui.add_enabled(i>0,egui::Button::new("Up")).clicked(){action=Some((i,0));}
    if ui.add_enabled(i+1<len,egui::Button::new("Down")).clicked(){action=Some((i,1));}
    if ui.small_button("Remove").clicked(){action=Some((i,2));}
   });
   ui.add(egui::TextEdit::singleline(&mut v.caption).hint_text("Optional subtitle").char_limit(160).desired_width(ui.available_width()));ui.separator();
  });}
  if let Some((i,k))=action{match k{0=>s.pieces.swap(i,i-1),1=>s.pieces.swap(i,i+1),_=>{s.pieces.remove(i);}}}
  let valid=crate::montage::total(&s.pieces);
  if let Ok(t)=valid.as_ref(){ui.label(format!("{len} ranges / {t:.1} s"));}else{caption(ui,valid.as_ref().unwrap_err());}
  ui.horizontal_wrapped(|ui|{for(i,n)in crate::media_tools::EXPORT_NAMES.iter().enumerate(){ui.selectable_value(&mut s.format,i,*n);}});
  ui.checkbox(&mut s.audio,"Include first audio track (silence for silent clips)");caption(ui,"Optional captions saved as separate SRT file. Audio mode retains first track only, not separate mic/game tracks. Original files untouched; no upload.");
  let idle=!a.recorder.busy()&&!a.replay.busy()&&a.media_export.is_none()&&a.media_rx.is_none()&&a.encoder_rx.is_none();
  if ui.add_enabled(idle&&valid.is_ok(),egui::Button::new("Export montage")).clicked()&&!a.is_smoke{
   let p=s.pieces.clone();let mode=s.format;let rate=a.settings.capture_mbps;let audio=s.audio;
   let output=crate::record::videos().join(format!("HBoost_Montage_{}.mp4",crate::media_tools::stamp()));
   let(tx,rx)=std::sync::mpsc::channel();a.media_export=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::montage::render_audio(&p,&output,mode,rate,audio).map(|_|output));});
  }
  if ui.small_button("Clear edit queue").clicked(){s.pieces.clear();}
 });
 montage_store(ui,s);
}

#[derive(Clone,Default)]
struct GalleryThumbs{
 enabled:bool,textures:std::collections::BTreeMap<std::path::PathBuf,egui::TextureHandle>,failed:std::collections::BTreeSet<std::path::PathBuf>,
 pending:Option<(std::path::PathBuf,std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<image::RgbaImage,String>>>>)>
}
fn thumb_state(ui:&Ui)->GalleryThumbs{ui.data(|d|d.get_temp::<GalleryThumbs>(egui::Id::new("hb-clip-thumbs"))).unwrap_or_default()}
fn thumb_poll(ui:&Ui,s:&mut GalleryThumbs){
 let result=s.pending.as_ref().and_then(|(p,rx)|rx.lock().ok().and_then(|r|match r.try_recv(){Ok(v)=>Some((p.clone(),v)),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some((p.clone(),Err("Thumbnail worker stopped".into()))),_=>None}));
 if let Some((p,v))=result{s.pending=None;match v{Ok(img)=>{
  if s.textures.len()>=24{if let Some(k)=s.textures.keys().next().cloned(){s.textures.remove(&k);}}
  s.textures.insert(p,ui.ctx().load_texture("clip-thumb",egui::ColorImage::from_rgba_unmultiplied([img.width() as usize,img.height() as usize],img.as_raw()),egui::TextureOptions::LINEAR));
 },Err(_)=>{if s.failed.len()<200{s.failed.insert(p);}}}}
}
fn thumb_row(a:&BoostApp,ui:&mut Ui,s:&mut GalleryThumbs,path:&std::path::Path,idle:bool){
 if let Some(t)=s.textures.get(path){let scale=(96./t.size()[0] as f32).min(54./t.size()[1] as f32);ui.add(egui::Image::new(t).fit_to_exact_size(t.size_vec2()*scale));return;}
 let(r,_)=ui.allocate_exact_size(vec2(96.,54.),Sense::hover());ui.painter().rect_filled(r,4.,CARD);ui.painter().text(r.center(),egui::Align2::CENTER_CENTER,if s.failed.contains(path){"Unavailable"}else{"Preview"},FontId::proportional(10.),MUTED);
 if idle&&!a.is_smoke&&!crate::suite_finish::gpu_pressure(a.vram_budget.as_ref().and_then(|v|v.as_ref().ok()).filter(|v|v.meaningful()).map(|v|v.fraction()),a.settings.pressure_quiet)&&s.pending.is_none()&&!s.failed.contains(path)&&s.textures.len()<24{
  let p=path.to_owned();let work=p.clone();let(tx,rx)=std::sync::mpsc::channel();
  s.pending=Some((p,std::sync::Arc::new(std::sync::Mutex::new(rx))));
  std::thread::spawn(move||{let v=crate::media_tools::still(&work,0.).map(|img|image::DynamicImage::ImageRgba8(img).thumbnail(160,90).to_rgba8());let _=tx.send(v);});
 }
}
#[derive(Clone,Default)]struct CaptureCompare{before:Option<OptMeasurement>,after:Option<OptMeasurement>,mode:Option<(bool,bool,u8,u8,u8,u32)>,message:String}
fn capture_metrics_panel(a:&mut BoostApp,ui:&mut Ui){
 overhead_matrix(a,ui);
 let id=egui::Id::new("capture-compare-v114");let mut s=ui.data(|d|d.get_temp::<CaptureCompare>(id)).unwrap_or_default();
 let recording=a.recorder.active();let replay=a.replay.active();
 frame(PANEL).show(ui,|ui|{
 heading(ui,"RECORDING OVERHEAD / REAL GAME METRICS");
 let disk_id=egui::Id::new("capture-disk-read");let now=std::time::Instant::now();
 let disk=ui.data(|d|d.get_temp::<(std::time::Instant,Option<u64>)>(disk_id)).filter(|(t,_)|now.duration_since(*t).as_secs()<5).unwrap_or_else(||(now,crate::suite_finish::free_bytes(&crate::record::videos())));
 ui.data_mut(|d|d.insert_temp(disk_id,disk));
 if let Some(n)=disk.1{ui.label(format!("Available capture disk: {:.2} GiB",n as f64/1073741824.));if n<1024*1024*1024{ui.colored_label(RED,"Low free space: avoid starting another recording");}}
 if let Some(w)=a.native_mirror.as_ref().filter(|w|w.running()){let v=w.diagnostics();caption(ui,&format!("Own GPU output diagnostics: {} duplicates / {} failed generated presents / {} non-submitted",v.duplicate_frames,v.failed_fg,v.non_submitted));}
 if let Some(p)=a.recorder.progress(){caption(ui,&format!("FFmpeg progress: frames {:?}, drops {:?}, duplicates {:?}, encode FPS {:?}",p.frames,p.drops,p.duplicates,p.fps));}
 caption(ui,"FFmpeg output counters do not measure all missed desktop frames; own GPU counters separate.");
 caption(ui,"Manual comparison in same game and scene. Disable recording and replay for baseline. Wait 10 seconds after starting recording. Not a controlled benchmark.");
 ui.horizontal_wrapped(|ui|{
 if ui.add_enabled(!recording&&!replay,egui::Button::new("Measure capture OFF")).clicked(){s.before=opt_sample(a);s.after=None;s.mode=Some((a.native_mirror.as_ref().is_some_and(|w|w.running()),a.overlay_open,a.native_mirror.as_ref().filter(|w|w.running()).map_or(1,|w|w.multiplier()),a.settings.native_scale_mode,a.settings.processing_percent,a.settings.preview_fps_limit));s.message=if s.before.is_some(){"Baseline sampled".into()}else{"Collect real game frames first".into()};}
 let ready=(recording&&a.recorder.elapsed().is_some_and(|v|v>=10.))||(replay&&a.replay.ready());
 if ui.add_enabled(ready&&s.before.is_some(),egui::Button::new("Measure capture ON")).clicked(){
  if let (Some(before),Some(after))=(&s.before,opt_sample(a)){
   if opt_measurement_pair(before,&after)&&s.mode==Some((a.native_mirror.as_ref().is_some_and(|w|w.running()),a.overlay_open,a.native_mirror.as_ref().filter(|w|w.running()).map_or(1,|w|w.multiplier()),a.settings.native_scale_mode,a.settings.processing_percent,a.settings.preview_fps_limit)){let row=crate::suite_finish::MeasuredOverhead{unix:crate::utility_book::now(),feature:if a.native_mirror.as_ref().is_some_and(|w|w.multiplier()>1){3}else{4},baseline:before.fps,active:after.fps,baseline_low:before.low,active_low:after.low,fg:a.native_mirror.as_ref().filter(|w|w.running()).map_or(1,|w|w.multiplier())};if row.valid()&&!a.is_smoke{let k=a.profile_key();let h=a.settings.overhead_history.entry(k).or_default();h.push(row);if h.len()>24{h.remove(0);}a.save();}s.after=Some(after);s.message="Measured comparison".into();}else{s.message="Same game/process and output/HUD state required; wait 10 seconds".into();}
  }else{s.message="Actual FPS sample unavailable".into();}
 }
 if ui.small_button("Clear").clicked(){s=Default::default();}
 });
 if let Some(v)=&s.before{ui.label(format!("OFF: {:.1} FPS / {:.2} ms",v.fps,v.ms));}
 if let (Some(x),Some(y))=(&s.before,&s.after){ui.label(format!("ON: {:.1} FPS / {:.2} ms",y.fps,y.ms));if let Some(v)=opt_delta(x.fps,y.fps){ui.label(RichText::new(format!("FPS difference {v:+.2}%")).strong().color(RED));}
 if let(Some(a),Some(b))=(x.low,y.low){ui.label(format!("1% low: {a:.1} -> {b:.1} FPS"));}caption(ui,"Actual game FPS only; generated output frames excluded.");}
 if !s.message.is_empty(){caption(ui,&s.message);}
 });ui.data_mut(|d|d.insert_temp(id,s));
}

fn audio_gain_panel(a:&mut BoostApp,ui:&mut Ui,path:&std::path::Path){
 egui::CollapsingHeader::new("Separate audio track levels / export").show(ui,|ui|{
 let id=egui::Id::new("clip-track-gain");let mut v=ui.data(|d|d.get_temp::<([f32;2],bool)>(id)).unwrap_or(([1.,1.],false));
 ui.add(egui::Slider::new(&mut v.0[0],0.0..=2.).text("Track 1 gain"));ui.add(egui::Slider::new(&mut v.0[1],0.0..=2.).text("Track 2 gain"));ui.checkbox(&mut v.1,"Mute track 2");
 caption(ui,"Track order depends on the source. Check which is microphone; this is post-processing, not live mic control. Video copied unchanged.");
 if ui.add_enabled(a.media_export.is_none(),egui::Button::new("Export new MKV with adjusted tracks")).clicked()&&!a.is_smoke{
 let src=path.to_owned();let out=crate::record::videos().join(format!("HBoost_Audio_{}.mkv",crate::media_tools::stamp()));let(tx,rx)=std::sync::mpsc::channel();a.media_export=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::media_tools::adjust_audio(&src,&out,v.0,v.1).map(|_|out));});
 }
 ui.data_mut(|d|d.insert_temp(id,v));
 });
}

#[derive(Clone,Default)]struct MontageScrub{at:f64,texture:Option<egui::TextureHandle>,pending:Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<image::RgbaImage,String>>>>>,label:String,error:String}
fn montage_scrub(a:&mut BoostApp,ui:&mut Ui,pieces:&[crate::montage::Piece]){
 let Ok(total)=crate::montage::total(pieces)else{return;};let id=egui::Id::new("montage-scrub-v117");let mut s=ui.data(|d|d.get_temp::<MontageScrub>(id)).unwrap_or_default();
 let result=s.pending.as_ref().and_then(|r|r.lock().ok().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Preview stopped".into())),_=>None}));
 if let Some(v)=result{s.pending=None;match v{Ok(im)=>{s.texture=Some(ui.ctx().load_texture("montage-scrub",egui::ColorImage::from_rgba_unmultiplied([im.width()as usize,im.height()as usize],im.as_raw()),egui::TextureOptions::LINEAR));s.error.clear();},Err(e)=>s.error=e}}
 s.at=s.at.clamp(0.,total);ui.add(egui::Slider::new(&mut s.at,0.0..=total).suffix(" s").text("Montage playhead"));
 if let Some((i,t))=crate::montage::locate(pieces,s.at){caption(ui,&format!("Range {} / source {:.2}s / {}",i+1,t,pieces[i].caption));
 let idle=!a.recorder.busy()&&!a.replay.busy()&&a.media_export.is_none()&&s.pending.is_none();
 if ui.add_enabled(idle,egui::Button::new("Decode preview at playhead")).clicked()&&!a.is_smoke{let path=pieces[i].path.clone();let(tx,rx)=std::sync::mpsc::channel();s.pending=Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));s.label=format!("Range {} at {:.2}s",i+1,t);s.texture=None;std::thread::spawn(move||{let _=tx.send(crate::media_tools::still(&path,t));});}}
 if s.pending.is_some(){ui.spinner();ui.ctx().request_repaint_after(std::time::Duration::from_millis(120));}
 if let Some(t)=&s.texture{caption(ui,&s.label);let z=(ui.available_width()/t.size()[0]as f32).min(1.);ui.add(egui::Image::new(t).fit_to_exact_size(t.size_vec2()*z));}
 if !s.error.is_empty(){caption(ui,&s.error);}caption(ui,"Manual exact-range still preview; not continuous video playback.");ui.data_mut(|d|d.insert_temp(id,s));
}

fn release_idle_media_cache(a:&BoostApp,ctx:&egui::Context){
 if a.page==Page::Capture{return;}
 if !a.settings.pressure_quiet{return;}
 let id=egui::Id::new("hb-clip-thumbs");
 ctx.data_mut(|d|{if let Some(mut s)=d.get_temp::<GalleryThumbs>(id){if !s.textures.is_empty(){s.textures.clear();d.insert_temp(id,s);}}});
}
