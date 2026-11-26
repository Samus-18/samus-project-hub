fn processing_budget_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"GPU PROCESSING RESOLUTION");
        caption(ui,"Area-weighted GPU downsampling before the effect stack, then reconstruction to the original output dimensions. Lower resolution may save shader work but can reduce detail; never changes the game resolution.");
        let mut p=app.settings.processing_percent;
        ui.horizontal_wrapped(|ui|{
            for value in crate::render_budget::PERCENTAGES{
                ui.selectable_value(&mut p,value,format!("{value}%"));
            }
        });
        if p!=app.settings.processing_percent{
            app.settings.processing_percent=p;
            app.settings.processing_profiles.insert(app.profile_key(),p);
            app.render_guard.reset();app.save();app.refresh_native_mirror();
        }
        let effective=app.render_guard.effective.unwrap_or(p);
        ui.colored_label(V25_MINT,format!("Active processing request: {effective}%"));
        let pixels=(effective as f32/100.0).powi(2)*100.0;
        ui.add(egui::ProgressBar::new(pixels/100.0).text(format!("{pixels:.0}% of full-size effect pixels")));
        if ui.checkbox(&mut app.render_guard_enabled,"Adaptive processing resolution (separate 1x preview)").changed(){
            app.render_guard.reset();app.refresh_native_mirror();
        }
        if app.render_guard_enabled{
            if ui.add(egui::Slider::new(&mut app.target_fps,30..=165).suffix(" FPS").text("Game target")).changed(){app.save();}
            caption(ui,"Uses fresh measured ETW game FPS only, waits for sustained changes, rebuilds own preview at most once per 10s. Recovery is slower. Off by default.");
        }
        caption(ui,"Pixel percentage is calculated, not measured GPU speedup. Down/up-scaling has its own cost. FSR1/NIS can reconstruct reduced SDR effect output.");
        let mut tuning=app.settings.lighting.clone();let mut changed=false;
        changed|=ui.checkbox(&mut tuning.reconstruction.mip_prefilter,"GPU mipmap prefilter (alternative to area)").changed();
        changed|=ui.add(egui::Slider::new(&mut tuning.reconstruction.sharpness,0.0..=1.).text("Reconstruction sharpness")).changed();
        if changed{app.apply_native_lighting(tuning);}
        caption(ui,"Optional driver-generated mip chain, only below 100% processing. Additional bounded sharpening after reconstruction; not vendor CAS. Default remains area-weighted and sharpness off.");
    });
}
fn own_memory_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"HORIZON RESOURCE MONITOR");
        if outline_button(ui,"Refresh own memory"){app.own_memory=Some(crate::process_memory::read());}
        match &app.own_memory{
            Some(Ok(m))=>{
                ui.horizontal_wrapped(|ui|{
                    ui.label(format!("{:.1} MiB resident",m.working_mib));
                    ui.label(format!("{:.1} MiB private",m.private_mib));
                    ui.label(format!("{:.1} MiB peak",m.peak_mib));
                });
                caption(ui,&format!("Process page faults since launch: {} / CPU frametime aggregation: {}",m.faults,crate::metric_math::implementation()));
            }
            Some(Err(e))=>caption(ui,e),None=>caption(ui,"Read only, refreshed every six seconds while running."),
        }
        caption(ui,"Own process RAM only. Not game RAM, not VRAM. No standby-list clearing or forced memory trimming.");
    });
}
