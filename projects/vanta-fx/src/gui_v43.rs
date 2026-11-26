fn curated_library_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"MY GAME LIBRARY");
  caption(ui,"Manually selected local games. Steam/Epic/GOG manifests import names only; choose the trusted game EXE yourself. No automatic launch or injected code.");
  ui.horizontal_wrapped(|ui|{
   ui.add(egui::TextEdit::singleline(&mut app.library_title).hint_text("Game name").char_limit(100).desired_width(150.));
   if outline_button(ui,"Add local EXE")&&!app.is_smoke{
    if let Some(p)=crate::filedialog::select_game_exe(){
     let name=if app.library_title.trim().is_empty(){p.file_stem().map(|v|v.to_string_lossy().into_owned()).unwrap_or_default()}else{app.library_title.clone()};
     match crate::game_library::Entry::manual(&name,p).and_then(|e|crate::game_library::insert(&mut app.settings.curated_games,e)){
      Ok(())=>{app.library_title.clear();app.library_confirm=None;app.save();},Err(e)=>app.toast(e)
     }
    }
   }
   if outline_button(ui,"Import Steam / Epic / GOG manifest")&&!app.is_smoke{
    if let Some(p)=crate::filedialog::select_launcher_manifest(){
     match crate::game_library::read_manifest(&p).and_then(|e|crate::game_library::insert(&mut app.settings.curated_games,e)){
      Ok(())=>{app.library_confirm=None;app.save();},Err(e)=>app.toast(e)
     }
    }
   }
  });
  if ui.add_enabled(app.library_scan.is_none(),egui::Button::new("Scan local manifest folder")).clicked()&&!app.is_smoke{
   if let Some(p)=crate::filedialog::select_directory(){let(tx,rx)=std::sync::mpsc::channel();app.library_scan=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::game_library::scan_dir(&p));});}
  }
  if ui.add_enabled(app.library_scan.is_none(),egui::Button::new("Discover installed launcher manifests")).clicked()&&!app.is_smoke{
   let(tx,rx)=std::sync::mpsc::channel();app.library_scan=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::launcher_discovery::discover());});
  }
  caption(ui,"Explicit local discovery: Steam registry and library folders, Epic manifests, common GOG installation folders. Custom GOG locations: choose a manifest folder. No automatic launch.");
  if app.library_scan.is_some(){ui.spinner();}
  ui.add(egui::TextEdit::singleline(&mut app.library_filter).hint_text("Find saved game").char_limit(64));
  let q=app.library_filter.trim().to_lowercase();
  let mut remove=None;
  egui::ScrollArea::vertical().id_salt("curated-game-list").max_height(290.).show(ui,|ui|{
   for (i,entry) in app.settings.curated_games.clone().into_iter().enumerate(){
    if !entry.title.to_lowercase().contains(&q){continue;}
    ui.push_id(i,|ui|{
     ui.label(RichText::new(&entry.title).strong());caption(ui,&entry.source);
     if let Some(n)=entry.banner.as_ref(){
      if !app.library_banner_cache.contains_key(n){
       if let Ok(img)=crate::game_library::load_banner(n){
        if app.library_banner_cache.len()>=32{app.library_banner_cache.clear();}
        let t=ui.ctx().load_texture(n,egui::ColorImage::from_rgba_unmultiplied([img.width() as usize,img.height() as usize],img.as_raw()),egui::TextureOptions::LINEAR);
        app.library_banner_cache.insert(n.clone(),t);
       }
      }
      if let Some(t)=app.library_banner_cache.get(n){ui.add(egui::Image::new(t).fit_to_exact_size(egui::vec2(160.,90.)));}
     }
     if let Some(p)=&entry.executable{caption(ui,&p.display().to_string());}else{caption(ui,"Executable not selected");}
     ui.horizontal_wrapped(|ui|{
      if outline_button(ui,"Choose EXE")&&!app.is_smoke{
       if let Some(p)=crate::filedialog::select_game_exe(){
        match crate::game_library::Entry::manual(&entry.title,p){
         Ok(v)=>{app.settings.curated_games[i].executable=v.executable;app.library_confirm=None;app.save();},Err(e)=>app.toast(e)
        }
       }
      }
      if ui.add_enabled(entry.executable.is_some(),egui::Button::new("Launch...")).clicked(){app.library_confirm=Some(i);}
      if outline_button(ui,"Banner")&&!app.is_smoke{if let Some(p)=crate::filedialog::select_image(){
       match crate::game_library::import_banner(&p){Ok(n)=>{app.settings.curated_games[i].banner=Some(n);app.save();},Err(e)=>app.toast(e)}
      }}
      if entry.banner.is_some()&&outline_button(ui,"Clear banner"){app.settings.curated_games[i].banner=None;app.save();}
      if outline_button(ui,"Remove entry"){remove=Some(i);}
     });
     if app.library_confirm==Some(i){
      caption(ui,"Start the selected executable normally? Only launch games you trust.");
      ui.horizontal(|ui|{
       if red_button(ui,"Confirm launch")&&!app.is_smoke{
        app.library_confirm=None;match crate::game_library::launch(&entry){Ok(())=>app.toast("Game process launched; select its window once available"),Err(e)=>app.toast(e)}
       }
       if outline_button(ui,"Cancel"){app.library_confirm=None;}
      });
     }
     ui.separator();
    });
   }
  });
  if let Some(i)=remove{app.settings.curated_games.remove(i);app.library_confirm=None;app.save();}
  caption(ui,"Removing an entry removes only its library listing, not any game files.");
 });
}

fn save_vault_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"SAVE VAULT / LOCAL BACKUPS");
  caption(ui,"Select only a game's save folder. Close the game first. Copies files without changing originals; 512 MiB / 4096 files max, no links or junctions. Empty directories are omitted.");
  let idle=app.save_vault_job.is_none();
  ui.add_enabled_ui(idle,|ui|{
   if outline_button(ui,"Choose save folder")&&!app.is_smoke{app.save_vault_source=crate::filedialog::select_directory();}
   if let Some(p)=app.save_vault_source.clone(){
    caption(ui,&p.display().to_string());
    if red_button(ui,"Create and verify backup")&&!app.is_smoke{
     let(tx,rx)=std::sync::mpsc::channel();app.save_vault_job=Some(rx);
     std::thread::spawn(move||{let _=tx.send(crate::save_vault::create(&p).map(|s|format!("Verified {}: {} files, {} bytes",s.name,s.files,s.bytes)));});
    }
   }
   if outline_button(ui,"Refresh backups"){app.save_vault_names=crate::save_vault::list();}
   egui::ScrollArea::vertical().id_salt("save-vault-list").max_height(120.).show(ui,|ui|{
    for n in &app.save_vault_names{if ui.selectable_label(app.save_vault_selected.as_ref()==Some(n),n).clicked(){app.save_vault_selected=Some(n.clone());}}
   });
   if let Some(name)=app.save_vault_selected.clone(){
    if outline_button(ui,"Verify selected")&&!app.is_smoke{
     let(tx,rx)=std::sync::mpsc::channel();app.save_vault_job=Some(rx);let n=name.clone();std::thread::spawn(move||{let _=tx.send(crate::save_vault::verify(&n).map(|s|format!("Verified {} files / {} bytes",s.files,s.bytes)));});
    }
    let preview_id=egui::Id::new("verified-vault-preview");
    let mut preview=ui.data(|d|d.get_temp::<VaultPreview>(preview_id)).unwrap_or_default();
    if let Some(job)=&preview.job{let result=job.lock().ok().and_then(|r|r.try_recv().ok());if let Some(v)=result{preview.job=None;match v{Ok(s)=>{preview.summary=Some(s);preview.error.clear();},Err(e)=>{preview.summary=None;preview.error=e;}}}}
    if ui.add_enabled(preview.job.is_none(),egui::Button::new("Verify and preview selected backup")).clicked()&&!app.is_smoke{
     preview.summary=None;preview.error.clear();let(tx,rx)=std::sync::mpsc::channel();preview.job=Some(std::sync::Arc::new(std::sync::Mutex::new(rx)));let n=name.clone();std::thread::spawn(move||{let _=tx.send(crate::save_vault::verify(&n));});
    }
    if preview.job.is_some(){ui.spinner();ui.ctx().request_repaint_after(std::time::Duration::from_millis(250));}
    let verified=preview.summary.as_ref().is_some_and(|s|s.name==name);
    if verified{if let Some(s)=&preview.summary{caption(ui,&format!("Verified preview: {} / {} files / {:.2} MiB",s.name,s.files,s.bytes as f64/1048576.));}}
    if !preview.error.is_empty(){caption(ui,&preview.error);}
    ui.data_mut(|d|d.insert_temp(preview_id,preview));
    caption(ui,"Restore verifies all checksums again and always creates a new directory. Review selected backup name before choosing destination.");
    if ui.add_enabled(verified,egui::Button::new("Preview / restore to NEW folder")).clicked()&&!app.is_smoke{
     if let Some(p)=crate::filedialog::select_directory(){let(tx,rx)=std::sync::mpsc::channel();app.save_vault_job=Some(rx);std::thread::spawn(move||{let _=tx.send(crate::save_vault::restore(&name,&p).map(|p|format!("Restored to {}",p.display())));});}
    }
   }
  });
  if !idle{ui.spinner();caption(ui,"Local file operation running");}
  caption(ui,"Checksums detect accidental corruption, not authenticity. No overwrites, automatic uploads or deletion of original saves.");
 });
}

#[derive(Clone,Default)]struct VaultPreview{summary:Option<crate::save_vault::Summary>,error:String,job:Option<std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<Result<crate::save_vault::Summary,String>>>> >}
