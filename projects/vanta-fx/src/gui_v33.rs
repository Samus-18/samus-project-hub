// New independent functional tools in 033.
fn affinity_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"CPU AFFINITY / LOGICAL PROCESSORS");
        caption(ui,"Manual Windows scheduler control for a pinned game. All originally allowed CPUs is normally best; restricting cores may reduce FPS. No automatic affinity changes.");
        let selected=app.telem.active_game.as_ref().filter(|g|Some(g.pid)==app.telem.selected_pid).map(|g|g.pid);
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Pin game in Library"){app.page=Page::Library;}
            if ui.add_enabled(selected.is_some()&&!app.cpu_affinity.active(),egui::Button::new("Read allowed CPUs")).clicked(){
                if let Some(pid)=selected{
                    match crate::affinity::read(pid){
                        Ok(mask)=>app.affinity_draft=Some((pid,mask,mask)),
                        Err(e)=>app.toast(e)
                    }
                }
            }
        });
        if let Some((pid,original,mut chosen))=app.affinity_draft{
            if Some(pid)==selected{
                caption(ui,&format!("PID {pid}: {} allowed logical CPUs / {} selected",original.count_ones(),chosen.count_ones()));
                ui.add_enabled_ui(!app.cpu_affinity.active(),|ui|{
                    ui.horizontal_wrapped(|ui|{
                        for i in 0..usize::BITS{
                            let bit=1usize<<i;
                            if original&bit!=0{
                                let on=chosen&bit!=0;
                                if ui.selectable_label(on,format!("CPU {i:02}")).clicked(){chosen^=bit;}
                            }
                        }
                    });
                    if outline_button(ui,"Select all originally allowed"){chosen=original;}
                    if ui.add_enabled(crate::affinity::validate(chosen,original)&&chosen!=original,egui::Button::new("Apply selected affinity")).clicked(){
                        match app.cpu_affinity.apply(pid,chosen,original){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}
                    }
                });
                app.affinity_draft=Some((pid,original,chosen));
            }
        }else{caption(ui,"Read the selected process before choosing CPUs.");}
        if app.cpu_affinity.active(){
            ui.colored_label(V25_MINT,"Manual affinity active");
            if red_button(ui,"Restore original CPU affinity"){
                match app.cpu_affinity.restore(){Ok(msg)=>app.toast(msg),Err(e)=>app.toast(e)}
            }
        }
        caption(ui,"Normal exit, game switch and emergency stop restore the original mask if it was not changed externally. Windows can refuse access. No performance gain is guaranteed.");
    });
}
fn shortcut_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"GLOBAL SHORTCUTS");
  caption(ui,"Only Alt+Z is bound by default. All actions below can be assigned individually. Ctrl+Shift+F12 is always emergency stop; F12 cancels running macros.");
  ui.horizontal_wrapped(|ui|{
   if outline_button(ui,"Reset: Alt+Z only"){app.settings.global_shortcuts=Default::default();app.save();}
   if outline_button(ui,"Disable all optional shortcuts"){app.settings.global_shortcuts.entries.clear();app.save();}
   if red_button(ui,"Emergency stop"){app.emergency_stop();}
  });
  if ui.checkbox(&mut app.settings.hud_f12,"Optional bare F12 toggles HUD when no macro is armed").changed(){app.save();}
  let mut pending:Option<(&str,Option<crate::shortcut_registry::Chord>)>=None;
  for (action,description) in crate::shortcut_registry::ACTIONS{
   ui.push_id(action,|ui|{
    let previous=app.settings.global_shortcuts.get(action);
    let mut active=previous.is_some();
    let mut bind=previous.unwrap_or(crate::shortcut_registry::Chord{key:0x55,mods:6});
    ui.horizontal_wrapped(|ui|{
     if ui.checkbox(&mut active,description).changed(){pending=Some((action,active.then_some(bind)));}
     if active{
      let prior=bind;
      egui::ComboBox::from_id_salt("mods").selected_text(crate::shortcut_registry::MODIFIERS.iter().find(|m|m.1==bind.mods).map(|m|m.0).unwrap_or("?")).show_ui(ui,|ui|{
       for (label,m) in crate::shortcut_registry::MODIFIERS{ui.selectable_value(&mut bind.mods,m,label);}
      });
      egui::ComboBox::from_id_salt("key").selected_text(if bind.key>=0x70{format!("F{}",bind.key-0x6f)}else{char::from_u32(u32::from(bind.key)).unwrap_or('?').to_string()}).show_ui(ui,|ui|{
       for code in 0x41u16..=0x5a{ui.selectable_value(&mut bind.key,code,char::from_u32(u32::from(code)).unwrap().to_string());}
       for code in 0x70u16..=0x7a{ui.selectable_value(&mut bind.key,code,format!("F{}",code-0x6f));}
      });
      if bind!=prior {pending=Some((action,Some(bind)));}
     }else{ui.weak("Unbound");}
    });
   });
  }
  if let Some((action,chosen))=pending{
   let mut bank=app.settings.global_shortcuts.clone();
   match bank.assign(action,chosen){
    Ok(())=>{app.settings.global_shortcuts=bank;app.macro_armed_pid=None;app.macro_runner.cancel();app.save();},
    Err(e)=>app.toast(e)
   }
  }
  caption(ui,"No keyboard hooks or key history. Hotkeys are exact modifier matches; unavailable when other apps or Windows consume them. Unbound = no global polling.");
 });
}
fn view_tools_panel(app:&mut BoostApp,ui:&mut Ui){
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"VIEW TOOLS / NATIVE GPU");
        caption(ui,"Choose a single native effect and save it as this game's custom stack. Existing stack is replaced only when you click a choice. Open Stack Builder to compose effects.");
        let entries=[
            (39usize,"Adaptive detail","Contrast-gated sharpening, original"),
            (40,"Edge smoothing","Directional SDR antialiasing, original"),
            (41,"Local exposure","Local 2D brightness adaptation"),
            (42,"Center magnifier 2x","Bilinear center crop, own output"),
            (43,"Center magnifier 4x","Accessibility zoom, own output"),
            (44,"Warm display","Reduced blue channel, SDR"),
            (45,"False color","Luminance heatmap, not real thermal sensing"),
            (46,"Highlight roll-off","SDR highlights, not HDR"),
            (47,"Luminance detail","Bounded luma sharpen"),
            (48,"Barrel lens","Subtle image distortion"),
            (49,"Film grain","Static fine film texture"),
            (50,"Histogram exposure","Global 64-bin GPU luminance metering; SDR only"),
            (52,"Linear contrast","Linear-light SDR contrast"),
            (53,"Reinhard-style","SDR shoulder curve"),
            (54,"Edge protection","Detail enhancement gated by strong edges"),
            (55,"Gradient deband","Local smooth-region filtering"),
            (56,"Highlight diffusion","Restrained 2D highlight glow"),
            (57,"Warm/cool separation","Selective luminance-based tint"),
            (58,"Ink contour","Original soft comic-style outline"),
            (59,"Vignette","Soft lens falloff"),
            (60,"Relief contact shade","Color-only local occlusion estimate; no real depth"),
            (64,"FXAA 3.11","Reference quality 12 with RGB luma"),(65,"LumaSharpen 9-tap","Bounded Gaussian luminance sharpening"),(66,"ACES-style SDR","Filmic shoulder; not HDR")
        ];
        for (id,name,description) in entries{
            frame(CARD).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                ui.horizontal_wrapped(|ui|{
                    ui.label(RichText::new(name).strong().color(WHITE));
                    caption(ui,description);
                    if outline_button(ui,"Select effect"){app.apply_custom_native_stack(vec![id]);}
                });
            });
            ui.add_space(4.0);
        }
        ui.horizontal_wrapped(|ui|{
            if outline_button(ui,"Restore standard look"){app.select_native_preset(app.settings.native_preset);}
            if outline_button(ui,"Stack Builder"){app.page=Page::Filters;app.filter_view=3;}
        });
        v22_native_toggles(app,ui);
        caption(ui,"No game FOV, no aim automation, no true depth/HDR. Zoom is a post-process on Horizon's captured output; protected exclusive fullscreen can hide it.");
    });
}
