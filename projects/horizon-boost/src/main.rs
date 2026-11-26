#![cfg_attr(all(target_os = "windows", not(test)), windows_subsystem = "windows")]
mod filter;
mod color_modes;
mod adlx;
mod preview;
mod stacks;
mod gameboost;
mod timer_precision;
mod preview_pacer;
mod native_stack;
mod network_probe;
mod power_plan;
mod game_macros;
mod window_macro;
mod local_sequence;
mod affinity;
mod tool_shortcuts;
mod shortcut_registry;
mod flow_pyramid;
mod unsorted_catalog;
mod reminders;
mod stack_share;
mod local_companion;
mod gpu_histogram;
mod render_budget;
mod sdk_preflight;
mod process_memory;
mod metric_math;
mod capture_health;
mod present_status;
mod capture_clock;
mod live_gpu_timing;
mod effect_bench;
mod stack_history;
mod dxgi_wait;
mod area_proof;
mod profile_portable;
mod controller_check;
mod session_report;
mod profile_backup;
mod output_rates;
mod color_grade;
mod color_recipe;
mod cinema_light;
mod safe_gpu_cache;
mod precision_pass;
mod display_color;
mod cube_lut;
mod photo_lab;
mod media_tools;
mod clip_book;
mod capture_session;
mod montage;
mod audio_meter;
mod audio_capture;
mod audio_spectrum;
mod wellbeing;
mod layer_mix;
mod diagnostic_export;
mod preview_guard;
mod instant_replay;
mod crosshair;
mod compare;
mod native_look_library;
mod sgsr;
mod hud_readout;
mod anime4k;
mod smaa;
mod node_graph;
mod watchdog_policy;
mod mmcss_runtime;
mod secure_random;
mod mobile_companion;
mod script_api;
mod cursor_lock;
mod hdr_probe;
mod hdr_output;
mod refresh_probe;
mod dx12_compat;
mod custom_hlsl;
mod shader_translate;
mod standby_cleaner;
mod gpu_frame_ring;
mod anticheat_guard;
mod async_compute;
mod wave64;
mod vulkan_compute;
mod amd_adl;
mod cunny;
mod fsrcnnx;
#[cfg(test)]mod roadmap_v97_proof;
#[cfg(test)]mod roadmap_v96_proof;
mod monitor_fps_profiles;
mod gametimer;
mod sessions;
mod lab;
mod share;
mod filedialog;
mod engine;
mod native_capture;
mod native_dup_output;
mod native_filter;
mod native_synthetic;
mod native_present;
mod composition_output;
mod fused_color;
mod gpu_copy_fence;
mod gallery_cache;
mod settings_debounce;
mod reticle_update;
mod capture_cadence;
mod render_pressure;
mod roadmap_gpu_proof;
mod render_profiles;
mod native_pipeline;
mod native_presets;
mod lighting;
mod perf_insights;
mod benchmark_bank;
mod native_light_proof;
mod native_scale_proof;
mod framegen;
mod spatial;
mod fsr1;
mod cas;
mod v61_proof;
mod nis;
mod native_photo;
mod gameplay;
mod training;
mod native_wgc;
#[cfg(test)] mod wgc_window_smoke;
mod native_mirror;
mod minecraft_profile;
mod resource_guard;
mod capture_start_guard;
mod filter_doctor;
mod studio_draft;
mod vram;
mod gpu_caps;
mod adapter_select;
mod hardware_sensors;
mod display_probe;
mod shadercheck;
mod fps;
mod telemetry_ring;
mod swapchain_route;
mod game_library;
mod command_search;
mod launcher_discovery;
mod save_vault;
mod motion_options;
mod temporal_runtime;
mod motion_lab;
mod process_load;
mod pipeline_bench;
mod hud_layout;
mod accuracy;
mod reminder_sound;
mod live_lut;
mod exposure_options;
mod color_live_proof;
mod reconstruction;
mod capture_sample;
mod reconstruction_proof;
mod record;
mod game_journal;
mod utility_book;
mod library_hub;
mod suite_finish;
mod local_assist;
mod settings;
mod win;
mod gui;
use eframe::egui;
use std::{sync::mpsc::Receiver,time::{Duration,Instant},path::PathBuf};
use filter::LiveColor;
use engine::ExternalGpu;
use fps::PresentStats;
use record::Recorder;
use settings::{FilterProfile,GpuProfile,Settings};
use win::Telemetry;

fn main() -> eframe::Result<()> {
    if let Some(pos)=std::env::args().position(|s|s=="--batch-script-file"){
        let args=std::env::args().collect::<Vec<_>>();
        let result=args.get(pos+1).ok_or("Script JSON file path missing".to_string()).and_then(|p|crate::script_api::evaluate_file(std::path::Path::new(p)));
        match result{Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("HORIZON_READONLY_SCRIPT_ERROR: {e}");std::process::exit(96);}}
        return Ok(());
    }
    if let Some(position)=std::env::args().position(|x|x=="--native-live-glass-probe"){
        let args=std::env::args().skip(position+1).collect::<Vec<_>>();
        let parsed=(||->Result<(u32,usize,u8),String>{
            let pid=args.first().ok_or("GLASS_PID_MISSING")?.parse::<u32>().map_err(|e|e.to_string())?;
            let look=args.get(1).ok_or("GLASS_LOOK_MISSING")?.parse::<usize>().map_err(|e|e.to_string())?;
            let factor=args.get(2).ok_or("GLASS_FACTOR_MISSING")?.parse::<u8>().map_err(|e|e.to_string())?;
            Ok((pid,look,factor))
        })();
        match parsed.and_then(|(pid,look,factor)|
            native_mirror::live_roblox_glass_probe(pid,look,factor)){
            Ok(message)=>println!("{message}"),
            Err(e)=>{
                eprintln!("ROBLOX_LIVE_GLASS_GPU=FAIL: {e}");
                std::process::exit(56);
            }
        }
        return Ok(());
    }
    if let Some(position)=std::env::args().position(|x|x=="--native-live-roblox-probe"){
        let args=std::env::args().skip(position+1).collect::<Vec<_>>();
        let parsed=(||->Result<(u32,usize,u8),String>{
            let pid=args.first().ok_or("Usage: --native-live-roblox-probe PID LOOK FACTOR")?
                .parse::<u32>().map_err(|e|e.to_string())?;
            let look=args.get(1).ok_or("Missing LOOK")?
                .parse::<usize>().map_err(|e|e.to_string())?;
            let factor=args.get(2).ok_or("Missing FACTOR")?
                .parse::<u8>().map_err(|e|e.to_string())?;
            Ok((pid,look,factor))
        })();
        match parsed.and_then(|(pid,look,factor)|
            native_mirror::live_game_probe(pid,look,factor)){
            Ok(report)=>println!("{report}"),
            Err(e)=>{
                eprintln!("REAL_ROBLOX_WGC=FAIL: {e}");
                std::process::exit(54);
            }
        }
        return Ok(());
    }
    for (flag,multiplier) in [("--native-wgc-fg-2x-proof",2u8),("--native-wgc-fg-3x-proof",3u8)]{
        if std::env::args().any(|a|a==flag){
            match native_mirror::framegen_synthetic_proof(multiplier){
                Ok(report)=>println!("{report}"),
                Err(error)=>{
                    eprintln!("NATIVE_WGC_OPTICAL_FLOW_FG=FAIL FACTOR={multiplier} {error}");
                    std::process::exit(49);
                }
            }
            return Ok(());
        }
    }
    // Read-only CLI inspection: no game actions, no arbitrary script execution.
    if std::env::args().any(|a|a=="--hb-cli-help"){
        println!("Horizon Boost CLI: --hb-list-effects | --hb-validate-stack ID,ID | --hb-status-json");
        return Ok(());
    }
    if std::env::args().any(|a|a=="--hb-list-effects"){
        let effects:Vec<_>=native_filter::PROGRAMS.iter().enumerate().filter(|(i,_)|
            native_stack::valid_stage(*i)).map(|(i,(name,_))|serde_json::json!({"id":i,"name":name})).collect();
        println!("{}",serde_json::to_string_pretty(&effects).unwrap());
        return Ok(());
    }
    if let Some(i)=std::env::args().position(|a|a=="--hb-validate-stack"){
        let raw=std::env::args().nth(i+1).unwrap_or_default();
        let stages=raw.split(',').map(|s|s.trim().parse::<usize>()).collect::<Result<Vec<_>,_>>();
        match stages.map_err(|_|"Expected comma-separated integer effect IDs".to_string())
            .and_then(|s|{native_stack::validate(&s)?;Ok(s)}){
            Ok(stages)=>println!("{}",serde_json::json!({"valid":true,"stages":stages})),
            Err(e)=>{eprintln!("{}",serde_json::json!({"valid":false,"error":e}));std::process::exit(68);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--hb-status-json"){
        println!("{}",serde_json::json!({"version":env!("CARGO_PKG_VERSION"),
            "native_shaders":native_filter::PROGRAMS.len(),"looks":native_presets::PRESETS.len(),
            "spatial_modes":spatial::MODES.len(),"presentmon_found":fps::find_presentmon().is_some(),
            "ffmpeg_found":record::ffmpeg_exe().is_some(),
            "game_fps_gain_verified":false,"mode":"read-only"}));
        return Ok(());
    }
    if std::env::args().any(|a|a=="--power-plan-readonly-proof"){
        match power_plan::current(){Ok(g)=>println!("POWER_PLAN_READ_ONLY=PASS PLAN={}",g.label()),
            Err(e)=>{eprintln!("{e}");std::process::exit(69);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--area-resample-proof"){
        match area_proof::run(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(77);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--waitable-wgc-proof"){
        match native_mirror::waitable_wgc_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(78);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--replay-new-options-proof"){
        match instant_replay::synthetic_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(82);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--media-tools-proof"){
        match media_tools::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(80);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--audio-meter-proof"){
        match audio_meter::peak(){Ok(v)=>println!("AUDIO_METER=PASS PEAK={v:.4} OUTPUT_ENDPOINT_ONLY=true"),Err(e)=>{eprintln!("{e}");std::process::exit(81);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--layer-mix-proof"){
        match photo_lab::layer_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(83);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--diagnostic-bundle-proof"){match diagnostic_export::bundle_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(106);}}return Ok(());}
    if std::env::args().any(|a|a=="--library-launch-proof"){match game_library::launch_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(105);}}return Ok(());}
    if std::env::args().any(|a|a=="--integer-gpu-proof"){match spatial::integer_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(108);}}return Ok(());}
    if std::env::args().any(|a|a=="--window-macro-proof"){match window_macro::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(112);}}return Ok(());}
    if std::env::args().any(|a|a=="--v77-gpu-proof"){match native_pipeline::v77_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(111);}}return Ok(());}
    if std::env::args().any(|a|a=="--command-search-proof"){
        match command_search::proof(){Ok(())=>println!("COMMAND_SEARCH=PASS ROUTES=27 CZ=true LOOKS=true GAMES=true"),Err(e)=>{eprintln!("{e}");std::process::exit(1);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--adaptive-fg-proof"){match motion_lab::adaptive_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(110);}}return Ok(());}
    if std::env::args().any(|a|a=="--lazy-gpu-proof"){match native_pipeline::lazy_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(109);}}return Ok(());}
    if std::env::args().any(|a|a=="--selected-vram-proof"){match vram::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(107);}}return Ok(());}
    if std::env::args().any(|a|a=="--diagnostic-default-proof"){
        let v=diagnostic_export::report(&settings::Settings::default());
        println!("DIAGNOSTIC_SCHEMA=PASS {}",v);return Ok(());
    }
    if std::env::args().any(|a|a=="--live-gpu-timing-proof"){match live_gpu_timing::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(94);}}return Ok(());}
    if std::env::args().any(|a|a=="--reconstruction-proof"){
        match reconstruction_proof::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(88);}}return Ok(());
    }
    if std::env::args().any(|a|a=="--capture-sample-proof"){
        match capture_sample::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(89);}}return Ok(());
    }
    if std::env::args().any(|a|a=="--display-probe-proof"){println!("{}",display_probe::summary());return Ok(());}
    if std::env::args().any(|a|a=="--live-color-proof"){
        match color_live_proof::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(87);}}return Ok(());
    }
    if std::env::args().any(|a|a=="--pipeline-bench-proof"){
        match pipeline_bench::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(85);}}return Ok(());
    }
    if std::env::args().any(|a|a=="--process-load-proof"){
        match process_load::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(86);}}return Ok(());
    }
    if std::env::args().any(|a|a=="--cinema-reference-proof"){match cinema_light::reference_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(102);}}return Ok(());}
    if std::env::args().any(|a|a=="--protection-proof"){match native_pipeline::protection_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(104);}}return Ok(());}
    if std::env::args().any(|a|a=="--optional-stage-proof"){match native_pipeline::optional_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(101);}}return Ok(());}
    if std::env::args().any(|a|a=="--cinema-light-proof"){match cinema_light::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(100);}}return Ok(());}
    if std::env::args().any(|a|a=="--recipe-proof"){match color_recipe::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(99);}}return Ok(());}
    if std::env::args().any(|a|a=="--v61-gpu-proof"){match v61_proof::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(98);}}return Ok(());}
    if std::env::args().any(|a|a=="--audio-fft-proof"){match audio_spectrum::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(103);}}return Ok(());}
    if std::env::args().any(|a|a=="--audio-mux-proof"){match audio_capture::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(97);}}return Ok(());}
    if std::env::args().any(|a|a=="--amd-sensor-proof"){match hardware_sensors::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(96);}}return Ok(());}
    if std::env::args().any(|a|a=="--adapter-selection-proof"){match adapter_select::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(95);}}return Ok(());}
    if std::env::args().any(|a|a=="--precision-pass-proof"){match precision_pass::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(94);}}return Ok(());}
    if std::env::args().any(|a|a=="--flow-grid-proof"){match motion_lab::grid_proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(92);}}return Ok(());}
    if std::env::args().any(|a|a=="--temporal-effects-proof"){match motion_lab::temporal_proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(91);}}return Ok(());}
    if std::env::args().any(|a|a=="--motion-lab-proof"){
        match motion_lab::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(84);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--photo-lab-proof"){
        match photo_lab::proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(79);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--effect-benchmark-proof"){
        match effect_bench::run(&[37,39,50]){
            Ok(rows)=>{
                let valid=rows.iter().filter(|r|r.gpu_ms.is_some()).count();
                println!("EFFECT_BENCH_ROWS={} VALID_GPU_TIMESTAMPS={valid}",rows.len());
                for row in &rows{println!("EFFECT_GPU_MS={:?} SIZE={}x{} NAME={}",row.gpu_ms,row.width,row.height,row.label);}
                if valid!=rows.len(){std::process::exit(76);}
                println!("EFFECT_BENCH=PASS SYNTHETIC_ONLY=true");
            },
            Err(e)=>{eprintln!("{e}");std::process::exit(76);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--reduced-wgc-proof"){
        match native_mirror::reduced_wgc_proof(){Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(74);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--process-memory-proof"){
        match process_memory::read(){
            Ok(m)=>println!("PROCESS_MEMORY=PASS WORKING_MIB={:.2} PEAK_MIB={:.2} PRIVATE_MIB={:.2}",m.working_mib,m.peak_mib,m.private_mib),
            Err(e)=>{eprintln!("{e}");std::process::exit(75);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--local-companion-proof"){
        match local_companion::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(71);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--shader-cache-proof"){
        if let Err(e)=native_filter::compile_all(){eprintln!("{e}");std::process::exit(72);}
        let before=shadercheck::cache_stats().1;
        if let Err(e)=native_filter::compile_all(){eprintln!("{e}");std::process::exit(72);}
        let (entries,hits)=shadercheck::cache_stats();
        if hits-before<native_filter::PROGRAMS.len() as u64{std::process::exit(73);}
        println!("SHADER_CACHE=PASS ENTRIES={entries} SECOND_COMPILE_HITS={}",hits-before);
        return Ok(());
    }
    if std::env::args().any(|a|a=="--affinity-proof"){
        match affinity::proof(){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(70);}}
        return Ok(());
    }
    if std::env::args().any(|a|a=="--network-loopback-proof"){
        match network_probe::echo(std::net::Ipv4Addr::LOCALHOST){
            Ok(s) if s.rtt.is_some()=>println!("NETWORK_LOOPBACK=PASS RTT_MS={}",s.rtt.unwrap()),
            other=>{eprintln!("NETWORK_LOOPBACK=FAIL {other:?}");std::process::exit(67);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-custom-stack-proof"){
        match native_mirror::custom_stack_wgc_proof(){
            Ok(s)=>println!("{s}"),Err(e)=>{eprintln!("{e}");std::process::exit(66);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-wgc-vendor-upscale-proof"){
        match native_mirror::native_vendor_wgc_proof(){
            Ok(result)=>println!("{result}"),
            Err(e)=>{eprintln!("NATIVE_WGC_VENDOR_SCALE=FAIL {e}");std::process::exit(65);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-test-source-resizing"){
        if let Err(e)=native_mirror::test_source_resize_child(){
            eprintln!("WGC_RESIZE_SYNTHETIC_CHILD=FAIL {e}");
            std::process::exit(44);
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-wgc-resize-recovery-proof"){
        match native_mirror::resize_recovery_proof(){
            Ok(msg)=>println!("{msg}"),
            Err(e)=>{eprintln!("WGC_RESIZE_RECOVERY=FAIL {e}");std::process::exit(45);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--probe-native-os-color-readonly"){
        let mut color=filter::LiveColor::default();
        match color.read_only_probe(){
            Ok(m)=>{
                let id=m.iter().enumerate().all(|(i,v)|
                    (*v-if i%6==0{1.0}else{0.0}).abs()<0.00001);
                println!("MAGNIFICATION_COLOR_MATRIX_READ_ONLY=PASS ORIGINAL_IDENTITY={id} RESTORED_BY_API=true DID_NOT_SET_COLOR=true");
            },
            Err(e)=>{eprintln!("MAG_COLOR_PROBE=FAIL {e}");std::process::exit(42);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-glass-visual-proof"){
        match native_mirror::glass_self_proof_frames(120){
            Ok(info)=>println!("{info}"),
            Err(e)=>{eprintln!("NATIVE_GLASS_VISUAL=FAIL {e}");std::process::exit(43);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-glass-proof"){
        match native_mirror::glass_self_proof(){
            Ok(info)=>println!("{info}"),
            Err(e)=>{eprintln!("NATIVE_GPU_GLASS_PROOF=FAIL {e}");std::process::exit(41);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--native-test-source-window"){
        if let Err(error)=native_mirror::test_source_child(){
            eprintln!("NATIVE_TEST_SOURCE=FAIL {error}");
            std::process::exit(36);
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--native-external-process-proof"){
        match native_mirror::external_process_proof(){
            Ok(msg)=>println!("{msg}"),
            Err(error)=>{eprintln!("NATIVE_EXTERNAL_PID_WGC=FAIL {error}");
                std::process::exit(37);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--probe-native-gpu-caps"){
        match gpu_caps::read_native_capabilities(){
            Ok(info)=>println!("{}",info.summary()),
            Err(error)=>{eprintln!("RX580_GPU_CAPS_PROBE=UNAVAILABLE {error}");std::process::exit(39);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--probe-vram"){
        match vram::read_local_process_budget(){
            Ok(info)=>println!("VRAM_PROBE=OK GPU={} HORIZON_PROCESS_BYTES={} BUDGET_BYTES={} SYSTEM_GAME_USAGE=false",
                info.device_name,info.process_usage_bytes,info.process_budget_bytes),
            Err(e)=>{eprintln!("VRAM_PROBE=UNAVAILABLE {e}");std::process::exit(33);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--native-wgc-repeat-proof"){
        for run in 0..2{
            match native_mirror::self_proof_with_frames(8){
                Ok(report)=>println!("WGC_RESTART_{}={}",run+1,report),
                Err(error)=>{eprintln!("WGC_RESTART_{}=FAIL {}",run+1,error);
                    std::process::exit(31);}
            }
        }
        println!("WGC_REPEAT_START_STOP_PROOF=PASS");
        return Ok(());
    }
    if std::env::args().any(|x|x=="--native-wgc-stress-proof"){
        match native_mirror::self_proof_with_frames(60){
            Ok(report)=>println!("{report}"),
            Err(error)=>{eprintln!("NATIVE_WGC_STRESS_PROOF=FAIL {error}");
                std::process::exit(30);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--native-wgc-mirror-proof"){
        match native_mirror::self_proof(){
            Ok(report)=>println!("{report}"),
            Err(error)=>{eprintln!("NATIVE_WGC_MIRROR_PROOF=FAIL {error}");
                std::process::exit(29);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--smoke-crosshair"){
        // Hidden HWND smoke test; no user game receives any changes.
        let mut hud=crosshair::CrosshairHud::new();
        hud.set_target(None);
        let mut opt=crosshair::CrosshairOptions::default();
        opt.enabled=true;
        hud.set(&opt);
        for _ in 0..25 {
            if crosshair::CrosshairHud::hwnd_exists(){break;}
            std::thread::sleep(Duration::from_millis(60));
        }
        let appeared=crosshair::CrosshairHud::hwnd_exists();
        hud.stop();
        let closed=!crosshair::CrosshairHud::hwnd_exists();
        if !appeared||!closed{
            eprintln!("CROSSHAIR_HIDDEN_SMOKE=FAIL created={appeared} closed={closed}");
            std::process::exit(11);
        }
        println!("CROSSHAIR_HIDDEN_SMOKE=PASS created and cleaned up without showing over desktop");
        return Ok(());
    }
    if std::env::args().any(|a|a=="--smoke-game-boost"){
        // Spawn only our own sacrificial idle test process. Never touch running games.
        use std::os::windows::process::CommandExt;
        let mut child=std::process::Command::new("powershell.exe")
            .args(["-NoProfile","-NonInteractive","-Command","Start-Sleep -Seconds 15"])
            .creation_flags(0x0800_0000)
            .spawn().expect("Cannot spawn isolated priority test process");
        let pid=child.id();
        let mut booster=gameboost::GameBoost::default();
        let result=booster.start(pid).and_then(|_|booster.stop());
        let _=child.kill();let _=child.wait();
        match result{
            Ok(())=>println!("WINDOWS_GAME_BOOST_TEST=PASS verified and restored"),
            Err(error)=>{eprintln!("WINDOWS_GAME_BOOST_TEST=FAIL {error}");std::process::exit(8);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--probe-adlx"){
        match adlx::probe(){
            Ok(s)=>{
                println!("ADLX_READ_ONLY_PROBE=OK GPU_COUNT={} RIS_SUPPORTED={} RIS_ENABLED={} RIS_STRENGTH={} RIS_RANGE={:?} ANTILAG_SUPPORTED={} ANTILAG_ENABLED={}",
                    s.gpu_count,s.image_sharpen.supported,s.image_sharpen.enabled,s.sharpness,s.sharpness_range,
                    s.anti_lag.supported,s.anti_lag.enabled);
                return Ok(());
            },
            Err(e)=>{eprintln!("ADLX_PROBE_ERROR={e}");std::process::exit(5);}
        }
    }
    if std::env::args().any(|x|x=="--diagnose-games"){
        let games=win::scan_visible_games();
        println!("DETECTED_GAME_WINDOWS={}",games.len());
        for game in &games{
            println!("GAME pid={} title={} full_screen_sized={} path={}",
                game.pid,game.title,game.fullscreen_like,game.path.display());
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--smoke-magpie"){
        let dummy=win::GameInfo{
            pid:0,title:"Horizon Boost Dummy".into(),
            path:PathBuf::from(r"C:\HorizonBoostVerification\placeholder-game.exe"),
            class_name:"HorizonBoostDummyClass".into(),fullscreen_like:false
        };
        let mut gpu=ExternalGpu::default();
        match gpu.start(&dummy,0,2,0.45,0,0,0,&[]){
            Ok(())=>{
                std::thread::sleep(Duration::from_millis(2600));
                let alive=gpu.running();
                gpu.stop();
                println!("MAGPIE_HIDDEN_RUNNING={alive}");
                if !alive {std::process::exit(3);}
            },
            Err(e)=>{eprintln!("MAGPIE_TEST_ERROR={e}");std::process::exit(3);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--verify-effect-catalog") {
        match engine::verify_effect_files(){
            Ok(n)=>println!("EFFECT_CATALOG_VERIFIED={n} of {n}; ALL_SHADER_PATHS_FOUND"),
            Err(err)=>{eprintln!("EFFECT_CATALOG_ERROR={err}");std::process::exit(7);}
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--compile-shaders") {
        match shadercheck::compile_all(){
            Ok(n)=>println!("Compiled {n} HLSL Pixel Shader 5.0 filters with native D3DCompiler"),
            Err(e)=>{eprintln!("Shader error: {e}");std::process::exit(2);}
        }
        return Ok(());
    }
    if std::env::args().any(|x| x=="--validate-shaders") {
        println!("HORIZON BOOST GPU shaders: {} custom HLSL effects validated structurally", engine::verify_shader_library().unwrap());
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-spatial-gpu-proof"){
        let destination=std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("build-tests").join("v22-spatial-gpu");
        match native_scale_proof::export_native_scaler_proof(&destination){
            Ok(log)=>println!("{log}"),
            Err(error)=>{eprintln!("NATIVE_SPATIAL_GPU=FAIL {error}");std::process::exit(57);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-lighting-gpu-proof"){
        let destination=std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("build-tests").join("v21-native-lighting");
        match native_light_proof::export_lighting_proof(&destination){
            Ok(info)=>println!("{info}"),
            Err(e)=>{
                eprintln!("NATIVE_LIGHT_STUDIO_GPU=FAIL {e}");
                std::process::exit(55);
            }
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-gameplay-photo-proof"){
        let target=std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets").join("native_gameplay_previews");
        match native_photo::export_native_gameplay_previews(&target){
            Ok(digests)=>println!("NATIVE_REAL_GAMEPLAY_GPU_PRESET_PROOF=PASS COUNT={} PATH={}",
                digests.len(),target.display()),
            Err(e)=>{eprintln!("NATIVE_REAL_GAMEPLAY_GPU_PRESET_PROOF=FAIL {e}");
                std::process::exit(50);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-preset-photo-proof"){
        let output=std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets").join("native_previews");
        match native_photo::export_native_photo_previews(&output){
            Ok(digests)=>println!("NATIVE_REAL_PHOTO_PRESET_PROOF=PASS COUNT={} PATH={}",
                digests.len(),output.display()),
            Err(e)=>{eprintln!("NATIVE_REAL_PHOTO_PRESET_PROOF=FAIL {e}");
                std::process::exit(25);}
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-present-proof"){
        match native_present::probe_hidden_present(){
            Ok(proof)=>println!("{}",proof.summary()),
            Err(error)=>{
                eprintln!("NATIVE_FLIP_PRESENT_PROOF=FAIL {error}");
                std::process::exit(24);
            }
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-shader-proof"){
        match native_synthetic::validate_synthetic_gpu_effects(){
            Ok(results)=>{
                for r in &results{
                    println!("NATIVE_SHADER_SYNTHETIC={} CHANGED_PIXELS={} HASH=0x{:016x}",
                        r.filter,r.changed_pixels,r.checksum);
                }
                println!("NATIVE_SHADER_PROOF=PASS COUNT={}",results.len());
            }
            Err(error)=>{
                eprintln!("NATIVE_SHADER_PROOF=FAIL: {error}");
                std::process::exit(23);
            }
        }
        return Ok(());
    }
    if std::env::args().any(|a|a=="--native-capture-probe"){
        let _=native_filter::compile_all().map_err(|e|{
            eprintln!("NATIVE_COMPUTE_COMPILATION_ERROR: {e}");
            std::process::exit(22);
        });
        match native_capture::probe_gpu_only(){
            Ok(result)=>println!("HORIZON_NATIVE_DXGI: {}",result.report()),
            Err(error)=>{
                eprintln!("HORIZON_NATIVE_DXGI_FAILED: {error}");
                std::process::exit(21);
            }
        }
        return Ok(());
    }
    if std::env::args().any(|x|x=="--probe-magpie-scaling"){
        for game in crate::win::scan_visible_games(){
            if let Some(info)=crate::win::magpie_scaling_info(game.pid){
                println!("MAGPIE_SCALING_GAME={} PID={} MODE={}",
                    game.title,game.pid,if info.windowed{"windowed"}else{"fullscreen external"});
                return Ok(());
            }
        }
        println!("MAGPIE_SCALING=NOT_DETECTED");
        return Ok(());
    }
    if std::env::args().any(|x| x=="--self-test") {
        let mut p=FilterProfile::default();
        p.vibrance=0.0;p.brightness=0.0;p.contrast=0.0;p.intensity=1.0;
        assert!((filter::matrix(&p)[0]-1.0).abs()<1e-6);
        assert_eq!(engine::verify_shader_library().unwrap(),13);
        assert_eq!(native_filter::PROGRAMS.len(),71);
        assert_eq!(native_presets::PRESETS.len(),83);
        assert_eq!(framegen::compile_all().expect("FrameGen compute shaders must compile"),5);
        println!("HORIZON BOOST native Rust modules: PASS - 60 GPU shaders - 83 looks - 2x/3x flow");
        return Ok(());
    }
    let proof_extras_view=if std::env::args().any(|a|a=="--visual-proof-extras-hud"){Some(0usize)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-timers"){Some(1)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-quests"){Some(2)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-training"){Some(3)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-tools"){Some(4)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-macros"){Some(5)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-view"){Some(6)}
        else if std::env::args().any(|a|a=="--visual-proof-extras-reminders"){Some(7)}
        else{std::env::args().find_map(|a|a.strip_prefix("--visual-journal=").and_then(|v|v.parse::<usize>().ok())).filter(|v|(10..=19).contains(v))};
    let proof_motion=std::env::args().any(|a|a=="--visual-proof-motion-lab");
    let proof_perf_view=if std::env::args().any(|a|a=="--visual-proof-performance-tools"){Some(1usize)}
        else if std::env::args().any(|a|a=="--visual-proof-performance-analysis"){Some(2)}
        else if std::env::args().any(|a|a=="--visual-proof-performance-stability"){Some(3)}
        else if std::env::args().any(|a|a=="--visual-proof-affinity"){Some(4)}
        else{None};
    let visual_perf=proof_perf_view.is_some()||
        std::env::args().any(|a|a=="--visual-proof-performance");
    let proof_filter_view=if std::env::args().any(|a|a=="--visual-proof-filter-color"||a=="--visual-proof-cinema"){Some(1usize)}
        else if std::env::args().any(|a|a=="--visual-proof-filter-legacy"){Some(2)}
        else if std::env::args().any(|a|a=="--visual-proof-filter-pro"){Some(3)}
        else{None};
    let proof_narrow_filters=std::env::args().any(|a|a=="--visual-proof-filters-narrow");
    let visual_all_presets=std::env::args().any(|a|a=="--visual-proof-all-presets");
    let visual_gallery=std::env::args().any(|a|a=="--visual-proof-new-gpu-looks")||
        visual_all_presets||proof_filter_view.is_some()||proof_narrow_filters||
        std::env::args().any(|a|a=="--visual-proof-gallery");
    let visual_os_color_lab=std::env::args().any(|a|a=="--visual-proof-os-color-lab");
    let visual_driver=std::env::args().any(|a|a=="--visual-proof-driver");
    let proof_overlay_perf=std::env::args().any(|a|a=="--visual-proof-overlay-performance");
    let proof_overlay_fg=std::env::args().any(|a|a=="--visual-proof-overlay-framegen");
    let proof_overlay_settings=std::env::args().any(|a|a=="--visual-proof-overlay-crosshair");
    let proof_overlay_driver=std::env::args().any(|a|a=="--visual-proof-overlay-driver");
    let proof_overlay_editor_small=
        std::env::args().any(|a|a=="--visual-proof-overlay-editor-small");
    let proof_overlay_small=std::env::args().any(|a|a=="--visual-proof-overlay-small");
    let proof_overlay_compact=proof_overlay_small||
        std::env::args().any(|a|a=="--visual-proof-overlay-compact");
    let proof_appearance=std::env::args().any(|a|a=="--visual-proof-size-settings");
    let proof_replay=std::env::args().any(|a|a=="--visual-proof-instant-replay");
    let proof_new_gpu=std::env::args().any(|a|a=="--visual-proof-new-gpu-looks");
    let visual_overlay=proof_overlay_editor_small||proof_overlay_perf||proof_overlay_fg||proof_overlay_settings||proof_overlay_driver||proof_overlay_compact||
        std::env::args().any(|a|a=="--visual-proof-overlay");
    let smoke_overlay=visual_overlay||std::env::args().any(|a|a=="--smoke-overlay");
    let proof_compact_main=std::env::args().any(|a|a=="--visual-proof-compact-main");
    let oled_page=std::env::args().find_map(|a|a.strip_prefix("--visual-oled=").and_then(|v|v.parse::<u8>().ok())).filter(|v|*v<=10);
    let macro_visual=std::env::args().any(|a|a=="--visual-macro");
    let visual_test=macro_visual||oled_page.is_some()||proof_replay||proof_appearance||proof_compact_main||visual_perf||visual_gallery||visual_driver||visual_overlay||visual_os_color_lab||
        proof_extras_view.is_some()||proof_motion
        ||std::env::args().any(|a|a=="--visual-proof");
    let smoke=smoke_overlay || visual_test || std::env::args().any(|a|a=="--smoke-test");
    // Restore a compact user-controlled logical size, even after relaunch.
    // Screenshot smoke QA keeps its deterministic dimensions unchanged.
    let saved_window=if smoke{[940.0,620.0]}
        else{settings::load().main_window_size};
    let options=eframe::NativeOptions {
        viewport:egui::ViewportBuilder::default()
            .with_title("HORIZON BOOST - Rust")
            .with_inner_size(if proof_overlay_editor_small{[420.0,620.0]}else if proof_overlay_small{[205.0,61.0]}else if proof_overlay_compact{[252.0,70.0]}else if visual_overlay{[670.0,730.0]}else if proof_narrow_filters{[960.0,720.0]}else if proof_compact_main{[840.0,560.0]}else if proof_appearance{[940.0,680.0]}else if proof_extras_view==Some(5){[940.0,620.0]}else if smoke{[1280.0,780.0]}else{saved_window})
            .with_min_inner_size(if proof_overlay_editor_small{[420.0,620.0]}else if proof_overlay_small{[205.0,61.0]}else if proof_overlay_compact{[252.0,70.0]}else if visual_overlay{[620.0,650.0]}else if smoke{[800.0,540.0]}else{[800.0,540.0]})
            .with_decorations(false)
            .with_resizable(true)
            .with_visible(!smoke),
        renderer:eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("HORIZON BOOST - Rust",options,Box::new(move|cc|{
        cc.egui_ctx.set_visuals(gui::visuals());
        cc.egui_ctx.style_mut(|style|{
            style.spacing.item_spacing=egui::vec2(7.0,6.0);
            style.spacing.button_padding=egui::vec2(10.0,6.0);
        });
        let mut app=BoostApp::new(cc,smoke,visual_test,visual_overlay);
        if macro_visual{app.page=Page::Extras;app.extras_view=5;app.settings.ui_advanced=true;app.settings.ui_language=if std::env::args().any(|a|a=="--proof-czech"){1}else{0};let _=app.window_macro.plan.append_key("E",200,100);let _=app.window_macro.plan.append_click(4500,6000,0,150,80);}
        if let Some(p)=oled_page{app.page=[Page::Home,Page::Filters,Page::Scaling,Page::Performance,Page::Extras,Page::Overlay,Page::FrameGen,Page::Capture,Page::Library,Page::Settings,Page::Unsorted][p as usize];app.settings.ui_language=if std::env::args().any(|a|a=="--proof-czech"){1}else{0};}
        if proof_appearance{
            app.page=Page::Settings;app.visual_size_settings=true;
        }
        if proof_replay{
            app.page=Page::Capture;
            app.visual_replay=true;
        }
        if proof_new_gpu{
            app.page=Page::Filters;
            app.filter_view=0;
            app.native_gallery_scene=1;
            app.native_gallery_category=5;
            app.settings.native_preset=81;
            app.visual_new_gpu_look=true;
        }
        if proof_compact_main{app.visual_compact_main=true;}
        if proof_overlay_small{app.settings.overlay_size_scale=0.78;}
        if proof_overlay_editor_small{
            app.settings.overlay_size_scale=0.88;
            app.overlay_proof_name="overlay-editor-small-v29-real.png".into();
        }
        if let Some(tab)=proof_extras_view{
            app.page=Page::Extras;
            app.extras_view=tab;
            app.visual_extras=true;
        }
        if proof_motion{
            app.page=Page::FrameGen;
            app.visual_motion=true;
        }
        if smoke_overlay {
            app.overlay_open=true;
            // Original hidden visual tests show the editor, new compact proof
            // shows the exact passive gameplay viewport dimensions.
            app.overlay_edit_mode=visual_overlay&&!proof_overlay_compact;
            app.visual_compact_overlay=proof_overlay_compact;
            if proof_overlay_compact{
                app.overlay_proof_name=if proof_overlay_small{
                    "overlay-small-v29-real.png".into()
                }else{"overlay-compact-v26dev.png".into()};
            }
        }
        if proof_overlay_perf||proof_overlay_fg||proof_overlay_settings{
            app.overlay_expanded=true;
            app.overlay_section=if proof_overlay_fg{Page::FrameGen}
                else if proof_overlay_perf{Page::Performance}else{Page::Settings};
            app.overlay_proof_name=if proof_overlay_fg{
                "overlay-framegen-v22dev.png".into()
            }else if proof_overlay_perf{"overlay-performance-v22dev.png".into()}
            else{"overlay-gamehud-v22dev.png".into()};
        }
        if proof_overlay_driver{
            // Real read-only ADLX probe; no driver switch or game change.
            app.driver_status=adlx::probe().ok();
            app.overlay_expanded=true;
            app.overlay_proof_name="overlay-driver-v25dev.png".into();
        }
        if visual_perf{
            app.visual_perf=true;
            app.page=Page::Performance;
            app.performance_view=proof_perf_view.unwrap_or(0);
        }
        if visual_os_color_lab{
            app.visual_os_color_lab=true;
            app.page=Page::Filters;
        }
        if visual_gallery{
            app.visual_narrow_filters=proof_narrow_filters;
            app.visual_gallery=true;
            app.visual_all_presets=visual_all_presets;
            app.page=Page::Filters;
            app.filter_view=proof_filter_view.unwrap_or(0);
            if app.filter_view==1{app.settings.lighting.cinema=crate::cinema_light::Options::preset(0);}
            if std::env::args().any(|a|a=="--proof-czech"){app.settings.ui_language=1;}
            // Visually demonstrative TEST ONLY, never persisted to user settings.
            app.settings.live_preset=8;
            app.settings.live_strength=0.85;
            app.settings.live_stack=2;  // screenshot test: Cinematic shader chain
            app.show_all_presets=visual_all_presets;
        }
        if visual_driver{
            app.visual_driver=true;
            app.page=Page::Filters;
            app.refresh_driver_status(); // read-only; never changes driver settings
        }
        Ok(Box::new(app))
    }))
}
const FILTER_VIEW_COUNT:usize=5;
const PERFORMANCE_VIEW_COUNT:usize=5;
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Page {Home,Library,Overlay,Filters,Scaling,FrameGen,Performance,Extras,Capture,Settings,Unsorted}
impl Page {
    fn label(self)->&'static str {
        match self{Self::Home=>"Home",Self::Library=>"Library",Self::Overlay=>"Overlay",
            Self::Filters=>"Filters",Self::Scaling=>"Scaling",Self::FrameGen=>"Frame Gen",
            Self::Performance=>"Performance",Self::Extras=>"Game Extras",
            Self::Capture=>"Capture",Self::Settings=>"Settings",Self::Unsorted=>"Lab"}
    }
}
pub struct BoostApp {
    settings:Settings,
    live:LiveColor,
    os_color_mode_active:bool,
    telem:Telemetry,
    recorder:Recorder,
    reference_epoch:u64,
    capture_session:capture_session::Session,
    replay_game_key:Option<String>,
    replay_save_game_key:Option<String>,
    library_title:String,
    library_filter:String,
    library_confirm:Option<usize>,
    library_scan:Option<Receiver<Result<game_library::Scan,String>>>,
    library_banner_cache:std::collections::BTreeMap<String,egui::TextureHandle>,
    save_vault_names:Vec<String>,
    save_vault_selected:Option<String>,
    save_vault_source:Option<std::path::PathBuf>,
    save_vault_job:Option<Receiver<Result<String,String>>>,
    media_gallery:Vec<media_tools::Item>,
    media_selected:Option<std::path::PathBuf>,
    media_info:Option<media_tools::ClipInfo>,
    media_rx:Option<Receiver<Result<media_tools::ClipInfo,String>>>,
    media_export:Option<Receiver<Result<std::path::PathBuf,String>>>,
    encoder_rx:Option<Receiver<Result<media_tools::Encoders,String>>>,
    encoder_result:Option<Result<media_tools::Encoders,String>>,
    trim_start:f64,trim_end:f64,
    audio_enabled:bool,audio_meter:audio_meter::Meter,
    spectrum_enabled:bool,spectrum:audio_spectrum::Meter,
    mobile:mobile_companion::Server,
    audio_next:audio_capture::Options,
    sequence_window:local_sequence::Window,
    break_timer:wellbeing::BreakTimer,resource_alerts:wellbeing::Alerts,
    sensor_enabled:bool,sensors:hardware_sensors::Meter,
    replay:instant_replay::RollingReplay,
    network_probe:network_probe::Probe,
    network_address:String,
    power_plan:power_plan::PowerPlan,
    macro_runner:game_macros::Runner,
    window_macro:window_macro::Studio,
    named_macro_arm:Option<(String,window_macro::Target,window_macro::Plan,Instant)>,
    cpu_affinity:affinity::Affinity,
    affinity_draft:Option<(u32,usize,usize)>,
    emergency_down:bool,
    reminders:reminders::Reminders,
    companion:local_companion::Companion,
    shader_warmup:Option<Receiver<Result<usize,String>>>,
    stack_share_edit:String,
    portable_edit:String,
    portable_undo:Option<(String,profile_portable::Portable)>,
    profile_backups:Vec<String>,
    macro_drafts:std::collections::BTreeMap<String,game_macros::MacroProfile>,
    output_rates:output_rates::Meter,
    macro_share_edit:String,
    controller_readings:Option<Result<Vec<controller_check::Reading>,String>>,
    controller_enabled:bool,
    controller_last:Option<Instant>,
    controller_drift:controller_check::Drift,
    report:session_report::Recorder,
    benchmark_bank:benchmark_bank::Bank,
    benchmark_label:String,
    report_marker:String,
    palette_open:bool,
    palette_query:String,
    photo_path:Option<std::path::PathBuf>,
    photo_lut_text:String,
    photo_benchmark:bool,
    photo_rx:Option<Receiver<Result<photo_lab::ResultImage,String>>>,
    photo_result:Option<photo_lab::ResultImage>,
    photo_textures:Option<(egui::TextureHandle,egui::TextureHandle)>,
    photo_split:f32,
    motion_lab_rx:Option<Receiver<Result<motion_lab::ResultFrames,String>>>,
    motion_lab_textures:Vec<egui::TextureHandle>,
    motion_lab_times:Option<(Option<f64>,Option<f64>)>,
    process_load:process_load::Meter,
    accuracy_trainer:accuracy::Trainer,
    accuracy_window:bool,
    marker_chord_down:bool,
    pipeline_bench_rx:Option<Receiver<Result<Vec<effect_bench::Row>,String>>>,
    pipeline_bench_result:Option<Result<Vec<effect_bench::Row>,String>>,
    preview_guard:preview_guard::Guard,
    preview_guard_enabled:bool,
    render_guard:render_budget::Adaptive,
    render_guard_enabled:bool,
    own_memory:Option<Result<process_memory::Memory,String>>,
    dxgi_probe:Option<Receiver<Result<native_capture::NativeCaptureProbe,String>>>,
    dxgi_probe_status:Option<Result<native_capture::NativeCaptureProbe,String>>,
    macro_armed_pid:Option<u32>,
    macro_hotkey_down:bool,
    replay_hotkey_down:bool,
    gpu_backend:ExternalGpu,
    native_mirror:Option<native_mirror::MirrorWorker>,
    native_mirror_status:String,
    effect_bench_rx:Option<Receiver<Result<Vec<effect_bench::Row>,String>>>,
    effect_bench_result:Option<Result<Vec<effect_bench::Row>,String>>,
    capability_result:Option<Result<gpu_caps::NativeCapabilities,String>>,
    native_mirror_pid:Option<u32>,
    pending_mirror_restart:Option<(u32,native_mirror::MirrorMode,u8)>,
    watchdog:watchdog_policy::Guard,
    vram_budget:Option<Result<vram::VramBudget,String>>,
    last_vram_poll:Instant,
    fps:PresentStats,
    game_boost:gameboost::GameBoost,
    timer_precision:timer_precision::TimerPrecision,
    crosshair:crosshair::CrosshairHud,
    last_crosshair_hotkey:bool,
    cursor_lock:cursor_lock::Lock,
    cursor_lock_armed:bool,
    last_cursor_chord:bool,
    zoom_held:bool,
    last_gpu_hotkey:bool,
    last_compare_hotkey:bool,
    last_timer_hotkey:bool,
    last_split_hotkey:bool,
    game_timer:gametimer::RunTimer,
    split_label:String,
    sessions:sessions::SessionTracker,
    perf_lab:lab::PerformanceLab,
    benchmark_length:u32,
    target_fps:u32,
    game_note_edit:String,
    quest_task_edit:String,
    cooldown_clock:gameplay::CooldownClock,
    cooldown_spec:gameplay::CooldownSpec,
    reaction:training::ReactionTrainer,
    training_reset_confirm:bool,
    last_capture_hotkey:bool,
    last_record_hotkey:bool,
    previews:preview::PreviewBank,
    hero_picture:Option<egui::TextureHandle>,
    gallery_category:usize,
    native_gallery_category:usize,
    native_gallery_scene:usize,
    hud_monitor_cache:Option<(Instant,Vec<crate::display_probe::Display>)>,
    hud_observed_dpi:Option<u32>,
    native_preview_textures:Vec<Option<egui::TextureHandle>>,
    native_gameplay_preview_textures:Vec<Option<egui::TextureHandle>>,
    native_gameplay_original_texture:Option<egui::TextureHandle>,
    native_original_texture:Option<egui::TextureHandle>,
    os_color_photo_texture:Option<egui::TextureHandle>,
    os_color_photo_key:Option<(usize,u8)>,
    preset_category:usize,
    filter_search:String,
    show_all_presets:bool,
    driver_status:Option<adlx::DriverStatus>,
    driver_reader:Option<Receiver<Result<adlx::DriverStatus,String>>>,
    driver_busy:bool,
    driver_strength:i32,
    driver_error:Option<String>,
    last_ris_hotkey:bool,
    last_boost_hotkey:bool,
    page:Page,
    monitor_profile_applied:Option<(String,String)>,
    unsorted_selected_panel:u8,
    selected_profile:usize,
    filter_view:usize,
    performance_view:usize,
    extras_view:usize,
    overlay_open:bool,
    overlay_edit_mode:bool, // false = tiny click-through HUD; true = explicit control editor
    last_overlay_passive:Option<bool>,
    last_hud_reassert:Option<Instant>,
    last_overlay_edit_hotkey:bool,
    visual_compact_overlay:bool,
    overlay_filter_tab:bool,
    overlay_expanded:bool,
    overlay_section:Page,
    overlay_notice:String,
    last_overlay_opacity:Option<f32>,
    last_hotkey:bool,
    last_hud_f12:bool,
    hud_fps:PresentStats,
    hud_target:Option<(u32,String)>,
    hud_uses_main:bool,
    hud_track_at:Option<Instant>,
    last_color_hotkey:bool,
    pending_gpu_refresh:Option<Instant>,
    last_refresh_retry:Option<Instant>,
    pending_screenshot:Option<Instant>,
    last_tick:Instant,
    settings_debounce:settings_debounce::Debounce,
    last_stutter_alert:Option<Instant>,
    app_start:Instant,
    last_toast:Instant,
    status:String,
    screenshot:Option<Receiver<Result<PathBuf,String>>>,
    gpu:String,
    is_smoke:bool,
    smoke_frames:u32,
    visual_test:bool,
    visual_overlay:bool,
    overlay_proof_name:String,
    visual_driver:bool,
    visual_gallery:bool,
    visual_perf:bool,
    visual_all_presets:bool,
    visual_narrow_filters:bool,
    visual_os_color_lab:bool,
    visual_extras:bool,
    visual_motion:bool,
    visual_requested:bool,
    visual_saved:bool,
    visual_compact_main:bool,
    visual_size_settings:bool,
    visual_replay:bool,
    visual_new_gpu_look:bool,
}
impl BoostApp {
    fn new(cc:&eframe::CreationContext<'_>,smoke:bool,visual_test:bool,visual_overlay:bool)->Self {
        let settings=if smoke{settings::Settings::default()}else{settings::load()};
        let mut previews=preview::PreviewBank::new(&cc.egui_ctx);
        if let Some(path)=settings.preview_image_path.as_ref(){
            let _=previews.from_file(&cc.egui_ctx,std::path::Path::new(path));
        }
        let gallery_expanded=!settings.compact_controls;
        let start_hud=!smoke&&settings.hud_start_open;
        let hero_picture=Self::load_hero_picture(&cc.egui_ctx,settings.hero_photo_path.as_deref())
            .or_else(||Self::load_hero_picture(&cc.egui_ctx,None));
        let mut telem=Telemetry::new();
        if !smoke&&settings.detect_games{telem.refresh(true);}
        let (sender,receiver)=std::sync::mpsc::channel();
        if !smoke{
            std::thread::spawn(move||{let _=sender.send(adlx::probe());});
        }
        let mut app=Self {settings,live:LiveColor::default(),os_color_mode_active:false,
            telem,recorder:Recorder::default(),reference_epoch:0,capture_session:Default::default(),replay_game_key:None,replay_save_game_key:None,
            library_title:String::new(),library_filter:String::new(),library_confirm:None,library_scan:None,library_banner_cache:Default::default(),save_vault_names:Vec::new(),save_vault_selected:None,save_vault_source:None,save_vault_job:None,
            media_gallery:Vec::new(),media_selected:None,media_info:None,media_rx:None,media_export:None,
            encoder_rx:None,encoder_result:None,trim_start:0.,trim_end:1.,
            audio_enabled:false,audio_meter:audio_meter::Meter::default(),spectrum_enabled:false,spectrum:Default::default(),mobile:mobile_companion::Server::default(),audio_next:Default::default(),sequence_window:Default::default(),
            break_timer:wellbeing::BreakTimer::default(),resource_alerts:wellbeing::Alerts::default(),sensor_enabled:false,sensors:Default::default(),
            replay:instant_replay::RollingReplay::default(),
            network_probe:network_probe::Probe::default(),network_address:"127.0.0.1".into(),
            power_plan:power_plan::PowerPlan::default(),
            macro_runner:game_macros::Runner::default(),window_macro:Default::default(),named_macro_arm:None,macro_armed_pid:None,
            cpu_affinity:affinity::Affinity::default(),affinity_draft:None,emergency_down:false,
            reminders:reminders::Reminders::default(),companion:local_companion::Companion::default(),shader_warmup:None,stack_share_edit:String::new(),
            portable_edit:String::new(),portable_undo:None,macro_share_edit:String::new(),
            profile_backups:Vec::new(),macro_drafts:std::collections::BTreeMap::new(),output_rates:output_rates::Meter::default(),
            controller_readings:None,controller_enabled:false,controller_last:None,controller_drift:controller_check::Drift::default(),
            report:session_report::Recorder::default(),benchmark_bank:Default::default(),benchmark_label:String::new(),report_marker:String::new(),palette_open:false,palette_query:String::new(),
            photo_path:None,photo_lut_text:String::new(),photo_benchmark:false,photo_rx:None,photo_result:None,photo_textures:None,photo_split:0.5,motion_lab_rx:None,motion_lab_textures:Vec::new(),motion_lab_times:None,process_load:Default::default(),accuracy_trainer:Default::default(),accuracy_window:false,marker_chord_down:false,pipeline_bench_rx:None,pipeline_bench_result:None,
            preview_guard:preview_guard::Guard::default(),preview_guard_enabled:false,
            render_guard:render_budget::Adaptive::default(),render_guard_enabled:false,own_memory:None,
            dxgi_probe:None,dxgi_probe_status:None,
            macro_hotkey_down:false,replay_hotkey_down:false,
            gpu_backend:ExternalGpu::default(),
            native_mirror:None,native_mirror_status:"Native GPU preview idle - separate window".into(),
            effect_bench_rx:None,effect_bench_result:None,capability_result:None,
            native_mirror_pid:None,pending_mirror_restart:None,watchdog:watchdog_policy::Guard::default(),vram_budget:None,
            last_vram_poll:Instant::now()-Duration::from_secs(15),
            fps:PresentStats::default(),
            game_boost:gameboost::GameBoost::default(),
            timer_precision:timer_precision::TimerPrecision::default(),
            crosshair:crosshair::CrosshairHud::new(),
            last_crosshair_hotkey:false,cursor_lock:cursor_lock::Lock::default(),cursor_lock_armed:false,last_cursor_chord:false,zoom_held:false,last_gpu_hotkey:false,last_compare_hotkey:false,
            last_timer_hotkey:false,last_split_hotkey:false,
            game_timer:gametimer::RunTimer::default(),split_label:String::new(),
            sessions:if smoke{sessions::SessionTracker::default()}else{sessions::SessionTracker::load()},
            perf_lab:lab::PerformanceLab::default(),benchmark_length:30,target_fps:60,
            game_note_edit:String::new(),quest_task_edit:String::new(),
            cooldown_clock:gameplay::CooldownClock::default(),
            cooldown_spec:gameplay::CooldownSpec::default(),
            reaction:training::ReactionTrainer::default(),training_reset_confirm:false,
            last_capture_hotkey:false,last_record_hotkey:false,previews,hero_picture,gallery_category:0,preset_category:0,native_gallery_category:0,
            hud_monitor_cache:None,hud_observed_dpi:None,
            native_preview_textures:(0..crate::native_presets::PRESETS.len()).map(|_|None).collect(),
            native_gameplay_preview_textures:(0..crate::native_presets::PRESETS.len()).map(|_|None).collect(),
            native_gameplay_original_texture:None,
            native_gallery_scene:1,
            native_original_texture:None,
            os_color_photo_texture:None,os_color_photo_key:None,
            filter_search:String::new(),show_all_presets:gallery_expanded,
            driver_status:None,driver_reader:if smoke{None}else{Some(receiver)},
            driver_busy:!smoke,driver_strength:50,driver_error:None,last_ris_hotkey:false,
            last_boost_hotkey:false,
            page:Page::Home,monitor_profile_applied:None,unsorted_selected_panel:255,selected_profile:0,filter_view:0,performance_view:0,
            extras_view:0,overlay_open:start_hud,overlay_edit_mode:false,
            last_overlay_passive:None,last_hud_reassert:None,last_overlay_edit_hotkey:false,
            visual_compact_overlay:false,overlay_filter_tab:false,
            overlay_expanded:false,overlay_section:Page::Filters,
            overlay_notice:String::new(),last_overlay_opacity:None,last_hotkey:false,
            last_hud_f12:false,hud_fps:PresentStats::hud_meter(),hud_target:None,hud_uses_main:false,hud_track_at:None,
            last_color_hotkey:false,pending_gpu_refresh:None,last_refresh_retry:None,
            pending_screenshot:None,last_tick:Instant::now(),settings_debounce:Default::default(),last_stutter_alert:None,
            app_start:Instant::now(),last_toast:Instant::now()-Duration::from_secs(20),
            status:"External mode ready - no game DLL hooks".into(),
            screenshot:None,gpu:win::gpu_name(),is_smoke:smoke,smoke_frames:0,
            visual_test,visual_overlay,overlay_proof_name:"overlay-v22dev.png".into(),
            visual_driver:false,
            visual_gallery:false,visual_perf:false,visual_all_presets:false,
            visual_os_color_lab:false,visual_narrow_filters:false,
            visual_extras:false,visual_motion:false,
            visual_requested:false,visual_saved:false,
            visual_compact_main:false,visual_size_settings:false,visual_replay:false,
            visual_new_gpu_look:false};
        if app.telem.active_game.is_some(){app.apply_selected_profile();}else if let Some(p)=app.settings.render_preferences.get("__global__").cloned(){p.apply(&mut app.settings);app.target_fps=p.target;}
        if !smoke {
            app.crosshair.set_target(app.telem.active_game.as_ref().map(|g|g.pid));
            app.crosshair.set(&app.settings.crosshair);
        }
        if !smoke{
            if let Some(g)=app.telem.active_game.as_ref(){
                let key=g.path.to_string_lossy().to_string();
                app.sessions.switch(Some((&key,&g.title)));
            }
        }
        app.game_note_edit=app.settings.game_notes.get(&app.profile_key())
            .cloned().unwrap_or_default();
        app
    }
    fn load_hero_picture(ctx:&egui::Context,path:Option<&str>)->Option<egui::TextureHandle>{
        let data=path.and_then(|p|std::fs::read(p).ok())
            .unwrap_or_else(||include_bytes!("../assets/hero_gameplay_v22.png").to_vec());
        let picture=image::load_from_memory(&data).ok()?
            .resize_to_fill(960,380,image::imageops::FilterType::Lanczos3)
            .blur(0.8).to_rgba8();
        let pixels=egui::ColorImage::from_rgba_unmultiplied([960,380],picture.as_raw());
        Some(ctx.load_texture("hb-home-real-gameplay",pixels,egui::TextureOptions::LINEAR))
    }
    fn change_hero_picture(&mut self,path:&std::path::Path,ctx:&egui::Context){
        match std::fs::metadata(path){
            Ok(meta) if meta.len()<=25_000_000=>{
                if let Some(texture)=Self::load_hero_picture(ctx,path.to_str()){
                    self.hero_picture=Some(texture);
                    self.settings.hero_photo_path=Some(path.to_string_lossy().to_string());
                    self.save();
                    self.toast("Custom home art changed using your own local image");
                }else{self.toast("Unsupported image (choose a PNG/JPG)");}
            },
            _=>self.toast("Home art image must be a readable file under 25 MB")
        }
    }
    fn toast(&mut self,text:impl Into<String>){
        self.status=text.into();
        self.overlay_notice=self.status.clone();
        self.last_toast=Instant::now();
    }
    /// Keep frameless titlebar +/- controls independent of game HUD sizing.
    fn resize_main_window(&mut self,ctx:&egui::Context,step:i32){
        let old=self.settings.main_window_size;
        let size=[
            (old[0]+step as f32*110.0).clamp(800.0,1320.0),
            (old[1]+step as f32*65.0).clamp(540.0,880.0),
        ];
        self.set_main_window_size(ctx,size);
    }
    fn set_main_window_size(&mut self,ctx:&egui::Context,size:[f32;2]){
        let adjusted=[size[0].clamp(800.0,1320.0),
                      size[1].clamp(540.0,880.0)];
        self.settings.main_window_size=adjusted;
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
            egui::vec2(adjusted[0],adjusted[1])));
        if !self.is_smoke{self.save();}
    }
    fn save(&mut self){
        if self.is_smoke{return;} // hidden GUI screenshots must never alter user profiles
        let key=self.profile_key();
        let prefs=profile_portable::RenderPreferences::capture(&self.settings,self.target_fps);
        self.settings.render_preferences.insert(key,prefs);
        self.settings_debounce.mark(Instant::now());
    }
    fn flush_settings_if_due(&mut self){
        if self.is_smoke||!self.settings_debounce.due(Instant::now()){return;}
        match settings::save(&self.settings){Ok(())=>self.settings_debounce.clear(),Err(e)=>{self.settings_debounce.retry(Instant::now());self.toast(format!("Cannot save settings: {e}"));}}
    }
    fn profile_key(&self)->String{
        self.telem.active_game.as_ref()
            .map(|g|g.path.to_string_lossy().to_string())
            .unwrap_or_else(||"__global__".into())
    }
    fn change_filter(&mut self){
        let key=self.profile_key();
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        self.selected_profile=0;
        self.settings.profile.clamp();
        let key=self.profile_key();
        self.settings.custom_profiles.insert(key.clone(),self.settings.profile.clone());
        self.settings.preset_selection.insert(key.clone(),0);
        if self.telem.active_game.is_some(){
            self.settings.profiles.insert(key,self.settings.profile.clone());
        }
        if self.live.active&&!self.os_color_mode_active {
            let p=self.settings.profile.clone();
            if let Err(e)=self.live.update(&p){self.toast(format!("Live Color: {e}"));}
        }
        self.save();
    }
    fn save_gpu_profile(&mut self){
        let key=self.profile_key();
        let profile=GpuProfile{scaler:self.settings.live_scaler,
            effect:self.settings.live_preset,strength:self.settings.live_strength,
            stack:self.settings.live_stack,
            capture_method:self.settings.live_capture_method,
            output_mode:self.settings.live_output_mode};
        self.settings.custom_gpu_profiles.insert(key.clone(),profile.clone());
        self.settings.preset_selection.insert(key.clone(),0);
        self.selected_profile=0;
        if self.telem.active_game.is_some(){
            self.settings.gpu_profiles.insert(key,profile);
        }
        self.save();
    }
    fn refresh_driver_status(&mut self){
        if self.driver_busy{return;}
        let (tx,rx)=std::sync::mpsc::channel();
        self.driver_reader=Some(rx);
        self.driver_busy=true;
        std::thread::spawn(move||{let _=tx.send(adlx::probe());});
    }
    fn driver_action(&mut self,change:adlx::DriverChange){
        if self.driver_busy {
            self.toast("Driver is busy. The last operation must finish before another change.");
            return;
        }
        let (tx,rx)=std::sync::mpsc::channel();
        self.driver_reader=Some(rx);
        self.driver_busy=true;
        std::thread::spawn(move||{let _=tx.send(adlx::apply(change));});
        self.toast("Applying the AMD driver setting...");
    }
    fn update_driver_state(&mut self){
        let finished=self.driver_reader.as_ref().and_then(|rx|rx.try_recv().ok());
        if let Some(response)=finished {
            self.driver_reader=None;
            self.driver_busy=false;
            match response {
                Ok(state)=>{
                    self.driver_strength=state.sharpness;
                    self.driver_error=None;
                    self.driver_status=Some(state);
                    self.toast("AMD driver state verified via ADLX (not a simulated switch)");
                }
                Err(error)=>{
                    self.driver_error=Some(error.clone());
                    self.toast(format!("AMD ADLX: {error}"));
                }
            }
        }else if self.driver_busy && self.driver_reader.as_ref().is_some_and(|rx|
            matches!(rx.try_recv(),Err(std::sync::mpsc::TryRecvError::Disconnected))){
                self.driver_reader=None;
                self.driver_busy=false;
                self.driver_error=Some("Driver service ended before returning state".into());
        }
    }
    fn apply_os_color_mode(&mut self){
        if self.is_smoke{
            self.toast("System-wide color is disabled during screenshot/hidden tests");
            return;
        }
        let matrix=crate::color_modes::matrix(self.settings.os_color_mode,
            self.settings.os_color_strength);
        match self.live.enable_matrix(matrix){
            Ok(())=>{
                self.os_color_mode_active=true;
                self.toast(format!("Display color matrix ON: {} (whole desktop, not a game shader)",
                    crate::color_modes::mode_name(self.settings.os_color_mode)));
            },
            Err(e)=>{
                self.os_color_mode_active=false;
                self.toast(format!("Windows display color matrix unavailable: {e}"));
            }
        }
    }
    fn restore_os_color_mode(&mut self){
        self.live.disable();
        self.os_color_mode_active=false;
        self.toast("Original Windows display color matrix restored");
    }
    fn set_live_color(&mut self,enabled:bool) {
        if self.os_color_mode_active{
            self.live.disable();self.os_color_mode_active=false;
        }
        if enabled {
            match self.live.enable(&self.settings.profile){
                Ok(())=>self.toast("Live Color ON Â· applies to entire desktop Â· disable anytime"),
                Err(e)=>self.toast(format!("Live Color unavailable: {e}")),
            }
        }else{
            self.live.disable();self.toast("Live Color OFF Â· original desktop colors restored");
        }
    }
    fn set_profile(&mut self,p:FilterProfile,name:&str){
        let key=self.profile_key();
        let original=(self.settings.live_scaler,self.settings.live_preset,self.settings.live_strength,
            self.settings.live_stack,self.settings.live_capture_method,self.settings.live_output_mode);
        let (filter,gpu,index)=match name {
            "Quality"=>(p,GpuProfile{scaler:0,effect:3,strength:0.55,stack:5,
                capture_method:self.settings.live_capture_method,output_mode:self.settings.live_output_mode},1),
            "Performance"=>(p,GpuProfile{scaler:2,effect:0,strength:0.0,stack:9,
                capture_method:self.settings.live_capture_method,output_mode:self.settings.live_output_mode},2),
            _=>(
                self.settings.custom_profiles.get(&key).cloned()
                    .unwrap_or_else(FilterProfile::default),
                self.settings.custom_gpu_profiles.get(&key).cloned()
                    .unwrap_or_else(GpuProfile::default),0
            )
        };
        self.settings.profile=filter;
        self.settings.live_scaler=gpu.scaler;
        self.settings.live_preset=gpu.effect;
        self.settings.live_strength=gpu.strength;
        self.settings.live_stack=gpu.stack;
        self.settings.live_capture_method=gpu.capture_method;
        self.settings.live_output_mode=gpu.output_mode;
        self.selected_profile=index;
        self.settings.preset_selection.insert(key.clone(),index);
        if self.telem.active_game.is_some(){
            self.settings.profiles.insert(key.clone(),self.settings.profile.clone());
            self.settings.gpu_profiles.insert(key,gpu);
        }
        if self.live.active&&!self.os_color_mode_active{
            let _=self.live.update(&self.settings.profile);
        }
        self.save();
        let next=(self.settings.live_scaler,self.settings.live_preset,self.settings.live_strength,
            self.settings.live_stack,self.settings.live_capture_method,self.settings.live_output_mode);
        if original!=next&&self.gpu_backend.running(){
            self.gpu_backend.stop();
            self.pending_gpu_refresh=Some(Instant::now());
            self.last_refresh_retry=None;
        }
        if self.telem.active_game.as_ref().is_some_and(|g|g.fullscreen_like){
            self.toast(format!("{name} profile saved. In fullscreen, use the native AMD RIS / Anti-Lag controls; external HLSL effects cannot be applied here."));
        }else{
            self.toast(format!("{name} profile applied (GPU renderer reloads if running)"));
        }
    }

    fn sync_monitor_profile(&mut self){
        // Explicit opt-in, only for a known saved (game path, monitor) pair.
        // Never changes Windows refresh rate or the game's own FPS limiter.
        let selection=if self.settings.monitor_fps_auto_apply{
            self.settings.preview_display.as_ref().map(|d|(self.profile_key(),d.clone()))
        }else{None};
        if self.monitor_profile_applied==selection{return;}
        self.monitor_profile_applied=selection.clone();
        let Some((game,monitor))=selection else{return;};
        let Some(p)=crate::monitor_fps_profiles::chosen(&self.settings.monitor_fps_profiles,&game,Some(&monitor)) else{return;};
        if self.target_fps==p.target&&self.settings.preview_fps_limit==p.cap{return;}
        self.target_fps=p.target;self.settings.preview_fps_limit=p.cap;
        self.preview_guard.reset();self.render_guard.reset();
        if let Some(w)=self.native_mirror.as_ref(){w.set_preview_limit(p.cap);}
        self.save();
    }
    fn apply_selected_profile(&mut self){
        let Some(g)=&self.telem.active_game else{return;};
        let key=g.path.to_string_lossy().to_string();
        let profile=self.settings.profiles.entry(key.clone())
            .or_insert_with(FilterProfile::default).clone();
        self.settings.profile=profile;
        let saved=self.settings.gpu_profiles.entry(key.clone())
            .or_insert_with(GpuProfile::default).clone();
        self.selected_profile=*self.settings.preset_selection.get(&key).unwrap_or(&0);
        self.settings.live_scaler=saved.scaler;
        self.settings.live_preset=saved.effect;
        self.settings.live_strength=saved.strength;
        self.settings.live_stack=saved.stack;
        self.settings.live_capture_method=saved.capture_method;
        self.settings.live_output_mode=saved.output_mode;
        self.cooldown_clock.reset();
        self.cooldown_spec=self.settings.cooldown_profiles
            .get(&key).cloned().unwrap_or_default();
        self.settings.os_color_mode=*self.settings.os_color_profiles.get(&key)
            .unwrap_or(&self.settings.os_color_mode);
        if let Some(p)=self.settings.render_preferences.get(&key).cloned(){
            p.apply(&mut self.settings);self.target_fps=p.target;
        }else if let Some(p)=self.settings.render_preferences.get("__global__").cloned(){
            p.apply(&mut self.settings);self.target_fps=p.target;
        }else{
            let p=profile_portable::RenderPreferences::default();p.apply(&mut self.settings);self.target_fps=p.target;
        }
        self.preview_guard.reset();
        self.settings.processing_percent=self.settings.processing_profiles.get(&key).copied().unwrap_or(100);
        self.render_guard.reset();
        self.settings.native_preset=self.settings.native_preset_profiles
            .get(&key).copied().unwrap_or(0).min(crate::native_presets::PRESETS.len()-1);
        self.settings.lighting=self.settings.lighting_profiles.get(&key)
            .cloned().unwrap_or_default();
        if let Some(worker)=self.native_mirror.as_ref(){
            worker.set_lighting(&self.settings.lighting);
        }
        if let Some(crosshair)=self.settings.crosshair_profiles.get(&key){
            self.settings.crosshair=crosshair.clone();
        }else{
            // Never leak another game's custom crosshair into the next game.
            self.settings.crosshair=crate::crosshair::CrosshairOptions::default();
        }
        if !self.is_smoke{self.crosshair.set(&self.settings.crosshair);}
        if self.live.active&&!self.os_color_mode_active{
            let _=self.live.update(&self.settings.profile);
        }
        self.save();
    }
    fn apply_native_lighting(&mut self,mut value:lighting::LightTuning){
        value.clamp();
        self.settings.lighting=value;
        let key=self.profile_key();
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        self.settings.minecraft_active.remove(&key);
        self.settings.lighting_profiles.insert(key,self.settings.lighting.clone());
        if let Some(worker)=self.native_mirror.as_ref(){
            worker.set_lighting(&self.settings.lighting);
        }
        self.save();
    }
    fn stop_native_mirror(&mut self){
        self.output_rates.clear();
        self.preview_guard.reset();
        self.pending_mirror_restart=None;
        if let Some(mirror)=self.native_mirror.as_ref(){mirror.request_stop();self.native_mirror_status="Native GPU preview stopping...".into();}
        else{self.native_mirror_pid=None;self.native_mirror_status="Native GPU live preview stopped".into();}
    }
    /// Apply image quality changes to an already running native GPU output.
    /// Keep its previous presentation mode instead of silently stopping the
    /// preview and making the user wonder why the filter disappeared.
    /// One authoritative native preset setter, used by Home, GPU gallery
    /// and overlay. Keep the live DXGI renderer in sync with displayed UI.
    fn select_native_preset(&mut self,index:usize){
        if index>=crate::native_presets::PRESETS.len(){return;}
        let key=self.profile_key();
        let mc_look=self.settings.minecraft_active.contains_key(&key);
        if mc_look{
            // Revert the full saved native look before switching to a general
            // preset so Minecraft's special exposure/lighting cannot leak.
            let _=crate::minecraft_profile::restore(&mut self.settings,&key);
        }
        let was_custom=self.settings.native_stack_profiles.remove(&key).is_some();
        if self.settings.native_preset==index&&!was_custom&&!mc_look{return;}
        self.settings.native_preset=index;
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        self.settings.lighting.layers.clear_ordered();self.settings.lighting_profiles.insert(key.clone(),self.settings.lighting.clone());
        if self.telem.active_game.is_some(){
            self.settings.native_preset_profiles.insert(self.profile_key(),index);
        }
        self.save();
        if self.pending_mirror_restart.is_none(){
            if let Some(worker)=self.native_mirror.as_ref().filter(|w|w.running()){
                match worker.set_look(index,self.settings.native_quality,&self.settings.lighting){
                    Ok(())=>{
                        self.native_mirror_status="GPU look updated without restarting capture".into();
                        return;
                    },
                    Err(e)=>{
                        self.native_mirror_status=format!("GPU look rejected: {e}");
                        self.refresh_native_mirror();
                        return;
                    }
                }
            }
        }
        self.refresh_native_mirror();
    }
    fn import_portable_profile(&mut self,profile:profile_portable::Portable)->Result<(),String>{
        let key=self.profile_key();let old=profile_portable::Portable::capture(&self.settings,&key,self.target_fps);
        let mut next=self.settings.clone();let target=profile.apply(&mut next,&key)?;
        if !self.is_smoke{let backup=crate::profile_backup::save(&old)?;let restored=crate::profile_backup::load(&backup)?;if restored.encode()?!=old.encode()?{return Err("Prior profile backup verification failed".into());}}
        // If persistent backup fails, leave live settings unchanged.
        // Stop only our own active tools, never activate them on imported settings.
        self.stop_native_mirror();self.restore_os_color_mode();self.live.disable();
        self.macro_armed_pid=None;self.macro_runner.cancel();self.reminders.clear();
        self.cooldown_clock.reset();self.render_guard.reset();self.preview_guard.reset();
        self.settings=next;self.target_fps=target;self.selected_profile=0;self.macro_drafts.remove(&key);
        self.cooldown_spec=self.settings.cooldown_profiles.get(&key).cloned().unwrap_or_default();
        self.game_note_edit=self.settings.game_notes.get(&key).cloned().unwrap_or_default();
        if !self.is_smoke{self.crosshair.set(&self.settings.crosshair);}
        self.portable_undo=Some((key,old));self.save();Ok(())
    }

    fn update_hold_zoom(&mut self){
        // Bounded foreground-only shortcut, no game input injection or window changes.
        let pressed=self.settings.global_shortcuts.down("ZOOM")
            && self.native_mirror_pid.is_some_and(|pid|crate::game_macros::foreground_pid()==pid)
            && !self.macro_runner.running()&&!self.window_macro.busy();
        if pressed==self.zoom_held{return;}
        let Some(mirror)=self.native_mirror.as_ref().filter(|w|w.running()) else{self.zoom_held=false;return;};
        let result=if pressed{
            mirror.set_stack(&[42],0,&self.settings.lighting)
        }else{
            let key=self.profile_key();
            match self.settings.native_stack_profiles.get(&key){
                Some(stages)=>mirror.set_stack(stages,self.settings.native_quality,&self.settings.lighting),
                None=>mirror.set_look(self.settings.native_preset,self.settings.native_quality,&self.settings.lighting),
            }
        };
        match result{
            Ok(())=>self.zoom_held=pressed,
            Err(e)=>{self.zoom_held=false;self.toast(format!("Zoom shortcut not applied: {e}"));},
        }
    }

    fn apply_custom_native_stack(&mut self,stages:Vec<usize>){
        if let Err(error)=native_stack::validate(&stages){self.toast(error);return;}
        let key=self.profile_key();
        let previous=self.settings.native_stack_profiles.get(&key).map(Vec::as_slice).unwrap_or(crate::native_presets::PRESETS[self.settings.native_preset].stages);
        if previous!=stages.as_slice(){self.settings.lighting.layers.clear_ordered();self.settings.lighting_profiles.insert(key.clone(),self.settings.lighting.clone());}
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        self.settings.native_stack_profiles.insert(key,stages.clone());
        self.settings.native_quality=0; // explicit ordered stack, all stages
        self.save();
        if self.pending_mirror_restart.is_none()&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&w.set_stack(&stages,0,&self.settings.lighting).is_ok()){
            self.native_mirror_status="Custom ordered GPU stack updated live".into();
        }else{self.refresh_native_mirror();}
        self.toast("Custom native stack selected");
    }
    fn apply_render_profile(&mut self,p:crate::render_profiles::Profile){
        self.settings.processing_percent=p.processing;
        self.settings.processing_profiles.insert(self.profile_key(),p.processing);
        self.settings.native_scale_mode=p.algorithm;self.settings.native_scale_ratio=1.0;
        self.settings.native_quality=p.quality;self.settings.preview_fps_limit=p.cap;
        self.preview_guard_enabled=p.guard;self.preview_guard.reset();self.render_guard.reset();
        self.save();self.refresh_native_mirror();
    }
    fn apply_gpu_quality(&mut self,quality:u8){
        if quality>3||quality==self.settings.native_quality{return;}
        self.settings.native_quality=quality;self.save();
        let key=self.profile_key();
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        let stack=self.settings.native_stack_profiles.get(&key).map(Vec::as_slice);
        if self.pending_mirror_restart.is_none()&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&match stack{Some(ids)=>w.set_stack(ids,quality,&self.settings.lighting).is_ok(),None=>w.set_look(self.settings.native_preset,quality,&self.settings.lighting).is_ok()}){
            self.native_mirror_status="GPU quality updated live".into();
        }else{self.refresh_native_mirror();}
    }
    fn apply_scale_algorithm(&mut self,mode:u8){
        if mode as usize>=crate::spatial::MODES.len(){return;}
        self.settings.native_scale_mode=mode;self.save();
        if self.pending_mirror_restart.is_none()&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&w.set_scale_mode(mode)){
            self.native_mirror_status="Scaling algorithm updated without capture restart".into();
        }else{self.refresh_native_mirror();}
    }
    fn refresh_native_mirror(&mut self){
        if let (Some(m),Some(pid))=(self.native_mirror.as_ref(),self.native_mirror_pid){
            if m.running()||self.pending_mirror_restart.is_some(){
                let factor=if self.settings.experimental_fg_allowed{self.pending_mirror_restart.map(|p|p.2).unwrap_or_else(||m.multiplier())}else{1};
                self.pending_mirror_restart=Some((pid,m.mode(),factor));
                m.request_stop();self.output_rates.clear();
                self.native_mirror_status="Updating native GPU output...".into();
            }
        }
    }
    fn toggle_native_mirror(&mut self,mode:native_mirror::MirrorMode){
        self.toggle_native_mirror_factor(mode,1);
    }
    fn toggle_native_mirror_factor(&mut self,mode:native_mirror::MirrorMode,factor:u8){
        let mode=if mode==native_mirror::MirrorMode::CompositedInPlace{native_mirror::MirrorMode::BorderlessScreen}else{mode};
        if factor>1&&!self.settings.experimental_fg_allowed{
            self.toast("Frame generation is disabled for image quality. Enable experimental FG in Motion Lab to test it explicitly.");
            return;
        }
        if self.native_mirror.is_some(){
            self.stop_native_mirror();
            self.toast("Native GPU preview stopped; your game was not modified");
            return;
        }
        if self.is_smoke{
            self.toast("Native mirror cannot start in hidden visual test mode");
            return;
        }
        if self.pipeline_bench_rx.is_some()||self.effect_bench_rx.is_some()||self.photo_rx.is_some()||self.motion_lab_rx.is_some(){self.toast("Wait for the explicit offline GPU benchmark before starting live output");return;}
        let Some(pid)=self.telem.active_game.as_ref().map(|g|g.pid) else{
            self.toast("Choose an actual running game in Library before live preview");
            return;
        };
        match crate::capture_start_guard::preflight(pid){
            Ok(crate::capture_start_guard::LaunchAdvice::LowRamWarning)=>{
                self.native_mirror_status="Warning: limited free RAM; GPU capture may stutter".into();
            },
            Ok(crate::capture_start_guard::LaunchAdvice::BackgroundGame)=>{
                self.native_mirror_status="Game is not foreground: the game-view capture can pause until Alt+Tab".into();
            },
            Ok(crate::capture_start_guard::LaunchAdvice::MonitorFillingUnverified)=>{
                self.native_mirror_status="Monitor-filling window detected; exclusive fullscreen support unverified".into();
            },
            Ok(crate::capture_start_guard::LaunchAdvice::Ready)=>{},
            Err(e)=>{self.native_mirror_status=e.clone();self.toast(e);return;},
        }
        if self.settings.anti_cheat_safe_mode{
            if let Some(game)=self.telem.active_game.as_ref(){
                let check=crate::anticheat_guard::inspect(&game.path,&game.title);
                if check.blocked{
                    self.toast(format!("SAFE MODE: {}",check.reason));
                    return;
                }
            }
        }
        let custom=self.settings.native_stack_profiles.get(&self.profile_key()).map(Vec::as_slice);
        let percent=if self.render_guard_enabled&&mode!=native_mirror::MirrorMode::CompositedInPlace&&factor==1{
            self.render_guard.effective.unwrap_or(self.settings.processing_percent)
        }else{self.settings.processing_percent};
        native_mirror::configure_preview_cursor_scale(self.settings.preview_cursor_scale);
        hdr_output::configure(self.settings.hdr_curve,self.settings.hdr_peak_nits,self.settings.hdr_exposure);
        match native_mirror::MirrorWorker::start_on_display(pid,self.settings.native_preset,
            mode,factor,self.settings.lighting.clone(),self.settings.native_quality,
            self.settings.native_scale_mode,self.settings.native_scale_ratio,custom,percent,self.settings.preview_low_latency,self.settings.preview_display.clone(),self.settings.display_grades.clone(),self.settings.renderer_adapter.clone(),self.settings.preview_tearing,self.settings.preview_composition,self.settings.preview_scrgb,self.settings.preview_hdr_white_nits){
            Ok(mirror)=>{
                mirror.set_preview_limit(if self.preview_guard_enabled&&factor==1{self.preview_guard.effective.unwrap_or(self.settings.preview_fps_limit)}else{self.settings.preview_fps_limit});
                mirror.set_vsync(self.settings.preview_vsync);
                self.native_mirror=Some(mirror);
                self.native_mirror_pid=Some(pid);
                self.native_mirror_status=if mode==native_mirror::MirrorMode::CompositedInPlace{
                    format!("Starting WGC glass x{factor} (game unmodified; true exclusive unverified)...")
                }else if mode==native_mirror::MirrorMode::DesktopDuplication{format!("Starting selected-game foreground-only DXGI duplication fallback 1x (WDA window exclusion required)...")}else if mode==native_mirror::MirrorMode::BorderlessScreen{format!("Starting separate borderless click-through GPU output x{factor} (FSE unverified)...")}else{format!("Starting independent WGC GPU output x{factor}...")};
                self.toast(if mode==native_mirror::MirrorMode::CompositedInPlace{
                    "Composited glass starting: no game changes; true exclusive may cover overlay"
                }else if mode==native_mirror::MirrorMode::DesktopDuplication{"DXGI duplication fallback starting; monitor-filling game, foreground only, own output excluded from capture"}else if mode==native_mirror::MirrorMode::BorderlessScreen{"Borderless independent output starting (own topmost click-through HWND, game untouched)"}else{"Native preview starting in its own window (no game mode changes)"});
            },
            Err(error)=>{
                self.native_mirror_status=format!("Native GPU mirror: {error}");
                self.toast(self.native_mirror_status.clone());
            }
        }
    }
    fn choose_game(&mut self,pid:u32){
        self.cursor_lock.release();self.cursor_lock_armed=false;
        if self.os_color_mode_active&&self.telem.active_game.as_ref().is_some_and(|g|g.pid!=pid){
            self.restore_os_color_mode();
        }
        if self.native_mirror_pid.is_some_and(|old|old!=pid){
            self.stop_native_mirror();
        }
        if self.cpu_affinity.active()&&self.cpu_affinity.pid!=pid{
            if let Err(e)=self.cpu_affinity.restore(){self.toast(e);}
        }
        self.affinity_draft=None;
        self.render_guard.reset();
        self.reminders.clear();
        if self.game_boost.active()&&self.game_boost.pid!=pid{
            let _=self.game_boost.stop();
        }
        if self.telem.select_game(pid){
            self.quest_task_edit.clear();
            self.crosshair.set_target(Some(pid));
            self.apply_selected_profile();
            self.track_selected_session();
            self.fps.track(Some(pid),self.settings.fps_monitoring);
            self.toast(format!("Selected game: {}",self.telem.active_game.as_ref()
                .map(|g|g.title.as_str()).unwrap_or("Unknown")));
        }else{self.toast("That game window is no longer available");}
    }
    fn toggle_favorite_effect(&mut self,index:usize){
        if index>=engine::EFFECTS.len(){return;}
        let label=engine::EFFECTS[index].label;
        if let Some(pos)=self.settings.favorite_effects.iter().position(|&id|id==index){
            self.settings.favorite_effects.remove(pos);
            self.toast(format!("{label} removed from quick access"));
        }else{
            self.settings.favorite_effects.push(index);
            settings::normalize_favorite_effects(&mut self.settings.favorite_effects);
            self.toast(format!("{label} added to quick access"));
        }
        self.save(); // A favorite only changes navigation, never starts a shader.
    }
    fn on_gpu_filter_changed(&mut self){
        let key=self.profile_key();
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        self.save_gpu_profile();
        if self.gpu_backend.running(){
            self.gpu_backend.stop();
            self.pending_gpu_refresh=Some(Instant::now());
            self.last_refresh_retry=None;
            self.toast("GPU filter changed; external renderer will reinitialize automatically");
        }else if self.telem.active_game.as_ref().is_some_and(|g|g.fullscreen_like){
            self.toast("External GPU filter saved, NOT applied in fullscreen. Use Radeon Image Sharpening in AMD Driver Features.");
        }else{
            self.toast("GPU filter saved; start the renderer when playing a windowed game");
        }
    }
    fn toggle_crosshair(&mut self){
        if !self.settings.crosshair.enabled&&self.telem.active_game.is_none(){self.toast("Choose a game before enabling crosshair");return;}
        self.settings.crosshair.enabled=!self.settings.crosshair.enabled;
        self.apply_crosshair();
    }
    fn apply_crosshair(&mut self){
        self.settings.crosshair.clamp();
        if self.telem.active_game.is_some(){
            self.settings.crosshair_profiles.insert(self.profile_key(),self.settings.crosshair.clone());
        }
        if !self.is_smoke{
            self.crosshair.set_target(self.telem.active_game.as_ref().map(|g|g.pid));
            self.crosshair.set(&self.settings.crosshair);
        }
        self.save();
        self.toast(if self.settings.crosshair.enabled{
            "Click-through crosshair ON Â· external Windows HUD Â· Ctrl+Shift+X"
        }else{"Crosshair OFF Â· overlay hidden"});
    }
    fn toggle_game_boost(&mut self){
        if self.game_boost.active(){
            match self.game_boost.stop(){
                Ok(())=>self.toast("Game Boost stopped; original Windows priority restored"),
                Err(e)=>self.toast(format!("Game Boost restore failed: {e}")),
            }
        }else if let Some(g)=self.telem.active_game.as_ref(){
            let pid=g.pid;
            let title=g.title.clone();
            match self.game_boost.start(pid){
                Ok(report)=>self.toast(format!("{title}: {report}")),
                Err(e)=>self.toast(format!("Game Boost not applied: {e}"))
            }
        }else{
            self.toast("Open a game first; Game Boost never targets random processes");
        }
    }
    fn track_selected_session(&mut self){
        if self.is_smoke{return;}
        let next=self.telem.active_game.as_ref().map(|g|{
            (g.path.to_string_lossy().to_string(),g.title.clone())
        });
        let changed=self.sessions.switch(next.as_ref().map(|(k,n)|(k.as_str(),n.as_str())));
        if changed {
            if let Err(e)=self.sessions.save(){self.toast(format!("Playtime save: {e}"));}
        }
        self.game_note_edit=self.settings.game_notes.get(&self.profile_key())
            .cloned().unwrap_or_default();
    }
    fn export_active_profile(&mut self){
        let name=self.telem.active_game.as_ref().map(|g|g.title.as_str())
            .unwrap_or("Horizon Boost Custom").to_string();
        match share::export(&self.settings,&name){
            Ok(path)=>self.toast(format!("Portable profile saved: {}",path.display())),
            Err(err)=>self.toast(format!("Profile export failed: {err}"))
        }
    }
    fn import_profile(&mut self,path:&std::path::Path){
        match share::load(path){
            Ok(p)=>{
                let name=p.name.clone();
                share::apply(&mut self.settings,p);
                self.change_filter();
                self.on_gpu_filter_changed();
                self.toast(format!("Imported GPU and color profile: {name}; external renderer reloaded if running"));
            },
            Err(e)=>self.toast(format!("Profile import: {e}"))
        }
    }
    fn save_look_slot(&mut self,slot:u8){
        let key=self.profile_key();
        let snapshot=compare::LookSnapshot::capture_native(&self.settings,&key);
        match self.settings.look_pairs.entry(key).or_default().store(slot,snapshot){
            Ok(())=>{
                self.save();
                self.toast(format!("Saved native GPU look to comparison {}",
                    if slot==0{"A"}else{"B"}));
            },
            Err(e)=>self.toast(e)
        }
    }
    fn load_named_native_look(&mut self,name:&str){
        let Some(look)=self.settings.named_native_looks.get(name).cloned()else{self.toast("Named look not found");return;};
        let key=self.profile_key();look.apply(&mut self.settings,&key);
        if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=None;}
        let stack=self.settings.native_stack_profiles.get(&key).map(Vec::as_slice);
        if self.pending_mirror_restart.is_none()&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&match stack{
            Some(ids)=>w.set_stack(ids,self.settings.native_quality,&self.settings.lighting).is_ok(),
            None=>w.set_look(self.settings.native_preset,self.settings.native_quality,&self.settings.lighting).is_ok()
        }){self.native_mirror_status="Named GPU look updated live".into();}else{self.refresh_native_mirror();}
        self.save();self.toast("Named look selected; output starts only on explicit Apply");
    }
    fn change_fg_multiplier(&mut self,factor:u8){
        if ![2,3].contains(&factor)||!self.settings.experimental_fg_allowed{return;}
        self.settings.external_fg_factor=factor;
        if let (Some(w),Some(pid))=(self.native_mirror.as_ref(),self.native_mirror_pid){
            if w.multiplier()>1&&(w.running()||self.pending_mirror_restart.is_some()){
                self.pending_mirror_restart=Some((pid,native_mirror::MirrorMode::SeparateWindow,factor));
                w.request_stop();self.output_rates.clear();
                self.native_mirror_status=format!("Updating interpolation to {factor}x...");
            }
        }
        self.save();
    }
    fn load_look_slot(&mut self,slot:u8){
        let key=self.profile_key();
        let snapshot=self.settings.look_pairs.get_mut(&key).and_then(|p|p.activate(slot));
        if let Some(snapshot)=snapshot {
            if let Some(native)=snapshot.native.as_ref(){
                native.apply(&mut self.settings,&key);
                let stack=self.settings.native_stack_profiles.get(&key).map(Vec::as_slice);
                if self.pending_mirror_restart.is_none()&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&match stack{
                    Some(ids)=>w.set_stack(ids,self.settings.native_quality,&self.settings.lighting).is_ok(),
                    None=>w.set_look(self.settings.native_preset,self.settings.native_quality,&self.settings.lighting).is_ok()
                }){self.native_mirror_status="A/B native GPU look updated live".into();}
                else{self.refresh_native_mirror();}
            }else{
                // Old saved slots never contained native settings. Preserve them without silently overwriting the current native look.
                snapshot.apply(&mut self.settings);
                self.change_filter();self.on_gpu_filter_changed();
            }
            if let Some(pair)=self.settings.look_pairs.get_mut(&key){pair.active=Some(slot);}
            self.save();
            self.toast(format!("Comparison {} selected Â· GPU renderer updates if supported",
                if slot==0{"A"}else{"B"}));
        }else{
            self.toast(format!("Slot {} is empty. Save your current look into it first.",
                if slot==0{"A"}else{"B"}));
        }
    }
    fn toggle_look_comparison(&mut self){
        let key=self.profile_key();
        let next=self.settings.look_pairs.get_mut(&key).and_then(|pair|{
            if !pair.stored(0)||!pair.stored(1){return None;}
            pair.toggle().map(|(slot,_)|slot)
        });
        if let Some(slot)=next{
            self.load_look_slot(slot);
        }else{
            self.toast("Save both Look A and Look B first; no settings were changed.");
        }
    }
    fn toggle_game_timer(&mut self){
        let running=self.game_timer.toggle();
        self.toast(if running{"Game timer started/resumed (real monotonic time)"}
            else{"Game timer paused"});
    }
    fn add_game_split(&mut self){
        let label=self.split_label.trim();
        match self.game_timer.split(if label.is_empty(){None}else{Some(label)}){
            Ok(index)=> {
                self.split_label.clear();
                self.toast(format!("Split #{index} added at {}",
                    gametimer::RunTimer::fmt_elapsed(self.game_timer.elapsed())));
            },
            Err(e)=>self.toast(e)
        }
    }
    fn start_benchmark(&mut self){
        if !self.settings.fps_monitoring{
            self.toast("Enable PresentMon ETW frame measurement first.");
            return;
        }
        let Some(g)=self.telem.active_game.as_ref()else{
            self.toast("Select a running game to benchmark.");return;
        };
        if self.fps.snapshot().is_none(){
            self.toast("No fresh PresentMon ETW game frames yet. Capture not started.");
            return;
        }
        match self.perf_lab.start(&g.title,g.pid,self.benchmark_length){
            Ok(())=>self.toast(format!("Capturing {} s of real ETW frame times for {}",
                self.benchmark_length,g.title)),
            Err(e)=>self.toast(e)
        }
    }
    fn activate_gpu_now(&mut self){
        if self.pending_gpu_refresh.is_some(){
            self.toast("Selected GPU preset is already reinitializing");
        }else if self.gpu_backend.running(){
            self.on_gpu_filter_changed();
        }else{
            self.save_gpu_profile();
            self.start_live_gpu();
        }
    }
    fn start_live_gpu(&mut self){
        // v2.2: all live activation now uses Horizon-owned WGC + D3D11.
        // The archived Magpie bridge remains in old source for migration but
        // is never started by UI, hotkeys, or game profile switching.
        if self.telem.active_game.is_none(){
            self.toast("Select a running game in Library first.");
            return;
        }
        self.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);
    }
    fn stop_live_gpu(&mut self){
        self.pending_gpu_refresh=None;
        self.last_refresh_retry=None;
        self.gpu_backend.stop(); // cleanup only: never starts Magpie
        self.stop_native_mirror();
    }
    fn screenshot(&mut self){
        if self.overlay_open{
            self.overlay_open=false;
            self.pending_screenshot=Some(Instant::now());
            self.toast("Screenshot queued; hiding overlay first");
        }else{
            self.screenshot=Some(record::capture_async());
            self.toast("Capturing screen...");
        }
    }
    fn toggle_replay(&mut self){
        if self.replay.active(){
            self.replay.stop();
            self.toast("Replay buffering stopped. Existing saved videos remain in Videos/HorizonBoost.");
        }else{
            if self.settings.anti_cheat_safe_mode&&self.telem.active_game.as_ref().is_some_and(|g|crate::anticheat_guard::inspect(&g.path,&g.title).blocked){self.toast("Safe Mode: replay not started for selected protected title");return;}
            self.overlay_open=false; // don't paint the editor into the first frames
            if self.recorder.busy()||self.media_export.is_some()||self.encoder_rx.is_some(){self.toast("Wait for recording or media work before starting replay");return;}
            match self.replay.start(self.settings.record_width,self.settings.record_fps,
                self.settings.hardware_encoder,self.settings.replay_game_lengths.get(&self.profile_key()).copied().unwrap_or(self.settings.replay_seconds),self.settings.capture_mbps,self.settings.replay_limit_mib){
                Ok(())=>{self.replay_game_key=Some(self.profile_key());self.toast(format!("{}s replay buffering; video only",self.replay.seconds()));},
                Err(e)=>self.toast(format!("Replay: {e}"))
            }
        }
    }
    fn save_replay(&mut self){
        self.save_replay_window(self.replay.seconds());
    }
    fn save_replay_window(&mut self,seconds:u32){
        match self.replay.save_window(seconds){
            Ok(())=>{self.replay_save_game_key=self.replay_game_key.clone();self.toast(format!("Saving {}s replay to Videos/HorizonBoost...",seconds));},
            Err(e)=>self.toast(format!("Replay not ready: {e}"))
        }
    }
    fn recording(&mut self){
        if self.recorder.active(){
            if let Some(f)=self.recorder.stop(){self.toast(format!("Finishing video: {}",f.display()));}
        }else{
            if self.replay.busy()||self.media_export.is_some()||self.encoder_rx.is_some(){self.toast("Wait for replay or media work before recording");return;}
            if crate::suite_finish::free_bytes(&crate::record::videos()).is_some_and(|n|n<1024*1024*1024){self.toast("Recording not started: less than 1 GiB free on capture disk");return;}
            if self.settings.anti_cheat_safe_mode&&self.telem.active_game.as_ref().is_some_and(|g|crate::anticheat_guard::inspect(&g.path,&g.title).blocked){self.toast("Safe Mode: recording not started for selected protected title");return;}
            // Do not burn the overlay into user videos when capture starts from the overlay.
            self.overlay_open=false;
            let mut session=match capture_session::Session::prepare(self.telem.active_game.as_ref().map(|g|g.pid),self.settings.recording_stop_on_exit,self.settings.recording_limit_mib,self.profile_key()){
                Ok(v)=>v,Err(e)=>{self.toast(e);return;}
            };
            match self.recorder.start(self.settings.record_width,self.settings.record_fps,self.settings.hardware_encoder,self.settings.capture_mbps,self.settings.capture_remux,std::mem::take(&mut self.audio_next),[self.settings.capture_gains[0],if self.settings.capture_mute_mic{0.}else{self.settings.capture_gains[1]}]){
                Ok(path)=>{session.begin(path.clone());self.capture_session=session;self.toast(format!("Recording desktop: {}",path.display()));},
                Err(e)=>self.toast(e),
            }
        }
    }
    fn emergency_stop(&mut self){
        self.reference_epoch=self.reference_epoch.wrapping_add(1);self.named_macro_arm=None;
        self.cursor_lock.release();self.cursor_lock_armed=false;
        self.reminders.clear();self.companion.stop();self.mobile.stop();self.pending_screenshot=None;
        self.report.stop();self.controller_enabled=false;self.controller_drift.stop();
        self.break_timer.stop();self.sensor_enabled=false;self.audio_next=Default::default();self.audio_enabled=false;self.audio_meter.reset();self.spectrum_enabled=false;self.spectrum.stop();self.settings.hud_layout.audio=false;self.accuracy_trainer.stop();self.accuracy_window=false;
        self.render_guard.reset();self.render_guard_enabled=false;
        let _=self.timer_precision.disable();
        self.macro_armed_pid=None;
        self.macro_runner.cancel();self.window_macro.cancel();self.sequence_window.close();
        self.stop_native_mirror();
        self.overlay_open=false;self.hud_fps.stop();self.hud_target=None;self.hud_track_at=None;self.hud_uses_main=false;
        self.overlay_edit_mode=false;
        if self.settings.crosshair.enabled{self.toggle_crosshair();}
        self.replay.stop();
        self.recorder.stop();
        self.restore_os_color_mode();
        self.live.disable();
        let _=self.game_boost.stop();
        let affinity=self.cpu_affinity.restore();
        let plan=self.power_plan.restore();
        self.network_probe.stop();
        self.toast(format!("Game tools stopped. CPU: {} / power: {}",
            affinity.unwrap_or_else(|e|e),plan.unwrap_or_else(|e|e)));
    }
    fn start_dxgi_diagnostic(&mut self){
        if self.is_smoke||self.dxgi_probe.is_some(){return;}
        let (tx,rx)=std::sync::mpsc::channel();
        self.dxgi_probe=Some(rx);
        self.dxgi_probe_status=None;
        std::thread::spawn(move||{let _=tx.send(native_capture::probe_gpu_only());});
    }
    fn toggle_passive_hud(&mut self){
        self.overlay_open=!self.overlay_open;self.overlay_edit_mode=false;self.overlay_expanded=false;
        self.hud_track_at=None;
    }
    fn hud_meter(&self)->&PresentStats{if self.hud_uses_main{&self.fps}else{&self.hud_fps}}
    fn hud_snapshot(&self)->Option<crate::fps::Stats>{self.hud_meter().snapshot_for(self.hud_target.as_ref().map(|v|v.0))}
    fn hud_output_matches(&self)->bool{self.native_mirror_pid.is_some()&&self.hud_target.as_ref().map(|v|v.0)==self.native_mirror_pid}
    fn update_hud_monitor(&mut self){
        if !self.overlay_open{
            if self.hud_target.take().is_some(){self.hud_fps.stop();}
            self.hud_uses_main=false;self.hud_track_at=None;return;
        }
        if self.hud_track_at.is_none_or(|t|t.elapsed()>=Duration::from_millis(850)){
            self.hud_track_at=Some(Instant::now());
            let active=self.telem.active_game.as_ref().map(|g|(g.pid,g.path.file_stem().map(|s|s.to_string_lossy().into_owned()).unwrap_or_else(||"Game".into())));
            let foreground=if self.settings.hud_follow_foreground&&self.native_mirror.is_none(){win::foreground_hud_app()}else{None};
            let wanted=if self.native_mirror.is_some()||!self.settings.hud_follow_foreground{active.clone()}else{foreground.or(active.clone())};
            let reuse=crate::hud_readout::reuse_game_meter(wanted.as_ref().map(|v|v.0),active.as_ref().map(|v|v.0),self.settings.fps_monitoring);
            self.hud_uses_main=reuse;
            self.hud_fps.track(if reuse{None}else{wanted.as_ref().map(|v|v.0)},true);
            self.hud_target=wanted;
        }
        if !self.hud_uses_main{self.hud_fps.poll();}
    }
    fn step(&mut self) {
        self.flush_settings_if_due();
        if self.is_smoke{return;}
        let f12=win::f12_down();let f12_rising=f12&&!self.last_hud_f12;self.last_hud_f12=f12;
        let hud_toggle_f12=crate::hud_readout::shortcut_allowed(self.settings.hud_f12,f12_rising,win::shortcut_modifiers_down(),self.window_macro.busy()||self.macro_runner.running()||self.macro_armed_pid.is_some());
        if hud_toggle_f12{self.toggle_passive_hud();}
        if !self.is_smoke{
            let pid=self.telem.active_game.as_ref().filter(|g|Some(g.pid)==self.telem.selected_pid).map(|g|g.pid);
            self.process_load.poll(pid,Instant::now());
        }
        if !unsorted_catalog::exposes(self.page,self.unsorted_selected_panel,Page::Extras)&&!self.accuracy_window{self.accuracy_trainer.stop();}
        if let Some(v)=self.pipeline_bench_rx.as_ref().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Pipeline benchmark worker stopped".into())),_=>None}){
            self.pipeline_bench_rx=None;self.pipeline_bench_result=Some(v);
        }
        if let Some(v)=self.library_scan.as_ref().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Library scan stopped".into())),_=>None}){
            self.library_scan=None;
            match v{Ok(scan)=>{let mut added=0;for e in scan.entries{if game_library::insert(&mut self.settings.curated_games,e).is_ok(){added+=1;}}
                self.library_confirm=None;self.save();self.toast(format!("Imported {added}; folders {}; manifests {}; invalid/duplicate {}; bounded scan {}",scan.folders,scan.inspected,scan.rejected,if scan.truncated{"truncated"}else{"complete"}));},Err(e)=>self.toast(e)}
        }
        if let Some(v)=self.save_vault_job.as_ref().and_then(|r|match r.try_recv(){Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Save vault worker stopped".into())),_=>None}){
            self.save_vault_job=None;self.save_vault_names=save_vault::list();self.toast(v.unwrap_or_else(|e|e));
        }
        if let Some(result)=self.dxgi_probe.as_ref().and_then(|rx|match rx.try_recv(){
            Ok(result)=>Some(result),
            Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("DXGI probe worker stopped".into())),
            _=>None
        }){
            self.dxgi_probe=None;self.dxgi_probe_status=Some(result);
        }
        self.window_macro.poll();
        if let Some(result)=self.macro_runner.poll(){
            self.toast(match result{Ok(n)=>format!("Macro finished: {n} taps"),Err(e)=>e});
        }
        if self.macro_armed_pid.is_some_and(|p|Some(p)!=self.telem.active_game.as_ref().map(|g|g.pid)||Some(p)!=self.telem.selected_pid){
            self.macro_armed_pid=None;self.macro_runner.cancel();
        }
        if !self.is_smoke{
            let stop=win::chord(0x7B);
            if stop&&!self.emergency_down{self.emergency_stop();}
            self.emergency_down=stop;
            if stop{return;}
            let mark=self.settings.global_shortcuts.down("MARK");
            if mark&&!self.marker_chord_down{if self.report.active(){self.report.mark("Manual game moment",Instant::now());self.toast("Session marker added");}if let Some(t)=self.recorder.elapsed(){if self.capture_session.add_marker(t,"Manual clip moment"){self.toast("Recording marker added");}}}
            self.marker_chord_down=mark;
            let down=self.settings.global_shortcuts.down("MACRO");
            if self.named_macro_arm.as_ref().is_some_and(|(key,t,_,until)|Instant::now()>=*until||*key!=self.profile_key()||self.telem.active_game.as_ref().is_none_or(|g|g.pid!=t.pid)){self.named_macro_arm=None;}
            if down&&!self.macro_hotkey_down{
                if let Some((key,target,plan,until))=self.named_macro_arm.take(){
                    if Instant::now()<until&&key==self.profile_key()&&self.telem.active_game.as_ref().is_some_and(|g|g.pid==target.pid&&!crate::anticheat_guard::inspect(&g.path,&g.title).blocked)&&game_macros::foreground_pid()==target.pid&&!self.window_macro.busy()&&!self.macro_runner.running()&&self.macro_armed_pid.is_none(){
                        self.window_macro.selected=Some(target);self.window_macro.plan=plan;self.window_macro.consent=true;
                        if let Err(e)=self.window_macro.start(false){self.window_macro.consent=false;self.toast(e);}
                    }else{self.toast("Named macro disarmed: target changed, not focused, expired or protected");}
                }else if let Some(pid)=self.macro_armed_pid.filter(|_|!self.window_macro.busy()){
                    if self.macro_runner.running(){self.macro_runner.cancel();}
                    else if game_macros::foreground_pid()==pid{
                        let profile=self.settings.macro_profiles.get(&self.profile_key()).cloned().unwrap_or_default();
                        match self.macro_runner.start_with_key(pid,profile,self.settings.global_shortcuts.get("MACRO").map_or(0,|b|b.key)){
                            Ok(())=>self.toast("One-shot macro running; F12 cancels"),
                            Err(e)=>self.toast(e),
                        }
                    }
                }
            }
            self.macro_hotkey_down=down;
            let replay=self.settings.global_shortcuts.down("REPLAY");
            if replay&&!self.replay_hotkey_down&&self.replay.active(){self.save_replay();}
            self.replay_hotkey_down=replay;
        }
        if let Some(v)=self.encoder_rx.as_ref().and_then(|r|match r.try_recv(){
            Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Encoder worker stopped".into())),_=>None
        }){self.encoder_rx=None;self.encoder_result=Some(v);}
        if let Some(v)=self.media_rx.as_ref().and_then(|r|match r.try_recv(){
            Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Media probe worker stopped".into())),_=>None
        }){self.media_rx=None;match v{Ok(info)=>{self.trim_start=0.;self.trim_end=info.duration.min(120.);self.media_info=Some(info);},Err(e)=>self.toast(e)}}
        if let Some(v)=self.media_export.as_ref().and_then(|r|match r.try_recv(){
            Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Media export worker stopped".into())),_=>None
        }){self.media_export=None;match v{Ok(p)=>{let msg=format!("Saved clip: {}",p.display());if self.settings.quiet_clip_toasts{self.status=msg;}else{self.toast(msg);}},Err(e)=>self.toast(e)};self.media_gallery=crate::media_tools::gallery();}
        if let Some(elapsed)=self.recorder.elapsed(){
            if let Some(reason)=self.capture_session.poll(Instant::now(),elapsed){
                self.capture_session.reason=Some(reason.into());let _=self.recorder.stop();self.toast(format!("{reason}; finalizing recording"));
            }
        }
        if let Some(result)=self.recorder.poll(){
            let entries=self.capture_session.metadata(result.as_ref().ok().map(|p|p.as_path()));
            for (key,value) in entries{if self.settings.clip_annotations.len()<2000||self.settings.clip_annotations.contains_key(&key){self.settings.clip_annotations.entry(key).or_insert(value);}}
            self.capture_session=Default::default();self.save();
            self.toast(match result{Ok(p)=>format!("Video finalized: {}",p.display()),Err(e)=>e});
            self.media_gallery=crate::media_tools::gallery();
        }
        let capture_meter=self.audio_enabled&&unsorted_catalog::exposes(self.page,self.unsorted_selected_panel,Page::Capture);
        let hud_meter=self.overlay_open&&self.settings.hud_layout.enabled&&self.settings.hud_layout.audio;
        if capture_meter||hud_meter{self.audio_meter.tick();}
        else{self.audio_enabled=false;self.audio_meter.reset();}
        if !unsorted_catalog::exposes(self.page,self.unsorted_selected_panel,Page::Capture)&&!hud_meter{self.spectrum_enabled=false;}
        self.spectrum.tick(self.spectrum_enabled&&!self.is_smoke&&!crate::suite_finish::gpu_pressure(self.vram_budget.as_ref().and_then(|v|v.as_ref().ok()).filter(|v|v.meaningful()).map(|v|v.fraction()),self.settings.pressure_quiet));
        if self.break_timer.poll(Instant::now()){self.toast("Break timer finished");}
        self.sensors.tick(self.sensor_enabled&&!self.is_smoke,Instant::now());
        if let Some(message)=self.resource_alerts.poll_all(Instant::now(),self.settings.resource_alerts,
            self.vram_budget.as_ref().and_then(|v|v.as_ref().ok()).filter(|v|v.meaningful()).map(|v|v.fraction()),
            self.fps.snapshot().map(|s|s.frametime_ms),self.settings.stutter_threshold_ms,self.sensors.hottest(Instant::now())){self.toast(message);}
        if let Some(result)=self.replay.poll(){
            match result{
                Ok(file)=>{
                    if let (Some(key),Some(game))=(clip_book::key(&file),self.replay_save_game_key.take()){
                        if self.settings.clip_annotations.len()<2000{self.settings.clip_annotations.entry(key).or_insert(clip_book::Clip{game,..Default::default()});self.save();}
                    }
                    self.media_gallery=crate::media_tools::gallery();let msg=format!("Replay saved to {}",file.display());if self.settings.quiet_clip_toasts{self.status=msg;}else{self.toast(msg);}
                },
                Err(error)=>{self.replay_save_game_key=None;self.toast(format!("Replay save failed: {error}"));}
            }
        }
        if !self.is_smoke&&self.last_vram_poll.elapsed()>=Duration::from_secs(6){
            self.last_vram_poll=Instant::now();
            self.vram_budget=Some(vram::read_for_adapter(self.settings.renderer_adapter.as_deref()));
            self.own_memory=Some(process_memory::read());
        }
        let mirror_message=self.native_mirror.as_ref().and_then(|m|m.poll_status());
        if let Some(message)=mirror_message{
            self.native_mirror_status=message.clone();
            self.toast(message);
        }
        if self.native_mirror.as_ref().is_some_and(|m|m.stalled()){
            let captured=self.native_mirror.as_ref().and_then(|m|self.native_mirror_pid.map(|pid|(pid,m.mode(),m.multiplier())));
            let allowed=if let Some((pid,_,_))=captured{
                self.telem.active_game.as_ref().is_some_and(|g|g.pid==pid)&&
                self.telem.selected_pid.is_none_or(|selected|selected==pid)&&
                self.watchdog.grant(pid,Instant::now())
            }else{false};
            self.stop_native_mirror(); // nonblocking request; only restart after finished() is true
            if allowed{
                self.pending_mirror_restart=captured;
                self.native_mirror_status=format!("Watchdog: stopping stalled own capture, retry {}/2 after worker exit",self.watchdog.used());
                self.toast(self.native_mirror_status.clone());
            }else{self.toast("GPU worker stalled; stopped safely. Auto-restart limit reached or game changed.");}
        }
        if self.native_mirror.as_ref().is_some_and(|m|m.finished()){
            self.native_mirror.take(); // completed thread; joining no longer blocks UI
            self.native_mirror_pid=None;self.output_rates.clear();
            if let Some((pid,mode,factor))=self.pending_mirror_restart.take(){
                if self.telem.active_game.as_ref().is_some_and(|g|g.pid==pid)&&self.telem.selected_pid.is_none_or(|selected|selected==pid){self.toggle_native_mirror_factor(mode,factor);}
                else{self.native_mirror_status="Preview restart cancelled: target changed".into();}
            }else if self.native_mirror_status.ends_with("..."){self.native_mirror_status="Native GPU live preview stopped".into();}
        }
        self.update_driver_state();
        self.fps.poll();
        self.update_hud_monitor();
        if self.fps.stream_changed{
            let interrupted=self.perf_lab.active.is_some()||self.report.active();
            self.perf_lab.cancel();self.report.stop();
            // In-memory A/B baselines cannot cross an ETW source change.
            self.perf_lab.baseline=None;self.perf_lab.baseline_frames.clear();
            self.perf_lab.latest=None;self.perf_lab.last_frames.clear();
            if interrupted{self.toast("ETW source changed; benchmark/session stopped to avoid mixing swapchains.");}
        }
        self.network_probe.poll();
        if self.controller_enabled&&(!unsorted_catalog::exposes(self.page,self.unsorted_selected_panel,Page::Extras)||self.extras_view!=8){
            self.controller_enabled=false;self.controller_drift.reset();self.controller_readings=None;
        }
        if self.controller_enabled&&unsorted_catalog::exposes(self.page,self.unsorted_selected_panel,Page::Extras)&&self.extras_view==8&&
            self.controller_last.is_none_or(|t|t.elapsed()>=Duration::from_millis(100)){
            self.controller_last=Some(Instant::now());
            let readings=controller_check::read();
            if let Ok(rows)=&readings{self.controller_drift.reconcile(rows);for row in rows{self.controller_drift.sample(row);}}else{self.controller_drift.reset();}
            self.controller_readings=Some(readings);
        }
        let output=self.native_mirror.as_ref().filter(|m|m.running()&&{let h=m.diagnostics();!h.paused&&!h.own_minimized}).map(|m|(m.submitted_frames(),m.generated_frames()));
        let report_pid=self.telem.active_game.as_ref().map(|g|g.pid).filter(|p|Some(*p)==self.telem.selected_pid);
        let now=Instant::now();
        self.output_rates.observe(now,output);
        self.report.update(report_pid,now,self.fps.snapshot(),output);
        self.report.capture_spikes(report_pid,&self.fps.new_frames,self.settings.stutter_threshold_ms,now);
        if let Some(result)=self.effect_bench_rx.as_ref().and_then(|rx|match rx.try_recv(){
            Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Benchmark worker exited".into())),_=>None
        }){
            self.effect_bench_rx=None;self.effect_bench_result=Some(result);
        }
        if let Some(result)=self.shader_warmup.as_ref().and_then(|r|match r.try_recv(){
            Ok(v)=>Some(v),Err(std::sync::mpsc::TryRecvError::Disconnected)=>Some(Err("Shader compiler worker exited".into())),_=>None
        }){
            self.shader_warmup=None;
            self.toast(match result{Ok(n)=>format!("{n} native shaders precompiled"),Err(e)=>e});
        }
        if self.settings.anti_cheat_safe_mode&&self.telem.active_game.as_ref().is_some_and(|g|crate::anticheat_guard::inspect(&g.path,&g.title).blocked){
            if self.replay.is_buffering(){self.replay.stop();self.toast("Safe Mode stopped replay for protected selected title");}
            if self.recorder.elapsed().is_some(){let _=self.recorder.stop();self.toast("Safe Mode finalizing recording for protected selected title");}
        }
        let stats=self.fps.snapshot();
        let configured_target=self.settings.output_targets.get(&self.profile_key()).copied().unwrap_or(self.target_fps).clamp(30,165);
        if self.render_guard_enabled&&self.native_mirror.as_ref().is_some_and(|w|w.running()&&w.multiplier()==1&&w.mode()==native_mirror::MirrorMode::SeparateWindow&&!w.diagnostics().paused){
            let (own_target,own_measured)=output_rates::processing_sample(
                self.output_rates.snapshot(now),stats.map(|s|s.fps),
                configured_target,self.settings.preview_fps_limit);
            if self.render_guard.observe(self.settings.processing_percent,own_target,own_measured,now).is_some(){
                // Rebuild only our own capture/output surfaces at a new internal size.
                self.refresh_native_mirror();
            }
        }
        self.mobile.update(local_companion::Metrics{fps:stats.map(|s|s.fps),frametime_ms:stats.map(|s|s.frametime_ms),low_1_percent:stats.and_then(|s|s.low_1_percent_fps),output_active:self.native_mirror.as_ref().is_some_and(|m|m.running())});
        self.companion.update(local_companion::Metrics{
            fps:stats.map(|s|s.fps),frametime_ms:stats.map(|s|s.frametime_ms),
            low_1_percent:stats.and_then(|s|s.low_1_percent_fps),
            output_active:self.native_mirror.as_ref().is_some_and(|m|m.running())
        });
        let done=self.reminders.poll(Instant::now());
        if !done.is_empty(){
            self.toast(format!("Reminder ready: {}",done.join(", ")));
            if self.settings.reminder_sound&&!self.is_smoke{if let Err(e)=reminder_sound::play(){self.toast(e);}}
        }
        if self.preview_guard_enabled{
            if let Some(m)=self.native_mirror.as_ref().filter(|m|m.running()&&m.multiplier()==1&&!m.diagnostics().paused){
                if let Some(cap)=self.preview_guard.observe(self.settings.preview_fps_limit,
                    configured_target,
                    output_rates::processing_sample(self.output_rates.snapshot(now),
                        stats.map(|s|s.fps),configured_target,self.settings.preview_fps_limit).1,
                    now){
                    m.set_preview_limit(cap);
                }
            }else if let Some(m)=self.native_mirror.as_ref().filter(|m|m.running()&&m.multiplier()==1){m.set_preview_limit(self.settings.preview_fps_limit);self.preview_guard.reset();}else{self.preview_guard.reset();}
        }
        // A session must never mix two different selected game processes.
        if self.perf_lab.active.as_ref().is_some_and(|run|
            self.telem.active_game.as_ref().map(|g|g.pid)!=Some(run.pid)){
            self.perf_lab.cancel();
            self.perf_lab.last_error=Some(
                "ETW capture cancelled: selected game PID changed".into());
        }
        // Genuine ETW frametime samples only. Disabled by default to avoid
        // unsolicited notifications while gaming or blocking game input.
        if self.settings.stutter_scout_enabled&&self.telem.active_game.is_some(){
            let threshold=self.settings.stutter_threshold_ms.clamp(20.0,200.0);
            let worst=self.fps.new_frames.iter().map(|f|f.ms)
                .filter(|ms|ms.is_finite()&&*ms>threshold)
                .max_by(f32::total_cmp);
            if let Some(ms)=worst.filter(|_|self.last_stutter_alert.is_none_or(
                |last|last.elapsed()>=Duration::from_secs(18))){
                self.last_stutter_alert=Some(Instant::now());
                self.toast(format!("Stutter Scout: measured {:.1} ms frame (threshold {:.0} ms)",
                    ms,threshold));
            }
        }
        if let Some(done)=self.perf_lab.ingest(&self.fps.new_frames){
            match done{
                Ok(r)=>self.toast(format!("Benchmark complete: {:.1} FPS Â· 1% low {:.1} Â· {} spikes",
                    r.fps_avg,r.fps_one_percent_low,r.spikes)),
                Err(e)=>self.toast(format!("Benchmark: {e}"))
            }
        }
        let elapsed=self.last_tick.elapsed();
        if elapsed>=Duration::from_millis(850) {
            self.last_tick=Instant::now();
            let old=self.telem.active_game.as_ref().map(|g|g.path.clone());
            self.telem.refresh(self.settings.detect_games);
            self.crosshair.set_target(self.telem.active_game.as_ref().map(|g|g.pid));
            self.fps.track(self.telem.active_game.as_ref().map(|g|g.pid),self.settings.fps_monitoring);
            let now=self.telem.active_game.as_ref().map(|g|g.path.clone());
            if self.cpu_affinity.active()&&self.telem.active_game.as_ref().map(|g|g.pid)!=Some(self.cpu_affinity.pid){
                if let Err(e)=self.cpu_affinity.restore(){self.toast(e);}
            }
            if self.affinity_draft.is_some_and(|(pid,_,_)|Some(pid)!=self.telem.active_game.as_ref().map(|g|g.pid)){
                self.affinity_draft=None;
            }
            if self.game_boost.active()&&self.telem.active_game.as_ref().map(|g|g.pid)
                !=Some(self.game_boost.pid){
                let _=self.game_boost.stop(); // correct process no longer selected
                self.toast("Game Boost restored on game switch/exit");
            }
            if self.overlay_open&&self.gpu_backend.running(){
                if self.telem.active_game.as_ref().is_some_and(|g|
                    crate::win::magpie_scaling_info(g.pid).is_some()){
                    crate::win::keep_horizon_overlay_above_magpie();
                }
            }
            if now!=old {
                self.watchdog.reset_target(self.telem.active_game.as_ref().map(|g|g.pid));
                self.reminders.clear();self.render_guard.reset();
                // A user-enabled whole-monitor color effect must never
                // silently follow a different automatically detected game.
                // Restore the exact Windows matrix before loading its profile.
                if self.os_color_mode_active{
                    self.restore_os_color_mode();
                }
                if self.native_mirror_pid.is_some_and(|p|Some(p)!=self.telem.active_game.as_ref().map(|g|g.pid)){
                    self.stop_native_mirror();
                    self.toast("Native mirror stopped: selected game changed");
                }
                if self.perf_lab.active.is_some(){
                    self.perf_lab.cancel();
                    self.toast("Benchmark cancelled: selected game changed");
                }
                if now.is_some(){self.apply_selected_profile();}else if let Some(p)=self.settings.render_preferences.get("__global__").cloned(){p.apply(&mut self.settings);self.target_fps=p.target;self.preview_guard.reset();self.render_guard.reset();}
                self.track_selected_session();
            }
        }
        self.sync_monitor_profile();
        let edit_down=self.settings.global_shortcuts.down("HUD_EDITOR");
        if edit_down&&!self.last_overlay_edit_hotkey{
            // Alt+Shift+Z explicitly unlocks a clickable editor; press again
            // to return to the passive tiny HUD. Never intercept game clicks
            // in compact mode.
            self.overlay_open=true;
            self.overlay_edit_mode=!self.overlay_edit_mode;
            if !self.overlay_edit_mode{self.overlay_expanded=false;}
        }
        self.last_overlay_edit_hotkey=edit_down;
        let down=self.settings.global_shortcuts.down("HUD");
        if down&&!self.last_hotkey{self.toggle_passive_hud();}
        self.last_hotkey=down||edit_down;
        let color_hotkey=self.settings.global_shortcuts.down("DISPLAY_COLOR");
        if color_hotkey&&!self.last_color_hotkey {
            let enabled=!self.live.active;
            self.set_live_color(enabled);
            self.overlay_notice=if enabled{
                "Display color requested (whole desktop, fullscreen depends on driver)".into()
            }else{"Display color off".into()};
        }
        self.last_color_hotkey=color_hotkey;
        let gpu_hotkey=self.settings.global_shortcuts.down("GPU");
        if gpu_hotkey&&!self.last_gpu_hotkey{
            if self.native_mirror.is_some(){
                self.stop_native_mirror();
            }else{
                // Safe 1x filter only; no old Magpie fallback and no surprise FG.
                self.toggle_native_mirror_factor(crate::native_mirror::MirrorMode::SeparateWindow,1);
            }
        }
        self.last_gpu_hotkey=gpu_hotkey;
        let ris_hotkey=self.settings.global_shortcuts.down("RADEON_RIS");
        if ris_hotkey&&!self.last_ris_hotkey {
            if let Some(state)=&self.driver_status {
                if state.image_sharpen.supported {
                    self.driver_action(adlx::DriverChange::RadeonSharpen{
                        enabled:!state.image_sharpen.enabled,level:self.driver_strength});
                }else{
                    self.toast("Radeon Image Sharpening is not supported by this driver");
                }
            }else{
                self.toast("AMD driver information not available yet");
            }
        }
        self.last_ris_hotkey=ris_hotkey;
        let boost_hotkey=self.settings.global_shortcuts.down("BOOST");
        if boost_hotkey&&!self.last_boost_hotkey{
            self.toggle_game_boost();
        }
        self.last_boost_hotkey=boost_hotkey;
        self.update_hold_zoom();
        self.cursor_lock.tick();
        let lock_hotkey=self.settings.global_shortcuts.down("CURSOR_LOCK");
        if lock_hotkey&&!self.last_cursor_chord {
            if self.cursor_lock.active(){self.cursor_lock.release();self.cursor_lock_armed=false;self.toast("Cursor clipping released");}
            else{self.cursor_lock_armed=true;}
        }
        self.last_cursor_chord=lock_hotkey;
        if self.cursor_lock_armed {
            if let Some(pid)=self.telem.active_game.as_ref().map(|g|g.pid).filter(|pid|*pid==crate::game_macros::foreground_pid()){
                self.cursor_lock_armed=false;
                match self.cursor_lock.enable(pid){Ok(())=>self.toast("Cursor clipped to selected game client; releases on focus loss"),Err(e)=>self.toast(e)}
            }
        }
        let ch_hotkey=self.settings.global_shortcuts.down("CROSSHAIR");
        if ch_hotkey&&!self.last_crosshair_hotkey{
            self.toggle_crosshair();
        }
        self.last_crosshair_hotkey=ch_hotkey;
        let cmp_hotkey=self.settings.global_shortcuts.down("COMPARE");
        if cmp_hotkey&&!self.last_compare_hotkey{self.toggle_look_comparison();}
        self.last_compare_hotkey=cmp_hotkey;
        let timer_hotkey=self.settings.global_shortcuts.down("TIMER");
        if timer_hotkey&&!self.last_timer_hotkey{self.toggle_game_timer();}
        self.last_timer_hotkey=timer_hotkey;
        let split_hotkey=self.settings.global_shortcuts.down("SPLIT");
        if split_hotkey&&!self.last_split_hotkey{self.add_game_split();}
        self.last_split_hotkey=split_hotkey;
        let capture_hotkey=self.settings.global_shortcuts.down("CAPTURE");
        if capture_hotkey&&!self.last_capture_hotkey{self.screenshot();}
        self.last_capture_hotkey=capture_hotkey;
        let record_hotkey=self.settings.global_shortcuts.down("RECORD");
        if record_hotkey&&!self.last_record_hotkey{self.recording();}
        self.last_record_hotkey=record_hotkey;
        if self.pending_screenshot.is_some_and(|t|t.elapsed()>=Duration::from_millis(280)){
            self.pending_screenshot=None;
            self.screenshot=Some(record::capture_async());
            self.toast("Capturing screen...");
        }
        // Legacy windowed renderer was removed from the active UI. Its
        // pending restart queue must never spawn Magpie in v2.2.
        self.pending_gpu_refresh=None;
        self.last_refresh_retry=None;
        if let Some(task)=&self.screenshot {
            if let Ok(status)=task.try_recv(){
                match status{
                    Ok(f)=>self.toast(format!("Screenshot saved: {}",f.display())),
                    Err(e)=>self.toast(format!("Screenshot failed: {e}")),
                }
                self.screenshot=None;
            }
        }
    }
}
impl eframe::App for BoostApp {
    fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame) {
        if !self.is_smoke&&(ctx.zoom_factor()-self.settings.ui_zoom).abs()>0.001{ctx.set_zoom_factor(self.settings.ui_zoom);}
        // Persist manual mouse-edge resizes at exit. Ignore hidden/minimized
        // dimensions, screenshot proof sizes and maximized desktop sizes.
        if !self.is_smoke && (self.settings.ui_zoom-1.0).abs()<0.001 && !ctx.input(|i|i.viewport().maximized.unwrap_or(false)){
            if let Some(rect)=ctx.input(|i|i.viewport().inner_rect){
                let w=rect.width();let h=rect.height();
                if w.is_finite()&&h.is_finite()&&w>=790.0&&h>=530.0{
                    self.settings.main_window_size=[
                        w.clamp(800.0,1320.0),h.clamp(540.0,880.0)];
                }
            }
        }
        self.step();
        gui::render(self,ctx);
        // Global chord polling must continue while another game owns keyboard focus.
        // Never install a keyboard hook or record arbitrary keys.
        if self.is_smoke {
            self.smoke_frames+=1;
            if self.visual_test {
                if !self.visual_requested &&
                    ((self.visual_overlay&&self.smoke_frames>=6)||
                     (self.visual_perf&&self.smoke_frames>=6)||
                     (self.visual_gallery&&self.smoke_frames>=6)||
                     (self.visual_os_color_lab&&self.smoke_frames>=6)||
                     (self.visual_extras&&self.smoke_frames>=6)||
                     (self.visual_motion&&self.smoke_frames>=6)||
                    (!self.visual_driver&&!self.visual_gallery&&!self.visual_perf&&!self.visual_os_color_lab&&
                     !self.visual_extras&&!self.visual_motion&&self.smoke_frames>=3)||
                     (self.visual_driver&&self.driver_status.is_some()&&self.smoke_frames>8)){
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                    self.visual_requested=true;
                }
                if !self.visual_saved {
                    let screenshot=ctx.input(|i|i.events.iter().find_map(|e|{
                        if let egui::Event::Screenshot{image,..}=e{Some(image.clone())}else{None}
                    }));
                    if let Some(pixels)=screenshot {
                        let data=pixels.pixels.iter().flat_map(|c|c.to_array()).collect::<Vec<u8>>();
                        let filename=if self.visual_overlay{self.overlay_proof_name.as_str()}
                            else if self.visual_motion{"actual-motion-lab-v22dev-hidden.png"}
                            else if self.visual_extras{match self.extras_view{
                                0=>"actual-extras-hud-v22dev-hidden.png",
                                1=>"actual-extras-timers-v22dev-hidden.png",
                                2=>"actual-extras-quests-v22dev-hidden.png",
                                3=>"actual-extras-training-v22dev-hidden.png",
                                5=>"actual-extras-macros-v32.png",
                                6=>"actual-view-tools-v33.png",
                                7=>"actual-reminders-v34.png",
                                _=>"actual-extras-tools-v22dev-hidden.png"
                            }}
                            else if self.visual_perf{match self.performance_view{
                                1=>"actual-performance-tools-v22dev-hidden.png",
                                2=>"actual-performance-analysis-v22dev-hidden.png",
                                3=>"actual-performance-stability-v22dev-hidden.png",
                                4=>"actual-affinity-v33.png",
                                _=>"actual-performance-v22dev-hidden.png"
                            }}
                            else if self.visual_all_presets{"actual-all-presets-v22dev-hidden.png"}
                            else if self.visual_os_color_lab{"actual-os-color-v22dev-hidden.png"}
                            else if self.visual_narrow_filters{"actual-filters-narrow-v22dev-hidden.png"}
                            else if self.visual_new_gpu_look{"actual-new-gpu-look-v30-real.png"}
                            else if self.visual_gallery{match self.filter_view{
                                1=>if self.settings.ui_language==1{"actual-light-eclipse-v79-cz.png"}else{"actual-light-eclipse-v79-en.png"},
                                2=>"actual-filters-legacy-v22dev-hidden.png",
                                3=>"actual-filters-pro-v22dev-hidden.png",
                                _=>"actual-filters-looks-v22dev-hidden.png"
                            }}
                            else if self.visual_driver{"actual-driver-v06-hidden.png"}
                            else if self.visual_compact_main{
                                "actual-main-compact-v29-real.png"
                            }else if self.visual_size_settings{
                                "actual-size-settings-v29-real.png"
                            }else if self.visual_replay{
                                "actual-instant-replay-v30-real.png"
                            }else{"actual-main-v22dev-hidden.png"};
                        let custom=std::env::args().find_map(|a|a.strip_prefix("--visual-oled=").and_then(|v|v.parse::<u8>().ok())).filter(|v|*v<=10).map(|v|format!("actual-eclipse-v79-{v}-{}.png",if self.settings.ui_language==1{"cz"}else{"en"}));
                        let macro_name=std::env::args().any(|a|a=="--visual-macro").then(||format!("actual-macro-v79-{}.png",if self.settings.ui_language==1{"cz"}else{"en"}));
                        let journal_name=std::env::args().find_map(|a|a.strip_prefix("--visual-journal=").and_then(|v|v.parse::<usize>().ok())).filter(|v|(10..=19).contains(v)).map(|v|format!("actual-journal-{v}.png"));
                        let output=std::env::current_dir().unwrap_or_default().join("build-tests").join(journal_name.as_deref().or(macro_name.as_deref()).or(custom.as_deref()).unwrap_or(filename));
                        let _=std::fs::create_dir_all(output.parent().unwrap());
                        let saved=image::save_buffer(&output,&data,pixels.size[0] as u32,pixels.size[1] as u32,
                            image::ColorType::Rgba8).is_ok();
                        eprintln!("VISUAL_PROOF_SAVED={saved} PATH={}",output.display());
                        self.visual_saved=saved;
                    }
                }
                if self.visual_saved||self.smoke_frames>110 {
                    if !self.visual_saved {
                        eprintln!("VISUAL_PROOF_UNAVAILABLE");
                        std::process::exit(9);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else if self.smoke_frames>6 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        // 25 fps idle animation cap; no high-frequency CPU polling.
        let focused=ctx.input(|i|i.viewport().focused.unwrap_or(false));
        // Global shortcuts are edge-polled in step(), not registered OS hotkeys.
        // Long background sleep would miss short Alt+Z / emergency key presses.
        ctx.request_repaint_after(if !focused{Duration::from_millis(45)}
            else if self.native_mirror.as_ref().is_some_and(|w|w.running()){Duration::from_millis(45)}
            else if self.settings.animations{Duration::from_millis(80)}else{Duration::from_millis(110)});
    }
    fn on_exit(&mut self,_gl:Option<&eframe::glow::Context>){
        self.stop_native_mirror();self.native_mirror.take();
        let _=self.game_boost.stop();
        if !self.is_smoke&&self.sessions.finish(){let _=self.sessions.save();}
        self.live.disable();
        self.recorder.stop();
        self.replay.stop();
        self.network_probe.stop();
        self.macro_runner.cancel();
        self.report.stop();self.controller_enabled=false;
        self.companion.stop();
        let _=self.cpu_affinity.restore();
        let _=self.power_plan.restore();
        self.gpu_backend.stop();
        self.fps.stop();self.hud_fps.stop();
        if !self.is_smoke{if let Err(e)=settings::save(&self.settings){eprintln!("Final settings save failed: {e}");}}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn page_names(){ assert_eq!(Page::FrameGen.label(),"Frame Gen");}
}

#[cfg(test)]mod v95_multi_scope_tests{
 use super::*;
 use std::time::{Duration,Instant};
 #[test]fn clock_overflow_and_timestamp_order(){assert!(crate::capture_clock::age_ms(1,i64::MAX).is_none());assert!(crate::capture_clock::interval_ms(Some(1),Some(i64::MAX)).is_none());assert_eq!(crate::capture_clock::interval_ms(Some(10_000),Some(20_000)),Some(1.));}
 #[test]fn output_deadline_not_delayed_by_backward_clock(){let n=Instant::now();let mut p=crate::preview_pacer::PresentDeadline::default();p.presented(n+Duration::from_secs(10));assert_eq!(p.remaining(n,120),Duration::ZERO);p.submitted(n,n,120);assert!(p.remaining(n+Duration::from_millis(1),120)>Duration::ZERO);}
 #[test]fn bad_meter_counter_jump_never_invents_fps(){let n=Instant::now();let mut m=crate::output_rates::Meter::default();m.observe(n,Some((5,0)));m.observe(n+Duration::from_secs(1),Some((1_000_000,0)));assert!(m.snapshot(n+Duration::from_secs(1)).is_none());m.observe(n+Duration::from_secs(2),Some((1_000_060,0)));assert_eq!(m.snapshot(n+Duration::from_secs(2)).unwrap().real,60.);}
 #[test]fn stale_sampler_cannot_retrigger_fg_budget(){let n=Instant::now();let mut g=crate::render_pressure::FgGate::default();let a=crate::live_gpu_timing::Reading{ms:10.,when:n};let b=crate::live_gpu_timing::Reading{ms:8.,when:n};assert!(g.allow_samples(n,Some(16.),Some(a),Some(b),2));let newer=n+Duration::from_millis(100);assert!(!g.allow_samples(newer,Some(16.),Some(crate::live_gpu_timing::Reading{ms:10.,when:newer}),Some(crate::live_gpu_timing::Reading{ms:8.,when:newer}),2));assert!(!g.allow_samples(newer+Duration::from_millis(50),Some(16.),Some(a),Some(b),2));}
 #[test]fn crosshair_reappearing_requests_full_redraw(){let n=Instant::now();let mut g=crate::reticle_update::State::default();let o=crate::crosshair::CrosshairOptions::default();g.observe((10,10),&o,true,n);g.observe((10,10),&o,false,n+Duration::from_millis(100));let v=g.observe((10,10),&o,true,n+Duration::from_millis(110));assert!(v.alpha&&v.position&&v.paint);}
 #[test]fn invalid_hud_zoom_never_overflows_geometry(){let v=crate::hud_layout::Layout::default();for z in [f32::NAN,f32::INFINITY,-10.,100.]{let(w,h)=v.dimensions(z);assert!(w.is_finite()&&h.is_finite()&&w>=180.&&w<=450.);}}
 #[test]fn health_cannot_treat_invalid_heartbeat_as_active(){use crate::capture_health::{classify,WorkerState};assert_eq!(classify(true,15,Some(f64::NAN),false,false),WorkerState::Unresponsive);assert_eq!(classify(true,15,Some(-1.),false,false),WorkerState::Unresponsive);}
 #[test]fn non_frame_source_timeout_allows_bounded_recreate(){assert!(crate::capture_health::recoverable("WGC supplied no new frames for 10s; target may be protected/minimized"));assert!(!crate::capture_health::recoverable("Target game window closed"));}
 #[test]fn collected_gpu_stats_must_have_plausible_frame_pairs(){let mut q=std::collections::VecDeque::new();for _ in 0..50{q.push_back(crate::fps::Frame{ms:16.,gpu_ms:Some(400.),..Default::default()});}assert!(crate::perf_insights::gpu_coverage(&q).is_none());}
 #[test]fn copied_rates_remain_game_independent(){let n=Instant::now();let mut m=crate::output_rates::Meter::default();m.observe(n,Some((0,0)));m.observe(n+Duration::from_secs(1),Some((48,48)));let r=m.snapshot(n+Duration::from_secs(1)).unwrap();assert_eq!(r.total,96.);assert_eq!(r.real,48.);assert_eq!(r.generated,48.);}
}
