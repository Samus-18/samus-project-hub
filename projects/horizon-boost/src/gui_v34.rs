fn shader_cache_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"NATIVE SHADER BYTECODE CACHE");
        let (count,hits)=crate::shadercheck::cache_stats();
        caption(ui,&format!("{count} compiled programs cached / {hits} cache hits. Process-local, maximum 16 MB."));
        if ui.add_enabled(app.shader_warmup.is_none(),egui::Button::new("Precompile native shaders")).clicked()&&!app.is_smoke{
            let (tx,rx)=std::sync::mpsc::channel();app.shader_warmup=Some(rx);
            std::thread::spawn(move||{let _=tx.send(crate::native_filter::compile_all());});
        }
        if app.shader_warmup.is_some(){caption(ui,"Compiling on worker thread...");}
        caption(ui,"Reuses real DXBC across GPU preview restarts and resizing. Does not compile the game's own shaders.");
    });
}
fn reminder_dashboard(app:&mut BoostApp,ui:&mut Ui){
    break_and_alert_panel(app,ui);ui.add_space(8.);
    let key=app.profile_key();let mut specs=app.settings.reminder_profiles.get(&key).cloned().unwrap_or_else(crate::reminders::defaults);
    if specs.len()!=4{specs=crate::reminders::defaults();}
    let mut changed=false;let now=std::time::Instant::now();
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"REMINDER BOARD");
        caption(ui,&format!("{} running / {} paused",app.reminders.active_count(),app.reminders.paused_count()));
        if ui.checkbox(&mut app.settings.reminder_sound,"Play custom reminder sound").changed(){app.save();}
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Choose PCM WAV")&&!app.is_smoke{
                if let Some(p)=crate::filedialog::select_reminder_wav(){match crate::reminder_sound::import(&p){Ok(())=>app.toast("Reminder sound imported"),Err(e)=>app.toast(e)}}
            }
            if outline_button(ui,"Preview sound")&&!app.is_smoke{if let Err(e)=crate::reminder_sound::play(){app.toast(e);}}
        });
        caption(ui,"Local 8/16-bit PCM WAV, up to 3 seconds. Off by default.");
        caption(ui,"Four independent manual timers saved per game. No detection of real abilities or game state.");
        for (i,spec) in specs.iter_mut().enumerate(){
            frame(CARD).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                ui.horizontal_wrapped(|ui|{
                    ui.label(RichText::new(format!("{:02}",i+1)).color(RED).strong());
                    changed|=ui.add(egui::TextEdit::singleline(&mut spec.label).desired_width(105.0).char_limit(32)).changed();
                    changed|=ui.add(egui::DragValue::new(&mut spec.seconds).range(3..=3600).suffix(" s")).changed();
                    if outline_button(ui,if app.reminders.remaining(i,now).is_some(){"Restart"}else{"Start"}){app.reminders.start(i,spec,now);}
                    if ui.add_enabled(app.reminders.remaining(i,now).is_some(),egui::Button::new("Stop")).clicked(){app.reminders.stop(i);}
                    if app.reminders.paused(i){if outline_button(ui,"Resume"){app.reminders.resume(i,now);}}else if app.reminders.remaining(i,now).is_some()&&outline_button(ui,"Pause"){app.reminders.pause(i,now);}
                    if let Some(sec)=app.reminders.remaining(i,now){
                        ui.colored_label(if app.reminders.paused(i){MUTED}else{V25_MINT},format!("{sec}s{}",if app.reminders.paused(i){" paused"}else{""}));
                    }else{caption(ui,"Idle");}
                });
                if let Some(label)=app.reminders.active_label(i){if label!=spec.label.trim(){caption(ui,&format!("Running as: {label}"));}}
                if let Some(f)=app.reminders.fraction(i,now){
                    ui.add(egui::ProgressBar::new(f).desired_width(ui.available_width()).animate(app.settings.animations));
                }
            });ui.add_space(4.0);
        }
        ui.horizontal_wrapped(|ui|{if outline_button(ui,"Pause all"){app.reminders.pause_all(now);}if outline_button(ui,"Resume all"){app.reminders.resume_all(now);}if outline_button(ui,"Reset all reminders"){app.reminders.clear();}});
        caption(ui,"Choose Next reminder in HUD settings. Reminders announce completion inside Horizon and reset when switching games.");
    });
    if changed{crate::reminders::sanitize(&mut specs);app.settings.reminder_profiles.insert(key,specs);app.save();}
    ui.add_space(8.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"LOCAL TELEMETRY COMPANION");
        caption(ui,"Read-only browser dashboard on this PC. Localhost only, no LAN/phone access, no game names, no remote commands. Off by default.");
        if let Some(addr)=app.companion.addr{
            let url=format!("http://{addr}/");
            ui.horizontal_wrapped(|ui|{
                ui.colored_label(V25_MINT,&url);
                if outline_button(ui,"Copy URL"){ui.ctx().copy_text(url.clone());}
                if red_button(ui,"Stop dashboard"){app.companion.stop();}
            });
        }else if outline_button(ui,"Start local dashboard")&&!app.is_smoke{
            if let Err(e)=app.companion.start(){app.toast(e);}
        }
    });
}
