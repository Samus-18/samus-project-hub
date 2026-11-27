use crate::{BoostApp,Page,adlx,lab};
use crate::settings::FilterProfile;
use eframe::egui::{self,Align,Color32,CornerRadius,Frame,Layout,RichText,Stroke,StrokeKind,Ui,Vec2,ViewportBuilder,ViewportCommand,ViewportId};

const BG:Color32=Color32::from_rgb(10,12,17);
const PANEL:Color32=Color32::from_rgb(24,24,34);
const CARD:Color32=Color32::from_rgb(32,32,44);
const RED:Color32=Color32::from_rgb(240,48,63);
const MUTED:Color32=Color32::from_rgb(169,177,189);
const WHITE:Color32=Color32::from_rgb(246,247,250);
const BORDER:Color32=Color32::from_rgb(40,40,45);
pub fn visuals()->egui::Visuals {
    let mut v=egui::Visuals::dark();
    v.panel_fill=BG;v.window_fill=PANEL;v.extreme_bg_color=Color32::from_rgb(8,8,10);
    v.override_text_color=Some(WHITE);
    v.selection.bg_fill=RED;
    v.selection.stroke=Stroke::new(1.0_f32,WHITE);
    v.widgets.noninteractive.bg_fill=PANEL;
    v.widgets.inactive.bg_fill=CARD;
    v.widgets.inactive.weak_bg_fill=CARD;
    v.widgets.inactive.bg_stroke=Stroke::new(0.7_f32,BORDER);
    v.widgets.active.weak_bg_fill=Color32::from_rgb(98,40,56);
    v.widgets.hovered.bg_fill=Color32::from_rgb(45,24,31);
    v.widgets.hovered.weak_bg_fill=Color32::from_rgb(45,24,31);
    v.widgets.active.bg_fill=Color32::from_rgb(78,25,38);
    v.widgets.inactive.corner_radius=CornerRadius::same(8);
    v.widgets.hovered.corner_radius=CornerRadius::same(8);
    v.widgets.active.corner_radius=CornerRadius::same(8);
    v
}
fn frame(fill:Color32)->Frame {
    Frame::new().fill(fill).stroke(Stroke::new(1.0_f32,BORDER))
        .corner_radius(14).inner_margin(egui::Margin::same(14))
}
fn title(ui:&mut Ui,t:&str){
    ui.label(RichText::new(tr(ui,t)).size(28.0).strong().color(WHITE));
}
fn caption(ui:&mut Ui,t:&str){ui.label(RichText::new(tr(ui,t)).size(12.0).color(MUTED));}
fn text(ui:&mut Ui,t:&str){ui.label(RichText::new(tr(ui,t)).size(13.8).color(WHITE));}
fn heading(ui:&mut Ui,t:&str){ui.label(RichText::new(tr(ui,t)).size(16.0).strong().color(WHITE));}
fn fps_text(app:&BoostApp,metric:&str)->String{
    let Some(stats)=app.fps.snapshot() else{return "—".to_string()};
    match metric{
        "fps"=>format!("{:.0}",stats.fps),
        "frametime"=>format!("{:.1} ms",stats.frametime_ms),
        "low"=>stats.low_1_percent_fps.map(|v|format!("{v:.0}")).unwrap_or_else(||"—".into()),
        "gpu"=>stats.gpu_ms.map(|v|format!("{v:.1} ms")).unwrap_or_else(||"—".into()),
        "latency"=>stats.latency_ms.map(|v|format!("{v:.1} ms")).unwrap_or_else(||"—".into()),
        _=>"—".into()
    }
}
fn red_button(ui:&mut Ui,label:&str)->bool{
    let response=ui.add(egui::Button::new(RichText::new(tr(ui,label)).color(WHITE))
        .fill(Color32::from_rgb(122,32,60)).stroke(Stroke::new(1.0_f32,RED))
        .min_size(Vec2::new(96.0,31.0)));
    motion_button(ui,&response);response.clicked()
}
fn outline_button(ui:&mut Ui,label:&str)->bool{
    let response=ui.add(egui::Button::new(RichText::new(tr(ui,label)).color(WHITE))
        .fill(CARD).stroke(Stroke::new(1.0_f32,BORDER))
        .min_size(Vec2::new(88.0,29.0)));
    motion_button(ui,&response);response.clicked()
}
fn section(ui:&mut Ui,title_text:&str,subtitle:&str){
    ui.horizontal(|ui|{
        let (r,_)=ui.allocate_exact_size(Vec2::new(13.0,21.0),egui::Sense::hover());
        ui.painter().add(egui::Shape::convex_polygon(vec![
            r.left_bottom()+egui::vec2(0.0,-2.0),
            r.left_top()+egui::vec2(9.0,0.0),
            r.right_top()+egui::vec2(0.0,0.0),
            r.left_bottom()+egui::vec2(4.0,-2.0)
        ],RED,Stroke::NONE));
        title(ui,title_text);if !subtitle.is_empty(){ui.label(RichText::new("(i)").size(11.).color(MUTED)).on_hover_text(subtitle);}
    });
    ui.add_space(13.0);
}
pub fn render(app:&mut BoostApp,ctx:&egui::Context){
    assist_tick(app,ctx);
    highlight_tick(app,ctx);
    release_idle_media_cache(app,ctx);
    reference_viewport(app,ctx);
    let motion=app.settings.animations&&!crate::suite_finish::gpu_pressure(app.vram_budget.as_ref().and_then(|v|v.as_ref().ok()).filter(|v|v.meaningful()).map(|v|v.fraction()),app.settings.pressure_quiet)&&(app.is_smoke||ctx.input(|i|i.viewport().focused.unwrap_or(false)));
    ctx.style_mut(|style|{style.animation_time=if motion{0.16}else{0.};});
    ctx.data_mut(|d|{d.insert_temp(egui::Id::new("hb-czech"),app.settings.ui_language==1);d.insert_temp(egui::Id::new("hb-motion-ui"),motion);});
    let theme=app.settings.ui_theme;
    let current=ctx.data(|d|d.get_temp::<u8>(egui::Id::new("hb-theme-current")));
    if current!=Some(theme){
        let mut v=vf_visuals();
        if theme==1{v.panel_fill=Color32::from_rgb(3,3,5);v.window_fill=Color32::from_rgb(12,12,16);v.widgets.inactive.bg_fill=Color32::from_rgb(19,19,24);v.widgets.inactive.weak_bg_fill=v.widgets.inactive.bg_fill;}
        if theme==2{v.panel_fill=Color32::from_rgb(8,15,24);v.window_fill=Color32::from_rgb(18,30,43);v.widgets.inactive.bg_fill=Color32::from_rgb(26,43,61);v.widgets.inactive.weak_bg_fill=v.widgets.inactive.bg_fill;v.selection.bg_fill=Color32::from_rgb(45,96,140);}
        ctx.set_visuals(v);ctx.data_mut(|d|d.insert_temp(egui::Id::new("hb-theme-current"),theme));
    }
    if app.is_smoke&&app.visual_os_color_lab{
        egui::CentralPanel::default().frame(Frame::new().fill(BG)
            .inner_margin(egui::Margin::symmetric(28,26))).show(ctx,|ui|{
            heading(ui,"HORIZON BOOST · Windows native display color");
            caption(ui,"Hidden visual QA only: buttons are not clicked and the actual desktop colors remain unchanged.");
            ui.add_space(12.0);
            native_os_color_lab(app,ui);
        });
        animated_window_outline(app,ctx,"color-lab-proof");
        return;
    }
    // Hidden screenshot proof: paint the same overlay widget tree in a hidden
    // primary viewport so screenshots work without briefly showing a game HUD.
    if app.is_smoke&&std::env::args().any(|a|a=="--visual-macro-overlay"){
      egui::CentralPanel::default().frame(Frame::new().fill(Color32::from_rgb(13,19,26)).inner_margin(egui::Margin::same(2))).show(ctx,|ui|vf_macro_badge(app,ui));
      return;
    }
    if app.is_smoke&&app.visual_overlay{
        egui::CentralPanel::default().frame(Frame::new()
            .fill(Color32::from_rgb(14,15,22))
            .inner_margin(if app.visual_compact_overlay{
                egui::Margin::symmetric(8,5)
            }else{egui::Margin::symmetric(12,9)})).show(ctx,|ui|{
                if app.visual_compact_overlay{
                    v26_compact_overlay(app,ui);
                }else{
                    egui::ScrollArea::vertical().auto_shrink([false,false])
                        .show(ui,|ui|v26_overlay_editor(app,ctx,ui));
                }
            });
        animated_window_outline(app,ctx,"overlay-proof");
        return;
    }
    // Try to clip only our own native frameless window corners.
    // Never touches the game window and avoids the previous square black tips.
    if !app.is_smoke{let _=crate::win::round_horizon_window(false);}
    egui::TopBottomPanel::top("titlebar").exact_height(52.0).frame(
        Frame::new().fill(Color32::from_rgb(8,8,10)).inner_margin(egui::Margin::symmetric(14,8))
    ).show(ctx,|ui|vf_header(app,ctx,ui));

    // Narrow laptop windows / 135% accessible zoom use a scrollable
    // top navigation. No clipped icons, no page loss and no extra OS window.
    let narrow=ctx.content_rect().width()<860.;
    if narrow{
        egui::TopBottomPanel::top("hb-compact-nav")
        .exact_height(44.)
        .frame(Frame::new().fill(Color32::from_rgb(6,6,9))
            .inner_margin(egui::Margin::symmetric(9,5)))
        .show(ctx,|ui|vf_compact(app,ui));
    }else{
        egui::SidePanel::left("nav").exact_width(NAV_WIDTH).resizable(false)
        .frame(Frame::new().fill(Color32::from_rgb(11,11,14)).inner_margin(egui::Margin::symmetric(9,10)))
        .show(ctx,|ui|{egui::ScrollArea::vertical().id_salt("nav-scroll").auto_shrink([false,false]).show(ui,|ui|vf_navigation(app,ui));});
    }

    command_palette(app,ctx);
    if app.sequence_window.open{
        let mut close=false;
        let phase=app.sequence_window.player.tick(&app.sequence_window.plan,std::time::Instant::now());
        ctx.show_viewport_immediate(ViewportId::from_hash_of("hb-local-seq-v61"),ViewportBuilder::default().with_title("Horizon Boost - Isolated sequence").with_inner_size([430.,260.]).with_resizable(true),|child,_|{
            if child.input(|i|i.viewport().close_requested()||i.key_pressed(egui::Key::Escape)){close=true;}
            egui::CentralPanel::default().show(child,|ui|{
                heading(ui,"ISOLATED HOLD / GAP PREVIEW");
                caption(ui,"No keyboard events are sent. Esc closes this test.");
                if let Some((i,held,left))=phase{ui.colored_label(if held{V25_MINT}else{MUTED},RichText::new(format!("{} / {}",app.sequence_window.plan.steps[i].key,if held{"HOLD"}else{"GAP"})).size(30.));ui.label(format!("{left} ms"));child.request_repaint_after(std::time::Duration::from_millis(16));}else{ui.label("Sequence complete");}
                if red_button(ui,"Close / cancel"){close=true;}
            });
        });
        if close{app.sequence_window.close();}
    }
    if app.accuracy_window&&!app.is_smoke{
        ctx.show_viewport_immediate(ViewportId::from_hash_of("hb-accuracy-v54"),ViewportBuilder::default().with_title("Horizon Boost - Local practice").with_inner_size([620.,420.]).with_resizable(true),|child,_|{
            if child.input(|i|i.viewport().close_requested()){app.accuracy_window=false;app.accuracy_trainer.stop();}
            egui::CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(egui::Margin::same(14))).show(child,|ui|{accuracy_panel(app,ui);});
        });
    }
    egui::TopBottomPanel::bottom("status").exact_height(25.0)
        .frame(Frame::new().fill(Color32::from_rgb(8,8,10)).inner_margin(egui::Margin::symmetric(15,4)))
        .show(ctx,|ui|{
            ui.horizontal(|ui|{
                ui.colored_label(RED,"FX");
                caption(ui,&app.status);if app.recorder.elapsed().is_some(){ui.colored_label(RED,"REC");}else if app.replay.active(){ui.label("REPLAY");}
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{caption(ui,"VANTA FX · V139 DEV  /  D3D11");});
            });
        });

    egui::CentralPanel::default().frame(Frame::new().fill(BG).inner_margin(
        if ctx.content_rect().width()<860.{egui::Margin::symmetric(10,10)}
        else{egui::Margin::symmetric(20,14)}))
        .show(ctx,|ui|{
            // V138: flat low-noise surface; no fullscreen purple gradients.
            // Each page/sub-view owns its scroll position. Changing from a long
            // shader gallery to the compact Color Lab must never leave the
            // newly selected tab apparently blank at an old scroll offset.
            let scroll_key=match app.page{
                Page::Filters=>format!("filters-{}",app.filter_view),
                Page::Performance=>format!("performance-{}",app.performance_view),
                Page::Extras=>format!("extras-{}",app.extras_view),
                other=>format!("page-{other:?}"),
            };
            egui::ScrollArea::vertical().id_salt(scroll_key)
                .auto_shrink([false,false]).show(ui,|ui|{
                let content_width=ui.available_width().max(280.0);
                ui.set_width(content_width);
                ui.set_max_width(content_width);
                // No animated page-wide decoration.
                vf_content(app,ui);

            });
        });
    if app.overlay_open{
        // The HUD is independent of selected-game focus; never minimize its parent automatically.
        let screen=crate::win::screen_size();
        let (width,height,x,y)=v26_hud_geometry(app,screen);
        let editor_at_start=app.overlay_edit_mode;
        let changed=app.last_overlay_passive!=Some(!editor_at_start);
        ctx.show_viewport_immediate(ViewportId::from_hash_of("vanta-overlay"),
            ViewportBuilder::default().with_title(crate::win::HUD_TITLE)
                .with_inner_size([width,height])
                .with_position([x,y])
                .with_always_on_top()
                .with_active(editor_at_start)
                .with_taskbar(false)
                .with_mouse_passthrough(!editor_at_start)
                // The viewport must exist before native Win32 click-through styles
                // can be configured. MousePassThrough is enabled at creation;
                // fail closed in the interaction check below if OS refuses.
                .with_visible(!app.is_smoke)
                .with_decorations(false)
                .with_resizable(false),
            |ov_ctx,_class|{
                if ov_ctx.input(|i|i.viewport().close_requested()){
                    app.overlay_open=false;
                    return;
                }
                egui::CentralPanel::default().frame(Frame::new()
                    .fill(Color32::from_rgb(13,14,22))
                    .stroke(Stroke::new(0.9_f32,Color32::from_rgb(115,55,85)))
                    .inner_margin(if app.overlay_edit_mode{
                        egui::Margin::symmetric(12,9)
                    }else{
                        egui::Margin::symmetric(8,5)
                    }))
                    .show(ov_ctx,|ui|{
                        if app.overlay_edit_mode{
                            egui::ScrollArea::vertical().auto_shrink([false,false])
                                .show(ui,|ui|v26_overlay_editor(app,ov_ctx,ui));
                        }else{
                            v26_compact_overlay(app,ui);
                        }
                    });
                if !app.is_smoke{
                    let (nw,nh,nx,ny)=v26_hud_geometry(app,screen);
                    let observed=ov_ctx.input(|i|i.viewport().inner_rect);
                    let resize=observed.is_none_or(|r|
                        (nw-r.width()).abs()>4.0||(nh-r.height()).abs()>4.0);
                    if resize{
                        ov_ctx.send_viewport_cmd(ViewportCommand::InnerSize(
                            egui::vec2(nw,nh)));
                    }
                    let moved=ov_ctx.input(|i|i.viewport().outer_rect).is_none_or(|r|
                        (r.min.x-nx).abs()>4.0||(r.min.y-ny).abs()>4.0);
                    if editor_at_start&&!changed&&moved {
                        if let Some(outer)=ov_ctx.input(|i|i.viewport().outer_rect){
                            let target=app.settings.hud_layout.display.as_deref().and_then(|name|
                                crate::display_probe::select(&crate::display_probe::read(),Some(name)));
                            let desktop=target.as_ref().map_or(screen,|d|(d.width,d.height));
                            let origin=target.as_ref().map_or((0f32,0f32),|d|(d.x as f32,d.y as f32));
                            let pos=(outer.min.x-origin.0,outer.min.y-origin.1);
                            if let Some(anchor)=crate::hud_layout::anchor_from_outer(desktop,
                                (outer.width(),outer.height()),pos){
                                if app.settings.hud_layout.custom_position.is_none_or(|p|
                                    (p[0]-anchor[0]).abs()>0.002||(p[1]-anchor[1]).abs()>0.002){
                                    app.settings.hud_layout.custom_position=Some(anchor);
                                    app.save();
                                }
                            }
                        }
                    }
                    if !editor_at_start&&app.settings.hud_layout.display.is_some(){
                        if app.hud_monitor_cache.as_ref().is_none_or(|(at,_)|at.elapsed()>=std::time::Duration::from_secs(2)){app.hud_monitor_cache=Some((std::time::Instant::now(),crate::display_probe::read()));}
                        if let Some((_,rows))=app.hud_monitor_cache.as_ref(){
                            if let Some(target)=crate::display_probe::select(rows,app.settings.hud_layout.display.as_deref()){
                                let corner=app.settings.hud_corner;
                                let anchor=app.settings.hud_layout.custom_position.unwrap_or([if corner==0||corner==2{0.}else{1.},if corner>=2{1.}else{0.}]);
                                app.hud_observed_dpi=crate::win::place_own_hud(&target,anchor).ok().flatten();
                            }
                        }
                    }else if changed||(!editor_at_start&&moved){
                        app.hud_observed_dpi=None;
                        ov_ctx.send_viewport_cmd(ViewportCommand::OuterPosition(egui::pos2(nx,ny)));
                    }
                }
                if !app.is_smoke{let _=crate::win::round_horizon_window(true);ov_ctx.request_repaint_after(std::time::Duration::from_millis(100));}
                animated_window_outline(app,ov_ctx,"overlay");
                if app.visual_overlay{
                    let screenshot=ov_ctx.input(|i|i.events.iter().find_map(|event|{
                        if let egui::Event::Screenshot{image,..}=event{Some(image.clone())}else{None}
                    }));
                    if let Some(img)=screenshot{
                        let bytes=img.pixels.iter().flat_map(|c|c.to_array()).collect::<Vec<u8>>();
                        let output=std::env::current_dir().unwrap_or_default()
                            .join("build-tests").join(&app.overlay_proof_name);
                        let _=std::fs::create_dir_all(output.parent().unwrap());
                        let saved=image::save_buffer(&output,&bytes,img.size[0] as u32,
                            img.size[1] as u32,image::ColorType::Rgba8).is_ok();
                        eprintln!("OVERLAY_VISUAL_PROOF_SAVED={saved} PATH={}",output.display());
                        app.visual_saved=saved;
                    }
                    if !app.visual_requested{
                        ov_ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default()));
                        app.visual_requested=true;
                    }
                }
            });
        // Style changes are applied ONLY to our own overlay HWND.
        // Fail closed if OS refuses mouse pass-through, so games remain
        // interactable even if this user's Windows build is unusual.
        let passive=!app.overlay_edit_mode;
        if app.last_overlay_passive!=Some(passive)||
            app.last_overlay_opacity!=Some(app.settings.overlay_opacity)||
            (!app.is_smoke&&passive&&app.last_hud_reassert.is_none_or(|t|t.elapsed()>=std::time::Duration::from_millis(900))){
            match crate::win::configure_overlay_interaction(
                passive,app.settings.overlay_opacity){
                Ok(())=>{
                    app.last_overlay_passive=Some(passive);
                    app.last_hud_reassert=Some(std::time::Instant::now());
                    if !app.is_smoke&&passive&&!crate::win::show_passive_hud(){app.overlay_open=false;app.status="HUD hidden: could not safely show click-through window".into();}
                    app.last_overlay_opacity=Some(app.settings.overlay_opacity);
                    if app.is_smoke{eprintln!("HUD_INPUT_MODE={} STATUS=PASS",
                        if passive{"PASSTHROUGH"}else{"INTERACTIVE"});}
                },
                Err(error)=>{
                    if error.contains("does not exist yet"){
                        // The viewport is not created until the first frame.
                    }else{
                        app.overlay_open=false;
                        app.status=format!("HUD hidden for game mouse safety: {error}");
                    }
                }
            }
        }
    }else{
        app.last_overlay_opacity=None;
        app.last_overlay_passive=None;
        app.last_hud_reassert=None;
    }
    vf_macro_overlay(app,ctx);
    animated_window_outline(app,ctx,"main");
}
fn trim(s:&str,n:usize)->String{if s.chars().count()>n{format!("{}…",s.chars().take(n).collect::<String>())}else{s.to_owned()}}
include!("gui_extra.rs");
include!("gui_gpu.rs");
include!("gui_design.rs");
include!("gui_overlay.rs");
include!("gui_overlay_panels.rs");
include!("gui_gamehud.rs");
include!("gui_tools.rs");
include!("gui_border.rs");
include!("gui_v22.rs");
include!("gui_v25.rs");
include!("gui_v26.rs");
include!("gui_driver.rs");
include!("gui_gallery.rs");
include!("gui_stacks.rs");
include!("gui_native.rs");
include!("gui_gameplay.rs");
include!("gui_lab.rs");
include!("gui_extras.rs");
include!("gui_perf_insights.rs");
include!("gui_training.rs");
include!("gui_v33.rs");
include!("gui_v34.rs");
include!("gui_v35.rs");
include!("gui_v36.rs");
include!("gui_v38.rs");
include!("gui_v39.rs");
include!("gui_v40.rs");
include!("gui_v43.rs");
include!("gui_v45.rs");
include!("gui_v46.rs");
include!("gui_v49.rs");
include!("gui_oled.rs");
include!("gui_oled_workspaces.rs");
include!("gui_eclipse.rs");
include!("gui_motion.rs");
include!("gui_optimizer.rs");
include!("gui_suite.rs");
include!("gui_remake.rs");
include!("gui_remake_help.rs");
include!("gui_remake_more.rs");
include!("gui_remake_studio.rs");
include!("gui_game_journal.rs");
include!("gui_finish.rs");
include!("gui_unsorted.rs");

include!("gui_reborn.rs");
