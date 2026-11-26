use std::time::{Duration,Instant};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Spec{pub label:String,pub seconds:u32}
pub fn defaults()->Vec<Spec>{vec![
 Spec{label:"Ability".into(),seconds:15},Spec{label:"Potion".into(),seconds:30},
 Spec{label:"Boss phase".into(),seconds:60},Spec{label:"Break".into(),seconds:300}]}
pub fn sanitize(v:&mut Vec<Spec>){
 v.truncate(4);
 for s in v.iter_mut(){s.seconds=s.seconds.clamp(3,3600);s.label=s.label.trim().chars().filter(|c|!c.is_control()).take(32).collect();if s.label.is_empty(){s.label="Reminder".into();}}
}
#[derive(Default)]
pub struct Reminders{ends:[Option<Instant>;4],paused:[Option<Duration>;4],totals:[Duration;4],labels:[String;4]}
impl Reminders{
 pub fn start(&mut self,i:usize,s:&Spec,now:Instant){
  if i<4{let d=Duration::from_secs(s.seconds.clamp(3,3600)as u64);self.totals[i]=d;self.paused[i]=None;self.ends[i]=Some(now+d);self.labels[i]=s.label.trim().chars().filter(|c|!c.is_control()).take(32).collect();if self.labels[i].is_empty(){self.labels[i]="Reminder".into();}}
 }
 pub fn stop(&mut self,i:usize){if i<4{self.ends[i]=None;self.paused[i]=None;}}
 pub fn clear(&mut self){self.ends=[None;4];self.paused=[None;4];}
  pub fn active_count(&self)->usize{self.ends.iter().filter(|t|t.is_some()).count()}
 pub fn paused_count(&self)->usize{self.paused.iter().filter(|t|t.is_some()).count()}
 pub fn active_label(&self,i:usize)->Option<&str>{(self.ends.get(i)?.is_some()||self.paused.get(i)?.is_some()).then(||self.labels[i].as_str())}
 pub fn paused(&self,i:usize)->bool{self.paused.get(i).is_some_and(Option::is_some)}
 pub fn pause(&mut self,i:usize,now:Instant){if i<4&&self.ends[i].is_some_and(|t|t>now){self.paused[i]=self.ends[i].take().map(|t|t.saturating_duration_since(now));}}
 pub fn resume(&mut self,i:usize,now:Instant){if i<4&&let Some(d)=self.paused[i].take(){self.ends[i]=Some(now+d);}}
 pub fn pause_all(&mut self,now:Instant){for i in 0..4{self.pause(i,now);}}
 pub fn resume_all(&mut self,now:Instant){for i in 0..4{self.resume(i,now);}}
 fn left(&self,i:usize,now:Instant)->Option<Duration>{self.paused.get(i).copied().flatten().or_else(||self.ends.get(i).copied().flatten().map(|t|t.saturating_duration_since(now)))}
 pub fn remaining(&self,i:usize,now:Instant)->Option<u64>{self.left(i,now).map(|d|d.as_millis().div_ceil(1000)as u64)}
 pub fn fraction(&self,i:usize,now:Instant)->Option<f32>{Some((self.left(i,now)?.as_secs_f32()/self.totals.get(i)?.as_secs_f32().max(0.001)).clamp(0.,1.))}

 pub fn poll(&mut self,now:Instant)->Vec<String>{
  let mut done=Vec::new();
  for i in 0..4{if self.ends[i].is_some_and(|t|t<=now){self.ends[i]=None;done.push(self.labels[i].clone());}}
  done
 }
 pub fn next_seconds(&self,now:Instant)->Option<u64>{self.ends.iter().flatten().map(|t|t.saturating_duration_since(now).as_millis().div_ceil(1000)as u64).min()}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn independent_once_only_reminders(){
  let mut t=Reminders::default();let now=Instant::now();let a=defaults();
  t.start(0,&a[0],now);t.start(1,&a[1],now);
  assert_eq!(t.poll(now+Duration::from_secs(16)),vec!["Ability"]);
  assert!(t.poll(now+Duration::from_secs(17)).is_empty());
  assert_eq!(t.poll(now+Duration::from_secs(31)),vec!["Potion"]);
 }
}

#[cfg(test)]mod pause_tests{use super::*;#[test]fn pause_is_exact_and_not_reported_as_due(){let now=Instant::now();let mut r=Reminders::default();let s=Spec{label:"Boss".into(),seconds:3};r.start(0,&s,now);r.pause_all(now+Duration::from_millis(1500));assert_eq!(r.remaining(0,now+Duration::from_secs(10)),Some(2));assert_eq!(r.fraction(0,now),Some(0.5));assert!(r.next_seconds(now).is_none());assert!(r.poll(now+Duration::from_secs(10)).is_empty());r.resume_all(now+Duration::from_secs(10));assert!(r.poll(now+Duration::from_millis(11499)).is_empty());assert_eq!(r.poll(now+Duration::from_millis(11500)),vec!["Boss"]);}}

#[cfg(test)]mod active_tests{use super::*;#[test]fn active_label_and_counts_survive_pause(){let n=Instant::now();let mut r=Reminders::default();r.start(0,&Spec{label:"  test  ".into(),seconds:10},n);assert_eq!(r.active_label(0),Some("test"));assert_eq!(r.active_count(),1);r.pause(0,n);assert_eq!(r.paused_count(),1);assert_eq!(r.active_count(),0);r.clear();assert_eq!(r.active_label(0),None);}}
