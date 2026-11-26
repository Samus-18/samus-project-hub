//! Bounded local library of named native GPU snapshots, independent of A/B slots.
use std::collections::BTreeMap;
use crate::compare::NativeLook;
pub const MAX_LOOKS:usize=128;
pub fn name(raw:&str)->Result<String,String>{
 let s=raw.trim();
 if s.is_empty()||s.chars().count()>48||s.chars().any(char::is_control){return Err("Use a name of 1 to 48 printable characters".into());}
 Ok(s.to_owned())
}
pub fn save(bank:&mut BTreeMap<String,NativeLook>,raw:&str,mut look:NativeLook,replace:bool)->Result<(),String>{
 let key=name(raw)?;
 if bank.contains_key(&key)&&!replace{return Err("Name exists; select Replace explicitly".into());}
 if !bank.contains_key(&key)&&bank.len()>=MAX_LOOKS{return Err("Named look library is full".into());}
 look.sanitize();bank.insert(key,look);Ok(())
}
pub fn sanitize(bank:&mut BTreeMap<String,NativeLook>){
 bank.retain(|k,v|{v.sanitize();name(k).is_ok_and(|n|n==*k)});
 while bank.len()>MAX_LOOKS{bank.pop_last();}
}
pub fn effects(query:&str)->Vec<usize>{
 crate::native_filter::PROGRAMS.iter().enumerate().filter_map(|(i,(label,_))|(crate::native_stack::valid_stage(i)&&crate::command_search::matches(query,label)).then_some(i)).collect()
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn every_valid_effect_discoverable(){let all=effects("");assert_eq!(all.len(),crate::native_filter::PROGRAMS.len()-2);assert!(all.contains(&63));assert_eq!(effects("CRT").len(),3);assert!(effects("unfindable").is_empty());}
 #[test]fn named_library_serializes_mixes_and_prevents_implicit_replace(){let mut s=crate::settings::Settings::default();let mut v=NativeLook{stack:Some(vec![5,7,5]),quality:0,..Default::default()};v.lighting.layers.set_at(&[5,7,5],2,0.25);save(&mut s.named_native_looks," Mine ",v.clone(),false).unwrap();assert!(save(&mut s.named_native_looks,"Mine",NativeLook::default(),false).is_err());let r:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(r.named_native_looks["Mine"],v);assert!(name("a\nb").is_err());}
}
