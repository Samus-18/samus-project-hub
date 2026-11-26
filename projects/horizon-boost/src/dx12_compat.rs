//! A real offline D3D12 hardware queue plus D3D11On12 GPU compute proof.
//! This is NOT yet the application runtime Vulkan/D3D12 backend switch.
//! Uses one Direct3D12 DIRECT queue; its D3D11On12 compatibility layer runs
//! existing source shader code on that D3D12 queue. No games or swapchains.
use std::{ffi::c_void,sync::{Arc,Mutex,OnceLock},thread};
use windows::{
 core::Interface,
 Win32::Graphics::{
  Direct3D::D3D_FEATURE_LEVEL_11_0,
  Direct3D11::{D3D11_CREATE_DEVICE_BGRA_SUPPORT,ID3D11Device,ID3D11DeviceContext},
  Direct3D11on12::D3D11On12CreateDevice,
  Direct3D12::{
   D3D12CreateDevice,D3D12_COMMAND_QUEUE_DESC,D3D12_COMMAND_LIST_TYPE_DIRECT,
   D3D12_COMMAND_QUEUE_PRIORITY_NORMAL,D3D12_COMMAND_QUEUE_FLAG_NONE,
   D3D12_FEATURE_DATA_D3D12_OPTIONS1,D3D12_FEATURE_D3D12_OPTIONS1,
   ID3D12Device,ID3D12CommandQueue,
  },
 },
};
#[derive(Debug,Clone)]
pub struct Proof{
 pub gpu_compute_on_d3d12_queue:bool,
 pub wave_ops_advertised:Option<bool>,
 pub wave_lane_min:Option<u32>,pub wave_lane_max:Option<u32>,
 pub output_pixels_changed:usize,
}
pub fn synthetic_d3d12_compatibility_proof()->Result<Proof,String>{
 let mut device:Option<ID3D12Device>=None;
 unsafe{D3D12CreateDevice(None::<&windows::core::IUnknown>,D3D_FEATURE_LEVEL_11_0,&mut device)}
  .map_err(|e|format!("Native hardware D3D12 unavailable: {e}"))?;
 let dev=device.ok_or("No D3D12 hardware device")?;
 let queue_desc=D3D12_COMMAND_QUEUE_DESC{
  Type:D3D12_COMMAND_LIST_TYPE_DIRECT,
  Priority:D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
  Flags:D3D12_COMMAND_QUEUE_FLAG_NONE,NodeMask:0,
 };
 let queue:ID3D12CommandQueue=unsafe{dev.CreateCommandQueue(&queue_desc)}
  .map_err(|e|format!("D3D12 hardware queue unavailable: {e}"))?;
 let mut opt=D3D12_FEATURE_DATA_D3D12_OPTIONS1::default();
 let has_wave=unsafe{dev.CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS1,
    (&mut opt as *mut D3D12_FEATURE_DATA_D3D12_OPTIONS1).cast::<c_void>(),
    std::mem::size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS1>() as u32)}.is_ok();
 // WaveOps feature capability is not proof of native Wave64 shader dispatch.
 let wave_ops=has_wave.then_some(opt.WaveOps.as_bool());
 let (wave_min,wave_max)=if wave_ops==Some(true){
  (Some(opt.WaveLaneCountMin),Some(opt.WaveLaneCountMax))
 }else{(None,None)};
 let raw=queue.cast::<windows::core::IUnknown>().map_err(|e|e.to_string())?;
 let mut wrapped:Option<ID3D11Device>=None;
 let mut context:Option<ID3D11DeviceContext>=None;
 unsafe{D3D11On12CreateDevice(&dev,D3D11_CREATE_DEVICE_BGRA_SUPPORT.0,
  Some(&[D3D_FEATURE_LEVEL_11_0]),Some(&[Some(raw)]),0,
  Some(&mut wrapped),Some(&mut context),None)}
  .map_err(|e|format!("D3D11On12 cannot create an interop GPU device: {e}"))?;
 let gpu=crate::photo_lab::Gpu{device:wrapped.ok_or("Interop D3D11 device missing")?,
    ctx:context.ok_or("Interop D3D11 context missing")?};
 const W:u32=47;const H:u32=31;
 let original=image::RgbaImage::from_fn(W,H,|x,y|{
   image::Rgba([((x*7)%110+110)as u8,((y*11)%105+35)as u8,((x+y)%88+30)as u8,255])
 });
 let source=gpu.upload(&original)?;
 let pipeline=crate::native_pipeline::NativeGpuPipeline::new(&gpu.device,W,H)?;
 let result=pipeline.dispatch(&gpu.device,&gpu.ctx,&source,&[7])?;
 unsafe{gpu.ctx.Flush();}
 let pixels=gpu.read(&result)?;
 let changed=pixels.pixels().zip(original.pixels()).filter(|(a,b)|a!=b).count();
 if changed<100{return Err(format!("D3D12 queue computed no actual filtered GPU pixels (changed {changed})"));}
 if pixels.pixels().any(|p|p[3]!=255){return Err("D3D12 GPU output alpha channel corrupted".into());}
 Ok(Proof{gpu_compute_on_d3d12_queue:true,
    wave_ops_advertised:wave_ops,wave_lane_min:wave_min,wave_lane_max:wave_max,
    output_pixels_changed:changed})
}
#[derive(Default)]
struct Status{running:bool,result:Option<String>}
static STATE:OnceLock<Arc<Mutex<Status>>>=OnceLock::new();
fn state()->&'static Arc<Mutex<Status>>{STATE.get_or_init(||Arc::new(Mutex::new(Status::default())))}
pub fn start()->Result<(),String>{
 let state=state().clone();
 {
  let mut status=state.lock().map_err(|_|"D3D12 proof state unavailable")?;
  if status.running{return Err("D3D12 proof already running".into());}
  status.running=true;status.result=None;
 }
 let worker_state=state.clone();
 let worker=thread::Builder::new().name("Horizon D3D12 own hardware diagnostic".into()).spawn(move||{
  let message=match synthetic_d3d12_compatibility_proof(){
   Ok(report)=>format!("D3D12 DIRECT queue + D3D11On12 native GPU filter: {} altered pixels. WaveOps {:?}, lane range {:?}-{:?}. Compatibility proof only: NOT a Vulkan/D3D12 runtime backend, async compute, or native Wave64 shader.",
    report.output_pixels_changed,report.wave_ops_advertised,report.wave_lane_min,report.wave_lane_max),
   Err(e)=>format!("D3D12 compatibility diagnostic unavailable: {e}"),
  };
  if let Ok(mut g)=worker_state.lock(){g.running=false;g.result=Some(message);}
 });
 if let Err(e)=worker{
  if let Ok(mut g)=state.lock(){g.running=false;}
  return Err(format!("Could not launch D3D12 proof worker: {e}"));
 }
 Ok(())
}
pub fn snapshot()->(bool,Option<String>){
 state().lock().map(|v|(v.running,v.result.clone())).unwrap_or((false,None))
}
#[cfg(test)]mod tests{
 #[test]fn real_d3d12_hardware_queue_executes_compatible_own_gpu_shader(){
  let p=super::synthetic_d3d12_compatibility_proof().expect("RX 580 DX12/D3D11On12 test must not silently pretend success");
  assert!(p.gpu_compute_on_d3d12_queue&&p.output_pixels_changed>100);
 }
}
