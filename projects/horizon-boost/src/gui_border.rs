// Precisely rounded, inset animated outline. OS window clip is applied in
// win::round_horizon_window for opaque frameless windows (no black square corners).
fn animated_window_outline(app:&BoostApp,ctx:&egui::Context,key:&str){
    let r=ctx.content_rect().shrink(2.1);
    if r.width()<50.0||r.height()<50.0{return;}
    let radius=if key=="overlay"{8.0}else{11.0};
    let paint=ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,egui::Id::new(("hb-round-border",key))));
    paint.rect_stroke(r,radius,Stroke::new(1.1_f32,
        Color32::from_rgb(114,45,58)),StrokeKind::Inside);
    let focused=ctx.input(|i|i.viewport().focused.unwrap_or(false));
    if crate::suite_finish::gpu_pressure(app.vram_budget.as_ref().and_then(|v|v.as_ref().ok()).filter(|v|v.meaningful()).map(|v|v.fraction()),app.settings.pressure_quiet)||!app.settings.animations||(!focused&&!app.is_smoke&&(app.settings.quiet_ui||app.native_mirror.as_ref().is_some_and(|w|w.running()))){return;}
    let time=app.app_start.elapsed().as_secs_f32();
    let top=(r.width()-2.0*radius).max(1.0);
    let side=(r.height()-2.0*radius).max(1.0);
    let arc=std::f32::consts::FRAC_PI_2*radius;
    let perimeter=2.0*(top+side)+4.0*arc;
    let point=|distance:f32|->egui::Pos2{
        let mut x=distance.rem_euclid(perimeter);
        if x<top{return pos2(r.left()+radius+x,r.top());} x-=top;
        if x<arc{
            let a=-std::f32::consts::FRAC_PI_2+x/radius;
            return pos2(r.right()-radius+radius*a.cos(),r.top()+radius+radius*a.sin());
        }x-=arc;
        if x<side{return pos2(r.right(),r.top()+radius+x);}x-=side;
        if x<arc{
            let a=x/radius;
            return pos2(r.right()-radius+radius*a.cos(),r.bottom()-radius+radius*a.sin());
        }x-=arc;
        if x<top{return pos2(r.right()-radius-x,r.bottom());}x-=top;
        if x<arc{
            let a=std::f32::consts::FRAC_PI_2+x/radius;
            return pos2(r.left()+radius+radius*a.cos(),r.bottom()-radius+radius*a.sin());
        }x-=arc;
        if x<side{return pos2(r.left(),r.bottom()-radius-x);}x-=side;
        let a=std::f32::consts::PI+x/radius;
        pos2(r.left()+radius+radius*a.cos(),r.top()+radius+radius*a.sin())
    };
    let offset=(time*128.0)%perimeter;
    // Short, soft highlight instead of a cut-off diagonal in window corners.
    for j in 0..45{
        let t=j as f32/44.0;
        let light=(1.0-((t-0.5).abs()*2.0)).max(0.0);
        let a=Color32::from_rgba_unmultiplied(255,95,112,
            (22.0+178.0*light) as u8);
        paint.line_segment([point(offset+j as f32*2.8),
            point(offset+(j+1) as f32*2.8)],
            Stroke::new(1.0+1.6*light,a));
    }
}
