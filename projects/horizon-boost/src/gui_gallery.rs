// Interactive filter library with real, local before/after picture previews.
// Original HorizonBoost effects use formula-level CPU references; vendor built-in
// Magpie effects use explicitly labeled approximations, never fake GPU capture.
const GROUPS:[&str;7]=["FEATURED","COLOR","DETAIL","CINEMATIC","UTILITY","FAVORITES","ALL 20"];
const FEATURED:&[usize]=&[1,2,3,5,8,11,14,18];
const COLOR:&[usize]=&[1,4,10,11,13];
const DETAIL:&[usize]=&[2,3,14,15,16];
const CINEMATIC:&[usize]=&[5,6,7,8,9];
const UTILITY:&[usize]=&[0,12,17,18,19];
fn grouped_indices(group:usize,favorites:&[usize])->Vec<usize>{
    match group{
        0=>FEATURED.to_vec(),
        1=>COLOR.to_vec(),2=>DETAIL.to_vec(),3=>CINEMATIC.to_vec(),
        4=>UTILITY.to_vec(),5=>favorites.to_vec(),_ =>(0..EFFECTS.len()).collect()
    }
}
fn filter_gallery(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        let width=ui.available_width();
        ui.horizontal_wrapped(|ui|{
            heading(ui,"FILTER STUDIO");
            caption(ui,"20 legacy GPU effects · genuine photo before/after · no AI pictures");
            ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                if outline_button(ui,"Reset to photo"){
                    app.previews.restore(ui.ctx());
                    app.settings.preview_image_path=None;
                    app.save();
                    app.toast("Original real-photo preview restored");
                }
                if outline_button(ui,"Choose image…"){
                    if let Some(path)=crate::filedialog::select_image(){
                        match app.previews.from_file(ui.ctx(),&path){
                            Ok(())=>{
                                app.settings.preview_image_path=Some(path.to_string_lossy().to_string());
                                app.save();
                                app.toast("Preview photo loaded and remembered");
                            },
                            Err(err)=>app.toast(format!("Preview error: {err}")),
                        }
                    }
                }
            });
        });
        ui.horizontal_wrapped(|ui|{
            caption(ui,"REAL PHOTO");
            for (idx,(label,_)) in crate::preview::BUILTIN_PHOTOS.iter().enumerate(){
                if outline_button(ui,if idx==0{"Rocket photo"}else{"Astronaut photo"}){
                    app.previews.use_photo(ui.ctx(),idx);
                    app.settings.preview_image_path=None;
                    app.save();
                    app.toast(format!("Photo reference: {label}"));
                }
            }
        });
        caption(ui,&format!("Source: {} · Public-domain photographs, original pixels processed locally",app.previews.source_label));
        ui.add_space(5.0);
        ui.horizontal_wrapped(|ui|{
            caption(ui,"FIND AN EFFECT");
            ui.add(egui::TextEdit::singleline(&mut app.filter_search)
                .hint_text("Search by name or purpose...")
                .desired_width(230.0));
            if !app.filter_search.is_empty()&&ui.small_button("Clear").clicked(){
                app.filter_search.clear();
            }
            let current=app.settings.live_preset;
            let saved=app.settings.favorite_effects.contains(&current);
            if outline_button(ui,if saved{"★ Pinned to quick access"}else{"☆ Pin current filter"}){
                app.toggle_favorite_effect(current);
            }
        });
        ui.horizontal_wrapped(|ui|{
            caption(ui,"CHOOSE STANDALONE FILTER");
            let mut selected=app.settings.live_preset;
            egui::ComboBox::from_id_salt("hb-filter-gallery-quick-select")
                .width(240.0)
                .selected_text(EFFECTS[selected].label)
                .show_ui(ui,|ui|{
                    for (i,filter) in EFFECTS.iter().enumerate(){
                        ui.selectable_value(&mut selected,i,filter.label);
                    }
                });
            if selected!=app.settings.live_preset{
                app.settings.live_preset=selected;
                app.settings.live_stack=0;
                app.on_gpu_filter_changed();
            }
        });
        ui.add_space(7.0);
        let index=app.settings.live_preset;
        let intensity=app.settings.live_strength;
        let stack=app.settings.live_stack;
        let is_exact=stack==0&&crate::preview::PreviewBank::exact_reference(index);
        ui.horizontal_wrapped(|ui|{
            ui.colored_label(RED,if stack==0{EFFECTS[index].label.to_owned()}
                else{crate::stacks::STACKS[stack].label.to_owned()});
            caption(ui,if stack>0{
                "Sequential shader stack · local CPU reference preview"
            }else if is_exact{
                "Original HLSL formula · CPU reference preview"
            }else{
                "External Magpie effect · illustrative preview, not pixel-exact"
            });
        });
        caption(ui,if stack==0{EFFECTS[index].desc}
            else{crate::stacks::STACKS[stack].desc});
        ui.add_space(7.0);
        let image_width=((width-20.0)/2.0).min(405.0).max(160.0);
        let image_height=image_width*9.0/16.0;
        let before=app.previews.original().map(|h|h.id());
        let after=app.previews.selected(ui.ctx(),index,intensity,stack,
            &app.settings.custom_stages).map(|h|h.id());
        ui.horizontal(|ui|{
            ui.spacing_mut().item_spacing.x=10.0;
            for (image,caption_text) in [(before,"BEFORE"),(after,"AFTER · REFERENCE")]{
                ui.vertical(|ui|{
                    ui.label(egui::RichText::new(caption_text).size(11.5).color(MUTED));
                    if let Some(texture)=image {
                        let (rect,_)=ui.allocate_exact_size(Vec2::new(image_width,image_height),
                            egui::Sense::hover());
                        let p=ui.painter();
                        p.image(texture,rect,
                            Rect::from_min_max(pos2(0.0,0.0),pos2(1.0,1.0)),Color32::WHITE);
                        p.rect_stroke(rect,8.0,Stroke::new(1.0_f32,BORDER),StrokeKind::Inside);
                    }
                });
            }
        });
        ui.add_space(9.0);
        let mut commit=false;
        ui.horizontal_wrapped(|ui|{
            caption(ui,"FILTER INTENSITY");
            let response=ui.add(egui::Slider::new(&mut app.settings.live_strength,0.0..=1.0)
                .fixed_decimals(2).text("Strength"));
            if response.drag_stopped()||response.lost_focus()&&response.changed(){
                commit=true;
            }
            if red_button(ui,"Activate in game"){
                app.activate_gpu_now();
            }
            if outline_button(ui,"Reset strength"){
                app.settings.live_strength=0.5;
                commit=true;
            }
        });
        if commit{app.on_gpu_filter_changed();}
        caption(ui,"Preview is a local reference simulation, not a capture from your game.");
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui|{
            for (i,label) in GROUPS.iter().enumerate(){
                if ui.selectable_label(app.gallery_category==i,*label).clicked(){
                    app.gallery_category=i;
                }
                ui.add_space(4.0);
            }
        });
        ui.add_space(5.0);
        let query=app.filter_search.trim().to_lowercase();
        let entries:Vec<usize>=grouped_indices(app.gallery_category,&app.settings.favorite_effects)
            .into_iter().filter(|&idx|query.is_empty()||
                EFFECTS[idx].label.to_lowercase().contains(&query)||
                EFFECTS[idx].desc.to_lowercase().contains(&query)).collect();
        if entries.is_empty(){
            caption(ui,if app.gallery_category==5&&query.is_empty(){
                "No favorites yet. Pin any standalone filter to add it to this view."
            }else{"No matching effects. Try a different keyword or category."});
        }
        let column_count=if width>=740.0{4usize}else if width>=460.0{3}else{2};
        let gutter=9.0;
        for chunk in entries.chunks(column_count){
            ui.horizontal(|ui|{
                ui.spacing_mut().item_spacing.x=gutter;
                let card_w=((width-gutter*(column_count-1) as f32)/column_count as f32).max(130.0);
                for idx in chunk{
                    let (rect,resp)=click_surface(ui,card_w,119.0,index==*idx);
                    let p=ui.painter();
                    let image_rect=Rect::from_min_size(
                        rect.min+vec2(6.0,6.0),vec2(card_w-12.0,78.0));
                    if let Some(image)=app.previews.thumbnail(ui.ctx(),*idx){
                        p.image(image.id(),image_rect,
                            Rect::from_min_max(pos2(0.0,0.0),pos2(1.0,1.0)),Color32::WHITE);
                    }
                    p.rect_stroke(image_rect,4.0,Stroke::new(0.7_f32,BORDER),StrokeKind::Inside);
                    if app.settings.favorite_effects.contains(idx){
                        p.text(rect.right_top()+vec2(-10.0,10.0),egui::Align2::RIGHT_TOP,
                            "★",FontId::proportional(15.0),Color32::from_rgb(255,135,165));
                    }
                    p.text(rect.min+vec2(8.0,90.0),egui::Align2::LEFT_TOP,
                        trim(EFFECTS[*idx].label,25),FontId::proportional(11.3),WHITE);
                    p.text(rect.right_bottom()-vec2(7.0,6.0),egui::Align2::RIGHT_BOTTOM,
                        if *idx<=13{"HB"}else{"GPU"},FontId::proportional(8.9),
                        if *idx<=13{Color32::from_rgb(255,122,155)}else{MUTED});
                    if resp.clicked(){
                        app.settings.live_preset=*idx;
                        app.settings.live_stack=0;
                        app.on_gpu_filter_changed();
                    }
                }
            });
            ui.add_space(8.0);
        }
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(6.0);
        gpu_status(app,ui);
        ui.add_space(5.0);
        if outline_button(ui,"Verify all 20 filter files"){
            match crate::engine::verify_effect_files(){
                Ok(count)=>match crate::shadercheck::compile_all(){
                    Ok(compiled)=>app.toast(format!(
                        "{count} filter choices resolved, {compiled} custom HLSL shaders compiled (game rendering not yet tested)"
                    )),
                    Err(error)=>app.toast(format!("HLSL compile failed: {error}")),
                },
                Err(error)=>app.toast(format!("Missing effect file: {error}")),
            }
        }
        caption(ui,"Filter verification checks actual disk assets and HLSL syntax; GPU output still requires a game test.");
        caption(ui,"Spatial Magpie shaders require a supported composited/windowed game.");
        caption(ui,"Native fullscreen RIS/Anti-Lag are separate AMD driver features further below.");
    });
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn groups_cover_all_twenty_effects(){
        assert_eq!(grouped_indices(6,&[]),(0..20).collect::<Vec<_>>());
        assert_eq!(grouped_indices(5,&[2,8]),vec![2,8]);
        for g in 0..7{
            for index in grouped_indices(g,&[2,8]){assert!(index<EFFECTS.len());}
        }
        assert_eq!(FEATURED.len(),8);
    }
}
