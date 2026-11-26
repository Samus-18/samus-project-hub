fn capture_health_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"CAPTURE HEALTH / OWN OUTPUT");
  caption(ui,"Latest-frame dequeue, bounded recovery and paused rendering while source is minimized. Independent preview cap uses successful presents; empty capture polls do not extend the deadline.");
  let mut tuning=app.settings.lighting.clone();
  let mut changed=ui.checkbox(&mut tuning.reconstruction.capture_diagnostic,"Optional sampled black/static-frame diagnosis").changed();
  changed|=ui.checkbox(&mut tuning.reconstruction.animated_grain,"Animate film grain (1x, without temporal effects)").changed();
  changed|=ui.checkbox(&mut tuning.reconstruction.fused_color,"Fuse compatible color passes (lower memory traffic)").changed();
  changed|=ui.checkbox(&mut tuning.reconstruction.live_gpu_timing,"Sample live GPU phase timestamps (once per second)").changed();
  if changed{app.apply_native_lighting(tuning);}
  caption(ui,"Opt-in GPU 16x16 sparse aggregate approximately once per second; only 16-byte nonblocking summary read, no screenshots or saved pixels. Repeated samples can be a normal static scene.");
  if ui.checkbox(&mut app.settings.preview_vsync,"VSync for independent 1x preview").changed(){
   if let Some(w)=app.native_mirror.as_ref(){w.set_vsync(app.settings.preview_vsync);}
   app.save();
  }
  if ui.checkbox(&mut app.settings.preview_tearing,"Allow tearing / VRR-compatible separate 1x output").changed(){app.save();app.refresh_native_mirror();}
  caption(ui,"Requires VSync OFF and DXGI feature support. Allows tearing only in own flip-model separate preview, not proof of active FreeSync, no game VSync change.");
  if ui.checkbox(&mut app.settings.preview_composition,"DirectComposition separate output (experimental)").changed(){app.save();app.refresh_native_mirror();}
  caption(ui,"Alternative visual composition path, separate 1x only. Falls back to standard DXGI; no game injection.");
  if ui.checkbox(&mut app.settings.preview_low_latency,"DXGI waitable queue (separate 1x)").changed(){
   app.save();app.refresh_native_mirror();
  }
  caption(ui,"One queued frame for Horizon's own output, waits before dispatch. Falls back if unsupported. Does not change the game's render queue.");
  if let Some(w)=app.native_mirror.as_ref(){
   let h=w.diagnostics();
   caption(ui,&format!("Output: {} / nominal refresh: {:?} Hz / FG queue deferrals: {}",if w.composition_active(){"DirectComposition"}else if w.mode()==crate::native_mirror::MirrorMode::CompositedInPlace{"Glass"}else{"HWND flip"},w.nominal_refresh(),w.fg_backlog_deferrals()));
   if h.live_look_rejected{ui.colored_label(RED,"Requested shader stack rejected; using last valid stack or neutral output.");}
   if h.optional_warning!=0{
    let names=[(1,"LUT"),(2,"Cinema"),(4,"Color recipe"),(8,"Precision"),(16,"Native stage"),(32,"HUD protection"),(64,"Layer mix"),(128,"Histogram"),(256,"Fused color (separate passes restored)")].into_iter().filter(|(bit,_)|h.optional_warning&bit!=0).map(|(_,n)|n).collect::<Vec<_>>().join(", ");
    ui.colored_label(RED,format!("Optional GPU stage fallback: {names}"));
    caption(ui,"Preview continues with last working optional stage or bypass. Restart Horizon's own preview to retry a cached stage failure. Device-loss still triggers recovery.");
   }
   if let (Some(input),Some(output),Some(mode))=(h.scale_input,h.scale_output,h.effective_scale_mode){caption(ui,&format!("Scaler: {}x{} -> {}x{} / {} / runtime fallbacks: {}",input.0,input.1,output.0,output.1,crate::spatial::MODES.get(mode as usize).unwrap_or(&"Unknown"),h.scale_fallbacks));}
   caption(ui,&format!("Color dispatches saved last frame: {} / capture copy synchronization: {}", w.fused_dispatches_saved(),if w.fence_active(){"GPU fence event"}else{"compatibility query"}));
   caption(ui,&format!("DXGI tearing support {:?}; last real Present allow-tearing flag {}",h.tearing_supported,h.tearing_active));
   ui.colored_label(V25_MINT,if !w.running(){"STOPPING"}else if h.paused{"SOURCE PAUSED"}else{"WORKER ACTIVE"});
   caption(ui,&format!("Real presents: {} / interpolated: {} / rebuilds: {}",w.submitted_frames(),w.generated_frames(),w.rebuild_count()));
   caption(ui,&format!("Older WGC frames discarded: {} / repeated timestamps: {} / empty capture polls: {}",h.dequeued,h.duplicate_frames,h.empty));
   caption(ui,&format!("Stale frames skipped before GPU work: {}",h.stale_pre_filter));
   if h.own_minimized{caption(ui,"Own preview minimized: GPU dispatch paused.");}
   caption(ui,if h.capture_interval_active{"WGC native minimum interval accepted; software deadline remains active."}else{"Software capture deadline; native interval unavailable or not requested."});
   if app.settings.lighting.reconstruction.capture_diagnostic{
    caption(ui,if h.paused{"Sample diagnostic paused"}else{match h.sample_code{1=>"Sampled content not near-black",2=>"Near-black sampled content (may be intentional)",3=>"Sample diagnostic unavailable",_=>"Waiting for aggregate"}});
    if !h.paused&&matches!(h.sample_code,1|2){caption(ui,&format!("Repeated sample signatures: {} (not proof of a frozen game)",h.sample_repeats));}
   }
   if w.multiplier()>1{caption(ui,&format!("FG requested {}x / last allowed {}x / age deferrals {} / reduced decisions {} / GPU failures {}",w.multiplier(),h.fg_effective,h.fg_stale,h.fg_reduced,h.fg_gpu_errors));}
   caption(ui,&format!("FG safety deferrals: {} / unsuccessful synthetic presents: {}",h.deferred,h.failed_fg));
   if w.multiplier()>1{caption(ui,&format!("FG gate: {} / GPU budget deferrals: {}",crate::render_pressure::fg_reason(h.fg_reason),h.fg_pressure_skips));}
   caption(ui,&format!("Waitable queue: {} / wait timeouts: {}",if h.waitable_active{"active"}else{"off or fallback"},h.wait_timeouts));
   let fmt=|v:Option<f64>|v.map(|x|format!("{x:.2} ms")).unwrap_or_else(||"Unavailable".into());
   ui.label(format!("Last capture + effects CPU wall: {}",fmt(h.capture_ms)));
   ui.label(format!("Last interpolation / submit CPU wall: {}",fmt(h.submit_ms)));
   ui.label(format!("WGC frame age at dequeue: {}",if h.paused{"Paused".into()}else{fmt(h.capture_age_ms)}));
   ui.label(format!("WGC source age at own Present return: {}",if h.paused{"Paused".into()}else{fmt(h.present_age_ms)}));
   caption(ui,&format!("Capture worker heartbeat: {}",fmt(h.worker_age_ms)));
   caption(ui,&format!("Native look stages selected: {} / non-submitted own Presents: {}",h.effective_passes,h.non_submitted));
   caption(ui,"Present-return age is not physical scanout or input latency. Occluded/busy status is not counted as a submitted frame.");
   if app.settings.lighting.reconstruction.live_gpu_timing{
    for(i,n)in ["Capture GPU copy","Prefilter + shader GPU commands","Scale + reconstruction GPU commands","Temporal / FG motion estimate GPU commands","FG synthesis GPU commands"].iter().enumerate(){
      caption(ui,&format!("{n}: {}",if h.paused{"Paused".into()}else{fmt(h.gpu_phase_ms[i])}));
    }
    caption(ui,"Sparse D3D11 timestamps; unavailable or stale values stay blank. No live query wait, no extra Flush. FG synthesis is one sampled phase; timings exclude Present, not end-to-end latency.");
   }
  }else{caption(ui,"No active own output. No live capture counters.");}
  caption(ui,"These are actual worker counters and CPU wall intervals, not game FPS, GPU timestamp intervals or input-to-photon latency. A static scene cannot be classified as a frozen game just by its pixels.");
 });
}
fn hardware_bench_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"GPU LAB / MEASURED EFFECT COST");
  caption(ui,"Synthetic texture only. D3D11 timestamp queries measure built-in effects at 720p and the selected ordered stack at 720p, 1080p and 1440p. No game capture or FPS boost estimate.");
  if outline_button(ui,"Read actual D3D11 capabilities")&&!app.is_smoke{
   app.capability_result=Some(crate::gpu_caps::read_native_capabilities());
  }
  match &app.capability_result{
   Some(Ok(v))=>{ui.label(&v.adapter);caption(ui,&format!("FL11: {} / RGBA8 UAV: {} / RGBA16F UAV: {} / R32F UAV: {}",
      v.feature_level_11,v.rgba8_typed_uav,v.rgba16f_typed_uav,v.r32f_typed_uav));}
   Some(Err(e))=>caption(ui,e),None=>{}
  }
  let idle=app.native_mirror.is_none()&&app.effect_bench_rx.is_none()&&app.pipeline_bench_rx.is_none()&&app.photo_rx.is_none()&&app.motion_lab_rx.is_none();
  if ui.add_enabled(idle,egui::Button::new("Run GPU benchmark")).clicked()&&!app.is_smoke{
   let stack=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());
   let (tx,rx)=std::sync::mpsc::channel();app.effect_bench_rx=Some(rx);app.effect_bench_result=None;
   std::thread::spawn(move||{let _=tx.send(crate::effect_bench::run(&stack));});
  }
  if ui.add_enabled(idle,egui::Button::new("Benchmark filter fusion A/B")).clicked()&&!app.is_smoke{
   let stack=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());
   let tuning=app.settings.lighting.clone();let(tx,rx)=std::sync::mpsc::channel();app.effect_bench_rx=Some(rx);app.effect_bench_result=None;
   std::thread::spawn(move||{let _=tx.send(crate::effect_bench::fusion_comparison(&stack,&tuning));});
  }
  if ui.add_enabled(idle,egui::Button::new("Benchmark current lighting A/B")).clicked()&&!app.is_smoke{
   let stack=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());
   let tuning=app.settings.lighting.clone();let(tx,rx)=std::sync::mpsc::channel();app.effect_bench_rx=Some(rx);app.effect_bench_result=None;
   std::thread::spawn(move||{let _=tx.send(crate::effect_bench::run_lighting(&stack,&tuning));});
  }
  if app.effect_bench_rx.is_some(){caption(ui,"Measuring real GPU timestamps. Avoid benchmarking while gaming.");}
  match &app.effect_bench_result{
   Some(Ok(rows))=>{
    if outline_button(ui,"Copy benchmark JSON"){if let Ok(s)=serde_json::to_string_pretty(rows){ui.ctx().copy_text(s);}}
    egui::ScrollArea::vertical().id_salt("gpu-bench-results").max_height(240.0).show(ui,|ui|{
     for row in rows{ui.horizontal_wrapped(|ui|{
       ui.label(&row.label);caption(ui,&format!("{}x{} / {}",row.width,row.height,
        row.gpu_ms.map(|v|format!("{v:.3} ms")).unwrap_or("Unavailable".into())));
     });}
    });
   }
   Some(Err(e))=>caption(ui,e),None=>{}
  }
  caption(ui,"A missing/disjoint GPU timestamp is unavailable, never replaced by a guessed result.");
 });
}
