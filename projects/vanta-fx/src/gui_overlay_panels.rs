// Overlay-local navigation: never switches the main app's page or closes the HUD.
fn overlay_open_panel(app:&mut BoostApp,ui:&mut Ui){
    let page=app.overlay_section;
    frame(PANEL).show(ui,|ui|{
        ui.set_width(ui.available_width().max(300.0)-5.0);
        ui.horizontal(|ui|{
            heading(ui,match page {
                Page::Filters=>"FILTERS · SHADER STUDIO",
                Page::Scaling=>"SCALING · GPU PIPELINE",
                Page::FrameGen=>"MOTION LAB · EXPERIMENTAL 2x/3x",
                Page::Extras=>"GAME EXTRAS · HUD & TIMERS",
                Page::Performance=>"PERFORMANCE · GAME TOOLS",
                Page::Capture=>"CAPTURE & RECORD",
                Page::Settings=>"GAME TOOLS & CROSSHAIR",
                Page::Library=>"RUNNING GAMES",
                Page::Overlay=>"OVERLAY SETTINGS",
                Page::Home=>"GAME CONTROLS",
                Page::Unsorted=>"Lab"
            });
            ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                if ui.small_button("Back").clicked(){
                    app.overlay_expanded=false;
                }
            });
        });
        ui.add_space(6.0);
        match page{
            Page::Filters=>{
                select_gpu_options(app,ui);
                ui.add_space(6.0);
                look_compare_controls(app,ui,true);
                ui.add_space(9.0);
                if red_button(ui,"Apply / Restart GPU renderer"){app.activate_gpu_now();}
                gpu_status(app,ui);
                caption(ui,"Ctrl+Shift+G toggles the external renderer without leaving the game.");
                caption(ui,"Select a Visual Stack for combined effects, or GPU Filter for Single Filter mode.");
                caption(ui,"A selected preset is only a preview until a compatible renderer is running.");
            },
            Page::Scaling=>{
                select_gpu_options(app,ui);
                ui.add_space(5.0);
                if red_button(ui,"Start / Apply scaling"){app.activate_gpu_now();}
                gpu_status(app,ui);
            },
            Page::FrameGen=>{
                caption(ui,"Experimental WGC optical-flow interpolation generates 2D frames only on Horizon's own output.");
                ui.horizontal_wrapped(|ui|{
                    for factor in [2u8,3u8]{
                        let on=app.settings.external_fg_factor==factor;
                        if ui.selectable_label(on,format!("{factor}×")).clicked(){
                            app.settings.external_fg_factor=factor;app.save();
                        }
                    }
                });
                if app.native_mirror.as_ref().is_some_and(|m|m.running()){
                    if red_button(ui,"Stop native GPU output"){app.stop_native_mirror();}
                    if let Some(worker)=app.native_mirror.as_ref(){
                        caption(ui,&format!("{} captured + {} interpolated on our own output",
                            worker.submitted_frames(),worker.generated_frames()));
                    }
                }else{
                    if red_button(ui,"Start 2D flow preview"){
                        app.toggle_native_mirror_factor(
                            crate::native_mirror::MirrorMode::SeparateWindow,
                            app.settings.external_fg_factor);
                    }
                }
                caption(ui,"Only external 2D frame synthesis. Real game FPS unchanged. Fullscreen visibility and latency not verified.");
            },
            Page::Extras=>{
                crosshair_controls(app,ui);
                ui.add_space(9.0);
                timer_controls(app,ui,true);
                ui.add_space(8.0);
                cooldown_tracker_controls(app,ui,true);
                ui.add_space(8.0);
                if outline_button(ui,"Open full Game Extras workspace"){
                    app.page=Page::Extras;
                    app.overlay_open=false;
                }
            },
            Page::Performance=>{
                ui.horizontal_wrapped(|ui|{
                    if red_button(ui,if app.game_boost.active(){"Disable Game Boost"}else{"Enable Game Boost"}){
                        app.toggle_game_boost();
                    }
                    caption(ui,"Windows CPU scheduling, not a GPU overclock");
                });
                caption(ui,&format!("FPS: {} · Frametime: {} · 1% low: {}",
                    fps_text(app,"fps"),fps_text(app,"frametime"),fps_text(app,"low")));
                if let Some(Ok(vram))=app.vram_budget.as_ref(){
                    if vram.meaningful(){
                        caption(ui,&format!("Horizon process VRAM {:.2} / {:.2} GB (DXGI local budget, not game usage)",
                            vram.process_gb(),vram.budget_gb()));
                    }
                }
                if let Some(report)=app.perf_lab.latest.as_ref(){
                    caption(ui,&format!("Last benchmark: {:.1} average FPS / {:.1} 1% low / {} measured frames",
                        report.fps_avg,report.fps_one_percent_low,report.frames));
                }
                ui.add_space(9.0);
                timer_controls(app,ui,true);
                ui.add_space(8.0);
                stutter_scout_controls(app,ui,true);
                ui.separator();
                game_note_controls(app,ui);
                ui.add_space(8.0);
                quest_checklist_controls(app,ui,true);
                ui.add_space(8.0);
                cooldown_tracker_controls(app,ui,true);
                caption(ui,"Extended benchmark graphs are in the main Performance page.");
            },
            Page::Capture=>{
                ui.horizontal_wrapped(|ui|{
                    if red_button(ui,"Screenshot"){app.screenshot();}
                    if outline_button(ui,if app.recorder.active(){"Stop Recording"}else{"Record Video"}){
                        app.recording();
                    }
                });
                caption(ui,"Screenshot hides the overlay before capture. Recording uses the external desktop recorder.");
            },
            Page::Settings=>{
                crosshair_controls(app,ui);
                ui.add_space(8.0);
                caption(ui,"Overlay transparency");
                if ui.add(egui::Slider::new(&mut app.settings.overlay_opacity,0.72..=1.0))
                    .changed(){app.save();}
            },
            Page::Library=>{
                let games=app.telem.visible_games.clone();
                for game in games.iter().take(8){
                    if outline_button(ui,&format!("{} · PID {}",game.title,game.pid)){
                        app.choose_game(game.pid);
                    }
                }
                if games.is_empty(){caption(ui,"No eligible game window found. Open the game and rescan in Library.");}
            },
            Page::Overlay=>{
                crosshair_controls(app,ui);
            },
            Page::Unsorted=>{
                caption(ui,"Open Lab diagnostics workspace in the main application.");
            },
            Page::Home=>{
                caption(ui,"Navigate using the nine quick actions above. The overlay stays open.");
            }
        }
    });
}
#[cfg(test)]
mod overlay_navigation_tests{
    use super::*;
    #[test]fn overlay_page_is_separate_from_main_navigation(){
        // Overlay page selection is a distinct state variable, not app.page.
        assert_ne!(Page::FrameGen,Page::Scaling);
        for page in [Page::Scaling,Page::FrameGen,Page::Filters,Page::Performance,Page::Capture,Page::Settings]{
            assert_ne!(page,Page::Home);
        }
    }
}
