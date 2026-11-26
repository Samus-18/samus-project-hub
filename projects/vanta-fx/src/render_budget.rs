//! Own GPU processing resolution; never touches game settings or HWND size.
pub const PERCENTAGES:[u8;4]=[50,67,75,100];
pub fn valid(v:u8)->u8{if PERCENTAGES.contains(&v){v}else{100}}
pub fn size((w,h):(u32,u32),p:u8)->(u32,u32){
 let p=valid(p) as u64;
 (((w as u64*p+50)/100).max(1) as u32,((h as u64*p+50)/100).max(1) as u32)
}
#[derive(Default)]
pub struct Adaptive{bad:u8,good:u8,last:Option<std::time::Instant>,pub effective:Option<u8>,config:Option<(u8,u32)>}
impl Adaptive{
 pub fn reset(&mut self){*self=Self::default();}
 pub fn observe(&mut self,base:u8,target:u32,measured:Option<f32>,now:std::time::Instant)->Option<u8>{
  let base=valid(base);let target=target.clamp(30,165);let config=(base,target);
  if self.config!=Some(config){
   let changed=self.config.is_some();self.reset();self.config=Some(config);
   if changed{self.last=Some(now);return Some(base);}
  }
  if self.last.is_some_and(|t|now.saturating_duration_since(t)<std::time::Duration::from_secs(5)){return None;}
  self.last=Some(now);
  let Some(fps)=measured.filter(|v|v.is_finite()&&*v>0.0)else{self.bad=0;self.good=0;return None;};
  let current=self.effective.unwrap_or(valid(base));let max=valid(base);
  if fps<target as f32*0.80{self.bad=self.bad.saturating_add(1);self.good=0;}
  else if fps>=target as f32*0.97{self.good=self.good.saturating_add(1);self.bad=0;}
  else{self.bad=0;self.good=0;}
  let next=if self.bad>=2{self.bad=0;PERCENTAGES.iter().rev().copied().find(|v|*v<current).unwrap_or(current)}
  else if self.good>=6{self.good=0;PERCENTAGES.iter().copied().find(|v|*v>current&&*v<=max).unwrap_or(current)}
  else{return None;};
  if next!=current{self.effective=Some(next);Some(next)}else{None}
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn bounded_gpu_working_dimensions(){assert_eq!(size((1920,1080),50),(960,540));assert_eq!(size((1920,1080),75),(1440,810));assert_eq!(size((1,1),50),(1,1));assert_eq!(size((1920,1080),123),(1920,1080));}
 #[test]fn no_fake_data_or_rapid_switching(){
  let n=std::time::Instant::now();let mut a=Adaptive::default();
  assert_eq!(a.observe(100,60,None,n),None);
  assert_eq!(a.observe(100,60,Some(40.),n+std::time::Duration::from_secs(5)),None);
  assert_eq!(a.observe(100,60,Some(40.),n+std::time::Duration::from_secs(10)),Some(75));
  assert_eq!(a.observe(100,60,Some(40.),n+std::time::Duration::from_secs(11)),None);
 }
}

#[cfg(test)]mod change_tests{use super::*;#[test]fn user_quality_change_resets_budget(){let n=std::time::Instant::now();let mut a=Adaptive::default();a.observe(100,60,Some(30.),n);assert_eq!(a.observe(100,60,Some(30.),n+std::time::Duration::from_secs(5)),Some(75));assert_eq!(a.observe(50,60,Some(40.),n+std::time::Duration::from_secs(6)),Some(50));assert!(a.effective.is_none());}}

#[cfg(test)]mod recovery_tests{use super::*;#[test]fn capped_measurements_can_restore_quality(){let n=std::time::Instant::now();let mut a=Adaptive::default();a.observe(100,60,Some(40.),n);assert_eq!(a.observe(100,60,Some(40.),n+std::time::Duration::from_secs(5)),Some(75));for i in [10,15,20,25,30]{assert_eq!(a.observe(100,60,Some(60.),n+std::time::Duration::from_secs(i)),None);}assert_eq!(a.observe(100,60,Some(60.),n+std::time::Duration::from_secs(35)),Some(100));}}
