// Categorized original controls, migrated intact from the former staging hub.
fn suite_quick_controls(app:&mut BoostApp,ui:&mut Ui){
 egui::CollapsingHeader::new("RYCHLÉ OVLÁDÁNÍ").default_open(false).id_salt("unsorted-quick").show(ui,|ui|{
  frame(CARD).show(ui,|ui|{
   let name=app.telem.active_game.as_ref().map(|g|g.title.clone()).unwrap_or_else(||"Nevybraná hra".into());
   ui.horizontal_wrapped(|ui|{
    ui.label(format!("Hra: {}",trim(&name,35)));
    if outline_button(ui,"Vybrat hru"){app.page=Page::Library;}
    ui.label(format!("FPS: {}",fps_text(app,"fps")));
    if let Some(w)=app.native_mirror.as_ref(){let d=w.diagnostics();caption(ui,&format!("Capture state: {} | duplicate {} / stale {}",crate::capture_health::classify(w.running(),w.submitted_frames(),d.worker_age_ms,d.paused,d.own_minimized).label(),d.duplicate_frames,d.stale_pre_filter));if let Some(rates)=app.output_rates.snapshot(std::time::Instant::now()){caption(ui,&format!("GAME {} FPS  /  OWN REAL {:.0} + FG {:.0} = {:.0} FPS (x{})",fps_text(app,"fps"),rates.real,rates.generated,rates.total,w.multiplier()));}else{caption(ui,&format!("Own output x{}: {} real + {} generated frames",w.multiplier(),w.submitted_frames(),w.generated_frames()));}}
   });
   ui.horizontal_wrapped(|ui|{
    let can_run=app.telem.active_game.is_some()&&!app.is_smoke;
    if app.native_mirror.is_some(){
     if red_button(ui,"Vypnout GPU výstup"){app.stop_native_mirror();}
    }else{
     if ui.add_enabled(can_run,egui::Button::new("Zapnout GPU filtry 1x")).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);}
     if ui.add_enabled(can_run,egui::Button::new("Borderless fullscreen output (experimental)")).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::BorderlessScreen,1);}
     if ui.add_enabled(can_run,egui::Button::new("Native borderless GPU output (experimental)")).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::BorderlessScreen,1);}
     if ui.add_enabled(can_run,egui::Button::new("DXGI desktop duplication fallback (monitor-filling game only)")).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::DesktopDuplication,1);}
     let factor=app.settings.external_fg_factor.clamp(2,3);
     if ui.add_enabled(can_run&&app.settings.experimental_fg_allowed,egui::Button::new(format!("FrameGen x{factor} - own output"))).clicked(){app.settings.external_fg_factor=factor;app.save();app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,factor);}
     if ui.add_enabled(can_run&&app.settings.experimental_fg_allowed,egui::Button::new(format!("Borderless FG x{factor} (experimental)"))).clicked(){app.settings.external_fg_factor=factor;app.save();app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::BorderlessScreen,factor);}
    }
    if outline_button(ui,if app.overlay_open{"Vypnout FPS HUD"}else{"Zapnout FPS HUD"})&&!app.is_smoke{app.toggle_passive_hud();}
   });
   ui.horizontal_wrapped(|ui|{
    if ui.checkbox(&mut app.settings.experimental_fg_allowed,"Enable experimental FrameGen (own output only)").changed(){app.save();}
    for factor in [2u8,3u8]{if ui.selectable_value(&mut app.settings.external_fg_factor,factor,format!("x{factor}")).changed(){app.save();}}
    if ui.checkbox(&mut app.settings.monitor_fps_auto_apply,"Auto-apply saved game + monitor FPS profile").changed(){app.monitor_profile_applied=None;app.save();}
    if ui.checkbox(&mut app.settings.hud_f12,"F12: FPS HUD").changed(){app.save();}
    if ui.checkbox(&mut app.settings.fps_monitoring,"Měřit FPS (ETW)").changed(){app.save();}
    if ui.checkbox(&mut app.settings.detect_games,"Hledat hry").changed(){app.save();}
   });
   ui.horizontal_wrapped(|ui|{
    if outline_button(ui,"Snímek obrazovky")&&!app.is_smoke{app.screenshot();}
    if outline_button(ui,if app.recorder.active(){"Zastavit záznam"}else{"Video záznam"})&&!app.is_smoke{app.recording();}
    if red_button(ui,"Nouzově vše vypnout"){app.emergency_stop();}
   });
   caption(ui,"FrameGen generuje snímky jen ve vlastním okně; nezdvojnásobí skutečné FPS herního enginu. Fullscreen a hra zatím nejsou ověřeny.");
  });
 });

}
fn suite_moved_visual(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"VISUAL / EXTRA CONTROLS");
 egui::CollapsingHeader::new("SHADER NODE GRAPH – REAL D3D11 STAGES").default_open(false).id_salt("unsorted-graph").show(ui,|ui|node_graph_panel(app,ui));
 egui::CollapsingHeader::new("HDR DISPLAY CAPABILITY (READ ONLY)").default_open(false).id_salt("hb-hdr-cap").show(ui,|ui|{
   caption(ui,"DXGI reports current display color space and luminance. The own output is SDR unless the optional real FP16 scRGB pipeline confirms active; monitor HDR metadata is not physical calibration.");
   let id=egui::Id::new("hb-hdr-probe");
   if ui.small_button("Read DXGI Advanced Color capability").clicked(){
    let output=crate::hdr_probe::report().unwrap_or_else(|e|format!("HDR info unavailable: {e}"));
    ui.data_mut(|d|d.insert_temp(id,output));
   }
   if let Some(status)=ui.data(|d|d.get_temp::<String>(id)){ui.label(status);}
 });
 egui::CollapsingHeader::new("LIVE CUSTOM SHADERS: HLSL / WGSL / SPIR-V").default_open(false).id_salt("hb-custom-hlsl").show(ui,|ui|{
  caption(ui,"User-selected filename from settings/custom-shaders: HLSL .hlsl, WGSL .wgsl, SPIR-V .spv. WGSL/SPIR-V translated by Naga into validated D3D11 CS_5_0. HLSL max 8KiB, SPIR-V 16KiB. No game injection.");
  let id=egui::Id::new("hb-hlsl-name");
  let mut name=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_else(||"example.hlsl".into());
  ui.horizontal_wrapped(|ui|{
   ui.label("File");ui.add(egui::TextEdit::singleline(&mut name).desired_width(180.).char_limit(48));
   if ui.button("Create default example").clicked(){match crate::custom_hlsl::make_example(){Ok(p)=>app.toast(format!("Example file ready: {}",p.display())),Err(e)=>app.toast(e)}}
   if ui.button("Validate, translate and live reload").clicked(){
    match crate::custom_hlsl::load_named(&name){
     Ok(version)=>{app.refresh_native_mirror();app.toast(format!("Custom HLSL loaded r{version}; active own GPU output rebuilding"));},
     Err(e)=>app.toast(format!("Custom HLSL rejected, old GPU stage preserved: {e}"))
    }
   }
   if ui.button("Use converted shader as active stack").clicked(){
    let ids=vec![crate::native_filter::CUSTOM_STAGE];
    app.apply_custom_native_stack(ids);
   }
  });
  ui.data_mut(|d|d.insert_temp(id,name));
  caption(ui,&format!("Source folder: {} | Revision {}",crate::custom_hlsl::folder().display(),crate::custom_hlsl::revision()));
 });
 egui::CollapsingHeader::new("LIVE WGC GPU A/B SPLIT – SOURCE VS FILTERED").default_open(false).id_salt("hb-live-ab").show(ui,|ui|{
  caption(ui,"Real WGC captured game image in own GPU output: original source vs currently active shaders. This is not a game-engine overlay or exclusive-fullscreen claim.");
  let mut light=app.settings.lighting.clone();
  let mut active=light.layers.split.is_some();
  let mut changed=ui.checkbox(&mut active,"Show source/effect split (only own output)").changed();
  let mut mix=light.layers.split.unwrap_or(0.5);
  if ui.add_enabled(active,egui::Slider::new(&mut mix,0.05..=0.95).text("Split position")).changed(){changed=true;}
  if changed{light.layers.split=active.then_some(mix);app.apply_native_lighting(light);}
  if app.native_mirror.is_none(){caption(ui,"Choose running game and start own native GPU output above to compare live captured frames.");}
 });
 egui::CollapsingHeader::new("REAL HDR-CAPABLE OWN SC RGB PREVIEW").default_open(false).id_salt("hb-own-scrgb").show(ui,|ui|{
   caption(ui,"Only opt-in Horizon-owned x1 preview on an actively HDR-capable Windows monitor. Actual DXGI 16-bit floating point scRGB output; source game is SDR, this does NOT recover HDR highlights, replace Windows Auto HDR, or change game/display settings. Falls back to SDR on unsupported monitors.");
   if ui.checkbox(&mut app.settings.preview_scrgb,"Use FP16 scRGB own output on HDR active monitor").changed(){
      app.save();
      if app.native_mirror.is_some(){app.toast("Restart own GPU output to apply scRGB preference");}
   }
   if ui.add(egui::Slider::new(&mut app.settings.preview_hdr_white_nits,80.0..=400.0).suffix(" nits SDR white").text("White reference")).changed(){app.save();}
   let mut changed=false;
     ui.horizontal_wrapped(|ui|{for(i,n)in ["Neutral SDR","Reinhard","ACES-style"].iter().enumerate(){changed|=ui.selectable_value(&mut app.settings.hdr_curve,i as u8,*n).changed();}});
     changed|=ui.add(egui::Slider::new(&mut app.settings.hdr_peak_nits,80..=2000).suffix(" nits").text("Output luminance ceiling")).changed();
     changed|=ui.add(egui::Slider::new(&mut app.settings.hdr_exposure,5..=40).text("Curve exposure (tenths)")).changed();
     if changed{crate::hdr_output::configure(app.settings.hdr_curve,app.settings.hdr_peak_nits,app.settings.hdr_exposure);app.save();}
     caption(ui,"Creative luminance mapping on actual own FP16 output only. Neutral is unchanged. Not Windows Auto HDR or physical calibration.");
     if ui.button("Suggest SDR white from selected HDR monitor metadata").clicked(){
      let suggested=app.settings.preview_display.as_deref().and_then(|name|
          crate::hdr_probe::probe().ok().and_then(|rows|crate::hdr_probe::suggested_white(&rows,name)));
      if let Some(white)=suggested{
          app.settings.preview_hdr_white_nits=white;app.save();
          app.toast(format!("Applied DXGI metadata starting point: {white:.0} nits; not measured physical calibration. Restart own output to apply."));
      }else{
          app.toast("No exact selected active HDR monitor with plausible DXGI peak/full-frame luminance; calibration unchanged.");
      }
   }
   if let Some(output)=app.native_mirror.as_ref(){
      let active=output.hdr_active();
      ui.label(if active{"DXGI own scRGB output ACTIVE"}else{"DXGI own output currently SDR"});
   }
 });
}
fn suite_moved_system_tune(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"SYSTEM TUNE / EXTRA CONTROLS");
 egui::CollapsingHeader::new("MOBILE DASHBOARD (TRUSTED LAN ONLY)").default_open(false).id_salt("hb-mobile").show(ui,|ui|{
  caption(ui,"Off by default. Explicitly starts a token-protected read-only HTTP dashboard visible to devices on your trusted Wi-Fi. No TLS.");
  if app.mobile.addr.is_some(){if let Some(url)=app.mobile.connection_hint(){ui.monospace(url);}if ui.small_button("Stop mobile dashboard").clicked(){app.mobile.stop();}}
  else if ui.button("Enable mobile dashboard on trusted LAN").clicked(){if let Err(e)=app.mobile.start(){app.toast(e);}}
 });
 egui::CollapsingHeader::new("OWN PROCESS MEMORY - MEASURED").default_open(false).id_salt("hb-v101-memory").show(ui,|ui|{
  caption(ui,"Measured Windows process working set / private bytes; not a promise of <30 MiB. Never clears RAM automatically.");
  match crate::process_memory::read(){
   Ok(memory)=>{ui.label(memory.report());}
   Err(e)=>{ui.label(format!("Memory measurement unavailable: {e}"));}
  };
 });
 egui::CollapsingHeader::new("WINDOWS STANDBY CACHE CLEANER (MANUAL)").default_open(false).id_salt("hb-standby-cache").show(ui,|ui|{
   caption(ui,"Windows standby memory is a useful disk cache. Clearing it can INCREASE stutter and does not increase FPS by itself. User confirmation required, never scheduled or automatic. Only attempts a purge below 3 GiB available RAM; Windows may deny privileges.");
   match crate::standby_cleaner::ram(){
    Ok((total,available,load))=>ui.label(format!("RAM: {:.1} GiB total / {:.1} GiB available / {}% load",total as f64/1073741824.,available as f64/1073741824.,load)),
    Err(e)=>ui.label(e)
   };
   let id=egui::Id::new("hb-standby-ack");
   let mut armed=ui.data(|d|d.get_temp::<bool>(id)).unwrap_or(false);
   ui.checkbox(&mut armed,"I understand this may cause longer loading or stuttering");
   if ui.add_enabled(armed,egui::Button::new("Purge standby pages ONCE (experimental)")).clicked(){
    armed=false;
    match crate::standby_cleaner::purge_confirmed(){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}
   }
   ui.data_mut(|d|d.insert_temp(id,armed));
 });
 egui::CollapsingHeader::new("AMD RX580 DRIVER FAN AND TEMPERATURE (READ ONLY)").default_open(false).id_salt("hb-amd-adl").show(ui,|ui|{
  caption(ui,"User-invoked official AMD ADL2 driver getter functions, no fan/power/voltage writes. Only one active AMD logical adapter permitted; refuses ambiguous mappings and unsupported legacy Overdrive5.");
  let (busy,msg)=crate::amd_adl::snapshot();
  if ui.add_enabled(!busy,egui::Button::new("Read AMD driver fan/temperature capability")).clicked(){
    if let Err(e)=crate::amd_adl::start(){app.toast(e);}
  }
  if busy{ui.label("Reading AMD driver (read-only)...");}
  if let Some(msg)=msg{ui.label(msg);}
 });
}
fn suite_moved_lab(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"LAB / EXTRA CONTROLS");
 egui::CollapsingHeader::new("READ-ONLY AUTOMATION SCRIPT / CLI").default_open(false).id_salt("hb-readonly-script").show(ui,|ui|{
  caption(ui,"Small JSON-only validation/listing batch script. No Lua/Rhai, no commands to modify games or Windows.");
  let key=egui::Id::new("hb-script-input");let output=egui::Id::new("hb-script-output");
  let mut src=ui.data(|d|d.get_temp::<String>(key)).unwrap_or_else(||r#"{"version":1,"commands":[{"op":"list_shaders"},{"op":"validate_stack","stages":[67,68,2]}]}"#.into());
  ui.add(egui::TextEdit::multiline(&mut src).desired_rows(4).desired_width(f32::INFINITY).char_limit(8192));
  if ui.small_button("Run read-only validation").clicked(){
   let result=crate::script_api::evaluate(&src).unwrap_or_else(|e|format!("Script error: {e}"));
   ui.data_mut(|d|d.insert_temp(output,result));
  }
  ui.data_mut(|d|d.insert_temp(key,src));
  if let Some(result)=ui.data(|d|d.get_temp::<String>(output)){
   egui::ScrollArea::vertical().max_height(220.).show(ui,|ui|{ui.monospace(result);});
  }
 });
 egui::CollapsingHeader::new("DIRECTX 12 HARDWARE COMPUTE DIAGNOSTIC").default_open(false).id_salt("hb-dx12-interop").show(ui,|ui|{
  caption(ui,"Optional real GPU test: create a D3D12 hardware queue, run the Horizon D3D11 compute shader via Windows D3D11On12, and inspect WaveOps lane capabilities. No game, desktop capture, driver tuning or API backend switch.");
  let (running,result)=crate::dx12_compat::snapshot();
  if ui.add_enabled(!running,egui::Button::new("Test D3D12 GPU compute compatibility")).clicked(){
   if let Err(e)=crate::dx12_compat::start(){app.toast(e);}
  }
  if running{ui.label("Testing own offline D3D12 GPU texture...");}
  if let Some(message)=result{ui.label(message);}
 });
 egui::CollapsingHeader::new("INDEPENDENT D3D12 ASYNC COMPUTE QUEUE").default_open(false).id_salt("hb-async-queues").show(ui,|ui|{
  caption(ui,"Optional headless hardware self-test of separate D3D12 COMPUTE/DIRECT queues, GPU buffer command and fences. Not yet the live WGC async shader scheduler. No game or desktop capture.");
  let (busy,message)=crate::async_compute::snapshot();
  if ui.add_enabled(!busy,egui::Button::new("Test RX580 DX12 compute queue")).clicked(){
   if let Err(e)=crate::async_compute::start(){app.toast(e);}
  }
  if busy{ui.label("GPU compute-queue test running in a separate worker...");}
  if let Some(text)=message{ui.label(text);}
 });
 egui::CollapsingHeader::new("RX580 NATIVE D3D12 WAVE64 / SHADER MODEL 6").default_open(false).id_salt("hb-wave64").show(ui,|ui|{
  caption(ui,"Official Microsoft DXC compiled the original shader at development time. Optional offline D3D12 hardware dispatch queries actual WaveGetLaneCount per GPU lane. No DLL injection or desktop/game capture.");
  let (busy,result)=crate::wave64::snapshot();
  if ui.add_enabled(!busy,egui::Button::new("Run actual native WaveOps kernel")).clicked(){
   if let Err(e)=crate::wave64::start(){app.toast(e);}
  }
  if busy{ui.label("Executing native D3D12 compute shader...");}
  if let Some(text)=result{ui.label(text);}
 });
 egui::CollapsingHeader::new("OFFICIAL AMD OPTICAL FLOW / XBOX GAME BAR - SDK READINESS").default_open(false).id_salt("hb-external-sdks").show(ui,|ui|{
  caption(ui,"Honest offline prerequisites only; these two official vendor integrations remain not implemented and are NOT silently substituted by custom optical flow or our own overlay.");
  ui.monospace(crate::sdk_preflight::report());
 });
 egui::CollapsingHeader::new("REAL AMD VULKAN GPU COMPUTE PROOF").default_open(false).id_salt("hb-vulkan").show(ui,|ui|{
  caption(ui,"Explicit AMD Vulkan ICD synthetic compute: Naga compiles the original bounded storage-buffer shader into SPIR-V, then Vulkan performs 256 integer transforms on GPU. No app API switch yet, no game or desktop capture.");
  let(busy,message)=crate::vulkan_compute::snapshot();
  if ui.add_enabled(!busy,egui::Button::new("Test Vulkan compute on AMD GPU")).clicked(){
    if let Err(e)=crate::vulkan_compute::start(){app.toast(e);}
  }
  if busy{ui.label("Vulkan compute worker active...");}
  if let Some(text)=message{ui.label(text);}
 });
}
fn suite_moved_game_tools(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"GAME TOOLS / EXTRA CONTROLS");
 egui::CollapsingHeader::new("CURSOR CLIP – OPT IN").default_open(false).id_salt("hb-cursor-clip").show(ui,|ui|{
  caption(ui,"Safe user-armed game cursor clipping. Recomputes client bounds on resize and releases on minimize, focus loss or emergency stop. The own preview has optional 150%/200% Win32 cursor magnification; the game cursor is never modified or moved.");
  ui.horizontal_wrapped(|ui|{
   ui.label("OWN preview cursor (game cursor untouched):");
   for scale in [100u8,150,200]{
    if ui.selectable_value(&mut app.settings.preview_cursor_scale,scale,format!("{scale}%")).changed(){
     crate::native_mirror::configure_preview_cursor_scale(scale);app.save();
    }
   }
  });
  ui.label(format!("Shortcut: {}",app.settings.global_shortcuts.label("CURSOR_LOCK")));
  if app.cursor_lock.active(){if ui.button("Release game cursor").clicked(){app.cursor_lock.release();}}
  else if app.cursor_lock_armed{if ui.button("Cancel armed cursor clip").clicked(){app.cursor_lock_armed=false;}}
  else if ui.button("Arm clip for selected game on next focus").clicked(){app.cursor_lock_armed=true;}
 });
}
fn suite_moved_boost(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"BOOST / EXTRA CONTROLS");
 egui::CollapsingHeader::new("MEASURE PHYSICAL OUTPUT REFRESH (READ ONLY)").default_open(false).id_salt("hb-refresh-physical").show(ui,|ui|{
  caption(ui,"Explicit DXGI WaitForVBlank measurement on selected output, no game FPS manipulation. May take around half a second in background. Tearing capability is not proof that active FreeSync/VRR is enabled.");
  let (working,result)=crate::refresh_probe::snapshot();
  if ui.add_enabled(!working,egui::Button::new("Measure actual output VBlank Hz")).clicked(){
    if let Err(e)=crate::refresh_probe::start(app.settings.preview_display.clone()){app.toast(e);}
  }
  if working{ui.label("Measuring output blanking in separate worker...");}
  if let Some(result)=result{ui.label(result);}
 });
}
fn suite_moved_settings(app:&mut BoostApp,ui:&mut Ui){
 ui.add_space(10.);
 heading(ui,"SETTINGS / EXTRA CONTROLS");
 egui::CollapsingHeader::new("SAFE MODE / ANTI-CHEAT HEURISTIC").default_open(false).id_salt("hb-safe-mode").show(ui,|ui|{
   caption(ui,"Default ON. Blocks known anti-cheat-sensitive game titles/processes from external capture. No DLL injection, no game-memory access. Detection is a heuristic, NOT anti-cheat certification.");
   if ui.checkbox(&mut app.settings.anti_cheat_safe_mode,"Restrict external capture in recognized protected games").changed(){app.save();}
   if let Some(game)=app.telem.active_game.as_ref(){
    let verdict=crate::anticheat_guard::inspect(&game.path,&game.title);
    ui.label(format!("Selected game: {}",verdict.reason));
   }
 });
}

fn node_graph_panel(app:&mut BoostApp,ui:&mut Ui){
 let key=app.profile_key();
 let fallback=app.settings.native_stack_profiles.get(&key).map(Vec::as_slice)
  .unwrap_or(crate::native_presets::PRESETS[app.settings.native_preset].stages);
 let mut graph=app.settings.node_graph_profiles.get(&key).cloned()
  .unwrap_or_else(||crate::node_graph::Graph::from_stages(fallback));
 caption(ui,"Real shader pipeline: nodes are native HLSL stages; arrows represent processing order. Fan-in does NOT mix textures.");
 let mut changed=false;let mut remove=None;
 for n in &mut graph.nodes{
  ui.push_id(n.id,|ui|{
   ui.horizontal_wrapped(|ui|{
    ui.label(format!("Node {:02}",n.id));
    egui::ComboBox::from_id_salt("native-stage").selected_text(crate::native_filter::PROGRAMS.get(n.stage).map(|p|p.0).unwrap_or("?")).show_ui(ui,|ui|{
     for (i,(label,_)) in crate::native_filter::PROGRAMS.iter().enumerate(){
      if crate::native_stack::valid_stage(i)&&ui.selectable_value(&mut n.stage,i,*label).changed(){changed=true;}
     }
    });
    if ui.small_button("Remove").clicked(){remove=Some(n.id);}
   });
  });
 }
 if let Some(id)=remove{graph.remove(id);changed=true;}
 ui.horizontal_wrapped(|ui|{
  if ui.add_enabled(graph.nodes.len()<8,egui::Button::new("Add shader node")).clicked(){if let Err(e)=graph.add(2){app.toast(e)}else{changed=true;}}
  if ui.small_button("Clear connections").clicked(){graph.edges.clear();changed=true;}
 });
 let menu_id=ui.id().with("graph-connection");let mut pair=ui.data(|d|d.get_temp::<(u8,u8)>(menu_id)).unwrap_or((0,1));
 ui.horizontal_wrapped(|ui|{
  ui.label("From");
  egui::ComboBox::from_id_salt("from").selected_text(format!("#{}",pair.0)).show_ui(ui,|ui|{
   for n in &graph.nodes{ui.selectable_value(&mut pair.0,n.id,format!("#{}",n.id));}
  });
  ui.label("To");
  egui::ComboBox::from_id_salt("to").selected_text(format!("#{}",pair.1)).show_ui(ui,|ui|{
   for n in &graph.nodes{ui.selectable_value(&mut pair.1,n.id,format!("#{}",n.id));}
  });
  if ui.small_button("Connect").clicked(){match graph.connect(pair.0,pair.1){Ok(())=>changed=true,Err(e)=>app.toast(e)}}
 });
 ui.data_mut(|d|d.insert_temp(menu_id,pair));
 for (i,&(a,b)) in graph.edges.clone().iter().enumerate(){
  ui.horizontal(|ui|{ui.label(format!("#{a} -> #{b}"));if ui.small_button("Disconnect").clicked(){graph.edges.remove(i);changed=true;}});
  if changed{break;}
 }
 let compiled=graph.compile();
 match &compiled{
  Ok(stages)=>caption(ui,&format!("Ready: {} shaders, {} graph links, order {:?}",stages.len(),graph.edges.len(),stages)),
  Err(e)=>caption(ui,&format!("Graph invalid: {e}")),
 }
 if changed{
  if compiled.is_ok(){app.settings.node_graph_profiles.insert(key.clone(),graph.clone());app.save();}
  else{app.toast("Graph would be invalid. Fix it before saving.");}
 }
 if ui.add_enabled(compiled.is_ok(),egui::Button::new("Apply graph to real native GPU stack")).clicked(){
  if let Ok(stages)=compiled{app.settings.node_graph_profiles.insert(key,graph);app.apply_custom_native_stack(stages);}
 }
}
