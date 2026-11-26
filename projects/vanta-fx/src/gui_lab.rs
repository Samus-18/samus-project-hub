// Performance Lab / Session tracker UI. No fake game performance numbers.
fn performance_lab_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"HORIZON BENCHMARK LAB");
        caption(ui,"Manual game FPS A/B: 1) run ETW benchmark with filters OFF, 2) save A baseline, 3) enable filters, 4) record B with the SAME game, scene and duration. Never treats synthetic frames as game FPS.");
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui|{
            caption(ui,"CAPTURE LENGTH");
            egui::ComboBox::from_id_salt("horizon-benchmark-length")
                .selected_text(format!("{} seconds",app.benchmark_length))
                .show_ui(ui,|ui|{
                    for len in [10u32,30,60,120]{
                        ui.selectable_value(&mut app.benchmark_length,len,format!("{len} seconds"));
                    }
                });
            caption(ui,"TARGET FPS");
            if ui.add(egui::Slider::new(&mut app.target_fps,30..=165).text("FPS")).changed(){app.save();}
            if app.perf_lab.active.is_some(){
                if red_button(ui,"Cancel capture"){
                    app.perf_lab.cancel();
                    app.toast("Benchmark capture cancelled");
                }
            }else if red_button(ui,"Start Benchmark"){
                app.start_benchmark();
            }
        });
        if let Some(left)=app.perf_lab.remaining(){
            ui.horizontal(|ui|{
                ui.spinner();
                ui.colored_label(Color32::from_rgb(149,221,171),
                    format!("RECORDING · {left}s left · {} actual frames",
                        app.perf_lab.active.as_ref().map_or(0,|b|b.frames.len())));
            });
        }else if let Some(e)=app.perf_lab.last_error.as_ref(){
            ui.colored_label(Color32::from_rgb(238,176,113),e);
        }
        ui.add_space(8.0);
        if let Some(result)=app.perf_lab.latest.clone(){
            ui.horizontal_wrapped(|ui|{
                metric(ui,"AVERAGE FPS",&format!("{:.1}",result.fps_avg),RED);
                metric(ui,"1% LOW",&format!("{:.1}",result.fps_one_percent_low),
                    Color32::from_rgb(208,139,229));
                metric(ui,"P99 FRAMETIME",&format!("{:.1} ms",result.frametime_p99_ms),
                    Color32::from_rgb(233,167,108));
            });
            ui.add_space(4.0);
            caption(ui,&format!("{} real frames · {}s · {:.1} ms average · {:.1} ms worst",
                result.frames,result.seconds,result.frametime_avg_ms,result.worst_frame_ms));
            caption(ui,&format!("0.1% low {:.1} FPS · {} stalls (greater than 2× median or 33ms)",
                result.fps_point_one_percent_low,result.spikes));
            ui.colored_label(Color32::from_rgb(243,193,141),
                lab::recommendation(&result,app.target_fps));
            let budget_ms=1000.0/app.target_fps.max(1) as f32;
            let too_gpu_heavy=result.gpu_mean_ms.is_some_and(|g|g>budget_ms*0.90);
            if too_gpu_heavy {
                ui.horizontal_wrapped(|ui|{
                    if red_button(ui,"Apply FPS Priority preset"){
                        app.settings.live_stack=9;
                        app.settings.live_strength=0.0;
                        app.on_gpu_filter_changed();
                    }
                    caption(ui,"Opt-in: removes extra HLSL passes from the external renderer.");
                });
            }
            ui.add_space(5.0);
            ui.horizontal_wrapped(|ui|{
                if outline_button(ui,"Export detailed CSV"){
                    match app.perf_lab.export(){
                        Ok(file)=>app.toast(format!("Benchmark exported: {}",file.display())),
                        Err(err)=>app.toast(err)
                    }
                }
                if outline_button(ui,"Save as BASELINE A"){
                    match app.perf_lab.set_baseline(){
                        Ok(())=>app.toast("Baseline A saved from genuine ETW frames. Change filters and record a new B run."),
                        Err(e)=>app.toast(format!("Baseline not saved: {e}"))
                    }
                }
            });
            if let Some(base)=app.perf_lab.baseline.as_ref(){
                caption(ui,&format!("A BASELINE · {} · PID {} · run #{} · {} ETW frames",
                    base.game,base.pid,base.run_id,base.frames));
                match app.perf_lab.comparison(){
                    Ok(delta)=>{
                        ui.add_space(8.0);
                        heading(ui,"MEASURED A/B · REAL GAME ETW ONLY");
                        let width=((ui.available_width()-18.0)/3.0).max(120.0);
                        ui.horizontal_wrapped(|ui|{
                            mini_metric(ui,width,"AVG FPS CHANGE",
                                &format!("{:+.2}",delta.fps_delta),WHITE);
                            mini_metric(ui,width,"1% LOW CHANGE",
                                &format!("{:+.2}",delta.low_1_percent_delta),WHITE);
                            mini_metric(ui,width,"P99 CHANGE",
                                &format!("{:+.2} ms",delta.p99_ms_delta),WHITE);
                        });
                        caption(ui,&format!("P95 frametime {:+.2} ms",
                            delta.p95_ms_delta));
                        if let Some(gpu)=delta.gpu_ms_delta{
                            caption(ui,&format!("ETW mean GPU time {gpu:+.2} ms"));
                        }else{caption(ui,"ETW GPU timing unavailable for this comparison");}
                        if let Some(latency)=delta.latency_ms_delta{
                            caption(ui,&format!("ETW display latency {latency:+.2} ms"));
                        }
                        if let Some(input)=delta.input_to_photon_ms_delta{
                            caption(ui,&format!("ETW input-to-photon average {input:+.2} ms"));
                        }else{caption(ui,"PresentMon input-to-photon metric not available in BOTH runs");}
                        if let Some(click)=delta.click_to_photon_ms_delta{
                            caption(ui,&format!("ETW click-to-photon average {click:+.2} ms"));
                        }
                        caption(ui,"Observational measurements only. Results DO NOT prove a boost unless identical scenes, caps, graphics settings and thermal conditions were controlled.");
                        if outline_button(ui,"Export A/B: two raw CSVs + JSON"){
                            match app.perf_lab.export_ab(){
                                Ok(file)=>app.toast(format!("Auditable A/B results: {}",file.display())),
                                Err(e)=>app.toast(format!("A/B export blocked: {e}"))
                            }
                        }
                    },
                    Err(reason)=>caption(ui,&format!("A/B pending: {reason}"))
                }
            }
        }else{
            caption(ui,"No benchmark collected. Start a capture while a game is running.");
        }
        if let Some(snap)=app.fps.snapshot(){
            ui.add_space(8.0);
            let budget=1000.0/app.target_fps.max(1) as f32;
            let gap=snap.frametime_ms-budget;
            caption(ui,&format!("Live FPS budget: {} FPS = {:.2} ms/frame · current {:.2} ms · budget {:+.2} ms",
                app.target_fps,budget,snap.frametime_ms,gap));
            // Unmodified raw game samples, not predicted/generated FPS.
            let samples:Vec<f32>=app.fps.frames.iter().rev().take(120).map(|f|f.ms).collect();
            if samples.len()>1{
                let (rect,_)=ui.allocate_exact_size(
                    egui::vec2(ui.available_width().min(840.0),84.0),egui::Sense::hover());
                ui.painter().rect_filled(rect,6.0,CARD);
                let ceiling=samples.iter().copied().fold(budget*2.0,f32::max).clamp(24.0,200.0);
                let mut points=Vec::new();
                for (i,ms) in samples.iter().rev().enumerate(){
                    let x=rect.left()+5.0+i as f32/(samples.len()-1) as f32*(rect.width()-10.0);
                    let y=rect.bottom()-5.0-(ms/ceiling).clamp(0.0,1.0)*(rect.height()-10.0);
                    points.push(pos2(x,y));
                }
                let by=rect.bottom()-5.0-(budget/ceiling).clamp(0.0,1.0)*(rect.height()-10.0);
                ui.painter().line_segment([pos2(rect.left()+4.0,by),
                    pos2(rect.right()-4.0,by)],Stroke::new(1.0_f32,MUTED));
                ui.painter().add(egui::Shape::line(points,Stroke::new(1.6_f32,RED)));
            }
        }
    });
}
fn session_tracker_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        heading(ui,"GAME SESSIONS · LOCAL HISTORY");
        caption(ui,"Tracks recognized game windows while Horizon Boost is running. No process injection.");
        ui.add_space(8.0);
        if let Some(current)=app.sessions.current.as_ref(){
            let duration=app.sessions.current_elapsed().as_secs();
            let total=app.sessions.played_seconds(&current.key);
            let (count,completed,longest)=app.sessions.summary(&current.key);
            if count>0{caption(ui,&format!("Completed sessions {count} / average {:.1} min / longest {:.1} min",completed as f64/count as f64/60.,longest as f64/60.));}
            ui.horizontal_wrapped(|ui|{
                ui.colored_label(Color32::from_rgb(156,230,165),format!("PLAYING · {}",current.name));
                caption(ui,&format!("Session {:02}:{:02}:{:02} · Total recorded {:.1}h",
                    duration/3600,(duration/60)%60,duration%60,total as f32/3600.0));
            });
        }else{caption(ui,"No tracked game right now.");}
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Export playtime CSV"){
                match app.sessions.export_csv(){
                    Ok(p)=>app.toast(format!("Playtime saved: {}",p.display())),
                    Err(e)=>app.toast(format!("Export failed: {e}"))
                }
            }
            caption(ui,&format!("{} completed local game sessions",app.sessions.history.len()));
        });
        if !app.sessions.history.is_empty(){
            ui.separator();
            for item in app.sessions.history.iter().rev().take(5){
                caption(ui,&format!("{} · {}m",item.game_name,item.duration_secs/60));
            }
        }
        ui.add_space(9.0);
        ui.separator();
        stutter_scout_controls(app,ui,false);
    });
}
/// Manual per-game gameplay tools are intentionally separate from measured
/// ETW benchmarks and local session analysis.
fn gameplay_notebook_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"GAMEPLAY NOTEBOOK");
        caption(ui,"Manual quests and local game notes · cooldown tools are in the separate Timers tab.");
        ui.add_space(10.0);
        quest_checklist_controls(app,ui,false);
        ui.add_space(13.0);
        heading(ui,"NOTES FOR THIS GAME");
        let current=app.telem.active_game.as_ref().map(|g|g.title.clone());
        caption(ui,&format!("Notes for {}",current.unwrap_or_else(||"Global / no selected game".into())));
        ui.add(egui::TextEdit::multiline(&mut app.game_note_edit).desired_rows(3)
            .desired_width(ui.available_width().min(800.0))
            .hint_text("Quest reminders, graphics tweaks, routes, settings ..."));
        if red_button(ui,"Save game note"){
            let key=app.profile_key();
            let content=app.game_note_edit.chars().take(4000).collect::<String>();
            if content.trim().is_empty(){app.settings.game_notes.remove(&key);}
            else{app.settings.game_notes.insert(key,content);}
            app.save();
            app.toast("Game note saved locally");
        }
    });
}
