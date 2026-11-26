//! Explicit Horizon preview color overrides by Windows logical output name.
//! No monitor calibration, ICC profile, desktop, driver or game settings changes.
use crate::{color_grade::Grade,lighting::LightTuning};
use std::collections::BTreeMap;
pub type Profiles=BTreeMap<String,Grade>;
pub fn normalize(profiles:&mut Profiles){
 profiles.retain(|k,_|!k.is_empty()&&k.len()<=64&&!k.chars().any(char::is_control));
 while profiles.len()>32{profiles.pop_last();}
 for g in profiles.values_mut(){g.clamp();}
}
pub fn store(profiles:&mut Profiles,key:&str,grade:&Grade)->bool{
 if key.is_empty()||key.len()>64||key.chars().any(char::is_control)||(!profiles.contains_key(key)&&profiles.len()>=32){return false;}
 let mut g=grade.clone();g.clamp();profiles.insert(key.to_owned(),g);true
}
pub fn apply(t:&mut LightTuning,profiles:&Profiles,output:Option<&str>)->bool{
 let Some(g)=output.and_then(|key|profiles.get(key))else{return false};
 t.grade=g.clone();true
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn separate_override_and_no_leak(){let base=LightTuning::default();let mut g=Grade::default();g.enabled=true;g.exposure=0.75;let mut p=Profiles::new();assert!(store(&mut p,"display1",&g));let mut t=base.clone();assert!(apply(&mut t,&p,Some("display1")));assert_eq!(t.grade.exposure,0.75);assert_eq!(base.grade.exposure,0.);let mut t=base.clone();assert!(!apply(&mut t,&p,Some("display2")));assert!(!apply(&mut t,&p,None));assert_eq!(t,base);}
 #[test]fn finite_bounded_legacy(){let mut p=Profiles::new();let mut g=Grade::default();g.exposure=f32::NAN;assert!(store(&mut p,"display1",&g));assert_eq!(p["display1"].exposure,0.);assert!(!store(&mut p,"",&g));let s:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert!(s.display_grades.is_empty());}
}
