//! Coalesce slider changes; normal shutdown still writes the final settings.
use std::time::{Duration,Instant};
#[derive(Default)]pub struct Debounce{first:Option<Instant>,last:Option<Instant>,retry_at:Option<Instant>}
impl Debounce{
 pub fn mark(&mut self,now:Instant){self.first.get_or_insert(now);self.last=Some(now);}
 pub fn due(&self,now:Instant)->bool{
  if self.retry_at.is_some_and(|t|now<t){return false;}
  self.first.is_some_and(|t|now.saturating_duration_since(t)>=Duration::from_secs(1))||
  self.last.is_some_and(|t|now.saturating_duration_since(t)>=Duration::from_millis(250))
 }
 pub fn clear(&mut self){*self=Self::default();}
 pub fn retry(&mut self,now:Instant){self.retry_at=Some(now+Duration::from_secs(3));}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn quiet_period_and_maximum_delay(){let n=Instant::now();let mut d=Debounce::default();assert!(!d.due(n));d.mark(n);assert!(!d.due(n+Duration::from_millis(249)));assert!(d.due(n+Duration::from_millis(250)));for i in 1..10{d.mark(n+Duration::from_millis(i*100));}assert!(d.due(n+Duration::from_secs(1)));d.clear();assert!(!d.due(n+Duration::from_secs(5)));}
 #[test]fn failed_write_retries_without_looping(){let n=Instant::now();let mut d=Debounce::default();d.mark(n);d.retry(n);assert!(!d.due(n+Duration::from_secs(2)));assert!(d.due(n+Duration::from_secs(3)));}
}
