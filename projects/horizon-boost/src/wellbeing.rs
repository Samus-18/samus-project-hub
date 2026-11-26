use std::time::{Instant,Duration};
#[derive(Default)]pub struct BreakTimer{end:Option<Instant>}
impl BreakTimer{
 pub fn start(&mut self,now:Instant,min:u32){self.end=Some(now+Duration::from_secs(min.clamp(5,180) as u64*60));}
 pub fn stop(&mut self){self.end=None;}
 pub fn remaining(&self,now:Instant)->Option<u64>{self.end.map(|t|t.saturating_duration_since(now).as_secs())}
 pub fn poll(&mut self,now:Instant)->bool{if self.end.is_some_and(|t|t<=now){self.stop();true}else{false}}
}
#[derive(Default)]pub struct Alerts{last:Option<Instant>}
impl Alerts{
 pub fn poll(&mut self,now:Instant,enabled:bool,vram:Option<f32>,frame:Option<f32>,threshold:f32)->Option<&'static str>{
  self.poll_all(now,enabled,vram,frame,threshold,None)
 }
 pub fn poll_all(&mut self,now:Instant,enabled:bool,vram:Option<f32>,frame:Option<f32>,threshold:f32,temp:Option<f32>)->Option<&'static str>{
  if !enabled||self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(30)){return None;}
  let message=if temp.is_some_and(|v|v.is_finite()&&v>=85.){Some("AMD measured GPU edge temperature >=85 C")}
  else if vram.is_some_and(|v|v.is_finite()&&v>=0.90){Some("Horizon own VRAM budget above 90%")}
  else if frame.is_some_and(|v|v.is_finite()&&v>=threshold.clamp(20.,200.)){Some("Measured game average frametime above threshold")}else{None};
  if message.is_some(){self.last=Some(now);}message
 }
}
#[cfg(test)]mod tests{
 #[test]fn break_fires_once_and_alerts_throttle(){
  use super::*;let n=Instant::now();let mut b=BreakTimer::default();b.start(n,5);assert!(!b.poll(n));assert!(b.poll(n+Duration::from_secs(300)));assert!(!b.poll(n+Duration::from_secs(301)));
  let mut a=Alerts::default();assert!(a.poll(n,true,None,None,50.).is_none());assert!(a.poll(n,true,Some(0.95_f32),None,50.).is_some());assert!(a.poll(n,true,Some(1.),None,50.).is_none());
 }
}
