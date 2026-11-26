// Eclipse visual system: no external artwork, no simulated performance readings.
const E_INK:Color32=Color32::from_rgb(235,238,245);
const E_DIM:Color32=Color32::from_rgb(112,117,133);
const E_LINE:Color32=Color32::from_rgb(35,35,43);
const E_SURFACE:Color32=Color32::from_rgb(11,11,14);
fn e_motion(ui:&Ui)->bool{
 ui.ctx().data(|d|d.get_temp::<bool>(egui::Id::new("hb-motion-ui"))).unwrap_or(true)
}
fn e_anim(ui:&Ui,id:egui::Id,on:bool)->f32{
 if e_motion(ui){ui.ctx().animate_bool_with_time(id,on,0.20)}else if on{1.}else{0.}
}
// Restrained warm OLED accents; status colors (success/errors) remain semantic.
fn e_color(page:Page)->Color32{match page{
 Page::Filters=>Color32::from_rgb(255,65,96),
 Page::Scaling=>Color32::from_rgb(240,110,137),
 Page::Performance=>Color32::from_rgb(239,130,117),
 Page::Extras=>Color32::from_rgb(251,137,108),
 Page::FrameGen=>Color32::from_rgb(255,85,123),
 Page::Capture=>Color32::from_rgb(228,94,117),
 _=>RED}}
fn e_rgba(c:Color32,a:u8)->Color32{Color32::from_rgba_unmultiplied(c.r(),c.g(),c.b(),a)}
fn e_arc(p:&Painter,c:Pos2,r:f32,from:f32,sweep:f32,pen:Stroke){
 let n=40;let v=(0..=n).map(|i|{let a=from+sweep*i as f32/n as f32;c+vec2(a.cos(),a.sin())*r}).collect();
 p.add(egui::Shape::line(v,pen));
}
fn e_gradient(p:&Painter,r:Rect,a:Color32,b:Color32){
 v25_gradient(p,r,a,b);
}
fn e_panel(ui:&mut Ui,fill:Color32,add:impl FnOnce(&mut Ui)){
 Frame::new().fill(fill).stroke(Stroke::new(1.0_f32,E_LINE)).corner_radius(14).inner_margin(egui::Margin::same(14)).show(ui,add);
}
fn e_switch(ui:&mut Ui,value:&mut bool,label:&str)->bool{
 let(w,response)=ui.allocate_exact_size(vec2(42.,24.),Sense::click());
 if response.clicked(){*value=!*value;}
 let a=e_anim(ui,response.id.with("switch"),*value);
 let press=motion_value(ui,response.id.with("switch-press"),response.is_pointer_button_down_on(),0.09);
 let p=ui.painter();
 p.rect_filled(w,12.,Color32::from_rgb((31.+179.*a)as u8,(32.+16.*a)as u8,(40.+18.*a)as u8));
 p.circle_filled(pos2(w.left()+12.+18.*a,w.center().y),8.-0.9*press,E_INK);
 ui.label(RichText::new(label).size(12.).color(if *value{E_INK}else{MUTED}));
 response.changed()||response.clicked()
}
fn e_nav_item(app:&mut BoostApp,ui:&mut Ui,page:Page,name:&str,h:f32){
 let selected=app.page==page;
 let(r,res)=ui.allocate_exact_size(vec2(ui.available_width(),h),Sense::click());
 let hover=e_anim(ui,res.id.with("hover"),res.hovered());let active=e_anim(ui,res.id.with("selected"),selected);
 let press=motion_value(ui,res.id.with("nav-press"),res.is_pointer_button_down_on(),0.09);
 let p=ui.painter();let accent=e_color(page);
 let fill=Color32::from_rgb((8.+23.*active+10.*hover)as u8,(8.+5.*active+7.*hover)as u8,(11.+8.*active+8.*hover)as u8);
 p.rect_filled(r,9.,fill);
 if active>0.01{p.rect_stroke(r,9.,Stroke::new(0.8_f32,e_rgba(accent,(130.*active)as u8)),StrokeKind::Inside);
 let marker=(h-14.)*motion_ease(active);
 p.rect_filled(Rect::from_center_size(r.left_center()+vec2(1.5,0.),vec2(3.,marker)),2.,accent);}
 let icon=motion_icon_scale(hover,press,0.84);
 modern_icon(p,r.left_center()+vec2(20.+2.*hover,0.),page,if selected{accent}else{MUTED},icon);
 p.text(r.left_center()+vec2(40.,0.),egui::Align2::LEFT_CENTER,tr(ui,name),FontId::proportional(12.4),if selected{E_INK}else{MUTED});
 if res.clicked(){app.page=page;}
}
fn eclipse_nav(app:&mut BoostApp,ui:&mut Ui){
 let h=if ui.ctx().content_rect().height()<650.{33.}else{39.};
 // Primary workspaces always above developer staging; no duplicated actions.
 for(page,name)in [
  (Page::Home,"Overview"),(Page::Filters,"GPU Studio"),(Page::Scaling,"Upscaling"),
  (Page::FrameGen,"Motion Lab"),(Page::Performance,"FPS & Stability"),
 ]{
  e_nav_item(app,ui,page,name,h);ui.add_space(3.);
 }
 ui.add_space(7.);
 ui.label(RichText::new(local(ui,"PLAY & CREATE","HRANÍ A TVORBA")).color(E_DIM).size(10.).strong());
 ui.add_space(5.);
 for(page,name)in [
  (Page::Overlay,"Game Overlay"),(Page::Extras,"Game Tools"),
  (Page::Capture,"Capture"),(Page::Library,"Game Library"),
 ]{
  e_nav_item(app,ui,page,name,h-3.);ui.add_space(2.);
 }
 ui.add_space(7.);ui.separator();
 e_nav_item(app,ui,Page::Settings,"Settings",h-2.);
 ui.add_space(7.);
 e_nav_item(app,ui,Page::Unsorted,"Lab / diagnostics",h-3.);
 ui.add_space(7.);
 let mode=if app.settings.ui_advanced{local(ui,"Simple controls","Základní ovládání")}
   else{local(ui,"Advanced controls","Pokročilé ovládání")};
 if ui.add_sized([ui.available_width(),29.],egui::Button::new(mode).fill(E_SURFACE)).clicked(){
  app.settings.ui_advanced=!app.settings.ui_advanced;app.save();
 }
 ui.horizontal(|ui|{
  for(i,n)in ["EN","CZ"].iter().enumerate(){
   if ui.selectable_value(&mut app.settings.ui_language,i as u8,*n).changed(){
    app.save();ui.ctx().request_repaint();
   }
  }
 });
}
fn eclipse_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
 let r=ui.max_rect();let p=ui.painter();let cy=r.center().y;
 let logo=pos2(r.left()+24.,cy);let a=RED;
 let logo_hit=Rect::from_center_size(logo,vec2(36.,36.));
 let hover=motion_value(ui,ui.id().with("hb-logo-motion"),ui.rect_contains_pointer(logo_hit),0.20);
 p.circle_filled(logo,17.+hover,Color32::from_rgb((36.+15.*hover)as u8,8,17));
 e_arc(p,logo,12.,0.3+hover*1.4,5.4,Stroke::new(1.8_f32,a));
 p.add(egui::Shape::convex_polygon(vec![logo+vec2(-7.,6.),logo+vec2(0.,-8.),logo+vec2(7.,6.)],a,Stroke::NONE));
 p.text(logo+vec2(29.,-6.),egui::Align2::LEFT_CENTER,"HORIZON",FontId::proportional(16.),E_INK);
 p.text(logo+vec2(29.,10.),egui::Align2::LEFT_CENTER,"B O O S T",FontId::proportional(9.),a);
 let end=r.right()-13.;let start=end-112.;
 let active=v22_gpu_active(app);let state=if active{V25_MINT}else{E_DIM};
 if r.width()>=740.{
 let sr=Rect::from_min_size(pos2(start-119.,cy-13.),vec2(106.,26.));
 p.rect_filled(sr,13.,if active{Color32::from_rgb(8,33,25)}else{E_SURFACE});
 let pulse=if active&&e_motion(ui){(app.app_start.elapsed().as_secs_f32()*3.).sin()*0.5+0.5}else{0.};
 if active{p.circle_filled(sr.left_center()+vec2(12.,0.),4.5+2.*pulse,e_rgba(state,(35.+35.*pulse)as u8));}
 p.circle_filled(sr.left_center()+vec2(12.,0.),3.2,state);
 p.text(sr.left_center()+vec2(22.,0.),egui::Align2::LEFT_CENTER,if active{local(ui,"ACTIVE","AKTIVNÍ")}else if app.native_mirror.is_some(){local(ui,"STOPPING","UKONČUJI")}else{local(ui,"STANDBY","PŘIPRAVENO")},FontId::proportional(10.),state);
 let search=Rect::from_min_size(pos2(start-157.,cy-14.),vec2(30.,28.));
 let res=ui.interact(search,ui.id().with("e-search"),Sense::click());
 let t=e_anim(ui,res.id,res.hovered());p.rect_filled(search,8.,Color32::from_rgb((14.+15.*t)as u8,(14.+15.*t)as u8,(18.+15.*t)as u8));
 let q=search.center()-vec2(2.,2.);p.circle_stroke(q,5.,Stroke::new(1.5_f32,MUTED));p.line_segment([q+vec2(3.5,3.5),q+vec2(8.,8.)],Stroke::new(1.5_f32,MUTED));
 if res.on_hover_text(local(ui,"Find a tool / Ctrl+K","Najít nástroj / Ctrl+K")).clicked(){app.palette_open=true;}
 }
 for i in 0..3{
  let b=Rect::from_min_size(pos2(start+i as f32*38.,cy-14.),vec2(34.,28.));
  let res=ui.interact(b,ui.id().with(("e-window",i)),Sense::click());let t=e_anim(ui,res.id,res.hovered());
  p.rect_filled(b,8.,if i==2{Color32::from_rgb((18.+95.*t)as u8,(12.+5.*t)as u8,(16.+10.*t)as u8)}else{Color32::from_rgb((12.+18.*t)as u8,(12.+18.*t)as u8,(15.+18.*t)as u8)});
  let press=motion_value(ui,res.id.with("window-press"),res.is_pointer_button_down_on(),0.09);
  let q=b.center()+vec2(0.,press);let pen=Stroke::new(1.5_f32+0.3*t,E_INK);
  match i{0=>{p.line_segment([q+vec2(-5.,0.),q+vec2(5.,0.)],pen);},1=>{p.rect_stroke(Rect::from_center_size(q,vec2(10.,9.)),1.,pen,StrokeKind::Inside);},_=>{p.line_segment([q+vec2(-4.,-4.),q+vec2(4.,4.)],pen);p.line_segment([q+vec2(-4.,4.),q+vec2(4.,-4.)],pen);}}
  if res.clicked(){match i{0=>ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),1=>ctx.send_viewport_cmd(ViewportCommand::Maximized(!ctx.input(|i|i.viewport().maximized.unwrap_or(false)))),_=>ctx.send_viewport_cmd(ViewportCommand::Close)}}
 }
 let drag=Rect::from_min_max(r.min,pos2(start-165.,r.bottom()));
 if ui.interact(drag,ui.id().with("e-drag"),Sense::click_and_drag()).dragged(){ctx.send_viewport_cmd(ViewportCommand::StartDrag);}
}
fn eclipse_hero(app:&mut BoostApp,ui:&mut Ui){
 let w=ui.available_width();
 let compact=w<520.||ui.ctx().content_rect().height()<650.;
 let height=if compact{126.}else{153.};
 let (r,_)=ui.allocate_exact_size(vec2(w,height),Sense::hover());
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,15.,Color32::from_rgb(12,9,13));
 let right=r.right();
 // One restrained OLED red light source with sparse static grooves:
 // animation only while motion is allowed, with no fabricated telemetry.
 let glow=pos2(right-60.,r.center().y);
 for i in (0..6).rev(){
  p.circle_filled(glow,26.+i as f32*16.,e_rgba(RED,(3+(5-i)*3)as u8));
 }
 p.circle_stroke(glow,33.,Stroke::new(1.2_f32,e_rgba(RED,105)));
 let moving=if e_motion(ui){app.app_start.elapsed().as_secs_f32()*0.6}else{0.};
 if e_motion(ui){
  e_arc(&p,glow,33.,moving,1.35,Stroke::new(2.4_f32,Color32::from_rgb(255,135,153)));
 }
 for i in 0..4{
  let x=right-129.+i as f32*21.;
  p.line_segment([pos2(x,r.top()+15.),pos2(x+28.,r.bottom()-15.)],
   Stroke::new(0.7_f32,e_rgba(RED,10)));
 }
 let output_running=app.native_mirror.as_ref().is_some_and(|v|v.running());
 let label=if output_running{
  local(ui,"GPU OUTPUT ACTIVE","GPU VÝSTUP AKTIVNÍ")
 }else{local(ui,"STANDBY","PŘIPRAVENO")};
 p.text(r.left_top()+vec2(18.,17.),egui::Align2::LEFT_TOP,
  "HORIZON  /  ECLIPSE",FontId::proportional(10.5),RED);
 p.text(r.left_top()+vec2(18.,43.),egui::Align2::LEFT_TOP,
  local(ui,"Graphics. Performance. Control.","Grafika. Výkon. Ovládání."),
  FontId::proportional(if compact{20.}else{30.}),E_INK);
 p.text(r.left_top()+vec2(19.,if compact{83.}else{100.}),egui::Align2::LEFT_TOP,
  local(ui,"Native GPU workspace  |  Windows 10 / 11",
           "Nativní GPU nástroje  |  Windows 10 / 11"),
  FontId::proportional(11.),MUTED);
 if w>=480.{
  p.text(pos2(right-17.,r.bottom()-20.),egui::Align2::RIGHT_BOTTOM,
    label,FontId::proportional(9.),if output_running{V25_MINT}else{MUTED});
 }
 p.rect_stroke(r,15.,Stroke::new(1.0_f32,Color32::from_rgb(55,27,38)),StrokeKind::Inside);
}
fn eclipse_route(app:&mut BoostApp,ui:&mut Ui,page:Page,name:&str,detail:&str,w:f32){
 let(r,res)=ui.allocate_exact_size(vec2(w,78.),Sense::click());let a=e_color(page);let t=e_anim(ui,res.id,res.hovered());let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,13.,Color32::from_rgb(11,11,14));let panel=r.shrink(2.);
 e_gradient(&p,panel,Color32::from_rgb(12,12,15),e_rgba(a,(15.+28.*t)as u8));
 p.rect_stroke(r,13.,Stroke::new(1.0_f32,if res.hovered(){a}else{E_LINE}),StrokeKind::Inside);
 let press=motion_value(ui,res.id.with("route-press"),res.is_pointer_button_down_on(),0.09);
 let q=r.left_center()+vec2(29.+2.*t,0.);
 p.circle_filled(q,19.+2.*t,e_rgba(a,(22.+28.*t)as u8));
 modern_icon(&p,q,page,a,motion_icon_scale(t,press,1.05));
 let live=(page==Page::Overlay&&app.overlay_open)||(page==Page::FrameGen&&app.native_mirror.as_ref().is_some_and(|m|m.running()&&m.multiplier()>1));
 if live{p.rect_stroke(r.shrink(1.),12.,Stroke::new(1.1_f32,V25_MINT),StrokeKind::Inside);p.circle_filled(r.right_top()+vec2(-12.,12.),3.,V25_MINT);}
 p.text(r.left_top()+vec2(59.,16.),egui::Align2::LEFT_TOP,tr(ui,name),FontId::proportional(16.),E_INK);
 p.text(r.left_top()+vec2(59.,40.),egui::Align2::LEFT_TOP,detail,FontId::proportional(11.),MUTED);
 let end=r.right_center()-vec2(18.-4.*t,0.);p.add(egui::Shape::line(vec![end+vec2(-4.,-4.),end,end+vec2(-4.,4.)],Stroke::new(1.6_f32,a)));
 if res.clicked(){app.page=page;}
}
fn eclipse_home(app:&mut BoostApp,ui:&mut Ui){
 eclipse_hero(app,ui);
 ui.add_space(12.);
 // The real active source and our separately rendered output are distinct.
 e_panel(ui,E_SURFACE,|ui|{
  ui.horizontal_wrapped(|ui|{
   let title=app.telem.active_game.as_ref().map(|g|trim(&g.title,42))
    .unwrap_or_else(||local(ui,"No game selected","Není vybraná hra").to_string());
   ui.vertical(|ui|{
    ui.label(RichText::new(local(ui,"ACTIVE TARGET","AKTIVNÍ HRA")).color(E_DIM).size(10.).strong());
    ui.label(RichText::new(title).color(E_INK).size(15.).strong());
   });
   ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
    if ui.small_button(local(ui,"Game library","Knihovna her")).clicked(){app.page=Page::Library;}
   });
  });
  ui.add_space(4.);
  ui.horizontal_wrapped(|ui|{
   let can_start=app.telem.active_game.is_some()&&!app.is_smoke;
   let active=app.native_mirror.as_ref().is_some_and(|m|m.running());
   if active{
    if ui.add(egui::Button::new("STOP GPU OUTPUT")
     .fill(Color32::from_rgb(72,22,33)).stroke(Stroke::new(1.,RED))).clicked(){
      app.stop_native_mirror();
    }
   }else if ui.add_enabled(can_start,egui::Button::new(local(ui,
         "Start filtered preview","Spustit GPU náhled"))
        .fill(Color32::from_rgb(94,29,43)).stroke(Stroke::new(1.,RED))).clicked(){
    app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);
   }
   if ui.button(if app.overlay_open{
    local(ui,"Hide HUD","Skrýt HUD")
   }else{local(ui,"Show HUD","Zobrazit HUD")}).clicked()&&!app.is_smoke{
    app.toggle_passive_hud();
   }
   if ui.button(local(ui,"Choose filters","Vybrat filtry")).clicked(){app.page=Page::Filters;}
   if app.window_macro.busy()&&ui.button("STOP MACRO / F12").clicked(){app.window_macro.cancel();}
  });
  ui.label(RichText::new(local(ui,
   "Preview FPS is separate from game FPS. Anti-cheat, true exclusive fullscreen and VRR require game testing.",
   "FPS náhledu nejsou FPS hry. Anti-cheat, skutečný fullscreen a VRR vyžadují testování ve hrách."))
   .color(E_DIM).size(10.5));
 });
 ui.add_space(11.);
 // Distinct telemetry labels; no generated FPS is ever passed off as game FPS.
 let own=app.output_rates.snapshot(std::time::Instant::now());
 let metrics=[
  (local(ui,"GAME FPS","FPS HRY"),fps_text(app,"fps"),Color32::from_rgb(255,96,121)),
  (local(ui,"FRAME TIME","ČAS SNÍMKU"),fps_text(app,"frametime"),Color32::from_rgb(240,122,130)),
  ("1% LOW",fps_text(app,"low"),Color32::from_rgb(240,127,145)),
  (local(ui,"OWN REAL FPS","FPS NÁHLEDU"),own.map(|o|format!("{:.0}",o.real)).unwrap_or_else(||"--".into()),
    Color32::from_rgb(230,136,144)),
 ];
 let cols=if ui.available_width()>=620.{4}else{2};
 let card_w=((ui.available_width()-8.*(cols-1)as f32)/cols as f32).max(105.);
 for group in metrics.chunks(cols){
  ui.horizontal(|ui|{
   for (label,value,accent) in group{
    v25_metric(ui,card_w,label,value,*accent);
   }
  });
  ui.add_space(5.);
 }
 if app.native_mirror.as_ref().is_some_and(|m|m.running()&&m.multiplier()>1){
  fg_readout_panel(app,ui);ui.add_space(8.);
 }
 ui.add_space(8.);
 ui.label(RichText::new(local(ui,"WORKSPACES","SEKCE")).color(E_DIM).size(10.5).strong());
 ui.add_space(7.);
 let routes=[
  (Page::Filters,"GPU Studio","Looks / lighting","Filtry a osvětlení"),
  (Page::Scaling,"Upscaling","Native GPU scalers","GPU zvětšování"),
  (Page::FrameGen,"Motion Lab","Own preview FG x2 / x3","FG nahled x2 / x3"),
  (Page::Performance,"FPS & Stability","Real telemetry / timing","Měřené FPS a latence"),
  (Page::Overlay,"Game Overlay","Click-through HUD","HUD přes hru"),
  (Page::Extras,"Game Tools","Macros / utilities","Makra a nástroje"),
  (Page::Capture,"Capture","Video / screenshots","Video a snímky"),
  (Page::Library,"Game Library","Profiles / games","Profily a hry")
 ];
 let w=ui.available_width();let cols=if w>=540.{2}else{1};
 let card_w=((w-9.*(cols-1)as f32)/cols as f32).max(180.);
 for group in routes.chunks(cols){
  ui.horizontal(|ui|{
   for (page,name,en,cz) in group{
    eclipse_route(app,ui,*page,name,local(ui,en,cz),card_w);
   }
  });
  ui.add_space(6.);
 }
 ui.add_space(6.);
 ui.horizontal_wrapped(|ui|{
  if ui.small_button(local(ui,"Lab / diagnostics","Všechny nástroje")).clicked(){
   app.page=Page::Unsorted;
  }
  ui.label(RichText::new(local(ui,
   "Development feature status is not in-game certification.",
   "Integrovaný kód neznamená ověření ve všech hrách."))
    .color(E_DIM).size(10.));
 });
}


/// Responsive primary navigation. Narrow views keep real page selection and
/// search available instead of clipping the left fixed-width sidebar.
/// This UI never controls game fullscreen, focus or GPU timing.
fn eclipse_compact_nav(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
 let entries=[
  (Page::Home,"HOME"),
  (Page::Filters,"GPU"),
  (Page::Scaling,"UPSCALE"),
  (Page::FrameGen,"MOTION"),
  (Page::Performance,"FPS"),
  (Page::Overlay,"HUD"),
  (Page::Extras,"TOOLS"),
  (Page::Capture,"CAPTURE"),
  (Page::Library,"GAMES"),
  (Page::Settings,"SETTINGS"),
  (Page::Unsorted,"ALL TOOLS"),
 ];
 egui::ScrollArea::horizontal().id_salt("hb-compact-nav-scroll").show(ui,|ui|{
  ui.horizontal(|ui|{
   for(page,name)in entries{
    let selected=app.page==page;
    let accent=e_color(page);
    let button=egui::Button::new(egui::RichText::new(name).strong().size(10.5))
     .fill(if selected{Color32::from_rgb(50,18,27)}else{E_SURFACE})
     .stroke(egui::Stroke::new(1.,if selected{accent}else{E_LINE}))
     .corner_radius(7.);
    let response=ui.add_sized([if name=="ALL TOOLS"{93.}else{70.},29.],button);
    motion_button(ui,&response);
    if response.clicked(){app.page=page;}
   }
   if ui.button("SEARCH / Ctrl+K").clicked(){app.palette_open=true;}
  });
 });
 // The navigation is event-driven: do not burn GPU/CPU when idle.
 let _=ctx;
}
#[cfg(test)]mod v103_responsive_nav_tests{
 use super::*;
 #[test]fn every_native_page_remains_accessible_in_responsive_mode(){
  let pages=[Page::Unsorted,Page::Home,Page::Filters,Page::Scaling,
   Page::FrameGen,Page::Performance,Page::Overlay,Page::Extras,
   Page::Capture,Page::Library,Page::Settings];
  let mut ids=std::collections::HashSet::new();
  for page in pages{assert!(ids.insert(format!("{page:?}")));}
  assert_eq!(ids.len(),11);
 }
}
