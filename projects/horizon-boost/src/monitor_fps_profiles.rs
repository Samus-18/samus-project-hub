//! Explicit saved FPS target and own-output cap for each game and monitor.
use std::collections::BTreeMap;
use serde::{Serialize,Deserialize};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub struct Profile{pub target:u32,pub cap:u32}
impl Profile{pub fn valid(self)->bool{(30..=165).contains(&self.target)&&crate::preview_pacer::LIMITS.contains(&self.cap)}}
pub type Bank=BTreeMap<String,BTreeMap<String,Profile>>;
fn key(s:&str,max:usize)->bool{!s.trim().is_empty()&&s.len()<=max&&!s.chars().any(char::is_control)}
pub fn put(b:&mut Bank,game:&str,display:&str,p:Profile)->Result<(),String>{
 if !key(game,1024)||!key(display,64)||!p.valid(){return Err("Invalid monitor profile".into());}
 if !b.contains_key(game)&&b.len()>=128{return Err("Game monitor profile bank full".into());}
 let row=b.entry(game.into()).or_default();if !row.contains_key(display)&&row.len()>=16{return Err("Monitor profile bank full".into());}row.insert(display.into(),p);Ok(())
}
pub fn sanitize(b:&mut Bank){
 b.retain(|g,rows|{rows.retain(|d,p|key(d,64)&&p.valid());while rows.len()>16{rows.pop_last();}key(g,1024)&&!rows.is_empty()});while b.len()>128{b.pop_last();}
}
pub fn recommended(refresh:Option<u32>)->Option<Profile>{
 let r=refresh.filter(|r|*r>=30)?;
 let cap=crate::preview_pacer::LIMITS.iter().copied().filter(|c|*c>0&&*c<=r).max()?;
 Some(Profile{target:r.clamp(30,165),cap})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn profiles_are_game_and_display_isolated_and_saved(){let mut s=crate::settings::Settings::default();for(g,d,t,c)in [("a","monitor1",144,144),("a","monitor2",75,75),("b","monitor1",60,45)]{put(&mut s.monitor_fps_profiles,g,d,Profile{target:t,cap:c}).unwrap();}let r:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(r.monitor_fps_profiles["a"]["monitor1"].target,144);assert_eq!(r.monitor_fps_profiles["a"]["monitor2"].cap,75);assert_eq!(r.monitor_fps_profiles["b"]["monitor1"].cap,45);assert!(put(&mut s.monitor_fps_profiles,"a","bad",Profile{target:1,cap:7}).is_err());}
 #[test]fn nominal_refresh_uses_supported_cap_not_claimed_vrr(){assert_eq!(recommended(Some(144)),Some(Profile{target:144,cap:144}));assert_eq!(recommended(Some(120)),Some(Profile{target:120,cap:120}));assert_eq!(recommended(Some(77)),Some(Profile{target:77,cap:75}));assert_eq!(recommended(None),None);assert_eq!(recommended(Some(24)),None);}
}

pub fn chosen(bank:&Bank,game:&str,display:Option<&str>)->Option<Profile>{
 let d=display.filter(|d|key(d,64))?;
 bank.get(game)?.get(d).copied().filter(|v|v.valid())
}
#[cfg(test)]mod auto_profile_tests{
 use super::*;
 #[test]fn auto_only_exact_explicit_saved_pair(){
  let mut b=Bank::new();put(&mut b,"game A","display1",Profile{target:144,cap:120}).unwrap();
  assert_eq!(chosen(&b,"game A",Some("display1")),Some(Profile{target:144,cap:120}));
  assert_eq!(chosen(&b,"game B",Some("display1")),None);
  assert_eq!(chosen(&b,"game A",Some("display2")),None);
  assert_eq!(chosen(&b,"game A",None),None);
  assert!(!crate::settings::Settings::default().monitor_fps_auto_apply);
 }
 #[test]fn sanitize_invalid_saved_value_fails_closed(){
  let mut b=Bank::new();b.insert("g".into(),BTreeMap::from([("d".into(),Profile{target:20,cap:999})]));
  assert_eq!(chosen(&b,"g",Some("d")),None);sanitize(&mut b);assert!(b.is_empty());
 }
}
