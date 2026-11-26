
// Manual, per-game records. No game hooks, automation or network.
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Boss {pub name:String,pub attempts:u32,pub clears:u32,pub seconds:u64,pub best:Option<u32>}
impl Boss{
 pub fn record(&mut self,clear:bool,seconds:u32){
  if self.attempts>=1_000_000{return;}
  let seconds=seconds.min(86400);self.attempts+=1;
  self.seconds=self.seconds.saturating_add(u64::from(seconds));
  if clear{self.clears+=1;if seconds>0{self.best=Some(self.best.map_or(seconds,|v|v.min(seconds)));}}
 }
}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Loot{pub name:String,pub quantity:u32,pub note:String}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Entry{pub name:String,pub note:String,pub done:bool}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Counter{pub name:String,pub value:u32,pub goal:u32}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize)]
#[serde(default)]
pub struct Book{
 pub bosses:Vec<Boss>,pub loot:Vec<Loot>,pub routes:Vec<Entry>,
 pub builds:Vec<Entry>,pub achievements:Vec<Entry>,pub guides:Vec<Entry>,pub counters:Vec<Counter>
}
pub const MAX:usize=64;
pub fn clean(s:&mut String,max:usize){*s=s.trim().chars().take(max).collect();}
impl Book{
 pub fn sanitize(&mut self){
  self.bosses.truncate(MAX);for x in &mut self.bosses{clean(&mut x.name,80);x.attempts=x.attempts.min(1_000_000);x.clears=x.clears.min(x.attempts);x.seconds=x.seconds.min(u64::from(x.attempts)*86400);x.best=x.best.filter(|s|*s>0&&*s<=86400&&x.clears>0);}
  self.loot.truncate(MAX);for x in &mut self.loot{clean(&mut x.name,80);clean(&mut x.note,600);x.quantity=x.quantity.min(1_000_000);}
  for list in [&mut self.routes,&mut self.builds,&mut self.achievements,&mut self.guides]{
   list.truncate(MAX);for x in list{clean(&mut x.name,80);clean(&mut x.note,1600);}
  }
  self.counters.truncate(MAX);for x in &mut self.counters{clean(&mut x.name,80);x.value=x.value.min(1_000_000);x.goal=x.goal.clamp(1,1_000_000);}
 }
}
pub fn chance_at_least_one(percent:f64,n:u32)->Option<f64>{
 if !percent.is_finite()||!(0.0..=100.0).contains(&percent){return None;}
 if n==0||percent==0.{return Some(0.);}if percent==100.{return Some(100.);}
 Some((-((n as f64)*(-percent/100.).ln_1p()).exp_m1()*100.).clamp(0.,100.))
}
pub fn rolls_for_chance(percent:f64,target:f64)->Option<u64>{
 if !percent.is_finite()||!target.is_finite()||percent<=0.||percent>100.||target<=0.||target>=100.{return None;}
 if percent==100.{return Some(1);}
 let n=((-target/100.).ln_1p()/(-percent/100.).ln_1p()).ceil();
 if n.is_finite()&&n<=u64::MAX as f64{Some((n as u64).max(1))}else{None}
}
pub fn material(target:u32,per_craft:u32,owned:u32)->(u64,u64){
 let total=u64::from(target)*u64::from(per_craft);(total,total.saturating_sub(u64::from(owned)))
}
pub fn dps(damage:f64,seconds:f64)->Option<f64>{
 if damage.is_finite()&&seconds.is_finite()&&damage>=0.&&seconds>0.{let d=damage/seconds;d.is_finite().then_some(d)}else{None}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn boss_counts_and_best(){let mut b=Boss::default();b.record(false,20);b.record(true,45);b.record(true,30);assert_eq!((b.attempts,b.clears,b.seconds,b.best),(3,2,95,Some(30)));}
 #[test]fn drop_boundaries_and_exact(){assert_eq!(chance_at_least_one(100.,0),Some(0.));assert_eq!(chance_at_least_one(100.,1),Some(100.));assert_eq!(chance_at_least_one(0.,500),Some(0.));assert!((chance_at_least_one(50.,2).unwrap()-75.).abs()<1e-8);assert_eq!(chance_at_least_one(f64::NAN,1),None);}
 #[test]fn target_chance(){assert_eq!(rolls_for_chance(50.,75.),Some(2));assert_eq!(rolls_for_chance(0.,90.),None);assert_eq!(rolls_for_chance(100.,99.),Some(1));assert_eq!(rolls_for_chance(1.,100.),None);}
 #[test]fn material_never_underflows(){assert_eq!(material(4,5,7),(20,13));assert_eq!(material(4,5,99),(20,0));assert_eq!(material(u32::MAX,u32::MAX,0).0,u64::from(u32::MAX).pow(2));}
 #[test]fn damage_invalid(){assert_eq!(dps(300.,2.),Some(150.));assert_eq!(dps(300.,0.),None);assert_eq!(dps(-1.,2.),None);}
 #[test]fn bounded_and_roundtrip(){let mut b=Book::default();b.counters=vec![Counter{name:"x".repeat(200),value:2_000_000,goal:0};70];b.sanitize();assert_eq!(b.counters.len(),64);assert_eq!(b.counters[0].name.len(),80);assert_eq!(b.counters[0].goal,1);let x=serde_json::to_string(&b).unwrap();assert_eq!(serde_json::from_str::<Book>(&x).unwrap(),b);}
 #[test]fn legacy_settings_and_profile_isolation(){let mut s:crate::settings::Settings=serde_json::from_str("{}").unwrap();assert!(s.journal_profiles.is_empty());s.journal_profiles.entry("game-a".into()).or_default().bosses.push(Boss{name:"Boss".into(),..Default::default()});assert!(s.journal_profiles.entry("game-b".into()).or_default().bosses.is_empty());let s:crate::settings::Settings=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();assert_eq!(s.journal_profiles["game-a"].bosses[0].name,"Boss");}
}
