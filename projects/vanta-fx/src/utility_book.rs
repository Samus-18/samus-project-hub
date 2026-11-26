//! Explicit manual per-game tools. No input hooks, network or game memory access.
use serde::{Serialize,Deserialize};
pub const MAX:usize=32;
pub fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}
fn clean(v:&mut String,n:usize){*v=v.chars().take(n).collect();}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Timer{pub name:String,pub duration:u32,pub remaining:u32,pub deadline:Option<u64>,pub splits:Vec<u32>}
impl Default for Timer{fn default()->Self{Self{name:"Cooldown".into(),duration:60,remaining:60,deadline:None,splits:vec![]}}}
impl Timer{
 pub fn left(&self,t:u64)->u32{self.deadline.map_or(self.remaining,|x|x.saturating_sub(t).min(u64::from(self.duration)) as u32)}
 pub fn start(&mut self,t:u64){let left=self.left(t);if left>0{self.deadline=Some(t.saturating_add(u64::from(left)));}}
 pub fn pause(&mut self,t:u64){self.remaining=self.left(t);self.deadline=None;}
 pub fn reset(&mut self){self.remaining=self.duration;self.deadline=None;self.splits.clear();}
 pub fn split(&mut self,t:u64){if self.splits.len()<64{self.splits.push(self.duration.saturating_sub(self.left(t)));}}
 pub fn sanitize(&mut self){clean(&mut self.name,80);self.duration=self.duration.clamp(1,86400);self.remaining=self.remaining.min(self.duration);self.splits.truncate(64);for s in &mut self.splits{*s=(*s).min(self.duration);}}
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Quest{pub name:String,pub weekly:bool,pub hour:u8,pub weekday:u8,pub completed:Option<u64>}
impl Default for Quest{fn default()->Self{Self{name:"Daily quest".into(),weekly:false,hour:0,weekday:0,completed:None}}}
impl Quest{
 // UTC boundary: Monday=0. Unix epoch was Thursday. Explicitly not local DST.
 pub fn bucket(&self,t:u64)->i64{let day=(t.min(i64::MAX as u64) as i64-i64::from(self.hour.min(23))*3600).div_euclid(86400);if self.weekly{(day+3-i64::from(self.weekday.min(6))).div_euclid(7)}else{day}}
 pub fn done(&self,t:u64)->bool{self.completed.is_some_and(|x|x<=t&&self.bucket(x)==self.bucket(t))}
 pub fn set_done(&mut self,t:u64,v:bool){self.completed=v.then_some(t);}
 pub fn sanitize(&mut self){clean(&mut self.name,80);self.hour=self.hour.min(23);self.weekday=self.weekday.min(6);}
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Resource{pub name:String,pub amount:u32,pub cap:u32,pub interval:u32,pub anchor:u64}
impl Default for Resource{fn default()->Self{Self{name:"Energy".into(),amount:100,cap:100,interval:300,anchor:now()}}}
impl Resource{
 pub fn current(&self,t:u64)->u32{self.amount.saturating_add((t.saturating_sub(self.anchor)/u64::from(self.interval.max(1))).min(u64::from(u32::MAX)) as u32).min(self.cap)}
 pub fn settle(&mut self,t:u64){let step=u64::from(self.interval.max(1));let amount=self.current(t);self.anchor=if amount==self.cap||t<self.anchor{t}else{self.anchor.saturating_add(t.saturating_sub(self.anchor)/step*step)};self.amount=amount;}
 pub fn change(&mut self,t:u64,delta:i32){self.settle(t);self.amount=(i64::from(self.amount)+i64::from(delta)).clamp(0,i64::from(self.cap)) as u32;}
 pub fn sanitize(&mut self){clean(&mut self.name,80);self.cap=self.cap.clamp(1,1_000_000);self.amount=self.amount.min(self.cap);self.interval=self.interval.clamp(1,86400);}
}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Loadout{pub name:String,pub slots:Vec<Slot>,pub notes:String}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Slot{pub slot:String,pub item:String}
impl Loadout{fn sanitize(&mut self){clean(&mut self.name,80);clean(&mut self.notes,1600);self.slots.truncate(24);for s in &mut self.slots{clean(&mut s.slot,60);clean(&mut s.item,120);}}}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Book{pub timers:Vec<Timer>,pub quests:Vec<Quest>,pub resources:Vec<Resource>,pub loadouts:Vec<Loadout>}
impl Book{pub fn sanitize(&mut self){self.timers.truncate(MAX);self.quests.truncate(MAX);self.resources.truncate(MAX);self.loadouts.truncate(MAX);for x in &mut self.timers{x.sanitize();}for x in &mut self.quests{x.sanitize();}for x in &mut self.resources{x.sanitize();}for x in &mut self.loadouts{x.sanitize();}}}
// Conditional chance after already-failed rolls, with user-specified linear soft pity and hard guarantee.
pub fn pity(base:f64,soft:u32,increment:f64,hard:u32,failed:u32,next:u32)->Option<f64>{
 if !base.is_finite()||!increment.is_finite()||!(0.0..=100.).contains(&base)||!(0.0..=100.).contains(&increment)||hard==0||hard>10000||failed>=hard||next>10000||soft>hard{return None;}
 let mut survival=1.;
 for k in 1..=next{let roll=failed+k;if roll>=hard{return Some(100.);}let p=(base+increment*f64::from(roll.saturating_sub(soft))).min(100.)/100.;survival*=1.-p;}
 Some((1.-survival)*100.)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn timer_pause_resume_split(){let mut t=Timer::default();t.start(100);assert_eq!(t.left(125),35);t.split(125);t.pause(130);assert_eq!(t.left(900),30);t.start(200);assert_eq!(t.left(230),0);assert_eq!(t.splits,vec![25]);t.reset();assert_eq!(t.left(900),60);}
 #[test]fn quest_resets_utc(){let mut q=Quest{hour:4,..Default::default()};q.set_done(10000,true);assert!(q.done(14000));assert!(!q.done(14400));q.weekly=true;q.hour=0;q.weekday=0;q.set_done(3*86400,true);assert!(!q.done(4*86400));}
 #[test]fn resource_regeneration_cap_and_spend(){let mut r=Resource{amount:2,cap:10,interval:5,anchor:100,..Default::default()};assert_eq!(r.current(114),4);r.change(114,-3);assert_eq!(r.amount,1);assert_eq!(r.current(115),2);r.change(200,-1);assert_eq!(r.amount,9);assert_eq!(r.current(201),9);}
 #[test]fn pity_hard_and_soft(){assert_eq!(pity(1.,70,5.,90,89,1),Some(100.));assert_eq!(pity(0.,0,0.,90,0,89),Some(0.));assert!(pity(10.,2,20.,10,2,1).is_some_and(|x|(x-30.).abs()<1e-8));assert!(pity(1.,70,5.,90,90,1).is_none());}
 #[test]fn saved_profile_roundtrip(){let mut s=crate::settings::Settings::default();s.utility_profiles.entry("game-a".into()).or_default().timers.push(Timer::default());let v:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(v.utility_profiles["game-a"].timers.len(),1);assert!(!v.utility_profiles.contains_key("game-b"));}
}
