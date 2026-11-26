// Multi-pass shader stack gallery. Every selected preset is serialized into
// Magpie's *sequential* scalingMode.effects list; it is NOT only a UI theme.
fn visual_stacks_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        heading(ui,&format!("HORIZON PRESET STUDIO · {} LOOKS",crate::stacks::STACKS.len()));
        caption(ui,"44 color/detail looks, clean FPS mode, single filter and a 4-pass custom builder");
        caption(ui,"Original photo-based reference previews. GPU pipeline currently legacy WINDOWED only; native fullscreen work ongoing.");
        ui.add_space(9.0);
        let width=ui.available_width();
        let gap=7.0;
        let columns=if width>=800.0{5}else if width>=560.0{3}else{2};
        let card_w=(width-(columns-1) as f32*gap)/columns as f32;
        ui.horizontal_wrapped(|ui|{
            for (category,label) in ["Featured","Adventure","Cinematic","Tactical","All 47"].iter().enumerate(){
                if ui.selectable_label(app.preset_category==category,*label).clicked(){
                    app.preset_category=category;
                }
            }
        });
        let visible=crate::stacks::preset_category_indices(app.preset_category);
        for chunk in visible.chunks(columns){
            ui.horizontal(|ui|{
                ui.spacing_mut().item_spacing.x=gap;
                for index in chunk.iter().copied(){
                    let stack=&crate::stacks::STACKS[index];
                    let selected=app.settings.live_stack==index;
                    let (rect,resp)=click_surface(ui,card_w,105.0,selected);
                    let p=ui.painter();
                    let thumb=Rect::from_min_size(rect.min+vec2(5.0,5.0),
                        vec2(card_w-10.0,64.0));
                    if let Some(texture)=app.previews.stack_thumbnail(ui.ctx(),index,
                        app.settings.live_preset,&app.settings.custom_stages){
                        p.image(texture.id(),thumb,Rect::from_min_max(
                            pos2(0.0,0.0),pos2(1.0,1.0)),Color32::WHITE);
                    }
                    p.rect_stroke(thumb,5.0,Stroke::new(0.7_f32,BORDER),StrokeKind::Inside);
                    p.text(rect.left_top()+vec2(8.0,74.0),egui::Align2::LEFT_TOP,
                        trim(stack.label,24),FontId::proportional(11.5),
                        if selected{Color32::from_rgb(255,154,173)}else{WHITE});
                    p.text(rect.left_bottom()+vec2(8.0,-5.0),egui::Align2::LEFT_BOTTOM,
                        stack.cost,FontId::proportional(9.6),MUTED);
                    if resp.clicked(){
                        app.settings.live_stack=index;
                        app.on_gpu_filter_changed();
                    }
                }
            });
            ui.add_space(6.0);
        }
        caption(ui,"Legacy presets are real sequential effect chains for windowed games; previews are local CPU references.");
        let stack=&crate::stacks::STACKS[app.settings.live_stack];
        ui.horizontal_wrapped(|ui|{
            ui.colored_label(RED,stack.label);
            caption(ui,stack.desc);
        });
        if app.settings.live_stack==crate::stacks::CUSTOM_INDEX{
            // Native chain editing will be separate; preserve the original legacy custom index.
            ui.add_space(7.0);
            ui.separator();
            heading(ui,"CUSTOM GPU CHAIN BUILDER · 1 TO 4 PASSES");
            caption(ui,"Move ↑ / ↓ to reorder actual shader passes. Each stage costs GPU time; choose a windowed game.");
            let mut changed=false;
            let mut remove=None;
            let mut move_stage=None;
            for i in 0..app.settings.custom_stages.len(){
                let can_up=i>0;
                let can_down=i+1<app.settings.custom_stages.len();
                ui.horizontal_wrapped(|ui|{
                    caption(ui,&format!("#{}",i+1));
                    let stage=&mut app.settings.custom_stages[i];
                    stage.effect=stage.effect.clamp(1,crate::engine::EFFECTS.len()-1);
                    egui::ComboBox::from_id_salt(("hb-custom-stage",i))
                        .width(195.0)
                        .selected_text(crate::engine::EFFECTS[stage.effect].label)
                        .show_ui(ui,|ui|{
                            for effect in 1..crate::engine::EFFECTS.len(){
                                if ui.selectable_value(&mut stage.effect,effect,
                                    crate::engine::EFFECTS[effect].label).changed(){
                                    changed=true;
                                }
                            }
                        });
                    let strength=ui.add(egui::Slider::new(&mut stage.weight,0.0..=1.0)
                        .fixed_decimals(2).text("Stage strength"));
                    if strength.drag_stopped(){changed=true;}
                    if can_up&&outline_button(ui,"↑"){move_stage=Some((i,-1));}
                    if can_down&&outline_button(ui,"↓"){move_stage=Some((i,1));}
                    if app.settings.custom_stages.len()>1&&outline_button(ui,"Remove"){
                        remove=Some(i);
                    }
                });
            }
            if let Some(i)=remove{
                app.settings.custom_stages.remove(i);changed=true;
            }else if let Some((i,direction))=move_stage{
                if crate::stacks::move_custom_stage(&mut app.settings.custom_stages,i,direction){
                    changed=true;
                }
            }
            ui.horizontal_wrapped(|ui|{
                if app.settings.custom_stages.len()<4&&outline_button(ui,"Add shader stage"){
                    app.settings.custom_stages.push(crate::stacks::CustomStage::default());
                    changed=true;
                }
                if outline_button(ui,"Restore custom default"){
                    app.settings.custom_stages=crate::stacks::default_custom();changed=true;
                }
            });
            if changed{app.on_gpu_filter_changed();}
            ui.add_space(6.0);
        }
        ui.horizontal_wrapped(|ui|{
            if red_button(ui,"Activate selected preset"){
                app.activate_gpu_now();
            }
            if app.gpu_backend.running(){
                caption(ui,"Live external renderer active · selecting another preset automatically reloads");
            }else{
                caption(ui,"Click Activate to start real windowed GPU processing");
            }
        });
        if let Some(game)=app.telem.active_game.as_ref(){
            if game.fullscreen_like {
                ui.colored_label(Color32::from_rgb(244,175,106),
                    "Native fullscreen output unfinished; legacy Magpie resizing is disabled for screen-filling games.");
            }
        }
        caption(ui,"FPS Priority minimizes extra shader passes. It cannot increase a game's native rendering FPS by itself.");
    });
}
