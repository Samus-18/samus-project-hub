use serde::{Serialize,Deserialize};
use crate::{settings::{Settings,FilterProfile,GpuProfile},lighting::LightTuning,
 crosshair::CrosshairOptions,game_macros::MacroProfile,gameplay::{QuestTask,CooldownSpec}};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct RenderPreferences{
 pub scale_mode:u8,pub scale_ratio:f32,pub quality:u8,pub cap:u32,pub vsync:bool,pub waitable:bool,pub tearing:bool,pub target:u32
}
impl Default for RenderPreferences{
 fn default()->Self{Self{scale_mode:0,scale_ratio:1.,quality:3,cap:45,vsync:true,waitable:false,tearing:false,target:60}}
}
impl RenderPreferences{
 pub fn capture(s:&Settings,target:u32)->Self{Self{scale_mode:s.native_scale_mode,scale_ratio:s.native_scale_ratio,quality:s.native_quality,cap:s.preview_fps_limit,vsync:s.preview_vsync,waitable:s.preview_low_latency,tearing:s.preview_tearing,target}}
 pub fn valid(&self)->bool{
  (self.scale_mode as usize)<crate::spatial::MODES.len()&&self.scale_ratio.is_finite()&&crate::spatial::clamp_ratio(self.scale_ratio)==self.scale_ratio
  &&self.quality<=3&&crate::preview_pacer::LIMITS.contains(&self.cap)&&(30..=165).contains(&self.target)
 }
 pub fn apply(&self,s:&mut Settings){s.native_scale_mode=self.scale_mode;s.native_scale_ratio=self.scale_ratio;
  s.native_quality=self.quality;s.preview_fps_limit=self.cap;s.preview_vsync=self.vsync;s.preview_low_latency=self.waitable;s.preview_tearing=self.tearing;}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Portable{
 pub format:String,pub preset:usize,pub stack:Option<Vec<usize>>,pub processing:u8,pub render:RenderPreferences,
 pub color:FilterProfile,pub legacy:GpuProfile,pub lighting:LightTuning,pub crosshair:CrosshairOptions,
 pub macro_profile:MacroProfile,pub reminders:Vec<crate::reminders::Spec>,
 pub cooldown:CooldownSpec,pub checklist:Vec<QuestTask>,pub note:String,pub favorites:Vec<usize>
}
impl Portable{
 pub fn capture(s:&Settings,key:&str,target:u32)->Self{
  let mut crosshair=s.crosshair.clone();crosshair.enabled=false;
  let mut color=s.profile.clone();color.enabled=false;
  Self{format:"HorizonBoost.GameProfile.1".into(),preset:s.native_preset,stack:s.native_stack_profiles.get(key).cloned(),
   processing:s.processing_percent,render:RenderPreferences::capture(s,target),color,
   legacy:s.gpu_profiles.get(key).cloned().unwrap_or_default(),lighting:s.lighting.clone(),crosshair,
   macro_profile:s.macro_profiles.get(key).cloned().filter(|p|p.validate().is_ok()).unwrap_or_default(),
   reminders:s.reminder_profiles.get(key).cloned().unwrap_or_else(crate::reminders::defaults),
   cooldown:s.cooldown_profiles.get(key).cloned().unwrap_or_default(),
   checklist:s.quest_checklists.get(key).cloned().unwrap_or_default(),
   note:s.game_notes.get(key).cloned().unwrap_or_default(),favorites:s.native_favorites.clone()}
 }
 pub fn validate(&mut self)->Result<(),String>{
  if self.format!="HorizonBoost.GameProfile.1"{return Err("Unsupported profile format".into());}
  if self.preset>=crate::native_presets::PRESETS.len()||!crate::render_budget::PERCENTAGES.contains(&self.processing)||!self.render.valid(){
   return Err("Invalid native preset or render configuration".into());
  }
  if let Some(stages)=&self.stack{crate::native_stack::validate(stages)?;}
  self.macro_profile.validate()?;
  if self.note.chars().count()>4000||self.checklist.len()>20||self.reminders.len()!=4||self.favorites.len()>83{
   return Err("Profile data exceeds allowed limits".into());
  }
  self.color.clamp();self.color.enabled=false;self.legacy.sanitize();self.lighting.clamp();
  if self.lighting.lut.name.as_ref().is_some_and(|n|crate::live_lut::load(n).is_err()){self.lighting.lut.name=None;}
  self.crosshair.clamp();self.crosshair.enabled=false;
  crate::reminders::sanitize(&mut self.reminders);self.cooldown.sanitize();crate::gameplay::sanitize_tasks(&mut self.checklist);
  self.favorites.retain(|i|*i<crate::native_presets::PRESETS.len());self.favorites.sort_unstable();self.favorites.dedup();
  Ok(())
 }
 pub fn encode(&self)->Result<String,String>{
  let mut p=self.clone();p.validate()?;serde_json::to_string_pretty(&p).map_err(|e|e.to_string())
 }
 pub fn decode(s:&str)->Result<Self,String>{
  if s.len()>65536{return Err("Profile JSON exceeds 64 KiB".into());}
  let mut p:Self=serde_json::from_str(s).map_err(|e|format!("Invalid profile JSON: {e}"))?;p.validate()?;Ok(p)
 }
 pub fn apply(mut self,s:&mut Settings,key:&str)->Result<u32,String>{
  self.validate()?;
  let k=key.to_owned();
  s.native_preset=self.preset;s.native_preset_profiles.insert(k.clone(),self.preset);
  if let Some(v)=self.stack{s.native_stack_profiles.insert(k.clone(),v);}else{s.native_stack_profiles.remove(key);}
  s.processing_percent=self.processing;s.processing_profiles.insert(k.clone(),self.processing);
  self.render.apply(s);s.render_preferences.insert(k.clone(),self.render.clone());
  s.profile=self.color.clone();s.profiles.insert(k.clone(),self.color.clone());s.custom_profiles.insert(k.clone(),self.color);
  s.gpu_profiles.insert(k.clone(),self.legacy.clone());s.custom_gpu_profiles.insert(k.clone(),self.legacy.clone());
  s.live_scaler=self.legacy.scaler;s.live_preset=self.legacy.effect;s.live_strength=self.legacy.strength;
  s.live_stack=self.legacy.stack;s.live_capture_method=self.legacy.capture_method;s.live_output_mode=self.legacy.output_mode;
  s.preset_selection.insert(k.clone(),0);s.look_pairs.remove(key);
  s.lighting=self.lighting.clone();s.lighting_profiles.insert(k.clone(),self.lighting);
  s.crosshair=self.crosshair.clone();s.crosshair_profiles.insert(k.clone(),self.crosshair);
  s.macro_profiles.insert(k.clone(),self.macro_profile);s.reminder_profiles.insert(k.clone(),self.reminders);
  s.cooldown_profiles.insert(k.clone(),self.cooldown);s.quest_checklists.insert(k.clone(),self.checklist);
  s.game_notes.insert(k,self.note);s.native_favorites=self.favorites;
  Ok(self.render.target)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn roundtrip_and_safe_application(){
  let mut s=Settings::default();s.native_stack_profiles.insert("game".into(),vec![37,50,39]);s.native_preset=3;
  s.crosshair.enabled=true;s.profile.enabled=true;s.game_notes.insert("game".into(),"note".into());
  s.native_favorites=vec![2,7];s.processing_percent=75;s.preview_fps_limit=60;
  let raw=Portable::capture(&s,"game",90).encode().unwrap();
  assert!(!raw.contains("game.exe"));let p=Portable::decode(&raw).unwrap();
  let mut dest=Settings::default();assert_eq!(p.apply(&mut dest,"target").unwrap(),90);
  assert_eq!(dest.native_stack_profiles["target"],vec![37,50,39]);assert_eq!(dest.processing_percent,75);
  assert_eq!(dest.preview_fps_limit,60);assert_eq!(dest.game_notes["target"],"note");
  assert!(!dest.crosshair.enabled&&!dest.profile.enabled);
 }
 #[test]fn malformed_unknown_oversized_rejected(){
  let mut p=Portable::capture(&Settings::default(),"g",60);p.processing=1;assert!(p.encode().is_err());
  let raw=Portable::capture(&Settings::default(),"g",60).encode().unwrap();
  let mut value:serde_json::Value=serde_json::from_str(&raw).unwrap();value["command"]=serde_json::json!("run");
  assert!(Portable::decode(&value.to_string()).is_err());assert!(Portable::decode(&"x".repeat(65537)).is_err());
 }
}
