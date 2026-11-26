fn motion_quality_panel(app:&mut BoostApp,ui:&mut Ui){
 let mut tuning=app.settings.lighting.clone();let mut changed=false;
 frame(PANEL).show(ui,|ui|{
  heading(ui,"MOTION QUALITY / GPU");
  caption(ui,"Live options affect only experimental separate x2/x3 output. High-motion pixels fall back to the current source; not a guarantee of low latency or artifact-free gameplay.");
  caption(ui,"Automatic field detail: 16px at up to 1080p, 32px above 1080p. Coarser field saves estimate work at the cost of motion detail; not a game FPS claim.");
  let o=&mut tuning.motion;
  changed|=ui.add(egui::Slider::new(&mut o.edge_protection,0.0..=1.0).text("Still-edge protection")).changed();
  changed|=ui.add(egui::Slider::new(&mut o.consistency,0.5..=2.0).text("Mismatch rejection")).changed();
changed|=ui.add(egui::Slider::new(&mut o.disocclusion_guard,0.0..=1.0).text("Occlusion guard")).changed();
caption(ui,"Fills missing external FG regions from the latest real frame when forward/backward flows disagree. Color-only estimate, not game depth.");
  changed|=ui.add(egui::Slider::new(&mut o.max_motion,1.0..=12.0).suffix(" px").text("Motion limit")).changed();
  changed|=ui.checkbox(&mut o.smooth_flow,"Interpolate neighboring motion tiles").changed();
  changed|=ui.checkbox(&mut o.pyramid_flow,"Opt-in two-scale GPU flow (more quality/GPU cost; preview restart)").changed();
  changed|=ui.checkbox(&mut o.adaptive_fg,"Allow 3x to fall back to 2x on measured cadence").changed();
  changed|=ui.add(egui::Slider::new(&mut o.max_fg_age_ms,25.0..=250.0).suffix(" ms").text("Maximum source age for FG")).changed();
  caption(ui,"Missing or stale source age bypasses synthesis. Real output remains.");
  changed|=ui.checkbox(&mut o.mask,"Protect rectangular HUD region").changed();
  if o.mask{
   changed|=ui.add(egui::Slider::new(&mut o.left,0.0..=0.95).text("Left")).changed();
   changed|=ui.add(egui::Slider::new(&mut o.top,0.0..=0.95).text("Top")).changed();
   changed|=ui.add(egui::Slider::new(&mut o.right,(o.left+0.05)..=1.0).text("Right")).changed();
   changed|=ui.add(egui::Slider::new(&mut o.bottom,(o.top+0.05)..=1.0).text("Bottom")).changed();
   let (rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width().min(280.),95.),egui::Sense::hover());
   let _=response;ui.painter().rect_filled(rect,5.,BG);
   let m=egui::Rect::from_min_max(rect.min+egui::vec2(rect.width()*o.left,rect.height()*o.top),rect.min+egui::vec2(rect.width()*o.right,rect.height()*o.bottom));
   ui.painter().rect_filled(m,2.,Color32::from_rgb(44,75,130));
   ui.painter().rect_stroke(m,2.,Stroke::new(1.0_f32,WHITE),StrokeKind::Inside);
  }
  ui.separator();
  heading(ui,"TEMPORAL EFFECTS / 1X PREVIEW");
  caption(ui,"Optional, off by default. Real optical-flow reprojected history or directional motion blur on own separate 1x output. Not engine TAA; no generated frames. Bypassed for split, glass, x2/x3 and stale history.");
  egui::ComboBox::from_id_salt("temporal-effect").selected_text(["Off","History stabilization","Motion-direction blur"][o.temporal_mode.min(2) as usize]).show_ui(ui,|ui|{
   for (i,n) in ["Off","History stabilization","Motion-direction blur"].iter().enumerate(){changed|=ui.selectable_value(&mut o.temporal_mode,i as u8,*n).changed();}
  });
  if o.temporal_mode>0{changed|=ui.add(egui::Slider::new(&mut o.temporal_strength,0.0..=0.65).text("Temporal strength")).changed();}
  if outline_button(ui,"Reset motion options"){*o=Default::default();changed=true;}
  caption(ui,"Settings are per game. Rectangular mask copies real current pixels, not semantic recognition.");
 });
 if changed{let old=app.settings.lighting.motion.pyramid_flow;app.apply_native_lighting(tuning);if old!=app.settings.lighting.motion.pyramid_flow{app.refresh_native_mirror();}}
 if let Some(result)=app.motion_lab_rx.as_ref().and_then(|rx|match rx.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Motion lab worker stopped".into())),_=>None}){
  app.motion_lab_rx=None;
  match result{Ok(r)=>{
   app.motion_lab_times=Some((r.estimate_ms,r.warp_ms));
   app.motion_lab_textures=r.frames.iter().enumerate().map(|(i,img)|ui.ctx().load_texture(format!("motion-lab-{i}"),
    egui::ColorImage::from_rgba_unmultiplied([img.width() as usize,img.height() as usize],img.as_raw()),egui::TextureOptions::LINEAR)).collect();
  },Err(e)=>app.toast(e)}
 }
 frame(CARD).show(ui,|ui|{
  heading(ui,"OFFLINE MOTION LAB / REAL GPU");
  caption(ui,"Synthetic moving texture and fixed HUD; no desktop/game capture. Red rejection, green acceptance, blue protected mask; not ground-truth occlusion. Vector map encodes signed forward XY and disagreement.");
  let idle=app.motion_lab_rx.is_none()&&app.native_mirror.is_none()&&app.photo_rx.is_none()&&app.effect_bench_rx.is_none()&&app.pipeline_bench_rx.is_none();
  if ui.add_enabled(idle,egui::Button::new("Run synthetic motion test")).clicked()&&!app.is_smoke{
   let o=app.settings.lighting.motion.clone();let(tx,rx)=std::sync::mpsc::channel();app.motion_lab_rx=Some(rx);app.motion_lab_textures.clear();app.motion_lab_times=None;
   std::thread::spawn(move||{let _=tx.send(crate::motion_lab::render(&o));});
  }
  if app.motion_lab_rx.is_some(){ui.spinner();}
  if let Some((a,b))=app.motion_lab_times{
   let fmt=|v:Option<f64>|v.map(|v|format!("{v:.4} ms")).unwrap_or("Unavailable".into());
   caption(ui,&format!("320x180 GPU estimate {} / warp {}",fmt(a),fmt(b)));
  }
  ui.horizontal_wrapped(|ui|{
   for (i,t) in app.motion_lab_textures.iter().enumerate(){
    ui.vertical(|ui|{caption(ui,["Previous","Current","Interpolated","Rejection map","Motion vectors"][i.min(4)]);
     ui.add(egui::Image::new(t).fit_to_exact_size(egui::vec2(150.,84.375)));});
   }
  });
 });
}
