use std::time::{Instant,Duration};
#[derive(Default)]pub struct Trainer{pub active:bool,pub hits:u32,pub misses:u32,pub target:[f32;2],pub times:Vec<u32>,start:Option<Instant>,last:Option<Instant>,seed:u32,pub duration_secs:u64,pub radius:f32,stopped_at:Option<Instant>}
impl Trainer{
 pub fn start(&mut self,now:Instant){let duration_secs=if self.duration_secs==0{30}else{self.duration_secs.clamp(10,60)};let radius=if self.radius.is_finite()&&self.radius>=8.{self.radius.clamp(8.,24.)}else{17.};*self=Self{active:true,target:[0.5,0.5],start:Some(now),last:Some(now),seed:seed(now),duration_secs,radius,..Default::default()};}
 pub fn stop(&mut self){self.stop_at(Instant::now());}
 fn stop_at(&mut self,now:Instant){if self.active{self.stopped_at=Some(now);self.active=false;}}
 pub fn tick(&mut self,now:Instant){if self.active&&let Some(t)=self.start{let end=t+Duration::from_secs(self.duration_secs);if now>=end{self.stop_at(end);}}}
 pub fn remaining(&self,now:Instant)->u64{let limit=if self.duration_secs==0{30}else{self.duration_secs};self.start.map(|t|limit.saturating_sub(self.stopped_at.unwrap_or(now).saturating_duration_since(t).as_secs())).unwrap_or(limit)}
 pub fn click(&mut self,now:Instant,hit:bool){
  self.tick(now);if !self.active{return;}
  if !hit{self.misses=self.misses.saturating_add(1);return;}
  self.hits+=1;if let Some(t)=self.last{self.times.push(now.saturating_duration_since(t).as_millis().min(30000) as u32);}
  self.last=Some(now);
  let old=self.target;let mut next=old;
  for _ in 0..8{
   self.seed=self.seed.wrapping_mul(1664525).wrapping_add(1013904223);let x=(self.seed>>8)%801;
   self.seed=self.seed.wrapping_mul(1664525).wrapping_add(1013904223);let y=(self.seed>>8)%601;
   next=[0.1+x as f32/1000.,0.2+y as f32/1000.];
   if distance(old,next)>=0.18{break;}
  }
  if distance(old,next)<0.18{next=[if old[0]>0.5{0.15}else{0.85},if old[1]>0.5{0.25}else{0.75}];}
  self.target=next;if self.hits>=100{self.stop_at(now);}
 }
 pub fn median_ms(&self)->Option<f32>{let mut v=self.times.clone();v.sort_unstable();let n=v.len();if n==0{None}else if n%2==1{Some(v[n/2] as f32)}else{Some((v[n/2-1] as f32+v[n/2] as f32)/2.)}}
 pub fn p90_ms(&self)->Option<u32>{if self.times.len()<5{return None;}let mut v=self.times.clone();v.sort_unstable();Some(v[(v.len()*90).div_ceil(100)-1])}
 pub fn hits_per_minute(&self,now:Instant)->Option<f64>{let elapsed=self.stopped_at.unwrap_or(now).saturating_duration_since(self.start?).as_secs_f64();(elapsed>=1.).then(||self.hits as f64*60./elapsed)}
 pub fn accuracy(&self)->Option<f32>{let n=self.hits.saturating_add(self.misses);(n>0).then(||self.hits as f32*100./n as f32)}
}
#[cfg(test)]mod tests{
 #[test]fn manual_only_and_duration(){let n=std::time::Instant::now();let mut t=super::Trainer::default();t.click(n,true);assert_eq!(t.hits,0);t.start(n);t.click(n,false);t.click(n+std::time::Duration::from_millis(220),true);assert_eq!(t.accuracy(),Some(50.));assert_eq!(t.times,[220]);t.tick(n+std::time::Duration::from_secs(30));assert!(!t.active);}
}

#[cfg(test)]mod v49_tests{use super::*;#[test]fn configurable_manual_session_and_median(){let n=Instant::now();let mut t=Trainer{duration_secs:10,radius:999.,..Default::default()};t.start(n);assert_eq!(t.radius,24.);t.click(n+Duration::from_millis(100),true);t.click(n+Duration::from_millis(400),true);assert_eq!(t.median_ms(),Some(200.));t.stop_at(n+Duration::from_secs(2));assert_eq!(t.remaining(n+Duration::from_secs(30)),8);assert!(!t.active);t.start(n);t.tick(n+Duration::from_secs(10));assert_eq!(t.remaining(n+Duration::from_secs(11)),0);}}

fn seed(now:Instant)->u32{
 static START:std::sync::OnceLock<Instant>=std::sync::OnceLock::new();
 static RUN:std::sync::atomic::AtomicU32=std::sync::atomic::AtomicU32::new(0);
 let elapsed=now.saturating_duration_since(*START.get_or_init(||now)).as_nanos() as u32;
 elapsed^RUN.fetch_add(1,std::sync::atomic::Ordering::Relaxed).wrapping_mul(0x9e3779b9)^0x8137a9c3
}
pub fn target_position(target:[f32;2],size:[f32;2],radius:f32)->[f32;2]{
 std::array::from_fn(|i|{let s=if size[i].is_finite(){size[i].max(0.)}else{0.};let r=if radius.is_finite(){radius.max(0.)}else{0.};let t=if target[i].is_finite(){target[i].clamp(0.,1.)}else{0.5};let margin=r.min(s*0.5);margin+(s-2.*margin)*t})
}
#[cfg(test)]mod safe_area_tests{
 #[test]fn targets_fit_and_sessions_vary(){assert_eq!(super::target_position([0.,1.],[100.,100.],20.),[20.,80.]);assert_eq!(super::target_position([1.,0.],[20.,10.],20.),[10.,5.]);let n=std::time::Instant::now();assert_ne!(super::seed(n),super::seed(n));}
}

fn distance(a:[f32;2],b:[f32;2])->f32{((a[0]-b[0]).powi(2)+(a[1]-b[1]).powi(2)).sqrt()}
#[cfg(test)]mod spacing_tests{use super::*;#[test]fn practice_targets_have_spacing_and_speed_stops(){let n=Instant::now();let mut t=Trainer::default();t.start(n);for i in 1..=10{let old=t.target;t.click(n+Duration::from_millis(i*200),true);assert!(distance(old,t.target)>=0.18);}t.stop_at(n+Duration::from_secs(2));assert_eq!(t.p90_ms(),Some(200));assert_eq!(t.hits_per_minute(n+Duration::from_secs(90)),Some(300.));}}

#[cfg(test)]mod deadline_tests{use super::*;#[test]fn delayed_tick_does_not_reduce_throughput(){let n=Instant::now();let mut t=Trainer{duration_secs:10,..Default::default()};t.start(n);t.click(n+Duration::from_secs(1),true);t.tick(n+Duration::from_secs(100));assert_eq!(t.hits_per_minute(n+Duration::from_secs(100)),Some(6.));assert_eq!(target_position([f32::NAN,1.],[100.,f32::NAN],20.),[50.,0.]);}}
