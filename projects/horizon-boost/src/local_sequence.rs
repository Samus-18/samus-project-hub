//! Local visual sequence simulator, never sends keyboard or mouse events.
use serde::{Serialize,Deserialize};use std::time::Instant;
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]#[serde(deny_unknown_fields)]
pub struct Step{pub key:String,pub hold_ms:u32,pub gap_ms:u32}
#[derive(Clone,Debug,Serialize,Deserialize)]#[serde(deny_unknown_fields)]
pub struct Plan{pub steps:Vec<Step>}
impl Default for Plan{fn default()->Self{Self{steps:vec![Step{key:"E".into(),hold_ms:120,gap_ms:200}]}}}
impl Plan{
 pub fn duration(&self)->u32{self.steps.iter().map(|s|s.hold_ms.saturating_add(s.gap_ms)).fold(0,u32::saturating_add)}
 pub fn valid(&self)->bool{!self.steps.is_empty()&&self.steps.len()<=12&&self.duration()<=15000&&self.steps.iter().all(|s|crate::game_macros::key_code(&s.key).is_some()&&(20..=2000).contains(&s.hold_ms)&&(0..=3000).contains(&s.gap_ms))}
 pub fn at(&self,mut ms:u32)->Option<(usize,bool,u32)>{if !self.valid(){return None;}for(i,s)in self.steps.iter().enumerate(){if ms<s.hold_ms{return Some((i,true,s.hold_ms-ms));}ms-=s.hold_ms;if ms<s.gap_ms{return Some((i,false,s.gap_ms-ms));}ms-=s.gap_ms;}None}
}
#[derive(Clone,Default)]pub struct Player{start:Option<Instant>}
impl Player{
 pub fn start(&mut self,p:&Plan,now:Instant)->bool{if !p.valid(){return false;}self.start=Some(now);true}
 pub fn stop(&mut self){self.start=None;}
 pub fn active(&self)->bool{self.start.is_some()}
 pub fn tick(&mut self,p:&Plan,now:Instant)->Option<(usize,bool,u32)>{let n=self.start?;let ms=now.saturating_duration_since(n).as_millis().min(u32::MAX as u128) as u32;let out=p.at(ms);if out.is_none(){self.stop();}out}
}
#[cfg(test)]mod tests{use super::*;#[test]fn hold_gap_boundary_and_cancel(){let p=Plan::default();assert_eq!(p.at(0),Some((0,true,120)));assert_eq!(p.at(120),Some((0,false,200)));assert_eq!(p.at(320),None);let now=Instant::now();let mut r=Player::default();assert!(r.start(&p,now));assert!(r.tick(&p,now).is_some());r.stop();assert!(r.tick(&p,now).is_none());}}

#[derive(Clone,Default)]pub struct Window{pub plan:Plan,pub player:Player,pub open:bool}
impl Window{pub fn launch(&mut self,p:&Plan,now:Instant)->bool{if !p.valid(){return false;}self.plan=p.clone();self.open=true;self.player.start(p,now)}
 pub fn close(&mut self){self.open=false;self.player.stop();}
}
#[cfg(test)]mod window_tests{use super::*;#[test]fn snapshot_isolated_and_close_stops(){let mut w=Window::default();let mut p=Plan::default();assert!(w.launch(&p,Instant::now()));p.steps.clear();assert!(w.plan.valid());w.close();assert!(!w.open&&!w.player.active());}}
