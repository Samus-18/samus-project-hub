// Compact graphical native-D3D11 studio. No game injection or fake runtime claims.
#[derive(Clone)]
struct StudioUi{draft:crate::studio_draft::Draft,new_stage:usize,category:usize,search:String}
fn rm_studio(a:&mut BoostApp,ui:&mut Ui){
 let key=a.profile_key();
 egui::CollapsingHeader::new(rm(a,"BUILD YOUR OWN GPU FILTER • 8 layers","VLASTNÍ GPU FILTR • 8 vrstev"))
 .id_salt(("rm-studio",key.as_str())).default_open(false).show(ui,|ui|{
  let id=egui::Id::new(("rm-studio-state",key.as_str()));
  let mut state=ui.data(|d|d.get_temp::<StudioUi>(id))
   .unwrap_or_else(||StudioUi{draft:crate::studio_draft::Draft::from_settings(&a.settings,key.clone()),
    new_stage:2,category:4,search:String::new()});
  // Switching game resets the draft, not other games' saved configurations.
  if state.draft.key!=key{
   state=StudioUi{draft:crate::studio_draft::Draft::from_settings(&a.settings,key.clone()),
    new_stage:2,category:4,search:String::new()};
  }
  frame(PANEL).show(ui,|ui|{
   ui.set_min_width(ui.available_width());
   let count=state.draft.slots.len();
   ui.horizontal_wrapped(|ui|{
    ui.label(RichText::new(format!("{} / 8",count)).strong().size(17.));
    if a.settings.native_stack_profiles.contains_key(&key){ui.colored_label(V25_MINT,rm(a,"SAVED STACK","ULOŽENÁ SADA"));}
    else{ui.label(rm(a,"Draft (not active)","Návrh (není aktivní)"));}
    if a.native_mirror.as_ref().is_some_and(|w|w.running()){ui.colored_label(V25_MINT,rm(a,"GPU OUTPUT RUNNING","GPU VÝSTUP BĚŽÍ"));}
   });
   caption(ui,rm(a,"Combine built-in D3D11 effects, reorder and adjust the intensity of every occurrence. Nothing changes until Apply.","Kombinuj D3D11 efekty, měň pořadí a intenzitu každé vrstvy. Bez kliknutí Použít se nic nemění."));
   ui.horizontal_wrapped(|ui|{
    if ui.add_enabled(state.draft.can_undo(),egui::Button::new(rm(a,"Undo edit","Zpět"))).clicked(){state.draft.undo();}
    if ui.add_enabled(state.draft.can_redo(),egui::Button::new(rm(a,"Redo edit","Vpřed"))).clicked(){state.draft.redo();}
    if ui.button(rm(a,"Reload current","Načíst aktuální")).clicked(){
     state.draft=crate::studio_draft::Draft::from_settings(&a.settings,key.clone());
    }
   });
   for (i,slot) in state.draft.slots.clone().iter().enumerate(){
    let name=crate::native_filter::PROGRAMS.get(slot.id).map_or("Unknown GPU shader",|x|x.0);
    let group=crate::layer_mix::group(slot.id);
    let color=match group{0=>Color32::from_rgb(154,101,192),1=>Color32::from_rgb(236,151,81),2=>V25_MINT,_=>Color32::from_rgb(105,152,224)};
    frame(CARD).show(ui,|ui|{
     ui.horizontal_wrapped(|ui|{
      ui.colored_label(color,RichText::new(format!("{:02}",i+1)).strong().size(14.));
      ui.label(RichText::new(name).strong());
      ui.add_space(2.);
      if ui.add_enabled(i>0,egui::Button::new("↑")).on_hover_text(rm(a,"Move up","Posunout nahoru")).clicked(){let _=state.draft.move_by(i,-1);}
      if ui.add_enabled(i+1<count,egui::Button::new("↓")).on_hover_text(rm(a,"Move down","Posunout dolů")).clicked(){let _=state.draft.move_by(i,1);}
      if ui.small_button("×").on_hover_text(rm(a,"Remove effect","Odebrat efekt")).clicked(){let _=state.draft.remove(i);}
     });
     let mut strength=slot.weight;
     ui.horizontal_wrapped(|ui|{
      if ui.add(egui::Slider::new(&mut strength,0.0..=1.).text(rm(a,"Strength","Síla")).fixed_decimals(2)).changed(){
       let _=state.draft.strength(i,strength);
      }
      if ui.small_button(if slot.weight==0.{rm(a,"Enable","Zapnout")}else{rm(a,"Bypass","Vypnout")}).clicked(){
       let _=state.draft.strength(i,if slot.weight==0.{1.}else{0.});
      }
     });
    });
    ui.add_space(3.);
   }
   if count==0{ui.colored_label(RED,rm(a,"Add at least one effect to apply a stack.","Pro použití sady přidej aspoň jeden efekt."));}
   let labels=[(0,"Colors","Barvy"),(1,"Lighting","Světla"),(2,"Details","Detaily"),(3,"Lens / style","Čočka / styl"),(4,"All","Vše")];
   ui.horizontal_wrapped(|ui|{
    for(i,en,cs)in labels{ui.selectable_value(&mut state.category,i,rm(a,en,cs));}
   });
   ui.add(egui::TextEdit::singleline(&mut state.search).hint_text(rm(a,"Find a GPU effect...","Najít GPU efekt...")).char_limit(80));
   let needle=state.search.to_lowercase();
   let candidates=crate::native_filter::PROGRAMS.iter().enumerate()
    .filter(|(i,p)|crate::native_stack::valid_stage(*i)&&(state.category==4||crate::layer_mix::group(*i)==state.category)&&
     (needle.is_empty()||p.0.to_lowercase().contains(&needle)))
    .map(|(i,p)|(i,p.0)).collect::<Vec<_>>();
   if !candidates.iter().any(|x|x.0==state.new_stage){
    if let Some((i,_))=candidates.first(){state.new_stage=*i;}
   }
   ui.horizontal_wrapped(|ui|{
    egui::ComboBox::from_id_salt("studio-select-shader")
     .width(ui.available_width().min(290.))
     .selected_text(crate::native_filter::PROGRAMS.get(state.new_stage).map_or("Effect",|x|x.0))
     .show_ui(ui,|ui|{
      egui::ScrollArea::vertical().max_height(215.).show(ui,|ui|{
       for &(idx,name)in &candidates{ui.selectable_value(&mut state.new_stage,idx,name);}
      });
     });
    if ui.add_enabled(count<8&&candidates.iter().any(|x|x.0==state.new_stage),egui::Button::new(rm(a,"+ Add effect","+ Přidat efekt"))).clicked(){
     if let Err(e)=state.draft.add(state.new_stage){a.toast(e);}
    }
   });
   let applied=a.settings.native_stack_profiles.get(&key);
   let ids=state.draft.ids();
   let dirty=applied.is_none_or(|p|p!=&ids)||
    state.draft.slots.iter().enumerate().any(|(i,s)|a.settings.lighting.layers.strength_at(&ids,i)!=s.weight);
   ui.horizontal_wrapped(|ui|{
    if ui.add_enabled(a.telem.active_game.is_some()&&state.draft.validate().is_ok()&&!a.is_smoke,
      egui::Button::new(if dirty{rm(a,"APPLY GPU STACK","POUŽÍT GPU SADU")}else{rm(a,"Reapply GPU stack","Znovu použít GPU sadu")})
       .fill(Color32::from_rgb(148,38,65)).min_size(egui::vec2(180.,34.))).clicked(){
      let mut candidate=a.settings.clone();
      match state.draft.commit(&mut candidate){
       Err(e)=>a.toast(e),
       Ok(())=>{
        let tune=candidate.lighting.clone();
        let applied=if let Some(w)=a.native_mirror.as_ref().filter(|w|w.running()){
         w.set_stack(&ids,candidate.native_quality,&tune)
        }else{Ok(())};
        match applied{
         Err(e)=>{a.native_mirror_status=format!("Studio apply rejected: {e}");a.toast(e);},
         Ok(())=>{
          a.settings=candidate;a.save();
          a.native_mirror_status=if a.native_mirror.as_ref().is_some_and(|w|w.running()){
           "Live GPU shader stack applied without restart".into()
          }else{"GPU stack saved; start Horizon output to see the result".into()};
         }
        }
       }
      }
    }
    if ui.add_enabled(a.settings.studio_previous.contains_key(&key)&&!a.is_smoke,
      egui::Button::new(rm(a,"RESTORE PREVIOUS","OBNOVIT PŘEDCHOZÍ"))).clicked(){
      let mut candidate=a.settings.clone();
      match crate::studio_draft::revert(&mut candidate,&key){
       Err(e)=>a.toast(e),
       Ok(())=>{
        let output=if let Some(w)=a.native_mirror.as_ref().filter(|w|w.running()){
         if let Some(stages)=candidate.native_stack_profiles.get(&key){w.set_stack(stages,candidate.native_quality,&candidate.lighting)}
         else{w.set_look(candidate.native_preset,candidate.native_quality,&candidate.lighting)}
        }else{Ok(())};
        match output{
         Err(e)=>a.toast(e),
         Ok(())=>{a.settings=candidate;a.save();state.draft=crate::studio_draft::Draft::from_settings(&a.settings,key.clone());}
        }
       }
      }
    }
   });
   caption(ui,rm(a,"Saves settings for this game only. Shader stages run on Horizon's own output, not inside the game's engine.","Ukládá nastavení pouze pro tuto hru. Efekty běží ve výstupu Horizonu, ne uvnitř enginu hry."));
   egui::CollapsingHeader::new(rm(a,"Export / import a weighted GPU stack (JSON)","Export / import GPU sady s intenzitou (JSON)")).show(ui,|ui|{
    if ui.button(rm(a,"Copy draft JSON","Kopírovat návrh JSON")).clicked(){
     match crate::studio_draft::export(&state.draft.slots){
      Ok(text)=>ui.ctx().copy_text(text),
      Err(e)=>a.toast(e),
     }
    }
    ui.add(egui::TextEdit::multiline(&mut a.stack_share_edit).desired_rows(3).desired_width(ui.available_width()).char_limit(8192));
    if ui.button(rm(a,"Validate & load draft","Ověřit a načíst návrh")).clicked(){
     match crate::studio_draft::import(&a.stack_share_edit){
      Ok(slots)=>{if let Err(e)=state.draft.change(slots){a.toast(e);}},
      Err(e)=>a.toast(e),
     }
    }
    caption(ui,rm(a,"Import only loads a local draft, never executes code or enables effects automatically.","Import jen načte místní návrh, nespouští kód ani automaticky neaktivuje efekty."));
   });
   if let Some(w)=a.native_mirror.as_ref().filter(|w|w.running()){
    let snapshot=w.diagnostics();
    let parts=snapshot.gpu_phase_ms.iter().enumerate().filter_map(|(i,m)|m.map(|v|format!("GPU {}: {:.2} ms",i+1,v))).collect::<Vec<_>>();
    if !parts.is_empty(){ui.colored_label(V25_MINT,parts.join(" • "));}
    caption(ui,rm(a,"GPU timings are sampled output processing phases, not game FPS or an FPS gain prediction.","GPU časy měří fáze zpracování výstupu, nikoli FPS hry ani odhad zvýšení FPS."));
   }
  });
  ui.data_mut(|d|d.insert_temp(id,state));
 });
}
