fn display_topology_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"DISPLAY / READ-ONLY TOPOLOGY");
  let adapter_key=egui::Id::new("hardware-adapter-list-v59");
  let mut adapters=ui.data_mut(|d|d.get_temp::<Vec<crate::adapter_select::Adapter>>(adapter_key));
  if outline_button(ui,"Read renderer adapters")&&!app.is_smoke{match crate::adapter_select::list(){Ok(v)=>{adapters=Some(v.clone());ui.data_mut(|d|d.insert_temp(adapter_key,v));},Err(e)=>app.toast(e)}}
  if let Some(rows)=adapters{
   let mut sel=app.settings.renderer_adapter.clone();let old=sel.clone();
   egui::ComboBox::from_id_salt("renderer-adapter-selection").selected_text(rows.iter().find(|r|Some(&r.id)==sel.as_ref()).map(|r|r.name.as_str()).unwrap_or(if sel.is_some(){"Selected adapter unavailable"}else{"Automatic"})).show_ui(ui,|ui|{
    ui.selectable_value(&mut sel,None,"Automatic");
    for row in &rows{ui.selectable_value(&mut sel,Some(row.id.clone()),format!("{} / {} MiB VRAM",row.name,row.dedicated_mib));}
   });
   if sel!=old{app.settings.renderer_adapter=sel;app.save();app.refresh_native_mirror();}
  }
  caption(ui,"Own WGC D3D11 renderer only. An unavailable explicit adapter stops preview; select Automatic. Game adapter never modified.");
  caption(ui,"Current Windows display mode and desktop coordinates. Nominal refresh only, not measured VRR or exclusive fullscreen support.");
  let key=egui::Id::new("display-topology-cache-v49");
  let mut data=ui.data_mut(|d|d.get_temp::<Vec<crate::display_probe::Display>>(key));
  if outline_button(ui,"Refresh display modes")&&!app.is_smoke{
   let rows=crate::display_probe::read();
   ui.data_mut(|d|d.insert_temp(key,rows.clone()));data=Some(rows);
  }
  if let Some(rows)=data{
   let mut selected=app.settings.preview_display.clone();let old=selected.clone();
   egui::ComboBox::from_id_salt("preview-display-select").selected_text(selected.as_deref().unwrap_or("Automatic primary")).show_ui(ui,|ui|{
    ui.selectable_value(&mut selected,None,"Automatic primary");
    for d in &rows{if d.target().is_some(){ui.selectable_value(&mut selected,Some(d.name.clone()),format!("{} / {:?} Hz",d.name,d.refresh));}}
   });
   if selected!=old{
    app.settings.preview_display=selected;app.save();
    if app.native_mirror.as_ref().is_some_and(|w|w.mode()==crate::native_mirror::MirrorMode::SeparateWindow){app.refresh_native_mirror();}
   }
   caption(ui,"Separate preview placement only. Source game, fullscreen and game monitor are unchanged. Selected display missing: fall back to primary. Two matching topology changes trigger bounded own-output rebuild; multi-DPI behavior not verified.");
   if let Some(display)=crate::display_probe::select(&rows,app.settings.preview_display.as_deref()){
    ui.separator();heading(ui,"OUTPUT COLOR PROFILE");
    caption(ui,"Explicit override of the game color grade on this logical Windows output, separate preview only. Not ICC or physical calibration. Display numbers may change after reconnect.");
    let has=app.settings.display_grades.contains_key(&display.name);
    caption(ui,if has{"Saved output grade overrides the game grade here."}else{"No override: game grade used."});
    let mut changed=false;
    if outline_button(ui,"Save current game grade as output override"){
     changed=crate::display_color::store(&mut app.settings.display_grades,&display.name,&app.settings.lighting.grade);
    }
    if has&&outline_button(ui,"Remove output override"){
     app.settings.display_grades.remove(&display.name);changed=true;
    }
    if changed{app.save();if app.native_mirror.as_ref().is_some_and(|w|w.mode()==crate::native_mirror::MirrorMode::SeparateWindow){app.refresh_native_mirror();}}
   }
   if rows.is_empty(){caption(ui,"No active display returned by Windows.");}
   for d in &rows{
    ui.label(format!("{} {}",d.name,if d.primary{"PRIMARY"}else{""}));
    let n=|x:Option<u32>|x.map(|v|v.to_string()).unwrap_or("Unavailable".into());
    caption(ui,&format!("{} x {} / {} Hz nominal / {} bpp",n(d.width),n(d.height),n(d.refresh),n(d.bits)));
    caption(ui,&format!("Adapter: {} / position {:?}, {:?}",d.adapter,d.x,d.y));
   }
  }else{caption(ui,"Read display modes on demand. Own preview checks topology every 2s while active; no driver changes.");}
 });
}
