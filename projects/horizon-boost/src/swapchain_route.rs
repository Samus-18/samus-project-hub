//! Bounded observed-presentation routing, not a detector of the game's main HWND.
use std::{collections::BTreeMap,time::{Instant,Duration}};
#[derive(Default)]
pub struct Router{
 start:Option<Instant>,
 counts:BTreeMap<u64,u32>,
 overflow:bool,
 pub selected:Option<u64>,
 challenger:Option<(u64,u8)>,
}
impl Router{
 pub fn observe(&mut self,now:Instant,key:u64)->bool{
  if key==0{return false;}
  if self.start.is_none(){self.start=Some(now);}
  let dt=now.saturating_duration_since(self.start.unwrap());
  if dt>=Duration::from_secs(1){
   // Only contiguous observed windows may elect a new chain.
   if dt<=Duration::from_secs(3)&&!self.overflow{
    let total=self.counts.values().copied().sum::<u32>();
    let winner=self.counts.iter().max_by_key(|(key,count)|(**count,std::cmp::Reverse(**key))).map(|(k,v)|(*k,*v));
    if let Some((winner,n))=winner.filter(|(_,n)|*n>=8&&n.saturating_mul(5)>=total.saturating_mul(3)){
     if let Some(old)=self.selected{
      let old_count=self.counts.get(&old).copied().unwrap_or(0);
      if winner!=old&&(old_count==0||n>=old_count.saturating_mul(2)){
       let streak=if self.challenger.is_some_and(|(k,_)|k==winner){self.challenger.unwrap().1.saturating_add(1)}else{1};
       if streak>=2{self.selected=Some(winner);self.challenger=None;}
       else{self.challenger=Some((winner,streak));}
      }else{self.challenger=None;}
     }else{self.selected=Some(winner);}
    }else{self.challenger=None;}
   }else{self.challenger=None;if dt>Duration::from_secs(3){self.selected=None;}}
   self.start=Some(now);self.counts.clear();self.overflow=false;
  }
  if let Some(n)=self.counts.get_mut(&key){*n=n.saturating_add(1).min(10000);}
  else if self.counts.len()<16{self.counts.insert(key,1);}
  else{self.overflow=true;}
  self.selected==Some(key)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn bucket(r:&mut Router,n:Instant,a:u32,b:u32){
  for i in 0..a{r.observe(n+Duration::from_micros(i as u64),1);}
  for i in 0..b{r.observe(n+Duration::from_micros((a+i) as u64),2);}
 }
 #[test]fn selects_dominant_only_and_hysteresis(){
  let n=Instant::now();let mut r=Router::default();bucket(&mut r,n,60,5);
  let one=n+Duration::from_millis(1001);assert!(r.observe(one,1));assert!(!r.observe(one,2));
  bucket(&mut r,one+Duration::from_millis(1),5,60);
  let two=one+Duration::from_millis(1001);assert!(!r.observe(two,2));assert_eq!(r.selected,Some(1));
  bucket(&mut r,two+Duration::from_millis(1),3,60);
  let three=two+Duration::from_millis(1001);assert!(r.observe(three,2));assert_eq!(r.selected,Some(2));
 }
 #[test]fn ambiguous_streams_not_combined(){
  let n=Instant::now();let mut r=Router::default();bucket(&mut r,n,40,40);
  assert!(!r.observe(n+Duration::from_millis(1001),1));assert!(r.selected.is_none());
 }
 #[test]fn gap_and_overflow_do_not_elect(){
  let n=Instant::now();let mut r=Router::default();bucket(&mut r,n,60,0);
  assert!(!r.observe(n+Duration::from_secs(10),1));
  for key in 2..20{r.observe(n+Duration::from_secs(10),key);}
  for _ in 0..60{r.observe(n+Duration::from_secs(10),1);}
  assert!(!r.observe(n+Duration::from_secs(12),1));assert!(r.selected.is_none());
 }
}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn stale_selection_requires_new_election(){let n=Instant::now();let mut r=Router::default();for _ in 0..60{r.observe(n,1);}assert!(r.observe(n+Duration::from_secs(1),1));assert!(!r.observe(n+Duration::from_secs(9),1));assert_eq!(r.selected,None);for _ in 0..60{r.observe(n+Duration::from_secs(9),2);}assert!(r.observe(n+Duration::from_secs(10),2));}}
