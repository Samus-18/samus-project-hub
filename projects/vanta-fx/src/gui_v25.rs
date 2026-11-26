// Horizon Boost v2.5 — original compact dark UI: real data and real controls.
// Vector geometry only, with hover interpolation and a 25 FPS motion budget.
// Native D3D11/WGC renderers remain untouched by decorative UI animation.
const V25_BASE:Color32=Color32::from_rgb(12,13,20);
const V25_PANEL:Color32=Color32::from_rgb(23,25,35);
const V25_RAISED:Color32=Color32::from_rgb(32,34,47);
const V25_EDGE:Color32=Color32::from_rgb(58,61,80);
const V25_ACCENT:Color32=Color32::from_rgb(255,82,113);
const V25_MINT:Color32=Color32::from_rgb(124,226,193);
const V25_LAVENDER:Color32=Color32::from_rgb(187,159,251);

fn v25_surface(p:&Painter,r:Rect,selected:bool,hover:f32){
    let fill=if selected{
        Color32::from_rgb((59.0+13.0*hover) as u8,28,44)
    }else{
        Color32::from_rgb((24.0+13.0*hover) as u8,
            (26.0+9.0*hover) as u8,(36.0+10.0*hover) as u8)
    };
    p.rect_filled(r,12.0,fill);
    p.rect_stroke(r,12.0,Stroke::new(
        if selected{1.15_f32}else{0.8_f32},
        if selected{Color32::from_rgb(175,67,94)}else{V25_EDGE}
    ),StrokeKind::Inside);
}

fn v25_gradient(p:&Painter,r:Rect,left:Color32,right:Color32){
    let mut mesh=egui::Mesh::default();
    for (pt,color) in [
        (r.left_top(),left),(r.right_top(),right),
        (r.left_bottom(),left),(r.right_bottom(),right)
    ] {
        mesh.vertices.push(egui::epaint::Vertex{
            pos:pt,uv:egui::epaint::WHITE_UV,color
        });
    }
    mesh.indices.extend_from_slice(&[0,1,2,2,1,3]);
    p.add(egui::Shape::mesh(mesh));
}
fn v25_status_color(active:bool)->Color32{
    if active{V25_MINT}else{Color32::from_rgb(190,193,208)}
}
fn v25_pill(ui:&mut Ui,label:&str,selected:bool,width:f32)->bool{
    let (rect,resp)=ui.allocate_exact_size(vec2(width,30.0),Sense::click());
    let glow=motion_value(ui,resp.id.with("pill-v25"),resp.hovered(),0.16);
    let p=ui.painter();
    let fill=if selected{Color32::from_rgb(107,34,61)}
        else{Color32::from_rgb((32.0+18.0*glow) as u8,
            (32.0+7.0*glow) as u8,(45.0+11.0*glow) as u8)};
    p.rect_filled(rect,8.0,fill);
    p.rect_stroke(rect,8.0,Stroke::new(if selected{1.0_f32}else{0.7_f32},
        if selected{V25_ACCENT}else{V25_EDGE}),StrokeKind::Inside);
    p.text(rect.center(),egui::Align2::CENTER_CENTER,label,
        FontId::proportional(11.7),if selected{WHITE}else{MUTED});
    resp.clicked()
}
fn v25_action(ui:&mut Ui,width:f32,title:&str,detail:&str,
    icon_page:Page,primary:bool,enabled:bool)->bool{
    let (r,resp)=ui.allocate_exact_size(vec2(width,58.0),Sense::click());
    let pulse=motion_value(ui,resp.id.with("v25-action"),resp.hovered(),0.18);
    let p=ui.painter();
    let base=if primary{Color32::from_rgb((110.0+17.0*pulse) as u8,
            (31.0+7.0*pulse) as u8,(57.0+10.0*pulse) as u8)}
        else{Color32::from_rgb((31.0+12.0*pulse) as u8,
            (32.0+9.0*pulse) as u8,(44.0+10.0*pulse) as u8)};
    p.rect_filled(r,11.0,base);
    p.rect_stroke(r,11.0,Stroke::new(if primary{1.1_f32}else{0.9_f32},
        if primary{V25_ACCENT}else{V25_EDGE}),StrokeKind::Inside);
    let icon_x=r.left()+23.0;
    let ic=pos2(icon_x,r.center().y);
    p.circle_filled(ic,14.0,Color32::from_rgb(
        if primary{140}else{52},if primary{49}else{46},
        if primary{78}else{67}));
    modern_icon(p,ic,icon_page,if enabled{WHITE}else{MUTED},0.70);
    p.text(pos2(r.left()+45.0,r.top()+12.0),egui::Align2::LEFT_TOP,
        trim(title,22),FontId::proportional(12.5),if enabled{WHITE}else{MUTED});
    p.text(pos2(r.left()+45.0,r.top()+34.0),egui::Align2::LEFT_TOP,
        trim(detail,28),FontId::proportional(10.3),
        if primary{Color32::from_rgb(241,189,199)}else{MUTED});
    resp.clicked()
}
fn v25_metric(ui:&mut Ui,width:f32,title:&str,value:&str,accent:Color32){
    let (r,_)=ui.allocate_exact_size(vec2(width,68.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(r,12.0,E_SURFACE);
    p.rect_stroke(r,12.0,Stroke::new(0.8_f32,E_LINE),StrokeKind::Inside);
    p.rect_filled(Rect::from_min_size(r.min+vec2(11.0,9.0),vec2(18.0,2.0)),
        1.0,accent);
    p.text(r.min+vec2(11.0,16.0),egui::Align2::LEFT_TOP,
        title,FontId::proportional(9.5),MUTED);
    p.text(r.min+vec2(11.0,31.0),egui::Align2::LEFT_TOP,
        trim(value,18),FontId::proportional(if value.chars().count()>13{12.0}else{23.0}),
        WHITE);
}

fn v25_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let r=ui.max_rect();
    let p=ui.painter();
    p.rect_filled(r,0.0,Color32::from_rgb(8,8,10));
    p.line_segment([pos2(r.left()+12.0,r.bottom()-1.0),
        pos2(r.right()-12.0,r.bottom()-1.0)],
        Stroke::new(0.8_f32,Color32::from_rgb(68,44,66)));
    let origin=pos2(r.left()+22.0,r.center().y);
    let emblem=Rect::from_center_size(origin+vec2(14.0,0.0),vec2(33.0,33.0));
    p.rect_filled(emblem,9.0,Color32::from_rgb(71,28,47));
    p.add(egui::Shape::convex_polygon(vec![
        origin+vec2(1.0,8.0),origin+vec2(14.0,-12.0),
        origin+vec2(27.0,8.0)
    ],V25_ACCENT,Stroke::NONE));
    p.line_segment([origin+vec2(9.0,2.0),origin+vec2(20.0,2.0)],
        Stroke::new(2.0_f32,Color32::from_rgb(255,195,210)));
    let x=origin.x+49.0;
    p.text(pos2(x,origin.y-1.0),egui::Align2::LEFT_CENTER,
        "HORIZON",FontId::proportional(20.0),WHITE);
    p.text(pos2(x+103.0,origin.y-1.0),egui::Align2::LEFT_CENTER,
        "BOOST",FontId::proportional(20.0),V25_ACCENT);
    if r.width()>1060.0 {
        p.text(pos2(x+197.0,origin.y),egui::Align2::LEFT_CENTER,
            "GRAPHICS  /  PERFORMANCE",FontId::proportional(11.2),
            Color32::from_rgb(151,155,174));
    }
    let window_control_w=39.0;
    let gap=5.0;
    let controls_x=r.right()-3.0*window_control_w-2.0*gap-16.0;
    if r.width()>940.0{
        let status_r=Rect::from_min_size(pos2(controls_x-115.0,r.center().y-13.0),
            vec2(100.0,26.0));
        p.rect_filled(status_r,7.0,
            if v22_gpu_active(app){Color32::from_rgb(29,61,50)}
            else{Color32::from_rgb(34,37,47)});
        p.circle_filled(status_r.left_center()+vec2(12.0,0.0),3.5,
            v25_status_color(v22_gpu_active(app)));
        p.text(status_r.left_center()+vec2(22.0,0.0),
            egui::Align2::LEFT_CENTER,
            if v22_gpu_active(app){local(ui,"WGC ACTIVE","WGC AKTIVNÍ")}else{local(ui,"STANDBY","PŘIPRAVENO")},
            FontId::proportional(10.1),if v22_gpu_active(app){V25_MINT}else{MUTED});
    }
    if app.settings.animations{
        let time=app.app_start.elapsed().as_secs_f32();
        let period=((time*0.12).fract()*r.width()).min(r.width()-1.0);
        for i in 0..14{
            let falloff=(1.0-(i as f32/14.0)).powi(2);
            let xx=r.left()+(period-i as f32*4.0).rem_euclid(r.width());
            p.line_segment([pos2(xx,r.bottom()-1.0),pos2(xx+4.0,r.bottom()-1.0)],
                Stroke::new(1.1_f32,Color32::from_rgba_unmultiplied(255,82,119,
                    (160.0*falloff) as u8)));
        }
    }
    // Tiny size controls live beside the Windows minimize/maximize/close set.
    // Even the smallest main window can be resized without entering Settings.
    // These are explicit controls; dragging empty header space still moves it.
    for (idx,glyph) in [(0,"−"),(1,"+")] {
        let rect=Rect::from_min_size(
            pos2(controls_x-201.0+idx as f32*37.0,r.center().y-14.0),
            vec2(32.0,28.0));
        let response=ui.interact(rect,ui.id().with(("main-size",idx)),Sense::click());
        let hover=motion_value(ui,response.id,response.hovered(),0.12);
        p.rect_filled(rect,7.0,Color32::from_rgb(
            (32.0+35.0*hover) as u8,(33.0+12.0*hover) as u8,
            (46.0+18.0*hover) as u8));
        p.rect_stroke(rect,7.0,Stroke::new(0.8_f32,V25_EDGE),StrokeKind::Inside);
        p.text(rect.center(),egui::Align2::CENTER_CENTER,
            glyph,FontId::proportional(17.0),WHITE);
        if response.clicked(){app.resize_main_window(ctx,if idx==0{-1}else{1});}
    }
    for idx in 0..3 {
        let rect=Rect::from_min_size(pos2(controls_x+idx as f32*(window_control_w+gap),
            r.center().y-16.0),vec2(window_control_w,32.0));
        let response=ui.interact(rect,ui.id().with(("v25-viewport",idx)),Sense::click());
        let over=motion_value(ui,response.id,response.hovered(),0.12);
        let fill=if idx==2{
            Color32::from_rgb((44.0+124.0*over) as u8,
                (30.0+13.0*over) as u8,(45.0+34.0*over) as u8)
        }else{
            Color32::from_rgb((35.0+25.0*over) as u8,
                (35.0+12.0*over) as u8,(47.0+14.0*over) as u8)
        };
        p.rect_filled(rect,8.0,fill);
        let c=rect.center();
        match idx{
            0=>{p.line_segment([c+vec2(-5.0,0.0),c+vec2(5.0,0.0)],
                Stroke::new(1.5_f32,WHITE));},
            1=>{p.rect_stroke(Rect::from_center_size(c,vec2(10.0,9.0)),1.0,
                Stroke::new(1.4_f32,WHITE),StrokeKind::Inside);},
            _=>{
                p.line_segment([c+vec2(-4.0,-4.0),c+vec2(4.0,4.0)],Stroke::new(1.5_f32,WHITE));
                p.line_segment([c+vec2(-4.0,4.0),c+vec2(4.0,-4.0)],Stroke::new(1.5_f32,WHITE));
            }
        }
        if response.clicked(){
            match idx{
                0=>ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
                1=>{
                    let max=ctx.input(|i|i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
                },
                _=>ctx.send_viewport_cmd(ViewportCommand::Close),
            }
        }
    }
    let drag=Rect::from_min_max(r.min,pos2(controls_x-210.0,r.bottom()));
    if ui.interact(drag,ui.id().with("v25-header-drag"),Sense::click_and_drag()).dragged(){
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}
fn v25_sidebar_item(app:&mut BoostApp,ui:&mut Ui,page:Page,title:&str,compact:bool){
    let small=ui.ctx().content_rect().height()<660.0;
    let height=if compact{if small{25.0}else{31.0}}else{if small{31.0}else{38.0}};
    let (r,resp)=ui.allocate_exact_size(vec2(ui.available_width(),height),Sense::click());
    let hover=motion_value(ui,resp.id.with("nav-v25"),resp.hovered(),0.16);
    let active=app.page==page;
    let p=ui.painter();
    if active {
        p.rect_filled(r,9.0,Color32::from_rgb(48,13,25));
        p.rect_stroke(r,9.0,Stroke::new(0.9_f32,Color32::from_rgb(174,65,93)),StrokeKind::Inside);
        p.rect_filled(Rect::from_min_size(r.min+vec2(1.0,6.0),
            vec2(3.0,height-12.0)),2.0,V25_ACCENT);
    }else if hover>0.0{
        p.rect_filled(r,9.0,Color32::from_rgba_unmultiplied(73,65,85,(60.0*hover) as u8));
    }
    let icon_color=if active{WHITE}else{Color32::from_rgb(187,168,191)};
    modern_icon(p,pos2(r.left()+20.0,r.center().y),page,icon_color,
        if compact{0.65}else{0.77});
    p.text(pos2(r.left()+40.0,r.center().y),egui::Align2::LEFT_CENTER,
        tr(ui,title),FontId::proportional(if compact{11.7}else{13.0}),
        if active{WHITE}else{Color32::from_rgb(205,207,221)});
    if page==Page::FrameGen{
        p.circle_filled(r.right_center()-vec2(13.0,0.0),3.2,
            Color32::from_rgb(245,185,119));
    }else if page==Page::Filters&&v22_gpu_active(app){
        p.circle_filled(r.right_center()-vec2(12.0,0.0),3.2,V25_MINT);
    }
    if resp.clicked(){app.page=page;}
    ui.add_space(if compact{2.0}else{3.0});
}
fn v25_sidebar(app:&mut BoostApp,ui:&mut Ui){
    ui.add_space(3.0);
    ui.label(RichText::new(local(ui,"WORKSPACE","PRACOVNÍ PLOCHA")).strong().size(9.5)
        .color(Color32::from_rgb(153,159,179)));
    ui.add_space(7.0);
    for (page,name) in [
        (Page::Home,"Overview"),(Page::Filters,"GPU Studio"),
        (Page::Scaling,"Upscaling"),(Page::Performance,"FPS & Stability"),
        (Page::Extras,"Game Tools")
    ]{v25_sidebar_item(app,ui,page,name,false);}
    ui.add_space(10.0);
    ui.label(RichText::new(local(ui,"TOOLS","NÁSTROJE")).strong().size(9.5)
        .color(Color32::from_rgb(153,159,179)));
    ui.add_space(5.0);
    for (page,name) in [
        (Page::Overlay,"Game Overlay"),(Page::FrameGen,"Motion Lab"),
        (Page::Capture,"Capture"),(Page::Library,"Game Library"),
        (Page::Settings,"Settings")
    ]{v25_sidebar_item(app,ui,page,name,true);}
    ui.add_space(5.);
    ui.separator();
    let mode=if app.settings.ui_advanced{local(ui,"Simple mode","Jednoduchý režim")}else{local(ui,"Advanced mode","Pokročilý režim")};
    if ui.add_sized([ui.available_width(),29.],egui::Button::new(mode)).clicked(){app.settings.ui_advanced=!app.settings.ui_advanced;app.save();}
    ui.horizontal(|ui|{for(i,n)in ["EN","CZ"].iter().enumerate(){if ui.selectable_value(&mut app.settings.ui_language,i as u8,*n).changed(){app.save();ui.ctx().request_repaint();}}});
    if ui.available_height()>24.{ui.colored_label(V25_MINT,"D3D11 / WGC");}

}


/// Compact homescreen still has art and all four actions, but does not
/// bury workspace shortcuts below the fold at 840x560 or 940x620.
fn v25_hero_compact(app:&mut BoostApp,ui:&mut Ui,width:f32){
    let (r,_)=ui.allocate_exact_size(vec2(width,105.0),Sense::hover());
    let p=ui.painter().with_clip_rect(r);
    p.rect_filled(r,12.0,Color32::from_rgb(27,27,44));
    if let Some(image)=app.hero_picture.as_ref(){
        p.image(image.id(),r.shrink(1.0),Rect::from_min_max(pos2(0.0,0.0),pos2(1.0,1.0)),
            Color32::from_rgb(173,173,199));
        v25_gradient(&p,r,Color32::from_rgba_unmultiplied(11,12,24,246),
            Color32::from_rgba_unmultiplied(18,15,31,50));
    }
    // Subtle animated scan line and faceted energy motif on the compact hero.
    let t=if app.settings.animations{app.app_start.elapsed().as_secs_f32()}else{0.0};
    let center=pos2(r.right()-86.0,r.top()+45.0);
    for i in 0..3{
        let radius=24.0+i as f32*16.0+(t*1.3).sin()*2.0;
        let points=(0..=6).map(|j|{
            let a=j as f32*std::f32::consts::TAU/6.0+t*0.15*(i as f32+1.0);
            center+vec2(a.cos()*radius,a.sin()*radius)
        }).collect::<Vec<_>>();
        p.add(egui::Shape::line(points,Stroke::new(0.8_f32,Color32::from_rgba_unmultiplied(255,82,113,52-i*12))));
    }
    p.rect_stroke(r,12.0,Stroke::new(0.85_f32,
        Color32::from_rgb(78,68,91)),StrokeKind::Inside);
    let x=r.left()+18.0;let y=r.top()+13.0;
    p.rect_filled(Rect::from_min_size(pos2(x,y),vec2(3.0,11.0)),
        1.5,V25_ACCENT);
    p.text(pos2(x+11.0,y+5.0),egui::Align2::LEFT_CENTER,
        "ACTIVE SESSION  /  GAME",FontId::proportional(9.8),
        Color32::from_rgb(215,189,203));
    p.text(pos2(x,y+25.0),egui::Align2::LEFT_TOP,
        trim(&v22_game_name(app),24),FontId::proportional(21.0),WHITE);
    let detected=app.telem.active_game.is_some();
    p.circle_filled(pos2(x+4.0,y+70.0),3.6,
        if detected{V25_MINT}else{Color32::from_rgb(243,173,114)});
    p.text(pos2(x+15.0,y+70.0),egui::Align2::LEFT_CENTER,
        if detected{"GAME DETECTED · NATIVE D3D11"}
        else{"SELECT A GAME · EXTERNAL D3D11"},
        FontId::proportional(10.3),MUTED);
    let art=Rect::from_min_size(pos2(r.right()-115.0,r.bottom()-35.0),
        vec2(99.0,25.0));
    p.rect_filled(art,7.0,Color32::from_rgba_unmultiplied(43,28,55,230));
    p.rect_stroke(art,7.0,Stroke::new(0.8_f32,V25_ACCENT),StrokeKind::Inside);
    p.text(art.center(),egui::Align2::CENTER_CENTER,"CHANGE ART",
        FontId::proportional(10.2),WHITE);
    if ui.interact(art,ui.id().with("v29-hero-art"),Sense::click()).clicked(){
        if let Some(path)=crate::filedialog::select_image(){
            app.change_hero_picture(&path,ui.ctx());
        }
    }
}
fn v25_hero(app:&mut BoostApp,ui:&mut Ui,width:f32){
    let (r,_)=ui.allocate_exact_size(vec2(width,178.0),Sense::hover());
    let p=ui.painter().with_clip_rect(r);
    p.rect_filled(r,14.0,Color32::from_rgb(27,27,44));
    if let Some(img)=&app.hero_picture{
        let inner=r.shrink(1.0);
        p.image(img.id(),inner,Rect::from_min_max(pos2(0.0,0.0),pos2(1.0,1.0)),
            Color32::from_rgb(180,183,207));
        v25_gradient(&p,r,
            Color32::from_rgba_unmultiplied(12,12,23,244),
            Color32::from_rgba_unmultiplied(16,13,27,42));
    }else{
        v25_gradient(&p,r,Color32::from_rgb(24,24,40),
            Color32::from_rgb(52,33,66));
        for j in 0..12{
            let y=r.top()+23.0+j as f32*16.0;
            p.line_segment([pos2(r.right()-330.0,y),
                pos2(r.right()-10.0,y)],
                Stroke::new(0.7_f32,Color32::from_rgba_unmultiplied(173,129,189,18)));
        }
    }
    let time=app.app_start.elapsed().as_secs_f32();
    if app.settings.animations{
        let bx=r.right()-93.0;let by=r.top()+65.0;
        let breath=(time*0.9).sin()*3.0;
        for j in 0..3{
            p.circle_stroke(pos2(bx,by),36.0+j as f32*22.0+breath,
                Stroke::new(0.85_f32,Color32::from_rgba_unmultiplied(
                    247,97,147,(55-j*12) as u8)));
        }
        p.line_segment([pos2(bx-12.0,by),pos2(bx+12.0,by)],
            Stroke::new(1.0_f32,V25_ACCENT));
        p.line_segment([pos2(bx,by-12.0),pos2(bx,by+12.0)],
            Stroke::new(1.0_f32,V25_ACCENT));
    }
    p.rect_stroke(r,14.0,Stroke::new(0.9_f32,Color32::from_rgb(78,68,91)),
        StrokeKind::Inside);
    let x=r.left()+23.0;let y=r.top()+19.0;
    p.rect_filled(Rect::from_min_size(pos2(x,y),vec2(3.0,13.0)),1.5,V25_ACCENT);
    p.text(pos2(x+12.0,y+6.0),egui::Align2::LEFT_CENTER,
        "ACTIVE SESSION  /  SELECTED GAME",FontId::proportional(10.3),
        Color32::from_rgb(215,189,203));
    let game=v22_game_name(app);
    p.text(pos2(x,y+35.0),egui::Align2::LEFT_TOP,trim(&game,36),
        FontId::proportional(25.0),WHITE);
    let running=app.telem.active_game.is_some();
    let selected_status=if running{"GAME DETECTED"}else{"SELECT A RUNNING GAME"};
    p.circle_filled(pos2(x+4.0,y+86.0),4.0,
        if running{V25_MINT}else{Color32::from_rgb(245,181,121)});
    p.text(pos2(x+15.0,y+86.0),egui::Align2::LEFT_CENTER,
        selected_status,FontId::proportional(11.4),
        if running{V25_MINT}else{MUTED});
    let (sw,sh)=crate::win::screen_size();
    p.text(pos2(x,y+111.0),egui::Align2::LEFT_TOP,
        format!("DISPLAY  {sw} × {sh}  ·  NATIVE D3D11 / WGC"),
        FontId::proportional(10.9),Color32::from_rgb(199,190,205));
    p.text(pos2(x,y+131.0),egui::Align2::LEFT_TOP,
        "External processing · no game DLL injection",
        FontId::proportional(10.2),Color32::from_rgb(165,162,179));

    if r.width()>665.0{
        let info=Rect::from_min_size(pos2(r.right()-203.0,r.top()+17.0),
            vec2(183.0,49.0));
        p.rect_filled(info,9.0,Color32::from_rgba_unmultiplied(17,19,29,216));
        p.rect_stroke(info,9.0,Stroke::new(0.8_f32,
            Color32::from_rgba_unmultiplied(138,124,161,110)),StrokeKind::Inside);
        p.text(info.left_top()+vec2(12.0,9.0),egui::Align2::LEFT_TOP,
            "GPU OUTPUT STATUS",FontId::proportional(9.7),MUTED);
        let active=app.native_mirror.as_ref().filter(|m|m.running());
        let label=match active{
            None=>"DISABLED",
            Some(m) if m.multiplier()>1=>"EXPERIMENTAL PREVIEW",
            Some(m) if m.mode()==crate::native_mirror::MirrorMode::SeparateWindow=>
                "SEPARATE 1× PREVIEW",
            Some(_)=>"GLASS  /  NATIVE 1×",
        };
        p.text(info.left_top()+vec2(12.0,26.0),egui::Align2::LEFT_TOP,
            label,FontId::proportional(11.8),
            if active.is_some(){V25_MINT}else{Color32::from_rgb(230,194,200)});
    }
    let art=Rect::from_min_size(pos2(r.right()-128.0,r.bottom()-36.0),
        vec2(108.0,25.0));
    p.rect_filled(art,7.0,Color32::from_rgba_unmultiplied(44,29,55,230));
    p.rect_stroke(art,7.0,Stroke::new(0.9_f32,V25_ACCENT),StrokeKind::Inside);
    p.text(art.center(),egui::Align2::CENTER_CENTER,"CHANGE ART",
        FontId::proportional(10.6),WHITE);
    if ui.interact(art,ui.id().with("v25-hero-art"),Sense::click()).clicked(){
        if let Some(path)=crate::filedialog::select_image(){
            app.change_hero_picture(&path,ui.ctx());
        }
    }
}
fn v25_route(app:&mut BoostApp,ui:&mut Ui,width:f32,page:Page,name:&str,description:&str,
    accent:Color32,index:usize){
    let (r,resp)=ui.allocate_exact_size(vec2(width,72.0),Sense::click());
    let hover=motion_value(ui,resp.id.with(("route-v25",index)),
        resp.hovered(),0.20);
    v25_surface(ui.painter(),r,false,hover);
    let p=ui.painter();
    let symbol=Rect::from_min_size(r.min+vec2(12.0,14.0),vec2(43.0,43.0));
    p.rect_filled(symbol,10.0,Color32::from_rgba_unmultiplied(
        accent.r(),accent.g(),accent.b(),36));
    modern_icon(p,symbol.center(),page,accent,0.85);
    p.text(r.min+vec2(66.0,16.0),egui::Align2::LEFT_TOP,
        name,FontId::proportional(13.4),WHITE);
    p.text(r.min+vec2(66.0,39.0),egui::Align2::LEFT_TOP,
        trim(description,49),FontId::proportional(10.9),MUTED);
    let arrow=pos2(r.right()-17.0,r.center().y);
    let stroke=Stroke::new(1.2+0.3*hover,accent);
    p.line_segment([arrow-vec2(7.0,0.0),arrow+vec2(1.0,0.0)],stroke);
    p.line_segment([arrow+vec2(-3.0,-4.0),arrow+vec2(1.0,0.0)],stroke);
    p.line_segment([arrow+vec2(-3.0,4.0),arrow+vec2(1.0,0.0)],stroke);
    if resp.clicked(){app.page=page;}
}
fn v25_home(app:&mut BoostApp,ui:&mut Ui){
    let width=ui.available_width().max(320.0);
    ui.horizontal_wrapped(|ui|{
        ui.label(RichText::new("Control Center").size(25.0).strong().color(WHITE));
        ui.add_space(8.0);
        caption(ui,"Graphics, performance and gameplay in one place");
    });
    ui.add_space(10.0);
    let compact_home=width<660.0||ui.ctx().content_rect().height()<700.0;
    if compact_home{v25_hero_compact(app,ui,width);}
    else{v25_hero(app,ui,width);}
    ui.add_space(if compact_home{8.0}else{12.0});
    let cols=if width<660.0{2}else{4};
    let gap=9.0;
    let w=((width-gap*(cols as f32-1.0))/cols as f32).max(130.0);
    for row in 0..(4+cols-1)/cols{
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=gap;
            for column in 0..cols{
                let idx=row*cols+column;
                match idx{
                    0=>{
                        let on=v22_gpu_active(app);
                        if v25_action(ui,w,if on{"STOP GPU OUTPUT"}else{"APPLY GPU LOOK"},
                            if on{"Return to original"}else{"Native Glass · 1×"},
                            Page::Filters,!on,true){
                            if on{app.stop_native_mirror();}
                            else{app.toggle_native_mirror_factor(
                                crate::native_mirror::MirrorMode::CompositedInPlace,1);}
                        }
                    },
                    1=>{
                        if v25_action(ui,w,"LIVE PREVIEW","Separate window · 1×",
                            Page::Scaling,false,true){
                            if v22_gpu_active(app){app.stop_native_mirror();}
                            app.toggle_native_mirror_factor(
                                crate::native_mirror::MirrorMode::SeparateWindow,1);
                        }
                    },
                    2=>{
                        if v25_action(ui,w,if app.overlay_open{"HIDE HUD"}else{"OPEN GAME HUD"},
                            "Alt+Z · quick controls",Page::Overlay,false,true){
                            app.overlay_open=!app.overlay_open;
                            if app.overlay_open{
                                app.overlay_edit_mode=false;
                                app.overlay_expanded=false;
                            }
                        }
                    },
                    3=>{
                        if v25_action(ui,w,"CHOOSE GAME","Per-game profiles",
                            Page::Library,false,true){app.page=Page::Library;}
                    },
                    _=>{}
                }
            }
        });
        ui.add_space(8.0);
    }
    ui.horizontal(|ui|{
        if v25_pill(ui,"EDIT HUD CONTROLS",false,150.0){
            app.overlay_open=true;
            app.overlay_edit_mode=true;
        }
        caption(ui,"Alt+Z: small HUD / hide   ·   Alt+Shift+Z: edit / minimize");
    });
    ui.add_space(5.0);
    ui.horizontal(|ui|{
        ui.label(RichText::new("YOUR WORKSPACES").size(11.0).strong()
            .color(Color32::from_rgb(203,161,184)));
        ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
            caption(ui,if app.settings.animations{"SMOOTH MOTION UI"}else{"REDUCED MOTION UI"});
        });
    });
    ui.add_space(7.0);
    let cols=if width<590.0{1}else{2};
    let gap=10.0;
    let w=(width-gap*(cols as f32-1.0))/cols as f32;
    let entries=[
        (Page::Filters,"GPU Studio","83 native looks · 60 HLSL effects",V25_ACCENT),
        (Page::Scaling,"Spatial Upscaling","10 independent D3D11 upscalers",V25_LAVENDER),
        (Page::Performance,"FPS & Stability","Measured ETW FPS · frame pacing",V25_MINT),
        (Page::Extras,"Game Tools","HUD · macros · zoom",Color32::from_rgb(245,179,116)),
    ];
    for (row,items) in entries.chunks(cols).enumerate(){
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=gap;
            for (col,(page,name,description,color)) in items.iter().enumerate(){
                v25_route(app,ui,w,*page,name,description,*color,row*cols+col);
            }
        });
        ui.add_space(8.0);
    }
    let (r,resp)=ui.allocate_exact_size(vec2(width,47.0),Sense::click());
    let hover=motion_value(ui,resp.id,resp.hovered(),0.18);
    v25_surface(ui.painter(),r,false,hover);
    let p=ui.painter();
    modern_icon(p,pos2(r.left()+22.0,r.center().y),
        Page::FrameGen,Color32::from_rgb(246,177,123),0.75);
    p.text(r.min+vec2(41.0,10.0),egui::Align2::LEFT_TOP,"MOTION LAB",
        FontId::proportional(11.7),WHITE);
    p.text(r.min+vec2(41.0,28.0),egui::Align2::LEFT_TOP,
        "Optical-flow x2/x3: opt-in preview only · not game FPS",
        FontId::proportional(10.0),MUTED);
    p.text(pos2(r.right()-17.0,r.center().y),
        egui::Align2::RIGHT_CENTER,"EXPLORE  ›",
        FontId::proportional(11.0),Color32::from_rgb(245,181,127));
    if resp.clicked(){app.page=Page::FrameGen;}
    ui.add_space(9.0);
    ui.horizontal(|ui|{
        ui.label(RichText::new("LIVE TELEMETRY").size(10.7).strong()
            .color(Color32::from_rgb(202,170,190)));
        ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
            caption(ui,"ETW data only · never synthetic game FPS");
        });
    });
    ui.add_space(6.0);
    let gap=9.0;
    let w=(width-3.0*gap)/4.0;
    let capture=app.native_mirror.as_ref()
        .and_then(|m|m.capture_average_fps())
        .map(|v|format!("{v:.1}")).unwrap_or_else(||"—".to_owned());
    ui.horizontal(|ui|{
        ui.spacing_mut().item_spacing.x=gap;
        v25_metric(ui,w,"GAME FPS",&fps_text(app,"fps"),V25_MINT);
        v25_metric(ui,w,"FRAME TIME",&fps_text(app,"frametime"),
            Color32::from_rgb(241,191,122));
        v25_metric(ui,w,"1% LOW",&fps_text(app,"low"),V25_LAVENDER);
        v25_metric(ui,w,"WGC PROCESSED",&capture,V25_ACCENT);
    });
}

fn v25_overlay_height(app:&BoostApp)->f32{
    let mut target:f32=if app.overlay_expanded{
        if app.settings.experimental_fg_allowed{648.0}else{602.0}
    }else{423.0};
    if app.overlay_expanded&&app.driver_status.as_ref().is_some_and(|d|
        d.image_sharpen.supported||d.anti_lag.supported){
        target+=59.0;
    }
    // Stay on-screen on 720p monitors; contents scroll when space is limited.
    let desktop_height=crate::win::screen_size().1 as f32;
    target.min((desktop_height-72.0).max(350.0))
}
fn v25_overlay_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui,width:f32){
    let (r,_)=ui.allocate_exact_size(vec2(width,43.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(r,10.0,Color32::from_rgb(26,27,39));
    p.rect_stroke(r,10.0,Stroke::new(0.8_f32,V25_EDGE),StrokeKind::Inside);
    let x=r.left()+14.0;
    let y=r.center().y;
    p.add(egui::Shape::convex_polygon(vec![
        pos2(x,y+10.0),pos2(x+13.0,y-10.0),pos2(x+27.0,y+10.0)
    ],V25_ACCENT,Stroke::NONE));
    p.text(pos2(x+36.0,y),egui::Align2::LEFT_CENTER,
        "HORIZON",FontId::proportional(16.7),WHITE);
    p.text(pos2(x+127.0,y),egui::Align2::LEFT_CENTER,
        "BOOST",FontId::proportional(16.7),V25_ACCENT);
    let status=Rect::from_min_size(pos2(x+205.0,y-11.0),vec2(88.0,22.0));
    p.rect_filled(status,6.0,Color32::from_rgb(31,65,52));
    p.circle_filled(status.left_center()+vec2(12.0,0.0),3.0,V25_MINT);
    p.text(status.left_center()+vec2(21.0,0.0),egui::Align2::LEFT_CENTER,
        "GAME HUD",FontId::proportional(9.6),V25_MINT);
    if app.settings.animations{
        let t=app.app_start.elapsed().as_secs_f32();
        let x0=r.left()+19.0+((t*65.0)%115.0);
        p.line_segment([pos2(x0,r.bottom()-1.0),pos2(x0+26.0,r.bottom()-1.0)],
            Stroke::new(1.3_f32,Color32::from_rgba_unmultiplied(255,95,132,110)));
    }
    let close=Rect::from_center_size(pos2(r.right()-20.0,y),vec2(29.0,29.0));
    let resp=ui.interact(close,ui.id().with("v25-hud-close"),Sense::click());
    p.rect_filled(close,8.0,if resp.hovered(){
        Color32::from_rgb(136,45,73)
    }else{Color32::from_rgb(52,40,59)});
    p.line_segment([close.center()+vec2(-4.0,-4.0),
        close.center()+vec2(4.0,4.0)],Stroke::new(1.6_f32,WHITE));
    p.line_segment([close.center()+vec2(-4.0,4.0),
        close.center()+vec2(4.0,-4.0)],Stroke::new(1.6_f32,WHITE));
    if resp.clicked(){app.overlay_open=false;}
    let handle=Rect::from_min_max(r.min,pos2(close.left()-8.0,r.bottom()));
    if ui.interact(handle,ui.id().with("v25-hud-drag"),Sense::click_and_drag()).dragged(){
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}
fn v25_overlay_action(ui:&mut Ui,width:f32,title:&str,page:Page,
    selected:bool,primary:bool)->bool{
    let (r,resp)=ui.allocate_exact_size(vec2(width,38.0),Sense::click());
    let hover=motion_value(ui,resp.id.with("ov-action-v25"),
        resp.hovered(),0.15);
    let p=ui.painter();
    let fill=if selected||primary{
        Color32::from_rgb((98.0+24.0*hover) as u8,
            (32.0+8.0*hover) as u8,(57.0+9.0*hover) as u8)
    }else{
        Color32::from_rgb((33.0+18.0*hover) as u8,
            (34.0+8.0*hover) as u8,(46.0+8.0*hover) as u8)
    };
    p.rect_filled(r,9.0,fill);
    p.rect_stroke(r,9.0,Stroke::new(0.9_f32,if selected||primary{
        V25_ACCENT
    }else{V25_EDGE}),StrokeKind::Inside);
    modern_icon(p,pos2(r.left()+21.0,r.center().y),page,
        if selected||primary{WHITE}else{Color32::from_rgb(205,185,210)},0.65);
    p.text(pos2(r.left()+39.0,r.center().y),egui::Align2::LEFT_CENTER,
        title,FontId::proportional(11.3),WHITE);
    resp.clicked()
}
fn v25_overlay_status(app:&BoostApp,ui:&mut Ui,width:f32){
    let (r,_)=ui.allocate_exact_size(vec2(width,43.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(r,10.0,Color32::from_rgb(27,29,40));
    p.rect_stroke(r,10.0,Stroke::new(0.8_f32,V25_EDGE),StrokeKind::Inside);
    p.circle_filled(r.left_center()+vec2(13.0,0.0),3.5,
        if app.telem.active_game.is_some(){V25_MINT}
        else{Color32::from_rgb(211,170,123)});
    p.text(r.left_top()+vec2(25.0,7.0),egui::Align2::LEFT_TOP,
        "ACTIVE GAME",FontId::proportional(9.3),MUTED);
    p.text(r.left_top()+vec2(25.0,23.0),egui::Align2::LEFT_TOP,
        trim(&v22_game_name(app),24),FontId::proportional(12.0),WHITE);
    let gpu=app.native_mirror.as_ref().filter(|m|m.running());
    let mode=match gpu{
        None=>"OFF",
        Some(m) if m.multiplier()>1=>"PREVIEW FG",
        Some(m) if m.mode()==crate::native_mirror::MirrorMode::SeparateWindow=>
            "OUTPUT PREVIEW 1×",
        Some(_)=>"NATIVE GLASS 1×"
    };
    p.text(r.right_top()-vec2(13.0,-9.0),egui::Align2::RIGHT_TOP,
        "OWN GPU OUTPUT",FontId::proportional(9.3),MUTED);
    p.text(r.right_top()-vec2(13.0,-25.0),egui::Align2::RIGHT_TOP,
        mode,FontId::proportional(11.3),
        if gpu.is_some(){V25_MINT}else{MUTED});
}
fn v25_overlay_advanced(app:&mut BoostApp,ui:&mut Ui){
    Frame::new().fill(Color32::from_rgb(24,25,37))
        .stroke(Stroke::new(0.8_f32,V25_EDGE)).corner_radius(10)
        .inner_margin(egui::Margin::same(11)).show(ui,|ui|{
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui|{
                caption(ui,"UPSCALE ENGINE");
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    caption(ui,"D3D11  /  NO MAGPIE");
                });
            });
            ui.add_space(5.0);
            ui.horizontal(|ui|{
                caption(ui,"ALGORITHM");
                egui::ComboBox::from_id_salt("v25-hud-upscale")
                    .width(222.0)
                    .selected_text(crate::spatial::MODES[
                        app.settings.native_scale_mode.min((crate::spatial::MODES.len()-1)as u8) as usize])
                    .show_ui(ui,|ui|{
                        let mut choice=app.settings.native_scale_mode;
                        for (i,label) in crate::spatial::MODES.iter().enumerate(){
                            ui.selectable_value(&mut choice,i as u8,*label);
                        }
                        if choice!=app.settings.native_scale_mode{
                            app.apply_scale_algorithm(choice);
                        }
                    });
            });
            ui.horizontal(|ui|{
                caption(ui,"GPU PASSES");
                egui::ComboBox::from_id_salt("v25-hud-pass")
                    .width(157.0)
                    .selected_text(match app.settings.native_quality{
                        3=>"Adaptive",2=>"Eco / 1",1=>"Balanced / 2",_=>"Full"
                    })
                    .show_ui(ui,|ui|{
                        let mut q=app.settings.native_quality;
                        for (value,label) in [
                            (3u8,"Adaptive"),(2,"Eco / 1 pass"),
                            (1,"Balanced / 2 passes"),(0,"Full")
                        ]{
                            ui.selectable_value(&mut q,value,label);
                        }
                        if q!=app.settings.native_quality{
                            app.settings.native_quality=q;
                            app.save();app.refresh_native_mirror();
                        }
                    });
            });
            ui.add_space(5.0);
            let mut allowed=app.settings.experimental_fg_allowed;
            if ui.checkbox(&mut allowed,"Experimental FG · preview only").changed(){
                app.settings.experimental_fg_allowed=allowed;
                if !allowed{
                    if app.native_mirror.as_ref().is_some_and(|m|m.multiplier()>1){
                        app.stop_native_mirror();
                    }
                    app.settings.external_fg_factor=1;
                }else if app.settings.external_fg_factor<2{
                    app.settings.external_fg_factor=2;
                }
                app.save();
            }
            if allowed{
                ui.horizontal(|ui|{
                    for factor in [2u8,3u8]{
                        let same=app.native_mirror.as_ref().is_some_and(|m|
                            m.running()&&m.multiplier()==factor&&
                            m.mode()==crate::native_mirror::MirrorMode::SeparateWindow);
                        let label=if same{
                            format!("Stop preview ×{factor}")
                        }else{format!("Test FG ×{factor}")};
                        if v25_pill(ui,&label,same,136.0){
                            if app.native_mirror.is_some(){app.stop_native_mirror();}
                            if !same{
                                app.settings.external_fg_factor=factor;
                                app.save();
                                app.toggle_native_mirror_factor(
                                    crate::native_mirror::MirrorMode::SeparateWindow,factor);
                            }
                        }
                    }
                });
            }
            caption(ui,"These settings affect Horizon's output. Generated frames do not change the game engine FPS.");
            if let Some(driver)=app.driver_status.clone(){
                if driver.image_sharpen.supported||driver.anti_lag.supported{
                    ui.add_space(5.0);
                    ui.separator();
                    ui.horizontal_wrapped(|ui|{
                        caption(ui,"AMD DRIVER");
                        if driver.image_sharpen.supported{
                            let name=if driver.image_sharpen.enabled{"RIS  ON"}
                                else{"RIS  OFF"};
                            if v25_pill(ui,name,driver.image_sharpen.enabled,98.0)
                                &&!app.driver_busy{
                                app.driver_action(adlx::DriverChange::RadeonSharpen{
                                    enabled:!driver.image_sharpen.enabled,
                                    level:app.driver_strength});
                            }
                        }
                        if driver.anti_lag.supported{
                            let name=if driver.anti_lag.enabled{"ANTI-LAG  ON"}
                                else{"ANTI-LAG  OFF"};
                            if v25_pill(ui,name,driver.anti_lag.enabled,124.0)
                                &&!app.driver_busy{
                                app.driver_action(adlx::DriverChange::AntiLag(
                                    !driver.anti_lag.enabled));
                            }
                        }
                    });
                    caption(ui,"Driver state from ADLX. Changes only on click.");
                }
            }
        });
}
/// Responsive overlay actions: no text overflows into neighbors.
fn v25_overlay_quick_action(app:&mut BoostApp,ui:&mut Ui,w:f32,idx:usize){
    match idx{
        0=>{
            let active=v22_gpu_active(app);
            if v25_overlay_action(ui,w,if active{"GPU ON"}else{"ENABLE GPU"},
                Page::Filters,active,false){
                if active{app.stop_native_mirror();}
                else{app.toggle_native_mirror_factor(
                    crate::native_mirror::MirrorMode::CompositedInPlace,1);}
            }
        },
        1=>{
            let active=app.settings.crosshair.enabled;
            if v25_overlay_action(ui,w,if active{"AIM ON"}else{"CROSSHAIR"},
                Page::Extras,active,false){app.toggle_crosshair();}
        },
        2=>{
            let active=app.game_timer.running();
            if v25_overlay_action(ui,w,if active{"PAUSE"}else{"TIMER"},
                Page::Performance,active,false){app.toggle_game_timer();}
        },
        3=>{
            if v25_overlay_action(ui,w,"CAPTURE",Page::Capture,false,false){
                app.screenshot();
            }
        },
        _=>{}
    }
}
fn v25_overlay(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let width=ui.available_width().max(330.0);
    ui.set_width(width);
    v25_overlay_header(app,ctx,ui,width);
    ui.add_space(8.0);
    let gap=7.0;
    let metric_width=(width-2.0*gap)/3.0;
    ui.horizontal(|ui|{
        ui.spacing_mut().item_spacing.x=gap;
        v25_metric(ui,metric_width,"GAME FPS / ETW",&fps_text(app,"fps"),V25_MINT);
        v25_metric(ui,metric_width,"FRAME TIME",&fps_text(app,"frametime"),
            Color32::from_rgb(244,189,120));
        v25_metric(ui,metric_width,"1% LOW",&fps_text(app,"low"),V25_LAVENDER);
    });
    ui.add_space(8.0);
    v25_overlay_status(app,ui,width);
    ui.add_space(9.0);
    ui.label(RichText::new("QUICK CONTROLS").strong().size(9.8)
        .color(Color32::from_rgb(195,164,183)));
    ui.add_space(5.0);
    // At <=460 logical px, four buttons cause text to overlap next button.
    // Use two spacious rows and keep every action fully clickable.
    let quick_cols=if width<460.0{2}else{4};
    let btn_w=(width-gap*(quick_cols as f32-1.0))/quick_cols as f32;
    for row in 0..(4+quick_cols-1)/quick_cols{
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=gap;
            for col in 0..quick_cols{
                v25_overlay_quick_action(app,ui,btn_w,row*quick_cols+col);
            }
        });
        if quick_cols==2&&row==0{ui.add_space(5.0);}
    }
    ui.add_space(7.0);
    ui.horizontal_wrapped(|ui|{
        if outline_button(ui,"Save replay"){app.save_replay();}
        if outline_button(ui,if app.replay.active(){"Stop replay"}else{"Start replay"}){app.toggle_replay();}
        if app.macro_armed_pid.is_some()&&outline_button(ui,"Disarm macro"){app.macro_armed_pid=None;app.macro_runner.cancel();}
    });
    caption(ui,&format!("{} replay / {} macro",
        app.settings.global_shortcuts.label("REPLAY"),
        app.settings.global_shortcuts.label("MACRO")));
    if outline_button(ui,"STOP ALL GAME TOOLS"){app.emergency_stop();}
    ui.add_space(10.0);
    ui.label(RichText::new("LIVE COLOR PRESETS").strong().size(9.8)
        .color(Color32::from_rgb(195,164,183)));
    ui.add_space(5.0);
    let choices=[
        (64usize,"Daylight"),(65,"Shadow Pop"),(66,"Indirect"),
        (69,"Soft Glow"),(77,"Balanced")
    ];
    let slot=(width-4.0*gap)/5.0;
    ui.horizontal(|ui|{
        ui.spacing_mut().item_spacing.x=gap;
        for (i,label) in choices{
            if v25_pill(ui,label,app.settings.native_preset==i,slot){
                app.select_native_preset(i);
            }
        }
    });
    ui.add_space(9.0);
    let (r,resp)=ui.allocate_exact_size(vec2(width,34.0),Sense::click());
    let hover=motion_value(ui,resp.id.with("hud-expand"),
        resp.hovered(),0.18);
    v25_surface(ui.painter(),r,app.overlay_expanded,hover);
    let p=ui.painter();
    modern_icon(p,r.left_center()+vec2(19.0,0.0),Page::Scaling,
        if app.overlay_expanded{WHITE}else{V25_LAVENDER},0.65);
    p.text(r.left_center()+vec2(37.0,0.0),egui::Align2::LEFT_CENTER,
        "UPSCALING  /  MOTION SETTINGS",FontId::proportional(11.5),WHITE);
    p.text(r.right_center()-vec2(14.0,0.0),egui::Align2::RIGHT_CENTER,
        if app.overlay_expanded{"−"}else{"+"},FontId::proportional(17.0),
        Color32::from_rgb(231,193,207));
    if resp.clicked(){app.overlay_expanded=!app.overlay_expanded;}
    if app.overlay_expanded{
        ui.add_space(7.0);
        v25_overlay_advanced(app,ui);
    }
    ui.add_space(7.0);
    ui.horizontal_wrapped(|ui|{
        caption(ui,"Alt+Z to hide  ·  settings stay inside the HUD");
        if let Some(m)=app.native_mirror.as_ref().filter(|m|m.multiplier()>1){
            caption(ui,&format!("  ·  {} synthetic frames (own output)",m.generated_frames()));
        }
    });
}
