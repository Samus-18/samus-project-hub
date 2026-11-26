//! Per-game A/B look comparison. These are genuine graphics/color snapshots,
//! never fake performance modes and never automatic GPU driver changes.
use crate::settings::{FilterProfile,GpuProfile};
use crate::stacks::{CustomStage,default_custom};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]
pub struct LookSnapshot {
    pub color:FilterProfile,
    pub gpu:GpuProfile,
    pub custom_stages:Vec<CustomStage>,
    pub native:Option<NativeLook>,
}
impl Default for LookSnapshot {
    fn default()->Self{
        Self{color:FilterProfile::default(),gpu:GpuProfile::default(),custom_stages:default_custom(),native:None}
    }
}
impl LookSnapshot {
    pub fn capture(settings:&crate::settings::Settings)->Self{
        Self{color:settings.profile.clone(),
            gpu:GpuProfile{scaler:settings.live_scaler,effect:settings.live_preset,
                strength:settings.live_strength,stack:settings.live_stack,
                capture_method:settings.live_capture_method,output_mode:settings.live_output_mode},
            custom_stages:settings.custom_stages.clone(),native:None}
    }
    pub fn capture_native(settings:&crate::settings::Settings,key:&str)->Self{
        let mut value=Self::capture(settings);value.native=Some(NativeLook::capture(settings,key));value
    }
    pub fn sanitize(&mut self){
        if let Some(v)=self.native.as_mut(){v.sanitize();}
        self.color.clamp();
        self.gpu.sanitize();
        self.custom_stages.truncate(4);
        for stage in &mut self.custom_stages{
            stage.effect=stage.effect.clamp(1,crate::engine::EFFECTS.len()-1);
            stage.weight=if stage.weight.is_finite(){stage.weight.clamp(0.0,1.0)}else{0.5};
        }
    }
    pub fn apply(&self,settings:&mut crate::settings::Settings){
        settings.profile=self.color.clone();
        settings.live_scaler=self.gpu.scaler;
        settings.live_preset=self.gpu.effect;
        settings.live_strength=self.gpu.strength;
        settings.live_stack=self.gpu.stack;
        settings.live_capture_method=self.gpu.capture_method;
        settings.live_output_mode=self.gpu.output_mode;
        settings.custom_stages=self.custom_stages.clone();
    }
}
/// Native-only A/B snapshot: intentionally excludes driver, OS color, capture target and resolution.
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
#[serde(default)]
pub struct NativeLook{
    pub preset:usize,pub stack:Option<Vec<usize>>,pub lighting:crate::lighting::LightTuning,pub quality:u8,
}
impl Default for NativeLook{
    fn default()->Self{Self{preset:0,stack:None,lighting:Default::default(),quality:3}}
}
impl NativeLook{
    pub fn capture(s:&crate::settings::Settings,key:&str)->Self{
        let mut v=Self{preset:s.native_preset,stack:s.native_stack_profiles.get(key).cloned(),lighting:s.lighting.clone(),quality:s.native_quality};
        v.sanitize();v
    }
    pub fn sanitize(&mut self){
        self.preset=self.preset.min(crate::native_presets::PRESETS.len()-1);self.quality=self.quality.min(3);
        if self.stack.as_ref().is_some_and(|v|crate::native_stack::validate(v).is_err()){self.stack=None;self.lighting.layers.clear_ordered();}
        self.lighting.clamp();
        let stages=self.stack.as_deref().unwrap_or(crate::native_presets::PRESETS[self.preset].stages);
        if self.lighting.layers.ordered_ids!=stages{self.lighting.layers.clear_ordered();}
    }
    pub fn apply(&self,s:&mut crate::settings::Settings,key:&str){
        let mut v=self.clone();v.sanitize();s.native_preset=v.preset;s.native_quality=v.quality;
        s.native_preset_profiles.insert(key.into(),v.preset);
        if let Some(stack)=v.stack{s.native_stack_profiles.insert(key.into(),stack);}else{s.native_stack_profiles.remove(key);}
        s.lighting=v.lighting;s.lighting_profiles.insert(key.into(),s.lighting.clone());
    }
}
#[derive(Clone,Debug,Default,Serialize,Deserialize)]
#[serde(default)]
pub struct LookPair {
    pub a:Option<LookSnapshot>,
    pub b:Option<LookSnapshot>,
    pub active:Option<u8>
}
impl LookPair {
    pub fn stored(&self,slot:u8)->bool{self.slot(slot).is_some()}
    pub fn slot(&self,slot:u8)->Option<&LookSnapshot>{
        match slot{0=>self.a.as_ref(),1=>self.b.as_ref(),_=>None}
    }
    pub fn store(&mut self,slot:u8,p:LookSnapshot)->Result<(),String>{
        match slot{
            0=>self.a=Some(p),
            1=>self.b=Some(p),
            _=>return Err("A/B slot must be A or B".into())
        }
        self.active=Some(slot);
        Ok(())
    }
    pub fn activate(&mut self,slot:u8)->Option<LookSnapshot>{
        let selected=self.slot(slot)?.clone();
        self.active=Some(slot);
        Some(selected)
    }
    pub fn toggle(&mut self)->Option<(u8,LookSnapshot)>{
        let requested=if self.active==Some(0){1}else{0};
        if let Some(s)=self.activate(requested){return Some((requested,s));}
        let fallback=if requested==0{1}else{0};
        self.activate(fallback).map(|s|(fallback,s))
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn stores_and_switches_real_look_settings(){
        let mut config=crate::settings::Settings::default();
        let mut pair=LookPair::default();
        pair.store(0,LookSnapshot::capture(&config)).unwrap();
        config.live_stack=2;
        config.profile.brightness=0.2;
        pair.store(1,LookSnapshot::capture(&config)).unwrap();
        let (index,one)=pair.toggle().unwrap();
        assert_eq!(index,0);one.apply(&mut config);
        assert_eq!(config.live_stack,0);
        assert!(config.profile.brightness<0.2);
        let (index,two)=pair.toggle().unwrap();
        assert_eq!(index,1);two.apply(&mut config);
        assert_eq!(config.live_stack,2);
        assert_eq!(config.profile.brightness,0.2);
    }
    #[test]fn no_invented_slot_or_overflow(){
        let mut p=LookPair::default();
        assert!(p.toggle().is_none());
        assert!(p.store(7,LookSnapshot::default()).is_err());
        assert!(p.activate(1).is_none());
    }
    #[test]fn imported_snapshot_is_sanitized(){
        let mut p=LookSnapshot::default();
        p.gpu.stack=usize::MAX;
        p.gpu.strength=f32::NAN;
        p.custom_stages=vec![CustomStage{effect:999,weight:77.0};9];
        p.sanitize();
        assert_eq!(p.gpu.stack,crate::stacks::STACKS.len()-1);
        assert_eq!(p.custom_stages.len(),4);
        assert_eq!(p.gpu.strength,0.5);
    }
    #[test]fn per_game_json_roundtrip(){
        let pair=LookPair{a:Some(LookSnapshot::default()),b:None,active:Some(0)};
        let out=serde_json::to_string(&pair).unwrap();
        let copy:LookPair=serde_json::from_str(&out).unwrap();
        assert!(copy.stored(0)&&!copy.stored(1));
    }
}

#[cfg(test)]mod native_ab_tests{use super::*;
 #[test]fn native_slot_roundtrip_preserves_repeated_effects_and_strengths(){let mut s=crate::settings::Settings::default();let key="fixture";s.native_preset=2;s.native_quality=0;s.native_stack_profiles.insert(key.into(),vec![7,2,7]);s.lighting.layers.set_at(&[7,2,7],0,0.2);s.lighting.layers.set_at(&[7,2,7],2,0.8);let a=LookSnapshot::capture_native(&s,key);let raw=serde_json::to_string(&a).unwrap();let b:LookSnapshot=serde_json::from_str(&raw).unwrap();s.native_stack_profiles.clear();s.lighting=Default::default();s.native_quality=3;let ratio=s.native_scale_ratio;b.native.unwrap().apply(&mut s,key);assert_eq!(s.native_stack_profiles[key],vec![7,2,7]);assert_eq!(s.lighting.layers.strength_at(&[7,2,7],0),0.2);assert_eq!(s.lighting.layers.strength_at(&[7,2,7],2),0.8);assert_eq!(s.native_quality,0);assert_eq!(s.native_scale_ratio,ratio);}
 #[test]fn legacy_snapshot_never_invents_native_settings(){let a:LookSnapshot=serde_json::from_str("{}").unwrap();assert!(a.native.is_none());}
 #[test]fn native_preset_removes_stale_custom_stack(){let mut s=crate::settings::Settings::default();s.native_stack_profiles.insert("fixture".into(),vec![7,2]);let v=NativeLook{preset:1,..Default::default()};v.apply(&mut s,"fixture");assert!(!s.native_stack_profiles.contains_key("fixture"));assert_eq!(s.native_preset,1);}
}
