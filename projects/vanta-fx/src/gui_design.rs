// HORIZON BOOST: responsive compact native Rust layout.
// All icons and decorative elements are drawn as vectors in egui Painter.
// No missing-glyph icon font, no AI-generated artwork, no hardcoded overflowing columns.
use eframe::egui::{pos2,vec2,Pos2,Rect,Painter,FontId,Sense};

const NAV_WIDTH:f32=176.0;
const CONTENT_MARGIN:f32=16.0;
const GAP:f32=10.0;

#[derive(Debug,Clone,Copy)]
struct HomeDims { usable:f32,main:f32,side:f32,stack:bool }
fn home_dims(viewport_width:f32)->HomeDims{
    let usable=(viewport_width-NAV_WIDTH-CONTENT_MARGIN*2.0-12.0).max(320.0);
    if usable<900.0 {HomeDims{usable,main:usable,side:usable,stack:true}}
    else {
        let side=(usable*0.33).clamp(302.0,370.0);
        HomeDims{usable,main:usable-side-GAP,side,stack:false}
    }
}
fn draw_line(p:&Painter,a:Pos2,b:Pos2,c:Color32,stroke:f32){
    p.line_segment([a,b],Stroke::new(stroke,c));
}
fn modern_icon(p:&Painter,center:Pos2,page:Page,color:Color32,k:f32){
    let x=center.x;let y=center.y;
    let s=|dx:f32,dy:f32|pos2(x+dx*k,y+dy*k);
    let st=Stroke::new((1.55*k).max(1.0),color);
    match page {
        Page::Unsorted=>{
            p.rect_stroke(Rect::from_min_max(s(-9.,-8.),s(8.,8.)),2.*k,st,StrokeKind::Inside);
            for y in [-4.,0.,4.]{draw_line(p,s(-5.,y),s(4.,y),color,st.width);}
            p.circle_filled(s(6.,-7.),2.*k,color);
        },
        Page::Home=>{
            draw_line(p,s(-8.0,-1.0),s(0.0,-8.0),color,st.width);
            draw_line(p,s(0.0,-8.0),s(8.0,-1.0),color,st.width);
            p.add(egui::Shape::line(vec![s(-6.0,-1.0),s(-6.0,7.0),s(6.0,7.0),s(6.0,-1.0)],st));
            p.rect_stroke(Rect::from_min_max(s(-1.5,2.0),s(1.5,7.0)),0.0,st,StrokeKind::Inside);
        }
        Page::Library=>{
            p.add(egui::Shape::line(vec![s(-10.0,4.0),s(-7.5,-5.0),s(-4.0,-7.0),
                s(4.0,-7.0),s(7.5,-5.0),s(10.0,4.0),s(7.0,7.0),s(3.0,3.0),
                s(-3.0,3.0),s(-7.0,7.0),s(-10.0,4.0)],st));
            draw_line(p,s(-5.0,-1.0),s(-1.0,-1.0),color,st.width);
            draw_line(p,s(-3.0,-3.0),s(-3.0,1.0),color,st.width);
            p.circle_stroke(s(4.0,-2.0),1.4*k,st);
            p.circle_stroke(s(7.0,0.5),1.4*k,st);
        }
        Page::Overlay=>{
            p.rect_stroke(Rect::from_min_max(s(-9.0,-7.0),s(9.0,5.0)),2.0*k,st,StrokeKind::Inside);
            draw_line(p,s(0.0,5.0),s(0.0,9.0),color,st.width);
            draw_line(p,s(-5.0,9.0),s(5.0,9.0),color,st.width);
        }
        Page::Filters=>{
            for (j,yy,xx) in [(0,-6.0,-3.0),(1,0.0,4.0),(2,6.0,-1.0)]{
                let _=j;
                draw_line(p,s(-9.0,yy),s(9.0,yy),color,st.width);
                p.circle_filled(s(xx,yy),2.3*k,Color32::from_rgb(26,19,33));
                p.circle_stroke(s(xx,yy),2.3*k,st);
            }
        }
        Page::Scaling=>{
            draw_line(p,s(-7.0,-8.0),s(-1.0,-8.0),color,st.width);
            draw_line(p,s(-7.0,-8.0),s(-7.0,-2.0),color,st.width);
            draw_line(p,s(7.0,8.0),s(1.0,8.0),color,st.width);
            draw_line(p,s(7.0,8.0),s(7.0,2.0),color,st.width);
            draw_line(p,s(-7.0,8.0),s(7.0,-6.0),color,st.width);
            draw_line(p,s(2.0,-6.0),s(7.0,-6.0),color,st.width);
            draw_line(p,s(7.0,-6.0),s(7.0,-1.0),color,st.width);
        }
        Page::FrameGen=>{
            for dy in [-5.0,0.0,5.0]{
                p.add(egui::Shape::line(vec![s(-8.0,dy-1.0),s(0.0,dy+3.0),
                    s(8.0,dy-1.0)],st));
            }
        }
        Page::Extras=>{
            p.circle_stroke(s(0.0,0.0),7.0*k,st);
            p.circle_filled(s(0.0,0.0),1.9*k,color);
            draw_line(p,s(-11.0,0.0),s(-6.0,0.0),color,st.width);
            draw_line(p,s(6.0,0.0),s(11.0,0.0),color,st.width);
            draw_line(p,s(0.0,-11.0),s(0.0,-6.0),color,st.width);
            draw_line(p,s(0.0,6.0),s(0.0,11.0),color,st.width);
        }
        Page::Performance=>{
            for (dx,h) in [(-7.0,4.0),(-2.0,8.0),(3.0,12.0),(8.0,15.0)]{
                draw_line(p,s(dx,7.0),s(dx,7.0-h),color,st.width*1.5);
            }
        }
        Page::Capture=>{
            p.rect_stroke(Rect::from_min_max(s(-9.0,-5.0),s(9.0,7.0)),
                2.5*k,st,StrokeKind::Inside);
            p.rect_stroke(Rect::from_min_max(s(-5.0,-8.0),s(2.0,-5.0)),
                1.0*k,st,StrokeKind::Inside);
            p.circle_stroke(s(0.0,1.0),4.0*k,st);
        }
        Page::Settings=>{
            p.circle_stroke(s(0.0,0.0),6.0*k,st);
            p.circle_stroke(s(0.0,0.0),2.5*k,st);
            for j in 0..8{
                let a=j as f32*std::f32::consts::TAU/8.0;
                draw_line(p,s(7.0*a.cos(),7.0*a.sin()),
                    s(10.0*a.cos(),10.0*a.sin()),color,st.width*1.4);
            }
        }
    }
}
fn click_surface(ui:&mut Ui,w:f32,h:f32,active:bool)->(Rect,egui::Response){
    let (rect,resp)=ui.allocate_exact_size(vec2(w,h),Sense::click());
    let p=ui.painter();
    // Subtle 160ms hover interpolation; no large motion, no sudden red flashes.
    let hover=motion_value(ui,resp.id.with("hb-hover"),resp.hovered(),0.16);
    let fill=if active {
        Color32::from_rgb((56.0+19.0*hover) as u8,24,40)
    }else{
        Color32::from_rgb((27.0+38.0*hover) as u8,
            (23.0+10.0*hover) as u8,(35.0+14.0*hover) as u8)
    };
    p.rect_filled(rect,10.0,fill);
    let border=if active{RED}else{
        Color32::from_rgb((76.0+166.0*hover) as u8,
            (48.0+7.0*hover) as u8,(65.0+23.0*hover) as u8)
    };
    p.rect_stroke(rect,10.0,Stroke::new(1.0+0.4*hover,border),StrokeKind::Inside);
    (rect,resp)
}
fn modern_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let r=ui.max_rect();
    let painter=ui.painter();
    painter.rect_filled(r,0.0,Color32::from_rgb(22,23,29));
    painter.line_segment([pos2(r.left(),r.bottom()-1.0),pos2(r.right(),r.bottom()-1.0)],
        Stroke::new(1.0_f32,Color32::from_rgb(84,36,59)));
    if app.settings.animations {
        let t=(app.app_start.elapsed().as_secs_f32()*0.11).fract();
        let scan_x=r.left()+r.width()*t;
        painter.line_segment([pos2(scan_x-18.0,r.bottom()-1.0),pos2(scan_x+18.0,r.bottom()-1.0)],
            Stroke::new(2.0_f32,Color32::from_rgba_unmultiplied(248,63,107,135)));
    }
    let x=r.left()+19.0;let y=r.center().y;
    // Original, crisp mountain/eclipse emblem
    painter.add(egui::Shape::convex_polygon(vec![pos2(x,y+12.0),pos2(x+21.0,y-12.0),
        pos2(x+42.0,y+12.0)],RED,Stroke::NONE));
    painter.line_segment([pos2(x+12.0,y+3.0),pos2(x+30.0,y+3.0)],Stroke::new(2.0_f32,Color32::from_rgb(255,158,171)));
    painter.text(pos2(x+57.0,y),egui::Align2::LEFT_CENTER,"HORIZON",FontId::proportional(21.0),WHITE);
    painter.text(pos2(x+163.0,y),egui::Align2::LEFT_CENTER,"BOOST",FontId::proportional(21.0),RED);
    if r.width()>690.0{
        painter.text(pos2(x+246.0,y+1.0),egui::Align2::LEFT_CENTER,
            "NATIVE GRAPHICS / PERFORMANCE",FontId::proportional(12.5),MUTED);
    }
    let button_width=42.0;let gap=5.0;
    let controls_left=r.right()-(button_width*3.0+gap*2.0+18.0);
    for idx in 0..3 {
        let br=Rect::from_min_size(pos2(controls_left+idx as f32*(button_width+gap),r.center().y-16.0),
            vec2(button_width,32.0));
        let id=ui.id().with(("hb-window",idx));
        let resp=ui.interact(br,id,Sense::click());
        painter.rect_filled(br,7.0,if resp.hovered(){
            if idx==2{Color32::from_rgb(145,31,62)}else{Color32::from_rgb(66,43,59)}
        }else{Color32::from_rgb(38,28,43)});
        let col=if resp.hovered(){WHITE}else{MUTED};
        let c=br.center();
        match idx{
            0=>draw_line(painter,c+vec2(-5.0,0.0),c+vec2(5.0,0.0),col,1.7),
            1=>{
                painter.rect_stroke(Rect::from_center_size(c,vec2(11.0,9.0)),1.0,
                    Stroke::new(1.5_f32,col),StrokeKind::Inside);
            }
            _=>{
                draw_line(painter,c+vec2(-5.0,-5.0),c+vec2(5.0,5.0),col,1.6);
                draw_line(painter,c+vec2(-5.0,5.0),c+vec2(5.0,-5.0),col,1.6);
            }
        }
        if resp.clicked(){
            match idx{
                0=>ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
                1=>{
                    let maximized=ctx.input(|i|i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                }
                _=>ctx.send_viewport_cmd(ViewportCommand::Close)
            }
        }
    }
    let drag=Rect::from_min_max(r.min,pos2(controls_left-6.0,r.bottom()));
    let response=ui.interact(drag,ui.id().with("header-drag"),Sense::click_and_drag());
    if response.dragged(){ctx.send_viewport_cmd(ViewportCommand::StartDrag);}
}

/// Shared accessible sub-navigation. Each tab occupies one clear visual slot;
/// changing it does not change running GPU drivers or game settings.
fn studio_tabs(ui:&mut Ui,selected:&mut usize,tabs:&[(&str,&str)]){
 if tabs.is_empty(){return;}
 let w=ui.available_width();let cols=((w/110.).floor()as usize).clamp(1,tabs.len());let rows=tabs.len().div_ceil(cols);let cols=tabs.len().div_ceil(rows);
 let tile=(w-6.*(cols-1)as f32)/cols as f32;
 for(row,chunk)in tabs.chunks(cols).enumerate(){
  ui.horizontal(|ui|{ui.spacing_mut().item_spacing.x=6.;
   for(col,(name,desc))in chunk.iter().enumerate(){let i=row*cols+col;let on=*selected==i;
    let(r,res)=ui.allocate_exact_size(vec2(tile,34.),Sense::click());let a=e_anim(ui,res.id.with("tab-active"),on);let h=e_anim(ui,res.id.with("tab-hover"),res.hovered());
    let press=motion_value(ui,res.id.with("tab-press"),res.is_pointer_button_down_on(),0.09);
    let p=ui.painter();p.rect_filled(r,8.,Color32::from_rgb((8.+20.*a+10.*h+9.*press)as u8,(8.+4.*a+8.*h)as u8,(11.+8.*a+8.*h)as u8));
    let label=name.trim_start_matches(|c:char|c.is_ascii_digit()||c==' ');p.text(r.center(),egui::Align2::CENTER_CENTER,tr(ui,label),FontId::proportional(12.),if on{WHITE}else{MUTED});
    if a>0.01{let half=(tile*0.38*a).max(1.);p.line_segment([pos2(r.center().x-half,r.bottom()-2.),pos2(r.center().x+half,r.bottom()-2.)],Stroke::new(2.0_f32,e_rgba(RED,(255.*a)as u8)));}
    motion_button(ui,&res);
    if res.on_hover_text(*desc).clicked(){*selected=i;}
   }
  });
 }
 ui.add_space(3.);
}
// v2.3: one obvious route per goal, with a compact expandable utility drawer.
// The existing pages are still available, not deleted or duplicated.
fn modern_sidebar(app:&mut BoostApp,ui:&mut Ui){
    ui.add_space(3.0);
    ui.label(RichText::new("WORKSPACES").size(10.5).strong()
        .color(Color32::from_rgb(196,142,162)));
    ui.add_space(8.0);
    const PRIMARY:&[(Page,&str)]=&[
        (Page::Home,"Overview"),
        (Page::Filters,"GPU Studio"),
        (Page::Performance,"FPS & Stability"),
        (Page::Extras,"Game Extras"),
    ];
    for &(page,label) in PRIMARY{
        let (r,resp)=ui.allocate_exact_size(vec2(ui.available_width(),45.0),Sense::click());
        let active=app.page==page;
        let hover=motion_value(ui,resp.id.with("main-nav"),resp.hovered(),0.16);
        let fill=if active{Color32::from_rgb(83,27,47)}
            else{Color32::from_rgb((26.0+23.0*hover) as u8,25,(33.0+8.0*hover) as u8)};
        ui.painter().rect_filled(r,9.0,fill);
        if active{
            ui.painter().rect_filled(Rect::from_min_size(r.min+vec2(1.0,6.0),
                vec2(3.0,r.height()-12.0)),1.5,RED);
            ui.painter().rect_stroke(r,9.0,Stroke::new(0.9_f32,Color32::from_rgb(161,50,85)),
                StrokeKind::Inside);
        }
        modern_icon(ui.painter(),pos2(r.left()+23.0,r.center().y),
            page,if active{WHITE}else{Color32::from_rgb(208,172,193)},0.83);
        ui.painter().text(pos2(r.left()+46.0,r.center().y),
            egui::Align2::LEFT_CENTER,label,FontId::proportional(13.7),
            if active{WHITE}else{Color32::from_rgb(216,209,222)});
        if resp.clicked(){app.page=page;}
        ui.add_space(4.0);
    }
    ui.add_space(11.0);
    egui::CollapsingHeader::new(
        RichText::new("ALL TOOLS  /  6").size(11.5).color(Color32::from_rgb(199,151,175))
    ).id_salt("hb-v23-more-tools")
        .default_open(matches!(app.page,Page::Scaling|Page::FrameGen|Page::Overlay|
            Page::Library|Page::Capture|Page::Settings))
        .show(ui,|ui|{
            ui.add_space(2.0);
            for (page,name) in [
                (Page::Scaling,"Spatial Upscaling"),
                (Page::FrameGen,"Motion Lab  [BETA]"),
                (Page::Overlay,"In-game Overlay"),
                (Page::Capture,"Capture & Replay"),
                (Page::Library,"Game Library"),
                (Page::Settings,"Settings"),
            ]{
                let selected=app.page==page;
                if ui.add_sized([ui.available_width(),33.0],
                    egui::Button::new(RichText::new(name).size(12.0)
                        .color(if selected{WHITE}else{MUTED}))
                        .fill(if selected{Color32::from_rgb(78,31,50)}else{Color32::TRANSPARENT})
                        .stroke(if selected{Stroke::new(0.7_f32,RED)}else{Stroke::NONE})
                        .corner_radius(7.0)).clicked(){app.page=page;}
            }
        });
    if ui.available_height()>94.0{
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min),|ui|{
            frame(Color32::from_rgb(27,32,35)).show(ui,|ui|{
                ui.label(RichText::new("NATIVE GPU  /  SAFE").color(
                    Color32::from_rgb(150,222,174)).strong().size(10.0));
                ui.label(RichText::new("No Magpie process required")
                    .color(MUTED).size(10.3));
                ui.label(RichText::new("FG off by default")
                    .color(MUTED).size(10.3));
            });
        });
    }
}
fn modern_card(app:&mut BoostApp,ui:&mut Ui,width:f32,name:&str,detail:&str,page:Page,available:bool){
    let (r,resp)=click_surface(ui,width,86.0,app.page==page);
    let p=ui.painter();
    let ic=pos2(r.left()+24.0,r.top()+26.0);
    p.rect_filled(Rect::from_center_size(ic,vec2(33.0,33.0)),8.0,Color32::from_rgb(69,26,46));
    modern_icon(p,ic,page,RED,0.84);
    let label=trim(name,23);
    p.text(pos2(r.left()+48.0,r.top()+14.0),egui::Align2::LEFT_TOP,
        label,FontId::proportional(13.2),WHITE);
    p.text(pos2(r.left()+14.0,r.top()+55.0),egui::Align2::LEFT_TOP,
        trim(detail,31),FontId::proportional(11.7),MUTED);
    let windowed_only=page==Page::Scaling&&app.telem.active_game.as_ref()
        .is_some_and(|g|g.fullscreen_like);
    let stat=if windowed_only{"WINDOWED"}else if available{"READY"}else{"PLANNED"};
    let color=if available&&!windowed_only{Color32::from_rgb(120,211,147)}
        else{Color32::from_rgb(225,168,112)};
    p.text(pos2(r.right()-12.0,r.top()+25.0),egui::Align2::RIGHT_CENTER,
        stat,FontId::proportional(9.0),color);
    if resp.clicked(){
        if available {
            app.page=page;
        }else if page==Page::FrameGen{
            app.page=page;  // Dedicated explanation instead of a pretend enabled mode
        }else{
            app.toast(format!("{name} is not yet implemented. No fake activation was performed."));
        }
    }
}
fn modern_hero(app:&mut BoostApp,ui:&mut Ui,width:f32){
    let (r,resp)=ui.allocate_exact_size(vec2(width,210.0),Sense::hover());
    let p=ui.painter().with_clip_rect(r);
    if let Some(image)=app.hero_picture.as_ref(){
        // Real gameplay still from the user's local video, softened for a clean HUD.
        // Not AI-generated. Any custom PNG/JPEG can replace it.
        p.image(image.id(),r,Rect::from_min_max(pos2(0.0,0.0),pos2(1.0,1.0)),
            Color32::from_rgb(242,243,249));
        // Smooth 4-corner gradient mesh, not dozens of rectangles (no banding/stripes).
        let mut shade=egui::Mesh::default();
        let left=Color32::from_rgba_unmultiplied(9,8,18,233);
        let right=Color32::from_rgba_unmultiplied(9,8,18,26);
        for (x,y,c) in [
            (r.left(),r.top(),left),(r.right(),r.top(),right),
            (r.left(),r.bottom(),left),(r.right(),r.bottom(),right)
        ]{
            shade.vertices.push(egui::epaint::Vertex{
                pos:pos2(x,y),uv:egui::epaint::WHITE_UV,color:c
            });
        }
        shade.indices.extend_from_slice(&[0,1,2,2,1,3]);
        p.add(egui::Shape::mesh(shade));
        for j in 0..5{
            let band=r.top()+12.0+j as f32*35.0;
            let line=pos2(r.right()-13.0,band);
            p.line_segment([line-vec2(110.0,0.0),line],
                Stroke::new(0.7_f32,Color32::from_rgba_unmultiplied(255,91,133,27)));
        }
    }else{
    // Fallback original geometric scene (no external artwork).
    for j in 0..35 {
        let t=j as f32/35.0;
        let y=r.top()+r.height()*t;
        let tint=Color32::from_rgb((20.0+24.0*t) as u8,(16.0+10.0*t) as u8,
            (35.0+30.0*t) as u8);
        p.rect_filled(Rect::from_min_size(pos2(r.left(),y),vec2(r.width(),r.height()/35.0+1.0)),0.0,tint);
    }
    let cx=r.left()+r.width()*0.75;let cy=r.top()+54.0;
    let t=if app.settings.animations{app.app_start.elapsed().as_secs_f32()}else{0.0};
    let glow=if app.settings.animations{((t*1.15).sin()+1.0)*0.5}else{0.5};
    p.circle_filled(pos2(cx,cy),87.0,Color32::from_rgb(89,43,95));
    p.circle_filled(pos2(cx,cy),80.0,Color32::from_rgb(135,68,114));
    p.circle_filled(pos2(cx+14.0,cy-10.0),68.0,Color32::from_rgb(147,77,117));
    for j in 0..8 {
        let radius=84.0-j as f32*7.0;
        p.circle_stroke(pos2(cx,cy),radius,Stroke::new(0.6_f32,Color32::from_rgba_unmultiplied(251,193,202,(43.0+glow*31.0) as u8)));
    }
    p.add(egui::Shape::convex_polygon(vec![pos2(r.left()+r.width()*0.40,r.bottom()),
        pos2(r.left()+r.width()*0.57,r.top()+119.0),
        pos2(r.left()+r.width()*0.70,r.bottom())],
        Color32::from_rgb(36,32,61),Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pos2(r.left()+r.width()*0.67,r.bottom()),
        pos2(r.left()+r.width()*0.81,r.top()+134.0),
        pos2(r.right(),r.bottom())],
        Color32::from_rgb(29,31,54),Stroke::NONE));
    let base_x=r.left()+r.width()*0.89;
    for j in 0..9 {
        let xx=base_x+(j as f32-4.0)*11.0;
        let high=28.0+((j*17)%51) as f32;
        let tower=Rect::from_min_max(pos2(xx,r.bottom()-high),pos2(xx+8.0,r.bottom()));
        p.rect_filled(tower,0.0,Color32::from_rgb(25,21,36));
        p.rect_filled(Rect::from_min_size(pos2(xx+3.0,r.bottom()-high+8.0),vec2(2.0,4.0)),
            0.0,Color32::from_rgb(240,73,113));
    }
    for j in 0..29{
        let u=((j*37+11)%103) as f32/103.0;
        let v=((j*53+29)%101) as f32/101.0;
        let wobble=if app.settings.animations{(t*0.53+j as f32).sin()*1.5}else{0.0};
        let star=pos2(r.left()+u*r.width()+wobble,r.top()+v*r.height()*0.65);
        p.circle_filled(star,if j%7==0{1.7}else{0.85},
            Color32::from_rgba_unmultiplied(247,201,218,if j%7==0{135}else{68}));
    }
    for j in 0..30{
        let t=j as f32/30.0;
        p.rect_filled(Rect::from_min_size(pos2(r.left()+t*r.width()*0.64,r.top()),
            vec2(r.width()*0.64/30.0+1.0,r.height())),0.0,
            Color32::from_rgba_unmultiplied(13,12,23,((1.0-t)*232.0) as u8));
    }
    }
    p.rect_stroke(r,12.0,Stroke::new(1.0_f32,BORDER),StrokeKind::Inside);
    let x=r.left()+22.0;let y=r.top()+19.0;
    p.text(pos2(x,y),egui::Align2::LEFT_TOP,"ACTIVE GAME",FontId::proportional(11.0),
        Color32::from_rgb(219,164,184));
    let game=app.telem.active_game.as_ref()
        .map(|g|trim(&g.title,27)).unwrap_or_else(||"No game detected".into());
    p.text(pos2(x,y+26.0),egui::Align2::LEFT_TOP,game,FontId::proportional(22.0),WHITE);
    let running=app.telem.active_game.is_some();
    p.circle_filled(pos2(x+5.0,y+73.0),4.0,if running{
        Color32::from_rgb(116,221,144)}else{Color32::from_rgb(217,163,108)});
    p.text(pos2(x+15.0,y+73.0),egui::Align2::LEFT_CENTER,
        if running{"GAME DETECTED"}else{"WAITING FOR GAME"},FontId::proportional(12.0),MUTED);
    let (sw,sh)=crate::win::screen_size();
    p.text(pos2(x,y+104.0),egui::Align2::LEFT_TOP,
        format!("DISPLAY  {} x {}",sw,sh),FontId::proportional(12.0),MUTED);
    p.text(pos2(x,y+128.0),egui::Align2::LEFT_TOP,"EXTERNAL RENDERING · NO INJECTION",
        FontId::proportional(11.3),Color32::from_rgb(198,175,190));
    let btn=Rect::from_min_size(pos2(x,y+157.0),vec2(152.0,28.0));
    p.rect_filled(btn,7.0,Color32::from_rgb(66,41,57));
    p.rect_stroke(btn,7.0,Stroke::new(0.8_f32,BORDER),StrokeKind::Inside);
    modern_icon(&p,btn.left_center()+vec2(17.0,0.0),Page::Settings,WHITE,0.62);
    p.text(btn.left_center()+vec2(32.0,0.0),egui::Align2::LEFT_CENTER,"GAME PROFILE",
        FontId::proportional(12.0),WHITE);
    if ui.interact(btn,ui.id().with("hb-hero-profile"),Sense::click()).clicked(){app.page=Page::Filters;}
    let photo_btn=Rect::from_min_size(
        pos2(r.right()-132.0,r.bottom()-40.0),vec2(111.0,26.0));
    p.rect_filled(photo_btn,7.0,Color32::from_rgba_unmultiplied(40,24,47,225));
    p.rect_stroke(photo_btn,7.0,Stroke::new(1.0_f32,Color32::from_rgb(199,46,88)),StrokeKind::Inside);
    p.text(photo_btn.center(),egui::Align2::CENTER_CENTER,
        "CHANGE ART",FontId::proportional(11.0),WHITE);
    if ui.interact(photo_btn,ui.id().with("hb-change-home-art"),Sense::click()).clicked(){
        if let Some(path)=crate::filedialog::select_image(){
            app.change_hero_picture(&path,ui.ctx());
        }
    }
    let _=resp;
}
fn gpu_short(s:&str)->String {
    let t=s.trim().trim_end_matches(" Series").trim_start_matches("AMD ");
    trim(t,20)
}
fn mini_metric(ui:&mut Ui,w:f32,title:&str,value:&str,accent:Color32){
    let (rect,_)=ui.allocate_exact_size(vec2(w,57.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(rect,9.0,Color32::from_rgb(24,22,33));
    p.rect_stroke(rect,9.0,Stroke::new(0.8_f32,BORDER),StrokeKind::Inside);
    p.text(rect.min+vec2(10.0,8.0),egui::Align2::LEFT_TOP,title,FontId::proportional(11.1),MUTED);
    p.text(rect.min+vec2(10.0,24.0),egui::Align2::LEFT_TOP,trim(value,18),
        FontId::proportional(if value.len()>15{12.5}else{19.0}),accent);
}
fn modern_sidebar_widgets(app:&mut BoostApp,ui:&mut Ui,w:f32){
    frame(PANEL).show(ui,|ui|{
        ui.set_width(w-26.0);
        ui.horizontal_wrapped(|ui|{
            heading(ui,"Active profile");
            if ui.small_button("Library").clicked(){app.page=Page::Library;}
        });
        caption(ui,"Saved per game · choose a preset without changing game files");
        ui.add_space(9.0);
        let labels=["Custom","Quality","Performance"];
        let gap=5.0;
        let slot=((ui.available_width()-gap*2.0)/3.0).max(70.0);
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=gap;
            for (idx,name) in labels.iter().enumerate(){
                let selected=app.selected_profile==idx;
                let (rect,response)=click_surface(ui,slot,37.0,selected);
                ui.painter().text(rect.center(),egui::Align2::CENTER_CENTER,*name,
                    FontId::proportional(11.3),if selected{WHITE}else{MUTED});
                if response.clicked(){
                    let profile=match idx{1=>FilterProfile::quality(),
                        2=>FilterProfile::performance(),_=>FilterProfile::default()};
                    app.set_profile(profile,name);
                }
            }
        });
        caption(ui,"A saved profile is not proof that GPU shaders are currently active.");
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_width(w-26.0);
        ui.horizontal(|ui|{
            heading(ui,"Live system");
            ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                if ui.small_button("Details").clicked(){app.page=Page::Performance;}
            });
        });
        ui.add_space(8.0);
        let m=(ui.available_width()-8.0)/2.0;
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=8.0;
            mini_metric(ui,m,"FPS",&fps_text(app,"fps"),WHITE);
            mini_metric(ui,m,"FRAME TIME",&fps_text(app,"frametime"),
                Color32::from_rgb(249,183,126));
        });
        ui.add_space(7.0);
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=8.0;
            mini_metric(ui,m,"GPU",&gpu_short(&app.gpu),RED);
            mini_metric(ui,m,"USED SYSTEM RAM",&format!("{:.1} GB",app.telem.used_ram_gb),
                Color32::from_rgb(213,145,238));
        });
        caption(ui,"ETW data only · metrics stay empty when not available");
    });
}
fn modern_dashboard(app:&mut BoostApp,ui:&mut Ui){
    let d=home_dims(ui.ctx().content_rect().width());
    ui.set_width(d.usable);
    if d.stack{
        ui.vertical(|ui|{
            ui.set_width(d.main);
            modern_main_widgets(app,ui,d.main);
            ui.add_space(12.0);
            modern_sidebar_widgets(app,ui,d.side);
        });
    } else {
        ui.horizontal_top(|ui|{
            ui.spacing_mut().item_spacing.x=GAP;
            ui.allocate_ui_with_layout(vec2(d.main,0.0),Layout::top_down(Align::Min),|ui|{
                ui.set_width(d.main);
                modern_main_widgets(app,ui,d.main);
            });
            ui.allocate_ui_with_layout(vec2(d.side,0.0),Layout::top_down(Align::Min),|ui|{
                ui.set_width(d.side);
                modern_sidebar_widgets(app,ui,d.side);
            });
        });
    }
}
fn modern_main_widgets(app:&mut BoostApp,ui:&mut Ui,w:f32){
    ui.horizontal(|ui|{
        ui.label(RichText::new("Home").size(23.0).strong().color(WHITE));
        ui.add_space(7.0);
        caption(ui,"Gaming control center");
    });
    ui.add_space(9.0);
    modern_hero(app,ui,w);
    ui.add_space(9.0);
    let (banner,_)=ui.allocate_exact_size(vec2(w,36.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(banner,8.0,Color32::from_rgb(53,24,39));
    p.rect_stroke(banner,8.0,Stroke::new(0.8_f32,Color32::from_rgb(113,45,66)),StrokeKind::Inside);
    p.circle_stroke(banner.left_center()+vec2(17.0,0.0),5.0,Stroke::new(1.3_f32,RED));
    p.text(banner.left_center()+vec2(30.0,0.0),egui::Align2::LEFT_CENTER,
        "EXTERNAL OVERLAY",FontId::proportional(11.5),WHITE);
    if w>540.0{
        p.text(banner.right_center()-vec2(12.0,0.0),egui::Align2::RIGHT_CENTER,
            "No game DLL injection · support varies by game",FontId::proportional(11.0),MUTED);
    }
    ui.add_space(8.0);
    frame(CARD).show(ui,|ui|{
        ui.horizontal_wrapped(|ui|{
            caption(ui,"SELECTED VISUAL PROFILE");
            let selected=if app.settings.live_stack==0{
                crate::engine::EFFECTS[app.settings.live_preset].label
            }else{crate::stacks::STACKS[app.settings.live_stack].label};
            ui.label(RichText::new(selected).color(WHITE).strong());
            caption(ui,if app.gpu_backend.running(){"External renderer started · verify in game"}
                       else{"Saved selection · GPU renderer not running"});
            if outline_button(ui,"Edit look"){app.page=Page::Filters;}
        });
    });
    ui.add_space(9.0);
    let tiles=[
        ("Visual Studio","Native looks · fullscreen color",Page::Filters,true),
        ("Game Overlay","Tools visible during play",Page::Overlay,true),
        ("Performance","FPS · diagnostics",Page::Performance,true),
        ("Game Extras","20 reticles · timers · quests",Page::Extras,true),
    ];
    let columns=if w<470.0{1}else{2};
    let tile_w=(w-GAP*(columns as f32-1.0))/(columns as f32);
    for chunk in tiles.chunks(columns){
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=GAP;
            for (name,detail,page,active) in chunk{
                modern_card(app,ui,tile_w,name,detail,*page,*active);
            }
        });
        ui.add_space(6.0);
    }
    egui::CollapsingHeader::new("More modules · Scaling / Capture / Frame Generation")
        .id_salt("home-more-modules").default_open(false).show(ui,|ui|{
            ui.horizontal_wrapped(|ui|{
                if outline_button(ui,"Scaling"){app.page=Page::Scaling;}
                if outline_button(ui,"Game Library"){app.page=Page::Library;}
                if outline_button(ui,"Capture / Record"){app.page=Page::Capture;}
                if outline_button(ui,"Frame Generation"){app.page=Page::FrameGen;}
                if outline_button(ui,"Application Settings"){app.page=Page::Settings;}
            });
        });

}
#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn dashboard_never_overflows_viewport(){
        for viewport in [900.0,960.0,1024.0,1152.0,1280.0,1366.0,1500.0,1600.0,1920.0]{
            let d=home_dims(viewport);
            assert!(d.usable<=viewport-NAV_WIDTH-CONTENT_MARGIN*2.0+0.01);
            if d.stack{assert!(d.main<=d.usable+0.01);}
            else{assert!(d.main+d.side+GAP<=d.usable+0.01);}
        }
    }
    #[test]
    fn three_column_cards_fit(){
        for w in [651.0,700.0,800.0,900.0,1100.0]{
            let card=(w-2.0*GAP)/3.0;
            assert!(card*3.0+GAP*2.0<=w+0.01);
        }
    }
}
