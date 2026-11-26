// Performance lab uses captured PresentMon ETW samples only.
// Metrics are intentionally absent if no valid samples exist.
fn performance_stability_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"EXTERNAL WGC OUTPUT · SEPARATE FROM GAME FPS");
        if let Some(m)=app.native_mirror.as_ref().filter(|m|m.running()){
            let rates=app.output_rates.snapshot(std::time::Instant::now());
            let source=rates.map(|f|format!("{:.1}",f.real)).unwrap_or("Collecting".into());
            let out=rates.map(|f|format!("{:.1}",f.total)).unwrap_or("Collecting".into());
            ui.horizontal_wrapped(|ui|{
                mini_metric(ui,160.0,"OUR CAPTURE FPS",&source,WHITE);
                mini_metric(ui,160.0,"OWN PRESENTS / S",&out,
                    Color32::from_rgb(156,217,175));
                mini_metric(ui,160.0,"SYNTHETIC FRAMES",
                    &m.generated_frames().to_string(),RED);
            });
            caption(ui,"Recent own presentation rates measured over approximately one second. Not displayed-frame rate, game-engine FPS, GPU load or input latency.");
            if let Some(v)=rates{caption(ui,&format!("Synthetic presentation rate {:.1}/s; lifetime synthetic count shown above.",v.generated));}
            if let Some(low)=app.fps.low_01_fps(){caption(ui,&format!("Measured game 0.1% low {low:.1} FPS (1000–1200 valid frames)."));}
        }else{
            caption(ui,"Independent WGC output is OFF. Open GPU Studio to start a filtered preview. Measured game FPS remains separate below.");
        }
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"REAL FRAME PACING · 600-FRAME WINDOW");
        caption(ui,"Measured PresentMon ETW intervals, not interpolation FPS or synthetic screenshots.");
        caption(ui,if app.fps.routed(){"Dominant observed swapchain selected (heuristic, not guaranteed main game surface)."}else{"Waiting for one dominant swapchain with valid process metadata."});
        caption(ui,&format!("Excluded other/unselected chain samples: {} / stale queue samples: {}",app.fps.rejected_secondary,app.fps.rejected_stale));
        caption(ui,&format!("Telemetry channel overflow: {} discarded ETW samples (not game dropped frames)",app.fps.queue_overflow()));
        caption(ui,"A source switch clears measurement history and stops active reports; no mixed A/B comparison.");
        let health=crate::perf_insights::FrameHealth::from_frames(&app.fps.frames);
        let Some(data)=health else{
            ui.add_space(12.0);
            caption(ui,"Collect at least 45 valid frames in a running selected game to compute frame-time tails, spikes and jitter.");
            if app.fps.snapshot().is_none(){caption(ui,"Currently no fresh game ETW measurements.");}
            return;
        };
        let width=((ui.available_width()-20.0)/4.0).max(105.0);
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=6.0;
            mini_metric(ui,width,"P95 FRAME TIME",&format!("{:.2} ms",data.p95_ms),WHITE);
            mini_metric(ui,width,"P99 FRAME TIME",&format!("{:.2} ms",data.p99_ms),RED);
            mini_metric(ui,width,"JITTER / FRAME",&format!("{:.2} ms",data.jitter_ms),
                Color32::from_rgb(205,148,230));
            mini_metric(ui,width,"SPIKE COUNT",&format!("{}",data.spike_count),
                Color32::from_rgb(245,185,112));
        });
        ui.add_space(8.0);
        caption(ui,&format!("{} valid frame intervals · median {:.2} ms · average {:.2} ms · worst {:.1} ms",
            data.count,data.median_ms,data.mean_ms,data.max_ms));
        caption(ui,if app.fps.snapshot().is_some(){
            "SOURCE ACTIVE · showing most recent captured frames"
        }else{"LAST RECORDED WINDOW · no new game frames currently available"});
        caption(ui,&format!("Window {:.2} s of frame intervals / longest spike streak: {}",data.sample_duration_ms/1000.,data.longest_spike_run));
        if app.fps.snapshot().is_some(){if let Some((miss,n))=crate::perf_insights::FrameHealth::target_misses(&app.fps.frames,app.target_fps){caption(ui,&format!("{} FPS target: {miss}/{n} intervals over budget ({:.1}%)",app.target_fps,100.*miss as f64/n as f64));}}
        ui.add_space(12.0);
        heading(ui,"FRAME-TIME DISTRIBUTION");
        caption(ui,"Lower ms is smoother. Bars represent proportions, not invented FPS numbers.");
        let width=ui.available_width().min(1000.0);
        let row_h=27.0;let gap=6.0;let max=data.count.max(1) as f32;
        for (i,label) in ["0–8.33 ms","8.33–16.67 ms","16.67–33.33 ms",
            "33.33–50 ms","Over 50 ms"].iter().enumerate(){
            let (bar,_) = ui.allocate_exact_size(vec2(width,row_h),egui::Sense::hover());
            let track=Rect::from_min_size(bar.min+vec2(147.0,3.0),
                vec2((width-237.0).max(80.0),row_h-6.0));
            ui.painter().rect_filled(track,6.0,Color32::from_rgb(32,25,40));
            let share=data.bands[i] as f32/max;
            if share>0.0{
                let c=if i<2{Color32::from_rgb(105,203,148)}
                    else if i==2{Color32::from_rgb(218,177,111)}
                    else{Color32::from_rgb(231,78,105)};
                ui.painter().rect_filled(Rect::from_min_size(track.min,
                    vec2(track.width()*share,track.height())),6.0,c);
            }
            ui.painter().text(bar.min+vec2(3.0,row_h/2.0),egui::Align2::LEFT_CENTER,
                *label,egui::FontId::proportional(12.0),MUTED);
            ui.painter().text(bar.right_center()-vec2(4.0,0.0),egui::Align2::RIGHT_CENTER,
                format!("{:.1}%",share*100.0),egui::FontId::proportional(12.5),WHITE);
            ui.add_space(gap);
        }
        ui.add_space(6.0);
        heading(ui,"MEASURED STABLE-CAP ADVISOR");
        if let Some(cap)=data.suggested_cap.filter(|_|app.fps.snapshot().is_some()){
            ui.horizontal_wrapped(|ui|{
                ui.colored_label(RED,egui::RichText::new(format!("{cap} FPS"))
                    .size(18.0).strong());
                caption(ui,"suggested software/game limit from 95th-percentile frame time");
            });
        }else{
            caption(ui,"A fresh window of at least 180 real frames is needed for a measured cap recommendation.");
        }
        caption(ui,"This is a recommendation only. Horizon Boost NEVER changes Roblox's frame cap or display mode automatically.");
        caption(ui,&format!("Missed budgets: {} above 16.67 ms (60 FPS), {} above 8.33 ms (120 FPS)",
            data.misses_60,data.misses_120));
        ui.add_space(7.0);
        ui.separator();heading(ui,"COMPARE MEASURED WINDOWS");
        caption(ui,"Save up to eight current real ETW windows. Compare only the same selected game; not a controlled repeated-scene benchmark.");
        ui.add(egui::TextEdit::singleline(&mut app.benchmark_label).char_limit(32).hint_text("Run label, e.g. filters off"));
        if ui.add_enabled(app.fps.snapshot().is_some()&&app.telem.active_game.is_some(),egui::Button::new("Save current measurement")).clicked(){
            if let Some(s)=app.fps.snapshot(){let key=app.profile_key();let label=if app.benchmark_label.trim().is_empty(){format!("Run {}",app.benchmark_bank.rows.len()+1)}else{app.benchmark_label.clone()};
                match app.benchmark_bank.push(&key,&label,data.clone(),s.low_1_percent_fps,app.fps.low_01_fps()){Ok(())=>app.benchmark_label.clear(),Err(e)=>app.toast(e)}
            }
        }
        for (i,v) in app.benchmark_bank.rows.iter().enumerate(){
            ui.label(format!("{}: {} frames / mean {:.2} / P95 {:.2} / P99 {:.2} ms",v.label,v.health.count,v.health.mean_ms,v.health.p95_ms,v.health.p99_ms));
            if i>0{if let Some(d)=app.benchmark_bank.delta(0,i){caption(ui,&format!("vs first: mean {:+.2}, P95 {:+.2}, P99 {:+.2} ms",d[0],d[1],d[2]));}else{caption(ui,"Different source: comparison disabled");}}
        }
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Copy measured comparison JSON"){match app.benchmark_bank.json(){Ok(v)=>ui.ctx().copy_text(v),Err(e)=>app.toast(e)}}
            if outline_button(ui,"Clear comparison"){app.benchmark_bank.rows.clear();}
        });
        if ui.add_enabled(app.fps.snapshot().is_some(),egui::Button::new("Export measured JSON")).clicked(){
            let title=app.telem.active_game.as_ref().map(|g|g.title.as_str())
                .unwrap_or("No active game");
            match data.export(title){
                Ok(dest)=>app.toast(format!("Frame report exported: {}",dest.display())),
                Err(e)=>app.toast(format!("Frame report could not save: {e}"))
            }
        }
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"GPU/CPU HEADROOM · WHEN AVAILABLE");
        caption(ui,"ETW GPU time is meaningful only when the graphics driver publishes real GPUTime samples.");
        if let Some(stats)=app.fps.snapshot(){
            if let Some(gpu)=stats.gpu_ms{
                let gpu_ratio=(gpu/stats.frametime_ms.max(0.1)).clamp(0.0,2.0);
                caption(ui,&format!("Measured mean GPU {:.2} ms vs total {:.2} ms · GPU share {:.0}%",
                    gpu,stats.frametime_ms,gpu_ratio*100.0));
                if gpu_ratio>0.85{
                    ui.colored_label(Color32::from_rgb(249,183,114),
                        "High reported GPU time relative to frame time. Try lower game graphics or compare Horizon output off/on; this is not proof of a specific bottleneck.");
                }else{
                    caption(ui,"GPU time is not the whole frame; CPU simulation or presentation may contribute. This is not a precise CPU bottleneck classifier.");
                }
            }else{caption(ui,"No driver GPU timings. Bottleneck is NOT guessed.");}
        }else{caption(ui,"No fresh ETW source. Output FPS and GPU times remain unavailable.");}
        ui.add_space(8.0);
        if let Some(stats)=app.fps.snapshot(){
            if let Some(input)=stats.input_to_photon_ms{
                caption(ui,&format!("PresentMon measured input-to-photon {:.2} ms (when published by game/driver)",input));
            }else{caption(ui,"Input-to-photon latency unavailable; not inferred from FPS");}
            if let Some(click)=stats.click_to_photon_ms{
                caption(ui,&format!("PresentMon measured click-to-photon {:.2} ms",click));
            }
        }
        ui.add_space(8.0);
        if outline_button(ui,"Open native output tools"){
            app.page=Page::Filters;
            app.filter_view=0;
        }
    });
}
