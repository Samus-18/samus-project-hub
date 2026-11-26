// Original compact Game Extras and external interpolation laboratory.
// All controls are explicit opt-in and act only on Horizon's own Windows UI.
fn game_extras_page(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Game Tools","Your in-game toolkit · graphics, capture, practice and controls");
    const TABS:[(&str,&str);10]=[
        ("01 HUD","20 reticles · independent per-game click-through Windows overlay"),
        ("02 Timers","Speedrun splits · boss/potion/ability countdowns"),
        ("03 Quests","Per-game checklist, local notes and route reminders"),
        ("04 Training","Reaction tester and manually tracked clear/fail streaks"),
        ("05 Quick Actions","Screenshot, video, scheduling and real frame pacing diagnostics"),
        ("06 Macros","Finite one-shot key sequences for offline games / accessibility"),
        ("07 View Tools","Native magnification and image clarity, optional GPU output"),
        ("08 Reminders","Independent manual timers and local dashboard"),
        ("09 Controller","Read-only XInput buttons, axes, battery and drift"),
        ("10 Session Lab","Explicit measured telemetry, markers and export")
    ];
    studio_tabs(ui,&mut app.extras_view,&TABS);
    ui.add_space(13.0);
    game_extras_body(app,ui);
}
fn game_extras_body(app:&mut BoostApp,ui:&mut Ui){
    match app.extras_view.min(9){
        0=>{
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                crosshair_controls(app,ui);
            });
        },
        1=>{
            timer_panel(app,ui,false);
            ui.add_space(11.0);
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                cooldown_tracker_controls(app,ui,false);
            });
        },
        2=>{
            gameplay_notebook_panel(app,ui);
        },
        3=>{
            training_controls(app,ui);
            if !app.accuracy_window{accuracy_panel(app,ui);}
            if outline_button(ui,if app.accuracy_window{"Close separate practice"}else{"Open separate practice"})&&!app.is_smoke{app.accuracy_window=!app.accuracy_window;if !app.accuracy_window{app.accuracy_trainer.stop();}}
        },
        5=>{window_macro_panel(app,ui);egui::CollapsingHeader::new("Offline game key sequence").show(ui,|ui|{macro_panel(app,ui);});egui::CollapsingHeader::new("Local sequence lab").show(ui,|ui|{local_sequence_panel(app,ui);});},
        6=>view_tools_panel(app,ui),
        7=>reminder_dashboard(app,ui),
        8=>controller_panel(app,ui),
        9=>session_report_panel(app,ui),
        _=>{
            v22_game_moments(app,ui);
            ui.add_space(11.0);
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                heading(ui,"CAPTURE AND HIGHLIGHTS");
                caption(ui,"Tools that operate outside the game. Screenshots automatically hide the overlay.");
                ui.horizontal_wrapped(|ui|{
                    if red_button(ui,"Take screenshot"){app.screenshot();}
                    if outline_button(ui,if app.recorder.active(){"Stop recording"}else{"Record desktop video"}){
                        app.recording();
                    }
                });
                caption(ui,"Ctrl+Shift+P screenshot · Ctrl+Shift+R recording");
            });
            ui.add_space(10.0);
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                heading(ui,"FRAME-PACING ADVISOR · REAL ETW");
                caption(ui,"Read-only analysis. Does not silently change your game's FPS cap or power plan.");
                let target=app.target_fps.clamp(30,165);
                let budget=1000.0/target as f32;
                ui.horizontal_wrapped(|ui|{
                    caption(ui,"Target");
                    if ui.add(egui::Slider::new(&mut app.target_fps,30..=165)
                        .suffix(" FPS")).changed(){
                        app.target_fps=app.target_fps.clamp(30,165);app.save();
                    }
                    caption(ui,&format!("{budget:.2} ms/frame budget"));
                });
                if let Some(actual)=app.fps.snapshot(){
                    let delta=actual.frametime_ms-budget;
                    ui.colored_label(if delta>0.0{Color32::from_rgb(245,170,107)}
                        else{Color32::from_rgb(143,221,163)},
                        format!("Real frame time {:.2} ms · {:+.2} ms vs target",
                            actual.frametime_ms,delta));
                    if let Some(low)=actual.low_1_percent_fps{
                        caption(ui,&format!("Actual PresentMon 1% low {low:.1} FPS; no generated frames counted as game FPS"));
                    }
                }else{caption(ui,"No measured game frames yet — PresentMon ETW has no active sample.");}
            });
            ui.add_space(10.0);
            frame(PANEL).show(ui,|ui|{
                ui.set_min_width(ui.available_width());
                heading(ui,"GAME BOOST · OPT-IN WINDOWS SCHEDULER");
                caption(ui,"Original priority is restored when the game changes or Horizon Boost closes.");
                if red_button(ui,if app.game_boost.active(){"Disable Game Boost"}else{"Enable Game Boost"}){
                    app.toggle_game_boost();
                }
                if outline_button(ui,"Open Performance analysis"){
                    app.page=Page::Performance;
                    app.performance_view=2;
                }
            });
        }
    }
}
fn flow_timeline(app:&BoostApp,ui:&mut Ui){
    let width=ui.available_width().min(820.0).max(320.0);
    let (rect,_)=ui.allocate_exact_size(vec2(width,116.0),Sense::hover());
    let p=ui.painter();
    p.rect_filled(rect,11.0,Color32::from_rgb(22,19,30));
    p.rect_stroke(rect,11.0,Stroke::new(1.0_f32,BORDER),StrokeKind::Inside);
    let ratio=app.settings.external_fg_factor.clamp(2,3);
    let count=ratio as usize+1;
    let mut points=Vec::new();
    for i in 0..count {
        let u=(i as f32+0.5)/count as f32;
        let x=rect.left()+u*rect.width();
        let pos=pos2(x,rect.top()+48.0);
        points.push(pos);
    }
    if points.len()>1{
        p.line_segment([points[0],*points.last().unwrap()],
            Stroke::new(1.3_f32,Color32::from_rgb(103,61,81)));
    }
    let tick=if app.settings.animations{app.app_start.elapsed().as_secs_f32()}else{0.0};
    for(i,pt)in points.iter().enumerate(){
        let real=i==0||i+1==count;
        let pulse=(tick*2.0+i as f32*0.58).sin()*0.5+0.5;
        let col=if real{RED}else{Color32::from_rgb(174,121,241)};
        p.circle_filled(*pt,if real{6.0}else{4.5}+pulse*0.6,col);
        p.text(pos2(pt.x,rect.top()+21.0),egui::Align2::CENTER_CENTER,
            if real{"REAL"}else{"FLOW"},egui::FontId::proportional(11.0),MUTED);
        p.text(pos2(pt.x,rect.top()+76.0),egui::Align2::CENTER_CENTER,
            if i==0{"FRAME A".into()}else if i+1==count{"FRAME B".into()}
            else{format!("SYNTH {i}")},
            egui::FontId::proportional(11.3),WHITE);
    }
}
fn framegen_page(app:&mut BoostApp,ui:&mut Ui){
    section(ui,"Motion Lab","Experimental 2× / 3× optical-flow interpolation on Horizon's own WGC output");
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        ui.horizontal_wrapped(|ui|{
            heading(ui,"NATIVE FLOW FRAME GENERATION");
            ui.colored_label(Color32::from_rgb(229,178,113),
                "EXPERIMENTAL · NOT IN-ENGINE FPS");
        });
        caption(ui,"GPU tile motion search + bidirectional warp across successive captured frames. No DLL injection or changes to game resolution.");
        ui.add_space(8.0);
        flow_timeline(app,ui);
        ui.add_space(11.0);
        ui.horizontal_wrapped(|ui|{
            text(ui,"Generation multiplier");
            for factor in [2u8,3u8]{
                let on=app.settings.external_fg_factor==factor;
                if ui.selectable_label(on,format!("{factor}× · {} synthetic frame(s)",factor-1))
                    .clicked(){
                    app.settings.external_fg_factor=factor;
                    app.save();
                }
            }
        });
        ui.add_space(7.0);
        let active=app.native_mirror.as_ref().is_some_and(|m|m.running());
        ui.horizontal_wrapped(|ui|{
            if active{
                if red_button(ui,"Stop GPU output"){app.stop_native_mirror();}
            }else{
                if red_button(ui,"Start independent FG preview"){
                    app.toggle_native_mirror_factor(
                        crate::native_mirror::MirrorMode::SeparateWindow,
                        app.settings.external_fg_factor);
                }
                if outline_button(ui,"Glass over game · Experimental"){
                    app.toggle_native_mirror_factor(
                        crate::native_mirror::MirrorMode::CompositedInPlace,
                        app.settings.external_fg_factor);
                }
            }
        });
        if let Some(worker)=app.native_mirror.as_ref(){
            caption(ui,&format!("Own output · captured {} REAL + {} SYNTHETIC frames · multiplier {}×",
                worker.submitted_frames(),worker.generated_frames(),worker.multiplier()));
            caption(ui,"Numbers show own output frames, NOT an increase of engine-rendered game FPS.");
        }else{caption(ui,&app.native_mirror_status);}
        ui.add_space(9.0);
        caption(ui,"Synthetic interpolation is opt-in and can add latency/artifacts during rapid motion. RX 580 throughput and game-specific compatibility require live testing.");
    });
    ui.add_space(10.0);
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"ACTUAL FPS AND FRAME BUDGET");
        ui.horizontal_wrapped(|ui|{
            let width=((ui.available_width()-16.0)/3.0).max(120.0);
            mini_metric(ui,width,"SOURCE FPS",&fps_text(app,"fps"),RED);
            mini_metric(ui,width,"SOURCE FRAME TIME",&fps_text(app,"frametime"),WHITE);
            mini_metric(ui,width,"1% LOW",&fps_text(app,"low"),Color32::from_rgb(213,145,239));
        });
        if app.fps.snapshot().is_some(){
            caption(ui,"The displayed source numbers are measured via PresentMon ETW. Generated frames are not added to these figures.");
        }else{
            caption(ui,"No source game frame data currently measured. Output counters appear after actual WGC capture starts.");
        }
    });
    ui.add_space(9.0);
    egui::CollapsingHeader::new("Supported path, constraints and safety")
        .id_salt("flowfg-technical").default_open(false).show(ui,|ui|{
            caption(ui,"2D optical flow has no game motion vectors, depth or UI masking. Thin objects, HUD and scene cuts can ghost.");
            caption(ui,"True legacy exclusive may hide the overlay. Protected game capture may fail; no anti-cheat bypass or game file modifications are attempted.");
            caption(ui,"This is not FSR 3 in-game integration, not DLSS Multi Frame Generation and not driver AFMF. Frame pacing and real latency are not yet validated in Zelda/Roblox.");
        });
}

fn macro_panel(app:&mut BoostApp,ui:&mut Ui){
    let key=app.profile_key();
    let mut profile=app.macro_drafts.get(&key).cloned().unwrap_or_else(||app.settings.macro_profiles.get(&key).cloned().unwrap_or_default());
    let mut changed=false;
    frame(PANEL).show(ui,|ui|{
        ui.set_min_width(ui.available_width());
        heading(ui,"ONE-SHOT MACRO STUDIO");
        caption(ui,"Offline and accessibility use only. No automatic farming, loops, aim assistance or bypass of game restrictions.");
        ui.add_space(8.0);
        let armed=app.macro_armed_pid.is_some();
        ui.colored_label(if armed{V25_MINT}else{MUTED},
            if armed{"ARMED · configured shortcut runs once"}else{"DISARMED · no keys sent"});
        caption(ui,&format!("Pin a game, arm manually, then focus it and use {}. F12 or focus loss cancels and releases a held key. No background key presses.",app.settings.global_shortcuts.label("MACRO")));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui|{
            let target=app.telem.active_game.as_ref().filter(|g|Some(g.pid)==app.telem.selected_pid).map(|g|g.pid);
            if armed{
                if red_button(ui,"Disarm / stop"){app.macro_armed_pid=None;app.macro_runner.cancel();}
            }else if ui.add_enabled(target.is_some()&&profile.validate().is_ok()&&!app.window_macro.busy(),egui::Button::new("Arm offline macro")).clicked(){
                app.settings.macro_profiles.insert(key.clone(),profile.clone());app.macro_drafts.remove(&key);app.save();
                app.macro_armed_pid=target;
            }
            if outline_button(ui,"Select game"){app.page=Page::Library;}
            if app.macro_runner.running(){ui.colored_label(V25_MINT,"RUNNING");}
        });
        ui.add_space(10.0);
        let mut remove=None;let mut swap=None;
        let step_count=profile.steps.len();
        ui.add_enabled_ui(!armed&&!app.macro_runner.running(),|ui|{
            for (i,step) in profile.steps.iter_mut().enumerate(){
                frame(CARD).show(ui,|ui|{
                    ui.set_min_width(ui.available_width());
                    ui.horizontal_wrapped(|ui|{
                        ui.colored_label(RED,format!("{:02}",i+1));
                        changed|=ui.add(egui::TextEdit::singleline(&mut step.key).desired_width(70.0).char_limit(8)).changed();
                        changed|=ui.add(egui::Slider::new(&mut step.hold_ms,0..=2000).suffix(" ms").text("Hold (0=tap)")).changed();
                        changed|=ui.add(egui::Slider::new(&mut step.delay_ms,100..=3000).suffix(" ms").text("Gap")).changed();
                        if ui.add_enabled(i>0,egui::Button::new("Up")).clicked(){swap=Some((i,i-1));}
                        if ui.add_enabled(i+1<step_count,egui::Button::new("Down")).clicked(){swap=Some((i,i+1));}
                        if ui.small_button("Remove").clicked(){remove=Some(i);}
                    });
                });
                ui.add_space(4.0);
            }
            if let Some((a,b))=swap{profile.steps.swap(a,b);changed=true;}
            if let Some(i)=remove{profile.steps.remove(i);changed=true;}
            if ui.add_enabled(profile.steps.len()<12,egui::Button::new("+ Add tap")).clicked(){
                profile.steps.push(crate::game_macros::Step{key:"E".into(),delay_ms:200,hold_ms:0});changed=true;
            }
        });
        if let Err(e)=profile.validate(){ui.colored_label(RED,e);}
        else{caption(ui,&format!("{} taps · {:.1} s maximum sequence · saved per game",profile.steps.len(),profile.steps.iter().map(|s|(s.delay_ms+s.hold_ms) as f32).sum::<f32>()/1000.0));}
    });
    if !armed_or_running(app){
        frame(PANEL).show(ui,|ui|{
            heading(ui,"MACRO PORTABILITY / SAFE DRY RUN");
            if outline_button(ui,"Export macro JSON"){match profile.portable_json(){Ok(v)=>app.macro_share_edit=v,Err(e)=>app.toast(e)}}
            ui.add(egui::TextEdit::multiline(&mut app.macro_share_edit).desired_rows(3).char_limit(8192).desired_width(ui.available_width()));
            if outline_button(ui,"Validate and import"){match crate::game_macros::MacroProfile::from_portable_json(&app.macro_share_edit){
                Ok(v)=>{profile=v;changed=true;},Err(e)=>app.toast(e)
            }}
            if let Ok(timeline)=profile.timeline(){
                caption(ui,"Dry run timeline only; no keyboard events are sent.");
                for (ms,key) in timeline{ui.label(format!("{ms:05} ms / {key}"));}
            }
        });
    }
    if changed&&profile.validate().is_ok(){app.macro_drafts.remove(&key);app.settings.macro_profiles.insert(key,profile);app.save();}
    else if changed{app.macro_drafts.insert(key,profile);}
}
fn local_sequence_panel(app:&mut BoostApp,ui:&mut Ui){
 frame(PANEL).show(ui,|ui|{
  heading(ui,"LOCAL HOLD / GAP SEQUENCE LAB");
  caption(ui,"Safe visual playback only. Never sends a key, controls a game or arms the offline macro runner.");
  let id=egui::Id::new(("local-seq-v58",app.profile_key()));
  let mut plan=ui.data_mut(|d|d.get_temp::<crate::local_sequence::Plan>(id)).unwrap_or_default();
  let mut player=ui.data_mut(|d|d.get_temp::<crate::local_sequence::Player>(id.with("player"))).unwrap_or_default();
  let now=std::time::Instant::now();let phase=player.tick(&plan,now);
  let mut remove=None;ui.add_enabled_ui(!player.active(),|ui|{
   for (i,s) in plan.steps.iter_mut().enumerate(){ui.push_id(i,|ui|{
    ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut s.key).char_limit(8).desired_width(60.));ui.add(egui::Slider::new(&mut s.hold_ms,20..=2000).text("Hold ms"));ui.add(egui::Slider::new(&mut s.gap_ms,0..=3000).text("Gap ms"));if ui.small_button("Remove").clicked(){remove=Some(i);}});
   });}
   if let Some(i)=remove{plan.steps.remove(i);}
   if plan.steps.len()<12&&outline_button(ui,"Add local step"){plan.steps.push(crate::local_sequence::Step{key:"E".into(),hold_ms:120,gap_ms:200});}
  });
  if !plan.valid(){caption(ui,"1-12 supported keys, 15 second limit.");}
  ui.horizontal_wrapped(|ui|{if ui.add_enabled(plan.valid()&&!player.active(),egui::Button::new("Play local preview")).clicked(){player.start(&plan,now);}if red_button(ui,"Cancel local preview"){player.stop();app.sequence_window.close();}if ui.add_enabled(plan.valid(),egui::Button::new("Open isolated sequence window")).clicked(){app.sequence_window.launch(&plan,now);}});
  if let Some((i,held,remaining))=phase.filter(|_|player.active()){ui.colored_label(if held{V25_MINT}else{MUTED},format!("{} / {} / {} ms",plan.steps[i].key,if held{"HOLD"}else{"GAP"},remaining));ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));}
  let mut raw=ui.data_mut(|d|d.get_temp::<String>(id.with("json"))).unwrap_or_default();
  ui.add_enabled_ui(!player.active(),|ui|{if outline_button(ui,"Export local plan JSON"){raw=serde_json::to_string_pretty(&plan).unwrap_or_default();}ui.add(egui::TextEdit::multiline(&mut raw).char_limit(4096).desired_rows(3));if outline_button(ui,"Import local plan"){if let Ok(p)=serde_json::from_str::<crate::local_sequence::Plan>(&raw){if p.valid(){plan=p;}}}});
  ui.data_mut(|d|{d.insert_temp(id,plan);d.insert_temp(id.with("player"),player);d.insert_temp(id.with("json"),raw);});
 });
}
fn armed_or_running(app:&BoostApp)->bool{app.macro_armed_pid.is_some()||app.macro_runner.running()}

fn window_macro_panel(app:&mut BoostApp,ui:&mut Ui){
 named_macro_binding(app,ui);
 let profile=app.profile_key();let scope_id=ui.id().with("window-macro-global-library");let mut global=ui.data(|d|d.get_temp::<bool>(scope_id)).unwrap_or(false);
 let blocked=app.macro_runner.running()||app.macro_armed_pid.is_some()||app.is_smoke;
 let s=&mut app.window_macro;let mut persist=false;let before=s.plan.clone();let mut history_action=false;
 frame(PANEL).show(ui,|ui|{
  heading(ui,local(ui,"Macro Studio","Studio maker"));caption(ui,local(ui,"Record and replay a selected window. Offline or ordinary desktop use only. Never record passwords or private text.","Záznam a přehrání ve vybraném okně. Jen offline nebo běžné aplikace. Nezaznamenávej hesla ani soukromý text."));
  ui.horizontal_wrapped(|ui|{ui.colored_label(if s.busy(){RED}else{MUTED},s.mode());ui.label(format!("{} events",s.progress()));if s.busy(){ui.spinner();if red_button(ui,"Stop / F12"){s.cancel();}ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));}});
  ui.add_enabled_ui(!s.busy()&&!blocked,|ui|{
   if outline_button(ui,local(ui,"Find windows","Najít okna")){s.refresh();}
   let old=s.selected.clone();
   egui::ComboBox::from_id_salt("macro-window").selected_text(s.selected.as_ref().map(|t|trim(&t.title,48)).unwrap_or("Select window".into())).width(ui.available_width().min(420.)).show_ui(ui,|ui|{
    for t in &s.windows{ui.selectable_value(&mut s.selected,Some(t.clone()),trim(&t.title,64));}
   });
   if old!=s.selected{s.consent=false;}
   ui.checkbox(&mut s.consent,local(ui,"I selected a permitted target; no sensitive data","Vybral jsem povolený cíl bez citlivých údajů"));
   ui.horizontal_wrapped(|ui|{
    if ui.add_enabled(s.selected.is_some()&&s.consent,egui::Button::new(local(ui,"Record","Nahrát")).fill(RED)).clicked(){if let Err(e)=s.start(true){s.status=e;}}
    if ui.add_enabled(s.selected.is_some()&&s.consent&&s.plan.validate().is_ok(),egui::Button::new(local(ui,"Play","Přehrát"))).clicked(){if let Err(e)=s.start(false){s.status=e;}}
   });
  });
  if !s.status.is_empty(){caption(ui,&s.status);}
  caption(ui,"Pause/Break pauses playback only at an input-released boundary. Press again to resume; F12, focus loss or 30s timeout stops.");
  ui.add_space(8.);
  ui.add_enabled_ui(!s.busy(),|ui|{
   ui.horizontal_wrapped(|ui|{
    ui.add(egui::Slider::new(&mut s.plan.speed,50..=200).suffix("%").text("Speed"));
    ui.add(egui::Slider::new(&mut s.plan.repeat,1..=10).text("Repeat"));
    ui.add(egui::DragValue::new(&mut s.plan.between_ms).range(0..=10000).suffix(" ms gap"));
   });
   caption(ui,&format!("{} events / {:.2} s per run / {:.2} s total",s.plan.events.len(),s.plan.single_duration()as f64/1000.,s.plan.duration()as f64/1000.));
   if s.plan.validate().is_ok(){ui.colored_label(V25_MINT,"Sequence valid");}
   if let Err(e)=s.plan.validate(){caption(ui,&e);}
   ui.horizontal_wrapped(|ui|{
    if outline_button(ui,"New / clear"){s.plan=Default::default();s.preview_at=None;s.name.clear();}
    if ui.add_enabled(s.history.can_undo(),egui::Button::new("Undo")).clicked(){history_action=s.history.undo(&mut s.plan);s.preview_at=None;}
    if ui.add_enabled(s.history.can_redo(),egui::Button::new("Redo")).clicked(){history_action=s.history.redo(&mut s.plan);s.preview_at=None;}
    if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Visual dry run")).clicked(){s.preview_at=Some(std::time::Instant::now());}
    if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Copy macro JSON")).clicked(){if let Ok(v)=s.plan.export(){ui.ctx().copy_text(v);}}
    if ui.add_enabled(s.preview_at.is_some(),egui::Button::new("Stop preview")).clicked(){s.preview_at=None;}
   });
   if s.preview_at.is_some()&&s.plan.validate().is_err(){s.preview_at=None;}
   if let Some(at)=s.preview_at{let ms=at.elapsed().as_millis().min(u128::from(u64::MAX))as u64;let p=crate::window_macro::preview(&s.plan,ms);
    let(r,_)=ui.allocate_exact_size(vec2(ui.available_width(),100.),Sense::hover());ui.painter().rect_filled(r,8.,BG);
    if let Some([x,y])=p.point{let pt=pos2(r.left()+r.width()*f32::from(x)/10000.,r.top()+r.height()*f32::from(y)/10000.);ui.painter().circle_filled(pt,if p.buttons.is_empty(){5.}else{9.},RED);}
    ui.label(format!("Cycle {} / {} · events {} · held keys {} · buttons {}",p.cycle,s.plan.repeat,p.applied,p.keys.len(),p.buttons.len()));
    ui.add(egui::ProgressBar::new((ms as f32/s.plan.duration().max(1)as f32).clamp(0.,1.)));
    if ms>=s.plan.duration(){s.preview_at=None;}else{ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));}
   }
   egui::CollapsingHeader::new("Inspect / scrub dry-run timeline").show(ui,|ui|{
    let id=ui.id().with("macro-scrub");let mut ms=ui.data(|d|d.get_temp::<u64>(id)).unwrap_or(0).min(s.plan.duration());
    if s.plan.validate().is_ok(){if ui.add(egui::Slider::new(&mut ms,0..=s.plan.duration()).suffix(" ms")).changed(){s.preview_at=None;}
    let p=crate::window_macro::preview(&s.plan,ms);ui.label(format!("Cycle {} / {} events / {} keys held / {} buttons held",p.cycle,p.applied,p.keys.len(),p.buttons.len()));}
    ui.data_mut(|d|d.insert_temp(id,ms));caption(ui,"Manual preview is paused at selected timestamp; never sends input.");
   });
   ui.horizontal_wrapped(|ui|{if ui.checkbox(&mut global,"Shared macro library").changed(){ui.data_mut(|d|d.insert_temp(scope_id,global));s.name.clear();s.preview_at=None;}caption(ui,if global{"Shared library"}else{"Current game profile"});});
   ui.horizontal_wrapped(|ui|{
    ui.add(egui::TextEdit::singleline(&mut s.name).hint_text("Macro name").char_limit(48));
    if outline_button(ui,"Save named macro"){let result=if global{s.plan.save(&s.name,&mut app.settings.window_macros)}else if app.settings.window_macro_profiles.len()>=128&&!app.settings.window_macro_profiles.contains_key(&profile){Err("Macro profile limit reached".into())}else{s.plan.save(&s.name,app.settings.window_macro_profiles.entry(profile.clone()).or_default())};match result{Ok(())=>{persist=true;s.status="Saved".into();},Err(e)=>s.status=e}}
   });
   let names=if global{app.settings.window_macros.keys().cloned().collect::<Vec<_>>()}else{app.settings.window_macro_profiles.get(&profile).map(|b|b.keys().cloned().collect()).unwrap_or_default()};
   egui::ComboBox::from_id_salt("saved-window-macros").selected_text(if s.name.is_empty(){"Saved macros"}else{&s.name}).show_ui(ui,|ui|{for n in names{if ui.selectable_label(s.name==n,&n).clicked(){s.name=n.clone();if let Some(p)=if global{app.settings.window_macros.get(&n)}else{app.settings.window_macro_profiles.get(&profile).and_then(|b|b.get(&n))}{s.plan=p.clone();s.preview_at=None;}}}});
   let exists=if global{app.settings.window_macros.contains_key(&s.name)}else{app.settings.window_macro_profiles.get(&profile).is_some_and(|b|b.contains_key(&s.name))};
   if exists&&outline_button(ui,"Delete saved macro"){if global{app.settings.window_macros.remove(&s.name);}else if let Some(b)=app.settings.window_macro_profiles.get_mut(&profile){b.remove(&s.name);}persist=true;s.name.clear();}
   egui::CollapsingHeader::new("Offline starter templates").show(ui,|ui|{
    caption(ui,"Load a finite editable sample into the editor; does not play or send input. Existing Undo history preserved.");
    ui.horizontal_wrapped(|ui|{
     for(i,label)in ["Single key","Two keys","Center click"].iter().enumerate(){
      if ui.button(*label).clicked(){
       match crate::window_macro::starter(i){Ok(p)=>{s.plan=p;s.preview_at=None;s.status="Template loaded, not running".into();},Err(e)=>s.status=e}
      }
     }
    });
   });
   egui::CollapsingHeader::new("Add actions manually").show(ui,|ui|{
    let id=ui.id().with("macro-manual");let mut key=ui.data(|d|d.get_temp::<String>(id.with("key"))).unwrap_or("E".into());
    let mut hold=ui.data(|d|d.get_temp::<u32>(id.with("hold"))).unwrap_or(100);let mut wait=ui.data(|d|d.get_temp::<u32>(id.with("wait"))).unwrap_or(250);
    let mut xy=ui.data(|d|d.get_temp::<[u16;2]>(id.with("xy"))).unwrap_or([5000,5000]);
    let mut button=ui.data(|d|d.get_temp::<u8>(id.with("button"))).unwrap_or(0);
    ui.horizontal_wrapped(|ui|{ui.add(egui::TextEdit::singleline(&mut key).desired_width(75.).char_limit(8));ui.add(egui::DragValue::new(&mut wait).range(0..=5000).suffix(" ms wait"));ui.add(egui::DragValue::new(&mut hold).range(0..=5000).suffix(" ms hold"));
     if outline_button(ui,"Add key press"){if let Err(e)=s.plan.append_key(&key,wait,hold){s.status=e;}}
    });
    let(r,res)=ui.allocate_exact_size(vec2(ui.available_width().min(300.),100.),Sense::click_and_drag());ui.painter().rect_filled(r,6.,BG);
    if(res.clicked()||res.dragged())&&let Some(q)=res.interact_pointer_pos(){xy=[(((q.x-r.left())/r.width()).clamp(0.,1.)*10000.)as u16,(((q.y-r.top())/r.height()).clamp(0.,1.)*10000.)as u16];}
    ui.painter().circle_filled(pos2(r.left()+r.width()*xy[0]as f32/10000.,r.top()+r.height()*xy[1]as f32/10000.),5.,RED);
    ui.horizontal_wrapped(|ui|{ui.selectable_value(&mut button,0,"Left");ui.selectable_value(&mut button,1,"Right");
     if outline_button(ui,"Add click"){if let Err(e)=s.plan.append_click(xy[0],xy[1],button,wait,hold){s.status=e;}}
     if outline_button(ui,"Add double click"){if let Err(e)=s.plan.append_double_click(xy[0],xy[1],button,wait,hold.clamp(10,400),125){s.status=e;}}
    });
    ui.data_mut(|d|{d.insert_temp(id.with("key"),key);d.insert_temp(id.with("wait"),wait);d.insert_temp(id.with("hold"),hold);d.insert_temp(id.with("xy"),xy);d.insert_temp(id.with("button"),button);});
   });
   egui::CollapsingHeader::new("Timeline tools (transactional)").show(ui,|ui|{
    caption(ui,"Edits only the local plan; Undo reverses changes. Protected multiplayer targets remain blocked.");
    ui.horizontal_wrapped(|ui|{
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Flip X")).clicked(){if let Err(e)=s.plan.mirror_pointer(true,false){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Flip Y")).clicked(){if let Err(e)=s.plan.mirror_pointer(false,true){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Move +5% X")).clicked(){if let Err(e)=s.plan.translate_pointer(500,0){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Move -5% X")).clicked(){if let Err(e)=s.plan.translate_pointer(-500,0){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Move +5% Y")).clicked(){if let Err(e)=s.plan.translate_pointer(0,500){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Move -5% Y")).clicked(){if let Err(e)=s.plan.translate_pointer(0,-500){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Wait x0.75")).clicked(){if let Err(e)=s.plan.scale_waits(75){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Wait x1.25")).clicked(){if let Err(e)=s.plan.scale_waits(125){s.status=e;}}
    });
    ui.horizontal_wrapped(|ui|{
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("+250 ms before start")).clicked(){if let Err(e)=s.plan.shift_start(250){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("-250 ms before start")).clicked(){if let Err(e)=s.plan.shift_start(-250){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Double actions")).clicked(){if let Err(e)=s.plan.double_sequence(){s.status=e;}}
     if ui.add_enabled(s.plan.validate().is_ok(),egui::Button::new("Remove exact duplicate moves")).clicked(){match s.plan.compact_exact_duplicates(){Ok(n)=>s.status=format!("Removed {n} redundant exact zero-wait pointer moves"),Err(e)=>s.status=e}}
    });
   });
   egui::CollapsingHeader::new("Timeline editor").show(ui,|ui|{
    let mut remove=None;let mut swap=None;let mut wait_edit=None;let count=s.plan.events.len();let mut timeline_ms=0u64;let speed=s.plan.speed.max(1) as u64;
    egui::ScrollArea::vertical().max_height(280.).id_salt("macro-events").show(ui,|ui|{
     for(i,e)in s.plan.events.iter_mut().enumerate(){ui.push_id(i,|ui|{ui.horizontal_wrapped(|ui|{
      ui.label(format!("{:03}",i+1));let mut wait=e.wait_ms;if ui.add(egui::DragValue::new(&mut wait).range(0..=60000).suffix(" ms")).changed(){wait_edit=Some((i,wait));}timeline_ms+=(u64::from(e.wait_ms)*100).div_ceil(speed);ui.label(format!("@ {:.3}s",timeline_ms as f64/1000.));ui.label(e.action.label());
      if ui.add_enabled(i>0,egui::Button::new("↑")).clicked(){swap=Some((i,i-1));}
      if ui.add_enabled(i+1<count,egui::Button::new("↓")).clicked(){swap=Some((i,i+1));}
      if ui.small_button("Remove").clicked(){remove=Some(i);}
     });});}
    });
    if let Some((i,wait))=wait_edit{if let Err(e)=s.plan.set_wait(i,wait){s.status=e;}}
    if let Some(i)=remove{if let Err(e)=s.plan.remove_balanced(i){s.status=e;}else{s.status="Action removed".into();}}if let Some((a,b))=swap{if let Err(e)=s.plan.swap_balanced(a,b){s.status=e;}}
    if let Err(e)=s.plan.validate(){ui.colored_label(RED,e);}
   });
   egui::CollapsingHeader::new("Import / export JSON").show(ui,|ui|{
    if outline_button(ui,"Export"){match s.plan.export(){Ok(v)=>s.raw=v,Err(e)=>s.status=e}}
    ui.add(egui::TextEdit::multiline(&mut s.raw).desired_rows(4).char_limit(65536).desired_width(ui.available_width()));
    if outline_button(ui,"Validate and import"){match crate::window_macro::Plan::import(&s.raw){Ok(p)=>{s.plan=p;s.preview_at=None;s.status="Imported; not running".into();},Err(e)=>s.status=e}}
   });
  });
  caption(ui,local(ui,"16 ms sampled input, up to 60 s / 512 events, 10 repeats, F12 or focus loss stops. No global recorder or anti-cheat bypass.","Vzorkování 16 ms, max. 60 s / 512 událostí, 10 opakování. F12 nebo ztráta fokusu zastaví. Bez globálního záznamu či obcházení anti-cheatu."));
 });
 if !history_action&&before!=s.plan{s.history.record(before);s.preview_at=None;}
 if persist{app.save();}
}
