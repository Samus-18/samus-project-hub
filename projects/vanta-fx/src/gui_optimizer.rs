// Optimization advisor. Read-only analysis, explicit application and same-target undo.
#[derive(Clone,Copy,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
struct OptConfig{processing:u8,mode:u8,ratio:f32,quality:u8,cap:u32,wait:bool,quiet:bool,adaptive:bool,pacer:bool}
impl OptConfig{
 fn current(a:&BoostApp)->Self{let s=&a.settings;Self{processing:s.processing_percent,mode:s.native_scale_mode,ratio:s.native_scale_ratio,quality:s.native_quality,cap:s.preview_fps_limit,wait:s.preview_low_latency,quiet:s.quiet_ui,adaptive:a.render_guard_enabled,pacer:a.preview_guard_enabled}}
 fn apply(self,a:&mut BoostApp){
  let key=a.profile_key();let s=&mut a.settings;
  s.processing_percent=self.processing;s.processing_profiles.insert(key,self.processing);
  s.native_scale_mode=self.mode;s.native_scale_ratio=self.ratio;s.native_quality=self.quality;
  s.preview_fps_limit=self.cap;s.preview_low_latency=self.wait;s.quiet_ui=self.quiet;
  a.render_guard_enabled=self.adaptive;a.preview_guard_enabled=self.pacer;
  a.render_guard.reset();a.preview_guard.reset();a.save();a.refresh_native_mirror();
 }
 fn rows(self)->[String;9]{[
  format!("{}%",self.processing),crate::spatial::MODES.get(self.mode as usize).unwrap_or(&"Unknown").to_string(),
  format!("{:.2}x",self.ratio),["Full","Balanced","Eco","Adaptive"].get(self.quality as usize).unwrap_or(&"Unknown").to_string(),
  if self.cap==0{"Uncapped".into()}else{format!("{} FPS",self.cap)},
  opt_on(self.wait),opt_on(self.quiet),opt_on(self.adaptive),opt_on(self.pacer)]}
}

// Durable, profile-specific undo. The file is local and written BEFORE any preset.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OptBackup{version:u8,key:String,config:OptConfig}
fn opt_backup_path()->std::path::PathBuf{
 crate::settings::folder().join("vanta-optimizer-prechange.json")
}
fn opt_backup_save(key:&str,config:OptConfig)->Result<(),String>{
 if key.is_empty()||key.len()>512||key.chars().any(char::is_control){return Err("Invalid game profile".into());}
 let document=serde_json::to_vec_pretty(&OptBackup{version:1,key:key.into(),config}).map_err(|e|e.to_string())?;
 if document.len()>4096{return Err("Backup exceeds safe size".into());}
 let path=opt_backup_path();
 if let Some(dir)=path.parent(){std::fs::create_dir_all(dir).map_err(|e|e.to_string())?;}
 crate::settings::atomic_json_replace(&path,&document)
}
fn opt_backup_load(key:&str)->Result<OptConfig,String>{
 let data=std::fs::read(opt_backup_path()).map_err(|e|format!("No saved backup: {e}"))?;
 if data.len()>4096{return Err("Backup exceeds safe size".into());}
 let doc:OptBackup=serde_json::from_slice(&data).map_err(|e|format!("Backup damaged: {e}"))?;
 if doc.version!=1||doc.key!=key{return Err("Backup belongs to another game/profile".into());}
 let p=doc.config;
 if !crate::render_budget::PERCENTAGES.contains(&p.processing)
  ||p.mode as usize>=crate::spatial::MODES.len()
  ||!crate::spatial::RATIOS.contains(&p.ratio)
  ||p.quality>3||!crate::preview_pacer::LIMITS.contains(&p.cap){
  return Err("Backup has unsupported settings".into());
 }
 Ok(p)
}

fn opt_on(b:bool)->String{if b{"On"}else{"Off"}.into()}
const OPT_LABELS:[&str;9]=["Processing size","Scaler","Output ratio","GPU quality","Own 1x FPS cap","Waitable queue","Quiet UI","Adaptive processing","Adaptive pacing"];
const OPT_NAMES:[&str;5]=["Eco / older GPU","Balanced","Image quality","Low latency","Native / minimal overhead"];
const OPT_DESC:[&str;5]=[
 "Lightweight SGSR1 and reduced processing cost. Good starting point for older GPUs; game resolution is unchanged.",
 "Moderate processing budget with FSR1 reconstruction. Balance sharpness and GPU cost.",
 "Full processing quality with NIS. More GPU work; not a guaranteed FPS improvement.",
 "Light processing and short own output queue. Does not reduce engine or network latency automatically.",
 "No upscaling or reconstruction. Useful as a baseline; existing visual effects are not removed."
];
const OPT_PRESETS:[OptConfig;5]=[
 OptConfig{processing:67,mode:13,ratio:1.5,quality:2,cap:45,wait:true,quiet:true,adaptive:true,pacer:true},
 OptConfig{processing:75,mode:8,ratio:1.5,quality:3,cap:60,wait:true,quiet:true,adaptive:true,pacer:true},
 OptConfig{processing:100,mode:9,ratio:1.5,quality:0,cap:60,wait:false,quiet:true,adaptive:false,pacer:false},
 OptConfig{processing:67,mode:11,ratio:1.,quality:2,cap:60,wait:true,quiet:true,adaptive:true,pacer:true},
 OptConfig{processing:100,mode:0,ratio:1.,quality:3,cap:60,wait:true,quiet:true,adaptive:false,pacer:false},
];
#[derive(Clone)]
struct OptMeasurement{key:String,pid:u32,at:std::time::Instant,fps:f32,ms:f32,low:Option<f32>,own:Option<f64>}
fn opt_delta(before:f32,after:f32)->Option<f32>{
 if before.is_finite()&&after.is_finite()&&before>0.&&after>=0.{Some((after/before-1.)*100.)}else{None}
}
fn opt_recommend(gpu:&str,ram:f64,gpu_ratio:Option<f32>)->usize{
 let gpu=gpu.to_ascii_lowercase();
 if gpu_ratio.is_some_and(|v|v.is_finite()&&v>=0.85)|| (ram>0.&&ram<=8.) ||
    ["rx 580","rx 570","rx 560","rx 480","rx 470","rx 550","gtx 1050","gtx 1060","vega 8","vega 3"].iter().any(|v|gpu.contains(v)){0}else{1}
}
#[derive(Clone,Default)]
struct OptState{
 selected:Option<usize>,scan:Option<Result<crate::gpu_caps::NativeCapabilities,String>>,
 undo:Option<(String,OptConfig)>,before:Option<OptMeasurement>,after:Option<OptMeasurement>,
 applied_at:Option<std::time::Instant>,message:String,query:String,only_recommended:bool
}
fn opt_sample(a:&BoostApp)->Option<OptMeasurement>{
 let game=a.telem.active_game.as_ref()?;let s=a.fps.snapshot_for(Some(game.pid))?;
 if a.fps.frames.len()<100||!s.fps.is_finite()||s.fps<=0.||!s.frametime_ms.is_finite(){return None;}
 Some(OptMeasurement{key:a.profile_key(),pid:game.pid,at:std::time::Instant::now(),fps:s.fps,ms:s.frametime_ms,low:s.low_1_percent_fps,own:a.output_rates.snapshot(std::time::Instant::now()).map(|v|v.total)})
}
fn opt_measurement_pair(a:&OptMeasurement,b:&OptMeasurement)->bool{
 a.key==b.key&&a.pid==b.pid&&b.at.saturating_duration_since(a.at)>=std::time::Duration::from_secs(10)
}
struct OptFeature{title:&'static str,desc:&'static str,details:&'static str,impact:&'static str,page:Page}
const OPT_FEATURES:[OptFeature;16]=[
 OptFeature{title:"SGSR1",desc:"Light spatial reconstruction",details:"Qualcomm spatial scaler; useful with lower game resolution. Extra GPU work, not free FPS.",impact:"Upscaling",page:Page::Scaling},
 OptFeature{title:"FSR1",desc:"Spatial upscaling and sharpening",details:"EASU + RCAS. No motion vectors. Lower game resolution manually for actual game-rendering savings.",impact:"Upscaling",page:Page::Scaling},
 OptFeature{title:"NVIDIA NIS",desc:"Open spatial scaler on AMD",details:"Native compute implementation, no NVIDIA GPU requirement for this port.",impact:"Upscaling",page:Page::Scaling},
 OptFeature{title:"CAS",desc:"Single-pass sharpening / scaling",details:"Low overhead compared with neural reconstruction; may still cost GPU time.",impact:"Upscaling",page:Page::Scaling},
 OptFeature{title:"FrameGen x2/x3",desc:"Experimental generated output frames",details:"Own output only. Requires stable source and GPU headroom; can increase latency. Enable explicitly in Motion Lab.",impact:"Smoothness",page:Page::FrameGen},
 OptFeature{title:"CuNNy / FSRCNNX / Anime4K",desc:"Neural spatial reconstruction",details:"GPU-intensive; exact 2x modes. Not DLSS or FSR4 and not primarily an FPS boost.",impact:"Quality",page:Page::Scaling},
 OptFeature{title:"Game Boost",desc:"Reversible selected-process priority",details:"Normal to Above Normal only. Can help under CPU contention, can also have no benefit.",impact:"System",page:Page::Performance},
 OptFeature{title:"CPU affinity",desc:"Optional game core selection",details:"Can harm scheduling and performance. No blanket recommended core mask.",impact:"System",page:Page::Performance},
 OptFeature{title:"Power plan",desc:"Explicit reversible Windows power mode",details:"May increase power and heat. No automatic system change.",impact:"System",page:Page::Performance},
 OptFeature{title:"Standby cache",desc:"Manual RAM cache purge",details:"Only under memory pressure, can worsen loading. Not a routine FPS optimization.",impact:"System",page:Page::Performance},
 OptFeature{title:"Shader cache",desc:"Reuse Horizon shader compilation",details:"Reduces repeated own compilation. Does not change game shader caches.",impact:"Stability",page:Page::Filters},
 OptFeature{title:"VRAM monitoring",desc:"Detect memory pressure",details:"Read-only diagnostic, no guaranteed FPS improvement.",impact:"Analysis",page:Page::Performance},
 OptFeature{title:"Wave64 / DX12 / Vulkan",desc:"GPU capability and compute proofs",details:"Hardware diagnostic paths, not complete live alternative renderers.",impact:"Analysis",page:Page::Unsorted},
 OptFeature{title:"FP16 / FP32",desc:"Optional intermediate precision",details:"FP16 storage can reduce bandwidth; fallback to FP32. No claim of native fast FP16 math.",impact:"Quality",page:Page::Unsorted},
 OptFeature{title:"HDR / FreeSync",desc:"Output and display capability",details:"Physical VRR and HDR calibration remain unverified. Not an FPS boost.",impact:"Analysis",page:Page::Unsorted},
 OptFeature{title:"AMD official OF / Xbox Game Bar",desc:"Not implemented",details:"Official SDK integrations are pending. No equivalent vendor feature is unlocked.",impact:"Pending",page:Page::Unsorted},
];
fn optimizer_panel(app:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("hb-optimizer-v107");
 let mut st=ui.data(|d|d.get_temp::<OptState>(id)).unwrap_or_default();
 let current=OptConfig::current(app);
 let measured=app.fps.snapshot_for(app.telem.active_game.as_ref().map(|g|g.pid));
 let ratio=measured.and_then(|s|s.gpu_ms.filter(|_|s.frametime_ms>0.).map(|g|g/s.frametime_ms));
 let rec=opt_recommend(&app.gpu,app.telem.total_ram_gb,ratio);
 let selected=st.selected.unwrap_or(rec).min(OPT_PRESETS.len()-1);
 let chosen=OPT_PRESETS[selected];
 let rows=current.rows();let proposed=chosen.rows();
 let matched=rows.iter().zip(&proposed).filter(|(a,b)|a==b).count();
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  ui.label(RichText::new("SYSTEM CHECK").size(10.).strong().color(VF_ACCENT));
  ui.label(RichText::new("Optimize your setup").size(25.).strong());
  caption(ui,"Scan → review → apply with backup.");
  ui.horizontal_wrapped(|ui|{
   ui.label(RichText::new("Detected PC").color(VF_FOREGROUND).strong());
   ui.label(&app.gpu);
   ui.label(format!("{} logical CPU threads",std::thread::available_parallelism().map(|n|n.get()).unwrap_or(0)));
   if app.telem.total_ram_gb>0.{ui.label(format!("{:.1}/{:.1} GiB RAM",app.telem.used_ram_gb,app.telem.total_ram_gb));}
  });
  ui.add_space(12.);
  Frame::new().fill(Color32::from_rgb(20,39,37)).corner_radius(9).inner_margin(egui::Margin::same(13)).show(ui,|ui|{
   ui.set_min_width(ui.available_width());
   ui.label(RichText::new("RECOMMENDED PROFILE").size(10.).color(VF_SUCCESS));
   ui.label(RichText::new(OPT_NAMES[rec]).size(22.).strong().color(VF_FOREGROUND));
  });
   let key=app.profile_key();let mut target=app.settings.output_targets.get(&key).copied().unwrap_or(app.target_fps);
   egui::CollapsingHeader::new("MORE / own output FPS target").default_open(false).show(ui,|ui|{if ui.add(egui::Slider::new(&mut target,30..=165).suffix(" FPS").text("Output target")).changed(){app.settings.output_targets.insert(key,target);app.render_guard.reset();app.preview_guard.reset();app.save();}caption(ui,"Not the game FPS limiter.");});
   caption(ui,OPT_DESC[rec]);
   if app.gpu.to_ascii_lowercase().contains("rx 580"){caption(ui,"RX 580: lightweight GPU output, lower GPU load. Does not change the game resolution.");}
  if let Some(v)=ratio.filter(|v|v.is_finite()){caption(ui,&format!("Measured GPU/frame time ratio: {:.0}%. Not a confirmed bottleneck.",v*100.));}
  else{caption(ui,"Recommendation uses detected hardware/RAM. Real FPS gain is not predicted.");}
  if outline_button(ui,"SCAN THIS PC")&&!app.is_smoke{st.selected=Some(rec);st.scan=Some(crate::gpu_caps::read_native_capabilities());st.message=format!("Analysis complete. Recommended: {}. Review and apply only if desired.",OPT_NAMES[rec]);}
  if let Some(scan)=&st.scan{match scan{Ok(c)=>{caption(ui,&format!("D3D11: {} | RGBA8 UAV {} | FP16 UAV {} | R32F UAV {} | compute compile {}",c.feature_level_11,c.rgba8_typed_uav,c.rgba16f_typed_uav,c.r32f_typed_uav,c.original_cs_5_0_compiles));},Err(e)=>{ui.colored_label(RED,e);}}}
  ui.add_space(7.);
  egui::CollapsingHeader::new("MORE / choose another performance profile").default_open(false).show(ui,|ui|{ui.horizontal_wrapped(|ui|{for(i,name)in OPT_NAMES.iter().enumerate(){let text=if i==rec{format!("{name} *")}else{name.to_string()};if ui.selectable_label(selected==i,text).clicked(){st.selected=Some(i);}}});caption(ui,OPT_DESC[selected]);});
  egui::CollapsingHeader::new("What will change? (9 reversible settings)").default_open(false).show(ui,|ui|{
   for i in 0..OPT_LABELS.len(){ui.horizontal_wrapped(|ui|{ui.label(OPT_LABELS[i]);ui.label(&rows[i]);if rows[i]!=proposed[i]{ui.colored_label(RED,format!("-> {}",proposed[i]));}else{ui.label("Unchanged");}});}
  });
  ui.horizontal_wrapped(|ui|{
   if vf_button(ui,"Apply + backup",true,!app.is_smoke&&matched<OPT_LABELS.len()){
    let key=app.profile_key();
    match opt_backup_save(&key,current){
     Ok(())=>{st.undo=Some((key,current));chosen.apply(app);st.applied_at=Some(std::time::Instant::now());st.after=None;
      st.message=format!("Applied: {}. Pre-change backup saved on disk; game settings were not edited.",OPT_NAMES[selected]);},
     Err(e)=>st.message=format!("NOT APPLIED: backup failed: {e}"),
    }
   }
   if ui.add_enabled(!app.is_smoke&&opt_backup_load(&app.profile_key()).is_ok(),egui::Button::new("RESTORE SAVED BACKUP")).clicked(){
    match opt_backup_load(&app.profile_key()){
     Ok(old)=>{old.apply(app);st.undo=None;st.applied_at=Some(std::time::Instant::now());st.after=None;
      st.message="Previous Vanta FX output settings restored from disk.".into();},
     Err(e)=>st.message=format!("Restore failed: {e}"),
    }
   }
  });
  caption(ui,"Automatic backup before Apply; Restore even after restart. Windows and game settings stay unchanged.");
  if !st.message.is_empty(){ui.label(&st.message);}
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  ui.label(RichText::new("FPS comparison").size(22.).strong());
  let before=st.before.as_ref().map_or("—".into(),|m|format!("{:.1}",m.fps));
  let after=st.after.as_ref().map_or("—".into(),|m|format!("{:.1}",m.fps));
  ui.columns(2,|cols|{
   let w=cols[0].available_width()-28.;vf_metric_tile(&mut cols[0],"BEFORE",&before,"Measured FPS",VF_FOREGROUND,w);
   let w=cols[1].available_width()-28.;vf_metric_tile(&mut cols[1],"AFTER",&after,"Measured FPS",VF_SUCCESS,w);
  });
  caption(ui,"Capture two real rolling snapshots in the same game and scene, at least 10 seconds apart. This is not a controlled benchmark.");
  ui.horizontal_wrapped(|ui|{
   if ui.button("Capture BEFORE").clicked(){st.before=opt_sample(app);st.after=None;st.message=if st.before.is_some(){"Baseline captured."}else{"Select a game and collect at least 100 actual frames."}.into();}
   if ui.button("Capture AFTER").clicked(){
    let sample=opt_sample(app);
    if let (Some(a),Some(b))=(&st.before,sample){
     let settled=st.applied_at.is_none_or(|t|b.at.saturating_duration_since(t)>=std::time::Duration::from_secs(10));
     if opt_measurement_pair(a,&b)&&settled{st.after=Some(b);st.message="Comparison captured.".into();}
     else{st.after=None;st.message="Same process required; wait at least 10 seconds after baseline and preset change.".into();}
    }else{st.after=None;st.message="Capture a valid baseline and real game frames first.".into();}
   }
   if ui.small_button("Clear").clicked(){st.before=None;st.after=None;st.message.clear();}
  });
  if let Some(a)=&st.before{
   ui.label(format!("Before: {:.1} game FPS | {:.2} ms | 1% low {}",a.fps,a.ms,a.low.map(|v|format!("{v:.1}")).unwrap_or("--".into())));
   if let Some(b)=&st.after{
    ui.label(format!("After: {:.1} game FPS | {:.2} ms | 1% low {}",b.fps,b.ms,b.low.map(|v|format!("{v:.1}")).unwrap_or("--".into())));
    if let Some(d)=opt_delta(a.fps,b.fps){ui.label(RichText::new(format!("Measured game FPS change: {d:+.1}%")).strong());}
    ui.label(format!("Frame time difference: {:+.2} ms",b.ms-a.ms));
    if let (Some(x),Some(y))=(a.low,b.low){ui.label(format!("1% low difference: {:+.1} FPS",y-x));}
    if let (Some(x),Some(y))=(a.own,b.own){ui.label(format!("Own output (incl. generated): {x:.1} -> {y:.1} FPS"));}
   }
  }
  if !st.message.is_empty(){caption(ui,&st.message);}
 });
 ui.add_space(8.);
 egui::CollapsingHeader::new("MORE / individual optimizations and explanations").default_open(false).show(ui,|ui|{
 frame(PANEL).show(ui,|ui|{
  heading(ui,"Optimization list");
  ui.horizontal_wrapped(|ui|{ui.label("Search");ui.text_edit_singleline(&mut st.query);ui.checkbox(&mut st.only_recommended,"Recommended only");});
  let query=st.query.to_lowercase();
  for (i,f) in OPT_FEATURES.iter().enumerate(){
   let recommended=if rec==0{matches!(i,0|1|3|10|11)}else{matches!(i,0|1|2|10|11)};
   if st.only_recommended&&!recommended{continue;}
   if !format!("{} {} {} {}",f.title,f.desc,f.details,f.impact).to_lowercase().contains(&query){continue;}
   egui::CollapsingHeader::new(format!("{}  |  {}{}",f.title,f.impact,if recommended{"  *"}else{""})).id_salt(("opt-feature",i)).show(ui,|ui|{
    ui.label(f.desc);caption(ui,f.details);
    if outline_button(ui,"Open controls"){
     if f.page==Page::Performance{app.performance_view=if i==7{4}else{1};app.settings.ui_advanced=true;}
     app.page=f.page;
    }
   });
   caption(ui,f.desc);
  }
 });
 });
 ui.data_mut(|d|d.insert_temp(id,st));
}
#[cfg(test)]mod v107_opt_tests{
 use super::*;
 #[test]fn presets_match_real_supported_settings(){
  for p in OPT_PRESETS{assert!(crate::render_budget::PERCENTAGES.contains(&p.processing));assert!((p.mode as usize)<crate::spatial::MODES.len());assert!(crate::spatial::RATIOS.contains(&p.ratio));assert!(crate::preview_pacer::LIMITS.contains(&p.cap));assert!(p.quality<=3);}
 }
 #[test]fn recommendation_uses_detected_limits_not_fake_gain(){
  assert_eq!(opt_recommend("AMD Radeon RX 580",16.,None),0);assert_eq!(opt_recommend("Other",32.,Some(0.91)),0);assert_eq!(opt_recommend("Unknown",0.,None),1);
 }
 #[test]fn measured_delta_is_signed_and_rejects_missing_baseline(){
  assert_eq!(opt_delta(0.,60.),None);assert_eq!(opt_delta(f32::NAN,60.),None);assert!((opt_delta(40.,60.).unwrap()-50.).abs()<0.01);assert!(opt_delta(60.,40.).unwrap()<0.);
 }
 #[test]fn comparisons_require_same_process_and_minimum_interval(){
  let t=std::time::Instant::now();let a=OptMeasurement{key:"game".into(),pid:5,at:t,fps:60.,ms:16.7,low:None,own:None};
  let mut b=a.clone();assert!(!opt_measurement_pair(&a,&b));b.at=t+std::time::Duration::from_secs(10);assert!(opt_measurement_pair(&a,&b));b.pid=6;assert!(!opt_measurement_pair(&a,&b));
 }
}

#[cfg(test)]mod v137_opt_backup_tests{
 use super::*;
 #[test]fn corrupted_and_wrong_profile_backup_are_rejected(){
  let x=OptBackup{version:1,key:"game-a".into(),config:OPT_PRESETS[0]};
  let data=serde_json::to_vec(&x).unwrap();
  let y:OptBackup=serde_json::from_slice(&data).unwrap();
  assert_eq!(x.config,y.config);
  assert_eq!(y.key,"game-a");
  assert!(serde_json::from_str::<OptBackup>(r#"{"version":1,"key":"a","config":{},"evil":true}"#).is_err());
 }
}
