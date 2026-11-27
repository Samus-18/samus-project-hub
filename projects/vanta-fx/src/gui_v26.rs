// Horizon Boost v2.6 — Passive HUD never covers the screen with controls.
// Compact mode is small and mouse-pass-through. Editor mode is opt-in.
const V26_MINI_WIDTH:f32=252.0;
const V26_MINI_HEIGHT:f32=70.0;
const V26_EDIT_WIDTH:f32=456.0;
// Logical sizes scale in the UI itself, not only in the viewport bounds.
fn v26_hud_geometry(app:&BoostApp,screen:(u32,u32))->(f32,f32,f32,f32){
    let z=app.settings.overlay_size_scale.clamp(0.78,1.35);
    let (width,height)=if app.overlay_edit_mode{
        let w=(V26_EDIT_WIDTH*z).clamp(398.0,(screen.0 as f32-26.0).max(340.0));
        // Keep the editor under 65% of desktop height with vertical scrolling;
        // opt-in edit mode must never blanket the user's game.
        let max_height=(screen.1 as f32*0.65).min(720.0);
        let h=((v25_overlay_height(app)+30.0)*z)
            .clamp(320.0,max_height.max(320.0));
        (w,h)
    }else if app.settings.hud_layout.enabled{
        app.settings.hud_layout.dimensions(z)
    }else{
        ((V26_MINI_WIDTH*z).min((screen.0 as f32-20.0).max(160.0)),
         V26_MINI_HEIGHT*z)
    };
    // Guarantee every own HUD viewport remains usable on small or rotated
    // displays, including non-integer UI scale. No game windows are resized.
    let (width,height)=v103_clamp_hud_size((width,height),screen);
    let corner=if app.overlay_edit_mode{1}else{app.settings.hud_corner};
    let x=if corner==0||corner==2{15.0}else{(screen.0 as f32-width-15.0).max(4.0)};
    let y=if corner>=2{(screen.1 as f32-height-15.0).max(4.0)}else{15.0};
    let (x,y)=app.settings.hud_layout.position(screen,(width,height)).unwrap_or((x,y));
    let max_x=(screen.0 as f32-width-4.).max(4.);
    let max_y=(screen.1 as f32-height-4.).max(4.);
    (width,height,x.clamp(4.,max_x),y.clamp(4.,max_y))
}
fn v26_compact_overlay(app:&BoostApp,ui:&mut Ui){
 if app.settings.hud_layout.enabled{widget_hud(app,ui);return;}
 let z=app.settings.overlay_size_scale.clamp(0.78,1.35);
 let width=ui.available_width().max(146.);
 let (r,_)=ui.allocate_exact_size(egui::vec2(width,55.*z),egui::Sense::hover());
 let p=ui.painter().with_clip_rect(r);
 p.rect_filled(r,8.*z,Color32::from_rgb(9,13,17));
 let tl=|x:f32,y:f32|r.min+egui::vec2(x*z,y*z);
 let rt=|x:f32,y:f32|r.right_top()+egui::vec2(x*z,y*z);
 let v=current_fg_readout(app,true);
 let mint=Color32::from_rgb(90,220,183);
 p.text(tl(9.,3.),egui::Align2::LEFT_TOP,"GAME FPS",FontId::proportional(9.*z),MUTED);
 p.text(tl(9.,13.),egui::Align2::LEFT_TOP,&v.game,FontId::proportional(26.*z),mint);
 p.text(tl(9.,43.),egui::Align2::LEFT_TOP,format!("FT {}",hud_stats_text(app,"frametime")),FontId::monospace(9.*z),MUTED);
 p.line_segment([tl(92.,8.),tl(92.,48.)],Stroke::new(1.,Color32::from_rgb(38,48,55)));
 p.text(tl(103.,5.),egui::Align2::LEFT_TOP,if app.settings.hud_show_fg{"OUTPUT FPS"}else{"GPU TIME"},FontId::proportional(9.*z),MUTED);
 let secondary=if app.settings.hud_show_fg{if v.state=="FG off"{"OFF".into()}else{v.output.clone()}}else{hud_stats_text(app,"gpu")};
 p.text(tl(103.,17.),egui::Align2::LEFT_TOP,secondary,FontId::proportional(20.*z),WHITE);
 p.text(tl(103.,43.),egui::Align2::LEFT_TOP,if app.settings.hud_show_fg{v.state.as_str()}else{"LIVE"},FontId::monospace(9.*z),MUTED);
 p.text(rt(-6.,43.),egui::Align2::RIGHT_TOP,"Alt+Z",FontId::monospace(9.*z),MUTED);
}
fn v26_overlay_editor(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let width=ui.available_width().max(310.0);
    ui.horizontal(|ui|{
        ui.label(RichText::new("CONTROL MODE").strong().size(10.3)
            .color(V25_MINT));
        ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
            if ui.add(egui::Button::new(
                RichText::new("MINIMIZE TO GAME").strong().size(11.0))
                .fill(Color32::from_rgb(68,31,55))
                .stroke(Stroke::new(0.9_f32,V25_ACCENT))
                .min_size(egui::vec2(152.0,29.0))).clicked(){
                app.overlay_edit_mode=false;
                app.overlay_expanded=false;
            }
            if ui.small_button("+").on_hover_text("Make HUD and editor larger").clicked(){
                app.settings.overlay_size_scale=(app.settings.overlay_size_scale+0.08).min(1.35);
                app.save();
            }
            if ui.small_button("−").on_hover_text("Make HUD and editor smaller").clicked(){
                app.settings.overlay_size_scale=(app.settings.overlay_size_scale-0.08).max(0.78);
                app.save();
            }
        });
    });
    ui.add_space(6.0);
    ui.set_width(width);
    v25_overlay(app,ctx,ui);
}
#[cfg(test)]
mod v26_hud_tests {
    use super::*;
    #[test]
    fn compact_hud_is_252_by_70_not_a_screen_overlay(){
        assert_eq!(V26_MINI_WIDTH,252.0);
        assert_eq!(V26_MINI_HEIGHT,70.0);
        assert!(V26_MINI_WIDTH*V26_MINI_HEIGHT < 20000.0);
    }
    #[test]
    fn viewport_scaling_is_separate_and_never_fullscreen(){
        let _ = (V26_EDIT_WIDTH, V26_MINI_WIDTH, V26_MINI_HEIGHT);
        let user_default=crate::settings::Settings::default();
        assert!((V26_MINI_WIDTH*user_default.overlay_size_scale)<252.0);
        assert!((V26_MINI_HEIGHT*user_default.overlay_size_scale)<70.0);
        assert!((V26_EDIT_WIDTH*1.35)<640.0);
    }
}

fn hud_stats_text(app:&BoostApp,metric:&str)->String{
 let Some(s)=app.hud_snapshot()else{return "--".into()};
 match metric{"fps"=>format!("{:.0}",s.fps),"frametime"=>format!("{:.1} ms",s.frametime_ms),"low"=>s.low_1_percent_fps.map(|v|format!("{v:.0}")).unwrap_or("--".into()),"gpu"=>s.gpu_ms.map(|v|format!("{v:.1} ms")).unwrap_or("--".into()),_=>"--".into()}
}
fn current_fg_readout(app:&BoostApp,hud:bool)->crate::hud_readout::Readout{
 let game=if hud{app.hud_snapshot()}else{app.fps.snapshot_for(app.native_mirror_pid.or_else(||app.telem.active_game.as_ref().map(|g|g.pid)))}.map(|s|f64::from(s.fps));
 let worker=app.native_mirror.as_ref().filter(|w|w.running());
 let same=!hud||app.hud_output_matches();
 let worker=worker.filter(|_|same);
 let requested=worker.map(|w|w.multiplier());
 let effective=worker.and_then(|w|{let d=w.diagnostics();(!d.paused&&!d.own_minimized&&d.fg_effective>0).then_some(d.fg_effective.min(3) as u8)});
 let rates=worker.and_then(|w|{let d=w.diagnostics();if d.paused||d.own_minimized{None}else{app.output_rates.snapshot(std::time::Instant::now())}});
 crate::hud_readout::readout(game,rates,requested,effective)
}
fn fg_readout_panel(app:&BoostApp,ui:&mut Ui){
 let v=current_fg_readout(app,false);
 frame(CARD).show(ui,|ui|{
  ui.horizontal_wrapped(|ui|{ui.label(RichText::new(format!("{} FPS",v.game)).size(23.).strong().color(WHITE));caption(ui,"GAME");ui.label(RichText::new(">").size(18.).color(RED));ui.label(RichText::new(format!("{} FPS",v.output)).size(23.).strong().color(V25_MINT));caption(ui,"OUTPUT");});
  ui.horizontal_wrapped(|ui|{ui.colored_label(RED,&v.state);caption(ui,&v.detail);});
  caption(ui,"Game = ETW source FPS now. Output = real + generated successful own presents; not a measured FG-off baseline or display scanout.");
 });
}
fn hud_monitor_options(app:&mut BoostApp,ui:&mut Ui){
 let old=(app.settings.hud_follow_foreground,app.settings.hud_f12,app.settings.hud_start_open,app.settings.hud_show_fg);
 ui.checkbox(&mut app.settings.hud_follow_foreground,"Follow foreground apps / games (ETW only)");
 ui.checkbox(&mut app.settings.hud_f12,"F12 toggles HUD (disabled during macros)");
 ui.checkbox(&mut app.settings.hud_start_open,"Open passive HUD on launch");
 ui.checkbox(&mut app.settings.hud_show_fg,"Show game FPS and FrameGen output");
 if old!=(app.settings.hud_follow_foreground,app.settings.hud_f12,app.settings.hud_start_open,app.settings.hud_show_fg){app.hud_track_at=None;app.save();}
 ui.horizontal_wrapped(|ui|{caption(ui,"Target");ui.label(app.hud_target.as_ref().map(|v|v.1.as_str()).unwrap_or("Waiting for window"));});
 if app.overlay_open&&app.hud_snapshot().is_none(){caption(ui,&app.hud_meter().note);}
 caption(ui,"Always-on-top, no mouse capture. Game/preview FPS are measured independently. F12 remains macro stop while a macro is active.");
}

/// Pure, display-bounded viewport geometry. Values are logical egui pixels.
fn v103_clamp_hud_size(size:(f32,f32),screen:(u32,u32))->(f32,f32){
 let max_w=(screen.0 as f32-8.).max(80.);
 let max_h=(screen.1 as f32-8.).max(60.);
 let w=if size.0.is_finite(){size.0.clamp(80.,max_w)}else{80.};
 let h=if size.1.is_finite(){size.1.clamp(60.,max_h)}else{60.};
 (w,h)
}
#[cfg(test)]mod v103_hud_scale_tests{
 use super::*;
 #[test]fn hud_respects_short_and_portrait_displays(){
  assert_eq!(v103_clamp_hud_size((1000.,750.),(800,600)),(792.,592.));
  assert_eq!(v103_clamp_hud_size((400.,1300.),(600,800)),(400.,792.));
  assert_eq!(v103_clamp_hud_size((220.,80.),(3840,2160)),(220.,80.));
  assert_eq!(v103_clamp_hud_size((f32::NAN,0.),(1920,1080)),(80.,60.));
 }
}
