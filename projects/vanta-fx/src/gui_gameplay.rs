// Optional manual game aids, no game memory reading, automating gameplay,
// key injection, and no false claims about exclusive fullscreen overlays.
fn cooldown_tracker_controls(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    heading(ui,"MANUAL COOLDOWN TRACKER");
    caption(ui,"Start your own ability/potion reminder; actual game state is never read");
    ui.horizontal_wrapped(|ui|{
        ui.add(egui::TextEdit::singleline(&mut app.cooldown_spec.label)
            .desired_width(if compact{145.0}else{200.0})
            .hint_text("Potion / ability"));
        ui.add(egui::Slider::new(&mut app.cooldown_spec.duration_seconds,3..=600)
            .suffix(" s").text("Cooldown"));
    });
    ui.horizontal_wrapped(|ui|{
        if red_button(ui,"Start timer"){
            app.cooldown_spec.sanitize();
            app.cooldown_clock.start();
            let key=app.profile_key();
            app.settings.cooldown_profiles.insert(key,app.cooldown_spec.clone());
            app.save();
        }
        if outline_button(ui,"Stop / reset"){app.cooldown_clock.reset();}
        let text=match app.cooldown_clock.remaining(app.cooldown_spec.duration_seconds){
            Some(t) if t>std::time::Duration::ZERO =>format!("{:.1} s remaining",t.as_secs_f32()),
            Some(_)=>"READY".into(),
            None=>"Not running".into()
        };
        ui.colored_label(
            if app.cooldown_clock.active(app.cooldown_spec.duration_seconds){
                Color32::from_rgb(247,198,113)
            }else{Color32::from_rgb(121,221,161)},text);
    });
    if !compact{
        caption(ui,"Can be used for manual ability cooldowns, crafting, boss routes and potion timing.");
    }
}
fn quest_checklist_controls(app:&mut BoostApp,ui:&mut Ui,compact:bool){
    heading(ui,"QUEST CHECKLIST · PER GAME");
    caption(ui,"Your manually entered objectives; no game scanning or quest-memory access");
    let key=app.profile_key();
    let mut items=app.settings.quest_checklists.get(&key).cloned().unwrap_or_default();
    let mut changed=false;
    let visible=if compact{items.len().min(7)}else{items.len()};
    for task in items.iter_mut().take(visible){
        if ui.checkbox(&mut task.done,&task.label).changed(){changed=true;}
    }
    if visible<items.len(){
        caption(ui,&format!("{} more quests are available in main Performance.",items.len()-visible));
    }
    ui.horizontal_wrapped(|ui|{
        ui.add(egui::TextEdit::singleline(&mut app.quest_task_edit)
            .desired_width(if compact{200.0}else{320.0})
            .hint_text("Add a quest / item / boss goal"));
        let can_add=items.len()<20&&!app.quest_task_edit.trim().is_empty();
        if ui.add_enabled(can_add,egui::Button::new("Add")
            .fill(CARD).stroke(Stroke::new(1.0_f32,BORDER))).clicked(){
            let label=app.quest_task_edit.trim().chars().take(90).collect::<String>();
            if !label.is_empty(){
                items.push(crate::gameplay::QuestTask{label,done:false});
                app.quest_task_edit.clear();
                changed=true;
            }
        }
        if items.iter().any(|item|item.done)&&outline_button(ui,"Clear completed"){
            items.retain(|item|!item.done);
            changed=true;
        }
    });
    if changed{
        crate::gameplay::sanitize_tasks(&mut items);
        if items.is_empty(){app.settings.quest_checklists.remove(&key);}
        else{app.settings.quest_checklists.insert(key,items);}
        app.save();
    }
}
