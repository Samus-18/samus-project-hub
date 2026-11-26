fn color_photo_lab(app:&mut BoostApp,ui:&mut Ui){
 let mut tuning=app.settings.lighting.clone();let mut changed=false;
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"COLOR LAB / NATIVE GPU GRADE");
  caption(ui,"Live SDR grading of Horizon's own captured output. Separate from Light Studio. No HDR claim, game injection or display calibration.");
  changed|=ui.checkbox(&mut tuning.grade.enabled,"Enable color grading").changed();
  changed|=ui.checkbox(&mut tuning.grade.linear,"Linear-light exposure and color operations").changed();
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Neutral"){tuning.grade=crate::color_grade::Grade{enabled:true,..Default::default()};changed=true;}
   if outline_button(ui,"Warm film"){tuning.grade=crate::color_grade::Grade{enabled:true,temperature:0.35,contrast:1.09,saturation:0.88,gamma:1.12,..Default::default()};changed=true;}
   if outline_button(ui,"Cool clarity"){tuning.grade=crate::color_grade::Grade{enabled:true,temperature:-0.3,contrast:1.12,saturation:1.08,exposure:0.12,..Default::default()};changed=true;}
   if outline_button(ui,"Reset / off"){tuning.grade=Default::default();changed=true;}
  });
  ui.add_enabled_ui(tuning.grade.enabled,|ui|{
   let g=&mut tuning.grade;
   egui::CollapsingHeader::new("Exposure / lift / gamma / gain").default_open(true).show(ui,|ui|{
    changed|=ui.add(egui::Slider::new(&mut g.exposure,-2.0..=2.0).text("Exposure EV")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.lift,-0.15..=0.15).text("Lift")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.gamma,0.5..=2.0).text("Gamma")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.gain,0.5..=1.5).text("Gain")).changed();
   });
   egui::CollapsingHeader::new("White balance / contrast / saturation").show(ui,|ui|{
    changed|=ui.add(egui::Slider::new(&mut g.temperature,-1.0..=1.0).text("Temperature")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.tint,-1.0..=1.0).text("Tint")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.contrast,0.5..=1.5).text("Contrast")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.saturation,0.0..=2.0).text("Saturation")).changed();
   });
   egui::CollapsingHeader::new("RGB / luma midpoint curves").show(ui,|ui|{
    changed|=ui.add(egui::Slider::new(&mut g.red,-0.25..=0.25).text("Red")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.green,-0.25..=0.25).text("Green")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.blue,-0.25..=0.25).text("Blue")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.luma,-0.25..=0.25).text("Luma")).changed();
    caption(ui,"Endpoint-preserving midpoint curves, not an unrestricted multi-point curve editor.");
   });
   egui::CollapsingHeader::new("Selective hue").show(ui,|ui|{
    changed|=ui.add(egui::Slider::new(&mut g.hue,0.0..=1.0).text("Hue center")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.hue_width,0.03..=0.5).text("Range")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.hue_shift,-0.5..=0.5).text("Hue shift")).changed();
    changed|=ui.add(egui::Slider::new(&mut g.selective_sat,0.0..=2.0).text("Selected saturation")).changed();
   });
   changed|=ui.add(egui::Slider::new(&mut g.mix,0.0..=1.0).text("Overall mix")).changed();
  });
  egui::CollapsingHeader::new("Five-point RGB / luminance curves").show(ui,|ui|{
   ui.horizontal_wrapped(|ui|{for (i,n) in ["Neutral luma","S-curve","Soft film","Lift shadows"].iter().enumerate(){if outline_button(ui,n){tuning.grade.curve_preset(i as u8);changed=true;}}});
   for (i,name) in ["Red","Green","Blue","Luminance"].iter().enumerate(){ui.push_id(i,|ui|{
    ui.label(*name);
    let curve=&mut tuning.grade.curves[i];
    let (r,response)=ui.allocate_exact_size(egui::vec2(ui.available_width().min(310.),92.),egui::Sense::click_and_drag());
    ui.painter().rect_filled(r,4.,BG);
    if (response.dragged()||response.clicked())&&let Some(p)=response.interact_pointer_pos(){
     let k=(((p.x-r.left())/r.width()).clamp(0.,1.)*4.).round() as usize;
     curve[k]=(1.-(p.y-r.top())/r.height()).clamp(0.,1.);changed=true;
    }
    let color=[Color32::from_rgb(255,105,110),V25_MINT,Color32::from_rgb(106,158,255),WHITE][i];
    let points=curve.iter().enumerate().map(|(j,v)|egui::pos2(r.left()+r.width()*j as f32/4.,r.bottom()-r.height()*v)).collect::<Vec<_>>();
    ui.painter().add(egui::Shape::line(points.clone(),Stroke::new(1.7_f32,color)));
    for p in points{ui.painter().circle_filled(p,3.,color);}
    if outline_button(ui,"Reset curve"){*curve=[0.,0.25,0.5,0.75,1.];changed=true;}
   });}
   caption(ui,"Drag nearest control point. Five bounded knots, piecewise-linear SDR mapping; active when color grading is enabled.");
  });
  egui::CollapsingHeader::new("Luminance gradient / live GPU").show(ui,|ui|{
   let g=&mut tuning.grade;
   for (i,label) in ["Shadows","Midtones","Highlights"].iter().enumerate(){
    ui.horizontal(|ui|{ui.label(*label);changed|=ui.color_edit_button_rgb(&mut g.gradient[i]).changed();});
   }
   changed|=ui.add(egui::Slider::new(&mut g.gradient_mid,0.05..=0.95).text("Midtone position")).changed();
   changed|=ui.add(egui::Slider::new(&mut g.gradient_mix,0.0..=1.0).text("Gradient strength")).changed();
   let (r,_)=ui.allocate_exact_size(egui::vec2(ui.available_width().min(400.),22.),egui::Sense::hover());
   for i in 0..64{let t=(i as f32+0.5)/64.;let (a,b,k)=if t<=g.gradient_mid{(g.gradient[0],g.gradient[1],t/g.gradient_mid)}else{(g.gradient[1],g.gradient[2],(t-g.gradient_mid)/(1.-g.gradient_mid))};let rgb=std::array::from_fn::<f32,3,_>(|j|a[j]+(b[j]-a[j])*k);let c=Color32::from_rgb((rgb[0]*255.) as u8,(rgb[1]*255.) as u8,(rgb[2]*255.) as u8);
    ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(r.left()+r.width()*i as f32/64.,r.top()),egui::pos2(r.left()+r.width()*(i+1) as f32/64.,r.bottom())),0.,c);
   }
   if outline_button(ui,"Reset gradient"){let d=crate::color_grade::Grade::default();g.gradient=d.gradient;g.gradient_mix=0.;g.gradient_mid=d.gradient_mid;changed=true;}
   caption(ui,"Three color stops indexed by SDR output luminance. Enable color grading to apply. Zero strength bypasses gradient.");
  });
  live_color_controls(app,ui,&mut tuning,&mut changed);
  caption(ui,"Current grade is saved per game and sent to the real GPU constant buffer without a renderer restart.");
  v22_native_toggles(app,ui);
 });
 if changed{app.apply_native_lighting(tuning);}
 ui.add_space(8.);
 photo_reference_panel(app,ui);
}
fn photo_reference_panel(app:&mut BoostApp,ui:&mut Ui){
 if let Some(result)=app.photo_rx.as_ref().and_then(|rx|match rx.try_recv(){
  Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Reference worker stopped".into())),_=>None
 }){
  app.photo_rx=None;
  match result{
   Ok(v)=>{
    let texture=|name:&str,img:&image::RgbaImage|ui.ctx().load_texture(name,
      egui::ColorImage::from_rgba_unmultiplied([img.width() as usize,img.height() as usize],img.as_raw()),egui::TextureOptions::LINEAR);
    app.photo_textures=Some((texture("photo-reference-before",&v.before),texture("photo-reference-after",&v.after)));
    app.photo_result=Some(v);
   },Err(e)=>app.toast(e)
  }
 }
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"REFERENCE PHOTO / REAL GPU A/B");
  caption(ui,"Explicit local image only. Runs the selected native stack and current grade on hardware D3D11. No game or desktop screenshot. New results require Render reference.");
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Choose local PNG/JPEG")&&!app.is_smoke{if let Some(p)=crate::filedialog::select_image(){app.photo_path=Some(p);}}
   if outline_button(ui,"Bundled reference"){app.photo_path=None;}
  });
  caption(ui,app.photo_path.as_ref().and_then(|p|p.file_name()).and_then(|s|s.to_str()).unwrap_or("Bundled authentic gameplay reference"));
  egui::CollapsingHeader::new("Optional .cube LUT / offline only").show(ui,|ui|{
   caption(ui,"Paste a standard 3D SDR .cube (size 2–33, normalized input/output). GPU trilinear interpolation; not applied to live output.");
   ui.add(egui::TextEdit::multiline(&mut app.photo_lut_text).desired_rows(4).desired_width(ui.available_width()).char_limit(4*1024*1024));
   if outline_button(ui,"Load .cube file")&&!app.is_smoke{
    if let Some(p)=crate::filedialog::select_cube(){
     match std::fs::metadata(&p).map_err(|e|e.to_string()).and_then(|m|if m.len()>4*1024*1024{Err("Cube exceeds 4 MiB".into())}else{std::fs::read_to_string(&p).map_err(|e|e.to_string())}){
      Ok(v)=>app.photo_lut_text=v,Err(e)=>app.toast(e)
     }
    }
   }
   if outline_button(ui,"Clear LUT"){app.photo_lut_text.clear();}
  });
  ui.checkbox(&mut app.photo_benchmark,"Compare 10 upscalers: PSNR + GPU time");
  let idle=app.photo_rx.is_none()&&app.native_mirror.is_none()&&app.effect_bench_rx.is_none()&&app.pipeline_bench_rx.is_none()&&app.motion_lab_rx.is_none();
  if ui.add_enabled(idle,egui::Button::new("Render reference")).clicked()&&!app.is_smoke{
   let lut=if app.photo_lut_text.trim().is_empty(){Ok(None)}else{crate::cube_lut::Cube::parse(&app.photo_lut_text).map(Some)};
   match lut{
    Err(e)=>app.toast(e),Ok(lut)=>{
     ui.data_mut(|d|d.remove::<String>(egui::Id::new("oled-light-reference-sig")));
     let path=app.photo_path.clone();let stages=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());
     let lighting=app.settings.lighting.clone();let benchmark=app.photo_benchmark;
     let(tx,rx)=std::sync::mpsc::channel();app.photo_rx=Some(rx);app.photo_result=None;app.photo_textures=None;
     std::thread::spawn(move||{let _=tx.send(crate::photo_lab::render(path.as_deref(),&stages,&lighting,lut.as_ref(),benchmark));});
    }
   }
  }
  if app.photo_rx.is_some(){caption(ui,"Processing reference on GPU...");ui.spinner();}
  if let (Some(v),Some((a,b)))=(&app.photo_result,&app.photo_textures){
   let width=ui.available_width().max(100.);let height=width*v.before.height() as f32/v.before.width() as f32;
   let (r,response)=ui.allocate_exact_size(egui::vec2(width,height),egui::Sense::click_and_drag());
   if (response.dragged()||response.clicked())&&let Some(pos)=response.interact_pointer_pos(){app.photo_split=((pos.x-r.left())/r.width()).clamp(0.,1.);}
   let full=egui::Rect::from_min_max(egui::pos2(0.,0.),egui::pos2(1.,1.));
   ui.painter().image(a.id(),r,full,WHITE);
   let split=r.left()+r.width()*app.photo_split;
   let clip=egui::Rect::from_min_max(egui::pos2(split,r.top()),r.right_bottom());
   ui.painter().with_clip_rect(clip).image(b.id(),r,full,WHITE);
   ui.painter().line_segment([egui::pos2(split,r.top()),egui::pos2(split,r.bottom())],Stroke::new(2.0_f32,RED));
   ui.horizontal(|ui|{caption(ui,"BEFORE");ui.add(egui::Slider::new(&mut app.photo_split,0.0..=1.0).show_value(false));caption(ui,"AFTER");});
   caption(ui,&format!("{} pixels changed / mean RGB absolute change {:.2}/255 / {}x{}",v.changed,v.mean_delta,v.before.width(),v.before.height()));
   let (hist,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),60.),egui::Sense::hover());
   let before=crate::photo_lab::histogram(&v.before);
   let max=v.histogram.iter().chain(before.iter()).copied().max().unwrap_or(1).max(1) as f32;
   for(i,n)in v.histogram.iter().enumerate(){
    let w=hist.width()/32.;let h=hist.height()*(*n as f32/max);
    let bar=egui::Rect::from_min_max(egui::pos2(hist.left()+i as f32*w,hist.bottom()-h),egui::pos2(hist.left()+(i+1) as f32*w-1.,hist.bottom()));
    ui.painter().rect_filled(bar,1.,Color32::from_rgb(190,95,125));
    let old=hist.bottom()-hist.height()*before[i] as f32/max;
    ui.painter().line_segment([egui::pos2(bar.left(),old),egui::pos2(bar.right(),old)],Stroke::new(1.5_f32,V25_MINT));
   }
   caption(ui,"Measured reference luminance distribution (32 bins), green = source, red = processed.");
   for s in &v.scales{ui.horizontal_wrapped(|ui|{
    ui.label(&s.name);caption(ui,&format!("PSNR {} / GPU {}",s.psnr.map(|x|format!("{x:.2} dB")).unwrap_or("Exact".into()),s.gpu_ms.map(|x|format!("{x:.3} ms")).unwrap_or("Unavailable".into())));
   });}
   if !v.scales.is_empty(){caption(ui,"PSNR measures reconstruction error against this reference only, not perceived quality or gameplay FPS.");}
  }
 });
}

fn live_color_controls(app:&mut BoostApp,ui:&mut Ui,tuning:&mut crate::lighting::LightTuning,changed:&mut bool){
 egui::CollapsingHeader::new("Custom GPU color recipe / safe hot update").show(ui,|ui|{
  caption(ui,"Compile a bounded JSON recipe into a single HLSL GPU pass. 1-8 operations: exposure, contrast, saturation, gamma, tint, invert, shoulder. No raw HLSL, file access, loops or extra resources.");
  let id=egui::Id::new(("recipe-draft-v62",app.profile_key()));
  let mut raw=ui.data_mut(|d|d.get_temp::<String>(id)).unwrap_or_else(||serde_json::to_string_pretty(&tuning.recipe).unwrap_or_default());
  ui.add(egui::TextEdit::multiline(&mut raw).desired_rows(9).char_limit(4096).desired_width(ui.available_width()));
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Load active recipe"){raw=serde_json::to_string_pretty(&tuning.recipe).unwrap_or_default();}
   if outline_button(ui,"Example"){raw=serde_json::to_string_pretty(&crate::color_recipe::Recipe{enabled:true,..Default::default()}).unwrap_or_default();}
   if outline_button(ui,"Validate, compile and apply"){
    match crate::color_recipe::Recipe::decode(&raw).and_then(|r|{crate::shadercheck::compile_native_hlsl(&r.source()?)?;Ok(r)}){
     Ok(r)=>{tuning.recipe=r;*changed=true;app.toast("Recipe compiled; GPU update on next captured frame");},Err(e)=>app.toast(e)
    }
   }
   if outline_button(ui,"Disable recipe"){tuning.recipe.enabled=false;*changed=true;}
  });
  ui.data_mut(|d|d.insert_temp(id,raw));
  caption(ui,if tuning.recipe.active(){"Active recipe enabled; editing draft does not change GPU until Apply."}else{"Recipe disabled."});
 });
 egui::CollapsingHeader::new("High-precision linear finish / SDR").show(ui,|ui|{
  let o=&mut tuning.precision;
  *changed|=ui.checkbox(&mut o.enabled,"Enable optional FP16 finish (FP32 fallback)").changed();
  *changed|=ui.checkbox(&mut o.force_fp32,"Force FP32 temporary texture").changed();
  *changed|=ui.add(egui::Slider::new(&mut o.exposure,-2.0..=2.).text("Finish exposure EV")).changed();
  *changed|=ui.add(egui::Slider::new(&mut o.shoulder,0.0..=1.).text("Highlight shoulder")).changed();
  *changed|=ui.add(egui::Slider::new(&mut o.contrast,0.5..=1.5).text("Linear contrast")).changed();
  *changed|=ui.add(egui::Slider::new(&mut o.saturation,0.0..=2.).text("Linear saturation")).changed();
  *changed|=ui.add(egui::Slider::new(&mut o.mix,0.0..=1.).text("Finish mix")).changed();
  *changed|=ui.checkbox(&mut o.quant10,"Emulate 10-bit quantization before SDR output").changed();
  *changed|=ui.add(egui::Slider::new(&mut o.dither,0.0..=1.).text("Ordered output dither")).changed();
  let id=egui::Id::new("precision-preset-json");let mut raw=ui.data_mut(|d|d.get_temp::<String>(id)).unwrap_or_default();
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Export finish preset"){raw=serde_json::to_string_pretty(o).unwrap_or_default();}
   if outline_button(ui,"Validate / import"){match crate::precision_pass::Options::decode(&raw){Ok(v)=>{*o=v;*changed=true;},Err(e)=>app.toast(e)}}
   if outline_button(ui,"Reset finish"){*o=Default::default();*changed=true;}
  });
  ui.add(egui::TextEdit::multiline(&mut raw).char_limit(4096).desired_rows(3).desired_width(ui.available_width()));ui.data_mut(|d|d.insert_temp(id,raw));
  caption(ui,"Two GPU passes, one bounded floating point temporary texture. Final display remains RGBA8 SDR, no true HDR/10-bit monitor output or unrestricted HLSL. Extra VRAM; disabled by default.");
 });
 egui::CollapsingHeader::new("Live 3D LUT / cached GPU pass").show(ui,|ui|{
  caption(ui,"Normalized 3D .cube or horizontal PNG strip N*N by N (N=2–33, red-fastest). One FP32 lookup texture per selected LUT. External SDR output only.");
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Import .cube")&&!app.is_smoke{if let Some(p)=crate::filedialog::select_cube(){match crate::live_lut::import(&p){Ok(id)=>{tuning.lut.name=Some(id);*changed=true;},Err(e)=>app.toast(e)}}}
   if outline_button(ui,"Import PNG strip")&&!app.is_smoke{if let Some(p)=crate::filedialog::select_image(){match crate::live_lut::import(&p){Ok(id)=>{tuning.lut.name=Some(id);*changed=true;},Err(e)=>app.toast(e)}}}
   if outline_button(ui,"Disable LUT"){tuning.lut.name=None;*changed=true;}
  });
  egui::ComboBox::from_id_salt("live-lut-list").selected_text(tuning.lut.name.as_deref().unwrap_or("Off")).show_ui(ui,|ui|{
   for n in crate::live_lut::list(){if ui.selectable_label(tuning.lut.name.as_ref()==Some(&n),&n).clicked(){tuning.lut.name=Some(n);*changed=true;}}
  });
  *changed|=ui.add(egui::Slider::new(&mut tuning.lut.strength,0.0..=1.).text("LUT strength")).changed();
  caption(ui,"Local LUT files are not embedded in portable profile JSON. Missing LUT references are cleared on import. Identity and inverse are diagnostic presets.");
 });
 egui::CollapsingHeader::new("Temporal histogram exposure").show(ui,|ui|{
  *changed|=ui.checkbox(&mut tuning.exposure.enabled,"Apply histogram exposure").changed();
  *changed|=ui.checkbox(&mut tuning.exposure.temporal,"Smooth exposure over time").changed();
  *changed|=ui.add(egui::Slider::new(&mut tuning.exposure.seconds,0.05..=3.).suffix(" s").text("Adaptation time")).changed();
  *changed|=ui.add(egui::Slider::new(&mut tuning.exposure.reset_delta,0.1..=0.75).text("Abrupt gain reset")).changed();
  caption(ui,"GPU scalar history resets after an idle gap or large exposure-gain change. Not semantic scene-cut detection, not HDR.");
 });
}
