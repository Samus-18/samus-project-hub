// Horizon Boost v2.2 redesign: one native GPU path, 3 primary workspaces,
// short purposeful controls. Legacy GPL scaler remains archived but unexposed.
fn v22_switch_to(app:&mut BoostApp,ui:&mut Ui,label:&str,page:Page){
    if outline_button(ui,label){app.page=page;}
}
fn v22_game_name(app:&BoostApp)->String{
    app.telem.active_game.as_ref().map(|g|g.title.clone())
        .unwrap_or_else(||"No game selected".into())
}
fn v22_gpu_active(app:&BoostApp)->bool{
    app.native_mirror.as_ref().is_some_and(|m|m.running())
}
fn v22_native_toggles(app:&mut BoostApp,ui:&mut Ui){
    let active=v22_gpu_active(app);
    ui.horizontal_wrapped(|ui|{
        if active{
            if red_button(ui,"STOP GPU EFFECTS"){app.stop_native_mirror();}
        }else{
            if red_button(ui,"APPLY OVER GAME"){
                app.toggle_native_mirror_factor(
                    crate::native_mirror::MirrorMode::BorderlessScreen,1);
            }
            if outline_button(ui,"Preview in own window"){
                app.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);
            }
        }
        if outline_button(ui,if app.overlay_open{"Hide HUD"}else{"Open HUD"}){
            app.overlay_open=!app.overlay_open;
        }
        if outline_button(ui,"Game Library"){app.page=Page::Library;}
    });
}
fn v22_home(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Home","One workspace for visuals, responsiveness and gameplay tools");
    let width=ui.available_width();
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        ui.horizontal_wrapped(|ui|{
            ui.colored_label(Color32::from_rgb(139,218,170),"● GAME");
            ui.label(RichText::new(v22_game_name(app)).color(WHITE).strong());
            caption(ui,if v22_gpu_active(app){"GPU GLASS ACTIVE"}
                else{"External rendering OFF"});
        });
        ui.add_space(8.0);
        v22_native_toggles(app,ui);
        ui.add_space(5.0);
        caption(ui,"Native Windows Graphics Capture + D3D11. Never starts Magpie or changes Roblox render mode.");
    });
    ui.add_space(12.0);
    modern_hero(app,ui,width);
    ui.add_space(13.0);
    ui.label(RichText::new("CHOOSE YOUR WORKFLOW").color(WHITE).strong().size(15.5));
    ui.add_space(7.0);
    let options=[
        ("01  GPU Studio","Natural lights · 83 real looks",Page::Filters),
        ("02  Spatial Scaling","Independent GPU reconstruction",Page::Scaling),
        ("03  Game Extras","Crosshair · timers · practice",Page::Extras),
        ("04  Performance","Measured FPS · stability",Page::Performance),
    ];
    let columns=if width<590.0{1}else{2};
    let gap=10.0;
    for chunk in options.chunks(columns){
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=gap;
            let w=(width-gap*(columns as f32-1.0))/columns as f32;
            for (name,desc,page) in chunk{
                let (rect,resp)=click_surface(ui,w,68.0,false);
                modern_icon(ui.painter(),rect.min+vec2(25.0,26.0),*page,
                    Color32::from_rgb(252,91,123),0.75);
                ui.painter().text(rect.min+vec2(46.0,14.0),egui::Align2::LEFT_TOP,
                    trim(name,26),FontId::proportional(13.0),WHITE);
                ui.painter().text(rect.min+vec2(46.0,39.0),egui::Align2::LEFT_TOP,
                    trim(desc,34),FontId::proportional(11.0),MUTED);
                if resp.clicked(){app.page=*page;}
            }
        });
        ui.add_space(7.0);
    }
    ui.add_space(9.0);
    frame(CARD).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        ui.horizontal_wrapped(|ui|{
            caption(ui,"FRAME GENERATION");
            ui.label(RichText::new(if app.settings.experimental_fg_allowed{
                "Experimental - armed"
            }else{"Disabled for quality"}).color(
                if app.settings.experimental_fg_allowed{
                    Color32::from_rgb(248,180,108)
                }else{Color32::from_rgb(153,212,174)}));
            v22_switch_to(app,ui,"Motion Lab",Page::FrameGen);
        });
        caption(ui,"Your previous 2x/3x was reducing smoothness. Synthetic output FPS is NOT game FPS.");
    });
}
fn v22_filters(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"GPU Studio","Native D3D11 effects, calibrated lighting and display color");
    const TABS:[(&str,&str);5]=[
        ("01  Native Looks","83 original GPU looks · two authentic photographic references"),
        ("02  Light Studio","Live local light/shadow tuning, no depth buffer or DLL hooks"),
        ("03  Screen Color","Optional driver-independent Windows desktop color controls"),
        ("04  Stack Builder","Build your own ordered native GPU pipeline"),
        ("05  Color Lab","Adjustable native color grade and actual reference photo tools"),
    ];
    app.filter_view=app.filter_view.min(4);
    studio_tabs(ui,&mut app.filter_view,&TABS);
    ui.add_space(12.0);
    match app.filter_view{
        0=>{
            if !app.settings.ui_advanced{oled_looks(app,ui);return;}
            frame(CARD).show(ui,|ui|{
                ui.horizontal_wrapped(|ui|{
                    caption(ui,"RECOMMENDED");
                    if outline_button(ui,"Natural Daylight"){
                        app.select_native_preset(64);
                    }
                    if outline_button(ui,"Soft Indirect"){
                        app.select_native_preset(66);
                    }
                    if outline_button(ui,"Balanced View"){
                        app.select_native_preset(77);
                    }
                });
                caption(ui,"Extreme night-vision/green presets are stylized, not realistic lighting.");
            });
            ui.add_space(9.0);
            native_gpu_gallery(app,ui);
        },
        1=>{
            if !app.settings.ui_advanced{oled_light(app,ui);return;}
            native_lighting_controls(app,ui);
            ui.add_space(9.0);
            frame(PANEL).show(ui,|ui|{
                caption(ui,"Light Studio uses a 2D color image. Correct ray-traced GI, geometric shadows, scene depth and material data are unavailable without game integration.");
                v22_native_toggles(app,ui);
            });
        },
        2=>native_os_color_lab(app,ui),
        4=>color_photo_lab(app,ui),
        _=>native_stack_builder(app,ui),
    }
}
fn v22_scaling(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Spatial Scaling","Native D3D11 reconstruction · NO Magpie · no game DLL");
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"OWN GPU UPSCALER");
        caption(ui,"Choose the spatial reconstruction algorithm and target pixel ratio. This affects Horizon's own output window, NOT the render resolution chosen by Roblox.");
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui|{
            caption(ui,"ALGORITHM");
            egui::ComboBox::from_id_salt("v22-native-scaler")
                .width(285.0)
                .selected_text(crate::spatial::MODES[app.settings.native_scale_mode as usize])
                .show_ui(ui,|ui|{
                    let mut selection=app.settings.native_scale_mode;
                    for (index,name) in crate::spatial::MODES.iter().enumerate(){
                        ui.selectable_value(&mut selection,index as u8,*name);
                    }
                    if selection!=app.settings.native_scale_mode{
                        app.apply_scale_algorithm(selection);
                    }
                });
        });
        ui.add_space(7.0);
        ui.horizontal_wrapped(|ui|{
            caption(ui,"TARGET OUTPUT");
            for ratio in crate::spatial::RATIOS{
                let on=(app.settings.native_scale_ratio-ratio).abs()<0.01;
                let name=if ratio==1.0{"Original".into()}else{format!("{ratio:.2}x")};
                if ui.selectable_label(on,name).clicked()&&!on{
                    app.settings.native_scale_ratio=ratio;app.save();
                    app.refresh_native_mirror();
                }
            }
        });
        ui.add_space(10.0);
        let game_full=app.telem.active_game.as_ref().is_some_and(|g|g.fullscreen_like);
        if game_full{
            ui.colored_label(Color32::from_rgb(232,186,126),
                "Fullscreen-sized source: capture already fills the monitor.");
            caption(ui,"At 100% processing size, a same-size scaling pass is skipped. At lower processing percentages, Horizon reconstructs its own output. Game resolution never changes.");
        }else{
            caption(ui,"When the captured game has fewer pixels than the target monitor, the independent preview can reconstruct to 1.25x / 1.5x / 2x.");
        }
        ui.add_space(8.0);
        v22_native_toggles(app,ui);
        caption(ui,"Only 1x reconstruction is applied in same-rectangle Glass mode; there is never any forced window-to-fullscreen conversion.");
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"QUALITY / GPU BUDGET");
        caption(ui,"Adaptive starts with all selected stages and adjusts using measured GPU filter cost, not source game FPS.");
        ui.horizontal_wrapped(|ui|{
            for (q,name) in [(3u8,"Adaptive (recommended)"),(2,"Eco: 1 pass"),(1,"Balanced: 2 passes"),(0,"Full visuals")]{
                if ui.selectable_label(app.settings.native_quality==q,name).clicked(){
                    if app.settings.native_quality!=q{
                        app.settings.native_quality=q;app.save();
                        app.refresh_native_mirror();
                    }
                }
            }
        });
        ui.add_space(10.0);
        caption(ui,"Ten native D3D11 algorithms: 7 original spatial modes, official AMD FSR1 EASU+RCAS, vendor NVIDIA NIS (runs on AMD), and Lanczos3. External output only; no in-engine scaling.");
    });
}
fn v22_motion(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Motion Lab","Safe native output first · experimental optical flow only by opt-in");
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"MOTION QUALITY GUARD");
        caption(ui,"The old 2x/3x mode can delay real frames, ghost the HUD, and feel worse. It is OFF by default; no hidden animation or FPS multiplication.");
        ui.add_space(11.0);
        let mut allowed=app.settings.experimental_fg_allowed;
        if ui.checkbox(&mut allowed,"Allow EXPERIMENTAL 2D frame synthesis").changed(){
            app.settings.experimental_fg_allowed=allowed;
            if !allowed{
                // The user asked to disable harmful FG; stop our own output
                // rather than retaining an old 2x/3x worker.
                if app.native_mirror.as_ref().is_some_and(|m|m.multiplier()>1){
                    app.stop_native_mirror();
                }
                app.settings.external_fg_factor=1;
            }else if app.settings.external_fg_factor<=1{
                app.settings.external_fg_factor=2;
            }
            app.save();
        }
        ui.add_space(7.0);
        if allowed{
            ui.horizontal_wrapped(|ui|{
                caption(ui,"MULTIPLIER");
                for f in [2u8,3u8]{
                    if ui.selectable_label(app.settings.external_fg_factor==f,
                        format!("{f}x (experimental)")).clicked(){
                        app.change_fg_multiplier(f);
                    }
                }
            });
            ui.add_space(7.0);
            ui.horizontal_wrapped(|ui|{
                if v22_gpu_active(app){
                    if red_button(ui,"Stop interpolation"){app.stop_native_mirror();}
                }else{
                    if red_button(ui,"Test 2D FG in preview"){
                        app.toggle_native_mirror_factor(
                            crate::native_mirror::MirrorMode::SeparateWindow,
                            app.settings.external_fg_factor);
                    }
                    caption(ui,"Preview only. Real game Glass stays native x1 to protect responsiveness.");
                }
            });
        }else{
            ui.colored_label(Color32::from_rgb(145,221,171),"NATIVE FILTERING ONLY · NO GENERATED FRAMES");
            v22_native_toggles(app,ui);
        }
        if let Some(m)=app.native_mirror.as_ref(){
            caption(ui,&format!("Own output: {} WGC source frames, {} synthetic · {}x",
                m.submitted_frames(),m.generated_frames(),m.multiplier()));
        }
        ui.add_space(6.0);
        caption(ui,"Game display remains 1x. Synthetic frames are visible only in the separate preview, not the game; ETW game FPS is counted independently.");
    });
    ui.add_space(12.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"FRAME PACING · WHAT WE ACTUALLY MEASURE");
        fg_readout_panel(app,ui);
        caption(ui,"Independent FPS readings, never one misleading multiplied counter");
        ui.add_space(9.0);
        let width=((ui.available_width()-20.0)/3.0).max(120.0);
        let capture=app.native_mirror.as_ref()
            .and_then(|m|m.capture_average_fps())
            .map(|f|format!("{f:.1}")).unwrap_or_else(||"—".into());
        let output=app.native_mirror.as_ref()
            .and_then(|m|m.output_average_fps())
            .map(|f|format!("{f:.1}")).unwrap_or_else(||"—".into());
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=8.0;
            mini_metric(ui,width,"GAME FPS / ETW",&fps_text(app,"fps"),WHITE);
            mini_metric(ui,width,"WGC PROCESSED",&capture,Color32::from_rgb(158,221,180));
            mini_metric(ui,width,"OWN OUTPUT FPS",&output,Color32::from_rgb(211,166,235));
        });
        ui.add_space(6.0);
        caption(ui,"No valid PresentMon data = no game FPS. WGC figures include capture/GPU processing/pauses and cannot establish end-to-end input latency.");
    });
    ui.add_space(12.0);
    motion_quality_panel(app,ui);
    ui.add_space(12.0);
    frame(CARD).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"WHY WE SKIP SYNTHETIC FRAMES");
        caption(ui,"Live generation now attempts not to block the source Present queue. If the capture cadence is too slow, synthetic frames are discarded instead of inflating FPS at the cost of responsiveness.");
        ui.add_space(9.0);
        flow_timeline(app,ui);
        caption(ui,"2D optical flow is not DLSS/FSR game-integrated motion vectors. Fast camera turns, HUD and disocclusions may still distort; keep this OFF unless you are explicitly testing.");
    });
}
fn v22_overlay(app:&mut BoostApp,ctx:&egui::Context,ui:&mut Ui){
    overlay_header(app,ctx,ui);
    ui.add_space(8.0);
    let w=ui.available_width();
    let tile=((w-14.0)/3.0).max(120.0);
    ui.horizontal(|ui|{
        ui.spacing_mut().item_spacing.x=7.0;
        mini_metric(ui,tile,"GAME FPS",&fps_text(app,"fps"),WHITE);
        mini_metric(ui,tile,"FRAME TIME",&fps_text(app,"frametime"),Color32::from_rgb(225,190,136));
        mini_metric(ui,tile,"1% LOW",&fps_text(app,"low"),Color32::from_rgb(183,149,232));
    });
    ui.add_space(10.0);
    frame(CARD).show(ui,|ui|{
        ui.horizontal_wrapped(|ui|{
            caption(ui,"NOW PLAYING");
            ui.label(RichText::new(trim(&v22_game_name(app),44)).strong().color(WHITE));
        });
        caption(ui,if v22_gpu_active(app){"GPU GLASS ACTIVE"}
            else{"Original game display · effects OFF"});
        if let Some(m)=app.native_mirror.as_ref(){
            let captured=m.capture_average_fps().map(|f|format!("{f:.1}"))
                .unwrap_or_else(||"warming up".into());
            caption(ui,&format!("Own WGC processed: {captured} FPS · synthetic {} frames. NOT game FPS.",m.generated_frames()));
        }
    });
    ui.add_space(10.0);
    ui.label(RichText::new("QUICK ACTIONS").size(10.5).strong().color(MUTED));
    ui.add_space(5.0);
    ui.horizontal_wrapped(|ui|{
        if red_button(ui,if v22_gpu_active(app){"Disable GPU"}else{"Enable GPU"}){
            if v22_gpu_active(app){app.stop_native_mirror();}
            else{app.toggle_native_mirror_factor(
                crate::native_mirror::MirrorMode::BorderlessScreen,1);}
        }
        if outline_button(ui,if app.settings.crosshair.enabled{"Crosshair ON"}else{"Crosshair OFF"}){
            app.toggle_crosshair();
        }
        if outline_button(ui,if app.game_timer.running(){"Stopwatch running"}else{"Start timer"}){
            app.toggle_game_timer();
        }
        if outline_button(ui,"Screenshot"){app.screenshot();}
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"QUICK LOOK");
        caption(ui,"Switch one of the conservative looks; does not switch the main app page.");
        let choices=[(64usize,"Daylight"),(65,"Shadow Pop"),(66,"Indirect Light"),
            (69,"Natural Glow"),(77,"Balanced View")];
        ui.horizontal_wrapped(|ui|{
            for (idx,label) in choices{
                if ui.selectable_label(app.settings.native_preset==idx,label).clicked(){
                    app.select_native_preset(idx);
                }
            }
        });
        caption(ui,"Extremely colored/green and comic looks remain separate in GPU Studio.");
    });
    ui.add_space(9.0);
    egui::CollapsingHeader::new("ADVANCED  /  UPSCALING & GPU QUALITY")
        .id_salt("hb24-overlay-controls")
        .default_open(false)
        .show(ui,|ui|{
            frame(PANEL).show(ui,|ui|{
                caption(ui,"Controls stay inside the HUD. Active native output reloads when you change them.");
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui|{
                    caption(ui,"RECONSTRUCTION");
                    egui::ComboBox::from_id_salt("hb24-hud-scaler")
                        .width(224.0)
                        .selected_text(crate::spatial::MODES[app.settings.native_scale_mode as usize])
                        .show_ui(ui,|ui|{
                            let mut choice=app.settings.native_scale_mode;
                            for (i,name) in crate::spatial::MODES.iter().enumerate(){
                                ui.selectable_value(&mut choice,i as u8,*name);
                            }
                            if choice!=app.settings.native_scale_mode{
                                app.apply_scale_algorithm(choice);
                            }
                        });
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui|{
                    caption(ui,"GPU WORKLOAD");
                    egui::ComboBox::from_id_salt("hb24-hud-quality")
                        .width(186.0)
                        .selected_text(match app.settings.native_quality{
                            3=>"Adaptive",2=>"Eco",1=>"Balanced",_=>"Full",
                        })
                        .show_ui(ui,|ui|{
                            let mut q=app.settings.native_quality;
                            for (value,label) in [(3u8,"Adaptive"),(2,"Eco"),(1,"Balanced"),(0,"Full")]{
                                ui.selectable_value(&mut q,value,label);
                            }
                            if q!=app.settings.native_quality{
                                app.settings.native_quality=q;app.save();
                                app.refresh_native_mirror();
                            }
                        });
                });
                ui.add_space(5.0);
                caption(ui,"Frame Generation is preview-only; this HUD never starts synthetic Glass over the game.");
                ui.add_space(8.0);
                let mut armed=app.settings.experimental_fg_allowed;
                if ui.checkbox(&mut armed,"Allow experimental FG in SEPARATE preview").changed(){
                    app.settings.experimental_fg_allowed=armed;
                    if !armed{
                        if app.native_mirror.as_ref().is_some_and(|m|m.multiplier()>1){
                            app.stop_native_mirror();
                        }
                        app.settings.external_fg_factor=1;
                    }else if app.settings.external_fg_factor==1{
                        app.settings.external_fg_factor=2;
                    }
                    app.save();
                }
                if armed{
                    ui.horizontal_wrapped(|ui|{
                        for factor in [2u8,3u8]{
                            let same=app.native_mirror.as_ref().is_some_and(|m|
                                m.running()&&m.multiplier()==factor&&
                                m.mode()==crate::native_mirror::MirrorMode::SeparateWindow);
                            let name=if same{format!("Stop FG x{factor}")}
                                else{format!("Preview FG x{factor}")};
                            if outline_button(ui,&name){
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
                    caption(ui,"Extra images only in Horizon's preview. Game engine FPS cannot increase.");
                }
            });
        });
    caption(ui,"Alt+Z toggles this panel · stays open when editing · no game injection.");
}

/// Manual moments attached to the selected game profile. No game process
/// inspection, no input automation and no falsely promised instant replay.
fn v22_game_moments(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"LOCAL SESSION BOOKMARKS");
        caption(ui,"Mark memorable attempts, bosses or discoveries without touching game memory. The stopwatch time is included when running.");
        let key=app.profile_key();
        let count=app.settings.moments.get(&key).map_or(0,|v|v.len());
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui|{
            if red_button(ui,"Add moment marker"){
                let stamp=std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                let elapsed=app.game_timer.elapsed().as_secs();
                let items=app.settings.moments.entry(key.clone()).or_default();
                items.push(crate::gameplay::GameMoment{
                    timestamp_unix:stamp,
                    timer_elapsed_secs:elapsed,
                    label:format!("Moment {}",count+1),
                });
                crate::gameplay::sanitize_moments(items);
                app.save();
                app.toast(format!("Saved local game moment {} at {}",count+1,
                    crate::gametimer::RunTimer::fmt_elapsed(std::time::Duration::from_secs(elapsed))));
            }
            if count>0&&outline_button(ui,"Export moments JSON"){
                let dir=crate::settings::folder().join("reports");
                let result=(||->Result<std::path::PathBuf,String>{
                    std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
                    let file=dir.join("game-moments.json");
                    let data=serde_json::json!({
                        "game_profile":key,
                        "source":"manual Horizon Boost markers; no game data extracted",
                        "moments":app.settings.moments.get(&key),
                    });
                    std::fs::write(&file,serde_json::to_vec_pretty(&data)
                        .map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                    Ok(file)
                })();
                match result{
                    Ok(path)=>app.toast(format!("Moment markers exported: {}",path.display())),
                    Err(error)=>app.toast(format!("Marker export failed: {error}")),
                }
            }
            caption(ui,&format!("{count}/60 marks"));
        });
        if let Some(rows)=app.settings.moments.get(&key){
            for moment in rows.iter().rev().take(6){
                let time=crate::gametimer::RunTimer::fmt_elapsed(
                    std::time::Duration::from_secs(moment.timer_elapsed_secs));
                caption(ui,&format!("{}  ·  TIMER {}",moment.label,time));
            }
        }
        caption(ui,"These are text bookmarks, NOT video replay clips. Existing desktop recording remains under Capture.");
    });
}
