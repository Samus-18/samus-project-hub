//! Precise but conservative FPS pacing of OUR OWN DXGI preview.
 //! Does not modify the game Present queue or clock. No hooks or injected DLL.
use std::time::{Duration,Instant};

/// 0 means uncapped. Only allow explicit FPS values tested in the GUI.
pub const LIMITS:[u32;10]=[0,30,45,60,75,90,100,120,144,165];
pub fn valid_limit(v:u32)->u32{
    if LIMITS.contains(&v){v}else{45}
}
pub fn frame_budget(fps:u32)->Option<Duration>{
    match valid_limit(fps){
        0=>None,
        fps=>Some(Duration::from_secs_f64(1.0/fps as f64))
    }
}
/// Called only after own processed real frame presents have submitted.
pub fn pace_own_present(start:Instant,limit:u32){
    if let Some(budget)=frame_budget(limit){
        // Don't postpone any further real capture if work already exceeded
        // budget; OS sleep overshoot may be a few milliseconds.
        let remaining=budget.saturating_sub(start.elapsed());
        if !remaining.is_zero(){precise_sleep(remaining);}
    }
}
/// Sleep in bounded slices until the own-window presentation deadline.
 /// Near the deadline we yield instead of requesting an overshooting millisecond
 /// sleep. Never changes OS timer resolution or a game's Present pacing.
pub fn wait_step(remaining:Duration)->bool{
 if remaining.is_zero(){return false;}
 if remaining>Duration::from_micros(650){
  precise_sleep(planned_sleep(remaining));
 }else{
  std::thread::yield_now();
 }
 true
}
pub fn planned_sleep(remaining:Duration)->Duration{
 if remaining>Duration::from_micros(650){
  remaining.min(Duration::from_millis(4)).saturating_sub(Duration::from_micros(300))
 }else{Duration::ZERO}
}
/// Deadline is based on successful presentations, not empty capture polls.
#[derive(Default)]
pub struct PresentDeadline{last:Option<Instant>,last_submit:Option<Instant>}
impl PresentDeadline{
    pub fn remaining(&self,now:Instant,limit:u32)->Duration{
        match (self.last,frame_budget(limit)){
            (Some(last),Some(period))=>now.checked_duration_since(last).map_or(Duration::ZERO,|v|period.saturating_sub(v)),
            _=>Duration::ZERO
        }
    }
    pub fn presented(&mut self,now:Instant){self.last=Some(now);}
    // Preserve start cadence, but account for a slow previous frame rather than allowing a burst.
    pub fn submitted(&mut self,start:Instant,completed:Instant,limit:u32){
        self.last_submit=Some(completed);
        let prior=self.last.filter(|t|*t<=completed);self.last=Some(start.min(completed));
        if let (Some(p),Some(period))=(prior,frame_budget(limit)){if completed.saturating_duration_since(p)>period.saturating_mul(2){self.last=Some(completed);}}
    }
    pub fn reset(&mut self){self.last=None;self.last_submit=None;}
}
/// Measured own capture throughput relative to requested preview cap.
pub fn adaptive_pass_limit(current:usize,observed:f32,available:usize,limit:u32)->usize{
    if available==0{return 0;}
    let current=current.clamp(1,available.min(3));
    if !observed.is_finite()||observed<=0.0{return 1;}
    let target=if valid_limit(limit)==0{60.0}else{valid_limit(limit) as f32};
    if observed<target*0.78{current.saturating_sub(1).max(1)}
    else if observed>=target*0.93{(current+1).min(available.min(3))}
    else{current}
}
#[cfg(test)]
mod adaptive_tests{
    use super::*;
    #[test]fn promotes_at_real_configured_45_cap(){
        assert_eq!(adaptive_pass_limit(1,44.0,4,45),2);
        assert_eq!(adaptive_pass_limit(2,44.0,4,45),3);
        assert_eq!(adaptive_pass_limit(3,30.0,4,45),2);
        assert_eq!(adaptive_pass_limit(2,40.0,4,45),2);
        assert_eq!(adaptive_pass_limit(2,f32::NAN,4,45),1);
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn preview_limits_are_bounded_and_do_not_modify_game(){
        assert_eq!(valid_limit(0),0);
        assert_eq!(valid_limit(45),45);
        assert_eq!(valid_limit(1234),45);
        assert_eq!(frame_budget(0),None);
        assert!(frame_budget(30).unwrap()>frame_budget(60).unwrap());
        assert!(frame_budget(60).unwrap()>frame_budget(120).unwrap());
    }
}

#[cfg(test)]mod deadline_tests{
 use super::*;
 #[test]fn empty_polls_do_not_extend_deadline(){
  let t=Instant::now();let mut p=PresentDeadline::default();
  assert_eq!(p.remaining(t,60),Duration::ZERO);
  p.presented(t);
  assert!(p.remaining(t+Duration::from_millis(5),60)>Duration::ZERO);
  assert_eq!(p.remaining(t+Duration::from_millis(17),60),Duration::ZERO);
  assert_eq!(p.remaining(t,0),Duration::ZERO);
  p.reset();assert_eq!(p.remaining(t,30),Duration::ZERO);
 }
}

#[derive(Default)]pub struct GpuBudget{bad:u8,good:u8,last:Option<Instant>,sample:Option<Instant>}
impl GpuBudget{
 pub fn reset(&mut self){*self=Self::default();}
 pub fn observe_reading(&mut self,now:Instant,current:usize,available:usize,cap:u32,reading:Option<crate::live_gpu_timing::Reading>)->usize{
  if available==0{return 0;}
  let Some(r)=reading else{return self.observe(now,current,available,cap,None);};
  if self.sample==Some(r.when)||self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(3)){return current.clamp(1,available.min(8));}
  self.sample=Some(r.when);self.observe(now,current,available,cap,Some(r.ms))
 }
 pub fn observe(&mut self,now:Instant,current:usize,available:usize,cap:u32,gpu_ms:Option<f64>)->usize{
  if available==0{return 0;}
  let max=available.min(8);let current=current.clamp(1,max);
  if self.last.is_some_and(|t|now.saturating_duration_since(t)<Duration::from_secs(3)){return current;}
  self.last=Some(now);
  let Some(ms)=gpu_ms.filter(|v|v.is_finite()&&*v>0.)else{self.bad=0;self.good=0;return current;};
  let target=if valid_limit(cap)==0{60}else{valid_limit(cap)};
  let budget=1000./target as f64;
  // Reserve 65% of frame interval for the game, capture, composition and scaling.
  if ms>budget*0.35{self.bad=self.bad.saturating_add(1);self.good=0;}
  else if ms<budget*0.18{self.good=self.good.saturating_add(1);self.bad=0;}
  else{self.bad=0;self.good=0;}
  if self.bad>=2{self.bad=0;current.saturating_sub(1).max(1)}
  else if self.good>=3{self.good=0;(current+1).min(max)}else{current}
 }
}
#[cfg(test)]mod gpu_budget_tests{use super::*;#[test]fn no_source_fps_downgrade_and_all_eight_passes(){let n=Instant::now();let mut b=GpuBudget::default();assert_eq!(b.observe(n,8,8,60,None),8);assert_eq!(b.observe(n+Duration::from_secs(3),8,8,60,Some(12.)),8);assert_eq!(b.observe(n+Duration::from_secs(6),8,8,60,Some(12.)),7);for i in 3..5{assert_eq!(b.observe(n+Duration::from_secs(i*3),7,8,60,Some(0.5)),7);}assert_eq!(b.observe(n+Duration::from_secs(15),7,8,60,Some(0.5)),8);b.reset();assert_eq!(b.observe(n,8,8,60,None),8);}}

#[cfg(test)]mod v88_tests{use super::*;#[test]fn long_frame_rebases_deadline_without_catchup_burst(){let n=Instant::now();let mut p=PresentDeadline::default();p.submitted(n,n+Duration::from_millis(2),60);assert!(p.remaining(n+Duration::from_millis(3),60)>Duration::ZERO);let end=n+Duration::from_millis(100);p.submitted(n+Duration::from_millis(17),end,60);assert!(p.remaining(end,60)>Duration::ZERO);p.reset();assert_eq!(p.remaining(end,60),Duration::ZERO);}}

#[cfg(test)]mod v91_refresh_tests{use super::*;#[test]fn high_refresh_budgets_and_limits(){for hz in [75,100,144,165]{assert_eq!(valid_limit(hz),hz);assert!((frame_budget(hz).unwrap().as_secs_f64()-1./hz as f64).abs()<1e-8);}}}

#[cfg(test)]mod v101_pacing_tests{
 use super::*;
 #[test]fn does_not_request_coarse_wait_close_to_deadline(){
  assert_eq!(planned_sleep(Duration::ZERO),Duration::ZERO);
  assert_eq!(planned_sleep(Duration::from_micros(200)),Duration::ZERO);
  assert_eq!(planned_sleep(Duration::from_micros(651)),Duration::from_micros(351));
  assert_eq!(planned_sleep(Duration::from_millis(2)),Duration::from_micros(1700));
  assert_eq!(planned_sleep(Duration::from_millis(25)),Duration::from_micros(3700));
 }
 #[test]fn sleep_policy_has_no_game_present_hooks(){
  let t=Instant::now();let p=PresentDeadline::default();
  assert_eq!(p.remaining(t,144),Duration::ZERO);
  assert!(!wait_step(Duration::ZERO));
 }
}

#[cfg(windows)]
struct PrivateTimer(*mut std::ffi::c_void);
#[cfg(windows)]
#[link(name="kernel32")]
unsafe extern "system"{
 fn CreateWaitableTimerExW(a:*const std::ffi::c_void,name:*const u16,flags:u32,access:u32)->*mut std::ffi::c_void;
 fn SetWaitableTimer(h:*mut std::ffi::c_void,due:*const i64,period:i32,callback:*const std::ffi::c_void,arg:*const std::ffi::c_void,resume:i32)->i32;
 fn WaitForSingleObject(h:*mut std::ffi::c_void,ms:u32)->u32;
 fn CloseHandle(h:*mut std::ffi::c_void)->i32;
}
#[cfg(windows)]
impl Drop for PrivateTimer{fn drop(&mut self){if !self.0.is_null(){unsafe{CloseHandle(self.0);}}}}
fn relative_ticks(d:Duration)->i64{-(d.as_nanos().div_ceil(100).clamp(1,i64::MAX as u128) as i64)}
#[cfg(windows)]
fn precise_sleep(d:Duration){
 if d.is_zero(){return;}
 thread_local!{static TIMER:PrivateTimer=PrivateTimer(unsafe{CreateWaitableTimerExW(std::ptr::null(),std::ptr::null(),2,0x00100002)});}
 let ok=TIMER.with(|t|{if t.0.is_null(){return false;}let due=relative_ticks(d);unsafe{SetWaitableTimer(t.0,&due,0,std::ptr::null(),std::ptr::null(),0)!=0&&WaitForSingleObject(t.0,d.as_millis().saturating_add(20).min(u32::MAX as u128-1)as u32)==0}});
 if !ok{std::thread::sleep(d);}
}
#[cfg(not(windows))]
fn precise_sleep(d:Duration){std::thread::sleep(d);}
#[cfg(test)]mod precision_v118{use super::*;#[test]fn bounded_relative_timer(){assert_eq!(relative_ticks(Duration::from_nanos(1)),-1);assert_eq!(relative_ticks(Duration::from_millis(2)),-20000);let t=Instant::now();precise_sleep(Duration::from_millis(2));assert!(t.elapsed()>=Duration::from_millis(1));}}
