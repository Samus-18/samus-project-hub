use crate::engine::{EFFECTS, SCALERS, find_magpie_exe};

fn select_gpu_options(app:&mut BoostApp,ui:&mut Ui) {
    let mut save=false;
    ui.horizontal_wrapped(|ui|{
        caption(ui,"WINDOWED VISUAL STACK (LEGACY)");
        egui::ComboBox::from_id_salt("horizon-live-stack")
            .width(245.0)
            .selected_text(crate::stacks::STACKS[app.settings.live_stack].label)
            .show_ui(ui,|ui|{
                for (i,preset) in crate::stacks::STACKS.iter().enumerate(){
                    if ui.selectable_value(&mut app.settings.live_stack,i,preset.label).changed(){
                        save=true;
                    }
                }
            });
    });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui|{
        caption(ui,"WINDOWED UPSCALER (LEGACY)");
        egui::ComboBox::from_id_salt("horizon-live-scaler")
            .width(245.0).selected_text(SCALERS[app.settings.live_scaler])
            .show_ui(ui,|ui|{
                for (i,s) in SCALERS.iter().enumerate(){
                    if ui.selectable_value(&mut app.settings.live_scaler,i,*s).changed(){save=true;}
                }
            });
    });
    ui.add_space(7.0);
    egui::CollapsingHeader::new("LEGACY WINDOWED CAPTURE · WGC / DXGI / GDI")
        .default_open(false).show(ui,|ui|{
            ui.horizontal_wrapped(|ui|{
                caption(ui,"CAPTURE METHOD");
                egui::ComboBox::from_id_salt("horizon-live-capture")
                    .width(260.0)
                    .selected_text(crate::engine::CAPTURE_METHODS[app.settings.live_capture_method])
                    .show_ui(ui,|ui|{
                        for (i,label) in crate::engine::CAPTURE_METHODS.iter().enumerate(){
                            if ui.selectable_value(&mut app.settings.live_capture_method,i,*label).changed(){save=true;}
                        }
                    });
            });
            caption(ui,"Magpie fallback can process windowed games only. No window-to-fullscreen conversion is offered.");
            caption(ui,"The new native DXGI/D3D11 shader backend is separate and is not yet presented over games.");
        });
    ui.add_space(7.0);
    ui.horizontal_wrapped(|ui|{
        caption(ui,if app.settings.live_stack==0{
            "WINDOWED FILTER (ACTIVE)"
        }else{"WINDOWED FILTER (REPLACE STACK)"});
        egui::ComboBox::from_id_salt("horizon-live-filter")
            .width(260.0).selected_text(EFFECTS[app.settings.live_preset].label)
            .show_ui(ui,|ui|{
                for (i,filter) in EFFECTS.iter().enumerate(){
                    if ui.selectable_value(&mut app.settings.live_preset,i,filter.label).changed(){
                        app.settings.live_stack=0;
                        save=true;
                    }
                }
            });
    });
    ui.add_space(5.0);
    if !app.settings.favorite_effects.is_empty(){
        let favorites=app.settings.favorite_effects.clone();
        ui.horizontal_wrapped(|ui|{
            caption(ui,"PINNED FILTERS");
            for index in favorites.into_iter().take(6){
                if index>=EFFECTS.len(){continue;}
                let active=app.settings.live_stack==0&&app.settings.live_preset==index;
                if ui.selectable_label(active,trim(EFFECTS[index].label,17)).clicked(){
                    app.settings.live_preset=index;
                    app.settings.live_stack=0;
                    save=true;
                }
            }
        });
    }
    ui.add_space(5.0);
    ui.horizontal(|ui|{
        caption(ui,"FILTER STRENGTH");
        if ui.add(egui::Slider::new(&mut app.settings.live_strength,0.0..=1.0)
            .fixed_decimals(2)).changed(){save=true;}
    });
    ui.add_space(4.0);
    let chain=crate::engine::selected_effect_chain(
        app.settings.live_scaler,app.settings.live_preset,
        app.settings.live_stack,&app.settings.custom_stages);
    egui::CollapsingHeader::new(format!("GPU PASS CHAIN · {} actual stages",chain.len()))
        .default_open(false).show(ui,|ui|{
            for (i,name) in chain.iter().enumerate(){
                caption(ui,&format!("{:02}  {}",i+1,name.replace('\\', " / ")));
            }
            caption(ui,"These names are serialized to Magpie. A running process does not prove visible capture.");
        });
    caption(ui,if app.settings.live_stack==0{
        EFFECTS[app.settings.live_preset].desc
    }else{crate::stacks::STACKS[app.settings.live_stack].desc});
    if app.settings.live_stack!=0{
        caption(ui,"Visual Stack overrides the single GPU Filter selection; selecting another filter switches to Single.");
    }
    if save {
        app.on_gpu_filter_changed();
    }
}
fn gpu_status(app:&mut BoostApp,ui:&mut Ui) {
    if find_magpie_exe().is_none(){
        ui.colored_label(RED,"External GPU engine missing from the release/engines/Magpie folder");
        return;
    }
    let running=app.gpu_backend.running();
    let fullscreen=app.telem.active_game.as_ref().is_some_and(|g|g.fullscreen_like);
    let missing=app.telem.active_game.is_none();
    if fullscreen&&!running{
        ui.colored_label(Color32::from_rgb(243,182,108),
            "Fullscreen-like game · legacy scaling disabled.");
        caption(ui,"Native DXGI GPU capture + shaders: verified on RX 580.");
        caption(ui,"Output over the exclusive game: not implemented yet.");
        return;
    }
    if missing&&!running{
        ui.colored_label(Color32::from_rgb(240,174,109),
            "Choose a game for the optional legacy windowed renderer.");
        if red_button(ui,"Choose running game"){
            if app.overlay_open {
                app.overlay_section=Page::Library;
                app.overlay_expanded=true;
            }else{
                app.page=Page::Library;
            }
        }
        return;
    }
    ui.horizontal_wrapped(|ui|{
        let status=if app.pending_gpu_refresh.is_some(){
            "Reinitializing GPU filter..."
        }else if running {
            if let Some(game)=app.telem.active_game.as_ref(){
                if crate::win::magpie_scaling_info(game.pid).is_some(){
                    "Scaling window confirmed for selected game · pixel quality unverified"
                }else{
                    "Magpie process open · waiting for selected game to be foreground"
                }
            }else{"Magpie open · no game selected"}
        }else{"Legacy windowed renderer ready"};
        ui.colored_label(if running{Color32::from_rgb(155,222,169)}else{MUTED},status);
        if running {
            if red_button(ui,"Stop legacy renderer"){app.stop_live_gpu();}
        }else if red_button(ui,"Start legacy windowed renderer"){app.start_live_gpu();}
    });
    if running {
        if let Some(g)=app.telem.active_game.as_ref(){
            if let Some(info)=crate::win::magpie_scaling_info(g.pid){
                caption(ui,if info.windowed{
                    "VERIFIED OUTPUT: external scaled window exists for this game's PID."
                }else{
                    "VERIFIED OUTPUT: fullscreen external Magpie window exists for this game's PID."
                });
                caption(ui,"This confirms Magpie output, not measured image sharpness or true game FPS.");
            }
        }
        caption(ui,&format!("Configured: {} · {}",
            crate::engine::CAPTURE_METHODS[app.settings.live_capture_method],
            crate::engine::OUTPUT_MODES[app.settings.live_output_mode]));
    }
    if let Some(err)=app.gpu_backend.last_error.as_deref(){caption(ui,err);}
}
fn scaling_page(app:&mut BoostApp,ui:&mut Ui) {
    section(ui,"Scaling","Native DXGI GPU pipeline in development · legacy windowed filters separate");
    frame(PANEL).show(ui,|ui|{
        heading(ui,"OPTIONAL LEGACY WINDOWED RENDERING");
        ui.add_space(7.0);
        if app.telem.active_game.is_some() {
            let title=app.telem.active_game.as_ref().unwrap().title.clone();
            ui.colored_label(Color32::from_rgb(126,232,150),
                format!("● Detected window: {title}"));
        } else {
            caption(ui,"Open a supported windowed game first. Auto-detection selects its EXE and window class.");
        }
        ui.add_space(9.0);
        select_gpu_options(app,ui);
        ui.add_space(13.0);
        gpu_status(app,ui);
        ui.add_space(10.0);
        caption(ui,"Magpie 0.13.0 / Direct3D 11 pipeline runs as an independent GPLv3 process.");
        caption(ui,"Ctrl+Shift+G toggles ONLY legacy windowed processing; it does not activate fullscreen filters.");
        caption(ui,"Native RX 580 DXGI capture and D3D11 compute shaders are separate; fullscreen presentation is unfinished.");
    });
    ui.add_space(12.0);
    egui::CollapsingHeader::new("Existing AMD driver tools · already active on this RX 580")
        .default_open(false).show(ui,|ui|driver_panel(app,ui,false));
    ui.add_space(12.0);
    egui::CollapsingHeader::new("Effect reference · scaling, sharpening and upscalers")
        .id_salt("scaling-effect-reference").default_open(false).show(ui,|ui|{
        frame(PANEL).show(ui,|ui|{
            heading(ui,"EFFECT LIBRARY");
            caption(ui,"These are real GPU shaders, not ordinary Windows desktop color-only adjustments.");
            ui.add_space(11.0);
            let desc = [
                ("FSR 1","Edge-aware scaling + RCAS spatial sharpen"),
                ("NVIDIA NIS","Vendor-neutral open sharpening + scaling"),
                ("Clarity+","Original localized microcontrast filter"),
                ("Smart Sharpen+","Original high-pass sharpen with halo limiter"),
                ("Soft Bloom","Original multi-sample highlight glow"),
                ("SMAA / FXAA","Subpixel and approximate anti-aliasing"),
                ("Snapdragon SGSR","Fast edge-aware upscaling for RX 580"),
                ("Anime4K 3D","Stylized and anime-game enhancement · GPU heavy"),
                ("FSRCNNX","Neural super-resolution shader · GPU heavy"),
                ("CuNNy Veryfast","2x lightweight neural upscaler · GPU costly"),
            ];
            for chunk in desc.chunks(3){
                ui.columns(3,|cols|{
                    for (i,(name,detail)) in chunk.iter().enumerate(){
                        frame(CARD).show(&mut cols[i],|ui|{
                            ui.label(RichText::new(*name).color(WHITE).strong());
                            caption(ui,detail);
                        });
                    }
                });
            }
        });
        });
    ui.add_space(12.0);
    frame(Color32::from_rgb(43,25,39)).show(ui,|ui|{
        caption(ui,"The external renderer does not alter game EXE files and doesn't require game hooks.");
        caption(ui,"This renderer adds processing overhead; it does not guarantee increased FPS or low input lag.");
    });
}
