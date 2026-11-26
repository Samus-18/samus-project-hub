// Horizon Control Center: one primary destination per feature family.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
enum SuiteArea{Home,Boost,Visual,Tools,Macros,Overlay,Capture,Library,System,Settings,Lab}
const SUITE_AREAS:[SuiteArea;11]=[SuiteArea::Home,SuiteArea::Boost,SuiteArea::Visual,SuiteArea::Tools,SuiteArea::Macros,SuiteArea::Overlay,SuiteArea::Capture,SuiteArea::Library,SuiteArea::System,SuiteArea::Settings,SuiteArea::Lab];
impl SuiteArea{
 fn name(self)->&'static str{match self{Self::Home=>"Home",Self::Boost=>"Boost",Self::Visual=>"Visual",Self::Tools=>"Game Tools",Self::Macros=>"Macro Studio",Self::Overlay=>"Overlay",Self::Capture=>"Capture",Self::Library=>"Library",Self::System=>"System Tune",Self::Settings=>"Settings",Self::Lab=>"Lab"}}
 fn detail(self)->&'static str{match self{Self::Home=>"Your gaming control center",Self::Boost=>"Optimize / scale / motion / stability",Self::Visual=>"Filters, effects, color and shaders",Self::Tools=>"Crosshair, practice, timers and game utilities",Self::Macros=>"Record, edit and replay local sequences",Self::Overlay=>"HUD widgets and layout",Self::Capture=>"Screenshots, recordings and replay",Self::Library=>"Games, profiles and saves",Self::System=>"Hardware analysis and reversible system tuning",Self::Settings=>"Appearance, shortcuts and safety",Self::Lab=>"Hardware proofs and unfinished integrations"}}
 fn icon(self)->Page{match self{Self::Home=>Page::Home,Self::Boost=>Page::Performance,Self::Visual=>Page::Filters,Self::Tools=>Page::Extras,Self::Macros=>Page::Settings,Self::Overlay=>Page::Overlay,Self::Capture=>Page::Capture,Self::Library=>Page::Library,Self::System=>Page::Scaling,Self::Settings=>Page::Settings,Self::Lab=>Page::Unsorted}}
 fn go(self,a:&mut BoostApp){match self{
 Self::Home=>a.page=Page::Home,Self::Boost=>{a.page=Page::Performance;a.performance_view=0;},
 Self::Visual=>a.page=Page::Filters,Self::Tools=>{a.page=Page::Extras;if a.extras_view==5{a.extras_view=0;}},
 Self::Macros=>{a.page=Page::Extras;a.extras_view=5;},
 Self::Overlay=>a.page=Page::Overlay,Self::Capture=>a.page=Page::Capture,Self::Library=>a.page=Page::Library,
 Self::System=>{a.page=Page::Performance;a.performance_view=1;},Self::Settings=>a.page=Page::Settings,Self::Lab=>{a.page=Page::Unsorted;a.unsorted_selected_panel=255;}
 }}
}
fn suite_current(a:&BoostApp)->SuiteArea{match a.page{
 Page::Home=>SuiteArea::Home,Page::Filters|Page::Scaling|Page::FrameGen=>SuiteArea::Visual,
 Page::Performance=>if matches!(a.performance_view,1|4){SuiteArea::System}else{SuiteArea::Visual},
 Page::Extras=>if a.extras_view==5{SuiteArea::Macros}else{SuiteArea::Tools},
 Page::Overlay=>SuiteArea::Overlay,Page::Capture=>SuiteArea::Capture,Page::Library=>SuiteArea::Library,
 Page::Settings=>SuiteArea::Settings,Page::Unsorted=>SuiteArea::Lab}}
fn suite_owner(id:&str)->SuiteArea{
 match id.parse::<u16>().unwrap_or(0){
 195=>SuiteArea::Home,
 3|5|7|9|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|41|42|45|48|63|74|90|94|98|109|110|111|112|113|114|119|120|121|122|123|124|125|126|127|128|129|130|159|162|163|164|165|166=>SuiteArea::Boost,
 27|28|29|30|31|32|33|34|35|36|37|38|39|40|55|61|62|64|65|66|67|68|69|70|71|72|73|75|78|87|88|95|131|132|133|134|135|136|137|138|139|140|141|142|143|144|145|146|147=>SuiteArea::Visual,
 8|50|51|86|167|168|169|176|177|178|179|180=>SuiteArea::Tools,
 170|171|172|173=>SuiteArea::Macros,
 4|49|91|151|152|153|154|155|156|184=>SuiteArea::Overlay,
 1|2|52|89|107|181|182|183|185|186|187|188|189|190=>SuiteArea::Capture,
 53|92|93|97|148|149|150|161|175|191|192|193|194=>SuiteArea::Library,
 43|44|46|47|82|83|84|85|96|157|158=>SuiteArea::System,
 6|54|99|174|196|197|198|200=>SuiteArea::Settings,
 10|56|57|58|59|60|76|77|79|80|81|100|101|102|103|104|105|106|108|115|116|117|118|160|199=>SuiteArea::Lab,
 _=>SuiteArea::Lab}
}
fn suite_sidebar(a:&mut BoostApp,ui:&mut Ui){
 ui.add_space(7.);
 ui.label(RichText::new("HORIZON / CONTROL CENTER").size(9.5).strong().color(MUTED));
 ui.add_space(8.);
 for(area,label)in [(SuiteArea::Home,rm(a,"WORKSPACE","PŘEHLED")),(SuiteArea::Boost,rm(a,"PLAY","HRANÍ")),(SuiteArea::System,rm(a,"MANAGE","SPRÁVA"))]{
  ui.add_space(5.);ui.label(RichText::new(label).size(10.).color(E_DIM).strong());
  let entries:&[SuiteArea]=match area{SuiteArea::Home=>&[SuiteArea::Home],SuiteArea::Boost=>&[SuiteArea::Boost,SuiteArea::Visual,SuiteArea::Tools,SuiteArea::Macros,SuiteArea::Overlay,SuiteArea::Capture],_=>&[SuiteArea::Library,SuiteArea::System,SuiteArea::Settings,SuiteArea::Lab]};
  for &x in entries{
   if x==SuiteArea::Lab&&!a.settings.ui_advanced{continue;}
   ui.push_id(x.name(),|ui|{
    let active=suite_current(a)==x;let(r,res)=ui.allocate_exact_size(vec2(ui.available_width(),32.),Sense::click());
    let t=motion_value(ui,res.id.with("suite-hover"),res.hovered(),0.14);
    let selected=motion_value(ui,res.id.with("suite-select"),active,0.18);
    let press=motion_value(ui,res.id.with("suite-press"),res.is_pointer_button_down_on(),0.09);
    let p=ui.painter();p.rect_filled(r,6.,Color32::from_rgb((12.+32.*selected+10.*t)as u8,(12.+3.*t)as u8,(14.+5.*selected)as u8));
    if selected>0.001{p.rect_filled(Rect::from_center_size(r.left_center()+vec2(1.5,0.),vec2(3.,23.*selected)),1.,RED);}
    modern_icon(p,r.left_center()+vec2(18.,0.),x.icon(),if active{RED}else{MUTED},motion_icon_scale(t,press,0.76));
    p.text(r.left_center()+vec2(35.,0.),egui::Align2::LEFT_CENTER,rm_name(a,x),FontId::proportional(12.5),if active{WHITE}else{MUTED});
    if res.on_hover_text(rm_detail(a,x)).clicked(){x.go(a);}
   });
   ui.add_space(3.);
  }
 }
 ui.separator();
 ui.horizontal_wrapped(|ui|{for(i,n)in ["EN","CZ"].iter().enumerate(){if ui.selectable_value(&mut a.settings.ui_language,i as u8,*n).changed(){a.save();}}});
 if ui.small_button(if a.settings.ui_advanced{rm(a,"Simple controls","Jednoduché ovládání")}else{rm(a,"Advanced controls","Pokročilé ovládání")}).clicked(){a.settings.ui_advanced=!a.settings.ui_advanced;a.save();}
}
fn suite_compact(a:&mut BoostApp,ui:&mut Ui){
 egui::ScrollArea::horizontal().id_salt("suite-compact").show(ui,|ui|ui.horizontal(|ui|{
  for x in SUITE_AREAS{let res=ui.selectable_label(suite_current(a)==x,rm_name(a,x));motion_button(ui,&res);if res.clicked(){x.go(a);}}
  if ui.button(rm(a,"Search","Hledat")).clicked(){a.palette_open=true;}
 }));
}
fn suite_catalog(a:&mut BoostApp,ui:&mut Ui,area:SuiteArea){
 let all=crate::unsorted_catalog::entries();
 let total=all.iter().filter(|e|suite_owner(&e.id)==area).count();
 if total==0{return;}
 egui::CollapsingHeader::new(format!("Feature index / {total} original requests")).id_salt(("suite-index",area.name())).default_open(false).show(ui,|ui|{
  caption(ui,"Original IDs and implementation status retained. Integrated code is not certification in every game.");
  let id=ui.id().with("suite-index-search");let mut query=ui.data(|d|d.get_temp::<String>(id)).unwrap_or_default();
  ui.add(egui::TextEdit::singleline(&mut query).hint_text("Find feature by name or original ID").desired_width(ui.available_width().min(360.)));
  ui.data_mut(|d|d.insert_temp(id,query.clone()));
  for e in crate::unsorted_catalog::matching(all,&query,0).into_iter().filter(|e|suite_owner(&e.id)==area){
   ui.horizontal_wrapped(|ui|{ui.label(RichText::new(format!("#{} {}",e.id,e.request)).size(12.));
    ui.label(RichText::new(match e.status.as_str(){"INTEGRATED_NOT_GAME_VERIFIED"=>"Integrated / game unverified","PARTIAL"=>"Partial","LEGACY_ONLY_NOT_NATIVE"=>"Legacy only",_=>"Not implemented"}).size(10.).color(if e.status=="INTEGRATED_NOT_GAME_VERIFIED"{MUTED}else{RED}));});
   ui.separator();
  }
  let _=a;
 });
}
fn suite_card(a:&mut BoostApp,ui:&mut Ui,area:SuiteArea,w:f32){
 let detail=ui.painter().layout(rm_detail(a,area).to_string(),FontId::proportional(10.5),MUTED,(w-72.).max(20.));let height=(44.+detail.size().y).max(70.);let(r,res)=ui.allocate_exact_size(vec2(w,height),Sense::click());let t=motion_value(ui,res.id.with("suite-card"),res.hovered(),0.15);
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,8.,Color32::from_rgb((18.+14.*t)as u8,15,18));
 p.rect_stroke(r,8.,Stroke::new(1.,if res.hovered(){RED}else{BORDER}),StrokeKind::Inside);
 p.rect_filled(Rect::from_min_size(r.left_top()+vec2(12.,15.),vec2(38.,38.)),8.,Color32::from_rgb(48,13,20));
 modern_icon(&p,r.left_top()+vec2(31.,34.),area.icon(),RED,0.95+0.08*t);
 p.text(r.left_top()+vec2(62.,13.),egui::Align2::LEFT_TOP,rm_name(a,area),FontId::proportional(15.),WHITE);
 p.galley(r.left_top()+vec2(62.,36.),detail,MUTED);
 if res.on_hover_text(rm_detail(a,area)).clicked(){area.go(a);}
}
fn suite_home(a:&mut BoostApp,ui:&mut Ui){
 let w=ui.available_width();
 let(r,_)=ui.allocate_exact_size(vec2(w,132.),Sense::hover());
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,10.,Color32::from_rgb(19,10,14));
 e_gradient(&p,r.shrink(2.),Color32::from_rgb(12,10,13),Color32::from_rgb(55,10,22));
 p.rect_stroke(r,10.,Stroke::new(1.,Color32::from_rgb(85,24,39)),StrokeKind::Inside);
 let x=r.right()-45.;for i in 0..5{p.line_segment([pos2(x-i as f32*18.,r.top()+12.),pos2(x-i as f32*18.-66.,r.bottom()-12.)],Stroke::new(1.,e_rgba(RED,30)));}
 p.text(r.left_top()+vec2(21.,17.),egui::Align2::LEFT_TOP,"HORIZON  /  GAMING SUITE",FontId::proportional(11.),RED);
 p.text(r.left_top()+vec2(20.,43.),egui::Align2::LEFT_TOP,"Your game. Your control.",FontId::proportional(if w<440.{23.}else{32.}),WHITE);
 p.text(r.left_top()+vec2(21.,93.),egui::Align2::LEFT_TOP,"Performance, visual tools, macros and more.",FontId::proportional(12.),MUTED);
 ui.add_space(10.);
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  ui.label(RichText::new("SESSION").size(10.).color(MUTED).strong());
  heading(ui,&a.telem.active_game.as_ref().map(|g|trim(&g.title,50)).unwrap_or_else(||"No game selected".into()));
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Choose game"){SuiteArea::Library.go(a);}
   if ui.add_enabled(!a.is_smoke&&a.telem.active_game.is_some(),egui::Button::new(if a.native_mirror.is_some(){"Stop GPU output"}else{"Start GPU output"})).clicked(){if a.native_mirror.is_some(){a.stop_native_mirror();}else{a.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);}}
   if outline_button(ui,if a.overlay_open{"Hide HUD"}else{"Show HUD"})&&!a.is_smoke{a.toggle_passive_hud();}
   if outline_button(ui,"Macro Studio"){SuiteArea::Macros.go(a);}
  });
  ui.horizontal_wrapped(|ui|{ui.label(format!("Game FPS: {}",fps_text(a,"fps")));ui.label(format!("Frame time: {}",fps_text(a,"frametime")));ui.label(format!("Own real FPS: {}",a.output_rates.snapshot(std::time::Instant::now()).map(|v|format!("{:.0}",v.real)).unwrap_or("--".into())));});
 });
 ui.add_space(12.);
 ui.label(RichText::new("PLAY / CREATE").size(11.).color(MUTED).strong());ui.add_space(5.);
 let cards=[SuiteArea::Tools,SuiteArea::Macros,SuiteArea::Boost,SuiteArea::Visual,SuiteArea::Overlay,SuiteArea::Capture,SuiteArea::Library,SuiteArea::System];
 let cols=if w>=610.{2}else{1};let cw=(w-8.*(cols-1)as f32)/cols as f32;
 for row in cards.chunks(cols){ui.horizontal(|ui|{for &area in row{suite_card(a,ui,area,cw);}});ui.add_space(6.);}
 suite_quick_controls(a,ui);
}
fn suite_boost(a:&mut BoostApp,ui:&mut Ui){
 ui.horizontal_wrapped(|ui|{
  for (name,page,view) in [("Optimize",Page::Performance,0usize),("Upscaling",Page::Scaling,0),("Frame generation",Page::FrameGen,0),("Analysis",Page::Performance,2),("Stability",Page::Performance,3)]{
   let active=a.page==page&&(page!=Page::Performance||a.performance_view==view);
   let res=ui.selectable_label(active,name);motion_button(ui,&res);
   if res.clicked(){a.page=page;if page==Page::Performance{a.performance_view=view;}}
  }
 });
 ui.add_space(10.);
 match a.page{
 Page::Scaling=>if a.settings.ui_advanced{v22_scaling(a,ui)}else{oled_scaling(a,ui)},
 Page::FrameGen=>if a.settings.ui_advanced{v22_motion(a,ui)}else{oled_motion(a,ui)},
 _=>match a.performance_view{2=>performance_analysis(a,ui),3=>performance_stability_panel(a,ui),_=>{optimizer_panel(a,ui);egui::CollapsingHeader::new("Live metrics").show(ui,|ui|performance_live_summary(a,ui));}}
 }
 suite_moved_boost(a,ui);
}
fn suite_game_tools(a:&mut BoostApp,ui:&mut Ui){
 // Keep all original extras_view IDs, but present only one tool at a time.
 // The interface no longer dumps ~20 buttons into a horizontal wall.
 let tools=[
 (0,0,"Crosshair","Zaměřovač","Custom on-screen pointer; does not aim for you.","Vlastní značka na obrazovce; sama nemíří."),
 (0,6,"Zoom & view","Zoom a obraz","Magnification, brightness and useful view helpers.","Přiblížení, jas a nástroje zobrazení."),
 (0,4,"Quick actions","Rychlé akce","Shortcuts and reversible on-screen conveniences.","Zkratky a vratné pomůcky na obrazovce."),
 (0,8,"Controller","Ovladač","Controller-related adjustments.","Nastavení pro ovladač."),
 (1,1,"Timers","Časovače","Simple countdowns and session breaks.","Odpočítávání a přestávky."),
 (1,7,"Reminders","Připomínky","Game-session alerts and notes.","Upozornění a poznámky."),
 (1,14,"Cooldowns","Cooldowny","Track cooldown timers for offline play.","Sledování časů obnovy schopností."),
 (1,15,"Daily / weekly","Denní / týdenní","Recurring local tasks.","Opakované místní úkoly."),
 (1,16,"Resources","Zdroje","Track materials and resource goals.","Přehled materiálů a cílů."),
 (2,2,"Quests & notes","Úkoly a poznámky","Your quest log.","Tvůj záznamník úkolů."),
 (2,10,"Boss & loot","Bossové a loot","Boss and reward checklist.","Seznam bossů a odměn."),
 (2,11,"Builds & guides","Buildy a návody","Store player builds and hints.","Uložení buildů a tipů."),
 (2,17,"Loadouts","Výbava","Remember loadouts.","Zapamatování výbavy."),
 (2,18,"Map notes","Poznámky mapy","Keep locations and map reminders.","Ukládání míst na mapě."),
 (3,3,"Practice","Trénink","Offline practice helpers.","Pomůcky pro offline trénink."),
 (3,9,"Session lab","Herní relace","Game session overview.","Přehled herní relace."),
 (3,12,"Goals","Cíle","Personal progress.","Osobní postup."),
 (3,13,"Calculators","Kalkulačky","Simple in-game calculators.","Jednoduché výpočty."),
 (3,19,"Session goals","Cíle relace","Goals for the current session.","Cíle právě hrané relace."),
 ];
 let mut group=tools.iter().find(|x|x.1==a.extras_view).map_or(0,|x|x.0);
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  heading(ui,rm(a,"Game tools / Choose a category","Herní nástroje / Vyber kategorii"));
  ui.horizontal_wrapped(|ui|{
   for(i,en,cs)in [(0,"Screen","Obrazovka"),(1,"Timers","Časovače"),(2,"Notes","Poznámky"),(3,"Practice","Trénink")]{
    if ui.selectable_label(group==i,rm(a,en,cs)).clicked(){
     group=i;
     if let Some(t)=tools.iter().find(|t|t.0==i){a.extras_view=t.1;}
    }
   }
  });
  ui.add_space(4.);
  let (name,desc)=tools.iter().find(|t|t.1==a.extras_view)
   .map(|t|(rm(a,t.2,t.3),rm(a,t.4,t.5))).unwrap_or((rm(a,"Choose a tool","Vyber nástroj"),""));
  egui::ComboBox::from_id_salt("hb-simple-tool-chooser")
   .selected_text(name).width(ui.available_width().min(350.)).show_ui(ui,|ui|{
    for t in tools.iter().filter(|t|t.0==group){
     ui.selectable_value(&mut a.extras_view,t.1,local(ui,t.2,t.3));
    }
   });
  caption(ui,desc);
  ui.horizontal_wrapped(|ui|{
   if ui.button(rm(a,"Open simple macros","Otevřít jednoduchá makra")).clicked(){SuiteArea::Macros.go(a);}
   if ui.button(rm(a,"Open HUD","Otevřít HUD")).clicked(){SuiteArea::Overlay.go(a);}
  });
  caption(ui,rm(a,"Only the chosen tool is shown below. Your previous tools and saved settings remain available.","Níže je jen vybraný nástroj. Původní funkce a uložená nastavení zůstávají dostupné."));
 });
 ui.add_space(7.);
 if a.extras_view==18{map_notes_panel(a,ui);}
 else if a.extras_view==19{session_goal_panel(a,ui);}
 else if a.extras_view>=14{utility_panel(a,ui);}
 else if a.extras_view>=10{game_journal_panel(a,ui);}
 else{game_extras_body(a,ui);
  if a.settings.ui_advanced{
   egui::CollapsingHeader::new(rm(a,"Advanced game controls","Pokročilé herní ovládání"))
    .default_open(false).show(ui,|ui|suite_moved_game_tools(a,ui));
  }
 }
}
fn suite_content(a:&mut BoostApp,ui:&mut Ui){
 let area=suite_current(a);
 if area!=SuiteArea::Home{
  ui.label(RichText::new(format!("CONTROL CENTER   /   {}",rm_name(a,area).to_uppercase())).size(10.).strong().color(RED));
  ui.label(RichText::new(rm_name(a,area)).size(28.).strong().color(WHITE));
  caption(ui,rm_detail(a,area));rm_help_panel(a,ui,area);ui.add_space(10.);
 }
 match area{
 SuiteArea::Home=>rm_home(a,ui),
 SuiteArea::Boost=>rm_boost(a,ui),
 SuiteArea::Visual=>{rm_visual(a,ui);rm_how_to_minecraft(a,ui);},
 SuiteArea::Tools=>suite_game_tools(a,ui),
 SuiteArea::Macros=>rm_macros(a,ui),
 SuiteArea::Overlay=>rm_overlay(a,ui),
 SuiteArea::Capture=>rm_capture(a,ui),
 SuiteArea::Library=>rm_library(a,ui),
 SuiteArea::System=>rm_system(a,ui),
 SuiteArea::Settings=>{rm_settings(a,ui);egui::CollapsingHeader::new(rm(a,"Feature handbook / 257 IDs","Příručka funkcí / 257 ID")).show(ui,|ui|rm_catalogue(a,ui));},
 SuiteArea::Lab=>{
  egui::CollapsingHeader::new(rm(a,"Feature handbook (257 items)","Příručka funkcí (257 položek)")).default_open(true).show(ui,|ui|rm_catalogue(a,ui));
  let c=crate::unsorted_catalog::counts();
  frame(PANEL).show(ui,|ui|{ui.set_min_width(ui.available_width());heading(ui,"Development status");ui.label(format!("{} / 200 integrated in source  |  {} partial  |  {} legacy  |  {} missing",c[0],c[1],c[2],c[3]));caption(ui,"No claim of game certification. Hardware tests are explicit and read-only.");});
  suite_moved_lab(a,ui);
  egui::CollapsingHeader::new("Open technical requirements (all workspaces)").show(ui,|ui|{
   for e in crate::unsorted_catalog::entries().iter().filter(|e|e.status!="INTEGRATED_NOT_GAME_VERIFIED"){
    ui.horizontal_wrapped(|ui|{ui.label(format!("#{} {}",e.id,e.request));if ui.small_button(suite_owner(&e.id).name()).clicked(){suite_owner(&e.id).go(a);}});ui.separator();
   }
  });
 }
 }
  if a.settings.ui_advanced||area==SuiteArea::Lab{
  egui::CollapsingHeader::new(rm(a,"All advanced features / original IDs","Všechny pokročilé funkce / původní ID"))
   .default_open(false).show(ui,|ui|suite_catalog(a,ui,area));
 }
}
#[cfg(test)]mod suite_tests{
 use super::*;
 #[test]fn all_200_have_exactly_one_explicit_owner_and_preserve_status(){
  let e=crate::unsorted_catalog::entries();let mut counts=[0usize;11];
  for v in e{let a=suite_owner(&v.id);counts[SUITE_AREAS.iter().position(|x|*x==a).unwrap()]+=1;}
  assert_eq!(counts.iter().sum::<usize>(),200);assert!(counts.iter().all(|n|*n>0));
  assert_eq!(crate::unsorted_catalog::counts(),[180,17,1,2]);
  assert_eq!(suite_owner("170"),SuiteArea::Macros);assert_eq!(suite_owner("185"),SuiteArea::Capture);
  assert_eq!(suite_owner("010"),SuiteArea::Lab);
 }
}
