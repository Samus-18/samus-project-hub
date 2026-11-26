//! Only explicit, reversible Horizon-side Minecraft GPU effects.
//! Never modifies Minecraft files, renderer options, keys, or process memory.
use serde::{Serialize,Deserialize};
use crate::{compare::NativeLook,settings::Settings};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default)]
pub struct Undo{pub native:NativeLook,pub budget:u8}
impl Default for Undo{fn default()->Self{Self{native:NativeLook::default(),budget:100}}}
pub fn apply(settings:&mut Settings,key:&str,kind:u8)->Result<(),String>{
 if key=="__global__"||key.is_empty()||key.len()>520{return Err("Choose a running game before applying a Minecraft profile".into());}
 if kind>3{return Err("Unknown Minecraft preset".into());}
 let previous=Undo{native:NativeLook::capture(settings,key),budget:settings.processing_percent};
 settings.minecraft_undo.entry(key.to_string()).or_insert(previous);
 settings.native_stack_profiles.remove(key);
 settings.lighting.layers.clear_ordered();
 // Explicit profile replaces only Horizon-side GPU processing.
 settings.lighting.enabled=false;
 settings.lighting.cinema.enabled=false;
 settings.lighting.grade.enabled=false;
 settings.lighting.grade=Default::default();
 let (look,quality,budget) = match kind{
 0=>(0,3,90),   // Natural game color grade
 1=>(4,2,85),   // Existing dark-region-only shader
 2=>(5,1,90),   // Warm cinematic profile
 _=>(2,2,65),   // Single lightweight shader
 };
 settings.native_preset=look;
 settings.native_quality=quality;
 settings.processing_percent=budget;
 let grade=&mut settings.lighting.grade;
 match kind{
 0=>{grade.enabled=true;grade.saturation=1.06;grade.contrast=1.03;},
 1=>{grade.enabled=true;grade.gamma=0.91;grade.saturation=1.03;grade.contrast=1.01;},
 2=>{grade.enabled=true;grade.saturation=1.10;grade.contrast=1.09;grade.temperature=0.11;grade.exposure=0.06;
     settings.lighting.enabled=true;settings.lighting.glow=0.22;settings.lighting.film=0.17;},
 _=>{},
 }
 settings.lighting.clamp();
 settings.native_preset_profiles.insert(key.to_string(),look);
 settings.processing_profiles.insert(key.to_string(),budget);
 settings.lighting_profiles.insert(key.to_string(),settings.lighting.clone());
 settings.minecraft_active.insert(key.to_string(),kind);
 Ok(())
}
pub fn restore(settings:&mut Settings,key:&str)->Result<(),String>{
 let undo=settings.minecraft_undo.remove(key).ok_or("No saved Minecraft preset to restore")?;
 undo.native.apply(settings,key);
 settings.processing_percent=undo.budget.clamp(50,100);
 settings.processing_profiles.insert(key.to_string(),settings.processing_percent);
 settings.minecraft_active.remove(key);
 Ok(())
}
pub fn sanitize(settings:&mut Settings){
 settings.minecraft_active.retain(|k,v|k.len()<=520&&*v<=3);
 settings.minecraft_undo.retain(|k,v|{
  if k.len()>520{return false;}
  v.native.sanitize();v.budget=v.budget.clamp(50,100);true
 });
 if settings.minecraft_undo.len()>100{settings.minecraft_undo.clear();settings.minecraft_active.clear();}
 settings.minecraft_active.retain(|k,_|settings.minecraft_undo.contains_key(k));
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn profile_alters_only_explicit_target_and_is_reversible(){
  let mut s=Settings::default();s.native_quality=3;s.native_preset=7;s.processing_percent=80;
  let a="C:/games/Minecraft/javaw.exe";let b="C:/games/other.exe";
  s.native_preset_profiles.insert(b.into(),17);
  apply(&mut s,a,1).unwrap();assert_eq!(s.native_preset,4);assert_eq!(s.processing_percent,85);
  assert_eq!(s.native_preset_profiles.get(b),Some(&17));
  restore(&mut s,a).unwrap();assert_eq!(s.native_preset,7);assert_eq!(s.native_quality,3);assert_eq!(s.processing_percent,80);
  assert!(!s.minecraft_undo.contains_key(a));assert_eq!(s.native_preset_profiles.get(b),Some(&17));
 }
 #[test]fn cannot_apply_without_game_or_replace_original_undo(){
  let mut s=Settings::default();
  assert!(apply(&mut s,"__global__",1).is_err());
  apply(&mut s,"MINECRAFT",0).unwrap();let first=s.minecraft_undo.get("MINECRAFT").unwrap().native.preset;
  apply(&mut s,"MINECRAFT",3).unwrap();
  assert_eq!(s.minecraft_undo.get("MINECRAFT").unwrap().native.preset,first);
  restore(&mut s,"MINECRAFT").unwrap();
  assert!(restore(&mut s,"MINECRAFT").is_err());
 }
 #[test]fn profile_settings_roundtrip_and_invalid_values_rejected(){
  let mut s=Settings::default();apply(&mut s,"minecraft-java",2).unwrap();
  let json=serde_json::to_vec(&s).unwrap();
  let mut restored:Settings=serde_json::from_slice(&json).unwrap();sanitize(&mut restored);
  assert_eq!(restored.minecraft_active.get("minecraft-java"),Some(&2));
  restore(&mut restored,"minecraft-java").unwrap();
  assert!(apply(&mut restored,"minecraft-java",5).is_err());
 }
}
