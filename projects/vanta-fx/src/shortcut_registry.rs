//! Configurable global shortcuts. No hooks, keylogging or synthetic keystrokes.
//! Only Alt+Z is enabled by default; Ctrl+Shift+F12 emergency stop is reserved.
use std::collections::BTreeMap;
use serde::{Serialize,Deserialize};
pub const ACTIONS:[(&str,&str);17]=[
 ("HUD","Toggle passive HUD"),("HUD_EDITOR","Edit passive HUD"),
 ("GPU","Toggle own GPU output"),("DISPLAY_COLOR","Display color"),
 ("RADEON_RIS","Radeon Image Sharpening"),("BOOST","Game Boost"),
 ("CROSSHAIR","Crosshair"),("COMPARE","Compare looks"),
 ("TIMER","Game timer"),("SPLIT","Timer split"),("CAPTURE","Screenshot"),
 ("RECORD","Record"),("MARK","Session marker"),
 ("MACRO","Foreground macro"),("REPLAY","Save replay"),
 ("CURSOR_LOCK","Clip cursor to selected foreground game"),
 ("ZOOM","Hold 2x magnifier in own GPU output")];
pub const MODIFIERS:[(&str,u8);6]=[
 ("Alt",1),("Ctrl",2),("Ctrl+Shift",6),("Alt+Shift",5),
 ("Ctrl+Alt",3),("Ctrl+Alt+Shift",7)];
#[derive(Clone,Copy,Debug,Eq,PartialEq,Ord,PartialOrd,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chord{pub key:u16,pub mods:u8}
impl Chord {
 pub fn valid(self)->bool{
  (self.mods&!7)==0&&(self.mods&3)!=0 &&
  ((0x41..=0x5a).contains(&self.key)||(0x70..=0x7a).contains(&self.key)) &&
  !(self.key==0x73&&self.mods&1!=0)
 }
 pub fn label(self)->String{
  let m=MODIFIERS.iter().find(|x|x.1==self.mods).map(|x|x.0).unwrap_or("?");
  let key=if (0x41..=0x5a).contains(&self.key){char::from_u32(self.key as u32).unwrap().to_string()}
   else{format!("F{}",self.key.saturating_sub(0x6f))};
  format!("{m}+{key}")
 }
 pub fn active_with(self,key:bool,ctrl:bool,alt:bool,shift:bool,win:bool)->bool{
  self.valid()&&!win&&key&&ctrl==(self.mods&2!=0)&&alt==(self.mods&1!=0)&&shift==(self.mods&4!=0)
 }
 pub fn down(self)->bool{
  #[link(name="user32")]unsafe extern "system"{fn GetAsyncKeyState(vkey:i32)->i16;}
  let down=|v:i32|unsafe{GetAsyncKeyState(v) as u16&0x8000!=0};
  self.active_with(down(i32::from(self.key)),down(0x11),down(0x12),down(0x10),down(0x5b)||down(0x5c))
 }
}
#[derive(Clone,Debug,Eq,PartialEq,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct Registry{pub entries:BTreeMap<String,Chord>}
impl Default for Registry {
 fn default()->Self{Self{entries:BTreeMap::from([("HUD".into(),Chord{key:0x5a,mods:1})])}}
}
impl Registry {
 pub fn get(&self,action:&str)->Option<Chord>{self.entries.get(action).copied()}
 pub fn label(&self,action:&str)->String{self.get(action).map(Chord::label).unwrap_or_else(||"Unbound".into())}
 pub fn down(&self,action:&str)->bool{self.get(action).is_some_and(Chord::down)}
 pub fn assign(&mut self,action:&str,chord:Option<Chord>)->Result<(),String>{
  if !ACTIONS.iter().any(|x|x.0==action){return Err("Unknown action".into());}
  if let Some(chord)=chord{
   if !chord.valid(){return Err("Unsupported chord; use Alt or Ctrl with A-Z / F1-F11 (not Alt+F4)".into());}
   if let Some((other,_))=self.entries.iter().find(|(k,v)|k.as_str()!=action&&**v==chord){
    return Err(format!("Already bound to {other}"));
   }
   self.entries.insert(action.into(),chord);
  }else{self.entries.remove(action);}
  Ok(())
 }
 pub fn sanitize(&mut self){
  let mut result=Self{entries:BTreeMap::new()};
  for (key,_) in ACTIONS{
   if let Some(chord)=self.get(key){let _=result.assign(key,Some(chord));}
  }
  *self=result;
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn default_has_only_hud_and_mandatory_safety_remains_separate(){
  let r=Registry::default();assert_eq!(r.entries.len(),1);assert_eq!(r.label("HUD"),"Alt+Z");
  for(a,_)in ACTIONS{if a!="HUD"{assert_eq!(r.get(a),None);}}
 }
 #[test]fn exact_modifier_mask_prevents_accidental_activation(){
  let c=Chord{key:0x5a,mods:1};
  assert!(c.active_with(true,false,true,false,false));
  assert!(!c.active_with(true,false,true,true,false));
  assert!(!c.active_with(true,true,true,false,false));
  assert!(!c.active_with(true,false,true,false,true));
  assert!(!c.active_with(false,false,true,false,false));
 }
 #[test]fn collisions_unknowns_and_unsafe_hotkeys_rejected(){
  let mut r=Registry::default();let alt_z=Chord{key:0x5a,mods:1};
  assert!(r.assign("GPU",Some(alt_z)).is_err());
  assert!(r.assign("NOT_AN_ACTION",None).is_err());
  assert!(r.assign("GPU",Some(Chord{key:0x73,mods:1})).is_err());
  assert!(r.assign("GPU",Some(Chord{key:0x7b,mods:6})).is_err());
  assert!(r.assign("GPU",Some(Chord{key:0x47,mods:6})).is_ok());
  assert_eq!(r.label("GPU"),"Ctrl+Shift+G");
  assert!(r.assign("GPU",None).is_ok());assert_eq!(r.label("GPU"),"Unbound");
 }
 #[test]fn corrupt_old_profile_sanitized_and_serializable(){
  let mut s=crate::settings::Settings::default();
  s.global_shortcuts.assign("CROSSHAIR",Some(Chord{key:0x58,mods:6})).unwrap();
  let data=serde_json::to_string(&s).unwrap();
  let restored:crate::settings::Settings=serde_json::from_str(&data).unwrap();
  assert_eq!(restored.global_shortcuts,s.global_shortcuts);
  let mut invalid=Registry{entries:BTreeMap::from([("BOGUS".into(),Chord{key:22,mods:0}),("GPU".into(),Chord{key:0x5a,mods:1}),("HUD".into(),Chord{key:0x5a,mods:1})])};
  invalid.sanitize();assert_eq!(invalid.entries.len(),1);assert_eq!(invalid.get("HUD"),Some(Chord{key:0x5a,mods:1}));
  assert_eq!(serde_json::from_str::<Registry>("{}").unwrap(),Registry::default());
 }
}
