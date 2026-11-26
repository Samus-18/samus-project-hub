//! Local, portable game graphic profile files. No game EXE attachment or DLL injection.
use serde::{Serialize,Deserialize};
use std::{fs,path::{Path,PathBuf},time::{SystemTime,UNIX_EPOCH}};
use crate::{settings::{GpuProfile,FilterProfile,Settings},stacks::CustomStage};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]
pub struct SharedProfile{
    pub format:u32,
    pub name:String,
    pub color:FilterProfile,
    pub gpu:GpuProfile,
    pub custom_stages:Vec<CustomStage>
}
impl Default for SharedProfile{
    fn default()->Self{
        Self{format:1,name:"My Horizon Boost Profile".into(),color:FilterProfile::default(),
            gpu:GpuProfile::default(),custom_stages:crate::stacks::default_custom()}
    }
}
impl SharedProfile{
    pub fn from_settings(settings:&Settings,name:&str)->Self{
        Self{format:1,name:name.into(),color:settings.profile.clone(),
            gpu:GpuProfile{scaler:settings.live_scaler,effect:settings.live_preset,
                strength:settings.live_strength,stack:settings.live_stack,
                capture_method:settings.live_capture_method,output_mode:settings.live_output_mode},
            custom_stages:settings.custom_stages.clone()}
    }
    pub fn validate(&mut self)->Result<(),String>{
        if self.format!=1{return Err("This profile uses an unsupported version".into());}
        self.name=self.name.chars().take(80).collect();
        self.color.clamp();
        self.gpu.sanitize();
        self.custom_stages.truncate(4);
        for s in &mut self.custom_stages{
            s.effect=s.effect.clamp(1,crate::engine::EFFECTS.len()-1);
            s.weight=if s.weight.is_finite(){s.weight.clamp(0.0,1.0)}else{0.5};
        }
        Ok(())
    }
}
pub fn export(settings:&Settings,name:&str)->Result<PathBuf,String>{
    let target=crate::settings::folder().join("profiles");
    fs::create_dir_all(&target).map_err(|e|e.to_string())?;
    let stem=name.chars().filter(|c|c.is_ascii_alphanumeric()||*c=='-'||*c=='_')
        .take(32).collect::<String>();
    let stem=if stem.is_empty(){"game".into()}else{stem};
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e|e.to_string())?.as_secs();
    let output=target.join(format!("{stem}-{stamp}.json"));
    let profile=SharedProfile::from_settings(settings,name);
    fs::write(&output,serde_json::to_vec_pretty(&profile).map_err(|e|e.to_string())?)
        .map_err(|e|e.to_string())?;
    Ok(output)
}
pub fn load(path:&Path)->Result<SharedProfile,String>{
    if !path.extension().is_some_and(|ext|ext.eq_ignore_ascii_case("json")){
        return Err("Only .json profile files are accepted".into());
    }
    let metadata=fs::metadata(path).map_err(|e|e.to_string())?;
    if metadata.len()>262144{
        return Err("Profile file is too large (256KB maximum)".into());
    }
    let data=fs::read(path).map_err(|e|e.to_string())?;
    let json=serde_json::from_slice::<serde_json::Value>(&data)
        .map_err(|e|format!("Invalid JSON profile: {e}"))?;
    if json.get("format").and_then(|x|x.as_u64())!=Some(1)||
        !json.get("gpu").is_some_and(|v|v.is_object())||
        !json.get("color").is_some_and(|v|v.is_object()){
        return Err("Not a Horizon Boost graphics profile: missing format, GPU or color data".into());
    }
    let mut profile=serde_json::from_value::<SharedProfile>(json)
        .map_err(|e|format!("Invalid Horizon Boost JSON profile: {e}"))?;
    profile.validate()?;
    Ok(profile)
}
pub fn apply(settings:&mut Settings,profile:SharedProfile){
    settings.profile=profile.color;
    settings.live_scaler=profile.gpu.scaler;
    settings.live_preset=profile.gpu.effect;
    settings.live_strength=profile.gpu.strength;
    settings.live_stack=profile.gpu.stack;
    settings.live_capture_method=profile.gpu.capture_method;
    settings.live_output_mode=profile.gpu.output_mode;
    settings.custom_stages=profile.custom_stages;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn portable_profile_roundtrip(){
        let settings=Settings::default();
        let p=SharedProfile::from_settings(&settings,"Zelda TotK");
        let data=serde_json::to_string(&p).unwrap();
        let mut copy:SharedProfile=serde_json::from_str(&data).unwrap();
        copy.validate().unwrap();
        assert_eq!(copy.name,"Zelda TotK");
        assert_eq!(copy.gpu.stack,settings.live_stack);
        assert_eq!(copy.custom_stages.len(),2);
    }
    #[test]fn malformed_profile_is_safely_clamped(){
        let mut p=SharedProfile::default();
        p.gpu.stack=usize::MAX;
        p.gpu.scaler=usize::MAX;
        p.custom_stages=vec![CustomStage{effect:999,weight:10.0};6];
        p.validate().unwrap();
        assert_eq!(p.gpu.stack,crate::stacks::STACKS.len()-1);
        assert!(p.custom_stages.iter().all(|s|s.weight<=1.0));
        assert_eq!(p.custom_stages.len(),4);
    }
}
