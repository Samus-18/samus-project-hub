//! Conservative *own-capture* watchdog; no game restart, no driver reset.
use std::time::{Duration,Instant};
#[derive(Default,Debug)]pub struct Guard{target:Option<u32>,attempts:u8,last:Option<Instant>}
impl Guard{
 pub fn reset_target(&mut self,pid:Option<u32>){
  if pid!=self.target{self.target=pid;self.attempts=0;self.last=None;}
 }
 pub fn grant(&mut self,pid:u32,now:Instant)->bool{
  if pid==0{return false;}
  if self.target!=Some(pid){self.reset_target(Some(pid));}
  if self.last.is_some_and(|t|now.checked_duration_since(t).is_some_and(|v|v>=Duration::from_secs(120))){
   self.attempts=0;self.last=None;
  }
  if self.attempts>=2||self.last.is_some_and(|t|now.checked_duration_since(t).is_none_or(|v|v<Duration::from_secs(30))){return false;}
  self.attempts+=1;self.last=Some(now);true
 }
 pub fn used(&self)->u8{self.attempts}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn bounded_restart_backoff_and_source_identity(){
  let n=Instant::now();let mut g=Guard::default();
  assert!(!g.grant(0,n));assert!(g.grant(10,n));assert_eq!(g.used(),1);
  assert!(!g.grant(10,n+Duration::from_secs(5)));
  assert!(g.grant(10,n+Duration::from_secs(31)));assert!(!g.grant(10,n+Duration::from_secs(90)));
  assert!(g.grant(10,n+Duration::from_secs(160)));assert_eq!(g.used(),1);
  g.reset_target(Some(20));assert_eq!(g.used(),0);assert!(g.grant(20,n+Duration::from_secs(161)));
 }
 #[test]fn backwards_clock_cannot_bypass_cooldown(){
  let n=Instant::now();let mut g=Guard::default();assert!(g.grant(5,n));assert!(!g.grant(5,n-Duration::from_secs(1)));
 }
}
