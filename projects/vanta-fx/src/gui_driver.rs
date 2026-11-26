// AMD ADLX native Radeon driver controls. Actual vendor state, no game hooking.
fn driver_panel(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    frame(PANEL).show(ui,|ui|{
        if !compact {
            heading(ui,"AMD RADEON · FULLSCREEN DRIVER FEATURES");
            caption(ui,"Native AMD driver controls · no Magpie window, no EXE injection");
            ui.add_space(8.0);
        }else{
            heading(ui,"AMD GPU · NATIVE FEATURES");
        }
        if app.driver_busy{
            ui.horizontal(|ui|{
                ui.spinner();
                caption(ui,"Reading / applying AMD driver settings...");
            });
            return;
        }
        let state=app.driver_status.clone();
        let Some(status)=state else{
            caption(ui,"AMD ADLX driver state is not available yet.");
            if let Some(error)=&app.driver_error{
                ui.colored_label(Color32::from_rgb(244,184,117),error);
            }
            if outline_button(ui,"Check AMD driver"){app.refresh_driver_status();}
            return;
        };
        if status.image_sharpen.supported{
            ui.horizontal_wrapped(|ui|{
                text(ui,"Radeon Image Sharpening");
                ui.colored_label(
                    if status.image_sharpen.enabled{Color32::from_rgb(142,235,174)}
                    else{MUTED},
                    if status.image_sharpen.enabled{"ON (driver confirmed)"}else{"OFF (driver confirmed)"}
                );
                if red_button(ui,if status.image_sharpen.enabled{"Disable RIS"}else{"Enable RIS"}){
                    app.driver_action(adlx::DriverChange::RadeonSharpen{
                        enabled:!status.image_sharpen.enabled,
                        level:app.driver_strength
                    });
                }
            });
            let r=status.sharpness_range;
            let step=r.step.max(1);
            app.driver_strength=app.driver_strength.clamp(r.min_value,r.max_value);
            ui.horizontal_wrapped(|ui|{
                ui.add(egui::Slider::new(&mut app.driver_strength,
                    r.min_value..=r.max_value)
                    .step_by(step as f64)
                    .text("RIS strength"));
                if outline_button(ui,"Apply strength"){
                    app.driver_action(adlx::DriverChange::RadeonSharpen{
                        enabled:true,level:app.driver_strength
                    });
                }
            });
            caption(ui,&format!("Current actual driver sharpness: {} | supports {}–{} step {}",
                status.sharpness,r.min_value,r.max_value,step));
        }else{
            caption(ui,"Radeon Image Sharpening: unsupported by this driver");
        }
        ui.add_space(7.0);
        ui.separator();
        ui.add_space(5.0);
        if status.anti_lag.supported{
            ui.horizontal_wrapped(|ui|{
                text(ui,"AMD Radeon Anti-Lag");
                ui.colored_label(
                    if status.anti_lag.enabled{Color32::from_rgb(142,235,174)}else{MUTED},
                    if status.anti_lag.enabled{"ON (driver confirmed)"}else{"OFF (driver confirmed)"}
                );
                if outline_button(ui,if status.anti_lag.enabled{"Disable Anti-Lag"}else{"Enable Anti-Lag"}){
                    app.driver_action(adlx::DriverChange::AntiLag(!status.anti_lag.enabled));
                }
            });
        }else{
            caption(ui,"AMD Radeon Anti-Lag: unavailable on this graphics adapter");
        }
        ui.add_space(5.0);
        if !compact{
            caption(ui,"Settings are GPU/driver-wide (they are not separate per-game profiles).");
            caption(ui,"RIS can sharpen supported games in fullscreen (including supported Vulkan paths).");
            caption(ui,"Freestyle-like Bloom / Clarity / Filmic spatial passes still require the external compositing renderer.");
            caption(ui,"Ctrl + Shift + K toggles RIS even while Horizon Boost is in the background.");
            caption(ui,"Existing AMD driver settings are never modified without a click or explicit hotkey.");
        }
        if ui.small_button("Read actual driver state again").clicked(){
            app.refresh_driver_status();
        }
    });
}
