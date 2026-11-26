//! Isolated external GPU filter/scaling backend for Windows.
 //! Magpie is an independent GPLv3 application; it is NOT embedded into the Rust binary.
 //! No game DLL injection, no patching game files, no anti-cheat bypass.
use crate::win::GameInfo;
use serde_json::{Value,json};
use std::{
    fs,
    path::{Path,PathBuf},
    process::{Child,Command,Stdio},
    time::Duration,
};
use std::os::windows::process::CommandExt;
const NO_WINDOW:u32=0x08000000;
// Magpie is an independent process; IPC shutdown targets only the child PID.
#[derive(Clone,Copy)]
pub struct Effect {pub label:&'static str,pub desc:&'static str,pub name:Option<&'static str>}
pub const EFFECTS:&[Effect]=&[
    Effect{label:"Off / Upscaler only",desc:"Clean image; no extra shader",name:None},
    Effect{label:"Dynamic Vibrance+",desc:"Adaptive saturation, not merely a color matrix",name:Some("HorizonBoost\\HB_DynamicVibrance")},
    Effect{label:"Clarity+",desc:"Spatial high-pass microcontrast",name:Some("HorizonBoost\\HB_Clarity")},
    Effect{label:"Smart Sharpen+",desc:"Local adaptive sharpening with halo limiter",name:Some("HorizonBoost\\HB_SmartSharpen")},
    Effect{label:"Filmic Tone",desc:"Filmic response curve and highlight warmth",name:Some("HorizonBoost\\HB_Filmic")},
    Effect{label:"Soft Bloom",desc:"Highlight-aware multi-tap glow",name:Some("HorizonBoost\\HB_BloomLite")},
    Effect{label:"Lens Vignette",desc:"Smooth film lens edge darkening",name:Some("HorizonBoost\\HB_Vignette")},
    Effect{label:"Chromatic Lens",desc:"Lens-edge RGB aberration",name:Some("HorizonBoost\\HB_Chromatic")},
    Effect{label:"Anime Ink",desc:"Cartoon style edge outlines + posterization",name:Some("HorizonBoost\\HB_EdgeInk")},
    Effect{label:"Film Grain",desc:"Stylized image texture",name:Some("HorizonBoost\\HB_FilmGrain")},
    Effect{label:"Night Vision",desc:"Luminance-lift + night optics",name:Some("HorizonBoost\\HB_NightVision")},
    Effect{label:"Teal & Orange",desc:"Cinema shadow and highlight color grading",name:Some("HorizonBoost\\HB_TealOrange")},
    Effect{label:"Smart Deband",desc:"Edge-preserving smoothing of flat gradients",name:Some("HorizonBoost\\HB_DebandSoft")},
    Effect{label:"Color Separation",desc:"Red/green color separation assist",name:Some("HorizonBoost\\HB_ColorAssist")},
    Effect{label:"AMD FidelityFX CAS",desc:"Officially open-source AMD contrast adaptive sharpen",name:Some("CAS\\CAS")},
    Effect{label:"NVIDIA NVSharpen",desc:"NVIDIA image sharpening shader in Magpie",name:Some("NIS\\NVSharpen")},
    Effect{label:"LumaSharpen",desc:"Fine luma detail sharpening",name:Some("Sharpen\\LumaSharpen")},
    Effect{label:"FXAA High",desc:"Spatial anti-aliasing",name:Some("FXAA\\FXAA_High")},
    Effect{label:"SMAA High",desc:"Subpixel morphological anti-aliasing",name:Some("SMAA\\SMAA_High")},
    Effect{label:"Deband",desc:"Magpie's multi-pass debanding",name:Some("Deband")},
];
pub const SCALERS:&[&str]=&["AMD FSR 1 (EASU + RCAS)","NVIDIA Image Scaling (NIS)","Lanczos","Snapdragon SGSR (fast)","Anime4K 3D (stylized)","FSRCNNX (heavy)","CuNNy Veryfast (2x)"];
// Magpie v0.13 enum: GraphicsCapture=0, DesktopDuplication=1, GDI=2.
pub const CAPTURE_METHODS:&[&str]=&["Windows Graphics Capture (WGC)","DXGI Desktop Duplication","GDI compatibility"];
// Legacy Magpie ONLY supports its original windowed output in HORIZON BOOST.
// The fullscreen scaling toggle from v1.4 was explicitly rejected by the user.
pub const OUTPUT_MODES:&[&str]=&["Legacy windowed output only"];
pub const CUSTOM_SHADERS:&[(&str,&str)]=&[
    ("HB_DynamicVibrance.hlsl",include_str!("../shaders/HB_DynamicVibrance.hlsl")),
    ("HB_Clarity.hlsl",include_str!("../shaders/HB_Clarity.hlsl")),
    ("HB_SmartSharpen.hlsl",include_str!("../shaders/HB_SmartSharpen.hlsl")),
    ("HB_Filmic.hlsl",include_str!("../shaders/HB_Filmic.hlsl")),
    ("HB_BloomLite.hlsl",include_str!("../shaders/HB_BloomLite.hlsl")),
    ("HB_Vignette.hlsl",include_str!("../shaders/HB_Vignette.hlsl")),
    ("HB_Chromatic.hlsl",include_str!("../shaders/HB_Chromatic.hlsl")),
    ("HB_EdgeInk.hlsl",include_str!("../shaders/HB_EdgeInk.hlsl")),
    ("HB_FilmGrain.hlsl",include_str!("../shaders/HB_FilmGrain.hlsl")),
    ("HB_NightVision.hlsl",include_str!("../shaders/HB_NightVision.hlsl")),
    ("HB_TealOrange.hlsl",include_str!("../shaders/HB_TealOrange.hlsl")),
    ("HB_DebandSoft.hlsl",include_str!("../shaders/HB_DebandSoft.hlsl")),
    ("HB_ColorAssist.hlsl",include_str!("../shaders/HB_ColorAssist.hlsl")),
];
pub fn find_magpie_exe()->Option<PathBuf>{
    let exe=std::env::current_exe().ok()?;
    for parent in exe.ancestors().take(8){
        let candidate=parent.join("engines").join("Magpie").join("Magpie.exe");
        if candidate.is_file(){return Some(candidate);}
    }
    None
}
fn install_shaders(dir:&Path)->Result<(),String>{
    let dest=dir.join("effects").join("HorizonBoost");
    fs::create_dir_all(&dest).map_err(|e|e.to_string())?;
    for (name,content) in CUSTOM_SHADERS {
        let output=dest.join(name);
        if fs::read_to_string(&output).ok().as_deref()!=Some(content){
            fs::write(&output,content).map_err(|e|e.to_string())?;
        }
    }
    Ok(())
}
fn base_scaler(scaler:usize)->Vec<Value>{
    match scaler {
        1=>vec![json!({"name":"NIS\\NIS","scalingType":1,"scale":{"x":1.0,"y":1.0},
              "parameters":{"sharpness":0.45}})],
        2=>vec![json!({"name":"Lanczos","scalingType":1,"scale":{"x":1.0,"y":1.0}})],
        3=>vec![json!({"name":"SGSR","scalingType":1,"scale":{"x":1.0,"y":1.0},
              "parameters":{"EdgeSharpness":2.0,"EdgeThreshold":8.0}})],
        4=>vec![json!({"name":"Anime4K\\Anime4K_3D_Upscale_US"})],
        5=>vec![json!({"name":"FSRCNNX\\FSRCNNX"})],
        6=>vec![json!({"name":"CuNNy2\\CuNNy-veryfast-NVL"})],
        _=>vec![
            json!({"name":"FSR\\FSR_EASU","scalingType":1,"scale":{"x":1.0,"y":1.0}}),
            json!({"name":"FSR\\FSR_RCAS","parameters":{"sharpness":0.75}}),
        ],
    }
}
fn effect_parameter(index:usize,strength:f32)->Value {
    let v=strength.clamp(0.0,1.0);
    match index {
        1..=13 =>json!({"strength":v}),
        14|15 =>json!({"sharpness":v}),
        16 =>json!({"sharpStrength":(v*1.5).max(0.01)}),
        _=>json!(null)
    }
}
fn build_modes(strength:f32)->Vec<Value>{
    let mut out=Vec::with_capacity(SCALERS.len()*EFFECTS.len());
    for (s,scaler) in SCALERS.iter().enumerate(){
        for (index,effect) in EFFECTS.iter().enumerate(){
            let mut effects=base_scaler(s);
            if let Some(name)=effect.name {
                let parameters=effect_parameter(index,strength);
                if parameters.is_null() {effects.push(json!({"name":name}));}
                else{effects.push(json!({"name":name,"parameters":parameters}));}
            }
            out.push(json!({"name":format!("HB | {} | {}",scaler,effect.label),"effects":effects}));
        }
    }
    out
}
fn generate_config(game:&GameInfo,scaler:usize,effect:usize,strength:f32,stack:usize,
    capture_method:usize,output_mode:usize,custom:&[crate::stacks::CustomStage])->Result<Value,String>{
    if game.class_name.is_empty(){return Err("Game window class is unknown".into());}
    if scaler>=SCALERS.len()||effect>=EFFECTS.len()||stack>=crate::stacks::STACKS.len(){
        return Err("Invalid GPU scaler, filter or visual stack".into());
    }
    if capture_method>=CAPTURE_METHODS.len()||output_mode>=OUTPUT_MODES.len(){
        return Err("Invalid Magpie capture or external display mode".into());
    }
    let raw=include_str!("../assets/magpie_template.json").trim_start_matches('\u{feff}');
    let mut cfg:Value=serde_json::from_str(raw).map_err(|e|e.to_string())?;
    let mut modes=build_modes(strength);
    let selected_mode=if stack==0{scaler*EFFECTS.len()+effect}else{
        let mut effects=base_scaler(scaler);
        let stages=if stack==crate::stacks::CUSTOM_INDEX {
            custom.iter().take(4).map(|s|(s.effect,s.weight*strength.clamp(0.0,1.0)))
                .collect::<Vec<_>>()
        }else{crate::stacks::active_stages(stack,strength)};
        for (index,amount) in stages{
            let Some(name)=EFFECTS.get(index).and_then(|e|e.name) else{continue;};
            let params=effect_parameter(index,amount);
            if params.is_null(){effects.push(json!({"name":name}));}
            else{effects.push(json!({"name":name,"parameters":params}));}
        }
        let ix=modes.len();
        modes.push(json!({"name":format!("HORIZON STACK | {}",crate::stacks::STACKS[stack].label),
            "effects":effects}));
        ix
    };
    let default=cfg["profiles"].as_array()
        .and_then(|v|v.first()).cloned().ok_or("Magpie profile template not found")?;
    let mut assigned=default.clone();
    assigned["name"]=json!(format!("Horizon Boost: {}",game.title));
    assigned["packaged"]=json!(false);
    assigned["pathRule"]=json!(game.path.to_string_lossy().to_string());
    assigned["classNameRule"]=json!(game.class_name);
    assigned["captureMethod"]=json!(capture_method);
    assigned["autoScale"]=json!(2); // Strictly windowed legacy mode; never force fullscreen resizing
    assigned["scalingMode"]=json!(selected_mode);
    assigned["3DGameMode"]=json!(true);
    assigned["frameRateLimiterEnabled"]=json!(false);
    cfg["profiles"]=json!([default,assigned]);
    cfg["scalingModes"]=json!(modes);
    cfg["showNotifyIcon"]=json!(false);
    cfg["autoCheckForUpdates"]=json!(false);
    cfg["alwaysRunAsAdmin"]=json!(false);
    Ok(cfg)
}
fn other_magpie_running()->bool {
    let s=sysinfo::System::new_all();
    s.processes().values().any(|p| {
        let n=p.name().to_string_lossy();
        n.eq_ignore_ascii_case("Magpie.exe") || n.eq_ignore_ascii_case("Magpie")
    })
}

#[derive(Default)]
pub struct ExternalGpu{
    child:Option<Child>,
    pub last_error:Option<String>,
    last_game:Option<PathBuf>,
}
impl ExternalGpu{
    pub fn running(&mut self)->bool {
        match self.child.as_mut(){
            Some(p)=>match p.try_wait(){
                Ok(None)=>true,
                Ok(Some(status))=>{
                    self.last_error=Some(format!("External GPU engine exited: {status}"));
                    self.child=None;false
                },
                Err(e)=>{self.last_error=Some(format!("Process status: {e}"));self.child=None;false},
            },
            None=>false
        }
    }
    pub fn start(&mut self,game:&GameInfo,scaler:usize,effect:usize,strength:f32,stack:usize,
        capture_method:usize,output_mode:usize,custom:&[crate::stacks::CustomStage])->Result<(),String>{
        if self.running(){return Err("GPU engine is already running. Stop it before editing the profile.".into());}
        if other_magpie_running(){
            return Err("Another Magpie instance is already running. Close that instance first; Horizon Boost will not interfere with it.".into());
        }
        let exe=find_magpie_exe().ok_or("External GPU backend is missing from engines/Magpie")?;
        let folder=exe.parent().ok_or("Magpie directory not available")?;
        install_shaders(folder)?;
        let config=generate_config(game,scaler,effect,strength,stack,capture_method,output_mode,custom)?;
        let config_dir=folder.join("config");
        fs::create_dir_all(&config_dir).map_err(|e|e.to_string())?;
        let temp=config_dir.join("config.horizon-tmp.json");
        let file=config_dir.join("config.json");
        fs::write(&temp,serde_json::to_vec_pretty(&config).map_err(|e|e.to_string())?)
            .map_err(|e|e.to_string())?;
        if file.exists(){fs::remove_file(&file).map_err(|e|e.to_string())?;}
        fs::rename(&temp,&file).map_err(|e|e.to_string())?;
        let child=Command::new(&exe).arg("-t").current_dir(folder)
            .creation_flags(NO_WINDOW)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e|format!("Magpie launch failed: {e}"))?;
        self.child=Some(child);
        self.last_game=Some(game.path.clone());
        self.last_error=None;
        Ok(())
    }
    pub fn stop(&mut self){
        if let Some(mut child)=self.child.take(){
            // Avoid broadcasting shutdown to an unrelated/user-started Magpie process.
            crate::win::post_registered_message_to_pid(child.id(),"WM_MAGPIE_QUIT");
            std::thread::spawn(move||{
                for _ in 0..60{
                    if matches!(child.try_wait(),Ok(Some(_))){return;}
                    std::thread::sleep(Duration::from_millis(50));
                }
                let _=child.kill();
                let _=child.wait();
            });
        }
    }
}
impl Drop for ExternalGpu{fn drop(&mut self){self.stop();}}
/// Runtime preflight: verify every one of the 20 user-visible choices points
/// to a real Magpie effect file. This does not claim a successful game render.
pub fn verify_effect_files()->Result<usize,String>{
    let exe=find_magpie_exe().ok_or("Magpie.exe unavailable in the distribution")?;
    let base=exe.parent().ok_or("Cannot locate Magpie engine directory")?.join("effects");
    let mandatory=["FSR\\FSR_EASU","FSR\\FSR_RCAS","NIS\\NIS","Lanczos","SGSR","Anime4K\\Anime4K_3D_Upscale_US","FSRCNNX\\FSRCNNX","CuNNy2\\CuNNy-veryfast-NVL"];
    let mut names=mandatory.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    for item in EFFECTS{
        if let Some(path)=item.name{names.push(path.to_owned());}
    }
    let mut unique=std::collections::BTreeSet::new();
    for name in names{
        if !unique.insert(name.clone()){continue;}
        let file=base.join(format!("{}.hlsl",name.replace('\\',"/")));
        if !file.is_file(){
            return Err(format!("Missing Magpie effect: {} (for choice {})",file.display(),name));
        }
    }
    Ok(EFFECTS.len())
}
/// Exposes precisely the ordered effect names supplied to the external renderer.
pub fn selected_effect_chain(scaler:usize,effect:usize,stack:usize,
    custom:&[crate::stacks::CustomStage])->Vec<String>{
    if scaler>=SCALERS.len()||stack>=crate::stacks::STACKS.len(){return Vec::new();}
    let mut names=base_scaler(scaler).iter()
        .filter_map(|v|v["name"].as_str().map(str::to_owned)).collect::<Vec<_>>();
    if stack==0{
        if let Some(name)=EFFECTS.get(effect).and_then(|e|e.name){names.push(name.to_owned());}
    }else{
        let stages=if stack==crate::stacks::CUSTOM_INDEX {
            custom.iter().take(4).map(|s|s.effect).collect::<Vec<_>>()
        }else{crate::stacks::STACKS[stack].stages.iter().map(|s|s.effect).collect()};
        for index in stages{
            if let Some(name)=EFFECTS.get(index).and_then(|e|e.name){names.push(name.to_owned());}
        }
    }
    names
}
pub fn verify_shader_library()->Result<usize,String>{
    for (name,src) in CUSTOM_SHADERS{
        for section in ["//!MAGPIE EFFECT","//!VERSION 4","//!PASS 1","//!OUT OUTPUT","float4 Pass1"]{
            if !src.contains(section){return Err(format!("{name}: missing {section}"));}
        }
        if !src.contains("GetInputPt()") && !src.contains("INPUT.SampleLevel"){
            return Err(format!("{name}: no visible image samples"));
        }
    }
    Ok(CUSTOM_SHADERS.len())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn shader_library_valid_structure(){assert_eq!(verify_shader_library().unwrap(),13);}
    #[test]fn mode_count(){assert_eq!(build_modes(0.45).len(),SCALERS.len()*EFFECTS.len());}
    #[test]fn new_upscalers_use_real_bundled_effect_names(){
        assert_eq!(SCALERS.len(),7);
        assert_eq!(base_scaler(3)[0]["name"],"SGSR");
        assert_eq!(base_scaler(4)[0]["name"],"Anime4K\\Anime4K_3D_Upscale_US");
        assert_eq!(base_scaler(5)[0]["name"],"FSRCNNX\\FSRCNNX");
        assert_eq!(base_scaler(6)[0]["name"],"CuNNy2\\CuNNy-veryfast-NVL");
        assert_eq!(selected_effect_chain(3,2,2,&[]).len(),5);
        assert_eq!(selected_effect_chain(5,2,0,&[]).len(),2);
    }
    #[test]fn visible_chain_is_same_as_serialized_gpu_config(){
        let g=GameInfo{pid:13,title:"Chain".into(),path:PathBuf::from(r"C:\Games\chain.exe"),
            class_name:"GameWin".into(),fullscreen_like:false};
        for (scaler,stack) in [(0,0),(1,2),(3,2),(4,0),(5,crate::stacks::CUSTOM_INDEX)]{
            let custom=crate::stacks::default_custom();
            let cfg=generate_config(&g,scaler,2,0.75,stack,0,0,&custom).unwrap();
            let ix=cfg["profiles"][1]["scalingMode"].as_u64().unwrap() as usize;
            let actual=cfg["scalingModes"][ix]["effects"].as_array().unwrap().iter()
                .filter_map(|v|v["name"].as_str().map(str::to_owned)).collect::<Vec<_>>();
            assert_eq!(actual,selected_effect_chain(scaler,2,stack,&custom));
        }
    }
    #[test]fn preset_config_creates_ordered_gpu_shader_chain(){
        let g=GameInfo{pid:10,title:"Test".into(),path:PathBuf::from(r"C:\Games\mygame.exe"),
            class_name:"TestClass".into(),fullscreen_like:false};
        let c=generate_config(&g,0,2,0.75,2,0,0,&[]).unwrap();
        let ix=c["profiles"][1]["scalingMode"].as_u64().unwrap() as usize;
        let modes=c["scalingModes"].as_array().unwrap();
        assert_eq!(ix,SCALERS.len()*EFFECTS.len());
        let fx=modes[ix]["effects"].as_array().unwrap();
        assert_eq!(fx.len(),6); // FSR EASU, FSR RCAS, then 4 filters.
        assert_eq!(fx[2]["name"],"HorizonBoost\\HB_Filmic");
        assert_eq!(fx[3]["name"],"HorizonBoost\\HB_BloomLite");
        assert_eq!(fx[4]["name"],"HorizonBoost\\HB_TealOrange");
        assert_eq!(fx[5]["name"],"HorizonBoost\\HB_Vignette");
        let fps=generate_config(&g,0,2,0.75,9,0,0,&[]).unwrap();
        let ix=fps["profiles"][1]["scalingMode"].as_u64().unwrap() as usize;
        assert_eq!(fps["scalingModes"][ix]["effects"].as_array().unwrap().len(),2);
    }
    #[test]fn user_editor_reaches_real_shader_chain(){
        let g=GameInfo{pid:10,title:"Test".into(),path:PathBuf::from(r"C:\Games\test.exe"),
            class_name:"TestWindow".into(),fullscreen_like:false};
        let input=[crate::stacks::CustomStage{effect:8,weight:0.3},
            crate::stacks::CustomStage{effect:5,weight:0.8}];
        let config=generate_config(&g,0,2,0.75,crate::stacks::CUSTOM_INDEX,0,0,&input).unwrap();
        let ix=config["profiles"][1]["scalingMode"].as_u64().unwrap() as usize;
        let effects=config["scalingModes"][ix]["effects"].as_array().unwrap();
        assert_eq!(effects.len(),4);
        assert_eq!(effects[2]["name"],"HorizonBoost\\HB_EdgeInk");
        assert_eq!(effects[3]["name"],"HorizonBoost\\HB_BloomLite");
        assert!((effects[3]["parameters"]["strength"].as_f64().unwrap()-0.6).abs()<1e-5);
    }
    #[test]fn legacy_windowed_only_never_rewrites_fullscreen_display(){
        let g=GameInfo{pid:55,title:"Capture".into(),path:PathBuf::from(r"C:\Games\capture.exe"),
            class_name:"CaptureWindow".into(),fullscreen_like:false};
        for capture in 0..CAPTURE_METHODS.len(){
            for output in 0..OUTPUT_MODES.len(){
                let cfg=generate_config(&g,6,2,0.8,2,capture,output,&[]).unwrap();
                assert_eq!(cfg["profiles"][1]["captureMethod"],capture);
                assert_eq!(cfg["profiles"][1]["autoScale"],2);
                let i=cfg["profiles"][1]["scalingMode"].as_u64().unwrap() as usize;
                assert_eq!(cfg["scalingModes"][i]["effects"][0]["name"],"CuNNy2\\CuNNy-veryfast-NVL");
            }
        }
        assert!(generate_config(&g,0,0,1.0,0,10,0,&[]).is_err());
        assert!(generate_config(&g,0,0,1.0,0,0,1,&[]).is_err());
        assert!(generate_config(&g,0,0,1.0,0,0,99,&[]).is_err());
    }
    #[test]fn config_is_isolated_and_matching(){
        let g=GameInfo{pid:10,title:"Test Game".into(),path:PathBuf::from(r"C:\Games\test.exe"),class_name:"SampleWindowClass".into(),fullscreen_like:false};
        let cfg=generate_config(&g,0,2,0.38,0,0,0,&[]).unwrap();
        assert_eq!(cfg["profiles"][1]["pathRule"],r"C:\Games\test.exe");
        assert_eq!(cfg["profiles"][1]["classNameRule"],"SampleWindowClass");
        assert_eq!(cfg["profiles"][1]["autoScale"],2);
        assert_eq!(cfg["profiles"][1]["scalingMode"],2);
        assert!((cfg["scalingModes"][2]["effects"][2]["parameters"]["strength"].as_f64().unwrap()-0.38).abs()<1e-5);
        assert!((cfg["scalingModes"][14]["effects"][2]["parameters"]["sharpness"].as_f64().unwrap()-0.38).abs()<1e-5);
        assert_eq!(cfg["profiles"].as_array().unwrap().len(),2);
    }
}