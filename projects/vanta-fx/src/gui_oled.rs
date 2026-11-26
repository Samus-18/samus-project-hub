// OLED simple workspace; advanced tools remain available without changing GPU state.
fn lang(ui:&Ui)->bool{ui.ctx().data(|d|d.get_temp::<bool>(egui::Id::new("hb-czech"))).unwrap_or(false)}
fn local<'a>(ui:&Ui,en:&'a str,cz:&'a str)->&'a str{if lang(ui){cz}else{en}}
fn translate<'a>(cz:bool,s:&'a str)->&'a str{if !cz{return s;}match s{
 "Overview"=>"Přehled","GPU Studio"=>"Grafika","Upscaling"=>"Zvětšení obrazu","FPS & Stability"=>"FPS a stabilita","Performance"=>"Výkon","Game Tools"=>"Herní nástroje","Game Overlay"=>"Herní overlay","Motion Lab"=>"Pohyb","Capture"=>"Záznam","Game Library"=>"Knihovna her","Settings"=>"Nastavení",
 "01  Native Looks"=>"Vzhledy","02  Light Studio"=>"Osvětlení","03  Screen Color"=>"Barvy monitoru","04  Stack Builder"=>"Vrstvy","05  Color Lab"=>"Barvy",
 "01 Live Metrics"=>"Živá měření","02 Game Tools"=>"Herní nástroje","03 Analysis"=>"Analýza","04 Stability"=>"Stabilita","05 CPU Affinity"=>"Jádra CPU",
 "01 HUD"=>"HUD","02 Timers"=>"Časovače","03 Quests"=>"Úkoly","04 Training"=>"Trénink","05 Quick Actions"=>"Rychlé akce","06 Macros"=>"Makra","07 View Tools"=>"Obraz","08 Reminders"=>"Připomínky","09 Controller"=>"Ovladač","10 Session Lab"=>"Relace",
 "Native Looks"=>"Vzhledy","Light Studio"=>"Osvětlení","Screen Color"=>"Barvy monitoru","Stack Builder"=>"Vrstvy","Color Lab"=>"Barvy",
 "Live Metrics"=>"Živá měření","Analysis"=>"Analýza","Stability"=>"Stabilita","CPU Affinity"=>"Jádra CPU","Timers"=>"Časovače","Quests"=>"Úkoly","Training"=>"Trénink","Quick Actions"=>"Rychlé akce","Macros"=>"Makra","View Tools"=>"Obraz","Reminders"=>"Připomínky","Controller"=>"Ovladač","Session Lab"=>"Relace",
 "Home"=>"Domů","Spatial Scaling"=>"Zvětšení obrazu","Enable"=>"Zapnout","Natural"=>"Přirozené","Cinematic"=>"Filmové","Shadow Pop"=>"Výrazné stíny","Radiance"=>"Záře","Balanced"=>"Vyvážené","Deep shadows"=>"Hluboké stíny","Radiant color"=>"Zářivé barvy","Natural soft"=>"Jemné","Reset to OFF"=>"Vypnout a obnovit","Cinema off / reset"=>"Vypnout a obnovit",
 "Contact shadows"=>"Kontaktní stíny","Highlight glow"=>"Záře světel","Color bounce"=>"Odraz barev","Fine detail"=>"Detaily","Filmic range"=>"Filmový rozsah","Sun warmth"=>"Teplota světla","Overall intensity"=>"Intenzita",
 "Contact shade"=>"Stíny","Indirect color"=>"Odraz barev","Light diffusion"=>"Rozptyl světla","Edge sheen"=>"Odlesky hran","Light reach"=>"Dosah světla","Local exposure"=>"Místní expozice","Highlight rolloff"=>"Jasné plochy","Master mix"=>"Intenzita",
 "Apply over game"=>"Použít ve hře","APPLY OVER GAME"=>"Použít ve hře","STOP GPU EFFECTS"=>"Vypnout efekty","Preview in own window"=>"Náhled v okně","Open HUD"=>"Otevřít HUD","Hide HUD"=>"Skrýt HUD",
 "Live GPU preview"=>"Živý náhled","Stop native GPU output"=>"Vypnout efekty","Glass over game – Experimental"=>"Přes hru – experimentální","Open Stack Builder"=>"Otevřít vrstvy","Adaptive"=>"Adaptivní","Full"=>"Plná kvalita","Eco"=>"Úsporné","Original"=>"Původní","SpaceX photo"=>"Fotografie SpaceX","Real gameplay still"=>"Herní snímek",
 "Application appearance"=>"Vzhled aplikace","UI zoom"=>"Velikost rozhraní","Reset UI zoom"=>"Výchozí velikost","Overlay opacity"=>"Průhlednost overlay","Emergency stop"=>"Nouzové vypnutí","Find a tool / Ctrl+K"=>"Najít nástroj / Ctrl+K","Settings saved"=>"Nastavení uloženo","No game selected"=>"Nevybraná hra",
 "Copy JSON"=>"Kopírovat JSON","Copy CSV"=>"Kopírovat CSV","Save report files"=>"Uložit výkaz","Stop session"=>"Ukončit relaci","Add marker"=>"Přidat značku","Choose local PNG/JPEG"=>"Vybrat PNG/JPEG","Bundled reference"=>"Ukázkový obrázek","Reset / off"=>"Vypnout a obnovit",
 _=>s}}
fn tr<'a>(ui:&Ui,s:&'a str)->&'a str{translate(lang(ui),s)}
fn oled_choice(ui:&mut Ui,label:&str,detail:&str,page:Page,on:bool,w:f32)->bool{
 oled_choice_variant(ui,label,detail,page,on,w,None)
}
fn oled_choice_variant(ui:&mut Ui,label:&str,detail:&str,page:Page,on:bool,w:f32,variant:Option<u8>)->bool{
 let(r,response)=ui.allocate_exact_size(vec2(w,if variant.is_some(){76.}else{65.}),Sense::click());
 let p0=ui.painter().with_clip_rect(r);let p=&p0;
 let fade=motion_value(ui,response.id.with("oled-hover"),response.hovered(),0.18);
 let active=motion_value(ui,response.id.with("oled-active"),on,0.22);
 let press=motion_value(ui,response.id.with("oled-press"),response.is_pointer_button_down_on(),0.09);
 let k=motion_icon_scale(fade,press,1.);
 let fill=Color32::from_rgb((12.+32.*active+9.*fade)as u8,(12.+3.*active+5.*fade)as u8,(15.+11.*active+5.*fade)as u8);
 let accent=match variant{Some(0)=>Color32::from_rgb(255,87,111),Some(1)=>Color32::from_rgb(160,125,255),Some(2)=>Color32::from_rgb(255,176,83),Some(_)=>Color32::from_rgb(98,222,178),None=>e_color(page)};
 p.rect_filled(r,12.,fill);
 e_gradient(p,r.shrink(3.),Color32::from_rgb(12,12,15),e_rgba(accent,(18.+45.*active+22.*fade)as u8));
 p.rect_stroke(r,12.,Stroke::new(1.0_f32,if on||response.hovered(){accent}else{BORDER}),StrokeKind::Inside);
 p.circle_filled(r.left_center()+vec2(25.+1.5*fade,0.),15.*k,if on{Color32::from_rgb(85,19,34)}else{CARD});
 let center=r.left_center()+vec2(25.+1.5*fade,0.);let ink=accent;let stroke=Stroke::new(1.5_f32*k,ink);
 if let Some(v)=variant{
  match v{
   0=>{p.circle_stroke(center,7.,stroke);p.line_segment([center+vec2(-7.,0.),center+vec2(7.,0.)],stroke);},
   1=>{p.circle_stroke(center,7.,stroke);p.circle_filled(center+vec2(3.,-2.),5.,if on{Color32::from_rgb(85,19,34)}else{CARD});},
   2=>{p.circle_filled(center,3.5,ink);for i in 0..8{let a=i as f32*std::f32::consts::TAU/8.;let d=vec2(a.cos(),a.sin());p.line_segment([center+d*6.,center+d*10.],stroke);}},
   _=>{p.add(egui::Shape::line(vec![center+vec2(-8.,2.),center+vec2(-4.,-3.),center,center+vec2(4.,3.),center+vec2(8.,-2.)],stroke));}
  }
 }else{modern_icon(p,center,page,ink,0.83*k);}
 p.text(r.left_top()+vec2(48.,11.),egui::Align2::LEFT_TOP,label,FontId::proportional(13.),WHITE);
 p.text(r.left_top()+vec2(48.,33.),egui::Align2::LEFT_TOP,detail,FontId::proportional(10.5),MUTED);
 if active>0.001{
  let width=(r.width()-20.).max(0.)*motion_ease(active);
  p.rect_filled(Rect::from_center_size(pos2(r.center().x,r.bottom()-2.),vec2(width,2.)),1.,accent);
 }
 motion_button(ui,&response);
 response.clicked()
}
fn active_output_for_choice(app:&BoostApp)->bool{app.native_mirror.is_some()}
fn oled_action_bar(app:&mut BoostApp,ui:&mut Ui){
 e_panel(ui,E_SURFACE,|ui|{
  ui.set_min_width(ui.available_width());ui.horizontal_wrapped(|ui|{
   let active=v22_gpu_active(app);let name=app.telem.active_game.as_ref().map(|g|g.title.as_str()).unwrap_or(tr(ui,"No game selected"));
   let(q,_)=ui.allocate_exact_size(vec2(12.,16.),Sense::hover());ui.painter().circle_filled(q.center(),4.,if active{V25_MINT}else{E_DIM});
   ui.label(RichText::new(trim(name,32)).strong().size(14.));
   if outline_button(ui,local(ui,"Choose game","Vybrat hru")){app.page=Page::Library;}
   if outline_button(ui,"Find feature / Ctrl+K"){app.palette_open=true;}
   ui.add_space(5.);
   if active{if red_button(ui,local(ui,"Stop effects","Vypnout efekty"))&&!app.is_smoke{app.stop_native_mirror();}}
   else if app.native_mirror.is_some(){
    caption(ui,local(ui,"Stopping / updating…","Ukončování / aktualizace…"));
    if app.pending_mirror_restart.is_some()&&outline_button(ui,local(ui,"Cancel update","Zrušit aktualizaci")){app.stop_native_mirror();}
   }else{
    let ready=app.telem.active_game.is_some()&&!app.is_smoke;
    if ui.add_enabled(ready,egui::Button::new(RichText::new(local(ui,"Apply effects","Použít efekty")).strong()).fill(Color32::from_rgb(147,21,50)).min_size(vec2(116.,29.))).on_hover_text(local(ui,"Own independent D3D11 filtered output; no game hooks or Glass. Exclusive fullscreen remains unverified.","Experimentální bezokrajový overlay; exkluzivní fullscreen neověřen.")).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);}
if app.settings.ui_advanced&&ui.add_enabled(ready,egui::Button::new(local(ui,"Native GPU borderless output","Nativní GPU výstup bez okrajů"))).clicked(){app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::BorderlessScreen,1);}
   }
   if outline_button(ui,if app.overlay_open{tr(ui,"Hide HUD")}else{tr(ui,"Open HUD")})&&!app.is_smoke{app.toggle_passive_hud();}
  });
  if !active_output_for_choice(app){
    ui.horizontal_wrapped(|ui|{
      if ui.checkbox(&mut app.settings.preview_composition,"DirectComposition preview").changed(){app.save();}
      caption(ui,"Separate GPU output, not an injected game overlay.");
    });
  }
  if let Some(w)=app.native_mirror.as_ref(){
    let h=w.diagnostics();
    caption(ui,&format!("{} | {} | {} real / {} generated",if w.composition_active(){"DirectComposition"}else if w.mode()==crate::native_mirror::MirrorMode::CompositedInPlace{"Glass"}else{"HWND flip"},if !w.running(){"stopping"}else if h.own_minimized{"minimized"}else if h.paused{"paused"}else if w.submitted_frames()==0{"starting"}else{"active"},w.submitted_frames(),w.generated_frames()));
  }
 });
}
fn oled_home(app:&mut BoostApp,ui:&mut Ui){
 section(ui,"Overview","");
 let w=ui.available_width();
 let (r,_)=ui.allocate_exact_size(vec2(w,if ui.ctx().content_rect().height()<660.{106.}else{150.}),Sense::hover());
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,12.,CARD);
 if let Some(t)=app.hero_picture.as_ref(){p.image(t.id(),r,Rect::from_min_max(pos2(0.,0.),pos2(1.,1.)),Color32::from_gray(145));}
 v25_gradient(&p,r,Color32::from_rgba_unmultiplied(0,0,0,245),Color32::from_rgba_unmultiplied(0,0,0,40));
 p.text(r.left_center()+vec2(19.,-15.),egui::Align2::LEFT_CENTER,local(ui,"YOUR GAME. YOUR LOOK.","TVOJE HRA. TVŮJ VZHLED."),FontId::proportional(24.),WHITE);
 p.text(r.left_center()+vec2(20.,17.),egui::Align2::LEFT_CENTER,"D3D11   /   HORIZON BOOST",FontId::proportional(11.),RED);
 ui.add_space(8.);oled_action_bar(app,ui);ui.add_space(10.);
 if red_button(ui,"Lab / diagnostics"){app.page=Page::Unsorted;}
 let cards=[(Page::FrameGen,"Motion Lab","Frame interpolation","Interpolace"),(Page::Overlay,"Game Overlay","Always-on HUD","HUD"),(Page::Filters,"GPU Studio","Graphics / lighting","Grafika / osvětlení"),(Page::Scaling,"Upscaling","Image reconstruction","Zvětšení obrazu"),(Page::Performance,"FPS & Stability","Measured performance","Měřený výkon"),(Page::Extras,"Game Tools","HUD / timers / practice","HUD / časovače / trénink")];
 let cols=if w>=490.{2}else{1};let cw=(w-8.*(cols-1)as f32)/cols as f32;
 for chunk in cards.chunks(cols){ui.horizontal(|ui|{for(page,name,en,cz)in chunk{if oled_choice(ui,tr(ui,name),local(ui,en,cz),*page,false,cw){app.page=*page;}}});}
 ui.add_space(8.);
 let values=[("FPS",fps_text(app,"fps"),V25_MINT),(local(ui,"Frame time","Čas snímku"),fps_text(app,"frametime"),RED),("1% low",fps_text(app,"low"),V25_LAVENDER)];
 ui.horizontal(|ui|{for(n,v,a)in values{v25_metric(ui,(w-16.)/3.,n,&v,a);}});
}
fn oled_light(app:&mut BoostApp,ui:&mut Ui){
 oled_action_bar(app,ui);ui.add_space(8.);
 let mut t=app.settings.lighting.clone();let previous=t.clone();
 frame(PANEL).show(ui,|ui|{
  ui.set_min_width(ui.available_width());
  ui.horizontal_wrapped(|ui|{
   heading(ui,local(ui,"Cinema Radiance","Filmové osvětlení"));
   let label=local(ui,"Enabled","Zapnuto");e_switch(ui,&mut t.cinema.enabled,label);
   ui.label(RichText::new(local(ui,"2D lighting","2D osvětlení")).color(MUTED)).on_hover_text(local(ui,"Color-only postprocessing. No depth, geometry or true path tracing.","Zpracování barev bez geometrie, hloubky a skutečného path tracingu."));
  });
  ui.add_space(8.);
  let variants=[("Balanced","Vyvážené","Soft shadows","Jemné stíny"),("Deep shadows","Hluboké stíny","Higher contrast","Vyšší kontrast"),("Radiant color","Zářivé barvy","More light","Více světla"),("Natural soft","Přirozené","Subtle effect","Jemný efekt")];
  let cols=if ui.available_width()>=850.{4}else if ui.available_width()>=490.{2}else{1};let w=(ui.available_width()-8.*(cols-1)as f32)/cols as f32;
  for(row,chunk)in variants.chunks(cols).enumerate(){ui.horizontal(|ui|{for(col,(en,cz,de,dc))in chunk.iter().enumerate(){let i=(row*cols+col)as u8;let p=crate::cinema_light::Options::preset(i);let selected=t.cinema.enabled&&t.cinema==p;
   if oled_choice_variant(ui,local(ui,en,cz),local(ui,de,dc),Page::Filters,selected,w,Some(i)){t.cinema=p;}
  }});}
  ui.add_space(4.);
  ui.add_enabled_ui(t.cinema.enabled,|ui|{
   ui.horizontal_wrapped(|ui|{
    ui.label(local(ui,"Quality","Kvalita"));
    for(i,(en,cz))in [("Eco","Úsporná"),("Balanced","Vyvážená"),("Fine","Jemná")].iter().enumerate(){ui.selectable_value(&mut t.cinema.quality,i as u8,local(ui,en,cz));}
   });
   ui.columns(2,|c|{
    let a=local(&c[0],"Intensity","Intenzita");c[0].add(egui::Slider::new(&mut t.cinema.mix,0.0..=1.).text(a));
    let a=local(&c[1],"Shadows","Stíny");c[1].add(egui::Slider::new(&mut t.cinema.shadow,0.0..=1.).text(a));
    let a=local(&c[0],"Light glow","Záře");c[0].add(egui::Slider::new(&mut t.cinema.bloom,0.0..=1.).text(a));
    let a=local(&c[1],"Color bounce","Odraz barev");c[1].add(egui::Slider::new(&mut t.cinema.bounce,0.0..=1.).text(a));
   });
  });
  ui.horizontal_wrapped(|ui|{
   ui.checkbox(&mut t.reconstruction.animated_grain,"Animated grain (1x)");
   ui.checkbox(&mut t.bypass_look,local(ui,"Lighting only","Pouze osvětlení"));
   ui.checkbox(&mut t.layers.protect.rectangle,local(ui,"Protect HUD area","Chránit oblast HUD")).on_hover_text(local(ui,"Default lower-screen rectangle; edit in Advanced.","Výchozí spodní oblast; uprav v Pokročilé."));
   if outline_button(ui,local(ui,"Reset","Obnovit")){t.cinema=Default::default();}
  });
 });
 if t!=previous{app.apply_native_lighting(t);}
 ui.add_space(8.);
 oled_light_reference(app,ui);
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  ui.horizontal_wrapped(|ui|{
   heading(ui,local(ui,"Fine tuning","Jemné ladění"));
   if outline_button(ui,local(ui,"Advanced","Pokročilé")){app.settings.ui_advanced=true;app.save();}
   if outline_button(ui,local(ui,"Color & photo lab","Barvy a foto test")){app.filter_view=4;}
  });
  caption(ui,local(ui,"No true ray tracing. Effects use only the captured image.","Bez skutečného ray tracingu. Efekty pracují pouze se snímkem."));
 });
}
fn oled_settings(app:&mut BoostApp,ui:&mut Ui){
 section(ui,"Settings","");
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Language","Jazyk"));
  ui.horizontal(|ui|{
   for(i,n)in ["English","Čeština"].iter().enumerate(){if ui.selectable_value(&mut app.settings.ui_language,i as u8,*n).changed(){app.save();ui.ctx().request_repaint();}}
  });
  caption(ui,local(ui,"Main interface is bilingual; some advanced diagnostics remain English.","Hlavní rozhraní je dvojjazyčné; některé pokročilé diagnostiky zůstávají anglicky."));
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Appearance","Vzhled"));
  let motion=local(ui,"Animations","Animace");if ui.checkbox(&mut app.settings.animations,motion).changed(){app.save();}
  let zoom=local(ui,"Interface size","Velikost rozhraní");if ui.add(egui::Slider::new(&mut app.settings.ui_zoom,0.75..=1.35).step_by(0.05).text(zoom)).changed(){app.save();}
  let mut advanced=app.settings.ui_advanced;if ui.checkbox(&mut advanced,local(ui,"Advanced controls","Pokročilé ovládání")).changed(){app.settings.ui_advanced=advanced;app.save();}
  ui.horizontal_wrapped(|ui|{for(n,size)in [(local(ui,"Compact","Malé"),[840.,560.]),(local(ui,"Normal","Normální"),[940.,620.]),(local(ui,"Large","Velké"),[1180.,760.])]{if outline_button(ui,n)&&!app.is_smoke{app.set_main_window_size(ui.ctx(),size);}}});
 });
 ui.add_space(8.);
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Game HUD","Herní HUD"));
  let scale=local(ui,"Size","Velikost");if ui.add(egui::Slider::new(&mut app.settings.overlay_size_scale,0.78..=1.35).text(scale)).changed(){app.save();}
  let opacity=local(ui,"Opacity","Průhlednost");if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.).text(opacity)).changed(){app.save();}
  if outline_button(ui,local(ui,"Edit HUD","Upravit HUD"))&&!app.is_smoke{app.overlay_open=true;app.overlay_edit_mode=true;}
  if red_button(ui,local(ui,"Emergency stop","Nouzové vypnutí"))&&!app.is_smoke{app.emergency_stop();}
 });
}
#[cfg(test)]mod oled_tests{use super::*;#[test]fn language_defaults_and_roundtrip(){let s:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert_eq!(s.ui_language,0);assert!(!s.ui_advanced);assert_eq!(translate(false,"Settings"),"Settings");assert_eq!(translate(true,"Settings"),"Nastavení");assert_eq!(translate(true,"D3D11"),"D3D11");let mut s=s;s.ui_language=1;s.ui_advanced=true;let b:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(b.ui_language,1);assert!(b.ui_advanced);}}

fn oled_light_reference(app:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("oled-light-reference-sig");
 let signature=|app:&BoostApp|serde_json::to_string(&(app.settings.native_preset,app.settings.native_stack_profiles.get(&app.profile_key()),&app.settings.lighting)).unwrap_or_default();
 let mut sig=ui.data(|d|d.get_temp::<String>(id));
 if sig.is_some(){
  if let Some(result)=app.photo_rx.as_ref().and_then(|rx|match rx.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Reference worker stopped".into())),_=>None}){
   app.photo_rx=None;
   match result{Ok(v)=>{let make=|n:&str,i:&image::RgbaImage|ui.ctx().load_texture(n,egui::ColorImage::from_rgba_unmultiplied([i.width()as usize,i.height()as usize],i.as_raw()),egui::TextureOptions::LINEAR);
    app.photo_textures=Some((make("oled-light-before",&v.before),make("oled-light-after",&v.after)));app.photo_result=Some(v);},Err(e)=>{sig=None;app.toast(e);}}
  }
 }
 egui::CollapsingHeader::new(local(ui,"Real GPU preview (optional)","Skutečný GPU náhled (volitelný)")).show(ui,|ui|{
  let idle=app.photo_rx.is_none()&&app.native_mirror.is_none()&&app.effect_bench_rx.is_none()&&app.pipeline_bench_rx.is_none()&&app.motion_lab_rx.is_none();
  if ui.add_enabled(idle&&!app.is_smoke,egui::Button::new(local(ui,"Render current lighting","Vykreslit aktuální osvětlení"))).clicked(){
   let stages=app.settings.native_stack_profiles.get(&app.profile_key()).cloned().unwrap_or_else(||crate::native_presets::PRESETS[app.settings.native_preset].stages.to_vec());let lighting=app.settings.lighting.clone();
   let(tx,rx)=std::sync::mpsc::channel();app.photo_rx=Some(rx);app.photo_result=None;app.photo_textures=None;sig=Some(signature(app));
   std::thread::spawn(move||{let _=tx.send(crate::photo_lab::render(None,&stages,&lighting,None,false));});
  }
  if app.photo_rx.is_some(){ui.spinner();ui.ctx().request_repaint_after(std::time::Duration::from_millis(80));}
  if let(Some(saved),Some(v),Some((a,b)))=(&sig,&app.photo_result,&app.photo_textures){
   let fresh=*saved==signature(app);
   if !fresh{ui.colored_label(RED,local(ui,"Settings changed - render again","Nastavení změněno – vykresli znovu"));}
   let w=ui.available_width().min(560.);let height=w*v.before.height()as f32/v.before.width()as f32;
   let(r,res)=ui.allocate_exact_size(vec2(w,height),Sense::click_and_drag());if(res.dragged()||res.clicked())&&let Some(q)=res.interact_pointer_pos(){app.photo_split=((q.x-r.left())/r.width()).clamp(0.,1.);}
   let uv=Rect::from_min_max(pos2(0.,0.),pos2(1.,1.));ui.painter().image(a.id(),r,uv,WHITE);
   let x=r.left()+r.width()*app.photo_split;ui.painter().with_clip_rect(Rect::from_min_max(pos2(x,r.top()),r.right_bottom())).image(b.id(),r,uv,WHITE);ui.painter().line_segment([pos2(x,r.top()),pos2(x,r.bottom())],Stroke::new(2.0_f32,RED));
   caption(ui,local(ui,"Drag to compare. Bundled reference, not live game capture.","Tažením porovnej. Ukázkový snímek, nikoli živý záznam hry."));
  }
 });
 if let Some(sig)=sig{ui.data_mut(|d|d.insert_temp(id,sig));}else{ui.data_mut(|d|d.remove::<String>(id));}
}
