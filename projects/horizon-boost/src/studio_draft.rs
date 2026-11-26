//! User-controlled, bounded native GPU filter stack editor. No Magpie or game injection.
use serde::{Serialize,Deserialize};
use crate::{compare::NativeLook,settings::Settings};
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot{pub id:usize,pub weight:f32}
impl Slot{
 pub fn new(id:usize,weight:f32)->Self{Self{id,weight}}
}
fn valid(slots:&[Slot])->Result<(),String>{
 let ids=slots.iter().map(|v|v.id).collect::<Vec<_>>();
 crate::native_stack::validate(&ids)?;
 if slots.iter().any(|v|!v.weight.is_finite()||v.weight<0.||v.weight>1.){
  return Err("Shader strength must be finite between 0 and 1".into());
 }
 Ok(())
}
#[derive(Clone,Debug)]
pub struct Draft{
 pub key:String,
 pub slots:Vec<Slot>,
 past:Vec<Vec<Slot>>,future:Vec<Vec<Slot>>,
}
impl Draft{
 pub fn from_settings(s:&Settings,key:String)->Self{
  let ids=s.native_stack_profiles.get(&key).cloned()
   .unwrap_or_else(||crate::native_presets::PRESETS[s.native_preset].stages.to_vec());
  let slots=ids.iter().enumerate()
   .map(|(i,&id)|Slot::new(id,s.lighting.layers.strength_at(&ids,i))).collect();
  Self{key,slots,past:Vec::new(),future:Vec::new()}
 }
 fn record(&mut self,old:Vec<Slot>){
  if self.past.len()>=32{self.past.remove(0);}
  self.past.push(old);self.future.clear();
 }
 pub fn change(&mut self,next:Vec<Slot>)->Result<(),String>{
  if next.len()>crate::native_stack::MAX_STAGES{return Err("A GPU look supports at most 8 effects".into());}
  if next.iter().any(|s|!crate::native_stack::valid_stage(s.id)||!s.weight.is_finite()||s.weight<0.||s.weight>1.){
   return Err("Invalid native shader or strength".into());
  }
  if next!=self.slots{let old=std::mem::replace(&mut self.slots,next);self.record(old);}
  Ok(())
 }
 pub fn add(&mut self,id:usize)->Result<(),String>{
  let mut v=self.slots.clone();v.push(Slot::new(id,1.));self.change(v)
 }
 pub fn remove(&mut self,index:usize)->Result<(),String>{
  if index>=self.slots.len(){return Err("Effect index out of range".into());}
  let mut v=self.slots.clone();v.remove(index);self.change(v)
 }
 pub fn move_by(&mut self,index:usize,delta:isize)->Result<(),String>{
  let to=index as isize+delta;
  if index>=self.slots.len()||to<0||to as usize>=self.slots.len(){return Err("Cannot move effect outside stack".into());}
  let mut v=self.slots.clone();v.swap(index,to as usize);self.change(v)
 }
 pub fn strength(&mut self,index:usize,value:f32)->Result<(),String>{
  if index>=self.slots.len()||!value.is_finite(){return Err("Invalid effect strength".into());}
  let mut v=self.slots.clone();v[index].weight=value.clamp(0.,1.);
  self.change(v)
 }
 pub fn undo(&mut self)->bool{
  let Some(old)=self.past.pop()else{return false;};
  self.future.push(std::mem::replace(&mut self.slots,old));true
 }
 pub fn redo(&mut self)->bool{
  let Some(next)=self.future.pop()else{return false;};
  self.past.push(std::mem::replace(&mut self.slots,next));true
 }
 pub fn can_undo(&self)->bool{!self.past.is_empty()}
 pub fn can_redo(&self)->bool{!self.future.is_empty()}
 pub fn validate(&self)->Result<(),String>{valid(&self.slots)}
 pub fn ids(&self)->Vec<usize>{self.slots.iter().map(|s|s.id).collect()}
 pub fn tuned(&self,base:&crate::lighting::LightTuning)->Result<crate::lighting::LightTuning,String>{
  self.validate()?;
  let ids=self.ids();
  let mut light=base.clone();
  light.layers.clear_ordered();
  for(i,v)in self.slots.iter().enumerate(){light.layers.set_at(&ids,i,v.weight);}
  light.clamp();
  Ok(light)
 }
 pub fn commit(&self,settings:&mut Settings)->Result<(),String>{
  self.validate()?;
  if self.key.is_empty()||self.key=="__global__"||self.key.len()>520{return Err("Select a game before applying a stack".into());}
  if settings.minecraft_active.contains_key(&self.key){
   crate::minecraft_profile::restore(settings,&self.key)?;
  }
  if !settings.studio_previous.contains_key(&self.key){
   let previous=NativeLook::capture(settings,&self.key);
   settings.studio_previous.insert(self.key.clone(),previous);
  }
  let lights=self.tuned(&settings.lighting)?;
  settings.native_stack_profiles.insert(self.key.clone(),self.ids());
  settings.lighting=lights;
  settings.lighting_profiles.insert(self.key.clone(),settings.lighting.clone());
  settings.minecraft_active.remove(&self.key);
  if let Some(p)=settings.look_pairs.get_mut(&self.key){p.active=None;}
  Ok(())
 }
}
pub fn revert(settings:&mut Settings,key:&str)->Result<(),String>{
 let previous=settings.studio_previous.remove(key).ok_or("There is no previous applied look to restore")?;
 previous.apply(settings,key);
 if let Some(p)=settings.look_pairs.get_mut(key){p.active=None;}
 Ok(())
}
pub fn sanitize_previous(settings:&mut Settings){
 settings.studio_previous.retain(|key,v|{
  if key.is_empty()||key=="__global__"||key.len()>520{return false;}
  v.sanitize();true
 });
 if settings.studio_previous.len()>100{settings.studio_previous.clear();}
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Share{format:String,slots:Vec<Slot>}
pub fn export(slots:&[Slot])->Result<String,String>{
 valid(slots)?;
 serde_json::to_string_pretty(&Share{format:"HorizonBoost.NativeStudio.1".into(),slots:slots.to_vec()}).map_err(|e|e.to_string())
}
pub fn import(text:&str)->Result<Vec<Slot>,String>{
 if text.len()>8192{return Err("Native Studio JSON must be smaller than 8 KB".into());}
 let share:Share=serde_json::from_str(text).map_err(|e|format!("Invalid Studio JSON: {e}"))?;
 if share.format!="HorizonBoost.NativeStudio.1"{return Err("Unsupported Native Studio JSON version".into());}
 valid(&share.slots)?;Ok(share.slots)
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn duplicate_effect_strength_is_independent_and_move_preserves_weight(){
  let mut d=Draft::from_settings(&Settings::default(),"game.exe".into());
  d.change(vec![Slot::new(2,0.1),Slot::new(2,0.9)]).unwrap();
  d.move_by(1,-1).unwrap();
  assert_eq!(d.slots,vec![Slot::new(2,0.9),Slot::new(2,0.1)]);
  let tuned=d.tuned(&Default::default()).unwrap();
  assert_eq!(tuned.layers.strength_at(&[2,2],0),0.9);
  assert_eq!(tuned.layers.strength_at(&[2,2],1),0.1);
 }
 #[test]fn undo_redo_and_branch(){
  let mut d=Draft::from_settings(&Settings::default(),"game.exe".into());
  let before=d.slots.clone();
  d.add(2).unwrap();let after=d.slots.clone();
  assert!(d.undo());assert_eq!(d.slots,before);
  assert!(d.redo());assert_eq!(d.slots,after);
  assert!(d.undo());d.add(5).unwrap();assert!(!d.can_redo());
 }
 #[test]fn rejected_invalid_stages_and_strengths(){
  let mut d=Draft::from_settings(&Settings::default(),"game.exe".into());
  assert!(d.add(999).is_err());
  assert!(d.change(vec![Slot::new(2,f32::NAN)]).is_err());
  assert!(d.change(vec![Slot::new(2,1.);9]).is_err());
  assert!(d.change(vec![]).is_ok());assert!(d.validate().is_err());
 }
 #[test]fn strict_export_import_roundtrip_and_reject_scripts(){
  let ids=vec![Slot::new(2,0.2),Slot::new(2,0.8),Slot::new(4,1.)];
  let json=export(&ids).unwrap();assert_eq!(import(&json).unwrap(),ids);
  assert!(import(r#"{"format":"HorizonBoost.NativeStudio.1","slots":[{"id":2,"weight":1.0}],"cmd":"exec"}"#).is_err());
  assert!(import(r#"{"format":"HorizonBoost.NativeStudio.1","slots":[{"id":999,"weight":1.0}]}"#).is_err());
  assert!(import(r#"{"format":"HorizonBoost.NativeStudio.1","slots":[{"id":2,"weight":4.0}]}"#).is_err());
 }
 #[test]fn commit_restore_and_per_game_isolation(){
  let mut settings=Settings::default();
  settings.native_preset=5;settings.native_stack_profiles.insert("another.exe".into(),vec![7]);
  let mut d=Draft::from_settings(&settings,"minecraft.exe".into());
  d.change(vec![Slot::new(2,0.4),Slot::new(37,0.8)]).unwrap();
  d.commit(&mut settings).unwrap();
  assert_eq!(settings.native_stack_profiles["minecraft.exe"],vec![2,37]);
  assert_eq!(settings.native_stack_profiles["another.exe"],vec![7]);
  assert_eq!(settings.lighting.layers.strength_at(&[2,37],0),0.4);
  revert(&mut settings,"minecraft.exe").unwrap();
  assert_eq!(settings.native_preset,5);
  assert!(!settings.native_stack_profiles.contains_key("minecraft.exe"));
  assert!(revert(&mut settings,"minecraft.exe").is_err());
 }
 #[test]fn persisted_previous_look_is_bounded(){
  let mut settings=Settings::default();
  let d=Draft::from_settings(&settings,"game.exe".into());
  d.commit(&mut settings).unwrap();
  let saved=serde_json::to_string(&settings).unwrap();
  let mut loaded:Settings=serde_json::from_str(&saved).unwrap();
  sanitize_previous(&mut loaded);
  assert!(revert(&mut loaded,"game.exe").is_ok());
 }
}
