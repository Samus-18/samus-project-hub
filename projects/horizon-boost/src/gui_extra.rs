fn library(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Library","Automatic game recognition · no EXE injection");
    frame(PANEL).show(ui,|ui|{
        heading(ui,"CURRENT GAME");
        ui.add_space(6.0);
        if let Some(g)=&app.telem.active_game {
            text(ui,&g.title);caption(ui,&g.path.display().to_string());
            caption(ui,&format!("PID {} · CPU {:.2} logical cores · Game memory {} MB",
                g.pid,app.telem.cpu_percent/100.0,app.telem.game_ram_mb));
            caption(ui,if g.fullscreen_like{
                "Screen-filling or maximized HWND; this does not prove exclusive mode"
            }else{"Windowed game window detected"});
            if let Some(path)=app.fps.snapshot().and_then(|s|s.present_path){
                caption(ui,&format!("PresentMon ETW actual output: {}",path.label()));
            }else{caption(ui,"Live ETW PresentMode unavailable until real game frames arrive");}
        }else {
            caption(ui,"No game selected. Choose a running game below; no EXE modification is required.");
        }
        ui.add_space(10.0);
        if red_button(ui,"Refresh detection"){app.telem.refresh(true);}
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"RUNNING GAME WINDOWS");
        caption(ui,"Select an already-running game, including fullscreen TotK. Nothing is injected.");
        ui.add_space(8.0);
        let games=app.telem.visible_games.clone();
        if games.is_empty(){
            caption(ui,"No recognized game windows. Refresh detection to rescan.");
        }
        for game in games{
            ui.horizontal_wrapped(|ui|{
                let selected=app.telem.active_game.as_ref().is_some_and(|g|g.pid==game.pid);
                if ui.selectable_label(selected,format!("{}  ·  PID {}",game.title,game.pid)).clicked(){
                    app.choose_game(game.pid);
                }
                caption(ui,if game.fullscreen_like{"Fullscreen-sized"}else{"Windowed"});
                if app.telem.selected_pid==Some(game.pid){caption(ui,"Pinned");}
            });
            ui.add_space(4.0);
        }
        if outline_button(ui,"Auto-select foreground game"){
            app.telem.unlock_game();
            app.telem.refresh(true);
            app.toast("Automatic game selection enabled");
        }
    });
    ui.add_space(11.0);
    curated_library_panel(app,ui);
    ui.add_space(11.0);
    save_vault_panel(app,ui);
    ui.add_space(11.0);
    egui::CollapsingHeader::new("Saved profiles · import / export")
        .id_salt("library-saved-profiles").default_open(false).show(ui,|ui|{
        frame(PANEL).show(ui,|ui|{
            heading(ui,"SAVED GAME PROFILES");
            caption(ui,"Export a real game graphics profile or import a friend's .json settings.");
            ui.add_space(7.0);
            ui.horizontal_wrapped(|ui|{
                if outline_button(ui,"Export active profile"){
                    app.export_active_profile();
                }
                if outline_button(ui,"Import JSON profile"){
                    if let Some(path)=crate::filedialog::select_profile(){
                        app.import_profile(&path);
                    }
                }
            });
            ui.add_space(10.0);
            if app.settings.profiles.is_empty(){caption(ui,"No per-game profiles yet.");}
            let keys=app.settings.profiles.keys().take(24).cloned().collect::<Vec<_>>();
            for key in keys{
                ui.horizontal(|ui|{
                    let title=std::path::Path::new(&key).file_stem()
                        .map(|x|x.to_string_lossy().to_string()).unwrap_or_else(||key.clone());
                    if outline_button(ui,&format!("Profile: {title}")){
                        let running=app.telem.visible_games.iter().find(|g|{
                            g.path.to_string_lossy().eq_ignore_ascii_case(&key)
                        }).map(|g|g.pid);
                        if let Some(pid)=running {app.choose_game(pid);}
                        else {
                            app.toast("Saved profile found, but its game is not running. Launch it normally to restore the profile.");
                        }
                    }
                    caption(ui,&trim(&key,75));
                });
                ui.add_space(6.0);
            }
        });    });

}
fn overlay_page(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Overlay","Alt+Z or Ctrl+Shift+H · global hotkeys while the app is running");
    frame(PANEL).show(ui,|ui|{
        heading(ui,"ALT + Z — Overlay controls");ui.add_space(10.0);hud_monitor_options(app,ui);
        ui.horizontal(|ui|{
            text(ui,if app.overlay_open{"● Overlay visible"}else{"○ Overlay hidden"});
            if red_button(ui,if app.overlay_open{"Hide overlay"}else{"Show overlay"}){
                app.overlay_open=!app.overlay_open;
            }
        });
        ui.add_space(13.0);
        text(ui,"Transparency");
        if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.0).suffix(" opacity"))
            .changed(){app.save();}
        caption(ui,"Overlay opacity now controls the actual overlay window.");
        caption(ui,"Alt+Z or Ctrl+Shift+H opens the overlay in supported desktop/borderless modes.");
        caption(ui,"Ctrl+Shift+J toggles display-wide color changes without opening the overlay.");
        caption(ui,"Ctrl+Shift+K toggles AMD Radeon Image Sharpening via the GPU driver.");
        caption(ui,"Ctrl+Shift+B Game Boost · Ctrl+Shift+P Screenshot · Ctrl+Shift+R Record.");
        caption(ui,"Ctrl+Shift+Y Compare looks · Ctrl+Shift+T Timer · Ctrl+Shift+M Split.");
        caption(ui,"True exclusive fullscreen can hide all external windows; this is an OS limitation.");
    });
    ui.add_space(12.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"FULLSCREEN COMPATIBILITY CHECK");
        caption(ui,"Protected exclusive fullscreen and anti-cheat can hide or block ordinary external windows. Horizon Boost does not force borderless mode or bypass protections.");
        if let Some(mode)=app.fps.snapshot().and_then(|s|s.present_path){
            ui.colored_label(V25_MINT,mode.label());caption(ui,mode.glass_note());
        }else{caption(ui,"No fresh ETW present-mode data. Fullscreen visibility is unknown.");}
        ui.add_space(7.0);
        if ui.add_enabled(app.dxgi_probe.is_none(),egui::Button::new("Test native DXGI capture")).clicked(){
            app.start_dxgi_diagnostic();
        }
        if app.dxgi_probe.is_some(){caption(ui,"Probing desktop GPU capture…");}
        match app.dxgi_probe_status.as_ref(){
            Some(Ok(v))=>{
                ui.colored_label(if v.gpu_copy_complete{V25_MINT}else{MUTED},v.status);
                caption(ui,&format!("{} · {}×{} · shader passes {}",v.gpu_name,v.width,v.height,v.shader_passes));
                caption(ui,"GPU-only capture diagnostic; no screenshots saved. It does not prove visible exclusive fullscreen effects.");
            }
            Some(Err(e))=>caption(ui,e),None=>{}
        }
    });
}
fn filter_slider(ui:&mut Ui,name:&str,value:&mut f32,range:std::ops::RangeInclusive<f32>)->bool{
    ui.horizontal(|ui|{
        ui.add_sized([104.0,23.0],egui::Label::new(RichText::new(name).size(13.0)));
        ui.add(egui::Slider::new(value,range).show_value(true))
    }).inner.changed()
}
/// The former single giant Filters screen is intentionally split into
/// mutually exclusive workspaces. All existing controls remain reachable.
fn filters(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    if compact {
        basic_display_color(app,ui);
        return;
    }
    section(ui,"Filters","Dedicated workspaces for real GPU looks, Windows color, windowed filters and advanced tools.");
    const TABS:[(&str,&str);4]=[
        ("01  GPU Looks","Real photo previews · native WGC GPU shader output · preview or composited glass"),
        ("02  Screen Color","Windows Magnification color presets and optional precise display tuning"),
        ("03  Legacy","Optional external windowed GPU pipeline · no Magpie-forced fullscreen conversion"),
        ("04  Pro Tools","Shader stacks, A/B comparison, AMD driver controls and saved quality profiles"),
    ];
    studio_tabs(ui,&mut app.filter_view,&TABS);
    ui.add_space(12.0);
    match app.filter_view.min(crate::FILTER_VIEW_COUNT-1) {
        0=>{
            native_gpu_gallery(app,ui);
        },
        1=>{
            native_os_color_lab(app,ui);
            ui.add_space(10.0);
            egui::CollapsingHeader::new("Fine-tune classic Windows display color")
                .id_salt("classic_display_color").default_open(false).show(ui,|ui|{
                    basic_display_color(app,ui);
                });
        },
        2=>{
            legacy_visual_control(app,ui);
            ui.add_space(10.0);
            filter_gallery(app,ui);
        },
        _=>{
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                ui.columns(2,|cols|{
                    heading(&mut cols[0],"A / B LOOK COMPARISON");
                    caption(&mut cols[0],"Save two profiles and switch between them when comparing.");
                    cols[0].add_space(7.0);
                    look_compare_controls(app,&mut cols[0],true);
                    heading(&mut cols[1],"SAVED QUALITY PRESETS");
                    caption(&mut cols[1],"Opt-in per-game presets · no automatic driver overrides");
                    cols[1].add_space(10.0);
                    cols[1].horizontal_wrapped(|ui|{
                        if red_button(ui,"Quality"){app.set_profile(FilterProfile::quality(),"Quality");}
                        if outline_button(ui,"Performance"){app.set_profile(FilterProfile::performance(),"Performance");}
                        if outline_button(ui,"Defaults"){app.set_profile(FilterProfile::default(),"Default");}
                    });
                    cols[1].add_space(8.0);
                    caption(&mut cols[1],if app.gpu_backend.running(){
                        "Legacy GPU renderer started · verify the actual image in game"
                    }else{
                        "Look settings saved · legacy GPU renderer is not active"
                    });
                    cols[1].horizontal(|ui|{
                        if outline_button(ui,"Open GPU Looks"){app.filter_view=0;}
                    });
                });
            });
            ui.add_space(12.0);
            egui::CollapsingHeader::new("Preset library · 47 looks / custom shader builder")
                .id_salt("advanced-47-presets").default_open(false)
                .show(ui,|ui|visual_stacks_panel(app,ui));
            ui.add_space(8.0);
            egui::CollapsingHeader::new("AMD RIS and Anti-Lag · real driver state")
                .id_salt("amd-advanced-filter").default_open(false)
                .show(ui,|ui|driver_panel(app,ui,false));
        }
    }
}
fn legacy_visual_control(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"VISUAL CONTROL CENTER");
        caption(ui,"Choose a windowed-game look and verify whether the legacy renderer is really active.");
        ui.add_space(10.0);
        if ui.available_width()>=760.0 {
            ui.columns(2,|cols|{
                cols[0].set_min_width(cols[0].available_width());
                heading(&mut cols[0],"01 · SELECT LOOK");
                select_gpu_options(app,&mut cols[0]);
                cols[1].set_min_width(cols[1].available_width());
                heading(&mut cols[1],"02 · VERIFY STATUS");
                gpu_status(app,&mut cols[1]);
            });
        } else {
            select_gpu_options(app,ui);
            ui.add_space(6.0);
            gpu_status(app,ui);
        }
        let mode=if app.settings.live_stack==0{
            format!("SINGLE FILTER · {}",crate::engine::EFFECTS[app.settings.live_preset].label)
        }else{
            format!("MULTISTAGE · {}",crate::stacks::STACKS[app.settings.live_stack].label)
        };
        ui.add_space(7.0);
        ui.colored_label(RED,mode);
    });
}
fn basic_display_color(app:&mut BoostApp,ui:&mut Ui){
    let mut changed=false;
    frame(PANEL).show(ui,|ui|{
        ui.horizontal(|ui|{
            heading(ui,"Display Color · whole monitor");
            ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                let mut enabled=app.live.active;
                if ui.toggle_value(&mut enabled,"● LIVE COLOR").changed(){app.set_live_color(enabled);}
            });
        });
        caption(ui,"Live Color changes the entire Windows screen; it is restored when disabled or when the app exits.");
        ui.add_space(14.0);
        let p=&mut app.settings.profile;
        changed|=filter_slider(ui,"Vibrance",&mut p.vibrance,0.0..=1.0);
        changed|=filter_slider(ui,"Contrast",&mut p.contrast,-0.5..=0.5);
        changed|=filter_slider(ui,"Brightness",&mut p.brightness,-0.5..=0.5);
        changed|=filter_slider(ui,"Intensity",&mut p.intensity,0.0..=1.0);
        caption(ui,"These controls use a Windows system API. They do not become shader-based clarity/bloom.");
        caption(ui,"May work on borderless/fullscreen output; exclusive fullscreen behavior is driver-dependent.");
    });
    if changed {app.change_filter();}
}
fn performance(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Performance","Monitor your game, tune safe Windows tools, or inspect longer sessions.");
    egui::CollapsingHeader::new("Optimize / presets & PC analysis").default_open(false).id_salt("hb-optimizer-advanced").show(ui,|ui|optimizer_panel(app,ui));
    ui.add_space(8.);
    const TABS:[(&str,&str);5]=[
        ("01 Live Metrics","PresentMon ETW FPS, frame time and 1% low"),
        ("02 Game Tools","Opt-in scheduler Game Boost, existing Radeon controls and timers"),
        ("03 Analysis","Measured benchmarks, stutter scouting and session reports"),
        ("04 Stability","P95/P99, 600-frame distribution, jitter and stable FPS-cap advice"),
        ("05 CPU Affinity","Read and optionally restrict the selected game CPU mask"),
    ];
    studio_tabs(ui,&mut app.performance_view,&TABS);
    ui.add_space(12.0);
    match app.performance_view.min(crate::PERFORMANCE_VIEW_COUNT-1){
        0=>performance_live_summary(app,ui),
        1=>performance_game_tools(app,ui),
        2=>performance_analysis(app,ui),
        4=>affinity_panel(app,ui),
        _=>performance_stability_panel(app,ui),
    }
}
fn performance_live_summary(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        let width=((ui.available_width()-16.0)/3.0).max(105.0);
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=8.0;
            mini_metric(ui,width,"FPS",&fps_text(app,"fps"),RED);
            mini_metric(ui,width,"FRAME TIME",&fps_text(app,"frametime"),Color32::from_rgb(234,155,78));
            mini_metric(ui,width,"1% LOW",&fps_text(app,"low"),Color32::from_rgb(203,116,230));
        });
        caption(ui,if app.fps.snapshot().is_some(){
            "Measured via Intel PresentMon ETW · real GPU frame timings"
        }else{"No game frames captured yet · data is never fabricated"});
        if let Some(stats)=app.fps.snapshot(){
            ui.add_space(6.0);
            heading(ui,"FRAME PACING INSIGHT");
            if let Some(one_low)=stats.low_1_percent_fps{
                if one_low<stats.fps*0.68{
                    ui.colored_label(Color32::from_rgb(245,176,113),
                        "Noticeable 1% low gap. Look for loading stutters, background work or CPU bottlenecks.");
                }else{
                    caption(ui,"1% lows are relatively close to the measured average FPS.");
                }
            }else{
                caption(ui,"Collect at least 100 real frames to diagnose 1% low consistency.");
            }
            if let Some(gpu)=stats.gpu_ms{
                if gpu>stats.frametime_ms*0.85 {
                    caption(ui,"GPU time is close to frame time. Lowering graphics settings may help more than scheduler priority.");
                }else {
                    caption(ui,"GPU time is below overall frame time; CPU work, frame caps, presentation and idle waits are all possible. Cause not established.");
                }
            }
        }
    });
    ui.add_space(12.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"Live system readings");
        let width=((ui.available_width()-16.0)/3.0).max(105.0);
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=8.0;
            mini_metric(ui,width,"CPU CORES",&format!("{:.2}",app.telem.cpu_percent/100.0),RED);
            mini_metric(ui,width,"PROCESS RAM",&format!("{} MB",app.telem.game_ram_mb),MUTED);
            mini_metric(ui,width,"SYSTEM RAM",&format!("{:0.1}/{:0.1} GB",app.telem.used_ram_gb,
                app.telem.total_ram_gb),Color32::from_rgb(207,143,234));
        });
        ui.add_space(13.0);
        caption(ui,"CPU usage history · real measurements sampled ~1 second");
        let (r,_)=ui.allocate_exact_size(Vec2::new(ui.available_width().min(1080.0),110.0),egui::Sense::hover());
        ui.painter().rect_filled(r,8.0,CARD);
        let samples=&app.telem.cpu_history;
        if samples.len()>1 {
            let mut points=Vec::new();
            for (i,v) in samples.iter().enumerate(){
                let x=r.left()+8.0+(i as f32/(samples.len()-1) as f32)*(r.width()-16.0);
                let y=r.bottom()-6.0-v.clamp(0.0,100.0)/100.0*(r.height()-12.0);
                points.push(egui::pos2(x,y));
            }
            ui.painter().add(egui::Shape::line(points,Stroke::new(2.0_f32,RED)));
        }
        ui.add_space(10.0);
        text(ui,&format!("Graphics adapter: {}",app.gpu));
        ui.add_space(8.0);
        heading(ui,"GPU VRAM · READ ONLY");
        match app.vram_budget.as_ref(){
            Some(Ok(info)) if info.meaningful()=>{
                caption(ui,&format!("{} · Horizon Boost process {:.2} GB / adapter budget {:.2} GB",
                    info.device_name,info.process_gb(),info.budget_gb()));
                ui.add(egui::ProgressBar::new(info.fraction()).desired_width(
                    ui.available_width().min(600.0)).show_percentage());
                caption(ui,"DXGI per-process memory usage + local allocation budget. NOT total game/system VRAM.");
                if info.process_usage_bytes>info.process_budget_bytes{ui.colored_label(RED,"Horizon is over its local GPU memory budget. Reduce preview resolution or disable costly stages.");}
                else if info.fraction()>0.80{
                    ui.colored_label(Color32::from_rgb(245,181,116),
                        "Horizon Boost process is near its local GPU memory budget.");
                }
            },
            Some(Err(e))=>caption(ui,&format!("Read-only DXGI VRAM info unavailable: {e}")),
            _=>caption(ui,"DXGI process-local VRAM budget pending; no numbers invented"),
        }
    });
}
fn performance_game_tools(app:&mut BoostApp,ui:&mut Ui){
    processing_budget_panel(app,ui);
    ui.add_space(8.0);
    own_memory_panel(app,ui);
    process_load_panel(app,ui);
    frame(PANEL).show(ui,|ui|{
     heading(ui,"HARDWARE SENSOR READINGS / AMD ADL");
     ui.checkbox(&mut app.sensor_enabled,"Read AMD GPU temperatures every 5 seconds");
     match app.sensors.value(std::time::Instant::now()){
      Some(Ok(rows))=>{for s in rows{let f=|v:Option<f32>|v.map(|n|format!("{n:.1} C")).unwrap_or("Unavailable".into());ui.label(&s.name);caption(ui,&format!("Edge {} / hotspot {} / memory {}",f(s.edge),f(s.hotspot),f(s.memory)));}if rows.is_empty(){caption(ui,"No supported AMD adapter sensors.");}},
      Some(Err(e))=>caption(ui,e),None=>caption(ui,"Not sampled / stale"),
     }
     caption(ui,"Actual installed driver readings. CPU temperature is unavailable, not estimated. No fan, voltage or clock writes. Emergency stop disables polling.");
    });
    ui.add_space(8.0);
    capture_health_panel(app,ui);
    display_topology_panel(app,ui);
    ui.add_space(8.0);
    hardware_bench_panel(app,ui);
    pipeline_profile_panel(app,ui);
    ui.add_space(8.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"OWN GPU PREVIEW · FRAME BUDGET");
        caption(ui,"Limits Horizon's independent WGC/DXGI preview rate only. It NEVER changes the game's actual FPS, render queue or refresh rate.");
        ui.add_space(7.0);
        ui.horizontal_wrapped(|ui|{
            ui.label("Preview limit");
            let current=app.settings.preview_fps_limit;
            let name=if current==0{"Uncapped".to_string()}else{format!("{current} FPS")};
            egui::ComboBox::from_id_salt("preview-fps-limit")
                .selected_text(name).width(118.0).show_ui(ui,|ui|{
                    for cap in crate::preview_pacer::LIMITS {
                        let label=if cap==0{"Uncapped".to_owned()}else{format!("{cap} FPS")};
                        if ui.selectable_label(current==cap,label).clicked(){
                            app.settings.preview_fps_limit=cap;
                            app.preview_guard.reset();
                            if let Some(worker)=app.native_mirror.as_ref(){
                                worker.set_preview_limit(cap);
                            }
                            app.save();
                        }
                    }
                });
            if let Some(mirror)=app.native_mirror.as_ref(){
                if let Some(value)=mirror.capture_average_fps(){
                    ui.colored_label(V25_MINT,format!("{value:.1} captured FPS (own preview)"));
                }
            }
        });
        caption(ui,"Default 45. Windows timer granularity and source capture can cause deviations from the cap. Higher limits may cost GPU/CPU. Experimental x2/x3 has its own pacing; no game FPS modification.");
        ui.add_space(6.0);
        if ui.checkbox(&mut app.preview_guard_enabled,"Adaptive preview guard · use measured game FPS").changed(){
            app.preview_guard.reset();
            if let Some(m)=app.native_mirror.as_ref(){m.set_preview_limit(app.settings.preview_fps_limit);}
        }
        if app.preview_guard_enabled{
            ui.horizontal_wrapped(|ui|{
                if ui.add(egui::Slider::new(&mut app.target_fps,30..=165).text("Game target").suffix(" FPS")).changed(){app.save();}
                let cap=app.preview_guard.effective.unwrap_or(app.settings.preview_fps_limit);
                caption(ui,&format!("Own output cap: {}",if cap==0{"uncapped".into()}else{format!("{cap} FPS")}));
            });
            caption(ui,"After sustained low ETW game FPS, reduces only Horizon's own 1x output workload. Recovers slowly after stable readings. Off by default; no game-setting changes.");
        }
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"GAME BOOST · WINDOWS SCHEDULER");
        caption(ui,"A new native OS feature, not a Radeon driver toggle.");
        caption(ui,"Opt-in: raises the selected NORMAL-priority game to ABOVE NORMAL only.");
        caption(ui,"No DLL injection, no High/Realtime priority, and no system power-plan changes.");
        ui.add_space(9.0);
        if let Some(game)=app.telem.active_game.as_ref(){
            caption(ui,&format!("Detected: {} (PID {})",game.title,game.pid));
        }else{caption(ui,"No game selected. Open Library to choose a running game.");}
        ui.horizontal_wrapped(|ui|{
            let active=app.game_boost.active();
            if red_button(ui,if active{"Disable Game Boost"}else{"Enable Game Boost"}){
                app.toggle_game_boost();
            }
            ui.colored_label(if active{Color32::from_rgb(141,225,161)}else{MUTED},
                if active{"ABOVE NORMAL · verified by Windows"}else{"OFF · original priority"});
        });
        caption(ui,"May help CPU-bound scheduling, but does not guarantee more FPS on GPU-limited RX 580.");
        caption(ui,"Automatically restores original priority when the game changes, exits, or Horizon Boost closes.");
        caption(ui,"Ctrl + Shift + B toggles Game Boost without opening the overlay.");
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"OPTIONAL 1 MS TIMER · HORIZON PROCESS ONLY");
        caption(ui,"Windows timeBeginPeriod(1) adjusts scheduling granularity for Horizon Boost's own timers. It does not unlock game FPS, change the GPU clock or reduce measured input latency by itself.");
        caption(ui,"Higher timer precision can increase idle CPU and power usage. It starts OFF and is automatically released when Horizon Boost exits.");
        ui.horizontal_wrapped(|ui|{
            let active=app.timer_precision.active();
            if outline_button(ui,if active{"Release 1 ms timer"}else{"Request 1 ms timer"}){
                let result=if active {
                    app.timer_precision.disable().map(|()|"1 ms timer released".to_string())
                }else{app.timer_precision.enable()};
                match result{
                    Ok(message)=>app.toast(message),
                    Err(error)=>app.toast(format!("Timer request: {error}")),
                }
            }
            ui.colored_label(if active{Color32::from_rgb(141,225,161)}else{MUTED},
                if active{"OS REQUEST ACTIVE · OWN PROCESS"}else{"OFF · DEFAULT"});
        });
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"WINDOWS POWER PLAN");
        caption(ui,"Optional system-wide change. No automatic activation. High performance can increase heat and power use; it does not guarantee higher FPS.");
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Read current plan"){
                match app.power_plan.refresh(){Ok(())=>{},Err(e)=>app.toast(e)}
            }
            if let Some(g)=app.power_plan.observed{ui.label(g.label());}
        });
        ui.horizontal_wrapped(|ui|{
            for (i,name) in [(0u8,"Balanced"),(1u8,"High performance"),(2u8,"Power saver")]{
                if outline_button(ui,name){
                    match app.power_plan.apply(i){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}
                }
            }
            if ui.add_enabled(app.power_plan.active(),egui::Button::new("Restore original")).clicked(){
                match app.power_plan.restore(){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}
            }
        });
        caption(ui,"Normal close restores the original plan. If the app is interrupted, a recovery journal allows explicit manual restoration. Never switches Windows power plans automatically.");
        if ui.small_button("Check interrupted-session recovery").clicked(){match app.power_plan.recovery_hint(){Ok(Some(msg))=>app.toast(msg),Ok(None)=>app.toast("No pending power-plan recovery"),Err(e)=>app.toast(e)}}
        if ui.small_button("Restore/clear interrupted-session journal (confirm Windows state)").clicked(){match app.power_plan.recover_previous(){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}}
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"NETWORK CHECK · ICMP");
        caption(ui,"Optional one-address IPv4 ping every 2 seconds. 127.0.0.1 tests your own PC. Enter your router/server address manually; not measured game-server RTT.");
        ui.horizontal_wrapped(|ui|{
            ui.add_enabled(!app.network_probe.active()&&!app.network_probe.stopping(),
                egui::TextEdit::singleline(&mut app.network_address).desired_width(145.0));
            if app.network_probe.active(){
                if outline_button(ui,"Stop network check"){app.network_probe.stop();}
            }else if app.network_probe.stopping(){ui.spinner();caption(ui,"Stopping network check");}
            else if red_button(ui,"Start network check"){
                if let Err(e)=app.network_probe.start(&app.network_address){app.toast(e);}
            }
        });
        if let Some((rtt,loss,n))=app.network_probe.stats(){
            let rtt=rtt.map(|ms|format!("{ms} ms")).unwrap_or("Timeout / no reply".into());
            ui.colored_label(V25_MINT,format!("{rtt} · {loss:.0}% no replies · {n} probes"));
        }else{caption(ui,if app.network_probe.active(){"Waiting for ICMP responses"}else{"OFF · no probes sent"});}
        if let Some(d)=app.network_probe.distribution(){
            let fmt=|v:Option<f64>|v.map(|v|format!("{v:.1} ms")).unwrap_or("Unavailable".into());
            caption(ui,&format!("{} successful replies / median {} / P95 {} / adjacent-reply jitter {}",d.replies,fmt(d.median),fmt(d.p95.map(f64::from)),fmt(d.jitter)));
        }
        if app.network_probe.discarded>0{caption(ui,&format!("{} monitor samples lost to GUI queue overflow; measurement window restarted (not network loss)",app.network_probe.discarded));}
        if let Some(error)=app.network_probe.error.as_deref(){caption(ui,error);}
        caption(ui,"ICMP may be blocked or deprioritized. No-reply percentage is not proof of in-game packet loss. Use HUD settings to display ping.");
    });
    ui.add_space(10.0);
    timer_panel(app,ui,false);
    ui.add_space(8.0);
    egui::CollapsingHeader::new("Gameplay notebook · quests / cooldowns / local notes")
        .id_salt("perf-game-notebook").default_open(false)
        .show(ui,|ui|gameplay_notebook_panel(app,ui));
    ui.add_space(10.0);
    egui::CollapsingHeader::new("AMD driver features · RIS / Anti-Lag")
        .id_salt("perf-amd-tools").default_open(false).show(ui,|ui|{
            driver_panel(app,ui,true);
        });
}
fn performance_analysis(app:&mut BoostApp,ui:&mut Ui){
    performance_lab_panel(app,ui);
    ui.add_space(10.0);
    session_tracker_panel(app,ui);
}
fn metric(ui:&mut Ui,label:&str,value:&str,color:Color32){
    frame(CARD).show(ui,|ui|{
        ui.set_min_width(132.0);
        caption(ui,label);
        ui.label(RichText::new(value).size(23.0).strong().color(color));
    });
}
fn capture(app:&mut BoostApp,ui:&mut Ui){
 let id=egui::Id::new("capture-workspace-v114");let mut tab=ui.data(|d|d.get_temp::<usize>(id)).unwrap_or(0).min(5);
 ui.horizontal_wrapped(|ui|{for(i,n)in [rm(app,"Record & replay","Záznam a replay"),rm(app,"Clip library","Knihovna klipů"),rm(app,"Montage","Střih"),rm(app,"Audio","Zvuk"),rm(app,"Encoder settings","Enkodér"),rm(app,"Performance","Výkon")].iter().enumerate(){let res=ui.selectable_label(tab==i,*n);motion_button(ui,&res);if res.clicked(){tab=i;}}});
 ui.data_mut(|d|d.insert_temp(id,tab));ui.add_space(10.);
 ui.horizontal_wrapped(|ui|{ui.label(if app.recorder.active(){rm(app,"REC / recording","REC / nahrávání")}else if app.recorder.finishing(){rm(app,"Finalizing","Dokončování")}else if app.replay.active(){rm(app,"Replay buffer active","Replay aktivní")}else{rm(app,"Capture idle","Záznam vypnutý")});if app.media_export.is_some(){ui.spinner();ui.label("Export in progress");}});
 match tab{0=>{capture_record_controls(app,ui);egui::CollapsingHeader::new(rm(app,"Highlights, OCR & voice","Highlighty, OCR a hlas")).show(ui,|ui|{highlight_panel(app,ui);assist_panel(app,ui);});},1=>capture_library_panel(app,ui),2=>montage_panel(app,ui),3=>capture_audio_panel(app,ui),4=>capture_encoder_panel(app,ui),_=>capture_metrics_panel(app,ui)}
}
fn capture_record_controls(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,rm(app,"Screenshot","Snímek obrazovky"));caption(ui,rm(app,"Capture your Windows display into Pictures/HorizonBoost.","Uloží obrazovku Windows do Pictures/HorizonBoost."));
        ui.add_space(8.0);
        ui.horizontal(|ui|{
            if red_button(ui,rm(app,"Take Screenshot","Pořídit snímek")){app.screenshot();}
            if outline_button(ui,rm(app,"Open Screenshots","Otevřít snímky")){crate::record::open_folder("pictures");}
        });
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,rm(app,"INSTANT REPLAY / VIDEO BUFFER","INSTANT REPLAY"));
        caption(ui,rm(app,"Optional AMD AMF or CPU FFmpeg desktop video. No game hooks or audio.","Video z plochy přes AMD AMF nebo CPU. Replay je bez zvuku."));
        let enabled=app.replay.active();
        ui.add_enabled_ui(!enabled&&!app.replay.saving(),|ui|{
            ui.horizontal_wrapped(|ui|{
                let key=app.profile_key();let mut seconds=app.settings.replay_game_lengths.get(&key).copied().unwrap_or(app.settings.replay_seconds);
                let mut changed=false;
                for n in [15,30,60,120]{changed|=ui.selectable_value(&mut seconds,n,format!("{n} s")).changed();}
                changed|=ui.add(egui::Slider::new(&mut seconds,15..=120).step_by(5.).suffix(" s").text(rm(app,"Custom","Vlastní"))).changed();
                if changed{app.settings.replay_game_lengths.insert(key,crate::instant_replay::replay_seconds(seconds));app.save();}
                if enabled{ui.label(format!("Active: {} s",app.replay.seconds()));}
            });
        });
        ui.horizontal_wrapped(|ui|{
            if red_button(ui,if enabled{rm(app,"Stop Replay Buffer","Vypnout replay")}else{rm(app,"Start Replay Buffer","Zapnout replay")}){app.toggle_replay();}
            if ui.add_enabled(enabled&&app.replay.ready()&&!app.replay.saving(),
                egui::Button::new(rm(app,"Save replay","Uložit replay")).fill(CARD)).clicked(){app.save_replay();}
        });
        ui.colored_label(if enabled{V25_MINT}else{MUTED},
            if app.replay.saving(){rm(app,"SAVING","UKLÁDÁM")}else if enabled&&app.replay.ready(){rm(app,"BUFFER READY","REPLAY PŘIPRAVEN")}else if enabled{rm(app,"BUFFER WARMING UP","REPLAY SE PLNÍ")}else{rm(app,"OFF","VYPNUTO")});
        caption(ui,rm(app,"Replay uses complete 5-second segments. Configure disk and bitrate in Encoder settings. Recording and replay are exclusive.","Replay využívá dokončené pětisekundové úseky. Disk a bitrate nastav v Enkodéru. Replay a záznam nelze spustit současně."));
        if let Some(last)=app.replay.last_file.as_ref(){
            caption(ui,&format!("Last replay: {}",last.display()));
        }
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,rm(app,"Video Recorder","Videozáznam"));
        caption(ui,rm(app,"Desktop video to MKV; optional MP4 export and audio consent in Audio tab.","Video plochy do MKV; volitelný MP4 export a souhlas se zvukem na kartě Zvuk."));
        ui.add_space(9.0);
        ui.horizontal(|ui|{
            if red_button(ui,if app.recorder.active(){rm(app,"Stop Recording","Ukončit záznam")}else{rm(app,"Start Recording","Spustit záznam")}){
                app.recording();
            }
            if outline_button(ui,rm(app,"Open Videos","Otevřít videa")){crate::record::open_folder("videos");}
        });
        ui.add_space(7.0);
        if app.recorder.active(){
            ui.colored_label(RED,"● REC · Recording desktop");
        }else{caption(ui,rm(app,"No active recording","Bez aktivního záznamu"));}
        ui.add_space(9.0);
        ui.horizontal(|ui|{
            ui.label("FPS");
            let mut fps=app.settings.record_fps as i32;
            if ui.add(egui::Slider::new(&mut fps,15..=60)).changed(){
                app.settings.record_fps=fps as u32;app.save();
            }
            ui.label(rm(app,"Resolution","Rozlišení"));
            let selected=app.settings.record_width;
            egui::ComboBox::from_id_salt("record-width").selected_text(if selected==1280{"720p"}else{"1080p"})
                .show_ui(ui,|ui|{
                    if ui.selectable_value(&mut app.settings.record_width,1280,"720p").changed(){app.save();}
                    if ui.selectable_value(&mut app.settings.record_width,1920,"1080p").changed(){app.save();}
                });
        });
        let encoder_label=rm(app,"Prefer AMD AMF hardware encoding","Preferovat hardwarový AMD AMF enkodér");if ui.checkbox(&mut app.settings.hardware_encoder,encoder_label).changed(){app.save();}
        recording_session_panel(app,ui);
    });
    ui.add_space(8.);
}
fn settings(app:&mut BoostApp,ui:&mut Ui){
    if outline_button(ui,"Find a tool / Ctrl+K"){app.palette_open=true;}
    section(ui,"Settings","Animations · overlay · automatic profiles · gaming shortcuts");
    frame(PANEL).show(ui,|ui|{
        heading(ui,"Application appearance");
        ui.horizontal(|ui|{for(i,n)in ["English","Čeština"].iter().enumerate(){if ui.selectable_value(&mut app.settings.ui_language,i as u8,*n).changed(){app.save();}}});
        if outline_button(ui,local(ui,"Return to simple mode","Zpět do jednoduchého režimu")){app.settings.ui_advanced=false;app.save();}
        ui.horizontal_wrapped(|ui|{
            for(i,name) in ["Crimson","Obsidian","Ice"].iter().enumerate(){
                if ui.selectable_value(&mut app.settings.ui_theme,i as u8,*name).changed(){app.save();}
            }
        });
        caption(ui,"Theme changes panel surfaces and control styling. Red branding stays consistent.");
        if ui.add(egui::Slider::new(&mut app.settings.ui_zoom,0.75..=1.35).step_by(0.05).text("UI zoom")).changed(){app.save();}
        if outline_button(ui,"Reset UI zoom"){app.settings.ui_zoom=1.0;app.save();}
        caption(ui,"WINDOW SIZE · independent from the passive gaming HUD");
        ui.horizontal_wrapped(|ui|{
            for (name,size) in [
                ("Small 840×560",[840.0,560.0]),
                ("Balanced 940×620",[940.0,620.0]),
                ("Large 1180×760",[1180.0,760.0]),
            ] {
                if outline_button(ui,name){
                    app.set_main_window_size(ui.ctx(),size);
                }
            }
        });
        caption(ui,"You can also use the − / + buttons next to the window controls or drag a window edge. The last size is saved when you exit.");
        ui.add_space(9.0);
        caption(ui,"HUD / OVERLAY SIZE · still mouse-pass-through while compact");
        let mut percent=app.settings.overlay_size_scale*100.0;
        if ui.add(egui::Slider::new(&mut percent,78.0..=135.0)
            .step_by(1.0).text("Scale").suffix(" %")).changed(){
            app.settings.overlay_size_scale=(percent/100.0).clamp(0.78,1.35);
            app.save();
        }
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Tiny HUD (80%)"){
                app.settings.overlay_size_scale=0.80;app.save();
            }
            if outline_button(ui,"Default (88%)"){
                app.settings.overlay_size_scale=0.88;app.save();
            }
            if outline_button(ui,"Larger HUD (115%)"){
                app.settings.overlay_size_scale=1.15;app.save();
            }
        });
        caption(ui,"Size takes effect immediately for both the mini HUD and the editor. Mini HUD remains passive; open controls only with Alt+Shift+Z.");
        ui.horizontal_wrapped(|ui|{
            let corners=["Top left","Top right","Bottom left","Bottom right"];
            egui::ComboBox::from_id_salt("hud-corner").selected_text(corners[app.settings.hud_corner.min(3) as usize])
                .show_ui(ui,|ui|{
                    for (i,name) in corners.iter().enumerate(){
                        if ui.selectable_value(&mut app.settings.hud_corner,i as u8,*name).changed(){app.save();}
                    }
                });
            let metrics=["GPU state","Frame time","1% low FPS","GPU time","Network ping","Stopwatch","Macro state","Next reminder","0.1% low FPS","Own output rate"];
            egui::ComboBox::from_id_salt("hud-metric").selected_text(metrics[app.settings.hud_metric.min(9) as usize])
                .show_ui(ui,|ui|{
                    for (i,name) in metrics.iter().enumerate(){
                        if ui.selectable_value(&mut app.settings.hud_metric,i as u8,*name).changed(){app.save();}
                    }
                });
        });
        ui.add_space(10.0);
        if ui.checkbox(&mut app.settings.animations,"Subtle motion: light sweep, border and hover transitions").changed(){app.save();}
        if ui.checkbox(&mut app.settings.compact_controls,"Compact preset gallery (show recommended first)").changed(){
            app.show_all_presets=!app.settings.compact_controls;
            app.save();
        }
        caption(ui,"Animations run at a capped refresh rate to avoid unnecessary GPU load.");
        ui.add_space(10.0);
        text(ui,"Overlay opacity");
        if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.0))
            .changed(){app.save();}
    });

    ui.add_space(10.0);
    shortcut_panel(app,ui);
    hud_layout_panel(app,ui);
    ui.add_space(10.0);
    portable_profile_panel(app,ui);
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"PRIVATE TECHNICAL DIAGNOSTICS");
        caption(ui,"Whitelist of app version, feature counts, renderer preferences, dependencies and numeric own-capture counters. No raw logs, notes, paths, process IDs or account data.");
        if outline_button(ui,"Copy diagnostic JSON"){
            if let Ok(v)=serde_json::to_string_pretty(&crate::diagnostic_export::with_capture(&app.settings,app.native_mirror.as_ref().map(|w|w.diagnostics()))){ui.ctx().copy_text(v);}
        }
        if outline_button(ui,"Save anonymized diagnostic ZIP")&&!app.is_smoke{
            let result=(||->Result<String,String>{let dir=crate::settings::folder().join("diagnostics");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
             let path=dir.join(format!("diagnostic-{}.zip",crate::media_tools::stamp()));let bytes=crate::diagnostic_export::bundle(&app.settings,app.native_mirror.as_ref().map(|w|w.diagnostics()))?;
             crate::settings::atomic_json_replace(&path,&bytes)?;Ok(format!("Saved privacy-safe ZIP: {}",path.display()))})();app.toast(result.unwrap_or_else(|e|e));
        }
        if outline_button(ui,"Save diagnostic JSON")&&!app.is_smoke{
            let result=(||->Result<String,String>{
                let dir=crate::settings::folder().join("diagnostics");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
                let path=dir.join(format!("diagnostic-{}.json",crate::media_tools::stamp()));
                let data=serde_json::to_vec_pretty(&crate::diagnostic_export::with_capture(&app.settings,app.native_mirror.as_ref().map(|w|w.diagnostics()))).map_err(|e|e.to_string())?;
                crate::settings::atomic_json_replace(&path,&data)?;Ok(format!("Diagnostic saved: {}",path.display()))
            })();app.toast(result.unwrap_or_else(|e|e));
        }
    });
    ui.add_space(10.0);
    shader_cache_panel(app,ui);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"GAME EXTRAS MOVED TO THEIR OWN WORKSPACE");
        caption(ui,"Crosshair, stopwatch, quests and gameplay aids now live together in Game Extras.");
        if outline_button(ui,"Open Game Extras"){app.page=Page::Extras;}
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"WINDOWS GAMING QUICK TOOLS");
        caption(ui,"Opens official Windows Settings pages. Never changes registry, power plan or GPU settings automatically.");
        ui.add_space(7.0);
        ui.horizontal_wrapped(|ui|{
            for (i,(name,_)) in crate::win::WINDOWS_GAME_TOOLS.iter().enumerate(){
                if outline_button(ui,name){
                    match crate::win::open_game_tool(i){
                        Ok(())=>app.toast(format!("Windows Settings: {name}")),
                        Err(e)=>app.toast(e)
                    }
                }
            }
        });
    });
    ui.add_space(11.0);
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        heading(ui,"System");
        text(ui,&format!("GPU: {}",app.gpu));
        if ui.checkbox(&mut app.settings.detect_games,"Detect foreground games automatically").changed(){app.save();}
        if ui.checkbox(&mut app.settings.fps_monitoring,"Measure real FPS and frametimes using PresentMon ETW").changed(){
            if !app.settings.fps_monitoring{app.fps.stop();}
            app.save();
        }
        caption(ui,&format!("ETW status: {}",app.fps.note));
        if outline_button(ui,"Restore original screen colors"){app.set_live_color(false);}
        caption(ui,"No unrequested popups. Actions show inline status messages.");
        caption(ui,"Screenshot: Ctrl+Shift+P · Record: Ctrl+Shift+R · Game Boost: Ctrl+Shift+B.");
        caption(ui,"Crosshair: Ctrl+Shift+X · Compare A/B: Ctrl+Shift+Y.");
        caption(ui,"Stopwatch: Ctrl+Shift+T · Mark split: Ctrl+Shift+M.");
        caption(ui,"Game files and anti-cheat modules are not modified.");
    });
}
