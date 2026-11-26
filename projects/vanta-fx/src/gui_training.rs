// Independent game practice, real local reaction time and manually entered
// clear/fail streaks. No reads of online games or synthetic winning stats.
fn training_controls(app:&mut BoostApp,ui:&mut Ui){
    let key=app.profile_key();
    let mut record=app.settings.training_profiles.get(&key).cloned().unwrap_or_default();
    let mut changed=false;
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"REACTION TEST · LOCAL UI ONLY");
        caption(ui,"Wait for green, then click. Timing uses a monotonic clock and the actual visible UI signal; not in-game input latency.");
        if !ui.ctx().input(|i|i.focused)&&app.reaction.ongoing(){app.reaction.reset();}
        app.reaction.tick();
        let phase=app.reaction.phase();
        let (label,color)=match phase{
            crate::training::Phase::Idle=>("START REACTION TEST",Color32::from_rgb(97,33,66)),
            crate::training::Phase::Waiting=>("WAIT FOR GREEN",Color32::from_rgb(109,69,31)),
            crate::training::Phase::Ready=>("CLICK NOW!",Color32::from_rgb(28,118,82)),
            crate::training::Phase::TooEarly=>("TOO EARLY · RETRY",Color32::from_rgb(120,35,48)),
            crate::training::Phase::Result(_)=>("TRY ANOTHER ROUND",Color32::from_rgb(76,40,94)),
        };
        let width=ui.available_width().min(1050.0);
        if ui.add(egui::Button::new(RichText::new(label).size(19.0).strong().color(WHITE))
            .fill(color).stroke(Stroke::new(1.0_f32,Color32::from_rgb(159,67,94)))
            .min_size(vec2(width,94.0))).clicked(){
            match phase{
                crate::training::Phase::Waiting|crate::training::Phase::Ready=>{
                    if let Some(ms)=app.reaction.tap(){
                        record.record_reaction(ms);changed=true;
                    }
                },
                _=>app.reaction.start()
            }
        }
        if app.reaction.ongoing(){
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        }
        if let crate::training::Phase::Result(ms)=app.reaction.phase(){
            ui.colored_label(Color32::from_rgb(143,225,165),
                RichText::new(format!("REACTION: {ms} ms")).size(19.0).strong());
        }else if app.reaction.phase()==crate::training::Phase::TooEarly{
            ui.colored_label(RED,"False start. Wait until the button turns green.");
        }
        let best=record.best_reaction().map(|x|format!("{x} ms")).unwrap_or("—".into());
        let avg=record.avg_reaction().map(|x|format!("{x} ms")).unwrap_or("—".into());
        ui.horizontal_wrapped(|ui|{
            mini_metric(ui,180.0,"BEST RESPONSE",&best,Color32::from_rgb(138,229,166));
            mini_metric(ui,180.0,"AVERAGE",&avg,WHITE);
            mini_metric(ui,180.0,"RECORDED ATTEMPTS",&record.reactions_ms.len().to_string(),
                Color32::from_rgb(210,150,239));
        });
    });
    ui.add_space(11.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"PRACTICE STREAK · PER GAME");
        caption(ui,"Manual clear/fail counter for boss attempts, quests and practice runs; no automatic Roblox progress detection.");
        ui.horizontal_wrapped(|ui|{
            mini_metric(ui,157.0,"CLEARED",&record.clears.to_string(),Color32::from_rgb(141,227,166));
            mini_metric(ui,157.0,"FAILED",&record.fails.to_string(),RED);
            mini_metric(ui,157.0,"CURRENT STREAK",&record.streak.to_string(),WHITE);
            mini_metric(ui,157.0,"BEST STREAK",&record.best_streak.to_string(),Color32::from_rgb(232,184,99));
        });
        ui.add_space(9.0);
        ui.horizontal_wrapped(|ui|{
            if red_button(ui,"Clear attempt +1"){record.clear();changed=true;}
            if outline_button(ui,"Fail attempt +1"){record.fail();changed=true;}
            if !app.training_reset_confirm{
                if outline_button(ui,"Clear history..."){app.training_reset_confirm=true;}
            }else{
                if red_button(ui,"CONFIRM RESET"){
                    record=crate::training::TrainingProgress::default();
                    app.reaction.reset();changed=true;
                    app.training_reset_confirm=false;
                }
                if outline_button(ui,"Cancel"){app.training_reset_confirm=false;}
            }
        });
    });
    if changed{
        record.sanitize();
        app.settings.training_profiles.insert(key,record);
        app.save();
    }
}
