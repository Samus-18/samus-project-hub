//! Opt-in read-only scanout cadence measurement via DXGI WaitForVBlank.
//! This is physical output blanking time, not game FPS and not proof of VRR.
use std::{sync::{Arc,Mutex,OnceLock},time::Instant,thread};
use windows::{core::Interface,Win32::Graphics::Dxgi::{CreateDXGIFactory1,IDXGIFactory1,IDXGIFactory2,IDXGIOutput6}};
#[derive(Debug,Clone,Copy)]
pub struct ResultData{pub measured_hz:f64,pub median_ms:f64,pub jitter_mad_ms:f64,pub tearing_allowed:Option<bool>}
pub fn summarize_ms(mut intervals:Vec<f64>,tearing:Option<bool>)->Result<ResultData,String>{
 intervals.retain(|n|n.is_finite()&&*n>=3.0&&*n<=100.0);
 if intervals.len()<8{return Err("Insufficient valid output blanking intervals".into());}
 intervals.sort_by(f64::total_cmp);
 let middle=intervals[intervals.len()/2];
 let mut errors=intervals.iter().map(|n|(n-middle).abs()).collect::<Vec<_>>();
 errors.sort_by(f64::total_cmp);
 Ok(ResultData{measured_hz:1000./middle,median_ms:middle,jitter_mad_ms:errors[errors.len()/2],tearing_allowed:tearing})
}
pub fn measure(requested:Option<&str>)->Result<ResultData,String>{
 let displays=crate::display_probe::read();
 let target=crate::display_probe::select(&displays,requested).ok_or("No attached Windows monitor to measure")?;
 let factory:IDXGIFactory1=unsafe{CreateDXGIFactory1()}.map_err(|e|format!("DXGI factory: {e}"))?;
 let tearing=factory.cast::<IDXGIFactory2>().ok().and_then(|v|crate::adapter_select::tearing(&v));
 for ai in 0..16{
  let Ok(adapter)=(unsafe{factory.EnumAdapters1(ai)})else{break;};
  for di in 0..16{
   let Ok(output)=(unsafe{adapter.EnumOutputs(di)})else{break;};
   let Ok(desc)=output.cast::<IDXGIOutput6>().and_then(|v|unsafe{v.GetDesc1()})else{continue;};
   let name=String::from_utf16_lossy(&desc.DeviceName).trim_end_matches('\0').to_owned();
   if !name.eq_ignore_ascii_case(&target.name){continue;}
   let mut intervals=Vec::with_capacity(24);
   let mut previous=Instant::now();
   for step in 0..27{
    unsafe{output.WaitForVBlank()}.map_err(|e|format!("Output VBlank measurement failed: {e}"))?;
    let now=Instant::now();
    if step>=3{intervals.push(now.duration_since(previous).as_secs_f64()*1000.);}
    previous=now;
   }
   return summarize_ms(intervals,tearing);
  }
 }
 Err("Selected output could not provide DXGI VBlank timing".into())
}
#[derive(Default)]struct Snapshot{busy:bool,result:Option<String>}
static STATUS:OnceLock<Arc<Mutex<Snapshot>>>=OnceLock::new();
fn state()->&'static Arc<Mutex<Snapshot>>{STATUS.get_or_init(||Arc::new(Mutex::new(Snapshot::default())))}
pub fn start(requested:Option<String>)->Result<(),String>{
 let cell=Arc::clone(state());
 {
  let mut s=cell.lock().map_err(|_|"Refresh measurement state unavailable")?;
  if s.busy{return Err("Output refresh measurement already active".into());}
  s.busy=true;s.result=None;
 }
 let ready=Arc::clone(&cell);
 let spawned=thread::Builder::new().name("Horizon DXGI VBlank read-only".into()).spawn(move||{
  let result=measure(requested.as_deref());
  let message=match result{
   Ok(v)=>format!("Measured output VBlank {:.2} Hz (median {:.3}ms, jitter MAD {:.3}ms). DXGI tearing support: {:?}. Active FreeSync/VRR NOT established; game frame rate unchanged.",v.measured_hz,v.median_ms,v.jitter_mad_ms,v.tearing_allowed),
   Err(e)=>format!("DXGI VBlank measurement unavailable: {e}"),
  };
  if let Ok(mut s)=ready.lock(){s.result=Some(message);s.busy=false;}
 });
 if let Err(e)=spawned{
  if let Ok(mut s)=cell.lock(){s.busy=false;s.result=Some(format!("Worker launch failed: {e}"));}
  return Err(format!("Cannot launch output refresh probe: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|s|(s.busy,s.result.clone())).unwrap_or((false,None))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn measured_vblank_median_filters_invalid_intervals_and_avoids_false_vrr_claim(){
  let a=summarize_ms(vec![16.7,16.6,16.5,16.7,16.6,16.5,16.7,16.7,9999.,f64::NAN,16.6],Some(true)).unwrap();
  assert!((a.measured_hz-60.24).abs()<2.);
  assert!(a.jitter_mad_ms<0.15);
  assert_eq!(a.tearing_allowed,Some(true));
  assert!(summarize_ms(vec![0.,-1.,16.],None).is_err());
 }
 #[test]fn not_polled_or_started_by_default(){let (busy,_) = snapshot();assert!(!busy);}
}
