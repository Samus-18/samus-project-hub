use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterProfile {
    pub vibrance: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub sharpen: f32,
    pub intensity: f32,
    pub enabled: bool,
}
impl Default for FilterProfile {
    fn default() -> Self {
        Self { vibrance: 0.32, contrast: 0.18, brightness: 0.06, sharpen: 0.4, intensity: 1.0, enabled: false }
    }
}
impl FilterProfile {
    pub fn clamp(&mut self) {
        self.vibrance = self.vibrance.clamp(0.0, 1.0);
        self.contrast = self.contrast.clamp(-0.5, 0.5);
        self.brightness = self.brightness.clamp(-0.5, 0.5);
        self.sharpen = self.sharpen.clamp(0.0, 1.0);
        self.intensity = self.intensity.clamp(0.0, 1.0);
    }
    pub fn quality() -> Self {
        Self { vibrance: 0.27, contrast: 0.11, brightness: 0.03, sharpen: 0.55, intensity: 0.85, enabled: false }
    }
    pub fn performance() -> Self {
        Self { vibrance: 0.0, contrast: 0.0, brightness: 0.0, sharpen: 0.0, intensity: 0.0, enabled: false }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GpuProfile {
    pub scaler:usize,
    pub effect:usize,
    pub strength:f32,
    pub stack:usize,
    pub capture_method:usize,
    pub output_mode:usize
}
impl Default for GpuProfile {
    fn default()->Self{Self{scaler:0,effect:2,strength:0.45,stack:0,capture_method:0,output_mode:0}}
}
impl GpuProfile {
    pub fn sanitize(&mut self){
        self.scaler=self.scaler.min(crate::engine::SCALERS.len()-1);
        self.effect=self.effect.min(crate::engine::EFFECTS.len()-1);
        self.strength=if self.strength.is_finite(){self.strength.clamp(0.0,1.0)}else{0.5};
        self.stack=crate::stacks::valid(self.stack);
        self.capture_method=self.capture_method.min(crate::engine::CAPTURE_METHODS.len()-1);
        self.output_mode=self.output_mode.min(crate::engine::OUTPUT_MODES.len()-1);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub profile: FilterProfile,
    pub profiles: BTreeMap<String, FilterProfile>,
    pub gpu_profiles: BTreeMap<String, GpuProfile>,
    pub custom_profiles: BTreeMap<String, FilterProfile>,
    pub custom_gpu_profiles: BTreeMap<String, GpuProfile>,
    pub preset_selection: BTreeMap<String, usize>,
    pub animations: bool,
    pub overlay_opacity: f32,
    /// Logical client size; configurable without maximizing the desktop.
    pub main_window_size:[f32;2],
    /// HUD+editor size (the HUD is still click-through when passive).
    pub overlay_size_scale:f32,
    pub hud_corner:u8,
    pub hud_metric:u8,
    pub hud_follow_foreground:bool,
    pub hud_f12:bool,
    pub hud_start_open:bool,
    pub hud_show_fg:bool,
    pub macro_profiles:BTreeMap<String,crate::game_macros::MacroProfile>,
    pub window_macros:BTreeMap<String,crate::window_macro::Plan>,
    pub window_macro_profiles:BTreeMap<String,BTreeMap<String,crate::window_macro::Plan>>,
    pub named_macro_hotkeys:BTreeMap<String,String>,
    pub tool_shortcuts:crate::tool_shortcuts::Shortcuts,
    pub global_shortcuts:crate::shortcut_registry::Registry,
    pub replay_seconds:u32,
    pub recording_limit_mib:u32,
    pub capture_gains:[f32;2],
    pub overhead_history:BTreeMap<String,Vec<crate::suite_finish::MeasuredOverhead>>,
    pub overhead_limits:BTreeMap<String,u8>,
    pub output_targets:BTreeMap<String,u32>,
    pub capture_mute_mic:bool,
    pub recording_stop_on_exit:bool,
    pub replay_game_lengths:BTreeMap<String,u32>,
    pub clip_annotations:BTreeMap<String,crate::clip_book::Clip>,
    pub capture_mbps:u32,
    pub replay_limit_mib:u32,
    pub capture_remux:bool,
    pub resource_alerts:bool,
    pub break_minutes:u32,
    pub reminder_profiles:BTreeMap<String,Vec<crate::reminders::Spec>>,
    pub record_fps: u32,
    /// Real frames presented by Horizon's own preview only, not the game.
    pub preview_fps_limit:u32,
    pub preview_vsync:bool,
    pub preview_cursor_scale:u8,
    pub preview_low_latency:bool,
    pub preview_composition:bool,
    pub preview_scrgb:bool,
    pub preview_hdr_white_nits:f32,
    pub hdr_curve:u8,pub hdr_peak_nits:u32,pub hdr_exposure:u8,
    pub preview_tearing:bool,pub renderer_adapter:Option<String>,
    pub preview_display:Option<String>,
    pub display_grades:crate::display_color::Profiles,
    pub record_width: u32,
    pub hardware_encoder: bool,
    pub detect_games: bool,
    pub fps_monitoring: bool,
    pub anti_cheat_safe_mode:bool,
    pub stutter_scout_enabled:bool,
    pub stutter_threshold_ms:f32,
    pub live_preset: usize,
    pub live_scaler: usize,
    pub live_strength: f32,
    pub live_stack:usize,
    pub live_capture_method:usize,
    pub live_output_mode:usize,
    pub custom_stages:Vec<crate::stacks::CustomStage>,
    pub preview_image_path: Option<String>,
    pub game_notes:BTreeMap<String,String>,
    pub crosshair:crate::crosshair::CrosshairOptions,
    pub crosshair_profiles:BTreeMap<String,crate::crosshair::CrosshairOptions>,
    pub hero_photo_path:Option<String>,
    pub look_pairs:BTreeMap<String,crate::compare::LookPair>,
    pub named_native_looks:BTreeMap<String,crate::compare::NativeLook>,
    pub compact_controls:bool,
    pub favorite_effects:Vec<usize>,
    pub native_preset:usize,
    pub native_preset_profiles:BTreeMap<String,usize>,
    pub native_output_modes:BTreeMap<String,u8>,
    pub minecraft_undo:BTreeMap<String,crate::minecraft_profile::Undo>,
    pub minecraft_active:BTreeMap<String,u8>,
    pub studio_previous:BTreeMap<String,crate::compare::NativeLook>,
    pub native_stack_profiles:BTreeMap<String,Vec<usize>>,
    pub node_graph_profiles:BTreeMap<String,crate::node_graph::Graph>,
    pub lighting:crate::lighting::LightTuning,
    pub lighting_profiles:BTreeMap<String,crate::lighting::LightTuning>,
    pub external_fg_factor:u8,
    pub experimental_fg_allowed:bool,
    pub native_quality:u8,
    pub native_scale_mode:u8,
    pub native_scale_ratio:f32,
    pub processing_percent:u8,
    pub processing_profiles:BTreeMap<String,u8>,
    pub render_preferences:BTreeMap<String,crate::profile_portable::RenderPreferences>,
    pub monitor_fps_profiles:BTreeMap<String,BTreeMap<String,crate::monitor_fps_profiles::Profile>>,
    pub monitor_fps_auto_apply:bool,
    pub native_favorites:Vec<usize>,
    pub curated_games:Vec<crate::game_library::Entry>,
    pub quiet_ui:bool,
    pub ui_zoom:f32,
    pub hud_layout:crate::hud_layout::Layout,
    pub reminder_sound:bool,
    pub ui_theme:u8,
    pub ui_language:u8,
    pub ui_advanced:bool,
    pub controller_deadzone:f32,
    pub os_color_mode:usize,
    pub os_color_strength:f32,
    pub os_color_profiles:BTreeMap<String,usize>,
    pub quest_checklists:BTreeMap<String,Vec<crate::gameplay::QuestTask>>,
    pub cooldown_profiles:BTreeMap<String,crate::gameplay::CooldownSpec>,
    pub training_profiles:BTreeMap<String,crate::training::TrainingProgress>,
    pub journal_profiles:BTreeMap<String,crate::game_journal::Book>,
    pub utility_profiles:BTreeMap<String,crate::utility_book::Book>,
    pub library_meta:BTreeMap<String,crate::library_hub::Meta>,
    pub map_notes:BTreeMap<String,crate::suite_finish::MapNote>,
    pub hud_profiles:BTreeMap<String,crate::hud_layout::Layout>,
    pub quiet_clip_toasts:bool,
    pub pressure_quiet:bool,
    pub optional_modules:crate::suite_finish::ModuleConfig,
    pub session_goals:BTreeMap<String,u32>,
    pub moments:BTreeMap<String,Vec<crate::gameplay::GameMoment>>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: FilterProfile::default(),profiles: BTreeMap::new(),gpu_profiles:BTreeMap::new(),
            custom_profiles:BTreeMap::new(),custom_gpu_profiles:BTreeMap::new(),
            preset_selection:BTreeMap::new(),
            animations: true, overlay_opacity: 1.0,
            main_window_size:[940.0,620.0],overlay_size_scale:0.88,
            hud_corner:1,hud_metric:0,hud_follow_foreground:true,hud_f12:false,hud_start_open:false,hud_show_fg:true,
            macro_profiles:BTreeMap::new(),window_macros:BTreeMap::new(),window_macro_profiles:BTreeMap::new(),named_macro_hotkeys:BTreeMap::new(),tool_shortcuts:crate::tool_shortcuts::Shortcuts::default(),global_shortcuts:crate::shortcut_registry::Registry::default(),
            replay_seconds:30,recording_limit_mib:4096,capture_gains:[1.,1.],overhead_history:BTreeMap::new(),overhead_limits:BTreeMap::new(),output_targets:BTreeMap::new(),capture_mute_mic:false,recording_stop_on_exit:false,replay_game_lengths:BTreeMap::new(),clip_annotations:BTreeMap::new(),capture_mbps:6,replay_limit_mib:256,capture_remux:false,resource_alerts:false,break_minutes:45,reminder_profiles:BTreeMap::new(),
            record_fps: 30, preview_fps_limit:45,preview_vsync:true,preview_cursor_scale:100,preview_low_latency:false,preview_composition:false,preview_scrgb:false,preview_hdr_white_nits:100.,hdr_curve:0,hdr_peak_nits:400,hdr_exposure:10,preview_tearing:false,renderer_adapter:None,preview_display:None,display_grades:BTreeMap::new(),
            record_width: 1280, hardware_encoder: true, detect_games: true, fps_monitoring:true,anti_cheat_safe_mode:true,
            stutter_scout_enabled:false,stutter_threshold_ms:50.0,
            live_preset: 2, live_scaler: 0, live_strength:0.45,live_stack:0,
            live_capture_method:0,live_output_mode:0,
            custom_stages:crate::stacks::default_custom(),
            preview_image_path:None,
            game_notes:BTreeMap::new(),
            crosshair:crate::crosshair::CrosshairOptions::default(),
            crosshair_profiles:BTreeMap::new(),
            hero_photo_path:None,
            look_pairs:BTreeMap::new(),named_native_looks:BTreeMap::new(),
            compact_controls:true,
            favorite_effects:Vec::new(),
             native_preset:0,native_preset_profiles:BTreeMap::new(),native_output_modes:BTreeMap::new(),minecraft_undo:BTreeMap::new(),minecraft_active:BTreeMap::new(),studio_previous:BTreeMap::new(),
            native_stack_profiles:BTreeMap::new(),node_graph_profiles:BTreeMap::new(),
            lighting:crate::lighting::LightTuning::default(),lighting_profiles:BTreeMap::new(),
            external_fg_factor:1,experimental_fg_allowed:false,native_quality:3,
            native_scale_mode:0,native_scale_ratio:1.0,processing_percent:100,processing_profiles:BTreeMap::new(),
            render_preferences:BTreeMap::new(),monitor_fps_profiles:BTreeMap::new(),monitor_fps_auto_apply:false,native_favorites:Vec::new(),curated_games:Vec::new(),quiet_ui:true,ui_zoom:1.0,hud_layout:Default::default(),reminder_sound:false,ui_theme:0,ui_language:0,ui_advanced:false,controller_deadzone:0.20,
            os_color_mode:0,os_color_strength:0.80,os_color_profiles:BTreeMap::new(),
            quest_checklists:BTreeMap::new(),cooldown_profiles:BTreeMap::new(),
            training_profiles:BTreeMap::new(),journal_profiles:BTreeMap::new(),utility_profiles:BTreeMap::new(),library_meta:BTreeMap::new(),map_notes:BTreeMap::new(),hud_profiles:BTreeMap::new(),quiet_clip_toasts:false,pressure_quiet:false,optional_modules:Default::default(),session_goals:BTreeMap::new(),
            moments:BTreeMap::new(),
        }
    }
}
pub fn normalize_favorite_effects(ids:&mut Vec<usize>){
    ids.retain(|&id|id<crate::engine::EFFECTS.len());
    ids.sort_unstable();
    ids.dedup();
}
pub fn folder() -> PathBuf {
    if let Ok(exe)=std::env::current_exe(){if let Some(parent)=exe.parent(){if std::fs::read(parent.join("portable.flag")).ok().as_deref()==Some(b"HorizonBoost.Portable.1"){return parent.join("data");}}}
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("HorizonBoostRust")
}
pub fn load() -> Settings {
    let path=folder().join("settings.json");
    let data=fs::read(&path).ok().and_then(|bytes|serde_json::from_slice::<Settings>(&bytes).ok());
    let mut value=data.unwrap_or_default();
    value.profile.clamp();
    value.record_fps=value.record_fps.clamp(15,60);
    if value.renderer_adapter.as_ref().is_some_and(|s|!crate::adapter_select::valid_id(s)){value.renderer_adapter=None;}
    if value.preview_display.as_ref().is_some_and(|s|s.len()>64||s.is_empty()){value.preview_display=None;}
    if !matches!(value.preview_cursor_scale,100|150|200){value.preview_cursor_scale=100;}
    value.hdr_curve=value.hdr_curve.min(2);value.hdr_peak_nits=value.hdr_peak_nits.clamp(80,10000);value.hdr_exposure=value.hdr_exposure.clamp(5,40);
    value.preview_hdr_white_nits=if value.preview_hdr_white_nits.is_finite(){value.preview_hdr_white_nits.clamp(80.,400.)}else{100.};
    value.preview_fps_limit=match value.preview_fps_limit{
        0|30|45|60|75|90|100|120|144|165=>value.preview_fps_limit,_=>45
    };
    value.stutter_threshold_ms=if value.stutter_threshold_ms.is_finite(){
        value.stutter_threshold_ms.clamp(20.0,200.0)
    }else{50.0};
    value.live_preset=value.live_preset.min(crate::engine::EFFECTS.len().saturating_sub(1));
    value.live_scaler=value.live_scaler.min(crate::engine::SCALERS.len().saturating_sub(1));
    value.live_strength=value.live_strength.clamp(0.0,1.0);
    value.live_stack=crate::stacks::valid(value.live_stack);
    value.live_capture_method=value.live_capture_method.min(crate::engine::CAPTURE_METHODS.len()-1);
    value.live_output_mode=value.live_output_mode.min(crate::engine::OUTPUT_MODES.len()-1);
    for tasks in value.quest_checklists.values_mut(){crate::gameplay::sanitize_tasks(tasks);}
    for cooldown in value.cooldown_profiles.values_mut(){cooldown.sanitize();}
    for practice in value.training_profiles.values_mut(){practice.sanitize();}
    for book in value.journal_profiles.values_mut(){book.sanitize();}
    for book in value.utility_profiles.values_mut(){book.sanitize();}
    for meta in value.library_meta.values_mut(){meta.sanitize();}
    for map in value.map_notes.values_mut(){map.sanitize();}
    for hud in value.hud_profiles.values_mut(){hud.sanitize();hud.audio=false;}
    for goal in value.session_goals.values_mut(){*goal=(*goal).clamp(5,720);}
    for moments in value.moments.values_mut(){crate::gameplay::sanitize_moments(moments);}
    value.os_color_mode=crate::color_modes::valid_mode(value.os_color_mode);
    value.os_color_strength=if value.os_color_strength.is_finite(){
        value.os_color_strength.clamp(0.0,1.0)
    }else{0.8};
    for mode in value.os_color_profiles.values_mut(){
        *mode=crate::color_modes::valid_mode(*mode);
    }
    value.native_preset=value.native_preset.min(crate::native_presets::PRESETS.len()-1);
    value.native_stack_profiles.retain(|_,stages|crate::native_stack::validate(stages).is_ok());
    value.node_graph_profiles.retain(|_,g|g.compile().is_ok());
    crate::display_color::normalize(&mut value.display_grades);
    value.lighting.clamp();
    for v in value.lighting_profiles.values_mut(){v.clamp();}
    value.external_fg_factor=value.external_fg_factor.clamp(1,3);
    value.native_quality=value.native_quality.min(3);
    value.native_scale_mode=value.native_scale_mode.min((crate::spatial::MODES.len()-1)as u8);
    value.native_scale_ratio=crate::spatial::clamp_ratio(value.native_scale_ratio);
    value.processing_percent=crate::render_budget::valid(value.processing_percent);
    value.render_preferences.retain(|_,v|v.valid());
    value.ui_zoom=if value.ui_zoom.is_finite(){value.ui_zoom.clamp(0.75,1.35)}else{1.0};
    value.hud_layout.sanitize();value.ui_theme=value.ui_theme.min(2);value.ui_language=value.ui_language.min(1);
    value.curated_games.retain(|v|v.valid());value.curated_games.truncate(200);
    value.native_favorites.retain(|v|*v<crate::native_presets::PRESETS.len());
    value.native_favorites.sort_unstable();value.native_favorites.dedup();
    value.controller_deadzone=if value.controller_deadzone.is_finite(){value.controller_deadzone.clamp(0.,0.95)}else{0.20};
    for p in value.processing_profiles.values_mut(){*p=crate::render_budget::valid(*p);}
    value.native_output_modes.retain(|key,mode|key.len()<=520&&*mode<=2);
    crate::minecraft_profile::sanitize(&mut value);
    crate::studio_draft::sanitize_previous(&mut value);
    for id in value.native_preset_profiles.values_mut(){
        *id=(*id).min(crate::native_presets::PRESETS.len()-1);
    }
    value.crosshair.clamp();
    for v in value.crosshair_profiles.values_mut(){v.clamp();}
    normalize_favorite_effects(&mut value.favorite_effects);
    crate::native_look_library::sanitize(&mut value.named_native_looks);
    for pair in value.look_pairs.values_mut(){
        if let Some(a)=pair.a.as_mut(){a.sanitize();}
        if let Some(b)=pair.b.as_mut(){b.sanitize();}
        if pair.active.is_some_and(|slot|slot>1||!pair.stored(slot)){pair.active=None;}
    }
    value.custom_stages.truncate(4);
    for stage in &mut value.custom_stages{
        stage.effect=stage.effect.clamp(1,crate::engine::EFFECTS.len()-1);
        stage.weight=if stage.weight.is_finite(){stage.weight.clamp(0.0,1.0)}else{0.5};
    }
    for preset in value.gpu_profiles.values_mut(){preset.sanitize();}
    value.record_width=if value.record_width>=1600{1920}else{1280};
    value.overlay_opacity=value.overlay_opacity.clamp(0.72,1.0);
    for (axis,min,max,fallback) in [(0,800.0,1320.0,940.0),
        (1,540.0,880.0,620.0)]{
        let val=value.main_window_size[axis];
        value.main_window_size[axis]=if val.is_finite(){
            val.clamp(min,max)
        }else{fallback};
    }
    value.hud_corner=value.hud_corner.min(3);
    value.hud_metric=value.hud_metric.min(9);
    value.tool_shortcuts.sanitize();
    value.global_shortcuts.sanitize();
    crate::monitor_fps_profiles::sanitize(&mut value.monitor_fps_profiles);
    value.replay_seconds=crate::instant_replay::replay_seconds(value.replay_seconds);
    value.recording_limit_mib=crate::capture_session::limit_mib(value.recording_limit_mib);
    value.optional_modules.sanitize();
    for v in value.overhead_limits.values_mut(){*v=(*v).clamp(1,50);}
    for v in value.output_targets.values_mut(){*v=(*v).clamp(30,165);}
    for v in value.overhead_history.values_mut(){v.retain(crate::suite_finish::MeasuredOverhead::valid);if v.len()>24{v.drain(..v.len()-24);}}
    for x in &mut value.capture_gains{*x=if x.is_finite(){x.clamp(0.,2.)}else{1.};}
    for n in value.replay_game_lengths.values_mut(){*n=crate::instant_replay::replay_seconds(*n);}
    crate::clip_book::sanitize_all(&mut value.clip_annotations);
    value.capture_mbps=crate::media_tools::bitrate(value.capture_mbps);value.replay_limit_mib=crate::media_tools::replay_limit(value.replay_limit_mib);value.break_minutes=value.break_minutes.clamp(5,180);
    for v in value.reminder_profiles.values_mut(){crate::reminders::sanitize(v);}
    value.macro_profiles.retain(|_,p|p.validate().is_ok());crate::window_macro::sanitize(&mut value.window_macros);crate::window_macro::sanitize_profiles(&mut value.window_macro_profiles);
    value.named_macro_hotkeys.retain(|k,n|value.window_macro_profiles.get(k).is_some_and(|b|b.contains_key(n)));
    value.overlay_size_scale=if value.overlay_size_scale.is_finite(){
        value.overlay_size_scale.clamp(0.78,1.35)
    }else{0.88};
    for p in value.profiles.values_mut() {p.clamp();}
    for p in value.custom_profiles.values_mut(){p.clamp();}
    for p in value.custom_gpu_profiles.values_mut(){p.sanitize();}
    for n in value.preset_selection.values_mut(){*n=(*n).min(2);}
    for note in value.game_notes.values_mut(){
        if note.chars().count()>4000{*note=note.chars().take(4000).collect();}
    }
    value
}
pub fn save(s:&Settings) -> Result<(),String> {
    let dir=folder();
    fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let dest=dir.join("settings.json");
    atomic_json_replace(&dest,&serde_json::to_vec_pretty(s).map_err(|e|e.to_string())?)
}
/// Replace only after the new contents have reached the filesystem.
pub(crate) fn atomic_json_replace(dest:&std::path::Path,bytes:&[u8])->Result<(),String>{
    use std::{io::Write,os::windows::ffi::OsStrExt,sync::atomic::{AtomicU64,Ordering}};
    #[link(name="kernel32")]
    unsafe extern "system"{fn MoveFileExW(old:*const u16,new:*const u16,flags:u32)->i32;}
    static SEQUENCE:AtomicU64=AtomicU64::new(0);
    let temp=dest.with_extension(format!("tmp-{}-{}",std::process::id(),SEQUENCE.fetch_add(1,Ordering::Relaxed)));
    let result=(||{
        let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&temp).map_err(|e|e.to_string())?;
        file.write_all(bytes).map_err(|e|e.to_string())?;
        file.sync_all().map_err(|e|e.to_string())?;drop(file);
        let wide=|p:&std::path::Path|p.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<_>>();
        let a=wide(&temp);let b=wide(dest);
        if unsafe{MoveFileExW(a.as_ptr(),b.as_ptr(),0x1|0x8)}==0{
            return Err(format!("Settings replace failed (old file retained): {}",std::io::Error::last_os_error()));
        }
        Ok(())
    })();
    if result.is_err(){let _=fs::remove_file(temp);}
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn atomic_settings_replace_keeps_valid_file(){
        let root=std::env::temp_dir().join(format!("hb-settings-qa-{}",std::process::id()));
        fs::create_dir_all(&root).unwrap();let path=root.join("settings.json");
        atomic_json_replace(&path,b"{}").unwrap();
        atomic_json_replace(&path,b"{\"ok\":true}").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(),"{\"ok\":true}");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]fn precision_own_preview_limit_is_backward_compatible(){
        let mut settings=Settings::default();
        assert_eq!(settings.preview_fps_limit,45);
        settings.preview_fps_limit=90;
        let restored:Settings=serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.preview_fps_limit,90);
        let mut old=serde_json::to_value(settings).unwrap();
        old.as_object_mut().unwrap().remove("preview_fps_limit");
        assert_eq!(serde_json::from_value::<Settings>(old).unwrap().preview_fps_limit,45);
    }
    #[test]fn window_sizes_are_saved_and_default_to_compact(){
        let normal=Settings::default();
        assert_eq!(normal.main_window_size,[940.0,620.0]);
        assert!((normal.overlay_size_scale-0.88).abs()<0.01);
        let restored:Settings=serde_json::from_str(
            &serde_json::to_string(&normal).unwrap()).unwrap();
        assert_eq!(normal.main_window_size,restored.main_window_size);
        assert_eq!(normal.overlay_size_scale,restored.overlay_size_scale);
        let mut legacy=serde_json::to_value(normal).unwrap();
        legacy.as_object_mut().unwrap().remove("overlay_size_scale");
        legacy.as_object_mut().unwrap().remove("main_window_size");
        let migrated:Settings=serde_json::from_value(legacy).unwrap();
        assert_eq!(migrated.main_window_size,[940.0,620.0]);
        assert!((migrated.overlay_size_scale-0.88).abs()<0.01);
    }
    #[test] fn clamp_limits(){ let mut x=FilterProfile{vibrance:100.0,contrast:-8.0,brightness:3.0,sharpen:-3.0,intensity:20.0,enabled:false};
        x.clamp();assert_eq!(x.vibrance,1.0);assert_eq!(x.contrast,-0.5);assert_eq!(x.sharpen,0.0);}
    #[test] fn manual_gameplay_data_survives_restart(){
        let mut s=Settings::default();
        s.quest_checklists.insert("Zelda.exe".into(),vec![
            crate::gameplay::QuestTask{label:"Find shrine".into(),done:false}
        ]);
        s.cooldown_profiles.insert("Zelda.exe".into(),crate::gameplay::CooldownSpec{
            label:"Potion".into(),duration_seconds:45
        });
        let recovered:Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(recovered.quest_checklists["Zelda.exe"][0].label,"Find shrine");
        assert_eq!(recovered.cooldown_profiles["Zelda.exe"].duration_seconds,45);
    }
    #[test]fn native_os_color_mode_profiles_do_not_auto_activate(){
        let mut value=Settings::default();
        value.os_color_mode=9;
        value.os_color_strength=0.66;
        value.os_color_profiles.insert("TotK.exe".into(),4);
        let encoded=serde_json::to_string(&value).unwrap();
        let restored:Settings=serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored.os_color_mode,9);
        assert_eq!(restored.os_color_profiles.get("TotK.exe"),Some(&4));
        assert_eq!(restored.os_color_strength,0.66);
        // Runtime LiveColor.active is NOT serialized anywhere.
    }
    #[test] fn native_photo_presets_persist_per_game(){
        let mut initial=Settings::default();
        initial.native_preset=23;
        initial.native_preset_profiles.insert("TotK.exe".into(),13);
        let restored:Settings=serde_json::from_str(&serde_json::to_string(&initial).unwrap()).unwrap();
        assert_eq!(restored.native_preset,23);
        assert_eq!(restored.native_preset_profiles["TotK.exe"],13);
    }
    #[test] fn stutter_scout_defaults_off_and_is_backwards_compatible(){
        let mut old=serde_json::to_value(Settings::default()).unwrap();
        old.as_object_mut().unwrap().remove("stutter_scout_enabled");
        old.as_object_mut().unwrap().remove("stutter_threshold_ms");
        let loaded:Settings=serde_json::from_value(old).unwrap();
        assert!(!loaded.stutter_scout_enabled);
        assert_eq!(loaded.stutter_threshold_ms,50.0);
    }
    #[test] fn crosshair_per_game_profiles_survive_serialization(){
        let mut s=Settings::default();
        let mut aim=crate::crosshair::CrosshairOptions::default();
        aim.style=9;aim.outline=true;aim.opacity=180;
        s.crosshair_profiles.insert("C:\\\\Games\\\\totk.exe".into(),aim);
        let value=serde_json::to_string(&s).unwrap();
        let next:Settings=serde_json::from_str(&value).unwrap();
        let opt=next.crosshair_profiles.values().next().unwrap();
        assert_eq!(opt.style,9);assert_eq!(opt.opacity,180);assert!(opt.outline);
    }
    #[test] fn serde_roundtrip() {let v=Settings::default();let s=serde_json::to_string(&v).unwrap();
        let a:Settings=serde_json::from_str(&s).unwrap();assert_eq!(a.record_fps,30);assert_eq!(a.profile.vibrance,0.32);}
    #[test] fn comparison_profiles_survive_restart_and_migration(){
        let mut settings=Settings::default();
        let mut pair=crate::compare::LookPair::default();
        pair.store(0,crate::compare::LookSnapshot::capture(&settings)).unwrap();
        settings.live_stack=2;
        pair.store(1,crate::compare::LookSnapshot::capture(&settings)).unwrap();
        settings.look_pairs.insert("C:\\Games\\Zelda\\totk.exe".into(),pair);
        settings.compact_controls=false;
        let json=serde_json::to_string(&settings).unwrap();
        let reopened:Settings=serde_json::from_str(&json).unwrap();
        assert!(!reopened.compact_controls);
        let pair=reopened.look_pairs.get("C:\\Games\\Zelda\\totk.exe").unwrap();
        assert!(pair.stored(0)&&pair.stored(1));
        assert_eq!(pair.b.as_ref().unwrap().gpu.stack,2);
    }
    #[test] fn favorite_effects_validate_without_touching_legacy_profiles(){
        let mut ids=vec![3,2,3,usize::MAX,0,2];
        normalize_favorite_effects(&mut ids);
        assert_eq!(ids,vec![0,2,3]);
        let json=serde_json::to_string(&Settings::default()).unwrap();
        let reopened:Settings=serde_json::from_str(&json).unwrap();
        assert!(reopened.favorite_effects.is_empty());
    }
    #[test] fn capture_defaults_and_legacy_gpu_migration(){
        let old=serde_json::json!({"scaler":1,"effect":2,"strength":0.42,"stack":3});
        let mut gpu:GpuProfile=serde_json::from_value(old).unwrap();
        assert_eq!(gpu.capture_method,0);
        assert_eq!(gpu.output_mode,0);
        gpu.capture_method=999;gpu.output_mode=999;gpu.sanitize();
        assert_eq!(gpu.capture_method,crate::engine::CAPTURE_METHODS.len()-1);
        assert_eq!(gpu.output_mode,crate::engine::OUTPUT_MODES.len()-1);
    }
    #[test] fn old_v05_settings_migrate_without_losing_profiles(){
        let mut old=serde_json::to_value(Settings::default()).unwrap();
        for key in ["custom_profiles","custom_gpu_profiles","preset_selection","preview_image_path","game_notes","custom_stages","crosshair","crosshair_profiles","hero_photo_path","look_pairs","compact_controls","favorite_effects", "live_capture_method","live_output_mode","stutter_scout_enabled","stutter_threshold_ms"]{
            old.as_object_mut().unwrap().remove(key);
        }
        old["profiles"]=serde_json::json!({"C:\\Games\\zelda\\totk.exe":{
            "vibrance":0.37,"contrast":0.10,"brightness":0.04,
            "sharpen":0.31,"intensity":1.0,"enabled":false
        }});
        let loaded:Settings=serde_json::from_value(old).unwrap();
        assert_eq!(loaded.profiles.len(),1);
        assert!((loaded.profiles.values().next().unwrap().vibrance-0.37).abs()<1e-6);
        assert!(loaded.custom_profiles.is_empty());
        assert!(loaded.preset_selection.is_empty());
    }
}

#[cfg(test)]mod v102_zoom_tests{
 #[test]fn legacy_values_default_and_accessible_manual_scale_clamps(){
  let s:super::Settings=serde_json::from_str("{}").unwrap();
  assert!((s.ui_zoom-1.0).abs()<0.001);
  let mut json=serde_json::to_value(s).unwrap();
  json["ui_zoom"]=serde_json::json!(1.3);
  let x:super::Settings=serde_json::from_value(json).unwrap();
  assert!((x.ui_zoom-1.3).abs()<0.001);
 }
}

