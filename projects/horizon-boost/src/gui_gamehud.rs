// In-game reticle controls shared between the external Overlay and Settings.
const CROSSHAIR_STYLE_CZ:[&str;20]=[
 "Tečka","Kříž s mezerou","Kruh","Kruh s tečkou","Zaměřovač T",
 "Optika","Rohy","Šipka","Diagonální X","Dva kruhy",
 "Zaměřovací mřížka","Mikro přesnost","Klasické T",
 "Rohová optika","Přesný diamant","Dělená šipka","Dálkoměr",
 "Radarový cíl","Jemná tečka se značkami","Světelný cíl",
];
fn crosshair_style_local(ui:&Ui,i:usize)->&'static str{
 local(ui,crate::crosshair::STYLE_LABELS[i],CROSSHAIR_STYLE_CZ[i])
}
fn crosshair_controls(app:&mut BoostApp,ui:&mut Ui){
    app.settings.crosshair.clamp();
    let mut change=false;
    ui.horizontal_wrapped(|ui|{
        heading(ui,local(ui,"CROSSHAIR PRO • 20 RETICLES","ZAMĚŘOVAČ • 20 TYPŮ"));
        let label=if app.settings.crosshair.enabled{local(ui,"ON","ZAPNUTO")}else{local(ui,"OFF","VYPNUTO")};
        if ui.add_enabled(app.settings.crosshair.enabled||app.telem.active_game.is_some(),egui::Button::new(if app.settings.crosshair.enabled{local(ui,"Turn off crosshair","Vypnout zaměřovač")}else{local(ui,"Show crosshair","Zapnout zaměřovač")})).clicked(){ app.settings.crosshair.enabled=!app.settings.crosshair.enabled;
            change=true;
        }
        ui.label(label); caption(ui,"Ctrl + Shift + X");
    });
    caption(ui,local(ui,"Click-through Windows HUD. No game injection or changed EXE.","Průchozí Windows HUD. Bez injekce nebo úprav EXE hry."));
    let target=app.telem.active_game.as_ref().map(|g|g.title.as_str());
    ui.label(format!("{}: {}",local(ui,"Target","Cíl"),target.unwrap_or(local(ui,"Choose a game first","Nejdřív vyber hru"))));
    if app.settings.crosshair.enabled{ui.colored_label(if app.crosshair.visible(){V25_MINT}else{MUTED},if app.crosshair.visible(){local(ui,"Visible in selected game","Viditelný ve vybrané hře")}else{local(ui,"Waiting for game focus","Čekám na fokus hry")});}
    ui.horizontal_wrapped(|ui|{
        if ui.small_button(local(ui,"Classic","Klasický")).clicked(){app.settings.crosshair.style=1;app.settings.crosshair.size=18;app.settings.crosshair.gap=5;change=true;}
        if ui.small_button(local(ui,"Micro dot","Mikro tečka")).clicked(){app.settings.crosshair.style=0;app.settings.crosshair.size=5;app.settings.crosshair.gap=0;change=true;}
        if ui.small_button(local(ui,"Ring","Kruh")).clicked(){app.settings.crosshair.style=2;app.settings.crosshair.size=14;app.settings.crosshair.gap=4;change=true;}
        if ui.small_button(local(ui,"Diamond","Diamant")).clicked(){app.settings.crosshair.style=14;app.settings.crosshair.size=16;app.settings.crosshair.gap=4;change=true;}
        if ui.small_button(local(ui,"Reset","Obnovit")).clicked(){let enabled=app.settings.crosshair.enabled;app.settings.crosshair=Default::default();app.settings.crosshair.enabled=enabled;change=true;}
    });
    ui.horizontal_wrapped(|ui|{
        caption(ui,local(ui,"STYLE","TVAR"));
        egui::ComboBox::from_id_salt(ui.id().with("hb-hud-style")).width(130.0)
            .selected_text(crosshair_style_local(ui,app.settings.crosshair.style))
            .show_ui(ui,|ui|{
                for (i,label) in crate::crosshair::STYLE_LABELS.iter().enumerate(){
                    if ui.selectable_value(&mut app.settings.crosshair.style,i,crosshair_style_local(ui,i)).changed(){change=true;}
                }
            });
        caption(ui,local(ui,"COLOR","BARVA"));
        egui::ComboBox::from_id_salt(ui.id().with("hb-hud-color")).width(115.0)
            .selected_text(local(ui,["White","Red","Green","Cyan","Gold"][app.settings.crosshair.color],["Bílá","Červená","Zelená","Tyrkysová","Zlatá"][app.settings.crosshair.color]))
            .show_ui(ui,|ui|{
                for (i,label) in ["White","Red","Green","Cyan","Gold"].iter().enumerate(){
                    if ui.selectable_value(&mut app.settings.crosshair.color,i,local(ui,label,["Bílá","Červená","Zelená","Tyrkysová","Zlatá"][i])).changed(){change=true;}
                }
            });
    });
    ui.horizontal_wrapped(|ui|{
        if ui.add(egui::Slider::new(&mut app.settings.crosshair.size,3..=68).text(local(ui,"Size","Velikost"))).changed(){change=true;}
        if ui.add(egui::Slider::new(&mut app.settings.crosshair.thickness,1..=6).text(local(ui,"Width","Tloušťka"))).changed(){change=true;}
        if ui.add(egui::Slider::new(&mut app.settings.crosshair.gap,0..=20).text(local(ui,"Gap","Mezera"))).changed(){change=true;}
    });
    ui.horizontal_wrapped(|ui|{
        if ui.checkbox(&mut app.settings.crosshair.outline,local(ui,"Contrast outline","Kontrastní obrys")).changed(){change=true;}
        if ui.small_button(local(ui,"High contrast","Vysoký kontrast")).clicked(){app.settings.crosshair.color=0;app.settings.crosshair.outline=true;app.settings.crosshair.opacity=255;change=true;}
        caption(ui,local(ui,"Opacity","Průhlednost"));
        if ui.add(egui::Slider::new(&mut app.settings.crosshair.opacity,70..=255)
            .show_value(false).text("")).changed(){change=true;}
    });
    caption(ui,local(ui,"Per-game saved profile. On-screen reticle never takes game clicks.","Profil se ukládá pro každou hru. Zaměřovač nepřebírá kliknutí."));
    let (r,_)=ui.allocate_exact_size(vec2(ui.available_width().min(360.0),118.0),Sense::hover());
    ui.painter().rect_filled(r,9.0,Color32::from_rgb(15,16,27));
    let cx=r.center();
    let opt=&app.settings.crosshair;
    let rgb=opt.color_value();
    let clr=Color32::from_rgb((rgb&255) as u8,((rgb>>8)&255) as u8,((rgb>>16)&255) as u8);
    let shape=crate::crosshair::geometry(opt.style,opt.size,opt.gap);
    let scale=(r.width().min(r.height())/(2.0*opt.size as f32+20.0)).min(1.0);
    let alpha=opt.opacity;
    let colored=Color32::from_rgba_unmultiplied(clr.r(),clr.g(),clr.b(),alpha);
    let paint=|paint_color:Color32,width:f32|{
        let pen=Stroke::new((width*scale).max(0.8),paint_color);
        for (start,end) in &shape.lines{
            ui.painter().line_segment([cx+vec2(start[0] as f32,start[1] as f32)*scale,
                cx+vec2(end[0] as f32,end[1] as f32)*scale],pen);
        }
        for rad in &shape.rings{ui.painter().circle_stroke(cx,*rad as f32*scale,pen);}
        if shape.dot>0{ui.painter().circle_filled(cx,shape.dot as f32*scale,paint_color);}
    };
    if opt.outline{paint(Color32::from_black_alpha(alpha),opt.thickness as f32+3.0);}
    paint(colored,opt.thickness as f32);
    if change{app.apply_crosshair();}
    caption(ui,local(ui,"Works in windowed/borderless games; exclusive or protected modes may hide it.","Funguje v okně nebo bez okrajů; exkluzivní či chráněné režimy jej mohou skrýt."));
}
