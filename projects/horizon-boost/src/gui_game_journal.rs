
// Native egui panels, per-game persistent book, lightweight manual controls.
fn journal_entries(ui:&mut Ui,title:&str,items:&mut Vec<crate::game_journal::Entry>,check:bool)->bool{
 let mut changed=false;let mut remove=None;
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,title);
  if ui.add_enabled(items.len()<crate::game_journal::MAX,egui::Button::new("Add entry")).clicked(){
   items.push(crate::game_journal::Entry{name:"New entry".into(),..Default::default()});changed=true;
  }
  for (i,x) in items.iter_mut().enumerate(){ui.push_id(i,|ui|{
   ui.horizontal_wrapped(|ui|{
    if check{changed|=ui.checkbox(&mut x.done,"").changed();}
    changed|=ui.add(egui::TextEdit::singleline(&mut x.name).desired_width(230.).char_limit(80)).changed();
    if ui.small_button("Remove").clicked(){remove=Some(i);}
   });
   changed|=ui.add(egui::TextEdit::multiline(&mut x.note).desired_rows(2).desired_width(ui.available_width()).char_limit(1600).hint_text("Notes / details")).changed();
   ui.separator();
  });}
 });
 if let Some(i)=remove{items.remove(i);changed=true;}changed
}
fn journal_bosses(ui:&mut Ui,items:&mut Vec<crate::game_journal::Boss>)->bool{
 let mut changed=false;let mut remove=None;
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"BOSS TRACKER");
  caption(ui,"Record attempts yourself. Enter duration in seconds; clears and best time are saved for this game.");
  if ui.add_enabled(items.len()<crate::game_journal::MAX,egui::Button::new("Add boss")).clicked(){items.push(crate::game_journal::Boss{name:"New boss".into(),..Default::default()});changed=true;}
  for(i,x)in items.iter_mut().enumerate(){ui.push_id(i,|ui|{
   changed|=ui.add(egui::TextEdit::singleline(&mut x.name).char_limit(80)).changed();
   let rate=if x.attempts>0{100.*x.clears as f32/x.attempts as f32}else{0.};
   ui.label(format!("{} attempts   /   {} clears   /   {:.1}% success",x.attempts,x.clears,rate));
   caption(ui,&format!("Total time {} s  |  Best clear {}",x.seconds,x.best.map(|v|format!("{v} s")).unwrap_or("--".into())));
   let id=ui.id().with("duration");let mut seconds=ui.data(|d|d.get_temp::<u32>(id)).unwrap_or(0);
   ui.horizontal_wrapped(|ui|{
    ui.add(egui::DragValue::new(&mut seconds).range(0..=86400).suffix(" s"));
    if outline_button(ui,"Failed attempt"){x.record(false,seconds);changed=true;}
    if red_button(ui,"Cleared"){x.record(true,seconds);changed=true;}
    if ui.small_button("Remove").clicked(){remove=Some(i);}
   });
   ui.data_mut(|d|d.insert_temp(id,seconds));ui.separator();
  });}
 });
 if let Some(i)=remove{items.remove(i);changed=true;}changed
}
fn journal_loot(ui:&mut Ui,items:&mut Vec<crate::game_journal::Loot>)->bool{
 let mut changed=false;let mut remove=None;
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"LOOT & DROPS");caption(ui,"Manual inventory log, independent of the game.");
  if ui.add_enabled(items.len()<crate::game_journal::MAX,egui::Button::new("Add loot item")).clicked(){items.push(crate::game_journal::Loot{name:"New item".into(),..Default::default()});changed=true;}
  for(i,x)in items.iter_mut().enumerate(){ui.push_id(i,|ui|{
   ui.horizontal_wrapped(|ui|{
    changed|=ui.add(egui::TextEdit::singleline(&mut x.name).desired_width(190.).char_limit(80)).changed();
    if ui.small_button("-").clicked(){x.quantity=x.quantity.saturating_sub(1);changed=true;}
    changed|=ui.add(egui::DragValue::new(&mut x.quantity).range(0..=1_000_000)).changed();
    if ui.small_button("+").clicked(){x.quantity=x.quantity.saturating_add(1).min(1_000_000);changed=true;}
    if ui.small_button("Remove").clicked(){remove=Some(i);}
   });
   changed|=ui.add(egui::TextEdit::singleline(&mut x.note).desired_width(ui.available_width()).char_limit(600).hint_text("Source / drop location")).changed();ui.separator();
  });}
 });
 if let Some(i)=remove{items.remove(i);changed=true;}changed
}
fn journal_counters(ui:&mut Ui,items:&mut Vec<crate::game_journal::Counter>)->bool{
 let mut changed=false;let mut remove=None;
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"COUNTERS & MILESTONES");caption(ui,"Track manual farming counts, kills, resources or personal goals.");
  if ui.add_enabled(items.len()<crate::game_journal::MAX,egui::Button::new("Add counter")).clicked(){items.push(crate::game_journal::Counter{name:"New goal".into(),goal:100,value:0});changed=true;}
  for(i,x)in items.iter_mut().enumerate(){ui.push_id(i,|ui|{
   changed|=ui.add(egui::TextEdit::singleline(&mut x.name).char_limit(80)).changed();
   ui.horizontal_wrapped(|ui|{
    if ui.small_button("-").clicked(){x.value=x.value.saturating_sub(1);changed=true;}
    changed|=ui.add(egui::DragValue::new(&mut x.value).range(0..=1_000_000)).changed();
    if ui.small_button("+").clicked(){x.value=x.value.saturating_add(1).min(1_000_000);changed=true;}
    ui.label("Goal");changed|=ui.add(egui::DragValue::new(&mut x.goal).range(1..=1_000_000)).changed();
    if ui.small_button("Remove").clicked(){remove=Some(i);}
   });
   ui.add(egui::ProgressBar::new((x.value as f32/x.goal.max(1) as f32).min(1.)).text(format!("{}/{}",x.value,x.goal)));
   ui.separator();
  });}
 });
 if let Some(i)=remove{items.remove(i);changed=true;}changed
}
fn journal_calculators(ui:&mut Ui){
 use crate::game_journal::*;
 let id=ui.id().with("journal-calc");let mut v=ui.data(|d|d.get_temp::<(u32,u32,u32,f64,u32,f64,f64,f64)>(id)).unwrap_or((10,4,12,1.,100,95.,12000.,30.));
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"CRAFTING MATERIALS");
  ui.horizontal_wrapped(|ui|{ui.label("Crafts");ui.add(egui::DragValue::new(&mut v.0).range(0..=1_000_000));ui.label("Material / craft");ui.add(egui::DragValue::new(&mut v.1).range(0..=1_000_000));ui.label("Owned");ui.add(egui::DragValue::new(&mut v.2).range(0..=1_000_000));});
  let(t,missing)=material(v.0,v.1,v.2);ui.label(RichText::new(format!("Required: {t}    Missing: {missing}")).strong().color(RED));
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"DROP CHANCE");
  caption(ui,"Independent rolls with fixed probability; does not model pity or changing drop rates.");
  ui.horizontal_wrapped(|ui|{ui.label("Chance per roll");ui.add(egui::DragValue::new(&mut v.3).range(0.0..=100.).speed(0.01).suffix("%"));ui.label("Rolls");ui.add(egui::DragValue::new(&mut v.4).range(0..=1_000_000));ui.label("Target");ui.add(egui::DragValue::new(&mut v.5).range(0.01..=99.99).suffix("%"));});
  if let Some(x)=chance_at_least_one(v.3,v.4){ui.label(RichText::new(format!("At least one drop: {x:.3}%")).strong().color(RED));}
  ui.label(format!("Rolls for target: {}",rolls_for_chance(v.3,v.5).map(|n|n.to_string()).unwrap_or("Not reachable".into())));
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());heading(ui,"DAMAGE / DPS");
  caption(ui,"Enter your own measured damage and duration; does not read live game data.");
  ui.horizontal_wrapped(|ui|{ui.label("Total damage");ui.add(egui::DragValue::new(&mut v.6).range(0.0..=1e15));ui.label("Seconds");ui.add(egui::DragValue::new(&mut v.7).range(0.01..=86400.).speed(0.1));});
  if let Some(x)=dps(v.6,v.7){ui.label(RichText::new(format!("{x:.2} damage / second")).strong().color(RED));}
 });
 ui.data_mut(|d|d.insert_temp(id,v));
}
fn game_journal_panel(app:&mut BoostApp,ui:&mut Ui){
 let key=app.profile_key();
 let label=app.telem.active_game.as_ref().map(|g|g.title.as_str()).unwrap_or("Global notebook (no game selected)");
 caption(ui,&format!("Saved locally / {label}"));
 if app.extras_view==13{journal_calculators(ui);ui.add_space(8.);pity_panel(ui);return;}
 let mut book=app.settings.journal_profiles.get(&key).cloned().unwrap_or_default();
 let mut changed=false;
 ui.push_id(&key,|ui|{match app.extras_view{
  10=>{changed|=journal_bosses(ui,&mut book.bosses);ui.add_space(8.);changed|=journal_loot(ui,&mut book.loot);ui.add_space(8.);changed|=journal_entries(ui,"FARMING ROUTES",&mut book.routes,true);}
  11=>{changed|=journal_entries(ui,"BUILDS / LOADOUTS",&mut book.builds,false);ui.add_space(8.);changed|=journal_entries(ui,"QUICK GUIDES",&mut book.guides,false);}
  12=>{changed|=journal_counters(ui,&mut book.counters);ui.add_space(8.);changed|=journal_entries(ui,"ACHIEVEMENTS / PROGRESS",&mut book.achievements,true);}
  _=>{}
 }});
 if changed&&!app.is_smoke{
  // Keep whitespace while editing; validation on loading, lengths constrained by widgets.
  app.settings.journal_profiles.insert(key,book);app.save();
 }
}

fn utility_panel(a:&mut BoostApp,ui:&mut Ui){
 use crate::utility_book::*;
 let key=a.profile_key();let now=now();let mut book=a.settings.utility_profiles.get(&key).cloned().unwrap_or_default();
 let mut changed=false;let mut remove=None;
 caption(ui,"Local per-game records. No automatic game interaction.");
 ui.push_id((&key,a.extras_view),|ui|{frame(PANEL).show(ui,|ui|{
 ui.set_min_width(ui.available_width());
 match a.extras_view{
 14=>{
 heading(ui,"COOLDOWNS / SPLIT TIMERS");caption(ui,"Manual timers continue by system clock across app restarts. No automated input.");
 if ui.add_enabled(book.timers.len()<MAX,egui::Button::new("Add timer")).clicked(){book.timers.push(Timer::default());changed=true;}
 for(i,v)in book.timers.iter_mut().enumerate(){ui.push_id(i,|ui|{
 changed|=ui.add(egui::TextEdit::singleline(&mut v.name).char_limit(80)).changed();
 let left=v.left(now);ui.label(RichText::new(format!("{}:{:02}",left/60,left%60)).size(25.).color(RED));
 ui.add(egui::ProgressBar::new(left as f32/v.duration.max(1) as f32));
 ui.horizontal_wrapped(|ui|{
 if ui.add(egui::DragValue::new(&mut v.duration).range(1..=86400).suffix(" s")).changed(){v.reset();changed=true;}
 if ui.button("Start / resume").clicked(){v.start(now);changed=true;}
 if ui.button("Pause").clicked(){v.pause(now);changed=true;}
 if ui.button("Reset").clicked(){v.reset();changed=true;}
 if ui.button("Split").clicked(){v.split(now);changed=true;}
 if ui.small_button("Remove").clicked(){remove=Some(i);}
 });if !v.splits.is_empty(){caption(ui,&format!("Splits (elapsed seconds): {:?}",v.splits));}
 ui.separator();if v.deadline.is_some()&&left>0{ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));}
 });}if let Some(i)=remove{book.timers.remove(i);changed=true;}
 }
 15=>{
 heading(ui,"DAILY / WEEKLY QUESTS");caption(ui,"Choose reset hour in UTC, weekly day Monday=0. Completed marks expire at configured boundary.");
 if ui.add_enabled(book.quests.len()<MAX,egui::Button::new("Add recurring quest")).clicked(){book.quests.push(Quest::default());changed=true;}
 for(i,v)in book.quests.iter_mut().enumerate(){ui.push_id(i,|ui|{
 let mut done=v.done(now);
 ui.horizontal_wrapped(|ui|{
 if ui.checkbox(&mut done,"").changed(){v.set_done(now,done);changed=true;}
 changed|=ui.add(egui::TextEdit::singleline(&mut v.name).char_limit(80)).changed();
 if ui.small_button("Remove").clicked(){remove=Some(i);}
 });
 ui.horizontal_wrapped(|ui|{
 changed|=ui.checkbox(&mut v.weekly,"Weekly").changed();ui.label("Reset UTC");changed|=ui.add(egui::DragValue::new(&mut v.hour).range(0..=23).suffix(":00")).changed();
 if v.weekly{egui::ComboBox::from_id_salt("weekday").selected_text(["Mon","Tue","Wed","Thu","Fri","Sat","Sun"][v.weekday.min(6) as usize]).show_ui(ui,|ui|{for(i,n)in ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"].iter().enumerate(){changed|=ui.selectable_value(&mut v.weekday,i as u8,*n).changed();}});}
 });ui.separator();
 });}if let Some(i)=remove{book.quests.remove(i);changed=true;}
 ui.ctx().request_repaint_after(std::time::Duration::from_secs(30));
 }
 16=>{
 heading(ui,"RESOURCE / ENERGY TRACKER");caption(ui,"Manual amount + one unit every selected interval. Uses wall clock, not live game data.");
 if ui.add_enabled(book.resources.len()<MAX,egui::Button::new("Add resource")).clicked(){book.resources.push(Resource::default());changed=true;}
 for(i,v)in book.resources.iter_mut().enumerate(){ui.push_id(i,|ui|{
 changed|=ui.add(egui::TextEdit::singleline(&mut v.name).char_limit(80)).changed();
 let amount=v.current(now);ui.add(egui::ProgressBar::new(amount as f32/v.cap.max(1) as f32).text(format!("{amount} / {}",v.cap)));
 let mut cap=v.cap;let mut interval=v.interval;let mut value=amount;
 ui.horizontal_wrapped(|ui|{
 ui.label("Capacity");if ui.add(egui::DragValue::new(&mut cap).range(1..=1_000_000)).changed(){v.settle(now);v.cap=cap;v.amount=v.amount.min(cap);changed=true;}
 ui.label("Seconds / unit");if ui.add(egui::DragValue::new(&mut interval).range(1..=86400)).changed(){v.settle(now);v.interval=interval;v.anchor=now;changed=true;}
 ui.label("Set amount");if ui.add(egui::DragValue::new(&mut value).range(0..=v.cap)).changed(){v.amount=value;v.anchor=now;changed=true;}
 if ui.button("-1").clicked(){v.change(now,-1);changed=true;}if ui.button("+1").clicked(){v.change(now,1);changed=true;}
 if ui.small_button("Remove").clicked(){remove=Some(i);}
 });ui.separator();if amount<v.cap{ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));}
 });}if let Some(i)=remove{book.resources.remove(i);changed=true;}
 }
 17=>{
 heading(ui,"STRUCTURED LOADOUTS");caption(ui,"Custom named equipment slots for any game. Earlier build notes remain available.");
 if ui.add_enabled(book.loadouts.len()<MAX,egui::Button::new("New loadout")).clicked(){book.loadouts.push(Loadout{name:"New build".into(),slots:vec![Slot{slot:"Weapon".into(),item:String::new()},Slot{slot:"Armor".into(),item:String::new()}],notes:String::new()});changed=true;}
 for(i,v)in book.loadouts.iter_mut().enumerate(){ui.push_id(i,|ui|{
 ui.horizontal_wrapped(|ui|{changed|=ui.add(egui::TextEdit::singleline(&mut v.name).char_limit(80)).changed();if ui.small_button("Remove loadout").clicked(){remove=Some(i);}});
 if ui.add_enabled(v.slots.len()<24,egui::Button::new("Add equipment slot")).clicked(){v.slots.push(Slot::default());changed=true;}
 let mut del=None;for(j,s)in v.slots.iter_mut().enumerate(){ui.push_id(j,|ui|{ui.horizontal_wrapped(|ui|{
 changed|=ui.add(egui::TextEdit::singleline(&mut s.slot).hint_text("Slot").desired_width(100.).char_limit(60)).changed();
 changed|=ui.add(egui::TextEdit::singleline(&mut s.item).hint_text("Equipment / skill").desired_width(220.).char_limit(120)).changed();
 if ui.small_button("Remove").clicked(){del=Some(j);}
 });});}if let Some(j)=del{v.slots.remove(j);changed=true;}
 changed|=ui.add(egui::TextEdit::multiline(&mut v.notes).hint_text("Build notes").desired_width(ui.available_width()).char_limit(1600)).changed();ui.separator();
 });}if let Some(i)=remove{book.loadouts.remove(i);changed=true;}
 }
 _=>{}
 }
 });});
 if changed&&!a.is_smoke{book.sanitize();a.settings.utility_profiles.insert(key,book);a.save();}
}
fn pity_panel(ui:&mut Ui){
 let id=egui::Id::new("pity-calc-v113");let mut v=ui.data(|d|d.get_temp::<(f64,u32,f64,u32,u32,u32)>(id)).unwrap_or((1.,70,5.,90,0,10));
 frame(PANEL).show(ui,|ui|{heading(ui,"PITY / GUARANTEED DROP MODEL");caption(ui,"User-entered model: fixed base chance, linear increase after soft threshold, guaranteed drop on hard threshold. Not a claim about any specific game.");
 ui.horizontal_wrapped(|ui|{ui.label("Base %");ui.add(egui::DragValue::new(&mut v.0).range(0.0..=100.).speed(0.1));ui.label("Soft after");ui.add(egui::DragValue::new(&mut v.1).range(0..=10000));ui.label("Increase / roll %");ui.add(egui::DragValue::new(&mut v.2).range(0.0..=100.));});
 ui.horizontal_wrapped(|ui|{ui.label("Hard guarantee");ui.add(egui::DragValue::new(&mut v.3).range(1..=10000));ui.label("Already failed");ui.add(egui::DragValue::new(&mut v.4).range(0..=10000));ui.label("Next rolls");ui.add(egui::DragValue::new(&mut v.5).range(0..=10000));});
 if let Some(p)=crate::utility_book::pity(v.0,v.1,v.2,v.3,v.4,v.5){ui.label(RichText::new(format!("Chance of at least one: {p:.3}%")).color(RED).strong());}else{caption(ui,"Check thresholds and failed rolls.");}
 });ui.data_mut(|d|d.insert_temp(id,v));
}

#[derive(Clone,Default)]struct LibraryHubState{key:Option<String>,query:String,favorites:bool,restore:Option<crate::library_hub::Backup>,restore_key:Option<String>,undo:Option<(String,crate::library_hub::Backup)>,message:String}
fn library_hub_panel(a:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("library-hub-v114");let mut st=ui.data(|d|d.get_temp::<LibraryHubState>(id)).unwrap_or_default();
 let active=a.profile_key();let selected=st.key.clone().unwrap_or_else(||active.clone());
 frame(PANEL).show(ui,|ui|{
 ui.set_min_width(ui.available_width());heading(ui,"GAME HUB / COLLECTIONS");
 ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut st.query).hint_text("Search game, tags, collection").char_limit(100));ui.checkbox(&mut st.favorites,"Favorites");if ui.button("Current game").clicked(){st.key=None;st.restore=None;}});
 let mut entries:Vec<(String,String)>=a.settings.curated_games.iter().map(|e|(crate::library_hub::entry_key(e),e.title.clone())).collect();
 if !entries.iter().any(|(k,_)|k==&active){entries.insert(0,(active.clone(),a.telem.active_game.as_ref().map(|g|g.title.clone()).unwrap_or_else(||"Global notebook".into())));}
 egui::ScrollArea::vertical().id_salt("library-hub-list").max_height(150.).show(ui,|ui|{
 for(k,name)in &entries{let meta=a.settings.library_meta.get(k).cloned().unwrap_or_default();if !meta.matches(name,&st.query)||(st.favorites&&!meta.favorite){continue;}if ui.selectable_label(&selected==k,format!("{}{}",if meta.favorite{"* "}else{""},name)).clicked(){st.key=Some(k.clone());st.restore=None;}}
 });
 let key=st.key.clone().unwrap_or(active.clone());let title=entries.iter().find(|(k,_)|k==&key).map(|(_,n)|n.as_str()).unwrap_or("Saved profile");
 heading(ui,title);
 let mut meta=a.settings.library_meta.get(&key).cloned().unwrap_or_default();let mut change=false;
 ui.horizontal_wrapped(|ui|{change|=ui.checkbox(&mut meta.favorite,"Favorite").changed();for(i,n)in ["Backlog","Playing","Completed","Paused"].iter().enumerate(){change|=ui.selectable_value(&mut meta.status,i as u8,*n).changed();}change|=ui.add(egui::Slider::new(&mut meta.rating,0..=10).text("Rating")).changed();});
 change|=ui.add(egui::TextEdit::singleline(&mut meta.tags).hint_text("Tags").char_limit(160)).changed();
 change|=ui.add(egui::TextEdit::singleline(&mut meta.collection).hint_text("Collection").char_limit(80)).changed();
 let (n,_,longest)=a.sessions.summary(&key);let secs=a.sessions.played_seconds(&key);
 ui.label(format!("Tracked playtime {:.1} h / {n} finished sessions / longest {:.1} min",secs as f64/3600.,longest as f64/60.));
 if let Some(last)=a.sessions.history.iter().rev().find(|v|v.game_key.eq_ignore_ascii_case(&key)){let days=crate::utility_book::now().saturating_sub(last.started_unix)/86400;caption(ui,&format!("Last recorded session started {days} day(s) ago"));}
 let clips=a.settings.clip_annotations.values().filter(|v|v.game==key).count();
 let tools=a.settings.utility_profiles.get(&key).map(|v|v.timers.len()+v.quests.len()+v.resources.len()+v.loadouts.len()).unwrap_or(0);
 ui.label(format!("{clips} annotated media / {tools} utility entries"));
 if let Some(p)=a.settings.render_preferences.get(&key){caption(ui,&format!("Saved rendering: {} / {:.2}x / cap {} / target {}",crate::spatial::MODES.get(p.scale_mode as usize).unwrap_or(&"Unknown"),p.scale_ratio,p.cap,p.target));}
 egui::CollapsingHeader::new("Related clip records").id_salt((&key,"clips")).show(ui,|ui|{for(k,v)in a.settings.clip_annotations.iter().filter(|(_,v)|v.game==key).take(40){ui.horizontal_wrapped(|ui|{ui.label(if v.title.is_empty(){k}else{&v.title});if ui.small_button("Copy filename").clicked(){ui.ctx().copy_text(k.clone());}});}});
 ui.horizontal_wrapped(|ui|{ui.label("Preferred workspace");for(i,n)in ["Boost","Game Tools","Capture","Macro Studio"].iter().enumerate(){change|=ui.selectable_value(&mut meta.workspace,i as u8,*n).changed();}});
 if key==active&&ui.button("Open preferred workspace").clicked(){match meta.workspace{1=>{a.page=Page::Extras;a.extras_view=10;},2=>a.page=Page::Capture,3=>{a.page=Page::Extras;a.extras_view=5;},_=>{a.page=Page::Performance;a.performance_view=0;}}}
 let mut note=a.settings.game_notes.get(&key).cloned().unwrap_or_default();
 if ui.add(egui::TextEdit::multiline(&mut note).hint_text("Game notes").desired_rows(2).char_limit(4000).desired_width(ui.available_width())).changed()&&!a.is_smoke{a.settings.game_notes.insert(key.clone(),note);a.save();}
 egui::CollapsingHeader::new("Launch checklist").id_salt(&key).show(ui,|ui|{change|=journal_entries(ui,"Before playing",&mut meta.checklist,true);});
 if meta.checklist.len()>20{meta.checklist.truncate(20);}
 if change&&!a.is_smoke{meta.sanitize();a.settings.library_meta.insert(key.clone(),meta);a.save();}
 if key==active{ui.horizontal_wrapped(|ui|{if ui.button("Game tools").clicked(){a.page=Page::Extras;a.extras_view=10;}if ui.button("Boost").clicked(){a.page=Page::Performance;a.performance_view=0;}if ui.button("Capture").clicked(){a.page=Page::Capture;}});}else{caption(ui,"Offline profile: notes, organization and backup are available; live controls require selecting the running game.");}
 egui::CollapsingHeader::new("Local utility backup / restore").show(ui,|ui|{
 caption(ui,"Export notes, utility records, game metadata and clip annotations (not videos, executable, or Windows settings). Import previews data before replacing the selected notebook.");
 if ui.button("Save backup JSON").clicked()&&!a.is_smoke{
  let b=crate::library_hub::Backup::from(&a.settings,&key);
  st.message=(||->Result<String,String>{let raw=serde_json::to_vec_pretty(&b).map_err(|e|e.to_string())?;if raw.len()>2*1024*1024{return Err("Backup exceeds 2 MiB".into());}let dir=crate::settings::folder().join("utility_backups");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let path=dir.join(format!("HBoost_Utility_{}.json",crate::media_tools::stamp()));crate::settings::atomic_json_replace(&path,&raw)?;Ok(format!("Saved {}",path.display()))})().unwrap_or_else(|e|e);
 }
 if ui.button("Choose backup JSON").clicked()&&!a.is_smoke{if let Some(path)=crate::filedialog::select_profile(){
  st.restore=None;let raw=std::fs::metadata(&path).map_err(|e|e.to_string()).and_then(|m|if m.len()<=2*1024*1024{std::fs::read_to_string(&path).map_err(|e|e.to_string())}else{Err("Backup exceeds 2 MiB".into())}).and_then(|s|crate::library_hub::Backup::decode(&s));match raw{Ok(v)=>{st.restore=Some(v);st.restore_key=Some(key.clone());st.message="Backup validated; not applied".into();},Err(e)=>st.message=e}
 }}
 if let Some(v)=&st.restore{ui.label(format!("Preview: {} timers, {} quests, {} resources, {} loadouts, {} clip records",v.utility.timers.len(),v.utility.quests.len(),v.utility.resources.len(),v.utility.loadouts.len(),v.clips.len()));if ui.add_enabled(st.restore_key.as_ref()==Some(&key),egui::Button::new("Confirm replace selected notebook")).clicked()&&!a.is_smoke{if let Some(v)=st.restore.take(){st.undo=Some((key.clone(),crate::library_hub::Backup::from(&a.settings,&key)));v.apply(&mut a.settings,&key);a.save();st.message="Selected notebook restored; other games preserved".into();}}if ui.button("Cancel restore").clicked(){st.restore=None;}}
 if ui.add_enabled(st.undo.as_ref().is_some_and(|(k,_)|k==&key),egui::Button::new("Undo notebook restore (session)")).clicked()&&!a.is_smoke{if let Some((_,v))=st.undo.take(){v.apply(&mut a.settings,&key);a.save();st.message="Previous notebook fields restored; newly imported clip records retained".into();}}
 if !st.message.is_empty(){caption(ui,&st.message);}
 });
 });ui.data_mut(|d|d.insert_temp(id,st));ui.add_space(10.);
}
