// Compact secondary viewport: 100% vector buttons; no icon font dependency.
// Stays external/topmost. No injection or anti-cheat bypass.
fn tile_selected(title:&str,page:Page,section:Page,expanded:bool,
    boost:bool,timer:bool,crosshair:bool)->bool{
    match title{
        "Game Boost"=>boost,
        "Timer"=>timer,
        "Crosshair"=>crosshair,
        "Performance"=>expanded&&section==Page::Performance,
        _=>expanded&&section==page
    }
}
fn overlay_tile(app:&mut BoostApp,ui:&mut Ui,w:f32,title:&str,page:Page){
    let active=tile_selected(title,page,app.overlay_section,app.overlay_expanded,
        app.game_boost.active(),app.game_timer.running(),app.settings.crosshair.enabled);
    let (r,resp)=click_surface(ui,w,42.0,active);
    modern_icon(ui.painter(),r.left_center()+vec2(19.0,0.0),page,RED,0.72);
    ui.painter().text(r.left_center()+vec2(39.0,0.0),egui::Align2::LEFT_CENTER,
        title,FontId::proportional(12.4),WHITE);
    if resp.clicked(){
        if title=="Game Boost"{
            app.toggle_game_boost();
        }else if title=="Unavailable Latency Mode"{
            app.overlay_notice="Driver-level low latency is not yet available. Real latency metrics are in Performance.".into();
        }else if title=="HDR"{
            app.overlay_notice="HDR display postprocessing is not implemented. No false ON switch.".into();
        }else{
            app.overlay_section=page;
            app.overlay_expanded=true;
            app.overlay_filter_tab=false;
            // Crucial: never mutate main app.page nor close the overlay from an overlay tile.
        }
    }
}
fn overlay_header(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let (r,_) = ui.allocate_exact_size(vec2(ui.available_width(),43.0),Sense::hover());
    let p=ui.painter();
    let x=r.left()+10.0;let y=r.center().y;
    p.add(egui::Shape::convex_polygon(vec![pos2(x,y+9.0),pos2(x+15.0,y-10.0),
        pos2(x+30.0,y+9.0)],RED,Stroke::NONE));
    p.text(pos2(x+40.0,y),egui::Align2::LEFT_CENTER,"HORIZON",FontId::proportional(19.0),WHITE);
    p.text(pos2(x+139.0,y),egui::Align2::LEFT_CENTER,"BOOST",FontId::proportional(19.0),RED);
    let active_rect=Rect::from_min_size(pos2(x+207.0,y-12.0),vec2(101.0,24.0));
    p.rect_filled(active_rect,6.0,Color32::from_rgb(17,56,37));
    p.circle_filled(active_rect.left_center()+vec2(12.0,0.0),3.1,Color32::from_rgb(90,222,143));
    p.text(active_rect.left_center()+vec2(22.0,0.0),egui::Align2::LEFT_CENTER,
        "OVERLAY ON",FontId::proportional(10.5),Color32::from_rgb(185,247,198));
    let close=Rect::from_center_size(pos2(r.right()-22.0,y),vec2(32.0,30.0));
    let resp=ui.interact(close,ui.id().with("overlay-close"),Sense::click());
    p.rect_filled(close,7.0,if resp.hovered(){Color32::from_rgb(133,29,55)}
        else{Color32::from_rgb(46,34,50)});
    draw_line(p,close.center()+vec2(-4.0,-4.0),close.center()+vec2(4.0,4.0),WHITE,1.5);
    draw_line(p,close.center()+vec2(-4.0,4.0),close.center()+vec2(4.0,-4.0),WHITE,1.5);
    if resp.clicked(){app.overlay_open=false;}
    let drag=Rect::from_min_max(r.min,pos2(close.left()-8.0,r.bottom()));
    if ui.interact(drag,ui.id().with("overlay-drag"),Sense::click_and_drag()).dragged(){
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}
fn modern_overlay_content(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    let width=ui.available_width().max(320.0);
    ui.set_width(width);
    overlay_header(app,ctx,ui);
    ui.add_space(7.0);
    let thirds=(width-14.0)/3.0;
    ui.horizontal(|ui|{
        ui.spacing_mut().item_spacing.x=7.0;
        mini_metric(ui,thirds,"FPS",&fps_text(app,"fps"),Color32::from_rgb(255,246,248));
        mini_metric(ui,thirds,"FRAME TIME",&fps_text(app,"frametime"),Color32::from_rgb(243,174,105));
        mini_metric(ui,thirds,"1% LOW",&fps_text(app,"low"),Color32::from_rgb(217,132,234));
    });
    ui.add_space(6.0);
    // The overlay's top row is for FPS only. Hardware detail is one compact
    // strip instead of four more heavy metric cards.
    frame(CARD).show(ui,|ui|{
        ui.horizontal_wrapped(|ui|{
            caption(ui,&format!("GPU  {}",gpu_short(&app.gpu)));
            ui.add_space(12.0);
            let cpu=if app.telem.active_game.is_some()&&app.telem.cpu_percent.is_finite(){
                format!("{:.2} cores",app.telem.cpu_percent/100.0)
            }else{"—".into()};
            caption(ui,&format!("CPU  {cpu}"));
            ui.add_space(12.0);
            caption(ui,&format!("RAM  {:.1} GB",app.telem.used_ram_gb));
            ui.add_space(12.0);
            caption(ui,&format!("Latency  {}",fps_text(app,"latency")));
        });
    });
    ui.add_space(9.0);
    ui.label(RichText::new("QUICK GAME TOOLS").size(10.0).strong()
        .color(Color32::from_rgb(216,148,172)));
    ui.add_space(5.0);
    let tiles=[
        ("Filters",Page::Filters),("Scaling",Page::Scaling),
        ("Performance",Page::Performance),("Crosshair",Page::Extras),
        ("Game Boost",Page::Performance),("Timer",Page::Performance)
    ];
    let tile_w=(width-14.0)/3.0;
    for chunk in tiles.chunks(3){
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=7.0;
            for (label,page) in chunk{overlay_tile(app,ui,tile_w,label,*page);}
        });
        ui.add_space(4.0);
    }
    egui::CollapsingHeader::new("More tools · Frame Gen / Capture / Library")
        .id_salt("overlay-extra-modules").default_open(false).show(ui,|ui|{
            let more=[
                ("Frame Gen",Page::FrameGen),
                ("Capture",Page::Capture),
                ("Library",Page::Library)
            ];
            ui.horizontal(|ui|{
                ui.spacing_mut().item_spacing.x=7.0;
                for (label,page) in more{overlay_tile(app,ui,tile_w,label,page);}
            });
        });
    ui.add_space(7.0);
    if app.overlay_expanded {
        overlay_open_panel(app,ui);
    }else if app.overlay_filter_tab {
        frame(PANEL).show(ui,|ui|{
            ui.set_width(width-26.0);
            ui.horizontal(|ui|{
                heading(ui,"GPU FILTERS");
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    if ui.small_button("Close editor").clicked(){app.overlay_filter_tab=false;}
                });
            });
            ui.add_space(4.0);
            select_gpu_options(app,ui);
            caption(ui,"Combined presets are sequential real GPU shader passes in supported windowed games.");
            ui.add_space(8.0);
            gpu_status(app,ui);
            caption(ui,"Full desktop color settings are in the main Filters page.");
        });
    }else{
        frame(PANEL).show(ui,|ui|{
            ui.set_width(width-26.0);
            ui.horizontal(|ui|{
                heading(ui,"Active visual profile");
                ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                    if red_button(ui,"Edit filters"){app.overlay_filter_tab=true;}
                });
            });
            ui.add_space(5.0);
            let active_stack=app.settings.live_stack;
            if active_stack==0 {
                let f=&crate::engine::EFFECTS[app.settings.live_preset];
                text(ui,f.label);
                caption(ui,f.desc);
            }else{
                let s=&crate::stacks::STACKS[active_stack];
                text(ui,s.label);
                caption(ui,s.desc);
            }
            caption(ui,"Selected profile is not proof that a game is rendering its shaders.");
            ui.add_space(5.0);
            gpu_status(app,ui);
            if let Some(d)=app.driver_status.clone(){
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui|{
                    if d.image_sharpen.supported{
                        let label=if d.image_sharpen.enabled{
                            format!("RIS ON · {}  |  Disable",d.sharpness)
                        }else{"RIS OFF  |  Enable".into()};
                        if ui.add_enabled(!app.driver_busy,egui::Button::new(label)).clicked(){
                            app.driver_action(adlx::DriverChange::RadeonSharpen{
                                enabled:!d.image_sharpen.enabled,level:app.driver_strength
                            });
                        }
                    }
                    if d.anti_lag.supported{
                        let label=if d.anti_lag.enabled{"Anti-Lag ON  |  Disable"}
                        else{"Anti-Lag OFF  |  Enable"};
                        if ui.add_enabled(!app.driver_busy,egui::Button::new(label)).clicked(){
                            app.driver_action(adlx::DriverChange::AntiLag(!d.anti_lag.enabled));
                        }
                    }
                });
            }
        });
    }
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui|{
        let pair=app.settings.look_pairs.get(&app.profile_key());
        if pair.is_some_and(|p|p.stored(0)&&p.stored(1)){
            if outline_button(ui,"Compare A / B"){app.toggle_look_comparison();}
        }
        let elapsed=crate::gametimer::RunTimer::fmt_elapsed(app.game_timer.elapsed());
        if outline_button(ui,&format!("{} {}",if app.game_timer.running(){"Pause"}else{"Timer"},elapsed)){
            app.toggle_game_timer();
        }
        if app.game_timer.running()&&outline_button(ui,"Split"){app.add_game_split();}
        if app.settings.crosshair.enabled{
            caption(ui,"Crosshair enabled · visible when game has focus");
        }
    });
    ui.add_space(7.0);
    ui.horizontal(|ui|{
        if outline_button(ui,"Screenshot"){
            app.screenshot(); // hides the overlay, then captures after it has disappeared
        }
        if red_button(ui,if app.recorder.active(){"Stop Recording"}else{"Record Video"}){
            app.recording();
        }
        if outline_button(ui,"Filters"){
            app.overlay_section=Page::Filters;
            app.overlay_expanded=true;
            app.overlay_filter_tab=false;
        }
    });
    ui.add_space(8.0);
    let r=ui.available_rect_before_wrap();
    let (bar,_) = ui.allocate_exact_size(vec2(width,30.0),Sense::hover());
    ui.painter().rect_filled(bar,7.0,Color32::from_rgb(44,25,40));
    ui.painter().text(bar.left_center()+vec2(12.0,0.0),egui::Align2::LEFT_CENTER,
        "External GPU filters  ·  No deep game hooking",FontId::proportional(10.8),MUTED);
    if !app.overlay_notice.is_empty(){
        ui.colored_label(Color32::from_rgb(245,190,121),&app.overlay_notice);
    }
    let _=r;
    ctx.request_repaint_after(std::time::Duration::from_millis(70));
}
#[cfg(test)]
mod tile_design_tests{
    use super::*;
    #[test]fn separate_quick_actions_are_not_all_highlighted(){
        assert!(tile_selected("Performance",Page::Performance,Page::Performance,true,false,false,false));
        assert!(!tile_selected("Game Boost",Page::Performance,Page::Performance,true,false,false,false));
        assert!(!tile_selected("Timer",Page::Performance,Page::Performance,true,false,false,false));
        assert!(tile_selected("Game Boost",Page::Performance,Page::Home,false,true,false,false));
        assert!(tile_selected("Timer",Page::Performance,Page::Home,false,false,true,false));
        assert!(tile_selected("Crosshair",Page::Extras,Page::Filters,false,false,false,true));
    }
}
