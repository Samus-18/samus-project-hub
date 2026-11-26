//! Bounded pressure policy for own capture and GPU work, no game control.
use std::time::{Duration,Instant};
#[derive(Default)]pub struct StaleGate{cap:Option<u32>,skipped:u8}
impl StaleGate{
 pub fn set_cap(&mut self,cap:Option<u32>){if cap!=self.cap{self.cap=cap;self.skipped=0;}}
 pub fn skip(&mut self,age:Option<f64>,queued:u64)->bool{
  let Some(cap)=self.cap else{return false;};
  let cap=if cap==0{60}else{crate::preview_pacer::valid_limit(cap)};
  let limit=(2000./cap as f64).max(40.);
  let skip=queued>=1&&age.is_some_and(|a|a.is_finite()&&a>limit)&&self.skipped<2;
  if skip{self.skipped+=1;}else{self.skipped=0;}skip
 }
}
pub fn idle_wait(since:Duration)->Duration{Duration::from_millis(if since>=Duration::from_secs(2){50}else if since>=Duration::from_millis(250){20}else{8})}
#[derive(Default)]pub struct FgGate{until:Option<Instant>,over:u8,last_sample:Option<(Instant,Instant)>}
impl FgGate{
 pub fn reset(&mut self){*self=Self::default();}
 pub fn penalize(&mut self,now:Instant){self.until=Some(now+Duration::from_millis(300));}
 pub fn allow_samples(&mut self,now:Instant,source_ms:Option<f64>,flow:Option<crate::live_gpu_timing::Reading>,warp:Option<crate::live_gpu_timing::Reading>,factor:u8)->bool{
  if factor<2{return true;}
  if self.until.is_some_and(|t|now<t){return false;}
  let (Some(f),Some(w))=(flow,warp)else{return true;};
  let pair=(f.when,w.when);
  if self.last_sample==Some(pair){return true;}
  if self.last_sample.is_some_and(|old|pair.0<=old.0||pair.1<=old.1){return true;}
  self.last_sample=Some(pair);self.allow(now,source_ms,Some(f.ms),Some(w.ms),factor)
 }
 pub fn allow(&mut self,now:Instant,source_ms:Option<f64>,flow:Option<f64>,warp:Option<f64>,factor:u8)->bool{
  if factor<2{return true;}
  if self.until.is_some_and(|t|now<t){return false;}
  let (Some(src),Some(flow),Some(warp))=(source_ms,flow,warp)else{return true;};
  if ![src,flow,warp].iter().all(|v|v.is_finite()&&*v>0.){return true;}
  let cost=flow+warp*f64::from(factor-1);
  if !cost.is_finite(){return true;}
  if cost>src*0.35{self.over=self.over.saturating_add(1);if self.over>=2{self.over=0;self.penalize(now);return false;}}
  else{self.over=0;}true
 }
}
pub fn fg_reason(n:u64)->&'static str{match n{0=>"Ready",1=>"Building frame history",2=>"Stale source",3=>"Capture queue busy",4=>"Cadence / refresh guard",5=>"GPU budget cooldown",6=>"GPU error; real frames only",_=>"Unknown"}}
#[cfg(test)]mod tests{use super::*;
 #[test]fn stale_gate_requires_known_age_and_newer_notification(){let mut g=StaleGate::default();g.set_cap(Some(60));assert!(!g.skip(None,3));assert!(!g.skip(Some(100.),0));assert!(!g.skip(Some(20.),2));assert!(g.skip(Some(100.),1));assert!(g.skip(Some(100.),1));assert!(!g.skip(Some(100.),1));assert!(g.skip(Some(100.),1));g.set_cap(None);assert!(!g.skip(Some(100.),3));}
 #[test]fn cooldown_is_bounded_and_missing_measurements_not_fabricated(){let mut g=FgGate::default();let n=Instant::now();assert!(g.allow(n,Some(16.),None,None,2));assert!(g.allow(n,Some(16.),Some(8.),Some(3.),2));assert!(!g.allow(n,Some(16.),Some(8.),Some(3.),2));assert!(!g.allow(n+Duration::from_millis(299),Some(16.),Some(1.),Some(1.),2));assert!(g.allow(n+Duration::from_millis(300),Some(16.),Some(1.),Some(1.),2));g.penalize(n);g.reset();assert!(g.allow(n,Some(16.),None,None,2));}
 #[test]fn idle_wait_bounded(){assert_eq!(idle_wait(Duration::ZERO),Duration::from_millis(8));assert_eq!(idle_wait(Duration::from_millis(250)),Duration::from_millis(20));assert_eq!(idle_wait(Duration::from_secs(10)),Duration::from_millis(50));}
}

#[cfg(test)]mod sample_tests{use super::*;use crate::live_gpu_timing::Reading;
 #[test]fn one_gpu_measurement_is_not_counted_as_multiple_samples(){let n=Instant::now();let mut g=FgGate::default();let a=Reading{ms:8.,when:n};let b=Reading{ms:3.,when:n};for i in 0..20{assert!(g.allow_samples(n+Duration::from_millis(i),Some(16.),Some(a),Some(b),2));}let t=n+Duration::from_secs(1);assert!(!g.allow_samples(t,Some(16.),Some(Reading{ms:8.,when:t}),Some(Reading{ms:3.,when:t}),2));assert!(!g.allow_samples(t+Duration::from_millis(100),Some(16.),Some(a),Some(b),2));}
}


#[cfg(test)]mod v95_out_of_order_tests{use super::*;use crate::live_gpu_timing::Reading;#[test]fn old_gpu_sample_does_not_count_as_new_pressure(){let n=Instant::now();let mut g=FgGate::default();let p=Reading{ms:8.,when:n};let q=Reading{ms:3.,when:n};assert!(g.allow_samples(n,Some(16.),Some(p),Some(q),2));let old=Reading{ms:9.,when:n-Duration::from_millis(1)};assert!(g.allow_samples(n+Duration::from_millis(100),Some(16.),Some(old),Some(old),2));assert!(!g.allow_samples(n+Duration::from_millis(101),Some(16.),Some(Reading{ms:9.,when:n+Duration::from_millis(101)}),Some(Reading{ms:3.,when:n+Duration::from_millis(101)}),2));}}
