// Compact practical game tools. Panels reuse established Horizon design.
fn look_compare_controls(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    let key=app.profile_key();
    let pair=app.settings.look_pairs.get(&key).cloned().unwrap_or_default();
    ui.horizontal_wrapped(|ui|{
        heading(ui,"A / B LOOK COMPARE");
        if !compact{caption(ui,"Ctrl + Shift + Y switches between stored looks");}
    });
    if !compact{
        caption(ui,"Save two native shader/color setups per game and switch their actual GPU stages.");
    }
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui|{
        for (slot,label) in [(0u8,"LOOK A"),(1u8,"LOOK B")]{
            let active=pair.active==Some(slot);
            let saved=pair.stored(slot);
            let native=pair.slot(slot).is_some_and(|s|s.native.is_some());
            let button=egui::Button::new(
                egui::RichText::new(format!("{}  {}",label,
                    if active{"● ACTIVE"}else if native{"Native"}else if saved{"Legacy"}else{"Empty"}))
                    .size(12.0).color(if active{WHITE}else{MUTED})
            ).fill(if active{Color32::from_rgb(100,26,52)}else{CARD})
                .stroke(Stroke::new(1.0_f32,if active{RED}else{BORDER}))
                .min_size(vec2(if compact{115.0}else{156.0},33.0));
            if ui.add_enabled(saved,button).clicked(){
                app.load_look_slot(slot);
            }
        }
    });
    ui.add_space(5.0);
    ui.horizontal_wrapped(|ui|{
        if outline_button(ui,"Save A"){app.save_look_slot(0);}
        if outline_button(ui,"Save B"){app.save_look_slot(1);}
        if ui.add_enabled(pair.stored(0)&&pair.stored(1),
            egui::Button::new("Switch A / B")
                .fill(Color32::from_rgb(98,30,51))
                .stroke(Stroke::new(1.0_f32,RED))).clicked(){
            app.toggle_look_comparison();
        }
    });
    if !compact{
        caption(ui,"Native slots preserve ordered effects, strengths, lighting and quality. Old legacy slots do not contain native effects; save again to upgrade. No game or display resolution changes.");
    }
}
fn timer_controls(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    ui.horizontal_wrapped(|ui|{
        heading(ui,"GAME STOPWATCH");
        let clock=crate::gametimer::RunTimer::fmt_elapsed(app.game_timer.elapsed());
        ui.colored_label(
            if app.game_timer.running(){Color32::from_rgb(143,223,158)}else{WHITE},
            egui::RichText::new(clock).monospace().size(18.0)
        );
    });
    if !compact{
        caption(ui,"A real monotonic timer for bosses, routes, challenges and farming. No modification of game time.");
    }
    ui.add_space(7.0);
    ui.horizontal_wrapped(|ui|{
        if red_button(ui,if app.game_timer.running(){"Pause"}else if app.game_timer.elapsed().as_nanos()==0{
            "Start timer"
        }else{"Resume timer"}){app.toggle_game_timer();}
        if ui.add_enabled(app.game_timer.running(),egui::Button::new("Mark split")
            .fill(CARD).stroke(Stroke::new(1.0_f32,BORDER))).clicked(){
            app.add_game_split();
        }
        if outline_button(ui,"Reset"){app.game_timer.reset();app.toast("Game stopwatch reset");}
    });
    if !compact{
        ui.add(egui::TextEdit::singleline(&mut app.split_label)
            .hint_text("Optional split name: boss, quest, lap ...")
            .desired_width(ui.available_width().min(420.0)));
    }
    let count=app.game_timer.splits.len();
    if count>0{
        if let Some(last)=app.game_timer.splits.last(){
            caption(ui,&format!("Latest: {} · {}  · total {} splits",
                last.label,
                crate::gametimer::RunTimer::fmt_elapsed(last.elapsed),count));
        }
        if !compact{
            for (i,s) in app.game_timer.splits.iter().enumerate().rev().take(6){
                caption(ui,&format!("{}    {}   +{}",trim(&s.label,45),crate::gametimer::RunTimer::fmt_elapsed(s.elapsed),crate::gametimer::RunTimer::fmt_elapsed(app.game_timer.split_delta(i).unwrap_or_default())));
            }
        }
        if !compact{
            if let Some((i,d))=app.game_timer.best_split(){caption(ui,&format!("Fastest segment: #{} / {}",i+1,crate::gametimer::RunTimer::fmt_elapsed(d)));}
            if ui.add_enabled(!app.split_label.trim().is_empty(),egui::Button::new("Rename last split")).clicked(){app.game_timer.rename_split(count-1,&app.split_label);}
            if outline_button(ui,"Undo last split"){app.game_timer.undo_split();}
        }
        if outline_button(ui,"Export split times"){
            match app.game_timer.export(){
                Ok(path)=>app.toast(format!("Game timer exported: {}",path.display())),
                Err(error)=>app.toast(format!("Export failed: {error}"))
            }
        }
    }else{caption(ui,"No splits yet · Ctrl+Shift+T start/pause, Ctrl+Shift+M mark split");}
}
fn timer_panel(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        timer_controls(app,ui,compact);
    });
}

// Game-only diagnostic from real PresentMon ETW; never invent FPS or game lag.
fn stutter_scout_controls(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    heading(ui,"STUTTER SCOUT · LIVE ETW");
    ui.horizontal_wrapped(|ui|{
        if ui.checkbox(&mut app.settings.stutter_scout_enabled,"Warn on frametime spikes").changed(){
            app.save();
        }
        if ui.add(egui::Slider::new(&mut app.settings.stutter_threshold_ms,20.0..=200.0)
            .suffix(" ms").text("Threshold")).changed(){
            app.save();
        }
    });
    if app.fps.snapshot().is_some(){
        let count=app.fps.frames.iter().rev().take(600).filter(|f|f.ms.is_finite()&&(app.settings.stutter_threshold_ms..=1000.).contains(&f.ms)).count();
        caption(ui,&format!("{} threshold crossings in last {} measured frames",count,app.fps.frames.len().min(600)));
    }else{caption(ui,"Waiting for fresh measured game frames");}
    if !compact{
        caption(ui,"Uses only real ETW presents; alerts no more often than once per 18 seconds. Default OFF.");
        caption(ui,"No framerate cap, game patch, GPU overclock or automatic Windows priority change.");
    }
}
fn game_note_controls(app:&mut BoostApp,ui:&mut Ui){
    heading(ui,"QUICK QUEST NOTES");
    caption(ui,"Local text for selected game · no game-memory reading");
    ui.add(egui::TextEdit::multiline(&mut app.game_note_edit)
        .desired_rows(2).desired_width(ui.available_width().min(440.0))
        .hint_text("Quest item, boss route, crafting list ..."));
    if outline_button(ui,"Save note"){
        let content=app.game_note_edit.chars().take(4000).collect::<String>();
        let key=app.profile_key();
        if content.trim().is_empty(){app.settings.game_notes.remove(&key);}
        else{app.settings.game_notes.insert(key,content);}
        app.save();
        app.toast("Quest note saved for selected game");
    }
}
