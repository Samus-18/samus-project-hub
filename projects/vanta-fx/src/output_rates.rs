use std::time::{Duration,Instant};
#[derive(Clone,Copy,Debug)]pub struct Rates{pub real:f64,pub generated:f64,pub total:f64}
#[derive(Default)]pub struct Meter{base:Option<(Instant,u64,u64)>,last:Option<(Instant,Rates)>}
impl Meter{
 pub fn clear(&mut self){*self=Self::default();}
 pub fn observe(&mut self,now:Instant,counters:Option<(u64,u64)>){
  let Some((r,g))=counters else{self.clear();return;};
  let Some((when,old_r,old_g))=self.base else{self.base=Some((now,r,g));return;};
  if r<old_r||g<old_g{self.clear();self.base=Some((now,r,g));return;}
  if now<when{self.clear();self.base=Some((now,r,g));return;}
  let dt=now.saturating_duration_since(when);
  if dt<Duration::from_secs(1){return;}
  self.base=Some((now,r,g));
  if dt>Duration::from_secs(5){self.last=None;return;}
  let real=(r-old_r) as f64/dt.as_secs_f64();let generated=(g-old_g) as f64/dt.as_secs_f64();
  self.last=if real<=2000.&&generated<=2000.{Some((now,Rates{real,generated,total:real+generated}))}else{None};
 }
 pub fn snapshot(&self,now:Instant)->Option<Rates>{
  self.last.filter(|(t,_)|now>=*t&&now.saturating_duration_since(*t)<=Duration::from_secs(2)).map(|(_,r)|r)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn separates_and_resets_rates(){
  let mut m=Meter::default();let n=Instant::now();m.observe(n,Some((10,5)));assert!(m.snapshot(n).is_none());
  let one=n+Duration::from_secs(1);m.observe(one,Some((40,65)));
  let v=m.snapshot(one).unwrap();assert_eq!((v.real,v.generated,v.total),(30.,60.,90.));
  m.observe(one+Duration::from_secs(1),Some((0,0)));assert!(m.snapshot(one).is_none());
  m.observe(one+Duration::from_secs(2),None);assert!(m.snapshot(one).is_none());
 }
}

#[cfg(test)]mod monotonic_tests{use super::*;#[test]fn timestamp_reversal_discards_old_rate(){let t=Instant::now();let mut m=Meter::default();m.observe(t,Some((0,0)));m.observe(t+Duration::from_secs(1),Some((50,50)));assert!(m.snapshot(t).is_none());m.observe(t,Some((60,60)));assert!(m.snapshot(t+Duration::from_secs(1)).is_none());}}


/// Real own-window feedback must not be polluted with generated FG frames
/// or source-game FPS. A source-bound game must not trigger endlessly
/// lowering own GPU quality when Horizon is already keeping up.
pub fn processing_sample(rates:Option<Rates>,game_fps:Option<f32>,user_target:u32,own_cap:u32)->(u32,Option<f32>){
 let target=user_target.clamp(30,165).min(if own_cap==0{165}else{crate::preview_pacer::valid_limit(own_cap).max(30)});
 let Some(rate)=rates.map(|r|r.real).filter(|v|v.is_finite()&&*v>0.&&*v<=2000.)else{return(target,None);};
 let source=game_fps.filter(|v|v.is_finite()&&*v>0.&&*v<=2000.);
 // If source is substantially slower than target and own output follows
 // the source, no amount of upscaling reduction can create more real frames.
 if source.is_some_and(|source|source<target as f32*0.9&&rate>=source as f64*0.80){
  return (target,None);
 }
 (target,Some(rate as f32))
}
#[cfg(test)]mod v102_source_vs_output_tests{
 use super::*;
 fn rates(real:f64,generated:f64)->Option<Rates>{Some(Rates{real,generated,total:real+generated})}
 #[test]fn real_own_output_instead_of_hero_game_fps(){
  let(t,actual)=processing_sample(rates(44.,60.),Some(144.),60,45);
  assert_eq!(t,45);assert_eq!(actual,Some(44.));
 }
 #[test]fn source_bottleneck_cannot_reduce_quality_when_output_tracks_game(){
  assert_eq!(processing_sample(rates(29.,0.),Some(30.),60,60),(60,None));
  assert_eq!(processing_sample(rates(20.,0.),Some(30.),60,60),(60,Some(20.)));
 }
 #[test]fn missing_real_frames_and_unlimited_cap_safe(){
  assert_eq!(processing_sample(None,Some(144.),144,0),(144,None));
  assert_eq!(processing_sample(rates(f64::NAN,0.),None,60,60),(60,None));
  assert_eq!(processing_sample(rates(80.,1000.),None,120,0),(120,Some(80.)));
 }
}

