use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::time::Duration;
#[derive(Default)]
pub struct Health{
 pub live_stages:std::sync::RwLock<Option<std::sync::Arc<[usize]>>>,
 pub live_scale_mode:AtomicU64,
 pub effective_scale_mode:AtomicU64,pub scale_fallbacks:AtomicU64,pub scale_runtime_fallback:AtomicBool,
 pub scale_input:AtomicU64,pub scale_output:AtomicU64,
 pub live_quality:AtomicU64,
 pub live_look_rejected:AtomicBool,
 pub capture_interval_active:AtomicBool,
 pub duplicate_frames:AtomicU64,
 pub stale_pre_filter:AtomicU64,pub fg_pressure_skips:AtomicU64,pub fg_reason:AtomicU64,pub own_minimized:AtomicBool,
 pub composition_requested:bool,pub composition_active:AtomicBool,
 pub hdr_requested:bool,pub hdr_active:AtomicBool,
 pub hdr_white_nits:f32,
 pub fg_backlog:AtomicU64,pub nominal_refresh:AtomicU64,
 pub fused_saved:AtomicU64,pub fence_active:AtomicBool,
 pub vsync:AtomicBool,
 pub waitable_requested:AtomicBool,
 pub tearing_requested:bool,pub tearing_supported:AtomicU64,pub tearing_active:AtomicBool,pub renderer_adapter:Option<String>,
 pub waitable_active:AtomicBool,
 pub wait_timeouts:AtomicU64,
 pub dequeued:AtomicU64,
 pub no_frame_polls:AtomicU64,
 pub deferred_fg:AtomicU64,
 pub failed_fg_present:AtomicU64,
 pub fg_effective:AtomicU64,pub fg_stale:AtomicU64,pub fg_reduced:AtomicU64,pub fg_gpu_errors:AtomicU64,
 pub paused:AtomicBool,
 pub sample_code:AtomicU64,
 pub sample_repeats:AtomicU64,
 pub optional_warning:AtomicU64,
 pub non_submitted:AtomicU64,
 pub effective_passes:AtomicU64,
 present_age_us:AtomicU64,
 worker_beat:AtomicU64,
 pub preview_display:Option<String>,
 pub display_grades:crate::display_color::Profiles,
 capture_us:AtomicU64,
 submit_us:AtomicU64,
 age_us:AtomicU64,
 phase_us:[AtomicU64;5],
}
#[derive(Clone,Copy,Debug)]
pub struct Snapshot{pub live_look_rejected:bool,pub effective_scale_mode:Option<u8>,pub scale_fallbacks:u64,pub scale_runtime_fallback:bool,pub scale_input:Option<(u32,u32)>,pub scale_output:Option<(u32,u32)>,pub stale_pre_filter:u64,pub fg_pressure_skips:u64,pub fg_reason:u64,pub own_minimized:bool,pub capture_interval_active:bool,pub duplicate_frames:u64,pub dequeued:u64,pub empty:u64,pub deferred:u64,pub failed_fg:u64,
 pub paused:bool,pub sample_code:u64,pub sample_repeats:u64,pub capture_ms:Option<f64>,pub submit_ms:Option<f64>,pub capture_age_ms:Option<f64>,pub gpu_phase_ms:[Option<f64>;5],pub waitable_active:bool,pub wait_timeouts:u64,pub tearing_supported:Option<bool>,pub tearing_active:bool,pub optional_warning:u32,pub non_submitted:u64,pub effective_passes:u64,pub present_age_ms:Option<f64>,pub worker_age_ms:Option<f64>,pub fg_effective:u64,pub fg_stale:u64,pub fg_reduced:u64,pub fg_gpu_errors:u64}
fn micros(d:Duration)->u64{(d.as_micros().min(u64::MAX as u128) as u64).saturating_add(1)}
impl Health{
 pub fn new()->Self{let s=Self::default();s.vsync.store(true,Ordering::Relaxed);s}
 pub fn beat(&self){if let Some(n)=crate::capture_clock::qpc_100ns(){self.worker_beat.store(n as u64,Ordering::Release);}}
 pub fn heartbeat_age_ms(&self)->Option<f64>{let stamp=self.worker_beat.load(Ordering::Acquire);let now=crate::capture_clock::qpc_100ns()? as u64;(stamp>0&&now>=stamp).then(||(now-stamp) as f64/10000.)}
 pub fn frame(&self,capture:Duration,submit:Duration){
  self.capture_us.store(micros(capture),Ordering::Relaxed);
  self.submit_us.store(micros(submit),Ordering::Relaxed);
 }
 pub fn phases(&self,age:Option<f64>,gpu:[Option<f64>;5]){
  let encode=|v:Option<f64>|v.filter(|x|x.is_finite()&&*x>=0.&&*x<=10000.).map_or(0,|v|(v*1000.).round() as u64+1);
  self.age_us.store(encode(age),Ordering::Relaxed);for(i,v)in gpu.iter().enumerate(){self.phase_us[i].store(encode(*v),Ordering::Relaxed);}
 }
 pub fn present_age(&self,age:Option<f64>){self.present_age_us.store(age.filter(|v|v.is_finite()&&*v>=0.&&*v<=10000.).map_or(0,|v|(v*1000.).round() as u64+1),Ordering::Relaxed);}
 pub fn clear_timings(&self){self.capture_us.store(0,Ordering::Relaxed);self.submit_us.store(0,Ordering::Relaxed);self.present_age(None);self.phases(None,[None;5]);}
 pub fn snapshot(&self)->Snapshot{
  let paused=self.paused.load(Ordering::Relaxed);
  let ms=|v:&AtomicU64|if paused{None}else{match v.load(Ordering::Relaxed){0=>None,n=>Some((n-1) as f64/1000.)}};
  let dims=|v:&AtomicU64|{let n=v.load(Ordering::Relaxed);let w=(n>>32)as u32;let h=n as u32;(w>0&&h>0).then_some((w,h))};
  Snapshot{live_look_rejected:self.live_look_rejected.load(Ordering::Relaxed),effective_scale_mode:self.effective_scale_mode.load(Ordering::Relaxed).checked_sub(1).map(|v|v as u8),scale_fallbacks:self.scale_fallbacks.load(Ordering::Relaxed),scale_runtime_fallback:self.scale_runtime_fallback.load(Ordering::Relaxed),scale_input:dims(&self.scale_input),scale_output:dims(&self.scale_output),stale_pre_filter:self.stale_pre_filter.load(Ordering::Relaxed),fg_pressure_skips:self.fg_pressure_skips.load(Ordering::Relaxed),fg_reason:self.fg_reason.load(Ordering::Relaxed),own_minimized:self.own_minimized.load(Ordering::Relaxed),capture_interval_active:self.capture_interval_active.load(Ordering::Relaxed),duplicate_frames:self.duplicate_frames.load(Ordering::Relaxed),fg_effective:if paused{0}else{self.fg_effective.load(Ordering::Relaxed)},fg_stale:self.fg_stale.load(Ordering::Relaxed),fg_reduced:self.fg_reduced.load(Ordering::Relaxed),fg_gpu_errors:self.fg_gpu_errors.load(Ordering::Relaxed),worker_age_ms:self.heartbeat_age_ms(),non_submitted:self.non_submitted.load(Ordering::Relaxed),effective_passes:self.effective_passes.load(Ordering::Relaxed),present_age_ms:ms(&self.present_age_us),optional_warning:self.optional_warning.load(Ordering::Relaxed) as u32,tearing_supported:match self.tearing_supported.load(Ordering::Relaxed){1=>Some(false),2=>Some(true),_=>None},tearing_active:self.tearing_active.load(Ordering::Relaxed),dequeued:self.dequeued.load(Ordering::Relaxed),empty:self.no_frame_polls.load(Ordering::Relaxed),
   deferred:self.deferred_fg.load(Ordering::Relaxed),failed_fg:self.failed_fg_present.load(Ordering::Relaxed),
   paused,sample_code:self.sample_code.load(Ordering::Relaxed),sample_repeats:self.sample_repeats.load(Ordering::Relaxed),capture_ms:ms(&self.capture_us),submit_ms:ms(&self.submit_us),capture_age_ms:ms(&self.age_us),gpu_phase_ms:std::array::from_fn(|i|ms(&self.phase_us[i])),
   waitable_active:self.waitable_active.load(Ordering::Relaxed),wait_timeouts:self.wait_timeouts.load(Ordering::Relaxed)}
 }
}
pub fn recoverable(error:&str)->bool{
 let lower=error.to_ascii_lowercase();
 lower.starts_with("wgc_rebuild_required:")||lower.starts_with("wgc_device_lost:")||lower.starts_with("dxgi_desktop_duplication_lost:")||
 lower.contains("0x887a0005")||lower.contains("0x887a0006")||lower.contains("0x887a0007")||
 lower.contains("dxgi_error_device_removed")||lower.contains("dxgi_error_device_reset")||lower.starts_with("wgc supplied no new frames for 10s")
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn counters_never_invent_timing(){
  let h=Health::new();assert!(h.snapshot().capture_ms.is_none());assert!(h.vsync.load(Ordering::Relaxed));
  h.frame(Duration::from_micros(1700),Duration::from_micros(900));
  assert_eq!(h.snapshot().capture_ms,Some(1.7));assert_eq!(h.snapshot().submit_ms,Some(0.9));
 }
 #[test]fn only_recoverable_errors_rebuild(){
  assert!(recoverable("WGC_REBUILD_REQUIRED: 10 -> 20"));
  assert!(recoverable("WGC_DEVICE_LOST: driver reset"));
  assert!(recoverable("DXGI error (0x887A0005)"));
  assert!(!recoverable("WGC refused protected target"));assert!(!recoverable("Target game window closed"));
 }
}

#[cfg(test)]mod phase_tests{use super::*;#[test]fn phase_values_missing_and_bounded(){let h=Health::new();assert_eq!(h.snapshot().gpu_phase_ms,[None;5]);h.phases(Some(4.25),[Some(0.1),None,Some(f64::NAN),Some(1.5),None]);assert_eq!(h.snapshot().capture_age_ms,Some(4.25));assert_eq!(h.snapshot().gpu_phase_ms,[Some(0.1),None,None,Some(1.5),None]);h.phases(None,[None;5]);assert!(h.snapshot().capture_age_ms.is_none());}}

#[cfg(test)]mod freshness_tests{use super::*;#[test]fn paused_or_stale_clears_all_time_fields(){let h=Health::new();h.frame(Duration::from_millis(3),Duration::from_millis(2));h.phases(Some(4.),[Some(1.);5]);h.present_age(Some(9.));assert_eq!(h.snapshot().present_age_ms,Some(9.));h.clear_timings();let s=h.snapshot();assert!(s.present_age_ms.is_none()&&s.capture_ms.is_none()&&s.submit_ms.is_none()&&s.capture_age_ms.is_none());assert_eq!(s.gpu_phase_ms,[None;5]);}}

#[cfg(test)]mod paused_tests{use super::*;#[test]fn paused_hides_even_preexisting_timestamps(){let h=Health::new();h.frame(Duration::from_millis(2),Duration::from_millis(3));h.present_age(Some(8.));h.paused.store(true,Ordering::Relaxed);let s=h.snapshot();assert!(s.capture_ms.is_none()&&s.present_age_ms.is_none()&&s.submit_ms.is_none());}}

#[cfg(test)]mod heartbeat_tests{use super::*;#[test]fn measured_heartbeat_has_no_fake_initial_value(){let h=Health::new();assert!(h.heartbeat_age_ms().is_none());h.beat();assert!(h.snapshot().worker_age_ms.is_some_and(|v|v>=0.&&v<1000.));}}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum WorkerState{Stopped,Starting,Active,Paused,Minimized,Unresponsive}
impl WorkerState{pub fn label(self)->&'static str{match self{
 Self::Stopped=>"Stopped",Self::Starting=>"Starting / waiting for first frame",
 Self::Active=>"Frame processor active",Self::Paused=>"Source paused",
 Self::Minimized=>"Output minimized",Self::Unresponsive=>"Worker heartbeat too old"
}}}
pub fn classify(running:bool,submitted:u64,beat_ms:Option<f64>,paused:bool,minimized:bool)->WorkerState{
 if !running{return WorkerState::Stopped;}
 if minimized{return WorkerState::Minimized;}
 if paused{return WorkerState::Paused;}
 if beat_ms.is_some_and(|n|!n.is_finite()||n<0.||n>=3500.){return WorkerState::Unresponsive;}
 if submitted==0{return WorkerState::Starting;}
 WorkerState::Active
}
#[cfg(test)]mod classified_state_tests{
 use super::*;
 #[test]fn correct_priority_and_no_fabricated_stall(){
  assert_eq!(classify(false,10,Some(5000.),false,false),WorkerState::Stopped);
  assert_eq!(classify(true,100,Some(5000.),true,false),WorkerState::Paused);
  assert_eq!(classify(true,100,Some(5000.),false,true),WorkerState::Minimized);
  assert_eq!(classify(true,0,None,false,false),WorkerState::Starting);
  assert_eq!(classify(true,0,Some(4000.),false,false),WorkerState::Unresponsive);
  assert_eq!(classify(true,5,Some(20.),false,false),WorkerState::Active);
  assert_eq!(classify(true,5,None,false,false),WorkerState::Active);
 }
}
